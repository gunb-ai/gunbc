# gpt-5-5-pro cross-review findings — 2026-05-12

**Provenance**: two independent exploratory analyses by gpt-5-5-pro against `main` at different commits:
- Review A: `main@6897445` — 14 findings, self-categorized 4 novel + 10 tracked
- Review B: `main@1de959e` — 10 findings, self-categorized 3 novel + 7 tracked

**Scope inspected**: `dsl/std/*.dag`, `src/v2/tests/src/*.rs`, `dsl/extdeps/`, `THESIS.md`, `INVARIANTS.md`, `MODELING.md`, `ROADMAP.md`.

**Convergence signal**: both reviews independently flagged tracked items (PRELUDE_BARE_RHS_ALIAS_IDENTS, hand-rolled BoundedLattice instances, dsl/std vs src/v3/std duplication, http_path mirror, build.rs priority list). When two independent runs converge on already-tracked items, it raises confidence that the novel items from those same runs are real structural facts rather than analysis noise — but the novel items still need per-finding grep-verification at HEAD before any work-item ratification.

**This doc enumerates the NOVEL findings only** (already-tracked items are owned by their existing ROADMAP rows; no re-tracking).

---

## Tier-1 — Real correctness bugs (P3 fail-closed / P2 single-authority gaps)

These three may invalidate "PASSING" gate claims and are worth Director-tier signal before next gate-state ratification.

### Finding 1 — `complexity.dag` `SameArgumentCall` can return successful `UnknownCost` summary

**Class**: P3 fail-closed inconsistency; lens-internal contradiction.

**Source**: Review A #3.

**Citations**:
- `src/v3/lenses/complexity.dag:186-189` — file's own design note: "no-descent recursion must align with `cost.dag`; `SameArgumentCall` should be `Miss`, not a successful `UnknownCost` summary"
- `src/v3/lenses/complexity.dag:190-208` — `complexity_when_descent_unknown` correctly returns `miss_complexity_summary_lookup()` for the `per_call_descent_operand_port -> None` path (consistent with design note)
- `src/v3/lenses/complexity.dag:255-273` — but the **other path** maps `SameArgumentCall` to `SymbolicCost::UnknownCost`
- `src/v3/lenses/complexity.dag:275-279` — `summary_from_iter_bound` converts `UnknownCost` into `hit_complexity_summary_lookup(conservative_unknown_summary(reason))`
- `src/v3/lenses/cost.dag:268-288` — sibling cost-lens is fail-closed: `pattern_to_iter_bound(SameArgumentCall)` returns `miss_symbolic_cost_lookup()`

**Bug shape**: if `SameArgumentCall` comes with a descent operand port, complexity reports a successful conservative-unknown summary, contradicting its own P3 note AND diverging from cost.dag fail-closed behavior.

**Fix shape**: either (a) make `SameArgumentCall` structurally unable to carry a descent operand, or (b) change `complexity.dag::pattern_to_iter_bound` to return `Lookup<SymbolicCost>` like `cost.dag` and return `Miss` for `SameArgumentCall`.

**Gate impact**: may invalidate **#79** (`complexity_lens_behaviorally_complete`) PASSING status. Pre-R3-close re-validation.

---

### Finding 2 — Duplicate field/variant labels silently accepted; first-match resolution

**Class**: P2 single-authority + P3 fail-closed; silent semantic ambiguity.

**Source**: Review B #1.

