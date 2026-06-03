// cli_test.rs — Hand-maintained `gunbc test` subcommand handler.
// Not generated — survives stage0 regeneration.
// RR-A §5.2: runs the manual corpus harness through `cli_run` / `dag run` machinery.

use crate::cli_run;

/// v4 corpus + dsl std.process harness (see `dsl/gunbc/gunbc_test_manual_corpus_harness.dag`).
const HARNESS_SOURCE_ROOTS: &[&str] = &["src/v4", "dsl"];

/// ProcessExit entry in `gunbc.gunbc_test_manual_corpus_harness` — not
/// `run_manual_testclaim_corpus_eval` (returns `CorpusEvalReport`, not `ProcessExit`).
const HARNESS_ENTRY_FN: &str = "gunbc_test_manual_corpus_harness_exit";

/// Entry point for `gunbc test`. Called from the generated main.rs (honors global `--dry-run`).
pub fn handle_test_with_options(dry_run: bool) {
    eprintln!(
        "gunbc test: manual corpus harness (--source-root {} --function {})",
        HARNESS_SOURCE_ROOTS.join(" "),
        HARNESS_ENTRY_FN
    );
    cli_run::handle_run_with_options(
        HARNESS_SOURCE_ROOTS
            .iter()
            .map(|s| s.to_string())
            .collect(),
        HARNESS_ENTRY_FN.to_string(),
        dry_run,
    );
}
