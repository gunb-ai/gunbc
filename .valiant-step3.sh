set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536

############ ROUND 1: regen with the MERGED compiler (keyed arm live) ############
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep "^error" | wc -l | xargs echo "round1 build_errors="
rm -rf target/rc1
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/rc1 --regen-receipt target/r1.receipt.txt \
  --source-root dag --source-root src/v2 2>&1 \
  | sed 's/^[[0-9;]*//g; s/^.*UTC //' \
  | grep -E "effect_diagnostic|effect summary|admission|first_generation_equal|drift|refused|FAIL|adjudicated" | head -10 | sed 's/^/  /'
echo "round1 keyed-arm-in-candidate=$(grep -c node_is_keyed_collection target/rc1/v1_compiler_emit_rust.rs) keyed-calls=$(grep -c 'node_is_keyed_collection(' target/rc1/v1_compiler_emit_rust.rs)"
# install: candidate replaces all mirrors
mkdir -p .valiant-backup
for f in $(ls src/v1/stage0/src/*.rs); do b=.valiant-backup/$(basename $f); [ -f "$b" ] || cp "$f" "$b"; done
cp -f target/rc1/*.rs src/v1/stage0/src/
cp -f target/rc1/src/v1/stage0/src/*.rs src/v1/stage0/src/ 2>/dev/null

############ ROUND 2: rebuild from candidate, regen again ############
if ! cargo build --release -p v1-compiler --bin claim_executor > /tmp/r2build.log 2>&1; then
  N=$(grep -cE "^error(\[|:)" /tmp/r2build.log)
  echo "round2: link blocked ($N errors) -> linking against .valiant-scaffold std-core bodies (iteration only)"
  cp -f .valiant-scaffold/*.rs src/v1/stage0/src/
  cargo build --release -p v1-compiler --bin claim_executor > /tmp/r2build2.log 2>&1 || { echo "ROUNDTWO BUILD FAILED"; grep "^error" -A10 /tmp/r2build2.log | head -50; exit 1; }
  echo "round2: link_errors_after_scaffold=$(grep -cE '^error(\[|:)' /tmp/r2build2.log)"
fi
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep "^error" | wc -l | xargs echo "round2 build_errors="
cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
  --regen-candidate-dir target/rc2 --regen-receipt target/r2.receipt.txt \
  --source-root dag --source-root src/v2 2>&1 \
  | sed 's/^[[0-9;]*//g; s/^.*UTC //' \
  | grep -E "effect_diagnostic|effect summary|admission|first_generation_equal|drift|refused|FAIL|adjudicated" | head -12 | sed 's/^/  /'
echo "round2 keyed-arm-in-candidate=$(grep -c node_is_keyed_collection target/rc2/v1_compiler_emit_rust.rs) keyed-calls=$(grep -c 'node_is_keyed_collection(' target/rc2/v1_compiler_emit_rust.rs)"

############ FINAL STATE: candidate + scaffold std-core (iteration only) ############
cp -f .valiant-scaffold/*.rs src/v1/stage0/src/
cargo build --release -p v1-stage0-std-core 2>&1 | tee /tmp/stdcore.log | grep -E "^error" | wc -l | xargs echo "STDCORE_ERRORS="
for f in std_algebra.rs std_types.rs std_computation.rs; do
  [ -n "$(diff -q src/v1/stage0/src/$f .valiant-scaffold/$f 2>/dev/null)" ] && echo "SCAFFOLD BYTE STILL PRESENT: $f"
done
# per-site attribution dump
grep -E "^ +--> src/v1/stage0/src/.*:[0-9]+" /tmp/stdcore.log | sed 's/^ *--> //' | sort -u > /tmp/sites.txt
echo "distinct error sites: $(wc -l < /tmp/sites.txt)"
while IFS=: read -r f l c; do
  echo "##### $f:$l"
  awk -v L="$l" 'NR>=L-14 && NR<=L+2 {printf "%6d %s\n", NR, $0}' "$f"
done < /tmp/sites.txt | head -320
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep "^error" | wc -l | xargs echo "FINAL_COMPILER_BUILD_ERRORS="
# how far from converged? compare final mirrors to candidate r2
diff -rq src/v1/stage0/src target/rc2 2>/dev/null | grep -v "Only in" | head -10 | sed 's/^/drift-vs-r2: /'
