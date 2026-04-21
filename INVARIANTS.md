> Part of: [THESIS.md](THESIS.md) — these invariants are the structural rules that enforce causal consistency. The thesis says "every causal link is validated"; this document says how.

# Compiler and Runtime Invariants

This is the reviewer-facing invariant index. Five principles anchor everything else. Per-rule receipts, historical incidents, and long-form rationale live under `docs/invariants/`, `docs/debt/`, and `docs/review-findings/`.

## The five principles

Every rule in this repo descends from one of five first principles. Growing sub-rules at the top level (C-1..C-10, E-5..E-9, L-7/L-8, DB-N) was how receipts got captured over time, but the underlying commitments compress to:

1. **Modeling Faithfulness** — every construct grounds in a declared source; ungrounded = invalid.
2. **Boundary Discipline** — boundaries carry enough declared information for mechanical consumers; every fact lives in exactly one authoritative place.
3. **Fail-Closed** — every path either succeeds fully or fails with a typed diagnostic; no fabricated plausible output.
4. **Decidability** — the accepted language stays within a bounded substrate whose correctness is structurally provable; lowering is the receipt.
5. **Progress Is Dissolution** — a change counts as progress only if it reduces ad-hoc state; scaffolds need dissolution paths.

Two load-bearing headings from the previous organization map directly into these principles: **Verifiability** rolls into Decidability (verification is a structural consequence of a closed system), and **Sustainability** rolls into Progress Is Dissolution (sustainability is the long-run framing of cost-of-change). Their sub-rules distribute into the principles where their motivating teeth live — detailed in the appendix.

Each principle below carries: the rule, why it stands alone, problem/solution shapes for pattern-matching, a historical dissolution receipt, and a cross-reference to the related rule IDs that elaborate it. Per-ID subdocs under `docs/invariants/` hold the long-form rationale.

---

## P1: Modeling Faithfulness

**Rule:** Every construct grounds in an identifiable external fact or a structural derivation from one; ungrounded constructs are not valid authorities.

**Why it stands alone:** Faithfulness is upstream of the rest. Performance, decidability, verifiability, and sustainability only matter if the model itself is faithful to the reality it claims to represent. Ungrounded authorities are structural fiction — no amount of downstream rigor recovers from them.

### Problem shape: Ungrounded heuristic

A downstream stage computes a fact that was never authored — complexity score, provenance category, likely intent — by applying rules to nearby signals. The heuristic has no declared source; outputs are plausible but trace to nothing. Every corner case becomes another rule; the rule set grows unboundedly.

**Solution shape:** Find where the missing fact should have been authored. Add a typed carrier at that layer so the fact propagates forward. The heuristic evaporates; downstream consumers read a declared fact.

**Receipt:** A complexity-scoring pass once encoded several thousand lines of heuristics to rebuild variant provenance during inference. Variant-constructor information had been silently discarded during parse. Dissolution: carry variant-provenance as a typed fact from parse forward; the heuristic pass evaporated.

### Problem shape: Unnamed substrate target

A design claim asserts "this needs no new substrate element — it composes from existing ones" without naming which element carries the new semantic. The claim is ungrounded; reviewers can't verify it, and implementation quietly adds the missing element without a ratchet receipt.

**Solution shape:** Every design commitment names the existing substrate element whose edges or fields will carry the new semantic. If no element qualifies, the design must declare a substrate extension with a dissolution ratchet. "No substrate change" is not a permissible claim without a named target.

**Receipt:** A design once claimed multi-target lowering composed from existing Arrow + Body primitives. On implementation it turned out `Arrow.body` didn't carry target-binding information. Fix: explicit substrate extension for external realization on `Arrow.body`; every future design commitment names its substrate target.

### Related rules (home-of-record here)

