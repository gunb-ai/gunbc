use crate::dag::{
    Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, PortId, PortState,
    TypeConnective, ValueBody,
};
use crate::diagnostics::Diagnostic;
use crate::lens_apply::{
    apply_lens_declaration, field_value_from_value_body, int_associativity_holds_all_triples,
    reflect_program_dag_nodes_in_file, ASSOCIATIVITY_WITNESS_TRIPLES,
};
use crate::lens_cost::{cost_of, CostLookup};
use crate::{compile_to_dag, CompileError};

/// Same on-disk lens as `v3-compiler/build.rs` splices into `user_authored_lens_compiles_gate`
/// (`emit_r1_gates_fixture`). `LensOutputEquals` applies this program for `named_function_count`
/// so evaluation cannot drift from the fixture-local stub (`INVARIANTS.md` P2).
///
/// **Dissolution:** remove this `include_str!` bridge when `DeclarationRef` (or an equivalent
/// substrate edge) resolves executable lens bodies from `program_dag` / `TestClaim.source` so the
/// runner does not key a second `Dag` on fixture declaration spelling.
pub const R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../lenses/named_function_count.dag"
));

/// Same on-disk lens as `src/v3/lenses/complexity.dag`. `LensOutputEquals(cost_of, …)` applies
/// [`crate::lens_cost::cost_of`] on the compiled claim program (T-LaneE) — not the fixture-local
/// `fn cost_of` stub body (`INVARIANTS.md` P2).
pub const R1_CANONICAL_COMPLEXITY_LENS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../lenses/complexity.dag"
));

/// Bind whose `value` port receives structural `cost_of` for `LensOutputEquals` / `DifferentialEquals`.
///
/// Today the runner keys this on `TestClaim.file_name` until `DeclarationRef` can name the bind
/// directly (M1(2.8) — same story as `r1_lens_output_input_from_program`).
fn cost_bind_for_claim_file(file_name: &str) -> Option<&'static str> {
    match file_name {
        "r1_merge_sort_pair.v3" => Some("merge_sort_out"),
        "fixture_compiler_nerd_canonical_complexity.v3" => Some("complexity_demo_out"),
        "fixture_compiler_nerd_canonical_parallelism.v3" => Some("total"),
        _ => None,
    }
}

