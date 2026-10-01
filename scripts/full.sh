set -u
cd /home/briansrls/.worktrees/gunbc/royal-newt-820
export CARGO_TARGET_DIR=/tmp/claude-1000/royal-target
B=$CARGO_TARGET_DIR/release
for f in src/v1/00_core.dag src/v1/04_infer.dag src/v1/04_env.dag src/v1/04_types.dag src/v1/04_access.dag src/v1/04_emit_info.dag dag/std/coercion.dag; do if $B/v1_src_dag_parse $f 2>&1 | grep -v "parse-clean" | grep -q .; then echo PARSE_FAIL $f; exit 1; fi; done
echo "== bootstrap"; git checkout HEAD -- src/v1/stage0/src/; CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|Finished"
echo "== regen"; rm -rf target/stage0-regen-candidate; $B/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/claude-1000/regen_full.txt 2>&1
grep -E "first_generation|refused" /tmp/claude-1000/regen_full.txt | head -2; grep -A5 "hard diag" /tmp/claude-1000/regen_full.txt | head -8 | cut -c1-250
[ -d target/stage0-regen-candidate/src ] || { echo NO_CANDIDATE; exit 1; }
cp target/stage0-regen-candidate/src/*.rs src/v1/stage0/src/; python3 /tmp/claude-1000/patch_witness.py; git checkout HEAD -- src/v1/stage0/src/cli_run/compile_clean.rs
echo "== build"; CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor --bin v1_src_dag_parse --bin gunbc --bin infer_semantics_witness 2>&1 | grep -E "^error|Finished"
echo "== claims"; F=dag/test/claim/text_boundary_identity_wall_witness_test.dag; timeout 1500 $B/claim_batch --source-root dag --source-root src/v2 --entry $F --functions $(grep -oP '(?<=^test fn )\w+' $F | paste -sd,) 2>&1 | grep -E "^PASS|^FAIL|THREW|error" | cut -c1-200
echo "== corpus head"; timeout 3500 $B/gunbc compile --source-root dag --source-root src/v2 --target dag --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir /tmp/claude-1000/wc_h > /tmp/claude-1000/wc_head.txt 2>&1
echo "== corpus base"; timeout 3500 /tmp/claude-1000/basemain-target/release/gunbc compile --source-root dag --source-root src/v2 --target dag --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --output-dir /tmp/claude-1000/wc_b > /tmp/claude-1000/wc_base.txt 2>&1
grep "blocking error" /tmp/claude-1000/wc_head.txt /tmp/claude-1000/wc_base.txt
cd /tmp/claude-1000; grep -E "^error\[" wc_base.txt | sort -u > eB.txt; grep -E "^error\[" wc_head.txt | sort -u > eH.txt; comm -13 eB.txt eH.txt > newF.txt; comm -23 eB.txt eH.txt > goneF.txt; wc -l newF.txt goneF.txt; cut -c1-260 newF.txt | head -40
cd /home/briansrls/.worktrees/gunbc/royal-newt-820
echo "== emit"; rm -rf /tmp/claude-1000/emitc; timeout 1800 $B/gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/00_compile.dag --target rust --output-dir /tmp/claude-1000/emitc 2>&1 | grep -E "blocking" | head -2
rm -rf /tmp/claude-1000/emitc_base; timeout 1800 /tmp/claude-1000/basemain-target/release/gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/00_compile.dag --target rust --output-dir /tmp/claude-1000/emitc_base 2>&1 | grep -E "blocking" | head -2
echo "emit diff files: $(diff -rq /tmp/claude-1000/emitc_base/src /tmp/claude-1000/emitc/src | wc -l)"; diff -rq /tmp/claude-1000/emitc_base/src /tmp/claude-1000/emitc/src | head -5
(cd /tmp/claude-1000/emitc && CARGO_TARGET_DIR=/tmp/claude-1000/emitc-target CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release > /tmp/claude-1000/emitc_build.txt 2>&1; grep -cE "^error" /tmp/claude-1000/emitc_build.txt; tail -1 /tmp/claude-1000/emitc_build.txt)
echo FULL_DONE
