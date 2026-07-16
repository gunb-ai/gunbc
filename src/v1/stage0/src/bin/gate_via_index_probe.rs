#![allow(clippy::disallowed_macros)]

//! [PROTOTYPE — lever 1, PR #6766, session clever-seal-476]
//!
//! Receipt harness for routing the whole-tree compile-clean gate through the shared
//! `MultiEntryIndex` cached path (`compile_clean_whole_tree_via_index_probe`):
//!
//! 1. **Via-index gate, cold** — whole-tree compile through the cached path; wall
//!    should ≈ the raw path's (same kernel, same modules, one universe).
//! 2. **Warm batch-2 simulation** — the heaviest witness entries resolved through the
//!    SAME process index (`resolve_entry_graph_shared` with the same roots vector):
//!    their per-entry resolve, which pays 10-30s cold in CI's batch-2, must land in
//!    milliseconds-to-seconds because every module in their closure is already typed.
//! 3. **Verdict equivalence** — the raw receipt path (`compile_sources`, the current
//!    gate authority) runs after and must agree green==green. (RED==RED on the
//!    `GUNBC_TEST_FLOOR_COMPILE_CLEAN_INJECT_UNRESOLVED` planted inject is a separate
//!    invocation of this same bin with the env set — both paths read the same
//!    source-plan loader, so the inject rides both.)

use std::process::ExitCode;
use std::time::Instant;

use v1_compiler::cli_run;

const WARM_ENTRIES: &[&str] = &[
    "dag/test/claim/host_identity_observation_witness_test.dag",
    "dag/test/claim/ci_deploy_witness_test.dag",
    "dag/test/claim/srv3_host_effect_apply_witness_test.dag",
];

fn main() -> ExitCode {
    let skip_raw = std::env::var("GATE_VIA_INDEX_SKIP_RAW")
        .map(|v| v == "1")
        .unwrap_or(false);

    // 1. via-index gate, cold
    let t0 = Instant::now();
    let via_index_ok = match cli_run::compile_clean_whole_tree_via_index_probe() {
        Ok(ok) => ok,
        Err(msg) => {
            eprintln!("[probe] via-index gate REFUSED: {msg}");
            return ExitCode::from(2);
        }
    };
    eprintln!(
        "[probe] via_index_gate ok={via_index_ok} wall_s={:.1}",
        t0.elapsed().as_secs_f64()
    );

    // 2. warm batch-2 simulation through the same index
    let roots = cli_run::compile_clean_via_index_roots();
    for entry in WARM_ENTRIES {
        let t = Instant::now();
        match cli_run::resolve_entry_graph_shared(&roots, entry) {
            Ok(_) => eprintln!(
                "[probe] warm_entry entry={entry} wall_s={:.3}",
                t.elapsed().as_secs_f64()
            ),
            Err(msg) => {
                eprintln!("[probe] warm_entry entry={entry} REFUSED: {msg}");
                return ExitCode::from(2);
            }
        }
    }

    // 3. verdict equivalence vs the raw receipt path (current gate authority)
    if skip_raw {
        eprintln!("[probe] raw path skipped (GATE_VIA_INDEX_SKIP_RAW=1)");
        return if via_index_ok {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }
    let t1 = Instant::now();
    let raw_ok = cli_run::witness_layer_roots_compile_clean_emit_check();
    eprintln!(
        "[probe] raw_gate ok={raw_ok} wall_s={:.1}",
        t1.elapsed().as_secs_f64()
    );
    if via_index_ok != raw_ok {
        eprintln!("[probe] VERDICT DIVERGENCE via_index={via_index_ok} raw={raw_ok}");
        return ExitCode::from(3);
    }
    eprintln!("[probe] verdicts agree: {via_index_ok}");
    if via_index_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
