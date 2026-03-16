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

## Stream 3: Fractal Node — dissolving structure into composition

**Goal:** Reduce the Node type's special-purpose fields into
`children` + `connective`. The foundational philosophy (logic as
single primitive, four-layer model, why types are logic) is in
`MODELING.md`. This stream is the operational plan.

### Design philosophy

See `MODELING.md` — "Foundational primitive: truth-valued structure."

Summary: the compiler's four layers are surface sugar → composition
layer (Node) → semantic kernel (types, effects, contracts) → foundation
(classical logic). The composition layer says how things connect. The
semantic kernel says what flows through. The foundation says why it's
sound. Node is the composition layer — it should know about connectives
and edges, not about domain-specific metadata.

### Current state (2026-03-16)

W10-W13 completed the first fractal step: `operations` and
`capabilities` dissolved into child Nodes carrying `PortContract`
metadata. Four fields became `children` + `port_contract`.

The emit dispatch is still a 6-deep if/else chain inferring node kind
from field combinations — this is the heuristic that dissolves as more
structure moves into children + connective.

### Incremental path

**Phase A — Response mappings → type structure + dispatch data**

Operation return types become conditional coproducts that the type
system already understands:
```
Ok { value: UserProfile } | NotFound { msg: String } | ServerError { detail: String }
```

The type part (coproduct of outcomes) goes into `return_type`. The
dispatch part (status 200 → Ok variant) is runtime data that moves
to the rendering layer. This eliminates `response` and `exit_mappings`
from PortContract.

Note: response mappings have `status: Expr` — a runtime value, not
a type. The conditional `status=200 IMPLIES UserProfile` is dependent
typing. The decomposition separates the type structure (coproduct of
outcomes — logic) from the runtime dispatch (which outcome based on
status code — computation).

**Phase B — Modifiers → node-level assertions**

`Idempotent`, `Readonly`, `Hermetic` are behavioral propositions —
assertions about the computation, not data shape. They need a home
that isn't PortContract but also isn't `Refined` (which constrains
data, not behavior). Likely extends `Predicate` or adds an
`assertions` field. Design work needed.

**Phase C — Mock responses → test companion**

Mocks are example inhabitants of the return type — test witnesses,
not type metadata. They separate from PortContract into a test layer
(possibly as child Nodes whose body is a literal witness — fractal).

**Phase D — PortContract dissolves**

