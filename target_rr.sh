CG=/sys/fs/cgroup/gunbc-probe; S=""; command -v sudo >/dev/null && S=sudo
echo "+memory" | $S tee /sys/fs/cgroup/cgroup.subtree_control >/dev/null; $S mkdir -p $CG; echo 23622320128 | $S tee $CG/memory.max >/dev/null; echo $$ | $S tee $CG/cgroup.procs >/dev/null
git stash push -q -- src/v1/stage0/src/cli_run/required_floor_runner.rs
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b.txt 2>&1; echo BUILD=$?; tail -3 /tmp/b.txt
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rg.txt 2>&1; echo REGEN=$?; tail -5 /tmp/rg.txt | cut -c1-300
C=target/stage0-regen-candidate/src
for f in $(cd $C && ls); do cmp -s $C/$f src/v1/stage0/src/$f || echo "DIFFERS $f"; done | head
echo "@@B64BEGIN"; base64 -w0 $C/v1_compiler_expected_red_roster_join.rs; echo; echo "@@B64END"
