# v4 SG-8 Worksheet — Module graph + carrier re-exports

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Worksheet PR [#4127](https://github.com/gunb-ai/gunbc/pull/4127) is on `main` (docs-only, same split as SG-RC [#4100](https://github.com/gunb-ai/gunbc/pull/4100)). Target Realization implementation is **authorized** — dispatch under Rust RCA Manager per #4137 §11.2.
> **Date:** 2026-05-31
> **Dispatch anchor (on `main`, not re-committed here):** [`docs/audit/v4-rustc-error-catalog-2026-05-31.md`](https://github.com/gunb-ai/gunbc/blob/main/docs/audit/v4-rustc-error-catalog-2026-05-31.md) §5 — SG-8 **~796** (E0425+E0432+E0433) per [#4086](https://github.com/gunb-ai/gunbc/pull/4086). Fresh M1 probe histogram is **implementation-PR evidence only** (not worksheet authority).
> **Primary consumer (implementation worker):** `src/v2/05_emit_rust.dag` `emit_imports` + generic type-alias emission (M1 v2 Rust emit path over full `src/v4` tree).
> **Canonical modeling home (export facts):** `src/v4/std/target_model.dag` (`v4.std.target_model`) for cross-target export-surface vocabulary; **live M1 fix lands in v2 emit** until `06_translate` owns Rust module files.

### Status (single authority — no contradiction)

| Layer | State |
| ----- | ----- |
| **Worksheet** | **READY-FOR-WORKER-DISPATCH** — §8 closed 2026-06-01 (`proud-fox-405`) |
| **Worksheet PR** | [#4127](https://github.com/gunb-ai/gunbc/pull/4127) — on `main` (docs-only; no emitter) |
| **Implementation dispatch** | **Authorized** — Target Realization (`05_emit_rust.dag` items (1)+(2)); §4 falsification in implementation PR; Rust RCA Manager fanout |

---

## Mechanical dispatch rule

> **No SG-8 implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved (§8 checklist).**
>
> **#4127 merge gate:** §10.0 worksheet content complete + §8 sign-off only. Falsification probes F1–F4 are proven in the **implementation PR**, not in the worksheet-only PR.

Acceptance is **§4 falsification probes**, not E0425/E0432/E0433 count reduction on the M1 probe.

---

## §10.0-adapted worksheet

```text
SG class:               SG-8
Representative emitted failure:
  // Import site: v4.compiler.target_carriers (imports String from v4.std.text)
  pub use crate::v4_std_text::{GoScalarKind};
  use crate::v4_std_text::GoScalarKind::{String};
  pub type TargetSource = FreeMonoid<Char>;
  // rustc E0432: GoScalarKind not in v4_std_text; String is a type alias there, not a variant.

  // Import site: v4.compiler.emit (imports TargetSource from target_carriers)
  pub use crate::v4_compiler_target_carriers::{CarrierKind};
  use crate::v4_compiler_target_carriers::CarrierKind::{TargetSource};
  // rustc E0432: CarrierKind lives in v4.std.pipeline, not target_carriers.

  // Import site: many modules
  pub use crate::v4_std_collection::{List};
  // rustc E0432: List<T> = FreeMonoid<T> generic alias never emitted in collection.rs

Immediate local patch:
  - Hand-add pub use lines to emitted crate root / shim modules.
  - Re-export CarrierKind from target_carriers.dag (duplicate authority).
  - Per-error mod rs patch table keyed by unresolved import spelling.
Why that patch is forbidden:
  - INVARIANTS P2: parallel authority — export surface must be derived from resolve/admission facts, not duplicated in shims.
  - Name-keyed tables (CarrierKind, List, GoScalarKind, …) calcify and do not scale to new carriers/modules.
  - Hides two independent emit bugs: (a) variant-parent confusion on type imports; (b) missing generic type-alias emission.
DFS path:
  std/ authority:
    - CarrierKind / PipelineStage: src/v4/std/pipeline.dag
    - List / Map / Set aliases: src/v4/std/collection.dag
    - TargetModelBundle: src/v4/std/target_model.dag
    - Char / String: src/v4/std/text.dag
    - GoScalarKind: src/v4/extdeps/languages/go.dag (not std.text)
  extdeps/language authority:
    - Rust module templates: dsl/extdeps/languages/rust/imports.dag (templates only — no export graph)
    - PubInPath / RustVisibility: src/v4/extdeps/languages/rust.dag (T-28 residual per TASKS.md)
  compiler stage consuming it:
    - v2 `emit_imports` in src/v2/05_emit_rust.dag (M1 full-tree emit)
    - v4 `src/v4/compiler/03_name_resolve.dag` owns admission/export binding (T-28-B); emit must consume **defining module**, not import-site module
  existing scaffold/dissolution notes:
    - T-28 dissolved catalog carrier; T-28-B admission in `compiler/03_name_resolve.dag` (per `src/v4/TASKS.md`)
    - TASKS.md T-28 residual: PubInPath visibility authority before PubInPath consumers execute
Deepest unsound boundary:
  Rust import emission treats every imported spelling as a potential enum variant, resolves variant parents against the **import statement's module**, and re-exports parent + child from that module. Type imports (registry `TypeItem`) and defining-module boundaries are ignored. Separately, parametric type aliases (`type List<T> = …`) classify as `type_decl` and emit nothing.
Systemic fix:
  (1) emit_imports: skip variant-parent expansion for graph type names (`ItemInfo.kind == TypeItem`); resolve `pub use` / variant paths from **defining** `ItemInfo.module_name`, not import site.
  (2) emit_typed_item: emit `pub type Foo<T> = …` for parametric type aliases (`is_type_decl_item` + alias rhs).
  (3) Follow-on (out of scope this PR): `TargetModuleExportSurface` row bundle in v4.std.target_model consumed by v4 06_translate when Rust emit migrates off v2.
Non-goals:
  - Hand-editing generated `src/v4_*.rs`.
  - Duplicating CarrierKind / List into shim modules.
  - SG-4 Char atom realization (separate class; may shrink overlap).
  - SG-2 generic arity (E0107/E0282).
  - M1 error-count reduction as acceptance.
Falsification probe:
  (F1) Add `import v4.std.text { String }` in a new test module that also imports nothing from go.dag — emitted Rust has **no** `GoScalarKind` use lines.
  (F2) `import v4.compiler.target_carriers { TargetSource }` — emitted Rust does **not** `pub use` CarrierKind from `v4_compiler_target_carriers`.
  (F3) `import v4.std.collection { List }` — `v4_std_collection.rs` contains `pub type List<…>`.
  (F4) New generic alias `type Pair<T> = FreeMonoid<T>` in a hermetic fixture `.dag` module — emits `pub type Pair<…>` via the generic alias path (no per-name emitter branch).
Metric allowed only as secondary:
  ~796 SG-8-family per committed catalog §5 (#4086); implementation PR attaches fresh probe receipt (not worksheet authority).
```

---

## §4 Falsification table (worker PROVEN rows)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| F1 | String type import cannot pull GoScalarKind parent | `rg 'GoScalarKind' emitted module` empty for fixture |
| F2 | TargetSource import does not re-export CarrierKind from wrong mod | `rg 'v4_compiler_target_carriers.*CarrierKind|CarrierKind.*v4_compiler_target_carriers' <emitted_fixture.rs>` empty (forbids wrong-module `pub use` / variant-parent path at :41–42); correct `CarrierKind` from `v4_std_pipeline` is allowed |
| F3 | List alias emitted in collection | `rg 'pub type List' v4_std_collection.rs` present |
| F4 | New generic alias emits via generic path | Hermetic fixture `.dag` module + **structural** `.dag TestClaim` (runner-evaluated) **or** `src/v2/tests` unit test; receipt: `rg 'pub type Pair' <emitted_fixture.rs>` in implementation PR. Prose/manual assertion alone is **not** PROVEN. |

---

## §5 Landing order (implementation — not worksheet-only PR)

```text
1. emit_imports: graph-type variant isolation + defining-module pub use (05_emit_rust.dag).
2. emit_typed_item: parametric type-alias emission (05_emit_rust.dag).
3. regen_stage0 + v4 M1 probe; attach §4 falsification table PROVEN rows.
4. Follow-on (separate dispatch): TargetModuleExportSurface in v4.std.target_model for 06_translate.
```

**Lane split:** Target Realization owns steps 1–2; Runtime/TestClaim owns step 3 transcript.

---

## §6 Downstream worker brief (dispatch after §8)

Implement §10.0 systemic fix (1)+(2) in `src/v2/05_emit_rust.dag` on `main` after #4127 merges. Re-run `scripts/v4-m1-rust-emit-probe.sh`; attach probe summary + forbidden-pattern greps as implementation PR evidence. F4 must ship a hermetic `.dag TestClaim` or `src/v2/tests` fixture with grep/compile receipt (per §4 table). Do not claim SG-8 PROVEN on error count alone.

**Worksheet-only PR non-goals:** `src/v2/05_emit_rust.dag`, `src/v2/stage0/`, or generated `src/v4_*.rs` edits (per SG-RC #4100 worksheet-only pattern).

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Single-authority fact: defining-module `pub use` + graph-type variant isolation (not import-site parent tables) — export surface derived from resolve/admission facts in `03_name_resolve.dag`, consumed by `emit_imports` (§10.0 systemic fix (1)+(2))
- [x] Spot-fix forbidden: per-error `pub use` patch tables / shim re-exports / hand-edited `src/v4_*.rs`
- [x] Falsification probes F1–F4 accepted (structural `.dag TestClaim` or `src/v2/tests` for F4 — no prose-only PROVEN)
- [x] DFS path lands in v2 `05_emit_rust.dag` for M1 emit (not duplicate `CarrierKind` in shims); follow-on `TargetModuleExportSurface` correctly deferred to separate dispatch (§5 step 4)
- [x] Cross-language note: module/export facts are Rust-emit path first; shared `TargetModuleExportSurface` in `v4.std.target_model` is follow-on — no per-language reinvention required for this worksheet's core fix
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`, Modeling DFS Arbiter per #4137 §11.2)

## Related artifacts (committed on `main` — navigation only)

- [`docs/audit/v4-rustc-error-catalog-2026-05-31.md`](https://github.com/gunb-ai/gunbc/blob/main/docs/audit/v4-rustc-error-catalog-2026-05-31.md) §5 — landed [#4086](https://github.com/gunb-ai/gunbc/pull/4086)
- [`docs/planning/v4-correctness-ladder-2026-05-30.md`](https://github.com/gunb-ai/gunbc/blob/main/docs/planning/v4-correctness-ladder-2026-05-30.md) §10.0 template — landed [#4120](https://github.com/gunb-ai/gunbc/pull/4120)
- `src/v4/TASKS.md` T-28 / T-28-B / PubInPath residual (on `main`)
