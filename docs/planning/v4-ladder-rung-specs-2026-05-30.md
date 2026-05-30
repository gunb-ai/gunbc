# v4 ladder rung specs — Phase 1 (`nat_semiring` × rungs 0–2)

> **Status:** MANAGER RATIFIED — Ladder/Fixture Manager (`keen-crab-361`), 2026-05-30. Operator sign-off: PR #3938 merged (`b129ce3f2`); §7 Phase 1 dispatch ratified per PM 2026-05-30.
> **Authority:** PR #3938 §11.1 lane 2. **9-rung ontology:** PR #3938 §6 (`docs/planning/v4-correctness-ladder-2026-05-30.md` on `session/nimble-dove-733`).
> **PR:** https://github.com/gunb-ai/gunbc/pull/3946 (`session/keen-crab-361`).
> **Line-number authority:** `origin/main` at `9cc2392cc` (2026-05-30). Re-verify with `git show origin/main:<path>` before dispatch if `main` advances.
> **Scope:** Ratify Phase 1 fixture; acceptance predicates for rungs 0–2 only. Rungs 3–9 blocked on Compiler Spine + Runtime/TestClaim interface definitions (§11.4 item 4).

---

## 0. Nine-rung ladder (reference only)

| Rung | Property | Phase in §7 |
| ---- | -------- | ----------- |
| 0 | Parses in target | 1 |
| 1 | Type-checks in Rust | 1 |
| 2 | Compiles in committed multi-target smoke set | 1 |
| 3 | Round-trip preserved | 2 |
| 4 | Emit runs; output matches `.dag` eval | 2 |
| 5 | Cross-target equivalence | 3 |
| 6 | Algebraic laws preserved post-emit | 3 |
| 7 | Self-emit fixpoint | 5+ |
| 8 | TestClaim corpus executes | 5+ |
| 9 | Lenses gate PRs | 5+ |

This file operationalizes **rungs 0–2** on the Phase 1 fixture only. Do not infer predicates for rungs 3–9 here.

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
| `R2-python-compile` | `python` | Emitted Python compiles / type-checks per project Python gate policy (`python3 -m py_compile` or equivalent). | `phase1/nat_semiring/rung2/python_compile_failed` |
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

**Output path (net-new — no prior rung-gate module exists):**

- `src/v4/test/claim/nat_semiring/rung_0_to_2_three_targets.dag`

**Structural templates (compose, do not copy blindly)** — paths and line ranges verified on `origin/main` at `9cc2392cc`:

| Pattern | File (line range) | Use for |
| ------- | ----------------- | ------- |
| Fixture import + drift guard | `src/v4/test/claim/round_trip/dag_ingest_round_trip.dag:57-62` | Comment + `import` binding pattern (`:62`); substitute `v4.test.claim.algebra_laws.nat_semiring { … }` for Phase 1 |
| `CompilesClaim` row | `src/v4/test/claim/self_host/claim_runner_compiles.dag:35-40` | Per-target compile claims; note `classification: TestClassification { tier: Tier1, layer: Integration }` — `Integration` is `TestgenLayer`, not a `TestClaim` variant |
| Fixture `List<TestClaim>` roster | `src/v4/test/claim/workflow/affected_set_ci_runner.dag:99-103` | Named roster for gate selection |
| Law-row `EqualsClaim` | `src/v4/test/claim/algebra_laws/nat_semiring.dag:112-166` | Fixture claim exports to import |

Minimum claim shapes (`v4.std.verification` — assertion coproduct is `CompilesClaim` | `DiagnosticClaim` | `EqualsClaim` | `RoundTripClaim`; `Integration` is only `TestgenLayer` inside `TestClassification`):

| Rung | Suggested shape | Notes |
| ---- | ----------------- | ----- |
| 0–1 | `CompilesClaim` per target slice | `input` = fixture law-subject or module anchor node; `classification: TestClassification { tier: Tier1, layer: Unit }` unless integration-tier receipt is intentional |
| 2 | Three `CompilesClaim` rows (rust / python / go) | Encode target in `label` (`fixture=… rung=2 target=rust predicate=R2-rust-compile`, etc.); optional `layer: Integration` on all three if tiering matches `claim_runner_compiles.dag:40` |

Execution may remain `Deferred` behind T-38 **only** for substrate claims; the **CI gate** must still invoke toolchain checks directly until `TestClaimRun` verdicts are PROVEN (`src/v4/compiler/05_eval.dag:1732-1736` — `RoundTripClaim` arm returns `Deferred`).

