#!/bin/bash
# THREE PHASES, ONE DISPATCH. Fixes the three ways the last two dispatches produced nothing:
#  - the stale-carrier trap: seedgreen/* mirrors were cut at an OLDER base and lack main's new
#    FileVerb::FileWriteBytes match arm, so swapping them into main's family breaks the lib build
#    (E0004). Mirrors therefore come from main (keep/main_*), never from seedgreen, except in the
#    convergence phase where the seed is whatever the emitter itself regenerated.
#  - the probe never fired: it prints only when GUNBC_EMIT_TRACE is set (never set), and the grep
#    filter was 'PROD-UNIT' while the printer says 'PROD leaf=' (filters come from the printer).
#  - rc=1 with NOTHING printed: every phase here dumps the tail of the tool's own log in full.
# NO git checkout / git clean. Restores by cp at the very end.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
build() { cargo build --release -p v1-compiler --bin "$1" 2>&1 | grep -E "^error|error\[|Finished" | tail -3; }
install_main_mirrors() { for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done; }
manifest() { git ls-files -z "$SEED" | xargs -0 sha256sum 2>/dev/null | sort > "$1"; }

echo "############ PHASE 1: TRUE CLEAN-MAIN BASELINE ############"
# My earlier 'clean main also refuses' was an ARTIFACT: that bisect installed main's 04_infer.dag but
# left MY census claim .dag in place, and my claim file calls unify_generics with text:/module_name:,
# parameters main's 5-parameter version does not declare -- an arity mismatch of my own making.
# Clean main is reached by REVERSE-APPLYING my own committed+uncommitted diff (2 small patches), not
# by shipping main's copies of every touched file: that staging dir pushed the sync payload to 53.9 MB
# and the runner's gRPC limit is 50 MB, so the dispatch never even started.
# snapshot first, restore by cp at the end of the phase (never git checkout/clean on the runner)
mkdir -p /tmp/snap
tar -c dag src/v1/04_infer.dag src/v1/04_types.dag src/v1/tests src/v2/test src/v1/tests 2>/dev/null | base64 -w0 > /tmp/snap/mine.tgz.b64
find keep/clean -type f | while read -r f; do rel=${f#keep/clean/}; case "$rel" in *.removed) rm -f "${rel%.removed}";; *) mkdir -p "$(dirname $rel)"; cp "$f" "$rel";; esac; done
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
build claim_executor
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/p1.log 2>&1
echo "P1 rc=$? header='$(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/p1.log | head -1)'"
echo "--- P1 output:"; head -30 /tmp/p1.log | cut -c1-230
cp /tmp/p1.log cleanmain.log
# restore my tree for phase 2: mirrors from my carriers, dag/claim files re-applied from the tar
base64 -d /tmp/snap/mine.tgz.b64 | tar -x 2>/dev/null
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "P1 restored: dagrelation=$(grep -c generic_types_unify src/v1/04_infer.dag) (want 2) claimtext=$(grep -c 'text:' src/v1/tests/claim/generic_identity_census.dag) (want 2)"

echo "############ PHASE 2: ARM B (my dag + main mirrors), then INSTALL-AND-CONVERGE ############"
cp keep/my_04_infer.dag src/v1/04_infer.dag
install_main_mirrors
build claim_executor
manifest /tmp/m0
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/p2r1.log 2>&1
echo "P2 r1 rc=$? header='$(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/p2r1.log | head -1)'"
echo "--- P2 r1 FULL log (nothing filtered):"; head -60 /tmp/p2r1.log | cut -c1-230
manifest /tmp/m1; echo "--- seed files r1 moved:"; comm -3 /tmp/m0 /tmp/m1 | head -12
round() { # $1 tag
  local tag=$1
  [ -d "$CAND" ] || { echo "R$tag: no candidate tree at $CAND"; return 1; }
  local inst=0
  for f in $(ls "$CAND" 2>/dev/null); do
    if [ -f "$SEED/$f" ]; then
      cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; echo "  R$tag installed $f"; inst=$((inst+1)); }
    else echo "  R$tag NEW candidate file (not in seed): $f"; cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); fi
  done
  echo "R$tag installed=$inst"
  [ "$inst" -gt 0 ]
}
i=2; again=0
if round 1; then again=1; fi
while [ "$again" -eq 1 ] && [ $i -le 3 ]; do
  build claim_executor
  manifest /tmp/m$i
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/p2r$i.log 2>&1
  echo "P2 r$i rc=$? header='$(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/p2r$i.log | head -1)'"
  echo "--- P2 r$i full log:"; head -50 /tmp/p2r$i.log | cut -c1-230
  manifest /tmp/mafter$i; echo "--- seed files r$i moved:"; comm -3 /tmp/m$i /tmp/mafter$i | head -12
  if round $i; then again=1; else again=0; fi
  i=$((i+1))
