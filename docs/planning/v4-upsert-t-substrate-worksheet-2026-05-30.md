# v4 Phase 1.4 Worksheet — `Upsert<T>` substrate landing (`dsl/std/patterns.dag`)

> **Status:** WORKSHEET DRAFT — Modeling DFS Manager §8 pending (proud-pike-680; `adhoc-4155bd37-f57`).
> **Date:** 2026-05-30
> **Dispatch anchor:** PR #3959 §6 Phase **1.4**; blocks approved Phase 1.5 `CiUpsertStep<T>` (`docs/planning/v4-ci-schema-worksheet-2026-05-30.md`).
> **Authority:** Operator canon 2026-05-29 — `dsl/std/patterns.dag` UPSERT\<T\> header (L15–38); `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`.

---

## Mechanical dispatch rule

> **No Phase 1.5 `CiUpsertStep<T>` substrate worker may land until Phase 1.4 completes and this worksheet is Modeling DFS Manager–approved.**

Phase 1.5 schema worksheet is already §8-approved **contingent** on 1.4. Acceptance here is **usable type substrate + honest stub disposition**, not pattern-sugar completeness unless parser gate clears in the same landing PR.

---

## §10.0-adapted worksheet

```text
Substrate class:        PHASE-1.4 (Upsert<T> generic primitive in dsl/std/patterns.dag)
Representative failure:  Phase 1.5 CiUpsertStep<T> = Upsert<T>{...} has no Upsert<T> target;
                         patterns.dag has operator canon + commented pattern bodies only;
                         fn content_upsert stub (S1) misleads tools.bootstrap/readme.
Immediate local patch:   Define CiDependencySource in workflow/ci.dag only; keep content_upsert
                         stub; add Upsert<T> as empty marker type in patterns.dag without phases.
Why forbidden:           Parallel CI-only carrier without patterns.dag home (P2); leaving S1
                         stub authoritative (audit S1/S3); pattern generics bypass that lands
                         CiUpsertStep without generic Upsert<T> type (Phase 1.5 worksheet lie).
DFS path:
  dsl/std authority:
    - dsl/std/patterns.dag — UPSERT<T> canon header; commented ensure/upsert/content_upsert (L127-157)
    - dsl/std/effects.dag — UpsertEffect witness (runtime meet on Map<K,V>)
    - dsl/std/types.dag — FilePath, ContentHash (downstream CI inputs — not re-defined here)
  v4 consumers (blocked until 1.4):
    - src/v4/workflow/ci.dag — CiUpsertStep<T> specialization (Phase 1.5)
    - docs/planning/v4-ci-schema-worksheet-2026-05-30.md §1.2–1.3
  parser / compiler:
    - ROADMAP desired parser feature: generic pattern declarations (pattern ensure<Check, Action>)
    - docs/audit/upsert-pattern-compiler-stray-2026-05-29.md §4 A1–A2
  tools strays (dissolve with 1.4 landing family):
    - dsl/tools/bootstrap.dag, readme.dag — import content_upsert (S3)
    - dsl/shared/dag_util.dag — render-then-upsert doc drift (S2)
Deepest unsound boundary:
  Operator canon exists as prose + commented patterns, but no **type-level** Upsert<T>
  specialization target; CI overhaul cannot bind CiUpsertStep without it.
Systemic fix:
  Land §1 Upsert<T> + phase carrier types in patterns.dag; mark fn content_upsert 🟡 dissolve-on;
  track parser A1 (pattern generics) as explicit follow-on OR same PR if parser lane delivers.
Non-goals:
  - Full uncomment of all patterns (transaction/retry) in one PR
  - Phase 1.5 ci.dag migration / CiSelectionReceipt active skip
  - Replacing UpsertEffect algebra
  - Resolving every audit S-row in one PR (A3–A5 optional siblings)
Falsification probe:
  (1) patterns.dag parses with Upsert<T> + phase types referenced from v4.workflow.ci stub import.
  (2) fn content_upsert carries 🟡 dissolve-on → pattern content_upsert (or deleted with tools fixed).
  (3) No new CI-only Upsert parallel type outside patterns.dag.
  (4) Phase 1.5 type alias CiUpsertStep<T> = Upsert<T>{...} typechecks in ci.dag (gate for 1.5 PR).
Metric allowed only as secondary:
  Count of tools importing real content_upsert — after A2, not before type lands.
```

---

## §1 Authoritative substrate catalog

### 1.1 `Upsert<T>` — generic four-phase carrier (Phase 1.4 delivers)

```dag
// dsl/std/patterns.dag — canonical home (NEW types in landing PR)

// Phase bodies are typed nodes/expressions, not Symbol strings (P2).
type VerifyCheck = Node      // 🟡 until dedicated VerifyCheck substrate exists
type CreateAction = Node
type ResolveExpr = Node

type Upsert<T> {
  verify: VerifyCheck
  create: CreateAction
  resolve: ResolveExpr
  // `inputs` NOT on generic Upsert<T> — CI specialization adds via CiUpsertStep (Phase 1.5)
  // Phantom/branding: T is the step payload specialization (e.g. VerificationReport, CiPipeline)
}
```

**Operational semantics** (unchanged from canon header): verify-first → recursive deps → create-if-missing → cache-outcome. Phase 1.4 types **encode** the shape; interpretive wiring remains T-22 / workflow eval consumers.

**Relationship to pattern sugar** (deferred sub-gate):

```dag
// Target when parser A1 lands (audit §4 A1–A2) — NOT blocking 1.4 type substrate:
// pattern upsert<Check, Create, Resolve: -> R>(...) -> { value: R }
```

