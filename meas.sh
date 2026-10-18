cd "$(git rev-parse --show-toplevel)"
M=src/v1/stage0/src/v1_compiler_infer.rs
export CARGO_TARGET_DIR=$PWD/target
cp $M /tmp/mirror_new.rs
run() { # $1 = tag
  cargo build --release --bin gunbc 2>&1 | tail -1
  ( time timeout 2700 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_$1.out 2>&1; echo "rc_$1=$?" ) 2>&1 | grep 'rc_\|real'
}
git show HEAD:$M > $M
run base
cp /tmp/mirror_new.rs $M
run new
wc -l /tmp/sh_base.out /tmp/sh_new.out
echo ==TAILBASE; tail -25 /tmp/sh_base.out | cut -c1-220
echo ==DIFF
diff <(grep -v 'ms\|secs\|elapsed\|real\|/tmp/' /tmp/sh_base.out) <(grep -v 'ms\|secs\|elapsed\|real\|/tmp/' /tmp/sh_new.out) | head -40 | cut -c1-220
echo ==DIFFEND
