# gunbc

Onboarding guide for contributors and agents. Non-negotiable invariants, repo map, and pointers to canonical docs.

## Installation

```bash
# Clone and bootstrap
git clone <repo>
cd gunbc
make install
```

The `make install` target bootstraps the repository:
1. Generates CLI entrypoints (`gunbc-codegen --mode=ensure`)
2. Generates `Makefile` and `.gitignore` (`gunbc-bootstrap --mode=ensure`)

After installation, run `make help` to see all available targets.

**Note:** The generated `Makefile` is gitignored. The handwritten `GNUmakefile` provides the `install` target and delegates everything else to the generated `Makefile`.

## Start Here

- **`docs/start-here.md`** — **Read this first.** Repo orientation, feature development process, conventions, lessons learned, pre-flight checklist, acceptance criteria, design doc index.
- `docs/modeling.md` for DAG modeling pattern catalog (layer taxonomy, examples, anti-patterns)
- `docs/handbook.md` for the conceptual map, pattern catalog, and e2e examples (single file — copy-friendly)
- `docs/design/v4/dsl-design.md` for the full DSL language specification
- `docs/design/service-codegen.md` for DSL-driven service codegen architecture
- `SPEC.md` for the formal IR specification
- `docs/design/testgen.md` for test generation and proof obligations

## Quick Context

gunbc is a **DSL-first workflow compiler** where **everything is a DAG**. The primary authoring surface is the `.dag` language — declarative definitions that compile to a typed Graph IR. The compiler pipeline is: `.dag` → parse → typecheck → lower → emit (Rust/Go/C/MIPS). The core thesis is **moving contradiction discovery from runtime to static analysis** — if a DAG validates, its wiring is correct, its types are sound, and its execution intent is unambiguous. Every pipeline stage is a lossless semantic translation; stages must never silently drop data, invent defaults, or swallow errors. See `docs/design/compilation-pipeline.md` for the full pipeline map and architectural principles.

### Compositional modeling

Every external system is modeled as a **composition of layered concerns** (TCP → TLS → HTTP → REST → provider → operation), where each layer imposes invariants on the generated code. Workflows name only the top layer; the compiler composes all layers into transport code, mocks, and test obligations. DSL structural blocks and keywords are the mechanism — `transport rest { ... }` (transport class), `config { endpoint: ..., auth: ... }` (service config), `readonly`/`idempotent` (behavioral properties) — each adds constraints that compose additively. Where the Rust substrate currently hand-wires what the DSL can derive (credential chains, transport triplets, error classification), those are active consolidation targets. See `docs/handbook.md` § "Compositional Modeling Philosophy" and `docs/design/modeling/annotation-to-dag-modeling.md` for the full treatment.

## Repo Map

| Path | Purpose |
| --- | --- |
| `dsl/` | **Primary authoring surface** — all `.dag` source files |
| `dsl/services/` | Service definitions (REST, Shell): gcp, github, cargo, git, llm |
| `dsl/tools/` | Tool workflows: clippy, gist, codegen, makegen, etc. |
| `dsl/pipelines/` | Pipeline compositions: ci |
| `core/daglang/` | DSL compiler: parse → typecheck → lower → emit |
| `core/ir/` | Core IR types, patterns, transport model, resource system |
| `core/exec/` | Execution engine, DryRun interception, simulation |
| `core/codegen/` | CLI and test generation |
| `core/test/` | MockSpec and test utilities |
| `lib/transport/` | Canonical I/O boundary; a few bootstrap/generator crates do direct I/O by exception (see `TODO/TODONE/clippy-pragma-audit.md`) |
| `lib/tools/` | General-purpose tool wrappers (clippy, deps, gist) |
| `gunbc-dag/` | Repo-specific DAGs and CLI entrypoints (ci, makegen, codegen, testgen, bootstrap) |
| `docs/design/` | Design documentation |

## Refactor-Pressure Checklist (PR Gate)

- Single source of truth: new concepts must have exactly one authoritative definition.
- No stringly references: names of nodes/targets/resources must be typed or derived.
- No hidden env/IO: env vars, clock, platform, and FS handles only via env/resource nodes.
- No ambient globals: exec mode and policy flags are explicit inputs.
- Fast path declared: any freshness/check logic documents fast and slow paths.
- Generated code linting: fix IR or clippy config, never add `#[allow]` in generated output.
- Translation layers are total or error: if parser/lowerer/resolver can't represent something, raise a typed error with source location — never drop silently or substitute a default.
- No stubs that look like features: if a type/field/annotation exists, it must be wired end-to-end with at least one hard test, or deleted.
- Model negative space: new service operations must declare at least one error response (when `response {}` blocks ship). No happy-path-only models.
- Parse at the boundary, match exhaustively: new string-based dispatch → enum at intake, exhaustive match internally. `_ => default` in match on known variants is a smell.

