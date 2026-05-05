# T-Ground-Lifetime-Analyzer — structural derivation of program intent (no annotations)

**Status:** PROPOSAL — dispatchable now (no design-cadence gate; LanguageSpec ownership-axis declarations land alongside via lane 6, but lane authoring + skeleton don't block on PR-I). Authored 2026-04-29 in parallel to T-Ground-LanguageSpec row-population dispatch.

**Lane:** T-Ground-Lifetime-Analyzer (M) — item 7 of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (line 33 + lane row line 66 + planned-future line 144).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)).

**Lineage / authorities consumed (no re-litigation):**
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Modeling problem 3 corrected (lines 116-150), worked Examples 3-4 (`String` → `Box<str>` / `&str` derivation; lines 523-635), R2-vs-R3 scope lock (line 635), lane row line 385.
- THESIS: `THESIS.md:171` — engine-retraction; **no annotation surface** is the load-bearing constraint.
- Substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 (line 86 onward).
- Sibling lane: [`t-ground-languagespec.md`](t-ground-languagespec.md) — declares the ownership / growability / encoding / lifetime axes per primitive; this lane derives the program-side facts that match those axes.
- Brief shape template: [`t-ground-engine-phase-1-typestructure.md`](t-ground-engine-phase-1-typestructure.md), [`t-ground-languagespec.md`](t-ground-languagespec.md).

---

## Framing question this lane answers

Given a `.dag` program, can the compiler derive — *from the program's own structure* (bindings, function signatures, function-body use sites, escape patterns) — the ownership / lifetime / growability / encoding intent each value carries, *without* a parallel annotation substrate?

A "yes" closes Modeling problem 3 corrected (`design-emission-model.md:116-150`) and supplies the per-binding facts the Coercion-Fold consumes for target-type selection (Examples 3-4: `String` → `Box<str>` vs `&str` falls out of program structure; no `@target(rust)` annotation needed).

