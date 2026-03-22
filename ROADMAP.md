# gunbc Roadmap — v2 Bootstrap Completion

The v2 compiler has proven self-consistency: stage1 == stage2 (byte-identical).
This roadmap tracks the remaining work to close out bootstrapping, retire v1,
and reach the target architecture.

## What's Done

| Milestone | Gate | Date |
|-----------|------|------|
| Self-compile pipeline | v2 processes own .dag through all 5 stages | 2026-03 |
| Bootstrap A5 | v1 → stage0 → stage1 (cargo check ✓) | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output (byte-identical) | 2026-03 |
| A7 Phase 1 | Self-compile: 0 cargo check errors | 2026-03 |
| TypeExpr→Node | 8 TypeExpr variants deleted | 2026-03 |
| Expr→Node | 21 Expr variants deleted, ExprData discriminator on Node | 2026-03 |
| Transport dissolution | TransportBinding (4 variants) deleted | 2026-03 |
| Node/TypedNode unified IR | W1–W13 complete, 129 tests passing | 2026-03 |
| Performance audit | 50,000x improvement (tokenize+parse: 24ms) | 2026-03 |
| OOM fix | node_type_deps container-wrapped cycle detection | 2026-03 |

---

## Current Compositional State (2026-03-22 Audit)

This audit is not a new roadmap phase. It is a map of where the compiler
currently behaves like the `extdeps/` compositional model and where it still
collapses layer authority.

| Layer | Current state | Meaning for the next passes |
|------|------|------|
| `00_core.dag` | Strong foundation, mostly target-agnostic | `Node`/`ExprData`/transport modeling is the right base. Core now owns kernel-type authority and the shared self-call classifier; remaining ownership leakage sits downstream in reconcile/emit rather than on core types. |
| `01_tokenize.dag` | Mostly clean syntax leaf | Tokenization is structurally isolated; bootstrap-specific Rc commentary and `SourceRef` are still host-artifact leakage. |
| `02_parse.dag` | Strong compositional lowering | Service/resource syntax already dissolves into uniform `Node` structure and records facts like `namespace_root` structurally. |
| `03_resolve.dag` | Cleanest authority boundary | Pure import graph construction with almost no target leakage. Keep using this as the reference for stage boundaries. |
| `04_reconcile.dag` | Main structural hotspot | Owns too many concerns at once: typing, call analysis, method classification, and emitter metadata. The Rust ownership hints have now mostly been pushed out. |
| `05_emit*.dag` | Partial extdeps-style composition | Shared emit imports language facts from `extdeps.languages.*`, and Rust now derives Rc decisions locally, but target policy is still split between shared emit and the per-target renderers. |
| `07_complexity.dag` / `07_ownership.dag` | Good proof layers | These are the best examples of compositional modeling of the compiler itself: proof objects, not runtime execution. Remaining issue is duplicated expression walking and the remaining classifier/renderer string dispatch. |
| `06_pipeline.dag` / `08_artifact.dag` / `09_trace.dag` | Narrowed to honest boundaries | `06_pipeline.dag` now owns only the real compile path and Go dispatch. `08_artifact.dag` is explicit-plan-only. `09_trace.dag` is now a normalized runtime trace contract, still not pipeline-wired. |

### Audit Reconciliation

This section reconciles the audit above with the existing phase plan.

- The v2 audit in `INVARIANTS.md` is directionally correct, but several items in the
  roadmap now need reinterpretation based on what has already landed.
- Theme 4 and Theme 6 are not optional cleanup. They are cross-cutting prerequisites
  for efficient Phase 1 work because they remove duplicate authority and dead branches
  before deeper semantic changes.
- `P1.8` is now deeper than it was in the first audit: `07_complexity.dag` has
  `intrinsic_method_cost_shape(...)`, `cost_of_expr(...)` reads reconcile-provided
  `method_semantics`, `receiver_size_var(...)` now follows those semantics instead of
  string names, and `04_reconcile.dag` resolves known method semantics/result types in
  one helper. Remaining work is mostly renderer-leaf dispatch and the source-level
  classifiers that still map strings into those enums.
- `P4.1` is no longer the full blocker it used to be. Shared emit already imports
  language type/keyword/container data from `extdeps.languages.*`. The remaining
  duplication is mostly per-target reserved-word/runtime tables, especially in the
  Python and Go renderers.