done
echo "P2 after: seed relation=$(grep -c generic_types_unify $SEED/v1_compiler_infer.rs) tvLeaf=$(grep -c binding_has_type_variable_leaf $SEED/v1_compiler_infer.rs) fold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs)"
cp /tmp/p2r1.log p2r1.log 2>/dev/null; cp /tmp/p1.log p1.log 2>/dev/null

echo "############ PHASE 3: PRODUCER PROBE on the converged seed ############"
# Insert the backtrace probe into the CONVERGED v1_compiler_infer.rs at the substitute_generics_apply
# bare-leaf arm. Anchor verified against the shipped seed; Rust will say so if this context moved.
python3 - <<'PY'
import io,sys
seed='src/v1/stage0/src/v1_compiler_infer.rs'
s=open(seed,encoding='utf8').read()
anchor="""                    match v1_rt::map_get(&subst, nm.clone()) {
                        Some(c) => {
                            crate::v1_std_core::preserve_outer_optional_cardinality("""
probe="""                    match v1_rt::map_get(&subst, nm.clone()) {
                        Some(c) => {
                            if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                                let __sh = crate::v1_compiler_infer_types::node_type_shape(
                                    c.clone(),
                                    source_indices.clone(),
                                );
                                if c.name.clone() == "Unit".to_string() {
                                    let mut __o = String::from("PROD leaf=");
                                    __o.push_str(&type_node_label(n.clone(), source_indices.clone()));
                                    __o.push_str(" leaf_span=");
                                    __o.push_str(&n.span.file.clone());
                                    __o.push_str(":");
                                    __o.push_str(&n.span.start.to_string());
                                    __o.push_str("-");
                                    __o.push_str(&n.span.end.to_string());
                                    __o.push_str(" inserted=");
                                    __o.push_str(&__sh);
                                    __o.push_str(" ins_expr=");
                                    __o.push_str(&format!("{:?}", c.expr_data.clone()));
                                    __o.push_str(" ins_inferred=");
                                    __o.push_str(&format!("{:?}", c.inferred.clone()));
                                    __o.push_str("\\n");
                                    __o.push_str(&format!(
                                        "{:?}",
                                        std::backtrace::Backtrace::force_capture()
                                    ));
                                    eprintln!("{}", __o);
                                }
                            }
                            crate::v1_std_core::preserve_outer_optional_cardinality("""
n=s.count(anchor)
if n!=1:
    print("ANCHOR-FAIL count=",n); sys.exit(4)
open(seed,'w',encoding='utf8').write(s.replace(anchor,probe))
print("probe inserted")
PY
echo "probe markers: $(grep -c 'PROD leaf=' $SEED/v1_compiler_infer.rs) backtrace=$(grep -c force_capture $SEED/v1_compiler_infer.rs)"
build gunbc
export GUNBC_EMIT_TRACE=1
RUST_BACKTRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/p3.log 2>&1
echo "P3 self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/p3.log)"
echo "=== PRODUCER ROWS (filter from the printer) ==="
grep -a "PROD leaf=" /tmp/p3.log | head -10 | cut -c1-300
echo "count=$(grep -ac 'PROD leaf=' /tmp/p3.log)"
echo "=== own-crate frames behind them ==="
grep -a -A60 "PROD leaf=" /tmp/p3.log | grep -aoE "v1_compiler[a-z_]*::[a-zA-Z0-9_:]+" | sort | uniq -c | sort -rn | head -20
echo "=== self-host tail (unfiltered, last 25) ==="
tail -25 /tmp/p3.log | cut -c1-220
cp /tmp/p3.log prod_selfhost3.log 2>/dev/null
echo "=== BEGIN-TAR ==="; ( cd $SEED && tar -c v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
install_main_mirrors
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: $(for f in $MIRRORS; do echo -n "$(basename $f)=$(sha256sum $SEED/$f|cut -c1-10) "; done) dag=$(sha256sum src/v1/04_infer.dag|cut -c1-10)"
echo ALL3_DONE
