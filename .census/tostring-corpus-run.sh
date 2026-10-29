# Corpus population: gunbc compile --output-dir /tmp/zb-corpus-out --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --source-root dag --source-root src/v2 (primary root dag), same as warm-crane-219's run at f11d2b019e7.
# Precondition: instrumented gunbc built from THIS tree (reading byte-identical to cb53b0e38d4).
set -u
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PWD/target} RUSTC_WRAPPER=
echo "HEAD $(git rev-parse HEAD)"
echo "BIN sha256 $(sha256sum ./target/release/gunbc | cut -d' ' -f1)"
rm -f corpus-census.log
./target/release/gunbc compile --output-dir /tmp/zb-corpus-out --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json --source-root dag --source-root src/v2 > corpus-census.log 2>&1; echo "compile exit=$?"
echo "=== TOSTRCENSUS lines"; grep -c "TOSTRCENSUS" corpus-census.log
echo "=== population lines (file-qualified):"; grep -E "^error\[[^:]+:[0-9]+:[0-9]+\].*TOSTRCENSUS" corpus-census.log | wc -l
