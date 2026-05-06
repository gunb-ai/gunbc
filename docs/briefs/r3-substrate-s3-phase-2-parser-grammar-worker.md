---
status: draft (worker brief; AUTHORED 2026-05-06; dispatchable per Q-MC sub-decision 3 ratification at gunbc#828 #issuecomment-4385530115 — Candidate C user surface `Int<N>` desugars to Candidate D substrate `Compose<Int, MachineWidth<N>>` parametrically)
authority parent: R3 Substrate Manager (#1739)
ratification: dispatchable per Q-MachineConstraint sub-decisions RATIFIED at gunbc#828 #issuecomment-4385530115 (sub-decision 2: parametric Compose<Algebra, MachineConstraint> type-level interaction); brief authored against post-#1856 substrate state
roadmap row: T-Numeric-Construction (S3 Phase-2; S3 Phase-1 carrier slice landed at #1856) + §1.8 ledger row #60 (substrate_gap_parser_grammar_closed)
authority docs:
  - docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md (parent S3 brief; Phase-2 scope)
  - PR #1856 (S3 Phase-1 carrier slice — `dsl/std/machine_constraints.dag` MERGED)
  - gunbc#828 #issuecomment-4385530115 (Q-MachineConstraint sub-decisions)
  - docs/r3-program-plan.md §1.4 (substrate_gap_parser_grammar_closed)
gates:
  - §1.8 ledger row #60 (substrate_gap_parser_grammar_closed) — Class 1 5-criteria Pass: substrate carriers ✓ (#1856) + parser handles generic interaction syntax + ≥3 algebra×constraint pairs emit + target primitives NOT primary substrate entities ✓ + v2-oracle parity
worker pin: valiant-ant-72 (#1765) — S3 Phase-1 precedent owner; substrate-state context absorbed; freed-pool post-#1856
---

# R3 Substrate S3 Phase-2 — `Compose<Algebra, MachineConstraint>` parser-grammar surface

## Context

### T-V2-Retirement coordination gate (precondition framing)

T-Numeric-Construction historically carried an internal cascade
gate on T-V2-Retirement landing first (path-(a) v2-refinement-syntax-blocker
resolution). **Director ratification at gunbc#828
#issuecomment-4385530115 implicitly supersedes** that gate via
Q-MachineConstraint sub-decision 6 (UNIVERSAL substrate posture) —
coordination moves from v2-side cascade to Grounding-side discharge.
PR #1856 landing without T-V2-Retirement first confirms
supersession in practice. This brief inherits the supersession;
T-V2-Retirement coordination is **not** a hard precondition.

If supersession is contested at dispatch, STOP and surface to
Substrate Mgr (#1739).

### Substrate state

S3 Phase-1 carrier slice landed at PR #1856 (`dsl/std/machine_constraints.dag`
on origin/main):
- `MachineWidth<bits>` sole machine-axis carrier
- `Compose<Algebra, MachineConstraint> = Phantom` parametric unary phantom sum
- DSL grammar limitation noted: `data` declarations don't allow generic
  parameters on the declaration name; worked around via `type` declaration

**Phase-2 scope**: parser-grammar surface for the **interaction syntax** —
the user-facing way to write `Int<64>` etc. that parses to
`Compose<Int, MachineWidth<64>>` shape (where `Int` in slot-1 is the
fully-applied algebraic concept = `AbelianGroup<GroupCompletion<Nat>>`).

Per Q-MachineConstraint sub-decision 3 RATIFIED (gunbc#828 #issuecomment-4385530115):
> "Type spelling: `Int<64>` parses/elaborates as `Compose<Int, MachineWidth<64>>`
> parametrically; first slot is the algebraic concept (fully-applied
> carrier+witness composite — `Int = AbelianGroup<GroupCompletion<Nat>>` per #1466)."

The brief lands the **parser surface** that allows users to write the
interaction syntax in `.dag` source.

## Grammar shape — RATIFIED per Q-MC sub-decision 3

Q-MachineConstraint sub-decision 3 (Brian directive 2026-05-06 at
gunbc#828 #issuecomment-4385530115) is the canonical authority:

> **(3) Type-level spelling** — `Int<64>` parses/elaborates as
> `Compose<Int, MachineWidth<64>>` parametrically; first slot is the
> **algebraic concept** (fully-applied carrier+witness composite —
> `Int = AbelianGroup<GroupCompletion<Nat>>` per #1466), second slot
> is the machine-constraint axis.

**User-facing surface**: `Int<N>` (Candidate C in earlier bikeshed)

**Substrate elaboration**: `Compose<Int, MachineWidth<N>>` (Candidate D)

Both are correct — they are surface vs substrate. Parser desugars
C → D parametrically; user can also write D directly. Aliases
(`type Int32 = Compose<...>`) emerge as DSL convention naturally.

Earlier brief candidates A (`@`-form) and B (`with`-form) are REJECTED
per Director assessment — A introduces a new operator without DSL
precedent; B reserves a new keyword that conflicts with potential
record-update / extension syntax.

### Discipline confirmation (per Director response)

- `feedback_compositional_not_templating`: D's compositional shape
  preserved structurally; C is convenience aliasing the reason
- `feedback_naming_is_aliasing`: type-system sees through the alias
- `feedback_reason_not_label`: substrate (D) IS the reason; C is the
  canonical label
- `feedback_construction_over_ratchets`: parser extension is minimal
  (numeric-literal-position recognition; well-bounded)

## Slice (post-ratification scope)

### Phase 2.1 — Parser surface

Author parser extension per Q-MC sub-decision 3 (RATIFIED shape):
- Extend generic-type-arg parser to recognize numeric-literal positions
  in the second slot (e.g., `Int<32>`, `UInt<64>`, `Real<64>`, `Nat<8>`)
- Elaborator desugars `Algebra<N>` → `Compose<Algebra, MachineWidth<N>>`
  parametrically (first slot is the fully-applied algebraic concept;
  second slot is the machine-constraint axis)
- Direct `Compose<Algebra, MachineWidth<N>>` user-authored form continues
  to work unchanged (no parser change needed for D-form; type aliases
  like `type Int32 = Compose<...>` emerge as DSL convention naturally)

### Phase 2.2 — Bootstrap demonstrator + ≥3 emission pairs

Per S3 brief Phase 4 + Q-MachineConstraint sub-decision 5 ("≥3 algebra × constraint pairs is minimum, not target"):
- Author bootstrap demonstrator using ratified syntax: ≥3 algebra×machine-axis pairs visible in std seed
- Cross-reference S9 Phase-1 Step 3 emission entries brief at
  `docs/briefs/r3-substrate-s9-phase-1-step-3-emission-entries-worker.md`
  (worker pin proud-lynx-311) for Int<32> / Int<64> / UInt<64> demonstrator

### Phase 2.3 — Class 1 5-criteria Pass receipt

Per `docs/r3-program-plan.md` §1.4: `substrate_gap_parser_grammar_closed`
Pass requires:
1. Substrate carriers exist ✓ (#1856)
2. Parser handles generic interaction syntax (this Phase 2.1)
3. ≥3 algebra×constraint pairs emit to target primitives (this Phase 2.2 + S9 Phase-1 step 3)
4. Target primitives NOT primary substrate entities ✓ (#1856 framing)
5. v2-oracle parity (cementing test)

PR body documents Phase 2 closure of criteria 2 + 3 (via cross-reference to S9 Phase-1 Step 3).

### Phase 2.4 — `numeric_construction_demonstration` (§1.8 #67) co-receipt

Coordinated with S9 Phase-1 Step 3 brief: Int<32> round-trip demonstration
runs through ratified syntax; emit Rust i32 produces correct numeric
value. Substrate Mgr partition ratified that `numeric_construction_demonstration`
folds into parent worker brief Acceptance bullets, NOT separate dispatch
— per S9 Phase-1 Step 3 brief Deliverable 4.

## Acceptance

- Parser handles ratified interaction syntax: `Algebra<N>` user-surface
  desugars to `Compose<Algebra, MachineWidth<N>>` substrate per Q-MC
  sub-decision 3 (gunbc#828 #issuecomment-4385530115); direct
  `Compose<...>` form continues to work unchanged
- ≥3 algebra×machine-axis pairs visible in std seed bootstrap
  demonstrator (per Q-MachineConstraint sub-decision 5)
- Cross-reference to S9 Phase-1 Step 3 emission entries brief
  (`r3-substrate-s9-phase-1-step-3-emission-entries-worker.md`) for
  per-pair emission lowering
- §1.8 ledger row #60 (`substrate_gap_parser_grammar_closed`) advances
  via Class 1 5-criteria receipt: criteria 1, 2, 3, 4 ✓; criterion 5
  v2-oracle parity may queue separately
- Bootstrap snapshot regen + parse corpus manifest refresh per ratified
  parser change
- `cargo test --workspace --exclude v2-compiler-tests` green
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`:
  section anchors / rule-text quotes only; no bare `:NNN`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **Ratified syntax conflicts with existing parser surface** (e.g.,
  candidate A's `@` operator collides with annotation precedent that
  surfaces during implementation): STOP — surface to Substrate Mgr;
  Director re-ratification needed
- **Lowerer desugar produces semantically wrong `Compose<...>` shape**
  (e.g., positional `Int<32>` → `Compose<Int, MachineWidth<32>>`
  ambiguous if position is "first generic arg" vs "numeric literal
  arg"): STOP — surface as substrate-extension
  question; positional encoding rules need explicit specification
- **Bootstrap demonstrator landing breaks downstream consumers** (e.g.,
  some `.dag` file expects `Int` to be the bare algebra-side carrier
  not the desugar-target): STOP — surface; consumer migration may need
  to land alongside parser change
- **Class 1 v2-oracle parity (criterion 5) cannot land in same slice**
  (e.g., v2 doesn't have the `Compose<...>` desugar; cementing test
  shape needs separate substrate work): acceptable carve-out per S3
  brief STOP-AND-ESCALATE bullet 1; Phase 2 closes 4-of-5 criteria;
  criterion 5 surfaces as separate slice. Document in PR body.

## Authority audit receipt

1. **Substrate exists?** Phase-1 carriers landed at #1856 — verified by
   Substrate Mgr independent grep:
   - `MachineWidth<bits>` at `dsl/std/machine_constraints.dag` ✓
   - `Compose<Algebra, MachineConstraint> = Phantom` at same file ✓
   - Algebraic-concept names (`Int` = AbelianGroup<GroupCompletion<Nat>>;
     `UInt` = CommutativeSemiring<Nat>; `Real` = ApproximateField<Rational>)
     in `dsl/std/integer.dag` / canonical equivalents (worker re-greps
     at dispatch). These are the slot-1 elaboration targets per
     Q-MC sub-decision 3 — NOT bare witness shapes
     (`AbelianGroup` / `CommutativeSemiring`)
   - Parser does NOT yet recognize the interaction syntax; that's
     this Phase 2 scope
2. **Existing brief?** S3 parent brief
   (`r3-substrate-s3-machine-constraint-carrier-worker.md`) names
   Phase-2 in slice section. This brief is the dispatch packet for
   that phase
3. **Design-doc match?** Q-MachineConstraint sub-decision 3 RATIFIED
   (gunbc#828 #issuecomment-4385530115) names the parametric
   `Compose<...>` shape AND the user-facing surface (`Int<N>`
   desugars to `Compose<Int, MachineWidth<N>>` parametrically).
   Both surface and substrate elaboration are ratified — Candidate C
   and D in the earlier brief draft
4. **Citations live?** Verified at HEAD post-#1856. Worker re-verifies
   at dispatch
5. **Carrier dissolves the bridge?** Yes — Phase-1 substrate carriers
   landed but the parser-grammar surface that allows users to consume
   them is the remaining gap. Phase-2 closes criteria 2 + 3 of Class
   1 5-criteria Pass for `substrate_gap_parser_grammar_closed`. The
   "bridge" is the parser-surface gap; this brief lands the parser
   side. Cross-reference S9 Phase-1 Step 3 closes the emission side
   (already authored in same Tier-1 batch)

## Provenance

Drafted 2026-05-06 post-#1856 merge per Director freed-pool pressure
at gunbc#828 #issuecomment-4392095857 + Tier-1 brief-queue
commitment at gunbc#846 #issuecomment-4390098574 (2/5 in queue).

Dispatchable per Q-MC sub-decision 3 ratification (Brian directive
2026-05-06 at gunbc#828 #issuecomment-4385530115). Earlier "HOLD
pending Q-MachineConstraint-Grammar-Shape" framing was redundant —
sub-decision 3 already ratified `Int<N>` user surface + `Compose<Int,
MachineWidth<N>>` substrate elaboration. Brief revised to reference
the ratified shape directly.

Substrate Mgr discipline pin (per Director gentle observation at
gunbc#1739 #issuecomment-4392382517): ratification-state-grep before
authoring a Director-ratification ask — grep `docs/r3-program-plan.md`
§10.3 + Q-* set for the question's domain. Folded into standing
dispatch-checklist alongside substrate-state-grep / same-slice-
dissolution / verifiable-triple / bundled-scope.

Cross-references S9 Phase-1 Step 3 brief at
`docs/briefs/r3-substrate-s9-phase-1-step-3-emission-entries-worker.md`
(worker pin proud-lynx-311). Both briefs together close
**criteria 1-4 of `substrate_gap_parser_grammar_closed` Class 1
5-criteria Pass** via parser-side (this brief) + emission-side
(S9 Phase-1 Step 3). **Criterion 5 (v2-oracle parity)** may carve
to a separate slice per Phase 2.3 / STOP-AND-ESCALATE bullet 1
language (consistent throughout this brief — same 4-of-5 framing).
Full 5/5 Pass closes when criterion 5 cementing test lands as
separate substrate work.
