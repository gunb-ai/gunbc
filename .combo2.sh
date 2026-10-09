SHA=$1
git fetch -q --depth=1 origin $SHA && git -c advice.detachedHead=false checkout -q -f $SHA || { echo FETCHFAIL; exit 3; }
echo "rev=$(git rev-parse HEAD)"
mkdir -p /sys/fs/cgroup/swr && echo 21474836480 > /sys/fs/cgroup/swr/memory.max && echo $BASHPID > /sys/fs/cgroup/swr/cgroup.procs
round() {
  cargo build --release -q -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | tail -3
  rm -rf target/stage0-regen-candidate
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen.txt 2>&1; echo "REGEN$1 rc=$?"; tail -3 /tmp/regen.txt | cut -c1-200
  n=0; for f in target/stage0-regen-candidate/src/*.rs; do b=$(basename $f); [ "$b" = std_measure.rs ] && continue; if ! cmp -s $f src/v1/stage0/src/$b; then echo "DRIFT$1 $b"; cp $f src/v1/stage0/src/$b; n=$((n+1)); fi; done; echo "DRIFTCOUNT$1=$n"
  target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/docs_projection_gate.dag --function regen > /tmp/docs.txt 2>&1; echo "DOCS$1 rc=$?"; tail -2 /tmp/docs.txt | cut -c1-200
}
round 1
round 2
git status --short | head
cargo build --release -q -p v1-compiler --bin gunbc 2>&1 | tail -3
printf 'module test.claim.zz_htc_fixture\nimport std.types { Char }\nimport std.process { ProcessExit, ExitSuccess }\nfn is_a(c: Char) -> Bool { true }\nfn probe(s: String) -> Bool { any(xs: s, predicate: fn(c) { is_a(c: c) }) }\nfn main_probe() -> ProcessExit { ExitSuccess }\n' > dag/test/claim/zz_htc_fixture.dag
GITHUB_SHA=$1 target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/zz_htc_fixture.dag --function main_probe > /tmp/f.txt 2>&1; echo "FIX rc=$?"; grep -aE "error|warning|mismatch|inhabit" /tmp/f.txt | grep -v pre-entry | cut -c1-500 | head -12
rm dag/test/claim/zz_htc_fixture.dag
echo "PATCH $(git diff | gzip -9c | base64 -w0)"
