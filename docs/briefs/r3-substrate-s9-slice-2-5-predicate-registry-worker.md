---
status: draft (worker brief; Director ratified Path 3 at gunbc#828 #issuecomment-4390333451 / 2026-05-06; dispatchable now)
authority parent: R3 Substrate Manager (#1739)
ratification: Path 3 RATIFIED at gunbc#828 #issuecomment-4390333451 (zesty-bear-812). Predicate-registry landing + PB-1 shim retirement bundled.
roadmap row: extends T-Numeric-Construction (S9 Slice 2.5); resolver-substrate slice landing
authority docs:
  - gunbc#828 #issuecomment-4390333451 (Path 3 ratification)
  - gunbc#828 #issuecomment-4390275410 (3-path scope-expansion surface)
  - gunbc#1746 #issuecomment-4390253616 (proud-lynx-311 pre-flight DFS — PB-1 shim asymmetry finding)
  - PR #1840 (partial Slice 2 receipt; NonNegativeInt = Nat MERGED)
  - `INVARIANTS.md#p1-modeling-faithfulness` (P1 substrate-fact-introduction procedure)
  - feedback_construction_over_ratchets, feedback_dissolve_bridges, feedback_no_textual_enforcement_bridges
gates:
  - PositiveInt = Refined<Nat, gt_zero> (or canonical where-sugar form) lands
  - PB-1 shim (`span.file == "dsl/std/types.dag"` special case in `lower_type_alias_refinements_phase`) retired
  - All existing types.dag predicate-using declarations (`where range(...)` / `where pattern(...)`) continue to resolve correctly post-registry
worker pin: proud-lynx-311 (#1746) — pre-flight DFS context absorbed; Director pre-cleared
---

# R3 Substrate S9 Slice 2.5 — predicate-registry landing + PB-1 shim retirement worker brief

## Context

proud-lynx-311's pre-flight DFS at gunbc#1746 #issuecomment-4390253616
surfaced that the original Slice 2.5 framing ("primitive + 1
consumer") is structurally impossible:

- Inside `dsl/std/types.dag`: `Nat where gt_zero` would ride the
  PB-1 shim at `src/v3/compiler/src/lower.rs` (`lower_type_alias_refinements_phase` doc-block) (`lower_type_alias_refinements_phase`)
  which special-cases `span.file == "dsl/std/types.dag"` to absorb
  unresolved predicates with placeholder refinement. Cheap path,
  but requires moving NonNegativeInt back to types.dag (retriggers
  Gap 1 transitive-load cycle resolved in Slice 2 partial-ship).
- Outside types.dag: shim doesn't apply; `gt_zero` (or any
  non-registered predicate) fails to resolve live, same failure
  mode as Slice 2 Gap 2 (`range(min: 1)` over Nat in `integer.dag`).

Director Path 3 RATIFIED: land predicate registry **AND** retire
PB-1 shim **in same slice**. The shim is itself a bridge per
`feedback_construction_over_ratchets` — load-bearing only because
predicate-resolution doesn't exist outside types.dag. Registry
landing makes shim unnecessary; retiring in same slice avoids
dead-shim residue.

Per `feedback_no_textual_enforcement_bridges`: file-path special-case
(`span.file == "dsl/std/types.dag"`) is the same anti-pattern shape
as textual enforcement — gating structural behavior on a textual
file identifier rather than on whether the predicate is registered.

## Scope (L-sized; 5 deliverables)

This is materially larger than the original Slice 2.5 framing. PR
authoring discipline reflects the L-sized scope.

### Deliverable 1 — `gt_zero` primitive predicate

Original Slice 2.5 scope. Add `gt_zero` to the predicate vocabulary:
- Recognized at parse / resolve time as a known-predicate identifier
- Defined for Nat (per Director ratification of Option 2 at gunbc#828
  #issuecomment-4390199218 — `gt_zero` IS the reason for "Nat without
  zero" / "Nat above structural bound"; `range(min: 1)` is misleading
  label)
- **Int-side `gt_zero` extensibility EXPLICITLY OUT-OF-SCOPE**
  for Slice 2.5. Named consumer demand is `PositiveInt` only;
  no in-slice Int-side `gt_zero` consumer exists. Per
  `feedback_construction_over_ratchets`: do not speculatively
  extend predicate-carrier compatibility ahead of named consumer
  demand. If a future Int-side consumer surfaces, that's a
  separate substrate-fact-introduction (P1 procedure) brief
  with its own consumer-demand receipt. Worker authoring-side
  judgment is **bounded**: register `gt_zero` for Nat only.

### Deliverable 2 — predicate registry infrastructure

Resolver-side substrate-fact-introduction (P1 procedure):

1. **Authoritative registry** for known predicate identifiers
   (`gt_zero`, `range`, `pattern`, `non_empty`, others worker
   catalogs from existing types.dag usage). Registry is the sole
   authority for predicate-name resolution.
2. **Resolution path** for `where <predicate-name>(...)`-form
   declarations: lookup against registry; emit `Diagnostic::ResolveError`
   if absent. No file-path-special-case routing.
3. **Type-checking surface**: registry entries declare carrier
   compatibility (e.g., `gt_zero` over `Nat`; `range` over `Int`
   and Nat; `pattern` over `String`; etc.) — worker decides shape
   based on existing entailment infrastructure.

### Deliverable 3 — PB-1 shim retirement

Dissolve `lower_type_alias_refinements_phase` `span.file == "dsl/std/types.dag"`
special-case:

1. Remove the file-path-special-case branch at `src/v3/compiler/src/lower.rs` (`lower_type_alias_refinements_phase` doc-block)
2. Replace placeholder-refinement absorption with registry-resolved
   refinement
3. Confirm shim's ratchet purpose ("would emit diagnostics in the
   std seed and break snapshot identity") no longer applies because
   registry resolves all in-use predicates

### Deliverable 4 — migration of existing types.dag predicates

DFS-catalog every `where range(...)` / `where pattern(...)` /
other-predicate declaration in `dsl/std/types.dag`. Each must:
1. Resolve through the registry post-shim-retirement
2. Continue to produce identical lowered shape (no semantic drift;
   bootstrap snapshot + parse corpus manifest must hold)
3. STOP if any declaration cannot migrate cleanly (e.g., requires
   parser/lowerer changes beyond predicate-resolution)

### Deliverable 5 — `PositiveInt = Refined<Nat, gt_zero>` consumer update

Original Slice 2.5 scope step 5. Per Q-Refined-Phantom-Composition (c)
RATIFIED, Refined<> over value-typed Nat works (Nat has value
structure unlike phantom Measure carrier). Worker greps existing
`dsl/std/types.dag` convention to choose between `Refined<Nat, gt_zero>`
form and `Nat where gt_zero` where-sugar form — alignment with
existing surface pattern is the discipline.

Verify `src/v3/std/approximate_field.dag` PositiveInt consumer
transparent under alias-swap (predicate change from `range(min: 1)`
over Int to `gt_zero` over Nat is invisible to value-level consumers
per proud-lynx-311's Slice 2 audit; algebra-side change unblocks
per Director ratification).

## Hard scope bars (Director ratification)

Per Director ratification at gunbc#828 #issuecomment-4390333451:

1. **No parallel predicate-resolution path**: registry is the sole
   authority post-landing
2. **No per-call-site predicate special-casing**: file-path /
   filename / declaration-name dispatch is anti-pattern
3. **PB-1 shim retirement is non-negotiable** in same slice — no
   carve-out path (Path 1 / Path 2 alternatives EXPLICITLY rejected
   per Director ratification)
4. **Cascade-clearance verification**: every existing predicate-using
   declaration in types.dag continues to resolve correctly post-registry;
   bootstrap snapshot + parse corpus manifest hold

## Slice — single PR per Director ratification

Single bundled PR per Path 3 RATIFIED. Bundling avoids
parallel-authority drift between "predicate registry exists in part
of the codebase" vs "shim still active for legacy seed declarations."

Phase ordering (PR-internal):
1. DFS catalog of shim-dependent declarations (PR body inventory)
2. Registry infrastructure landing (Deliverable 2)
3. `gt_zero` primitive enrollment (Deliverable 1)
4. PB-1 shim retirement (Deliverable 3)
5. Existing predicate migration verification (Deliverable 4)
6. `PositiveInt` consumer update (Deliverable 5)
7. Bootstrap snapshot regen + parse corpus manifest refresh

## Acceptance

- `gt_zero` primitive predicate enrolled in registry; resolves over
  Nat only (Int-side EXPLICITLY OUT-OF-SCOPE per Deliverable 1)
- Predicate registry infrastructure landed; sole authority for
  predicate-name resolution
- PB-1 shim retired (`span.file == "dsl/std/types.dag"` branch
  removed from `lower_type_alias_refinements_phase`)
- All existing types.dag predicate-using declarations migrated;
  bootstrap snapshot + parse corpus manifest hold
- `type PositiveInt = Refined<Nat, gt_zero>` (or canonical
  where-sugar form) lands; `approximate_field.dag` consumer
  transparent
- P1 substrate-fact-introduction receipt in PR body:
  - DFS-of-concept-DAG (no parallel predicate-registry under another
    name)
  - Named consumer demand (PositiveInt; existing `range`/`pattern`
    consumers no longer need shim coverage; future positive-quantity
    refinements via Refined<Nat, gt_zero>)
  - Carrier-shape rationale (registry as substrate-resolution authority;
    PB-1 retirement closes the bridge per `feedback_construction_over_ratchets`)
- `cargo test --workspace --exclude v2-compiler-tests` green (3
  pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`:
  section anchors / rule-text quotes only; no bare `:NNN`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **Existing `where pattern(...)` / `where range(...)` declaration
  in types.dag cannot migrate cleanly to registry-resolved form
  without parser/lowerer changes** beyond predicate-resolution:
  STOP and escalate as further substrate-fact-introduction question.
  Surface the specific declaration + the parser/lowerer extension
  needed; Director scope decision required (extend Slice 2.5 further
  vs carve declaration to follow-up).
- **PB-1 shim retirement causes bootstrap snapshot drift** that
  isn't mechanical (e.g., declaration shape changes semantically,
  not just refinement-shape): STOP — substrate-fact divergence;
  surface to Substrate Mgr.
- **Registry-resolution adds resolution latency that blows the
  strict-compile diagnostic ratchet timing budget**: STOP;
  performance-side substrate question; coordinate with PB Mgr.
- **DFS-of-concept-DAG reveals a parallel predicate-registry under
  another name**: re-frame as consumer migration to existing
  registry, not landing.
- **Path 3 sub-deliverable boundary becomes ambiguous** (e.g.,
  predicate registry needs to call into PB-1 shim for one
  declaration, blocking shim retirement): STOP — Director's hard
  scope bar #3 ("PB-1 shim retirement is non-negotiable in same
  slice — no carve-out path") needs Director re-engagement.

## Authority audit receipt

1. **Substrate exists?** Verified at HEAD post-#1840 merge by
   Substrate Mgr independent grep (parallel verification of
   proud-lynx-311's pre-flight DFS at gunbc#1746 #issuecomment-4390253616):
   - `gt_zero` / `gtzero`: zero matches across `dsl/std/` + `src/v3/`
     (verified via `grep -rn "gt_zero\|gtzero" dsl/std/ src/v3/`)
   - `dsl/std/integer.dag` carries `type NonNegativeInt = Nat`
     (declaration named `NonNegativeInt`; verified via
     `git show origin/main:dsl/std/integer.dag | grep "type NonNegativeInt"`;
     Slice 2 partial-ship per #1840 MERGED 16:54Z)
   - `dsl/std/types.dag` carries `type PositiveInt = Int where range(min: 1)`
     (declaration named `PositiveInt`; verified via
     `git show origin/main:dsl/std/types.dag | grep "type PositiveInt"`;
     unchanged in Slice 2 partial-ship; consumer of registry post-Slice-2.5)
   - PB-1 shim at `src/v3/compiler/src/lower.rs` doc-block (around
     `lower_type_alias_refinements_phase`) confirmed: heuristic
     gates on `span.file == "dsl/std/types.dag"`; comment explicitly
     names dissolution criteria ("delete the file gate / placeholders
     when one of these is [a real registry]" — paraphrase). The
     shim's own doc-block frames it as bridge-text per
     `feedback_construction_over_ratchets`.
   - Predicate registry does NOT exist as substrate-fact-introduction
     (verified via `grep -rn "predicate.registry\|known_predicate"
     src/v3/compiler/src/`); registry-shaped substrate-fact-introduction
     is part of this slice's scope.
   Worker re-greps at dispatch (per substrate-state-grep discipline);
   Mgr's independent grep is parallel verification, not substitute.
2. **Existing brief?** Original Slice 2.5 dispatch packet at
   gunbc#1746 #issuecomment-4390216297 (now superseded by Path 3
   ratification + this brief). No other competing brief.
3. **Design-doc match?** Director ratification at gunbc#828
   #issuecomment-4390333451 is the design-doc anchor. Worker re-reads
   ratification at dispatch.
4. **Citations live?** Verified at HEAD post-#1840 merge by Mgr
   independent grep (above): `dsl/std/integer.dag:133`,
   `dsl/std/types.dag:255`, PB-1 shim location confirmed.
   Worker re-verifies at dispatch.
5. **Carrier dissolves the bridge?** Yes — PB-1 shim is the bridge;
   predicate-registry landing is the structural fix; retirement in
   same slice closes the loop. Per `feedback_dissolve_bridges`: go
   structural, don't create intermediate enums (or in this case,
   intermediate special-case carve-outs).

## Provenance

Drafted 2026-05-06 post-Path-3-ratification at gunbc#828 #issuecomment-4390333451.
Director-tier directive on bundled scope; supersedes original
Slice 2.5 dispatch packet (primitive + 1 consumer framing). Worker
pin proud-lynx-311 — pre-flight DFS context absorbed; Director
pre-cleared. Sequencing: PR #1840 (partial Slice 2) merged at
16:54Z; Slice 2.5 dispatchable immediately on this brief landing.
Tier-1 brief authoring posture (S9 Phase-1 Step 3, S9 Phase-2 Float
coordination, T-LBP cementing tests, S3 Phase-2 parser-grammar)
holds independently — Path 3 reframe doesn't disturb the Tier-1
queue.

PM-channel routing per Director: surface scope-expansion to PM (#846)
for Brian-channel sanction. Not blocking; brief authoring proceeds
in parallel.