**Scope discrimination** (per codex inline #10376 BLOCKING reframing 2026-05-12T20:24Z):
- **ALREADY TRACKED** (NOT this finding's scope): record-literal duplicate-field rejection at `src/v3/compiler/src/lower.rs:4937-5023` via `duplicate_record_field(...)` — ROADMAP row at `ROADMAP.md:514` owns this. Documented below as reference-only to make the gap visible (runtime-data side has the check).
- **GENUINELY NOVEL** (this finding's actual scope): the **type-declaration side**, **variant-payload side**, and **sum-variant-label side** of the same M2/M7 single-authority discipline do NOT have analogous fail-closed checks. Three distinct sites; same class; not separately tracked.

**Citations**:
- `src/v3/compiler/parse_parser_body.txt:900-911` — `parse_record_fields` accepts duplicate record fields into a Vec with no duplicate check
- `src/v3/compiler/parse_parser_body.txt:1191-1198` — same helper used for `VariantPayload::Record`
- `src/v3/compiler/src/lower.rs:3289-3307` — lowering for ordinary record types copies each field into `Conj` children without duplicate check
- `src/v3/compiler/src/lower.rs:3454-3463` — record-payload variants same shape
- `src/v3/compiler/src/dag/builder.rs:315-330` — consumers use first-match lookup (`.find()`)
- `src/v3/compiler/src/infer.rs:4231-4239` — inference same first-match
- `src/v3/compiler/src/lower.rs:5410-5466` — sum constructor resolution also uses first-match for duplicate variant labels
- **Sibling fail-closed already exists** at `src/v3/compiler/src/lower.rs:4937-5023` — `lower_record_to_structural` rejects duplicate **record-literal** body fields via `duplicate_record_field(...)` — so the runtime-data side has the check; the type-declaration side does not.

**Bug shape**: `type T { x: Int, x: Bool }` builds successfully; consumers `.find()` first match. Same for variant payloads + sum variant names.

**Fix shape**: reject duplicate labels in `SurfaceField` lists during lowering for both `TypeRecord` + `VariantPayload::Record`, anchored to repeated field span. Also reject duplicate sum variant labels in `lower_type_sum`.

**Gate impact**: not directly gate-tied, but P2/P3 single-authority violation. Could fold into existing parse/lower fail-closed program.

---

### Finding 3 — `Certainty` has non-isomorphic dual authority; lossy projection

**Class**: M2 duplicate type authority; lossy concept projection.

**Source**: Review A #14.

**Citations**:
- `dsl/std/primitives.dag:27-38` — `Certainty = Proven | Amortized | Expected | Conservative` (4 variants); used in `PrimitiveContract.certainty`
- `src/v3/lenses/complexity.dag:65-70` — `Certainty = Proven | Conservative` (2 variants); explicitly says "no `BoundedLattice<Certainty>`"

**Bug shape**: variant set narrower in v3 complexity. Data carrying `Amortized` or `Expected` cannot map losslessly into the v3 lens type. Under M2, this is duplicate type authority unless the v3 type is renamed to a distinct concept and a projection from primitive certainty is declared.

**Fix shape**: either (a) reuse shared `dsl/std/primitives.dag::Certainty`, or (b) rename v3 field to a genuinely different concept + define explicit projection from shared certainty domain.

**Gate impact**: complexity-lens completeness claims rely on certainty being modeled correctly. May invalidate **#79** (complexity) and adjacently **#80** (cost) PASSING status if cost lens shares assumption.

---

## Tier-2 — Structural defense (P4 boundary discipline)

### Finding 4 — `post_emit_verifier` unbounded host execution

**Class**: P4 decidability / boundary discipline violation.

**Source**: Review A #2.

**Citations**:
- `src/v3/compiler/src/post_emit_verifier.rs:1-25` — module purpose: invoke target verifier commands from `CleanEmissionContract.post_emit_verifier`
- `src/v3/compiler/src/post_emit_verifier.rs:165-170` — runs `rustc`, `gofmt`, or `python3 -m py_compile` per module docs
- `src/v3/compiler/src/post_emit_verifier.rs:171-198` — `Command::output()` with no wall timeout, no bounded capture
- `src/v3/compiler/src/post_emit_verifier.rs:200-202` — converts entire stdout/stderr buffers into `String`s
- `src/v3/compiler/src/post_emit_verifier.rs:100-117` — stores full strings in failure carriers

**Bug shape**: no wall timeout, no bounded capture, no null/streaming policy. Diverges from bounded `ExecuteCommand` direction landed elsewhere.

**Fix shape**: reuse the same bounded host-spawn contract as `ExecuteCommand` — typed outcome, wall timeout, bounded or discarded child I/O, separate setup/spawn/exit-policy failures. The `PostEmitVerifier` `.dag` schema should carry those policy fields rather than the Rust harness inventing them.

---

## Tier-3 — Compositional modeling gaps

### Finding 5 — `CompositionVerdict` as first-breaker `Monoid<>`

**Class**: missed algebraic structure.

**Source**: Review A #8.

**Citations**:
- `dsl/std/effects.dag:146-148` — `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker }`
- `dsl/std/effects.dag:150-161` — `compose_effects` takes first non-idempotent operation from a list
- `src/v3/std/effects.dag:501-505` — v3 copy via `first_breaker_ref`: `None -> IdempotentComposition`, `Some -> BrokenBy`
- `dsl/std/algebra.dag:110-115` — existing `Monoid<T>` declaration (`op` + `identity`)

**Bug shape**: this is a monoid over "first failure wins" — identity is `IdempotentComposition`; `op(BrokenBy(x), _) = BrokenBy(x)`; `op(IdempotentComposition, y) = y`. Currently expressed procedurally over a list instead of declaring the algebra and using generic fold.

**Fix shape**: define `Monoid<CompositionVerdict>` witness, or generic `First<T>` / `FirstFailure<T>` monoid; express workflow composition as fold over that witness.

---

### Finding 6 — `build.rs` bootstrap priority list

**Class**: bootstrap dispatch gap; Rust authority for facts that should be `.dag`-declared.

**Source**: Review A #4 + Review B #8 (independently flagged).

**Citations**:
- `src/v3/compiler/build.rs:372-383` — comment explains required dependency: `std/list.dag` + `std/substrate.dag` must lower before sibling std files
- `src/v3/compiler/build.rs:387-401` — staged file list encoded in Rust: `list.dag`, `substrate_minimal.dag`, `effects.dag`, `substrate.dag`, `methods.dag`
- `src/v3/compiler/build.rs:403-416` — also excludes `tokenize.dag` + `parse_tables.dag` from bootstrap bundle by filename check

**Bug shape**: file dependency order is not derived from `.dag` imports/declarations — it's a second authority in a build script. Cuts against thesis claim that program IS the dependency graph.

**Fix shape**: staged-file loading should be a topological pass over declared module/import/dependency facts. Special exclusions should be `.dag` bootstrap-role facts, not filename checks in Rust.

---

### Finding 7 — GitHub Actions `Job.needs: List<String>`

**Class**: structural modeling gap in extdeps; direct thesis contradiction.

**Source**: Review B #5.

**Citations**:
- `dsl/extdeps/github/actions.dag:2248-2254` — `Workflow.jobs: List<Job>`
- `dsl/extdeps/github/actions.dag:2337-2349` — `Job.needs: List<String>` (string-keyed dependency edge)
- `dsl/extdeps/github/actions.dag:2228-2235` — file claims to model GitHub Actions platform facts per workflow syntax docs

**Bug shape**: workflow dependency edges are exactly the kind of dependency that should be structurally visible per thesis — currently hidden as strings that happen to match another field. Validation is a downstream string lookup instead of structural relationship.

**Fix shape**: introduce `JobId` / `JobRef`, model `jobs` as a keyed collection, make `needs: List<JobRef>`. A workflow-validation lens can then prove all refs are present and acyclic. T-Workflow-As-Data scope (gates #98-#103).

---

## Tier-4 — Small Rust string-list dissolution

### Finding 8 — `L5_REQUIRED_TOOLCHAINS` Rust string list

**Class**: Rust authority for facts that should be typed closed set; small but real M4/M8 violation.

**Source**: Review B #10.

**Citations**:
- `src/v3/compiler/src/test_runner.rs:105-108` — hardcoded list: `&["L5RustcToolchain", "L5Python3Toolchain", "L5GoToolchain"]`; comment acknowledges the duplication
- `src/v3/compiler/src/test_runner.rs:4190-4217` — runner extracts `DeclarationRef` targets, then turns them back into names and compares to the string set

**Bug shape**: requires-edges already carry `DeclarationRef`; runner discards declaration identity and compares string-keyed.

**Fix shape**: make toolchain requirements a typed closed set in `std.verification` or target specs; compare declaration identity or carrier inhabitance, not names.

---

## Lane-routing recommendation

| Finding | Tier | Suggested lane | Wave |
|---|---|---|---|
| 1 — complexity SameArgumentCall fail-closed | T1 (bug) | Substrate Mgr (warm-wolf-698) | **Wave-2** (Day 7-10); Director-tier signal for #79 validity |
| 2 — duplicate field/variant labels | T1 (bug) | Substrate Mgr | **Wave-2**; fold into parse/lower fail-closed program |
| 3 — Certainty non-isomorphism | T1 (bug) | Substrate Mgr | **Wave-2**; Director-tier signal for #79/#80 validity |
| 4 — post_emit_verifier unbounded | T2 (defense) | Verification Mgr (clever-tern-670) | **Wave-2**; ties to bounded-ExecuteCommand pattern |
| 5 — CompositionVerdict Monoid | T3 (modeling) | Substrate Mgr | **Wave-2**; algebra-witness pattern reuse |
| 6 — build.rs priority list | T3 (modeling) | Substrate Mgr | **Wave-2 or Wave-3**; bootstrap discipline |
| 7 — GH Actions Job.needs | T3 (modeling) | Substrate Mgr (T-WAD scope) | **Wave-2**; gates #98-#103 |
| 8 — L5_REQUIRED_TOOLCHAINS | T4 (small) | Verification Mgr | **Wave-2 or Wave-3** |

**Director-tier signals (highest priority)**:
- Finding 1 may invalidate **#79** PASSING claim
- Finding 3 may invalidate **#79** and adjacently **#80** PASSING claims
- Both should be Substrate Mgr brief-authored + worker-validated before any §1.8 status drift sweep that touches Cluster F.

---

## Status disposition (per-finding)

| Finding | Roadmap status (per gpt-5-5-pro self-categorization) |
|---|---|
| 1 | Partly tracked (broad complexity-honesty program at ROADMAP:430-433); specific contradiction novel |
| 2 | Novel extension (record-literal duplicates tracked at ROADMAP:514; type-decl + variant + sum-variant duplicates not named) |
| 3 | Partly tracked (complexity-honesty work mentioned at ROADMAP:430); concrete dual-authority novel |
| 4 | Novel specific finding (bounded ExecuteCommand landed for tests per ROADMAP:54,83; post_emit_verifier path not called out) |
| 5 | Mostly novel as algebraic framing (compose_effects mentioned at ROADMAP:92,642; missed Monoid framing not named) |
| 6 | Partly tracked (general Pure Bootstrap direction at ROADMAP:37,170; specific bootstrap priority dissolution not named) |
| 7 | Likely novel (workflow-as-data ActionRef gap at ROADMAP:68; Job.needs string-keyed not directly tracked) |
| 8 | Likely novel/adjacent (verification toolchain carrier debt tracked broadly; this constant not named) |

---

## Methodology note

These findings are EXPLORATORY analyses — not blocking PR reviews — but the convergence between two independent gpt-5-5-pro runs on overlapping tracked items raises analytic confidence that the novel ones are real structural facts (not analysis noise). Confidence is a heuristic, not authority: each novel finding still needs per-citation grep-verify at HEAD before being absorbed as a work-item.

Mgrs should grep-verify each citation at HEAD before authoring worker briefs (commit drift since 2026-05-12 cannot be assumed away).
