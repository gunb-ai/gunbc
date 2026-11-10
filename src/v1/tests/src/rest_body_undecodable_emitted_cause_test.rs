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
// FAKE SECRET IN EVERY BODY-DERIVED INPUT. The two body-derived inputs the arms could echo are the
// runtime io error (body read refused) and the serde_json message (decode refused). A credential
// carried in either reaches the published cause only if the emitted code splices the runtime
// `error` binding into it, so the guarantee is asserted on the emitted bytes: the emitted
// RestBodyUndecodable arms must carry the single declared-side authority's phrases as CONSTANT
// literals and must not splice any runtime error. The probe's request credential is planted with
// FAKE-SECRET-MARKER (mirroring the interpreter's t7 credential probe) so the same sentinel is in
// every body-derived input on this exchange.
//
// RED at main (the arms emitted the leaking expressions), GREEN at head (they route through
// v1.compiler.rest_cause_text).

const FAKE_SECRET_MARKER: &str = "FAKE-SECRET-MARKER";

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
    // <located> (<coproduct>)' template over declared facts -- not the serde message. Absent at
    // main (which emitted the '{}' format over `error`).
    assert!(
        emitted.contains(
            "body did not inhabit the declared output: the response body does not inhabit its \
             declared output type at"
        ),
        "the emitted decode arm did not publish the declared 'did not inhabit ... at ... (...)' \
         template\n--- emitted ---\n{emitted}"
    );
    // The shape the claims in dag/test/claim/rest_exchange_replay.dag pin: the located field and the
    // coproduct are present inside the 'at ... ( ... )' region.
    assert!(
        emitted.contains(" at ") && emitted.contains(" (") && emitted.contains(")"),
        "the emitted decode cause lost the pinned '... at <located> (<coproduct>)' shape\n--- emitted ---\n{emitted}"
    );

    // NO BODY-DERIVED TEXT REACHES THE CAUSE. Every RestBodyUndecodable arm must splice nothing from
    // the runtime error binding; the planted sentinel could ride out only through such a splice.
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
            assert!(
                !line.contains(FAKE_SECRET_MARKER),
                "the planted FAKE secret reached an emitted RestBodyUndecodable cause at line {idx}: {line}"
            );
        }
    }

    // And the planted sentinel must not appear on ANY emitted RestBodyUndecodable cause path at all.
    assert!(
        !emitted
            .lines()
            .any(|l| l.contains("RestBodyUndecodable") && l.contains(FAKE_SECRET_MARKER)),
        "the planted FAKE secret reached the emitted RestBodyUndecodable published cause"
    );
}
