#!/usr/bin/env bash
# B1 floor RSS measurement — uncapped peak + wall-clock, then exit codes at 8G/24G caps.
# Receipt authority: dag/gunbc/ci_floor_measurement.dag (update rows after green run).
#
# Usage:
#   ./scripts/b1_floor_rss_measurement.sh              # all three passes (uncapped, 8G, 24G)
#   ./scripts/b1_floor_rss_measurement.sh --uncapped-only
#   ./scripts/b1_floor_rss_measurement.sh --cap 8g     # single capped pass
#   ./scripts/b1_floor_rss_measurement.sh --cap 24g
#
# Capped passes require docker (--memory=CAP) or a writable cgroup-v2 subtree.
# Dashboard session containers expose a fixed memory.max (~31.27 GiB) and cannot
# emulate the 8 GiB runner cap without docker/cgroup write access.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CAP_8G=8589934592
CAP_24G=25769803776

BIN="${CLAIM_EXECUTOR:-$ROOT/target/release/claim_executor}"
FLOOR_CMD=(
  "$BIN"
  --source-root dag
  --source-root src/v2
  --plan-entry src/v2/workflow/ci_floor_plan.dag
  --plan-function gunbc_ci_floor_batches
  --notice-title "B1 floor RSS measurement"
)

MODE="all"
SINGLE_CAP=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --uncapped-only) MODE="uncapped"; shift ;;
    --cap)
      [[ $# -ge 2 ]] || { echo "b1: --cap requires 8g|24g" >&2; exit 2; }
      SINGLE_CAP="$2"
      MODE="single"
      shift 2
      ;;
    -h|--help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *) echo "b1: unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ ! -x "$BIN" ]]; then
  echo "b1: building claim_executor (release)…" >&2
  env -u RUSTC_WRAPPER CARGO_BUILD_JOBS=1 cargo build -p v1-compiler --release --features text_lookup_work_counter --bin claim_executor
fi

cap_bytes() {
  case "$1" in
    8g|8G) echo "$CAP_8G" ;;
    24g|24G) echo "$CAP_24G" ;;
    *) echo "b1: unknown cap $1 (want 8g or 24g)" >&2; return 1 ;;
  esac
}

docker_available() {
  local sock="${DOCKER_HOST:-}"
  if [[ -z "$sock" ]]; then
    for sock in /var/run/docker.sock /run/docker.sock; do
      [[ -S "$sock" ]] && break
    done
  fi
  [[ -S "${sock#unix://}" ]] || [[ "$sock" == unix://* && -S "${sock#unix://}" ]] || return 1
  docker info >/dev/null 2>&1
}

run_floor_labeled() {
  local label="$1"
  shift
  local log rc start end elapsed peak_rss cgroup_line
  log="$(mktemp "${TMPDIR:-/tmp}/b1-floor-${label}.XXXXXX.log")"
  start=$(date +%s)
  set +e
  if command -v stdbuf >/dev/null 2>&1; then
    stdbuf -oL -eL "$@" >"$log" 2>&1
  else
    "$@" >"$log" 2>&1
  fi
  rc=$?
  set -e
  end=$(date +%s)
  elapsed=$((end - start))
  peak_rss="$(rg -o '\[measurement\] floor peak RSS: [0-9]+' "$log" | tail -1 | rg -o '[0-9]+$' || true)"
  cgroup_line="$(rg '\[measurement\] cgroup peak:' "$log" | tail -1 || true)"
  echo "[b1-measurement] label=${label} exit=${rc} wall_seconds=${elapsed} peak_rss=${peak_rss:-unavailable}"
  [[ -n "$cgroup_line" ]] && echo "[b1-measurement] ${cgroup_line#\[measurement\] }"
  echo "[b1-measurement] log=${log}"
  case "$label" in
    uncapped) UNCAPPED_PEAK="${peak_rss:-}"; UNCAPPED_WALL="$elapsed"; UNCAPPED_EXIT="$rc" ;;
    cap-8g) EXIT_8G="$rc" ;;
    cap-24g) EXIT_24G="$rc" ;;
  esac
  return "$rc"
}

run_in_docker_cap() {
  local cap_label="$1"
  shift
  docker run --rm \
    --memory="$cap_label" \
    --memory-swap="$cap_label" \
    -v "$ROOT:$ROOT:rw" \
    -w "$ROOT" \
    -e HOME=/tmp \
    -e CARGO_BUILD_JOBS=1 \
    "${DOCKER_IMAGE:-ghcr.io/gunb-ai/ctrl-session:latest}" \
    "$@"
}

run_in_cgroup_cap() {
  local cap_b="$1"
  shift
  local cg="/sys/fs/cgroup/b1-floor-$$"
  mkdir -p "$cg"
  echo "$cap_b" >"$cg/memory.max"
  # Move this shell's children into the capped cgroup for the duration of the run.
  echo 0 >"$cg/cgroup.subtree_control" 2>/dev/null || true
  echo "+memory" >"$cg/cgroup.subtree_control" 2>/dev/null || true
  (
    echo $$ >"$cg/cgroup.procs"
    exec "$@"
  )
}

run_capped() {
  local cap_label="$1"
  local cap_b
  cap_b="$(cap_bytes "$cap_label")"
  if docker_available; then
    run_floor_labeled "cap-${cap_label}" run_in_docker_cap "$cap_label" "${FLOOR_CMD[@]}"
    return $?
  fi
  if mkdir -p "/sys/fs/cgroup/b1-floor-probe-$$" 2>/dev/null; then
    rmdir "/sys/fs/cgroup/b1-floor-probe-$$" 2>/dev/null || true
    run_floor_labeled "cap-${cap_label}" run_in_cgroup_cap "$cap_b" "${FLOOR_CMD[@]}"
    return $?
  fi
  echo "b1: capped run (${cap_label}) requires docker or writable cgroup-v2 — neither available" >&2
  echo "[b1-measurement] label=cap-${cap_label} exit=127 wall_seconds=0 peak_rss=unavailable reason=no_cap_enforcer"
  case "$cap_label" in
    8g|8G) EXIT_8G=127 ;;
    24g|24G) EXIT_24G=127 ;;
  esac
  return 127
}

UNCAPPED_PEAK=""
UNCAPPED_WALL=""
UNCAPPED_EXIT=""
EXIT_8G=""
EXIT_24G=""

if [[ "$MODE" == "all" || "$MODE" == "uncapped" ]]; then
  run_floor_labeled "uncapped" "${FLOOR_CMD[@]}" || true
fi

if [[ "$MODE" == "all" ]]; then
  run_capped 8g || EXIT_8G=$?
  run_capped 24g || EXIT_24G=$?
elif [[ "$MODE" == "single" ]]; then
  run_capped "$SINGLE_CAP"
fi

if [[ "$MODE" == "all" ]]; then
  echo "[b1-summary] uncapped_peak_rss=${UNCAPPED_PEAK:-unavailable} uncapped_wall_seconds=${UNCAPPED_WALL:-unavailable} exit_8g_cap=${EXIT_8G:-unavailable} exit_24g_cap=${EXIT_24G:-unavailable}"
fi
