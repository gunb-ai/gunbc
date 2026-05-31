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

## §8 Manager approval checklist (`cool-ibex-692`) — CLOSED 2026-05-30

- [x] Single-authority facts identified (collection realization + infer gate)
- [x] Spot-fixes forbidden (Ord patches, stub meet/join)
- [x] Two work items (not one "fix SG-5/6" blob)
- [x] Falsification probes accepted
- [ ] Worker dispatch — **authorized** (may parallel SG-2/SG-1 chain; independent substrate)

## Related artifacts

- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.3
- `docs/design-target-realization-canonical-home.md` §2 SG-3 sketch
- `docs/planning/v4-modeling-dfs-manager-pass-2026-05-30.md`
