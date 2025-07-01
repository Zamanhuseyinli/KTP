# KTP - Kernel Transfer Protocol

**KTP** (Kernel Transfer Protocol) is a **Rust** application designed to manage kernel file transfers and Git repository operations. It supports multiple transfer protocols and integrates kernel configuration and compilation tools.

---

## Features

### 1. Kernel File Transfer
- **SCP (via SSH)**: Securely transfer kernel files using SCP.
- **HTTP/HTTPS**: Download kernel files over HTTP or HTTPS.
- **FTP**: Transfer files using FTP with optional username and password.

### 2. Git Fetcher
- Clone, fetch, and sync Git repositories.
- Supports commit messages, author info, and pushing changes.

### 3. Kernel Configuration
- Run the `KTP.mk` installation script to set up the kernel environment.
- Launch `make menuconfig` kernel configuration interface.
- Optionally auto-compile the kernel after configuration.

### 4. Real-time Kernel Compilation
- Automatically trigger kernel compilation after file transfer or configuration.

---

## Supported Protocols

- **SCP** (SSH-based secure file transfer)
- **FTP** (with optional credentials)
- **HTTP/HTTPS**
- **Git Fetcher** (Git operations support)

---

## Example Commands

```bash
# Transfer kernel files via SCP
ktp scp --source=path/to/source --dest=/path/to/destination --username=your_username

# Transfer kernel files via HTTP
ktp http --source=http://example.com/kernel --dest=/path/to/destination

# Fetch from Git repo and optionally push changes
ktp git --source=https://github.com/user/repo.git --local-path=/local/repo --push=true
```

---

## Workflow

1. Specify the transfer protocol (SCP, HTTP, FTP, Git) via CLI.
2. Transfer kernel files or perform Git operations.
3. Launch kernel configuration interface (`make menuconfig`) post-transfer.
4. Trigger automatic kernel compilation if enabled.

---

# RepoWatcher & AIAnalyzer Integration (Python)

A **Python** application combining **RepoWatcher** and **AIAnalyzer** for monitoring repositories and analyzing them in real-time or offline. Supports FTP, SCP, HTTP, and local directory monitoring.

---

## Features

### RepoWatcher
- Monitors repositories or directories for changes.
- Supports **SCP**, **FTP**, **HTTP/HTTPS**, and **local directories**.
- Offers **livestream** or **offline** modes for repository changes.

### AIAnalyzer
- Performs AI-based analysis on repository contents.
- Supports **livestream** analysis for HTTP/HTTPS repositories.
- Can analyze **local directories** in **offline** mode.

### User Interaction
- Commands like `start` begin AI analysis.
- `exit` terminates the program.

---

## Example Commands

```bash
# Run RepoWatcher and AIAnalyzer for livestream HTTP repo
python3 main.py --uri http://example.com/repo --stream-type livestream
```
# example using [example using](https://raw.githubusercontent.com/Zamanhuseyinli/KTP/main/example.gif)
---

## Workflow

1. **Monitor repository or local directory** using RepoWatcher.
2. **Analyze repository content** based on stream type:
   - **Offline**: Local directory analysis.
   - **Livestream**: Live HTTP/HTTPS stream analysis.
3. **User commands** (`start`, `exit`) control analysis lifecycle.

---

## Modes

- **Single mode**: Monitor and analyze a single Git repository URL.
- **Multiple mode**: Monitor and analyze multiple Git repositories simultaneously.

Example for multiple mode:

```bash
aipropengine_ktp --uri https://github.com/Zamanhuseyinli/KTP --stream-type offlinestream --mode multiple
```

This saves repositories under `gitroot_multi` for collective AI analysis.

---

# Integration of KTP and RepoWatcher + AIAnalyzer

Both the **Rust-based KTP** and **Python-based RepoWatcher/AIAnalyzer** can be combined for a smooth kernel development workflow.

---

## Workflow Integration

1. **Kernel Transfer with KTP**
   - Transfer kernel files via SCP, FTP, or HTTP using KTP.
2. **Repo Monitoring with RepoWatcher**
   - Run Python RepoWatcher to monitor the transferred repo for changes.
3. **AI Analysis with AIAnalyzer**
   - Once RepoWatcher detects changes, AIAnalyzer performs analysis on the updated kernel or project files.

---

## Example Interaction

```bash
# Step 1: Transfer kernel files via KTP (SCP example)
ktp scp --source=path/to/source --dest=/path/to/destination --username=your_username

# Step 2: Monitor repo changes using RepoWatcher (livestream)
python3 main.py --uri scp://remotehost/repo --stream-type livestream

# Step 3: AI analysis triggered automatically upon detected changes
```

---

# Future Enhancements

- **Unified CLI**: Combine Rust KTP and Python RepoWatcher CLI for a streamlined user experience.
- **Cloud Integration**: Monitor cloud repositories (GitHub, GitLab) and trigger AI analysis.
- **Extended Protocol Support**: Add SFTP and other protocols.
- **Continuous Integration (CI)**: Automate AI analysis after each transfer or commit.

---

# Installation

### KTP (Rust)

```bash
git clone https://github.com/Zamanhuseyinli/ktp.git
cd ktp
cargo build --release

```

### RepoWatcher & AIAnalyzer (Python)

```bash
pip install -r requirements.txt
python setup.py install
aipropengine_ktp --help
```

---
# optional installation option
```bash
pip install -r requirements.txt
python Makemanifest
```

# Contribution

1. Fork the repository.  
2. Create a new branch.  
3. Make your changes.  
4. Submit a pull request.

---

# License

This project is licensed under the **GPLv2 License**. See the [LICENSE](./LICENSE) file for details.

