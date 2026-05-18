# Lane A PREFIX — Real lens enforcement in CI (T-23 slice A + T-12 cost + driver gate)

> Read **`src/v4/CULTURE.md`** first, then this brief. Shape follows **`src/v4/BRIEF_TEMPLATE.md`** (structural commitment; sections below map to that template).

## Dispatch posture

| Field | Value |
|--------|--------|
| **Brief ID** | `PREFIX-LENS-CI-1` |
| **Owner / manager lane** | `witty-cat-59` (successor root); Lane A execution historically `fierce-cat-31` |
| **STATUS** | **`DISPATCH_HOLD`** — brief is **authored and parked dispatch-ready**. **Do not** `dashboard-ops work-items create` for this PREFIX until **`witty-cat-59`** relays **operator ruling** on **Fork B** (L1.7–L1.12 dissolution lenses **#3313** sibling fan-out under `jolly-ibex`, gated **#3240**) and **Fork C** (v1 blast radius = fixed `.dag` fixture corpus fail-closed now vs affected-set Lane-B fast-follow). |
| **Parallel spine** | **T-8** (**`eager-ant-519`**, PR **#3311**) is a **separate work node** — manager continues merge/review discipline there (P3 walk fail-close). |

---

## Operator fork assumptions (binding for brief body)

### Fork A — **converged recommended-default** (carry to operator)

- **v4 substrate home:** registry + CI driver align with **`src/v4/lens/application.dag` (T-23)** and long-term **`src/v4/workflow/ci.dag` (T-24)** (“CI as data”).
- **Behavioral oracle:** first driver behavior is **seeded from v3** — `apply_lens_declaration` + **`src/v3/compiler/tests/integration*.rs`** patterns that already fold real lens machinery (study; do not cargo-cult).
- **Not end state:** **v3-only CI forever** is **explicitly out of scope** — it repeats the hand-authored CI gap **T-24** exists to dissolve.

**Caveat (one-line override hook):** *If the operator overrides Fork A to “v3-direct CI home first,” flip this brief’s implementation spine to wire the driver under the v3 harness first, then port the sealed interface to v4 — **small edit**: replace “driver crate/home” pointers and DECISIONS receipt rows; **do not** rewrite the contract pins below.*

### Fork B / Fork C

**Out of scope for PREFIX execution until ruled** — listed only so workers do not absorb ambiguity into scope creep.

---

## Immutable contract pins (**CONSUME — do not re-spec**)

Workers **must** treat these as read-only authorities; the PREFIX **fills** them, it does not fork parallel “lens runner” nouns.

| Authority | Role |
|-----------|------|
| `src/v4/lens/application.dag` | **Immutable header** — `SectionRef`, `EnforcedApplication` / `IntrospectApplication`, `apply_lens`, **D1** `subterm_at` / `apply_diff`, AGENT-1 composition notes, advisory→fail-closed bridge discipline. Status: **scaffold — fill per TASKS.md T-23**. |
| `src/v4/TASKS.md` §**T-23** (~L886+) | Modeling obligations for the application surface (carriers, `SectionRef`, default Introspect-only synthesis policy, `Enforce` bridge). |
| `src/v4/DECISIONS.md` | **C7 / report / synthesis** rows that **cite T-23** (e.g. **C7-REPORT** and related) — ledger text is the receipt plane; Practice **4 / 9** discipline applies. |
| `src/v4/lens/cost.dag` | **T-12** home — Status: **scaffold — fill per T-12**; lattice / realization fill may remain **honestly gated** where the header already says so. |
| `.github/workflows/ci.yml` **L-7 / L-8** | **SUPPLEMENTARY** grep ratchets on **v3** lens surfaces — **remain** until **T-24** emits CI from data; PREFIX adds a **real** driver gate **alongside**, not replacing, unless operator later collapses them. |

---

## WHY THIS MATTERS

CI currently enforces lenses mostly via **static proxies** (L-7/L-8 grep), not by **running** registered lens machinery over a `.dag` program **fail-closed** on **`Witness` / enforcement** outcomes. That gap lets lens drift hide until late. This PREFIX makes the **first real** “enumerate → run → enforce” loop **true**, without inventing a parallel design: it **instantiates** the already-ratified **T-23** surface and proves it with **T-12 cost** as the reference lens.

---

## SCOPE (immutable — three slices, one program)

**In scope (PREFIX):**

1. **Slice A — T-23 registry + driver skeleton (v4-home):** a **deterministic registry** of lens ids → entry metadata + a **single** Rust driver entrypoint that can run **one** selected lens over a **pinned** `.dag` fixture and return a **typed** pass/fail outcome (Diagnostics / Witness / Report per the lens), **fail-closed** on any internal error.
2. **Slice B — T-12 cost lens (first REAL registration):** implement enough of **`lens/cost.dag`** to register as a **real** lens instance proving Slice A — **honest-scaffold** where full **SymbolicCost** lattice realization remains gated per existing header; **no** fake hand-rolled graph walker — **P3**-gated walks stay **fail-closed** pending substrate (same dissolution discipline as Lane A T-8).
3. **Slice C — CI gate:** add **one** workflow step that invokes the **Rust driver** on a **closed fixture corpus** (versioned list), **fails the job** on any enforcement/`Witness` failure, and prints a **deterministic** summary for logs.

**Out of scope (explicit):**

- Implementing the full **T-24** emitter for `ci.yml` (derive-from-`workflow/ci.dag`) — **not** PREFIX-blocking; only **align** types/names so T-24 can absorb later.
- **Fork B / Fork C** bodies — **hold** for operator.
- Parallel fan-out of dissolution lenses (**#3313**) — **jolly-ibex** lane, gated per operator.

---

## SUBSTRATE YOU MAY USE (whitelist)

- `src/v4/lens/application.dag`, `src/v4/lens/cost.dag`, adjacent `src/v4/lens/*.dag` **already referenced** by those headers as consumers (no new lens files unless operator ratifies substrate extension).
- `src/v4/std/*.dag` already imported by the lens files above.
- `src/v3/compiler/**` **as REFERENCE only** for `apply_lens_declaration` / integration test **oracle** behavior (Fork A default).
- Existing **`v2-compiler`** / **`v3-compiler`** binaries and **`cargo`** test targets already in CI — prefer extending an **existing** harness binary over inventing a new top-level crate unless operator extends structure (**STOP** otherwise).

## SUBSTRATE YOU MAY NOT USE

- Any path outside the whitelist **without operator STOP resolution**.
- New `std/*` concepts — **STOP**.
- **Hand-Rust shims** that duplicate lens logic “for CI only” — forbidden; driver **calls** substrate, it does not re-derive lenses.

---

## DISCIPLINE (non-negotiable)

- **Pure / fail-closed / P4** per `INVARIANTS.md` + `CULTURE.md`.
- **Practice 9:** no multi-line architectural narration in `.dag` bodies — receipts in **`DECISIONS.md`**.
- **Practice 4 / 10:** dissolution-class diffs carry explicit **🔴 / 🟡 / 🟢** per **`docs/modeling-discipline.md`** + PR **#3244** bar; no hand-rolled **walkers** where a **named substrate primitive** is the plan-bound receipt (**P3** lex/parse walk is the Lane A parallel — do not “fake” lens walks either).

---

## BURN-DOWN OUTPUT BAR

Same as **`BRIEF_TEMPLATE.md`** Practices **4, 7–10** + manager pre-gate — worker does not self-`gh pr ready` on dispositions alone.

---

## TEST SURFACE / RUNNABLE ACCEPTANCE CRITERIA (binding)

All must pass locally **and** in CI for the PREFIX PR(s):

1. **`cargo run -p v2-compiler --release -- compile --source-root src/v4 --target dag --output-dir <tmp>`**  
   - **0 diagnostics** on the v4 graph after PREFIX lands (same bar as Lane A spine work).
2. **Driver self-test (new or extended integration target):**  
   - `cargo test -p v3-compiler <PREFIX_DRIVER_TEST_FILTER>` **or** the operator-approved equivalent harness — runs the **registry driver** over at least **two** pinned `.dag` fixtures:  
     - **Golden:** cost lens returns **expected** structured output (or expected **advisory** path if only Introspect is wired for slice-1 — **must be explicit in TestClaim**).  
     - **Diagnostic:** fixture provokes a **typed** failure — job must **fail** and surface the **Diagnostic** / enforcement path **deterministically**.
3. **CI workflow:** the new step **fails** when (2) would fail; **passes** when (2) passes; does not flake on unrelated matrix noise.

---

## REFERENCE (study, do not copy)

- `src/v3/compiler/tests/integration.rs` and referenced **`t_gate_58_apply_lens_self_application_test`** modules — real `apply_lens` / registry patterns.
- `src/v4/TASKS.md` **T-23**, **T-12**, **T-24** (alignment only).
- `.github/workflows/ci.yml` **L-7 / L-8** — supplementary discipline context.

---

## DEFINITION OF DONE

- **Slice A** merged: registry + driver callable from tests + CI.
- **Slice B** merged: **cost** lens registered and exercised on at least one fixture without violating **`lens/cost.dag`** honest-scaffold gates.
- **Slice C** merged: workflow step live, fail-closed semantics verified by a controlled failing fixture in CI (not only locally).
- **DECISIONS.md** updated with **🟡/🟢** receipts for any interim deferrals (lattice fill, extra lenses) — **no bare 🟡**.

---

## STOP TRIGGERS (binding)

Per **`BRIEF_TEMPLATE.md`** — especially: need for **new top-level crate**, **new std concept**, **splitting** `application.dag` without operator ratification, or temptation to **grep-only** “fake” the driver. **STOP** and escalate to **`witty-cat-59`**.

---

## YOUR DECISIONS (pre-decided vs worker)

| Decision | Disposition |
|----------|-------------|
| Fork A default (v4-home + v3 oracle) | **Pre-decided** — see caveat block for operator override. |
| Fixture corpus size / selection | **Worker** proposes **minimal closed set** (≥2 files); manager ack in PR. |
| Exact crate/test harness location | **Worker** proposes; **STOP** if it requires a forbidden new crate boundary. |

---

## REPORT-BACK

When this file lands on `main` (or the operator-requested branch), **`fierce-cat-31`** pings **`witty-cat-59`**: “**PREFIX brief authored:** `docs/briefs/r4-lane-a-lens-prefix-t23-t12-ci.md` — **DISPATCH_HOLD** until Fork B/C ruling.”
