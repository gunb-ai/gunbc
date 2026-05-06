---
status: draft (wait-window; awaits R3 host restoration before dispatch)
authority parent: R3 Substrate Manager (#1739)
ratification: Director (#828) ratified Option 2 + sub-questions §4.A=(a) / §4.B=List / §4.C=(i) / §4.D=(b) at cross-Mgr relay #issuecomment-4377533390 (2026-05-05); originating recommendation surfaced at #828 #issuecomment-4377459345
roadmap row: docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md (originating routing receipt; archived predecessor tidy-tern-769 #1288)
cross-mgr handoff: R3 Grounding (bold-ferret-748, #1745) — Grounding owns row-population follow-up + `coverage.rs` conversion
---

# R3 L6 — `EmissionPathProjection` substrate carrier slice

## Context

`src/v3/grounding_cross_target_meta/src/coverage.rs` projects L6
`(FormAxis × BehaviorAxis × ShapeATarget)` coverage by per-target
`MethodTemplateContract` list non-emptiness — honest only while
all Phase 1 rows are homogeneous (`Cardinality × Transform × target`).
The walker carries an in-source TODO to land per-row projection
before any non-`Cardinality × Transform` row enters the lists.

The routing-decision receipt at
`docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md`
selected Option 2 (sibling projection carrier keyed by
`MethodTemplateContractKey`) and STOPped pending Director sign-off
on §4.A–§4.D. R3 Substrate Mgr surfaced the (a)/List/(i)/(b)
recommendation at #828 #issuecomment-4377459345; Director ratified
that slate at the cross-Mgr relay routing carrier authoring back
to R3 Substrate.

This brief lands the **carrier slice only**. Grounding owns the
follow-up row-population PR + `coverage.rs` conversion (per §4.D=(b)).

## Slice

1. Author new file `src/v3/std/cross_target_coverage.dag` containing:

   Each new coproduct / sum declaration MUST carry a 🟢/🟡/🔴
   classification annotation per `docs/modeling-discipline.md`
   Practice 4 (Step 4 of the type-introduction checklist;
   `INVARIANTS.md#p1-modeling-faithfulness` worked example).
   The classifications below are pre-determined; worker copies
   them verbatim into the live source:

   - `ShapeATarget` → **🟢 TERMINAL** — closed set of three target
     languages (Rust / Python / Go). Adding a target is a
     substrate-scope event (new emitter, new method-template
     contracts), not a row-level addition; the 🟢 mark reflects
     that any new variant requires a P1 substrate-fact-introduction
     procedure on this declaration directly, not silent extension.
   - `FormAxis` → **🟡 SCAFFOLD** — hand-declared mirror of
     `v3_compiler::dag::TypeConnective`. Named dissolution trigger:
     option (c) substrate-`Disj`-mirroring generator (per Director
     ratification of §4.A at #issuecomment-4377533390). Generator
     landing retires this hand-declared form.
   - `BehaviorAxis` → **🟡 SCAFFOLD** — hand-declared mirror of
     `v3_compiler::dag::Behavior`. Same dissolution trigger as
     `FormAxis`.

   `MethodTemplateContractKey`, `EmissionCell`, and
   `EmissionPathProjection` are records (not coproducts) and do
   not require 🟢/🟡/🔴 marks per Practice 4 — the marks attach
   to sum-typed declarations whose variant set is the modeling
   choice. Worker re-verifies this against
   `docs/modeling-discipline.md` Practice 4 at dispatch time; if
   the discipline has been extended to require marks on records
   too, worker adds matching annotations.

   ```dag
   // Header note (mandatory): option (c) — generated mirror of
   // v3_compiler::dag::TypeConnective and v3_compiler::dag::Behavior
   // — is the correct dissolution target for the hand-declared
   // FormAxis / BehaviorAxis below, once a substrate-Disj-mirroring
   // generator exists. The (a) hand-declared form here is interim
   // until that generator lands. Per Director ratification of §4.A
   // at #issuecomment-4377533390.

   /// 🟢 TERMINAL — closed set of v3 target languages. New target
   /// addition routes through P1 substrate-fact-introduction
   /// procedure on this declaration; row-level addition is not
   /// authorized.
   type ShapeATarget = Rust | Python | Go

   /// 🟡 SCAFFOLD — hand-declared mirror of
   /// v3_compiler::dag::TypeConnective. Dissolution trigger:
   /// substrate-Disj-mirroring generator (option (c)).
   type FormAxis = ...      // mirrors v3_compiler::dag::TypeConnective discriminants (worker
                            // re-reads the live enum at dispatch time and lists the variants
                            // 1:1; new connectives added here only via P1 procedure on the
                            // upstream Rust enum first)

   /// 🟡 SCAFFOLD — hand-declared mirror of
   /// v3_compiler::dag::Behavior. Dissolution trigger:
   /// substrate-Disj-mirroring generator (option (c)).
   type BehaviorAxis = ...  // mirrors v3_compiler::dag::Behavior discriminants (same rule)

   type MethodTemplateContractKey {
     target: ShapeATarget
     dag_method: MethodRef
   }

   type EmissionCell {
     connective: FormAxis
     behavior: BehaviorAxis
   }

   type EmissionPathProjection {
     row_identity: MethodTemplateContractKey
     cells: List<EmissionCell>
   }

   data emission_path_projections: List<EmissionPathProjection> = []
   ```

   The `data` declaration ships **empty**. Grounding's follow-up
   PR populates the 42 Phase 1 rows.

2. Add a small ratchet asserting that:
   - The six types (`ShapeATarget`, `FormAxis`, `BehaviorAxis`, `MethodTemplateContractKey`, `EmissionCell`, `EmissionPathProjection`) and the `data` declaration exist with the
     ratified field shapes (typed-substrate read, no string scan).
   - When `emission_path_projections` is non-empty, its row count
     equals the sum of `MethodTemplateContract` row counts across
     `*_method_template_contracts` lists (row-count parity).
     The parity assertion is structurally trivial while the list
     is empty (vacuously true) and activates the load-bearing
     check once Grounding populates.

3. **No row population, no `coverage.rs` edits in this slice.**
   Both are scoped to the Grounding follow-up per §4.D=(b).

## Acceptance

- `src/v3/std/cross_target_coverage.dag` lands with the six
  types (`ShapeATarget`, `FormAxis`, `BehaviorAxis`, `MethodTemplateContractKey`, `EmissionCell`, `EmissionPathProjection`) + the empty `data` declaration + the mandatory
  option-(c) header note.
- **Practice 4 classification receipts on the live declaration
  (mandatory).** Each new sum-typed declaration carries an
  inline 🟢/🟡/🔴 doc comment per `docs/modeling-discipline.md`
  Practice 4 Step 4: `ShapeATarget` → 🟢 TERMINAL;
  `FormAxis` → 🟡 SCAFFOLD with named dissolution trigger
  (option (c) substrate-`Disj`-mirroring generator);
  `BehaviorAxis` → 🟡 SCAFFOLD with same dissolution trigger.
  The marks must be present on the `.dag` source in
  `cross_target_coverage.dag` itself, not only in the brief or
  PR body. The ratchet test (next bullet) verifies the marks
  are present.
- Ratchet test exists and passes (vacuous parity at this slice;
  carrier-shape gates structurally enforced; **classification
  marks present on each new sum-typed declaration**).
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile
  diagnostic ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- `MethodTemplateContract` rows untouched (Option 2 honors C1
  orthogonality — zero changes to authoring carriers).
- After landing, R3 Substrate Mgr signals R3 Grounding
  (bold-ferret-748, #1745) so they can dispatch row-population.

## STOP-AND-ESCALATE

- **`FormAxis` / `BehaviorAxis` variant set has drifted from
  `v3_compiler::dag::TypeConnective` / `Behavior` since the routing
  receipt was authored.** Worker re-reads the live Rust enums; if
  variant names or counts have moved, STOP — that's a P1
  modeling-faithfulness event for the upstream enum, and the
  hand-declared mirror cannot land into a divergent state. Surface
  to R3 Substrate Manager (#1739).
- **`MethodRef` shape has drifted** in a way that makes the
  proposed `MethodTemplateContractKey { target, dag_method:
  MethodRef }` join ambiguous against current
  `MethodTemplateContract` rows. STOP — the join is load-bearing
  (1:1 row-identity match) and silent re-shaping breaks Option 2's
  C2 (no string-name dispatch) guarantee.
- **A substrate-`Disj`-mirroring generator lands during this
  slice's authoring** (the option-(c) dissolution target). STOP —
  this slice's hand-declared form becomes immediately retire-able
  via the generator; rather than landing the interim form, surface
  for re-scoping toward generated mirrors.

## Cross-Mgr handoff post-landing

Once this slice lands, R3 Substrate Mgr signals R3 Grounding
(bold-ferret-748, #1745) per the routing receipt §4.D=(b)
hand-off. Grounding then dispatches:

- Row-population PR (42 Phase 1 rows, all `Cardinality × Transform
  × <target>` — single-element `cells` list per row).
- `coverage.rs` conversion from list-non-empty projection to per-row
  union over `emission_path_projections`.
- The row-count parity ratchet activates load-bearing at that
  point (no longer vacuous).

## Authority audit receipt

1. **Substrate exists?** No — `EmissionPathProjection` /
   `EmissionCell` / `MethodTemplateContractKey` /
   `cross_target_coverage.dag` do not exist at HEAD (verified by
   the routing receipt's `rg`). `MethodTemplateContract` exists
   in `src/v3/std/emit_model.dag`; this slice does not touch it.
2. **Existing brief?** The routing-decision receipt at
   `docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md`
   selected Option 2 and STOPped on §4.A–§4.D. This brief is the
   post-ratification authoring dispatch; the routing receipt
   remains the carrier-shape authority.
3. **Design-doc match?** The §4.A–§4.D recommendations in the
   routing receipt align with my surfaced (a)/List/(i)/(b) slate
   and Director's ratification of same. The carrier sketch in §1
   above mirrors the routing receipt §2 Option 2 sketch 1:1.
4. **Citations live?** Routing receipt verified at HEAD by
   wait-window scan (2026-05-05). `coverage.rs` TODO + walker
   shape verified at HEAD per the routing receipt's §1
   restatement.
5. **Carrier dissolves the bridge?** Yes — `EmissionPathProjection`
   carries the per-row projection fact as a typed row-local field,
   eliminating the list-non-empty proxy in `coverage.rs`.
   `MethodTemplateContractKey { target, dag_method }` provides 1:1
   row-identity for the source-row join; no string-name dispatch.
   The C1 orthogonality guarantee is honored (zero changes to
   `MethodTemplateContract` authoring rows).

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction (parallel-author posture); routing
receipt authored by archived predecessor tidy-tern-769 (#1288);
Director ratification arrived via cross-Mgr relay
#issuecomment-4377533390. Ratification pending host restoration
and parent dispatch slot allocation.