After A/B/C, PortContract has only `outputs: List<Field>`. Outputs
merge into `return_type` (the Ok variant's fields). PortContract is
deleted. An "operation" is now just a child Node with params and a
conditional return_type — the same structure as any other node.

Detection: parent has `transport` → children are operations. No
PortContract field needed.

**Phase E — shape dissolves via connective (W14-W17)**

`shape: TypeBody?` dissolves into `children` + `connective: Connective?`.
Record fields become child Nodes (connective: And). Sum variants become
child Nodes (connective: Or). Same proven pattern as W10-W13.
Detailed sketch follows.

**Phase F — Primitive dissolves**

Type definitions in .dag files define String, Int, etc. as compositions.
`Primitive` TypeExpr variant becomes a backend rendering hint. Requires
P1.5 language specs (backend recognition table). See `MODELING.md` for
why Int and String are too wide as primitives.

### Concrete sketch: the connective field

The connective is the single bit that gives a node's children logical
meaning. Without it, children are just a list. With it, they're a
proposition.

```dag
type Connective = And | Or

type Node {
  name: String
  span: SourceSpan
  children: List<Node>
  connective: Connective?   // ← the new field
  params: List<Param>
  return_type: TypeExpr?
  uses: List<ResourceUse>
  body: Expr?
  // shape: TypeBody?        ← DISSOLVES (replaced by children + connective)
  transport: TransportBinding?
  properties: List<FieldInit>
  type_annotation: TypeExpr?
  config: ServiceConfig?
  port_contract: PortContract?
}
```

Two values. That's the primitive manifested as a field.

- `And` — all children hold simultaneously (record fields, service
  operations, resource capabilities)
- `Or` — exactly one child holds (sum variants, optional)
- `none` — this node has no structural children (leaf, function,
  extern)

#### Example: record type

```
// Source: type Person { name: String, age: Int }

// Today:
Node {
  name: "Person"
  shape: Some { value: Record { fields: [
    Field { name: "name", type_expr: Named("String"), optional: false },
    Field { name: "age", type_expr: Named("Int"), optional: false }
  ] } }
  children: []
}

// With connective:
Node {
  name: "Person"
  connective: Some { value: And }
  children: [
    Node { name: "name", return_type: Named("String") },
    Node { name: "age", return_type: Named("Int") }
  ]
}
```

The `shape` field is gone. The record's structure IS its children.
The `And` connective says: all children must hold. This is what
"record" means — conjunction.

#### Example: sum type

```
// Source: type Result = Ok { value: Int } | Err { message: String }

// Today:
Node {
  name: "Result"
  shape: Some { value: Sum { variants: [
    Variant { name: "Ok", fields: [Field { name: "value", ... }] },
    Variant { name: "Err", fields: [Field { name: "message", ... }] }
  ] } }
  children: []
}

// With connective:
Node {
  name: "Result"
  connective: Some { value: Or }
  children: [
    Node {
      name: "Ok"
      connective: Some { value: And }
      children: [
        Node { name: "value", return_type: Named("Int") }
      ]
    },
    Node {
      name: "Err"
      connective: Some { value: And }
      children: [
        Node { name: "message", return_type: Named("String") }
      ]
    }
  ]
}
```

A sum type is `Or` at the top — exactly one variant holds. Each
variant is `And` — all its fields must hold. Same structure as
today's `Sum { variants: [Variant { fields }] }`, but expressed
as Node composition instead of a separate TypeBody representation.

#### Example: type alias

```
// Source: type Name = String

// Today:
Node {
  name: "Name"
  shape: Some { value: Alias { base: Named("String") } }
}

// With connective:
Node {
  name: "Name"
  connective: none          // no structural children
  return_type: Named("String")  // the aliased type
}
```

An alias has no children and no connective — it's a name that
points to another proposition. The `return_type` carries the
reference. This is the same as how functions use `return_type`
today.

#### Example: optional field

```
// Source: type Config { name: String, debug: Bool? }

// With connective:
Node {
  name: "Config"
  connective: Some { value: And }
  children: [
    Node { name: "name", return_type: Named("String") },
    Node { name: "debug", return_type: Optional { inner: Named("Bool") } }
  ]
}
```

The `optional: Bool` flag on `Field` dissolves into the type system.
An optional field is a child whose `return_type` is `Optional { inner: T }`
— which is `OR(T, Unit)` in logic. The optionality is in the type,
not in a separate boolean.

#### Example: default values

```
// Source: type Config { retries: Int = 3 }

// With connective:
Node {
  name: "Config"
  connective: Some { value: And }
  children: [
    Node {
      name: "retries"
      return_type: Named("Int")
      body: Some { value: Literal { value: LitInt { value: 3 } } }
    }
  ]
}
```

A default value is a child whose `body` is a proof witness. "This
field is Int, and here is a default witness: 3." The `default_value`
field on `Field` dissolves — it's just `body` on the child Node.

#### Example: service (already fractal from W10-W13)

```
Node {
  name: "git.Core"
  connective: Some { value: And }    // all operations available
  transport: Some { value: ShellBinding { ... } }
  children: [
    Node {
      name: "CurrentBranch"
      params: []
      return_type: Named("String")
      port_contract: Some { value: OperationContract { ... } }
    }
  ]
}
```

Services already have children from W10-W13. Adding `connective: And`
just makes explicit what was implicit: all operations are available
simultaneously.

#### What dissolves

| Old structure | Replaced by | Logic |
|---|---|---|
| `shape: TypeBody?` | `children` + `connective` | structure IS children |
| `TypeBody = Record \| Sum \| Alias` | `And` / `Or` / `none` | connective replaces tag |
| `Field` (in type structure) | child Node | a field is a sub-proposition |
| `Variant` | child Node with its own children | a variant is a sub-node |
| `Field.optional: Bool` | `return_type: Optional { ... }` | optionality is in the type |
| `Field.default_value` | `body` on the child Node | default is a proof witness |
| `Field.from_key` | `properties` on the child Node | serialization is metadata |

#### What stays (for now)

| Structure | Why it stays |
|---|---|
| `TypeExpr` | Type references remain a separate representation until TypeExpr→Node convergence (later phase) |
| `Param` | Function params are ports (preconditions), not structural children. Param might merge into Node later but it's a different relationship than connective children. |
| `Expr` | Computation stays separate until Expr→Node convergence (later phase) |
| `PortContract` | Dissolves separately per Phase A/B/C/D above |

#### Migration shape (same pattern as W10-W13)

This follows the proven 4-step pattern:

**W14: Additive.** Add `connective: Connective?` to Node and TypedNode.
All existing constructions get `connective: none`. No behavior change.

**W15: Parser dual-writes.** `parse_type_def` builds children from
fields/variants AND keeps `shape`. Record fields become child Nodes
with `return_type`. Sum variants become child Nodes with their own
children. Set `connective: And` or `Or`.

**W16: Migrate consumers.** Typecheck, resolve, and emit read from
`children` + `connective` instead of `shape`. The emit dispatch for
types becomes: `connective == And` → emit struct, `connective == Or`
→ emit enum, alias → emit type alias.

**W17: Delete shape.** Remove `shape: TypeBody?` from Node/TypedNode.
Delete `TypeBody`, `Field`-as-type-structure, `Variant`. The types
that remain are `Connective`, `Param`, and whatever `Field` usages
survive outside type definitions (like PortContract outputs, which
dissolve in their own phase).

#### The emit dispatch after W17

Today's 6-arm if/else chain simplifies because type dispatch
no longer uses `shape`:

```
// Today:
if item.shape != none { ... }                        // type
else if item.body != none && item.type_annotation == none { ... } // fn/func
else if item.body != none && item.type_annotation != none { ... } // data
else if item.transport != none && item.children > 0 { ... }      // service
else if item.transport == none && item.children > 0 { ... }      // resource
else if item.params > 0 && item.body == none { ... }             // extern

// After W17:
if item.connective != none && item.transport == none { ... }     // type def
else if item.body != none && item.type_annotation == none { ... } // fn/func
else if item.body != none && item.type_annotation != none { ... } // data
else if item.transport != none { ... }                            // service
else if item.connective == none && item.children > 0 { ... }     // resource
else if item.params > 0 && item.body == none { ... }             // extern
```

Not fewer arms yet — but the type arm is now structural
(`connective != none`) rather than representational (`shape != none`).
The real simplification comes when PortContract dissolves and
service/resource detection becomes purely connective + transport.

#### After all phases converge

When both `shape` and `port_contract` have dissolved, the Node
fields are:

```dag
type Node {
  name: String
  span: SourceSpan
  children: List<Node>
  connective: Connective?    // AND/OR — the logical primitive
  params: List<Param>        // preconditions (ports in)
  return_type: TypeExpr?     // postcondition (port out)
  uses: List<ResourceUse>    // resource dependencies
  body: Expr?                // proof / computation
  transport: TransportBinding?  // grounding in external reality
  properties: List<FieldInit>   // metadata
  type_annotation: TypeExpr?    // explicit type assertion
  config: ServiceConfig?        // service configuration
}
```

Compared to today: `shape`, `port_contract`, `operations`,
`capabilities` are all gone. Four fields dissolved into
`children` + `connective`. The logical structure is the
composition.

The remaining emit dispatch becomes:

| connective | transport | body | What it is |
|---|---|---|---|
| `And` or `Or` | none | none | type definition |
| none | none | some | fn/func/data |
| `And` | some | none | service (children are operations) |
| none or `And` | none | none, children > 0 | resource |
| none | none | none, params > 0 | extern |

Five patterns. All structural. No field-combination guessing —
the connective and transport together determine the logical role.

### Non-goals

- Deleting keywords from the surface syntax. Keywords are good parse
  sugar and good for readability. `service` and `fn` are ergonomic
  ways to say "build me a proposition with these structural properties."

- Expanding String to bits at compile time. The logical decomposition
  is the *model* — the backend renders efficiently. Just like math
  defines reals as Dedekind cuts but nobody computes with cuts.

- Purity for its own sake. If a pragmatic escape hatch (like the
  current PortContract) is needed to ship, keep it. The incremental
  path replaces scaffolding as the logical foundation matures, not
  before.

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
