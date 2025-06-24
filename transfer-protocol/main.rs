0use std::path::PathBuf;
use clap::{Arg, Command, Subcommand, Parser};
use tokio;
use rpassword::read_password;

mod ktp_controller;
mod gitfetcher; 

#[derive(Subcommand)]
enum Protocol {
    /// Transfer kernel via SCP/SSH
    Scp {
        #[arg(long, required = true)]
        source: String,
        #[arg(long, required = true)]
        dest: PathBuf,
        #[arg(long)]
        username: Option<String>,
    },
    /// Transfer kernel via HTTP/HTTPS
    Http {
        #[arg(long, required = true)]
        source: String,
        #[arg(long, required = true)]
        dest: PathBuf,
    },
    /// Transfer kernel via FTP
    Ftp {
        #[arg(long, required = true)]
        source: String,
        #[arg(long, required = true)]
        dest: PathBuf,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    /// Run KTP.mk installation script
    Mktp {
        #[arg(long, required = true)]
        dest: PathBuf,
    },
    /// Run kernel config interface (make menuconfig)
    Kconfig {
        #[arg(long, required = true)]
        dest: PathBuf,
    },
    /// Git fetcher tool
    Git {
        #[arg(long)]
        source: String,

        #[arg(long)]
        local_path: PathBuf,

        #[arg(long, default_value = "Automatic commit")]
        commit_message: String,

        #[arg(long, default_value = "GitFetcher")]
        author_name: String,

        #[arg(long, default_value = "gitfetcher@example.com")]
        author_email: String,

        #[arg(long, default_value = "origin")]
        remote_name: String,

        #[arg(long, default_value = "master")]
        branch: String,

        #[arg(long)]
        push: bool,
    },

#[derive(Parser)]
#[command(name = "KTP Controller")]
#[command(author = "Zaman Huseyinli")]
#[command(version = "0.1.0")]
#[command(about = "Kernel Transfer Protocol Controller with secure CLI")]
struct Cli {
    #[command(subcommand)]
    protocol: Protocol,

    /// Automatically compile kernel after transfer/configuration
    #[arg(long, default_value_t = true)]
    auto_compile: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let ktp = ktp_controller::KtpController::new();
    let gitfetcher = gitfetcher::GitFetcher::new();

    match cli.protocol {
        Protocol::Scp { source, dest, username } => {
            ktp.transfer_kernel(ktp_controller::TransferOptions {
                protocol: ktp_controller::TransferProtocol::SSH,
                source_url: source,
                destination_path: dest,
                auto_compile: cli.auto_compile,
                username,
                password: None,
            }).await?;
        }
        Protocol::Http { source, dest } => {
            ktp.transfer_kernel(ktp_controller::TransferOptions {
                protocol: ktp_controller::TransferProtocol::HTTP,
                source_url: source,
                destination_path: dest,
                auto_compile: cli.auto_compile,
                username: None,
                password: None,
            }).await?;
        }
        Protocol::Ftp { source, dest, username, mut password } => {
            if password.is_none() {
                println!("Enter FTP password (input hidden): ");
                password = Some(read_password()?);
            }
            ktp.transfer_kernel(ktp_controller::TransferOptions {
                protocol: ktp_controller::TransferProtocol::FTP,
                source_url: source,
                destination_path: dest,
                auto_compile: cli.auto_compile,
                username,
                password,
            }).await?;
        }
        Protocol::Mktp { dest } => {
            ktp.run_ktp_mk(&dest).await?;
        }
        Protocol::Kconfig { dest } => {
            ktp.kconfig_interface(&dest).await?;
            if cli.auto_compile {
                ktp.compile_kernel(&dest).await?;
            }
        }
        Protocol::Git {
            source,
            local_path,
            commit_message,
            author_name,
            author_email,
            remote_name,
            branch,
            push,
        } => {
            gitfetcher::run_git_operations(
                &source,
                &local_path,
                &commit_message,
                &author_name,
                &author_email,
                &remote_name,
                &branch,
                push,
            )
            .await?;
        }
     }

    Ok(())
}
