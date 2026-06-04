#!/usr/bin/env bash
# Ground-truth the v4 claim corpus by EXECUTION.
#
# Loops the existing single-witness CLI (`gunbc run --claim-run`) over every
# Bool witness in src/v4/test/claim/** and classifies each by its runtime
# outcome. This is the reproducibility harness behind
# docs/planning/v4-claim-corpus-execution-map-2026-06-04.md — re-run it to
# refresh the map after interpreter/substrate changes.
#
# Two witness classes:
#   pass 1 -- nullary `fn name() -> Bool`
#   pass 2 -- `data name: Bool = expr` (the runner evaluates these as 0-arg thunks)
#
# Classification is by OUTPUT TEXT, not exit code (a missing-fn run also exits 1):
#   GREEN   printed `true`            -- claim executed and holds
#   RED     printed `false`           -- claim executed and is FALSE (behavioral)
#   ERROR   compile/parse/runtime err -- witness never executed (infra/interp)
#   PERF    exceeded wall/mem cap     -- perf-wall, keystone track (NOT a red)
#
# Caps are MECHANICAL (not a hand-curated category allowlist) so the perf
# boundary cannot drift. Sequential to respect the container pids-cgroup ceiling.
#
# Usage: scripts/v4-claim-corpus-execution-map.sh [out_dir]
set -u
BIN=${GUNBC_BIN:-./target/release/gunbc}
ROOT=src/v4
CLAIMS="$ROOT/test/claim"
OUT=${1:-.claim-map}
WALL=${WITNESS_WALL_SECS:-60}
MEMKB=${WITNESS_MEM_KB:-6291456}   # 6 GiB vmem -> OOM self-classifies as PERF
mkdir -p "$OUT"
TSV="$OUT/results.tsv"
printf 'status\tclass\tfile\twitness\tsecs\texit\tlastline\n' > "$TSV"

if [ ! -x "$BIN" ]; then
  echo "error: $BIN not found; build with: cargo build --release -p v2-compiler --bin gunbc" >&2
  exit 2
fi

classify() { # $1=exit $2=output -> echoes status
  local ex="$1" out="$2"
  if [ "$ex" = 124 ]; then echo PERF; return; fi
  if printf '%s' "$out" | grep -qiE 'out of memory|memory allocation|cannot allocate'; then echo PERF; return; fi
  if printf '%s\n' "$out" | grep -qx 'true';  then echo GREEN; return; fi
  if printf '%s\n' "$out" | grep -qx 'false'; then echo RED;   return; fi
  echo ERROR
}

run_one() { # $1=class $2=file $3=witness
  local cls="$1" file="$2" w="$3" s e out ex secs last st
  s=$(date +%s)
  out=$( ( ulimit -v "$MEMKB" 2>/dev/null; timeout "$WALL" "$BIN" run \
            --source-root "$ROOT" --entry "$file" --function "$w" --claim-run ) 2>&1 )
  ex=$?; e=$(date +%s); secs=$((e-s))
  last=$(printf '%s\n' "$out" | grep -v '^resolved\|^running' | tail -1 | tr '\t' ' ')
  st=$(classify "$ex" "$out")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$st" "$cls" "$file" "$w" "$secs" "$ex" "$last" >> "$TSV"
}

# A data witness `data w: Bool = foo()` where `foo` is a nullary Bool fn in the
# SAME file is a rebinding — running it re-evaluates a fn already measured in
# pass 1, so it would double-count. Skip those; run only distinct data witnesses.
is_rebinding() { # $1=file $2=witness -> 0 (true) iff pure same-file fn rebinding
  local rhs callee
  rhs=$(grep -P "^data $2 *: *Bool *=" "$1" | head -1 | sed -E 's/^data [a-zA-Z0-9_]+ *: *Bool *= *//')
  printf '%s' "$rhs" | grep -qP '^[a-zA-Z0-9_]+\(\)[[:space:]]*$' || return 1
  callee=$(printf '%s' "$rhs" | grep -oP '^[a-zA-Z0-9_]+')
  grep -qP "^fn ${callee}\(\) -> Bool" "$1"
}

n=0
for f in $(find "$CLAIMS" -name '*.dag' | sort); do
  fns=$(grep -oP "^fn \K\w+(?=\(\) -> Bool)" "$f" 2>/dev/null)
  data=$(grep -oP "^data \K[a-zA-Z0-9_]+(?= *: *Bool *=)" "$f" 2>/dev/null)
  if [ -z "$fns" ] && [ -z "$data" ]; then
    printf 'NOWITNESS\tnone\t%s\t-\t0\t-\tno Bool witness (library/roster)\n' "$f" >> "$TSV"
    continue
  fi
  for w in $fns;  do [ -n "$w" ] && run_one fn "$f" "$w" && n=$((n+1)); done
  for w in $data; do
    [ -z "$w" ] && continue
    is_rebinding "$f" "$w" && continue   # dedup: skip fn rebindings (counted in pass 1)
    run_one data "$f" "$w" && n=$((n+1))
  done
  [ $((n % 25)) -lt 2 ] && echo "...$n witnesses run" >&2
done
echo "DONE: $n distinct witnesses executed" >&2

echo
echo "==== status x class ===="
tail -n +2 "$TSV" | awk -F'\t' '{print $1, $2}' | sort | uniq -c | sort -rn
