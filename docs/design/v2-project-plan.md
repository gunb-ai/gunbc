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

## C1: Core types — DONE

**File:** `src/v2/std/core.dag`
**Status:** Complete. 40+ types defined covering the full bootstrap subset.

All previously listed gaps are implemented: Primitive, ResourceUse,
Cast, ServiceConfig, MockResponseDef, Param.default_value,
ShellBinding, TypeApp, Domain predicate, FieldBinding.

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

## C2: Tokenizer — DONE

**File:** `src/v2/compiler/tokenize.dag`
**Status:** Complete. ~470 lines, 20+ functions. Handles keywords,
identifiers, integers, floats, strings, string interpolation
(StrBegin/StrMid/StrEnd with depth tracking), escape sequences,
punctuation, operators, newlines, comments. All kernel intrinsics
(scan_while, char_at, substring, etc.) implemented in v1 evaluator.

Verified: tokenize → evaluate chain works E2E (Phase 2 tests).

**Depends on:** C1 (Token/TokenKind types)

---

## C3: Parser — DONE

**File:** `src/v2/compiler/parse.dag`
**Status:** Complete. ~2500 lines, full recursive descent parser.
Handles all listed constructs. First-error-halt.

Verified: tokenize → parse chain works E2E (Phase 3 test on
`"module test"` fixture).

**Remaining work:** response/mock_response blocks not yet parsed
(OperationDef fields left empty). See Wave 4 in review follow-up.

**Depends on:** C2 (token stream)

---

## C4: Module resolver — DONE

**File:** `src/v2/compiler/resolve.dag`
**Status:** Complete. ~466 lines. Kahn's algorithm for topological
sort, import resolution, cycle detection, exported name validation.

**Depends on:** C3 (parsed modules)

---

## C5: Type resolver — DONE

**File:** `src/v2/compiler/typecheck.dag`
**Status:** Complete. ~1000 lines. Resolves Named→structural types,
builds type environments per module, kernel type env, recursive
cycle-breaker detection, post-typecheck validation.

Cross-stage types aligned with resolve.dag (Wave 1). Diagnostic
threading fixed (Wave 3). Cycle-breaker validation corrected.

**Depends on:** C4 (module graph with all type definitions visible)

---

## C6: Rust emitter — DONE

**File:** `src/v2/compiler/emit.dag`
**Status:** Complete. ~1100 lines. Emits Rust source files from
typed graph: structs, enums, functions, services, resources, data
constants, test files. Handles type expression emission, pattern
matching, string interpolation, binary ops, lambdas.

**Remaining work:** emitted tests don't invoke operations or
assert (Wave 4). No Cargo.toml emission (Wave 8).

**Depends on:** C5 (typed graph)

---

## I1: Pipeline integration — DONE

**File:** `src/v2/compiler/pipeline.dag`
**Status:** Complete. Wires tokenize → parse → resolve → typecheck
→ emit. Backend dispatch (Rust/Python). Resolver + typechecker +
parse diagnostics threaded to output.

Wire C2–C6 with effectful driver split:
- **Driver** (effectful): `discover_files(root) → List<SourceFile>`
- **Compiler** (pure): `compile_sources(sources, backend) → CompileResult`

Every stage returns `StageResult<T> { value, diagnostics }`.

**Acceptance:** Full acceptance test passes (all 3 levels).
**Depends on:** C6, T1
