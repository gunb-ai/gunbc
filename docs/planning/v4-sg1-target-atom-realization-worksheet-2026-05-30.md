# v4 SG-1 Worksheet — Kernel-ambient atoms / `TargetAtomRealization`

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §10.0 sign-off 2026-05-30 (`cool-ibex-692`; manager pass completes §11.4 item 2).
> **Date:** 2026-05-30
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-05-29.md` SG-1 row (~2978 E0423); `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.1, §10.1.1.
> **Canonical home:** `src/v4/std/target_model.dag` (`v4.std.target_model`) — ratified `docs/design-target-realization-canonical-home.md` §1 Option A.
> **Prerequisite:** SG-2 worksheet approved and **SG-2 worker PR landed** before SG-1 worker starts (`type_form` consumes `TargetTypeExpressionProjection`).

---

## Mechanical dispatch rule

> **No SG-1 implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Acceptance is dual-emit falsification + bidirectional readability (§10.6), not E0423 count reduction.

---

## §10.0-adapted worksheet

```text
SG class: SG-1
Representative emitted failure:
  pub fn loop_bound_edge() -> String {
      Symbol("loop_bound_edge".to_string())
  }
  // Symbol used as constructor on a type alias `type Symbol = String;`
Immediate local patch:
  Patch 06_translate value path so Symbol("x") becomes "x".to_string().
Why that patch is forbidden:
  Leaves type-emit and value-emit independently deriving Symbol's Rust realization.
  INVARIANTS P2 single-authority violation remains. Calcifies Symbol-as-edge-tag
  pattern std/node.dag:84-85 gates for dissolution (forbidden new consumers).
DFS path:
  std/ authority:
    - type Symbol bare at src/v4/std/node.dag:10 (kernel-ambient)
    - loop_bound_edge gated at src/v4/std/node.dag:84-85 (T-12; do not expand)
  extdeps/language authority:
    - rust.dag imports Symbol; no atom realization rows (2026-05-30 spot-check)
  compiler stage consuming it:
    - 06_translate type vs value paths derive Symbol independently today
  existing scaffold/dissolution notes:
    - node.dag:84-85 — Symbol-tagged Loop bound dissolution (T-12)
    - design-target-realization-canonical-home.md §3 — COEXIST bool/char sentinels;
      FORBIDDEN to delete rust_facts_* bundles in SG-1 PR
Deepest unsound boundary:
  Missing TargetAtomRealization. Carrier ONCE in v4.std.target_model;
  per-language rows in extdeps/languages/<lang>.dag.
Systemic fix:
  TargetAtomRealization { source_carrier: Node, target_model, type_form,
  value_form, constructor_form, display_name }.
  type_form instance of SG-2 TargetTypeExpressionProjection (no parallel vocab).
  Rust rows for Symbol, Bool, Char — additive only per §3 scaffold disposition.
Non-goals:
  - Value-emit template patch (layer-1 spot fix).
  - Rust-only carrier name.
  - loop_bound_edge / Symbol-tag Loop dissolution (T-12).
  - Deleting rust_facts_bool/char or rust_noninteger_facts_catalog members.
  - SG-CANDIDATE-1 parser sugar as prerequisite.
Falsification probe:
  Change Symbol realization row; verify BOTH type and value translate paths change.
  Grep 06_translate for string-keyed Symbol projection — expect zero after refactor.
Metric allowed only as secondary:
  ~2978 E0423 — evidence only.
```

---

## Tightened worker brief (dispatch downstream)

Verbatim constraints from `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.1 tightened brief, plus:

- Canonical home: `src/v4/std/target_model.dag`
- Consumer: `src/v4/compiler/06_translate.dag` (type + value paths)
- Scaffold table: `docs/design-target-realization-canonical-home.md` §3 (COEXIST; forbidden deletions)
- §10.6 bidirectional-readability falsification on Symbol/Bool/Char rows

---

## §8 Manager approval checklist (`cool-ibex-692`) — CLOSED 2026-05-30

- [x] Single-authority fact: `TargetAtomRealization` in `v4.std.target_model`
- [x] Cross-section: `type_form` consumes SG-2 substrate only
- [x] Scaffold disposition COEXIST ratified (inherits proud-pike-680 msg_7bf34553)
- [x] Spot-fix forbidden: value-only Symbol template patch
- [x] Falsification + bidirectional probes accepted
- [x] Prerequisite: SG-2 landed before worker start
- [ ] Worker dispatch — **authorized** after SG-2 PR; handoff to Target Realization Manager

## Related artifacts

- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.1, §10.1.1, §10.6
- `docs/design-target-realization-canonical-home.md` §3–§5
- `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md`
