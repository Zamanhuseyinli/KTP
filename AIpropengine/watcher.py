import os
import asyncio
from pathlib import Path
from urllib.parse import urlparse
import aioftp
import paramiko
from git import Repo, GitCommandError

LOCAL_STORAGE = Path.home() / ".cache" / "aipropengine-ktp" 
SUPPORTED_PROTOCOLS = ["http", "https", "ftp", "scp", "localdir"]

def validate_uri(uri: str) -> bool:
    parsed = urlparse(uri)
    if parsed.scheme not in SUPPORTED_PROTOCOLS:
        return False
    if parsed.scheme == "localdir":
        p = Path(parsed.path)
        return p.exists() and p.is_dir()
    elif parsed.scheme in ["http", "https"]:
        return True
    elif parsed.scheme in ["ftp", "scp"]:
        return bool(parsed.hostname) and bool(parsed.path) and parsed.path != "/"
    else:
        return False

async def check_ftp_repo_files(host, port, user, password, path) -> bool:
    try:
        async with aioftp.Client.context(host, port=port, user=user, password=password) as client:
            entries = await client.list(path)
            filenames = [entry.name for entry in entries]
            return (".git" in filenames and ".gitlang" in filenames)
    except Exception as e:
        print(f"FTP file check error: {e}")
        return False

def check_scp_repo_files(hostname, port, username, password, path) -> bool:
    try:
        ssh = paramiko.SSHClient()
        ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        ssh.connect(hostname, port=port, username=username, password=password)
        sftp = ssh.open_sftp()
        file_list = sftp.listdir(path)
        sftp.close()
        ssh.close()
        return (".git" in file_list and ".gitlang" in file_list)
    except Exception as e:
        print(f"SCP file check error: {e}")
        return False

class RepoWatcher:
    def __init__(
        self,
        uri: str,
        stream_type: str,
        ftp_user: str = None,
        ftp_pass: str = None,
        scp_user: str = None,
        scp_pass: str = None,
        gitroot_mode: str = "single"  # "single" or "multiple"
    ):
        if stream_type not in ["livestream", "offlinestream"]:
            raise ValueError("stream_type must be 'livestream' or 'offlinestream'")
        if not validate_uri(uri):
            raise ValueError(f"URI '{uri}' is invalid or not supported")

        self.uri = uri
import os
import asyncio
from pathlib import Path
from urllib.parse import urlparse
import argparse

import aioftp
import paramiko
from git import Repo, GitCommandError

LOCAL_STORAGE = Path.home() / ".cache" / "aipropengine-ktp"
SUPPORTED_PROTOCOLS = ["http", "https", "ftp", "scp", "localdir"]

def set_gitroot(mode="single"):
    """
    mode: "single" veya "multiple"
    """
    base_dir = LOCAL_STORAGE
    if mode == "single":
        gitroot_path = base_dir / "gitroot_single"
        gitroot_path.mkdir(parents=True, exist_ok=True)
        os.environ["GITROOT"] = str(gitroot_path)
    elif mode == "multiple":
        gitroot1 = base_dir / "gitroot_multi1"
        gitroot2 = base_dir / "gitroot_multi2"
        gitroot1.mkdir(parents=True, exist_ok=True)
        gitroot2.mkdir(parents=True, exist_ok=True)
        os.environ["GITROOT"] = f"{gitroot1},{gitroot2}"
    else:
        raise ValueError("Mode must be 'single' or 'multiple'")

    print(f"[INFO] GITROOT environment variable set to: {os.environ['GITROOT']}")

def validate_uri(uri: str) -> bool:
    parsed = urlparse(uri)
    if parsed.scheme not in SUPPORTED_PROTOCOLS:
        return False
    if parsed.scheme == "localdir":
        p = Path(parsed.path)
        return p.exists() and p.is_dir()
    elif parsed.scheme in ["http", "https"]:
        return True
    elif parsed.scheme in ["ftp", "scp"]:
        return bool(parsed.hostname) and bool(parsed.path) and parsed.path != "/"
    else:
        return False

