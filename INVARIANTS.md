> Part of: [THESIS.md](THESIS.md) — these invariants are the structural rules that enforce causal consistency. The thesis says "every causal link is validated"; this document says how.

# Compiler and Runtime Invariants

This is the reviewer-facing invariant index. Full examples, historical incidents, receipts, and long-form rationale now live under `docs/invariants/`, `docs/debt/`, and `docs/review-findings/`.

## How this doc is organized

Read this file for the active rule set and rule IDs. Read [docs/invariants/](docs/invariants/) for elaboration, [docs/debt/](docs/debt/) for tracked open debt, and [docs/review-findings/](docs/review-findings/) for archived branch-review receipts.

## Modeling Faithfulness Invariant

Every construct must be grounded in an identifiable external fact or a structural derivation from one; ungrounded constructs are not valid authorities here.

This rule is upstream of the rest of the file: performance, decidability, verifiability, and sustainability only matter if the model itself is faithful. See [docs/invariants/modeling-faithfulness-invariant.md](docs/invariants/modeling-faithfulness-invariant.md).

## Bounded Substrate Seed

The Rust-native seed that exists before any `.dag` declaration loads is a ratchet, not an escape hatch.

The seed may stay flat or shrink, but it may not grow without a narrowly argued exception or a paired deletion elsewhere in the seed. See [docs/invariants/bounded-substrate-seed.md](docs/invariants/bounded-substrate-seed.md).

## Lenses Are Substrate Declarations

The canonical form of a lens is a `.dag` declaration over the reflected substrate, not a permanent hand-written Rust module.

Rust lenses are tolerated only as bootstrap scaffolds while the compiled execution path closes. See [docs/invariants/lenses-are-substrate-declarations.md](docs/invariants/lenses-are-substrate-declarations.md).

### Reflected facts: when a boundary counts as “landed”

A reflected fact counts as landed only when the declaration, the realization, and a generated consumer proof all exist.

