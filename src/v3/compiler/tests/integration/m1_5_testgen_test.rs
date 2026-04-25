use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use v3_compiler::dag::{
    Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, PortState, TypeConnective,
    ValueBody,
};
use v3_compiler::lens_cost::cost_of;
use v3_compiler::lens_testgen::{GeneratedClaim, TestgenLens};
use v3_compiler::test_runner::{
    eval_algebraic_law_for_claim_program, evaluate_execute_command_m1_5,
    parse_execute_command_fields, AlgebraicLawProgramError, ExecuteCommandM1_5Proposition,
};
use v3_compiler::Diagnostic;

use crate::common::{cached_compile_any, cached_compile_outcome, CachedCompileOutcome};

#[derive(Clone)]
struct CachedGeneratedClaim {
    declaration_name: String,
    declaration_source: String,
    fields: Vec<(String, FieldValue)>,
}

impl CachedGeneratedClaim {
    fn declaration_name(&self) -> &str {
        &self.declaration_name
    }

    fn render_declaration_source(&self) -> &str {
        &self.declaration_source
    }

    fn fields(&self) -> &[(String, FieldValue)] {
        &self.fields
    }

    fn dag(&self) -> &Dag {
        bootstrapped_dag()
    }
}

impl From<GeneratedClaim<'_>> for CachedGeneratedClaim {
    fn from(claim: GeneratedClaim<'_>) -> Self {
        Self {
            declaration_name: claim.declaration_name().to_string(),
            declaration_source: claim.render_declaration_source(),
            fields: claim.fields().to_vec(),
        }
    }
}

fn compile_any(src: &str, file: &str) -> Dag {
    cached_compile_any(src, file)
}

fn bootstrapped_dag() -> &'static Dag {
    static DAG: OnceLock<Dag> = OnceLock::new();
    DAG.get_or_init(Dag::new)
}

fn generated_claims() -> &'static [CachedGeneratedClaim] {
    static CLAIMS: OnceLock<Vec<CachedGeneratedClaim>> = OnceLock::new();
    CLAIMS
        .get_or_init(|| {
            TestgenLens::new(bootstrapped_dag())
                .query()
                .into_iter()
                .map(CachedGeneratedClaim::from)
                .collect()
        })
        .as_slice()
}

fn generated_claim(name: &str) -> &'static CachedGeneratedClaim {
    generated_claims()
        .iter()
        .find(|claim| claim_name(claim) == name)
        .unwrap_or_else(|| panic!("generated claim `{name}` not found"))
}

fn generated_claim_decl<'a>(dag: &'a Dag, name: &str) -> &'a Declaration {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("generated claim declaration `{name}` not found"))
}

fn structural_fields(decl: &Declaration) -> &[(String, FieldValue)] {
    match &decl.value_body {
        Some(ValueBody::Structural { fields }) => fields.as_slice(),
        Some(ValueBody::Unparsed(_)) => panic!("generated claim should lower structurally"),
        Some(ValueBody::Scalar(_)) => {
            panic!("generated claim should lower as Structural (record shape), got Scalar")
        }
        None => panic!("generated claim declaration should carry a structural value body"),
    }
}

fn string_field(fields: &[(String, FieldValue)], label: &str) -> String {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected `{label}` to be a String literal field"))
}

fn variant_field<'a>(
    dag: &Dag,
    fields: &'a [(String, FieldValue)],
    label: &str,
) -> (String, &'a [FieldValue]) {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("expected `{label}` field"));
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected `{label}` to lower to a variant field");
    };
    (variant_label(dag, *constructor), payload.as_slice())
}

fn variant_label(dag: &Dag, variant_id: DeclarationId) -> String {
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == variant_id)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "variant declaration {:?} not found under any reflected sum",
                variant_id
            )
        })
}

fn variant_value<'a>(dag: &Dag, value: &'a FieldValue) -> (String, &'a [FieldValue]) {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected variant field value, got {value:?}");
    };
    (variant_label(dag, *constructor), payload.as_slice())
}

fn record_value(value: &FieldValue) -> &[(String, FieldValue)] {
    let FieldValue::Record(fields) = value else {
        panic!("expected record field value, got {value:?}");
    };
    fields.as_slice()
}

