# v4 ladder fixture — Phase 4 widening: `phase4/field_patch_monoid` (rungs 0–2)

> **Status:** MANAGER RATIFIED — Ladder/Fixture Manager (`wise-seal-69`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2. **9-rung ontology:** PR #3938 §6 — parent doc `docs/planning/v4-correctness-ladder-2026-05-30.md`.
> **Rung gate shape (PASS|FAIL|SKIP, prerequisites, blocking-receipt rules):** authoritative in `docs/planning/v4-ladder-rung-specs-2026-05-30.md` §2. **This doc does not duplicate §2** (CLAUDE.md ledger standing principle); it only names the fixture subject and the rung-2 target-set delta.
> **Predecessor structure:** #3946 (rungs 0–2 spec on `phase1/nat_semiring`), #4003 (rung-3), #3990 (rung-4).
> **Scope:** Ratify `phase4/field_patch_monoid` as Phase 4 Branch-using fixture; bind to the §2 acceptance predicates on rungs 0–2. Rungs 3–4 spec land in follow-up PRs (mirror #4003 / #3990).

---

## 1. Fixture ratification

### 1.1 Ratified Phase 4 fixture (Branch-using)

| Field | Value |
| ----- | ----- |
| **Fixture id** | `phase4/field_patch_monoid` |
| **Module path** | `v4.test.claim.algebra_laws.field_patch_monoid` |
| **Source file** | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag` |
| **Phase** | 4 (Branch-using widening; rungs 0–2 first, rungs 3–4 follow-up) |
| **Predecessor fixture** | `phase1/nat_semiring` (ratified #3946) |
| **Shape filter** | Branch-using — primary control shape is `if/else` (line 33 `fp_ok`) + coproduct discrimination over `FieldPatch { Inherit \| Override }` |

**Ratification rationale (operator-facing):**

- Already in the TestClaim corpus at landed quality (179 lines, 10 `CompilesClaim` rows); no new authoring required, no surface syntax debt.
- Exercises the **Branch-using** program shape the Phase 4 widening calls for: `if/else` selector (`fp_ok`), coproduct match-shape behaviour over `FieldPatch` constructors, and a falsification path via right-bias witnesses (Override wins over Override).
- Covers monoid laws beyond `nat_semiring`'s commutative-semiring axes (left/right identity, associativity, right-bias) — a meaningfully different algebraic surface, so a rung gate flip here surfaces gaps that the semiring fixture cannot.
- Same Node/Atom/Symbol carrier shape as `phase1/nat_semiring`, so emit/typecheck failures diagnose **branch-shape** gaps in `TargetAtomRealization` / `TargetTypeExpressionProjection`, not corpus noise.

### 1.2 Fixture subject (emit input)

The gate applies to the **committed module** at the path above, not to the full `src/v4` corpus.

**Included in the fixture subject:**

- All 10 `claim_*` `CompilesClaim` rows (left identity, right identity, right-bias, associativity, inherit-preserves, override-replaces, monoid-identity-preserves, monoid-left-identity, monoid-right-identity, monoid-right-bias).
- Supporting `data witness_*` rows and `fn fp_*` helpers (including the `if/else` branch carrier `fp_ok`).

**Excluded from Phase 4 rung gates (same as `phase1/nat_semiring`):**

- Global corpus compile (`src/v4/**/*.dag` → rustc).
- Histogram deltas on total rustc error count (see master spec §4).

### 1.3 Alternatives considered and rejected

| Candidate | Rejection |
| --------- | --------- |
| Author a fresh Branch-using fixture from scratch | Duplicates corpus-quality work; defers the rung gate's actual reach into branch shapes. CLAUDE.md "Cost of Change" — one source of truth per claim. |
| `phase1/nat_semiring` extended with branch claims | Conflates phase-1 algebraic surface (commutative semiring) with phase-4 control-shape widening — a single fixture failure no longer diagnoses one axis. |
| `is_prefix_of_prefix_check` as the Branch-using fixture | Mixes branch (`if`/`match`) with recursion (`is_prefix_of`). Reserved for Phase 4 Loop-using fixture (separate PR), where recursion is the primary signal. |

---

## 2. Rung gate shape

**Inherited verbatim from `docs/planning/v4-ladder-rung-specs-2026-05-30.md` §2.** PASS|FAIL|SKIP cell vocabulary, prerequisite chaining (R1 ← R0-rust-parse, R2-rust ⊇ R1, R2-go ← R0-go-parse), `upstream_blocked:*` and `*_emit_unavailable` receipts, row-aggregate (`PASS` iff every cell `PASS`, else `FAIL`, no row-level `SKIP`) — all unchanged for `phase4/field_patch_monoid`.

**Target set (Phase 4 Branch-using):** identical to `phase1/nat_semiring` — `dag + rust + python + go`. No target-set delta in this PR.

**Predicate ids** are namespaced by fixture id, e.g. `phase4/field_patch_monoid/rung1/rust_typecheck_failed`. The shape of the predicate id (`<fixture>/<rung>/<receipt>`) follows the master spec §2.4 verbatim.

**Verdict matrix output shape** (same as master §2.4):

```text
fixture=phase4/field_patch_monoid
  rung0: PASS | FAIL  (dag=… rust=… python=… go=…)
  rung1: PASS | FAIL  (rust=…)
  rung2: PASS | FAIL  (rust=… python=… go=…)
blocking_receipt: <predicate id> | upstream_blocked:<predicate-id> | none
```

---

## 3. TestClaim wiring target (worker implementation)

**Output path (to be authored by wiring PR):** `src/v4/test/claim/field_patch_monoid/rung_0_to_2_three_targets.dag` (mirror `src/v4/test/claim/nat_semiring/rung_0_to_2_three_targets.dag` landed via #3953).

**Structural templates** — paths verified on `origin/main` at `887f0f2ed`:

| Pattern | Reference | Use for |
| ------- | --------- | ------- |
| Fixture import + drift guard | `src/v4/test/claim/nat_semiring/rung_0_to_2_three_targets.dag` (whole-file template) | Whole-fixture pattern; substitute `v4.test.claim.algebra_laws.field_patch_monoid` |
| `CompilesClaim` row | `src/v4/test/claim/self_host/claim_runner_compiles.dag:35-40` | Per-target compile claims (Tier1, Integration where appropriate) |
| Fixture roster | `src/v4/test/claim/workflow/affected_set_ci_runner.dag:99-103` | Named `List<TestClaim>` roster for gate selection |
| Fixture claims to import | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag:92-179` | 10 `CompilesClaim` rows |

**CI wiring (where the matrix lands, follow-up PR):**

| Layer | File | Role |
| ----- | ---- | ---- |
| Authority (jobs/gates) | `src/v4/workflow/ci.dag` | New `CiJob` + `CiGate` for `phase4/field_patch_monoid` rungs 0–2 (mirror `Phase1NatSemiringRungGateCommand`). |
| GHA projection | `dsl/gunbc/ci_github_actions_workflow.dag` | New step/job projecting the modeled command. |
| Host gate script | `scripts/v4-phase4-field-patch-monoid-rung-gate.sh` (follow-up) | Fixture-scoped emit + toolchain; mirror `scripts/v4-phase1-nat-semiring-rung-gate.sh` post-#4015 alignment (§2.1 parse-only + §2.4 SKIP). |

**Worker brief triple (required on the wiring PR):**

```text
fixture=phase4/field_patch_monoid
rung=0|1|2
modeling_gap=none for pure wiring; SG-1|SG-2 only with Modeling DFS worksheet approval
predicate=<R0-*|R1-*|R2-*> expected to flip
```

---

## 4. SG / substrate work acceptance rule

**Same as master spec §3, with `phase4/field_patch_monoid` substituted.** SG fixes are accepted when at least one predicate moves `FAIL → PASS` (or `SKIP → PASS` once R0 cells executable) for `phase4/field_patch_monoid`; not when the global histogram drops. New claim modules must be bound to `fixture=phase4/field_patch_monoid` and a rung id.

---

## 5. "No rustc-clean as headline" enforcement

**Same as master spec §4.** Forbidden headlines (`0 rustc errors`, `rustc clean`, corpus-wide error count as the success criterion) remain forbidden for Phase 4 emit work. Required headline shape:

- `phase4/field_patch_monoid: rung2 PASS (rust python go)`
- `phase4/field_patch_monoid: rung1 PASS (rust); rung2 blocked — python_compile_failed`

Manager dispatch gate (master §4.4) applies unchanged: workers need fixture + rung + predicate id + Modeling DFS worksheet approval (for SG-1/SG-2 touches).

---

## 6. Spot-check receipts (vs `origin/main` @ `887f0f2ed`)

| Claim | Spot-check | Result |
| ----- | ---------- | ------ |
| `field_patch_monoid.dag` in corpus with 10 `CompilesClaim` rows | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag:1-179` | **CONFIRMED** — module imports `Inherit`, `Override`, `FieldPatch`, `compose_field_patch`, `apply_field_patch`; 10 `data claim_*: TestClaim = CompilesClaim` rows at :92-178 |
| `if/else` branch carrier present | `src/v4/test/claim/algebra_laws/field_patch_monoid.dag:32-34` | **CONFIRMED** — `fn fp_ok(b: Bool) -> Node { if b { fp_pass_node() } else { fp_fail_node() } }` |
| Master ladder spec on main | `docs/planning/v4-ladder-rung-specs-2026-05-30.md` | **CONFIRMED** — landed via #3946 (referenced as §2 authority) |
| Rung-3 spec template on main | `docs/planning/v4-ladder-rung-spec-rung3-2026-05-30.md` | **CONFIRMED** — landed via #4003 (follow-up PR template) |
| Rung-4 spec template on main | `docs/planning/v4-ladder-rung-spec-rung4-2026-05-30.md` | **CONFIRMED** — landed via #3990 (follow-up PR template) |
| Post-#4015 rung gate script alignment | `scripts/v4-phase1-nat-semiring-rung-gate.sh` | **CONFIRMED** — §2.1 parse-only + §2.4 SKIP vocabulary; structural template for the phase4 gate script. |

---

## 7. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Phase 4 Branch-using fixture = `phase4/field_patch_monoid` | **RATIFIED** |
| Rungs 0–2 acceptance predicates inherited from master spec §2 | **RATIFIED** |
| Target set (dag + rust + python + go) inherited from Phase 1 | **RATIFIED** |
| TestClaim roster (`rung_0_to_2_three_targets.dag` under `field_patch_monoid/`) | **PENDING** wiring PR |
| Host gate script (`v4-phase4-field-patch-monoid-rung-gate.sh`) | **PENDING** wiring PR |
| CI authority (`ci.dag` + GHA) | **PENDING** wiring PR |
| Rungs 3–4 specs | **PENDING** follow-up PRs (mirror #4003 / #3990) |

**Wave alignment:** W2.5 (Phase 4 widening) per `docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md` §5. Maturation work for v0.1.1 narrative; not gating Jun 1 v0.1.0 tag.
