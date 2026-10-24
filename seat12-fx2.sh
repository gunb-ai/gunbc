cargo build --release -p v1-compiler --bin gunbc || exit 1
cat > src/v2/test/claim/name_resolve/fxr.dag <<'D'
module v2.test.claim.name_resolve.fxr
import v2.test.claim.name_resolve.declarer_refused { dr_verdicts, dr_refused_first }
import v2.std.diagnostic { NonEmptyDiagnostics }
import std.optional { Absent, Optional, Present }
import std.process { ProcessExit, ExitSuccess, ExitFailure }
fn fx() -> ProcessExit {
  match dr_verdicts() {
    Absent => ExitFailure { code: 3, reason: "no verdicts" }
    Present { value: vs } => match dr_refused_first(v: vs.body_user) {
      Absent => ExitFailure { code: 3, reason: "not refused" }
      Present { value: d } => if d.head.reason == ^resolve_reason_declarer_refused { ExitSuccess } else { ExitFailure { code: 1, reason: "red as expected: head is not declarer_refused" } }
    }
  }
}
D
./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/fxr.dag --function fx > fx.out 2>&1; echo "FXRED exit=$?"; grep -a "red as expected\|error" fx.out | head -3 | cut -c1-300
cat > dag/gunbc/rfprobe.dag <<'D'
module gunbc.rfprobe
import gunbc.recurring_failure_mode.cascade_chain_head_names_a_node_the_importer_never_wrote { cascade_chain_head_names_a_node_the_importer_never_wrote }
import std.process { ProcessExit, ExitSuccess }
fn fx() -> ProcessExit { let r = cascade_chain_head_names_a_node_the_importer_never_wrote
  ExitSuccess }
D
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/rfprobe.dag --function fx > rf.out 2>&1; echo "RF exit=$?"; grep -a "error\|refus" rf.out | head -5 | cut -c1-300
