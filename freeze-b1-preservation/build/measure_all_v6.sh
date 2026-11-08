#!/usr/bin/env bash
# Every v6 receipt, against ONE frame, in one job, in a fixed order, with the cut-repo users serialized.
# Order rationale: the read-only users of /var/tmp/freeze2-cut-15695/repo (auditor, recon, the hermetic harness's
# clone) all run BEFORE negctl_seed_checker_v3, which checks a mutant commit out INTO that repo's working tree.
# Nothing else may be using that repo while it runs. Wall: roughly 18 min hermetic + 26 min negctl + a few.
set -uo pipefail
cd /tmp/packet
FRAME=packet-evidence-freeze2.sh
FS=$(sha256sum $FRAME | cut -d' ' -f1)
echo "### frame $FS  $(wc -l < $FRAME) lines  $(date -Iseconds)"
python3 build/check_accounting.py $FRAME > receipts/ACCOUNTING_v6.txt 2>&1; echo "accounting rc=$? -> $(tail -1 receipts/ACCOUNTING_v6.txt)"
python3 build/check_order.py $FRAME > receipts/ORDER_v6.txt 2>&1;          echo "order rc=$? -> $(tail -1 receipts/ORDER_v6.txt)"
python3 build/check_order.py .bak-frame-v5pre > receipts/ORDER_v6-inversion.txt 2>&1
echo "order-inversion rc=$? (MUST be 1) -> $(tail -1 receipts/ORDER_v6-inversion.txt)"
timeout 400 bash build/cap_kill_fixture_v1.sh $FRAME "$FS" > receipts/CAPKILL_v6.txt 2>&1
echo "cap_kill rc=$? -> $(tail -1 receipts/CAPKILL_v6.txt)"
timeout 200 bash gates_fixture_v2.sh > receipts/GATES_v6.txt 2>&1;          echo "gates rc=$? -> $(tail -1 receipts/GATES_v6.txt)"
timeout 200 bash argv_fixture_v4.sh  > receipts/ARGV_v6.txt 2>&1;           echo "argv  rc=$? -> $(tail -1 receipts/ARGV_v6.txt)"
timeout 200 bash outcome_fixture_v4.sh > receipts/OUTCOME_v6.txt 2>&1;      echo "outcome rc=$? -> $(tail -1 receipts/OUTCOME_v6.txt)"
timeout 300 env AUDIT_WORKTREE=/var/tmp/freeze2-cut-15695/repo \
  python3 audit_prereg_vs_v17_rev5.py /tmp/packet/PRE_REG_ROW_15695_freeze2_rev5.md > receipts/AUDIT_prereg_rev5_v17_v6.txt 2>&1
echo "audit rc=$? -> $(tail -1 receipts/AUDIT_prereg_rev5_v17_v6.txt)"
timeout 600 python3 negctl_auditor_rev5.py > receipts/NEGCTL_auditor_v6.txt 2>&1; echo "audit-ctl rc=$? -> $(tail -1 receipts/NEGCTL_auditor_v6.txt)"
timeout 900 bash recon_fixture_v4.sh > receipts/RECON_v6.txt 2>&1;          echo "recon rc=$? -> $(tail -1 receipts/RECON_v6.txt)"
echo "### hermetic start $(date -Iseconds)"
timeout 2400 bash hermetic_e2e_v2.sh $FRAME "$FS" > receipts/HERMETIC_e2e_v6.txt 2>&1
echo "hermetic rc=$? -> $(tail -1 receipts/HERMETIC_e2e_v6.txt)"
echo "### negctl start $(date -Iseconds)"
timeout 3000 bash negctl_seed_checker_v3.sh /var/tmp/freeze2-cut-15695/repo 4a9efe081d8046969eb4e9ef06d219e27ce31113 > receipts/NEGCTL_seed_checker_v6.txt 2>&1
echo "negctl rc=$? -> $(tail -1 receipts/NEGCTL_seed_checker_v6.txt)"
D=$(mktemp -d /var/tmp/seedprerun-v6.XXXXXX); chmod 700 "$D"; mkdir -p "$D/work"; chmod 700 "$D/work"
EXPECT_FILE=/tmp/packet/prerun_expectations_freeze2_v2.env DIGEST_TOOL=/tmp/packet/probe_block_digest.py \
REPO_DIR=/var/tmp/freeze2-cut-15695/repo FRAME_SCRIPT=/tmp/packet/packet-evidence-freeze2.sh \
timeout 900 bash prerun_checks_freeze2_seed_v2.sh "$D/work" 4a9efe081d8046969eb4e9ef06d219e27ce31113 > "$D/prerun.out" 2>&1
echo "prerun rc=$?"; cp "$D/work/seed-prerun-receipt.txt" /tmp/packet/receipts/seed-prerun-receipt-v6.txt && tail -2 /tmp/packet/receipts/seed-prerun-receipt-v6.txt
echo "### ALL DONE $(date -Iseconds)"
