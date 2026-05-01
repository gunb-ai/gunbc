# Lane 1 → Lane 2 corpus identity import contract

**Status:** PROPOSAL — research-only concrete import contract for **P2 single-authority** program text between Lane 1 (T-V-L4-L7-Direct) and Lane 2 (T-V-L5-Corpus). **Merged design/invariant anchors on `main`:** **[INVARIANTS §P2](../../INVARIANTS.md#p2-boundary-discipline)** (Boundary Discipline — no parallel editable authorities); **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** (Modeling Faithfulness — substrate-fact introduction before new carriers); [`r3-v-l5-corpus-readiness-audit.md`](r3-v-l5-corpus-readiness-audit.md) §4 ([§4 heading](r3-v-l5-corpus-readiness-audit.md#4-critical-path-consumption-from-lane-1-boundary)) — program-text import options + bridge **#4** posture; [`design-cross-target-equivalence.md`](../design-cross-target-equivalence.md) — L5 algebraic-equivalence lock; [`design-test-infra.md`](../design-test-infra.md) — `TestClaim` / DB-15 structural authority (no duplicate test-schema forks); [`r3-structure.md`](../r3-structure.md) — Verification gates including **`l5_cross_target_consistency`**. **Dispatch only (not merged authorities):** PR [**#1412**](https://github.com/gunb-ai/gunbc/pull/1412) (Lane 2 corpus extension spec — cite merged paths above for implementation contracts). Further context: P2 anchor PR **#1393**, bridge-retirement posture PR **#1394**, Lane 1 readiness audit ([`r3-v-l4-l7-direct-readiness-audit.md`](r3-v-l4-l7-direct-readiness-audit.md) / loyal-ibex **#1392** failure taxonomy), interim L5 skeleton (**#1408**, SG-0 bridge note **#1409**). **No substrate edits, no fixtures, no new `TestPredicate` variants** in this brief.

**Non-goals:** Steady-state **`include_str!`** corpus lifts in Rust (**#1394**); hand-maintained duplicate `TestClaim.source` strings; claiming L5 absorbs L4.

---

## In-repo authority anchors (not PR labels alone)

GitHub PR numbers (**#1393**, **#1394**, **#1412**, …) are **dispatch provenance**, not a substitute for merged design text. This contract’s **in-tree** Markdown links resolve from **`docs/briefs/`** as **`../../…`** for repo-root files (`INVARIANTS.md`, `TESTING.md`, `src/…`) and **`../…`** for other files under `docs/` — a **`../INVARIANTS.md`** link would incorrectly resolve under `docs/` and **does not exist** (common false “missing authority” report). §P1 / §P2 URL fragments (`#p1-modeling-faithfulness`, `#p2-boundary-discipline`) target GitHub’s auto-generated heading anchors for `## P1: Modeling Faithfulness` and `## P2: Boundary Discipline` in [`INVARIANTS.md`](../../INVARIANTS.md). **Same commitments (compact):** list items **1–2** under **[The five principles](../../INVARIANTS.md#the-five-principles)** (“Modeling Faithfulness”, “Boundary Discipline”). **Mechanical live check:** §receipt below — **(1)** every hyperlinked path exists on `origin/main`; **(2)** each **in-repo `#fragment`** this brief cites still has backing prose on `origin/main` (expected heading line or explicit `<a id="…">`), so anchor drift fails closed instead of passing on path existence alone.

- **[INVARIANTS.md](../../INVARIANTS.md#p2-boundary-discipline) — §P2 Boundary Discipline** — “every fact lives in exactly one authoritative place”; parallel copies are the failure mode import mechanisms must prevent.
- **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness) — Modeling Faithfulness** — substrate extensions (`CertifiedProgramText`, widening `TestClaim`, …) route through the §P1 substrate-fact introduction procedure; not ad hoc fixture forks.
- **[r3-v-l5-corpus-readiness-audit.md](r3-v-l5-corpus-readiness-audit.md#4-critical-path-consumption-from-lane-1-boundary) §4** — P2 program-text options + bridge-retirement posture for certification corpus lifts (Verification lane narrative).
- **[design-cross-target-equivalence.md](../design-cross-target-equivalence.md)** — corpus numeric policy, oracle policy, and algebraic equality domain L5 consumes.
- **[design-test-infra.md](../design-test-infra.md)** — DB-15 posture: `TestClaim` in [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) is the structural authority; duplicate prose/schema forks violate the same “single authority” discipline as P2 program text.
- **[r3-structure.md](../r3-structure.md)** — R3 gate **`l5_cross_target_consistency`** and Verification lane placement.
- **[ROADMAP.md](../../ROADMAP.md)** — program-level R3 Verification / debt narrative (dispatch context; named gates stay authoritative in [`r3-structure.md`](../r3-structure.md)).
- **[modeling-discipline.md](../modeling-discipline.md)** — read alongside any future **§P1** carrier that merges `source` + `file_name` into one nominal.

### In-tree link receipt (paths + cited `#fragments`)

Re-run whenever **`main`** moves materially. **Extend both steps** when adding paths or in-repo `#fragments` from this brief.

```bash
git fetch origin
for p in \
  INVARIANTS.md \
  ROADMAP.md \
  TESTING.md \
  docs/modeling-discipline.md \
  docs/design-cross-target-equivalence.md \
  docs/design-test-infra.md \
  docs/r3-structure.md \
  docs/r2-closure-ledger.md \
  docs/briefs/r3-v-l5-corpus-readiness-audit.md \
  docs/briefs/r3-v-l4-l7-direct-readiness-audit.md \
  src/v3/compiler/src/test_runner.rs \
  src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs \
  src/v3/std/verification.dag
do git cat-file -e "origin/main:$p" || exit 1; done

# Every `#fragment` hyperlinked from this brief — expected Markdown headings or explicit IDs on origin/main.
git show origin/main:INVARIANTS.md | rg -q '^## The five principles$' || exit 1
git show origin/main:INVARIANTS.md | rg -q '^## P1: Modeling Faithfulness$' || exit 1
git show origin/main:INVARIANTS.md | rg -q '^## P2: Boundary Discipline$' || exit 1
git show origin/main:INVARIANTS.md | rg -q '^## P5: Progress Is Dissolution$' || exit 1
git show origin/main:INVARIANTS.md | rg -q '<a id="db-1"></a>' || exit 1
git show origin/main:INVARIANTS.md | rg -q '<a id="c-5"></a>' || exit 1
git show origin/main:docs/briefs/r3-v-l5-corpus-readiness-audit.md | rg -q '^## 4\. Critical-path consumption from Lane 1 \(boundary\)$' || exit 1
git show origin/main:TESTING.md | rg -q "^### Don't assert on implementation details$" || exit 1
```

---

## Program identity binding (logical contract)

**Program identity** for one corpus row is the **inseparable pair** `(source_text, file_name)` passed to `compile_to_dag(source, file_name)`. Today `TestClaim` projects that identity into **two sibling fields** without a dedicated substrate record (**at HEAD** — see [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag)).

**Contract:** Authoring must treat the pair as **one binding** then **project** into `TestClaim`:

- **Never** edit `source` or `file_name` **in isolation** in steady state (that re-opens parallel authority and breaks diagnostics consistency).
- **Mechanisms (a)/(b):** a **single** import resolution or **single** generator transaction produces **both** projections for each lane; Lane 1 vs Lane 2 differs in **predicate / suite**, not in independently maintained halves of the pair.
- **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness) steady-state option:** a Director-ratified nominal (illustrative: **`CertifiedProgramText`**) holding both strings once, with lowering into `TestClaim` — eliminates the “two-field projection” drift surface at the substrate layer.

---

## Shared requirements (all mechanisms)

- **Single editable authority** per corpus program row — second copies are either generated or read-only structural imports.
- **Cross-lane row key (`HEAD`):** **`TestClaim.name`** — surfaced as **`TestClaimValue.claim_name`** after compile ([`test_runner.rs`](../../src/v3/compiler/src/test_runner.rs)) — is the **authoritative join** for pairing Lane 1 vs Lane 2 rows in CI ratchets; **`source` / `file_name`** remain the compiled program identity payload (§Program identity binding).
- **Program identity** per row is **one binding** projecting to **`TestClaim.source` + `TestClaim.file_name` together** — see §Program identity binding (not two unrelated strings).
- **CI-visible drift detection** — silent divergence between Lane 1 and Lane 2 rows is unacceptable (ratchet shape varies by mechanism below).
- **Harness assertion posture** for future integration code: pin **`ClaimResult` / typed outcomes** (`Pass` / `Fail` / `NotYetImplemented(_)`) structurally — today these variants are the Rust **`TestRunner`** carrier in [`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs) (`pub enum ClaimResult`, not a separate `.dag` nominal yet); combine with **[INVARIANTS DB-1](../../INVARIANTS.md#db-1)** (typed diagnostic carriers, not ad hoc warning text) and **[C-5](../../INVARIANTS.md#c-5)** (no string-sentinel probing); operational examples in [**TESTING.md**](../../TESTING.md#dont-assert-on-implementation-details) §“Don’t assert on implementation details.”

---

## Mechanism (a) — Shared `.dag` corpus module + declaration import

### Shape

1. **One fixture module** holds authoritative program text per certification row, e.g. path  
   `src/v3/compiler/tests/fixtures/corpus/r3_certification_corpus.dag`  
   with module name aligned to existing fixture discipline (illustrative: `std.r3_certification_corpus` — match repo naming rules when implementing).

2. **Per-program bindings** expose the **same** string the compiler uses for `compile_to_dag(source, file_name)` — today `TestClaim` carries `source: String` and `file_name: String` directly ([`src/v3/std/verification.dag`](../../src/v3/std/verification.dag)); there is **no** separate `ProgramSource { source, file_name }` nominal on substrate **at HEAD**.

3. **Concrete binding pattern (research target):**
   - **Pair discipline (mandatory):** One authoring step / one generator emission defines **both** `source` and `file_name`. **Do not** maintain two independently hand-edited `data …: String` declarations for the same row without a freshness ratchet — that is a **parallel-authority** footgun (violates [INVARIANTS §P2](../../INVARIANTS.md#p2-boundary-discipline) intent).
   - **Acceptable at HEAD:** (i) generated `.dag` fragment that sets both `TestClaim` fields from a **single** template input; (ii) compiler-supported structural literal that supplies both strings **atomically** in one declaration; (iii) interim **paired** imports only if CI ratchet below proves **both** fields stay byte-identical across lanes on every change.
   - Lane 1 fixture (`r3_verification_l4_emit_eval_match.dag` family) and Lane 2 fixture (`r3_verification_l5_corpus.dag` family) each consume the **same projected pair** for a given **row identity** — conventionally the substrate **`TestClaim.name`** field (see CI ratchet).

4. **If imports cannot splice into `TestClaim` fields** at lowering time (tooling gap), do **not** fork strings by hand — fall through to mechanism **(b)** or file a **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** substrate extension for a minimal **`CertifiedProgramText`** record type reused by both lanes (single carrier → dual projection at lowering).

### CI ratchet (byte-level)

- **Live realization shape (`HEAD`):** the Rust harness already projects each compiled `TestClaim` declaration into **[`TestClaimValue`](../../src/v3/compiler/src/test_runner.rs)** (`source`, `file_name`, `claim_name`, …) via **`TestClaimValue::from_declaration`** — not a separate `.dag` nominal, but a **real, typed integration boundary** used today (e.g. [`r3_verification_l4_l7_l5_skeleton_test.rs`](../../src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs)).
- **Join discipline (explicit boundary):** pairing Lane 1 vs Lane 2 rows uses an agreed **claim identity** — by default the substrate **`name`** string (== `TestClaimValue.claim_name`). There is **no** dedicated `CorpusKey` carrier **at HEAD**; if `name` is not a sufficient join key for a future corpus, introduce one under **[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)** rather than inventing ad hoc string conventions in fixtures.
- Add a **hermetic integration assertion**: compile the Lane 1 verification DAG + Lane 2 verification DAG, resolve the two structural `TestClaim` declarations with the **same** `claim_name`, build **`TestClaimValue`** for each, then `assert_eq!(l4.source, l5.source)` and `assert_eq!(l4.file_name, l5.file_name)`.
- This is the **import-resolution ratchet**: both lanes must resolve to identical bytes even if authored as separate `TestClaim` rows.

---

## Mechanism (b) — Generated rows from single source + freshness ratchet

### Shape

1. **Single text authority** on disk outside duplicated fixtures, e.g.  
   `src/v3/compiler/tests/corpus_sources/add_then_branch.v3`  
   (extension illustrative — pick one grammar consistent with both lanes).

2. **Deterministic generator** (acceptable homes: `xtask/` command or **`build.rs`** scoped to `v3-compiler` tests — choose by repo policy; generator must be **deterministic**, **side-effect free**, stable ordering).

3. **Outputs:** generated fragments **checked into git** that materialize, **in one transaction per row**, Lane 1 `TestClaim.source` **and** `file_name` plus Lane 2 `TestClaim.source` **and** `file_name` — both lanes’ pairs derived from the **same** generator inputs (never regenerate only one field for one lane).

### CI ratchet

- **`cargo … gen --check`** (pattern): regenerate into a temp dir or stdout; **fail** if `git diff` would be non-empty for tracked outputs.
- Alternative: integration test runs generator and compares to `include_bytes!` of checked-in expectation — still **generated bytes are canonical**, not hand-edited.

### Constraints

- No timestamps, no locale, no environment-dependent paths in output.
- Generator inputs are **the only** editable corpus prose for that row class.

---

## Mechanism (c) — Director-approved alternative (escape hatch only)

Use when **(a)** or **(b)** is blocked by a genuine toolchain limitation **and** Director documents the exception.

This hatch is **scaffold-class** relief, not exempt steady-state. **[INVARIANTS §P5 — Progress Is Dissolution](../../INVARIANTS.md#p5-progress-is-dissolution)** applies: scaffolds need explicit dissolution paths — in particular the §P5 *Scaffold without dissolution trigger* shape resolves only when every scaffold lands with a **named dissolution trigger** (a specific, checkable condition that closes it). Director approval that adds only a ledger hook **without** bounded scope **and** such a trigger blesses **untracked bridge debt** and does **not** meet INVARIANTS Dispatch-Discipline for scaffold introductions (paired dispatch: dissolution trigger + adjacent debt visibility).

### Acceptable patterns

- **Single** programmatic owner (generator or substrate-adjacent template) with **read-only** consumption elsewhere.
- **Bounded scope** in the Director record: which lanes, fixtures, row classes, or claim identities the exception covers — **no** open-ended waiver over arbitrary corpus rows.
- **Named dissolution trigger**: the checkable event that **retires** this exception (e.g. a §P1 substrate carrier lands; mechanism **(a)** / **(b)** becomes implementable and is scheduled). The trigger must be **machine-checkable or audit-checkable** in principle (merge milestone, issue/PR closure criterion, ledger row completion), not prose-only “eventually.”
- **Explicit ledger entry** tying the exception to bridge-retirement / **§P2** posture — e.g. [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) row, **ROADMAP** debt bullet, or owning brief §Acceptance naming the same scope + trigger (no silent carve-outs). **Ledger without scope + trigger is insufficient.**

### Anti-patterns

- Director records that cite “ledger” or bridge posture **only**, omitting **bounded scope** or **dissolution trigger**.
- Independently edited **duplicate** `TestClaim.source` blobs between Lane 1 and Lane 2 fixtures.
- **Serialized** copy/paste text with informal “keep in sync” comments but **no** CI equality ratchet.
- **Locale-sensitive** or **stdout-shaped** comparisons as stand-ins for source equality.

---

## Interim posture (#1408 / #1409)

The landed skeleton uses **sidecar `.v3` + embedded `TestClaim.source` + harness `assert_eq!`** — valid **only** as interim until `ForAllTargets` wiring + P2 steady-state mechanism lands; dissolution must **fold** that bridge per SG-0 census commentary (**#1409**). Do **not** treat Rust `include_str!` of corpus text as steady-state (**#1394**).

---

## Coordination

- **Lane 1 worker:** preserves failure taxonomy ordering from readiness audit (**#1392** surface) — import contract must not obscure emit vs run vs eval vs mismatch stages.
- **Lane 2 worker:** consumes **program identity only** — comparison semantics remain cross-target algebraic equivalence per [`design-cross-target-equivalence.md`](../design-cross-target-equivalence.md); PR [**#1412**](https://github.com/gunb-ai/gunbc/pull/1412) is dispatch for extension-shape detail once merged.

**[INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness) flags:** introducing **`CertifiedProgramText`**, **`ProgramSource`**, or widening `TestClaim` requires Director ratification — not a fixture-layer workaround.

**Reply path:** Verification Manager inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).
