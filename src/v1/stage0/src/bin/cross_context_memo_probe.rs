//! Does a `prepare_grammar` memo entry stored by one claim get HIT by a later claim that runs
//! in a SEPARATELY CONSTRUCTED `InterpContext`?
//!
//! THE SUBJECT IS TWO CLAIMS, NOT NINE THOUSAND. Cross-context reuse needs one store plus one
//! subsequent lookup; everything above that is cost bought with no discrimination. The whole
//! required-floor fold was being run to answer this and was OOM-killed for it.
//!
//! BOTH ARMS RUN IN ONE BINARY, and the one-context arm is not decoration. Key instability and
//! a harness that never reached `prepare_grammar` produce identical output, so a bare MISS is
//! unreadable. Arm A (one context, expected HIT) is the calibration row proving the counters
//! are wired and the claims reach the call at all; only against it is Arm B's verdict a fact
//! about context construction rather than about this harness.
use v1_compiler::cli_run;
use v1_compiler::v1_interpreter;

fn stats() -> v1_interpreter::PrepareGrammarCrossClaimMemoStats {
    v1_interpreter::prepare_grammar_cross_claim_memo_stats_snapshot()
}

fn main() {
    let source_roots = vec!["dag".to_string(), "src/v2".to_string()];
    let module = "v2.test.manual.body_lowering_normalize_add";
    // Two claims that both drive tokenize -> parse_module -> prepare_grammar.
    let (first, second) = (
        "body_lowering_fn_body_subtree_holds",
        "body_lowering_arrow_token_holds",
    );

    eprintln!("[probe] preparing repository once");
    let (prepared, prepared_sources) = match cli_run::prepare_repository_once(
        &source_roots,
        &cli_run::floor_prepared_subject_exclusions(),
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[probe] REFUSED: preparation failed: {e}");
            std::process::exit(2);
        }
    };
    // Same authority install the floor performs. It evicts the cross-claim memo as its last
    // step, so both arms below start from a cold map rather than from preparation's leftovers.
    let _guard = cli_run::register_floor_prepared_authority_guard(prepared_sources);

    let scope = match cli_run::claim_scope_for(&prepared, module) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[probe] REFUSED: no scope for {module}: {e}");
            std::process::exit(2);
        }
    };
    let digest = scope.scope_identity.clone();
    let mode = v1_interpreter::ExecutionMode::Hermetic;

    // ---- ARM A: two claims, ONE context (reproduces claim_batch; expected HIT) -------------
    let before_a = stats();
    {
        let frame = cli_run::evaluation_frame(&scope, mode, None, None);
        let _ = cli_run::run_claim_measured(&frame, &digest, first);
        let _ = cli_run::run_claim_measured(&frame, &digest, second);
    }
    let a = stats();
    let a_lookups = a.lookups - before_a.lookups;
    let a_hits = a.hits - before_a.hits;
    let a_inserts = a.inserts - before_a.inserts;
    println!("ARM_A_ONE_CONTEXT lookups={a_lookups} hits={a_hits} inserts={a_inserts}");

    // ---- ARM B: two claims, TWO separately constructed contexts (the open question) --------
    // Evict first so Arm B starts cold; otherwise Arm A's entry would serve it and the arm
    // would report a hit that proves nothing about context construction.
    v1_interpreter::clear_cross_claim_pure_memos();
    let before_b = stats();
    {
        let frame1 = cli_run::evaluation_frame(&scope, mode, None, None);
        let _ = cli_run::run_claim_measured(&frame1, &digest, first);
    }
    let mid_b = stats();
    {
        let frame2 = cli_run::evaluation_frame(&scope, mode, None, None);
        let _ = cli_run::run_claim_measured(&frame2, &digest, second);
    }
    let b = stats();
    println!(
        "ARM_B_TWO_CONTEXTS lookups={} hits={} inserts={} (first_context_alone: lookups={} hits={} inserts={})",
        b.lookups - before_b.lookups,
        b.hits - before_b.hits,
        b.inserts - before_b.inserts,
        mid_b.lookups - before_b.lookups,
        mid_b.hits - before_b.hits,
        mid_b.inserts - before_b.inserts,
    );

    // The verdict is stated by the pair, never by Arm B alone.
    let a_ok = a_lookups >= 2 && a_hits >= 1 && a_inserts >= 1;
    let b_hits = b.hits - before_b.hits;
    let b_lookups = b.lookups - before_b.lookups;
    if !a_ok {
        println!(
            "VERDICT UNREADABLE — calibration arm did not observe a hit (lookups={a_lookups} hits={a_hits} inserts={a_inserts}). \
Arm B says nothing about context construction: a harness that never reached prepare_grammar looks identical to an unstable key."
        );
        std::process::exit(3);
    }
    if b_lookups < 2 {
        println!("VERDICT UNREADABLE — Arm B made {b_lookups} lookups, needs 2 (one store, one subsequent lookup).");
        std::process::exit(3);
    }
    if b_hits >= 1 {
        println!("VERDICT KEY_SURVIVES_CONTEXT_CONSTRUCTION — a fresh context HIT an entry stored by an earlier one. A preparation-time warm can be reached by claims.");
    } else {
        println!("VERDICT KEY_UNSTABLE_ACROSS_CONTEXTS — calibration hit, but a fresh context MISSED an entry stored by an earlier one. A preparation-time warm cannot be reached; the memo is effectively disabled in the floor.");
    }
}