- Trace has now been reconciled with `src/v2/DESIGN.md` at the model level:
  `09_trace.dag` no longer describes an in-compiler interpreter. Remaining work is
  deciding whether runtime adapters/source maps get wired into the pipeline or remain
  explicitly external contracts.

### Structural Pass Order

These passes cut across phases. They should be executed in this order because each one
reduces the cost or risk of the next.

| Pass | Theme | What it changes first |
|------|------|------|
| S1 | Theme 4 | Single authority for kernel/primitive facts (`kernel_types`, `is_kernel_type`) |
| S2 | Theme 6 | Landed: pipeline owns compilation only, artifact is explicit-only, trace is an honest future-work contract |
| S3 | Theme 3 | In progress: known-method resolution is centralized and complexity now follows semantics; renderer/runtime cleanup remains |
| S4 | Theme 5 | Move Rust-only ownership/render policy out of core + reconcile |
| S5 | Theme 1 | Fuse duplicated `ExprData` walks in reconcile and complexity |
| S6 | Theme 2 | Shared emit dispatch with per-target leaves |
| S7 | Theme 7 | Final fabrication fallback cleanup and Dynamic-site audit |

The practical implication: Phase 1 should not be treated as only "fix inference gaps."
The first work inside Phase 1 is structural reduction of duplicate authority.

---

## Phase 1: Strict Soundness — COMPLETE

All P1 items implemented (2026-03-22).

- P1.1-P1.4: Type inference gaps fixed (Tuple, fold, map_insert, chaining)
- P1.5: node_type_equals tightened — Dynamic only equals Dynamic, structural fallback removed
- P1.6: Callable/function-value type — callable_node wraps params+return_type
- P1.7: TupleFirst/TupleSecond in all 3 emitters, pure match on access_style
- P1.8: Exhaustive IntrinsicMethod match in complexity analyzer
- P1.9: Non-ignored v2_strict_pipeline_smoke test (runs every cargo test)
- P1.10: ErrorCategory enum, Warning→Error for fail-closed diagnostics

**Ratchet:** `DIAG_RATCHET` in `src/v2/tests/src/lib.rs` — was 25.

### P1.1–P1.4: Type inference gaps (25 → 0)

These can be done in parallel. P1.4 is mostly resolved by P1.2+P1.3.

| ID | Item | What | File |
|----|------|------|------|
| P1.1 | Tuple type | `enumerate` returns `List<Tuple<Int, T>>`, not `List<T>`. Define Tuple in `dsl/std/types.dag`. Change known-method return-type inference in `04_reconcile.dag` to construct `List<Tuple<Int, element>>`. Update emit to render `(T0, T1)`. | `04_reconcile.dag`, `05_emit_rust.dag` |
| P1.2 | Fold accumulator | `fold` returns Dynamic. Extract init-arg type in ExprCall handler BEFORE resolving known-method return and thread it into method resolution. | `04_reconcile.dag` |
| P1.3 | map_insert structure | `infer_builtin_call_type:1617` returns bare `Map`. Return first arg's type instead (preserves key/value structure). Same for `map_merge`, `with`. | `04_reconcile.dag` |
| P1.4 | Field access chaining | Resolved by P1.2+P1.3 — chained `.bar` fails because upstream returns Dynamic. Verify; fix residual cases in `lookup_field_type_node`. | `04_reconcile.dag` |

### P1.5: Tighten node_type_equals (AFTER P1.1–P1.4)

Remove two permissive fallbacks. Must wait until Dynamic introduction points are fixed.

| Location | What | Fix |
|----------|------|-----|
| `:1017` | `Dynamic == *` returns true | Dynamic only equals Dynamic |
| `:1084-1087` | Structural fallback (same name + connective + child count) | Delete; types must match through container/optional/map branches |

### P1.6–P1.9: Independent items (parallel with P1.1–P1.4)

| ID | Item | What |
|----|------|------|
| P1.6 | Callable type | Add callable Node shape `(params) -> return_type`. FunctionValueBinding stores callable Node. Calls resolve through callable's return_type. |
| P1.7 | Field-access kind upstream | Add `TupleFirst`, `TupleSecond` to `FieldAccessStyle`. Set in reconcile. Emit becomes pure match on `access_style` — delete hardcoded `.0`/`.1`. |
| P1.8 | Dual classification | `classify_method_cost` in `07_complexity.dag` is single authority. Complexity analyzer reads from EmitGraphInfo, not its own parallel classifier. Delete `classify_method_cost`. |
| P1.9 | Non-ignored smoke test | Lightweight test running strict pipeline on synthetic 2-type, 2-function module via interpreter. Assert 0 diagnostics. Exercises full path without building binary. |

