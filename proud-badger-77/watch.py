import json
import os
import subprocess

STATE = "/tmp/pb77_watch_state.json"
ME = "proud-badger-77"
BRANCH = "nominal-child-edge-labels"
WT = "/home/briansrls/.worktrees/proud-badger-77"


def fetch(hours=24, limit=60):
    r = subprocess.run(["dashboard-ops", "messages", "mine", str(hours), str(limit)],
                       capture_output=True, text=True)
    return json.loads(r.stdout) if r.stdout.strip() else {}


def inbound(d):
    out = []
    for m in d.get("messages") or []:
        if not isinstance(m, dict):
            continue
        snd = m.get("sender_session_id") or m.get("from")
        rcv = m.get("recipient_session_id") or ""
        if snd == ME or (rcv and rcv != ME):
            continue
        out.append((m.get("id") or "", m.get("ts") or m.get("created_at") or "",
                    snd or "?", (m.get("body") or "").strip().replace("\n", " ")))
    out.sort(key=lambda r: r[1])
    return out


def head():
    r = subprocess.run(["git", "-C", WT, "ls-remote", "origin", f"refs/heads/{BRANCH}"],
                       capture_output=True, text=True)
    return r.stdout.split("\t")[0][:40] if r.stdout.strip() else "?"


seen = set()
if os.path.exists(STATE):
    try:
        seen = set(json.load(open(STATE)))
    except Exception:
        seen = set()

rs = inbound(fetch())
new = [r for r in rs if r[0] not in seen]
print(f"watch: {len(rs)} inbound in window, {len(new)} NEW since last check; {BRANCH} head={head()}")
for _id, t, f, b in new:
    print(f"--- NEW {(t or '')[11:19]} {f} | {b[:400]}")
json.dump(sorted(seen | {r[0] for r in rs}), open(STATE, "w"))
