#!/usr/bin/env bash
# Run the fold-arm disposition rows at TWO heads in ONE dispatch.
#
# HOW IT AVOIDS THE TWO WAYS THIS COULD BE A LIE:
#  1. The rows are APPENDED to the head's own cross_coproduct_generic_locate_test.rs, so they run on that
#     file's imports and its fold_leaf/fold_app/fold_disj builders. Nothing here hand-copies the Node field
#     list, and the instrument is byte-identical at both heads (it comes from the instrument ref). The head's
#     own carrier file is never MODIFIED in a PR: this happens only in a throwaway clone.
#  2. A libtest filter that matches nothing prints 'ok' and exits 0 (it has already produced a false pass in
#     this lane). So each row is first checked against --list, and an absent row is VOID_NO_MATCH, never a
#     pass. Build failures are echoed RAW: two earlier suites printed NOTHING here because my own grep hid
#     the compile error, which the head's fold_disj comment also records.
#
# HEAD A = the strip (widening reverted, mirrors = emitter output) -> what main's relation does.
# HEAD B = the Acc head -> what the widening makes it do. A row that behaves identically at both is not
# discriminating, and only the pair can say so.
set -uo pipefail
export GIT_PAGER=cat PAGER=cat GIT_TERMINAL_PROMPT=0 CARGO_PROFILE_DEV_DEBUG=0
REPO=${REPO:-https://github.com/gunb-ai/gunbc.git}
FIX_REF=${FIX_REF:-refs/heads/scratch/rebind-census}
ROWS=${ROWS:-fold_arm_rows_appendix.rs}
TARGET=${TARGET:-/root/gunbc_target_fold}
mkdir -p "$TARGET"; export CARGO_TARGET_DIR="$TARGET"
stamp(){ echo "[$(date -u +%H:%M:%S)] $*"; }

# The rows come from the instrument ref ONCE, into a path both heads read, so "same bytes at both heads"
# is a property of the run rather than of my care.
rm -rf /tmp/foldfix
FIX_BR=${FIX_REF#refs/heads/}
git clone -q --depth 1 -b "$FIX_BR" "$REPO" /tmp/foldfix 2>/tmp/foldfix_clone_err || { echo "VOID_FIXREF_CLONE ref=$FIX_REF branch=$FIX_BR"; sed 's/^/  clone: /' /tmp/foldfix_clone_err; exit 3; }
[ -s "/tmp/foldfix/$ROWS" ] || { echo "VOID_ROWS_MISSING /tmp/foldfix/$ROWS"; exit 3; }
echo "ROWS_SHA256=$(sha256sum "/tmp/foldfix/$ROWS" | cut -c1-16) rows=/tmp/foldfix/$ROWS"

for SPEC in "STRIP:refs/heads/scratch/acc-without-equality-widening" "ACC_HEAD:refs/heads/session/quiet-hawk-507"; do
  LABEL=${SPEC%%:*}; REF=${SPEC#*:}
  W=/tmp/fold_$LABEL
  rm -rf "$W"
  stamp "=== $LABEL: fresh clone of $REF ==="
  git clone -q "$REPO" "$W" || { echo "$LABEL VOID_CLONE"; continue; }
  git -C "$W" fetch -q origin "$REF" && git -C "$W" checkout -q --detach FETCH_HEAD || { echo "$LABEL VOID_CHECKOUT"; continue; }
  SHA=$(git -C "$W" rev-parse HEAD); echo "$LABEL MEASURED_AT $SHA"
  CARRIER="$W/src/v1/tests/src/cross_coproduct_generic_locate_test.rs"
  [ -s "$CARRIER" ] || { echo "$LABEL VOID_CARRIER_MISSING $CARRIER"; continue; }
  # The head's carrier is the harness; the instrument is appended, never substituted.
  cp "$CARRIER" "$CARRIER.orig"
  printf '\n// ===== APPENDED INSTRUMENT (%s @ %s) =====\n' "$FIX_REF" "$(sha256sum /tmp/foldfix/$ROWS | cut -c1-16)" >> "$CARRIER"
  cat /tmp/foldfix/$ROWS >> "$CARRIER"
  echo "$LABEL CARRIER_LINES=$(wc -l < "$CARRIER") (orig $(wc -l < "$CARRIER.orig"))"
  # The head already registers this module; assert it rather than adding a duplicate that would not compile.
  grep -qE '^\s*(#\[cfg\(test\)\]\s*)?mod cross_coproduct_generic_locate_test;' "$W/src/v1/tests/src/lib.rs" \
    || { echo "$LABEL VOID_MODULE_NOT_REGISTERED (refusing to add it: a second mod for the same file is a compile error, and silently editing lib.rs here would make the run unattributable)"; continue; }
  cd "$W" || continue
  stamp "$LABEL: cargo build --tests -p v1-compiler-tests (cold once, shared target dir after)"
  ( cargo build --tests -p v1-compiler-tests ) > "/tmp/fold_${LABEL}_build.log" 2>&1
  brc=$?
  echo "$LABEL BUILD_RC=$brc"
  if [ $brc -ne 0 ]; then
    echo "$LABEL BUILD_FAILED raw tail (NOT grepped):"; tail -50 "/tmp/fold_${LABEL}_build.log"
    continue
  fi
  BIN=$(find "$TARGET" -type f -name 'v1_compiler_tests-*' -perm -u+x -newermt '-2 hours' 2>/dev/null | head -1)
  [ -n "$BIN" ] || BIN=$(find "$TARGET" -type f -name 'v1_compiler_tests-*' -perm -u+x 2>/dev/null | head -1)
  [ -n "$BIN" ] || { echo "$LABEL VOID_NO_BIN"; continue; }
  echo "$LABEL BIN=$BIN"
  LIST=$("$BIN" --list 2>/dev/null | grep '^cross_coproduct_generic_locate_test::d[0-9]' | sed 's/: test$//')
  echo "$LABEL INSTRUMENT_ROWS_LISTED=$(printf '%s\n' "$LIST" | grep -c . ) (expected 7)"
  printf '%s\n' "$LIST" | sed "s/^/$LABEL LISTED /"
  for t in $(printf '%s\n' "$LIST"); do
    safe=${t//::/_}
    "$BIN" --exact "$t" --nocapture --test-threads=1 > "/tmp/fold_${LABEL}_${safe}.log" 2>&1
    rc=$?
    verdict=PASS
    [ $rc -ne 0 ] && verdict=FAIL
    grep -q "test result: ok. 1 passed" "/tmp/fold_${LABEL}_${safe}.log" || verdict=VOID_NO_MATCH
    echo "$LABEL $verdict rc=$rc $t"
    sed 's/^/'"$LABEL"'   | /' "/tmp/fold_${LABEL}_${safe}.log" | head -20
  done
  stamp "$LABEL done"
done
stamp "ALLDONE"
