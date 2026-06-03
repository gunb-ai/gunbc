# Cleanup sweep #3: `src/v4/extdeps/` — concerning-patterns catalog

**Session:** nimble-lynx-641  
**Date:** 2026-06-03  
**Scope:** 45 `.dag` files, ~37k LOC (`src/v4/extdeps/`; brief said 44 — +1 is `languages/fidelity.dag` split carrier)  
**Method:** Read-only inventory during hard freeze; no code edits.

## Slice: `src/v4/extdeps/` — overall scariness: 🟡

Correct home for language/transport facts, but most mass is wave-scaffolded LanguageModels with CP-1b dual grammar carriers (FormalProduction + operational GrammarExpr/LexRules maintained in parallel). Compiler/std still hold host-pin and cross-target tables that belong here.

### Per-file (worst-first)

- `src/v4/extdeps/languages/rust.dag` | 🔴 | bridges: CP-1b dual-carrier (formal + operational grammar/lex), wave-1/2 MVP scaffolds, T-6 layout-in-literals 🟡, E-6(b) staging | towers: 15-import concept-sink; List catalog tables (integer/float/atom/signature/collection realizations); PerLanguageFactBundleRegistry fold-build; target_value_template / serialize_source rows; heavy fold_list | Largest reference LanguageModel; authority for MVP emit/host pins still matched in `compiler/emit_host.dag` by string.
- `src/v4/extdeps/languages/dag.dag` | 🔴 | bridges: self-host wave-1 lex/grammar; T-36 round-trip readiness 🟡 gated conjuncts; T-8 per-identifier hashing deferred; C5 trivia fidelity | towers: 11 imports; FormalProduction + GrammarExpr parallel; fold for readiness receipts | Compiler pipeline (`source_authority`, `normalize`, `resolve`, `name_resolve`, `00_compile`) imports symbols but keeps round-trip law + serializer walk in compiler/.
- `src/v4/extdeps/languages/typescript.dag` | 🔴 | bridges: CP-1b dual-carrier; wave-2a grammar; derive_grammar_relation rows | towers: 16 imports; PerLanguageFactBundle; template/serialize_source; fold_list | SG-2 type-expression projection surface dense.
- `src/v4/extdeps/languages/kotlin.dag` | 🔴 | bridges: CP-1b dual-carrier; wave scaffolds | towers: 12 imports; grammar-heavy; catalogs + templates | JVM family; similar shape to java/swift.
- `src/v4/extdeps/languages/swift.dag` | 🔴 | bridges: CP-1b dual-carrier | towers: 12 imports; large grammar block; fold_list; realization catalogs | Wave-2a claims in `test/claim/parse`.
- `src/v4/extdeps/languages/java.dag` | 🔴 | bridges: CP-1b dual-carrier | towers: 12 imports; grammar + model_core bundles | Wave-2 grammar structure claims.
- `src/v4/extdeps/languages/wasm.dag` | 🔴 | bridges: grammar scaffold | towers: 12 imports; fold_list; target templates | Binary/text wasm facts + parse scaffold.
- `src/v4/extdeps/languages/go.dag` | 🔴 | bridges: CP-1b; MVP host row (`go_mvp1_source_text`) pinned from `compiler/emit_host` | towers: 13 imports; fact bundles; fold_list | `emit_host_authority_pin` duplicates source text here.
- `src/v4/extdeps/languages/python.dag` | 🔴 | bridges: CP-1b; MVP host row; many 🟡 dissolve-on | towers: 14 imports; PerLanguageFactBundle; fold_list | `python_mvp1_source_text` ↔ emit_host pin leak.
- `src/v4/extdeps/languages/ecmascript.dag` | 🔴 | bridges: CP-1b dual-carrier | towers: 13 imports; grammar + grounding bundles | ES/JS shared substrate bulk.
- `src/v4/extdeps/languages/cpp.dag` | 🔴 | bridges: CP-1b; operational scaffold | towers: 13 imports; grammar + target_model edges | Host C++ ABI defers to `cpp_abi.dag`.
- `src/v4/extdeps/formatters/clang_format.dag` | 🔴 | bridges: T-4.16 scaffold — full clang-format 23 option space hand-declared | towers: 68 🟢 terminal coproduct variants (style enum explosion); refinement literals | Config-as-types, not grammar; size is option taxonomy not logic.
- `src/v4/extdeps/languages/llvm_ir.dag` | 🔴 | bridges: fidelity disposition via `languages/fidelity.dag` | towers: grammar + structural carriers; 46 🟢 terminals | Lowers toward machine_code; C5 anchor.
- `src/v4/extdeps/languages/verilog.dag` | 🔴 | bridges: grammar schema probes; many 🟡 SL-/CP- dissolve markers | towers: IEEE structural coproducts (nets, ports) | Mostly green terminals but wide type surface.
- `src/v4/extdeps/languages/lean.dag` | 🔴 | bridges: grammar scaffold; fidelity | towers: model_core + grammar | Proof assistant facts; moderate size.
- `src/v4/extdeps/languages/machine_code.dag` | 🟡 | bridges: fidelity FailClosed/Modeled | towers: ISA-shaped carriers; some grammar | Companion to llvm_ir/ptx.
- `src/v4/extdeps/formats/openapi.dag` | 🟡 | bridges: 17 dissolve-on / staged OAS 3.1 | towers: concept-sink (coordination+json+json_schema+yaml+network); large coproduct API surface | Should stay format-layer; not a language.
- `src/v4/extdeps/formatters/rustfmt.dag` | 🟡 | bridges: tool profile + 🟡 gates | towers: rustfmt option coproducts | Thin runner; config bulk.
- `src/v4/extdeps/formats/spice.dag` | 🟡 | bridges: CP-1b grammar for SPICE netlists | towers: grammar + lex | Electrical format with parse scaffold.
- `src/v4/extdeps/languages/ptx.dag` | 🟡 | bridges: grammar scaffold | towers: NVIDIA PTX carriers | GPU assembly facts.
- `src/v4/extdeps/languages/english.dag` | 🟡 | bridges: NLP/grammar experimental | towers: grammar-heavy for non-code language | Odd sibling in `languages/` — verify slice ownership.
- `src/v4/extdeps/runtimes/v4_evaluator.dag` | 🟡 | bridges: Option-C concrete runtime; wave-1 interpretation symbols; RuntimeTarget wiring staged | towers: closed symbol registry for eval primitives; imports model_core+runtime | `05_eval.dag` imports wave-1 symbols + readiness from `dag.dag`.
- `src/v4/extdeps/frameworks/react.dag` | 🟡 | bridges: framework component model scaffold | towers: moderate coproduct UI surface | |
- `src/v4/extdeps/file_system.dag` | 🟡 | bridges: header says scaffold; Wave-2 fail-closed; POSIX path retained | towers: path coproducts (OS-1 dissolve) | `posix.dag` depends on this.
- `src/v4/extdeps/formats/sql.dag` | 🟡 | bridges: grammar/schema probe | towers: fold_list | SQL as format not language.
- `src/v4/extdeps/formats/toml.dag` | 🟡 | bridges: 7 staged gates | towers: witness coproducts for TOML profile | Mostly declarative.
- `src/v4/extdeps/formatters/prettier.dag` | 🟡 | bridges: multi-language formatter config | towers: option coproducts | |
- `src/v4/extdeps/formatters/lean4_format.dag` | 🟡 | bridges: 6 dissolve-on | towers: lean tool config types | Small runner surface.
- `src/v4/extdeps/coordination.dag` | 🟡 | bridges: exchange-pattern scaffold | towers: none heavy | `openapi` imports this.
- `src/v4/extdeps/cpp_abi.dag` | 🟡 | bridges: Itanium/MSVC ABI facts | towers: 10🟢 ABI coproducts | Separated from `languages/cpp.dag` — good split.
- `src/v4/extdeps/platform_detection.dag` | 🟡 | bridges: 1 dissolve-on | towers: host triple / platform enum tables | Transport detection, not grammar.
- `src/v4/extdeps/posix.dag` | 🟡 | bridges: process substrate (couples file_system) | towers: refinement-guarded Int newtypes | Used by formatters/runners; no v3 import.
- `src/v4/extdeps/coercion_widening.dag` | 🟡 | bridges: MVP single pair rust i32→python int (W3.2) | towers: hand-wired PreservationPredicate | Should become registry-as-data in extdeps.
- `src/v4/extdeps/formatters/black.dag` | 🟡 | bridges: 2 gates | towers: config coproducts | Thin.
- `src/v4/extdeps/formatters/swift_format.dag` | 🟡 | bridges: 2 gates | towers: swift-format options | Thin.
- `src/v4/extdeps/formatters/ktfmt.dag` | 🟡 | bridges: 2 gates | towers: ktfmt options | Thin.
- `src/v4/extdeps/formats/yaml.dag` | 🟡 | bridges: 5 gates; parse axis staged | towers: document model | openapi consumes.
- `src/v4/extdeps/formats/csv.dag` | 🟡 | bridges: 3 gates | towers: row/profile witnesses | Small.
- `src/v4/extdeps/formats/json_schema.dag` | 🟡 | bridges: 4 gates | towers: schema coproducts | Feeds openapi.
- `src/v4/extdeps/formatters/gofmt.dag` | 🟢 | bridges: 1 gofmt invocation row | towers: none | 21 LOC effective transport pin.
- `src/v4/extdeps/formats/json.dag` | 🟢 | bridges: parse/emit axes gated T-6/T-7 (declared, not implemented here) | towers: RFC witness coproducts (🟢 terminals) | Clean declarative JSON value model.
- `src/v4/extdeps/languages/fidelity.dag` | 🟢 | bridges: none | towers: none | 13-line shared FidelityDisposition carrier.
- `src/v4/extdeps/typecheckers/pyright.dag` | 🟢 | bridges: none | towers: tool_id + config record | 84 LOC; pairs with mypy.
- `src/v4/extdeps/typecheckers/mypy.dag` | 🟢 | bridges: none | towers: diagnostic code list | 49 LOC.
- `src/v4/extdeps/formatters/google_java_format.dag` | 🟢 | bridges: 1 formatter row | towers: none | 32 LOC.

