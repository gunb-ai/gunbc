# Q-Reification — Proposal (R3)

**Status:** DRAFT — surfaced to Director #828 for ratification.
**Authored:** 2026-05-07 by `quiet-swift-285` (Director-tier proposal authoring under `zesty-bear-812`).
**Lane consumer:** R3 Substrate (post-ratification carrier authoring → `warm-wolf-698` #2068).
**Downstream blockers:** #1970 (Evaluator E3.c revised representative), #1960 (Substrate ReflectedProgram&lt;T&gt;
carrier), #1961 (Verification V1 TC1 first slice), #1844 / E3 Option 3 producer broadening.

---

## 1. Status quo

### 1.1 What exists

- **Complete reflection has landed (Rust-side, PR #1170).** `reflect_program_dag_nodes_in_file(program: &Dag,
  source_file: &str, id_space: &Dag) -> FieldValue` at `src/v3/compiler/src/lens_apply.rs:305` produces a
  substrate-shaped `FieldValue` projection of the program DAG. Coverage: every `Behavior` variant, every
  `BranchPath`, both `LoopBound` variants, optional fields, sum payloads. `design-reflection-completeness.md`
  (LOCKED 2026-04-29) is the design contract this implementation discharges.
- **Substrate carriers for the reflected shape are already declared in `.dag`.** `src/v3/std/substrate.dag` declares
  `Dag` (line 504-equivalent), `Behavior` (5-way), `BranchPath`, `BranchPattern`, `LoopBound`, `TransformTarget`,
  `Witness<Carrier>` (in `dimensions.dag`), `WorkflowEffect` (in `effects.dag`). Reflection consumes these
  declarations as the structural ground truth.
- **A consumer-shape proof exists.** PR #1857 / E6-G1.a Option 3 (`tests/integration/e6_g1a_option3_static_lens_test.rs`)
  wires a static lens through the existing reflection path. **It is deliberately argument-opaque** — the lens
  does not inspect reflected program structure — so it produces a constant `DimensionReport<C>` and yields
  **vacuous** η-equivalence (per `r3-evaluator-phase5-post-e3-closure-handoff.md` §Live Residuals).

### 1.2 What is missing

- **A non-vacuous lens-over-Dag fold.** TC1 V1 requires a lens whose output **structurally depends on** the
  reflected program (per `r3-v-tc1-eta-equivalence-deeper-analysis.md` §What TC1 Asserts). The current
  `FieldValue` carrier *can* carry the structure, but no consumer reads it structurally — every consumer routes
  around it via Rust-side substrate accessors instead (the parallel-representation debt named in
  `design-reflection-completeness.md` §2).
- **A name for the structural fact "reflected program."** Multiple briefs (#1957, #1960, #1961, E3 producer
  briefs, evaluator post-E3 audit) refer to a `ReflectedProgram<T>` carrier as the unblocker. **No such carrier
  exists** in `src/v3/std/` or anywhere in the substrate; the name circulates as a deferred-shape placeholder.
- **The bridge across the language seam.** `design-reflection-completeness.md` §7.2 names the dissolution path
  ("once the Evaluator can execute `.dag` body authority for the reflection projection, the Rust mirror is the
  dissolution target"); `substrate-reflection-design.md` proposes the dissolution carrier shape (lenses as `.dag`
  `fn check(d: Dag) -> List<_>`). These two docs **agree on the direction** but neither has been ratified as the
  Q-Reification disposition.

### 1.3 Grep verification (HEAD, 2026-05-07)

- `grep -r "Reification" src/v3/std/ --include="*.dag"` → 0 results
- `grep -r "ReflectedProgram\|Reified\|reify" src/v3/std/ --include="*.dag"` → 0 results
- `grep -rn "ReflectedProgram" docs/` → 9 hits, all in briefs/audits naming the *deferred* carrier; no design
  doc names a structural shape for it
- `grep -rn "ReflectedProgram\|reflected_program" src/v3/` → 1 hit (a comment in
  `tests/integration/e6_g1a_option3_static_lens_test.rs:9` deferring lens-over-Dag folding to "`ReflectedProgram<T>`
  / typed declaration-reference carrier work")

**Finding:** `ReflectedProgram<T>` is **not** an undeclared-but-implied substrate fact — it is a name circulating
in worker briefs as a placeholder for a carrier the substrate has *not yet been asked to introduce*. The structural
fact it would name (the program, in lens-input shape) **already inhabits the substrate** as the declared `Dag`
type at `src/v3/std/substrate.dag`. This is the load-bearing observation behind §4 below.

---

## 2. The question

**Q-Reification:** What is the substrate carrier that names *"the program, in the shape a lens consumes,"* such
that a lens can fold over it non-vacuously and the η-equivalence obligation in TC1 V1 cashes structurally rather
than vacuously?

Equivalently (and more bluntly): does Q-Reification authorize a **new** substrate carrier (`ReflectedProgram<T>`),
or does it **ratify the structural fact** that the existing `Dag` carrier in `src/v3/std/substrate.dag` already
*is* the reflected program — with the consumers' job being to fold over it directly?

This is structurally a P1 substrate-fact-introduction procedure (per `INVARIANTS.md` §P1 +
`feedback_construction_over_ratchets`). The disposition determines whether Substrate Mgr authors a new carrier
or whether the existing one is the carrier and consumer wiring (Evaluator E3 + Verification TC1 V1) is the only
work left.

---

## 3. Options

### Option A — `Dag`-as-carrier (zero new substrate; ratify existing fact)

**Shape.** No new substrate type. Q-Reification ratifies that `src/v3/std/substrate.dag::Dag` *is* the reflected
program in lens-input shape. Lenses are written as `.dag` functions of signature `fn check(d: Dag) -> Output`
consuming `Dag` directly via field access (`d.nodes`, `d.declarations`). The `FieldValue` projection in
`lens_apply.rs` is the bootstrap-staging implementation; the dissolution target is "lens runs through Evaluator
over `Dag`," which is the path `design-reflection-completeness.md` §7.2 already names.

**This is the shape `substrate-reflection-design.md` argues for** (§3.0–§3.6, decisions §3.5/1c and §3.6/1a).
The design doc explicitly rejects a separate query-primitive layer (§3.2 "queries ARE field access") and a
separate reflected-program type, on the grounds that re-naming a carrier the substrate already declares is
parallel-representation debt at the lens-input boundary.

**Tradeoffs.**

- **Pro — zero scaffold growth.** No new substrate variant, no `ReflectedProgram<T>` declaration, no realization
  entry in `rust.dag`. Q-Reification reduces to "the question dissolves on the existing carrier."
- **Pro — passes `feedback_no_metadata_markers` cleanly.** The hypothetical `<T>` on `ReflectedProgram<T>` would
  be a phantom-tag distinguishing "this `Dag` is for lens consumption" from "this `Dag` is the program itself" —
  a metadata distinction without structural content. Option A removes that pressure by construction.
- **Pro — passes `feedback_dissolve_bridges`.** No string-identity bridge between `Dag` and `ReflectedProgram<T>`;
  no per-consumer projection function (`reflect_for_lens_X`); no Rust-side hand-rolled `reflect_behavior` tail
  surviving past the dissolution target named in `design-reflection-completeness.md` §6 invariant 4.
- **Pro — algebraic axioms verifiable in place.** The η-equivalence obligation TC1 V1 cashes is "reading
  `Dag` fields structurally produces a fold whose output depends on every reflected-shape fact." That property
  is verifiable directly against `Dag`'s declared shape (§8.1–§8.3 of `design-reflection-completeness.md` already
  enumerate the fixtures: every-field round-trip, branch-arm totality, loop-bound coproduct).
- **Pro — unblocks #1960, #1961, #1970 by absorption.** #1960 (`ReflectedProgram<T>` carrier) becomes "no carrier
  needed; consumer wiring through `Dag`." #1961 (TC1 V1 first slice) consumes the existing `Dag` carrier directly
  via the lens written in `.dag`. #1970 (Evaluator E3.c revised representative) re-pairs on a non-vacuous fold
  consuming `Dag` rather than argument-opaque `Dag`/`Behavior` inputs.
- **Con — requires the prereq slate from `substrate-reflection-design.md` §11.** Compositional `.dag` lenses
  need: Prereq 0 (template instantiation for function-typed parameters), Prereq 1 (field-access on local
  variables → `TransformTarget::FieldProject`), Prereq 2 (match-with-payload), Prereq 3 (lambda), Prereq 4
  (`list.dag`). **Many of these are landed or in flight** (Prereq 0 in `substrate-reflection-design.md` §11 shows
  ✓ for all five sub-checkboxes; Prereq 0.5 implicit-instantiation work in flight). The remaining slate is
  Substrate-lane work, not Q-Reification's lane.
- **Con — the path through `Dag` requires Evaluator runtime authority for the lens's higher-order folds.**
  PR-B (Evaluator runtime-value model) is the prerequisite per `design-reflection-completeness.md` §7.2. This is
  a sequencing constraint, not a structural one — the Evaluator landing is an existing R3 commitment.
- **Con — bootstrap-staging carrier `FieldValue` survives until the lens-via-Evaluator path lands.** This is a
  same-slice dissolution (per `feedback_same_slice_dissolution_discipline`): the acceptance gate names *one
  consumer wired through `Dag` directly*, not "FieldValue removed."

**4-pattern dissolution check (per `feedback_state_space_vs_behavioral_invariants` + `INVARIANTS.md` §P1):**

- *Pattern 1 (fact placement):* The fact "the program, in lens-input shape" already has a home — `Dag`. Adding a
  second home is parallel representation.
- *Pattern 2 (variant-is-data):* `Dag` is a record; there is no coproduct distinction Q-Reification needs to
  surface.
- *Pattern 3 (algebraic form):* The lens's algebra is downstream of reflection (per
  `design-reflection-completeness.md` §3 closure principle: reflection is structural, algebra is downstream).
  Reification does not name an algebra.
- *Pattern 4 (dimensional):* The `<T>` slot on a hypothetical `ReflectedProgram<T>` is dimensional with respect
  to "what algebra the consumer applies" — but that dimension lives **on the consumer**, not on the carrier. A
  lens parameterized on its output algebra carries the `<T>` itself; the input is just `Dag`.

**Verdict on 4-pattern:** TERMINAL. No new carrier authorized.

### Option B — `ReflectedProgram<T>` as a typed wrapper around `Dag`

**Shape.** Introduce a new substrate carrier:

```
type ReflectedProgram<T> {
  dag: Dag
  // T is a phantom witness of the lens-output algebra the program will be folded into
}
```

…in `src/v3/std/substrate.dag` (or a new `reification.dag`), realized in `rust.dag` as a Rust newtype wrapping
`Dag`. Lenses consume `ReflectedProgram<T>` instead of `Dag`. The `<T>` slot binds the lens's downstream algebra
at the carrier level.

**Tradeoffs.**

- **Pro — names the deferred carrier worker briefs cite.** #1960 / #1961 / #1970 reference `ReflectedProgram<T>`
  by name; introducing it discharges the textual reference directly.
- **Pro — type-level discrimination of "raw program DAG" vs "program DAG presented to a lens."** Some readers find
  this clearer.
- **Con, blocking — phantom-tag metadata violation.** The `<T>` parameter carries no structural content; it is a
  marker that distinguishes "this `Dag` is being read by a lens parameterized on `T`" from "this `Dag` is the
  program itself." Per `feedback_no_metadata_markers`, type-level phantom witnesses that exist only to label a
  carrier without changing its shape are the same anti-pattern as `__is_reified` string markers.
- **Con, blocking — parallel representation debt at the lens-input boundary.** `ReflectedProgram<T>` has the same
  structural fields as `Dag` (one field: `dag: Dag`) plus a phantom. Every consumer that wants the structural
  facts unwraps `.dag` and reads through. Per `feedback_parallel_representation_debt`, naming the same fact in
  two carriers is the failure class the project actively prevents.
- **Con — bridge construction.** Going from `Dag` (the substrate fact) to `ReflectedProgram<T>` (the lens-input
  carrier) requires a constructor: `fn reify(d: Dag) -> ReflectedProgram<T>`. That constructor is a string-identity
  bridge between two carriers that should be the same carrier — the canonical anti-bridge violation per
  `feedback_dissolve_bridges`.
- **Con — kernel-pollution risk.** Adding a new substrate carrier expands the kernel set per `INVARIANTS.md` C-1
  bounded-kernel discipline. The substrate audit (per `substrate-reflection-design.md` §3.0 seed-minimality
  invariant) ratchets *downward*; introducing a new wrapper carrier moves the count the wrong way.

**4-pattern dissolution check:**

- *Pattern 1 (fact placement):* The fact has a home (`Dag`); this option duplicates it. **FAIL.**
- *Pattern 2 (variant-is-data):* `ReflectedProgram<T>` is a record with one structural field; the `<T>` is
  metadata. **FAIL by metadata-marker discipline.**
- *Pattern 3 (algebraic form):* The `<T>` slot models lens-output algebra at the wrong layer (the carrier, not
  the lens body). **FAIL.**
- *Pattern 4 (dimensional):* The dimension `<T>` decomposes into "lens output type," which lives on the lens, not
  on the carrier. **FAIL.**

**Verdict:** rejected by 4-pattern check. (Surfaced for completeness because briefs cite the name.)

### Option C — Status-quo `FieldValue` carrier survives, lenses fold via Rust-side helpers

**Shape.** Keep reflection emitting `FieldValue` as today; expose a Rust-side helper API for lenses to fold over
it; no migration to `.dag`-authored lenses.

**Tradeoffs.**

- **Con, blocking — violates `design-reflection-completeness.md` §6 invariant 4.** "No Rust-side hand-rolled
  `reflect_behavior` tail" is an explicit anti-bridge invariant; this option preserves exactly that.
- **Con — no path to non-vacuous TC1 V1.** Argument-opaque consumers (the current Option 3 representative) stay
  vacuous; nothing changes structurally.
- **Verdict:** rejected — it is the status quo TC1 V1 is HELD against, not a forward direction.

---

## 4. Recommendation

**Recommend: Option A — `Dag`-as-carrier (no new substrate carrier).**

**Rationale.**

1. **The structural fact already exists.** `src/v3/std/substrate.dag::Dag` declares the program shape exhaustively
   (per the §1.3 grep + the LOCKED `design-reflection-completeness.md` spec). Q-Reification's job is to *ratify*
   that fact, not to author a new one. This is the construction-over-ratchets discipline applied at the
   ratification layer (`feedback_construction_over_ratchets`).

2. **Option B fails four invariants.** Phantom-tag metadata, parallel-representation debt, string-identity
   bridge, kernel-pollution. Each is independently load-bearing per existing feedback discipline.

3. **The dissolution path is named in two design docs that agree.** `design-reflection-completeness.md` §7.2
   ("Evaluator landing is the path through which Rust-side `reflect_behavior` retires") +
   `substrate-reflection-design.md` §3.2/§3.5/§3.6 (lenses as `.dag` functions over `Dag`, decisions 1c/1a locked
   for the prereq slate). Option A is the disposition that closes the gap between these two locked design
   surfaces.

4. **Worker briefs that cite `ReflectedProgram<T>` are pre-auth-queued for patch.** Per
   `r3-v-pattern-a-tc1-v1-worker.md` line 3 ("Pre-auth queue: patch this brief when Q-Reification / carrier land
   — do not replace wholesale"), the briefs anticipated the disposition and only need the carrier name
   reconciled. Patch `ReflectedProgram<T>` → `Dag` (with the consumer-wiring nuance: lens fold consumes `Dag`
   via `.dag` body authority through the Evaluator).

5. **Algebraic axioms verify at proposal time.** TC1 V1's η-equivalence obligation is: "η-rewrites of the lens
   body produce equivalent `DimensionReport<C>` outputs over the same reflected program." Under Option A, this
   reduces to "the lens body, written as a `.dag` `fn check(d: Dag) -> DimensionReport<C>`, exhibits η-equivalence
   under standard β/η rewrites of the substrate's lambda calculus" — which is a property of the substrate's
   evaluation semantics, not a property the carrier imposes. The `<T>` on a hypothetical `ReflectedProgram<T>`
   would not contribute to that proof.

---

## 5. Acceptance gates (same-slice)

Per `feedback_same_slice_dissolution_discipline`: the same R3 slice that ratifies Q-Reification must land at
least one downstream consumer wired through `Dag` directly, demonstrating the non-vacuous fold. **Same-slice
acceptance, not future-scope.**

**Gate A — Substrate side (Substrate Mgr `warm-wolf-698` #2068).**

- [ ] Q-Reification disposition recorded in `docs/r3-program-plan.md` §10.3 as "RATIFIED: `Dag` is the reflected
      program; no new carrier" with the row pointing to this proposal doc.
- [ ] `r3-v-pattern-a-tc1-v1-worker.md` patched: `Q-Reification + ReflectedProgram<T>` → `Q-Reification (Option A:
      Dag-as-carrier)` per pre-auth-queue clause.
- [ ] Receipt that no new `.dag` declaration was added to `src/v3/std/` for the carrier (pass-by-construction
      verification of Option A).

**Gate B — Consumer side (Verification Mgr `wise-bear-525` + Evaluator Mgr `crisp-bat-13`).**

- [ ] One TC1 V1 consumer slice landed where the lens body folds over `Dag` non-vacuously: at minimum, reads
      `d.nodes` and produces a `DimensionReport<C>` whose value depends on the count or shape of reflected
      `Behavior` variants (i.e., not constant).
- [ ] The same fixture under η-rewrite produces an equal `DimensionReport<C>` (this *is* TC1 V1's
      `tc1_eta_equivalence_executable` predicate, now non-vacuous).
- [ ] The fold path runs through Evaluator's substrate-fact projection authority (per
      `design-reflection-completeness.md` §7.2 dissolution target), not through `lens_apply.rs`'s Rust-side
      `reflect_behavior` tail.

**Gate C — Audit side (R3 Debt-Paydown Mgr `gentle-newt-665`).**

- [ ] `r3-evaluator-phase5-post-e3-closure-handoff.md` §Live Residuals updated: "Q-Reification / reflected program
      carrier" entry → CLOSED via Option A; the post-E3 audit queue's trigger #4 ("Q-Reification carrier work
      lands or changes the accepted reflected-program fold boundary") fires and the queue runs once to confirm
      no overclaim.

These three gates are same-slice (one R3 PR cluster, dispatched together post-ratification). Future-scope work
explicitly *outside* this slice: full `lens_apply.rs` Rust-side `reflect_behavior` retirement (deferred to
T-LensProducer-Retirement, which already gates on this proposal); migration of *all* lenses to `.dag` (per
`substrate-reflection-design.md` §12.5 M1–M4 staged rollout).

---

## 6. Open questions (deferred to Director)

1. **§10 Q3 from `substrate-reflection-design.md` (List realization — Disj vs Cardinality).** The reflected program
   carries `List<Behavior>` (`Dag.nodes`) and `List<Declaration>` (`Dag.declarations`). Per the reflection
   completeness fixtures, lenses fold these lists. `substrate-reflection-design.md` §10 Q3 flags an open question:
   are `List<T>` realized as `Disj` (Empty | Cons) for static pattern matching, or as `Cardinality(Unbounded, T)`
   for contiguous-memory targets? Under Option A, the lens reads `d.nodes` as a `List<T>`; the realization
   question affects which fold shapes lenses can express. **Director disposition:** ratify as deferred to the
   Prereq 4 (`list.dag`) PR (i.e., not blocking Q-Reification ratification), or surface as a sub-question that
   blocks?

2. **§10 Q2 (`result_port` canonical field name).** Today Rust's `BindNode.value`, `TransformNode.output`, etc.
   are not uniform. The `substrate-reflection-design.md` §3.2 design proposes renaming Rust fields to
   `result_port` to match the `.dag` canonical name. This is consumer-side cleanup; **does it block
   Q-Reification, or is it a follow-up Substrate hygiene PR?** Recommend: follow-up.

3. **`BranchNode` primary result port (§10 Q2 sub-question).** Per `design-reflection-completeness.md` §3.2.1
   note, `BranchNode` has no direct `result_port` because the output is per-path. Lenses folding over branches
   need to handle this asymmetry. Should the proposal name the asymmetry as a consumer-side fold pattern, or
   should Substrate be asked to introduce a "primary path" structural fact? **Recommend:** per-path output is
   the structural truth; lenses fold accordingly. No substrate change.

4. **Adjacent Q-* sharing authority.** `Q-EVAL-Lens-Fold-First-Slice` (Path A G1.a static representative,
   ACCEPTED 2026-05-06) is the *consumer-side* sequencing decision; Q-Reification is the *carrier-side*
   ratification. They share authority at the dispatch boundary. **Recommend joint patch:** when Q-Reification
   ratifies, `Q-EVAL-Lens-Fold-First-Slice` row updates to cite the Option A disposition and unholds TC1 V1.
   No counter-decision needed; the two rows align.

5. **`lens_apply.rs` Rust-side mirror retirement timing.** `design-reflection-completeness.md` §6 invariant 4
   names "no Rust-side hand-rolled `reflect_behavior` tail" as the post-T-LensProducer-Retirement state. Does
   Q-Reification ratification commit to that retirement same-slice, or defer to T-LensProducer-Retirement's own
   slice (gated on PR-B Evaluator runtime-value model + the Prereq slate)? **Recommend:** defer the Rust-side
   retirement to T-LensProducer-Retirement; Q-Reification ratifies the *carrier shape* (Option A), and the Rust
   mirror dissolves on its own schedule. This keeps the same-slice gates in §5 narrow and shippable.

---

## Cross-references

- `docs/design-reflection-completeness.md` — LOCKED 2026-04-29; defines complete reflection structurally; this
  proposal disposes the carrier-shape question that doc explicitly defers to ratification.
- `docs/substrate-reflection-design.md` §3.0–§3.6 + §10 — proposes the `Dag`-as-carrier shape and prereq slate
  this proposal recommends ratifying.
- `docs/r3-program-plan.md` §10.3 row Q-PAFS / Q-EVAL-Lens-Fold-First-Slice / Q-Reification — table rows this
  proposal updates on ratification.
- `docs/briefs/r3-v-pattern-a-tc1-v1-worker.md` — pre-auth-queued for patch on Q-Reification landing.
- `docs/audit/r3-evaluator-phase5-post-e3-closure-handoff.md` §Live Residuals — Q-Reification entry closed by
  this disposition.
- `docs/briefs/r3-pr-e6-g1a-option3-static-lens-worker.md`, `docs/briefs/r3-pr-e6-g1a-option3-feasibility-probe.md`,
  `docs/briefs/r3-pr-e8-w1-producer-contract-test-plan-worker.md` — cite `ReflectedProgram<T>` deferred carrier;
  patch on ratification.
- `INVARIANTS.md` §C-1 (bounded kernel), §C-3 (decidability), §C-8 (fail-closed), §P1 (substrate-fact-introduction
  procedure), §P2 (single-authority).
- `feedback_no_metadata_markers`, `feedback_dissolve_bridges`, `feedback_construction_over_ratchets`,
  `feedback_same_slice_dissolution_discipline`, `feedback_parallel_representation_debt`,
  `feedback_state_space_vs_behavioral_invariants`, `feedback_grep_substrate_before_naming_ratification`.

---

## Summary

**Recommendation:** Option A — ratify that the existing `src/v3/std/substrate.dag::Dag` carrier *is* the reflected
program; introduce no new substrate carrier; consumer wiring (Evaluator E3 producer, TC1 V1, lens-via-Evaluator
fold) is the only same-slice work. Option B (`ReflectedProgram<T>` typed wrapper) fails four invariants and is
rejected. Same-slice acceptance: one non-vacuous TC1 V1 consumer slice landed; pre-auth-queued briefs patched;
audit residual closed.