The short rule is that declaration alone is not consumption. See [docs/invariants/lenses-are-substrate-declarations.md](docs/invariants/lenses-are-substrate-declarations.md#reflected-facts-when-a-boundary-counts-as-landed).

## Every Dependency Is A Substrate Fact

Every downstream consumer must read dependencies from typed substrate facts or declared realizations, never from names, parallel tables, or hidden calling-convention knowledge.

If a consumer cannot answer its question by following typed edges and declared bindings, the fix belongs upstream. See [docs/invariants/every-dependency-is-a-substrate-fact.md](docs/invariants/every-dependency-is-a-substrate-fact.md).

## Root-Cause Depth Invariant

Fixes belong at the deepest unsound ancestor in the dependency graph, not at the first downstream symptom.

Review and diagnosis should walk upstream until the missing fact, incomplete type, or broken structure is found. See [docs/invariants/root-cause-depth-invariant.md](docs/invariants/root-cause-depth-invariant.md).

## Performance Invariant

Performance fixes must remove redundant work structurally, not layer caches or heuristics on top of an authority split.

The governing question is where unnecessary work first becomes representable, because that is where the durable fix belongs. See [docs/invariants/performance-invariant.md](docs/invariants/performance-invariant.md).

### Facts Flow Forward (2026-03-26)

Facts should move forward from declaration source to every consumer without being reconstructed downstream.

This is the recurring performance and authority lesson behind the recent receipts. See [docs/invariants/performance-invariant.md](docs/invariants/performance-invariant.md#facts-flow-forward-2026-03-26).

## Early Detection Invariant

Structural errors should fail at the earliest stage that can prove them, not at a later consumer.

Pushing failure downstream hides the actual cause and expands the blast radius. See [docs/invariants/early-detection-invariant.md](docs/invariants/early-detection-invariant.md).

## Strict Forward Progress

A change only counts as progress if it reduces the amount of ad hoc state, duplicate authority, or implicit behavior in the system.

Transitional scaffolds need explicit dissolution paths and cannot become the new steady state. See [docs/invariants/strict-forward-progress.md](docs/invariants/strict-forward-progress.md).

## Decidability Invariant

Every accepted program must remain within a closed, fail-closed system whose correctness questions are structurally decidable.

Bounded iteration, explicit lowering, and closed composition are load-bearing here. See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md).

### Structural proof from primitives

The decidability story starts from the primitive algebra and preserves boundedness through composition.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#structural-proof-from-primitives).

### Recursive syntax is sugar

Recursive surface forms are tolerated only when they lower to the bounded substrate without adding new semantic power.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#recursive-syntax-is-sugar).

### C-8

Fail-closed compilation: missing support rejects rather than fabricates. See [Decidability Invariant](docs/invariants/decidability-invariant.md#fail-closed-compilation).

### Tight upper bounds — no exceptions

The language only admits computation whose upper bounds are explicit enough to prove termination and cost properties.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#tight-upper-bounds-no-exceptions).

### Cost algebra is upstream of language primitives

Cost semantics are not a post hoc analysis layer; they are part of the modeling substrate.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#cost-algebra-is-upstream-of-language-primitives).

### Practical ergonomics

Ergonomics can add sugar, but only over the same closed semantic core.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#practical-ergonomics).

### Closure property

Composition must preserve the same proof obligations as its parts.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#closure-property).

### Lowering table

Lowering is the concrete receipt that every accepted surface form maps into the decidable substrate.

See [docs/invariants/decidability-invariant.md](docs/invariants/decidability-invariant.md#lowering-table).

## Verifiability Invariant

The system should make proofs and generated checks fall out of structure instead of relying on heuristic inspection.

The rule is not “add more tests”; it is “make the structure itself explain what must be tested and proved.” See [docs/invariants/verifiability-invariant.md](docs/invariants/verifiability-invariant.md).

### Structural proof from type system

Verification starts from typed structure, declared boundaries, and the same substrate facts consumers already rely on.

See [docs/invariants/verifiability-invariant.md](docs/invariants/verifiability-invariant.md#structural-proof-from-type-system).

### What this replaces

These rules replace parallel review checklists, ad hoc emission audits, and other duplicated authority surfaces.

See [docs/invariants/verifiability-invariant.md](docs/invariants/verifiability-invariant.md#what-this-replaces).

### The one boundary

Verification may cross into target-specific realization only at explicit, declared boundaries.

See [docs/invariants/verifiability-invariant.md](docs/invariants/verifiability-invariant.md#the-one-boundary).

### Relationship to decidability

Verifiability depends on the same closure and fail-closed discipline as decidability.

See [docs/invariants/verifiability-invariant.md](docs/invariants/verifiability-invariant.md#relationship-to-decidability).

## Sustainability Invariants

The governing metric for the codebase is cost of change: when one concept changes, the number of files that must change should be as close to 1 as possible.

The sub-invariants below are the active sustainability rule set. Their long-form rationale, examples, and dissolution receipts live under `docs/invariants/`.

### Escape Hatches (why violations keep recurring)

Recurring violations come from a small set of API surfaces that make the wrong thing easier than the right thing.

See [docs/invariants/escape-hatches.md](docs/invariants/escape-hatches.md).

### No short-term solutions (this is not a production codebase)

Representation changes must land atomically; this repo has no production constraints that justify bridges, staged migrations, or compatibility shims.

See [docs/invariants/no-short-term-solutions.md](docs/invariants/no-short-term-solutions.md).

### No duplicate representations

Every fact should be encoded in exactly one authoritative place.

See [docs/invariants/no-duplicate-representations.md](docs/invariants/no-duplicate-representations.md).

### Minimal information per interface

Functions and modeling units should take exactly the facts they read and no broader context bag.

See [docs/invariants/minimal-information-per-interface.md](docs/invariants/minimal-information-per-interface.md).

### No case enumeration for open sets

Open-ended behavior should be driven by structure or data tables, not string-keyed case lists.

See [docs/invariants/no-case-enumeration-for-open-sets.md](docs/invariants/no-case-enumeration-for-open-sets.md).

### No fallbacks that fabricate

Every path either succeeds fully or fails clearly; valid-looking fabricated fallback output is forbidden.

See [docs/invariants/no-fallbacks-that-fabricate.md](docs/invariants/no-fallbacks-that-fabricate.md).
### C-1
Missing arguments fail closed; no `LitNull` sentinels. See `docs/debt/`.
### C-2
Missing defaults or config fail closed; no `LitNull` sentinels. See `docs/debt/`.
### C-3
Parser recovery may not fabricate dummy `LitNull` nodes. See `docs/debt/`.
### C-4
Placeholder `<error:*>` types are forbidden as live compatibility carriers. See `docs/debt/`.
### C-5
Error detection may not rely on string-sentinel probing. See `docs/debt/`.
### C-6
Emit may not use `<error:unknown_*>` sentinels to preserve progress. See `docs/debt/`.
### C-7
`Dynamic` is not a universal compatibility fallback. See `docs/debt/`.
### C-9
Missing fields or values may not fabricate empty nodes or empty strings. See `docs/debt/`.
### C-10
Ownership gaps may not silently fall back to clone-based progress. See `docs/debt/`.

### Heuristics indicate lost structure

Heuristics are symptoms that an upstream fact was dropped and should be restored as structure.

See [docs/invariants/heuristics-indicate-lost-structure.md](docs/invariants/heuristics-indicate-lost-structure.md).

### Design commitments must name the substrate target

A design claim that says “no substrate change needed” must name the existing substrate element carrying that semantic.

See [docs/invariants/design-commitments-must-name-the-substrate-target.md](docs/invariants/design-commitments-must-name-the-substrate-target.md).

### Scaffold boundaries

Scaffolds are allowed only with an explicit dissolution trigger and enforcement path.

See [docs/invariants/scaffold-boundaries.md](docs/invariants/scaffold-boundaries.md).

### No parallel implementations

A second implementation of the same computation is structural debt unless one authority is deleted in the same change.

See [docs/invariants/no-parallel-implementations.md](docs/invariants/no-parallel-implementations.md).

### No bridges

Bridges normalize half-migrations and hide the actual cleanup cost that still has to be paid.

See [docs/invariants/no-bridges.md](docs/invariants/no-bridges.md).

### No deprecations

Deprecation markers are a production-code tool and are not a legitimate steady-state strategy here.

See [docs/invariants/no-deprecations.md](docs/invariants/no-deprecations.md).

### Layer opacity

Lower layers should expose declared facts, not leak storage shape or force downstream consumers to reverse engineer them.

See [docs/invariants/layer-opacity.md](docs/invariants/layer-opacity.md).

### Semantic authority after lowering

After lowering, the substrate is the semantic authority; later stages may translate or analyze it but may not reinterpret it.

See [docs/invariants/semantic-authority-after-lowering.md](docs/invariants/semantic-authority-after-lowering.md).

### Boundary sufficiency

Every boundary must carry enough declared information for the downstream stage to stay mechanical and fail closed.

See [docs/invariants/boundary-sufficiency.md](docs/invariants/boundary-sufficiency.md).

### Explicit boundary contracts

Boundary contracts must name both the semantic fact and the permitted realization shape explicitly.

See [docs/invariants/explicit-boundary-contracts.md](docs/invariants/explicit-boundary-contracts.md).

### Emission is translation, not decision-making

Emitters should translate declared structure into target syntax, never invent semantics or pick among undeclared interpretations.

See [docs/invariants/emission-is-translation-not-decision-making.md](docs/invariants/emission-is-translation-not-decision-making.md).

### E-5: Clean-emission contract is satisfied by construction (2026-04-17)

Clean-emission obligations belong in declared contracts and emitted consumers, not in hand-maintained target-side conventions.

See [docs/invariants/e-5-clean-emission-contract-is-satisfied-by-construction.md](docs/invariants/e-5-clean-emission-contract-is-satisfied-by-construction.md).

### E-6: No target-spec field without a same-PR consumer (2026-04-16)

A target-spec field is real only when a consumer lands with it in the same change.

See [docs/invariants/e-6-no-target-spec-field-without-a-same-pr-consumer.md](docs/invariants/e-6-no-target-spec-field-without-a-same-pr-consumer.md).

### E-7: No target-private realization schema without a dissolution ratchet (2026-04-16)

Target-private realization scaffolds need an explicit deletion path when shared schema is not ready yet.

See [docs/invariants/e-7-no-target-private-realization-schema-without-a-dissolution-ratchet.md](docs/invariants/e-7-no-target-private-realization-schema-without-a-dissolution-ratchet.md).

### E-8: Unsupported core behaviors fail closed, never collapse semantically (2026-04-16)

Missing target support must reject or surface unsupported behavior rather than collapsing it to a simpler meaning.

See [docs/invariants/e-8-unsupported-core-behaviors-fail-closed-never-collapse-semantically.md](docs/invariants/e-8-unsupported-core-behaviors-fail-closed-never-collapse-semantically.md).

### L-7

Lenses consume declared substrate query functions: lens implementations should read declared substrate query surfaces instead of reaching into hand-written storage details.

See [docs/invariants/l-7-lenses-consume-declared-substrate-query-functions.md](docs/invariants/l-7-lenses-consume-declared-substrate-query-functions.md).
### DB-5
Substrate keyed lookup is single-authority and shared across consumers. See [docs/design-substrate-keyed-lookup-api.md](docs/design-substrate-keyed-lookup-api.md).

### L-8

Lens Rust surfaces preserve typed failure carriers: Rust-facing lens APIs must keep typed failure carriers intact instead of flattening them into stringly or nullable conventions.

See [docs/invariants/l-8-lens-rust-surfaces-preserve-typed-failure-carriers.md](docs/invariants/l-8-lens-rust-surfaces-preserve-typed-failure-carriers.md).

### Single-authority metadata

Metadata is only legitimate when it is itself the declared authority for a fact rather than a parallel convenience copy.

See [docs/invariants/single-authority-metadata.md](docs/invariants/single-authority-metadata.md).

### Verification predicates are substrate consumers

Verification predicates should consume the same declared substrate facts as any other downstream consumer.

See [docs/invariants/verification-predicates-are-substrate-consumers.md](docs/invariants/verification-predicates-are-substrate-consumers.md).

### E-9: External realization lives on Arrow.body (2026-04-17)

External realization is a property of the Arrow body, not a second parallel channel.

See [docs/invariants/e-9-external-realization-lives-on-arrow-body.md](docs/invariants/e-9-external-realization-lives-on-arrow-body.md).
### DB-14
Substrate external primitives materialize through declared `Arrow.body` plus target bindings. See [docs/design-substrate-external-primitives.md](docs/design-substrate-external-primitives.md).
### DB-1
Corrections are typed diagnostic carriers, not ad hoc warning text. See [docs/design-correction-shape.md](docs/design-correction-shape.md).
### DB-4
Clean-emission behavior is a declared contract with real consumers. See [docs/design-clean-emission-contract.md](docs/design-clean-emission-contract.md).
### DB-9
Mutual recursion lowers structurally through declared cluster and descent facts. See [docs/design-mutual-recursion-lowering.md](docs/design-mutual-recursion-lowering.md).

## Engineering Standards

Engineering standards are still invariant-backed rules, but the long-form receipts now live out of line.

See [docs/invariants/engineering-standards.md](docs/invariants/engineering-standards.md).

## Documentation Describes Live State

Documentation must describe the repo’s current structural state rather than an aspirational or historical shape.

See [docs/invariants/documentation-describes-live-state.md](docs/invariants/documentation-describes-live-state.md).

## Testing Invariants

Testing exists to prove structural claims and protect single-authority behavior, not to paper over unclear semantics.

See [docs/invariants/testing-invariants.md](docs/invariants/testing-invariants.md).

## DB-8

Deterministic emission: the same semantic input should emit the same target output deterministically.

See [docs/invariants/deterministic-emission-db-8.md](docs/invariants/deterministic-emission-db-8.md).

## T11

Tiered Test Execution: test execution remains tiered by structural confidence across dry-run structure, selective real computation, and full integration.

See [docs/invariants/tiered-test-execution-t11.md](docs/invariants/tiered-test-execution-t11.md).

### Tier 1 — DryRun (structure)

Structure-only checks belong in the cheapest, fastest tier.

See [docs/invariants/tiered-test-execution-t11.md](docs/invariants/tiered-test-execution-t11.md#tier-1-dryrun-structure).

### Tier 2 — Selective Real (computation)

Selective real execution proves computations without paying full integration cost everywhere.

See [docs/invariants/tiered-test-execution-t11.md](docs/invariants/tiered-test-execution-t11.md#tier-2-selective-real-computation).

### Tier 3 — Full Real (integration)

Full-real execution is reserved for the external integrations that need end-to-end receipts.

See [docs/invariants/tiered-test-execution-t11.md](docs/invariants/tiered-test-execution-t11.md#tier-3-full-real-integration).

## Branch Review Findings

See `docs/review-findings/` for archived branch-review findings.

## Open Debt

See `docs/debt/` for tracked open debt.
