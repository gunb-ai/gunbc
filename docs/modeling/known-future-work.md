## Known future work

### P0 — Correctness (fabrication fallbacks)

The v2 compiler inherits v1's worst anti-pattern: silent defaults on
lookup miss. These mask real errors that cascade into wrong generated code.

**04_typecheck.dag** (21+ instances):
- `lookup_field_type` → `unit_type()` on missing field or non-Product type
- `infer_method_call_type` → `unit_type()` for unknown methods
- `infer_expr` wildcard `_ =>` → `unit_type()` for unhandled expression types
- Missing RecordLit type → placeholder `Named` instead of error

**05_emit.dag**:
- Anonymous products/coproducts → `serde_json::Value` (silent structure erasure)
- `emit_data_value_json` wildcard `_ => "null"` (variables/calls silently become null)
- `extract_service_name` → `"Unknown"` string on fail
- `.expect("valid data definition")` swallows JSON parse errors

**03_resolve.dag**:
- `find_index_in_list` → `-1` sentinel (should be Optional)
- `get_at_index_int` → `0` for missing in-degree (**corrupts topo sort**)
- `get_at_index` → `[]` for out-of-bounds (hides missing data)

**Fix:** Return `Optional` or `Result` types. Propagate `None` + diagnostic.

### P1 — Sustainability (structural modeling)

**Generic Result type** — highest leverage single change:
- 02_parse.dag defines **57 bespoke result types**
- 04_typecheck.dag defines **13+ more**
- All follow `{ value: T, state/diagnostics }` — one generic eliminates 70 types

**Structural token matching** — 02_parse.dag:
- **48+ uses of `kind_tag()`** extracting string from TokenKind, comparing with `==`
- Should be direct pattern matching on TokenKind variants

**Keyword table duplication** — between 01_tokenize.dag and 02_parse.dag:
- `keyword_to_name` (28 if-else clauses) + `keyword_to_arg_label` (23 more)
- Duplicates the tokenizer's `data keywords` table — derive from single source

**Method/predicate dispatch** — string chains that should be enums:
- `infer_method_call_type`: 12-branch if-else on string method names
- `parse_single_predicate`: 8 string matches on predicate names
- `emit_method_call`: hardcoded method name → Rust method mapping
- `emit_primitive_type` / `needs_reference` / `is_primitive_numeric`: string checks on type names

### P1 — Dummy sentinel values (02_parse.dag)

**20+ dummy node constructions** — empty string names, null spans as
error recovery. `Field { name: "", type_expr: Named { name: "" } }` is
an invalid state that should be unrepresentable. Downstream can't
distinguish "valid empty" from "error recovery." Fix: return Result,
never construct invalid AST.

### P2 — Anemic types and missing structure

**ParserState** (02_parse.dag): missing filename, module context,
recovery hints. Error diagnostics have `module_name: none`.

**Wildcard import**: both parse.dag and resolve.dag use `"*"` string
sentinel. Should be `ImportNames = All | Specific { names: List<String> }`.

**Resolve exports**: variant names conflated with type names — should
separate `ExportedNames { types, constructors, functions }`.

**Pipeline result types** (06_pipeline.dag): `ParseResult` has
independent `module?` and `error?` — should be sum `Ok | Err`.

### P2 — DSL std cleanup

- Reconcile `ContentEncoding` (types.dag) with `Encoding` (encoding.dag) — M2
- `containers.dag`: skeletal, no type definitions — define or delete
- `fidelity.dag`: wildcard `_ => Xl` (silent fallback), cost mappings lack justification
- `fermi.dag`: timeout data duplicated as both `data` and function body
- `render.dag`: `RenderMode` enum is dead code
- `symbols.dag`: `resolve_symbol` returns empty string on miss
- `types.dag`: GCP type duplicates, policy defaults embedded in types

### P2 — Extdeps improvements

- `github.dag`: should import Git types for shared concepts (branches, commits)
- `github/auth.dag`: minimal, magic string, no composition
- ~~`llm/llm.dag`: `LlmMessage.content` as String~~ — DONE: `List<ContentBlock>` with multimodal blocks
- String fields that should be enums: `AuthConfig.scheme`, ~~`ThinkingConfig.type`~~ (DONE),
  `CacheControl.type`, `GitRemote.fetch_refspec`, pagination cursors, `GistFile.language`
- Stale hardcoded data: model lists (anthropic, openai), GCP regions

### Accepted debt (dies with self-hosting)

Rust-specific constructs in 05_emit.dag (S81 from SUSTAINABILITY.md):
- `#[derive(...)]` and `#[serde(tag = ...)]` attributes
- Primitive type → Rust type mapping (Int→i64, etc.)
- `NonEmptyVec<T>`, `NonEmptyBTreeSet<T>` as raw Rust strings
- Hardcoded reqwest and std::process::Command
- `serde_json::json!(...)` macro

Not worth fixing — the v2 emitter replaces all of this.

---

