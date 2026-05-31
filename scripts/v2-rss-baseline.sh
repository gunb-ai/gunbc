#!/usr/bin/env bash
# Reproducible v2 build/test RSS baseline (srv1 methodology from merry-newt-844 brief).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

export CTRL_BUILD_BYPASS_SHIMS=1
export RUSTC_WRAPPER=

poll_rustc_rss() {
  local pid=$1
  local best_sum=0 best_max=0 best_n=0
  while kill -0 "$pid" 2>/dev/null; do
    read -r sum max n < <(
      ps -eo rss=,comm= 2>/dev/null | awk '
        $2 ~ /rustc/ { s+=$1; if ($1>mx) mx=$1; c++ }
        END { print s+0, mx+0, c+0 }
      '
    )
    if (( sum > best_sum )); then
      best_sum=$sum best_max=$max best_n=$n
    fi
    sleep 0.3
  done
  echo "$best_sum $best_max $best_n"
}

poll_test_rss() {
  local pid=$1
  local best=0
  while kill -0 "$pid" 2>/dev/null; do
    read -r peak < <(
      ps -eo rss=,comm= 2>/dev/null | awk '
        $2 ~ /v2-compiler-tests|full_dsl/ { if ($1>m) m=$1 }
        END { print m+0 }
      '
    )
    if (( peak > best )); then best=$peak; fi
    sleep 0.3
  done
  echo "$best"
}

kb_to_mb() { awk -v k="$1" 'BEGIN { printf "%.2f", k/1024 }'; }

measure_build() {
  local label=$1
  shift
  local crate=$1
  shift

  echo "=== $label ==="
  cargo clean -p "$crate" >/dev/null 2>&1 || true

  local start end wall
  start=$(date +%s.%N)
  cargo "$@" &
  local pid=$!
  read -r rss_sum rss_max rss_n < <(poll_rustc_rss "$pid")
  wait "$pid"
  end=$(date +%s.%N)
  wall=$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.1f", e-s }')

  echo "wall_s: $wall"
  echo "peak_rustc_rss_sum_kb: $rss_sum ($(kb_to_mb "$rss_sum") MB)"
  echo "peak_rustc_rss_max_kb: $rss_max ($(kb_to_mb "$rss_max") MB)"
  echo "peak_rustc_count: $rss_n"
}

measure_build "a) v2-compiler --lib (debug)" v2-compiler build -p v2-compiler --lib
echo ""

# Artifact sizes for (a)
if [[ -f target/debug/libv2_compiler.rlib ]]; then
  echo "libv2_compiler.rlib: $(du -b target/debug/libv2_compiler.rlib | awk '{printf "%.2f MB", $1/1024/1024}')"
fi
rmeta=$(find target/debug/deps -name 'libv2_compiler-*.rmeta' 2>/dev/null | head -1)
if [[ -n $rmeta ]]; then
  echo "libv2_compiler.rmeta: $(du -b "$rmeta" | awk '{printf "%.2f MB", $1/1024/1024}')"
fi
echo ""

measure_build "b) v2-compiler-tests --tests (debug)" v2-compiler-tests build -p v2-compiler-tests --tests
echo ""

test_bin=$(find target/debug/deps -name 'v2_compiler_tests-*' -perm -111 2>/dev/null | head -1)
if [[ -n $test_bin ]]; then
  echo "v2-compiler-tests binary: $(du -b "$test_bin" | awk '{printf "%.2f MB", $1/1024/1024}') ($test_bin)"
fi
echo ""

echo "=== c) full_dsl_compiles --ignored (runtime peak) ==="
cargo clean -p v2-compiler-tests >/dev/null 2>&1 || true
# Ensure test binary exists
cargo build -p v2-compiler-tests --tests >/dev/null

start=$(date +%s.%N)
cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored --nocapture >/dev/null 2>&1 &
pid=$!
read -r test_peak < <(poll_test_rss "$pid")
wait "$pid" || true
end=$(date +%s.%N)
wall=$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.1f", e-s }')

echo "wall_s: $wall"
echo "peak_test_binary_rss_kb: $test_peak ($(kb_to_mb "$test_peak") MB)"
