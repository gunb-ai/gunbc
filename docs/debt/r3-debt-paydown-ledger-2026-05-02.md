# R3 Debt-Paydown Ledger

**Date:** 2026-05-02
**Owner:** R3 Debt-Paydown Manager
**Authority parent:** [#1518](https://github.com/gunb-ai/gunbc/pull/1518)
**Scope:** ROADMAP.md tracked-debt rows under `## Tracked debts -- 2026-04 analyses` through the 2026-05-01 paired-analysis ingestion, plus rows that explicitly route to the R3 standing debt-paydown program.

## Authority Read

- `docs/r3-structure.md` names R3 Debt-Paydown as a standing program with closure gate `r3_debt_paydown_zero_remaining`: no tracked ROADMAP debt row survives R3 close.
- `INVARIANTS.md` P5(c) names the velocity tripwire: a 7-day introduction:dissolution ratio at or above 3:1 requires Director review after manual calibration.
- `ROADMAP.md` remains the tracked-row ledger. This ledger does not retire rows by itself; it identifies closure receipts and dispatch targets for retirement PRs.

## Baseline Counts

This pass classifies 69 ROADMAP-tracked debt rows in the ledger range:

| Bucket | Count | Meaning |
|---|---:|---|
| Open implementation / retirement work | 58 | Needs a named retirement PR, executable gate, owner closure receipt, or disposition decision. |
| Partially closed | 6 | Has a landed partial receipt but still names remaining work. |
| Retired / stale receipt row | 5 | ROADMAP already records retirement or stale-finding resolution; next action is ledger cleanup, not code. |

The highest concentration is in Substrate-adjacent rows: operator authority, value-body / mirror isomorphism, illegal-state carriers, bootstrap diagnostics, and algebra-law conformance. The second concentration is PB/Verification scaffolding: `test_runner.rs`, author-now/fire-later claims, bridge-ledger open rows, and SG-0 hand-Rust growth.

## Catalog

| Row | Introduced | Owner / lane | Status | Retirement shape |
|---|---|---|---|---|
| PR #726 E-I / E-C ship-with-debt receipt | 2026-04-24 | E-P / E-M | Open | Behavioral wiring, `promote_to_strict` dissolution, unified numeric-refinement authority. |
| PR #825 pre-merge review audit | 2026-04 | B4.4 / substrate-const parity | Open | Close optional trigger rows in `docs/debt/pr-825-pre-merge-review-audit.md`. |
| `rest_typed_response_body` | 2026-04-21 | Grounding services | Open | Typed REST response body support; remove JSON-path projection rows. |
| `openai_chat_message_full_coproduct` | 2026-04-21 | Grounding services | Open | Full OpenAI chat message coproduct, not narrow text-only row. |
| `anthropic_tool_result_full_content_surface` | 2026-04-21 | Grounding services | Open | Tool-result content modeled as nested/image-capable surface. |
| REST request wire serde alignment | 2026-04-21 | Grounding services | Open | Wire serde consumes typed carrier rows end-to-end. |
| `repeat_string` constant-fold receipt | 2026-04 | PB / evaluator | Partial | Interim bridge retired; remaining target is in-runner / modeled std body evaluation. |
| REST_OPS `CreateComment` test-table drift | 2026-04 | Grounding GitHub extdeps | Open | Test table consumes extdep operation facts; delete parallel path table. |
| `__BUG_NO_PROFILE_...` sentinel | 2026-04 | PB / fail-closed | Open | Return `Option` / typed failure instead of fabricated string. |
| `http_path.dag` `None => ""` | 2026-04 | Grounding / fail-closed | Open | Parser returns `Option` / `Result`. |
| `effects.dag` reparses typed transport from strings | 2026-04 | Grounding / effects | Open | `derive_op_effect` consumes typed transport declaration. |
| Forgeable `ResourceHandle` | 2026-04 | Substrate resources | Open | Opaque/private constructor or witness-carrier handle. |
| Hand-rolled lattice instances | 2026-04 | Substrate + Verification | Open | Declare real algebra inhabitants or weaken claims; law witnesses close. |
| `languages.dag` vs per-target emit tables | 2026-04 | Grounding LanguageSpec | Open | Emit consumes `languages.dag`; per-target duplicates dissolve. |
| Triple `MethodTranslation` schema | 2026-04 | Grounding LanguageSpec | Retired | ROADMAP records PR #1210 retirement; ledger cleanup only. |
| `dsl/std/effects.dag` vs `src/v3/std/effects.dag` | 2026-04 | Substrate / PB-zero | Open | Pick canonical home; collapse duplicate authority. |
| `container_template_algebra_rows` duplicate aliases | 2026-04 | Substrate type reflection | Open | Alias reflection derives rows; hand table deleted. |
| String-keyed authorities in `types.dag` / `coercion.dag` | 2026-04 | Substrate type reflection | Open | Structural declarations replace string-keyed tables. |
| `declaration_by_name(...)` emit pattern | 2026-04 | Substrate / emit | Open | Typed substrate access with cached declaration ids. |
| `pipeline_authority.rs` dual-authoring drift | 2026-04 / 2026-04-30 | Substrate / PB | Open | Generate one surface from the other or mark `fn compile` non-authoritative until structural lowering. |
| LLM service flattening | 2026-04 | Grounding services | Open | Service operations consume typed carriers and return typed outputs. |
| GitHub auth model bypass | 2026-04 | Grounding GitHub extdeps | Open | `github_token()` returns full `GitHubAuthToken`; remove hardcoded provider policy. |
| `errors.dag` dead generic layer | 2026-04 | Grounding / std errors | Open | Wire generic layer or delete it. |
| Fixed-width types not structurally fixed | 2026-04 | T-Numeric-Construction | Open | Alias / field refinement or cardinality carrier. |
| Surface int literals host-narrowed too early | 2026-04-24 | T-Numeric-Construction | Open | Concept-layer unbounded magnitude; target narrowing at reconciliation. |
| Peano witness carriers ratchet | 2026-04 | E-P / numeric refinement | Open | Shared `PositiveInt` / ranged-`Int` authority collapses literal bridges. |
| `parse_parser_body.txt` hand parser | 2026-04-20 | PB / SG-2b | Open | Structural `parse.dag` owns parser algorithm. |
| Bool inhabits `BooleanAlgebra<Bool>` not wired | 2026-04-21 | Substrate operators | Open | Structural inhabitance + logical operator resolution; delete hardcoded emit branches. |
| Top-level `ValueBody` list/sum/ref boundary | 2026-04-21 | Substrate | Partial | Rust gained `List`/`Map`; remaining closure needs mirror update and top-level variants consumed structurally. |
| `emit_rust_module` literal variant rename gap | 2026-04-21 | Emit / SG-3g | Open | Variant name mapping or rename facility. |
| `emit_rust_module` external tuple variant gap | 2026-04-21 | Emit / SG-3g | Open | Spec-driven constructor template by source kind. |
| File-preference rank scaffold | 2026-04-21 | PB-zero / v2 retirement | Open | Canonicalize duplicate modules; delete rank function and mirror policy. |
| SG-2c growth-discipline checkpoint | 2026-04-22 | Parser capability | Open | SG-2c proper capability work, not more tiny row extraction. |
| Emitter render-helper duplication | 2026-04-21 | Emit cleanup | Open | Shared emitter helper module after 1e P3.0. |
| Stale INVARIANTS refs sweep | 2026-04-21 | T-Receipts | Partial | Named files fixed; broader stale prose sweep remains. |
| Strict Forward Progress subdoc drift | 2026-04-21 | Invariants docs | Open | Rename/split bounded-execution vs dissolution-progress subdocs. |
| Lens capability honesty pass | 2026-04-21 | T-Lens-Behavioral-Parity | Open | Behavioral parity for cost/complexity/idempotency/parallelism; no proxy/stub residue. |
| Substrate carrier port program | 2026-04-22 | T-E-P-Producer-Broadening | Partial | E-T/E-C/E-I/E-P consumer/cementing completion; E-M pick closed. |
| CI ratchet architecture | 2026-04-21 | T-Receipts | Partial | Fresh timing audit, stale exemption deletion, per-exempt budgets. |
| `IntegrationRsScan` byte-literal workaround attractor | 2026-04-21 | T-Receipts | Open | Wider scanner or structural reader. |
| Stale `docs/briefs/` sweep | 2026-04-21 | T-Receipts | Partial | One brief banner fixed; broad sweep remains. |
| `lower_fn_body_into_existing_decl` fallback row | 2026-04-25 | Lowering | Retired | ROADMAP records stale finding resolved; ledger cleanup only. |
| `patch_lower_helpers_generated_type_alias_refinement` | 2026-04-25 | PB B7 | Retired | ROADMAP records PR #1014 retirement; ledger cleanup only. |
| Go branch emits `UnknownVariant` | 2026-04-25 | Emit / fail-closed | Retired | PR #820 added typed `EmitError::VariantParentNotFound`, removed the `UnknownVariant` fallback, and added missing-parent regression coverage. |
| Lens fold fallback + file-path semantics | 2026-04-25 | Lens application / identity carrier | Open | Require structural callable edge and fold-shape carrier. |
| `test_runner.rs` filename / sentinel bridges | 2026-04-25 | PB / Verification | Open | Structural claim roles and declaration refs replace filename/sentinel routing. |
| B4 bridge-retirement queue | 2026-04-25 | Bridge owners + Verification ledger | Open | Retire SourceSpan/file, lens-name, `include_str!`, and patching bridges in order. |
| Duplicate record-literal fields silently accepted | 2026-04-30 | R3 Substrate | Retired | PR #1551 (`929bfe650`) rejects duplicate fields in expression-position named-variant constructors before type-field projection. |
| `ValueBody` Rust/.dag mirror drift | 2026-04-30 | R3 Substrate + Verification | Open | Mirror update plus generated isomorphism / `DagShapeReport` gate. |
| `FieldMap` duplicate-free invariant lost in `.dag` mirror | 2026-04-30 | R3 Substrate | Open | Duplicate-free map carrier or keyed table construction. |
| Operator inference synthetic-arrow fallback | 2026-04-30 | R3 Substrate | Open | Structural algebra walk is authoritative; typed unsupported-operator diagnostic on miss. |
| `go_method_template_contracts` diagnostic mismatch | 2026-04-30 | R3 Substrate | Open | All three method-template contract declarations lower to `ValueBody::List` with empty diagnostics. |
| Missing diagnostics-empty bootstrap gate | 2026-04-30 | R3 Substrate + Verification | Open | `diagnostics_empty_after_bootstrap` ratchet for new bootstrap authorities. |
| `test_runner.rs` predicate-language growth | 2026-04-30 | R3 Evaluator + PB | Partial | Bundle 4b: `.github/PULL_REQUEST_TEMPLATE.md` Evaluator-freeze section + `docs/debt/r3-bundle-4b-test-runner-freeze-receipt.md` enforce a named dissolution hook for any `test_runner.rs` edit; bespoke arms remain until dissolved per brief table. |
| `dsl/v3/std/emit_model.dag` facade | 2026-04-30 | R3 Substrate / Grounding | Open | Ratchet facade as non-canonical; retire when v2/CI resolves canonical v3 std. |
| Method-template consumer migration | 2026-04-30 | R3 Grounding | Partial | PR #1549 (`dfc61af7e`) lands Phase 1 audit (`docs/briefs/method-template-consumer-migration-audit.md`): enumerates Rust + `.dag` consumers of `*_method_templates` / `rust_method_wraps_result` / `rust_simple_method_specs`, leaf-first retirement order. Phase 1 partial closures: Gap 1 + Gap 2 active in Substrate; Gap 3 reference-only via `BootstrapAuthority` carrier #1554. Subsequent landings: #1560 PB-Zero projection packet (decision-gated), #1561 `string_contains` registry classification, #1568 Rust projection oracle / test helper. **Gap 4 + Gap 5 (build-step consumer surface, leaf migrations) remain blocked** on either Substrate build-pipeline support for ephemeral generated-source-root `.dag` imports (tied to #1558 dissolution-first reframe) or a Gap 5 design that avoids needing that surface. Grounding's calm-tern Phase 2 leaf migration parked until then. |
| Reflection completeness over-trusted | 2026-04-30 | R3 Verification | Open | Generated conformance walker or equivalent mechanical theorem. |
| Provider/API mirror multiplication risk | 2026-04-30 | R3 Grounding | Open | Shared service ingestion path before more provider mirrors. |
| Hand-Rust acceptance growth | 2026-04-30 | R3 Verification + Substrate | Open | `.dag` TestClaim capability for reflected-Dag structural assertions. |
| `Json` + `Bytes` opaque kernel decomposition | 2026-05-01 | R3 Substrate | Open / disposition pending | Decide after T-Numeric-Construction whether this becomes an R3 lane. |
| SymbolicCost semiring annihilation violation | 2026-05-01 | R3 Substrate + Verification | Open | Fix normalization; add semiring law witnesses. |
| SubValueRelation bounded-lattice law violation | 2026-05-01 | R3 Substrate + Verification | Retired | PR #1543 (`3c77a60c9`) stops claiming `BoundedLattice` in `src/v3/std/induction.dag` and restates the actual meet-oriented / auxiliary-join helpers per Path B of the algebra-claim audit. |
| Emitter `as_bind().expect()` panic paths | 2026-05-01 | R3 Substrate / PB | Retired | PR #1548 (`0427f96f7`) landed the typed `BindNodeId` witness on `ArrowBody::UserDefined`; all six emitter sites consume `(*bind_id).bind(self.dag)`, guarded site uses `.bind_opt(dag)` returning typed `EmitError::MalformedUserDefinedCallable`. Local-typed-error path was rejected at design split as parallel-representation debt. |
| `??` / `%` syntax authority mismatch | 2026-05-01 | R3 Substrate + Grounding | Open | Remove unsupported rows or add full token/parse/operator chain. |
| `CollectionOps` / `StringOps` / `MapOps` duplicate operation surfaces | 2026-05-01 | R3 Grounding | Open | Target templates reference algebra method contracts / declaration refs. |
| Author-now/fire-later verification style | 2026-05-01 | R3 Verification | Open | Make one `BinaryDimensionReportEquals` claim actually execute. |
| Typed-carrier-landed + Rust-mirror-remains pattern | 2026-05-01 | R3 PB debt discipline | Open | Pair mirror introductions with isomorphism/generation or deletion. |
| BridgeLedgerZero known-open reporting | 2026-05-01 | R3 Verification | Retired | PR #1571 (`c3138a946`) lands the decreasing-open-count ratchet (`test(v3): add BridgeLedger open-count ratchet`); known-open count can no longer silently grow. Closes Bundle 4a. |
| Bare RHS alias table / numeric drift | 2026-05-01 | R3 Substrate | Open | Dissolve `PRELUDE_BARE_RHS_ALIAS_IDENTS`; add alias regression coverage. |

## De Facto Closed Or Cleanup-Only Rows

These rows should not consume implementation-worker capacity unless the ledger text itself needs cleanup; the final two are partial-closure notes rather than retired rows:

1. **Triple `MethodTranslation` schema**: retired by PR #1210; ROADMAP already records zero v2 consumers and safe deletion.
2. **`lower_fn_body_into_existing_decl` defensive Arrow re-derive**: ROADMAP records the cited symbol no longer exists and live lowering fails closed.
3. **`patch_lower_helpers_generated_type_alias_refinement` exact-string patching**: retired by PR #1014.
4. **Go branch emits `UnknownVariant`**: retired by PR #820; live Go branch emission now returns `EmitError::VariantParentNotFound` with regression coverage.
5. **Emitter `as_bind().expect()` panic paths**: retired by PR #1548 via the typed `BindNodeId` witness on `ArrowBody::UserDefined` (`src/v3/compiler/src/dag.rs:113`); all six emitter sites consume `(*bind_id).bind(self.dag)`; guarded `rust_target.rs:2503` site uses `.bind_opt(dag)` returning typed `EmitError::MalformedUserDefinedCallable`.
6. **E-M method carrier parity framing** inside the substrate-carrier port program: closed by structural subsumption pick; remaining work belongs to E-P consumer/cementing and non-carrier lens blockers.
7. **P0 repeat-string oracle bridge**: the interim `p0_repeat_string_v2_oracle_rust_bridge` is retired, but the broader modeled-evaluation target remains open. Track as partial, not as a fresh P0 bug.

## Highest-Leverage Retirement Targets

1. **Reject duplicate record-literal fields.**
   Small, correctness-critical, and already has a sibling duplicate-key pattern in map lowering. Closure removes a silent fact-drop bug rather than adding a new scaffold.

2. **Bootstrap diagnostics-empty gate for method-template contracts.**
   One structural ratchet can close both the `go_method_template_contracts` mismatch and the broader "shape test passed over diagnostic Dag" pattern.

3. **SymbolicCost semiring zero law.**
   High correctness impact: fixes cost-lens facts and seeds the Verification law-witness closure path.

4. **SubValueRelation algebra-claim correction.**
   Pairs naturally with the law-witness lane; either fixes the law or removes an invalid guarantee.

5. **`ValueBody` mirror update + first isomorphism receipt.**
   Slightly larger, but it attacks multiple rows at once: ValueBody drift, FieldMap illegal-state mirror, reflection-overtrust, and hand-mirror growth.

6. **Method-template consumer migration audit-to-retirement slice.**
   PRs populated rows; the invariant gain lands only when old runtime/emit tables stop serving consumers.

7. **BridgeLedgerZero decreasing-open-count ratchet.**
   Turns a reporting scaffold into pressure for actual row retirement across bridge owners.

8. **CI slow-test exemption fresh audit.**
   Bounded docs/script work that turns a partial T-Receipts row into measurable deletion opportunities.

(Two original highest-leverage targets are now retired and removed from this list: *Replace emitter `as_bind().expect()` panics with typed errors* — retired by PR #1548 via the stronger `BindNodeId` witness path (ledger row 92); *Go `UnknownVariant` fail-closed fix* — retired by PR #820, ledger close-out via PR #1545.)

## Velocity Tripwire Baseline

Recent merged PR titles from 2026-05-01 through 2026-05-02 show many docs/audit/readiness/spec PRs and fewer explicit retirement PRs. A title-only lower-bound scan finds several dissolution-shaped merges (`delete`, `retire`, `cleanup`, `dissolution`, `receipt`) but many more scaffold- or readiness-shaped introductions. Per `INVARIANTS.md` P5(c), this should be treated as a **calibration warning**, not an automatic Director-review trigger, because feature PRs may contain real deletions without retirement words in the title.

Next cadence pass should record:

| Window | Introduction candidates | Dissolution candidates | Required calibration |
|---|---:|---:|---|
| 2026-05-01..2026-05-02 | High: numerous readiness, scaffold, carrier, ratchet, and skeleton PRs | Low-to-moderate: cleanup / dissolution / receipt PRs present but fewer | Manual diff sweep for PRs that delete scaffolds or reduce SG-0 counts under feature titles. |

## Next Dispatch Packet

Recommended first retirement bundle for the Debt-Paydown Manager to coordinate:

1. **Substrate fail-closed mini-bundle:** duplicate record labels. (Emitter `as_bind()` retired by PR #1548 via `BindNodeId` witness; Go `UnknownVariant` retired by PR #820 / ledger close-out PR #1545.)
2. **Verification/substrate executable-gate bundle:** diagnostics-empty bootstrap ratchet plus one `BinaryDimensionReportEquals` claim that actually compares produced reports.
3. **Grounding consumer-retirement bundle:** method-template old-table consumer migration; prevent more row population from masking parallel authority.
4. **PB/Verification bridge discipline bundle:** BridgeLedgerZero decreasing-open-count ratchet and `test_runner.rs` bespoke-arm freeze rule.

Each bundle should require its PR body to name debt paid, debt newly found, and any remaining row by exact ROADMAP heading, matching the R3 per-PR debt receipt rule.
