# Cleanup-sweep #5 — slice: `src/v4/test/` (test-claim corpus)

**Date:** 2026-06-03 · **Sweep:** #5 · **Slice owner:** snappy-newt-713 · **PM:** nimble-dove-733

**Scope scanned:** 256 `.dag` files under `src/v4/test/` (253 under `claim/`, plus
`fixture/` + `coercion_fold_int_rust_fixture.dag` + `v2_run_preflight/`). High-level
cluster-granularity scan — not a per-fixture deep-dive.

## Slice: test — overall scariness: 🟡 (honest, heavily-marked scaffold; the danger is *volume of repetition*, not deception)

One-line why: This corpus is unusually disciplined — nearly every interim shape carries an
explicit `🟡 dissolve-on:` / `T-22 runner bridge` marker and the canonical ops it *does*
reach for (`content_hash` from `v4.std.node`) are the real ones. The scariness is not lying
tests; it is that **the same five hand-rolled shapes are copy-pasted across ~150 files**
because the modeled surface that would absorb them (a real claim/eval **runner** + canonical
coproduct reflection) has not landed. A small minority (`generated/`, `self_host/`, the
`mvp1_*_add_translate` family) cross from "honest interim" into "structurally asserts against
a fixture it assembled itself."

---

## Per-cluster rating (worst-first)

> Cluster granularity per PM guidance (slice is 253 files). Individual 🔴 files called out inline.

