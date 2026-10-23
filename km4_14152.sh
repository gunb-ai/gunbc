#!/bin/bash
# KERNEL-EMPTY-LEAF ROOT CAUSE, round 4 -- the decisive minimal arm. Rounds 1-3 failed on
# my own seed-patch errors (anchor miss; E0434 capture; E0282/E0599 in the tree walk), so
# this round instruments ONE function instead of walking trees.
# v1.std.core authored_name_at decides the leaf name that node_type_shape prints:
#   Some(span) => if source_indices has span.file -> slice the text (and an EMPTY slice is
#                 returned as "" deliberately);
#                 else if file starts "<kernel:" -> the name embedded in the span FILE;
#                 else -> node.name
#   None       -> (fallback)
# So Primitive() in a printed shape can mean either
#  (a) the leaf HAS .name but its ident_span is a zero-width/degenerate span inside a file
#      that IS in source_indices -> authored_name_at prefers the empty slice over .name.
#      Fix = fall back to .name; then the leaf is named again and (c) handles the row.
#  (b) the leaf genuinely has name:"" (v1.compiler.types
#      missing_kernel_container_profile_type) -> an ERROR-shaped node carried as a type; it
#      must REFUSE, and a nameless-leaf acceptance rule would wrongly accept it.
# The wrapper below distinguishes them by printing, for every LEAF whose printed name is
# empty, node.name + ident_span + whether the span's file is in source_indices.
# NO git checkout / git clean; mirrors restored by cp from carriers.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16) (want 36 / 5e9c37585fe088ac)"
python3 - <<'PY'
p="src/v1/stage0/src/v1_std_core.rs"
s=open(p,encoding='utf-8').read()
old="pub fn authored_name_at(\n    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,\n    node: Rc<Node>,\n) -> String {\n"
assert s.count(old)==1, "anchor=%d" % s.count(old)
wrap = """pub fn authored_name_at(
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    node: Rc<Node>,
) -> String {
    let __r = authored_name_at_inner(source_indices.clone(), node.clone());
    if std::env::var("GUNBC_EMIT_TRACE").is_ok()
        && __r == ""
        && node.children.is_empty()
        && node.connective == Connective::NoConnective
    {
        let in_table = node
            .ident_span
            .clone()
            .map(|s| v1_rt::map_get(&source_indices, s.file.clone()).is_some())
            .unwrap_or(false);
        eprintln!(
            "AN printed=\\\"\\\" node_name={:?} ident_span={} file={:?} start={:?} end={:?} in_source_table={} inferred={:?} expr_data={:?} decl={}",
            node.name.clone(),
            if node.ident_span.is_some() { "Some" } else { "NONE" },
            node.ident_span.clone().map(|s| s.file.clone()),
            node.ident_span.clone().map(|s| s.start),
            node.ident_span.clone().map(|s| s.end),
            in_table,
            node.inferred.clone(),
            (*node.expr_data.clone()).clone(),
            match node.declaration.clone() { Some(d) => format!("{}::{}", d.module_path, d.decl_name), None => "NONE".to_string() }
        );
    }
    __r
}

fn authored_name_at_inner(
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    node: Rc<Node>,
) -> String {
"""
s=s.replace(old, wrap)
open(p,'w',encoding='utf-8').write(s)
print("instrumented authored_name_at")
PY
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/km4.log 2>&1
echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/km4.log) AN=$(grep -ac '^AN printed' /tmp/km4.log)"
echo "--- distinct AN shapes (node_name / ident_span / in_table):"
grep "^AN printed" /tmp/km4.log | sed 's/inferred=.*//' | sort | uniq -c | sort -rn | head -14
echo "--- AN lines that are on a fold row (preceded within 40 lines by a bound-as refusal):"
grep -B2 "bound as" /tmp/km4.log | grep -c "^AN printed" || true
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo KM4_DONE