fn sum_variant(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
    payload: Vec<FieldValue>,
) -> FieldValue {
    let sum_decl = dag
        .declaration_by_name(sum_name)
        .unwrap_or_else(|| panic!("bootstrap should load `{sum_name}`"));
    let TypeConnective::Disj { variants } = &sum_decl.connective else {
        panic!("`{sum_name}` should lower to a Disj");
    };
    let constructor = variants
        .iter()
        .find(|variant| variant.label == variant_label)
        .map(|variant| variant.ty)
        .unwrap_or_else(|| panic!("variant `{variant_label}` not found under `{sum_name}`"));
    FieldValue::Variant {
        constructor,
        payload,
    }
}

fn diagnostic_detail_expectation(dag: &Dag, value: Option<&str>) -> FieldValue {
    match value {
        Some(text) => sum_variant(
            dag,
            "DiagnosticDetailExpectation",
            "Contains",
            vec![FieldValue::Literal(LiteralBits::String(text.to_string()))],
        ),
        None => sum_variant(dag, "DiagnosticDetailExpectation", "AnyDetail", Vec::new()),
    }
}

fn compiled_generated_claim(claim: &CachedGeneratedClaim) -> Dag {
    let dag = compile_any(
        claim.render_declaration_source(),
        "generated_test_claim.dag",
    );
    assert!(
        dag.diagnostics().is_empty(),
        "generated claim declaration should compile cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let decl = generated_claim_decl(&dag, claim.declaration_name());
    assert_eq!(
        decl.meta_tag,
        dag.declaration_by_name("TestClaim").map(|decl| decl.id),
        "generated data declaration should be typed as TestClaim"
    );
    let compiled_fields = structural_fields(decl);
    assert_eq!(string_field(compiled_fields, "name"), claim_name(claim));
    assert_eq!(
        string_field(compiled_fields, "source"),
        string_field(claim.fields(), "source")
    );
    assert_eq!(
        string_field(compiled_fields, "file_name"),
        string_field(claim.fields(), "file_name")
    );
    assert_eq!(
        variant_field(&dag, compiled_fields, "predicate").0,
        variant_field(claim.dag(), claim.fields(), "predicate").0
    );
    dag
}

fn claim_name(claim: &CachedGeneratedClaim) -> String {
    string_field(claim.fields(), "name")
}

fn runner_deferred_panic(label: &str) -> ! {
    panic!(
        "m1_5 testgen harness: TestPredicate::{label} is runner-deferred (not evaluable in this interpreter — do not treat as ordinary false)"
    )
}

/// `ExecuteCommand` is the **declared** host-process spawn boundary: everything else in this
/// harness stays data-only; arbitrary `command` + `args` use the same `std::process` path as
/// [`v3_compiler::test_runner::evaluate_execute_command_m1_5`].
///
/// Propositional **true** / **false** is **only** pass vs exit-code mismatch. Spawn errors,
/// timeouts, policy (`&` background), and other `Fail` or `NotYetImplemented` are **not** the
/// boolean “claim does not hold” — the harness **panics** and surfaces the full
/// `ClaimResult` (P3/DB-1: [`v3_compiler::test_runner::TestRunner`], `TESTING.md`).
fn execute_command_m1_5_holds(payload: &[FieldValue]) -> bool {
    let Some((command, args, expect_exit)) = parse_execute_command_fields(payload) else {
        panic!(
            "m1_5 testgen harness: ExecuteCommand payload malformed (cannot parse command/args/expect_exit_code) — do not treat as ordinary false"
        );
    };
    match evaluate_execute_command_m1_5(&command, &args, expect_exit) {
        Ok(ExecuteCommandM1_5Proposition::Satisfied) => true,
        Ok(ExecuteCommandM1_5Proposition::UnsatisfiedExitMismatch) => false,
        Err(r) => panic!(
            "m1_5 testgen harness: ExecuteCommand runner/claim outcome is not propositional (not exit mismatch): {r:?} — not ordinary false; see TestRunner / TESTING.md"
        ),
    }
}

fn claim_holds(claim: &CachedGeneratedClaim) -> bool {
    let source = string_field(claim.fields(), "source");
    let file_name = string_field(claim.fields(), "file_name");
    let predicate = claim
        .fields()
        .iter()
        .find(|(label, _)| label == "predicate")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("generated claim is missing `predicate`"));
    predicate_holds(claim.dag(), &source, &file_name, predicate)
}