A "no" — i.e., a case where program structure provably under-determines intent and the only resolution is user-authored markup — is a **thesis-level retraction signal**. Stop and post to manager (#1133); do not paper over with a re-introduced annotation surface.

---

## Load-bearing constraint: no annotation surface

Per `THESIS.md:171` engine-retraction discipline + `design-emission-model.md:120` (Modeling problem 3 retraction):

> The prior framing of this problem proposed `@target(rust) annotate` syntax to let users declare ownership/etc. **No annotations.** Annotations would be a parallel-authority shape — even if structurally well-formed, they introduce a vocabulary outside the program's own structural facts.

This lane MUST NOT introduce:
- New syntactic surface for ownership / lifetime / mutability hints.
- A separate annotation `.dag` substrate keyed by binding ID.
- "Optional" annotation overrides for cases where structural derivation under-determines (under-determination is fail-closed via `EmissionDiagnostic::UnderRefined { axis }`, NOT a fallback to user markup).
- Reflective annotation surfaces on existing types (e.g., a `lifetime_hint: Option<Lifetime>` field).

The program's *use pattern* IS the declaration of intent (`design-emission-model.md:148`). Asking the user to *also* annotate is duplicate authority with no dissolution path.

---

## Scope (Director-locked at `design-emission-model.md:635`)

**R2 covers** (this lane):
- (a) **Top-level data bindings** — `data x: T = ...` at module scope. Lifetime = `self`; ownership derives from whether any use site borrows or escapes (Example 3 in `design-emission-model.md:523-583`).
- (b) **Function parameters with transient use** — parameter `n` whose body uses are non-storing, non-escaping. Ownership = `Borrowed`; lifetime = `caller` (Example 4 in `design-emission-model.md:585-635`, Case A).
- (c) **Function return values** — must be `Owned` (Rust functions can't return references to local data without lifetime annotations the program doesn't carry; the *program* doesn't carry them, but the analyzer derives the structural fact).

**R3 covers** (out-of-scope here, folded into `T-LensProducer-Retirement` per locked decision at `design-emission-model.md:635`):
- (d) Closures.
- (e) Async lifetimes.
- (f) `Pin` / self-referential structures.

The R2 cut is the minimum sufficient set for the worked Coercion-Fold examples; R3 extends the analyzer to cover the lens-producer retirement consumers.

### A. Analyzer crate scaffold

Sibling crate (SG-0 ratchet preserved per pilot/grounding_engine precedent). Suggested location: `src/v3/grounding_lifetime/` (parallels `src/v3/grounding_pilot/` + `src/v3/grounding_engine/`); worker discretion. Workspace-member entry; deletable as a unit.

### B. Inputs (consumed; no parallel reflection)

Per `design-reflection-completeness.md` discipline (no per-consumer projection): the analyzer reads the same `Dag` reflection every other lens consumes. Specifically:
- **Program DAG** — bindings, function signatures, call sites, expression subtrees.
- **Binding scope structure** — the lexical/structural enclosing scope of each binding (program graph already carries this).
- **Use-site enumeration** — every reference to a binding from elsewhere in the DAG.
- **Function signature shape** — parameter declarations, return-type declarations.
- **LanguageSpec ownership-axis declarations** (sibling lane 6) — per-target axis vocabulary the analyzer's outputs MUST match (so the Coercion-Fold can compose program-side facts × target-side axes structurally). Cross-lane consumer: gated on T-Ground-LanguageSpec landing the axes (not on full per-target population).

### C. Outputs

Per-binding structural facts attached to the binding in a typed carrier the Coercion-Fold consumes:

```dag
type LifetimeFacts {
  ownership:  Ownership      // Owned | Borrowed | Conditional
  lifetime:   LifetimeScope  // Self | Caller | Source | Conditional
  growable:   Growability    // Yes | No | NotApplicable
  encoding:   Encoding       // (lens-extensible per LanguageSpec)
}
```

Variant names + axes mirror `design-emission-model.md:534-546` worked-example substrate facts; the canonical home for the carrier (and whether it lives on `Binding` directly vs. as a side-table keyed by binding ID) is a P1-procedure decision the worker MUST land in the lane PR — see G below. **Default position:** attach as a structural fact on the binding declaration itself (no side-table; consumer reads through reflection like every other binding fact).

### D. Algorithm (structural fold; no heuristics)

The analyzer is a **structural fold over program structure**, not a heuristic walker. Per `feedback_lenses_not_passes.md`: heuristic = missing physics. The fold is essentially Rust's borrow checker run *forward* (`design-emission-model.md:146`):
- Borrow checker (validation): *given* a user-authored ownership annotation, does the program's use pattern stay within that bound?
- Lifetime analyzer (derivation): *given* the program's use pattern, what ownership does the value need?

The forward direction has no degrees of freedom: each use site contributes a structural constraint; the meet of all constraints over all use sites is the unique answer (or fail-closed if the constraints contradict / under-determine).

**Rough fold shape** (to be sharpened in implementation):
1. For each binding `b`, enumerate all use sites `U(b)`.
2. For each `u ∈ U(b)`, classify the structural constraint on `b`'s ownership:
   - `u` stores `b` in a binding outliving `b`'s scope → `Owned` constraint.
   - `u` returns `b` from an enclosing function → `Owned` constraint.
   - `u` passes `b` to a function whose signature consumes-by-value → `Owned` constraint.
   - `u` passes `b` to a function whose signature borrows transiently → `Borrowed` constraint admissible.
   - `u` mutates `b` → `growable = Yes` constraint (for container/string types).
3. Meet the constraints over `U(b)` per axis. Where constraints contradict, fail-closed with `EmissionDiagnostic::ContradictoryUse { binding, sites }`. Where constraints under-determine an axis the LanguageSpec declares as load-bearing for the binding's algebra, fail-closed with `EmissionDiagnostic::UnderRefined { axis }`.
4. Emit per-binding `LifetimeFacts` consumable by Coercion-Fold.

### E. No ordering as emission policy

Per Modeling problem 4 corrected (`design-emission-model.md:152-162`): structural ordering is diagnostic-only. The analyzer never picks "smaller ownership" or "shorter lifetime" to disambiguate ties — under-determination fails-closed. Ordering enters only when constructing `EmissionDiagnostic::UnderRefined` resolution hints.

### F. Fail-closed by construction

Every detectable under-determination / contradiction lands as a structured diagnostic (per [`feedback_fail_closed_discipline.md`] / C-8). No silent `None` defaults; no implicit "assume Owned" fallbacks. The analyzer's output domain is `Result<LifetimeFacts, EmissionDiagnostic>` per binding.

### G. Substrate-fact-introduction P1 procedure (per `INVARIANTS.md` §P1)

Worker MUST run the 3-step procedure ([`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure)) for every new substrate type / variant / field introduced under this lane and cite the receipts in the PR body:
- **Step 1 (DAG-ancestor):** `LifetimeFacts` and its component types (`Ownership`, `LifetimeScope`, `Growability`, `Encoding`) — does an ancestor structural-fact carrier already exist (e.g., is `Ownership` definable as inhabitance of some existing `Lattice<T>` or algebra in `dsl/std/`)? Where to attach the fact: on `Binding` directly, on `Declaration`, or as a side-table? Default: structural attachment, cite receipt.
- **Step 2 (Coproduct-vs-coordinate):** `LifetimeFacts` is a record (4 axes co-inhabit per binding); each axis (`Ownership` / `LifetimeScope` / etc.) is a sum (alternatives, not coordinates). Cite the worked-example substrate at `design-emission-model.md:534-546` as receipt.
- **Step 3 (Primitive-vs-lens-extensible):** `Encoding` may be lens-extensible (per LanguageSpec axis vocabulary, which can grow as new targets land); `Ownership` / `LifetimeScope` / `Growability` are substrate primitives (every Shape-A target has these — they're computational primitives, not target-specific labels).

Per [`feedback_substrate_principle_audit.md`] and `r2-grounding-manager.md:106`, this is non-optional.

---

## Out of scope (do NOT do)

- **Target-type selection.** That's Coercion-Fold (sibling lane T-Ground-Coercion-Fold). This lane produces `LifetimeFacts`; the fold consumes them.
- **User-authored annotations** (retracted per Modeling problem 3 corrected). See "Load-bearing constraint" above.
- **R3 scope (d) closures, (e) async lifetimes, (f) Pin/self-referential.** Locked at `design-emission-model.md:635` to `T-LensProducer-Retirement` (R3).
- **`EmissionDiagnostic` carrier authoring.** T-Ground-Diagnostic (S; sibling lane). This lane is a Layer-1 consumer of `CompilerDiagnosticKind` (Q6.5 LIVE per #1129); produces diagnostic instances, doesn't extend the kind sum.
- **LanguageSpec ownership-axis vocabulary.** T-Ground-LanguageSpec (sibling lane 6). This lane *consumes* the axis vocabulary; doesn't author it.
- **Touching `src/v3/compiler/`** — SG-0 ratchet.
- **Re-litigating Q1 / Q2 / Q3 / Q4 / Q6.5 locks; the R2-vs-R3 scope split at `design-emission-model.md:635`; or the no-annotation lock at line 120.**

---

## Dependencies / gates

| Gate | Status | Lane impact |
|---|---|---|
| **No design-cadence gate for lane authoring + scaffold** | n/a | Lane crate + algorithm shape can land ahead of LanguageSpec axis population |
| **LanguageSpec ownership-axis declarations (sibling lane 6)** | per [`t-ground-languagespec.md`](t-ground-languagespec.md) | Required for the `Encoding` axis vocabulary (lens-extensible) and for end-to-end Coercion-Fold consumer wiring; not required for the analyzer's algorithm or the substrate-primitive axes (Ownership / LifetimeScope / Growability) |
| **Coercion-Fold consumer wiring** (T-Ground-Coercion-Fold) | held per option (c) | Consumer of `LifetimeFacts`; coordinated as part of Coercion-Fold dispatch, not gated on this lane's authoring |
| **Q6.5 (`CompilerDiagnosticKind`)** | LIVE on main (#1129) | Consumed for `EmissionDiagnostic::ContradictoryUse` / `UnderRefined` instances |

**Cross-program signals:**
- **Substrate Manager:** none required for this lane's authoring; if Step 1 of the P1 procedure surfaces a parent type whose extension is non-trivial, escalate per Hand-off discipline.
- **R3 Verification Manager:** signal landing for `T-LensProducer-Retirement` consumer side (R3 will fold the analyzer's output into the lens-producer surface).

---

## Sizing

**M** per `r2-grounding-manager.md:66` and `design-emission-model.md:385`. Distribution (informal):
- Crate scaffold (A): S — sibling-crate template; consumer entry point.
- Inputs / reflection wiring (B): S — reads existing `Dag` reflection.
- `LifetimeFacts` carrier + variants (C): S — small substrate addition with P1 receipts.
- Algorithm (D): M — structural fold over use-site enumeration; per-axis constraint meet.
- Fail-closed surface (F): S — diagnostic shape consumed.
- Tests + worked-example coverage (Examples 3-4): S — lifted from `design-emission-model.md:523-635`.

Bundle into one PR per `feedback_bundle_workstreams_per_pr.md` unless scope balloons; if (D) surfaces a structural case requiring annotation (thesis-level retraction signal), escalate to manager (#1133) immediately.

---

## Test plan

Per `TESTING.md` — hermetic, behavior-driven, unit-first; sub-second per `feedback_test_timeout_2s.md`.

Acceptance lifted to a `.dag` `TestClaim` (gate: `lifetime_analyzer_structural_derivation_landed` per `r2-grounding-manager.md:126`):

1. **Example 3 derivation parity** (`design-emission-model.md:523-583`) — top-level `data name: String = ...` with non-borrowing use sites yields `LifetimeFacts { ownership: Owned, lifetime: Self, growable: No }` matching the worked example's expected target (`Box<str>`).
2. **Example 4 derivation parity, Case A** (`design-emission-model.md:585-635`) — function parameter `n` with transient use yields `LifetimeFacts { ownership: Borrowed, lifetime: Caller }`; matches `&str` target.
3. **Example 4 derivation parity, Case B** — function parameter `n` stored / escaped yields `LifetimeFacts { ownership: Owned }`; matches `String` target.
4. **Function return = Owned** — every function-return-position binding analyzes to `Owned` regardless of body internals; no Rust-style elision short-circuits.
5. **Contradictory-use fail-closed** — a binding used both as borrow-only and as escape produces `EmissionDiagnostic::ContradictoryUse { binding, sites }`; no silent meet-to-Owned default.
6. **Under-determined-axis fail-closed** — a binding whose use sites under-determine an axis the algebra requires (e.g., growability for a `String` algebra demanding a unique answer) produces `EmissionDiagnostic::UnderRefined { axis }`.
7. **No-annotation regression** — the analyzer's input domain does NOT include any annotation surface; a regression test asserts the input shape is exactly `(Dag, LanguageSpec axes)` with no auxiliary annotation table.
8. **R3-scope rejection** — programs invoking closures / async / Pin produce `EmissionDiagnostic::OutOfR2Scope { construct }` rather than silently returning facts derived from a partial analysis.

`cargo test --workspace --exclude v2-compiler-tests`, `cargo test -p v2-compiler-tests`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` all clean.

---

## Dissolution claim

When this lane lands:
- **No new annotation substrate is introduced** — the dissolution receipt is *negative* (no new authority added). Verify by audit: the brief lands without a single new keyword, token, or syntactic form; lane PR's diff over `dsl/` introduces no new syntactic surface.
- The retracted "T-Ground-Annotation" lane framing is structurally extinct: the analyzer fills the role annotations would have played, and there is no escape hatch back to user markup. (Confirmed by test plan item 7.)
- Modeling problem 3 corrected (`design-emission-model.md:116-150`) closes — the structural derivation half of the engine-reframe lands.
- Coercion-Fold's program-intent inputs (per `design-emission-model.md:42`) become structurally complete for the R2-scoped binding cases (a)/(b)/(c).

The dissolution claim is *verifiable, not aspirational* — the analyzer's input-domain regression test (item 7) is the structural acceptance gate for "no annotation surface" by construction.

---

## Hand-off discipline

Escalate to manager (post on #1133, do **not** absorb in lane) if:

- **Structural derivation under-determines a case the worked examples (3-4) say should resolve.** This is a thesis-level retraction signal — the program *should* declare its intent through use, and a counter-example would re-open Modeling problem 3 retraction.
- **The P1 DAG-ancestor check (Step 1) reveals an existing parent whose extension is non-trivial** (e.g., `LifetimeFacts` should attach to `Binding` but `Binding` has no extension seam).
- **A use site classifies under multiple ownership constraints with no structural rule to resolve them** (other than diagnostic-only ordering).
- **Implementation requires touching `src/v3/compiler/`** (SG-0 ratchet violation).
- **A construct in (d)/(e)/(f) R3 scope appears in an R2 worked example or required test program.** That's a scope-line drift; escalate before quietly extending.
- **The analyzer's input domain grows to include a non-DAG-non-LanguageSpec source** (annotation table, env config, sidecar). That's a no-annotation-discipline violation regardless of how the source is shaped.

Per `feedback_root_causes_over_quick_fixes.md`: no quick fixes. Per `feedback_no_textual_enforcement_bridges.md`: no grep/regex bridges to "be structural."

---

## Acceptance — `.dag` gate

Lane closes under the `r2-grounding-manager.md:126` acceptance gate:

> `lifetime_analyzer_structural_derivation_landed` — ownership/lifetime derived from program use; no annotation surface introduced.

Authored as a `.dag` `TestClaim`. Per the **structural-acceptance-per-lane-close discipline** (`r2-grounding-manager.md:11`), the gate IS the demo — no separate artifact.

PR body covers: scope (R2 cases (a)/(b)/(c) only); analyzer crate location; algorithm shape; per-binding `LifetimeFacts` carrier + P1 receipts; Examples 3-4 derivation parity receipts; no-annotation-surface regression evidence (input-domain test); dissolution claim (negative — no new authority).

---

## What unblocks on merge

- **Coercion-Fold (T-Ground-Coercion-Fold)** consumer wiring becomes structurally tractable for Examples 3-4 worked cases.
- **R3 `T-LensProducer-Retirement`** has the (a)/(b)/(c) scope foundation it extends with (d)/(e)/(f) closure / async / Pin coverage.
- **Manager** updates lane row at `r2-grounding-manager.md:66` to LANDED; signals R2 Release Manager (closure ledger).

---

## Cross-refs

- Parent: [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane 7 of 11; row line 66)
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Modeling problem 3 corrected + Examples 3-4 + R2-vs-R3 lock at line 635
- Sibling lane (axis vocabulary): [`t-ground-languagespec.md`](t-ground-languagespec.md)
- THESIS: `THESIS.md:171` (engine-retraction; no-annotation discipline)
- Substrate-fact-introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
- Brief shape template: [`t-ground-engine-phase-1-typestructure.md`](t-ground-engine-phase-1-typestructure.md), [`t-ground-languagespec.md`](t-ground-languagespec.md)
- R3 successor lane: `T-LensProducer-Retirement` (per `design-emission-model.md:635`)
