# R3 T-Lens-Application-Surface — Verification cross-program partner readiness audit

**Status:** PROPOSAL — research-only. Foundational dispatch artifact for **T-Lens-Application-Surface** where **Verification Manager** is **cross-program partner only** (with Substrate Manager); **not** lane lead.

**Lane authority:** [`docs/r3-structure.md`](../r3-structure.md) lane table row **T-Lens-Application-Surface** (~**L148**) + §**Dependency on R2** cascade rule (~**L399**); design lock [`docs/design-lens-application-surface.md`](../design-lens-application-surface.md).

**Cross-refs:** [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md), [`docs/design-lens-framework.md`](../design-lens-framework.md), [`docs/lens-library-design.md`](../lens-library-design.md) (glob `LensApplication` lineage), [`r3-v-tests-as-data-completeness-readiness-audit.md`](r3-v-tests-as-data-completeness-readiness-audit.md) §5 (cementing Gate D), [`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md).

---

## 1. Canonical lane scope (quoted from authority)

From [`docs/r3-structure.md`](../r3-structure.md) **lane structure table** (~L148), **Covers** column (abridged for readability; gates listed verbatim):

> First-class authoring surface for applying lenses to arbitrary `.dag` sections (function / module / expression / declaration scope). Per user reframe 2026-05-02: lens application is a `.dag` declaration with configurable behavior — `apply_lens(lens, section, config)`. **Subsumes** prior T-Complexity-Contract-Compile-Error + T-User-Authored-Cost-Basis-Discipline as configurations of one mechanism. **Substrate carriers** (per design doc §2): **two separate top-level carriers** — `EnforcedApplication<Output, Budget>` and `IntrospectApplication<Output>`. … Plus `SectionRef` … + `LensEnforcement<Output, Budget>` … + `DiagnosticSeverity`. … **Gates:** `lens_application_carrier_landed`, `section_ref_substrate_landed`, `lens_enforcement_carrier_landed`, `enforce_violation_routing_landed`, `complexity_violation_compile_error_demonstrated`, `crdt_cost_basis_demonstrated`, `memory_peak_cost_basis_demonstrated`, `opt_in_iteration_parallelism_via_lens_application_demonstrated`. … **Design doc §8** resolves … open questions; … **standard cascade-gate (T-Lens-Behavioral-Parity COMPLETE) + R2-Evaluator landed.**

Full row text and R2 dependency column remain authoritative in-file.

---

## 2. Verification vs Substrate — partnership boundary

| Aspect | **Substrate Manager (lead territory)** | **Verification Manager (partner)** |
|--------|--------------------------------------|-------------------------------------|
| Lane table (~L148) | **Substrate Manager + Verification Manager (cross-program)** — shared ownership line | Same row — **no** Verification-only lane lead |
| Manager structure Item **2** (~L189) | — | **cross-program portion of T-Lens-Application-Surface** alongside **5 owned lanes** + behavioral-parity partner |
| Carrier introduction | **INVARIANTS §P1** substrate facts (`EnforcedApplication`, `IntrospectApplication`, `SectionRef`, enforcement routing) | Assert structural acceptance: **cementing**, **TestClaim** / runner wiring, register alignment, witness shapes — not authoring net-new carriers |
| Demonstration gates | Often lands emitting + semantic plumbing | **TestClaim**-level demonstrations and harness discipline (`ClaimResult` by shape, [`TESTING.md`](../../TESTING.md)) |

**Non-goals for Verification:** claiming sole lane leadership; inventing `TestPredicate` variants for this lane (§P1 only if ever needed).

---

## 3. Cascade dependency — upstream **T-Lens-Behavioral-Parity**

**Rule ([`docs/r3-structure.md`](../r3-structure.md) ~L399):** **T-Lens-Application-Surface** has an internal cascade: **T-Lens-Behavioral-Parity must reach BEHAVIORALLY COMPLETE before application-surface *dispatch***; **pre-cascade design-doc work** remains permitted.

**When “COMPLETE” fires (audit-level):** Per [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md), promoted rows must satisfy behavioral criteria (cementing where v2 counterpart exists — see register §**Rules** ~L99–103). Lane-level rollup includes gate **`lens_capability_register_zero_proxy_zero_stub`** ([`docs/r3-structure.md`](../r3-structure.md) ~L146).

**HEAD snapshot (register table ~L40–44):** **complexity** / **cost** remain **BEHAVIORALLY PROXY**; **parallelism** **STUB**; **effect_enumeration** **PARTIAL**. **idempotency** is **COMPLETE** (positive slice; does not clear the whole lane). **Verdict:** upstream lane **Open** / early **Partial** — **not** ready for application-surface **worker dispatch** under L399; design authoring may proceed.

---

## 4. Closure gates — HEAD disposition

No `lens_application.dag` (or `SectionRef` / `EnforcedApplication` names) under `src/v3/std/` at HEAD (`rg` over `src/v3/std` → **no matches**). Substrate carriers exist **only** as design sketches in [`design-lens-application-surface.md`](../design-lens-application-surface.md) (e.g. proposed `src/v3/std/lens_application.dag` ~L60).

| Gate | HEAD | Evidence / blocking |
|------|------|---------------------|
| `lens_application_carrier_landed` | **Open** | Carriers not in std tree; design doc only |
| `section_ref_substrate_landed` | **Open** | Same |
| `lens_enforcement_carrier_landed` | **Open** | Same |
| `enforce_violation_routing_landed` | **Open** | Same |
| `complexity_violation_compile_error_demonstrated` | **Open** | Depends on carriers + **complexity** lens completeness trajectory |
| `crdt_cost_basis_demonstrated` | **Open** | Same + **cost** lens |
| `memory_peak_cost_basis_demonstrated` | **Open** | Same |
| `opt_in_iteration_parallelism_via_lens_application_demonstrated` | **Open** | Same + **parallelism** lens unwiring **STUB** ([`v3-lens-capability-register.md`](../v3-lens-capability-register.md) ~L44) |

**Harness defaults:** **`OnceLock` + `cached_compile`** where applicable; **DB‑3 / DB‑20** split per [`design-dimension-abstraction.md`](../design-dimension-abstraction.md); **§P1** for any net-new substrate ([`INVARIANTS.md`](../../INVARIANTS.md#p1-modeling-faithfulness)).

---

## 5. Cross-claim coordination

- **T-Verification-L4-L7-Direct:** L7 algebraic-law witnesses and **cementing** receipts overlap the **same v2-oracle / capability-register** discipline that must stabilize **before** lenses are trustworthy inputs to `apply_lens` enforcement ([`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md)).
- **T-Free-Consequences-Demonstration:** Lens-algebra **TestClaims** and iteration-independence framing interact with **opt-in parallelism** demonstration gate — coordinate witness categories, not parallel ad-hoc `DimensionReport` shapes.
- **T-Tests-As-Data-Completeness:** [`r3-v-tests-as-data-completeness-readiness-audit.md`](r3-v-tests-as-data-completeness-readiness-audit.md) **Gate D** (cementing discipline) is shared substrate with **behavioral parity**; application-surface demos will eventually need **facet-3** / **TestClaim** execution paths.
- **T-Lens-Behavioral-Parity:** **Blocking partner** for dispatch timing (§3); Verification shares cementing + register alignment work.