**CI wiring (where the matrix lands):**

| Layer | File | Role |
| ----- | ---- | ---- |
| Authority (jobs/gates) | `src/v4/workflow/ci.dag` | Add a **new** `CiJob` + `CiGate` for `phase1/nat_semiring` rungs 0–2 — do not repurpose `M1RustEmitProbeCommand` (corpus-wide, diagnostic) or `TestClaimCorpusEvalCommand` (full corpus T-38). |
| GHA projection | `dsl/gunbc/ci_github_actions_workflow.dag` | New step/job projecting the modeled command. |
| Interim host script (if needed) | **New** `scripts/v4-phase1-nat-semiring-rung-gate.sh` | Fixture-scoped emit+toolchain; pattern from `scripts/v4-m1-rust-emit-probe.sh` but **single module path**, not full `src/v4`. |

**Worker brief triple (required on your PR):**

```text
fixture=phase1/nat_semiring
rung=0|1|2
modeling_gap=none for pure wiring; SG-1|SG-2 only with Modeling DFS worksheet approval
predicate=<R0-*|R1-*|R2-*> expected to flip
```

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

**Operator sign-off (2026-05-30):** PR #3938 merged; §7 Phase 1 dispatch ratified. Wiring PRs (#3953 substrate, ci.dag follow-up) may proceed to review. SG-1/SG-2 substrate touches still require Modeling DFS worksheet approval per §11.4.

---

## 5. Spot-check receipts (planning doc vs `origin/main` @ `9cc2392cc`)

Verified 2026-05-30 with `git cat-file -e origin/main:<path>` and `git show origin/main:<path>` (file sizes: `05_eval.dag` 1869 lines; not the stale 1119-line tree).

| Planning claim | Spot-check | Result |
| ---------------- | ---------- | ------ |
| `nat_semiring.dag` in corpus with algebra laws | `src/v4/test/claim/algebra_laws/nat_semiring.dag:1-166` | **CONFIRMED** — 6 `EqualsClaim` + 1 `DiagnosticClaim`; T-14, eval deferred comment at :3-4 |
| Round-trip eval deferred (rung 3) | `src/v4/compiler/05_eval.dag:1732-1736` | **CONFIRMED** — `RoundTripClaim { … } => Deferred { … }` |
| TestClaim corpus runner gated T-38 | `src/v4/test/claim/workflow/testclaim_corpus_runner.dag:1-4` | **CONFIRMED** — wedge; T-38 gated comment at :4 |
| CI: corpus rust probe, not fixture matrix | `src/v4/workflow/ci.dag:256-257` `M1RustEmitProbeCommand` job; header :5 T-38 note | **CONFIRMED** on `9cc2392cc` — no `phase1_nat_semiring` job (lands via #3955) |
| No Python/Go compile gate on v4 CI | `src/v4/workflow/ci.dag` — no `python`/`go` toolchain symbols | **CONFIRMED** — rung 2 gap matches planning §3 |
| Laws tested on model only, not post-emit (rung 6) | `nat_semiring.dag` uses `EqualsClaim` on Node subjects, not emitted code | **CONFIRMED** — aligns planning §9.3 row |
| Template paths in §2.5 exist on `main` | `dag_ingest_round_trip.dag`, `affected_set_ci_runner.dag`, `testclaim_corpus_runner.dag` | **CONFIRMED** on `9cc2392cc` |

---

## 6. Current baseline (as of ratification)

| Rung | Expected baseline on `main` | Notes |
| ---- | --------------------------- | ----- |
| 0 `dag` | **PASS** (module committed and parseable) | Receipt: file present + v4 parse in dev workflow. |
| 0 `rust/python/go` emit | **FAIL** (`GAP`) | Emit + parse not gated on fixture. |
| 1 | **FAIL** (`GAP`) | Corpus blocked; fixture-specific path unproven. |
| 2 | **FAIL** (`GAP`) | CI invokes Rust only; no Python/Go fixture gate. |

---

## 7. Manager sign-off

| Decision | Disposition |
| -------- | ----------- |
| Phase 1 fixture = `nat_semiring` | **RATIFIED** |
| Rungs 0–2 acceptance predicates (§2) | **RATIFIED** |
| Phase 1 target set = dag + rust + python + go | **RATIFIED** |
| No rustc-clean headline rule (§4) | **RATIFIED** |
| Worker wiring PR (#3953 + ci.dag follow-up) | **UNBLOCKED** (2026-05-30 PM) |