### Recurring patterns (top 5 in this slice)

1. **CP-1b dual-carrier hold** — Every major `languages/*.dag` declares `FormalProduction` flat rhs AND parallel `GrammarExpr`/`LexRules` “operational parse scaffold until convergence”; duplicates authority (rust/python/go/ts/java/kotlin/swift/ecmascript/dag/wasm/cpp).
2. **TargetModel catalog towers** — `List<TargetAtomRealization>`, signature catalogs, `target_value_template` / `serialize_source` rows built by fold over lists instead of registry-as-data projection from `std/target_model`.
3. **Wave/MVP scaffold markers** — wave-1 vs wave-2 lex/grammar, MVP-1 add-fn pins, T-36 readiness Bool conjuncts (especially `dag.dag` + `v4_evaluator` coupling).
4. **Grammar/syntax leakage outward (not inward)** — extdeps is authoritative for lex/grammar, but `compiler/emit_host.dag` still embeds rust/python/go `authority_source_text` string pins and `emit_host_rust_i64_node`; `std/rust_leaf_model_claim.dag` and `std/leaf_model_verification.dag` hold per-target diagnostic/claim vocab; `compiler/source_authority.dag` keeps dag round-trip law despite importing `dag_language_model_wave1_modeled` from here.
5. **Format vs language boundary blur** — sql/spice/openapi/json carry grammar probes; `languages/english.dag` is natural-language grammar in the languages tree.

