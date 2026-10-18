cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
D=src/v1/stage0/src
NEW=$(git rev-parse HEAD)
for which in base new; do
  if [ $which = base ]; then git checkout origin/main -- $D; else git checkout $NEW -- $D; fi
  /opt/cargo/bin/cargo build --release --bin gunbc 2>&1 | tail -1
  ( time timeout 2700 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_$which.out 2>&1; echo "rc_$which=$?" ) 2>&1 | grep 'rc_\|real'
  grep -a 'self-host v1->v2' /tmp/sh_$which.out | cut -c1-200
done
git checkout $NEW -- $D
echo ==DIFF
diff <(grep -av 'ms\|secs\|elapsed\|real\|/tmp/\|seed=' /tmp/sh_base.out) <(grep -av 'ms\|secs\|elapsed\|real\|/tmp/\|seed=' /tmp/sh_new.out) | head -40 | cut -c1-220
echo ==DIFFEND
