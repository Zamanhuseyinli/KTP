use tokio::sync::Mutex;
use tokio::fs::{self, File};
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncWriteExt, BufReader, AsyncReadExt};
use futures_util::StreamExt;
use std::error::Error;
use std::sync::Arc;
use async_ftp::FtpError;

pub enum TransferProtocol {
    SSH,
    HTTP,
    FTP,
    Cloud,
}

pub struct TransferOptions {
    pub protocol: TransferProtocol,
    pub source_url: String,
    pub destination_path: PathBuf,
    pub auto_compile: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct KtpController;

impl KtpController {
    pub fn new() -> Self {
        Self
    }

    pub async fn transfer_kernel(&self, opts: TransferOptions) -> Result<(), Box<dyn Error>> {
        self.validate_url(&opts.protocol, &opts.source_url)?;

        match opts.protocol {
            TransferProtocol::SSH | TransferProtocol::Cloud => {
                self.transfer_scp(&opts.source_url, &opts.destination_path, opts.username.clone()).await?;
            }
            TransferProtocol::HTTP => {
                self.transfer_http(&opts.source_url, &opts.destination_path).await?;
            }
            TransferProtocol::FTP => {
                self.transfer_ftp(&opts.source_url, &opts.destination_path, opts.username.clone(), opts.password.clone()).await?;
            }
        }

        if self.ktp_mk_exists(&opts.destination_path).await? {
            println!("KTP.mk detected. Starting automatic installation...");
            self.run_ktp_mk(&opts.destination_path).await?;
        } else if opts.auto_compile {
            self.clean_kernel(&opts.destination_path).await?;
            self.kconfig_interface(&opts.destination_path).await?;
            self.compile_kernel(&opts.destination_path).await?;
        }

        Ok(())
    }

    fn validate_url(&self, protocol: &TransferProtocol, url: &str) -> Result<(), Box<dyn Error>> {
        match protocol {
            TransferProtocol::HTTP => {
                if !url.starts_with("https://") {
                    println!("WARNING: Using a non-HTTPS URL may compromise security.");
                }
            }
            TransferProtocol::SSH | TransferProtocol::Cloud => {
                if !url.contains('@') || !url.contains(':') {
                    return Err("Invalid SSH/Cloud URL format; expected user@host:/path".into());
                }
            }
            TransferProtocol::FTP => {
                if !url.starts_with("ftp://") {
                    println!("WARNING: FTP URL should start with 'ftp://'");
                }
                println!("WARNING: FTP protocol is insecure; proceed with caution.");
            }
        }
        Ok(())
    }

    async fn ktp_mk_exists(&self, kernel_path: &PathBuf) -> Result<bool, Box<dyn Error>> {
        let ktp_mk_path = kernel_path.join("KTP.mk");
        Ok(tokio::fs::metadata(&ktp_mk_path).await.is_ok())
    }

    pub async fn run_ktp_mk(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let ktp_mk_path = kernel_path.join("KTP.mk");
        println!("Executing KTP.mk at: {:?}", ktp_mk_path);

        let status = TokioCommand::new("make")
            .arg("-f")
            .arg(&ktp_mk_path)
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            return Err("Installation via KTP.mk failed".into());
        }

        println!("KTP.mk installation finished successfully.");
        Ok(())
    }

    async fn transfer_scp(&self, source_url: &str, dest: &PathBuf, username: Option<String>) -> Result<(), Box<dyn Error>> {
        println!("Starting SCP transfer from '{}' to '{:?}'", source_url, dest);

        let final_url = if let Some(user) = username {
            if let Some(at_pos) = source_url.find('@') {
                let after_at = &source_url[at_pos + 1..];
                format!("{}@{}", user, after_at)
            } else {
                format!("{}@{}", user, source_url)
            }
        } else {
            source_url.to_string()
        };

        let status = TokioCommand::new("scp")
            .arg("-r")
            .arg(final_url)
            .arg(dest)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            return Err("SCP transfer failed".into());
        }

        println!("SCP transfer completed successfully.");
        Ok(())
    }

    async fn transfer_http(&self, url: &str, dest: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Starting HTTP download from '{}' to '{:?}'", url, dest);

        let filename = url.split('/').last().ok_or("Failed to extract filename from URL")?;
        let file_path = dest.join(filename);

        if !dest.exists() {
            fs::create_dir_all(dest).await?;
        }

        let client = reqwest::Client::new();
        let resp = client.get(url).send().await?;

        if !resp.status().is_success() {
            return Err(format!("HTTP request failed with status: {}", resp.status()).into());
        }

        let mut file = File::create(&file_path).await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        println!("File downloaded successfully to {:?}", file_path);

        Ok(())
    }

    async fn transfer_ftp(
        &self,
        url: &str,
        dest: &PathBuf,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        println!("Starting FTP transfer from '{}' to '{:?}'", url, dest);

        let parsed_url = url::Url::parse(url)?;
        let host = parsed_url.host_str().ok_or("FTP host not found")?;
        let port = parsed_url.port_or_known_default().unwrap_or(21);
        let remote_path = parsed_url.path().trim_start_matches('/');

        let user = username.unwrap_or_else(|| "anonymous".to_string());
        let pass = password.unwrap_or_else(|| "anonymous".to_string());

        let mut ftp_stream = async_ftp::FtpStream::connect((host, port)).await?;
        ftp_stream.login(&user, &pass).await?;

        let filename = std::path::Path::new(remote_path)
            .file_name()
            .ok_or("Failed to get file name from FTP path")?;

        if !dest.exists() {
            fs::create_dir_all(dest).await?;
        }

        let file_path = dest.join(filename);
        let file = File::create(&file_path).await?;
        let file = Arc::new(Mutex::new(file));

        ftp_stream
            .retr(remote_path, |reader: BufReader<async_ftp::DataStream>| {
                let file = Arc::clone(&file);
                async move {
                    let mut reader = reader;
                    let mut file = file.lock().await;
                    let mut buf = [0u8; 8192];

                    loop {
                        let n = reader
                            .read(&mut buf)
                            .await
                            .map_err(|e| FtpError::ConnectionError(e))?;
                        if n == 0 {
                            break;
                        }
                        file.write_all(&buf[..n])
                            .await
                            .map_err(|e| FtpError::ConnectionError(e))?;
                    }
                    Ok::<_, FtpError>(())
                }
            })
            .await?;

        println!("FTP file downloaded successfully to {:?}", file_path);

        Ok(())
    }

    async fn clean_kernel(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Running 'make clean' in {:?}", kernel_path);

        let status = TokioCommand::new("make")
            .arg("clean")
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            println!("Warning: 'make clean' failed, continuing...");
        } else {
            println!("'make clean' completed successfully.");
        }

        Ok(())
    }

    pub async fn kconfig_interface(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Launching 'make menuconfig' in {:?}", kernel_path);

        let status = TokioCommand::new("make")
            .arg("menuconfig")
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            println!("Warning: 'make menuconfig' was cancelled or failed.");
        } else {
            println!("Kconfig configuration completed.");
        }

        Ok(())
    }

    pub async fn compile_kernel(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Starting kernel compilation in {:?}", kernel_path);

        let status = TokioCommand::new("make")
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            return Err("Kernel compilation failed".into());
        }

        println!("Kernel compilation finished successfully.");
        Ok(())
    }
}
