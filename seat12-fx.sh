set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
for fn in a_declarer_body_refusal_heads_the_importer_chain_and_declarer_refused_is_last an_imported_name_from_a_cross_tree_refused_declarer_is_declarer_refused a_genuinely_undeclared_name_is_unbound; do
cat > /tmp/fxw.dag <<D
module v2.test.claim.name_resolve.fxw
import v2.test.claim.name_resolve.declarer_refused { $fn }
import std.process { ProcessExit, ExitSuccess, ExitFailure }
fn fx() -> ProcessExit { if $fn() { ExitSuccess } else { ExitFailure { code: 1, reason: "false" } } }
D
mkdir -p src/v2/test/claim/name_resolve && cp /tmp/fxw.dag src/v2/test/claim/name_resolve/fxw.dag
./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/fxw.dag --function fx > fx.out 2>&1; echo "FX $fn exit=$?"; tail -n 3 fx.out | cut -c1-300
done
./target/release/gunbc check --source-root dag --source-root src/v2 dag/gunbc/recurring_failure_mode/cascade_chain_head_names_a_node_the_importer_never_wrote.dag > rf.out 2>&1; echo "RF exit=$?"; tail -n 4 rf.out | cut -c1-300