fn predicate_holds(
    expectation_dag: &Dag,
    source: &str,
    file_name: &str,
    predicate: &FieldValue,
) -> bool {
    let (label, payload) = variant_value(expectation_dag, predicate);
    match label.as_str() {
        "Compiles" => cached_compile_outcome(source, file_name).is_clean(),
        "FailsWithDiagnostic" => {
            let [reference] = payload else {
                panic!("FailsWithDiagnostic payload should be a single DiagnosticReference");
            };
            match cached_compile_outcome(source, file_name) {
                CachedCompileOutcome::Clean(_) => false,
                CachedCompileOutcome::Semantic(dag) => {
                    diagnostic_matches(expectation_dag, &dag, reference)
                }
            }
        }
        "PortHasState" => {
            let [FieldValue::Literal(LiteralBits::String(bind_name)), expected_state] = payload
            else {
                panic!("PortHasState payload should be (String, PortStateExpectation)");
            };
            let dag = compile_any(source, file_name);
            let Some(bind) = dag.nodes().iter().find_map(|node| match node {
                Behavior::Bind(bind) if bind.name == *bind_name => Some(bind),
                _ => None,
            }) else {
                return false;
            };
            port_state_matches(
                expectation_dag,
                dag.port(bind.value).state(),
                expected_state,
            )
        }
        "CostBounded" => {
            let [FieldValue::Literal(LiteralBits::String(bind_name)), comparator, FieldValue::Literal(LiteralBits::Int(bound))] =
                payload
            else {
                panic!("CostBounded payload should be (String, ComparisonOp, Int)");
            };
            let dag = compile_any(source, file_name);
            let Some(bind) = dag.nodes().iter().find_map(|node| match node {
                Behavior::Bind(bind) if bind.name == *bind_name => Some(bind),
                _ => None,
            }) else {
                return false;
            };
            let actual = crate::common::require_fixture_cost_i64(
                cost_of(&dag, &bind.value),
                &format!("bind `{bind_name}`"),
            );
            compare_cost(
                expectation_dag,
                comparator,
                actual,
                i64::try_from(*bound).expect("testgen cost bound out of i64 range"),
            )
        }
        "OutputEquals" => runner_deferred_panic("OutputEquals"),
        "BehavioralObservation" => runner_deferred_panic("BehavioralObservation"),
        "MockBackedInvariant" => runner_deferred_panic("MockBackedInvariant"),
        "LensOutputEquals" => runner_deferred_panic("LensOutputEquals"),
        "DifferentialEquals" => runner_deferred_panic("DifferentialEquals"),
        "AlgebraicLaw" => {
            let inner = match cached_compile_outcome(source, file_name) {
                CachedCompileOutcome::Clean(dag) => dag,
                _ => return false,
            };
            match eval_algebraic_law_for_claim_program(expectation_dag, &inner, payload) {
                Ok(holds) => holds,
                Err(AlgebraicLawProgramError::UnsupportedLaw { .. }) => {
                    runner_deferred_panic("AlgebraicLaw")
                }
                Err(AlgebraicLawProgramError::MalformedPayload(message)) => panic!(
                    "m1_5 testgen harness: AlgebraicLaw payload malformed — do not treat as ordinary false: {message}"
                ),
            }
        }
        "ExecuteCommand" => {
            if !cached_compile_outcome(source, file_name).is_clean() {
                return false;
            }
            execute_command_m1_5_holds(payload)
        }
        "ForAllTargets" => runner_deferred_panic("ForAllTargets"),
        other => panic!("unsupported TestPredicate variant {other}"),
    }
}

fn catch_predicate_holds_panic_message(
    dag: &Dag,
    source: &str,
    file: &str,
    predicate: &FieldValue,
) -> String {
    let result = catch_unwind(AssertUnwindSafe(|| {
        predicate_holds(dag, source, file, predicate)
    }));
    assert!(
        result.is_err(),
        "expected predicate_holds to panic (harness fail-closed / runner-deferred path)"
    );
    let payload = result.unwrap_err();
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "non-string panic payload".to_string())
}

fn assert_runner_deferred_panics(
    dag: &Dag,
    source: &str,
    file: &str,
    predicate: &FieldValue,
    expected_label: &str,
) {
    let message = catch_predicate_holds_panic_message(dag, source, file, predicate);
    assert!(
        message.contains("runner-deferred") && message.contains(expected_label),
        "unexpected panic for {expected_label}: {message}"
    );
}

