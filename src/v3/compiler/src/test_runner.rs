use crate::dag::{
    ArithmeticOp, ArrowBody, Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits,
    OperatorKind, PortId, PortState, TransformTarget, TypeConnective, ValueBody,
};
use crate::diagnostics::Diagnostic;
use crate::lens_cost::{cost_of, CostLookup};
use crate::{compile_to_dag, CompileError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimResult {
    Pass,
    Fail(String),
    NotYetImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvaluation {
    pub claim_name: String,
    pub result: ClaimResult,
}

/// Hermetic `AlgebraicLaw` evaluation against a compiled claim program (`program_dag`).
///
/// Today only `Associativity` is supported, and only for the canonical
/// `fn <name>(a: Int, b: Int) -> Int = a + b` witness shape used by the
/// `lens_composition_associative` R1 gate. `lens_ref` is a [`FieldValue::Reference`]
/// into `fixture_dag`; the runner resolves the **name** and looks up the same
/// name in `program_dag`.
pub fn eval_algebraic_law_for_claim_program(
    fixture_dag: &Dag,
    program_dag: &Dag,
    payload: &[FieldValue],
) -> Result<bool, String> {
    let (law, lens_ref) = algebraic_law_payload_fields(payload)?;
    let (law_label, law_payload) = variant_fields(fixture_dag, law)?;
    if law_label != "Associativity" {
        return Err(format!(
            "AlgebraicLaw::{law_label} is not evaluable in the Rust runner yet"
        ));
    }
    if !law_payload.is_empty() {
        return Err("Associativity should be payload-free".to_string());
    }
    let lens_name = declaration_ref_name(fixture_dag, lens_ref)?;
    let Some(target) = program_dag.declaration_by_name(&lens_name) else {
        return Ok(false);
    };
    Ok(declaration_is_binary_int_add_associativity_witness(
        program_dag,
        target,
    ))
}

#[derive(Debug, Clone)]
pub struct TestClaimValue {
    pub claim_name: String,
    pub source: String,
    pub file_name: String,
    pub predicate: FieldValue,
    pub requires: Vec<FieldValue>,
}

pub struct TestRunner<'a> {
    dag: &'a Dag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiagnosticDetailFilter {
    Any,
    Contains(String),
}

impl<'a> TestRunner<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    pub fn run_suite(&self, suite_name: &str) -> Vec<ClaimEvaluation> {
        let Some(suite) = self.dag.declaration_by_name(suite_name) else {
            return vec![ClaimEvaluation {
                claim_name: suite_name.to_string(),
                result: ClaimResult::Fail(format!("TestSuite `{suite_name}` not found")),
            }];
        };
        let Some(fields) = structural_fields(suite) else {
            return vec![ClaimEvaluation {
                claim_name: suite_name.to_string(),
                result: ClaimResult::Fail(format!("TestSuite `{suite_name}` is not structural")),
            }];
        };
        let Some(FieldValue::List(claims)) = field(fields, "claims") else {
            return vec![ClaimEvaluation {
                claim_name: suite_name.to_string(),
                result: ClaimResult::Fail(format!("TestSuite `{suite_name}` is missing `claims`")),
            }];
        };
        claims
            .iter()
            .map(|claim_ref| match claim_ref {
                FieldValue::Reference(id) => {
                    let decl = self.dag.declaration(*id);
                    match TestClaimValue::from_declaration(decl) {
                        Ok(claim) => self.run_claim(&claim),
                        Err(reason) => ClaimEvaluation {
                            claim_name: decl
                                .name
                                .clone()
                                .unwrap_or_else(|| format!("Declaration#{}", id.raw())),
                            result: ClaimResult::Fail(reason),
                        },
                    }
                }
                other => ClaimEvaluation {
                    claim_name: suite_name.to_string(),
                    result: ClaimResult::Fail(format!(
                        "TestSuite `{suite_name}` claim entry is not a reference: {other:?}"
                    )),
                },
            })
            .collect()
    }

    pub fn run_claim(&self, claim: &TestClaimValue) -> ClaimEvaluation {
        let result = if !claim.requires.is_empty() {
            ClaimResult::Fail(format!(
                "TestClaim `{}` declares {} resource requirement(s), but the Rust runner cannot materialize `requires` yet",
                claim.claim_name,
                claim.requires.len()
            ))
        } else {
            match self.variant_value(&claim.predicate) {
                Some((label, payload)) => match label.as_str() {
                    "Compiles" => self.eval_compiles(claim),
                    "FailsWithDiagnostic" => self.eval_fails_with_diagnostic(claim, &payload),
                    "OutputEquals" => self.eval_output_equals(claim, &payload),
                    "PortHasState" => self.eval_port_has_state(claim, &payload),
                    "CostBounded" => self.eval_cost_bounded(claim, &payload),
                    "AlgebraicLaw" => self.eval_algebraic_law(claim, &payload),
                    _ => ClaimResult::NotYetImplemented,
                },
                None => ClaimResult::Fail("predicate is not a structural variant".to_string()),
            }
        };
        ClaimEvaluation {
            claim_name: claim.claim_name.clone(),
            result,
        }
    }

    fn eval_compiles(&self, claim: &TestClaimValue) -> ClaimResult {
        match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(_) => ClaimResult::Pass,
            Err(CompileError::Semantic(_)) => {
                ClaimResult::Fail("compiled with diagnostics".to_string())
            }
            Err(err) => {
                ClaimResult::Fail(format!("compile failed before semantic analysis: {err:?}"))
            }
        }
    }

    fn eval_fails_with_diagnostic(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [reference] = payload else {
            return ClaimResult::Fail(
                "FailsWithDiagnostic payload should be a DiagnosticReference".to_string(),
            );
        };
        match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(_) => ClaimResult::Fail("source compiled cleanly".to_string()),
            Err(CompileError::Semantic(dag)) => match self.diagnostic_matches(&dag, reference) {
                Ok(true) => ClaimResult::Pass,
                Ok(false) => ClaimResult::Fail("expected diagnostic was not found".to_string()),
                Err(reason) => ClaimResult::Fail(reason),
            },
            Err(CompileError::Tokenize(diagnostic)) | Err(CompileError::Parse(diagnostic)) => {
                match self.diagnostic_matches_single(&diagnostic, reference) {
                    Ok(true) => ClaimResult::Pass,
                    Ok(false) => ClaimResult::Fail("expected diagnostic was not found".to_string()),
                    Err(reason) => ClaimResult::Fail(reason),
                }
            }
        }
    }

    fn eval_output_equals(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(expected))] = payload else {
            return ClaimResult::Fail("OutputEquals payload should be a String".to_string());
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(err) => return ClaimResult::Fail(format!("source did not compile: {err:?}")),
        };
        let Some(value) = dag
            .declarations()
            .iter()
            .find(|decl| decl.span.file == claim.file_name && decl.value_body.is_some())
            .and_then(|decl| decl.value_body.as_ref())
        else {
            return ClaimResult::Fail("no data declaration value found".to_string());
        };
        let actual = render_value_body(&dag, value);
        if actual == *expected {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("expected `{expected}`, got `{actual}`"))
        }
    }

    fn eval_port_has_state(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(bind_name)), expected_state] = payload else {
            return ClaimResult::Fail(
                "PortHasState payload should be (String, PortStateExpectation)".to_string(),
            );
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => dag,
            Err(err) => return ClaimResult::Fail(format!("source did not lower: {err:?}")),
        };
        let Some(bind) = find_bind(&dag, bind_name, &claim.file_name) else {
            return ClaimResult::Fail(format!("bind `{bind_name}` not found"));
        };
        let Some((label, payload)) = self.variant_value(expected_state) else {
            return ClaimResult::Fail("state expectation is not a variant".to_string());
        };
        if !payload.is_empty() {
            return ClaimResult::Fail("state expectation should not carry payload".to_string());
        }
        let matches = matches!(
            (label.as_str(), dag.port(bind.value).state()),
            ("Resolved", PortState::Resolved(_))
                | ("Unresolved", PortState::Uninferred | PortState::Unresolved)
        );
        if matches {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("bind `{bind_name}` state did not match `{label}`"))
        }
    }

    fn eval_algebraic_law(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(_)) => {
                return ClaimResult::Fail(
                    "claim program compiled with diagnostics (AlgebraicLaw requires a clean compile)"
                        .to_string(),
                );
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "claim program did not compile (AlgebraicLaw): {err:?}"
                ));
            }
        };
        match eval_algebraic_law_for_claim_program(self.dag, &program_dag, payload) {
            Ok(true) => ClaimResult::Pass,
            Ok(false) => ClaimResult::Fail(
                "AlgebraicLaw associativity witness not satisfied (expected binary Int `+`)"
                    .to_string(),
            ),
            Err(reason) => ClaimResult::Fail(reason),
        }
    }

    fn eval_cost_bounded(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        let [FieldValue::Literal(LiteralBits::String(bind_name)), comparator, FieldValue::Literal(LiteralBits::Int(bound))] =
            payload
        else {
            return ClaimResult::Fail(
                "CostBounded payload should be (String, ComparisonOp, Int)".to_string(),
            );
        };
        let dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(err) => return ClaimResult::Fail(format!("source did not compile: {err:?}")),
        };
        let Some(bind) = find_bind(&dag, bind_name, &claim.file_name) else {
            return ClaimResult::Fail(format!("bind `{bind_name}` not found"));
        };
        let actual = match cost_of(&dag, &bind.value) {
            CostLookup::FoundCost { _0 } => _0,
            CostLookup::MissingCost => {
                return ClaimResult::Fail(format!("missing cost for bind `{bind_name}`"));
            }
        };
        if self.compare_cost(comparator, actual, *bound) {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("cost {actual} did not satisfy bound {bound}"))
        }
    }

    fn diagnostic_matches(&self, actual_dag: &Dag, reference: &FieldValue) -> Result<bool, String> {
        let reference = self.diagnostic_reference(reference)?;
        Ok(actual_dag
            .diagnostics()
            .iter()
            .any(|(_, diagnostic)| diagnostic_matches_reference(diagnostic, &reference)))
    }

    fn diagnostic_matches_single(
        &self,
        diagnostic: &Diagnostic,
        reference: &FieldValue,
    ) -> Result<bool, String> {
        let reference = self.diagnostic_reference(reference)?;
        Ok(diagnostic_matches_reference(diagnostic, &reference))
    }

    fn diagnostic_reference(
        &self,
        reference: &FieldValue,
    ) -> Result<(String, DiagnosticDetailFilter), String> {
        let Some(fields) = record_fields(reference) else {
            return Err("DiagnosticReference payload should be a record".to_string());
        };
        let Some(kind) = field(fields, "kind") else {
            return Err("DiagnosticReference is missing `kind`".to_string());
        };
        let Some(detail_contains) = field(fields, "detail_contains") else {
            return Err("DiagnosticReference is missing `detail_contains`".to_string());
        };
        let Some((kind_label, kind_payload)) = self.variant_value(kind) else {
            return Err("DiagnosticReference `kind` is not a variant".to_string());
        };
        if !kind_payload.is_empty() {
            return Err("DiagnosticReference `kind` should not carry payload".to_string());
        }
        Ok((kind_label, self.detail_filter(detail_contains)?))
    }

    fn detail_filter(&self, value: &FieldValue) -> Result<DiagnosticDetailFilter, String> {
        let Some((label, payload)) = self.variant_value(value) else {
            return Err("DiagnosticDetailExpectation is not a variant".to_string());
        };
        match label.as_str() {
            "AnyDetail" => {
                if payload.is_empty() {
                    Ok(DiagnosticDetailFilter::Any)
                } else {
                    Err("AnyDetail should not carry payload".to_string())
                }
            }
            "Contains" => match payload.as_slice() {
                [FieldValue::Literal(LiteralBits::String(text))] => {
                    Ok(DiagnosticDetailFilter::Contains(text.clone()))
                }
                _ => Err("Contains should carry a single String payload".to_string()),
            },
            other => Err(format!(
                "unsupported DiagnosticDetailExpectation variant `{other}`"
            )),
        }
    }

    fn compare_cost(&self, comparator: &FieldValue, actual: i64, bound: i64) -> bool {
        let Some((label, payload)) = self.variant_value(comparator) else {
            return false;
        };
        if !payload.is_empty() {
            return false;
        }
        match label.as_str() {
            "Eq" => actual == bound,
            "Lt" => actual < bound,
            "Le" => actual <= bound,
            "Gt" => actual > bound,
            "Ge" => actual >= bound,
            "Ne" => actual != bound,
            _ => false,
        }
    }

    fn variant_value(&self, value: &FieldValue) -> Option<(String, Vec<FieldValue>)> {
        match value {
            FieldValue::Variant {
                constructor,
                payload,
            } => Some((variant_label(self.dag, *constructor)?, payload.clone())),
            _ => None,
        }
    }
}

