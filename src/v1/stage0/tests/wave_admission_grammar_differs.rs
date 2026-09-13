//! THE GRAMMAR-DIFFERS ARM, EXECUTED. The whole point of the base-environment loader is the
//! partition inside the wave adjudication: when base and head speak different grammars, the base is
//! read under ITS OWN environment and the changed-file baseline reconstruction is abandoned for a
//! full base parse. Review 64939 on #10970 found that arm shipped with no executed evidence -- the
//! loader was proven, the partition was not. This is the evidence.
//!
//! THE SPECIMEN IS THE REAL ONE IN MINIATURE. The class row
//! (`gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change`) records the
//! `func` -> `fn` cut: every changed file is well formed under its own revision and unreadable under
//! the other's. So the scratch repository's BASE carries an extra item-form keyword, `zzfunc`, and a
//! probe module declared with it; the HEAD removes the keyword and rewrites the probe as `fn`. A gate
//! that parses the base under the head's grammar refuses that probe at the base -- the exact
//! `does not parse at the base revision` refusal the row quotes -- and answers `NotEvaluated`. A gate
//! that reads the base under the base's grammar reads it and adjudicates.
//!
//! TWO ASSERTIONS, EACH OWNING ONE HALF. The green drives the REAL adjudication
//! (`run_wave_admission_between`, the production body behind `run_required_wave_admission`) over
//! the pair and requires `Adjudicated`. The red is the direct pair on the very parse the partition
//! performs: `base_records` of the base probe REFUSES under the head environment and SUCCEEDS under
//! the base's. Without the red, a gate that happened to adjudicate for an unrelated reason would
//! pass the green; the red shows the distinction is load-bearing.
//!
//! The scratch repository is named explicitly at every git site; nothing here touches the process
//! cwd (`workspace_root()` memoizes it, and this binary's tests run concurrently).

use std::path::Path;
use std::process::Command;

use v1_compiler::cli_run::namespace_wave_admission::{
    base_records, load_parse_environment_at, materialize_revision_paths,
    run_wave_admission_between, WaveAdmissionOutcome,
};
use v1_compiler::cli_run::{run_dag_parse_sweep, workspace_root};
use v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment;

const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";
const PROBE_PATH: &str = "dag/probe/zz_grammar_probe.dag";
const PROBE_MODULE: &str = "probe.zz_grammar_probe";

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

/// The environment module with a third function form, `zzfunc`, admitted beside `fn`.
///
/// The row is `fn`'s row with the keyword changed, so a `zzfunc` declaration parses exactly as an
/// `fn` declaration would -- the grammar DIFFERENCE is the whole content of the base, and it is
/// confined to one keyword so nothing else about the corpus moves.
fn with_zzfunc(env_source: &str) -> String {
    let fn_row_start = env_source
        .find("  ItemForm { kind: FuncForm, keyword: \"fn\",")
        .expect("the fn item-form row");
    let fn_row_end = env_source[fn_row_start..]
        .find("},\n")
        .map(|i| fn_row_start + i + 3)
        .expect("the fn row's end");
    let fn_row = &env_source[fn_row_start..fn_row_end];
    let zz_row = fn_row.replacen("keyword: \"fn\"", "keyword: \"zzfunc\"", 1);
    assert_ne!(fn_row, zz_row, "the fn row's keyword did not rewrite");
    let mut out = String::new();
    out.push_str(&env_source[..fn_row_end]);
    out.push_str(&zz_row);
    out.push_str(&env_source[fn_row_end..]);
    let keyword_line = "\"fn\": true, \"func\": true,";
    assert!(
        out.contains(keyword_line),
        "the keyword_set row naming fn and func was not found"
    );
    out.replacen(
        keyword_line,
        "\"fn\": true, \"func\": true, \"zzfunc\": true,",
        1,
    )
}

