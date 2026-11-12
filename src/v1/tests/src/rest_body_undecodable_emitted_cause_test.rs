//! Witness: the EMITTED REST realization must publish a declared-side `RestBodyUndecodable` cause,
//! never a body-derived one.
//!
//! BACKGROUND. `gunbc.heal_publication` publishes `RestBodyUndecodable.cause` verbatim into a GitHub
//! PR comment (`rest_refusal_text`, `dag/gunbc/heal_publication.dag`), on a path taken while a
//! credential is in hand. The interpreter route (#16241) renders that cause from declared facts; the
//! EMITTED route used to splice the runtime io/serde error (`cause: error.to_string()` and
//! `cause: format!("body did not inhabit the declared output: {}", error)`) directly into the published
//! cause. That is a credential-exfiltration path (DESIGN single_authority / redaction): a serde decode
//! message can carry a spelling the response body presented, which the publisher then prints verbatim.
//! This witness locks the EMITTED bytes so neither arm can carry body-derived text.
//!
//! MECHANISM. The witness drives the EMITTER through `compile_entry_emission` over the entry fixture
//! `fixtures/rest_emitted_leak_probe/probe.dag` (a rest operation whose lowered call makes
//! `emit_rust_rest_result_lowering` bake the two `RestBodyUndecodable` causes), and asserts directly on
//! the generated program text -- the same bytes heal-publish commits. This is the emitter realization,
//! not the interpreter: it complements the `dag/test/claim/rest_exchange_replay.dag` claim (which checks
//! the interpreter route). This crate is NOT built by CI (the witnesses/v2-bootstrap typecheck step is
//! `cargo check --profile test -p v1-compiler --lib`, which never builds `v1-compiler-tests`), so the
//! executed receipts are run out-of-band by the remote build owner, labelled with the pre-existing
//! #16069 `helpers.rs` Rc-wrap this crate needs to compile.
//!
//! FAKE SECRET / marker. The enforcing check is the allow-list below (a cause must be one of the two
//! declared-side constant literals; a splice of the runtime io/serde `error` fails it), so this test is
//! RED on main and GREEN at the fix. A planted FAKE-SECRET string literal never appears in the fixed-
//! shape emitted lines at either ref (the sentinel is a runtime binding, not a literal in the generated
//! source), so a "does not contain FAKE_SECRET" assertion would be vacuous and is deliberately omitted;
//! the allow-list is the real discriminator.

use crate::helpers::workspace_root;
use v1_compiler::cli_run::compile_entry_emission;
use v1_compiler::v1_compiler_artifact::RenderTarget;

/// The entry fixture (repo-root `fixtures/` tree) whose lowered rest call reaches the REST lowering.
const PROBE_ENTRY: &str = "fixtures/rest_emitted_leak_probe/probe.dag";

