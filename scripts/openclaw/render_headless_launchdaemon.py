#!/usr/bin/env python3

from __future__ import annotations

import argparse
from pathlib import Path
from xml.sax.saxutils import escape


DEFAULT_LABEL = "ai.openclaw.gateway.headless"


def build_plist(label: str, openclaw_bin: str, username: str, home: str, workdir: str, stdout_path: str, stderr_path: str) -> str:
    values = {
        "label": escape(label),
        "openclaw_bin": escape(openclaw_bin),
        "username": escape(username),
        "home": escape(home),
        "workdir": escape(workdir),
        "stdout_path": escape(stdout_path),
        "stderr_path": escape(stderr_path),
    }
    return """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>

  <key>ProgramArguments</key>
  <array>
    <string>{openclaw_bin}</string>
    <string>gateway</string>
    <string>run</string>
  </array>

  <key>UserName</key>
  <string>{username}</string>

  <key>WorkingDirectory</key>
  <string>{workdir}</string>

  <key>EnvironmentVariables</key>
  <dict>
    <key>HOME</key>
    <string>{home}</string>
    <key>PATH</key>
    <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
  </dict>

  <key>RunAtLoad</key>
  <true/>

  <key>KeepAlive</key>
  <true/>

  <key>StandardOutPath</key>
  <string>{stdout_path}</string>

  <key>StandardErrorPath</key>
  <string>{stderr_path}</string>
</dict>
</plist>
""".format(**values)


def main() -> int:
    parser = argparse.ArgumentParser(description="Render a headless OpenClaw LaunchDaemon plist.")
    parser.add_argument("--label", default=DEFAULT_LABEL)
    parser.add_argument("--openclaw-bin", required=True)
    parser.add_argument("--username", required=True)
    parser.add_argument("--home", required=True)
    parser.add_argument("--workdir", required=True)
    parser.add_argument("--stdout-path", required=True)
    parser.add_argument("--stderr-path", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        build_plist(
            label=args.label,
            openclaw_bin=args.openclaw_bin,
            username=args.username,
            home=args.home,
            workdir=args.workdir,
            stdout_path=args.stdout_path,
            stderr_path=args.stderr_path,
        )
    )
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
