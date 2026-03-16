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

#### P0: Recursive type support (S85) — current gist pipeline blocker

The gist pipeline OOMs on `std.types` (item 55 of 101: `CredentialFlow`).
Root cause: recursive sum type triggers infinite recursion in the resolver
due to dropped cycle-detection state (S85). This is not an algorithmic
scaling issue — it is a missing language feature.

**Required work:**
1. Thread `resolving` through `resolve_field`, `resolve_variant`,
   `resolve_param` and their callers (signature changes) — smallest safe fix
2. Add a recursive-type test case through the full pipeline:
   `type Node = Leaf | Branch { children: List<Node> }`
3. Terminal: SCC analysis on the type dependency graph during resolve,
   producing cycle metadata carried structurally on `TypeBinding`

**Acceptance gate:** v2 compiler resolves `std.types` (101 type definitions
including recursive types) without OOM. Full pipeline completes for gist's
11 transitive dependencies.

#### Separate concern: lookup complexity (partially addressed)

The v2 typechecker previously used list-based environments for all name
lookups — O(n) per lookup, O(n*m*k) for cross-module resolution. This was
a real scaling bottleneck but is now **partially addressed**: type cache,
item registry, and module index all use `Map<K,V>` with O(1) lookups
(documented in SUSTAINABILITY.md gap analysis). Further optimization may
be needed as the module count grows, but this is not the current blocker.

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

#### P1.5: Language specifications and emitter layering

The current v2 emitters (`05_emit_rust.dag`, `05_emit_python.dag`) are 1000+
line monoliths with hardcoded language knowledge. Much of this knowledge —
naming conventions, keywords, literal spellings, comment syntax, import
syntax, type mappings — is surface spelling that belongs in language
specifications, not in code.

The codebase already has this insight: `dsl/std/languages.dag` models
languages as compositional facts. The fix is to **separate spelling from
semantics** and **shrink the per-backend emitters**, not to replace them
with a single template-driven renderer.

A fully template-driven single emitter would recreate heuristics inside
the templates, because ownership, borrowing, async strategy, TCO lowering,
operator precedence, destructuring, and error propagation are not just
syntax — they are irreducible semantic differences between target languages.

**The layered approach:**

1. **Language specs in `languages.dag`** for spelling and idiom metadata:
   naming conventions, keywords, type name mappings, comment syntax, import
   syntax, literal format strings. This is data derivable from language
   reference docs — model it the same way extdeps models API endpoints.

2. **Structural typed/code IR for semantics.** The typed graph from
   typecheck carries the semantic facts. Per-backend lowerers consume
   these facts, not raw AST.

3. **Thin per-backend lowerers** for irreducible semantic differences:
   Rust ownership/borrowing and `Result<T,E>` error propagation, Python's
   `__init__` pattern and exception handling, Go's multi-return error
   handling. These stay as code in per-backend modules — but they should
   be small (consulting the language spec for spelling) rather than
   1000-line monoliths that mix spelling with semantics.

**Required work:**

1. Extend `languages.dag` to cover statement syntax templates, expression
   syntax patterns, and module system conventions — all derivable from
   real language reference docs.

2. Refactor per-backend emitters to consult language specs for spelling
   decisions (type names, naming conventions, comment format, import
   syntax) instead of hardcoding them.

3. Extract shared emission logic (structural dispatch, scope management,
   tree walking) into `05_emit.dag` — this is already partially done.

4. Validate: emitted Rust still passes `cargo check`, emitted Python
   still passes `py_compile`.

**What this does NOT do:** Delete per-backend emitters or aim for a single
renderer. Adding Go means writing a thin Go lowerer that handles Go-specific
semantics (multi-return errors, goroutine patterns, interface satisfaction)
and consults `languages.dag` for Go spelling. The lowerer should be small
because the language spec carries most of the surface knowledge.

**Acceptance gate:** Per-backend emitters consult `languages.dag` for
spelling; shared structural logic lives in `05_emit.dag`; no hardcoded
type name mappings or naming conventions in per-backend code.

### Target languages

| Target | Spec | Runtime deps | Status |
|--------|------|-------------|--------|
| **Rust** | `dsl/std/languages.dag` `rust_language` | reqwest, tokio, clap | Current emitter works, refactor in P1.5 |
| **Python** | `dsl/std/languages.dag` `python_language` | aiohttp, argparse | Current emitter works, refactor in P1.5 |
| **Go** | `dsl/std/languages.dag` `go_language` | net/http, flag | Add thin lowerer + language spec in P1.5 |

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
- [ ] Per-backend emitters consult `languages.dag` for spelling — no hardcoded type/naming maps (P1.5)
- [ ] Go target via thin lowerer + language spec, not a 1000-line monolith (P1.5)

---

## Stream 3: Fractal Node — squeezing heuristics to the edges

