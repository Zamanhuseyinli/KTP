use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::fs;
use async_ftp::FtpStream;
use ssh2::Session;
use std::net::TcpStream;
use std::time::Duration;
use git2::{Repository, IndexAddOption, Cred, RemoteCallbacks, PushOptions, FetchOptions, AutotagOption};
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum EntryType {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub url: String,
    pub entry_type: EntryType,
}

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub commit_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct PatchEntry {
    pub patch_name: String,
    pub url: String,
    pub diff_content: Option<String>,
}

pub struct GitFetcher {
    client: Client,
}

impl GitFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("ktp-agent/1.0")
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    pub async fn is_valid_git_source(&self, source: &str) -> Result<bool> {
        if source.starts_with("http://") || source.starts_with("https://") {
            Ok(self.is_git_http_url(source))
        } else if source.starts_with("ftp://") {
            Ok(self.is_git_ftp_url(source))
        } else if Self::is_scp_like(source) {
            Ok(self.is_git_scp_url(source))
        } else {
            Ok(self.is_local_git_dir(source).await)
        }
    }

    fn is_git_http_url(&self, url: &str) -> bool {
        url.ends_with(".git") || url.contains(".git/")
    }

    fn is_git_ftp_url(&self, url: &str) -> bool {
        url.contains(".git") || url.contains(".gitlang")
    }

    fn is_git_scp_url(&self, url: &str) -> bool {
        url.contains(".git")
    }

    fn is_scp_like(url: &str) -> bool {
        url.contains(':') && !url.contains("://")
    }

    async fn is_local_git_dir(&self, path_str: &str) -> bool {
        let path = Path::new(path_str);
        if path.is_dir() {
            let git_dir = path.join(".git");
            fs::metadata(git_dir).await.is_ok()
        } else {
            false
        }
    }

    pub async fn fetch_http_url(&self, url: &str) -> Result<String> {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            anyhow::bail!("fetch_http_url only supports HTTP/HTTPS URLs");
        }
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("HTTP error: {}", resp.status());
        }
        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn fetch_ftp_listing(&self, url: &str) -> Result<Vec<String>> {
        let url_parsed = url.parse::<url::Url>()?;
        let host = url_parsed.host_str().ok_or_else(|| anyhow::anyhow!("No host in FTP URL"))?;
        let port = url_parsed.port().unwrap_or(21);
        let username = url_parsed.username();
        let password = url_parsed.password().unwrap_or("");

        let mut ftp_stream = FtpStream::connect((host, port)).await?;
        if !username.is_empty() {
            ftp_stream.login(username, password).await?;
        } else {
            ftp_stream.login("anonymous", "anonymous").await?;
        }

        let path = url_parsed.path();
        ftp_stream.cwd(path).await?;

        let list = ftp_stream.list(None).await?;
        ftp_stream.quit().await?;
        Ok(list)
    }





pub fn fetch_scp_listing(&self, scp_url: &str, username: &str, password: &str) -> Result<Vec<String>> {
    let parts: Vec<&str> = scp_url.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid SCP URL format");
    }

    let host_part = parts[0];
    let path = parts[1];
    let user_host: Vec<&str> = host_part.split('@').collect();
    let (user, host) = if user_host.len() == 2 {
        (user_host[0], user_host[1])
    } else {
        (username, host_part)
    };

    let tcp = TcpStream::connect(format!("{}:22", host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(user, password)?;
    if !sess.authenticated() {
        anyhow::bail!("SCP Authentication failed");
    }

    let sftp = sess.sftp()?;
    let mut handle = sftp.opendir(Path::new(path))?;

    let mut filenames = Vec::new();
    // 🔥 Tek tek readdir() çağrısı ile listeyi oku
    loop {
        match handle.readdir() {
            Ok((pathbuf, _stat)) => {
                if let Some(name) = pathbuf.file_name() {
                    filenames.push(name.to_string_lossy().into_owned());
                }
            }
            Err(ref e) if e.code() == ssh2::ErrorCode::Session(-37) => {
                // -37 = LIBSSH2_ERROR_SOCKET_RECV → tüm dosyalar okundu
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(filenames)
}
    pub async fn fetch_local_git_entries(&self, path_str: &str) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        let path = Path::new(path_str);
        let mut dir = fs::read_dir(path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name().to_string_lossy().to_string();
            entries.push(file_name);
        }
        Ok(entries)
    }

    // Git repository işlemleri için:

    /// Repo aç veya klonla
    pub fn open_or_clone_repo(url: &str, local_path: &Path) -> Result<Repository> {
        if local_path.exists() && local_path.join(".git").exists() {
            Ok(Repository::open(local_path)?)
        } else {
            Ok(Repository::clone(url, local_path)?)
        }
    }

    /// Fetch işlemi
    pub fn fetch_repo(repo: &Repository, remote_name: &str) -> Result<()> {
        let mut remote = repo.find_remote(remote_name)?;
        let mut cb = RemoteCallbacks::new();
        cb.credentials(|_url, username_from_url, _| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        fo.download_tags(AutotagOption::All);
        remote.fetch(&[] as &[&str], Some(&mut fo), None)?;
        Ok(())
    }

    /// Git add (tüm değişiklikleri indexe ekler)
    pub fn git_add_all(repo: &Repository) -> Result<()> {
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Commit oluştur
    pub fn git_commit(repo: &Repository, message: &str, _author_name: &str, _author_email: &str) -> Result<()> {
        let sig = repo.signature()?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head();
        let parents = if let Ok(head) = head {
            vec![head.peel_to_commit()?]
        } else {
            Vec::new()
        };
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents.iter().collect::<Vec<_>>())?;
        Ok(())
    }

    /// Push işlemi
    pub fn git_push(repo: &Repository, remote_name: &str, branch: &str) -> Result<()> {
        let mut remote = repo.find_remote(remote_name)?;
        let mut cb = RemoteCallbacks::new();
        cb.credentials(|_url, username_from_url, _| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(cb);
        remote.push(&[&format!("refs/heads/{}", branch)], Some(&mut push_opts))?;
        Ok(())
    }

    pub async fn run_git_operations(
        &self,
        source: &str,
        local_path: &PathBuf,
        commit_msg: &str,
        author: &str,
        email: &str,
        remote: &str,
        branch: &str,
        do_push: bool,
    ) -> Result<()> {
        let repo = Self::open_or_clone_repo(source, local_path)?;
        Self::fetch_repo(&repo, remote)?;
        Self::git_add_all(&repo)?;
        Self::git_commit(&repo, commit_msg, author, email)?;
        if do_push {
            Self::git_push(&repo, remote, branch)?;
        }
        Ok(())
    }
}
