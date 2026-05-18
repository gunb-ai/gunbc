# Lane A PREFIX — Real lens enforcement in CI (T-23 slice A + T-12 cost + driver gate)

> Read **`src/v4/CULTURE.md`** first, then this brief. Shape follows **`src/v4/BRIEF_TEMPLATE.md`** (structural commitment; sections below map to that template).

## Dispatch posture (**methodology shift — operator 2026-05-18**)

| Field | Value |
|--------|--------|
| **Brief ID** | `PREFIX-LENS-CI-1` |
| **Owner / manager lane** | `witty-cat-59` (successor root); Lane A execution historically `fierce-cat-31` |
| **STATUS** | **`DISPATCH_HOLD`** — PREFIX **implementation** dispatches **only** after the operator signs the **Acceptance PR for that batch** (see **§Acceptance-PR batches** — ~**5–6** coherent PRs, not one-per-lens micro-PRs). **Authoring prep** (witness bodies + issue-class text) may proceed **in parallel** per operator acceleration (**msg_58b537be-3d58-490f-aaac-a4685cbd3bef**); **do not** spawn the parallel authoring **fan-out** until operator **go**. **Fork B** / **Fork C** resolved in body below. |
| **Parallel spine** | **T-8** (**`eager-ant-519`**, PR **#3311**) — **unchanged**; manager keeps **P3 walk fail-close** review discipline (**witty-cat-59** directive). |

---

## Operator rulings consumed (**do not re-litigate here**)

### Fork A — **RESOLVED (v4 authoritative)**

- **v4 is the only design authority in CI.** v3 lenses are **compositional-only**; modeling-discipline / dissolution lenses (**L1.1–L1.12**) are **v4 CP-1–gated** by construction — **no v3 “design bridge”** exists for that class (operator-verified: `design-dissolution-lens.md` L1.x has **no** v3 form, gated on v4 CP-1).
- **INTERIM — one delete-dated supplementary step:** **exactly ONE** CI step invokes the **existing** v3 fold over the **whole** `.dag` corpus using the **existing whole-source-tree file-glob carrier** (same family as `LensApplication` / whole-tree gates in `docs/design-lens-application-surface.md` §1 opening + §3 file-glob discipline). **Zero new v3 design** — invocation only.
- **Labeling (binding):** that step is **`SUPPLEMENTARY` / `NON-AUTHORITATIVE` / `DELETE-DATED`** (deletion date set when v4 driver reaches parity), filed **alongside** **L-7 / L-8** (which remain supplementary grep ratchets until **T-24** emits CI from data). It is **not** a second lens authority.

**Removed:** the prior **“v3-direct CI override caveat”** — operator **rejected** that fork.

### Fork C — **RESOLVED MAXIMAL (whole corpus, v1)**

- **Slice C** runs the **v4-home driver** over **every** `.dag` file in the repo (whole-source-tree glob carrier — `docs/design-lens-application-surface.md` §1 / §3 whole-tree pattern), **fail-closed**, **v1**.
- **Scaling / feasibility:** whole-corpus enforcement is **O(1) lens-applications per program**, not O(files²) re-derivation — see **`docs/design-lens-application-surface.md` §5** (~L355–L362): the lens-application integration step is structurally the same fold class as existing `Lens<C>` application; the incremental cost is **O(applications)** (typically tens–hundreds per project), **not** millions. **Cite that section** in any PR that touches Slice C so scaling is not re-litigated.
- **Fixtures are NOT the blast-radius limiter:** **closed fixtures** exist **only** inside the **PREFIX Acceptance PR** as **immutable witness / acceptance contracts** (see `r4-lane-a-lens-prefix-acceptance.md`). They **never** cap CI corpus coverage.

### Fork B — **unchanged queueing (fan-out), not a PREFIX dispatch gate**

Dissolution lens wave remains **`jolly-ibex`** / **#3313** / **#3240** — tracked in fan-out section; **does not** extend `DISPATCH_HOLD` beyond the **batch** discipline in **§Acceptance-PR batches** once the operator signs the relevant Acceptance PRs.

---

## Immutable contract pins (**CONSUME — do not re-spec**)

| Authority | Role |
|-----------|------|
| `src/v4/lens/application.dag` | **Immutable header** — `SectionRef`, `EnforcedApplication` / `IntrospectApplication`, `apply_lens`, **D1** `subterm_at` / `apply_diff`, AGENT-1 composition notes, advisory→fail-closed bridge discipline. Status: **scaffold — fill per TASKS.md T-23**. |
| `src/v4/TASKS.md` §**T-23** (~L674) | Modeling obligations for the application surface (**three-parameter** `EnforcedApplication` / `EnforceableLens` — aligned with **`DECISIONS.md` T-23-PIN** + **`docs/design-lens-application-surface.md` §2**). |
| `src/v4/DECISIONS.md` | **C7 / report / synthesis** rows citing **T-23** — ledger receipts; Practice **4 / 9**. |
| `src/v4/lens/cost.dag` | **T-12** home — **scaffold — fill per T-12**; lattice fill may remain gated honestly. |
| `docs/design-lens-application-surface.md` | **Whole-tree glob + O(1)-applications** feasibility for Slice C (`§5` ~L355–L362). **§5.1** default `IntrospectApplication<ComplexitySummary>` synthesis (operator dev-speed lever). |
| `docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` | **T-23 Interface-Freeze broadcast** — frozen carrier digest, **CLI v0** template, **`LENS_ID`** registry; amend only operator-signed. |
| `docs/design-lens-framework.md` §2 | **`Witness<C> = Inhabits \| Violates`** and `DimensionReport` / `DimensionOk` / `DimensionFail` — **acceptance PR maps runnable AC here** (DB-15 enumerated `TestClaim` discipline). |
| `.github/workflows/ci.yml` **L-7 / L-8** | **Supplementary** static ratchets — **remain** until **T-24**. |

---

## Interface-Freeze keystone (**minimal delta — prefix of the prefix**)

**Current state:** `src/v4/lens/application.dag` carries the **ratified design contract in the file header** (three-parameter `EnforcedApplication` reconciled to **`docs/design-lens-application-surface.md` §2**, 2026-05-18) plus a **one-line** ledger cite **`DECISIONS.md` (T-23-PIN)**; the **pin body** is **`docs/briefs/r4-lane-a-lens-interface-freeze-pin.md`** (carrier digest + **CLI v0** + **`LENS_ID`** registry). The **module body is still empty** beyond `module v4.lens.application` — **TASKS.md T-23** mechanical port (parseable carriers + `Lens<C>` / id nominals — see pin §4) is pending.

**The keystone delta** (everything else parallelizes around this **small** freeze):

1. ~~**Header ↔ design authority ratification:**~~ **Done in header + pin doc** — canonical **`EnforcedApplication<Output, Budget, Projected>`**, **`IntrospectApplication<Output>`**, **`LensEnforcement` / `EnforceableLens`** bundle discipline.
2. **Minimal parseable `.dag` declarations** in `application.dag` (names + parameters stable; bodies may stay `...` / stub where honest): imports from declared std peers; **`SectionRef`** disjoint sum; the **two carriers**; enough **`apply_lens` / config** surface to discriminate **Enforce** vs **Introspect** with the span/severity fields the design doc already specifies — **not** full fold implementation, not AGENT-1 runtime, not per-lens completeness. **Blocked on** v4 `std` **`Lens<Output>`** + id nominals — see **pin §4** (does **not** block witness authoring).
3. ~~**Frozen driver/registry I/O pin**~~ **Parked** in **`r4-lane-a-lens-interface-freeze-pin.md` §3** — **one-line runnable-AC slot-in** per Acceptance batch; swap `TBD` for live argv when the binary lands.

**Explicitly not the keystone:** CP-1 front-end, full cost lattice, full complexity behavioral completeness, dissolution L1.x bodies — those ship in **parallel** or **later batches** once the interface string is frozen.

---

## Acceptance-PR batches (**coherent — operator value order ~5–6 signatures**)

| Order | Batch | Acceptance artifact scope (high level) |
|------:|--------|------------------------------------------|
| **1** | **Interface-Freeze** | Pin doc **`r4-lane-a-lens-interface-freeze-pin.md`** + **`application.dag`** header ratified + CLI/registry §3; parseable body remains §4 backlog. |
| **2** | **PREFIX driver / registry + corpus gate** | Slices A–C runnable AC, whole-corpus contract, delete-dated v3 step labels — evolves `r4-lane-a-lens-prefix-acceptance.md` / linked PR. |
| **3** | **Cost + complexity (shared `SymbolicCost` algebra)** | **Wave-1 #1 = complexity** (operator elevation); **cost** remains PREFIX **reference** / algebra root complexity composes on. One shared Acceptance PR for both. |
| **4** | **Wave-1 remainder** | **parallelism**, **effect_enumeration**, **idempotency** + pure structural readers **provenance**, **unused_parameters**, **structural_resolution** — **single** batched Acceptance PR (not six micro-PRs). |
| **5–6** | **Wave-2 dissolution L1.1–L1.12** | Coherent **sub-batches** (operator splits); **`design-dissolution-lens.md` §8** slipped-by ledger + v4 CP-1 gating unchanged. |

**Implementation dispatch:** **per batch** — workers touch impl only after **that** batch’s Acceptance PR is **signed**; witnesses **immutable** except red→green under operator amendment. **T-8 / #3311** — **P3 fail-close unchanged** (`eager-ant-519`).

---

## Witness parallelism vs Interface-Freeze (**dependency claim — lane view**)

**True without driver / CP-1 / registry code:** Acceptance **substance** that is only **(a)** issue-class prose tied to **existing** v3 lens `.dag` bodies + design docs, **(b)** failing `.dag` snippet + clean counterexample, **(c)** `design-dissolution-lens.md` **§8** slipped-by ledger rows for dissolution classes — is **authorable in parallel now**. Those witnesses **do not** need the v4 `application.dag` body to exist; they anchor on **today’s** behavioral authorities.

**Needs Interface-Freeze (or an explicit interim pin) before “final”:** **(i)** any row that asserts **v4 parse-tree / field-name** shape for `EnforcedApplication` / `SectionRef` in **checked-in v4 source**; **(ii)** harness code that **type-checks** witness snippets **as v4** `lens.application` types; **(iii)** the **canonical `driver …` one-liner** in runnable AC tables (until the CLI pin in §Interface-Freeze lands, use an explicit **`TBD — slot after keystone`** placeholder **or** interim **`v2-compiler compile` / v3 fold** invocation per Fork A — do **not** pretend the CLI is already frozen).

**Net:** parallel authoring is safe for **witness + issue-class bulk**; only the **runnable invocation column** and **v4 structural AST assertions** serialize on the **keystone** — typically **one replacable line** per batch once (3) is frozen.

---

## WHY THIS MATTERS

CI today has **no** real “fold registered lenses over programs, fail-closed on `Witness`/`DimensionFail`” gate. This PREFIX lands **v4-authoritative** driver/registry + **T-12 cost** as first real registration, with **one whole-corpus** fail-closed step, while a **delete-dated** v3 invocation provides **interim** behavioral coverage without inventing parallel design.

**Operator synthesis lever (complexity):** per **`docs/design-lens-application-surface.md` §5.1**, the lens fold **synthesizes** default **`IntrospectApplication<ComplexitySummary>`** for every function — **always-on** complexity introspection **with zero per-function authoring**, feeding downstream lens composition + debug surfaces. **Wave-1 #1** is **complexity**; **cost** stays the **algebraic reference** dimension complexity composes against — both ride **Acceptance batch 3** together.

---

## SCOPE (immutable — three slices)

1. **Slice A — T-23 registry + driver skeleton (v4-home):** deterministic **registry** + **one** Rust driver entrypoint; runs selected lens over **any** in-corpus `.dag` input path supplied by CI glob expansion; **fail-closed** on internal errors. **P5 / SG-0:** any **new or expanded** hand-Rust under `src/v3/` (including new `src/v3/compiler/tests/**` integration files) follows **§P5 — hand-Rust surface** below — no silent census debt.
2. **Slice B — T-12 cost lens (first REAL registration):** enough of **`lens/cost.dag`** to register and prove Slice A — **honest-scaffold** where lattice fill remains gated; **no** hand-rolled walkers; **P3**-gated walks stay **Rejected / not-realized** pending substrate. **Acceptance:** batched with **complexity** (batch **3** — operator: complexity **Wave-1 #1**, cost = compositional root).
3. **Slice C — CI gate (whole corpus):** **one** workflow step: invoke driver over **all** `.dag` files (whole-tree glob carrier); **merge-gate** outcomes (**exit 0 vs non-zero**) are **only** those enumerated as **corpus / aggregate** checks in **`r4-lane-a-lens-prefix-acceptance.md`** (see **§TEST SURFACE** — **never** use this step to “prove” a **red witness**; red witnesses are **harness-asserted** `DimensionFail` / `Violates` with **passing tests**). **Delete-dated** v3 fold step runs **alongside** as **supplementary non-authoritative** coverage until v4 parity.

**Out of scope:** full **T-24** `ci.yml` emitter (align only); **Wave 2** dissolution lenses until PREFIX+CP-1+#3240/#3244 gates clear.

---

## P5 / SG-0 — hand-Rust surface (**binding — INVARIANTS P5 Mechanism (b)**)

PREFIX implementation that touches **`src/v3/compiler/**`** hand-authored Rust (non-generated) MUST satisfy **one** of:

1. **Receipted expansion (default):** the PR adds **exactly one** checkable **P5 Mechanism (b)** receipt per **INVARIANTS.md** §P5 / **SG-0** table row, **in the same PR** as any new `EXPECTED_HAND_AUTHORED_NON_TEST` / `EXPECTED_HAND_AUTHORED_TEST` / fragment census line in `src/v3/compiler/tests/integration/sg0_census_test.rs`, and names the adjacent **`ROADMAP.md` § Nine lanes** row explicitly — **`T-PB-A`** / `pb_hand_rust_at_shim_floor` (non-test surface) or **`T-PB-B`** / `pb_rust_tests_outside_residual_zero` (test surface), matching the census partition the change lands in.
2. **No new v3 hand-Rust:** the observable driver + gates ship **only** through **`.dag` authorities**, **`v2-compiler`**, and/or **generated** surfaces already covered by existing census — **STOP** and escalate if the only apparent path is ad-hoc host Rust outside those receipts.

**Anti-pattern (forbidden):** a “diagnostic fixture” implemented as **“the CI workflow step must exit non-zero to prove we emit `Diagnostic`”** — that is **not** a dissolution receipt; it is an **unmergeable CI posture** that contradicts mergeable acceptance (see **TEST SURFACE**).

---

## Fan-out (**do not dispatch from this brief**)

### Wave 1 (~7) — v4-native compositional lenses

Port from **v3 behaviorally-complete** instances. **Operator batching:** **complexity + cost** = **Acceptance batch 3** (complexity **first priority**); **parallelism + effect_enumeration + idempotency + provenance + unused_parameters + structural_resolution** = **Acceptance batch 4** (one coherent PR). **Implementation** still **one lens / one PR** where useful, but **Acceptance signatures** follow the **two** Wave-1 batches above — not seven micro-Acceptance PRs.

### Wave 2 (~12) — dissolution lenses **L1.1–L1.12**

**Parallel**, gated on **PREFIX + v4 CP-1 front-end + #3240/#3244**; **no root lens**; cost/complexity remain **independent**. **`jolly-ibex`** lane owns dissolution fan-out per operator.

---

## SUBSTRATE YOU MAY USE / NOT USE

Unchanged intent from prior brief revision: **whitelist** v4 lens + std imports declared by those files; **REFERENCE** v3 tests/oracle for **behavior only**; **STOP** on new std / unauthorized new files / hand-Rust shims that re-derive lens logic.

---

## TEST SURFACE / RUNNABLE ACCEPTANCE (**split: corpus CI vs typed negative witnesses**)

1. **`v2-compiler compile --source-root src/v4`** — **0 diagnostics** (unchanged spine bar).
2. **Whole-corpus driver (Slice C) — mergeable CI step:** the workflow job that runs the driver over **all** `.dag` paths from the glob carrier **exits 0** when acceptance criteria are met (aggregate policy per **`r4-lane-a-lens-prefix-acceptance.md`** — e.g. zero **unexpected** `Violates` / `DimensionFail` escapes against the enumerated contract). This step **never** doubles as a “prove diagnostics by failing the job” harness.
3. **Typed negative / diagnostic witnesses — harness only:** red snippets and **`DimensionFail` / `Violates` / typed `Diagnostic`** expectations are asserted under **`cargo test`** (or the repo’s equivalent **DB-15** `TestClaim` runner path) so that **expected failure is a passing test** (asserted structured outcome). **CI stays green** when those tests pass. **Do not** specify witness proof as a non-zero exit from the whole-corpus job — that conflates “driver correctly reports violation” with “branch is unmergeable,” which contradicts “all acceptance criteria pass in CI.”
4. **PREFIX Acceptance PR** (`r4-lane-a-lens-prefix-acceptance.md`) — holds **immutable** red/green **witness snippets** + **enumerated** `TestClaim` / driver expectation table (**DB-15**), including explicit rows mapping runnable checks to **`Witness<C>`** / **`DimensionOk` / `DimensionFail`** (**`docs/design-lens-framework.md` §2**). **Implementation workers may not edit** witness blocks except **red→green** with **operator-signed** amendment to Acceptance PR (**anti-fabrication**).

---

## DEFINITION OF DONE

- Acceptance PR **signed** by operator (**DISPATCH_HOLD** lifts).
- Slices A–C merged per scope; **delete-dated** v3 step labeled and scheduled for removal at parity.
- **DECISIONS.md** receipts for any 🟡 interim deferrals.

---

## STOP TRIGGERS

Per **`BRIEF_TEMPLATE.md`** + **no editing Acceptance witnesses** without operator-signed acceptance amendment.

---

## REPORT-BACK

**Brief updated** (this file — **§Interface-Freeze keystone**, **§Acceptance-PR batches**, **§Witness parallelism vs Interface-Freeze**) + **Acceptance artifact** (`r4-lane-a-lens-prefix-acceptance.md`) — report SHA + PR # to **`witty-cat-59`** after push.