- **Modeling Faithfulness Invariant** — canonical statement
- **Bounded Substrate Seed** — Rust-native seed items must remain narrow; undeclared seed growth is ungrounded
- **Design Commitments Must Name The Substrate Target** — unnamed substrate = ungrounded claim
- **Heuristics Indicate Lost Structure** — heuristic output traces to a dropped upstream fact, not to a declared source
- **Documentation Describes Live State** — aspirational docs describe a reality the codebase doesn't embody

---

## P2: Boundary Discipline

**Rule:** Boundaries carry enough declared information for mechanical consumers, and every fact lives in exactly one authoritative place.

**Why it stands alone:** The cost of change is proportional to how many files encode the same fact. When a fact lives in two places — even "temporarily" — new consumers must choose between them, and the choice drifts. Consolidation later is the hardest refactor in the codebase. This principle prevents duplicate authority from entering the system in the first place.

**When a boundary counts as landed:** the declaration exists, the realization (Rust binding / data table) exists, *and a generated consumer proof exists* — i.e., generated code somewhere in the compiler consumes the declared surface. Declaration alone is staging, and a hand-written consumer is not generation.

### Problem shape: Parallel authority

A canonical accessor exists (e.g., a reflected substrate function), but a second path gets scaffolded next to it — a local walker, a parallel lookup table, an inline match. Each new consumer picks whichever is nearer. Drift is inevitable because divergence costs nothing.

**Solution shape:** Expose the canonical answer through the declared substrate surface; delete the parallel path in the same change. If the canonical answer doesn't exist yet, extend the substrate reflection *first*, then migrate consumers — not the other way around.

**Receipt:** Id → declaration lookup once had both a canonical Rust accessor and a parallel `.dag` list-walker that each consumer picked between. Dissolution: add the missing reflected accessor, delete the walker, migrate consumers to the typed accessor — the parallel authority disappeared.

### Problem shape: Consumer reverse-engineers storage shape

A downstream stage reads a lower layer not through its declared accessors but by traversing its internal storage — walking a list, checking a field convention, matching on a tag. The consumer must evolve in lockstep with the lower layer because storage shape has become part of the contract by accident.

**Solution shape:** Declare a typed query surface on the lower layer exposing the exact facts the consumer needs. The consumer reads through the typed query; storage shape underneath is free to change without touching consumers.

**Receipt:** Lens implementations once reached into hand-written storage details of the substrate, so every storage refactor cascaded into lens edits. Dissolution: declared substrate query functions (L-7); lenses became consumers of a typed boundary instead of the storage itself.

### Related rules (home-of-record here)

- **Lenses Are Substrate Declarations** + **Reflected-Facts-When-Landed**
- **Every Dependency Is A Substrate Fact**
- **Minimal Information Per Interface**
- **Layer Opacity** + **Semantic Authority After Lowering**
- **Boundary Sufficiency** + **Explicit Boundary Contracts**
- **Emission Is Translation, Not Decision-Making**
- **No Duplicate Representations** + **No Parallel Implementations** + **Single-Authority Metadata**
- **Root-Cause Depth Invariant** — fix at the deepest unsound boundary, not the first downstream symptom
- **Performance Invariant** + **Facts Flow Forward** — redundant work is dependency modeling at the wrong boundary
- **Verification Predicates Are Substrate Consumers** — verification is not its own authority
- **The One Boundary** (from Verifiability) — verification crosses target-specific realization only at declared boundaries
- **L-7: Lenses Consume Declared Substrate Query Functions**
- **L-8: Lens Rust Surfaces Preserve Typed Failure Carriers**
- **DB-5: Substrate Keyed Lookup Is Single-Authority**
- **E-6: No Target-Spec Field Without A Same-PR Consumer** — a target-spec field is real only when a consumer lands with it (same-PR-consumer is a boundary-validity rule, not a progress rule)
- **E-9: External Realization Lives On Arrow.body** — declares the single authority for external semantics
- **DB-14: Substrate External Primitives Materialize Through Declared Arrow.body Plus Target Bindings** — same single-authority principle for external primitives

---

## P3: Fail-Closed

