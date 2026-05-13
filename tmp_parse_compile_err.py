import os
import re
import subprocess
import sys

env = os.environ.copy()
env["CTRL_BUILD_BYPASS_SHIMS"] = "1"

p = subprocess.run(
    [
        "cargo",
        "test",
        "-p",
        "v3-compiler",
        "--test",
        "integration",
        "lens_self_application_demonstrated_ci_touch_all_affected_gates_order",
        "--",
        "--nocapture",
    ],
    cwd="/home/briansrls/.worktrees/jolly-bear-550",
    env=env,
    capture_output=True,
    text=True,
)
s = p.stdout + p.stderr
# Human-facing ResolveError names in Debug fmt
for m in re.finditer(r"ResolveError \{ name: \"([^\"]+)\"", s):
    print(m.group(1)[:300])
sys.exit(p.returncode)
