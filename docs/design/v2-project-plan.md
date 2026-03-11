# v2 Self-Hosted Compiler: Project Plan

## Scope

Bootstrap subset only: compile `gist.dag` and its 5 transitive
dependencies (~1100 lines across 6 .dag files) to Rust source files.

This exercises: `module`, `import`, `type` (record, sum, alias with
refinements), `fn`, `func` (with `uses` clause), `service` (with
`config`, `operation`, `transport`, `response`, `mock_response`),
`resource` (with `capability`), `data`, expressions (let, match,
if/else, field access, call, lambda, string interpolation, binary
ops, record/list literals, `as` cast).

Does NOT exercise: `pattern`, `interface`, `pipeline`, `profile`,
`test`, `fixture`, `project`, `feature`, `task`, `design`,
`component`, `environment`, `param`, `extern asset`.

## Acceptance

```bash
# Level 1: v2 emits Rust that compiles
v2-compile gist.dag --backend rust --output /tmp/v2/
cd /tmp/v2/ && cargo build

# Level 2: v2 emits tests that pass (hermetic, mock-based)
cd /tmp/v2/ && cargo test

# Level 3: v2 primary output matches v1 (excludes v2-only test files)
v1-compile gist.dag --backend rust --output /tmp/v1/
diff <(find /tmp/v1 -name "*.rs" | sort | xargs cat) \
     <(find /tmp/v2 -name "main.rs" | sort | xargs cat)
```

---

## Task graph

```
C1 ──→ C2 ──→ C3 ──→ C4 ──→ C5 ──→ C6 ──→ I1

T1 (independent) ─────────────────────────────┘
```

C1–C6 are sequential (each consumes the previous stage's output
type). T1 runs in parallel with everything.

---

## C1: Core types

**File:** `src/v2/std/core.dag`
**Status:** 32 types defined, ~80% complete for bootstrap subset.

**Remaining gaps to close:**

1. Add `Primitive` variant to `TypeExpr`:
   ```dag
   | Primitive { name: String, span: SourceSpan }
   ```
   For kernel types (String, Int, Bool, Secret, Json, Unit) that
   have no structural expansion. After typecheck, `Named("String")`
   becomes `Primitive("String")`.

2. Add `uses` clause to FuncDef:
   ```dag
   | FuncDef { ..., uses: List<ResourceUse>, ... }
   type ResourceUse { name: String, resource: TypeExpr, span: SourceSpan }
   ```

3. Add `Cast` variant to Expr:
   ```dag
   | Cast { expr: Expr, target: TypeExpr, span: SourceSpan }
   ```

4. Expand ServiceDef with config/response/mock_response:
   ```dag
   type ServiceConfig {
     endpoint: Expr
     auth: Expr?
     rate_limit: Expr?
     retry: Expr?
   }
   ```
   Add `config: ServiceConfig?` to ServiceDef.
   Add `response: List<ResponseMapping>` and
   `mock_response: List<MockResponseDef>` to OperationDef.

5. Add default values to Param:
   ```dag
   type Param { name: String, type_expr: TypeExpr, default_value: Expr?, span: SourceSpan }
   ```

6. Expand service transport to include shell transport:
   ```dag
   | ShellBinding { argv: List<Expr>, env: List<EnvDef> }
   ```
   (Already partially done — needs `argv` for git commands.)

**Acceptance:**
- v1 compiler parses core.dag without errors
- Every construct used in gist.dag's 6 transitive deps has a
  v2 type representation
- No string-typed fields where structural types exist

**Depends on:** Nothing

---

## T1: Test infrastructure

**Deliverable:** Script that compiles a .dag file with v1, serializes
the AST/output, and compares against v2's output.

Two test types:
- **Equivalence:** v1 output == v2 output (bootstrap correctness)
- **Invariant:** v2 output satisfies spec properties that v1 may
  violate (no unresolved Named after typecheck, no fabrication, etc.)

Also: a small set of hand-authored semantic fixtures where v2 is
explicitly allowed to disagree with v1 (fail-open cases that v2
correctly rejects).

**Acceptance:** Framework runs on a trivial 1-function .dag file
**Depends on:** Nothing

---

## C2: Tokenizer