fn diagnostic_matches(expectation_dag: &Dag, actual_dag: &Dag, reference: &FieldValue) -> bool {
    let fields = record_value(reference);
    let kind = fields
        .iter()
        .find(|(label, _)| label == "kind")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("DiagnosticReference is missing `kind`"));
    let detail_contains = fields
        .iter()
        .find(|(label, _)| label == "detail_contains")
        .map(|(_, value)| diagnostic_detail_filter(expectation_dag, value))
        .unwrap_or_else(|| panic!("DiagnosticReference is missing `detail_contains`"));
    let (kind_label, kind_payload) = variant_value(expectation_dag, kind);
    assert!(
        kind_payload.is_empty(),
        "DiagnosticKind variants should be payload-free, got {kind_payload:?}"
    );
    actual_dag
        .diagnostics()
        .iter()
        .any(|(_, diag)| match_detail(kind_label.as_str(), &detail_contains, diag))
}

fn diagnostic_detail_filter(dag: &Dag, value: &FieldValue) -> Option<String> {
    let (label, payload) = variant_value(dag, value);
    match label.as_str() {
        "AnyDetail" => {
            assert!(
                payload.is_empty(),
                "AnyDetail should be payload-free, got {payload:?}"
            );
            None
        }
        "Contains" => {
            let [FieldValue::Literal(LiteralBits::String(text))] = payload else {
                panic!("Contains should carry a single String literal payload, got {payload:?}");
            };
            Some(text.clone())
        }
        other => panic!("unsupported DiagnosticDetailExpectation variant {other}"),
    }
}

fn match_detail(kind_label: &str, detail_contains: &Option<String>, diag: &Diagnostic) -> bool {
    if diagnostic_kind(diag) != kind_label {
        return false;
    }
    match detail_contains {
        Some(text) => diagnostic_detail(diag).contains(text),
        None => true,
    }
}

fn diagnostic_kind(diag: &Diagnostic) -> &'static str {
    match diag {
        Diagnostic::TokenizerError { .. } => "TokenizerError",
        Diagnostic::ParseError { .. } => "ParseError",
        Diagnostic::TypeMismatch { .. } => "TypeMismatch",
        Diagnostic::UnitMismatch { .. } => "UnitMismatch",
        Diagnostic::ArityMismatch { .. } => "ArityMismatch",
        Diagnostic::ResolveError { .. } => "ResolveError",
        Diagnostic::BranchConditionNotBool { .. } => "BranchConditionNotBool",
        Diagnostic::MagnitudeOutOfRange { .. } => "MagnitudeOutOfRange",
    }
}

fn diagnostic_detail(diag: &Diagnostic) -> String {
    match diag {
        Diagnostic::TokenizerError { message, .. } | Diagnostic::ParseError { message, .. } => {
            message.clone()
        }
        Diagnostic::TypeMismatch {
            expected, actual, ..
        } => format!("expected {expected:?}, got {actual:?}"),
        Diagnostic::ArityMismatch {
            function,
            expected,
            actual,
            ..
        } => format!("{function} expected {expected}, got {actual}"),
        Diagnostic::UnitMismatch { .. } => diag.message(),
        Diagnostic::ResolveError { name, .. } => name.clone(),
        Diagnostic::BranchConditionNotBool { .. } | Diagnostic::MagnitudeOutOfRange { .. } => {
            diag.message()
        }
    }
}

fn port_state_matches(
    expectation_dag: &Dag,
    actual: &PortState,
    expected_state: &FieldValue,
) -> bool {
    let (label, payload) = variant_value(expectation_dag, expected_state);
    assert!(
        payload.is_empty(),
        "PortStateExpectation variants should be payload-free, got {payload:?}"
    );
    matches!(
        (label.as_str(), actual),
        ("Resolved", PortState::Resolved(_)) | ("Unresolved", PortState::Unresolved)
    )
}

fn compare_cost(expectation_dag: &Dag, comparator: &FieldValue, actual: i64, bound: i64) -> bool {
    let (label, payload) = variant_value(expectation_dag, comparator);
    assert!(
        payload.is_empty(),
        "ComparisonOp variants should be payload-free, got {payload:?}"
    );
    match label.as_str() {
        "Eq" => actual == bound,
        "Ne" => actual != bound,
        "Lt" => actual < bound,
        "Le" => actual <= bound,
        "Gt" => actual > bound,
        "Ge" => actual >= bound,
        other => panic!("unsupported ComparisonOp variant {other}"),
    }
}

