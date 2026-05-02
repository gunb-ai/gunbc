# R3 T-Tests-As-Data-Completeness — readiness audit

**Status:** PROPOSAL — research-only. Foundational dispatch artifact for **T-Tests-As-Data-Completeness** (Category E; Verification Manager). **No substrate edits**, no new `TestPredicate` variants, no fixture authoring in this document.

**Lane authority:** [`docs/r3-structure.md`](../r3-structure.md) table row **T-Tests-As-Data-Completeness** (L; gates `every_rust_test_ports_to_dag_or_generated`, `forall_exists_quantifier_substrate_landed`, `program_generator_carrier_landed`, `lens_cementing_test_discipline_complete`) — see ~L147.

**Design lock (read-only):** [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) — substrate shapes, migration path, facet-3 coverage thesis. This audit is **HEAD-state + gap** against that design, not a second authority.

**Cross-refs:** [`docs/design-test-infra.md`](../design-test-infra.md) (DB-15), [`TESTING.md`](../../TESTING.md), [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md), [`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md).

---

## 1. Executive snapshot (HEAD today)

| Closure gate | HEAD state (this audit) | Blocking on |
|---|---|---|
| `every_rust_test_ports_to_dag_or_generated` | **Open.** SG-0 census still enumerates **87** explicit hand-authored test paths under `src/v3/compiler/tests/` (`EXPECTED_HAND_AUTHORED_TEST` — [`sg0_census_test.rs`](../../src/v3/compiler/tests/integration/sg0_census_test.rs) L243+). That equals **all** `*.rs` leaves under `src/v3/compiler/tests/` today (tree count 87) — the ratchet is total, not partial. | Emission tables for `TestPredicate` → generated target tests; shrinking census to **0** per design doc §1.1 |
| `forall_exists_quantifier_substrate_landed` | **Open.** `TestPredicate` has **no** `ForAll` / `Exists` quantifier variants for property-based families — only execution-scoped scaffold **`ForAllTargets`** ([`verification.dag`](../../src/v3/std/verification.dag) L161–168). | **INVARIANTS §P1** substrate introduction for quantified claims (per design doc §2) |
| `program_generator_carrier_landed` | **Open.** No `ProgramGenerator` / `QuantifiedTestClaim` types in [`verification.dag`](../../src/v3/std/verification.dag) at HEAD; design-only in [`design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §2. | **§P1** + Substrate Manager coordination |
| `lens_cementing_test_discipline_complete` | **Partial.** Band-C cementing dispatch is **live** Rust integration ([`cementing_lens_registry_dispatch_test.rs`](../../src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs) — registry/capability alignment). **Zero-gap** “every COMPLETE lens has cementing module” is a **lane closure** target tied to **T-Lens-Behavioral-Parity** + capability register ([`v3-lens-capability-register.md`](../v3-lens-capability-register.md)). | Behavioral completeness rows + cementing module coverage; not a `TestPredicate` addition |

**Harness defaults (worker-facing):** **`OnceLock` + `cached_compile`** where applicable; assert **`ClaimResult`** / diagnostic carriers **by shape** ([`TESTING.md`](../../TESTING.md)); keep workflow parallelism out of compile-time dimension slots (**DB‑3 / DB‑20** split per [`design-dimension-abstraction.md`](../design-dimension-abstraction.md)).

---

## 2. Gate A — `every_rust_test_ports_to_dag_or_generated`

**Inventory:** [`sg0_census_test.rs`](../../src/v3/compiler/tests/integration/sg0_census_test.rs) **`EXPECTED_HAND_AUTHORED_TEST`** (L243+) — **87** paths — is the authoritative “still hand-Rust” set for the test partition.

**Qualitative portability (not a % — avoids false precision):**

- **Already DAG-adjacent:** large integration corpus uses **`cached_compile`** + fixture `.dag` / `.v3` and structural assertions (same *shape* as future `TestClaim` execution).
- **Ratchet / meta tests:** [`canonical_lens_bridge_ratchet_test.rs`](../../src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs) pins bridge surface in `test_runner.rs` — **not** a property of `TestClaim` yet; porting awaits bridge retirement / typed registry ([`r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md) narrative in-file).
- **Substrate structural acceptance:** many entries are Director-approved **hand-Rust over reflected `Dag`** until testgen can express the same structural claims as `.dag` `TestClaim` (comments in-list cite dissolution into `TestClaim` when testgen covers reflected substrate).

**Design-target:** **0** hand-authored test files at R3 close for this partition — [`design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §1.1 (facet 3).

---

## 3. Gate B — `forall_exists_quantifier_substrate_landed`

**HEAD:** No mathematical **`ForAll` / `Exists`** over program families on `TestPredicate`. **`ForAllTargets`** is **per emission target**, not quantification over generated programs ([`verification.dag`](../../src/v3/std/verification.dag) L153–168).

**Route:** **INVARIANTS §P1** — new carriers only via substrate-fact procedure ([`INVARIANTS.md`](../../INVARIANTS.md#p1-modeling-faithfulness)). Align with design doc §2 quantifier + `ProgramGenerator` sketch.

---

## 4. Gate C — `program_generator_carrier_landed`

**HEAD:** **Absent** from [`verification.dag`](../../src/v3/std/verification.dag). **Authority:** design doc §2 (`ProgramGenerator`, `ProgramShape`, quantified claim) — **not** implemented in-tree.

**Shared prerequisite:** any generator-driven claim will still project through **`TestClaimValue` / `compile_to_dag(source, file_name)`**-class facts — same projection surface as Lane 1↔Lane 2 import contract ([`r3-v-lane1-lane2-corpus-identity-import-spec.md`](r3-v-lane1-lane2-corpus-identity-import-spec.md)); coordinate with corpus + verification harness work.

---

## 5. Gate D — `lens_cementing_test_discipline_complete`

**HEAD patterns:**

- **Cementing dispatch:** [`cementing_lens_registry_dispatch_test.rs`](../../src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs) — Band-C registry/capability alignment, `compile_to_dag` oracle comparisons (see module docs L15–23).
- **Witness taxonomy:** [`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md) — `LensOutputEquals`, `DifferentialEquals`, `BinaryDimensionReportEquals` readiness differs per predicate; don’t confuse lens-fold with corpus runtime.

**Closure definition (lane):** every **`LensRegistryEntry`** marked **BEHAVIORALLY COMPLETE** with a real v2 counterpart has a **cementing module** — ties to **T-Lens-Behavioral-Parity** (Substrate + Verification cross-program per [`r3-structure.md`](../r3-structure.md)).

---

## 6. Slice progression (dispatch-ready staging)

| Slice | Fire criteria (proposal) |
|---|---|
| **1 — Inventory + census honesty** | SG-0 list reconciled to tree; every new test path either lands in census or removes hand-authorship |
| **2 — Predicate coverage mapping** | Each **class** of Rust test mapped to a **`TestPredicate`** variant or explicit “blocked on carrier” row (per design doc §3 migration table when extended) |
| **3 — Generated target tests** | Path B emission from [`design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §1.3 — per-target `TestClaim` rendering |
| **4 — Quantifiers + `ProgramGenerator`** | **§P1** carriers landed; runner can evaluate quantified claims |
| **5 — Facet-3 close** | Census **0** hand-authored test files; quantifiers + generator + cementing discipline satisfied |

---

## 7. Cross-lane coordination

- **T-Verification-L4-L7-Direct / L5-Corpus:** shared **`TestClaim` / `TestRunner`** infrastructure and import-contract identity discipline — any **`ProgramGenerator`** work should reuse the same projection boundaries.
- **T-Free-Consequences-Demonstration:** witness patterns overlap — use **witness survey** categories; avoid parallel `DimensionReport` misuse ([`r3-v-witness-shape-pattern-survey.md`](r3-v-witness-shape-pattern-survey.md)).
- **T-Lens-Behavioral-Parity / T-Lens-Application-Surface:** cementing + behavioral completeness **precede** full facet-3 closure for lens rows.

**Escalation:** shared **§P1** blockers → Director / Substrate Manager / inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).

---

## 8. Live-path verification receipt

Re-run whenever **`main`** moves or links change. **Extend** when adding hyperlinks.

```bash
git fetch origin
for p in \
  INVARIANTS.md \
  TESTING.md \
  docs/r3-structure.md \
  docs/design-tests-as-data-completeness.md \
  docs/design-test-infra.md \
  docs/design-dimension-abstraction.md \
  docs/v3-lens-capability-register.md \
  docs/briefs/r3-v-witness-shape-pattern-survey.md \
  docs/briefs/r3-v-lane1-lane2-corpus-identity-import-spec.md \
  docs/briefs/r2-pb-canonical-lens-bridge-disposition.md \
  src/v3/std/verification.dag \
  src/v3/compiler/src/test_runner.rs \
  src/v3/compiler/tests/integration/sg0_census_test.rs \
  src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs \
  src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs
do git cat-file -e "origin/main:$p" || exit 1; done
```

After this brief merges, add **`docs/briefs/r3-v-tests-as-data-completeness-readiness-audit.md`** to the loop.

---

## 9. Re-engagement

1. When **`design-tests-as-data-completeness.md`** or **`verification.dag`** changes materially, refresh §1–§5.
2. When **§P1** carriers for quantifiers/generator land, re-audit gates B–C against `origin/main` HEAD.

**Reply path:** Verification Manager inbox [#1276](https://github.com/gunb-ai/gunbc/issues/1276).
