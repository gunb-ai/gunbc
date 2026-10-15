#!/usr/bin/env bash
# One remote dispatch: emit+human cargo on 7faa288, then emit the same file on origin/main.
set -u
export CTRL_BUILD_MODE=local
unset RUSTC_WRAPPER || true
export CARGO_INCREMENTAL=0
export CARGO_TERM_COLOR=never
export RUSTFLAGS='-D warnings'
export GUNBC_NATIVE_PRODUCT_REUSE=fresh
export GUNBC_NATIVE_FAULT_EXPERIMENT=skip_pull_request
unset GUNBC_MEMORY_ENVELOPE_REQUIRED || true
unset GUNBC_NATIVE_CACHE_ROOT || true

(
  while true; do echo "HEARTBEAT $(date -u +%FT%TZ)" >&2; sleep 60; done
) &
HB=$!
trap 'kill $HB 2>/dev/null || true' EXIT

ROOT="$(pwd)"
echo "start_root=$ROOT sha=$(git rev-parse HEAD)"

emit_and_show() {
  local label=$1
  echo "=== BUILD $label $(git rev-parse HEAD) ==="
  cargo build --release -p v1-compiler --bin gunbc
  local G="$ROOT/target/release/gunbc"
  echo "=== EMIT $label ==="
  set +e
  "$G" test //gunbc/instruments:self-host 2>&1 | tee "/tmp/self-host-$label.log"
  local ex=${PIPESTATUS[0]}
  set -e
  echo "self-host_$label exit=$ex"
  local crate
  crate=$(grep -o 'crate_dir=[^ ]*' "/tmp/self-host-$label.log" | tail -1 | sed 's/^crate_dir=//')
  echo "crate_$label=$crate"
  if [ -z "$crate" ]; then
    echo "NO_CRATE_DIR $label"
    return 0
  fi
  local f
  f=$(find "$crate" -name 'v2_std_demand_engine.rs' | head -1)
  echo "demand_engine_$label=$f"
  if [ -n "$f" ]; then
    cp "$f" "/tmp/demand_engine_$label.rs"
    echo "=== $label demand_engine.rs:310-360 ==="
    nl -ba "$f" | sed -n '310,360p'
    echo "=== $label empty_map lines ==="
    grep -n 'empty_map\|rc_empty_map\|HashMap::new' "$f" | head -40
  fi
  echo "=== CARGO HUMAN $label ==="
  set +e
  (cd "$crate" && cargo build --release --offline --color never --manifest-path Cargo.toml) 2>&1 | tee "/tmp/cargo-human-$label.log"
  echo "cargo_human_$label exit=${PIPESTATUS[0]}"
  set -e
  echo "=== cargo human E0308/error $label ==="
  grep -n 'error\[E\|error: \|expected \|found ' "/tmp/cargo-human-$label.log" | head -80
}

git fetch origin main
git checkout --detach 7faa288c4d25780ba23e41322e3e3eb6755ba244
emit_and_show 7faa288

git checkout --detach origin/main
emit_and_show main

if [ -f /tmp/demand_engine_7faa288.rs ] && [ -f /tmp/demand_engine_main.rs ]; then
  echo "=== DIFF demand_engine 7faa288 vs main ==="
  diff -u /tmp/demand_engine_main.rs /tmp/demand_engine_7faa288.rs | head -200
fi
echo "=== ALL DONE ==="
