#!/usr/bin/env bash
# Corpus parse gate, kind of the step CI's `heal-generated-artifacts` job runs and that my earlier
# gates did NOT run -- which is how a .dag row with raw double quotes inside its string fields shipped
# and went red at 5cc80be8e59 (undefined variable 'this' on the row's REPAIR STATUS line).
#
# My parsecheck.py is a heuristic and reported that file clean. It is not the parser and must never
# again be the basis for "the row parses". This gate builds the real binary and runs the real parser
# over the corpus, the same entry the heal step uses.
#
# Usage: gate_corpus_parse_14152.sh <sha> <label> [extra .dag paths to assert were parsed]
set -u
SHA="${1:?sha}"; LABEL="${2:?label}"; shift 2 || true
URL=https://github.com/gunb-ai/gunbc
D=/tmp/gcp_$(echo "$LABEL" | tr -c 'a-zA-Z0-9' '_')
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_OPT_LEVEL=0 CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/root/gunbc_target_gcp}"
mkdir -p "$CARGO_TARGET_DIR"
stamp(){ echo "T=$(date -u +%H:%M:%S) $*"; }

stamp "corpus parse gate begin sha=$SHA label=$LABEL target=$(du -sh "$CARGO_TARGET_DIR" 2>/dev/null | cut -f1 || echo none)"
rm -rf "$D"; mkdir -p "$D"
timeout 600 git clone -q "$URL" "$D/repo" || { echo "FATAL clone"; exit 1; }
cd "$D/repo" || { echo "FATAL cd"; exit 1; }
timeout 300 git fetch -q origin "$SHA" || { echo "FATAL fetch $SHA"; exit 1; }
timeout 120 git checkout -q "$SHA" || { echo "FATAL checkout"; exit 1; }
echo "MEASURED_AT $(git rev-parse HEAD)"
[ "$(git rev-parse HEAD)" = "$SHA" ] || echo "WARNING requested $SHA"

stamp "build release binary begin"
timeout 2400 cargo build --release 2>&1 | tail -3
BUILD_RC=$?
echo "build rc=$BUILD_RC"
[ "$BUILD_RC" = 0 ] || { echo "VOID: the binary did not build, so no parse verdict means anything"; exit 6; }
BIN="$D/repo/target/release/gunbc"
[ -x "$BIN" ] || BIN="$(command -v gunbc || true)"
echo "binary=$BIN"
[ -x "$BIN" ] || { echo "VOID: no binary"; exit 6; }

# The exact command the failing CI step runs (recovered from the run log at 39381126080):
#   "$ROOT/target/release/gunbc" run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" \
#       --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet
stamp "corpus parse begin"
"$BIN" run --source-root "$D/repo/dag" --source-root "$D/repo/src/v2" \
  --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet \
  > "$D/parse.log" 2>&1
PARSE_RC=$?
echo "CORPUS_PARSE_RC=$PARSE_RC"
echo "parse log bytes=$(wc -c < "$D/parse.log")"
grep -E "^(error|error:)" "$D/parse.log" | head -12
echo "ERROR_LINES=$(grep -cE '^error' "$D/parse.log")"

# A refusal from the step's own logic is a legitimate outcome; an evaluator crash or a parse error is
# not. Separate them, because the step's refusal rendering is itself known-fragile
# (dag/gunbc/recurring_failure_mode/adding_a_seed_module_refuses_convergence_on_an_unowned_lib_rs.dag).
if grep -qE "NoSuchVariable|undefined variable|evaluating .* panicked|panic" "$D/parse.log"; then
  echo "CORPUS_PARSE_VERDICT=CRASH"
else
  case "$PARSE_RC" in
    0) echo "CORPUS_PARSE_VERDICT=CLEAN" ;;
    1|2) echo "CORPUS_PARSE_VERDICT=REFUSED_WITH_DIAGNOSTIC" ;;
    *) echo "CORPUS_PARSE_VERDICT=UNEXPECTED_RC" ;;
  esac
fi

# Report each extra path handed in. HONEST SCOPE: this only proves the file exists in the tree at this
# SHA and reports whether its name appears anywhere in the run trace; it does NOT prove the parser
# resolved it (that would need the evaluator to name every file it reads). The authoritative signal is
# CORPUS_PARSE_RC/VERDICT above, which parses the whole corpus the heal step resolves.
for p in "$@"; do
  [ -f "$D/repo/$p" ] && echo "ROW_IN_TREE $p" || echo "ROW_NOT_IN_TREE $p"
  if grep -q "$(basename "$p" .dag)" "$D/parse.log"; then echo "ROW_NAMED_IN_TRACE $p"; else echo "ROW_NOT_NAMED_IN_TRACE $p"; fi
done
echo "GATE DONE"
