set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
export RUST_BACKTRACE=1
S3="std_algebra.rs std_types.rs std_computation.rs"

install_candidate() {  # $1 = candidate dir
  # install as ONE complete set: every .rs in the candidate, mapped to src/v1/stage0/src/
  find "$1" -name '*.rs' | while read -r f; do
    b=$(basename "$f")
    case "$f" in
      */src/v1/stage0/src/bin/*) cp -f "$f" src/v1/stage0/src/bin/$b ;;
      *) cp -f "$f" src/v1/stage0/src/$b ;;
    esac
  done
}
overlay_scaffold() { for f in $S3; do cp -f .valiant-scaffold/$f src/v1/stage0/src/$f; done; }

regen() { # $1=canddir $2=receipt $3=log
  rm -rf "$1"
  cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
    --regen-candidate-dir "$1" --regen-receipt "$2" \
    --source-root dag --source-root src/v2 > "$3" 2>&1
  echo "regen_exit=$?"
  sed 's/^[[0-9;]*//g; s/^.*UTC //' "$3" > "$3.clean"
}

echo "########## BASELINE: install main mirrors as one set, scaffold the 3 std-core files ##########"
cp -f .valiant-main-mirrors/*.rs src/v1/stage0/src/
overlay_scaffold
for f in $S3; do cp -f .valiant-main-mirrors/$f src/v1/stage0/src/$f; done   # baseline std-core = main's (green)
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b0.log 2>&1
echo "baseline build_exit=$? errors=$(grep -cE '^error' /tmp/b0.log)"
grep -E "^error" -A6 /tmp/b0.log | head -30

############ ROUND 1 ############
echo "########## ROUND 1 regen (main mirrors emit) ##########"
regen target/rc1 target/r1.receipt.txt /tmp/rc1.log
grep -E "adjudicated|refused|drift|first_generation|equal|PASS|FAIL" /tmp/rc1.log.clean | grep -vE "^\[" | head -6
echo "r1 effect_lines=$(grep -c 'effect summary incomplete' /tmp/rc1.log.clean)"
grep "effect summary incomplete" /tmp/rc1.log.clean | head -4
install_candidate target/rc1
echo "r1 installed; keyed arm present=$(grep -c 'node_is_keyed_collection(' src/v1/stage0/src/v1_compiler_emit_rust.rs)"

############ ROUND 2 ############
echo "########## ROUND 2 (rebuild from r1 candidate) ##########"
if ! cargo build --release -p v1-compiler --bin claim_executor > /tmp/b2.log 2>&1; then
  echo "r2 link errors=$(grep -cE '^error' /tmp/b2.log) -> overlay scaffold (iteration only)"
  grep -E "^error" -A6 /tmp/b2.log | head -40
  overlay_scaffold
  cargo build --release -p v1-compiler --bin claim_executor > /tmp/b2b.log 2>&1 || { echo "R2 BUILD FAILED"; grep -E "^error" -A8 /tmp/b2b.log | head -40; }
fi
regen target/rc2 target/r2.receipt.txt /tmp/rc2.log
grep -E "adjudicated|refused|drift|first_generation|equal|PASS|FAIL" /tmp/rc2.log.clean | grep -vE "^\[" | head -8
echo "r2 effect_lines=$(grep -c 'effect summary incomplete' /tmp/rc2.log.clean)"
grep "effect summary incomplete" /tmp/rc2.log.clean | head -6
echo "=== drift r1-installed vs r2 candidate:"
DIFFOUT=$(for f in $(ls target/rc2/*.rs 2>/dev/null); do b=$(basename $f); cmp -s src/v1/stage0/src/$b $f || echo $b; done; find target/rc2/src -name '*.rs' 2>/dev/null | while read -r f; do b=$(basename $f); cmp -s src/v1/stage0/src/$b $f || echo $b; done | sort -u)
echo "$DIFFOUT" | grep -v "^$" | sed 's/^/drift: /'
NDRIFT=$(echo "$DIFFOUT" | grep -vc "^$")
echo "NDRIFT=$NDRIFT"
if [ "$NDRIFT" != "0" ]; then
  echo "########## ROUND 3 (rebuild from r2 candidate) ##########"
  install_candidate target/rc2
  if ! cargo build --release -p v1-compiler --bin claim_executor > /tmp/b3.log 2>&1; then
    echo "r3 link errors=$(grep -cE '^error' /tmp/b3.log) -> overlay scaffold"
    grep -E "^error" -A6 /tmp/b3.log | head -40
    overlay_scaffold
    cargo build --release -p v1-compiler --bin claim_executor > /tmp/b3b.log 2>&1 || { echo "R3 BUILD FAILED"; grep -E "^error" -A8 /tmp/b3b.log | head -40; }
  fi
  regen target/rc3 target/r3.receipt.txt /tmp/rc3.log
  grep -E "adjudicated|refused|drift|first_generation|equal|PASS|FAIL" /tmp/rc3.log.clean | grep -vE "^\[" | head -8
  echo "r3 effect_lines=$(grep -c 'effect summary incomplete' /tmp/rc3.log.clean)"
  grep "effect summary incomplete" /tmp/rc3.log.clean | head -6
  echo "=== drift r2-installed vs r3 candidate:"
  D3=$(for f in $(ls target/rc3/*.rs 2>/dev/null); do b=$(basename $f); cmp -s src/v1/stage0/src/$b $f || echo $b; done; find target/rc3/src -name '*.rs' 2>/dev/null | while read -r f; do b=$(basename $f); cmp -s src/v1/stage0/src/$b $f || echo $b; done | sort -u)
  echo "$D3" | grep -v "^$" | sed 's/^/drift3: /'
  echo "NDRIFT3=$(echo "$D3" | grep -vc '^$')"
fi

echo "########## FINAL: unscaffolded std-core pass on the last candidate ##########"
echo "scaffold contamination check (must be empty):"
for f in $S3; do [ -f .valiant-scaffold/$f ] && ! cmp -s src/v1/stage0/src/$f .valiant-scaffold/$f || echo "  SCAFFOLD-BYTE $f identical-to-scaffold"; done
cargo build --release -p v1-stage0-std-core > /tmp/stdcore.log 2>&1
echo "FINAL_STDCORE_ERRORS=$(grep -cE '^error' /tmp/stdcore.log)"
grep -E "^error" -A7 /tmp/stdcore.log | head -120
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep -cE '^error' | xargs echo FINAL_COMPILER_BUILD_ERRORS=
