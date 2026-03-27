# Tasks (warm-hen-138 / Stream 2: Expression Model & Frontend)

L1 Type Dissolution is on the `l1-type-dissolution` branch. Everything
below is non-L1 work.

## Stream 2: Expression Model & Frontend

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P5.1 | Token coherence: `TokenShape` replaces `TokenKind` | Planned | ~507 lines in `01_tokenize.dag`. Shape/payload separation. |
| P5.5 | Residual semantic enum cleanup | **Done (assessment)** | Language-construct enums stay. IntrinsicMethod/RuntimeBridgeMethod dissolve when method algebras/language extdeps land (blocked on L1). Minor: `LookupCallSemantics` never consumed, `LocalValueBinding` never matched, 3 RuntimeBridgeMethod pairs potentially redundant. |
| P5.12 | ExprData tag dissolution assessment | **Done** | Verdict: RETAIN. 143 match sites, 160+ constructions, zero string comparisons. Exhaustiveness guarantee essential. Post P5.11, ExprData is pure operator metadata — the right design. |
| — | Statement/expression emit classification | Planned | Python (and Go) are statement-oriented; emit assumes expression-orientation. Three symptoms: `return let`, broken `functools.reduce`, `return match`. Fix: pre-emit metadata tags bindings vs tail expression. Relates to FO-* and P5.11. |

## In-Progress

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P2.6 | `04_infer.dag` decomposition | In progress | Target: under 1500 lines. Sub-files extracted; remaining: shrink `04_infer.dag` itself. |

## Fan-Out Preservation

| ID | Item | Status | Notes |
|----|------|--------|-------|
| FO-3 | v1 emitter rendering audit | **Done** | 5 patterns: (1) unconditional TCO param cloning ~24 sites, (2) .clone() before read-only ops 65+ sites, (3) Rc::try_unwrap clone fallback 14 sites, (4) Rc::new vec slices 4 sites, (5) .clone().iter() loops 5 sites. Root cause: no fan-out tracking. v2 emitter already avoids pattern 3. |
| FO-4 | v2 emitter fan-out fact | **Done (computation)** | `binding_fan_out` added to ownership.dag. Reuses existing walk_expr + branch-aware merge. Remaining: wire into emit pipeline so Rust emitter reads fan-out for Rc decisions. |
| FO-5 | Fan-out preservation ratchet | Open | Blocked on FO-4 emit wiring. Count `.clone()` where fan-out=1. Target: 0. |

## Phase 4 Quality (not blocking)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| — | Go `interface{}` type holes (13 sites) | Blocked on L1 | Root cause: I-15. Needs P1.5 (containers structurally complete). |
| — | Go `// unhandled node` wildcard | **Done** | Changed to `panic()` in init function. |
| — | Python `_unimplemented()` (2 sites) | **Done** | Replaced with `emit_simple_expr` (proper value emission). |

## Test Generation Parity

All three backends (Rust, Go, Python) have structurally identical test generation:
- Shared `extract_test_projections` (single graph-walk entry point in `05_emit.dag`)
- Shared `emit_simple_expr` for mock value rendering
- Per-backend test file emission (`emit_test_file` / `emit_go_test_file` / `emit_py_test_file`)
- Per-backend operation test (`emit_operation_test` / `emit_go_operation_test` / `emit_py_operation_test`)
- Verification tests exist for all three (`rust_emit_generates_mock_test_file`, `python_emit_generates_mock_test_file`, `go_emit_generates_mock_test_file`)

Tests generate when service operations have `mock_response` blocks (e.g., GCP IAM, Secret Manager, GitHub API extdeps all have mock data).

| ID | Item | Status | Notes |
|----|------|--------|-------|
| TG-7 | Rust tests call operation with mock data | **Done** | Service instantiated with DryRunMode(true), operation called, result asserted Ok. |
| TG-8 | Go tests instantiate service | **Done (partial)** | Service struct instantiation added. Full invocation blocked on Go dry-run support. |
| TG-9 | Python tests instantiate service | **Done (partial)** | Service class instantiation added. Full invocation blocked on Python dry-run support. |
| TG-5 | Go test file syntax gate | Planned | `go vet` on emitted test files. |
| TG-6 | Python test file syntax gate | Planned | `ast.parse` on emitted test files. |
| — | Go/Python dry-run support | Planned | Prerequisite for full invocation tests in Go/Python. Rust already has DryRunMode. |

## Backlog

| ID | Item | Status | Notes |
|----|------|--------|-------|
| — | Anonymous record target resolution | Planned | R2 stopgap (`compile_error!` for index >= 4). Real fix: proper field access for any arity. |
| — | Generated self-hosting tests and stage contracts | Planned | Valuable once compile contract settles. |
| — | TCO backend contract cleanup | Planned | Stream 2. Formalize trampoline/TCO contract across backends. |
| — | SCC-aware return type resolution | Planned | Not blocking bootstrap. |
| — | `assemble_stage0` fixups (5 known issues) | Planned | Stream 2. 5 manual corrections per stage0 regeneration. |
| — | Full linear type checking | Planned | Ownership proof work started; full proof beyond current migration. |

## Review Follow-ups

| ID | Item | Status | Notes |
|----|------|--------|-------|
| F4 | Parser `item_kind` verification | **Done** | All data defs in dsl/ and src/v2/ have type annotations. No misclassification possible. |
| F7 | Data constants `.clone()` from `lazy_static` | **Done** | 53 occurrences in stage0. All String values (not Copy). Stage0 at 6.47s — no bottleneck. Will change when v2 replaces stage0 generation. |