## Invariants That Matter

- All runtime DAG world I/O happens through `TransportOps::Execute` nodes; build-time generators and a small set of bootstrap/config loaders are explicit exceptions (see `TODO/TODONE/clippy-pragma-audit.md`).
- Boundaries and entrypoints are inferred from unconnected ports.
- Tool handles are capability-based. When used, they flow through `tool:<id>` ports.
- Tool ports are excluded from user-facing workflow signatures.
- Errors are explicit; there are no silent fallbacks or warning-only failures.
- External systems are layered compositions — each DSL structural block (`transport rest { ... }`, `config { auth: ... }`, `response { ... }`) and keyword (`readonly`, `idempotent`) adds invariants that the compiler enforces in generated code, mocks, and tests.
- Generated files are never committed — the compiler extracts all output paths from `content_upsert` and `outputs` declarations, propagates them to the tool registry and `.gitignore`, and CI enforces that no generated file is tracked in git.

## Common Tasks

### DSL-first (primary path)

- **Add a new REST/Shell service:** Create `dsl/services/<provider>/<name>.dag` with `service` block and `operation` definitions. Identify the full layer stack (protocol, auth, provider, operation) and express each layer's invariants via structural blocks and keywords: `config { endpoint: ..., auth: ... }` (provider config), `transport rest { method: ..., path: ... }` or `transport shell { argv: [...] }` (transport class), `readonly`/`idempotent` (behavioral properties), `response { STATUS => TYPE }` (provider contract). Each block composes — the compiler generates transport code reflecting all layers.
- **Add a new tool workflow:** Create `dsl/tools/<name>.dag` — import services, define `fn` (pure) and `func` (effectful) blocks. Use `uses` declarations for resource/capability requirements — the compiler resolves them transitively.
- **Add a new pipeline:** Create `dsl/pipelines/<name>.dag` — import tools, define `pipeline` block with `stage` dependencies.

### Framework internals (rare)

- Add a new pattern: `core/ir/src/patterns/` and `core/ir/src/patterns/mod.rs`.
- Add a new transport: `core/ir/src/transport/` plus executor support in `lib/transport/`.
- Extend the emit pipeline: `core/daglang/daglang-emit/src/` (add `service_emit` functions per backend).

## Testing

```bash
cargo test
cargo test -p gunbc-ir
cargo test -p gunbc-ir -- --nocapture
```

## Related Projects

The `the-gunbai` repo contains the original design rationale, long-form design docs referenced by `SPEC.md`, and the **Understanding pattern** — the compositional modeling system that gunbc's DSL annotations are inspired by. Key inspirational patterns from gunbai:

- **Understanding = structured data about external systems** — behaviors, constraints, assumptions, unknowns, and explicit dependencies. Each Understanding composes with others via `depends_on` with behavior-scoped resolution.
- **Automatic derivation** — behavioral properties (`ReadOnly`, `Idempotent`, `FailsWhen`) automatically generate block I/O, mock specs, and contract tests. No manual `TestgenTargetDef` per target.
- **Layered semantic composition** — REST depends on HTTP; curl depends on network/DNS/TCP. Each layer overrides or extends the layer below. Transitive dependency resolution is automatic.
- **LanguageUnderstanding** — language-agnostic specs mapped to Rust/Python/TypeScript via structured type/syntax/naming tables. Same generator, multiple backends.
- **External dependency modeling** — tools declare runtime requirements (`uses net: Network`) and the system resolves prerequisites transitively.

gunbc's DSL achieves ~80% of this via interface contracts, structural block composition (`transport`, `config`, `response`), and `uses` declarations. The remaining gap is in the Rust substrate, where graph builders hand-wire what the DSL could derive. Active consolidation lanes target this gap (see `tasks.md`).

---

## Appendix: DSL Language Reference

Comprehensive reference for `.dag` language features. Each entry includes syntax and a proven-in location from the codebase.

### Declaration Forms

