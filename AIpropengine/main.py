import sys
from AIpropengine import version

# --version veya -v varsa önce versiyonu göster ve çık
if "--version" in sys.argv or "-v" in sys.argv:
    version.print_version()
    print("\033[94mcontact us:\033[92m admin@azccriminal.space\033[0m")
    print("\033[94mdeveloper:\033[92m Zaman Huseyinli\033[0m")
    print("\033[94msite:\033[92m https://azccriminal.space\033[0m")
    sys.exit(0)

import asyncio
from AIpropengine.watcher import RepoWatcher, set_gitroot
from AIpropengine.ai_analyzer import AIAnalyzer
from urllib.parse import urlparse
import argparse

def parse_args():
    parser = argparse.ArgumentParser(description="Watcher + AI Analyzer integration")
    parser.add_argument("--uri", type=str, required=True, help="Repo URI (http, https, ftp, scp, localdir)")
    parser.add_argument("--stream-type", choices=["livestream", "offlinestream"], required=True, help="Stream type")
    parser.add_argument("--mode", choices=["single", "multiple"], default="single", help="GITROOT mode")
    parser.add_argument("--ftp-user", type=str, help="FTP username (optional)")
    parser.add_argument("--ftp-pass", type=str, help="FTP password (optional)")
    parser.add_argument("--scp-user", type=str, help="SCP username (optional)")
    parser.add_argument("--scp-pass", type=str, help="SCP password (optional)")
    return parser.parse_args()

async def async_input(prompt: str = "") -> str:
    print(prompt, end="", flush=True)
    loop = asyncio.get_running_loop()
    line = await loop.run_in_executor(None, sys.stdin.readline)
    return line.strip()

async def async_main():
    args = parse_args()

    # GITROOT ortam değişkenini set et
    set_gitroot(args.mode)

    watcher = RepoWatcher(
        uri=args.uri,
        stream_type=args.stream_type,
        ftp_user=args.ftp_user,
        ftp_pass=args.ftp_pass,
        scp_user=args.scp_user,
        scp_pass=args.scp_pass,
    )
    ai = AIAnalyzer()

    watcher_task = asyncio.create_task(watcher.watch())

    print("Watcher started. Type 'start' and press Enter to launch AI analysis. Type 'exit' to quit.")

    try:
        while True:
            cmd = (await async_input("> ")).lower()
            if cmd == "start":
                scheme = urlparse(args.uri).scheme
                try:
                    if args.stream_type == "offlinestream":
                        # Eğer birden fazla repo varsa, her repo için ayrı bir analiz yapalım
                        uris = args.uri.split(",")  # Virgülle ayrılan URI'leri ayırıyoruz
                        for uri in uris:
                            await ai.analyze_local_dir(local_dir=None, gitroot_mode=args.mode)
                    elif args.stream_type == "livestream":
                        if scheme in ["http", "https"]:
                            await ai.analyze_livestream(args.uri)
                        else:
                            print("Livestream AI analysis is supported only for http/https.")
                except Exception as e:
                    print(f"AI analysis error: {e}")
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
    except KeyboardInterrupt:
        print("\nInterrupted by user. Exiting...")
        watcher_task.cancel()
        try:
            await watcher_task
        except asyncio.CancelledError:
            pass

def main():
    asyncio.run(async_main())

if __name__ == "__main__":
    main()
