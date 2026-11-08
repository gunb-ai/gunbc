#!/usr/bin/env bash
# All six receipts against ONE frame, in one job, in a fixed order. negctl_seed_checker_v2 mutates the cut repo's
# own working tree per case and the pre-run checker reads that repo, so those two must stay sequential and last.
set -uo pipefail
cd /tmp/packet
FRAME=packet-evidence-freeze2.sh
echo "### frame $(sha256sum $FRAME | cut -c1-16) $(wc -l < $FRAME) lines  $(date -Iseconds)"
python3 build/check_accounting.py $FRAME > accounting-receipt-v5.txt 2>&1; echo "accounting rc=$? -> $(tail -1 accounting-receipt-v5.txt)"
# The ordering checker, plus its INVERSION: the v4 frame (which carried the defect) must be refused by it. A
# checker with no failing case beside it proves nothing.
python3 build/check_order.py $FRAME > order-receipt-v5.txt 2>&1; echo "order rc=$? -> $(tail -1 order-receipt-v5.txt)"
python3 build/check_order.py .bak-frame-v5pre > order-receipt-v5-inversion.txt 2>&1
echo "order-inversion rc=$? (MUST be 1) -> $(tail -1 order-receipt-v5-inversion.txt)"
bash gates_fixture_v2.sh > gates-receipt-v5.txt 2>&1;               echo "gates rc=$? -> $(tail -1 gates-receipt-v5.txt)"
bash argv_fixture_v4.sh > argv-receipt-v5.txt 2>&1;                 echo "argv rc=$? -> $(tail -1 argv-receipt-v5.txt)"
bash outcome_fixture_v4.sh > outcome-receipt-v5.txt 2>&1;           echo "outcome rc=$? -> $(tail -1 outcome-receipt-v5.txt)"
# M3: the auditor now resolves every input from its own directory (claims-rtp.log, claims-ce.log are attached
# beside it) and takes the checkout for the declared-path check as an explicit argument instead of a hard-coded
# private path. The cut repo is checked out AT the seed, so it is the right checkout; it is read here and only
# mutated later by negctl, which is why this runs first.
timeout 300 env AUDIT_WORKTREE=/var/tmp/freeze2-cut-15695/repo \
  python3 audit_prereg_vs_v17_rev5.py /tmp/packet/PRE_REG_ROW_15695_freeze2_rev5.md > AUDIT_prereg_rev5_v17.txt 2>&1
echo "audit rc=$? -> $(tail -1 AUDIT_prereg_rev5_v17.txt)"
timeout 600 python3 negctl_auditor_rev5.py > NEGCTL_auditor_rev5.txt 2>&1; echo "audit-ctl rc=$? -> $(tail -1 NEGCTL_auditor_rev5.txt)"
bash recon_fixture_v4.sh > recon-receipt-v5.txt 2>&1;               echo "recon rc=$? -> $(tail -1 recon-receipt-v5.txt)"
echo "### negctl start $(date -Iseconds)"
bash negctl_seed_checker_v2.sh /var/tmp/freeze2-cut-15695/repo 4a9efe081d8046969eb4e9ef06d219e27ce31113 > NEGCTL_seed_checker_v5.txt 2>&1
echo "negctl rc=$? -> $(tail -1 NEGCTL_seed_checker_v5.txt)"
D=$(mktemp -d /var/tmp/seedprerun-v5.XXXXXX); chmod 700 "$D"; mkdir -p "$D/work"; chmod 700 "$D/work"
EXPECT_FILE=/tmp/packet/prerun_expectations_freeze2_v2.env DIGEST_TOOL=/tmp/packet/probe_block_digest.py \
REPO_DIR=/var/tmp/freeze2-cut-15695/repo FRAME_SCRIPT=/tmp/packet/packet-evidence-freeze2.sh \
timeout 900 bash prerun_checks_freeze2_seed_v2.sh "$D/work" 4a9efe081d8046969eb4e9ef06d219e27ce31113 > "$D/prerun.out" 2>&1
echo "prerun rc=$?"; cp "$D/seed-prerun-receipt.txt" /tmp/packet/seed-prerun-receipt.txt && echo "prerun receipt copied"
tail -2 /tmp/packet/seed-prerun-receipt.txt
echo "### ALL DONE $(date -Iseconds)"