| Cluster (files) | Rating | Bridges | Towers | Note |
|---|---|---|---|---|
| `claim/generated/` (9) | 🔴 | `testgen_emit_*` harness indirection (T-19) | **`coproduct_exhaustiveness.dag`**: 4-arm hand-rolled variant-discriminant match — the file's own header (lines 3/53/62/91) *forbids exactly this* and commits to canonical coproduct reflection | "generated corpus" that is actually checked-in hand-rolled scaffolding; broken codegen contract |
| `claim/self_host/` (3) | 🔴 | `self_host_runner_not_realized_diagnostic()` sentinel | scaffold-against-contract: asserts against the *absence* of the runner | `claim_rejects_until_substrate.dag`, `claim_t15_self_host_fixed_point.dag` (B1 operand placeholders until merkle digests land) |
| `claim/manual/` mvp1 translate family (5: rust/go/cpp/ts/python `_add_translate`) | 🔴 | `Outcome<TargetSource/CompileOutput>→Node` structural projection; `poisoned_authority_source_text` fallback guard | 11–16-arm `mvp1_*_canonical_grounding_for()` else-if ladders; manually-assembled target-model + emitted-add-fn the claim then structurally asserts against | Headers explicitly forbid "CompilesClaim sentinel-node bridge that drops Rejected diagnostics" — and then ship a structural projection that does | 
| `claim/impossible_bug/` (6) | 🔴/🟡 | all six defer Outcome→Bool via match ladders; "execution deferred to T-22 runner bridge" | hand-rolled effect-shape / diagnostic-path signatures | corpus rows (T-14); honest markers but every row is a projection |
| `claim/lens_affected_set/` (15) | 🟡→🔴 | `edit_locus_resolver.dag`: BinaryShim/host-git projection bridge | `irt1_empty_diff_frontier_receipt.dag`: ~150 manually-constructed Node trees, 12+ nested matches, no explicit dissolve trigger | biggest-fixture-assembly cluster in the slice |
| `claim/lens_synthesis/` (8) | 🟡 | none | `synthesis_gap_polynomial.dag`: triple-nested match; comment (97–98) admits a generic-T field-access workaround | T-17 active; towers are language-limitation workarounds |
| `claim/nat_semiring/` (7) | 🟡 | `rung_5`/`rung_6`: `emit_host_bridge` until T-22; CompilesClaim sentinels | label-as-dispatch: `fixture=…/rung=…/target=…/law=…/predicate=…` strings carry routing (registry-as-function) | `content_hash` use is healthy; honest interim |
| `claim/branch_dispatch/`, `claim/loop_linear_bound/` (3+3) | 🟡 | deferred behind T-38; CompilesClaim sentinels mirror nat_semiring | label-dispatch rosters | `content_hash` stability claims are healthy |
| `claim/language_model/` (20) | 🟡 | host-runner receipt rows (`scripts/v4-leaf-model-*-verify.sh`); `go_r3_external.dag` imports a v3 boundary test as a temporary host bridge | manual bool-atom predicates; pending-verification sentinels | receipts fail-closed until host runner overwrites (T-22 follow-on) |
| `claim/manual/` (rest of 67) | 🟡 | `multi_target_emit_verification_gate.dag` (v3 harness authority, dissolve-on T-38); `infer_ground_add_mvp.dag` (InferReceiptCase coproduct split) | `sg2_*_type_expression_projection`, `content_hash_*`, `sg_rc_layering` build large hand-node fixtures to feed *real* ops | heaviest concept-sink imports (17 domains in mvp1 files) but mostly honest |
| `claim/lens_application/` (11) | 🟡 | `feature:v2-nested-match` dissolve markers | 6+ witness/diagnostics match-arm towers | clean dissolve triggers |
| `claim/lens_fact_density/` (11) | 🟡 | scaffold "compile-only until T-22" | hand-rolled `carrier_spec_fact` match predicates | honest |
| `claim/lens_cost/` (8) | 🟡 | scaffold | match/case on cost/symbolic_cost | honest |
| `claim/lens_coverage/` (5) | 🟡 | scaffold (some missing trigger) | `discriminant_predicate_defect_key.dag` asserts against self-constructed fixture | honest-ish |
| `claim/lens_complexity/` (5) | 🟡 | scaffold T-19/T-22 | manually-assembled Node trees | honest |
| `claim/lens_testgen/` (3) | 🟡 | T-22 runner discovery | `dag_input_surface.dag`: 4 parallel identical `LanguageBehaviorEquivalence` match ladders, no registry | honest |
| `claim/lens_structural_resolution/` (6) | 🟡 | `🟡 dissolve-on: T-13 lens driver` | conditional `*_claim_rhs()` factories | clean intent |
| `claim/claim_pipeline/` (4) | 🟡 | `T-22-outcome-to-bool-claim-bridge` | `*_bool_atom()` Bool→Node projections | gated |
| `claim/parse/` (8) | 🟡 | T-22 outcome-to-bool bridge | per-language grammar scaffold | honest |
| `claim/algebra_laws/` (5) | 🟡 | T-19 codegen pending | nat-expression canonicalization match | honest |
| `claim/lens_*` singletons (effect, common, parallelism, ownership, idempotency, unused_parameters) | 🟡/🟢 | mixed | `*_claim_rhs()` factories | `lens_common/infer_fixture` is a clean shared helper (🟢) |
| `claim/grounding_go/`, `claim/grounding_typescript/` (3+3) | 🟢 | one deferred row each | evidence-schema (SG-1/2/5) | eval-based, honest fixtures |
| `claim/workflow/` (19) | 🟢 | minimal | structural affected_set claims | cleanest large cluster |
| `claim/computation_shapes/`, `claim/round_trip/`, `claim/boundary/`, `name_resolve`, `qualified_name`, `diagnostic_correction` | 🟢/🟡 | small | RoundTripClaim / content_hash used correctly | small, well-formed |

---

## Recurring patterns (top 5) — the unmodeled repetitive work

These are the concerning shapes: each is **copy-pasted across many files** because no single
modeled surface exists yet to absorb it. Ordered by blast radius.

1. **Outcome→Bool / Outcome→Node projection bridge** (~25+ files: `claim_pipeline/*`,
   `manual/mvp1_*`, `impossible_bug/*`, grammar claims). Every claim hand-writes
   `match outcome { Accepted => true, Rejected => false }` and re-wraps the Bool/Node so a
   `CompilesClaim`/`EqualsClaim` can assert on it. **Drops Rejected diagnostics.** Universally
   marked `dissolve-on: T-22 runner evaluates Outcome<T> natively`.