**File:** `src/v2/compiler/tokenize.dag`
**Status:** 11 functions, handles keywords/idents/ints/strings/
punctuation/operators/newlines/floats/comments/pipe. Missing:
string interpolation, escape sequences.

**Remaining gaps:**

1. String interpolation: `"hello {name}"` → `StrBegin("hello ")`,
   then the expression tokens, then `StrEnd("")`. Requires a
   depth stack to track nested `{`/`}` inside interpolations.

2. Escape sequences: `\"`, `\\`, `\n`, `\t` inside string literals.

3. Undefined primitives: `scan_while`, `scan_string_end`,
   `scan_to_eol`, `skip_horizontal_ws`, `char_at`, `substring`,
   `string_length`, `parse_int` — these are kernel-provided
   intrinsics that need to exist in the v1 evaluator.

**Acceptance:**
- Tokenize all 6 gist-dependency .dag files
- Token stream matches v1 tokenizer output (kinds + spans)
- `StrBegin`/`StrMid`/`StrEnd` emitted for interpolated strings

**Depends on:** C1 (Token/TokenKind types)

---

## C3: Parser

**File:** `src/v2/compiler/parse.dag` (not yet created)
**Status:** Not started.

Recursive descent, first-error-halt. Must parse:
- `module`, `import` (with structured bindings)
- `type` (Record, Sum, Alias with refinement predicates)
- `fn` (pure), `func` (with `uses` clause)
- `service` (with `config`, `operation`, `transport`, `response`,
  `mock_response`)
- `resource` (with `capability`)
- `data` declarations
- All Expr variants: let, match, if/else, field access, call,
  lambda, string interpolation, binary ops, record/list literals,
  `as` cast

**Acceptance:**
- Parse all 6 gist-dependency .dag files
- AST structure matches v1 parser (serialized JSON diff)
- **Invariant:** every TypeExpr is structural (Named/Product/etc),
  never a raw string

**Depends on:** C2 (token stream)

---

## C4: Module resolver

**File:** `src/v2/compiler/resolve.dag` (not yet created)
**Status:** Not started.

Resolves import references, builds dependency order, detects cycles.

**Acceptance:**
- Resolve gist.dag + 5 dependencies
- Import references resolve to correct modules
- Cycle detection works on synthetic circular import

**Depends on:** C3 (parsed modules)

---

## C5: Type resolver

**File:** `src/v2/compiler/typecheck.dag` (not yet created)
**Status:** Not started.

Resolves every `Named` TypeExpr to its structural form. Retains
nominal anchor (`name: Some("Span")`) alongside structure for
diagnostics and emission. Kernel primitives become
`Primitive { name: "String" }`.

**Invariant:** After typecheck, no unresolved `Named` references
remain. Every name is either `Primitive` (kernel types), a resolved
structural form, or a cycle-breaking `Named` that IS defined in
the module graph.

**Acceptance:**
- Resolve all types in gist.dag dependencies
- `ResourceHandle` → Product with 4 fields
- `CommitSha` → Refined { base: Primitive("String"), ... }
- No unresolved `Named` survives

**Depends on:** C4 (module graph with all type definitions visible)

---

## C6: Rust emitter

**File:** `src/v2/compiler/emit.dag` (not yet created)
**Status:** Not started.

Emits Rust source files AND Rust test files from the typed graph.
Scoped to gist.dag constructs:
- fn → Rust function
- func → workflow function with service call scaffolding
- type → Rust struct/enum
- data → Rust constant
- service operation → transport call code
- Expressions → Rust expression syntax

Emitted tests are hermetic: use `mock_response` from DSL source
as test fixtures. No network, no credentials.

**Acceptance:**
- Emit Rust for gist.dag
- `cargo build` succeeds on emitted output
- `cargo test` passes on emitted tests
- Primary output functionally equivalent to v1

**Depends on:** C5 (typed graph)

---

## I1: Pipeline integration

**File:** `src/v2/compiler/pipeline.dag`
**Status:** Skeleton with commented-out stages.

Wire C2–C6 with effectful driver split:
- **Driver** (effectful): `discover_files(root) → List<SourceFile>`
- **Compiler** (pure): `compile_sources(sources, backend) → CompileResult`

Every stage returns `StageResult<T> { value, diagnostics }`.

**Acceptance:** Full acceptance test passes (all 3 levels).
**Depends on:** C6, T1