**Escalation:** shared **§P1** or register/cementing conflicts → Director / inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).

---

## 6. Slice progression (dispatch-ready staging)

| Slice | Fire criteria (proposal) |
|-------|-------------------------|
| **1 — Authority frozen** | [`design-lens-application-surface.md`](../design-lens-application-surface.md) + [`r3-structure.md`](../r3-structure.md) row reconciled; no drift vs §8 resolutions |
| **2 — Upstream cascade** | **T-Lens-Behavioral-Parity** register rows + **`lens_capability_register_zero_proxy_zero_stub`** trajectory green enough for Director to lift **L399 dispatch** hold |
| **3 — Carriers landed** | Gates `lens_application_*` through `enforce_violation_routing_landed` wired in `src/v3/std/` + regen discipline |
| **4 — Verification harness** | **TestClaim** / runner can observe violations per **C-8** severity; cementing modules align with [`cementing_lens_registry_dispatch_test.rs`](../../src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs) pattern where applicable |
| **5 — Demonstrations** | Four **demonstrated** gates close with worked examples per design doc §4 |

---

## 7. Live-path verification receipt

```bash
git fetch origin
for p in \
  INVARIANTS.md \
  TESTING.md \
  docs/r3-structure.md \
  docs/design-lens-application-surface.md \
  docs/v3-lens-capability-register.md \
  docs/design-lens-framework.md \
  docs/lens-library-design.md \
  docs/design-dimension-abstraction.md \
  docs/briefs/r3-v-tests-as-data-completeness-readiness-audit.md \
  docs/briefs/r3-v-witness-shape-pattern-survey.md \
  src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs
do git cat-file -e "origin/main:$p" || exit 1; done
```

After merge, add **`docs/briefs/r3-v-lens-application-surface-readiness-audit.md`** to the loop.

---

## 8. Re-engagement

1. When **`design-lens-application-surface.md`** or the **lane row** changes, refresh §1–§4.
2. When **behavioral-parity** register rows flip, refresh §3–§4.

**Reply path:** Verification Manager inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).
