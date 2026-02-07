# Integration Test Generation: Repo-Wide Contracts

**Status**: Draft — February 2026  
**Owner**: Unassigned

## Problem

We recently shipped a mismatch between repeatable CLI flags and Makefile
invocation. The tool registry declared `extensions` as repeatable, the CLI parser
consumed one value per `--extensions`, but Makefile generation always emitted a
single `--extensions $(EXT)` expansion. Multi-value `EXT` silently dropped values.

Current tests only validate that flags appear in the Makefile. There are no
contract tests that assert "registry semantics" match "Makefile wiring" and
"CLI parsing" end-to-end. This is exactly the kind of integration drift that
should be caught automatically.

## Goals

1. Detect mismatches between tool registry entrypoints and Makefile rendering.
2. Validate CLI parsing for repeatable inputs in `--dry-run` mode without
   running any external tools or network calls.
3. Keep tests fast, hermetic, and non-flaky.

## Non-goals

1. Full end-to-end tests that run git, gh, or external services.
2. Verifying tool runtime logic beyond CLI input parsing.
3. Replacing existing testgen for DAG semantics.

## Sources of Truth

1. `gunbc_codegen::registry::all_tools()` (entrypoint metadata + cardinality).
2. Makefile renderer output from `gunbc-dag/src/makegen/render.rs`.
3. CLI `--dry-run` output, specifically the "Inputs:" block emitted by
   generated CLIs.

## Design Overview

### Tier 0: Static Makefile Contract Tests

Add registry-driven tests in `gunbc-dag` that render the Makefile and validate
the CLI argument shape per entrypoint.

Rules:

1. Repeatable entrypoints render as repeated flags using `$(foreach ...)`.
2. Non-repeatable entrypoints render as `$(if $(VAR),--flag $(VAR))`.
3. Help text for repeatables uses the `VAR=... ...` convention.

These tests are fast, deterministic, and catch the exact failure mode we saw.

## Test Taxonomy + Fermi Cost

We need a first-class model for test class and cost so integration tests can be
generated without spamming CI or requiring secrets by default.

### Test Classes

1. **Unit**: Pure logic, no transport boundaries, no external I/O.
2. **Hermetic**: Uses transport boundaries, but only in DryRun/MockSpec.
3. **Integration**: Executes Real mode or hits actual external systems.

### Fermi Cost (Opt-In)

Discrete cost buckets to gate expensive tests:

`XS`, `S`, `M`, `L`, `XL`

Default proposal:

1. Unit tests default to `XS`.
2. Hermetic tests default to `S`.
3. Integration tests default to `M` (opt-in).

### Inference + Overrides

Inference rules (initial heuristic, can be tuned later):

1. No transport nodes → `Unit`.
2. Transport nodes + DryRun only → `Hermetic`.
3. Real-mode execution or external dependencies → `Integration`.
4. Cost bump based on transport types:
   - File system → `S`
   - Shell (local) → `M`
   - Git → `M`
   - HTTP/REST → `L`
   - LLM providers → `XL`
   - Credentials/Secrets required → +1 tier

Overrides:

1. Allow per-target overrides in `#[testgen_target(...)]`.
2. Allow global overrides via env (`GUNBC_TEST_MAX_COST`, `RUN_LIVE_INTEGRATION`).

### Default Gating (Proposed)

1. `make test` runs everything at or below `S` (default).
2. `make test-all` sets `GUNBC_TEST_MAX_COST=XL` and `RUN_LIVE_INTEGRATION=1`.
3. Live tests still require required secrets to be present in the environment.

Tests above the max cost should **skip** with a clear reason, not fail.

### Tier 1: CLI Dry-Run Contract Tests

Generate integration tests per tool crate that run the tool binary in
`--dry-run` mode and assert that the printed inputs reflect the provided args.

Example for `gunbc-gist`:

1. Run `gunbc-gist --dry-run --extensions .rs --extensions .toml`.
2. Capture stdout.
3. Assert the `extensions` input prints `[".rs", ".toml"]`.

These tests stay hermetic because `--dry-run` uses mocks and does no I/O.

### Tier 2: Machine-Readable Input Echo (Optional)

If parsing human-readable output becomes brittle, add a new CLI flag:

`--print-inputs json`

This emits a JSON object for inputs, making tests stable and easy to parse.

## Test Generation Strategy

Phase 1 (Manual Test Harness):

1. Add a test module in `gunbc-dag/tests/cli_contract.rs` to validate Makefile
   rendering across all registry tools.
2. Add a single dry-run CLI test in `lib/tools/gist/tests/cli_contract.rs`.

Phase 2 (Generated Tests):

1. Extend codegen to emit per-tool CLI contract tests under
   `lib/tools/<tool>/tests/generated_cli_contract.rs`.
2. Use entrypoint metadata to generate argument lists and expected input prints.

## CI Integration

1. Tier 0 tests run as part of normal `cargo test -p gunbc-dag`.
2. Tier 1 tests can be gated behind a dedicated target, e.g. `make cli-contract`,
   and later added to CI once runtime cost is understood.

## Acceptance Criteria

1. A repeatable entrypoint rendered without `$(foreach ...)` fails Tier 0 tests.
2. A repeatable CLI flag that does not round-trip through `--dry-run` fails Tier 1.
3. The test suite runs without network access and without calling external tools.

## Open Questions

1. Should `--print-inputs json` be added now or only if output parsing becomes
   brittle?
2. Which tools are the initial Tier 1 targets beyond `gist`?
3. Should Tier 1 tests be opt-in (`make cli-contract`) or included in `make test`?

---

## Current Coverage Inventory (Repo-Wide)

### Generated Integration Tests (gunbc-testgen)

Generated by `gunbc-testgen` from `#[testgen_target]` in `graph_mock.rs`.

Targets:

1. `gunbc-dag`: `bootstrap`, `ci`, `makegen`, `pragma`, `testgen_dag`
2. `lib/tools/gist`: `snapshot`, `diff`, `recent`
3. `lib/tools/deps`: `deps`
4. `lib/review`: `inline`, `diff`
5. `lib/llm-ops`: `openai`, `anthropic`, `code-review`, `secrets`, `credential-lifecycle`

What they validate:

1. DryRun completion (smoke test)
2. Boundary interception (unless `no_boundary_tests`)
3. Optional input handling (missing vs wrong-type)
4. Flow tests for `flow_tests` targets (success/failure paths)
5. Resource simulation (MockSpec includes mocks for declared resources)
6. Signature checks (when `signature = ...` is provided)

What they do NOT validate:

1. CLI flag parsing and CLI ↔ DAG input mapping
2. Makefile ↔ CLI argument mapping
3. Real external I/O (all transport boundaries mocked)
4. LLM live integration (OpenAI/Anthropic API calls)
5. End-to-end tool behavior beyond DryRun

### Manual Integration Tests

1. `lib/tools/gist/tests/integration.rs`
2. `lib/transport/src/executor.rs` (real git integration tests)
3. `lib/transport/src/executor.rs` (real file/shell dispatch tests)
4. `gunbc-dag/tests/tool_registration.rs` (registry consistency)
5. `gunbc-dag/tests/mock_spec_registration.rs` (MockSpec coverage)

---

## External Reliance Matrix

Legend:

1. **Mocked** = covered only by DryRun / MockSpec tests
2. **Real** = tests execute real external operations
3. **None** = no integration coverage

| External Dependency | Where Used | Current Coverage | Gaps |
|---|---|---|---|
| Filesystem read/write | bootstrap, makegen, pragma, deps, gist | Real (transport file tests), Mocked (tool DAGs) | No tool-level fs integration tests |
| Shell / Cargo / Clippy | ci, build workflows | Real (transport shell tests), Mocked (tool DAGs) | No real toolchain integration tests |
| Git (ls-files, diff, rev-list) | gist, review, transport | Real (transport only), Mocked (gist/review) | No end-to-end tool tests using real git |
| HTTP / REST | LLM providers, GitHub API | Mocked | No local or live HTTP integration |
| GitHub API / gh | gist | Mocked | No live GitHub integration |
| LLM providers (OpenAI/Anthropic) | llm-ops, review | Mocked | No live LLM integration |
| Clock / Time | gist, others | Mocked | No real time-based integration |
| Credentials / Secrets | llm-ops, review | Unit tests only | No live credential usage |
| Locks / Leases | ci, llm-ops, deps | Mocked | No contention or concurrency integration |
| CI integrations (GitHub Actions) | ci graph | Mocked | No integration with actual CI systems |