#[test]
fn emitted_rest_body_undecodable_cause_carries_declared_text_not_body_derived_error() {
    // Emission needs the workspace as cwd (set_current_dir is process-global), so this is the single
    // test over this emission -- mirrors grounded_shared_carrier_wrap_test.
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let roots = vec![
        "dag".to_string(),
        "src/v2".to_string(),
        "fixtures/rest_emitted_leak_probe".to_string(),
    ];
    let run = compile_entry_emission(&roots, PROBE_ENTRY, true, RenderTarget::Rust);

    let emission = run.emissions.first().unwrap_or_else(|| {
        panic!(
            "the rest probe entry must reach emission; disposition: {}",
            run.measurement_line("rest-emitted-leak-probe")
        )
    });
    let result = &emission.result;
    let emitted: String = result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");

    // The REST lowering must actually have been emitted, else the allow-list below would pass
    // vacuously. On failure, dump compiler diagnostics + the emitted file paths + which rest-machinery
    // substrings survived, so a single re-run distinguishes a probe MODEL error (diagnostics non-empty /
    // files empty) from an entry that compiled but never lowered the rest call (files present, no
    // RestBodyUndecodable), from a lowering that fired but under an unexpected spelling.
    let file_paths: Vec<String> = result.files.iter().map(|f| f.path.clone()).collect();
    let markers = [
        "RestBodyUndecodable",
        "RestRefused",
        "RestAnswered",
        "rust_rest_refused",
        "RestResult",
        "ReadKey",
    ];
    let present: Vec<&str> = markers
        .iter()
        .copied()
        .filter(|m| emitted.contains(m))
        .collect();
    assert!(
        emitted.contains("RestBodyUndecodable"),
        "the REST lowering did not emit a RestBodyUndecodable refusal at all; probe did not reach \
         emit_rust_rest_result_lowering\n--- file count: {} ---\n--- file paths: {:?} ---\n--- \
         diagnostics ({}) ---\n{:?}\n--- rest-machinery substrings present: {:?} ---\n--- emitted ---\n{emitted}",
        result.files.len(),
        file_paths,
        result.diagnostics.len(),
        result.diagnostics,
        present,
    );

    // ALLOW-LIST over EVERY RestBodyUndecodable CONSTRUCTION (manager-requested inversion of a
    // deny-list: it fails unforeseen splice forms, not just the two known spellings). Each construction
    // must set its cause to one of EXACTLY the two declared-side forms the shared text module
    // (v1.compiler.rest_cause_text) produces -- a bare constant literal, or the pinned template filled
    // only with declared facts (no body-derived value). Any other cause expression -- `error.to_string()`,
    // `format!(..)`, a `+`-splice, a helper, or a different binding -- fails.
    //
    // A CONSTRUCTION is identified by the emit's construction marker `status: status as i64`; the
    // RestBodyUndecodable VARIANT DECLARATION (which carries the field TYPE `cause: NonEmptyStr`, not a
    // value) and unrelated mentions of the name are not constructions and are correctly not checked.
    //
    // The two allowed literals are the authority's exact text, so a wording change in the authority fails
    // this too (single authority). The decode form keeps the pinned '... at <located> (<coproduct>)' /
    // 'did not inhabit' shape the claims in dag/test/claim/rest_exchange_replay.dag pin, with <located>
    // honestly the withheld-field phrase (fail_closed / epistemic_humility -- never the output type).
    const READ_REFUSED_CAUSE: &str = "the response body could not be read from the host";
    const DECODE_CAUSE_PREFIX: &str =
        "body did not inhabit the declared output: the response body does \
         not inhabit its declared output type at the failing field withheld on a credential path (";

    let mut saw_read = false;
    let mut saw_decode = false;
    let mut offenders: Vec<String> = Vec::new();
    for (idx, line) in emitted.lines().enumerate() {
        if !(line.contains("RestBodyUndecodable") && line.contains("status: status as i64")) {
            continue;
        }
        // The cause expression is what follows `cause: ` up to its `.to_string()`.
        let after = line.split_once("cause: ").map_or(line, |(_, rest)| rest);
        let cause_expr = after
            .split_once(".to_string()")
            .map_or(after, |(expr, _)| expr)
            .trim();
        // A safe cause is a SINGLE string literal: it begins and ends with a double quote and the only
        // double quotes on it are those two -- so `" + <expr> + "` splicing cannot survive, and neither
        // can a format!/call form (which never begins with a quote).
        let inner = cause_expr
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'));
        let single_literal = match inner {
            Some(body) => {
                !body.contains('"') && !cause_expr.contains("format!") && !cause_expr.contains('+')
            }
            None => false,
        };
        let is_read = single_literal && inner == Some(READ_REFUSED_CAUSE);
        let is_decode = single_literal
            && inner.map_or(false, |b| {
                b.starts_with(DECODE_CAUSE_PREFIX) && b.ends_with(')')
            });
        if !(is_read || is_decode) {
            offenders.push(format!("emitted line {idx}: {line}"));
        }
        if is_read {
            saw_read = true;
        }
        if is_decode {
            saw_decode = true;
        }
    }

    // Collect every offending construction before failing, so a single RED run names BOTH leaky arms
    // (the body-read `error.to_string()` and the decode `format!(..)`) rather than stopping at the first.
    assert!(
        offenders.is_empty(),
        "found {} emitted RestBodyUndecodable construction(s) whose cause is not a declared-side \
         constant literal (a runtime splice or unforeseen form):\n{}",
        offenders.len(),
        offenders.join("\n")
    );

    // Both declared-side arms must be present, so the test cannot pass vacuously if an arm disappears
    // or is rewritten to an unlisted form. At main both constructions splice `error`, so the per-arm
    // assert above fires first and the run is RED; the presence guard is the corroboration at head.
    assert!(
        saw_read && saw_decode,
        "expected both the read-refused and the pinned-decode declared-side cause arms; \
         saw_read={saw_read} saw_decode={saw_decode}\n--- emitted ---\n{emitted}"
    );
}