### P1.10: Structural error variant (Root Cause C)

Add `ErrorCategory` enum (UnresolvedName | TypeMismatch | FieldNotFound | ...).
Error nodes carry `error_category: ErrorCategory?`. Each introduction site sets it.
The 2 Warning-severity diagnostics become Error — fail closed.

---

## Phase 2: Gist End-to-End — IN PROGRESS

**Gate:** `gist.dag` + 11 transitive deps → Rust → `cargo build` → `cargo run --
dry-run` → correct output.

**Status (2026-03-22):**
- P2.1: Interpreter path blocked — v1 interpreter can't handle multi-module real
  .dag files through compile_sources (lambda scoping issue: "unbound variable: t").
  Tests created: ignored full-pipeline test (stage0 binary), working synthetic
  service pipeline smoke test.
- P2.2: Already implemented — emit_rust.dag has real transport call emission
  (reqwest, Command, auth injection, dry-run mocking).
- P2.3: Already implemented — main.rs generation with workflow subcommands,
  clap args with defaults, function dispatch match arms.
- P2.4: Needs verification via stage0 binary.
- P2.5: Needs stage0 binary.

**Blocker:** Full gist E2E requires stage0 binary build (~2 min).
The v1 interpreter limitation will be resolved when v2 is self-hosting (P3+).

| ID | Item | Status |
|----|------|--------|
| P2.1 | Gist pipeline test | Partial — interpreter blocked, tests scaffolded |
| P2.2 | Service operation bodies | Done (pre-existing) |
| P2.3 | Main.rs workflow dispatch | Done (pre-existing) |
| P2.4 | Multi-module extdep imports | Needs stage0 verification |
| P2.5 | End-to-end build+run test | Needs stage0 binary |

**Files:** `05_emit_rust.dag`, `06_pipeline.dag`, `03_resolve.dag`,
`dsl/extdeps/languages/rust/runtime.dag`, `src/v2/tests/src/lib.rs`

---

## Phase 3: v1 Retirement

**Gate:** v2 compiles everything v1 can. v1 is no longer needed for any
compilation path. S76–S81 bootstrap scaffolding is dead code.

**Prerequisite:** Phase 2 complete (gist builds and runs).

| ID | Item | What |
|----|------|------|
| P3.1 | Verify parity | Enumerate all .dag files v1 compiles. Verify v2 produces equivalent output. Port any remaining v1-only paths. |
| P3.2 | Runtime shim dissolution | 21 functions in `v2_runtime_shim.rs` → template strings in `dsl/extdeps/languages/rust/runtime.dag`. Update `emit_v2_rt_module()` to read from runtime.dag. Functions: concat, char_at, string_length, substring, string_contains, lookup, index_by, to_string, empty_map, map_insert, map_merge, list_concat, str_eq, scan_while, skip_horizontal_ws, scan_to_eol, scan_string_end, code_point, from_code_point, filesystem_read, Concat trait. |
| P3.3 | Scaffolding verification | S76–S81 are only called by `assemble_v2_crate()`. Once v2 self-compile and gist work without v1, mark `#[deprecated]`. |
| P3.4 | Archive v1 | Move `src/v1/` → `archive/v1/`. Update Cargo workspace. Update CLAUDE.md. |

---

## Phase 4: Generic Emitter + Language Extdeps

**Gate:** Adding a new target language = writing a language extdep.
Zero compiler changes required.

**Prerequisite:** Phase 3 complete (v1 retired).

| ID | Item | What |
|----|------|------|
| P4.1 | Import aliasing | Blocker: `05_emit.dag:578-594` duplicates language data inline because all three extdeps define same-named declarations and imports lack `as` aliasing. Add `import { name as alias }` to tokenizer, parser, resolver. |
| P4.2 | LanguageSpec wiring | `LanguageSpec` exists in `dsl/std/languages.dag:393-429` but no emitter reads it. Add `load_language_spec(target) -> LanguageSpec`. Pass through emit functions. |
| P4.3 | Extract generic emit core | ~70% duplication across 3 emitter files (rust: 3606, python: 1168, go: 1195 lines). Extract shared skeleton: item dispatch, type structure classification, expression dispatch. Parameterize by LanguageSpec. Per-language files shrink to irreducible transforms (Rust: ownership/clone/borrow; Python: exceptions/comprehensions; Go: multi-return/interfaces). |
| P4.4 | `--target` CLI flag | `compile_sources` already takes `target: RenderTarget`. Wire through bootstrap main.rs Compile subcommand. |
| P4.5 | Validate equivalence | Self-compile + gist → same output with generic emitter. Fixed point holds. |

