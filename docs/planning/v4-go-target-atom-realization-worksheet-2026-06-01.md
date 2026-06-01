# v4 Go TargetAtomRealization Worksheet — Symbol / Bool / Char / Int

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Implementation lane: TR Manager `keen-heron-687`.
> **Date:** 2026-06-01
> **Dispatch anchor:** SG-1 analog — `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md` (APPROVED); `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.8.
> **Canonical home:** `src/v4/std/target_model.dag` (`TargetAtomRealization` carrier — **do not redefine**).
> **Language rows:** `src/v4/extdeps/languages/go.dag` (additive catalog only).

---

## Mechanical dispatch rule

> **No Go TargetAtomRealization implementation worker may land until:**
> 1. This worksheet is Arbiter-approved, **and**
> 2. Shared SG-1 carrier on main is consumed (not forked), **and**
> 3. Go leaf-model R3-external worksheet dependency on `go_atom_realization_symbol` is satisfied in the same impl wave or ordered before R3-external runner.

Acceptance: dual-path falsification on Symbol row changes **both** type and value translate paths (SG-1 §10.6 pattern), exercised on **Go emit** where available; rustc probe is not Go acceptance.

---

## §10.0-adapted worksheet

```text
SG class:               GO-SG-1-ANALOG (per-language TargetAtomRealization rows)
Shared authority:       v4.std.target_model::TargetAtomRealization (SG-1 APPROVED 2026-05-30)
Representative failure:  Go emit uses string heuristics for std/node.dag::Symbol / Bool / Char / Int
                         — type and value paths disagree; weather Go packages fail typecheck post-#4076.
Immediate local patch:   Special-case Symbol → string in 05_emit_go / runtime_go template.
Why forbidden:           P2 violation; calcifies per-template knowledge; blocks L1 self-compile slice.
DFS path:
  std/ authority:
    - Symbol kernel at src/v4/std/node.dag:10
    - TargetAtomRealization carrier + target_atom_realization_catalog_node at target_model.dag
  extdeps/language authority:
    - go.dag: NO TargetAtomRealization rows today (2026-06-01 spot-check)
    - rust.dag: rust_target_atom_realization_{symbol,bool,char} — pattern reference only
    - python.dag: python_atom_realization_symbol (minimal R3 scaffold — not full catalog)
  compiler stage:
    - v2 05_emit_go.dag + v3 emit Go path (post-#4076 layout) — consumes facts indirectly today
Deepest unsound boundary:
  Missing Go rows on shared TargetAtomRealization carrier; emit derives Go atom shapes ad hoc.
Systemic fix:
  Add go_target_atom_realization_{symbol,bool,char,int} rows + catalog edge on go MVP target_model
  (mirror rust_target_atom_realization_catalog wiring).
  Int row uses go_surface_spelling_int / go facts — NOT a separate parallel Int carrier name.
  Phase 1 (#4168) lands symbol/bool/char only — see §9 Int deferral (shared carrier lacks integer value template).
Non-goals:
  - Redefining TargetAtomRealization fields (Arbiter escalation if shape insufficient).
  - loop_bound_edge / Symbol-tagged Loop dissolution (T-12).
  - Deleting go primitive fact bundles.
  - Rust-only falsification as Go PASS substitute.
Falsification probe:
  Mutate go_target_atom_realization_symbol.type_form; re-emit Go for a minimal Symbol-valued
  workflow fixture; verify BOTH type-position and value-position emitted Go change together.
  Grep emit templates for string-keyed "Symbol" Go projection — expect zero after refactor.
Metric allowed only as secondary:
  Post-#4076 go build failures on weather/nat_semiring — evidence only.
```

---

## §4 Proposed Go rows (sketch — impl worker fills exact spellings)

| Kernel atom | `source_carrier` | `type_form` (sketch) | `value_form` (sketch) | Notes |
|---|---|---|---|---|
| Symbol | `symbol_kernel_type_node()` | `type Symbol string` or named string alias per Arbiter | string literal / `Symbol(...)` per row | Must match R3-external happy fixture |
| Bool | `bool_node()` | `bool` | `true` / `false` | `go_surface_spelling_bool` |
| Char | `char_kernel_type_node()` | `rune` | rune literal | Go has no `char` type |
| Int (platform) | `go_facts_int` inhabitant node | `int` | integer literal | Pairs with R1 claim; do not conflate with `int32` fixed width in atom row unless Arbiter splits |

**Coordination:** TR Manager (`keen-heron-687`) owns implementation after §8. Go RCA Manager owns worksheet + fact IDs; does not land emit patches without ratified worksheet.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Reuses SG-1 approved `TargetAtomRealization` carrier (no Go-local duplicate)
- [x] Char → `rune` mapping explicit (Go spec)
- [x] Int row authority is `go_facts_int`, not rust `i32` spelling
- [x] TR handoff: `keen-heron-687` owns rows after §8; Go RCA does not land emit patches
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

---

## §9 Implementation scope — phase 1 (#4168)

**Shipped:** `go_target_atom_realization_{symbol,bool,char}` + `target_model_edge_atom_realizations` on live `go_mvp1_target_model_node`; staging `host_bundle` excludes catalog edge (rust SG-1 pattern); `go_mvp1_binding_spellings` maps `go_surface_spelling_{string,bool,rune}` → `"string"` / `"bool"` / `"rune"` for `06_translate` `map_get`.

**Deferred (fail-closed):** `go_target_atom_realization_int` — `TargetValueTemplateKind` at `src/v4/std/target_model.dag:106-110` has Symbol/Bool/Char arms only; no integer literal template exists. Using `ValueSymbolIdentityPassthrough` for platform `int` would author false target-model facts (INVARIANTS P1/P3). Gate: `feature:go-target-atom-realization-int` in `go.dag`; dissolve-on: shared carrier adds integer `TargetValueTemplateKind` + value path. Until then platform Int R1 authority stays `go_facts_int` / `go_surface_spelling_int` (§4).

**P5 / hand-Rust:** Go row smoke is same-path expansion in `v4_std_target_realization_dag_smoke_test.rs` (+0 `EXPECTED_HAND_AUTHORED_TEST` paths; INVARIANTS_OPS + census row pre-exist on `main`).

---

## Related artifacts

- `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md`
- `docs/design-target-realization-canonical-home.md` §3 scaffold disposition
- `docs/planning/v4-go-leaf-model-verification-worksheet-2026-06-01.md` — R3-external dependency
