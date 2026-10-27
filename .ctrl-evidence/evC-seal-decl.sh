#!/bin/bash
# Dispatch C at this head: the seal controls and declaration control that dispatch B's
# 1h wall cut off. No prior section is repeated.
set -u
F() { grep -o '^test fn [a-z_0-9]*' "dag/test/claim/$1.dag" | sed 's/test fn //' | paste -sd, -; }
CB() { FN="${ONLY:-$(F $1)}"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry "dag/test/claim/$1.dag" --functions "$FN" "${@:2}" > /tmp/cb.out 2>&1; rc=$?; grep -E '^(PASS|FAIL|UNOBSERVED)|error:|claim_batch:|passed|failed' /tmp/cb.out | grep -v '^\[pre-entry\]\|typed_cache\|floor-' | cut -c1-500 | head -40; echo "EXIT[$1 ${*:2}]=$rc PASS=$(grep -c '^PASS' /tmp/cb.out) FAIL=$(grep -c '^FAIL' /tmp/cb.out)"; }
STORE=dag/gunbc/deploy_transition_store.dag; DT=dag/gunbc/deploy_transition.dag
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^ D artifacts' | head -10
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
cp $STORE /tmp/store.bak; cp $DT /tmp/dt.bak
for T in TransitionContext TransitionGuard TransitionResidue; do
  echo "##### SEAL CONTROL: sole_constructor stripped from $T"
  if [ -f $STORE ] && grep -q "^type $T sole_constructor {" $STORE; then sed -i "s/^type $T sole_constructor {/type $T {/" $STORE; else sed -i "s/^type $T sole_constructor {/type $T {/" $DT; fi
  grep -n "^type $T" $STORE $DT | head -2
  CB transition_inhibition_store_seal_witness
  cp /tmp/store.bak $STORE; cp /tmp/dt.bak $DT
done
echo "##### DECLARATION CONTROL: transition_inhibition_slots deleted"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='data transition_inhibition_slots: TransitionInhibitionDeclaration = TransitionInhibitionDeclaration {\n  member: "transition-inhibition.json" as NonEmptyStr,\n}\n'
assert s.count(old)==1; s=s.replace(old,''); open(p,'w').write(s)
PY
for spec in "dag/test/claim/deploy_transition_witness.dag:the_record_is_written_where_the_declaration_says" "dag/gunbc/roadmap/roadmap_belt_actuate.dag:belt_transition_admission_for_instance" "dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag:rlm_launch_deployment_receipt_bound"; do
  e=${spec%%:*}; f=${spec##*:}; echo "-- $e ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $e --functions $f > /tmp/decl.out 2>&1; echo "rc=$?"
  grep -E "transition_inhibition_slots" /tmp/decl.out | grep -E "error" | sed 's/ (imported.*//' | sort | uniq -c | head -6
  grep -E "^(PASS|FAIL)" /tmp/decl.out | cut -c1-200 | head -2
done
cp /tmp/store.bak $STORE
echo "##### DONE"
