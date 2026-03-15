# gunbc Roadmap

Two parallel streams. Stream 1 is the product milestone. Stream 2 is housekeeping
that can happen independently.

---

## Stream 1: Gist on v2

**Goal:** `gist`, `gist_diff`, and `gist_recent` compile through the v2 compiler
and execute against real GitHub/Git APIs, in both Rust and Python targets.

### Current state (2026-03-15)

The v2 compiler pipeline (tokenize -> parse -> resolve -> typecheck -> emit) is
fully implemented. Self-compile through resolve is proven on all 10 v2 modules
with zero errors. Typecheck and emit have full handler coverage for all item
types including `func`, `service`, `resource`, and `extern func` in both Rust
and Python renderers.

gist.dag is purely compositional: 4 pure functions, 3 workflow functions
(`func`), service calls (Git, GitHub), and resource usage (Network). No new type
definitions, no extern funcs. It has 11 transitive dependencies (including
`std/types.dag`).

### Gap analysis

#### P0: Typecheck performance rewrite

The v2 typechecker OOMs when processing 10+ modules in debug mode. Root cause:
the .dag language has no hash maps, so every name lookup is `list |> filter |>
first` -- O(n) per lookup, O(n*m) per module, O(n*m*k) for cross-module
resolution. With 11 modules containing hundreds of definitions, this explodes.

**Required work:**
1. Profile the generated v2 crate's typecheck on gist's 11 dependencies
   (identify the hot path -- likely `lookup_type`, `lookup_func_sig`, or scope
   extension)
2. Rewrite the typechecker's lookup strategy. Options:
   - Add `Map<K,V>` as a DSL built-in type with O(1) lookup (language change)
   - Move lookup-heavy functions to Rust runtime shims (pragmatic)
   - Restructure the typechecker to batch-build lookup tables per module as
     flat sorted lists with binary search (pure DSL, no language change)
3. Verify: `self_compile_all_modules` completes full pipeline without OOM

**Acceptance gate:** v2 compiler runs its full pipeline (tokenize through emit)
on its own 10 modules in debug mode without exceeding 4GB heap.

#### P1: TCO pass for emitted code (S84)

The v2 emitter has no tail-call optimization. v1's `fn_codegen.rs` has a TCO
pass; v2's `05_emit_rust.dag` does not. When the v2 compiler compiles gist's
recursive functions, the generated Rust will stack-overflow on deep inputs
without either TCO or stacker wrapping.

**Required work:**
1. Add TCO analysis to the v2 emit pipeline: detect tail-position self-calls
2. Emit `loop` + reassignment for tail-recursive functions (same transform as v1)
3. For non-tail recursion: stacker wrapping is already in the generated crate

**Acceptance gate:** Generated Rust for recursive .dag functions uses iterative
loops for tail calls. No stack overflow on inputs up to 10K lines.

#### P2: Gist compilation test

Feed gist.dag + its 11 transitive dependencies through the v2 pipeline and
verify the output compiles.

**Required work:**
1. Assemble gist's dependency chain: `std/types.dag`, `std/resources.dag`,
   `std/errors.dag`, `extdeps/cloud/cloud.dag`, `extdeps/cloud/gcp/gcp.dag`,
   `extdeps/github/github.dag`, `gunbc/auth/credentials.dag`, `extdeps/git.dag`,
   `extdeps/github/auth.dag`, `extdeps/github/gists.dag`, `gunbc/tools/gist.dag`
2. Add test: v2 compile all 11 files -> Rust target -> `cargo check` passes
3. Add test: v2 compile all 11 files -> Python target -> `python -m py_compile`
   passes

**Acceptance gate:** Emitted Rust and Python both pass syntax/type checking by
their respective compilers.

#### P3: Runtime bridge

The emitted code needs to perform I/O (HTTP calls, git commands, file reads).
The v2 emitter generates Rust with `reqwest`/`tokio` for services and shell
calls for git. This needs a runtime entry point.

