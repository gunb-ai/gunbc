#!/usr/bin/env bash
set -u
B=target/release/claim_batch
fns() { grep -oP '^test fn \K[a-z0-9_]+' "$1" | paste -sd, -; }
run() { "$B" $1 --source-root dag --source-root src/v2 --entry "dag/test/claim/$2" --functions "${3:-$(fns dag/test/claim/$2)}" > .out 2>&1; e=$?
  echo "== $2: PASS=$(grep -c '^PASS ' .out) FAIL=$(grep -c '^FAIL ' .out) exit=$e"; grep -E '^FAIL |panicked|error\[|compile error|error:' .out | cut -c1-300 | head -12; }
LS=dag/gunbc/deploy_transition_lease_store.dag
DT=dag/gunbc/deploy_transition.dag
cp $LS /tmp/ls.bak; cp $DT /tmp/dt.bak
restore() { cp /tmp/ls.bak $LS; cp /tmp/dt.bak $DT; }
trap restore EXIT
echo "##### L0 baseline seal witness (expect 14/0)"
run "" transition_lease_store_seal_witness.dag
echo "##### L1 restore BOTH admitting folds (expect exactly the 3 laundering REDs to fail)"
python3 - <<'PY'
ls='dag/gunbc/deploy_transition_lease_store.dag'; s=open(ls).read()
fold='''
// MUTATION: the deleted laundering fold, restored to prove the three REDs are not vacuous.
fn transition_lease_acquire_of(store: TransitionLeaseStore, outcome: FileHoldAcquireOutcome) -> TransitionLeaseAcquire {
  match outcome {
    FileHoldAcquired { slot_key: _, owner: o, generation: g } =>
      TransitionLeaseAcquired { held: TransitionLeaseHeld { store: store, owner: o, generation: g } }
    _ => transition_lease_acquire_refused(location: transition_lease_location(store: store), outcome: outcome)
  }
}
'''
i=s.index('fn transition_lease_acquire(')
open(ls,'w').write(s[:i]+fold+'\n'+s[i:])
dt='dag/gunbc/deploy_transition.dag'; t=open(dt).read()
fold2='''
// MUTATION: the deleted guard mint over an authored outcome, restored for the same probe.
fn lease_acquisition_of(context: TransitionContext, subject: TransitionSubject, holder: TransitionActor, outcome: TransitionLeaseAcquire) -> LeaseAcquisition {
  match outcome {
    TransitionLeaseAcquired { held: h } => LeaseAcquired(TransitionGuard { context: context, subject: subject, holder: holder, lease: h })
    TransitionLeaseRefused { refusal: r } => lease_acquisition_refused(refusal: r)
  }
}
'''
j=t.index('fn acquire_transition_lease(')
open(dt,'w').write(t[:j]+fold2+'\n'+t[j:])
PY
python3 - <<'PY'
# the probe needs TransitionLeaseAcquire importable from deploy_transition's surface? no: probe imports it from the store module.
print("mutation applied")
PY
run "" transition_lease_store_seal_witness.dag
restore; trap - EXIT
echo "##### L2 receipts on the restored tree"
run " --wet" deploy_transition_wet_witness.dag
run "" transition_inhibition_store_seal_witness.dag
run "" durable_exclusive_hold_nonholder_release_witness.dag
run "" roadmap/roadmap_dashboard_instance_witness.dag declared_dashboard_instance_set_is_the_lookup_authority