### Missing-substrate map (what towers hand-roll)

| Unmodeled repetitive work | Where it recurs | One shared surface that would dissolve it |
|---------------------------|-----------------|---------------------------------------------|
| **CP-1b bidirectional grammar** — second operational grammar/lex algebra beside FormalGrammar | All major `languages/*.dag`; `formats/spice.dag` | `std/grammar.dag` + `std/lexing.dag` convergence: project ParseGrammar/GrammarExpr from FormalGrammar (TASKS.md CP-1b) |
| **Registry-as-data / TotalMap** — `List`+fold catalogs, closed `if sym ==` registries | `rust.dag`, `python.dag`, `go.dag`, `typescript.dag`, `coercion_widening.dag`, `v4_evaluator.dag` | Substrate `Registry<K,V>` or TotalMap with derived insert/lookup (not hand-maintained list tables) |
| **Projection / typed receipts** — Bool conjuncts + Node witnesses for readiness | `dag.dag` T-36 cluster; `05_eval.dag` round-trip gate | `std/projection` typed receipt carriers (e.g. `DagRoundTripWave1Readiness`) replacing embedded ready atoms |
| **fold/algebra over registry edges** — fold_list builds catalogs, grammar relation rows, readiness | Language models, `dag.dag`, `sql.dag` | Derived fold over declared registry edges once registry-as-data lands |
| **Host-transport row** — string equality on `authority_source_text` | `compiler/emit_host.dag` ↔ `languages/{rust,python,go}.dag` MVP pins | Structural `runtime_row` on `TargetModel` in language extdeps (dissolve-on noted in emit_host) |
| **Grammar serializer / round-trip law** — parse/print canonical law in compiler | `compiler/source_authority.dag` (imports extdeps dag LM) | Move canonical emission law beside `dag_language_model` in `extdeps/languages/dag.dag` |

