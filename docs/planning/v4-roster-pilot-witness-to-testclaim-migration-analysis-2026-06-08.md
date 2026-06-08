# v4_roster_pilot `fn()->Bool` witnesses → structural TestClaim/TestClaimRun: migration analysis

**Status:** Analysis / blocker report. No implementation. No roster edits, no wrappers, no discovery
wiring, no deletes. Routed upward by Mgr-C (witty-pike-248) for an operator substrate decision.
**Date:** 2026-06-08
**Author session:** sunny-lynx-697 (C migration wave)
**Brief:** Mgr-C "C witness migration wave" — migrate `v4_roster_pilot`'s 38 `fn()->Bool` witnesses
into structural `TestClaim`/`TestClaimRun` data using a rename-vs-wrap mapping, with per-witness
discrimination proofs. Escalate if any witness cannot be migrated without changing semantics.

## TL;DR

**None** of the 38 `v4_roster_pilot` rows can be migrated into structural `TestClaim`/`TestClaimRun`
data without a semantic/design change. The current substrate has no `TestClaim` variant that
evaluates a runtime `Bool` witness; all five variants carry `Node` inputs that `run_test_claim`
evaluates by compiling them through the v2 interpreter. The two ways to bridge that gap each change
semantics:

1. **New `TestClaim` variant** that evaluates a runtime `Bool` witness — a load-bearing substrate
   change (`escalate-first`).
2. **Hand-encode each witness body as a `Node` literal** so it becomes an `EqualsClaim`/etc. — a
   second representation of code already expressed once, i.e. the "2FA-for-code" laundering
   anti-pattern the house philosophy forbids.

