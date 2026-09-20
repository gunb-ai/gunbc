//! THE OWNER PAYS AT ITS OWN MERGE, EXECUTED OVER A REAL REPOSITORY.
//!
//! Three changes over one base, each a commit on its own branch, each adjudicated by the production
//! body (`run_wave_admission_between`) the required run calls:
//!
//! - THE CONTROL: the change moves a binding and carries no admission. It must refuse as
//!   unadjudicated, or the two arms below prove nothing about admissions.
//! - THE RED: the same move, admitted by a row FILE in the tree -- the carrier that landed a
//!   consumed row on main three times in three days (gunbc#11587, #11660, #11681). It refuses at
//!   this change's own gate, before the row can land.
//! - THE POSITIVE CONTROL: the same move, admitted by the same row carried in the commit message.
//!   It is admitted, and the head tree it would land holds no admission at all -- so there is
//!   nothing left for the next change to delete.
//!
//! The scratch repository is named explicitly at every git site; nothing here touches the process
//! cwd (`workspace_root()` memoizes it, and this binary's tests run concurrently).

use std::path::Path;
use std::process::Command;

use v1_compiler::cli_run::namespace_wave_admission::{
    environment_load_refusal_text, materialize_revision_paths, report_unadjudicated,
    run_wave_admission_between, wave_admission_refusal, WaveAdmissionOutcome,
    ADMISSION_CARRIER_FENCE, ADMISSION_ROSTER_REL_PATH,
};
use v1_compiler::cli_run::{run_dag_parse_sweep, workspace_root};

const STEM: &str = "probe_carrier_consumer_use_it_widget";
const HOME: &str = "module probe.carrier_home\n\ndata widget: String = \"w\"\n";
const OTHER: &str = "module probe.carrier_other\n\ndata widget: String = \"o\"\n";
const CONSUMER_HOME: &str = "module probe.carrier_consumer\n\nimport probe.carrier_home { widget }\n\nfn use_it() -> String { widget }\n";
const CONSUMER_OTHER: &str = "module probe.carrier_consumer\n\nimport probe.carrier_other { widget }\n\nfn use_it() -> String { widget }\n";

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} failed to start: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn row() -> String {
    format!(
        "module gunbc.namespace.transition_admission.{STEM}\n\n\
         import std.types {{ NonEmptyStr, List }}\n\
         import std.decl_ref {{ decl_ref }}\n\
         import gunbc.compiler_frontend_program_interlock {{ TargetChanged }}\n\
         import gunbc.namespace.transition_admission {{ TransitionAdmission, Binding }}\n\n\
         data {STEM}: TransitionAdmission = TransitionAdmission {{\n\
           label: \"widget moves to carrier_other\" as NonEmptyStr,\n\
           subject: Binding {{\n\
             enclosing: decl_ref(\"probe.carrier_consumer\", \"use_it\"),\n\
             spelling: \"widget\" as NonEmptyStr,\n\
             expected_candidates: [decl_ref(\"probe.carrier_other\", \"widget\")],\n\
           }},\n\
           disposition: TargetChanged,\n\
         }}\n"
    )
}

/// The base: the real corpus plus three probe modules, with no admission directory.
fn build_base(scratch: &Path) -> String {
    let _ = std::fs::remove_dir_all(scratch);
    std::fs::create_dir_all(scratch).expect("create scratch repo");
    materialize_revision_paths(&workspace_root(), "HEAD", scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });
    let _ = std::fs::remove_dir_all(scratch.join(ADMISSION_ROSTER_REL_PATH));
    let probe = scratch.join("dag/probe");
    std::fs::create_dir_all(&probe).expect("probe dir");
    std::fs::write(probe.join("carrier_home.dag"), HOME).unwrap();
    std::fs::write(probe.join("carrier_other.dag"), OTHER).unwrap();
    std::fs::write(probe.join("carrier_consumer.dag"), CONSUMER_HOME).unwrap();
    git(scratch, &["init", "--quiet"]);
    git(scratch, &["config", "user.email", "probe@example.invalid"]);
    git(scratch, &["config", "user.name", "probe"]);
    git(scratch, &["add", "-A"]);
    git(scratch, &["commit", "--quiet", "-m", "base"]);
    git(scratch, &["rev-parse", "HEAD"])
}