2. **`*_claim_rhs()` / `*_bool_atom()` conditional data-shaping factories** (~56 files). Boilerplate
   `fn x_claim_rhs() -> Node { if x_holds() { pass_node } else { fail_node } }`. This is the
   per-claim hand-rolling of what a parameterized witness runner would do once.

3. **Hand-rolled coproduct variant-discriminant match ladders** (~31 files; worst:
   `generated/coproduct_exhaustiveness.dag`, `manual/infer_ground_add_mvp.dag`,
   `manual/mvp1_*_canonical_grounding_for()` 11–16-arm chains, `lens_*` witness matches).
   Manual `match`/else-if over coproduct variants instead of canonical reflection. The
   generated cluster *forbids this in its own header* yet contains it.

4. **Manually-assembled Node-tree fixtures the claim then structurally asserts against**
   (~59 files; worst: `lens_affected_set/irt1_*` ~150 nodes, `mvp1_*_add_translate`
   target-models/emitted-fns). The claim builds the expected artifact by hand and compares —
   so the assertion is about the fixture, not about a pipeline run.

5. **Label-as-dispatch / receipt-roster registry-as-function** (nat_semiring `rung_5/6`,
   `branch_dispatch`, `loop_linear_bound`, `language_model` host-runner receipts). Routing
   facts (`fixture=…/target=…/law=…/predicate=…`, or a `scripts/v4-…-verify.sh` path) encoded
   as strings/labels and parsed by the harness — a closed-vocab table standing in for data.

**Honest-interim qualifier:** the volume of `🟡 dissolve-on` / `T-22 runner bridge` markers
(120/253 files mention a bridge keyword; ~49 carry explicit dissolve triggers) means most of
this is *self-aware* debt with a named exit. The exceptions that lack a clear trigger
(`irt1_*`, some `lens_coverage`) and the self-forbidding cases (`generated/`, mvp1 family) are
the genuine 🔴.

---

## Missing-substrate map — one shared surface dissolves each tower

| Hand-rolled tower (where it recurs) | The one modeled surface that dissolves it |
|---|---|
| Outcome→Bool/Node projection (pattern #1) | **A real claim/eval *runner*** (`T-22`) that evaluates `Outcome<T>` natively and carries Rejected diagnostics — kills the `CompilesClaim` sentinel bridge entirely. This is the single highest-leverage missing surface for the whole slice. |
| `*_claim_rhs()` / `*_bool_atom()` factories (#2) | A **parameterized TestClaim witness** in `v4.compiler.eval` (witness = subject + expected Outcome), so per-claim pass/fail shaping is one function, not N. |
| Coproduct discriminant ladders (#3) | **Canonical coproduct reflection** (`T-19` codegen) — variant identity/exhaustiveness as a derived op over the coproduct decl, not a hand-matched arm list. |
| Hand-assembled Node-tree fixtures + structural compare (#4) | The runner (above) executing `translate/emit/compile_inferred` for real, plus `content_hash`-based structural equality (already canonical in `v4.std.node`) — assert on *run output*, not on a self-built twin. |
| Label-as-dispatch rosters / host-runner receipts (#5) | **Registry-as-data**: a modeled claim-roster (subject × target × law) the runner enumerates, replacing label-string parsing and `scripts/*-verify.sh` host bridges. Pairs with retiring `emit_host_bridge` / v3-boundary-test imports once the host runner lands. |

**Net:** four of the five towers collapse into **one** missing piece — the **T-22 claim/eval
runner** — and the fifth (#3) into **T-19 coproduct reflection**. The `content_hash` substrate
the corpus already depends on is healthy and is the model for how the rest should look.

---

## Flags for PM

- **Slice size:** as expected-large (253 claim files). Catalogued at cluster granularity per your
  guidance; individual 🔴 files named inline.
- **Candidate own-slice / escalation:** `generated/coproduct_exhaustiveness.dag` and the
  `mvp1_*_add_translate` family are the two spots where a file's *own header forbids* the shape
  it ships — worth design's attention independent of the bulk T-22 dissolve.
- **Cross-slice dependency:** this slice's #1 tower is dissolved by a *non-test* surface (the
  T-22 runner) — coordinate with whoever owns `compiler`/`eval`.