### Compiler bridges consuming this slice (cross-slice synthesis)

- `compiler/emit_host.dag` — host-transport shim; `authority_source_text` pins (dissolve-on: `TargetModel.runtime_row`).
- `compiler/00_compile.dag`, `03_normalize.dag`, `03_resolve.dag`, `03_name_resolve.dag` — import dag language symbols; normalization/resolution logic stays in compiler.
- `compiler/source_authority.dag` — dag parse/print canonical law (candidate move: `extdeps/languages/dag`).
- `compiler/05_eval.dag` — imports `v4_evaluator` + `dag_round_trip_wave1_authorities_ready`.
- `workflow/bootstrap.dag`, `lens/leaf_model_verification.dag` — direct language model imports (expected for claims).

**Notes:** No `v3.dag` imports under `extdeps/`. No `combine_hash`/`content_hash` towers in `extdeps/` (those live in compiler per sweep #1).

## Focus: concerning patterns (unmodeled repetitive work)

### 1. CP-1b dual grammar/lex carrier (bridge)

- **Pattern:** Each language file maintains `FormalProduction` flat rhs *and* a parallel `GrammarExpr`/`LexRules` “operational parse scaffold until CP-1b convergence.”
- **Recurs:** `languages/{rust,python,go,typescript,java,kotlin,swift,ecmascript,dag,wasm,cpp}.dag`, `formats/spice.dag`.
- **Dissolves with:** Single bidirectional grammar carrier in `std/grammar.dag` + `std/lexing.dag` — operational types become projections, not second authorities.

### 2. TargetModel catalog towers (hand-rolled)

- **Pattern:** `List<TargetAtomRealization>`, signature catalogs, `insert_per_language_fact_bundle_entry` + fold-built registries, `target_value_template` / `serialize_source` rows.
- **Recurs:** `rust.dag` (largest), `python.dag`, `go.dag`, `typescript.dag`, `java.dag`, `kotlin.dag`, `swift.dag`.
- **Dissolves with:** Registry-as-data + `std/target_model` derived projection (catalog nodes from declared edges, not list literals + fold).

### 3. Wave/MVP readiness Bool conjuncts (bridge)

- **Pattern:** T-36 `dag_round_trip_wave1_authorities_ready` and per-axis lex/grammar/C5 Bool classifiers; eval gate in `05_eval.dag`.
- **Recurs:** `dag.dag`, `runtimes/v4_evaluator.dag`, `compiler/05_eval.dag`.
- **Dissolves with:** Typed readiness receipt projection (`std/projection`); eval reads one substrate receipt, not conjunctive Bool + embedded atoms.

### 4. Host authority source-text pins (bridge — leakage to compiler)

- **Pattern:** MVP `*_mvp1_source_text` in extdeps matched by string equality in `emit_host.dag` (`emit_host_*_authority_pin`).
- **Recurs:** `languages/{rust,python,go}.dag` ↔ `compiler/emit_host.dag`.
- **Dissolves with:** `TargetModel.runtime_row` (structural host row on language model; remove compiler string pins).

### 5. Per-target claim/diagnostic vocab in std (leakage from extdeps)

- **Pattern:** Rust/Python/Go diagnostic symbols and leaf-model claim IDs live in `std/` instead of language extdeps.
- **Recurs:** `std/rust_leaf_model_claim.dag`, `std/leaf_model_verification.dag` (consumed by `lens/leaf_model_verification.dag` importing extdeps).
- **Dissolves with:** Move claim/diagnostic namespaces into respective `extdeps/languages/*.dag`; std keeps only cross-target verification algebra.

### 6. Coercion pair as hand-wired predicate (hand-rolled)

- **Pattern:** Single MVP widening pair `rust i32 → python int` as explicit `PreservationPredicate` fn.
- **Recurs:** `coercion_widening.dag` only (today) — pattern will repeat per pair without substrate.
- **Dissolves with:** Widening registry-as-data in extdeps + `std/coercion` fold driven by registry lookup.

### 7. Formatter/tool option taxonomy explosion (hand-rolled)

- **Pattern:** Full vendor option space as coproduct types (especially clang-format).
- **Recurs:** `formatters/clang_format.dag` (🔴 by size), `rustfmt.dag`, `prettier.dag`, lighter in `black.dag` / `swift_format.dag`.
- **Dissolves with:** Config schema projection from vendor spec (patch/refinement substrate) rather than hand-enumerated style enums — or explicit “DeclaredNormalized” scope boundaries per formatter.