**Architecture:**
```
compiler core (language-agnostic)
    ↓ reads
LanguageSpec interface (.dag contract)
    ↓ filled by
language extdep (dsl/extdeps/languages/rust/)
    ↓ rendered by
thin semantic renderer (irreducible differences only)
    ↓ produces
target source files
```

---

## Phase 5: Convergence

**Gate:** One type (Node) flows through the entire pipeline. `04_infer.dag`,
not `04_typecheck.dag`. Each dissolution step validated by re-bootstrapping
and proving stage1 == stage2.

**Prerequisite:** Phase 4 complete (generic emitter).

| ID | Item | What |
|----|------|------|
| P5.1 | Rename | `04_reconcile.dag` → `04_infer.dag`. Update all imports and test references. Re-bootstrap → fixed point. |
| P5.2 | Token dissolution | Token (`:30`) + TokenKind (77 variants, `:35-78`) → Node compositions. Largest dissolution. 4-step: add Node constructors → dual-write → migrate parser → delete types. |
| P5.3 | Module dissolution | Module (`:92`), Import (`:103`), ImportNames (`:99`) → Node Conj compositions. Update parser (produces) and resolver (consumes). |
| P5.4 | Diagnostic dissolution | Diagnostic (`:346`), Severity (`:353`), CompileResult (`:336`), TextFile (`:341`) → Node compositions. Update all producers/consumers. |
| P5.5 | Service types | ServiceConfig (`:293`), OperationDef (`:302`), CapabilityDef (`:316`) — may already dissolve during parsing. Verify; convert if not. |
| P5.6 | Semantic types | IntrinsicMethod (17 variants), BuiltinTypeKind (15 variants), VarBindingKind, FieldAccessStyle, etc. Closed enums stay as .dag type definitions producing Nodes. TransportKind already redundant. |

P5.2–P5.6 are independent; each must re-bootstrap and prove fixed point.

After convergence:
```
source → parse → resolve → infer → emit
           ↓        ↓        ↓       ↓
         Nodes    Nodes    Nodes   TextFiles
         (raw)  (imports  (types
                 linked)  filled)
```

---

## Deferred (in BACKLOG.md)

These items are tracked but not blocking bootstrap closure:

- Root Cause B: closed sets as strings (mechanical enum conversions)
- General generic syntax (`type Foo<T> = ...`)
- Full linear type checking (D-ownership sufficient for now)
- B3 Ph2a: SCC-aware return type resolution
- Widen V5: non-takeable fields in functional record update
- Anonymous record target resolution
- TCO backend contract

---

## Ordering

```
P1.1 (tuple)  ─┐
P1.2 (fold)   ─┤
P1.3 (map)    ─┼─→ P1.5 (tighten equality) ─→ P1.10 (error variant)
P1.4 (chain)  ─┘
P1.6 (callable) ────→ independent
P1.7 (field kind) ──→ independent
P1.8 (dual class) ──→ independent
P1.9 (smoke test) ──→ independent

P1.* done ──→ P2.1 → P2.2 → P2.3 ─→ P2.5
              P2.4 ─────────────────→ P2.5

P2 done ───→ P3.1 → P3.2 → P3.3 → P3.4

P3 done ───→ P4.1 → P4.2 → P4.3 → P4.4 → P4.5

P4 done ───→ P5.1 → P5.2 ─┐
                    P5.3 ─┤
                    P5.4 ─┼─→ all independent, each re-bootstraps
                    P5.5 ─┤
                    P5.6 ─┘
```

---

## Verification

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude gunbc-dag-tests` | After every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | After every change |
| 0 diagnostics | `cargo test -p gunbc-dag-tests v2_strict_compile_diagnostic_count -- --ignored` | End of Phase 1 |
| Fixed point | `cargo test -p gunbc-dag-tests v2_bootstrap_fixed_point -- --ignored` | After any .dag change |
| Gist pipeline | `cargo test -p gunbc-dag-tests v2_gist_full_pipeline -- --ignored` | End of Phase 2 |
| Gist e2e | `cargo test -p gunbc-dag-tests v2_gist_end_to_end -- --ignored` | End of Phase 2 |