impl TestClaimValue {
    pub fn from_declaration(decl: &Declaration) -> Result<Self, String> {
        let fields = structural_fields(decl)
            .ok_or_else(|| "TestClaim declaration is not structural".to_string())?;
        let claim_name = string_field(fields, "name")?;
        let source = string_field(fields, "source")?;
        let file_name = string_field(fields, "file_name")?;
        let predicate = field(fields, "predicate")
            .ok_or_else(|| "TestClaim is missing `predicate`".to_string())?
            .clone();
        let requires = match field(fields, "requires") {
            Some(FieldValue::List(values)) => values.clone(),
            Some(other) => return Err(format!("TestClaim `requires` is not a list: {other:?}")),
            None => return Err("TestClaim is missing `requires`".to_string()),
        };
        Ok(Self {
            claim_name,
            source,
            file_name,
            predicate,
            requires,
        })
    }
}

fn structural_fields(decl: &Declaration) -> Option<&[(String, FieldValue)]> {
    match decl.value_body.as_ref()? {
        ValueBody::Structural { fields } => Some(fields),
        ValueBody::Unparsed(_) | ValueBody::Scalar(_) => None,
    }
}

fn field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|(candidate, _)| candidate == label)
        .map(|(_, value)| value)
}

fn string_field(fields: &[(String, FieldValue)], label: &str) -> Result<String, String> {
    match field(fields, label) {
        Some(FieldValue::Literal(LiteralBits::String(value))) => Ok(value.clone()),
        Some(other) => Err(format!("TestClaim `{label}` is not a string: {other:?}")),
        None => Err(format!("TestClaim is missing `{label}`")),
    }
}