### 1.2 Specializations (reference — not re-authored)

| Name | Phases | Phase 1.4 status |
| ---- | ------ | ---------------- |
| `ensure<Check, Action>` | 1–3 | Commented — parser generics |
| `upsert<Check, Create, Resolve>` | 1–4 | Commented — parser generics |
| `content_upsert` | 1–4 filesystem | Stub `fn` → 🟡 dissolve-on; real `pattern` after A1+A2 |
| `UpsertEffect` | runtime witness | **Exists** — `dsl/std/effects.dag` (no change) |

### 1.3 `content_upsert` stub disposition (audit S1)

| Current | Required in 1.4 landing PR |
| ------- | -------------------------- |
| `fn content_upsert` fake `content == ""` | 🟡 mark + comment: **non-authoritative** until `pattern content_upsert` lands |
| tools import stub | Document in PR; **A4** may follow in tools lane (non-blocking 1.4 if marked) |

---

## §2 Parser / substrate prerequisites (explicit gate)

| Gate | Owner | 1.4 requirement |
| ---- | ----- | ---------------- |
| **P1.4-TYPE** | Modeling DFS + landing PR | `type Upsert<T>` + phase aliases parse in `patterns.dag` |
| **P1.4-PARSER** (A1) | Parser / class-5 | Generic `pattern upsert<...>` uncomment — **may trail** type landing |
| **P1.4-RESOURCES** | Compiler | `uses fs:` in patterns — still blocked for real `content_upsert` (separate from CI Upsert) |

**Manager ruling:** Phase **1.4 DONE** when **P1.4-TYPE** + falsification (1)(3)(4) pass. **P1.4-PARSER** is a tracked follow-on (audit A1) unless delivered in the same PR.

---

## §3 DFS concept-home map (M9)

```text
Concept              | Home                         | Phase 1.4 action
---------------------|------------------------------|------------------
Upsert<T> type         | dsl/std/patterns.dag         | DEFINE §1.1
Phase node carriers    | dsl/std/patterns.dag         | VerifyCheck/CreateAction/ResolveExpr
Pattern wiring         | dsl/std/patterns.dag (commented) | Unblock via P1.4-PARSER
Runtime witness        | dsl/std/effects.dag UpsertEffect | CONSUME only
CI specialization      | v4.workflow.ci CiUpsertStep  | Phase 1.5 (blocked)
Cache derivation       | v4.std.node content_hash     | Phase 1.5 (blocked)
```

---

## §4 Spot-fix register (forbidden)

| Pattern | Why forbidden |
| ------- | ------------- |
| `Upsert<T>` defined only in `v4/workflow/ci.dag` | Parallel authority — CI cannot own generic canon |
| `inputs: List<Symbol>` on generic `Upsert<T>` | String-keyed — belongs on `CiUpsertStep` only (Phase 1.5) |
| Keeping stub `content_upsert` without 🟡 mark | Audit S1 — false idempotency |
| Empty marker `type Upsert<T> = Unit` | Cannot specialize `CiUpsertStep<T>` with phases |
| Skipping falsification (4) before 1.5 dispatch | Phase 1.5 worksheet contingent on real target type |

---

## §5 Falsification probes (acceptance)

1. **Type landing:** `dsl/std/patterns.dag` exports `Upsert<T>` with `verify` / `create` / `resolve` fields per §1.1.
2. **CI alias gate:** In a staging change (may be Phase 1.5 PR), `type CiUpsertStep<T> = Upsert<T> { inputs: List<UpsertInputRef>; ... }` typechecks — proves 1.4 target is real.
3. **Stub honesty:** `content_upsert` stub marked 🟡 non-authoritative or removed with tools migrated.
4. **No parallel canon:** Grep — no second `Upsert<T>` outside `patterns.dag` + allowed specializations.
5. **Parser track:** If A1 not in PR, open follow-on with audit §4 A1 owner — documented in PR body.

---

## §6 Downstream worker brief (after §8 approval)

```text
Land §1.1 types in dsl/std/patterns.dag (+ imports as needed).

MUST:
  - Add Upsert<T> + VerifyCheck/CreateAction/ResolveExpr per §1.1
  - Mark or remove content_upsert stub per §1.3
  - Document P1.4-PARSER follow-on if pattern bodies stay commented

MUST NOT:
  - Any §4 forbidden pattern
  - Phase 1.5 ci.dag CiUpsertStep rows (separate dispatch after 1.4 merge)

Escalate to Modeling DFS:
  - If Upsert<T> cannot parse without parser changes not in scope
  - If VerifyCheck must be non-Node (new std module) — worksheet amendment required
```

---

## §7 Non-goals

- Uncomment `transaction` / `retry` patterns.
- Real filesystem `content_upsert` (needs `uses fs` binding).
- Phase 2.5 active CI skip.
- M1 rustc / SG-class emit work.

---

## §8 Manager approval checklist (proud-pike-680)

- [ ] §1.1 `Upsert<T>` + phase types approved as sole generic canon
- [ ] P1.4-TYPE vs P1.4-PARSER split accepted
- [ ] `content_upsert` stub disposition approved
- [ ] Falsification probes accepted
- [ ] Phase 1.5 worker dispatch authorized (after 1.4 merge + probe 4)

---

## Related artifacts

- `docs/planning/v4-ci-overhaul-2026-05-30.md` §5–§6
- `docs/planning/v4-ci-schema-worksheet-2026-05-30.md` §1.1–1.2
- `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`
- `dsl/std/patterns.dag` L15–38, L127–166
- `dsl/std/effects.dag` — `UpsertEffect`