/// Build the scratch pair: base speaks `zzfunc`, head does not. Returns (repo, base, head).
///
/// `label` keeps each test's repository distinct. The two tests in this binary run on separate
/// threads of ONE process, so a directory named by PID alone is shared mutable state: either test
/// could delete or rewrite the other's repository mid-commit. One directory per invocation.
fn build_grammar_differing_pair(label: &str) -> (std::path::PathBuf, String, String) {
    let root = workspace_root();
    let scratch = std::env::temp_dir().join(format!(
        "gunbc-wave-grammar-differs-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");

    // The whole `dag/` tree, through the same acquisition route the loader itself uses -- one
    // checked implementation rather than a hand-shell pipeline beside it.
    materialize_revision_paths(&root, "HEAD", &scratch, &["dag"])
        .unwrap_or_else(|e| panic!("materializing the scratch corpus failed: {e}"));

    // BASE: the grammar admits `zzfunc`, and one module uses it.
    let env_path = scratch.join(ENVIRONMENT_MODULE_PATH);
    let original_env = std::fs::read_to_string(&env_path).expect("read environment module");
    std::fs::write(&env_path, with_zzfunc(&original_env)).expect("write base environment");
    let probe_dir = scratch.join("dag/probe");
    std::fs::create_dir_all(&probe_dir).expect("create probe dir");
    std::fs::write(
        scratch.join(PROBE_PATH),
        format!("module {PROBE_MODULE}\n\nzzfunc probe_value() -> Int {{ 1 }}\n"),
    )
    .expect("write base probe");

    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &["commit", "--quiet", "-m", "base: grammar admits zzfunc"],
    );
    let base = git(&scratch, &["rev-parse", "HEAD"]);

    // HEAD: the keyword is cut and the probe is rewritten in the surviving form -- the
    // func -> fn migration in miniature, with every changed file well formed under its own
    // revision and the base probe unreadable under the head's grammar.
    std::fs::write(&env_path, &original_env).expect("write head environment");
    std::fs::write(
        scratch.join(PROBE_PATH),
        format!("module {PROBE_MODULE}\n\nfn probe_value() -> Int {{ 1 }}\n"),
    )
    .expect("write head probe");
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &[
            "commit",
            "--quiet",
            "-m",
            "head: zzfunc cut, probe rewritten as fn",
        ],
    );
    let head = git(&scratch, &["rev-parse", "HEAD"]);
    assert_ne!(base, head);

    (scratch, base, head)
}

/// THE RED: the base probe is unreadable under the head's grammar and readable under its own.
///
/// This is the parse the partition performs on every base-side file, isolated. If both environments
/// read the probe, the grammar difference is not load-bearing and the green below would prove
/// nothing about the partition; if neither reads it, the base is genuinely broken and the gate's
/// `NotEvaluated` would be correct.
#[test]
fn the_base_probe_refuses_under_the_head_grammar_and_reads_under_its_own() {
    let (scratch, base, _head) = build_grammar_differing_pair("red");
    let base_probe = git(&scratch, &["show", &format!("{base}:{PROBE_PATH}")]);
    assert!(
        base_probe.contains("zzfunc probe_value"),
        "the base probe is not the zzfunc declaration: {base_probe}"
    );

    let under_head = base_records(PROBE_PATH, &base_probe, dag_parse_environment());
    assert!(
        under_head.is_err(),
        "the base probe PARSED under the head grammar, so a grammar difference is not what this \
         pair carries and the partition is not exercised"
    );
    let refusal = under_head.unwrap_err();
    assert!(
        refusal.contains("does not parse at the base revision"),
        "refusal text does not name the class the row records: {refusal}"
    );

    let base_env = load_parse_environment_at(&scratch, &base)
        .unwrap_or_else(|e| panic!("base environment did not load: {e}"));
    let under_base = base_records(PROBE_PATH, &base_probe, base_env);
    let records = under_base.unwrap_or_else(|e| {
        panic!("the base probe did not parse under the base's OWN grammar: {e}")
    });
    assert!(
        records.iter().any(|r| r.rel_path == PROBE_PATH),
        "the base parse produced no record for the probe: {records:?}"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}

/// THE GREEN: the real adjudication, over a pair whose grammars differ, adjudicates.
///
/// A gate that read the base under the head's grammar answers `NotEvaluated` here with the row's
/// own refusal text -- which is what #10850's floor lane reports today. This drives the production
/// body through the grammar-differs arm: agreement detects the differing environment file, the base
/// environment is loaded, the changed-file reconstruction is abandoned, and every base-side file is
/// read under the base's grammar.
#[test]
fn a_grammar_change_between_base_and_head_is_adjudicated_not_refused() {
    let (scratch, base, head) = build_grammar_differing_pair("green");

    // The head index the way production builds it: the parse sweep over the scratch head tree.
    let sweep = run_dag_parse_sweep(&scratch, &["dag"]).unwrap_or_else(|errors| {
        panic!(
            "the scratch head tree did not parse-sweep clean ({} error(s)); first: {}",
            errors.len(),
            errors.first().map(String::as_str).unwrap_or("<none>")
        )
    });

    let outcome = run_wave_admission_between(&scratch, &base, &head, &sweep.index)
        .unwrap_or_else(|e| panic!("wave adjudication errored: {e}"));

    match outcome {
        WaveAdmissionOutcome::Adjudicated {
            base: b, head: h, ..
        } => {
            assert_eq!(b, base);
            assert_eq!(h, head);
        }
        WaveAdmissionOutcome::NotEvaluated { reason } => panic!(
            "the gate refused a base it should have read under its own grammar -- the \
             grammar-differs arm did not fire, or it fired and could not read the base: {reason}"
        ),
        other => panic!("unexpected outcome for a differing-grammar pair: {other:?}"),
    }

    let _ = std::fs::remove_dir_all(&scratch);
}
