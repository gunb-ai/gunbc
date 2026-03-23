# gunbc Design Document

_Last updated: 2026-03-22. This document is a comprehensive reference
for the gunbc project — its philosophy, architecture, language, compiler,
domain modeling, testing, and roadmap. It is written so that someone (or
an LLM) with no prior context can reason about design decisions without
reading the entire codebase._

---

## Table of Contents

1. [What gunbc Is](#1-what-gunbc-is)
2. [The Foundational Claim](#2-the-foundational-claim)
3. [Why DAGs Give You Compile-Time Superpowers](#3-why-dags-give-you-compile-time-superpowers)
4. [The Core Primitive: Node](#4-the-core-primitive-node)
5. [The Four-Layer Model](#5-the-four-layer-model)
6. [The Language (.dag)](#6-the-language-dag)
7. [Domain Modeling: Facts from Specs](#7-domain-modeling-facts-from-specs)
8. [The Compiler Pipeline](#8-the-compiler-pipeline)
9. [Emission and Target Languages](#9-emission-and-target-languages)
10. [Transport and I/O](#10-transport-and-io)
11. [Testing Infrastructure](#11-testing-infrastructure)
12. [The Invariants](#12-the-invariants)
13. [Self-Hosting and Bootstrap](#13-self-hosting-and-bootstrap)
14. [The Convergence Migration](#14-the-convergence-migration)
15. [Current State and Roadmap](#15-current-state-and-roadmap)
16. [Codebase Map](#16-codebase-map)

---

## 1. What gunbc Is

gunbc is a compiler for a DAG-based language that kills glue code.

The problem: glue bugs — the code between two systems that handles format
conversion, error mapping, auth, retries — are the most common source of
production bugs. Current software is bad at dependency modeling (making
systems work together correctly). LLM agents are currently better at this
because they've internalized the API docs, but LLM-driven workflows are
expensive, non-deterministic, and require the LLM in the hot path at
every step.

The thesis: if you fix dependency modeling (make it structural, causal,
fact-based from specs), you don't need an LLM to drive the workflow. The
program becomes deterministic — the graph IS the workflow, execution is
map-reduce over the graph. The LLM's role shifts from "runtime
orchestrator" to "compile-time assistant" (helping write `.dag` files).

```
Current agent model:
  user intent → LLM → (reason) → call service → (reason) → call service → output
  LLM in hot path at every step. Non-deterministic.

gunbc model:
  user intent → .dag → compiler → minimal valid graph → map-reduce → output
  Graph at compile time. Execution is deterministic traversal.
```

**Origin:** gunbc is a reaction to Google-scale dependency management
where domain modeling was popular but codebases couldn't scale to satisfy
all dependencies simultaneously. gunbc inverts: dependencies inform code
generation rather than constraining developers.

---

## 2. The Foundational Claim

**All technical systems obey cause and effect.** If they don't, it's
"art" — outside scope. A DAG is the natural representation of cause and
effect: causes precede effects, no cycles in causality. Therefore, if you
can reason about a system in terms of cause and effect, you can implement
it as a fractal DAG with layered composition.

This is the single architectural primitive. Everything else derives:

| Question | Answer |
|----------|--------|
| Why DAGs? | Causality is acyclic |
| Why facts, not compiler passes? | Facts are causes. The graph is the effect |
| Why single pass? | Cause → effect is one direction |
| Why structural composition? | Complex effects = compositions of simpler cause-effect chains |
| Why no heuristics? | Cause and effect is deterministic. Guessing = lost causal chain |
| Why no "fallbacks that fabricate"? | Fabricating breaks the causal chain |
| Why extdeps from specs? | Specs document the causal behavior of external systems |
| Why fractal? | Cause and effect is scale-invariant |

### Smart Facts + Dumb Compiler (the MLIR inversion)

Most compiler frameworks (MLIR, LLVM) use **dumb IR + smart passes** —
intelligence lives in optimization/transformation passes. gunbc inverts
this: **smart facts + dumb compiler**. Intelligence lives in `.dag`
domain models.

The compiler does one thing: compose facts and produce the unique minimal
graph consistent with all of them. No search, no optimization passes, no
heuristics. Hash lookups. The graph is derived, not transformed.

```
.dag intent   (what you want to happen)
+ extdep facts (what external systems actually do)
+ std facts    (what primitives mean)
= the one minimal valid graph
→ emit the code that graph describes
```

If the graph is "suboptimal," a fact is missing. Fix: improve the domain
model. Never propose adding a compiler pass.

### Why Single-Pass Works

The codebase invariants eliminate every category of work that would
require a pass:

- Explicit boundary contracts → nothing for a fixup pass to fix
- No fallbacks that fabricate → nothing for a cleanup pass to clean
- No duplicate representations → nothing for a consistency pass to
  reconcile
- Single-authority metadata → nothing for a merge pass to merge

Each layer takes a contract, produces a contract. Layers compose. Passes
are peers that fight over shared mutable state ("optimization soup").

---

## 3. Why DAGs Give You Compile-Time Superpowers

This is the key insight that separates gunbc from general-purpose
programming languages. A DAG — a directed acyclic graph — is
structurally constrained in ways that a general program is not, and those
constraints enable guarantees that are impossible (or undecidable) in
general programs.

### What a general program cannot guarantee

In a Turing-complete language with unbounded loops and arbitrary control
flow:

- **Termination** is undecidable (halting problem)
- **Resource usage** is unbounded and unpredictable
- **Data flow** is obscured by mutation, aliasing, and dynamic dispatch
- **Parallelism** requires explicit analysis or annotations
- **I/O boundaries** are invisible (any function might do I/O)
- **Ordering** is implicit in control flow, not inspectable

### What a DAG can guarantee at compile time

Because a DAG has no cycles and all edges are explicit:

| Property | How DAGs enable it |
|----------|-------------------|
| **Termination** | No cycles → execution always finishes |
| **Ordering** | Topological sort gives a total execution order — automatically |
| **Parallelism** | No edge between two nodes = independent = parallelizable. This is structural, not inferred |
| **I/O detection** | If a node has a `transport` field, it does I/O. If it doesn't, it's pure. Visible from the graph |
| **Data flow** | Every value flows through explicit edges with typed ports. No hidden mutation, no aliasing |
| **Resource tracking** | Resources are declared on nodes; conflicts are detectable by graph analysis |
| **Cardinality** | Each port declares [min, max] cardinality — the system knows at compile time whether something is a scalar, list, or optional |
| **Cost analysis** | Each node has a complexity contract; total cost is a structural fold over the graph |
| **Boundary detection** | Unconnected output ports = world writes. Unconnected input ports = world reads. Detectable by scanning edges |

These properties fall out of the graph structure for free. In a
general-purpose language, each one would require a separate analysis
pass, annotation system, or proof obligation — and most are simply
undecidable.

### The execution model

Execution is map-reduce over the graph:

1. Topologically sort the nodes
2. For each node: gather input values from upstream output ports
3. Execute the node (pure computation or transport I/O)
4. Store output values for downstream nodes
5. Repeat until all nodes are processed

Nodes with no edge between them can execute in parallel — this is
detectable from the graph, not annotated by the programmer. The scheduler
sees the structure.

---

## 4. The Core Primitive: Node

The entire system converges on a single recursive data structure: the
**Node**. Types, expressions, services, transports, parameters, and
operations are all Nodes. The compiler is a generic graph processor over
Nodes.

### Current Node definition (from `src/v2/00_core.dag`)

```dag
type Connective = Conj | Disj

type Node {
  name: String
  span: SourceSpan
  children: List<Node>        // Fractal composition
  connective: Connective?     // And (record/all-hold) or Or (sum/one-holds)
  params: List<Param>         // What flows in (preconditions)
  return_type: Node?          // What flows out (postcondition)
  uses: List<ResourceUse>     // Resource dependencies
  body: Node?                 // Computation / proof
  transport: Node?            // I/O grounding
  properties: List<FieldInit> // Extensible metadata
  type_annotation: Node?
  config: ServiceConfig?
  expr_data: ExprData         // Expression embedding
}
```

### Why each field is irreducible

| Field | Logical role | Why separate |
|-------|-------------|--------------|
| `children` + `connective` | Composition (AND/OR of sub-propositions) | The core primitive |
| `params` | Obligations — what must be supplied (IMPLIES antecedent) | Consumed, not composed |
| `return_type` | Guarantee — what is produced (IMPLIES consequent) | Flows out, not in |
| `body` | Proof — computation connecting params to return_type | HOW, not WHAT |
| `transport` | I/O grounding — where this node touches external reality | Must be structural so it can't be smuggled (Invariant 2) |
| `properties` | Extensible metadata | Escape hatch for domain facts |
| `expr_data` | Expression discriminator | Embedded computation (being converged into Node) |

### The convergence target

The project is actively migrating toward a fully converged model where
types, expressions, and transports are all just Nodes:

- **Types** are Nodes with `connective` (Conj for records, Disj for
  enums) and `children` (the fields/variants)
- **Expressions** are Nodes with `expr_data` discriminating the
  expression kind (literal, call, match, etc.)
- **Parameters** are Nodes (`Param` dissolves into Node — a param IS
  a Node)
- **Transports** are Nodes (compositional children describe behavior —
  `base_url`, `argv`, `auth` — not a hardcoded enum)

The irreducible kernel — what can't be a Node:

1. **Node** itself (circular if self-defined)
2. **Connective = Conj | Disj** (the logical primitive)
3. **Kernel primitives** (String, Int, Bool, List, Map) — engineering
   atoms the compiler treats as units

Everything else is composition.

### Naming is aliasing

Named types are namespaces for compositions, not opaque tokens. When you
write `type GitCommit { sha: CommitSha, message: String }`, the compiler
sees a Node with `connective: Conj` and two children. The name
`GitCommit` is an alias — the compiler sees through it to the structure.

Similarly, `List<String>` is `List` (a parameterized Node from std) with
`T=String`. The result is a Node. No special `ContainerKind` enum.
`Optional<T>` is `Disj(Some { value: T }, None)`. `T?` is sugar for
`Optional<T>`.

---

## 5. The Four-Layer Model

The compiler operates at four levels. Each layer is built on the one
below. No layer skips.

```
Surface sugar:      service, fn, type, operation    (user intent)
Composition layer:  Node, children, edges           (how things connect)
Semantic kernel:    types, effects, contracts        (what flows through nodes)
Foundation:         logical algebra                  (why it's sound)
```

### Foundation: classical bivalent logic

The smallest unambiguous primitive in digital computing is
**truth/falseness** — a proposition that holds or doesn't. This is the
only primitive. Everything above it — String, Int, Float, List, Map,
services, resources — is explicit composition built on logical structure.

- **AND (Conj)**: conjunction — all children hold simultaneously
  (record fields)
- **OR (Disj)**: disjunction — exactly one child holds (enum variants)
- **IMPLIES**: entailment — params (antecedent) produce return_type
  (consequent)

The composition layer (Node, children, connectives) is
model-independent. AND/OR/IMPLIES have analogs in every algebra
(probability theory, linear algebra, real analysis). The DAG structure
doesn't care which algebra is underneath — classical logic is a
parameter, not a hardwired assumption.

### Why Int and String are "too wide"

| "Primitive" | What it hides |
|-------------|--------------|
| `Int` | Bit width? Signed? Two's complement? Arbitrary precision? |
| `String` | Encoding? Null-terminated? Length-prefixed? Byte or code-point? |
| `Float` | IEEE 754 binary32? binary64? Decimal? Platform-dependent? |
| `Bool` | Unambiguous — this IS the primitive |

Every compiler answers these questions differently. The fix: make the
decisions explicit. Define Int as a composition with a precise
specification. The representation is the backend's job.

---

## 6. The Language (.dag)

`.dag` is a declaration language for defining nodes in directed graphs.
The surface syntax provides convenient sugar for common patterns, but
everything desugars to Nodes.

### Surface constructs

```dag
// Module declaration and imports
module extdeps.git
import std.types { CommitSha, FilePath, Timestamp }

// Type = Node with connective
type GitCommit {              // Conj: all fields hold
  sha: CommitSha
  message: String
  author: GitAuthor
  parent_shas: List<CommitSha>
}

type GitRef                   // Disj: one variant holds
  = BranchRef { name: String }
  | TagRef { name: String }
  | CommitRef { sha: CommitSha }

// Function = Node with params, return_type, and body
fn is_kernel_type(name: String) -> Bool {
  kernel_types |> any(t => t == name)
}

// Service = Node with children (operations), transport grounding
service git.Core {
  operation CurrentBranch {
    output { branch: String }
    readonly
    transport shell { argv: ["git", "rev-parse", "--abbrev-ref", "HEAD"] }
  }
}

// Data = compile-time constant
data kernel_types: List<String> = [
  "String", "Int", "Bool", "Float", "Secret", "Json", "Unit", "Bytes"
]

// Pattern = reusable DAG shape
pattern file_content_matches {
  input { file: FilePath, expected: String }
  output { matches: Bool }
  // ... node wiring
}
```

### Expression forms (ExprData)

Nodes can carry computation via `expr_data`. Current variants include:
literals, variables, field access, function calls, method calls, match
expressions, if/else, let bindings, record/list literals, binary/unary
operators, lambdas, string interpolation, blocks, casts, for-each,
indexing, slicing, and return.

Sub-expressions are themselves Nodes (not a separate Expr type).

### The direction of the language

The current surface syntax has explicit keywords for `type`, `service`,
`fn`, `pattern`, etc. These are all sugar that produce Nodes. The
migration path (L1 → L2 → L3 in the roadmap) progressively dissolves
compiler knowledge of these surface forms:

- **L1 (active):** The compiler stops knowing what specific types mean
  (`Optional`, `List`, `Map`, `Int`, etc.) — types become properties
  on Nodes defined in `.dag`
- **L2 (future):** The compiler stops knowing what expressions mean
  (`if`, `for`, `match`) — expression semantics move to `.dag`
- **L3 (future):** The compiler stops knowing how to parse surface
  syntax — the parser becomes data-driven

The end state: the compiler is a generic graph processor. All domain
knowledge lives in `.dag` definitions.

---

## 7. Domain Modeling: Facts from Specs

### Core principle: shared facts, not preferences

Every node in a `.dag` model is either:

- **An axiom** — a fact cited from a standard, specification, or API doc
- **A derivation** — composed from axioms via an objective relationship

At any cross-section of any DAG, the content should be
**non-controversial** — a shared fact that people actually agree on. The
resolution for a dispute is "here's the spec," not "here's why I think
this is a good abstraction."

### External dependencies (extdeps)

All external systems are modeled in `dsl/extdeps/` from their actual API
documentation. Real names, real endpoints, real versions. If you can't
link to a spec, you're inventing one.

The codebase currently models ~79 extdep modules across:

- **Cloud providers:** GCP (IAM, Secret Manager, STS), AWS (IAM,
  Lambda, S3, SQS, Secrets Manager), Azure (Identity, Key Vault, Blob)
- **Version control:** Git (object model, refs, diffs), GitHub (repos,
  issues, PRs, gists, auth)
- **LLM providers:** Anthropic, OpenAI, universal message protocol
- **Languages:** Rust, Python, Go (type maps, keywords, runtime
  functions, async patterns, error handling)
- **Infrastructure:** Shell, filesystem, secrets management,
  coordination

Each extdep is graded on spec fidelity (A through C). ~53% are Grade A
(spec-faithful with real names and versions).

### Example: Git object model (`dsl/extdeps/git.dag`)

```dag
module extdeps.git
import std.types { CommitSha, FilePath, Timestamp }

type ObjectType = BlobObj | TreeObj | CommitObj | TagObj

type GitCommit {
  sha: CommitSha
  message: String
  author: GitAuthor
  committer: GitAuthor
  parent_shas: List<CommitSha>
}

service git.Core {
  operation CurrentBranch {
    output { branch: String }
    readonly
    transport shell { argv: ["git", "rev-parse", "--abbrev-ref", "HEAD"] }
    exit { 0 => Unit, 128 => String "Not a git repository" }
  }
}
```

Every fact here is from Git's actual behavior — the object types, the
author/committer distinction (for SLSA provenance), the exit code
mapping. The service operation is grounded in a real shell command.

### Standard library (`dsl/std/`)

The standard library provides shared vocabulary — 26 files covering:

- **`types.dag`** — Kernel primitives, refinement types, branded types,
  temporal types, path types
- **`primitives.dag`** — Complexity contracts for every kernel primitive
  (O() notation for work, output_size, certainty)
- **`containers.dag`** — Collection types (List, Map, Set, Optional)
- **`resources.dag`** — Resource lifecycle model (handles, capabilities,
  acquire/release)
- **`patterns.dag`** — Reusable DAG shapes
- **`languages.dag`** — Language modeling
- **`errors.dag`** — Provider error shapes
- Plus: bits, cloud, credentials, encoding, float, integer, logic,
  unicode, width, etc.

### Why extdeps are the glue (and why this kills glue bugs)

The extdeps ARE the glue, but structural and verifiable:

- System A's extdep models its output from its API docs
- System B's extdep models its input from its API docs
- The compiler connects them; field mismatches are compile errors, not
  runtime bugs
- When System A changes its API, update the extdep → compiler shows
  what breaks

---

## 8. The Compiler Pipeline

### High-level flow

```
.dag source → parse → resolve → infer → emit → target source files
                                                      ↓
                                                 cargo build / go build / etc.
```

The compiler never executes the DAGs it produces. It emits source code in
the target language. That source code, when compiled and run by the
target toolchain, becomes the executor.

### Pipeline stages

| Stage | Current file | What it does | Input → Output |
|-------|-------------|--------------|----------------|
| **Tokenize** | `01_tokenize.dag` | String → flat token list | `String → List<Token>` |
| **Parse** | `02_parse.dag` | Token list → AST | `List<Token> → Module` (tree of Nodes) |
| **Resolve** | `03_resolve.dag` | Multi-module import resolution | `List<Module> → ModuleGraph` (topologically sorted) |
| **Infer** | `04_reconcile.dag` (target: `04_infer.dag`) | Type inference, namespace reconciliation | `ModuleGraph → ResolvedGraph` (return_types filled in) |
| **Emit** | `05_emit*.dag` | Code generation for target language | `ResolvedGraph → target source files` |

Supporting layers (not numbered stages):

| Layer | Current file | What it does |
|-------|-------------|--------------|
| **Pipeline** | `06_pipeline.dag` (target: `compile.dag`) | Orchestrates all stages |
| **Complexity** | `07_complexity.dag` (target: `complexity.dag`) | Cost algebra and complexity proofs |
| **Ownership** | `07_ownership.dag` (target: `ownership.dag`) | Resource ownership analysis |
| **Artifact** | `08_artifact.dag` (target: `artifact.dag`) | Artifact planning and packaging |
| **Trace** | `09_trace.dag` (target: `trace.dag`) | Runtime trace contracts |

### Each phase is a pure function

No phase mutates its input. No phase performs I/O (except filesystem
reads during import resolution). The compiler never executes the DAGs it
produces. Each stage takes a typed input and produces a typed output.

### One type flows through the pipeline

After the convergence migration, the pipeline becomes:

```
source → parse → resolve → infer → emit
           ↓        ↓        ↓       ↓
         Nodes    Nodes    Nodes   TextFiles
         (raw)  (imports  (types
                 linked)  filled)
```

One type (Node) flows through. Each phase enriches the same Nodes rather
than converting between representations.

### The two compiler implementations

| | v1 (Rust, `src/v1/`) | v2 (self-hosted, `src/v2/`) |
|---|---|---|
| **Written in** | Rust (~25 crates, 10 stages) | `.dag` (~14 files, ~20K lines) |
| **Types** | `TypeId` (string references + registry) | `TypeExpr` → Node (structural values, no registry) |
| **Status** | Working, being retired | Self-compiling, becoming primary |
| **Role** | Bootstrap compiler | The real compiler |

v1 exists to bootstrap v2. v2 is written in the language it compiles.
The v1 codebase will be archived once v2 can compile everything v1 still
matters for.

---

## 9. Emission and Target Languages

### Languages are extdeps

Languages (Rust, Python, Go) are external systems with specifications,
modeled the same way GitHub and Git are modeled — from real
documentation with real names and syntax.

Each language extdep in `dsl/extdeps/languages/<lang>/` provides:

| File | What it models |
|------|---------------|
| `types.dag` | Type mappings: `String→String`, `Int→i64`, `List<T>→Vec<T>` |
| `emit.dag` | Keywords, operators, reserved words, container templates |
| `runtime.dag` | Runtime function mappings (how to concat strings, map lists, etc.) |
| `errors.dag` | Error handling conventions |
| `async.dag` | Async/concurrency patterns |
| `imports.dag` | Import/package syntax |

Example from `dsl/extdeps/languages/rust/emit.dag`:

```dag
data rust_type_map: Map<String, String> = {
  "String": "String", "Int": "i64", "Float": "f64", "Bool": "bool",
  "Bytes": "Vec<u8>", "Unit": "()", "Secret": "String"
}

data rust_container_templates: Map<String, String> = {
  "list": "Vec<{0}>", "set": "std::collections::BTreeSet<{0}>",
  "optional": "Option<{0}>", "map": "BTreeMap<{0}, {1}>"
}
```

### Emitter architecture

The emitter receives the resolved Node graph and a language spec. It
generates target-language source files by reading facts from both:

- **Node graph:** what computations to emit, in what order
- **Language spec:** how to express those computations in the target
  language

Current state: three separate emitters (`05_emit_rust.dag` at 3634
lines, `05_emit_python.dag` at 1202 lines, `05_emit_go.dag` at 1226
lines) with duplicated tree-walking logic.

Target state (Phase 4): one shared emit walker drives all target
languages through a common spine. Per-language differences (ownership,
error models, async patterns) live in thin compiler-owned adapters.
Adding a new target = writing a language extdep + optional thin adapter.

### Target-agnostic IR is an invariant

DAG nodes assert truths about computation (types, cardinality, data
flow). These are target-agnostic. How to express those truths in a target
language (`Rust Box<T>`, `C T*`, `Go []T`) is a rendering decision that
lives in the emitter, never in the IR. The structural test: can you swap
the emitter backend without changing the IR?

---

## 10. Transport and I/O

### Invariant 2: World I/O is structural

A DAG node either does I/O or it doesn't — you can tell by looking at
the graph. Only the transport layer performs direct I/O. This is
enforced structurally: `transport != none` on a Node means I/O. Period.

The compiler's only hardcoded transport knowledge is that check. What
kind of I/O (REST, shell, file, gRPC) is determined by the transport
Node's structure — its children carry facts about the transport behavior.

### Transport as compositional Nodes

Transport is a dedicated field on Node (structural awareness), but its
value is a composed Node (compositional behavior):

```
// A REST transport — behavior derives from structure, not variant tag
transport: Node {
  children: [
    Node { name: "base_url", body: "https://api.github.com" },
    Node { name: "auth", children: [
      Node { name: "scheme", body: "Bearer" },
      Node { name: "header", body: "Authorization" },
    ]},
    Node { name: "headers", children: [...] }
  ]
}

// A shell transport — same structure, different facts
transport: Node {
  children: [
    Node { name: "argv", children: [...] },
    Node { name: "env", children: [...] }
  ]
}
```

The emitter derives behavior from structure:
- Has `base_url` child → generate HTTP client code
- Has `argv` child → generate subprocess execution
- Has `auth` child → generate auth injection

New transport kinds (gRPC, WebSocket) are new compositions of facts.
They don't require new compiler match arms.

### Boundary detection

Boundary detection is structural graph analysis, not annotation:

- **Unconnected output ports** = boundaries where data exits the DAG
  (world writes)
- **Unconnected input ports** = entrypoints where data enters the DAG
  (world reads)

Detected by scanning edges, never by annotations.

### Transport in emitted code

Service operations in `.dag` declare transport bindings:

```dag
operation CreateGist {
  input { files: Map<String, String>, public: Bool }
  output { id: String, html_url: String }
  transport rest {
    method: "POST"
    path: "/gists"
    auth: bearer(token)
  }
}
```

The emitter generates the three transport phases for each target:

1. **Prepare** — serialize inputs, build headers, construct request
2. **Execute** — send request over network/shell
3. **Parse** — deserialize response, map exit/status codes to variants

The rendering (reqwest vs net/http vs libcurl) comes from the language
extdep, not hardcoded in the emitter.

---

## 11. Testing Infrastructure

### Three test tiers

| Tier | Cost | What it tests | Transport behavior |
|------|------|--------------|-------------------|
| **DryRun** | XS (30s) | Structure: wiring, cardinality, guards, branching, ordering | All transports mocked |
| **Selective Real** | S-M (5-10min) | Computation: expressions, pure logic, sandboxed I/O | Virtual/sandboxed only |
| **Full Real** | L-XL (30min+) | Integration: live services, real credentials | Real network calls |

### Auto-generated tests from DAG structure

The system auto-generates test code from DAG structure:

1. **DAG test discovery** scans `.dag` files for callables and inline
   test declarations
2. **MockSpec synthesis** builds mock specifications from the DAG's
   transport nodes
3. **Test emission** generates `#[testgen_target]`-decorated Rust test
   functions
4. **Test registration** via `inventory` crate for compile-time
   discovery

Inline test syntax in `.dag` files:

```dag
fixture llm_cloud_env {
  mock cloud_env.config -> { ... }
}

test openai_chat_completion : llm_cloud_env {
  input chat_completion/prepare.api_key -> "..."
  mock chat_completion/execute.response -> rest_response(200, {...})
  expect result.content is String
}
```

### Fidelity ladder

Each transport has a fidelity ladder controlling how "real" the test is:

| Rung | Level | Cost | Description |
|------|-------|------|-------------|
| 0 | PureMock | XS | DryRun interception |
| 1 | VirtualIo | S | In-memory hermetic |
| 2 | Sandboxed | M | Real tempdir with cleanup |
| 3 | RealLocal | L | Real local execution |
| 4 | RealRemote | XL | Real network calls |

### MockSpec architecture

A MockSpec fully describes a test scenario:

- **Boundary mocks** — world-write nodes returning fixed values
- **Transport mocks** — intercepted I/O operations with scripted
  responses
- **Input expectations** — upstream value constraints
- **Input mocks** — DAG entry values
- **Node examples** — per-node I/O examples

The auto-mock system can generate MockSpecs from DAG structure alone,
probing the type registry for compatible default values.

### Testing invariants

- **Behavioral assertions only** — never assert internal implementation
  details
- **Hermetic** — no filesystem/network/environment side effects in unit
  tests
- **No tautological tests** — tests must encode independent
  specifications
- **Performance tested structurally** — via operation counts, not wall
  clocks; every stage returns StageMetrics

### Running tests

```bash
cargo test --workspace --exclude gunbc-dag-tests    # hand-written tests
cargo test -p gunbc-dag-tests                        # auto-generated DAG tests
cargo clippy --all-targets -- -D warnings            # lint
```

---

## 12. The Invariants

These invariants govern all work. They are not aspirational — violating
them triggers a stop-and-discuss protocol.

### Structural invariants

1. **Domain lives in the DSL, not in Rust.** If something can be
   expressed in `.dag` files, it must not be hardcoded in Rust
2. **World I/O is structural.** A DAG node either does I/O or it
   doesn't — visible from the graph
3. **Extdeps implement specifications, not abstractions.** Real names,
   real endpoints, real versions
4. **Each compiler phase is a pure function.** No mutation, no I/O
5. **Composition through layers, not abstraction.** Each layer only
   knows about layers below it
6. **DAG nodes are facts, rendering is separate.** IR is
   target-agnostic; rendering is the emitter's job
7. **The interpreter maps IR to execution — nothing more.** No domain
   logic, no compiler logic
8. **Every expression lowers to structural DAG nodes or compilation
   fails.** No opaque fragments, no fallback nodes
9. **Correctness by construction, not by validation.** If a property
   must hold, the API makes violations unrepresentable

### Sustainability invariants

- **No duplicate representations.** Every fact encoded in exactly one
  place
- **No case enumeration for open sets.** Structural walks over match
  arms that enumerate known cases
- **No fallbacks that fabricate.** Every code path succeeds fully or
  fails with a clear error
- **No parallel implementations.** Same computation in two forms → one
  must be deleted
- **Explicit boundary contracts.** Make illegal states
  unrepresentable at pipeline stage boundaries
- **Single-authority metadata.** One producer per piece of metadata

### Performance invariant

Performance is a correctness property. For every exposed interface, the
worst-case time and space bound must be known before committing to the
design. Accidental quadratic behavior, repeated rescans, and large
incidental clones are design bugs.

### Cost of change (the governing metric)

When the language grows by one type, one expression, or one transport,
how many files need editing? The sustainable compiler is one where that
number is **1**.

---

## 13. Self-Hosting and Bootstrap

### Bootstrap chain

```
v1 (Rust) compiles v2 .dag → Rust → rustc → v2-stage0 (binary)
v2-stage0 compiles v2 .dag → Rust → rustc → v2-stage1 (binary)
v2-stage1 compiles v2 .dag → Rust → rustc → v2-stage2 (binary)
```

**Fixed point:** stage1 output == stage2 output (byte-identical). The
compiler reproduces itself.

### Completed milestones

| Milestone | What | Date |
|-----------|------|------|
| Self-compile pipeline | v2 processes its own `.dag` through all 5 stages | 2026-03 |
| Bootstrap A5 | v1 → stage0 → stage1 (`cargo check`) | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output | 2026-03 |
| A7 Phase 1 | Self-compile: 0 `cargo check` errors | 2026-03 |

### What dies with self-hosting

The v1 bootstrap carries heuristics (S76-S81) that are scaffolding:
`variant_to_enum`, `struct_field_types`, `recursive_fields`,
`optional_fields`, `enum_accessor_fields`. These die naturally — the v2
compiler derives everything structurally from `.dag` source.

### Convergence validation

Each convergence step is validated by re-bootstrapping:

1. Modify `.dag` source (e.g., TypeExpr→Node step)
2. Self-compile: v2 compiles modified `.dag` → new binary
3. New binary compiles same `.dag` → verify fixed point holds
4. If fixed point breaks → behavioral difference → investigate

The fixed point IS the regression test.

---

## 14. The Convergence Migration

The project is in the middle of a large migration to dissolve all
compiler-internal types into Nodes. This is tracked as three layers in
the roadmap.

### L1: Types (active — 489 violations remaining)

The compiler stops knowing what specific types mean. `Optional`, `List`,
`Map`, `Int` become properties on Nodes defined in `.dag`, not special
cases hardcoded in the compiler.

| Category | Count | What the compiler still "knows" |
|----------|-------|-------------------------------|
| Connective field + `Conj`/`Disj` | 265 | Product vs coproduct semantics |
| Type constructors | 121 | `leaf_node`, `optional_node`, `container_node` |
| Type-name comparisons | 59 | `.name == "Optional"`, `"Map"`, etc. |
| `node_is_*` predicates | 24 | Type-specific dispatch |
| `builtin_type_kind()` calls | 20 | Hardcoded builtin classification |

L1 acceptance: `BuiltinTypeKind` deleted, all type constructors deleted,
zero type-name string matching, `connective` field removed from Node,
fixed point still holds.

### L2: Expressions (future)

The compiler stops knowing what expression forms mean. `if`, `for`,
`match`, `let` become `.dag`-defined structures.

### L3: Syntax (future)

The compiler stops knowing how to parse surface syntax. The parser
becomes data-driven.

### Already completed dissolutions

- TypeExpr → Node (8 variants deleted)
- Expr → Node (21 variants deleted; ExprData now lives on Node)
- TransportBinding → composed Nodes (4-variant enum deleted)
- TypedNode/TypedExpr/TypedMatchArm/TypedFieldInit/TypedStringPart
  (all dissolved)
- PortContract, ResponseMapping, ExitMapping, MockResponseDef (all
  dissolved)

---

## 15. Current State and Roadmap

### Current state (2026-03-22)

- **v2 stage0 compiles with 0 cargo check errors**
- Fixed point holds (stage1 == stage2)
- Diagnostic ratchet at 25 (blocking Phase 2)
- L1 type dissolution: 489 violations across 8 files
- Self-compile still hangs on full pipeline (algorithmic bottleneck)
- Performance: tokenize+parse down to ~24ms

### Phase execution order

| Phase | What | Gate |
|-------|------|------|
| **Phase 1** (active) | Naming cleanup, diagnostics → 0, L1 type dissolution | Diagnostics ratchet reaches 0; naming lands |
| **Phase 2** | `gist` end-to-end (compile → build → run) | Emitted gist crate builds and runs in dry-run |
| **Phase 3** | Compile contract, ownership wiring, v1 retirement | v2 compiles everything v1 still matters for |
| **Phase 4** | Shared emit spine, generated tests as projections | New backend = language facts + adapter, no shared-core changes |
| **Phase 5** | Remaining convergence (Token, Module, Diagnostic → Node) | One Node-centric internal model |

### Business track (parallel)

First target integration: Cursor cloud agent API / Composer 2 surface.
AG1 modeling can start once Phase 2 proves the compiler can emit a real
program. The integration validates that the compile-time graph approach
works for real agent workflow orchestration.

### After all phases complete

The system is:
- **Self-hosted** — written in `.dag`, compiled by itself
- **Structurally unified** — one type (Node) flows through the pipeline
- **Compositional** — everything is Conj/Disj + kernel primitives
- **Target-polymorphic** — Rust, Python, Go from the same source
- **Bootstrap-free** — no v1 dependency
- **Verified by fixed point** — compiler reproduces itself

---

## 16. Codebase Map

### Repository structure

```
src/v1/                          v1 compiler (Rust, 25 crates, 10 stages)
  00_foundation/                   IR, contracts, macros
  01_surfaces/                     CLI, codegen, workflow
  02_pipeline/                     Driver/orchestration
  03_source/                       Parse, resolve
  04_semantics/                    Typecheck
  05_graph/                        Lower, eval
  06_artifacts/                    Derive
  07_emit/                         Code emission
  08_materialize/                  Resolve, transport, blob, interpreter
  09_execute/                      Executor
  10_test/                         Test support, generated tests

src/v2/                          v2 self-hosted compiler (.dag)
  00_core.dag                      Compiler domain model (Token, AST, ExprData, Node)
  01_tokenize.dag                  Lexer
  02_parse.dag                     Parser (Pratt binding powers)
  03_resolve.dag                   Import resolution
  04_reconcile.dag                 Type inference (target name: 04_infer.dag)
  05_emit.dag                      Shared emit helpers
  05_emit_rust.dag                 Rust backend
  05_emit_python.dag               Python backend
  05_emit_go.dag                   Go backend
  06_pipeline.dag                  Compiler driver (target name: compile.dag)
  07_complexity.dag                Cost analysis (target name: complexity.dag)
  07_ownership.dag                 Ownership proofs (target name: ownership.dag)
  08_artifact.dag                  Artifact planning (target name: artifact.dag)
  09_trace.dag                     Trace contracts (target name: trace.dag)
  tests/                           Integration tests

dsl/                             Domain source files
  std/                             Standard library (26 files)
    types.dag                        Base types, refinements, branded types
    primitives.dag                   Complexity contracts for builtins
    containers.dag                   Collection types
    resources.dag                    Resource lifecycle model
    patterns.dag                     Reusable DAG shapes
    ...
  extdeps/                         External dependency models (~79 modules)
    git.dag                          Git object model
    shell.dag                        POSIX shell
    cargo.dag                        Rust build system
    github/                          GitHub API (repos, issues, PRs, gists)
    cloud/                           GCP, AWS, Azure
    llm/                             Anthropic, OpenAI
    languages/                       Rust, Python, Go (type maps, runtime, etc.)
    ...
  config/                          Build configuration
  tools/                           Bootstrap and codegen tools
```

### Key documentation files

| File | What it covers |
|------|---------------|
| `CLAUDE.md` | Project instructions and invariants (overrides defaults) |
| `INVARIANTS.md` | Full invariant catalog with postmortems |
| `MODELING.md` | DAG modeling quality guidelines |
| `ROADMAP.md` | Master schedule (canonical execution order) |
| `DESIGN-v2-compiler.md` | v2 architecture and TypeId→TypeExpr design rationale |

### Key source files for understanding the system

| File | What to learn |
|------|--------------|
| `src/v2/00_core.dag` | The compiler's data model (Node, ExprData, Token, Module) |
| `src/v2/04_reconcile.dag` | The inference hotspot (4871 lines, mixed concerns) |
| `src/v2/05_emit_rust.dag` | How Nodes become Rust code |
| `src/v2/06_pipeline.dag` | How stages wire together |
| `dsl/std/types.dag` | The type vocabulary |
| `dsl/extdeps/git.dag` | A clean example of spec-faithful domain modeling |
| `dsl/extdeps/languages/rust/emit.dag` | Language-as-extdep pattern |