fn record_fields(value: &FieldValue) -> Option<&[(String, FieldValue)]> {
    match value {
        FieldValue::Record(fields) => Some(fields),
        _ => None,
    }
}

fn find_bind<'a>(
    dag: &'a Dag,
    bind_name: &str,
    claim_file_name: &str,
) -> Option<&'a crate::dag::BindNode> {
    dag.nodes().iter().find_map(|node| match node {
        Behavior::Bind(bind) if bind.name == bind_name && bind.span.file == claim_file_name => {
            Some(bind)
        }
        _ => None,
    })
}

fn diagnostic_kind(diagnostic: &Diagnostic) -> &'static str {
    match diagnostic {
        Diagnostic::TokenizerError { .. } => "TokenizerError",
        Diagnostic::ParseError { .. } => "ParseError",
        Diagnostic::TypeMismatch { .. } => "TypeMismatch",
        Diagnostic::ArityMismatch { .. } => "ArityMismatch",
        Diagnostic::ResolveError { .. } => "ResolveError",
        Diagnostic::BranchConditionNotBool { .. } => "BranchConditionNotBool",
    }
}

fn diagnostic_matches_reference(
    diagnostic: &Diagnostic,
    reference: &(String, DiagnosticDetailFilter),
) -> bool {
    diagnostic_kind(diagnostic) == reference.0
        && match &reference.1 {
            DiagnosticDetailFilter::Any => true,
            DiagnosticDetailFilter::Contains(text) => diagnostic.message().contains(text),
        }
}

