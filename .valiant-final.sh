set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
install_candidate() {
  find "$1" -name '*.rs' | while read -r f; do
    b=$(basename "$f")
    case "$f" in
      */src/v1/stage0/src/bin/*) cp -f "$f" src/v1/stage0/src/bin/$b ;;
      *) cp -f "$f" src/v1/stage0/src/$b ;;
    esac
  done
}
echo "########## ROUND 1 (main mirrors + merged corpus incl. deliverables) ##########"
cp -f .valiant-main-mirrors/*.rs src/v1/stage0/src/
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b0.log 2>&1 || { echo BUILDFAIL; grep -E "^error" -A6 /tmp/b0.log | head -40; exit 1; }
rm -rf target/rc1
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/rc1 --regen-receipt target/r1.receipt.txt \
  --source-root dag --source-root src/v2 > /tmp/rc1.log 2>&1
echo "r1_regen_exit=$?"
sed 's/^[[0-9;]*//g; s/^.*UTC //' /tmp/rc1.log > /tmp/rc1.clean
grep -E "adjudicated|drift|hard diagnostic|admission" /tmp/rc1.clean | head -4
echo "r1 effect_lines=$(grep -c 'effect summary incomplete' /tmp/rc1.clean)"
install_candidate target/rc1
echo "keyed arm in installed emitter=$(grep -c 'node_is_keyed_collection(' src/v1/stage0/src/v1_compiler_emit_rust.rs)"
echo "########## FINAL UNSCAFFOLDED std-core pass (the real measurement) ##########"
for f in std_algebra.rs std_types.rs std_computation.rs; do cmp -s src/v1/stage0/src/$f .valiant-scaffold/$f && echo "SCAFFOLD BYTE PRESENT: $f"; done
cargo build --release -p v1-stage0-std-core > /tmp/stdcore.log 2>&1
N=$(grep -cE '^error' /tmp/stdcore.log)
echo "STDCORE_ERRORS=$N"
grep -E "^error" -A9 /tmp/stdcore.log | head -150
if [ "$N" != "0" ]; then
  grep -E "^ +--> src/v1/stage0/src/.*:[0-9]+" /tmp/stdcore.log | sed 's/^ *--> //' | sort -u > /tmp/sites.txt
  echo "distinct sites: $(wc -l < /tmp/sites.txt)"
  while IFS=: read -r f l c; do
    echo "##### $f:$l"
    awk -v L="$l" 'NR>=L-12 && NR<=L+1 {printf "%6d %s\n", NR, $0}' "$f" | grep -E "FinitelySupportedFunction|fn |Rc<im::HashMap|closure|\|\w+: " | head -6
  done < /tmp/sites.txt
fi
