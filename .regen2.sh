set -u
export CARGO_TARGET_DIR=$PWD/target
cargo build --release --bin gunbc 2>&1 | tail -1
echo "== main_wet"; ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet > /tmp/mw.log 2>&1; grep -a "invalid cast\|resolve failed" /tmp/mw.log | head -40; tail -3 /tmp/mw.log
cp -r src/v1/stage0/src /tmp/stage0_before
for i in 1 2 3 4; do
  echo "== round $i"
  cargo build --release --bin claim_executor 2>&1 | tail -1
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | tail -4
  C=target/stage0-regen-candidate/src/v1/stage0/src
  [ -d "$C" ] || C=$(find target/stage0-regen-candidate -type d -path '*v1/stage0/src' | head -1)
  n=0
  for f in $(cd $C && ls); do cmp -s $C/$f src/v1/stage0/src/$f || { cp $C/$f src/v1/stage0/src/$f; echo "installed $f"; n=$((n+1)); }; done
  echo "installed_count=$n"
  [ $n = 0 ] && break
done
echo "== TAR-B64-BEGIN"
(cd src/v1/stage0/src && for f in *; do cmp -s $f /tmp/stage0_before/$f || echo $f; done) > /tmp/changed.txt; cat /tmp/changed.txt
(cd src/v1/stage0/src && tar cz -T /tmp/changed.txt | base64 -w0); echo
echo "== TAR-B64-END"
bash .nc.sh
