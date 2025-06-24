import asyncio
from watcher import RepoWatcher
from ai_analyzer import AIAnalyzer
from urllib.parse import urlparse
import argparse
import sys

def parse_args():
    parser = argparse.ArgumentParser(description="Watcher + AI Analyzer integration")
    parser.add_argument("--uri", type=str, required=True, help="Repo URI (http, https, ftp, scp, localdir)")
    parser.add_argument("--stream-type", choices=["livestream", "offlinestream"], required=True, help="Stream type")
    parser.add_argument("--ftp-user", type=str, help="FTP username (optional)")
    parser.add_argument("--ftp-pass", type=str, help="FTP password (optional)")
    parser.add_argument("--scp-user", type=str, help="SCP username (optional)")
    parser.add_argument("--scp-pass", type=str, help="SCP password (optional)")
    return parser.parse_args()

async def async_input(prompt: str = "") -> str:
    print(prompt, end="", flush=True)
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, sys.stdin.readline)

async def main():
    args = parse_args()

    watcher = RepoWatcher(
        uri=args.uri,
        stream_type=args.stream_type,
        ftp_user=args.ftp_user,
        ftp_pass=args.ftp_pass,
        scp_user=args.scp_user,
        scp_pass=args.scp_pass,
    )
    ai = AIAnalyzer()

    # Start watcher task
    watcher_task = asyncio.create_task(watcher.watch())

    print("Watcher started. Type 'start' and press Enter to launch AI analysis. Type 'exit' to quit.")

    while True:
        cmd = (await async_input("> ")).strip().lower()
        if cmd == "start":
            scheme = urlparse(args.uri).scheme
            if args.stream_type == "offlinestream":
                await ai.analyze_local_dir(str(watcher.local_path))
            elif args.stream_type == "livestream":
                if scheme in ["http", "https"]:
                    await ai.analyze_livestream(args.uri)
                else:
                    print("Livestream AI analysis is supported only for http/https.")
        elif cmd == "exit":
            print("Exiting program...")
            watcher_task.cancel()
            try:
                await watcher_task
            except asyncio.CancelledError:
                pass
            break
        else:
            print("Unknown command. Please type 'start' or 'exit'.")

if __name__ == "__main__":
    asyncio.run(main())