fn compiles_predicate(dag: &Dag) -> FieldValue {
    sum_variant(dag, "TestPredicate", "Compiles", Vec::new())
}

fn diagnostic_predicate(dag: &Dag, kind: &str, detail_contains: Option<&str>) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "FailsWithDiagnostic",
        vec![FieldValue::Record(vec![
            (
                String::from("kind"),
                sum_variant(dag, "DiagnosticKind", kind, Vec::new()),
            ),
            (
                String::from("detail_contains"),
                diagnostic_detail_expectation(dag, detail_contains),
            ),
        ])],
    )
}

fn port_state_predicate(dag: &Dag, bind_name: &str, state: &str) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "PortHasState",
        vec![
            FieldValue::Literal(LiteralBits::String(bind_name.to_string())),
            sum_variant(dag, "PortStateExpectation", state, Vec::new()),
        ],
    )
}

fn cost_bounded_predicate(dag: &Dag, bind_name: &str, comparator: &str, bound: i64) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "CostBounded",
        vec![
            FieldValue::Literal(LiteralBits::String(bind_name.to_string())),
            sum_variant(dag, "ComparisonOp", comparator, Vec::new()),
            FieldValue::Literal(LiteralBits::Int(i128::from(bound))),
        ],
    )
}

fn execute_command_predicate(dag: &Dag) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "ExecuteCommand",
        vec![
            FieldValue::Literal(LiteralBits::String(String::from("true"))),
            FieldValue::List(Vec::new()),
            FieldValue::Literal(LiteralBits::Int(0i128)),
        ],
    )
}

fn for_all_targets_predicate(dag: &Dag) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "ForAllTargets",
        vec![
            FieldValue::Literal(LiteralBits::String(String::from("true"))),
            FieldValue::List(Vec::new()),
            FieldValue::Literal(LiteralBits::Int(0i128)),
        ],
    )
}

fn declaration_ref_field(dag: &Dag, name: &str) -> FieldValue {
    let id = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("bootstrap should declare `{name}`"))
        .id;
    FieldValue::Reference(id)
}

fn executable_today(claim: &CachedGeneratedClaim) -> bool {
    let (predicate, payload) = variant_field(claim.dag(), claim.fields(), "predicate");
    if predicate != "FailsWithDiagnostic" {
        return true;
    }
    let [reference] = payload else {
        panic!("FailsWithDiagnostic payload should be a single DiagnosticReference");
    };
    let kind = record_value(reference)
        .iter()
        .find(|(label, _)| label == "kind")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("DiagnosticReference is missing `kind`"));
    variant_value(claim.dag(), kind).0 != "TypeMismatch"
}

fn assert_generated_claim_holds(claim_name: &str, expected_predicate: &str) {
    let claim = generated_claim(claim_name);
    assert_eq!(
        variant_field(claim.dag(), claim.fields(), "predicate").0,
        expected_predicate,
        "representative generated claim `{claim_name}` should cover the expected predicate family"
    );
    assert!(
        executable_today(claim),
        "representative generated claim `{claim_name}` should be executable in today's compiler"
    );
    assert!(
        claim_holds(claim),
        "generated claim should hold: name={claim_name}"
    );
}

#[test]
#[ignore = "slow exhaustive testgen sweep; excluded from required PR CI wall-clock gate"]
fn testgen_lens_emits_claims_as_structural_testclaim_values() {
    let dag = bootstrapped_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load std files cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let claims = generated_claims();
    for claim in claims {
        compiled_generated_claim(claim);
    }
    let claim_names: Vec<_> = claims.iter().map(claim_name).collect();
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestPredicate variant Compiles compiles"),
        "expected a compile claim for TestPredicate::Compiles, got {:?}",
        claim_names
    );
    assert!(
        claim_names.iter().any(|name| name == "TestClaim compiles"),
        "expected a compile claim for TestClaim"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "List<Int> variant Empty compiles"),
        "expected a compile claim for List<Int>::Empty"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "List<Int> requires exhaustive match"),
        "expected a non-exhaustive-match claim for List<Int>"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim witness resolves"),
        "expected a port-state claim for TestClaim witnesses, got {:?}",
        claim_names
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim witness has bounded cost"),
        "expected a cost-bounded claim for TestClaim witnesses, got {:?}",
        claim_names
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim rejects field type mismatch"),
        "expected a field-type-mismatch claim for TestClaim"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim mismatched witness stays unresolved"),
        "expected an unresolved port-state claim for TestClaim mismatches, got {:?}",
        claim_names
    );
}

