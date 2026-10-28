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
# CONTROL: strip ALL six of my uncommitted deliverable files -> pure merged corpus (main merge only)
git checkout HEAD -- dag/std/algebra.dag dag/std/coercion.dag dag/gunbc/structural_realization_bindings.dag dag/test/claim/map_annotation_resolves_by_declaration_witness.dag dag/gunbc/recurring_failure_mode/one_name_two_representations_kernel_map_and_finitely_supported_function.dag src/v1/05_emit_rust.dag
echo "control corpus: deliverables stripped=$(git status --porcelain -- dag src/v1 | wc -l) dirty files remain"
cp -f .valiant-main-mirrors/*.rs src/v1/stage0/src/
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b0.log 2>&1 || { echo BUILDFAIL; grep -E "^error" -A6 /tmp/b0.log | head -40; exit 1; }
rm -rf target/ccc1
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/ccc1 --regen-receipt target/ccc1.receipt.txt \
  --source-root dag --source-root src/v2 > /tmp/ccc1.log 2>&1
echo "r1_regen_exit=$?"
sed 's/^[[0-9;]*//g; s/^.*UTC //' /tmp/ccc1.log > /tmp/ccc1.clean
grep -E "adjudicated|drift|hard diagnostic" /tmp/ccc1.clean | head -4
install_candidate target/ccc1
echo "=== ROUND 2 rebuild + regen (pure merged corpus):"
if ! cargo build --release -p v1-compiler --bin claim_executor > /tmp/b2.log 2>&1; then
  echo "r2 link errors=$(grep -cE '^error' /tmp/b2.log)"
  grep -E "^error" -A6 /tmp/b2.log | head -30
  cp -f .valiant-scaffold/*.rs src/v1/stage0/src/
  cargo build --release -p v1-compiler --bin claim_executor > /tmp/b2b.log 2>&1 || { echo "R2 BUILD FAILED even with scaffold"; grep -E "^error" -A8 /tmp/b2b.log | head -30; exit 1; }
fi
rm -rf target/ccc2
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/ccc2 --regen-receipt target/ccc2.receipt.txt \
  --source-root dag --source-root src/v2 > /tmp/ccc2.log 2>&1
echo "r2_regen_exit=$?"
sed 's/^[[0-9;]*//g; s/^.*UTC //' /tmp/ccc2.log > /tmp/ccc2.clean
grep -E "adjudicated|drift|hard diagnostic" /tmp/ccc2.clean | head -4
echo "r2 effect_lines=$(grep -c 'effect summary incomplete' /tmp/ccc2.clean)"
grep "effect summary incomplete" /tmp/ccc2.clean | head -3
echo "=== CONTROL VERDICT: r2_clean=$([ $(grep -c 'effect summary incomplete' /tmp/ccc2.clean) = 0 ] && echo YES || echo NO)"
