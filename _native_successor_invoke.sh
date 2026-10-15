#!/usr/bin/env bash
# Native successor invocation: pin origin/main, seed-build native CLI, native-emit subjects.
set -euo pipefail
PIN_SHA="${PIN_SHA:?PIN_SHA required}"
REPO_ROOT="${REPO_ROOT:-$(pwd)}"
cd "$REPO_ROOT"

stage() {
  echo "SUCCESSOR_STAGE=$1"
  echo "SUCCESSOR_SHA=$(git rev-parse HEAD)"
}

heard() {
  echo "SUCCESSOR_FIRST_REFUSAL_CLASS=$1"
  echo "SUCCESSOR_FIRST_REFUSAL_BEGIN"
  cat
  echo "SUCCESSOR_FIRST_REFUSAL_END"
}

echo "SUCCESSOR_PIN_SHA=$PIN_SHA"
stage pin_check
HEAD_SHA="$(git rev-parse HEAD)"
if [ "$HEAD_SHA" != "$PIN_SHA" ]; then
  echo "SUCCESSOR_REFUSAL=pin_mismatch HEAD=$HEAD_SHA PIN=$PIN_SHA" >&2
  exit 2
fi
# Remote BuildBuddy checkouts force HEAD to PIN_SHA; origin/main may be a stale
# cached ref from a depth-1 fetch of that object, so HEAD==PIN is the check.
echo "SUCCESSOR_HEAD_OK=$HEAD_SHA"
echo "SUCCESSOR_ORIGIN_MAIN_REF=$(git rev-parse origin/main 2>/dev/null || echo unset)"
uname -m
echo "SUCCESSOR_CARGO=$(command -v cargo)"
echo "SUCCESSOR_CTRL_BUILD_MODE=${CTRL_BUILD_MODE:-unset}"

stage seed_build_gunbc
cargo build --release -p v1-compiler --bin gunbc
GUNBC="$REPO_ROOT/target/release/gunbc"
test -x "$GUNBC"
echo "SUCCESSOR_GUNBC=$GUNBC"
ls -l "$GUNBC"

stage seed_emit_build_v2_native_cli
export GUNBC_NATIVE_PRODUCT_REUSE=fresh
CLI_TEST_LOG="$REPO_ROOT/target/native-successor/v2-native-cli-test.log"
mkdir -p "$(dirname "$CLI_TEST_LOG")"
set +e
"$GUNBC" test //gunbc/instruments:v2-native-cli 2>&1 | tee "$CLI_TEST_LOG"
CLI_TEST_RC=${PIPESTATUS[0]}
set -e
echo "SUCCESSOR_V2_NATIVE_CLI_TEST_RC=$CLI_TEST_RC"
if [ "$CLI_TEST_RC" -ne 0 ]; then
  heard seed_instrument_v2_native_cli <<EOF
gunbc test //gunbc/instruments:v2-native-cli exited $CLI_TEST_RC (seed emit/build/door of v2.cli.compile_cli; successor native emit not reached)
EOF
  exit "$CLI_TEST_RC"
fi

GEN_ONE="$(sed -n 's/.*generation_one_executable=//p' "$CLI_TEST_LOG" | tail -n 1)"
if [ -z "$GEN_ONE" ]; then
  GEN_ONE="$(find /tmp "$REPO_ROOT/target" /var/tmp -name 'v2-native-cli-gen-one-*' -type f 2>/dev/null | head -n 1 || true)"
fi
if [ -z "$GEN_ONE" ] || [ ! -x "$GEN_ONE" ]; then
  heard runtime_failure <<EOF
generation-one executable missing after v2-native-cli held (looked for generation_one_executable= in log and v2-native-cli-gen-one-* under /tmp)
EOF
  exit 1
fi
test -x "$GEN_ONE"
echo "SUCCESSOR_GEN_ONE=$GEN_ONE"
file "$GEN_ONE" || true
ls -l "$GEN_ONE"

