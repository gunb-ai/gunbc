#!/usr/bin/env bash
# (b) RED runner (stern-bat-401 msg_db49ee15 (3)): compile fixtures_alias_b/alias_red.dag with a
# gunbc built from a PRINTED SHA (normally main), with NO change to the fold, and print every
# diagnostic verbatim so the refusal SITE can be named from the message, not from my reading.
# USAGE: bash run_alias_red_b.sh <sha> <label>
set -uo pipefail
SHA="${1:?sha}"; LABEL="${2:?label}"
WORK=/tmp/aliasred_${LABEL}; rm -rf "$WORK"; mkdir -p "$WORK"
d=$WORK/repo
echo "T=$(date +%T) clone begin"
git clone -q --no-checkout https://github.com/gunb-ai/gunbc.git "$d" || { echo "VOID clone"; exit 7; }
( cd "$d" && git checkout -q "$SHA" ) || { echo "VOID checkout"; exit 7; }
echo "T=$(date +%T) MEASURED_AT $(cd $d && git rev-parse HEAD)"
cp -r fixtures_alias_b "$d/"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/tgt}"; mkdir -p "$CARGO_TARGET_DIR"
( cd "$d" && CARGO_PROFILE_DEV_DEBUG=0 cargo build --release -p v1-compiler --bin gunbc ) > "$WORK/build.log" 2>&1
brc=$?; bin="$CARGO_TARGET_DIR/release/gunbc"
echo "build rc=$brc exists=$([ -x "$bin" ] && echo yes || echo no)"
[ $brc -eq 0 ] && [ -x "$bin" ] || { tail -25 "$WORK/build.log"; exit 6; }
mkdir -p "$WORK/src" && cp fixtures_alias_b/alias_red.dag "$WORK/src/alias_red.dag"
( cd "$WORK/src" && "$bin" compile --source-dir . --output-dir "$WORK/emit" --target rust ) > "$WORK/compile.log" 2>&1
crc=$?
echo "COMPILE_RC=$crc"
echo "=== HARD DIAGNOSTIC COUNT ==="
grep -aoE "produced [0-9]+ hard diagnostic" "$WORK/compile.log" | head -2
echo "=== UNRESOLVED-IMPORT GUARD (an unresolved import invalidates the row) ==="
grep -ac "unresolved import" "$WORK/compile.log" | sed 's/^/UNRESOLVED_IMPORTS=/'
grep -ac "unresolved type" "$WORK/compile.log" | sed 's/^/UNRESOLVED_TYPES=/'
echo "=== diagnostics verbatim ==="
grep -anE "error|refus|mismatch|expected|got|unresolved|no field" "$WORK/compile.log" | head -40
echo "=== full log (first 120 lines) ==="
head -120 "$WORK/compile.log"
echo "T=$(date +%T) done"
