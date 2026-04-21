# INVARIANTS.md consolidation — format calibration (P2: Boundary Discipline)

## Context

`INVARIANTS.md` has grown to ~72 indexed items (including C-1..C-10, E-5..E-9, L-7/L-8, DB-1/4/5/8/9/14, and nested sub-rules). The growth is a symptom: several underlying first principles never got promoted to the top level, so sub-rules accumulated as repeated restatements whenever a new receipt landed. C-1..C-10 is one rule (fail-closed) said 10 ways. E-5..E-9 is scattered across three different principles despite sharing a name prefix.

Current cost: `INVARIANTS.md` is the only doc currently passed to the external reviewer (GPT Pro). With 72 axes to match against, pattern-matching dilutes — recent reviews have been anemic. Trimming the doc risks losing load-bearing concepts (coproduct dissolution, "no bridges," C-8), but leaving it at 72 items reproduces the dilution.

**Proposed collapse:** 72 sub-rules → 5 first principles, each carrying problem-shape / solution-shape / receipt examples; existing sub-rules become a short appendix index for PR-history referencing.

Proposed principles:
1. **Modeling Faithfulness** (5 current items)
2. **Boundary Discipline** (22 current items — biggest bucket)
3. **Fail-Closed** (17 current items)
4. **Decidability** (18 current items)
5. **Progress Is Dissolution** (10 current items)

This doc drafts one principle — **Boundary Discipline** — in the proposed format, as a calibration artifact. If the format works here (hardest case: 22 sub-items to compress), the follow-up PR rewrites the full `INVARIANTS.md` against all five principles and drops the doc from ~400 to ~200 lines.

## Calibration question

Read the draft below and flag:
- Length per principle (~40 lines for the largest bucket — too long / too short?)
- Tone of problem/solution shapes (structural enough to age well, or still too close to specific receipts?)
- Receipt format (useful anchor for a reviewer, or risks over-fitting to one historical case?)
- Related-invariants list (bare list readable, or should sub-rules collapse into prose?)

---

## Draft — Principle 2: Boundary Discipline

**Rule:** Boundaries carry enough declared information for mechanical consumers, and every fact lives in exactly one authoritative place.

**Why this is its own principle:** The cost of change is proportional to how many files encode the same fact. When a fact lives in two places — even "temporarily" — new consumers must choose between them, and the choice drifts. Consolidation later is the hardest refactor in the codebase. This principle prevents duplicate authority from entering the system in the first place.

**When a boundary counts as landed:** the declaration exists, the realization (Rust binding / data table) exists, *and* at least one consumer reads through the typed surface. Declaration alone is staging, not a landed boundary.

### Problem shape: Parallel authority

A canonical accessor exists (e.g., a reflected substrate function), but a second path gets scaffolded next to it — a local walker, a parallel lookup table, an inline match. Each new consumer picks whichever is nearer. Drift is inevitable because divergence costs nothing.

**Solution shape:** Expose the canonical answer through the declared substrate surface; delete the parallel path in the same change. If the canonical answer doesn't exist yet, extend the substrate reflection *first*, then migrate consumers — not the other way around.

**Receipt:** Id → declaration lookup once had both a canonical Rust accessor and a parallel `.dag` list-walker that each consumer picked between. Dissolution: add the missing reflected accessor, delete the walker, migrate consumers to the typed accessor — the parallel authority disappeared.

### Problem shape: Consumer reverse-engineers storage shape

A downstream stage reads a lower layer not through its declared accessors but by traversing its internal storage — walking a list, checking a field convention, matching on a tag. The consumer must evolve in lockstep with the lower layer because storage shape has become part of the contract by accident.

**Solution shape:** Declare a typed query surface on the lower layer exposing the exact facts the consumer needs. The consumer reads through the typed query; storage shape underneath is free to change without touching consumers.

**Receipt:** Lens implementations once reached into hand-written storage details of the substrate, so every storage refactor cascaded into lens edits. Dissolution: declared substrate query functions; lenses became consumers of a typed boundary instead of the storage itself.

### Related invariants (current sub-rule index)

- Lenses Are Substrate Declarations + Reflected-Facts-When-Landed
- Every Dependency Is A Substrate Fact
- Minimal Information Per Interface
- Layer Opacity + Semantic Authority After Lowering
- Boundary Sufficiency + Explicit Boundary Contracts
- Emission Is Translation, Not Decision-Making
- No Duplicate Representations + No Parallel Implementations + Single-Authority Metadata
- Root-Cause Depth (fix at the right boundary)
- Performance Invariant + Facts Flow Forward (redundant work = dependency modeling at the wrong boundary)
- Verification Predicates Are Substrate Consumers
- The One Boundary (verification crosses target-specific realization only at declared boundaries)
- L-7 (lenses consume declared substrate queries), L-8 (lens Rust surfaces preserve typed failure carriers)
- DB-5 (substrate keyed lookup single-authority)
- E-6 (target-spec field requires consumer), E-9 (external realization on `Arrow.body`), DB-14 (external primitives via `Arrow.body`)

---

## Plan (if format approved)

1. Follow-up PR rewrites `INVARIANTS.md`: 5 principles in the format above, with 2 examples each + related-invariants index.
2. Existing per-ID subdocs under `docs/invariants/` stay put — they're linked from the index and hold long-form rationale.
3. The C-series (C-1..C-10) collapses into one **Fail-Closed** entry with a short table of canonical sentinels rather than 10 separate sub-rules.
4. The E-series and DB-series distribute into their home principles (E-5/E-7 → Progress, E-6/E-9 → Boundary, E-8 → Fail-Closed, DB-1 → Fail-Closed, DB-4 → Progress, DB-5 → Boundary, DB-8/DB-9 → Decidability, DB-14 → Boundary).
5. Target length: ~200 lines total vs current ~400.

## STOP-AND-ESCALATE

- If the problem/solution/receipt format reads as too abstract for Pro to pattern-match against, escalate — may need one more layer of specificity (e.g., named anti-patterns).
- If the 5-principle partition turns out to have items genuinely straddling two principles (reviewer may spot this), escalate — may need a 6th principle or a cross-reference convention.
- If a load-bearing concept (coproduct dissolution, "recursion is sugar," "cost algebra upstream") can't be unambiguously placed under one principle, escalate — indicates the partition itself is wrong.

## Non-goals

- Not rewriting `docs/invariants/*.md` subdocs — those stay as long-form rationale.
- Not touching `THESIS.md`, `MODELING.md`, or `ROADMAP.md` — they have their own homes.
- Not removing the C-N / E-N / L-N / DB-N IDs from the index — PR history references them; they stay as appendix entries pointing to their home principle.