| Form | Syntax | Example |
|------|--------|---------|
| `module` | `module path.segments` | `module std.box_draw` |
| `import` | `import path` or `import path { Name1, Name2 }` | `import std.render { SemanticColor, Span }` |
| `type` (record) | `type Name { field: Type, field2: Type? }` | `type BoxChars { top_left: String, ... }` |
| `type` (sum) | `type Name = A \| B { field: Type } \| C` | `type BoxStyle = Closed \| OpenRight` |
| `type` (alias) | `type Alias = Base @annotation` | `type FilePath = String @non_empty` |
| `data` | `data name: Type = value` | `data unicode_box_chars: BoxChars = { top_left: "╭", ... }` |
| `fn` | `fn name(p: T) -> R { body }` | `fn box_chars_for_tier(tier: Tier) -> BoxChars { ... }` |
| `func` | `func name(p: T) -> { out: R } uses r: Res { body }` | `func clippy_lint(...) -> { clean: Bool } uses clippy: Clippy { ... }` |
| `pattern` | `pattern name(p: T) -> { out: R } uses r: Res { body }` | `pattern file_content_matches(path: String, ...) -> { matches: Bool }` |
| `extern func` | `extern func name(inputs) -> { outputs }` | `extern func render_tree(paths: List<String>) -> { return: String }` |
| `extern asset` | `extern asset name: Type` | Declares externally-provided static values |
| `service` | `service Provider.Name { operation Op { ... } }` | `service github.Gist { operation Create { ... } }` |
| `resource` | `resource Name { kind: K, capability name { ... } }` | `resource Clippy { kind: Capability, ... }` |
| `interface` | `interface Name { capability name(...) -> { ... } }` | `interface IssueProvider { capability discover(...) -> { ... } }` |
| `pipeline` | `pipeline name { stage s1 { } stage s2 [after s1] { } }` | `pipeline ci { stage codegen { } ... }` |
| `test` | `test name { mock ... expect ... }` | `test clippy_lint_all { ... }` |
| `fixture` | `fixture name { mock ... }` | `fixture deps_env { mock fs_env.handle -> "write_scope" }` |
| `profile` | `profile name { bind Interface { impl: Impl, config: ... } }` | Provider binding for deployment profiles |
| `uses` | `uses binding: ResourceType(key: val)` | `uses fs: Filesystem(mode: Read)` |
| `provides` | `provides binding: ResourceType` | `provides auth: AuthContext` |

### Type System

**Primitives**: `String`, `Int`, `Bool`, `Float`, `Bytes`, `Secret`, `Unit`, `Json`

**Generic containers**: `List<T>`, `Map<K, V>`, `Option<T>` (or `T?` sugar)

**Records** — labeled products with optional defaults:
```
type Config { name: String, retries: Int = 3, label: String? }
```

**Sum types** — tagged disjoint unions with optional payloads:
```
type Result = Ok { value: String } | Err { error: String }
type BoxStyle = Closed | OpenRight
```

**Refinement types** — aliases with constraint annotations:
```
type CommitSha = String @pattern("^[a-f0-9]{40}$")
type RetryCount = Int @range(min: 1, max: 5)
type NonEmptyStr = String @non_empty
type GistId = String @format(uuid)
type Char = Int @brand("Char") @range(min: 0, max: 1114111)
```

**Function types** as parameters: `fn(T) -> R` — e.g., `should_act: fn(Check.Output) -> Bool`

**Generics** (on types, fns, interfaces): `type Result<T, E> = Ok { value: T } | Err { error: E }`

### Pipe Methods

Methods invoked via `expr |> method(args)` or as part of chains.

**Collection operations** (List → List):
| Method | Signature | Example |
|--------|-----------|---------|
| `map` | `List<T> \|> map(f: T -> R) -> List<R>` | `items \|> map(i => "- {i}")` |
| `filter` | `List<T> \|> filter(f: T -> Bool) -> List<T>` | `stages \|> filter(s => s.success)` |
| `filter_map` | `List<T> \|> filter_map(f: T -> R?) -> List<R>` | `labels \|> filter_map(l => label_to_stage(label: l))` |
| `flat_map` | `List<T> \|> flat_map(f: T -> List<R>) -> List<R>` | `items \|> flat_map(v => [v])` |
| `sort_by` | `List<T> \|> sort_by(f: T -> K) -> List<T>` | `items \|> sort_by(item => key_fn(item))` |
| `append` | `List<T> \|> append(items: List<T>) -> List<T>` | `acc \|> append(items: [span])` |

**Aggregation** (List → scalar):
| Method | Signature | Example |
|--------|-----------|---------|
| `fold` | `List<T> \|> fold(init: A, f: (A, T) -> A) -> A` | `chars \|> fold(init: { result: "" }, f: (acc, c) => ...)` |
| `join` | `List<String> \|> join(sep: String) -> String` | `items \|> join("\n")` |
| `count` | `List<T> \|> count() -> Int` | `stages \|> count()` |
| `sum` | `List<Int> \|> sum() -> Int` | `widths \|> sum()` |
| `first` | `List<T> \|> first() -> T?` | `matches \|> first()` |
| `last` | `List<T> \|> last() -> T?` | — |
| `max_by` | `List<T> \|> max_by(f: T -> K) -> T?` | `stages \|> max_by(s => stage_ordinal(stage: s))` |
| `any` | `List<T> \|> any(f: T -> Bool) -> Bool` | `blocks \|> any(predicate: b => ...)` |
| `all` | `List<T> \|> all(f: T -> Bool) -> Bool` | `stages \|> all(s => s.success)` |
| `contains` | `List<T> \|> contains(item: T) -> Bool` | `codepoints \|> contains(item: cp)` |