#[test]
#[ignore = "slow exhaustive testgen sweep; excluded from required PR CI wall-clock gate"]
fn testgen_generated_claims_execute_against_compile_boundary() {
    let claims = generated_claims();
    assert!(
        !claims.is_empty(),
        "testgen lens should emit at least one claim against the bootstrapped stdlib"
    );
    for claim in claims.iter().filter(|claim| executable_today(claim)) {
        assert!(
            claim_holds(claim),
            "generated claim should hold: name={}",
            claim_name(claim)
        );
    }
}

#[test]
#[ignore = "bootstrapped testgen corpus query remains too expensive for required PR CI; keep as spot-check coverage only"]
fn representative_generated_claims_cover_predicate_families() {
    for (claim_name, expected_predicate) in [
        ("TestPredicate variant Compiles compiles", "Compiles"),
        ("List<Int> requires exhaustive match", "FailsWithDiagnostic"),
        ("TestClaim witness resolves", "PortHasState"),
        ("TestClaim witness has bounded cost", "CostBounded"),
    ] {
        let claim = generated_claim(claim_name);
        assert_eq!(
            variant_field(claim.dag(), claim.fields(), "predicate").0,
            expected_predicate,
            "representative generated claim `{claim_name}` should cover the expected predicate family"
        );
    }
}

#[test]
#[ignore = "bootstrapped testgen corpus query remains too expensive for required PR CI; keep as spot-check coverage only"]
fn representative_generated_claims_execute_against_compile_boundary() {
    for (claim_name, expected_predicate) in [
        ("TestPredicate variant Compiles compiles", "Compiles"),
        ("List<Int> requires exhaustive match", "FailsWithDiagnostic"),
        ("TestClaim witness has bounded cost", "CostBounded"),
    ] {
        assert_generated_claim_holds(claim_name, expected_predicate);
    }
}

#[test]
fn generic_predicate_interpreter_handles_representative_structural_predicates() {
    let dag = Dag::new();
    let positive_source = "type Box<T> { value: T }\nfn wrap(x: Int) -> Box<Int> = { value: x }\n";
    for (label, predicate) in [
        ("compiles", compiles_predicate(&dag)),
        (
            "resolved port-state",
            port_state_predicate(&dag, "wrap", "Resolved"),
        ),
        (
            "bounded cost",
            cost_bounded_predicate(&dag, "wrap", "Eq", 1),
        ),
    ] {
        assert!(
            predicate_holds(
                &dag,
                positive_source,
                "testgen_positive_representative_fixture.v3",
                &predicate,
            ),
            "expected representative positive predicate to hold: {label}"
        );
    }

    let negative_source = "fn broken(list: List<Int>) -> Bool = match list { Empty => true }\n";
    for (label, predicate) in [
        (
            "non-exhaustive diagnostic",
            diagnostic_predicate(&dag, "ResolveError", Some("non-exhaustive")),
        ),
        (
            "unresolved port-state",
            port_state_predicate(&dag, "broken", "Unresolved"),
        ),
    ] {
        assert!(
            predicate_holds(
                &dag,
                negative_source,
                "testgen_negative_representative_fixture.v3",
                &predicate,
            ),
            "expected representative negative predicate to hold: {label}"
        );
    }
}