**Rule:** Every path either succeeds fully or fails with a typed diagnostic; no fabricated plausible output.

**Why it stands alone:** Fail-closed is about *detection* behavior — when a consumer encounters missing or malformed input, what happens. If the answer is ever "substitute a plausible default," the original failure becomes invisible and diagnostics cascade from the wrong stage. This principle covers C-8 and all nine of its canonical sentinel instances (C-1..C-7, C-9, C-10), which differ only in *which* fabrication was prohibited.

### Problem shape: Fabricated fallback

A consumer encounters missing data and invents a replacement — a null sentinel for a missing argument, an `<error:unknown>` type for an unresolved reference, a silent clone to sidestep an ownership gap, a `"Unknown"` string for a missing name. Downstream code then sees plausible output and proceeds. The original missing fact is invisible; diagnostics cascade from the wrong stage.

**Solution shape:** Treat the detection point as a diagnostic boundary. Produce a typed failure carrier naming what was missing; propagate it forward so every consumer sees "this wasn't provided" instead of a substitute. If a consumer *needs* a value, the upstream contract is too weak — strengthen it, don't paper over.

**Receipt:** Parser error recovery once fabricated dummy literal nodes (C-3's LitNull pattern) to keep parsing moving; inference then treated those as real literals and produced bogus inferred types. Dissolution: typed parse-error carrier surfaced to consumers; inference refuses to proceed past it. The original parse failure became the only diagnostic instead of the tenth.

### Problem shape: Case enumeration for open sets

Behavior is driven by a string-keyed case list — operator symbols, target-language names, encoding formats — with a default branch. Adding a new case means editing the list; missing-case behavior is usually silent fall-through into a generic default. The set grows unboundedly and no reviewer can prove the list is exhaustive.

**Solution shape:** Drive the behavior from a typed data table whose structural properties make exhaustiveness provable. Each row is authored once; consumers read the row. Missing data is a typed diagnostic, not silent fall-through.

**Receipt:** Binary operator dispatch once matched against string symbols with a default branch that silently dropped unknown operators. Dissolution: data table in `parse_tables.dag` with a regen ratchet proving exhaustiveness; fall-through became a typed diagnostic.

### Related rules (home-of-record here)

- **No Fallbacks That Fabricate** — canonical statement
- **C-8: Fail-Closed Compilation** — missing support rejects rather than fabricates
- **C-1..C-7, C-9, C-10** — nine canonical sentinel patterns (missing args, missing defaults, parser recovery dummies, `<error:*>` types, string-sentinel probing, `<error:unknown_*>` in emit, `Dynamic` fallback, empty-node/empty-string fabrication, ownership clone fallback) — each is C-8 applied to a specific fabrication point
- **Early Detection Invariant** — structural errors fail at the earliest stage that can prove them
- **No Case Enumeration For Open Sets** — case-list fall-through is a fabrication pattern
- **E-8: Unsupported Core Behaviors Fail Closed, Never Collapse Semantically** — missing target support rejects or surfaces unsupported behavior (the "fail closed" choice is fail-closed teeth, not boundary discipline)
- **DB-1: Corrections Are Typed Diagnostic Carriers, Not Ad Hoc Warning Text** — primary home here because the rule's teeth are the anti-warning stance. Cross-reference: the typed-carrier *shape* is Boundary Discipline; the *motivating force* is refusing the fallback-to-warning-text escape hatch

---

## P4: Decidability

**Rule:** Every accepted program stays within a closed, fail-closed system whose correctness questions are structurally decidable.

**Why it stands alone:** Decidability is the semantic commitment that makes the language work. Bounded iteration, explicit lowering, and closed composition are load-bearing here. Recursion is sugar over a bounded substrate primitive, not a new capability. The previous `Verifiability Invariant` section lived here in substance — verification is what falls out when a closed, faithful system's structural properties become provable by construction; it's not a parallel authority.

### Problem shape: Unbounded semantic

A surface form is accepted without a proof that it terminates or that its cost is bounded — arbitrary recursion, open-ended self-reference, uncapped iteration. The analyzer either defers termination questions or adds a heuristic timeout. Correctness properties can't be proven structurally.

**Solution shape:** Either lower the surface form to a substrate primitive with an explicit bound (recursion → `Loop` with max depth, iteration → bounded fold), or reject it at the boundary. The substrate admits only forms whose upper bound is explicit.

**Receipt:** Mutual recursion once required an ad hoc termination check because the substrate had no structural representation for it. Dissolution (DB-9): mutual recursion lowers through declared cluster and descent facts; termination becomes a structural property of the lowering, not a separately-proved thing.

### Problem shape: Verification as separate pass

A verification predicate reads its own parallel copy of the facts — a second table, a traversal that re-derives structure, a rule set maintained alongside but not inside the substrate. Verification becomes its own authority; the substrate and the verifier drift.

**Solution shape:** Verification predicates are substrate consumers. They read the same declared facts every other consumer reads. Adding a verification rule means reading an existing substrate edge, not authoring a parallel structure.

**Receipt:** An emission check once walked its own private rendering of the substrate to verify clean-emission properties. Dissolution: verification became a consumer of the declared substrate queries; the private rendering deleted.

### Related rules (home-of-record here)

- **Decidability Invariant** — canonical statement
- **Structural Proof From Primitives** — decidability grounds in the primitive algebra
- **Recursive Syntax Is Sugar** — recursive surface forms lower to the bounded substrate without adding semantic power
- **Tight Upper Bounds — No Exceptions**
- **Cost Algebra Is Upstream Of Language Primitives** — cost semantics are part of the modeling substrate, not a post hoc layer
- **Practical Ergonomics** — sugar over the same closed semantic core
- **Closure Property** — composition preserves proof obligations
- **Lowering Table** — the concrete receipt that every surface form maps to the decidable substrate
- **Verifiability Invariant** (entire heading, folded in) + **Structural Proof From Type System** + **What This Replaces** + **Relationship To Decidability**
- **DB-9: Mutual Recursion Lowers Structurally Through Declared Cluster And Descent Facts**
- **DB-8: Deterministic Emission** — same semantic input → same target output (a decidability property of the emission relation)
- **Testing Invariants** — tests prove structural claims; protect single-authority behavior
- **T11: Tiered Test Execution** + **Tier 1 (DryRun)** + **Tier 2 (Selective Real)** + **Tier 3 (Full Real)** — test execution is tiered by structural confidence

---

## P5: Progress Is Dissolution

**Rule:** A change counts as progress only if it reduces ad-hoc state, duplicate authority, or implicit behavior. Scaffolds and intermediate representations need explicit dissolution paths.

**Why it stands alone:** Progress is about *steady-state shape* over time — what kinds of intermediate forms are legitimate. This is the long-run framing of cost-of-change: the governing metric is that when one concept changes, the number of files that must change approaches 1. The previous `Sustainability Invariants` heading lived here in substance — sustainability and dissolution are the same principle viewed at different timescales. This principle refuses bridges, deprecations, and migrations-as-steady-state as legitimate permanent shapes.

### Problem shape: Bridge as steady state

Two representations exist during a migration, connected by a bridge/adapter/shim. The bridge was meant to be temporary, but the migration stalled because each consumer has to move individually, and the bridge became the normal case. Any change now has to maintain both representations.

**Solution shape:** Land representation changes atomically — the new representation exists, the old one deletes, every consumer migrates in the same change. If that's too big for one change, the problem is usually that the new representation isn't actually ready; don't ship the bridge in the meantime.

**Receipt:** A parallel representation for operator metadata once connected through an adapter while consumers migrated one by one. Dissolution: the authority was declared on the new shape, all consumers cut over, adapter deleted — all in a single change.

### Problem shape: Scaffold without dissolution trigger

A scaffold (hand-maintained generated file, interim API surface, staged declaration) is introduced to unblock a lane. No explicit condition exists for when the scaffold should dissolve, so it persists indefinitely and becomes accidentally load-bearing. Future consumers wire into it because it's what exists.

**Solution shape:** Every scaffold lands with a named dissolution trigger — the specific, checkable condition that closes it (e.g., "when the substrate accessor lands, this inline walker dissolves"). A scaffold without a trigger is tracked debt in the wrong category; treat it as the failure-to-dissolve it actually is.

**Receipt:** Staged lens files once existed as hand-authored `.dag` with generated Rust that no consumer read. Dissolution trigger named in the PR: "when parse/parse_surface types converge, `expr_span` consumers wire in." When the trigger was reached, the staged files became real receipts; otherwise they would have been rollback candidates.

### Related rules (home-of-record here)

- **Strict Forward Progress** — canonical statement
- **Sustainability Invariants** (entire heading, folded in) — cost of change should approach 1
- **No Short-Term Solutions** — this is not a production codebase; no bridges, staged migrations, or compatibility shims justified
- **No Bridges** — bridges normalize half-migrations and hide cleanup cost
- **No Deprecations** — deprecation markers are a production-code tool, not a legitimate steady state
- **Scaffold Boundaries** — scaffolds require explicit dissolution triggers
- **Escape Hatches** — recurring violations come from API surfaces that make the wrong thing easier than the right thing
- **E-5: Clean-Emission Contract Is Satisfied By Construction** — clean-emission obligations belong in declared contracts, not hand-maintained target-side conventions (replacing-convention-by-construction is a progress move, not a boundary clarification)
- **E-7: No Target-Private Realization Schema Without A Dissolution Ratchet** — explicitly names the dissolution discipline in the rule title
- **DB-4: Clean-Emission Behavior Is A Declared Contract With Real Consumers** — same by-construction framing as E-5
- **Engineering Standards** — long-form receipts

---

## Appendix: ID index

Every numbered ID (C-N, E-N, L-N, DB-N) descends from one principle. The prose-name invariants are indexed under their principle above; this appendix exists so PR-history references like "violates C-8" or "see E-9" resolve quickly.

| ID | Home principle | Short form |
|---|---|---|
| C-1..C-7, C-9, C-10 | P3: Fail-Closed | canonical sentinel patterns, each is C-8 applied to a specific fabrication point |
| C-8 | P3: Fail-Closed | fail-closed compilation (canonical) |
| DB-1 | P3: Fail-Closed (cross-ref P2) | typed diagnostic carriers, not ad hoc warning text |
| DB-4 | P5: Progress Is Dissolution | clean-emission as declared contract with real consumers |
| DB-5 | P2: Boundary Discipline | substrate keyed lookup single-authority |
| DB-8 | P4: Decidability | deterministic emission |
| DB-9 | P4: Decidability | mutual recursion lowers structurally |
| DB-14 | P2: Boundary Discipline | external primitives materialize through `Arrow.body` |
| E-5 | P5: Progress Is Dissolution | clean-emission contract by construction |
| E-6 | P2: Boundary Discipline | no target-spec field without a same-PR consumer |
| E-7 | P5: Progress Is Dissolution | no target-private realization schema without a dissolution ratchet |
| E-8 | P3: Fail-Closed | unsupported core behaviors fail closed, never collapse semantically |
| E-9 | P2: Boundary Discipline | external realization lives on `Arrow.body` |
| L-7 | P2: Boundary Discipline | lenses consume declared substrate query functions |
| L-8 | P2: Boundary Discipline | lens Rust surfaces preserve typed failure carriers |
| T11 | P4: Decidability | tiered test execution (Tier 1/2/3 sub-rules under T11) |

---

## Pointers

- `docs/invariants/` — per-rule long-form rationale (one file per rule, preserved from the previous organization)
- `docs/debt/` — tracked open debt
- `docs/review-findings/` — archived branch-review receipts
- `docs/invariants/engineering-standards.md` — engineering standards (pointer)
