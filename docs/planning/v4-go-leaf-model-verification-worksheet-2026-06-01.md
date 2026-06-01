# v4 Go Leaf-Model Verification Worksheet — R1 / R2a / R2b / R3-external

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Post-#4149 reconciliation 2026-06-01 (`zesty-otter-480`): R2b `go_facts_int64` anchor (fixtures use `int64`, not platform `int`).
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.8.2 (Go L0); MW-D3 Go parity analog of Python #4117.
> **Pattern authority:** `src/v4/test/claim/language_model/python_{r1,r2a,r2b,r3_external}.dag` + `src/v4/lens/leaf_model_verification.dag` (landed #4117).
> **Model authority:** `src/v4/extdeps/languages/go.dag` (T-4.17 wave-2b; `go_surface_spelling_int`, `go_integer_algebra_inhabitance`, overflow facts).
> **Prerequisite for R3-external impl:** SG-1 Go worksheet ratified + `go.dag` lands **`go_target_atom_realization_symbol`** row and **`go_atom_realization_symbol`** fact_id (Rust dual-name pattern; R3 claim anchors fact_id only).

---

## Mechanical dispatch rule

> **No Go leaf-model implementation worker may land until this worksheet is Modeling DFS Arbiter–approved.**

Acceptance is dual-path verdict receipts (`happy` + `falsification`) per claim, not `go build` error-count reduction on full corpus.

**Verdict authority split (Practice 11 — not one rule for all claims):**

- **Compile-bound claims (R1, R2a, R3-external):** `go build` happy ACCEPTED + falsification REJECTED with modeled `target_diagnostic_go_*` on the existing **`TargetCompileVerdict`** carrier; toolchain is data on `TargetInvocation.toolchain` (`leaf_model_toolchain_go_build`), not a Go-named verdict type.
- **Runtime-bound claim (R2b only):** `go run` / `go test` runtime assert happy ACCEPTED + negative runtime probe; compile phase stays ACCEPTED for the same source (mirror `python_r2b`). Uses **`TargetRuntimeExerciseVerdict`** (target-agnostic runtime authority) — not `TargetGoCompileVerdict`.

**R3-internal (emit coupling receipt) is explicitly out of scope for Go L0** — same posture as Python #4117 (R3-external only). Rust retains R3-internal per `docs/planning/v4-leaf-model-verification-2026-05-30.md` §7.

---

## §10.0-adapted worksheet

```text
Migration class:        GO-LEAFMODEL-PHASE1 (go.dag R1 + R2a + R2b + R3-external)
Representative failure:  go.dag declares int surface spelling / algebra / overflow / Symbol projection
                         facts with zero external exercise — silent model drift (7951-class at emit).
Immediate local patch:   Hand-edit emitted Go in 06_translate or add Rust-only test asserting go.dag strings.
Why forbidden:           Parallel authority; does not falsify model claims; P2 violation vs extdeps facts.
DFS path:
  std/ authority:
    - LeafModelClaimId + fixture carriers at src/v4/std/leaf_model_verification.dag
    - Phase 1 verdict carriers today: TargetCompileVerdict (compile authority) and
      TargetPythonExerciseVerdict (CPython compile+exec split). Go MUST NOT add
      TargetGoCompileVerdict (Practice 11 — target-named duplicate of compile verdict).
    - 🟡 gated — feature:leaf-model-verdict-parameterization — bind Modeling DFS / T-22 —
      dissolve-on-arrival: TargetLeafModelToolchainVerdict subsumes TargetCompileVerdict +
      TargetPythonExerciseVerdict + TargetRuntimeExerciseVerdict; Go rows become call sites only.
    - Interim (this wave): (a) R1/R2a/R3-external consume TargetCompileVerdict + go diagnostic
      Symbol rows + TargetInvocation.toolchain = leaf_model_toolchain_go_build; (b) R2b consumes
      additive TargetRuntimeExerciseVerdict { Accepted | Rejected{diagnostic_code} } (runtime
      authority only — not a third target-named compile carrier).
  extdeps/language authority:
    - go.dag: go_surface_spelling_int, go_facts_int, go_integer_algebra_inhabitance(go_facts_int),
      go_facts_int64 + go_tag_overflow_signed_truncates on that bundle (R2b only — fixtures use int64),
      (R3) go_atom_realization_symbol fact_id → go_target_atom_realization_symbol row (SG-1 Go worksheet)
  claim corpus (sibling files, per planning §4 D-LMV-4 option b):
    - src/v4/test/claim/language_model/go_r1.dag
    - src/v4/test/claim/language_model/go_r2a.dag
    - src/v4/test/claim/language_model/go_r2b.dag
    - src/v4/test/claim/language_model/go_r3_external.dag
  lens fixtures:
    - src/v4/lens/leaf_model_verification.dag — go_r*_fixture* + fixture sources (mirror python_*)
  host runner (dissolve-on-arrival per python shells — required Phase 1):
    - scripts/v4-leaf-model-go-{r1,r2a,r2b,r3-external}-verify.sh
  host bridge v3 hand-Rust (forbidden unless §5 P5(b) receipt bundle lands in same PR):
    - src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_test.rs — NOT optional; omit entirely
      if worker cannot supply SG-0 pairing + ROADMAP deferral (see §5)
Deepest unsound boundary:
  No LeafModelClaimId rows or Go toolchain call sites on existing verdict carriers; Go zero.
Systemic fix:
  (1) Add target_diagnostic_go_* + leaf_model_toolchain_go_build; R1/R2a/R3-external on
      TargetCompileVerdict; R2b on TargetRuntimeExerciseVerdict (see §4 split).
  (2) Add four LeafModelClaimId data rows + lens fixture pairs + claim .dag files.
  (3) Host runners: go build (compile-bound) or go run/test (R2b only) per §4.
Non-goals:
  - R3-internal dual-emit mutation receipt (Rust-only L0 closure).
  - Full ~50-claim go.dag inventory (Phase 2 widening).
  - Complex128 algebra inhabitance (still 🟡 withheld in go.dag header).
  - Replacing T-22 modeled runner in this PR (shell bridge allowed; dissolve header required).
Falsification probe (split by authority — do not apply compile rule to R2b):
  Compile-bound (R1, R2a, R3-external): happy go build ACCEPTED; falsification go build REJECTED
    with modeled target_diagnostic_go_* on TargetCompileVerdict.
  Runtime-bound (R2b only): happy go run/test ACCEPTED when runtime assert matches overflow model;
    negative probe is wrong expected wrap value at runtime (compile stays ACCEPTED for same source).
Metric allowed only as secondary:
  Weather/nat_semiring go build health post-#4076 — evidence only, not acceptance.
```

---

## §4 Per-claim fixture contract (Go-specific)

### §4.1 Compile-bound claims (`TargetCompileVerdict` + `leaf_model_toolchain_go_build`)

| Claim ID | Fact anchor (`go.dag`) | Happy | Falsification | Expected falsification |
|---|---|---|---|---|
| `leaf_model_claim_go_r1_int_surface_spelling` | `go_surface_spelling_int` | `go build` PASS (`func r1() int { return 0 }`) | return type `i32` (invalid) | `TargetCompileRejected` + `target_diagnostic_go_undefined_type` |
| `leaf_model_claim_go_r2a_int_algebra_operations` | `go_integer_algebra_inhabitance(go_facts_int)` | `go build` PASS (add/compare on `int`) | call nonexistent method on `int` | `TargetCompileRejected` + `target_diagnostic_go_undefined_method` |
| `leaf_model_claim_go_r3_external_symbol_projection` | `go_atom_realization_symbol` (fact_id; row = `go_target_atom_realization_symbol`) | `go build` PASS per realization row | ctor/signature mismatch | `TargetCompileRejected` + modeled go diagnostic |

### §4.2 Runtime-bound claim (`TargetRuntimeExerciseVerdict` — R2b only)

Go spec ([Integer overflow](https://go.dev/ref/spec#Integer_overflow)): overflow is a **typed integer operation**, not an untyped constant fold. Fixtures MUST bind `math.MaxInt64` into a typed variable before `+ 1` so compile stays `TargetCompileAccepted` and only runtime exercises the wrap story.

**Authoritative fixture sources (lens `go_r2b_*_fixture_source` must match verbatim):**

Happy (`go_r2b_happy_fixture_source`):

```go
package main

import "math"

func main() {
    var x int64 = math.MaxInt64
    got := x + 1
    want := int64(math.MinInt64)
    if got != want {
        panic("silent signed wrap: expected MinInt64")
    }
}
```

Negative probe (`go_r2b_falsification_fixture_source`) — **same typed source shape**; only `want` changes:

```go
package main

import "math"

func main() {
    var x int64 = math.MaxInt64
    got := x + 1
    want := int64(0)
    if got != want {
        panic("deliberately wrong expected wrap value")
    }
}
```

| Claim ID | Fact anchor | Happy | Negative probe | Verdict carrier |
|---|---|---|---|---|
| `leaf_model_claim_go_r2b_int64_silent_overflow_truncates` | `go_facts_int64` bundle — overflow axis `go_tag_overflow_signed_truncates` via `go_integer_facts_node(facts: go_facts_int64)` (same disposition as other signed typed widths; **not** `go_facts_int` / platform `int`) | `go run` on happy source above → wrap matches `want` | `go run` on falsification source → panic (wrong `want` only) | Runtime: `Accepted` / `Rejected`; compile: `TargetCompileAccepted` for **both** sources |

**Forbidden R2b shapes:** `math.MaxInt64 + 1` as a single constant expression (may fail compile or fold at compile time); negative probe that changes the `+ 1` expression instead of `want`.

**R2b note:** Go models **silent truncation** on typed integer overflow (`go.dag` L181). Claim ↔ fixture width must match: **`go_facts_int64`** + `int64` source (R2a remains **`go_facts_int`** / platform `int`). Not Rust debug/release dual claim IDs unless Arbiter directs.

**Verifiers:** `go build` — R1/R2a/R3-external only. `go run` or `go test` — R2b only. `go vet` optional, not authority.

---

## §5 P5(b) / Pure Bootstrap — v3 hand-Rust bridge (if used)

Per `INVARIANTS.md` P5 and THESIS Pure Bootstrap 0-floor: any planned `src/v3/compiler/tests/boundary/*.rs` bridge carries **exactly one** checkable receipt. Shell-only Phase 1 is valid (mirror early lanes); adding hand-Rust is **forbidden** without all items below in the **same** implementation PR.

| Requirement | Authority |
|---|---|
| **SG-0 pairing** | PR body: `(c) Net +1 on EXPECTED_HAND_AUTHORED_TEST` for `v4_leaf_model_go_r1_test.rs` with one-line Phase 1 leaf-model Go R1 rationale (same shape as #3972 / #4022 append files under `scripts/ci-merge/`) |
| **Deleted-scaffold** | N/A at first landing; bridge is net-new with dissolve header only |
| **SG-0 shrink** | No shrink claimed on landing; retirement PR must pair delete with shrink or deferral row update |
| **Lane + ROADMAP deferral** | Dissolve-on-arrival trigger names T-22 `run_target_verification` for `go_r1.dag`; retirement owner **T-PB-B** `pb_rust_tests_outside_residual_zero` in `_internal/ROADMAP_OPS.md` |
| **integration.rs** | `#[path = "boundary/v4_leaf_model_go_r1_test.rs"]` mod registration (same as `v4_leaf_model_python_r1_test.rs`) |
| **Pairs with** | `scripts/v4-leaf-model-go-r1-verify.sh` — single host-transport story, not duplicate authority |

**Default dispatch (this worksheet):** Phase 1 Go leaf-model workers land **shell runners only** unless Arbiter explicitly authorizes the P5(b) bundle above. Do not author `v4_leaf_model_go_r1_test.rs` as an undocumented optional extra.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] **Shared verdict carriers:** compile-bound on `TargetCompileVerdict` + `target_diagnostic_go_*` + `leaf_model_toolchain_go_build`; R2b on additive shared `TargetRuntimeExerciseVerdict` — **no** `TargetGoCompileVerdict`
- [x] Four `LeafModelClaimId` symbols named and non-colliding with rust/python
- [x] R3-external blocked until Go SG-1 atom row lands (dependency accepted)
- [x] R3-internal explicitly deferred for Go L0
- [x] Host runner dissolve-on-arrival headers match python shell pattern
- [x] P5(b) boundary Rust optional per §5 table — shell-only default unless worker ships full receipt bundle
- [x] R2b fixtures use typed `var x int64 = math.MaxInt64; got := x + 1` (authoritative sources in §4.2)
- [x] R2b claim anchors `go_facts_int64`, not `go_facts_int` (width-aligned with fixtures; post-#4149 reconciliation)
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

---

## Related artifacts

- `docs/planning/v4-leaf-model-verification-2026-05-30.md` — framework §5–§7
- PR **#4117** — Python R1+R2a+R2b+R3-external landed pattern
- PR **#4076** — Go emit layout (prerequisite for L1; orthogonal to leaf-model claims)
- `docs/planning/v4-go-leaf-model-ci-runner-worksheet-2026-06-01.md` — CI wiring sibling
