#!/usr/bin/env bash
# Three measurements that must NOT overlap each other: negctl_seed_checker_v2 mutates the cut repo's own working
# tree per case, and the pre-run checker reads that repo. Sequential, one job, no race (a concurrent pair once
# produced a false 18-BAD reading).
set -uo pipefail
cd /tmp/packet
echo "### gates $(date -Iseconds)"; bash gates_fixture_v2.sh > gates-receipt-v4.txt 2>&1; echo "gates rc=$?"; tail -1 gates-receipt-v4.txt
echo "### negctl $(date -Iseconds)"; bash negctl_seed_checker_v2.sh /var/tmp/freeze2-cut-15695/repo 4a9efe081d8046969eb4e9ef06d219e27ce31113 > NEGCTL_seed_checker_v4.txt 2>&1; echo "negctl rc=$?"; tail -1 NEGCTL_seed_checker_v4.txt
echo "### seed-prerun $(date -Iseconds)"
D=$(mktemp -d /var/tmp/seedprerun-v4.XXXXXX); chmod 700 "$D"; mkdir -p "$D/work"; chmod 700 "$D/work"
EXPECT_FILE=/tmp/packet/prerun_expectations_freeze2_v2.env DIGEST_TOOL=/tmp/packet/probe_block_digest.py \
REPO_DIR=/var/tmp/freeze2-cut-15695/repo FRAME_SCRIPT=/tmp/packet/packet-evidence-freeze2.sh \
timeout 900 bash prerun_checks_freeze2_seed_v2.sh "$D/work" 4a9efe081d8046969eb4e9ef06d219e27ce31113 > "$D/prerun.out" 2>&1
echo "prerun rc=$?"; cp "$D/seed-prerun-receipt.txt" /tmp/packet/seed-prerun-receipt.txt && echo "receipt copied from $D"
tail -2 /tmp/packet/seed-prerun-receipt.txt
echo "### done $(date -Iseconds)"
