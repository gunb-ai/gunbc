set -uo pipefail
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
export GUNBC_BIND_MEMORY_CGROUP_BYTES=15032385536
S3="std_algebra.rs std_types.rs std_computation.rs"
install_candidate() {
  find "$1" -name '*.rs' | while read -r f; do
    b=$(basename "$f")
    case "$f" in
      */src/v1/stage0/src/bin/*) cp -f "$f" src/v1/stage0/src/bin/$b ;;
      *) cp -f "$f" src/v1/stage0/src/$b ;;
    esac
  done
}
regen() { # $1=canddir $2=receipt $3=cleanlog
  rm -rf "$1"
  cargo run --release -p v1-compiler --bin claim_executor -- --required-regen \
    --regen-candidate-dir "$1" --regen-receipt "$2" \
    --source-root dag --source-root src/v2 > "$1.raw.log" 2>&1
  echo "regen_exit=$?"
  sed 's/^[[0-9;]*//g; s/^.*UTC //' "$1.raw.log" > "$3"
  echo "EFFECT_DIAGS_$(basename $1)=$(grep -c 'effect summary incomplete' "$3")"
  grep -E "adjudicated|drift|hard diagnostic|admitted" "$3" | head -5
}
report_stdcore() { # $1=label
  cargo build --release -p v1-stage0-std-core > /tmp/sc_$1.log 2>&1
  N=$(grep -cE '^error' /tmp/sc_$1.log)
  echo "STDCORE_ERRORS_$1=$N"
  if [ "$N" != "0" ]; then
    grep -oE "stage0/src/[a-z_]+\.rs:[0-9]+" /tmp/sc_$1.log | sed 's#stage0/src/##' | sort -u | tr '\n' ' ' | sed 's/^/SITES_'$1': /'; echo
  fi
}
# restore my deliverables (in case a previous control stripped them)
for f in $(find .valiant-deliv -name '*.dag'); do cp -f "$f" "${f#.valiant-deliv/}"; done
echo "deliverables restored; dirty=$(git status --porcelain -- dag src/v1 | wc -l)"

echo "########## GEN-1: main mirrors + branch corpus (my fix present in .dag) ##########"
cp -f .valiant-main-mirrors/*.rs src/v1/stage0/src/
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b1.log 2>&1 || { echo GEN1_BUILDFAIL; grep -E "^error" -A6 /tmp/b1.log | head -40; exit 1; }
regen target/g1 target/g1.receipt.txt /tmp/g1.clean
install_candidate target/g1
echo "gen1 emitter: my-fix-helper=$(grep -c 'render_rust_applied_nonpeeling_type' src/v1/stage0/src/v1_compiler_emit_rust.rs) keyed=$(grep -c 'node_is_keyed_collection(' src/v1/stage0/src/v1_compiler_emit_rust.rs)"
report_stdcore gen1

echo "########## GEN-2 binary: gen-1 mirrors, scaffold ONLY the 3 std-core files to link ##########"
for f in $S3; do cp -f .valiant-scaffold/$f src/v1/stage0/src/$f; done
cargo build --release -p v1-compiler --bin claim_executor > /tmp/b2.log 2>&1 || { echo GEN2_BUILDFAIL; grep -E "^error" -A6 /tmp/b2.log | head -50; exit 1; }
echo "gen2 binary built (scaffolded std-core for link only)"
# restore gen-1 std-core bytes so the tree = gen-1 candidate before regen
for f in $S3; do cp -f target/g1/$f src/v1/stage0/src/$f 2>/dev/null || true; done
regen target/g2 target/g2.receipt.txt /tmp/g2.clean
install_candidate target/g2
echo "gen2 emitter: my-fix-helper=$(grep -c 'render_rust_applied_nonpeeling_type' src/v1/stage0/src/v1_compiler_emit_rust.rs) keyed=$(grep -c 'node_is_keyed_collection(' src/v1/stage0/src/v1_compiler_emit_rust.rs)"
report_stdcore gen2
echo "drift vs gen1:"; for f in $(ls target/g2/*.rs); do b=$(basename $f); cmp -s target/g1/$b $f || echo "  $b"; done | head -15

echo "########## GEN-3 binary: gen-2 mirrors, NO scaffold ##########"
if ! cargo build --release -p v1-compiler --bin claim_executor > /tmp/b3.log 2>&1; then
  echo "GEN3_LINKFAIL errors=$(grep -cE '^error' /tmp/b3.log)"; grep -E "^error" -A6 /tmp/b3.log | head -40
else
  echo "gen3 binary built UNSCAFFOLDED"
  regen target/g3 target/g3.receipt.txt /tmp/g3.clean
  echo "drift g2->g3:"; for f in $(ls target/g3/*.rs); do b=$(basename $f); cmp -s target/g2/$b $f || echo "  $b"; done | head -15
  echo "FIXED_POINT=$(for f in $(ls target/g3/*.rs); do b=$(basename $f); cmp -s target/g2/$b $f || echo x; done | wc -l) drifted-files"
fi

echo "########## FINAL STATE = gen-2 candidate mirrors (last regen output) ##########"
install_candidate target/g2
for f in $S3; do cmp -s src/v1/stage0/src/$f .valiant-scaffold/$f && echo "SCAFFOLD-BYTE-PRESENT $f"; done
report_stdcore final
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep -cE '^error' | xargs echo FINAL_COMPILER_BUILD_ERRORS=
echo "LOOP_DONE"
