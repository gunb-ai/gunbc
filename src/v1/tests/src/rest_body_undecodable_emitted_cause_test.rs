use crate::helpers::compile_dag;

// EMITTED-ROUTE CREDENTIAL-PATH WITNESS (the discriminator the brief names).
//
// A REST exchange's RestBodyUndecodable cause is a credential path: gunbc.heal_publication
// rest_refusal_text publishes it with a credential in hand. Before the fix the EMITTED realization
// of that refusal passed body-derived text straight through -- the body-read arm emitted
// `cause: error.to_string()` (the reqwest io error) and the decode arm emitted
// `format!("body did not inhabit the declared output: {}", error)` (the serde_json message, which
// can echo a spelling the body presented). claim_batch realizes the whole closure and is green
// either way, so it cannot tell the two states apart -- THIS test is the discriminator: it reads
// the EMITTED BYTES for a REST exchange and refuses while either arm splices a body-derived input.
//
// FAKE SECRET IN EVERY BODY-DERIVED INPUT, and WHY THE ASSERTION IS THE CODE SHAPE. The two
// body-derived inputs the arms could echo are the runtime io error (body read refused) and the
// serde_json message (decode refused); both reach the cause through the SAME vector -- the emitted
// code splicing the runtime `error` binding into it (`cause: error.to_string()` /
// `format!("...", error)`). A credential carried by the body therefore reaches the published cause
// if and only if the emitted cause splices a runtime value. So the guarantee is asserted as the
// absence of that splice: each emitted RestBodyUndecodable arm's cause must be a CONSTANT literal
// carrying the single declared-side authority's phrase, never a runtime expression. The probe's
// request credential is planted with FAKE-SECRET-MARKER (mirroring the interpreter's t7 credential
// probe) so the credential is present in every body-derived input on this exchange; it is a runtime
// call-argument, so it is not asserted literally against the emitted source (a fixed-shape cause
// never contains it at either ref -- asserting so would be vacuous). The splice-forbidding below is
// the enforceable form of the same barrier, and it is exactly what fires RED at main.
//
// RED at main (the arms emitted the leaking expressions), GREEN at head (they route through
// v1.compiler.rest_cause_text).

// TEST RECEIPT (ctrl review policy: a new hand-written Rust test carries a receipt). This file is
// hand-written Rust in the CONVENTIONAL hand-written test package v1-compiler-tests
// (src/v1/tests/src/*_test.rs), NOT hand-written Rust smuggled into the generated stage0 mirror that
// gunbc.integration_concern::grain_witness_tests_are_hand_written_rust_in_the_stage0_mirror governs
// -- like every other *_test.rs here it is ordinary hand-written test source, so no before/after seed
// census applies. Deferral, naming a lane and a tracking row: the emitted-route cause text it pins is
// owned by the single authority v1.compiler.rest_cause_text, whose interpreter-side fold-in (the fork)
// is tracked at dag/gunbc/rung_drop/gunbc_run_unreached_declarations_not_realized.dag (NEXT RUNG,
// dispatched by silent-bee-845) as PR-2; that row is the lane that owns this witness's subject text.

fn rest_exchange_probe() -> &'static str {
    "module fixture.rest_emitted_leak_probe\n\
     type Secret nominal_opaque = String\n\
     service fixture.RestEmittedLeak {\n\
       config {\n\
         endpoint: \"https://192.0.2.1\"\n\
         auth: Bearer\n\
         auth_input: access_token\n\
       }\n\
       operation ReadKey {\n\
         requires Network\n\
         input { access_token: Secret, name: String }\n\
         output { data: String from \"data\" }\n\
         readonly\n\
         transport rest { method: GET, path: \"/v1/key\" }\n\
       }\n\
     }\n\
     fn witness() -> Bool {\n\
       let r = fixture.RestEmittedLeak.ReadKey(access_token: \"FAKE-SECRET-MARKER\" as Secret, name: \"plain\")\n\
       true\n\
     }\n"
}

fn emitted_text(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> String {
    result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn emitted_rest_body_undecodable_cause_carries_declared_text_not_body_derived_error() {
    let result = compile_dag(rest_exchange_probe());
    let emitted = emitted_text(&result);

    // The REST result lowering must actually have been emitted, else the assertions below would
    // pass vacuously.
    assert!(
        emitted.contains("RestBodyUndecodable"),
        "the REST lowering did not emit a RestBodyUndecodable refusal at all; probe did not reach \
         emit_rust_rest_result_lowering\n--- emitted ---\n{emitted}"
    );

    // The body-read arm publishes the fixed read-refused phrase from the single authority, as a
    // constant -- not the io error. Absent at main (which emitted `cause: error.to_string()`).
    assert!(
        emitted.contains(
            "RestBodyUndecodable { status: status as i64, cause: \"the response body could not be \
             read from the host\".to_string() }"
        ),
        "the emitted body-read arm did not publish the declared read-refused phrase\n--- emitted ---\n{emitted}"
    );

    // The decode arm publishes the pinned 'body did not inhabit the declared output: <declared> at
    // <located> (<coproduct>)' template, but the emitted whole-body serde decode never located a
    // field, so <located> must honestly say the field is withheld -- NOT stand the output type in for
    // a field it never computed (gunbc.design_argument::fail_closed / epistemic_humility). Absent at
    // main, which emitted the '{}' format over `error`.
    assert!(
        emitted.contains(
            "body did not inhabit the declared output: the response body does not inhabit its \
             declared output type at the failing field withheld on a credential path ("
        ),
        "the emitted decode arm did not publish the declared 'did not inhabit ... at <withheld> \
         (<coproduct>)' template\n--- emitted ---\n{emitted}"
    );
    // The shape the claims in dag/test/claim/rest_exchange_replay.dag pin (the '... at <located>
    // (<coproduct>)' region) is kept, and no fabricated field: the located slot carries the
    // withheld-field phrase rather than a field the emitter never worked out.
    assert!(
        emitted.contains(" at ") && emitted.contains(" (") && emitted.contains(")"),
        "the emitted decode cause lost the pinned '... at <located> (<coproduct>)' shape\n--- emitted ---\n{emitted}"
    );

    // NO BODY-DERIVED TEXT REACHES THE CAUSE. Every RestBodyUndecodable arm must splice nothing from
    // the runtime error binding -- the only vector a body-derived credential could use to reach the
    // cause. At main these fire (the arms emitted `cause: error.to_string()` and
    // `format!("...", error)`); at head they hold (the cause is a constant declared-side literal).
    for (idx, line) in emitted.lines().enumerate() {
        if line.contains("RestBodyUndecodable") {
            assert!(
                !line.contains("error.to_string()"),
                "emitted RestBodyUndecodable splices the body-derived error at line {idx}: {line}"
            );
            assert!(
                !line.contains("format!("),
                "emitted RestBodyUndecodable formats body-derived text at line {idx}: {line}"
            );
        }
    }
}
