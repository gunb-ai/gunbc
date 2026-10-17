cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for w in belt_capture_intent_wet_witness attempt_worktree_claim_wet_witness belt_commit_outcome_persist_wet_witness session_activation_ensure_wet_witness; do
  F=$(grep -o '^test fn [a-z_0-9]*' dag/test/claim/roadmap/$w.dag | sed 's/test fn //' | paste -sd,)
  ./target/release/claim_batch --wet --source-root dag --source-root src/v2 --entry dag/test/claim/roadmap/$w.dag --functions $F > /tmp/$w.log 2>&1; echo $w rc=$?; grep -a 'error\|^PASS\|^FAIL' /tmp/$w.log | sort -u | head -14 | cut -c1-300
done
ls -d /tmp/gunbc_*wet_* 2>/dev/null | wc -l