**String methods**:
| Method | Signature | Example |
|--------|-----------|---------|
| `starts_with` | `String \|> starts_with(prefix: String) -> Bool` | `l \|> starts_with(prefix: "sdlc:")` |
| `ends_with` | `String \|> ends_with(suffix: String) -> Bool` | — |
| `repeat` | `String \|> repeat(n: Int) -> String` | `"#" \|> repeat(level)` |
| `replace_section` | `String \|> replace_section(section, replacement) -> String` | `template \|> replace_section("section", content)` |
| `chars` | `chars(s: String) -> List<Char>` | `chars(s: text) \|> map(...)` |

**Conversion methods**:
| Method | Signature |
|--------|-----------|
| `to_bytes` | `String \|> to_bytes() -> Bytes` |
| `to_json` | `T \|> to_json() -> Json` |
| `hash` | `T \|> hash() -> String` |

### Control Flow

**if/else**: `if cond { expr } else { expr }` — branches are expressions, return values.

**match** — pattern matching on sum types, strings, bools, null, integers:
```
match tier { Emoji => entry.emoji, Ascii => entry.ascii, _ => entry.unicode }
match label { "sdlc:idea" => Idea, _ => None }
match config.color { null => default, c => c }
match opt { Some(v) => v, None => fallback }
```

**for** — map sugar for effectful iteration:
```
details = for issue in discovered.issues { issues.get(id: issue.id) }
```

**lambda**: `param => expr` or `(p1, p2) => expr`
```
items |> map(item => "- {item}")
chars |> fold(init: { result: "" }, f: (acc, c) => acc with { result: acc.result + c })
```

**let** binding: `let base = string_display_width(s: title) + 10`

**return**: `return { field: expr }` — explicit output record from func/fn bodies.

**pipe operator** `|>`: chains method calls — `items |> map(f) |> filter(g) |> join("\n")`

**node guards**: `node action [after check, when should_act(check)]: Op` — dependency + conditional.

**record update**: `artifact with { status: InProgress }` — functional update of record fields.

**type cast**: `result as Url` — refine to narrower type.

### Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `%` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical | `&&`, `\|\|`, `!` |
| Null | `??` (coalesce), `?.` (optional chain) |
| String | `+` (concatenation) |

### String Features

**Interpolation**: `"{expr}"` — curly braces, no `$` prefix. Supports field access: `"{issue.id}"`.

**Concatenation**: `left + right` — string `+` operator.

**Escape sequences**: `\n`, `\t`, `\x1b[0m` (ANSI), `\\`.

### Service Blocks and Keywords

Transport, auth, and behavioral properties are expressed via **structural blocks and keywords**, not `@` annotations:

**Transport blocks** (on operations):
```
transport rest { method: POST, path: "/gists" }
transport shell { argv: ["cargo", "build", "--all-targets"] }
transport file { op: PROBE, path: "{path}" }
```

**Service config** (on services):
```
config { endpoint: "https://api.github.com", auth: BearerToken, auth_input: auth_token }
```

**Behavioral keywords** (on operations): `readonly`, `idempotent`, `hermetic`

**Contract declarations** (on interfaces): `contract get(id) after create(title, body, labels) => { found: true }`

### Annotations

Annotations (`@` prefixed) are used for **type refinement** and a few test/output markers:

**Refinement**: `@pattern("regex")`, `@range(min: N, max: N)`, `@non_empty`, `@format(uuid)`, `@brand("Name")`, `@content(Text)`, `@where(predicate_fn)`

**Test config**: `@tier(Unit)`, `@auto_mock(true)`, `@testgen_skip(true)`

**Output tracking**: `@outputs("glob")` — declares generated file paths for `.gitignore` and drift detection.

> **Note**: `docs/design/modeling/annotation-to-dag-modeling.md` tracks the full annotation census
> and migration plan. Several annotations (e.g., `@error_map`, `@retry`, `@requires`) are
> Category 2: declared intent with no enforcement. These are being migrated to structural
> DAG modeling per the compositional modeling philosophy.

### Known Limitations

Features NOT yet supported (see `tasks.md` FC-CF for status):

| Gap | Business Case | Workaround |
|-----|--------------|------------|
| `split(delim)` | Path parsing for tree rendering (e.g., `"a/b/c"` → `["a","b","c"]`) | Extern bridge (`render_tree`) |
| `zip()` | Parallel list assembly for snapshot content | Extern bridge (`build_snapshot_content`) |
| Recursive types | Self-referential structures (directory trees: `DirEntry { children: List<DirEntry> }`) | Extern bridge |
| Recursive functions | Tree traversal (flatten, render) | Extern bridge |
| `skip(n)` | Drop first N elements | Expressible via `fold` with index tracking |
| `enumerate()` | Index-aware iteration | Expressible via `fold` with counter accumulator |
