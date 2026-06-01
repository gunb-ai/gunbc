# v4 Go Leaf-Model Verification Worksheet — R1 / R2a / R2b / R3-external

> **Status:** **ready-for-review** — Go RCA Manager (`gentle-lynx-68`); ratification route: Modeling DFS Arbiter `proud-fox-405` (§10.0 single-authority-fact).
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.8.2 (Go L0); MW-D3 Go parity analog of Python #4117.
> **Pattern authority:** `src/v4/test/claim/language_model/python_{r1,r2a,r2b,r3_external}.dag` + `src/v4/lens/leaf_model_verification.dag` (landed #4117).
> **Model authority:** `src/v4/extdeps/languages/go.dag` (T-4.17 wave-2b; `go_surface_spelling_int`, `go_integer_algebra_inhabitance`, overflow facts).
> **Prerequisite for R3-external impl:** Go `TargetAtomRealization` rows worksheet (`docs/planning/v4-go-target-atom-realization-worksheet-2026-06-01.md`) ratified + `go_atom_realization_symbol` fact declared in `go.dag`.

---

## Mechanical dispatch rule

> **No Go leaf-model implementation worker may land until this worksheet is Modeling DFS Arbiter–approved.**

Acceptance is dual-path verdict receipts (`happy` + `falsification`) per claim, not `go build` error-count reduction on full corpus.

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
    - Phase 1 has TargetCompileVerdict (rustc) and TargetPythonExerciseVerdict only —
      Go needs additive TargetGoCompileVerdict (+ go diagnostic Symbol rows) in SAME PR as
      first Go claim, not a parallel verdict vocabulary elsewhere.
  extdeps/language authority:
    - go.dag: go_surface_spelling_int, go_facts_int, go_integer_algebra_inhabitance(go_facts_int),
      go_tag_overflow_signed_truncates, (R3) go_atom_realization_symbol — to be added per SG-1 Go worksheet
  claim corpus (sibling files, per planning §4 D-LMV-4 option b):
    - src/v4/test/claim/language_model/go_r1.dag
    - src/v4/test/claim/language_model/go_r2a.dag
    - src/v4/test/claim/language_model/go_r2b.dag
    - src/v4/test/claim/language_model/go_r3_external.dag
  lens fixtures:
    - src/v4/lens/leaf_model_verification.dag — go_r*_fixture* + fixture sources (mirror python_*)
  host runner (dissolve-on-arrival per python shells):
    - scripts/v4-leaf-model-go-{r1,r2a,r2b,r3-external}-verify.sh
    - optional boundary test src/v3/compiler/tests/boundary/v4_leaf_model_go_r1_test.rs
Deepest unsound boundary:
  No LeafModelClaimId rows or TargetGo verdict carrier for Go; Python/Rust Phase 1 proven, Go zero.
Systemic fix:
  (1) Extend std leaf_model_verification with TargetGoCompileVerdict { Accepted | Rejected{code} }
      and target_diagnostic_go_* for R1/R2a/R2b/R3-external probes below.
  (2) Add four LeafModelClaimId data rows + lens fixture pairs + claim .dag files.
  (3) Host runners invoke `go build` on minimal module.main per fixture (see §4).
Non-goals:
  - R3-internal dual-emit mutation receipt (Rust-only L0 closure).
  - Full ~50-claim go.dag inventory (Phase 2 widening).
  - Complex128 algebra inhabitance (still 🟡 withheld in go.dag header).
  - Replacing T-22 modeled runner in this PR (shell bridge allowed; dissolve header required).
Falsification probe (per claim):
  Each claim: happy `go build` PASS + falsification `go build` FAIL with modeled diagnostic code.
Metric allowed only as secondary:
  Weather/nat_semiring go build health post-#4076 — evidence only, not acceptance.
```

---

## §4 Per-claim fixture contract (Go-specific)

| Claim ID | Fact anchor (`go.dag`) | Happy fixture (sketch) | Falsification fixture | Expected falsification diagnostic |
|---|---|---|---|---|
| `leaf_model_claim_go_r1_int_surface_spelling` | `go_surface_spelling_int` | `package main; func r1() int { return 0 }` | return type `i32` (invalid) | `undefined: i32` (compile error) |
| `leaf_model_claim_go_r2a_int_algebra_operations` | `go_integer_algebra_inhabitance(go_facts_int)` | `func r2a(a,b int) (int,bool) { return a+b, a<b }` | call nonexistent method on `int` | method/operator not defined |
| `leaf_model_claim_go_r2b_int_silent_overflow_truncates` | `go_tag_overflow_signed_truncates` on `go_facts_int` | runtime `math.MaxInt64+1` wrap assert in `main` | N/A — runtime-only; falsification arm documents unfalsifiable-at-compile-time (mirror python_r2b posture) | both paths `Accepted` at compile; runtime asserts overflow story |
| `leaf_model_claim_go_r3_external_symbol_projection` | `go_atom_realization_symbol` (new) | emit per realization row (nominal `type Symbol string` or struct — **row content is SG-1 Go worksheet, not invented here**) | ctor/signature mismatch vs row | compile error on bad call |

**R2b note:** Go models **silent truncation** on typed integer overflow (`go.dag` L181 comment). R2b is a **runtime** receipt (`go run` / `go test`), not debug/release profile split like Rust. Do not import Rust's dual claim IDs unless Arbiter directs.

**Verifier:** `go build` (minimum L0); R2b may add `go run` for runtime assert. `go vet` is optional secondary, not authority.

---

## §8 Arbiter approval checklist (`proud-fox-405`)

- [ ] `TargetGoCompileVerdict` additive to `v4.std.leaf_model_verification` (no parallel module)
- [ ] Four `LeafModelClaimId` symbols named and non-colliding with rust/python
- [ ] R3-external blocked until Go SG-1 atom row exists (cross-worksheet dependency accepted)
- [ ] R3-internal explicitly deferred for Go L0
- [ ] Host runner dissolve-on-arrival headers match python shell pattern

**State after author:** `ready-for-review`

---

## Related artifacts

- `docs/planning/v4-leaf-model-verification-2026-05-30.md` — framework §5–§7
- PR **#4117** — Python R1+R2a+R2b+R3-external landed pattern
- PR **#4076** — Go emit layout (prerequisite for L1; orthogonal to leaf-model claims)
- `docs/planning/v4-go-leaf-model-ci-runner-worksheet-2026-06-01.md` — CI wiring sibling
