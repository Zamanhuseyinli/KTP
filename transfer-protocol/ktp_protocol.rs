use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioCommand;
use futures_util::StreamExt;
use std::error::Error;

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
    pub username: Option<String>,  // For FTP and SSH/SCP authentication
    pub password: Option<String>,  // For FTP authentication
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
            println!("KTP.mk found, starting automatic installation...");
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
                    println!("WARNING: Non-HTTPS URL is being used. This may cause security risks.");
                }
            }
            TransferProtocol::SSH | TransferProtocol::Cloud => {
                if !url.contains('@') || !url.contains(':') {
                    return Err("SSH/Cloud URL format invalid, expected user@host:/path".into());
                }
            }
            TransferProtocol::FTP => {
                if !url.starts_with("ftp://") {
                    println!("WARNING: FTP URL should start with ftp://");
                }
                println!("WARNING: FTP protocol is insecure. User assumes responsibility.");
            }
        }
        Ok(())
    }

    async fn ktp_mk_exists(&self, kernel_path: &PathBuf) -> Result<bool, Box<dyn Error>> {
        let ktp_mk_path = kernel_path.join("KTP.mk");
        Ok(ktp_mk_path.exists())
    }

    async fn run_ktp_mk(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let ktp_mk_path = kernel_path.join("KTP.mk");
        println!("Running KTP.mk file: {:?}", ktp_mk_path);

        let status = TokioCommand::new("make")
            .arg("-f")
            .arg(ktp_mk_path)
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            return Err("Installation via KTP.mk failed".into());
        }

        println!("KTP.mk installation completed successfully.");
        Ok(())
    }

    async fn transfer_scp(&self, source_url: &str, dest: &PathBuf, username: Option<String>) -> Result<(), Box<dyn Error>> {
        println!("Starting SCP transfer: {} -> {:?}", source_url, dest);

        // If username given, replace user in source_url with username (optional)
        // Otherwise assume source_url has user@host format.
        let final_url = if let Some(user) = username {
            // Attempt to replace username part in source_url (simple heuristic)
            if let Some(at_pos) = source_url.find('@') {
                let after_at = &source_url[at_pos+1..];
                format!("{}@{}", user, after_at)
            } else {
                // no user in source_url, prepend username@
                format!("{}@{}", user, source_url)
            }
        } else {
            source_url.to_string()
        };

        let status = TokioCommand::new("scp")
            .arg("-r")
            .arg(final_url)
            .arg(dest)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            return Err("SCP transfer failed".into());
        }

        println!("SCP transfer completed");
        Ok(())
    }

    async fn transfer_http(&self, url: &str, dest: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Starting HTTP transfer: {} -> {:?}", url, dest);

        let filename = url.split('/').last().ok_or("Failed to extract filename from URL")?;
        let file_path = dest.join(filename);

        if !dest.exists() {
            fs::create_dir_all(dest).await?;
        }

        let client = reqwest::Client::new();
        let resp = client.get(url).send().await?;

        if !resp.status().is_success() {
            return Err(format!("HTTP request failed: {}", resp.status()).into());
        }

        let mut file = fs::File::create(&file_path).await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        println!("File downloaded: {:?}", file_path);

        Ok(())
    }

    async fn transfer_ftp(&self, url: &str, dest: &PathBuf, username: Option<String>, password: Option<String>) -> Result<(), Box<dyn Error>> {
        println!("Starting FTP transfer: {} -> {:?}", url, dest);

        let parsed_url = url::Url::parse(url)?;
        let host = parsed_url.host_str().ok_or("FTP host not found")?;
        let port = parsed_url.port_or_known_default().unwrap_or(21);
        let path = parsed_url.path();

        let mut ftp_stream = ftp::FtpStream::connect((host, port)).await?;

        let user = username.unwrap_or_else(|| "anonymous".to_string());
        let pass = password.unwrap_or_else(|| "anonymous".to_string());

        ftp_stream.login(&user, &pass).await?;

        let remote_file = path.trim_start_matches('/');
        let bytes = ftp_stream.retr(remote_file).await?;
        ftp_stream.quit().await?;

        if !dest.exists() {
            fs::create_dir_all(dest).await?;
        }

        let filename = Path::new(remote_file)
            .file_name()
            .ok_or("Failed to get file name")?;

        let file_path = dest.join(filename);
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&bytes).await?;

        println!("FTP file downloaded: {:?}", file_path);

        Ok(())
    }

    async fn clean_kernel(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Starting kernel cleaning: {:?}", kernel_path);

        let status = TokioCommand::new("make")
            .arg("clean")
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            println!("make clean may have failed, continuing anyway.");
        } else {
            println!("make clean completed.");
        }

        Ok(())
    }

    async fn kconfig_interface(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Starting Kconfig interface (make menuconfig) in {:?}", kernel_path);

        let status = TokioCommand::new("make")
            .arg("menuconfig")
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            println!("Warning: Kconfig menuconfig exited with error or was cancelled.");
        } else {
            println!("Kconfig configuration completed.");
        }

        Ok(())
    }

    async fn compile_kernel(&self, kernel_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Starting kernel compilation: {:?}", kernel_path);

        let status = TokioCommand::new("make")
            .current_dir(kernel_path)
            .status()
            .await?;

        if !status.success() {
            return Err("Kernel compilation failed".into());
        }

        println!("Kernel compilation completed successfully");
        Ok(())
    }
}
