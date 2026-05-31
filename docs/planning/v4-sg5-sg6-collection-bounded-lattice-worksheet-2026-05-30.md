# v4 SG-5 / SG-6 Worksheet — Collection realization + BoundedLattice completeness

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §10.0 sign-off 2026-05-30 (`cool-ibex-692`; manager pass completes §11.4 item 2).
> **Date:** 2026-05-30
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-05-29.md` SG-5 + SG-6 rows; `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.3.
> **Canonical home:** `src/v4/std/target_model.dag` for `TargetCollectionRealization`; BoundedLattice gate in `04_infer`.
> **Dispatch shape:** **Two** work items (one per systemic fix below).

---

## Mechanical dispatch rule

> **No SG-5 or SG-6 implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Acceptance is falsification-probe behavior per class, not compile_error stub count.

---

## §10.0-adapted worksheet

```text
SG class: SG-5 + SG-6
Representative emitted failure:
  // SG-5: BTreeSet<T> requires T: Ord; model declared no constraint on Set<T>.
  pub type LookupKeys = Rc<Set<NodeId>>;

  // SG-6: BoundedLattice instances emit meet/join as compile_error! stubs.
Immediate local patch:
  - SG-5: patch emit to add T: Ord everywhere Set<T> is used.
  - SG-6: patch emit to emit dummy meet/join bodies.
Why that patch is forbidden:
  - Fabricates guarantees the model never proved (INVARIANTS P3).
  - SG-5: calcifies BTreeSet without modeling HashSet/Vec alternatives.
  - SG-6: hides incomplete instances under stub bodies.
DFS path:
  std/ authority:
    - Set<T> = PointwisePower<T> at src/v4/std/collection.dag
    - BoundedLattice<T> at src/v4/std/algebra.dag with meet/join fields
    - No Ord-eligibility on Set<T>; instances may omit meet/join witnesses
  extdeps/language authority:
    - rust.dag: no Set collection realization (2026-05-30 spot-check)
  compiler stage consuming it:
    - 06_translate derives BTreeSet + Ord without substrate authority
    - 04_infer allows partial BoundedLattice without typed diagnostic
  existing scaffold/dissolution notes:
    - none; missing-substrate gap
Deepest unsound boundary:
  (a) Collection-target realization choice undeclared per language.
  (b) BoundedLattice instance completeness not gated at infer.
Systemic fix:
  (a) TargetCollectionRealization in v4.std.target_model; Rust rows in rust.dag;
      06_translate consumes primary_form + constraints + alternatives OR fail-closed.
  (b) 04_infer completeness gate: populate meet/join OR partial-instance diagnostic;
      no silent compile_error! stubs at emit.
Non-goals:
  - Rust-side T: Ord patches without model authority.
  - Defaulting to BTreeSet without modeled alternatives.
  - Dummy meet/join bodies.
Falsification probe:
  (a) Set<NonOrdable> in test/ → typed fail-closed OR alt representation — NOT silent Ord.
  (b) BoundedLattice missing meet → typed partial-instance diagnostic — NOT stub body.
Metric allowed only as secondary:
  SG-5/6 error counts are evidence only.
```

---

## Tightened worker briefs (two dispatches)

### SG-5 / TargetCollectionRealization

```text
Author TargetCollectionRealization carrier in v4.std.target_model.
Add Rust rows for Set (and Map if in catalog scope) in extdeps/languages/rust.dag.
Refactor 06_translate collection projection to consume rows.
Verify Set<NonOrdable> falsification probe.
```

### SG-6 / BoundedLattice completeness gate

```text
Add BoundedLattice-instance completeness gate in 04_infer.
Partial instances surface typed diagnostics at consumer sites.
Verify missing-meet falsification probe.
Emit must not fabricate meet/join stubs.
```

**Lane split:** Target Realization owns (a); Compiler Spine owns (b) per §11.3.

---

## §5 Landing order (implementation — not worksheet-only PR)

```text
1. TargetCollectionRealization carrier + choice/witness coproducts in v4.std.target_model.
2. Rust Set collection_realization row on rust TargetModel bundle (extdeps/languages/rust.dag).
3. 06_translate collection projection consumes primary + alternatives OR fail-closed fallback.
4. bounded_lattice_completeness.dag (cycle-breaker) + 04_infer consumer completeness gate.
5. Manual falsification claims: sg5_set_non_ordable_falsification + infer_bounded_lattice_completeness_anchor.
```

**Lane split:** Target Realization owns steps 1–3; Compiler Spine owns step 4; Runtime/TestClaim owns step 5.

---

## §6 Downstream worker brief (dispatch after §8)

```text
Implement SG-5 + SG-6 per approved worksheet §10.0 on main.

MUST:
  - Author TargetCollectionRealization in v4.std.target_model; Rust Set row(s) in rust.dag;
    refactor 06_translate collection projection to consume bundle rows (fail-closed fallback).
  - Author bounded_lattice_instance_completeness in v4.std.bounded_lattice_completeness;
    wire 04_infer consumer gate (partial declarations tagged; consumer sites reject).
  - Land manual falsification probes: Set<NonOrdable> fail-closed (SG-5 CompilesClaim);
    missing-meet BoundedLattice infer rejection (SG-6 CompilesClaim).
  - Include variant-shape histogram for new coproduct arms (v4-substrate-pr-review-gate §3).

MUST NOT:
  - Rust-side T: Ord patches without model authority; silent BTreeSet default; dummy meet/join at emit.
  - Fold SG-COLLECTION-PROJECTION FreeMonoid→Vec band into this PR without Modeling DFS ack.
  - Use SG-5/6 rustc error-count reduction as acceptance.

Escalate to Modeling DFS:
  - declared_inhabitants witness wiring required before dissolving 🟡 SG-5-element-trait-eligibility.
  - Structural set_carrier recognition without Cardinality predicate (🟡 SG-5-set-carrier-recognition).
  - Need Map collection row beyond catalog scope without worksheet amendment.
```

---

## §7 Non-goals

- SG-COLLECTION-PROJECTION (~170 FreeMonoid→Vec errors) — separate dispatch; do not fold into SG-5/6 PR
- SG-RC-LAYERING / SG-1b / emit-binary M1 probe as primary acceptance
- `ci.dag` / ci.yml / shell gate migration
- Dissolving 🟡 MVP-1 kernel-atom eligibility or set_carrier predicate without named dissolve-on

---

## §8 Manager approval checklist (`cool-ibex-692`) — CLOSED 2026-05-30

- [x] Single-authority facts identified (collection realization + infer gate)
- [x] Spot-fixes forbidden (Ord patches, stub meet/join)
- [x] Two work items (not one "fix SG-5/6" blob)
- [x] Falsification probes accepted
- [x] Worker dispatch — **authorized** (`keen-bat-825`; Modeling DFS GO 2026-05-31)

## Related artifacts

- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.3
- `docs/design-target-realization-canonical-home.md` §2 SG-3 sketch
- `docs/planning/v4-modeling-dfs-manager-pass-2026-05-30.md`
