## KTP - Kernel Transfer Protocol

The **KTP** (Kernel Transfer Protocol) is a **Rust** application that helps manage kernel file transfers and Git repository operations. It supports multiple transfer protocols and integrates kernel configuration tools.

### Features

1. **Kernel File Transfer**:
    - **SCP** (via SSH): Transfer kernel files securely using SCP.
    - **HTTP/HTTPS**: Transfer kernel files via HTTP/HTTPS protocols.
    - **FTP**: Transfer files using FTP, with optional username and password.
  
2. **Git Fetcher**:
    - Sync Git repositories by cloning/fetching from remote repositories.
    - Supports commit messages, author details, and pushing changes.

3. **Kernel Configuration**:
    - Run **KTP.mk installation script** to set up the kernel environment.
    - **Make Menuconfig**: Run kernel configuration interface (`make menuconfig`), with an option to auto-compile after configuration.

4. **Real-time Kernel Compilation**:
    - After transferring kernel files or configuring, you can trigger **automatic compilation**.

### Protocols Supported
- **SCP** (SSH) for secure file transfer.
- **FTP** for file transfer with optional credentials.
- **HTTP/HTTPS** for kernel source file download.
- **Git Fetcher** for managing Git repositories.

### Example Command Usage

```bash
# Example SCP command for transferring kernel
ktp transfer --protocol=scp --source=path/to/source --dest=/path/to/destination --username=your_username

# Example HTTP transfer command
ktp transfer --protocol=http --source=http://example.com/kernel --dest=/path/to/destination

# Example Git command to fetch from remote repo
ktp git --source=https://github.com/user/repo.git --local_path=/local/repo --push=true
```

### Workflow

1. **CLI Commands**: Specify the transfer protocol (SCP, HTTP, FTP, Git).
2. **Kernel or Repo Transfer**: Files are transferred based on the protocol, and Git operations can be executed.
3. **Post-transfer Configuration**: After transferring, the kernel configuration interface is launched (`make menuconfig`).
4. **Automatic Compilation**: If enabled, the kernel is compiled after configuration or transfer.

---

## RepoWatcher & AIAnalyzer Integration (Python)

This **Python** application integrates **RepoWatcher** and **AIAnalyzer** to monitor repositories and analyze them in real-time or offline. It supports FTP, SCP, HTTP, and local directory monitoring.

### Features

1. **RepoWatcher**:
    - Monitors a repository or directory for changes.
    - Supports **SCP**, **FTP**, **HTTP/HTTPS**, and **local directories**.
    - Provides **livestream** or **offline** streaming for repository changes.

2. **AIAnalyzer**:
    - Performs AI-based analysis on repository contents.
    - Supports **livestream analysis** for HTTP/HTTPS repositories.
    - Can also analyze **local directories** in **offline mode**.

3. **User Interaction**:
    - Commands like `start` trigger AI analysis.
    - `exit` stops the program.

### Example Command Usage

```bash
# Run Python script for RepoWatcher and AIAnalyzer
python3 main.py --uri http://example.com/repo --stream-type livestream
```


### Workflow

1. **Monitor Repository**: The **RepoWatcher** monitors a repository URL or local directory.
2. **AI Analysis**: Based on the stream type:
    - **Offline**: Analyzes the local directory.
    - **Livestream**: Analyzes data streamed from HTTP/HTTPS repositories.
3. **User Commands**: Users can type `start` to begin AI analysis or `exit` to quit.

4. **Using Models**:

There are two modes: `single` and `multiple`.

- **Single**: If you use `single`, you can monitor only one GIT URL. Tools like AI analyzers will be able to analyze only that one URL.
  
- **Multiple**: If you use `multiple`, you can monitor multiple URLs and allow tools like AI analyzers to analyze several GIT repositories at once.

For example, using the command below:

```
Aipropengine-uri https://github.com/zamanhuseinli/ktp --stream-type offline --stream-mode multiple
```

This will save the data under the `gitroot_multi` directory. In this way, AI analyzer tools will be able to analyze all the repositories within the `gitroot_multi` directory.
```

Let me know if you need further adjustments!

---

## Integration of KTP and RepoWatcher + AIAnalyzer

Both the **Rust-based KTP** and **Python-based RepoWatcher/AIAnalyzer** can interact in the following ways:

### **Workflow Integration**

1. **Kernel Transfer with KTP**:
    - Transfer kernel files using **KTP** via SCP, FTP, or HTTP.
    - Once the transfer is completed, you can trigger **RepoWatcher** to monitor the transferred repository (if it’s part of a kernel repo or project).

2. **Repo Monitoring with RepoWatcher**:
    - The **Python app** (`main.py`) can monitor the repository for any changes made by the **Rust KTP tool**.
    - Once new changes are detected (e.g., after a kernel transfer), it triggers AI analysis via the **AIAnalyzer**.

### Example Interaction:

1. **Step 1**: Use **Rust KTP** to transfer kernel files via SCP or HTTP.

    ```bash
    ktp transfer --protocol=scp --source=path/to/source --dest=/path/to/destination --username=your_username
    ```

2. **Step 2**: After the transfer, run the **Python RepoWatcher** to monitor the repository.

    ```bash
    python3 main.py --uri scp://remotehost/repo --stream-type livestream
    ```

3. **Step 3**: Once the **RepoWatcher** detects new changes, it triggers **AIAnalyzer** to perform analysis on the newly transferred kernel files or project changes.

    ```bash
    # AI analysis triggered automatically
    ```

---

## Future Enhancements

1. **Unified CLI**:
   - Combine the **Rust KTP CLI** and **Python RepoWatcher CLI** into a single interface to streamline the workflow from kernel transfer to repository monitoring and AI analysis.

2. **Cloud Integration**:
   - Allow the **Python RepoWatcher** to monitor repositories stored in cloud storage (e.g., GitHub, GitLab) and trigger **AIAnalyzer** for cloud-based analysis.

3. **Extended Protocols**:
   - Add more protocols (e.g., **SFTP**) for enhanced file transfer capabilities.

4. **Automatic Continuous Integration (CI)**:
   - Automatically run **AI analysis** after each file transfer or Git commit as part of a CI pipeline.

---

## Installation

### **KTP (Rust)**

1. Clone the repository:

    ```bash
    git clone https://github.com/yourusername/ktp.git
    cd ktp
    cargo build --release
    ```

2. Install dependencies and run:

    ```bash
    cargo run -- --protocol=scp --source=path/to/source --dest=/path/to/destination
    ```

### **RepoWatcher & AIAnalyzer (Python)**

1. Install dependencies:

    ```bash
    pip install -r requirements.txt
    ```

2. Run the Python application:

    ```bash
    python3 main.py --uri http://example.com/repo --stream-type livestream
    ```

---

## Contribution

To contribute to either the **KTP** or **RepoWatcher/AIAnalyzer**, please follow these steps:

1. Fork the repository.
2. Create a new branch.
3. Make your changes and submit a pull request.

---

## License

This project is licensed under the **GPLV2 License**. See the [LICENSE](./LICENSE) file for details.
```