native_emit() {
  local entry="$1"
  local out="$2"
  local err="$3"
  stage "native_emit_${entry}"
  set +e
  "$GEN_ONE" emit --entry "$entry" --source-root dag --source-root src/v2 >"$out" 2>"$err"
  local rc=$?
  set -e
  echo "SUCCESSOR_NATIVE_EMIT_ENTRY=$entry"
  echo "SUCCESSOR_NATIVE_EMIT_RC=$rc"
  echo "SUCCESSOR_NATIVE_EMIT_STDOUT_BYTES=$(wc -c <"$out")"
  echo "SUCCESSOR_NATIVE_STDERR_BEGIN"
  cat "$err"
  echo "SUCCESSOR_NATIVE_STDERR_END"
  return "$rc"
}

OUT_DIR="$REPO_ROOT/target/native-successor"
mkdir -p "$OUT_DIR"
COMP_OUT="$OUT_DIR/v2.compiler.compile.json"
COMP_ERR="$OUT_DIR/v2.compiler.compile.stderr"
CLI_OUT="$OUT_DIR/v2.cli.compile_cli.json"
CLI_ERR="$OUT_DIR/v2.cli.compile_cli.stderr"

set +e
native_emit v2.compiler.compile "$COMP_OUT" "$COMP_ERR"
COMP_RC=$?
set -e
if [ "$COMP_RC" -ne 0 ]; then
  DETAIL="$(tail -c 8000 "$COMP_ERR"; echo; python3 -c "
import json,sys
p='$COMP_OUT'
try:
    d=json.load(open(p))
except Exception as e:
    print('stdout_not_json',e); sys.exit(0)
print('entry', d.get('entry'))
print('member_count', len(d.get('members') or []))
mods=d.get('modules') or []
print('module_count', len(mods))
for m in mods:
    if 'refused' in m:
        print('FIRST_MEMBER_REFUSED', m.get('module'), m.get('refused'))
        break
else:
    print('no_member_refusal_in_json')
" 2>/dev/null || true)"
  heard emission_refusal <<EOF
native CLI emit --entry v2.compiler.compile --source-root dag --source-root src/v2
exit=$COMP_RC
$DETAIL
EOF
  exit "$COMP_RC"
fi

python3 - <<'PY'
import json,sys
p=open("target/native-successor/v2.compiler.compile.json")
d=json.load(p)
mods=d.get("modules") or []
for m in mods:
    if "refused" in m:
        print("SUCCESSOR_COMPILER_MEMBER_REFUSED", m.get("module"), m.get("refused"))
        sys.exit(3)
print("SUCCESSOR_COMPILER_ALL_EMITTED", len(mods))
PY

set +e
native_emit v2.cli.compile_cli "$CLI_OUT" "$CLI_ERR"
CLI_RC=$?
set -e
if [ "$CLI_RC" -ne 0 ]; then
  DETAIL="$(tail -c 8000 "$CLI_ERR"; echo; python3 -c "
import json,sys
p='$CLI_OUT'
try:
    d=json.load(open(p))
except Exception as e:
    print('stdout_not_json',e); sys.exit(0)
print('entry', d.get('entry'))
print('member_count', len(d.get('members') or []))
mods=d.get('modules') or []
print('module_count', len(mods))
for m in mods:
    if 'refused' in m:
        print('FIRST_MEMBER_REFUSED', m.get('module'), m.get('refused'))
        break
else:
    print('no_member_refusal_in_json')
" 2>/dev/null || true)"
  heard emission_refusal <<EOF
native CLI emit --entry v2.cli.compile_cli --source-root dag --source-root src/v2
exit=$CLI_RC
$DETAIL
EOF
  exit "$CLI_RC"
fi

python3 - <<'PY'
import json,sys
p=open("target/native-successor/v2.cli.compile_cli.json")
d=json.load(p)
mods=d.get("modules") or []
for m in mods:
    if "refused" in m:
        print("SUCCESSOR_CLI_MEMBER_REFUSED", m.get("module"), m.get("refused"))
        sys.exit(3)
print("SUCCESSOR_CLI_ALL_EMITTED", len(mods))
PY

heard unexpected_full_emit <<EOF
native emit of both subjects returned 0 with no member refusals; cargo-build of native JSON emission is not yet assembled (JSON member texts, not seed workspace). STOP before inventing a second crate writer.
EOF
exit 4