---

## Gap Analysis

1. CLI ↔ DAG contract has no generated coverage.
2. Makefile ↔ CLI contract has no generated coverage.
3. External integrations (LLM, GitHub, filesystem, shell tools) are almost entirely mocked.
4. Only git transport has true integration tests, and those are transport-level, not tool-level.
5. LLM integration depends on mock responses only; no live API validation.

---

## Roadmap: Integration Test Generation (Tiered)

### Tier 0: Static Contract Tests (Fast, Hermetic)

1. Render Makefile and validate per-entrypoint argument shape.
2. Validate repeatable params use `$(foreach ...)`.
3. Validate help text includes repeatable hint (`VAR=... ...`).

### Tier 1: CLI Dry-Run Contract Tests (Hermetic)

1. Generate `--dry-run` CLI tests per tool.
2. Validate CLI argument parsing for repeatable values.
3. Validate CLI input echo (stdout parsing or `--print-inputs json`).

### Tier 2: Local Integration Harness (No External Network)

1. Add HTTP mock server for REST contracts (LLM + GitHub).
2. Use fixture JSON for provider responses.
3. Validate HTTP request shapes (method, headers, body).

### Tier 3: Live Integration Tests (Gated)

1. OpenAI live test with `OPENAI_API_KEY`.
2. Anthropic live test with `ANTHROPIC_API_KEY`.
3. GitHub live test with `GITHUB_TOKEN` for gist creation (optional).
4. Gate via env flag `RUN_LIVE_INTEGRATION=1`.

---

## LLM Integration Plan (OpenAI / Anthropic)

### Current State

1. `core/ir/src/transport/llm/*` has unit tests for request/response parsing.
2. `lib/llm-ops` testgen uses mock responses; no live calls.

### Required for “It Works” Confidence

1. **Request contract tests**: ensure method, URL, and headers are correct.
2. **Response contract tests**: validate parser handles provider JSON changes.
3. **Live API tests** (gated): verify real call returns parseable output.

### Proposed Test Layers

Layer A: Contract fixtures (offline)

1. Store provider response fixtures in `tests/fixtures/llm/`.
2. Validate `parse_openai_response` and `parse_anthropic_response`.
3. Validate request shape against expected JSON schema snapshot.

Layer B: Local HTTP mock (offline)

1. Spin up a local mock server.
2. Point provider base URL to the mock server.
3. Validate HTTP request fields and parse the mock response.

Layer C: Live (gated)

1. `RUN_LIVE_INTEGRATION=1` + `OPENAI_API_KEY` / `ANTHROPIC_API_KEY`.
2. Minimal prompt with low token budget.
3. Assert response fields: `content`, `model`, `finish_reason`.

### OpenAI and Anthropic Specifics

OpenAI:

1. Endpoint: `/v1/chat/completions` (or `/v1/responses` if used).
2. Required headers: `Authorization`, `Content-Type`, optional cache/reasoning headers.
3. Validate `finish_reason` and token usage.

Anthropic:

1. Endpoint: `/v1/messages`.
2. Required headers: `x-api-key`, `anthropic-version`.
3. Validate `finish_reason` and token usage.

---

## Proposed Ownership + Milestones

1. Add Tier 0 contract tests to `gunbc-dag`.
2. Add Tier 1 dry-run CLI contract tests for `gist` and `deps`.
3. Add LLM fixture parsing tests in `core/ir` or `lib/llm-ops`.
4. Add mock HTTP test harness for REST requests.
5. Add gated live LLM integration tests (OpenAI + Anthropic).

---

## Open Questions (Repo-Wide)

1. Should live LLM tests run in CI (opt-in) or nightly?
2. Where should fixtures live: `core/ir/tests/fixtures` or `lib/llm-ops/tests/fixtures`?
3. Should we add a `--print-inputs json` flag now to stabilize CLI tests?
4. Are we comfortable adding an HTTP mock dependency, or should we build a minimal local server?
