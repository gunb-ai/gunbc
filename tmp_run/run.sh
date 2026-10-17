cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
for f in test/claim/runner/runner_jit_listed_registration_witness test/claim/runner/runner_qualification_dispatch_witness test/claim/runner/runner_jit_deregistration_witness test/claim/github_app_registry; do
  echo "== $f"; ./target/release/gunbc run --source-root dag --source-root src --entry dag/$f.dag --claim-run 2>&1 | grep -E "^(PASS|FAIL)" | sed 's/ .*//' | sort | uniq -c
done
echo done
