set -u
git checkout -q f680b955a1f20e53c883a723929d2352f075dcc6 -- . 2>&1 | tail -1
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
mkdir -p /sys/fs/cgroup/gunbcrun && echo +memory > /sys/fs/cgroup/cgroup.subtree_control && echo 21474836480 > /sys/fs/cgroup/gunbcrun/memory.max && echo $BASHPID > /sys/fs/cgroup/gunbcrun/cgroup.procs
git rev-parse HEAD; git diff --stat f680b955a1f20e53c883a723929d2352f075dcc6 -- dag src/v2 | tail -1
./target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/live_deploy_unit_emission_oracle_witness_test.dag --functions the_approval_broker_unit_would_install_these_bytes 2>&1 | grep -E "^(PASS|FAIL)|error"
