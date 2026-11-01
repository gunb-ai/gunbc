// A FIELD READ OFF AN OPTIONAL REALIZES THE CHECKER'S DECLARED UNWRAP.
//
// `04_lookup` `field_summary_for_type` styles `.value` on an optional `OptionalUnwrap` and every
// other field read off an optional with value shape `OptionalValue`; the interpreter peels
// `Present { value }` and refuses `Absent` with a typed error. A `.dag` claim cannot observe a
// runtime refusal (it can only be true or false), so the RED is this test: real `.dag` source goes
// through the real resolve, check and interpret route (`run_claim_route`), because the fix keys on
// the checker's `value_shape` and a hand-built tree would skip the very thing being judged.
//
// THE CONTROL: the same source over a non-empty list reads the field and holds, so a refusal that
// fires on every field read cannot pass the RED.

const SOURCE: &str = "module test.claim.zz_scratch\n\
type ZzRow = ZzRow { status: String }\n\
data zz_rows: List<ZzRow> = [ZzRow { status: \"ok\" }]\n\
data zz_none: List<ZzRow> = []\n\
data zz_map: Map<String, ZzRow> = {\"k\": ZzRow { status: \"ok\" }}\n\
test fn zz_present_read() -> Bool {\n  zz_rows.first().status == \"ok\"\n}\n\
test fn zz_absent_read() -> Bool {\n  zz_none.first().status == \"ok\"\n}\n\
test fn zz_map_present_read() -> Bool {\n  zz_map.get(\"k\").status == \"ok\"\n}\n\
test fn zz_map_absent_read() -> Bool {\n  zz_map.get(\"z\").status == \"ok\"\n}\n\
test fn zz_map_match() -> Bool {\n  match zz_map.get(\"k\") { Present { value: v } => v.status == \"ok\"  Absent => false }\n}\n";

fn outcomes() -> Vec<v1_compiler::cli_run::ClaimRouteMember> {
    let dir = v1_compiler::cli_run::workspace_root()
        .join("target")
        .join("gunbc_interp_field_read_off_optional");
    let _ = std::fs::remove_dir_all(&dir);
    let claim_dir = dir.join("test").join("claim");
    std::fs::create_dir_all(&claim_dir).expect("scratch claim dir");
    std::fs::write(claim_dir.join("zz_scratch.dag"), SOURCE).expect("scratch claim source");
    std::env::set_current_dir(v1_compiler::cli_run::workspace_root()).expect("enter workspace");
    let mut roots = vec![dir.to_string_lossy().into_owned()];
    roots.extend(v1_compiler::cli_run::witness_layer_roots());
    v1_compiler::cli_run::run_claim_route(
        &roots,
        &|module| module == "test.claim.zz_scratch",
        &|_, _| true,
    )
    .expect("the scratch claims must route")
}

#[test]
fn a_field_read_off_a_present_first_holds_and_off_an_absent_first_refuses() {
    use v1_compiler::cli_run::ClaimOutcome;
    let members = outcomes();
    let find = |f: &str| {
        members
            .iter()
            .find(|m| m.function == f)
            .unwrap_or_else(|| panic!("claim {f} not routed"))
    };
    assert!(
        matches!(find("zz_present_read").outcome, ClaimOutcome::Pass),
        "CONTROL: a field read off a Present first must peel and hold"
    );
    match &find("zz_absent_read").outcome {
        ClaimOutcome::RuntimeError { message, .. } => assert!(
            message.contains("read off an Absent optional"),
            "must be the typed refusal, got: {message}"
        ),
        other => panic!("RED: a field read off an Absent first must refuse, got {other:?}"),
    }
    assert!(
        matches!(find("zz_map_present_read").outcome, ClaimOutcome::Pass),
        "CONTROL: a field read off a Present map get must peel and hold"
    );
    assert!(
        matches!(find("zz_map_match").outcome, ClaimOutcome::Pass),
        "CONTROL: match on map get answers Present/Absent by value"
    );
    match &find("zz_map_absent_read").outcome {
        ClaimOutcome::RuntimeError { message, .. } => assert!(
            message.contains("read off an Absent optional"),
            "must be the typed refusal, got: {message}"
        ),
        other => panic!("RED: a field read off an Absent map get must refuse, got {other:?}"),
    }
}