fn compile_r1_canonical_complexity_lens_dag() -> Result<Dag, String> {
    match compile_to_dag(R1_CANONICAL_COMPLEXITY_LENS, "src/v3/lenses/complexity.dag") {
        Ok(dag) if dag.diagnostics().is_empty() => Ok(dag),
        Ok(dag) => Err(format!(
            "canonical `complexity.dag` has diagnostics: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        )),
        Err(CompileError::Semantic(dag)) => Err(format!(
            "canonical `complexity.dag` failed inference: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        )),
        Err(err) => Err(format!("canonical `complexity.dag` did not compile: {err:?}")),
    }
}

/// Lower a `Lookup<Int>` `FieldValue` from the D1 lens interpreter into [`CostLookup`].
///
/// Constructor ids are resolved against `lens_dag` (the canonical `complexity.dag` compile).
fn cost_lookup_from_int_lookup_field_value(
    lens_dag: &Dag,
    value: &FieldValue,
) -> Result<CostLookup, String> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(format!(
            "expected `Lookup<Int>` variant from D1 `cost_of`, got {value:?}"
        ));
    };
    let label = variant_label(lens_dag, *constructor).ok_or_else(|| {
        format!(
            "unknown `Lookup<Int>` constructor id {}",
            constructor.raw()
        )
    })?;
    match label.as_str() {
        "Miss" => {
            if !payload.is_empty() {
                return Err("`Miss` should be payload-free".to_string());
            }
            Ok(CostLookup::Miss)
        }
        "Hit" => match payload.as_slice() {
            [FieldValue::Literal(LiteralBits::Int(i))] => Ok(CostLookup::Hit(*i)),
            _ => Err("`Hit` should carry a single Int payload".to_string()),
        },
        other => Err(format!(
            "expected `Miss` or `Hit` for `Lookup<Int>`, got `{other}`"
        )),
    }
}

/// T-LaneE `DifferentialEquals` cost lineage: **v3** = D1 `apply_lens_declaration` on the canonical
/// `.dag` `cost_of`; **v2** = Rust-generated [`cost_of`] (`lens_cost_generated`). These are
/// independent producers — the equality check can fail if they diverge (P3 / api-review #764).
fn eval_lane_e_differential_cost_lineage(
    lineage_name: &str,
    program_dag: &Dag,
    claim_file_name: &str,
    bind_port: PortId,
    lens_dag: &Dag,
) -> Result<CostLookup, String> {
    match lineage_name {
        "v3_program_cost" => {
            let Some(cost_decl) = lens_dag.declaration_by_name("cost_of") else {
                return Err("canonical complexity lens missing `cost_of` declaration".to_string());
            };
            let reflected = reflect_program_dag_nodes_in_file(
                program_dag,
                claim_file_name,
                lens_dag,
            )
            .map_err(|e| format!("reflect claim program for v3 lineage: {e:?}"))?;
            let port_arg = FieldValue::Literal(LiteralBits::Int(i64::from(bind_port.raw())));
            let fv = apply_lens_declaration(lens_dag, cost_decl.id, &[reflected, port_arg])
                .map_err(|e| format!("D1 apply of canonical `cost_of` (v3 lineage): {e:?}"))?;
            cost_lookup_from_int_lookup_field_value(lens_dag, &fv)
        }
        "v2_oracle_cost" => Ok(cost_of(program_dag, &bind_port)),
        _ => Err(format!(
            "unsupported lineage `{lineage_name}` for T-LaneE `DifferentialEquals` cost (expected `v3_program_cost` or `v2_oracle_cost`)"
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimResult {
    Pass,
    Fail(String),
    /// Runner does not implement this path yet; message is surfaced to tests and logs.
    NotYetImplemented(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvaluation {
    pub claim_name: String,
    pub result: ClaimResult,
}

/// Typed failure modes for [`eval_algebraic_law_for_claim_program`] (C-5: no string
/// sub-match on `Err` to classify behavior — discriminate on this enum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlgebraicLawProgramError {
    /// Law kind is not implemented in the public helper (M1.5 harness: treat as runner-deferred).
    UnsupportedLaw { law_label: String },
    /// Predicate payload or referenced structure is invalid for evaluation.
    MalformedPayload(String),
}

/// Hermetic `AlgebraicLaw` evaluation against a compiled claim program (`program_dag`).
///
/// **`Associativity` — bounded operational witness (T-LensAPI D3), not substrate law proof:**
/// uses [`int_associativity_holds_all_triples`](crate::lens_apply::int_associativity_holds_all_triples)
/// over [`ASSOCIATIVITY_WITNESS_TRIPLES`](crate::lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES) so a
/// single lucky `(a,b,c)` cannot certify a false law. This path does **not** consume quantified
/// associativity facts declared on `OrderedRing` / semigroup
/// carriers in `std.algebra` (those are not yet first-class runner inputs). Treating `Pass` here
/// as full algebraic law evidence would be weaker than a substrate-backed law check — the R1
/// gate is intentionally a **regression harness** that the witness lens behaves associatively on
/// the full witness set, not a proof for all `Int`. **Dissolution:** wire `AlgebraicLaw` to declared law
/// metadata / witnesses on disk and reserve sample-only checks to explicit testgen predicates, or
/// return [`ClaimResult::NotYetImplemented`] until that substrate surface exists.
///
/// `lens_ref` is a [`FieldValue::Reference`] into `fixture_dag`; the runner resolves the **name**
/// and looks up the same name in `program_dag`.
pub fn eval_algebraic_law_for_claim_program(
    fixture_dag: &Dag,
    program_dag: &Dag,
    payload: &[FieldValue],
) -> Result<bool, AlgebraicLawProgramError> {
    let (law, lens_ref) = algebraic_law_payload_fields(payload)?;
    let (law_label, law_payload) = variant_fields(fixture_dag, law)?;
    if law_label != "Associativity" {
        return Err(AlgebraicLawProgramError::UnsupportedLaw { law_label });
    }
    if !law_payload.is_empty() {
        return Err(AlgebraicLawProgramError::MalformedPayload(
            "Associativity should be payload-free".to_string(),
        ));
    }
    let lens_name = declaration_ref_name(fixture_dag, lens_ref)?;
    let Some(target) = program_dag.declaration_by_name(&lens_name) else {
        return Ok(false);
    };
    int_associativity_holds_all_triples(program_dag, target.id, ASSOCIATIVITY_WITNESS_TRIPLES)
        .map_err(|e| AlgebraicLawProgramError::MalformedPayload(format!("lens apply error: {e:?}")))
}

/// Compile-time ratchet (PR #741 / codex P1): `Associativity` must not regress to checking one
/// lucky `(a, b, c)` triple — the gate is a correctness signal only when the witness set has
/// material breadth (see `lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES`).
const _: () = assert!(ASSOCIATIVITY_WITNESS_TRIPLES.len() > 1);

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
                    "LensOutputEquals" => self.eval_lens_output_equals(claim, &payload),
                    "DifferentialEquals" => self.eval_differential_equals(claim, &payload),
                    "AlgebraicLaw" => self.eval_algebraic_law(claim, &payload),
                    "MockBackedInvariant" => self.eval_mock_backed_invariant(claim, &payload),
                    other => ClaimResult::NotYetImplemented(format!(
                        "TestPredicate::{other} is not wired in the Rust runner yet"
                    )),
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

    fn eval_lens_output_equals(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [lens_fv, input_fv, expected_fv] = payload else {
            return ClaimResult::Fail(format!(
                "LensOutputEquals payload should be exactly three DeclarationRef fields \
                 (lens_ref, input_ref, expected_ref); got {} payload slot(s)",
                payload.len()
            ));
        };
        let lens_id = match self.resolve_declaration_ref_id(lens_fv, "lens_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let input_id = match self.resolve_declaration_ref_id(input_fv, "input_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let expected_id = match self.resolve_declaration_ref_id(expected_fv, "expected_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        let lens_decl = self.dag.declaration(lens_id);
        let input_decl = self.dag.declaration(input_id);
        let expected_decl = self.dag.declaration(expected_id);

        let lens_name = decl_display_name(lens_id, lens_decl);
        let input_name = decl_display_name(input_id, input_decl);
        let expected_name = decl_display_name(expected_id, expected_decl);

        // R1 gate sentinel: `Dag` inputs are not yet expressible as structural `data` bodies in the
        // fixture DSL; `r1_lens_output_input_from_program` names a typed placeholder while the
        // runner reflects `Dag.nodes` from `TestClaim.source` / `file_name`.
        // **Dissolution trigger (ROADMAP / INVARIANTS P2):** replace string matching on this name
        // with a structural `TestClaim` / `std.verification` coproduct arm (reflection input vs
        // literal body) so runners do not key behavior on declaration spellings.
        const PROGRAM_INPUT_SENTINEL: &str = "r1_lens_output_input_from_program";

        if input_decl.value_body.is_none() {
            return ClaimResult::Fail(format!(
                "LensOutputEquals: input_ref `{input_name}` has no value body"
            ));
        }
        if expected_decl.value_body.is_none() {
            return ClaimResult::Fail(format!(
                "LensOutputEquals: expected_ref `{expected_name}` has no value body"
            ));
        }

        // INVARIANTS P2 (executable single authority): `DeclarationRef` for `lens_ref` still
        // resolves against the fixture `Dag` for lowering, but for `named_function_count` the
        // runner compiles `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` (same file as `build.rs` splices
        // for `user_authored_lens_compiles_gate`) for `apply_lens_declaration` — not the
        // fixture-local stub body. Other lens names: if `TestClaim.source` exports the same
        // declaration name, apply that program; else fall back to the fixture graph.
        //
        // **Dissolution trigger (name-keyed bridge):** delete the `lens_decl.name ==
        // Some("named_function_count")` arm and this entire parallel authority when
        // `DeclarationRef` resolves lens executable identity from `program_dag` (or structured
        // `TestClaim` metadata) without fixture-local stub bodies — same upstream fix as retiring
        // `PROGRAM_INPUT_SENTINEL` string dispatch above.
        // INVARIANTS P3 / TESTING: `TestClaim.source` must lower cleanly — never ignore
        // tokenize/parse failures and fall back to the fixture graph (that would let malformed
        // programs `Pass` when inputs/lens resolve only from the fixture).
        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: claim `source` / `{}` failed inference: {:?}",
                    claim.file_name,
                    dag.diagnostics().iter().collect::<Vec<_>>()
                ));
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: claim `source` / `{}` did not compile: {err:?}",
                    claim.file_name
                ));
            }
        };

        // T-LaneE (`cost_of`): structural `Lookup<Int>` from the Rust-generated lens on the claim
        // program's `merge_sort_out` bind vs a fixture `Lookup<Int>` expected value.
        if lens_decl.name.as_deref() == Some("cost_of") {
            if input_decl.name.as_deref() != Some(PROGRAM_INPUT_SENTINEL) {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): input_ref must be `{PROGRAM_INPUT_SENTINEL}` sentinel, got `{input_name}`"
                ));
            }
            let Some(cost_bind) = cost_bind_for_claim_file(&claim.file_name) else {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): no structural-cost bind mapping for file `{}`",
                    claim.file_name
                ));
            };
            let Some(bind) = find_bind(&program_dag, cost_bind, &claim.file_name) else {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): bind `{cost_bind}` not found in `{}`",
                    claim.file_name
                ));
            };
            let computed = cost_of(&program_dag, &bind.value);
            // M1(2.8): `Lookup<Int>` is not yet structurally authorable in `data` bodies for this
            // fixture module — compare the lens `Hit(n)` against a scalar `Int` witness.
            let expected_int = match expected_decl.value_body.as_ref() {
                Some(ValueBody::Scalar(LiteralBits::Int(i))) => *i,
                _ => {
                    return ClaimResult::Fail(format!(
                        "LensOutputEquals(cost_of): expected_ref `{expected_name}` must be `data …: Int = <literal>` (M1(2.8); `Lookup<Int>` data literals are deferred)"
                    ));
                }
            };
            return match computed {
                CostLookup::Hit(v) if v == expected_int => ClaimResult::Pass,
                CostLookup::Hit(v) => ClaimResult::Fail(format!(
                    "LensOutputEquals(cost_of): expected `{expected_int}`, computed `{v}` for bind `{cost_bind}`"
                )),
                CostLookup::Miss => ClaimResult::Fail(
                    "LensOutputEquals(cost_of): computed cost is Miss (malformed program)".to_string(),
                ),
            };
        }

        // INVARIANTS P2: reflected `FieldValue` List / `Behavior` variant ids must come from the
        // same `Dag` as `apply_lens_declaration` (canonical `named_function_count` vs claim).
        let canonical_named_function_count_dag: Option<Dag> = if lens_decl.name.as_deref()
            == Some("named_function_count")
        {
            Some(
                match compile_to_dag(
                    R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS,
                    "src/v3/lenses/named_function_count.dag",
                ) {
                    Ok(dag) => dag,
                    Err(CompileError::Semantic(dag)) => {
                        return ClaimResult::Fail(format!(
                            "LensOutputEquals: canonical `named_function_count` lens failed inference: {:?}",
                            dag.diagnostics().iter().collect::<Vec<_>>()
                        ));
                    }
                    Err(err) => {
                        return ClaimResult::Fail(format!(
                            "LensOutputEquals: canonical `named_function_count` lens did not compile: {err:?}"
                        ));
                    }
                },
            )
        } else {
            None
        };

        let input_field = if input_decl.name.as_deref() == Some(PROGRAM_INPUT_SENTINEL) {
            // P2: `id_space` must be the same `Dag` `apply_lens_declaration` will use for the lens
            // (canonical compile, claim `program_dag`, or merged fixture `self.dag`) so reflected
            // `List` / `Behavior` variant `DeclarationId`s are not mixed across graphs.
            let id_space: &Dag = if let Some(ref cld) = canonical_named_function_count_dag {
                cld
            } else if let Some(name) = lens_decl.name.as_deref() {
                if program_dag.declaration_by_name(name).is_some() {
                    &program_dag
                } else {
                    self.dag
                }
            } else {
                self.dag
            };
            match reflect_program_dag_nodes_in_file(&program_dag, &claim.file_name, id_space) {
                Ok(v) => v,
                Err(err) => {
                    return ClaimResult::Fail(format!(
                        "LensOutputEquals: could not reflect `Dag` nodes from claim program: {err:?}"
                    ));
                }
            }
        } else {
            match &input_decl.value_body {
                Some(body) => match field_value_from_value_body(self.dag, body) {
                    Ok(v) => v,
                    Err(err) => {
                        return ClaimResult::Fail(format!(
                            "LensOutputEquals: could not lower input_ref `{input_name}` value: {err:?}"
                        ));
                    }
                },
                None => {
                    return ClaimResult::Fail(format!(
                        "LensOutputEquals: input_ref `{input_name}` has no value body (use `{PROGRAM_INPUT_SENTINEL}` sentinel when the input `Dag` is only available via `TestClaim.source`)"
                    ));
                }
            }
        };

        let expected_field = match field_value_from_value_body(
            self.dag,
            expected_decl.value_body.as_ref().expect("checked"),
        ) {
            Ok(v) => v,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: could not lower expected_ref `{expected_name}` value: {err:?}"
                ));
            }
        };

        let (lens_program, lens_apply_id) =
            if let Some(ref cld) = canonical_named_function_count_dag {
                let Some(d) = cld.declaration_by_name("named_function_count") else {
                    return ClaimResult::Fail(
                    "LensOutputEquals: canonical named_function_count lens missing root declaration"
                        .to_string(),
                );
                };
                (cld, d.id)
            } else if let Some(name) = lens_decl.name.as_deref() {
                match program_dag.declaration_by_name(name) {
                    Some(d) => (&program_dag, d.id),
                    None => (self.dag, lens_id),
                }
            } else {
                (self.dag, lens_id)
            };

        let computed = match apply_lens_declaration(
            lens_program,
            lens_apply_id,
            std::slice::from_ref(&input_field),
        ) {
            Ok(v) => v,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "LensOutputEquals: applying lens `{lens_name}` failed: {err:?}"
                ));
            }
        };

        if computed == expected_field {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "LensOutputEquals: expected {} for `{expected_name}`, computed {} for lens `{lens_name}` (input `{input_name}`)",
                render_field_value(self.dag, &expected_field),
                render_field_value(self.dag, &computed),
            ))
        }
    }

    fn resolve_declaration_ref_id(
        &self,
        value: &FieldValue,
        field_label: &str,
    ) -> Result<DeclarationId, String> {
        match value {
            FieldValue::Reference(id) => Ok(*id),
            FieldValue::Record(fields) if fields.is_empty() => Err(format!(
                "LensOutputEquals `{field_label}`: DeclarationRef is the empty record literal {{}} — use an identifier \
                 so lowering emits FieldValue::Reference(DeclarationId), not an empty record",
            )),
            other => Err(format!(
                "LensOutputEquals `{field_label}`: expected FieldValue::Reference(DeclarationId) \
                 for a DeclarationRef edge, got {other:?}"
            )),
        }
    }

    fn eval_differential_equals(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [subject_fv, oracle_fv, input_fv] = payload else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals payload should be exactly three DeclarationRef fields \
                 (subject_ref, oracle_ref, input_ref); got {} payload slot(s)",
                payload.len()
            ));
        };
        let subject_id = match self.resolve_declaration_ref_id(subject_fv, "subject_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let oracle_id = match self.resolve_declaration_ref_id(oracle_fv, "oracle_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let input_id = match self.resolve_declaration_ref_id(input_fv, "input_ref") {
            Ok(id) => id,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        let subject_decl = self.dag.declaration(subject_id);
        let oracle_decl = self.dag.declaration(oracle_id);
        let input_decl = self.dag.declaration(input_id);

        let subject_lineage = decl_display_name(subject_id, subject_decl);
        let oracle_lineage = decl_display_name(oracle_id, oracle_decl);
        let input_name = decl_display_name(input_id, input_decl);

        const PROGRAM_INPUT_SENTINEL: &str = "r1_lens_output_input_from_program";
        if input_decl.name.as_deref() != Some(PROGRAM_INPUT_SENTINEL) {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: input_ref must be `{PROGRAM_INPUT_SENTINEL}` sentinel, got `{input_name}`"
            ));
        }

        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => {
                return ClaimResult::Fail(format!(
                    "DifferentialEquals: claim `source` / `{}` failed inference: {:?}",
                    claim.file_name,
                    dag.diagnostics().iter().collect::<Vec<_>>()
                ));
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "DifferentialEquals: claim `source` / `{}` did not compile: {err:?}",
                    claim.file_name
                ));
            }
        };

        let Some(cost_bind) = cost_bind_for_claim_file(&claim.file_name) else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: no structural-cost bind mapping for file `{}`",
                claim.file_name
            ));
        };
        let Some(bind) = find_bind(&program_dag, cost_bind, &claim.file_name) else {
            return ClaimResult::Fail(format!(
                "DifferentialEquals: bind `{cost_bind}` not found in `{}`",
                claim.file_name
            ));
        };

        if subject_lineage == oracle_lineage {
            return ClaimResult::Fail(
                "DifferentialEquals: subject_ref and oracle_ref must name distinct lineages"
                    .to_string(),
            );
        }

        let pairing_ok = (subject_lineage.as_str() == "v3_program_cost"
            && oracle_lineage.as_str() == "v2_oracle_cost")
            || (subject_lineage.as_str() == "v2_oracle_cost"
                && oracle_lineage.as_str() == "v3_program_cost");
        if !pairing_ok {
            return ClaimResult::NotYetImplemented(format!(
                "DifferentialEquals(cost): only the (v3_program_cost, v2_oracle_cost) lineage pairing is implemented; got ({subject_lineage}, {oracle_lineage})"
            ));
        }

        let lens_dag = match compile_r1_canonical_complexity_lens_dag() {
            Ok(d) => d,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        let subject_out = match eval_lane_e_differential_cost_lineage(
            subject_lineage.as_str(),
            &program_dag,
            &claim.file_name,
            bind.value,
            &lens_dag,
        ) {
            Ok(v) => v,
            Err(msg) => return ClaimResult::Fail(msg),
        };
        let oracle_out = match eval_lane_e_differential_cost_lineage(
            oracle_lineage.as_str(),
            &program_dag,
            &claim.file_name,
            bind.value,
            &lens_dag,
        ) {
            Ok(v) => v,
            Err(msg) => return ClaimResult::Fail(msg),
        };

        if subject_out == oracle_out {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!(
                "DifferentialEquals: subject `{subject_lineage}` output {subject_out:?} != oracle `{oracle_lineage}` output {oracle_out:?} (v3 .dag D1 vs v2 Rust oracle)"
            ))
        }
    }

    fn eval_algebraic_law(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult {
        // Only `Associativity` is wired via D3 multi-triple operational witness (see
        // `eval_algebraic_law_for_claim_program` — not substrate law-fact evaluation).
        // Other `AlgebraicLawKind` variants are `NotYetImplemented` (runner cannot evaluate yet),
        // not `Fail` (claim false).
        let (law, _) = match algebraic_law_payload_fields(payload) {
            Ok(parts) => parts,
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => {
                return ClaimResult::Fail(message);
            }
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => unreachable!(
                "algebraic_law_payload_fields only yields MalformedPayload (got UnsupportedLaw({law_label:?}))"
            ),
        };
        let (law_label, law_payload) = match variant_fields(self.dag, law) {
            Ok(parts) => parts,
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => {
                return ClaimResult::Fail(message);
            }
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => unreachable!(
                "variant_fields only yields MalformedPayload (got UnsupportedLaw({law_label:?}))"
            ),
        };
        if law_label != "Associativity" {
            return ClaimResult::NotYetImplemented(format!(
                "AlgebraicLaw::{law_label} is not wired in the Rust runner yet"
            ));
        }
        if !law_payload.is_empty() {
            return ClaimResult::Fail("Associativity should be payload-free".to_string());
        }

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
            Ok(false) => ClaimResult::Fail(format!(
                "AlgebraicLaw Associativity: operational witness failed (must pass all {} fixed \
                 Int triples in lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES; D1 apply — not a \
                 substrate declared-law check; see eval_algebraic_law_for_claim_program)",
                ASSOCIATIVITY_WITNESS_TRIPLES.len()
            )),
            Err(AlgebraicLawProgramError::MalformedPayload(message)) => ClaimResult::Fail(message),
            Err(AlgebraicLawProgramError::UnsupportedLaw { law_label }) => unreachable!(
                "eval_algebraic_law gated on Associativity; helper cannot return UnsupportedLaw({law_label:?})"
            ),
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
            CostLookup::Hit(actual) => actual,
            CostLookup::Miss => {
                return ClaimResult::Fail(format!("missing cost for bind `{bind_name}`"));
            }
        };
        if self.compare_cost(comparator, actual, *bound) {
            ClaimResult::Pass
        } else {
            ClaimResult::Fail(format!("cost {actual} did not satisfy bound {bound}"))
        }
    }

    fn eval_mock_backed_invariant(
        &self,
        claim: &TestClaimValue,
        payload: &[FieldValue],
    ) -> ClaimResult {
        let [subject, invariant] = payload else {
            return ClaimResult::Fail(
                "MockBackedInvariant payload should be (subject: DeclarationRef, invariant: DeclarationRef)"
                    .to_string(),
            );
        };
        let subject_name = match self.resolve_mock_declaration_ref_edge(subject, "subject") {
            Ok(name) => name,
            Err(reason) => return ClaimResult::Fail(reason),
        };
        let invariant_name = match self.resolve_mock_declaration_ref_edge(invariant, "invariant") {
            Ok(name) => name,
            Err(reason) => return ClaimResult::Fail(reason),
        };

        let program_dag = match compile_to_dag(&claim.source, &claim.file_name) {
            Ok(dag) => dag,
            Err(CompileError::Semantic(dag)) => {
                return ClaimResult::Fail(format!(
                    "MockBackedInvariant: claim `source` / `{}` failed inference: {:?}",
                    claim.file_name,
                    dag.diagnostics().iter().collect::<Vec<_>>()
                ));
            }
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "MockBackedInvariant: claim `source` / `{}` did not compile: {err:?}",
                    claim.file_name
                ));
            }
        };

        let Some(subject_decl) = program_dag.declaration_by_name(&subject_name) else {
            return ClaimResult::Fail(format!(
                "MockBackedInvariant: subject `{subject_name}` not found in compiled claim program"
            ));
        };
        let Some(invariant_decl) = program_dag.declaration_by_name(&invariant_name) else {
            return ClaimResult::Fail(format!(
                "MockBackedInvariant: invariant `{invariant_name}` not found in compiled claim program"
            ));
        };

        let subject_out = match apply_lens_declaration(&program_dag, subject_decl.id, &[]) {
            Ok(v) => v,
            Err(err) => {
                return ClaimResult::Fail(format!(
                    "MockBackedInvariant: applying subject `{subject_name}` failed: {err:?}"
                ));
            }
        };
        match apply_lens_declaration(&program_dag, invariant_decl.id, &[subject_out]) {
            Ok(FieldValue::Literal(LiteralBits::Bool(true))) => ClaimResult::Pass,
            Ok(other) => ClaimResult::Fail(format!(
                "MockBackedInvariant: invariant `{invariant_name}` did not return Bool(true), got {other:?}"
            )),
            Err(err) => ClaimResult::Fail(format!(
                "MockBackedInvariant: applying invariant `{invariant_name}` failed: {err:?}"
            )),
        }
    }

    fn resolve_mock_declaration_ref_edge(
        &self,
        value: &FieldValue,
        label: &str,
    ) -> Result<String, String> {
        match value {
            FieldValue::Reference(id) => Ok(self
                .dag
                .declaration(*id)
                .name
                .clone()
                .unwrap_or_else(|| format!("Declaration#{}", id.raw()))),
            FieldValue::Record(fields) if fields.is_empty() => Err(format!(
                "MockBackedInvariant `{label}` must be a DeclarationRef edge, got empty record literal"
            )),
            other => Err(format!(
                "MockBackedInvariant `{label}` must be a DeclarationRef edge, got {other:?}"
            )),
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

fn decl_display_name(id: DeclarationId, decl: &Declaration) -> String {
    decl.name
        .clone()
        .unwrap_or_else(|| format!("Declaration#{}", id.raw()))
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
) -> Result<(&FieldValue, &FieldValue), AlgebraicLawProgramError> {
    match payload {
        [law, lens_ref] => Ok((law, lens_ref)),
        [FieldValue::Record(fields)] => {
            let law = field(fields, "law").ok_or_else(|| {
                AlgebraicLawProgramError::MalformedPayload(
                    "AlgebraicLaw payload record is missing `law` field".to_string(),
                )
            })?;
            let lens_ref = field(fields, "lens_ref").ok_or_else(|| {
                AlgebraicLawProgramError::MalformedPayload(
                    "AlgebraicLaw payload record is missing `lens_ref` field".to_string(),
                )
            })?;
            Ok((law, lens_ref))
        }
        _ => Err(AlgebraicLawProgramError::MalformedPayload(format!(
            "AlgebraicLaw payload should be [law, lens_ref] or a record, got len {}",
            payload.len()
        ))),
    }
}

fn variant_fields<'a>(
    dag: &Dag,
    value: &'a FieldValue,
) -> Result<(String, &'a [FieldValue]), AlgebraicLawProgramError> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(AlgebraicLawProgramError::MalformedPayload(
            "expected AlgebraicLawKind variant".to_string(),
        ));
    };
    let label = variant_label(dag, *constructor).ok_or_else(|| {
        AlgebraicLawProgramError::MalformedPayload(format!(
            "variant constructor {:?} not found under any sum",
            constructor
        ))
    })?;
    Ok((label, payload.as_slice()))
}

fn declaration_ref_name(dag: &Dag, value: &FieldValue) -> Result<String, AlgebraicLawProgramError> {
    match value {
        FieldValue::Reference(id) => dag.declaration(*id).name.clone().ok_or_else(|| {
            AlgebraicLawProgramError::MalformedPayload(format!(
                "lens_ref declaration {:?} is anonymous",
                id
            ))
        }),
        other => Err(AlgebraicLawProgramError::MalformedPayload(format!(
            "lens_ref should be a DeclarationRef (FieldValue::Reference), got {other:?}"
        ))),
    }
}