fn render_value_body(dag: &Dag, value: &ValueBody) -> String {
    match value {
        ValueBody::Scalar(bits) => render_literal(bits),
        ValueBody::Structural { fields } => render_record(dag, fields),
        ValueBody::Unparsed(span) => format!("<unparsed:{}:{}>", span.file, span.byte_start),
    }
}

fn render_field_value(dag: &Dag, value: &FieldValue) -> String {
    match value {
        FieldValue::Literal(bits) => render_literal(bits),
        FieldValue::Reference(decl_id) => dag
            .declaration(*decl_id)
            .name
            .clone()
            .unwrap_or_else(|| format!("Declaration#{}", decl_id.raw())),
        FieldValue::Record(fields) => render_record(dag, fields),
        FieldValue::List(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| render_field_value(dag, value))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        FieldValue::Variant {
            constructor,
            payload,
        } => {
            let label = variant_label(dag, *constructor)
                .unwrap_or_else(|| format!("Variant#{}", constructor.raw()));
            if payload.is_empty() {
                label
            } else {
                format!(
                    "{}({})",
                    label,
                    payload
                        .iter()
                        .map(|value| render_field_value(dag, value))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn render_record(dag: &Dag, fields: &[(String, FieldValue)]) -> String {
    format!(
        "{{ {} }}",
        fields
            .iter()
            .map(|(label, value)| format!("{label}: {}", render_field_value(dag, value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_literal(bits: &LiteralBits) -> String {
    match bits {
        LiteralBits::Int(value) => value.to_string(),
        LiteralBits::Bool(value) => value.to_string(),
        LiteralBits::String(value) => quote_string(value),
    }
}

fn quote_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn variant_label(dag: &Dag, variant_id: DeclarationId) -> Option<String> {
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == variant_id)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
}

fn algebraic_law_payload_fields(
    payload: &[FieldValue],
) -> Result<(&FieldValue, &FieldValue), String> {
    match payload {
        [law, lens_ref] => Ok((law, lens_ref)),
        [FieldValue::Record(fields)] => {
            let law = field(fields, "law")
                .ok_or_else(|| "AlgebraicLaw payload record is missing `law` field".to_string())?;
            let lens_ref = field(fields, "lens_ref").ok_or_else(|| {
                "AlgebraicLaw payload record is missing `lens_ref` field".to_string()
            })?;
            Ok((law, lens_ref))
        }
        _ => Err(format!(
            "AlgebraicLaw payload should be [law, lens_ref] or a record, got len {}",
            payload.len()
        )),
    }
}

fn variant_fields<'a>(
    dag: &Dag,
    value: &'a FieldValue,
) -> Result<(String, &'a [FieldValue]), String> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err("expected AlgebraicLawKind variant".to_string());
    };
    let label = variant_label(dag, *constructor).ok_or_else(|| {
        format!(
            "variant constructor {:?} not found under any sum",
            constructor
        )
    })?;
    Ok((label, payload.as_slice()))
}

fn declaration_ref_name(dag: &Dag, value: &FieldValue) -> Result<String, String> {
    match value {
        FieldValue::Reference(id) => dag
            .declaration(*id)
            .name
            .clone()
            .ok_or_else(|| format!("lens_ref declaration {:?} is anonymous", id)),
        other => Err(format!(
            "lens_ref should be a DeclarationRef (FieldValue::Reference), got {other:?}"
        )),
    }
}

fn declaration_is_binary_int_add_associativity_witness(dag: &Dag, decl: &Declaration) -> bool {
    let TypeConnective::Arrow {
        inputs,
        output,
        body,
    } = &decl.connective
    else {
        return false;
    };
    let Some(int_decl) = dag.declaration_by_name("Int") else {
        return false;
    };
    let int_id = int_decl.id;
    if inputs.len() != 2 || inputs[0] != int_id || inputs[1] != int_id || *output != int_id {
        return false;
    }
    let ArrowBody::UserDefined(root) = body else {
        return false;
    };
    let Behavior::Bind(bind) = dag.node(*root) else {
        return false;
    };
    if bind.params.len() != 2 {
        return false;
    }
    let Some(producer) = dag.resolve_producer_opt(&bind.value) else {
        return false;
    };
    let Behavior::Transform(transform) = producer else {
        return false;
    };
    if !matches!(
        transform.target,
        TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add))
    ) {
        return false;
    }
    if transform.inputs.len() != 2 {
        return false;
    }
    same_port_id_set(&bind.params, &transform.inputs)
}

fn same_port_id_set(params: &[PortId], inputs: &[PortId]) -> bool {
    if params.len() != inputs.len() {
        return false;
    }
    let mut a: Vec<u32> = params.iter().map(|p| p.raw()).collect();
    let mut b: Vec<u32> = inputs.iter().map(|p| p.raw()).collect();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}
