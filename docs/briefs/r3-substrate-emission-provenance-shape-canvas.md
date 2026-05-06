---
status: canvas (Substrate Mgr authority; supersedes PR #1902 PROPOSAL pending Director ratification)
authority parent: R3 Substrate Manager (#1739)
ratification ask: Director (zesty-bear-812 inbox #828) — shape disposition + ledger retarget + dispatch path
roadmap row: §1.8 ledger row TBD (gate-name retarget pending — Finding 3)
authority docs:
  - PR #1902 (PM-authored proposal; codex BLOCKING surfaced 3 findings)
  - gunbc#828 #issuecomment-4392256151 (Director original ratification — shape framed as Lens<C> instance)
  - gunbc#1739 #issuecomment-4392419369 (Director Path 1 + Q-EmissionProvenance-Carrier-Fit ratification — assumed C = List<EmissionProvenance>)
  - gunbc#1739 #issuecomment-4392477594 (Substrate Mgr fold-rule-enumerability finding — rust_target.rs uses inline &str template names, not enumerable data)
  - gunbc#1739 #issuecomment-4392457172 (PM cross-relay — codex BLOCKING reframing)
  - src/v3/std/lens.dag — `Lens<C>` Director-locked 6-field carrier (`read: fn(Dag, Behavior) -> Witness<C>` per-Behavior fold)
---

# R3 Substrate canvas — emission-provenance substrate shape disposition

## Why canvas

PR #1902 PROPOSAL surfaced two real substrate-shape questions that
absorb-at-dispatch-time can't resolve:

1. **Category mismatch** (codex Finding 2): `Lens<C>.read` is
   per-Behavior; emission provenance is per-emitted-line. Earlier
   ratification absorbed `C = List<EmissionProvenance>` as a
   workaround, but this papers over a real shape question rather
   than answering it
2. **Fold-rule-name enumerability** (Mgr finding 2026-05-06):
   `src/v3/compiler/src/emit/rust_target.rs` uses inline `&str`
   template names to `render_named_template(...)`; rule names are
   not enumerable as data declarations. `EmissionOrigin::FoldRuleAutoEmit`
   carrier (codex Finding 1 reshape) cannot name the rule faithfully
   until rule-enumeration substrate-fact-introduction lands

Both questions need Director disposition before any worker dispatch.

## Question 1 — substrate shape for emission provenance

Four options:

### Option (a) — Per-Behavior framing (`Lens<C>`-compatible)

Author `Lens<EmissionProvenance>` where carrier `C` is a per-Behavior
aggregate (e.g., `List<EmissionProvenance>` per the earlier ratification,
or `Map<BehaviorId, List<EmissionProvenance>>` if more structure
needed).

- **Pro**: fits landed `Lens<C>` shape; uniform with T-CostLens-Composition
  precedent; minimal new substrate
- **Con**: doesn't deliver Brian's visualization use case directly
  (per-emitted-line metadata is what the slide consumes; per-Behavior
  aggregate requires post-fold flattening to reach per-line)
- **Resolves**: analytical/auditing use cases; lens framework
  uniformity. Does NOT resolve visualization

### Option (b) — Per-emitted-line instrumentation (NOT a lens)

Author per-emission-line instrumentation hook on the emitter — likely
a callback or accumulator threaded through the emission fold that
records `(emitted_line, EmissionOrigin)` tuples per line as emission
runs.

- **Pro**: matches Brian's visualization use case directly
  (per-line metadata stream); structurally faithful to the
  per-emitted-line nature of the data
- **Con**: NOT a lens (different substrate shape); doesn't compose
  with `apply_lens(...)` T-LAS surface; new substrate-fact-introduction
  shape (P1 procedure required)
- **Resolves**: visualization. Does NOT compose with lens framework

### Option (c) — Withdraw + re-canvas

Substrate Mgr authors a fresh canvas after deeper substrate-state-grep;
no immediate brief landing. PM recommendation per #issuecomment-4392457172.

- **Pro**: zero-pressure correctness
- **Con**: delays both visualization AND analytical use cases

### Option (d, Mgr surface) — Both substrates, distinct

Land both:
- `Lens<EmissionProvenance>` (per-Behavior aggregate) for analytical /
  auditing surface — composes with lens framework
- Per-emitted-line instrumentation hook for visualization — separate
  substrate-fact-introduction with its own brief / worker pin

The two are not competing; they serve different consumer surfaces.

- **Pro**: serves both use cases faithfully; no shape compromise
- **Con**: 2 substrates instead of 1; ~2x authoring cost; distinct
  briefs + worker pins
- **Resolves**: both. Highest authoring cost

### Mgr recommendation: (d) IF visualization is load-bearing for Brian's slide

If the slide (PR #1879 emission-intuition diagrams) genuinely needs
per-emitted-line metadata, (a) alone is insufficient. (d) is the
correct shape — same insight as Director's bundled-scope discipline:
parallel substrates that serve distinct consumers shouldn't be
collapsed.

If the slide can consume per-Behavior aggregate (with post-fold
flatten at the visualization layer), (a) suffices and (d) is overkill.

**This is a Brian-facing scope question** — Director discretion to
disposition, possibly with a Brian-channel sub-question on whether
the slide needs per-line or accepts per-Behavior aggregate.

## Question 2 — fold-rule-name enumerability prerequisite

Independent of Question 1 disposition: any path that names
`EmissionOrigin::FoldRuleAutoEmit` (whether (a) inside Lens carrier
or (b) per-line instrumentation tuple) requires fold-rule names to
be enumerable as data declarations, NOT inline `&str` template names
in emission code.

### T-Rule-Enumeration prerequisite brief

Substrate-fact-introduction:
- Name the LangSpec rule set as enumerable data (e.g.,
  `data emission_rules: List<RuleName>` in `src/v3/std/`)
- Refactor emission code to dispatch via named-rule lookup rather
  than inline template strings
- Practice 4 classification: 🟢 PRIMITIVE phantom-parameter (rule
  identifiers are sum-type-shaped or list-of-named-strings-shaped;
  worker chooses shape via DFS at dispatch)

This brief is **dispatch-ready immediately** regardless of Question 1
disposition — every Q1 path consumes it. Worker pin: smart-ram-167
or valiant-ibex-312 (freed-pool; Substrate authoring discipline fresh).

### Order of operations

1. T-Rule-Enumeration lands first (independent of Q1 path)
2. Q1 disposition guides downstream substrate authoring:
   - (a): single Lens<EmissionProvenance> brief consumes T-Rule-Enumeration
   - (b): per-line instrumentation brief consumes T-Rule-Enumeration
   - (d): both briefs consume T-Rule-Enumeration

T-Rule-Enumeration is prerequisite irrespective; can author + dispatch
in parallel with Q1 deliberation.

## Question 3 — §1.8 ledger gate retarget (codex Finding 3)

PR #1902's `emission_provenance_lens_landed` was assigned slot #89,
which is already taken by `section_ref_substrate_landed` (T-LAS).
Retarget options:

- **Under T-CostLens-Composition cluster** (#37-#40 analogous
  lens-instance gates) — fits if Q1 path (a)
- **Under T-LAS scope** — fits if Q1 path (a) and lens is consumer-ready
  via T-LAS application surface
- **New cluster** — fits if Q1 path (b) (instrumentation, not lens)
- **Two clusters** — Q1 path (d): one gate per substrate

Mgr recommendation: pin retarget to Q1 disposition. New gate(s) get
real numbers (next free slot beyond the current ledger top); cluster
location follows substrate shape.

## Director ratification ask

1. **Q1 shape**: (a) per-Behavior lens / (b) per-line instrumentation /
   (c) withdraw + re-canvas / (d) both
2. **Q2 prerequisite**: dispatch T-Rule-Enumeration brief in parallel
   with Q1 deliberation? (Mgr recommends YES — it's prerequisite to
   every Q1 path; idle worker capacity available)
3. **Q3 retarget**: cluster + slot for the new gate(s) follow Q1
   disposition; ratify on Q1 ratification

### Brian-channel sub-question (if Director disposes (a))

Does PR #1879 emission-intuition slide need per-emitted-line metadata,
or does per-Behavior aggregate (with post-fold flatten at the
visualization layer) suffice? If per-line is load-bearing, (a) alone
won't deliver the slide use case — (d) is the right shape.

## Worker disposition

- **smart-ram-167** + **valiant-ibex-312** stay freed-pool until Q1
  ratifies (or one of them dispatches on T-Rule-Enumeration brief in
  parallel per Q2 ratification)
- PR #1902 close-superseded by this canvas + revised brief once
  Director ratifies; PM holds from further authoring per
  #issuecomment-4392457172

## Provenance

Drafted 2026-05-06 by Substrate Mgr (quick-crab-830) post-PM
disposition handoff at gunbc#846 #issuecomment-4392510255. Canvas
supersedes PR #1902 PROPOSAL until Director ratification of shape +
prerequisite + retarget. Brian's ASAP framing satisfied via T-Rule-
Enumeration parallel-dispatch (Q2 path) regardless of Q1 deliberation
timeline.
