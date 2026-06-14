# v4 Go L1 Compiler-Slice Compile Worksheet — emit subset `go build` receipt

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). **Slice id:** `go_l1_nat_semiring_rung2` (post-L0 PROVEN).
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.8.1 rung **L1**; Go row target "L0 complete + **L1 fixture-scale**".
> **Prerequisites:** #4076 merged; L0 leaf-model claims PROVEN (`docs/planning/v4-go-leaf-model-verification-worksheet-2026-06-01.md`); Go SG-1/SG-2 rows landed for atoms/generics touched by slice.

> **Manager closeout:** the L0 R1/R2 prerequisites and the L2 fixture-execution
> sequence are split out in `docs/planning/v4-go-rca-manager-worksheets-2026-06-03.md`.

---

## Mechanical dispatch rule

> **L1 is NOT full v4 compiler self-compile (L4).** Acceptance is: a **named, bounded compiler subset** emits to Go and **`go build` succeeds** on that artifact with a structured receipt.

---

## §10.0-adapted worksheet

```text
Migration class:        GO-L1-COMPILER-SLICE-COMPILE
Representative failure:  #4076 fixed module layout for weather/nat_semiring but no receipt that a
                         declared v4 compiler-subset emits and typechecks as Go under go build.
Immediate local patch:   Manual go build of hand-patched emitted tree in PR description.
Why forbidden:           Non-reproducible; no FileSet/affected gate; not L1 rung receipt.
DFS path:
  fixture authority:
    - phase1/nat_semiring rungs (docs/planning/v4-ladder-rung-specs-2026-05-30.md R2-go-compile)
    - OR new minimal go_compiler_slice fixture (Arbiter picks one — prefer existing nat_semiring
      if post-#4076 green, else smallest add.dag-class slice)
  emit path:
    - v2→v4 Go emit for bounded ModuleGraph (not full 332-source closure on every PR)
  verifier:
    - go build (R2-go-compile bar); go test only if slice includes tests
Deepest unsound boundary:
  No structured L1 receipt tying emitted compiler-subset → go build PASS/FAIL headline.
Systemic fix:
  (1) Name compiler_slice_go_v1 ModuleGraph / FileSet in worksheet-approved fixture list.
  (2) CiUpsertStep or ladder rung row: emit_go_compiler_slice → go_build_verdict.
  (3) Receipt JSON: { slice_id, go_module_root, verdict, diagnostic_snippet? }.
Non-goals:
  - L2 cross-target eval equivalence (separate rung / #4081 Wc analog).
  - L3 self-output fixed point.
  - Replacing full v4-bootstrap-viability.sh.
Falsification probe:
  Deliberately break go.dag surface spelling for int in model; L1 step must FAIL before emit merge.
Metric allowed only as secondary:
  Full weather package go test count — not L1 acceptance.
```

---

## §4 Proposed L1 slice candidates (Arbiter picks one)

| Slice id | Fixture | Rationale | Blocked if |
|---|---|---|---|
| `go_l1_nat_semiring_rung2` | `phase1/nat_semiring` Go emit path | Post-#4076 layout fix; ladder R2-go-compile already defined | L0 go.dag claims not PROVEN |
| `go_l1_mvp1_add` | `mvp1_go_add_translate` manual claim neighborhood | Smallest translate surface in tree | Missing atom/type_expr rows |
| `go_l1_weather_pkg` | weather Go module | Real integration weight | Too large for first L1 — fallback only |

**Recommendation (Go RCA):** start `go_l1_nat_semiring_rung2` after L0 R1 PROVEN — reuses ladder vocabulary.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Named slice id: **`go_l1_nat_semiring_rung2`** (`phase1/nat_semiring` Go emit path; ladder R2-go-compile)
- [x] Distinct from L0 leaf-model claims (structured L1 receipt JSON)
- [x] Does not claim L2/L3/L4
- [x] Gated on L0 leaf-model claims PROVEN before dispatch
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

---

## Related artifacts

- `src/v4/test/claim/nat_semiring/rung_0_to_2_three_targets.dag` — `R2-go-compile` substrate roster (in-tree)
- `scripts/v4-nat-semiring-rung-gate.sh` — Phase 1 host transport for rungs 0–2
- PR **#4076** — module layout prerequisite
- PR **#3946** — ladder rung specs (lands `docs/planning/v4-ladder-rung-specs-2026-05-30.md` on merge)
- `docs/planning/v4-go-leaf-model-verification-worksheet-2026-06-01.md` — L0 Go R1/R2 prerequisite worksheet
- `docs/planning/v4-go-rca-manager-worksheets-2026-06-03.md` — manager worksheet split for R1/R2, L1 fixture-scale, and L2 fixture-execution sequencing
