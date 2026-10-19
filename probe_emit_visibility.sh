#!/usr/bin/env bash
# Does the minted-id property show up in the ONE artifact the required path can read -- the emitted
# Rust -- or only inside the resolved tree that a Rust unit test can reach?
#
# Why this question decides the review finding. review 82881 says the test that proves the fix never
# runs in required CI, and DESIGN section 5 wants "green by execution plus a discriminating input that
# goes red". DESIGN also says Rust unit tests are off the merge path. The merge path DOES run claim
# carriers (`test fn` rows over .dag sources). So if the freshness property is observable from emitted
# Rust, a carrier can hold it on the required path and the finding is fixable inside this PR. If it is
# not, then claiming a carrier would be theatre: a row that cannot see the defect stays green under the
# mutation, which is the exact failure §5 calls out -- worse than no row, because it looks like coverage.
#
# METHOD, pre-registered before running. Compile the same two-call fixture on two trees and grep the
# emitted Rust for the minted id strings.
#   repair tree  -> the emitted Rust must contain the span-tagged ids (K@<file>:<start>-<end>)
#   mutant tree  -> the emitted Rust must contain bare "K"/"V" ids and NOT the span-tagged ones
# Verdicts:
#   EMIT_VISIBLE   both hold  -> a carrier CAN discriminate; write one.
#   EMIT_INVISIBLE neither    -> the minted ids never reach emitted Rust; a carrier cannot observe the
#                                defect, and the honest answer to the reviewer is the §5 ceiling,
#                                stated with the rung-drop row that already records it.
#   INCONSISTENT   anything else, including "repair-only strings appear on both" -- which would mean
#                  the emitted file is a seed rather than a product, and must not be read as coverage.
set -u
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export GIT_PAGER=cat PAGER=cat GIT_TERMINAL_PROMPT=0
REPAIR="${1:?repair sha}"
MUT="${2:?mutant sha}"
URL=https://github.com/gunb-ai/gunbc
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
stamp(){ echo "T=$(date -u +%H:%M:%S) $*"; }

cat > /tmp/freshness_probe.dag <<'DAG'
module test.probe.freshness_emit
import std.types { Map }

fn collide() -> Bool {
  let a = empty_map()
  let b = empty_map()
  is_empty(a) && is_empty(b)
}
DAG

probe(){
  local sha="$1" label="$2" d="/tmp/emit_probe_$2"
  rm -rf "$d"; mkdir -p "$d"
  export CARGO_TARGET_DIR="/root/gunbc_target_probe_$2"
  timeout 600 git clone -q "$URL" "$d/repo" || { echo "FATAL clone $label"; return 1; }
  cd "$d/repo" || return 1
  timeout 300 git fetch -q origin "$sha" && timeout 120 git checkout -q "$sha" || { echo "FATAL checkout $label"; return 1; }
  echo "MEASURED_AT $label $sha $(git rev-parse --short HEAD)"
  echo "tree .dag: fresh=$(grep -c fresh_call_variables src/v1/04_infer.dag) namekeyed=$(grep -c 'map_insert(acc, g, type_variable_node(id: g))' src/v1/04_infer.dag)"
  cp /tmp/freshness_probe.dag src/probe_freshness_emit.dag 2>/dev/null || cp /tmp/freshness_probe.dag probe_freshness_emit.dag
  # Emit only. `--emit-rust` writes the generated Rust; that file is the artifact the merge path can read.
  timeout 900 cargo run -q --release --bin gunbc -- compile probe_freshness_emit.dag \
      --source-dir . --output-dir "$d/emit" --target rust > "$d/emit.log" 2>&1
  local rc=$?
  echo "EMIT rc=$rc  files=$(find "$d/emit" -name '*.rs' 2>/dev/null | wc -l)"
  if [ ! -d "$d/emit" ]; then
    # fallback: some builds write next to the source
    find . -newer /tmp/freshness_probe.dag -name '*probe_freshness*' 2>/dev/null | head -3
  fi
  local f; f=$(find "$d" -name '*probe_freshness*.rs' 2>/dev/null | head -1)
  [ -n "$f" ] || f=$(find . -name 'probe_freshness_emit.rs' 2>/dev/null | head -1)
  echo "EMITTED_FILE ${f:-NONE}"
  if [ -n "$f" ]; then
    echo "--- minted id strings in emitted Rust ---"
    grep -oE '"[KV]@[^"]*"|type_variable_node\("[KV]"\)|concat\([^)]*\)' "$f" 2>/dev/null | sort -u | head -8
    echo "SPAN_TAGGED_COUNT $(grep -cE '"[KV]@' "$f" 2>/dev/null || echo 0)"
    echo "BARE_ID_COUNT     $(grep -cE 'type_variable_node\("(K|V)"\)|[[:space:]]"(K|V)"[[:space:]]*[,)]' "$f" 2>/dev/null || echo 0)"
    echo "empty_map mentions: $(grep -c empty_map "$f" 2>/dev/null || echo 0)"
  fi
  tail -5 "$d/emit.log" | sed 's/^/  log| /'
}

stamp "repair probe"
probe "$REPAIR" repair
stamp "mutant probe"
probe "$MUT" mutant
stamp "ALLDONE"
