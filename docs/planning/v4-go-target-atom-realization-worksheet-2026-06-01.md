# v4 Go TargetAtomRealization Worksheet — Symbol / Bool / Char / Int

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Post-#4149 reconciliation 2026-06-01 (`zesty-otter-480`): SG-1 §3.1 dual-name (`go_target_atom_realization_*` row + `go_atom_realization_*` fact_id). Implementation: Go RCA Manager subtree (#4137 per-language lane).
> **Date:** 2026-06-01
> **Dispatch anchor:** SG-1 analog — `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md` (APPROVED); `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.8.
> **Canonical home:** `src/v4/std/target_model.dag` (`TargetAtomRealization` carrier — **do not redefine**).
> **Language rows:** `src/v4/extdeps/languages/go.dag` (additive catalog only).

---

## Mechanical dispatch rule

> **No Go TargetAtomRealization implementation worker may land until:**
> 1. This worksheet is Arbiter-approved, **and**
> 2. Shared SG-1 carrier on main is consumed (not forked), **and**
> 3. Go leaf-model R3-external dependency satisfied when **both** land in same wave (Rust dual-name pattern — one authority, two symbols):
>    - `go_target_atom_realization_symbol` — `TargetAtomRealization` row in `go.dag`
>    - `go_atom_realization_symbol` — leaf-model fact_id `Symbol` naming that row (R3-external claim anchor)

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
  Add go_atom_realization_symbol fact_id per rust.dag:870-871 (leaf-model claims reference fact_id,
  not the TargetAtomRealization data binding name — P2 single row, two named projections).
  Int row uses go_surface_spelling_int / go facts — NOT a separate parallel Int carrier name.
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

## §3.1 Naming convention (Rust pattern — not a fork)

| Symbol | Role | Consumer |
|---|---|---|
| `go_target_atom_realization_symbol` | `TargetAtomRealization` row (`data` binding) | `target_model` catalog edge; emit reads row fields |
| `go_atom_realization_symbol` | fact_id `Symbol` naming that row | `go_r3_external.dag` / lens fixture `surface_spelling_fact` |

Python landed R3 with only `python_atom_realization_symbol` (minimal scaffold). Go follows **full Rust SG-1** (row + fact_id). Leaf-model worksheets reference **fact_id only**; SG-1 worksheet authors **row + fact_id** together.

---

## §4 Proposed Go rows (sketch — impl worker fills exact spellings)

| Kernel atom | `source_carrier` | `type_form` (sketch) | `value_form` (sketch) | Notes |
|---|---|---|---|---|
| Symbol | `symbol_kernel_type_node()` | `type Symbol string` or named string alias per Arbiter | string literal / `Symbol(...)` per row | Must match R3-external happy fixture |
| Bool | `bool_node()` | `bool` | `true` / `false` | `go_surface_spelling_bool` |
| Char | `char_kernel_type_node()` | `rune` | rune literal | Go has no `char` type |
| Int (platform) | `go_facts_int` inhabitant node | `int` | integer literal | Pairs with R1 claim; do not conflate with `int32` fixed width in atom row unless Arbiter splits |

**Dispatch:** Go RCA Manager (`gentle-lynx-68`) spawns implementation worker under own subtree after §8. Arbiter owns shared `TargetAtomRealization` carrier only; Go RCA owns `go.dag` rows + emit consumption.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Reuses SG-1 approved `TargetAtomRealization` carrier (no Go-local duplicate)
- [x] Char → `rune` mapping explicit (Go spec)
- [x] Int row authority is `go_facts_int`, not rust `i32` spelling
- [x] SG-1 §3.1 dual-name: `go_target_atom_realization_symbol` row + `go_atom_realization_symbol` fact_id (post-#4149 reconciliation)
- [x] Go RCA Manager owns implementation dispatch after §8 (not parallel TR lane)
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

---

## Related artifacts

- `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md`
- `docs/design-target-realization-canonical-home.md` §3 scaffold disposition
- `docs/planning/v4-go-leaf-model-verification-worksheet-2026-06-01.md` — R3-external dependency