**Goal:** The compiler core operates on structural graph properties (edges,
ports, children, contracts) — never on keyword identity or field-combination
fingerprints. Heuristics live at two edges: the **frontend** (parse sugar that
captures user intent) and the **backend** (language-specific rendering
decisions). The core is the structural middle.

### Current state (2026-03-16)

W10-W13 completed the first fractal step: operations and capabilities dissolved
into child Nodes carrying `PortContract` metadata. The Node type lost 2 fields
(`operations`, `capabilities`); composition now flows through `children`.

```
Frontend (parse sugar)     Core (structural)         Backend (rendering)
─────────────────────      ──────────────────        ───────────────────
keyword "service"    →     Node with transport       →  Rust struct+impl
keyword "operation"  →       child Node with         →  async method
                            OperationContract
keyword "resource"   →     Node without transport    →  Rust trait
keyword "capability" →       child Node with         →  abstract method
                            CapabilityContract
```

### Where heuristics still live in the core

The emit dispatch (`emit_typed_item`) is a 6-deep if/else chain that infers
node kind from field combinations:

| Condition | Inferred kind |
|-----------|---------------|
| `shape != none` | type definition |
| `body != none && type_annotation == none` | fn/func |
| `body != none && type_annotation != none` | data constant |
| `transport != none && children > 0` | service |
| `transport == none && children > 0` | resource |
| `params > 0 && body == none` | extern func |

This works today but is fragile: every new construct needs a unique
field fingerprint, and overlapping patterns cause misclassification.
The check is reconstructing the keyword from the fields — exactly the
heuristic that should live at the frontend edge, not in the core.

Similarly, the typecheck builds the service registry by testing
`item.transport != none && item.children |> count > 0`. This is a
structural check (good) but it's inline in the typecheck rather than
being a property the parser asserted and the checker validated.

### Design direction: assert at the front, validate in the middle, render at the back

The compiler's job is to faithfully transform a graph of structural facts
into target-language text. The question is where "what kind of thing is this"
gets decided:

**Frontend (parser):** The user writes `service`, `fn`, `type` — these are
intent signals. The parser should capture that intent as a lightweight
structural marker (not a keyword string, but a property that constrains
what fields are valid). This is where user intent enters the system.

**Core (checker/resolver):** Validates that the structural properties are
consistent — a node with transport must have children with port_contracts,
a node with shape must not have a body, etc. The checker never asks "what
keyword?" — it checks edge constraints on the graph. If a node violates
constraints, the error message can reference the parse origin for diagnostics,
but the constraint itself is structural.

**Backend (emitters):** Renders structural facts into target syntax. The
emitter dispatch should follow from structural properties that the checker
has already validated, not from field-combination guessing. If the checker
guarantees that "nodes with transport always have children with
OperationContract", the emitter can rely on that without re-deriving it.

### Open questions

1. **What form does the frontend marker take?** Options range from an
   explicit `NodeKind` enum (pragmatic, easy to dispatch on, but reintroduces
   a closed set) to structural "shape constraints" that the parser attaches
   and the checker validates (more flexible, harder to get right). The marker
   should encode the *reason* for the constraint (what would break if violated),
   not the keyword label (which is surface syntax that can drift).

2. **How do edge constraints get specified?** Today they're implicit in
   the if/else chains. The S86 direction (checker validates edge constraints)
   points toward making these explicit — possibly as data in .dag files
   rather than code in the checker. This connects to the broader vision
   of domain-in-DSL-not-Rust.

3. **When does this pay off?** The current 6-arm dispatch works. The
   investment in formalizing constraints pays off when: (a) new node kinds
   are added (interface, pipeline, profile), (b) the emit dispatch needs to
   be target-agnostic, or (c) the checker needs to give precise diagnostics
   about *why* a node is malformed.

### Non-goals

- Deleting keywords from the surface syntax. Keywords are good parse sugar
  and good for readability. The point is that keywords inform the parser
  what to expect, not that they flow through the compiler as identity.

- A single universal "is this a service?" function. Different phases need
  different things — the resolver needs to know if something has children
  to resolve, the emitter needs to know how to render. These are different
  questions with different structural answers.

### Incremental path

This is not a rewrite. Each step tightens the boundary between frontend
intent and core structure:

1. **Done (W10-W13):** Operations/capabilities are children with PortContract.
   Emit and typecheck read from children, not from keyword-specific lists.

2. **Next:** Identify which emit dispatch conditions are structural invariants
   vs. accidental field patterns. For each one, either (a) the parser should
   assert it via a structural property, or (b) the checker should derive it
   from validated constraints.

3. **Later:** Edge constraints as data. The checker consults constraint
   definitions (possibly in .dag) rather than hardcoding if/else chains.
   Adding a new node kind means adding constraint data, not editing
   checker code.

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
