# Substrate PR Review Gate (v4)

> **Status:** ACTIVE — Modeling DFS Manager (proud-pike-680); work item `adhoc-b86786f5-9fe` track **(C)**.  
> **Date:** 2026-05-30  
> **Trigger:** Operator postmortem on PR #3962 (`TargetTypeExpression` six-arm tag-shadow); complements §10.0 DFS worksheets (SG-class) and dissolution-lens enforcement tracks **(A)** / **(B)**.

---

## When this gate applies

Every PR that **lands or materially changes substrate shape** in:

- `dsl/std/`, `src/v4/std/`, `src/v4/extdeps/`, `src/v4/lens/` (new carriers, coproducts, registries)
- Per-language realization rows in `extdeps/languages/`
- Workflow / CI schema carriers that introduce new generic specializations

**Not required for:** emit-template-only fixes, test-only PRs, docs-only PRs (unless the doc invents parallel ledger facts — Practice 9).

---

## Mechanical rule

> **Substrate-shape PRs must include the variant-shape histogram table below (§3) in the PR body or a linked planning doc. Modeling DFS (or delegated reviewer) checks M6 / Practice 11 before merge.**

SG-class work still requires a separate **§10.0 DFS worksheet** (`docs/planning/v4-correctness-ladder-2026-05-30.md` §10.0). This gate is the **substrate PR** complement, not a substitute.

---

## §1 Review checklist (source rules)

| # | Rule | Question on this PR |
|---|------|-------------------|
| **M9** | DFS the ontology | Does every new type trace to a parent concept in `dsl/std/` (or named concept home)? Any ad hoc vocabulary that duplicates an existing connective? |
| **P8** | Fact-bundle modeling (Practice 8) | Are spec primitives modeled as bundles, not hollow aliases? Coincidence cite when reuse is non-obvious? |
| **P4** | Coproduct dissolution (Practice 4) | Are closed vocabularies coproducts, not stringly dispatch? Any arm that should dissolve to a smaller carrier? |
| **M6** | One result pattern | Do multiple variants share **identical** field shapes differing only by tag? → collapse to product + kind enum (see #3962). |
| **P11** | Parameterize, don't duplicate (Practice 11) | Any parallel type that copies another module's shape? Concept-home respected? |
| **P10** | Don't hand-roll derived ops (Practice 10) | Are caches/digests/projections derived from declared subgraphs, not parallel `cache_key` payloads? |
| **P2** | Single authority (INVARIANTS) | Does each fact live in exactly one authoritative place? Compile-root mirrors must cite dsl canonical home + 🟡 dissolve-on. |
| **P3** | Fail-closed | Do partial inputs produce explicit `Rejected` / diagnostic carriers, not silent defaults? |

---

## §2 Blocking vs advisory

| Class | Example | Disposition |
|-------|---------|-------------|
| **Blocking** | M6 identical-payload coproduct arms (#3962) | Must collapse or dissolve before downstream carriers copy the pattern (SG-1, SG-5). |
| **Blocking** | New parallel authority for a fact already in std/ | Rehome or consume existing carrier. |
| **Advisory** | Missing 🟡 gate on staged mirror (v4 compile-root vs dsl/) | Add gate + dissolution target before merge if mirror is intentional. |
| **Advisory** | Histogram shows near-isomorphic arms | Open follow-on or cite why arms genuinely differ. |

---

## §3 Mandatory variant-shape histogram (PR body table)

For **every new or changed coproduct** `type T = A { … } | B { … } | …` in the PR, fill:

```text
| Variant | Field arity | Field types (canonical) | Payload fingerprint |
|---------|-------------|-------------------------|-------------------|
| ArmName | n           | (TypeId, …)             | e.g. { node: Node } |
```

**Pass criteria:**

- No two rows share the same **Payload fingerprint** unless the PR explicitly dissolves to `{ kind: <enum>, … }` (M6).
- If fingerprints match, PR must not add separate `Emitted*` wrapper types per arm.

**Worked example (blocking class — PR #3962):**

| Variant | Field arity | Field types | Payload fingerprint |
|---------|-------------|-------------|---------------------|
| TargetTypeExprEmittedAtom | 1 | Node | `{ node: Node }` |
| TargetTypeExprEmittedInstantiation | 1 | Node | `{ node: Node }` |
| … (six arms) | 1 | Node | `{ node: Node }` |

**Approved collapse:**

```dag
type TargetTypeExpression {
  kind: TargetTypeExprKind
  node: Node
}
```

---

## §4 Enforcement roadmap (not blocking substrate PRs today)

| Track | Owner work item | Catches |
|-------|-----------------|--------|
| **(A)** `v4.lens.structural_similarity` | `adhoc-1941f9fc-580` | Automated histogram from `TypeShape.variant_set` |
| **(B)** L1.4 IdenticalVariantPayload sub-signature | `adhoc-caef5039-f11` | Lens finding on identical payloads (#3962 class) |
| **(C)** This doc | `adhoc-b86786f5-9fe` | Human review discipline until (A)+(B) land |

Operator priority 2026-05-30: promote **(A)** from future to **needed for review safety**.

---

## §5 Related artifacts

- `MODELING.md` — M6, M9
- `docs/modeling-discipline.md` — Practices 4, 8, 10, 11
- `docs/design-dissolution-lens.md` — §5.1, §10.2 (`structural_similarity`), L1.4.b
- `docs/planning/v4-correctness-ladder-2026-05-30.md` — §10.0 DFS worksheets (SG-class)
- `INVARIANTS.md` — P2, P3

---

## §6 Manager approval

- [x] Checklist + histogram template — Modeling DFS §8 2026-05-30 (proud-pike-680)
- [ ] (A) structural_similarity producer on main
- [ ] (B) IdenticalVariantPayload sub-signature active in lens CI
