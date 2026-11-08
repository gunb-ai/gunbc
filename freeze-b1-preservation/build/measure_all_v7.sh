#!/usr/bin/env bash
# Every v7 receipt, against ONE frame (the v7 head), in one job, in a fixed order, with the cut-repo users
# serialized. Order rationale (unchanged from v6): the read-only users of /var/tmp/freeze2-cut-15695/repo
# (auditor, recon, the hermetic harness's clone) all run BEFORE negctl_seed_checker_v3, which checks a mutant
# commit out INTO that repo's working tree. Nothing else may use that repo while it runs.
# v7 additions: static_v7 (the v7-only rules) with its own inversions, the cap-kill fixture extended with the
# TERM-ignoring case and its inversion, and the hermetic suite grown from 9 cases to 15 -- four signal cases,
# the TERM-ignoring local operation at the frozen caps, and the synthetic child-exit-125 label case.
set -uo pipefail
cd /tmp/packet
FRAME=packet-evidence-freeze2.sh
FS=$(sha256sum $FRAME | cut -d' ' -f1)
echo "### frame $FS  $(wc -l < $FRAME) lines  $(date -Iseconds)"
python3 build/check_accounting.py $FRAME > receipts/ACCOUNTING_v7.txt 2>&1; echo "accounting rc=$? -> $(tail -1 receipts/ACCOUNTING_v7.txt)"
python3 build/check_order.py $FRAME > receipts/ORDER_v7.txt 2>&1;           echo "order rc=$? -> $(tail -1 receipts/ORDER_v7.txt)"
python3 build/check_order.py .bak-frame-v5pre > receipts/ORDER_v7-inversion.txt 2>&1
echo "order-inversion rc=$? (MUST be 1) -> $(tail -1 receipts/ORDER_v7-inversion.txt)"
python3 build/static_v7.py $FRAME > receipts/STATIC_v7.txt 2>&1;            echo "static rc=$? -> $(tail -1 receipts/STATIC_v7.txt)"
# Static controls over the frame, and the inversions that prove they are not decoration.  Each mutant is generated from
# the frame under test (build/mk_static_mutants.py refuses if an anchor has moved), and MUST be refused by exactly the
# rule that names the defect it reintroduces.  An inversion that is NOT refused means the corresponding claim in the
# packet is an assertion about a string, not a checked property.
python3 build/mk_static_mutants.py || { echo "MUTANT GENERATION FAILED -- the frame's anchors moved"; exit 1; }
INV_OK=0; INV_BAD=0
inv(){ # inv <label> <file> : static_v7 must refuse this file with rc 1
  python3 build/static_v7.py "$2" 4161 39 > "receipts/STATIC_v7-inv-$1.txt" 2>&1
  local rc=$?
  if [ "$rc" = 1 ]; then INV_OK=$((INV_OK+1))
    echo "inversion $1 ($2): refused as required -- $(grep -c '^BAD' "receipts/STATIC_v7-inv-$1.txt") rule(s) named"
  else INV_BAD=$((INV_BAD+1)); echo "inversion $1 ($2): NOT REFUSED (rc=$rc) -- the static control is BLIND"
  fi
}
inv rc_cause_site_removed build/mutant_rc_cause_site.sh
inv producer_cleared     build/mutant_origin_cleared.sh
inv doubled_prefix       build/mutant_stutter.sh
inv superseded_v6_frame  stage6-kept/packet-evidence-freeze2-stage6.sh
inv stage2_no_channel    .bak-frame-v7stage2
inv stage4_wrapper_ladder .bak-frame-v7stage4
echo "STATIC INVERSIONS: OK=$INV_OK BAD=$INV_BAD"
if [ "$INV_BAD" -gt 0 ]; then echo "SWEEP ABORTED: a static control failed to refuse its own inversion -- nothing measured after this point would be trustworthy"; exit 1; fi
timeout 600 bash build/cap_kill_fixture_v1.sh $FRAME "$FS" > receipts/CAPKILL_v7.txt 2>&1
echo "cap_kill rc=$? -> $(tail -1 receipts/CAPKILL_v7.txt)"
timeout 200 bash gates_fixture_v2.sh "$FRAME" "$FS" > receipts/GATES_v7.txt 2>&1;          echo "gates rc=$? -> $(tail -1 receipts/GATES_v7.txt)"
timeout 200 bash argv_fixture_v4.sh  > receipts/ARGV_v7.txt 2>&1;           echo "argv  rc=$? -> $(tail -1 receipts/ARGV_v7.txt)"
timeout 200 bash outcome_fixture_v4.sh > receipts/OUTCOME_v7.txt 2>&1;      echo "outcome rc=$? -> $(tail -1 receipts/OUTCOME_v7.txt)"
timeout 300 env AUDIT_WORKTREE=/var/tmp/freeze2-cut-15695/repo \
  python3 audit_prereg_vs_v17_rev5.py /tmp/packet/PRE_REG_ROW_15695_freeze2_rev5.md > receipts/AUDIT_prereg_rev5_v17_v7.txt 2>&1
echo "audit rc=$? -> $(tail -1 receipts/AUDIT_prereg_rev5_v17_v7.txt)"
timeout 600 python3 negctl_auditor_rev5.py > receipts/NEGCTL_auditor_v7.txt 2>&1; echo "audit-ctl rc=$? -> $(tail -1 receipts/NEGCTL_auditor_v7.txt)"
timeout 900 bash recon_fixture_v4.sh > receipts/RECON_v7.txt 2>&1;          echo "recon rc=$? -> $(tail -1 receipts/RECON_v7.txt)"
echo "### hermetic start $(date -Iseconds)"
timeout 4200 bash hermetic_e2e_v3.sh $FRAME "$FS" > receipts/HERMETIC_e2e_v7.txt 2>&1
echo "hermetic rc=$? -> $(tail -1 receipts/HERMETIC_e2e_v7.txt)"
echo "### negctl start $(date -Iseconds)"
timeout 3000 bash negctl_seed_checker_v3.sh /var/tmp/freeze2-cut-15695/repo 4a9efe081d8046969eb4e9ef06d219e27ce31113 > receipts/NEGCTL_seed_checker_v7.txt 2>&1
echo "negctl rc=$? -> $(tail -1 receipts/NEGCTL_seed_checker_v7.txt)"
D=$(mktemp -d /var/tmp/seedprerun-v7.XXXXXX); chmod 700 "$D"; mkdir -p "$D/work"; chmod 700 "$D/work"
EXPECT_FILE=/tmp/packet/prerun_expectations_freeze2_v2.env DIGEST_TOOL=/tmp/packet/probe_block_digest.py \
REPO_DIR=/var/tmp/freeze2-cut-15695/repo FRAME_SCRIPT=/tmp/packet/packet-evidence-freeze2.sh \
timeout 900 bash prerun_checks_freeze2_seed_v2.sh "$D/work" 4a9efe081d8046969eb4e9ef06d219e27ce31113 > "$D/prerun.out" 2>&1
echo "prerun rc=$?"; cp "$D/work/seed-prerun-receipt.txt" /tmp/packet/receipts/seed-prerun-receipt-v7.txt && tail -2 /tmp/packet/receipts/seed-prerun-receipt-v7.txt
bash build/compose_chain_v2.sh > receipts/CHAIN_v7.txt 2>&1; echo "chain rc=$? -> $(tail -1 receipts/CHAIN_v7.txt)"
echo "### ALL DONE $(date -Iseconds)"
