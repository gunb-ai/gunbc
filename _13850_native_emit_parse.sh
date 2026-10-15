#!/usr/bin/env bash
# Pin-checked gen-one emit of source_root_eval + compile_cli, then the expression_bodied label.
set -euo pipefail
PIN="${EXPECTED_PIN:?}"
git fetch --depth=1 origin "$PIN"
git checkout --force "$PIN"
HEAD=$(git rev-parse HEAD)
echo NATIVE_DISPATCH_PIN=$HEAD
test "$HEAD" = "$PIN"
export GITHUB_SHA="$PIN"
export GUNBC_NATIVE_PRODUCT_REUSE=fresh
export TMPDIR="${TMPDIR:-/tmp}"
export CARGO_TARGET_DIR="$PWD/target-private-$PIN"
mkdir -p "$CARGO_TARGET_DIR" /sys/fs/cgroup/tdc
echo "+memory" >/sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
echo 64424509440 >/sys/fs/cgroup/tdc/memory.max
echo 0 >/sys/fs/cgroup/tdc/memory.swap.max
echo $$ >/sys/fs/cgroup/tdc/cgroup.procs
cargo build --release -p v1-compiler --bin gunbc
GUNBC="$CARGO_TARGET_DIR/release/gunbc"
test -x "$GUNBC"

classify_first() {
  local entry="$1" err="$2" out="$3" rc="$4"
  echo FIRST_REFUSAL_ENTRY=$entry RC=$rc
  if grep -qE "error(\[|:).*unreachable call|error: could not compile|error\[E" "$err" 2>/dev/null; then
    echo FIRST_REFUSAL_STAGE=rustc
  elif [ "$rc" != 0 ]; then
    echo FIRST_REFUSAL_STAGE=emission
  else
    echo FIRST_REFUSAL_STAGE=none
  fi
  echo FIRST_REFUSAL_VERBATIM_BEGIN
  if [ -s "$err" ]; then head -c 8000 "$err"; echo; fi
  if [ -s "$out" ]; then
    python3 -c "
import json,sys
p=sys.argv[1]
try:
    d=json.load(open(p))
except Exception as e:
    print('stdout_not_json', e); sys.exit(0)
print('json_entry', d.get('entry'))
mods=d.get('modules') or d.get('members') or []
print('n', len(mods) if isinstance(mods, list) else type(mods))
if isinstance(mods, list):
    for m in mods:
        if isinstance(m, dict) and m.get('refused'):
            print('FIRST_MEMBER_REFUSED', m.get('module'), m.get('refused'))
            break
" "$out"
  fi
  echo FIRST_REFUSAL_VERBATIM_END
}

echo NATIVE_STAGE=v2_native_cli
set +e
"$GUNBC" test //gunbc/instruments:v2-native-cli 2>&1 | tee /tmp/v2-native-cli.log
CLI_RC=${PIPESTATUS[0]}
set -e
echo V2_NATIVE_CLI_RC=$CLI_RC
GEN_ONE=$(sed -n "s/.*generation_one_executable=//p" /tmp/v2-native-cli.log | tail -1)
echo GEN_ONE=$GEN_ONE
if [ "$CLI_RC" != 0 ] || [ -z "$GEN_ONE" ] || [ ! -x "$GEN_ONE" ]; then
  echo FIRST_REFUSAL_STAGE=emission
  echo FIRST_REFUSAL_ENTRY=v2-native-cli
  echo FIRST_REFUSAL_VERBATIM_BEGIN
  grep -E "REFUSAL|error|duplicate field|cause=" /tmp/v2-native-cli.log | tail -n 40
  echo FIRST_REFUSAL_VERBATIM_END
else
  mkdir -p /tmp/gen-one-emit
  for entry in v2.compiler.source_root_eval v2.cli.compile_cli; do
    out=/tmp/gen-one-emit/${entry}.json
    err=/tmp/gen-one-emit/${entry}.stderr
    set +e
    "$GEN_ONE" emit --entry "$entry" --source-root dag --source-root src/v2 >"$out" 2>"$err"
    rc=$?
    set -e
    classify_first "$entry" "$err" "$out" "$rc"
    if [ "$rc" = 0 ]; then
      echo EMIT_PASSED=$entry
    fi
  done
fi

echo NATIVE_STAGE=parse_label
set +e
"$GUNBC" test //v2/test/claim/parse/expression_bodied_fn_decl_parse:all
PARSE_RC=$?
set -e
echo PARSE_LABEL_RC=$PARSE_RC
echo NATIVE_DISPATCH_DONE