async def check_ftp_repo_files(host, port, user, password, path) -> bool:
    try:
        async with aioftp.Client.context(host, port=port, user=user, password=password) as client:
            entries = await client.list(path)
            filenames = [entry.name for entry in entries]
            return (".git" in filenames and ".gitlang" in filenames)
    except Exception as e:
        print(f"FTP file check error: {e}")
        return False

def check_scp_repo_files(hostname, port, username, password, path) -> bool:
    try:
        ssh = paramiko.SSHClient()
        ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        ssh.connect(hostname, port=port, username=username, password=password)
        sftp = ssh.open_sftp()
        file_list = sftp.listdir(path)
        sftp.close()
        ssh.close()
        return (".git" in file_list and ".gitlang" in file_list)
    except Exception as e:
        print(f"SCP file checking error  : {e}")
        return False

class RepoWatcher:
    def __init__(
        self, 
        uri: str, 
        stream_type: str, 
        ftp_user: str = None, ftp_pass: str = None, 
        scp_user: str = None, scp_pass: str = None
    ):
        if stream_type not in ["livestream", "offlinestream"]:
            raise ValueError("stream_type must be 'livestream' or 'offlinestream'")
        if not validate_uri(uri):
            raise ValueError(f"URI '{uri}' is invalid or not supported")
        self.uri = uri
        self.stream_type = stream_type

        gitroot_env = os.environ.get("GITROOT")
        if not gitroot_env:
            raise RuntimeError("GITROOT environment variable not set")

        if "," in gitroot_env and stream_type == "offlinestream":
            gitroots = gitroot_env.split(",")
            self.local_path = Path(gitroots[0]) / self._repo_name()
        else:
            self.local_path = Path(gitroot_env) / self._repo_name()

        Path(self.local_path.parent).mkdir(parents=True, exist_ok=True)

        self.ftp_user = ftp_user
        self.ftp_pass = ftp_pass
        self.scp_user = scp_user
        self.scp_pass = scp_pass

    def _repo_name(self) -> str:
        parsed = urlparse(self.uri)
        if parsed.scheme == "localdir":
            return Path(parsed.path).name
        else:
            return Path(parsed.path).stem

    async def watch(self):
        scheme = urlparse(self.uri).scheme
        if self.stream_type == "livestream":
            if scheme in ["http", "https"]:
                await self._watch_http_livestream()
            else:
                raise RuntimeError(f"Livestream not supported for scheme: {scheme}")
        else:
            if scheme in ["http", "https"]:
                await self._clone_or_pull_git()
            elif scheme == "ftp":
                parsed = urlparse(self.uri)
                host = parsed.hostname
                port = parsed.port or 21
                user = self.ftp_user or "anonymous"
                password = self.ftp_pass or ""
                path = parsed.path.rstrip("/")

                files_exist = await check_ftp_repo_files(host, port, user, password, path)
                if not files_exist:
                    raise RuntimeError(f"FTP repo missing required '.git' or '.gitlang' files at {self.uri}")

                await self._fetch_ftp_repo()
            elif scheme == "scp":
                parsed = urlparse(self.uri)
                hostname = parsed.hostname
                port = parsed.port or 22
                username = self.scp_user
                password = self.scp_pass
                remote_path = parsed.path.rstrip("/")

                if username is None:
                    raise RuntimeError("SCP username must be provided")

                files_exist = check_scp_repo_files(hostname, port, username, password, remote_path)
                if not files_exist:
                    raise RuntimeError(f"SCP repo missing required '.git' or '.gitlang' files at {self.uri}")

                await self._fetch_scp_repo()
            elif scheme == "localdir":
                await self._watch_local_dir()
            else:
                raise RuntimeError(f"Unsupported scheme for offlinestream: {scheme}")

    async def _watch_http_livestream(self):
        print(f"Starting HTTP livestream watcher for {self.uri}")
        import subprocess
        while True:
            result = subprocess.run(["git", "ls-remote", self.uri], capture_output=True, text=True)
            if result.returncode == 0:
                print(f"[Livestream] Remote refs for {self.uri}:\n{result.stdout}")
            else:
                print(f"[Livestream] Error checking remote: {result.stderr}")
            await asyncio.sleep(30)

    async def _clone_or_pull_git(self):
        print(f"Starting offline clone/pull for {self.uri}")
        if self.local_path.exists():
            try:
                repo = Repo(str(self.local_path))
                origin = repo.remotes.origin
                origin.fetch()
                repo.git.reset("--hard", "origin/master")
                print(f"Pulled latest changes in {self.local_path}")
            except GitCommandError as e:
                print(f"Git error during pull: {e}")
        else:
            try:
                print(f"Cloning {self.uri} into {self.local_path}")
                Repo.clone_from(self.uri, str(self.local_path))
            except GitCommandError as e:
                print(f"Git error during clone: {e}")

    async def _fetch_ftp_repo(self):
        print(f"Fetching FTP repo from {self.uri}")
        parsed = urlparse(self.uri)
        host = parsed.hostname
        port = parsed.port or 21
        user = self.ftp_user or "anonymous"
        password = self.ftp_pass or ""

        path = parsed.path.lstrip('/')
        dest_path = self.local_path
        dest_path.mkdir(parents=True, exist_ok=True)

        async with aioftp.Client.context(host, port=port, user=user, password=password) as client:
            print(f"Connected to FTP {host} as {user}")

            async def download_dir(remote_path, local_path):
                try:
                    entries = await client.list(remote_path)
                    for entry in entries:
                        name = entry.name
                        remote_item = f"{remote_path}/{name}"
                        local_item = local_path / name
                        if entry['type'] == 'file':
                            print(f"Downloading file {remote_item}")
                            await client.download(remote_item, str(local_item))
                        elif entry['type'] == 'dir':
                            local_item.mkdir(exist_ok=True)
                            await download_dir(remote_item, local_item)
                except Exception as e:
                    print(f"Error downloading FTP dir {remote_path}: {e}")

            await download_dir(path, dest_path)
        print(f"FTP repo fetched into {dest_path}")

    async def _fetch_scp_repo(self):
        print(f"Fetching SCP repo from {self.uri}")
        parsed = urlparse(self.uri)
        hostname = parsed.hostname
        port = parsed.port or 22
        username = self.scp_user
        password = self.scp_pass
        remote_path = parsed.path

        if username is None:
            raise RuntimeError("SCP username must be provided")

        dest_path = self.local_path
        dest_path.mkdir(parents=True, exist_ok=True)

        ssh = paramiko.SSHClient()
        ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        try:
            ssh.connect(hostname, port=port, username=username, password=password)
            sftp = ssh.open_sftp()

            import stat
            def recursive_scp_fetch(remote_dir, local_dir):
                os.makedirs(local_dir, exist_ok=True)
                for entry in sftp.listdir_attr(remote_dir):
                    remote_file = remote_dir + "/" + entry.filename
                    local_file = os.path.join(local_dir, entry.filename)
                    if stat.S_ISDIR(entry.st_mode):
                        recursive_scp_fetch(remote_file, local_file)
                    else:
                        print(f"Downloading SCP file {remote_file}")
                        sftp.get(remote_file, local_file)

            recursive_scp_fetch(remote_path, str(dest_path))
            sftp.close()
            ssh.close()
            print(f"SCP repo fetched into {dest_path}")
        except Exception as e:
            print(f"SCP fetch error: {e}")

    async def _watch_local_dir(self):
        print(f"Watching local directory {self.local_path}")
        last_snapshot = self._snapshot_dir(self.local_path)
        while True:
            await asyncio.sleep(60)
            current_snapshot = self._snapshot_dir(self.local_path)
            if current_snapshot != last_snapshot:
                print(f"Change detected in local directory {self.local_path}")
                last_snapshot = current_snapshot

    def _snapshot_dir(self, path: Path):
        snapshot = {}
        for root, dirs, files in os.walk(path):
            for f in files:
                fp = Path(root) / f
                try:
                    snapshot[str(fp)] = fp.stat().st_mtime
                except FileNotFoundError:
                    pass
        return snapshot