#[test]
fn extension_predicates_reach_interpreter_boundary() {
    let dag = Dag::new();
    let positive_source = "type Box<T> { value: T }\nfn wrap(x: Int) -> Box<Int> = { value: x }\n";
    let file = "extension_predicates_fixture.v3";

    assert!(
        predicate_holds(
            &dag,
            positive_source,
            file,
            &execute_command_predicate(&dag),
        ),
        "allowlisted ExecuteCommand should hold when the claim program compiles"
    );
    let echo_ok = sum_variant(
        &dag,
        "TestPredicate",
        "ExecuteCommand",
        vec![
            FieldValue::Literal(LiteralBits::String(String::from("echo"))),
            FieldValue::List(vec![FieldValue::Literal(LiteralBits::String(
                String::from("hi"),
            ))]),
            FieldValue::Literal(LiteralBits::Int(0)),
        ],
    );
    assert!(
        predicate_holds(&dag, positive_source, file, &echo_ok),
        "ExecuteCommand(echo, [\"hi\"], 0) should hold when the claim program compiles"
    );
    let disallowed_execute = sum_variant(
        &dag,
        "TestPredicate",
        "ExecuteCommand",
        vec![
            FieldValue::Literal(LiteralBits::String(String::from("true"))),
            FieldValue::List(Vec::new()),
            FieldValue::Literal(LiteralBits::Int(1)),
        ],
    );
    assert!(
        !predicate_holds(&dag, positive_source, file, &disallowed_execute),
        "exit-code mismatch: `true` exits 0, expected 1 — should not hold (bool false, not panic)"
    );
    let false_expects_zero = sum_variant(
        &dag,
        "TestPredicate",
        "ExecuteCommand",
        vec![
            FieldValue::Literal(LiteralBits::String(String::from("false"))),
            FieldValue::List(Vec::new()),
            FieldValue::Literal(LiteralBits::Int(0i128)),
        ],
    );
    assert!(
        !predicate_holds(&dag, positive_source, file, &false_expects_zero),
        "`false` exits non-zero: predicate should not hold when expect_exit is 0"
    );
    assert_runner_deferred_panics(
        &dag,
        positive_source,
        file,
        &for_all_targets_predicate(&dag),
        "ForAllTargets",
    );

    let lens = sum_variant(
        &dag,
        "TestPredicate",
        "LensOutputEquals",
        vec![
            declaration_ref_field(&dag, "Value"),
            declaration_ref_field(&dag, "Transform"),
            declaration_ref_field(&dag, "Bind"),
        ],
    );
    assert_runner_deferred_panics(&dag, positive_source, file, &lens, "LensOutputEquals");

    let diff = sum_variant(
        &dag,
        "TestPredicate",
        "DifferentialEquals",
        vec![
            declaration_ref_field(&dag, "Value"),
            declaration_ref_field(&dag, "Transform"),
            declaration_ref_field(&dag, "Bind"),
        ],
    );
    assert_runner_deferred_panics(&dag, positive_source, file, &diff, "DifferentialEquals");

    // Unsupported `AlgebraicLawKind` is classified by `AlgebraicLawProgramError` (not panic
    // message substrings). The M1.5 interpreter still panics on this path; the public helper
    // is the typed contract shared with the DB-15 `TestRunner` lane (`NotYetImplemented`).
    let algebraic_law_payload = vec![
        sum_variant(&dag, "AlgebraicLawKind", "Commutativity", Vec::new()),
        declaration_ref_field(&dag, "Value"),
    ];
    let inner = match cached_compile_outcome(positive_source, file) {
        CachedCompileOutcome::Clean(program_dag) => program_dag,
        other => panic!(
            "extension_predicates fixture should compile cleanly for AlgebraicLaw probe, got {other:?}"
        ),
    };
    assert_eq!(
        eval_algebraic_law_for_claim_program(&dag, &inner, &algebraic_law_payload),
        Err(AlgebraicLawProgramError::UnsupportedLaw {
            law_label: "Commutativity".to_string(),
        })
    );
    let law = sum_variant(
        &dag,
        "TestPredicate",
        "AlgebraicLaw",
        algebraic_law_payload.clone(),
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            predicate_holds(&dag, positive_source, file, &law);
        }))
        .is_err(),
        "M1.5 harness should panic fail-closed on runner-deferred AlgebraicLaw"
    );

    let behavioral = sum_variant(
        &dag,
        "TestPredicate",
        "BehavioralObservation",
        vec![
            declaration_ref_field(&dag, "Value"),
            declaration_ref_field(&dag, "Transform"),
            declaration_ref_field(&dag, "Bind"),
        ],
    );
    assert_runner_deferred_panics(
        &dag,
        positive_source,
        file,
        &behavioral,
        "BehavioralObservation",
    );

    let mock = sum_variant(
        &dag,
        "TestPredicate",
        "MockBackedInvariant",
        vec![
            declaration_ref_field(&dag, "Value"),
            declaration_ref_field(&dag, "Transform"),
        ],
    );
    assert_runner_deferred_panics(&dag, positive_source, file, &mock, "MockBackedInvariant");

    let output_equals = sum_variant(
        &dag,
        "TestPredicate",
        "OutputEquals",
        vec![FieldValue::Literal(LiteralBits::String(String::from(
            "let x: Int = 1",
        )))],
    );
    assert_runner_deferred_panics(&dag, positive_source, file, &output_equals, "OutputEquals");
}
