#!/usr/bin/env bash
# usage: run-slice.sh <arm-root> <file-list-slice> <out-dir>
# Sequential: first file records memory.max inside the scope. Parallelise by giving
# each job its own slice and out-dir (or the same out-dir if you lock the append).
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <arm-root> <file-list-slice> <out-dir>" >&2
  exit 2
fi

ARM_ROOT="$(cd "$1" && pwd)"
SLICE="$2"
OUT="$(mkdir -p "$3" && cd "$3" && pwd)"
GUNBC="${GUNBC:-$ARM_ROOT/target/release/gunbc}"
MEMORY_MAX="${MEMORY_MAX:-16G}"
RUNTIME_MAX="${RUNTIME_MAX:-1200}"

if [[ ! -x "$GUNBC" ]]; then
  echo "gunbc not executable: $GUNBC" >&2
  exit 2
fi
if [[ ! -f "$SLICE" ]]; then
  echo "missing slice: $SLICE" >&2
  exit 2
fi

MANIFEST="$OUT/runs.tsv"
READBACK="$OUT/memory.max.readback"
LOCK="$OUT/runs.lock"
if [[ ! -f "$MANIFEST" ]]; then
  printf 'path\treason\twall_sec\tcpu_user_sec\tcpu_sys_sec\tmax_rss_kb\tstatus\ttsv\n' >"$MANIFEST"
fi

parse_log() {
  python3 -c '
import re, sys
text = sys.stdin.read()
def grab(pat):
    m = re.search(pat, text)
    return m.group(1) if m else ""
wall = grab(r"Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):\s+(\S+)")
user = grab(r"User time \(seconds\):\s+(\S+)")
sys_t = grab(r"System time \(seconds\):\s+(\S+)")
rss = grab(r"Maximum resident set size \(kbytes\):\s+(\S+)")
secs = ""
if wall:
    parts = wall.split(":")
    try:
        if len(parts) == 3:
            secs = str(int(parts[0]) * 3600 + int(parts[1]) * 60 + float(parts[2]))
        elif len(parts) == 2:
            secs = str(int(parts[0]) * 60 + float(parts[1]))
        else:
            secs = wall
    except ValueError:
        secs = wall
tsv = ""
m = re.search(r"returned `(.*)`, not `ProcessExit`", text, re.S)
if m:
    tsv = m.group(1).replace("\n", " ").strip()
mem = grab(r"MEMORY_MAX_READBACK=(\S+)")
print("\t".join([secs, user, sys_t, rss, tsv, mem]))
'
}

reason_of() {
  local status="$1" tsv="$2"
  if [[ "$status" -eq 137 || "$status" -eq 9 ]]; then
    echo oom
    return
  fi
  if [[ "$status" -eq 124 || "$status" -eq 152 || "$status" -eq 143 ]]; then
    echo timeout
    return
  fi
  if [[ -z "$tsv" ]]; then
    echo other
    return
  fi
  local standing suspects
  standing="$(printf '%s' "$tsv" | awk -F'\t' '{print $2}')"
  suspects="$(printf '%s' "$tsv" | awk -F'\t' '{print $3}')"
  if [[ "$standing" == "scanned" ]]; then
    if [[ "$suspects" =~ ^[0-9]+$ && "$suspects" -gt 0 ]]; then
      echo suspect
    else
      echo ok
    fi
  else
    echo other
  fi
}

append_row() {
  local line="$1"
  if command -v flock >/dev/null 2>&1; then
    flock "$LOCK" bash -c 'printf "%s\n" "$1" >>"$2"' _ "$line" "$MANIFEST"
  else
    printf '%s\n' "$line" >>"$MANIFEST"
  fi
}

first=1
while IFS= read -r path || [[ -n "$path" ]]; do
  [[ -z "$path" || "$path" == \#* ]] && continue
  log="$OUT/$(echo "$path" | tr '/ ' '__').log"
  set +e
  (
    cd "$ARM_ROOT"
    systemd-run --user --scope --quiet --collect \
      -p "MemoryMax=${MEMORY_MAX}" \
      -p "RuntimeMaxSec=${RUNTIME_MAX}" \
      -- env \
        GUNBC="$GUNBC" \
        PATH_REL="$path" \
        READBACK_FIRST="$first" \
        bash -c '
          set +e
          if [ "$READBACK_FIRST" = 1 ]; then
            echo "MEMORY_MAX_READBACK=$(cg=$(awk -F: '/^0::/{print $3}' /proc/self/cgroup); : ${cg:=/}; cat /sys/fs/cgroup${cg}/memory.max 2>/dev/null || echo UNREADABLE)"
          fi
          exec /usr/bin/time -v "$GUNBC" run \
            --source-root dag --source-root src/v2 \
            --entry src/v2/lens/complexity_accumulator_copy/corpus_census.dag \
            --function census_tsv_line_for_path \
            --arg "path=$PATH_REL"
        '
  ) >"$log" 2>&1
  status=$?
  set -e
  parsed="$(parse_log <"$log")"
  IFS=$'\t' read -r wall user sys rss tsv mem <<<"$parsed"
  if [[ "$first" -eq 1 && -n "$mem" && ! -s "$READBACK" ]]; then
    echo "MEMORY_MAX_READBACK=$mem" >"$READBACK"
  fi
  reason="$(reason_of "$status" "$tsv")"
  append_row "$(printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s' "$path" "$reason" "${wall:-}" "${user:-}" "${sys:-}" "${rss:-}" "$status" "${tsv:-}")"
  first=0
done <"$SLICE"
