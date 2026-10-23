#!/bin/bash
# Why does regen refuse with 8 hard diagnostics at the post-merge tree, when 2 are the known
# 02_parse same-text pair and my 14 call-shape ones are fixed? Print EVERY diagnostic verbatim.
# Mid-merge tree: the two conflicting mirrors carry our side verbatim (my carriers); the other two
# auto-merged. NO git checkout / git clean.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done
echo "dag: relation=$(grep -c generic_types_unify src/v1/04_infer.dag) formalGuard=$(grep -c binding_is_unsettled_formal_annotation src/v1/04_infer.dag) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/04_infer.dag) fold=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep -E "^error|warning: unused|Finished" | tail -2
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/r4.log 2>&1
echo "REGEN rc=$?"
echo "=== EVERY line of the refusal, verbatim (no filtering) ==="
awk '/refused: v2 self-compile/,0' /tmp/r4.log | head -60 | cut -c1-230
echo "=== diagnostic-shape census ==="
grep -oE "^[a-z ]+: (expected|no parameter|missing|argument|unknown)" /tmp/r4.log | sort | uniq -c
echo "=== do any name MY symbols? ==="
for s in generic_types_unify generic_head_matches_application generic_alias_unfold generic_side_is_bare_text generic_type_node_name generic_type_head_of_name generic_types_are_the_text_pair binding_is_unsettled_formal_annotation binding_has_type_variable_leaf; do c=$(grep -c "$s" /tmp/r4.log); [ "$c" != "0" ] && echo "$s -> $c"; done
echo "(end mine)"
echo "=== the two known same-text rows ==="
grep -c "Container(FreeMonoid,Primitive(element))" /tmp/r4.log
cp /tmp/r4.log r4.log
for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done
echo "restored: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12)"; done)"
echo REGEN4_DONE
