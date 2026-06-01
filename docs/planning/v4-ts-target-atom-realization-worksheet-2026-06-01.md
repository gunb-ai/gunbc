# v4 TS TargetAtomRealization Worksheet (per-language rows)

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). **Requires** TS type-expr row landed first.  
> **Lane:** ALPHA/PREVIEW — NOT release-minimum.  
> **Shared carrier authority:** `src/v4/std/target_model.dag` — `TargetAtomRealization` (SG-1 landed on main). **Do NOT redefine carrier.**  
> **Prerequisite:** `v4-ts-target-type-expression-projection-worksheet-2026-06-01.md` (`type_form` MUST consume SG-2 row).  
> **Dispatch anchor:** SG-1 Rust pattern `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md`; PM brief maps Rust {Symbol, Bool, Char} → TS surface (see §PM mapping below).

---

## Mechanical dispatch rule

> **No TS TargetAtomRealization implementation worker may land until this worksheet is Modeling DFS Arbiter–approved.**

Acceptance is dual-emit falsification for Symbol (R3-external host runner) + catalog presence on `ts_wave1_target_model` bundle — not rustc-style E0423 counts.

---

## §10.0-adapted worksheet

```text
SG class:               TS-ATOM-REALIZATION (per-language rows on landed SG-1 carrier)
Representative failure:  Kernel-ambient Symbol/Bool used in translate without typescript.dag
                         TargetAtomRealization rows — type and value paths invent TS shapes independently.
Immediate local patch:
  Hardcode "type Symbol = string" or string-literal value emit in 06_translate TS branches.
Why that patch is forbidden:
  INVARIANTS P2 — atom realization is single authority in extdeps/languages/typescript.dag;
  duplicates rust.dag / python.dag spelling tables in emitter.
DFS path:
  std/ authority:
    - TargetAtomRealization, target_atom_realization_catalog_node, bundle edge keys
      at src/v4/std/target_model.dag (consume only)
  extdeps/language authority:
    - src/v4/extdeps/languages/typescript.dag — NEW rows + catalog list on TargetModel bundle
  compiler stage:
    - src/v4/compiler/06_translate.dag — consume catalog via target_model bundle (no TS name-keyed table)
  existing scaffold:
    - python.dag: python_atom_realization_symbol is Phase-1 Symbol stub (not full row) — TS must
      land full TargetAtomRealization rows, not copy the stub shape
Deepest unsound boundary:
  Missing per-target atom realization catalog on typescript TargetModel for kernel-ambient atoms
  used in emit.
Systemic fix:
  TargetAtomRealization rows for TS (Phase 1 L0 minimum):
    - Symbol  (kernel-ambient; R3-external + SG-1 parity)
    - Bool    (ECMA boolean / ts_facts_boolean)
    - String  (UTF-16 code-unit sequence; replaces Rust Char — TS has no Char primitive)
  Optional Phase 1.5 (Arbiter decision): number | bigint as atom-backed spellings only if emit
  already routes numeric atoms through TargetAtomRealization — otherwise defer to T-4 numeric facts.
  Each row: { source_carrier, target_model, type_form, value_form, constructor_form, display_name }
  where type_form uses ts_type_expression_projection() from SG-2 worksheet.
Non-goals:
  - Redefining TargetAtomRealization carrier in typescript.dag.
  - Rust Char row copy-paste (N/A on TS surface).
  - loop_bound_edge / Symbol-tag Loop dissolution (T-12; unchanged).
  - module import path / package.json layout (separate L1 packaging worksheet).
Falsification probe:
  Mutate Symbol row type_form (e.g., alias → branded interface); re-emit type + value positions;
  BOTH must change. Grep 06_translate for string-keyed Symbol TS projection — expect zero.
  R3-external host runner: happy/falsification fixtures per leaf-model R2/R3 worksheet.
Metric allowed only as secondary:
  tsc error count on atom-using emit snippets.
```

---

## PM brief mapping (Rust SG-1 → TypeScript surface)

| PM / Rust anchor | TypeScript realization | Disposition |
|------------------|------------------------|-------------|
| Symbol | ECMA global `symbol` primitive via `Symbol(description?)` factory (NOT `new Symbol`) | **L0 required** |
| Bool | `boolean` / `ts_surface_spelling_boolean` | **L0 required** |
| Char | *N/A* — use **String** (UTF-16 code units, `ts_facts_string`) | **Substitute** (not Char) |
| Int | *Split:* kernel `Int` routing uses **number** + **bigint** facts, not a single TS `Int` | **Defer atom row** unless Arbiter batches numeric atoms |

---

## Row sketches (authoring targets — not binding code)

```text
ts_target_atom_realization_symbol:
  type_form:     symbol  (primitive) or branded alias per Arbiter — NOT a nominal class shadowing global Symbol
  value_form:    Symbol(<string-literal>)  // ECMA factory call; must match R3-external happy fixture
  constructor_form: optional_absent()  // Symbol is not a constructor (ECMA-262); forbidden: new Symbol(...)

ts_target_atom_realization_bool:
  type_form:     boolean
  value_form:    true | false literals

ts_target_atom_realization_string:
  type_form:     string
  value_form:    "<utf16 code unit sequence>"  // falsification: conflate with Symbol row
```

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Single-authority fact: rows in `typescript.dag` only; carrier in `v4.std.target_model`
- [x] `type_form` consumes TS `TargetTypeExpressionProjection` (SG-2 prerequisite — gated)
- [x] Char→String substitution documented; number/bigint split documented
- [x] No emitter template patch without row mutation
- [x] **READY-FOR-WORKER-DISPATCH** after type-expr impl (`proud-fox-405`)

## Related artifacts

- `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md`
- `docs/design-target-realization-canonical-home.md`
- `src/v4/extdeps/languages/rust.dag` — `rust_target_atom_realization_*` reference rows
- `docs/planning/v4-ts-target-type-expression-projection-worksheet-2026-06-01.md`
- `docs/planning/v4-ts-leaf-model-r2-r3-external-worksheet-2026-06-01.md` (R3-external consumer)
