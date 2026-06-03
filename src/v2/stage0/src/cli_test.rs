// cli_test.rs — Hand-maintained `gunbc test` subcommand handler.
// Not generated — survives stage0 regeneration.
// RR-A §5.2: runs the manual corpus harness through `cli_run` / `dag run` machinery.

use crate::cli_run;

/// Mirrors `bootstrap_manual_corpus_harness.source_root` (v4_dag_source convention).
const HARNESS_SOURCE_ROOT: &str = "src/v4";

/// ProcessExit entry in `v4.test.claim.workflow.testclaim_corpus_runner` — not
/// `run_manual_testclaim_corpus_eval` (returns `CorpusEvalReport`, not `ProcessExit`).
const HARNESS_ENTRY_FN: &str = "gunbc_test_manual_corpus_harness_exit";

/// Entry point for `gunbc test`. Called from the generated main.rs (honors global `--dry-run`).
pub fn handle_test_with_options(dry_run: bool) {
    eprintln!(
        "gunbc test: manual corpus harness (--source-root {} --function {})",
        HARNESS_SOURCE_ROOT, HARNESS_ENTRY_FN
    );
    cli_run::handle_run_with_options(
        vec![HARNESS_SOURCE_ROOT.to_string()],
        HARNESS_ENTRY_FN.to_string(),
        dry_run,
    );
}