The brief's referenced "rename-vs-wrap mapping" artifact does not exist in the repo or in ctrl
planning docs (only still-seal's measured smoke split, which is a different axis). Its absence is
part of this finding.

This direction also reverses an operator-ratified ruling (ctrl#1467, see below).

## Substrate evidence

### TestClaim is Node-AST-evaluated (5 variants, all `Node`-carrying)

`src/v4/std/verification.dag:207` — `type TestClaim`:

| Variant | Node-carrying fields | Evaluated as |
|---|---|---|
| `CompilesClaim` | `input: Node`, `expected_value: Node` | compile `input`, compare runtime value to `expected_value` |
| `DiagnosticClaim` | `input: Node`, `expected_rejection: NonEmptyDiagnostics` | compile `input`, assert it is rejected |
| `EqualsClaim` | `lhs: Node`, `rhs: Node` | reduce `lhs`, assert equals reduced `rhs` |
| `StructuralEqualsClaim` | `lhs: Node`, `rhs: Node` | admit well-formed, node-equal `lhs`/`rhs` |
| `RoundTripClaim` | `input: Node` | round-trip `input` |

`run_test_claim` (`src/v4/compiler/05_eval.dag:1992`) takes a `TestClaimEvalSubject<Node>` —
`{ claim: TestClaim, context: EvalContext, tree: InferredTree, input: TestClaimTypedInput<Node> }`
(`05_eval.dag:462`) — and produces a `TestClaimRun<Node, RuntimeValue>` at **runtime**. There is no
`TestClaim` variant whose payload is a runtime `Bool` or a reference to a named `fn()->Bool`.

### TestClaimRun is not materialized as static data — by design

`src/v4/std/grounding.dag:348` states the design intent verbatim:

> This schema is the authority both consume; it does NOT itself run `run_test_claim` (no
> authoring-time co-authority `data run_*: TestClaimRun` row, no grep witness).

So even if a witness *were* a Node claim, the migration target (static
`data ...: TestClaimRun = ...` rows in the roster) is explicitly **not** how `TestClaimRun` is meant
to exist. `TestClaimRun` is produced by executing `run_test_claim` over a subject; the corpus path
(`manual_corpus_roster` + `testclaim_corpus_runner`, T-38-gated) is the executable runner, and even
it carries `TestClaimEvalSubject` rows (Node inputs), not bool witnesses.

### Conflict with operator-ratified ruling ctrl#1467

`gunbc-planning/v4-testclaim-route-through-v2-not-v3-2026-06-05.md` (ctrl#1467) ruled that a v4 test
**IS** a `fn -> Bool` witness, run through v2 via `gunbc run --source-root src/v4 --entry <file>
--function <fn> --claim-run`. The `v4_roster_pilot` rows are precisely that: `{label, entry,
function}` pointers consumed by `scripts/v4-testclaim-smoke-roster.sh`. Converting them into
`TestClaim` Node-data rows reverses that ruling — the bool-witness form was the deliberate choice
over a TestClaim/predicate corpus.

## Per-row classification (all 38)

Every row is an arbitrary `fn()->Bool` over std/extdeps values. None constructs a `TestClaim` or
feeds a `Node` to the interpreter. Grouped by witness *shape* (the closest TestClaim **intent**, and
why it is not a TestClaim **mechanically**):

### Shape A — `match` on a `Witness<A>` (`Holds`/`Violates`) — intent ≈ Diagnostic/Compiles
The witness already holds an evaluated `Witness` value, not a `Node` to compile.

- `edit_locus_narrow_resolution_holds` — `match resolve_edit_locus(...) { Holds => locus == affected_set_mid; Violates => false }`
- `edit_locus_fail_closed_holds` — `match resolve_edit_locus(...) { Holds => false; Violates => true }`
- `witness_model_core_bool_fact_lookup_discriminates` — nested `match` on `Holds`/`Violates` of `model_core_wave1_bool_fact_lookup` + `discriminant(...)`
- `sg_rc_f6_round_trip_owned` — `match target_reference_layer_probe_from_emitted_type(...) { ReferenceLayerProbeNotWrapped => true; _ => false }`
- `map_lookup_miss_violates_and_hit_holds` — `map_miss_is_violates() && map_hit_is_holds()` (delegates to Witness matches)

### Shape B — `match` on a coproduct-arm of a probe value — intent ≈ StructuralEquals
Compares a live value's *arm/fields* to expected; no two `Node`s.

- `extdeps_react_use_memo_use_callback_dependencies_are_required_lists_holds`
- `extdeps_react_effect_hooks_require_setup_ref_holds`
- `extdeps_react_host_element_declares_key_and_ref_holds`
- `extdeps_react_composite_element_declares_key_and_ref_holds`
- `extdeps_react_fragment_arm_declares_key_holds`
- `extdeps_react_element_children_are_list_create_element_child_holds`
- `extdeps_react_create_element_child_element_arm_wraps_react_element_holds`
- `text_string_freemonoid_carrier_holds`
- `text_host_string_roundtrip_holds`
- `text_host_string_empty_roundtrip_holds`
- `text_host_string_field_text_holds`

### Shape C — `==` / count / boolean-algebra over fn results — intent ≈ Equals
`lhs`/`rhs` are runtime values returned by DSL functions, not `Node`s.

- `witness_grounding_terminal_gate_runtime_authoritative_discriminates` — `f(RuntimeVerdicts) && (f(AuthoringTimeVerdictSurface) == false)`
- `discriminant_intrinsic_is_payload_blind_and_arm_sharp` — `discriminant(a1)==discriminant(a2)` + cross-arm distinct
- `parse_table_token_position_indices_no_extra_append_holds` — `length(...) == 3 && length(...) == 1 && ...`
- `coordination_obligation_exchange_arm_sharp_holds` — `exchange_is_*` predicates over obligation table
- `coordination_obligation_settlement_arm_sharp_holds` — `settlement_is_*` predicates over obligation table
- `lens_registry_singleton_row_counts_holds` — `lens_registry_row_count(...) == 1 && ...`
- `mvp1_rust_emit_add_fn_accepts_holds` — `match mvp1_emit_receipt { Accepted => source == rust_mvp1_source_text && d == None; ... }` *(closest to a real `CompilesClaim` — but still a Bool predicate over a precomputed `mvp1_emit_receipt`, not a `CompilesClaim` Node row; see note below)*

### Shape D — delegation to another `fn()->Bool` predicate — intent: indirection
A one-line call to another bool helper; same underlying shapes (A/B/C).

- `witness_lens_testgen_scheduled_generators_carry_provenance_green`
- `witness_lens_testgen_shadow_ci_run_receipt_green`
- `witness_lens_testgen_schedules_dag_input_surface_green`
- `witness_lens_testgen_bootstrap_generator_reifies_dag_input_surface_green`
- `ts_g2_sg1_symbol_carrier_holds`
- `ts_g2_sg5_absence_fail_closed_holds`
- `lens_application_introspect_advisory_holds`
- `lens_application_synthesis_gap_polynomial_holds`
- `lens_idempotency_write_effect_holds`
- `lens_identical_variant_payload_unrealized_scaffold_holds`
- `lens_affected_set_irt1_boundary_prune_holds`
- `lens_affected_set_irt1_fail_closed_pending_escalation_holds`
- `lens_affected_set_irt1_fail_closed_absorption_holds`
- `lens_affected_set_irt1_empty_diff_frontier_holds`
- `lens_registry_required_ids_resolve_holds`

**Rename/align vs wrap verdict:** 0 rename/align (none is natively a Node-AST `TestClaim`); all 38
are "wrap" candidates *only*, and wrapping is blocked by the missing substrate primitive (no
runtime-Bool claim variant; no quote/reflect primitive to lift a witness call into a `Node`).

### Note on `mvp1_rust_emit_add_fn_accepts_holds`
This is the only row whose *intent* is genuinely a `CompilesClaim` (emit `add` → expect the canonical
`rust_mvp1` source). Even here, the witness is a `Bool` predicate over a precomputed
`mvp1_emit_receipt` value, not a `CompilesClaim { input, expected_value }` row evaluated by
`run_test_claim`. It is the single plausible "rename to CompilesClaim" candidate, but only if the
emit/translate path is reachable as a `Node` input under `run_test_claim`'s `TestClaimEvalSubject`
(EvalContext + InferredTree). That is itself unverified substrate work; it is **not** a free rename.

## Why the discrimination requirement makes laundering visible

The brief's guardrail #2 (mutate the subject → migrated claim goes red) is exactly the test that
exposes the Node-literal laundering path. If a witness body is re-encoded as a `Node` literal, the
literal is a *copy* of the witness logic. Mutating the **real** subject (the DSL function the witness
exercises) would **not** turn the hand-encoded Node-literal claim red, because the literal no longer
references the real subject — it references its own frozen copy. So a Node-literal "migration" is a
green tautology by construction: it fails the discrimination guardrail. This is the mechanical reason
the laundering path is rejected, independent of the philosophy objection.

## Options (for the upward decision)

1. **New Bool-witness `TestClaim` variant / substrate decision.** Add a variant (or a sibling claim
   family) that evaluates a runtime `Bool` witness (or a named `entry`/`function` reference) and
   produces a `TestClaimRun`. This is a load-bearing substrate change to `verification.dag` /
   `05_eval.dag` and must be operator-ratified before implementation. It would also need to preserve
   discrimination (the run must re-execute the real witness, not a cached Bool), which argues for a
   *reference* (entry/function), i.e. essentially formalizing today's roster row as a claim subject —
   close to what `v4_roster_pilot` already is.
2. **Preserve System A as a distinct execution-witness system (recommended interim).** Keep
   `v4_roster_pilot` + `scripts/v4-testclaim-smoke-roster.sh` as the `fn()->Bool` route-through-v2
   witness system (ctrl#1467), separate from the Node-AST `TestClaim` corpus (System B,
   `manual_corpus_roster` / `testclaim_corpus_runner`). The two systems test different things (live
   witness execution vs Node-AST compile/equals) and need not be unified by force. No edits required.
3. **Hand-encode witness bodies as `Node` literals — REJECTED.** Dual representation / 2FA-for-code;
   fails the discrimination guardrail (see above); contradicts house philosophy. Listed only to mark
   it as considered and rejected.

## What was NOT done (per brief guardrails)
- No `v4_roster_pilot.dag` edits, no wrappers added, no discovery wiring, no consumer repoint, no
  deletes. This report is the entire deliverable.
