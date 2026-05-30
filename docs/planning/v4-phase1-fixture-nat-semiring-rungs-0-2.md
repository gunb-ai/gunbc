# Phase 1 fixture ratification: `nat_semiring` × rungs 0–2

> **Status:** MANAGER RATIFIED — Ladder/Fixture Manager (`keen-crab-361`), 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2 (Ladder / Fixture). Parent planning: `docs/planning/v4-correctness-ladder-2026-05-30.md` (PR #3938, `session/nimble-dove-733`).
> **Scope:** Ratify Phase 1 fixture; define acceptance predicates for rungs 0–2 only. Rungs 3–9 are out of scope until Compiler Spine + Runtime/TestClaim managers define executable runner receipts (planning doc §11.4 item 4).

---

## 1. Fixture ratification

### 1.1 Ratified Phase 1 fixture

| Field | Value |
| ----- | ----- |
| **Fixture id** | `phase1/nat_semiring` |
| **Module path** | `v4.test.claim.algebra_laws.nat_semiring` |
| **Source file** | `src/v4/test/claim/algebra_laws/nat_semiring.dag` |
| **Phase** | 1 (rungs 0–2 only) |
| **Widen-after** | Phase 4 adds two more fixtures (Branch-using, Loop-using) per planning doc §7 |

**Ratification rationale (operator-facing):**

- Already in the TestClaim corpus; no new surface syntax required to author the gate.
- Exercises high-value substrate shapes: `Node`, `Atom`, `Conj`, named `Edge`, algebra inhabitance (`CommutativeSemiring<Nat>`), six law subjects + one falsification `DiagnosticClaim`.
- Small enough for end-to-end emit gates; rich enough that rung 1/2 failures diagnose Node-shape gaps instead of corpus noise.
- Replaces corpus-wide “7951 rustc errors” as the **primary** progress signal for emit work in Phase 1.

### 1.2 Fixture subject (emit input)

The gate applies to the **committed module** at the path above, not to the full `src/v4` corpus.

**Included in the fixture subject:**

- All `data claim_nat_*` rows (six `EqualsClaim` law rows + one `DiagnosticClaim` falsification row).
- Supporting `data nat_*_input` law-subject nodes and `falsification_nat_add_wrong_identity`.

**Excluded from Phase 1 rung gates:**

- Global corpus compile (`src/v4/**/*.dag` → rustc).
- Histogram deltas on total rustc error count (see §4).

### 1.3 Alternatives considered and rejected for Phase 1

| Candidate | Rejection |
| --------- | --------- |
| `src/v4/test/fixture/dag_round_trip_mvp1.dag` | Better for rung 3 round-trip; weak on algebra / L7 subject matter for later rungs 5–6. |
| Full `src/v4/compiler/*.dag` | Too large for Phase 1; belongs to self-host / release lane (rung 7+). |
| Corpus-wide rustc clean | Single-rung (rung 1 only), wrong sequencing per planning doc §4–§5. |

---

## 2. Rung gate shape (acceptance predicates)

Each rung is a **binary** gate on the ratified fixture. A rung **passes** only when every predicate in its row holds; otherwise it **fails** with a named blocking receipt (§2.4).

Phase 1 **target set** (explicit subset of project-committed targets):

| Target id | Toolchain | Rung 0 | Rung 1 | Rung 2 |
| --------- | ----------- | :------: | :------: | :------: |
| `dag` | v4 parse | ✓ | — | — |
| `rust` | `rustc` / `cargo check` on emitted Rust | ✓ | ✓ | ✓ |
| `python` | `python3` compile check on emitted Python | ✓ | — | ✓ |
| `go` | `go build` (or project-standard Go check) on emitted Go | ✓ | — | ✓ |

Seven other targets (cpp, ts, lean, swift, …) are **deferred** to Phase 4+ widening; passing rung 2 on three targets is **smoke coverage**, not THESIS L6 closure (planning doc §6 note on rung 2 vs standard #9).

### 2.1 Rung 0 — parses in target

**Question:** Did the fixture parse in each required target’s surface?

| Predicate id | Target | Pass condition | Fail blocking receipt |
| ------------ | ------ | -------------- | --------------------- |
| `R0-dag-parse` | `dag` | `v4` parse of `nat_semiring.dag` yields `Accepted` module AST (no parse `Rejected` diagnostics). | `phase1/nat_semiring/rung0/dag_parse_rejected` |
| `R0-rust-parse` | `rust` | Emitted Rust for the fixture module parses under `rustc` frontend (parse-only or full compile). | `phase1/nat_semiring/rung0/rust_emit_parse_rejected` |
| `R0-python-parse` | `python` | Emitted Python for the fixture parses (`python3 -m py_compile` or equivalent). | `phase1/nat_semiring/rung0/python_emit_parse_rejected` |
| `R0-go-parse` | `go` | Emitted Go for the fixture parses (`go build` / `go vet` parse phase). | `phase1/nat_semiring/rung0/go_emit_parse_rejected` |

**Disposition vocabulary (PR #3938 §10.0):** rung 0 is `ship_disposition: PROVEN` only when all four predicates pass **via executable receipt** (CI log or local `TestClaimRun` verdict). Substrate-only parse claims without execution remain `engineering_state: SUBSTRATE_PRESENT`, `ship_disposition: GAP`.

### 2.2 Rung 1 — type-checks in Rust (standard #1, Rust slice)

**Question:** Does the fixture’s Rust emit type-check?

| Predicate id | Target | Pass condition | Fail blocking receipt |
| ------------ | ------ | -------------- | --------------------- |
| `R1-rust-typecheck` | `rust` | `rustc` / `cargo check` on emitted Rust exits 0 with no type errors for the fixture artifact. | `phase1/nat_semiring/rung1/rust_typecheck_failed` |

**Note:** Rung 1 is necessary but not sufficient for v4 close (planning doc §4). It must not be used as the sole PR success headline (§4).

### 2.3 Rung 2 — compiles in Phase 1 multi-target smoke (subset of standard #9 / L6)

**Question:** Does the fixture emit compile in Rust, Python, and Go?

| Predicate id | Target | Pass condition | Fail blocking receipt |
| ------------ | ------ | -------------- | --------------------- |
| `R2-rust-compile` | `rust` | Same as `R1-rust-typecheck` (rung 2 Rust ⊇ rung 1). | `phase1/nat_semiring/rung2/rust_compile_failed` |
| `R2-python-compile` | `python` | Emitted Python compiles / type-checks per project Go/Python gate policy. | `phase1/nat_semiring/rung2/python_compile_failed` |
| `R2-go-compile` | `go` | Emitted Go builds without compile errors. | `phase1/nat_semiring/rung2/go_compile_failed` |

**Explicit non-goals for rung 2 Phase 1:**

- Cross-target semantic equivalence (rung 5).
- Post-emit algebraic law re-check (rung 6).
- Full L6 “every form × every target” (requires stable target set + corpus-wide gate).

### 2.4 Verdict reporting shape

Gate output for operators, PR summaries, and CI must use this matrix (example):

```text
fixture=phase1/nat_semiring
  rung0: PASS | FAIL  (dag rust python go)
  rung1: PASS | FAIL  (rust)
  rung2: PASS | FAIL  (rust python go)
blocking_receipt: <predicate id> | none
```

Optional appendix (not headline): global rustc error count, top error classes, link to `docs/audit/v4-rustc-error-catalog-2026-05-29.md`.

### 2.5 TestClaim wiring target (worker implementation)

Follow-on worker lands substrate at:

- `src/v4/test/claim/nat_semiring/rung_0_to_2_three_targets.dag`

Minimum claim shapes (map to existing `v4.std.verification` variants):

| Rung | Suggested claim variant | Notes |
| ---- | ----------------------- | ----- |
| 0–1 | `CompilesClaim` per target slice | `input` = fixture module subject node; `expected_value` = same node until eval path supplies runtime value. |
| 2 | `CompilesClaim` × 3 targets OR one integration row with `TestClassification { tier: Tier1, layer: Integration }` | Must name `rust` / `python` / `go` in label or anchor metadata until target axis is a first-class field. |

Execution may remain `Deferred` behind T-38 **only** for claims that do not yet have runner wiring; the **CI gate** for Phase 1 must still invoke the toolchain checks directly until `TestClaimRun` verdicts are PROVEN. Substrate-only `CompilesClaim` rows without toolchain invocation do not satisfy §2.1–§2.3.

---

## 3. SG / substrate work acceptance rule (Phase 1)

From planning doc §7 “unit of work”:

> SG fixes accepted only when they move the fixture rung, not when the histogram drops.

**Enforcement:**

| Work type | Accept when | Reject when |
| --------- | ----------- | ----------- |
| SG-1 / SG-2 / emit substrate fix | At least one predicate in §2 moves FAIL → PASS for `phase1/nat_semiring` | Only global error count decreases; fixture matrix unchanged |
| New `TestClaim` / claim module | Bound to `fixture=phase1/nat_semiring` and a rung id | Orphan claim with no rung matrix row |
| CI change | Adds or tightens §2 matrix step | Adds only corpus-wide rustc gate as required check name |

---

## 4. “No rustc-clean as headline” (PR #3938 §11.1)

### 4.1 Forbidden headlines

The following must **not** appear as the primary PR title, primary CI check name, or first bullet in a merge summary for Phase 1 emit work:

- “0 rustc errors” / “rustc clean” / “7951 → N” as the **success criterion**
- “T-15 close” implied by corpus-wide compile green only
- “SG-1 fixed” without a fixture×rung matrix delta

### 4.2 Required headlines

Primary success statement must be fixture×rung shaped, e.g.:

- `phase1/nat_semiring: rung2 PASS (rust python go)`
- `phase1/nat_semiring: rung1 PASS (rust); rung2 blocked — python_compile_failed`

### 4.3 Allowed secondary metrics

Global rustc error count may appear only:

- In PR body appendix or audit docs
- With explicit label **diagnostic**, not **gate**
- After the fixture matrix section

### 4.4 Manager dispatch gate

Do not dispatch “Fix SG-*” or “M1-class-fix” workers without a brief that names:

1. `fixture=phase1/nat_semiring`
2. `rung` ∈ {0, 1, 2}
3. `predicate id` from §2.1–§2.3 expected to flip
4. Modeling DFS worksheet approval when touching `TargetAtomRealization` / `TargetTypeExpressionProjection` (planning doc §11.4)

---

## 5. Current baseline (as of ratification)

| Rung | Expected baseline on `main` | Notes |
| ---- | --------------------------- | ----- |
| 0 `dag` | **PASS** (module committed and parseable) | Receipt: file present + v4 parse in dev workflow. |
| 0 `rust/python/go` emit | **FAIL** (`GAP`) | Emit + parse not gated on fixture. |
| 1 | **FAIL** (`GAP`) | Corpus blocked; fixture-specific path unproven. |
| 2 | **FAIL** (`GAP`) | CI invokes Rust only; no Python/Go fixture gate. |

This baseline is planning truth until a worker lands §2.5 with executable receipts. Updating the matrix to PASS requires evidence, not declaration.

---

## 6. Related artifacts

| Artifact | Role |
| -------- | ---- |
| `docs/planning/v4-correctness-ladder-2026-05-30.md` | 9-rung ontology, §7 sequencing (PR #3938) |
| `src/v4/test/claim/algebra_laws/nat_semiring.dag` | Ratified fixture corpus |
| `src/v4/std/nat.dag` | Algebra law symbols imported by fixture |
| `docs/audit/v4-close-interrogation-validation-2026-05-30.md` | Probe-level disposition; complements ladder gates |
| `docs/audit/v4-rustc-error-catalog-2026-05-29.md` | Diagnostic appendix only |

---

## 7. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Phase 1 fixture = `nat_semiring` | **RATIFIED** |
| Rungs 0–2 acceptance predicates (§2) | **RATIFIED** |
| Phase 1 target set = dag + rust + python + go | **RATIFIED** |
| No rustc-clean headline rule (§4) | **RATIFIED** |
| Worker may wire `rung_0_to_2_three_targets.dag` + CI | **UNBLOCKED** (pending Modeling DFS approval for any SG-1/SG-2 substrate touches) |
