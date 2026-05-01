# R3 Lane 2 — L5 corpus extension spec (algebraic-equivalence harness primitives)

**Status:** PROPOSAL — research-only. Builds on the **slice 1 L5 skeleton already merged to `main`** (fixture + seed paths listed under **§Live-repo anchors**; merge PR **#1408**) and on readiness audits (**#1390**, **#1393**, **#1394**), extending them into a **slice 2–5 corpus-shape** spec for the eventual Lane 2 implementation worker. **No implementation**, no substrate, no new `TestPredicate` variants, no new fixtures in this document.

**Director authority (read-only):** [`docs/r3-structure.md`](../r3-structure.md) gate **`l5_cross_target_consistency`** (L56 narrative).

**Upstream briefs (read-only):** [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md), [`r3-v-l5-corpus-scaffold-notes.md`](r3-v-l5-corpus-scaffold-notes.md), [`r3-v-l5-corpus-readiness-audit.md`](r3-v-l5-corpus-readiness-audit.md). Semantic lock: [`design-cross-target-equivalence.md`](../design-cross-target-equivalence.md).

**Test harness discipline (defaults for future worker code):** integration receipts should follow **OnceLock + `cached_compile`** amortization where appropriate; assert **`ClaimResult` variants by shape** (`Pass` / `Fail` / `NotYetImplemented(_)`) without pinning raw diagnostic strings — per [**TESTING.md**](../../TESTING.md#dont-assert-on-implementation-details) ("Don't assert on implementation details": match diagnostics structurally, not substring text).

**Live-repo anchors (`main`):** This document is **research-only** and lands as a single `.md` file, but its claims are grounded in **already-merged** artifacts — not hypothetical paths. Each path exists in **`git`** at **both** `origin/main` and **this branch's `HEAD`** (`git cat-file -e HEAD:<path>`):

| Role | Path |
|------|------|
| L5 skeleton `.dag` fixture | `src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag` |
| Sidecar seed program | `src/v3/compiler/tests/fixtures/r3_l5_corpus/add_then_branch_seed.v3` |
| L4 / L7 sibling skeleton fixtures | `src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag`, `src/v3/compiler/tests/fixtures/r3_verification_l7_algebraic_laws.dag` |
| NYI integration receipts | `src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs` |
| `TestClaim` / `TestPredicate::ForAllTargets` substrate | `src/v3/std/verification.dag` |
| Runner dispatch (`ForAllTargets` → NYI default arm today) | `src/v3/compiler/src/test_runner.rs` |
| Lane 2 standby briefs | `docs/briefs/r3-v-l5-corpus-worker.md`, `docs/briefs/r3-v-l5-corpus-scaffold-notes.md`, `docs/briefs/r3-v-l5-corpus-readiness-audit.md` |

### Live-path + substrate verification receipt

Re-run whenever **`main`** moves materially (paths deleted/renamed). This loop lists **every repository-relative path hyperlinked from this brief** (fixtures, runner, `verification.dag`, numeric grounding docs, Director / semantic-lock docs, INVARIANTS / TESTING anchors). **Extend the list when adding new links.**

```bash
git fetch origin
for p in \
  INVARIANTS.md \
  TESTING.md \
  dsl/std/integer.dag \
  docs/design-cross-target-equivalence.md \
  docs/design-numeric-construction.md \
  docs/r3-structure.md \
  docs/thesis/r2-r3-thesis-mapping.md \
  docs/briefs/r3-v-l5-corpus-readiness-audit.md \
  docs/briefs/r3-v-l5-corpus-scaffold-notes.md \
  docs/briefs/r3-v-l5-corpus-worker.md \
  src/v3/compiler/src/test_runner.rs \
  src/v3/compiler/tests/fixtures/r3_l5_corpus/add_then_branch_seed.v3 \
  src/v3/compiler/tests/fixtures/r3_verification_l4_emit_eval_match.dag \
  src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag \
  src/v3/compiler/tests/fixtures/r3_verification_l7_algebraic_laws.dag \
  src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs \
  src/v3/std/verification.dag
do git cat-file -e "origin/main:$p" || exit 1; done
```

**`ForAllTargets` spot-check:** `rg -n "ForAllTargets" src/v3/std/verification.dag` — confirms the scaffold sum arm `{ command: String, args: List<String>, expect_exit_code: Int }` exists (line numbers intentionally **not** pinned in prose).

Substrate **changes** (new carriers, new predicate variants) are **out of scope** for this PR — route via **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** when implementation dispatch proposes them ([§7](#7-substrate-introduction-flag-invariants-p1)).

---

## 1. Corpus seed-to-extension audit (coverage progression)

Slice numbering aligns with [`r3-v-l5-corpus-scaffold-notes.md`](r3-v-l5-corpus-scaffold-notes.md) §"Coverage Progression"; below adds **what to certify** beyond the minimal seed.

| Slice | Scope | Intended program classes | Gate toward strict fire |
|-------|--------|---------------------------|-------------------------|
| **1 — Seed** | **Landed** conceptually via skeleton | Single `add` + `match` + `Int` observable (`add_then_branch` family); no IO, effects, floats, host libs | Proves emit/run/observation plumbing only — **does not** close Lane 2 |
| **2 — Primitive values** | First expansion | `Bool` literals and Boolean connectives safe at emit boundary; `Int` arithmetic (+, −, ×) **only under §1.1** (shared overflow/range rule — not division-only gating); guarded `/` only if all targets share identical partiality; **simple records** (named fields, order-stable lowering); **simple variants** with payloads drawn only from prior primitives — **no** `String`/unicode/stdlib, no filesystem paths | Surfaces **match lowering**, **call ABI**, **literal range** regressions early |
| **3 — Collections** | Lists / maps | Homogeneous `List<T>` / map-like carriers **only after** Rust/Python/Go agree on a **shared structural observation encoding** (not debug `println!`, not locale-dependent formatting) | Highest coordination cost — defer until observation codec is frozen |
| **4 — User-program corpus** | Lane 1 identity | Stable programs promoted from **Lane 1 certification corpus**; each row classified by **observable value shape** (primitive vs record vs simple variant vs later list) | Breadth comes from **program identity reuse**, not ad hoc L5-only prose |
| **5 — Strict L5 fire** | Acceptance | Corpus wide enough to represent the **accepted certification surface**; **every materialized row** passes for the **frozen** Shape A target set | **`l5_cross_target_consistency`** strict mode — still **not** byte identity |

### 1.1 Slice-2 `Int` arithmetic — overflow / range gate (Tier 2 totality)

Slice 2 cannot admit **`+` / `−` / `×`** on default `Int` for strict cross-target rows until **overflow and intermediate-range behavior** are as constrained as division partiality already is. Rust / Python / Go agree on **IEEE-ish numeric towers only where the LanguageSpec + row numeric policy say so**; for fixed-width `Int` the missing piece is a **single named rule** every target realizes the same way.

**Grounding today:** default `Int` is **`Int64`** (`type Int = Int64` — [`dsl/std/integer.dag`](../../dsl/std/integer.dag)). Broader magnitude / refinement-parametric overflow policy is **R3 numeric-construction dispatch** — see [`docs/design-numeric-construction.md`](../design-numeric-construction.md) and the integer-overflow row in [`docs/thesis/r2-r3-thesis-mapping.md`](../thesis/r2-r3-thesis-mapping.md) — not an implicit “every target wraps the same” assumption.

**Design-lock hook:** [`design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) §Corpus Policy already requires a per-row **numeric policy**. For slice-2 `Int` ops, that policy must record **one** of:

1. **Proven overflow-freedom on `i64`** — every literal, operand, and intermediate for `+` / `−` / `×` is in a range where overflow **cannot** occur (conservative small-integer programs). This is the **default** allowed path for Tier-2 totality **until** (2) exists.
2. **Named cross-target overflow semantics** — specified in LanguageSpec / numeric policy (e.g. trap vs two’s-complement wrap vs fail-closed emit) and shared by all Shape A emitters. Until this is written and wired, **saturation-edge** arithmetic programs are **not** strict-L5 evidence (fail closed, same spirit as deferred float policy in the design lock).

**Division / remainder:** unchanged — use `/` (and related partial ops) only when partiality matches **across** targets.

**Non-claim:** This brief does not pick wrapping vs trap; it **requires** the policy surface above before +/−/× drives cross-target certification.

**Explicit non-claims:** No L6 form-coverage absorption; no assertion that L5 retires or dissolves L4; Lane 2 still compares **targets to each other**, not target-vs-evaluator (see scaffold notes §"Critical-Path Consumption From Lane 1").

---

## 2. Algebraic-equivalence harness primitive

**Definition (operational):** For one `TestClaim` row, compile the **same** `.dag` program text to each frozen target, run each artifact under the harness policy, read the **named output bind**, and judge **PASS** iff there exists a single **normalized structural value** such that every target’s observation is **algebraically equal** to that value.

**Structural value domain (phased):**

- **Phase A (slices 1–2):** `Int`, `Bool`, **finite product records**, and **closed simple variants** whose payloads range only over Phase A types — the same structural **`VariantValue`** equality surface as [`design-cross-target-equivalence.md`](../design-cross-target-equivalence.md#equality-domain) (tag = declaration/constructor identity; recursive payload equality). Record field order is normalized per LanguageSpec tables (implementation detail for the worker). **Phase A excludes** slice‑3 **collections** (`List`, maps): those remain Phase B once the observation codec lands.
- **Phase B (slice 3+):** finite **lists** / **maps** whose elements are Phase A values once §3 observation contract extends — still **closed** algebraic sums/products only; no opaque host pointers.

**Normalization:** Each target emits or prints something **losslessly parseable** into the shared domain (binary encodings acceptable only if spec’d cross-target). **Not accepted:** raw stdout byte equality, string fuzzy match, or “same console text.”

**Comparison:** Algebraic equality on the normalized domain (value equality on closed algebraic types). **Optional oracle constants** in `.dag` are **evidence only** — they do **not** define L5 authority (oracle-centric rows blur into Lane 1 / evaluator comparison).

---

## 3. Producer dispatch shape (per frozen target)

For each target **T** ∈ {Rust, Python, Go} and each corpus **program** **P** (single `TestClaim.source` authority):

1. **Emit:** Lower **P** to a target artifact under the same `Dag` / module identity as sibling targets (deterministic emit already stressed on `main`; L5 adds **semantic** cross-target agreement).
2. **Target compile:** Invoke the grounded toolchain for **T**; failures are **per-target compile failures** (taxonomy §4).
3. **Target run:** Execute in the harness sandbox with agreed argv/env **absent host nondeterminism** unless explicitly typed later.
4. **Capture named bind:** Resolve the agreed observable (e.g. `l5_out`) — **no** silent fallback bind names.
5. **Parse to structural value:** Map captured material into Phase A/B domain (§2); mismatch is **observation parse failure**.
6. **Algebraic equality:** After all targets succeed through step 5, compare normalized values; mismatch is **cross-target mismatch** even if each target “ran fine.”

**`ForAllTargets` substrate (named element):** [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) declares `type TestPredicate` including scaffold variant **`ForAllTargets { command: String, args: List<String>, expect_exit_code: Int }`** (search `ForAllTargets` in-file — line numbers drift across edits). That existing sum arm is the **substrate target** for L5 staging rows; this spec does **not** propose a new `TestPredicate` variant. The raw `(command, args, exit_code)` payload remains **insufficient** for strict cross-target **value** observation (readiness audit §2); runner extension should schedule the **six-step pipeline** §3 behind this variant (or an absorbed successor capability), reserving **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** only for genuinely new substrate facts (e.g. a typed cross-target observation carrier), not for re-labeling `ForAllTargets`.

---

## 4. Cross-target failure taxonomy

Ordered diagnostic stages (each must be **fail-closed** and **attributed** to the failing stage — mirror readiness audit §5):

1. **Emit failure** — compiler cannot produce **T**’s artifact for **P**.
2. **Per-target compile failure** — **T**’s toolchain rejects the emitted source.
3. **Run failure** — runtime error, timeout, or policy abort for **T**.
4. **Observation parse failure** — output cannot be mapped into the structural domain.
5. **Cross-target mismatch** — parses succeed but normalized values differ.

**Ordering matters for triage:** earlier stages preempt later ones. Implementations should surface **which target** failed when reporting stages 2–4.

---

## 5. Target-library portability by slice

| Slice class | Primary risk surfaces | Mitigation spec |
|-------------|----------------------|-----------------|
| **1 — Seed** | Match lowering, call ABI, integer literal edges | Keep observable **`Int`** small and branch-disciplined; document fallback `1+2` seed if branch parity lags (scaffold notes) |
| **2 — Primitives** | Boolean compares, record layout, arithmetic partiality | §1.1 `Int` overflow/range gate for `+` / `−` / `×`; avoid target-specific math libs; gate division/mod on shared partiality tables |
| **3 — Collections** | Iterator protocols, map ordering, stringification traps | **Block** slice 3 until observation codec + ordering semantics are written down cross-target |
| **4 — Lane 1 imports** | Drift between Lane 1 authority and L5 row materialization | Enforce **P2** mechanisms (§6) — never maintain independent edited duplicates |
| **5 — Fire** | Corpus breadth vs flakiness | Require frozen target-set revision **Director-visible** when toolchain pins move |

---

## 6. P2 single-authority for program text (#1393 / bridge #1394)

Parallel editable copies of program text violate **[INVARIANTS §P2](../../INVARIANTS.md#p2-boundary-discipline)** — every fact lives in exactly one authoritative place.

Steady-state **must not** rely on hand-maintained duplicate `TestClaim.source` strings that diverge from Lane 1.

**Preferred mechanisms (pick per row class):**

- **(a)** Shared **`.dag` corpus module** or **declaration import** so Lane 1 and Lane 2 reference the **same** structural program definition.
- **(b)** **Generated** L5 `TestClaim` rows from a single generator input + **CI equality ratchet** (generated bytes checked into git; drift fails CI).
- **(c)** **Director-approved** alternative that preserves a **single** editable authority.

**Bridge-retirement alignment (#1394):** Treat **new** Rust `include_str!` lifts of corpus text as **transitional** only while `include_str!` side-channel retirement is incomplete on the closure ledger; steady state favors **(a)** or **(b)** (readiness audit §4 **Authority note**). The skeleton’s `.v3` + embedded-string ratchet is an **explicit interim pattern** — dissolution must fold that bridge when the NYI harness retires (SG-0 census comment on `main`).

---

## 7. Substrate introduction flag (INVARIANTS §P1)

If slice 2–3 requires a **new nominal carrier** for observations (e.g. typed `CrossTargetValue` in substrate), or extends comparators beyond closed algebraic types, that is **not** a fixture-level workaround — route **Director ratification** per **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** before landing enum/type edits.

---

## Coordination

- **cool-crab-614:** second-batch witness shapes — keep observation domain compatible with §2–§3.
- **loyal-ibex-851:** Lane 1 per-(algebra, law) content — orthogonal to L5 value comparison but shares **corpus identity** imports for slice 4.

**Reply path:** Verification Manager inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).
