set -euo pipefail
export PATH="/home/briansrls/.cargo/bin:$PATH"
gate_head=${1:-fabd35f5a2476de5b7bd76b2010394c39c4522ef}
gate_root=$(mktemp -d /home/briansrls/royal-hawk-241/build-gate.XXXXXX)
printf '%s\n' "$gate_root/build-gate.log"
git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$gate_root/repo"
git -C "$gate_root/repo" fetch origin "$gate_head"
git -C "$gate_root/repo" checkout --detach "$gate_head"
mkdir "$gate_root/runner-temp"
cat > "$gate_root/observe.py" <<'PY'
import hashlib, os, pathlib, signal, subprocess, time
repo = pathlib.Path.cwd()
stale = [p for p in (repo/'target/release').glob('gunbc-emitted-closure-*') if p.is_file() and os.access(p, os.X_OK)]
print('BUILD_GATE stale_binaries_deleted=' + str(len(stale)), flush=True)
for p in stale: p.unlink()
started = time.time_ns()
print(f'BUILD_GATE started_ns={started} head={os.environ["GITHUB_SHA"]}', flush=True)
proc = subprocess.Popen(['./target/release/claim_executor', '--v2-native-route', '--source-root', 'dag', '--source-root', 'src/v2'], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
seen = False
try:
    for line in proc.stdout:
        print(line, end='', flush=True)
        if 'EmittedCompilerBuildFailed' in line: raise RuntimeError('emitted build refused')
        if 'emitted crate built' in line:
            if 'exit_status=0 warning_count=0' not in line: raise RuntimeError('build did not meet gate')
            seen = True
        if seen and 'emitted compiler at ' in line:
            binary = pathlib.Path(line.split('emitted compiler at ', 1)[1].split(' (sha256 ', 1)[0])
            stat = binary.stat()
            if stat.st_mtime_ns < started: raise RuntimeError('binary predates route start')
            print(f'BUILD_GATE binary={binary} sha256={hashlib.sha256(binary.read_bytes()).hexdigest()} mtime_ns={stat.st_mtime_ns}', flush=True)
            print('BUILD_GATE PASS; stopped after build proof; NOT A MEASUREMENT', flush=True)
            break
    else: raise RuntimeError('route exited without complete build proof')
finally:
    if proc.poll() is None:
        os.killpg(proc.pid, signal.SIGTERM)
        try: proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait()
PY
cd "$gate_root/repo"
systemd-run --user --scope -p MemoryMax=24G env PATH="$PATH" RUNNER_TEMP="$gate_root/runner-temp" GITHUB_SHA="$gate_head" CTRL_BUILD_MODE=local RUSTFLAGS='-D warnings' bash -c 'set -euo pipefail; cargo build --release -p v1-compiler --bin claim_executor; python3 "$1"' bash "$gate_root/observe.py" > "$gate_root/build-gate.log" 2>&1