/// One change on its own branch from `base`: the binding moves, plus whatever carrier `arm` adds.
fn change(scratch: &Path, base: &str, branch: &str, row_file: bool, message: &str) -> String {
    git(scratch, &["checkout", "--quiet", "-b", branch, base]);
    std::fs::write(
        scratch.join("dag/probe/carrier_consumer.dag"),
        CONSUMER_OTHER,
    )
    .unwrap();
    if row_file {
        let dir = scratch.join(ADMISSION_ROSTER_REL_PATH);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{STEM}.dag")), row()).unwrap();
    }
    git(scratch, &["add", "-A"]);
    git(scratch, &["commit", "--quiet", "-m", message]);
    git(scratch, &["rev-parse", "HEAD"])
}

fn adjudicate_checked_out(
    scratch: &Path,
    base: &str,
    head: &str,
) -> Result<WaveAdmissionOutcome, String> {
    let sweep = run_dag_parse_sweep(scratch, &["dag"]).unwrap_or_else(|errors| {
        panic!(
            "the scratch head tree did not parse-sweep clean ({} error(s)); first: {}",
            errors.len(),
            errors.first().map(String::as_str).unwrap_or("<none>")
        )
    });
    run_wave_admission_between(scratch, base, head, &sweep.index)
}

#[test]
fn a_row_file_in_the_tree_refuses_at_its_owners_own_gate() {
    let scratch = std::env::temp_dir().join(format!("gunbc-wave-carrier-{}", std::process::id()));
    let base = build_base(&scratch);

    // THE CONTROL: the move with no admission refuses, so the admission is what the arms vary.
    let bare = change(&scratch, &base, "bare", false, "move widget, no admission");
    let outcome = adjudicate_checked_out(&scratch, &base, &bare).expect("adjudicates");
    match &outcome {
        WaveAdmissionOutcome::Adjudicated { report, .. } => assert!(
            !report_unadjudicated(report).is_empty(),
            "PLANT NEVER REACHED: the unadmitted move must be an unadjudicated delta"
        ),
        other => panic!("expected Adjudicated, got {other:?}"),
    }
    assert!(wave_admission_refusal(&outcome).is_some());

    // THE RED: the owner lands its admission as a tree row. Refused at its own gate.
    let in_tree = change(
        &scratch,
        &base,
        "in_tree",
        true,
        "move widget, row in the tree",
    );
    let err = adjudicate_checked_out(&scratch, &base, &in_tree)
        .expect_err("a row file in the tree must refuse at its owner's own gate");
    assert!(err.contains(&format!("{STEM}.dag")), "names the row: {err}");
    assert!(
        err.contains(ADMISSION_CARRIER_FENCE),
        "names the remedy: {err}"
    );

    // THE POSITIVE CONTROL: the same row, carried by the change. Admitted, and nothing is left.
    let message = format!(
        "move widget, admission carried\n\n{ADMISSION_CARRIER_FENCE}\n{}```\n",
        row()
    );
    let carried = change(&scratch, &base, "carried", false, &message);
    let outcome = adjudicate_checked_out(&scratch, &base, &carried).expect("adjudicates");
    assert_eq!(
        wave_admission_refusal(&outcome),
        None,
        "the carried admission must admit the move: {outcome:?}"
    );
    match &outcome {
        WaveAdmissionOutcome::Adjudicated { report, .. } => assert!(
            report
                .deltas
                .iter()
                .any(|d| d.admitted_by.as_deref() == Some("widget moves to carrier_other")),
            "admitted BY the carried row, not by an absent delta: {:?}",
            report.deltas
        ),
        other => panic!("expected Adjudicated, got {other:?}"),
    }
    assert_eq!(
        git(
            &scratch,
            &[
                "ls-tree",
                "-r",
                "--name-only",
                &carried,
                "--",
                ADMISSION_ROSTER_REL_PATH
            ]
        ),
        "",
        "the tree this change lands must hold no admission for the next change to delete"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}