**Required work:**
1. Generate a `main.rs` that wires CLI args -> compiled pipeline entry points
   (reuse v1's `tool_discovery.rs` pattern or write a v2-native equivalent)
2. Generate `Cargo.toml` with runtime dependencies (`reqwest`, `tokio`,
   `serde_json`, `clap`)
3. Dry-run support: intercept I/O at service boundaries (same pattern as v1's
   `DryRun` mode)

**Acceptance gate:** `cargo run -- gist --dry-run` on the v2-compiled gist
produces the same dry-run output as v1's `make gist-dry`.

#### P4: End-to-end execution

**Required work:**
1. Real execution: `cargo run -- gist` creates a GitHub gist (with valid token)
2. Verify all three variants: `gist`, `gist_diff`, `gist_recent`
3. Python target: `python gist.py` produces equivalent output

#### P1.5: Unify emitters via language specifications

The current v2 emitters (`05_emit_rust.dag`, `05_emit_python.dag`) are 1000+
line monoliths with hardcoded language knowledge. This violates the same
principle the extdeps follow: **external systems are modeled as structural
facts, not as code.**

A programming language's syntax is an external specification, just like
GitHub's REST API. The codebase already has this insight — `dsl/std/languages.dag`
models languages as compositional facts (type mappings, naming conventions,
comment syntax). The v1 architecture also has a target-agnostic `code_ir` with
tiered lowering. But the v2 emitters bypass both, embedding language knowledge
directly in rendering logic.

**The fix: language-specification-driven rendering.**

The pattern already exists in the codebase:
- `dsl/extdeps/github/gists.dag` models "what is the GitHub Gists API" as facts
- `dsl/std/languages.dag` models "what is Rust" as facts
- A renderer should consume these facts the same way a transport consumes an
  API spec — mechanically, with no language-specific code paths

**Required work:**

1. **Extend `languages.dag` into a full rendering specification.** The current
   model covers naming and type mapping. It needs:
   - Statement syntax: how a let-binding, function def, match/if, for-loop
     looks in each language
   - Expression syntax: operator precedence, string interpolation, method calls
   - Module system: imports, visibility, module declarations
   - Idioms: error handling (Result/try vs exceptions), async patterns,
     ownership (Rust-specific)

   Each of these is a structural fact about the language, derivable from its
   specification. Model them the same way extdeps models API endpoints — real
   syntax from real language references.

2. **Replace per-language emitters with a single data-driven renderer.** One
   `05_emit.dag` module that:
   - Takes a `TypedGraph` + `Language` specification
   - Walks the typed AST
   - At each node, consults the language spec for the rendering
   - Produces target text

   The renderer is a pure function: `render(typed_graph, language_spec) -> files`.
   Adding Go means adding `data go_language: LanguageSpec = { ... }`, not
   writing a new 1000-line emitter module.

3. **Validate with existing targets.** The unified renderer must produce
   identical output to the current `05_emit_rust.dag` and `05_emit_python.dag`
   for gist's 11 modules. Diff the output to prove equivalence.

4. **Delete `05_emit_rust.dag` and `05_emit_python.dag`** once the unified
   renderer produces equivalent output.

**What stays language-specific:** Idioms that can't be expressed as syntax
templates (Rust ownership/borrowing, Python's `__init__` pattern, Go's
multi-return error handling). These are modeled as language-specific rendering
strategies in the spec, not as code in the renderer. The spec says "Rust
functions return `Result<T, E>`"; the renderer applies that mechanically.

**Acceptance gate:** One emitter module, language specs in .dag data
declarations, `cargo check` passes for Rust output, `py_compile` passes for
Python output, output is byte-identical to current per-language emitters on
gist's dependency chain.

### Target languages

| Target | Spec | Runtime deps | Status |
|--------|------|-------------|--------|
| **Rust** | `dsl/std/languages.dag` `rust_language` | reqwest, tokio, clap | Current emitter works, unify in P1.5 |
| **Python** | `dsl/std/languages.dag` `python_language` | aiohttp, argparse | Current emitter works, unify in P1.5 |
| **Go** | `dsl/std/languages.dag` `go_language` | net/http, flag | Add spec after unification — no new emitter code |

### Acceptance criteria (ship gate)

All of the following must pass in CI:

- [ ] `v2_compile_gist_rust` -- v2 compiles gist (11 files) -> Rust -> `cargo check`
- [ ] `v2_compile_gist_python` -- v2 compiles gist (11 files) -> Python -> `py_compile`
- [ ] `v2_compile_gist_go` -- v2 compiles gist (11 files) -> Go -> `go build`
- [ ] `v2_gist_dry_run_rust` -- compiled Rust gist produces correct dry-run output
- [ ] `v2_gist_dry_run_python` -- compiled Python gist produces correct dry-run output
- [ ] `v2_gist_dry_run_go` -- compiled Go gist produces correct dry-run output
- [ ] `v2_gist_real_rust` -- compiled Rust gist creates a real GitHub gist (manual gate)
- [ ] `v2_gist_real_python` -- compiled Python gist creates a real GitHub gist (manual gate)
- [ ] `v2_gist_real_go` -- compiled Go gist creates a real GitHub gist (manual gate)
- [ ] v2 self-compile full pipeline completes without OOM (P0 prerequisite)
- [ ] No stack overflow on any .dag file up to 4000 lines (P1 prerequisite)
- [ ] Single unified emitter -- `05_emit_rust.dag` and `05_emit_python.dag` deleted (P1.5)
- [ ] Go target added by data declaration only, no new emitter module (P1.5)

---

## Stream 2: Sustainability cleanup

**Goal:** Close out the sustainability ledger. Delete stale documentation that
was written during v2 development and is now superseded by working code.

### Docs to delete

These documents were planning/design artifacts for work that is now implemented.
The code is the source of truth.

| File | Reason |
|------|--------|
| `DESIGN-v2-compiler.md` | v2 design is implemented; architecture in `src/v2/DESIGN.md` |
| `WORKBOARD.md` | Superseded by this roadmap |
| `src/v2/WORKBOARD.md` | Superseded by this roadmap |
| `src/v2/DESIGN-typed-ast.md` | Typed AST is implemented |
| `src/v2/DESIGN-parse-split.md` | Parser/tokenizer split is implemented |
| `src/v2/POSTMORTEM.md` | Issues are fixed; findings migrated to SUSTAINABILITY.md |
| `src/v2/PERFORMANCE.md` | Audit completed; findings acted on |
| `src/v2/workstreams/WS-B-parser-tokenizer.md` | Implemented |
| `src/v2/workstreams/WS-C-typecheck-resolve.md` | Implemented |
| `src/v2/workstreams/WS-D-emitter.md` | Implemented |
| `src/v2/workstreams/WS-E-pipeline-core.md` | Implemented |
| `src/v2/workstreams/WS-F-rust-codegen.md` | Implemented |
| `src/v2/workstreams/WS-G-runtime-shims.md` | Implemented |

### Docs to keep

| File | Reason |
|------|--------|
| `CLAUDE.md` | Live project instructions |
| `README.md` | Repo overview |
| `MODELING.md` | Domain modeling guidelines (evergreen) |
| `ROADMAP.md` | This file |
| `src/v1/ARCHITECTURE.md` | v1 architecture reference (needed while v1 exists) |
| `src/v1/README.md` | v1 invariants |
| `src/v1/SUSTAINABILITY.md` | Violation ledger (update, don't delete) |
| `src/v2/DESIGN.md` | v2 architecture reference (evergreen) |
| `dsl/extdeps/extdeps.md` | Extdeps modeling guidelines |

### SUSTAINABILITY.md cleanup

Open findings to resolve:

| Finding | Status | Action |
|---------|--------|--------|
| **S83** (evaluator stack overflow) | **Fixed** this session | Mark fixed -- stacker wrapping on eval_expr, eval_expr_s, eval_non_sibling_call_raw |
| **S84** (v2 emitter no TCO) | Open | Stream 1 P1 -- implement TCO pass |
| **S82** (namespace collision) | Fixed | Already marked -- rename to `lookup_func_sig_in_scope` |
| **S76-S81** (type-unaware codegen) | Terminal | Die with self-hosting -- mark as terminal, no action |
| **S52** (parser mutual recursion) | Bounded | Stacker handles this now -- mark as mitigated |

### Acceptance criteria

- [ ] All files in "delete" table removed from repo
- [ ] `src/v2/workstreams/` directory deleted
- [ ] SUSTAINABILITY.md updated: S83 marked fixed, S52 updated, S76-S81 marked terminal
- [ ] `cargo test --workspace --exclude gunbc-dag-tests` still passes
- [ ] `cargo clippy --all-targets -- -D warnings` clean
