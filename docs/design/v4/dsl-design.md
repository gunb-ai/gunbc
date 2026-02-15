# Causal DAG Language: Design Document

**Status**: Working Draft — February 2026
**Repo**: New (harvesting from gunbc, the-gunbai, gunb.ai)

**Scope:** Sections 1–10 are **normative** language and runtime behavior. Sections 11–12 and Appendices contain rationale, comparison, worked examples, and implementation plans — they are **informative** and non-binding.

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Three Generations of Evidence](#2-three-generations-of-evidence)
3. [Design Principles](#3-design-principles)
4. [Language Constructs](#4-language-constructs) — types, fn (functor protocol), resources, services, patterns, journeys, pipelines
5. [Module System and Discovery](#5-module-system-and-discovery)
6. [Terminal Progress Model](#6-terminal-progress-model)
7. [Resource Model](#7-resource-model) — conflict keying, mode inference
8. [Error Model and Execution Semantics](#8-error-model-and-execution-semantics) — failure propagation, scheduling, loops, NodeId stability
9. [Compiler Pipeline](#9-compiler-pipeline)
10. [Multi-Target Emission](#10-multi-target-emission) — functor protocol compilation, 100%/0% split
11. [What to Harvest](#11-what-to-harvest)
12. [Phasing](#12-phasing)

**Appendices**:

- [A. Content Upsert (Makegen)](#appendix-a-content-upsert-makegen)
- [B. Cloud Credential Acquisition (GCP)](#appendix-b-cloud-credential-acquisition-gcp)
- [C. Service Composition (Gist Snapshot)](#appendix-c-service-composition-gist-snapshot)
- [D. CI Pipeline](#appendix-d-ci-pipeline)
- [E. Tool Installation (Upsert)](#appendix-e-tool-installation-upsert)
- [F. LLM Review Workflow](#appendix-f-llm-review-workflow)
- [G. Rendering / Emission](#appendix-g-rendering--emission) (collapsed)
- [H. Pattern Catalog](#appendix-h-pattern-catalog)
- [I. Inspiration Targets](#appendix-i-inspiration-targets)
- [J. Cross-Repository Capability Matrix](#appendix-j-cross-repository-capability-matrix) (trimmed)
- [K. Root Cause Analysis](#appendix-k-root-cause-analysis--why-gunbc-got-out-of-control)
- [L. References](#appendix-l-references) (consolidation status + file paths)
- [M. Competitive Landscape](#appendix-m-competitive-landscape-and-alternatives-analysis) (trimmed)
- [N. Model-Based Testing and Auto-Generated Mocks](#appendix-n-model-based-testing-and-auto-generated-mocks)

---

## 1. Problem Statement

We need a language for authoring causal DAGs that:

1. **Compresses graph authoring** from thousands of lines of host-language builder code to tens of lines of declarations, while preserving the structural guarantees (acyclicity, type safety, port saturation) that the IR provides.

2. **Makes terminal progress structural** — the **progress skeleton** (boundaries, wave depth, grouping opportunities, scatter expansion points) is known at compile time; runtime supplies **instance counts** for dynamic expansions (loop fan-out). This enables static visualization (before execution), live progress, and post-execution replay from the same manifest.

3. **Solves discovery by construction** — every `.dag` file is auto-discovered via the filesystem. No registration macros, no hardcoded lists, no islands. dag-viz can see itself.

4. **Models resources with lifecycle** — acquire, use, release. Resources are declared (`uses fs: Filesystem`), not manually wired through environment nodes and `res:` port conventions.

5. **Is language-agnostic** — `.dag` files compile to a target-independent IR. Codegen backends emit Rust, Go, Python, TypeScript, or any language. The semantics are simple enough that a register machine (MIPS, WASM) is a valid target.

6. **Generates 100% of the host-language code** — the compiler emits types, pure function implementations (from `fn` functor bodies), transport wiring, test harnesses, CLI entrypoints, progress renderers, and Makefile/CI YAML. The developer writes everything in `.dag` files.

### Requirements Traceability

Each requirement maps to language constructs, a compiler pass, and a concrete output artifact.

| Requirement | Language mechanism | Compiler pass | Output artifact |
|---|---|---|---|
| 1. Compress graph authoring | `journey`, `pattern`, implicit edges, `fn` | PatternExpand → Lower | GraphIR (same Nodes/Edges as hand-wired builders) |
| 2. Structural progress | SubDag boundaries, loop expansion points, `@interactive` | Derive | ProgressManifest (topology, groups, scatter points) |
| 3. Discovery by construction | Filesystem module graph, `dag.toml` manifest | Discover | ModuleGraph = workspace catalog of journeys/tools |
| 4. Resources with lifecycle | `resource`, `uses`, conflict keys, `@auth` | TypeCheck → Lower → Validate | Acquire/release nodes, conflict errors, mock specs |
| 5. Language-agnostic | Target-independent IR, `CodegenBackend` trait | Emit | Rust / Go / Python / TypeScript code |
| 6. 100% generated host code | Functor protocol (`fn`), service descriptors, `@mock_response` | Emit | Types, transports, CLI, tests, progress, Makefile/CI YAML |

---

## 2. Three Generations of Evidence

### 2.1 gunb.ai (Go, v1)

**What it was**: DAG-based LLM orchestration. Go + protobuf. Ticket/lease execution.

**What it proved**:
- DAGs work for complex workflow orchestration
- Terminal output capture (`CaptureWriter`) is essential for understanding subprocess behavior
- Lease-based resource coordination works for parallel execution

**What failed**:
- All tests handwritten — no structural derivation
- Adding a tool: ~500 lines across 5+ files, all manually coordinated
- Terminal output was captured but not modeled — couldn't see DAG shape during execution

### 2.2 the-gunbai (Rust, v2)

**What it was**: Understanding-driven codegen. Structured documents about external systems that generate integration code. 40+ understandings, 195+ behaviors.

**What it proved**:
- Codegen from structured knowledge scales
- Contract tests CAN be generated from behavior patterns
- The TUI progress system was the standout UX achievement:
  - Four rendering modes: Plain, Inline, TUI (full-screen animated DAG), JSONL
  - Edge pulse animations showing "energy flow" through the graph
  - Wave-based layout (nodes grouped by topological depth)
  - Scatter group progress for parallel tasks (`[2/5]`)

**What the TUI looked like**:
```
gist ─ 3/6 ━━━━━━━━━━━░░░░░░ 50%
  [✓ branch] [✓ files] [◐ render] [○ upload] [○ parse] [○ done]

Expanded:
  Wave 0           Wave 1          Wave 2          Wave 3
╭────────────╮  ╭───────────╮  ╭──────────╮  ╭──────────╮
│ ✓ branch   │──│ ◐ render  │──│ ○ upload │──│ ○ parse  │
│   4ms      │  │   48ms    │  │          │  │          │
╰────────────╯  ╰───────────╯  ╰──────────╯  ╰──────────╯
```

**What failed**:
- Behavior/testing still largely handwritten despite codegen
- No IR — the graph was implicit in runtime orchestration
- The TUI was runtime-only — couldn't visualize a graph before execution

### 2.3 gunbc (Rust, v3 — current)

**What it was**: Full IR with typed DAGs, structural invariants, transport boundaries, proof-obligation testgen.

**What it proved**:
- IR model enables structural guarantees (acyclicity, type safety, cardinality)
- Proof-obligation testgen works (2,334 generated tests, 885 handwritten — 73% generated)
- Transport boundary pattern (prepare/execute/parse) cleanly isolates I/O
- DryRun interception at transport boundaries enables zero-I/O testing
- Content upsert, credential chain, and transport triplet are universal patterns
- Frame-based progress display with pure `build_frame()` function

**What failed**:
- No front-end language — 7,000+ lines of hand-wired graph builders
- Transport types colonized the IR (17 transport modules inside `core/ir/`)
- Discovery was segregated (6 registration islands)
- dag-viz couldn't visualize itself (not in hardcoded workspace DAG list)
- Progress rendering was rebuilt from scratch (lost the-gunbai's TUI quality)
- Resources worked but lifecycle was implicit
- `Value`/`ValueExpr` parallel hierarchies
- Endless refactoring: design docs for fixing the design grew faster than fixes shipped

**The discovery problem**:
```rust
// gunbc-dag/src/workspace/subdags/mod.rs — HARDCODED
pub fn build_workspace_dag() -> Result<Dag<WorkspaceOp>, BuilderError> {
    dag.add_node(makegen::build_makegen_subdag());
    dag.add_node(clippy::build_clippy_lint_all_subdag());
    dag.add_node(deps::build_deps_install_subdag()?);
    // ... manually listed
    // New tools must be manually added here — e.g., dag-viz isn't in this list
}
```

### 2.4 Summary Table

| Concern | gunb.ai | the-gunbai | gunbc | DSL (target) |
|---------|---------|------------|-------|---------------|
| Graph authoring | Handwritten Go | Handwritten Rust | Handwritten Rust builders | `.dag` files |
| Tests | All handwritten | Mostly handwritten | 73% generated (testgen) | 95%+ generated |
| Terminal progress | Captured stdout | Full TUI with DAG viz | Frame-based, no TUI | Structural: in the IR |
| Discovery | Manual | Manual | 6 registration islands | Module system |
| Resources | Lease/heartbeat | Implicit | Typed, no lifecycle | First-class lifecycle |
| Target language | Go only | Rust only | Rust only | Language-agnostic |
| Adding a tool | ~500 lines, 5 files | ~300 lines, 3 files | ~200 lines, 2 files | ~20 lines, 1 file |

---

## 3. Design Principles

**P1: Causality is a DAG.** Every workflow is a directed acyclic graph of typed, pure nodes connected by typed edges, with I/O isolated to transport boundaries.

**P2: One type, every level.** A node is either opaque or contains a sub-DAG. Same structure from shell commands up to multi-service pipelines. (From V3 minimal spec.)

**P3: No freeform strings for semantics.** Types are enums. Identifiers are validated newtypes. Extension lanes are declared, not freeform. (From V2 P2.)

**P4: If it validates, wiring is correct.** The compiler proves structural correctness once. Developers test business logic. (From gunbc SPEC.md.)

**P5: Transport is late-bound.** The core IR does not contain backend-specific transport types (no `reqwest`, no `subprocess`). The DSL contains transport *descriptors* (`@rest`, `@shell`, `@file`) that lower into a generic `ExternalCall` node + structured `TransportDescriptor`. Concrete client code is determined by the codegen backend. (From gunbc design commitment #7.)

**P6: Progress is a view, never a constraint.** The progress display observes the DAG and infers sections (from SubDag boundaries), groups (from parallel siblings), and waves (from topological depth). It never imposes structure on the DAG or requires authors to declare display metadata. The only "grouping" information progress can rely on is structure that already affects execution: SubDag expansion boundaries, pipeline stages, and loop expansions. The DAG never declares *display groups*; it may declare *execution affordances* (e.g., `@interactive`, `@streamed`) that affect terminal ownership and capture semantics, which progress reacts to. (Synthesized from gunb.ai's CaptureWriter + the-gunbai's TUI + gunbc's FrameRenderer.)

**P7: Discovery is the filesystem.** Every `.dag` file in the project is auto-discovered. The module graph IS the workspace DAG. No registration macros, no hardcoded lists. (New — fixing gunbc's 6 registration islands.)

**P8: Resources have lifecycle.** Acquire, use, release. The compiler inserts lifecycle nodes, detects conflicts, and generates mock specs. (From V2 P6, extending gunbc's `res:` model.)

**P9: The language is total.** The DSL is declarative and total. `fn` functors are pure and operate on finite data — no general recursion, no I/O primitives, no unbounded loops. Journeys and pipelines are the imperative shell that sequences I/O through services and resources. Side effects occur only at runtime at transport boundaries. Compilation always terminates. (From Dhall inspiration.)

**P10: Language-agnostic.** `.dag` files are like `.proto` files. The IR is the contract. Codegen backends are plugins. The semantics (node = pure function, transport = syscall, edge = data flow) map to any execution model.

### Graph Invariants (from gunbc `overview.md`)

These invariants define what makes a well-formed graph. They are **carried forward** from gunbc into the DSL — the DSL compiler must enforce all of them.

**I1: Node Purity.** Every node is either pure (deterministic, no side effects) or a transport execute node (the designated I/O boundary). Pure nodes can be memoized, parallelized, and reasoned about locally. In the DSL: `fn` nodes and prepare/parse nodes are pure; `service` operations compile to transport execute nodes.

**I2: Transport Boundary.** All world I/O flows through `TransportRequest → Execute → TransportResponse`. Domain ops construct requests (pure). Transport executes them (I/O boundary). Result processing parses responses (pure). The DSL's `@rest`/`@shell`/`@file` annotations compile to this triplet.

**I3: Observable I/O.** All I/O operations are visible as explicit nodes in the graph structure. DryRun can intercept any transport. Visualization shows I/O nodes explicitly. Composition can wrap I/O in retry/circuit breaker. Hidden I/O inside opaque nodes is forbidden.

**I4: Minimal Graph.** Workflows use the minimum nodes necessary, with maximum reuse of canonical patterns. No redundant/dead nodes. Use patterns (`upsert`, `content_upsert`, `transaction`) instead of ad-hoc equivalents.

**I5: Deterministic Ordering.** Fan-in produces deterministic collection order via canonical edge ordering. Sort key: `(from_node_id, from_port_name, edge_index)`. Same DAG always produces the same collection order.

**I6: No Escape Hatches.** The system cannot be bypassed. If I/O must go through transport, there is no function to skip it. "Just this once" exceptions don't exist.

**I7: No Fallbacks.** Operations either succeed or fail. No silent degradation. No default values when something is missing — fail. No "best effort." Fail fast. (Exception: explicit `match` arms that model alternative paths.)

**I8: No Runtime Warnings.** At runtime, errors are clear signals, not optional advisories. If something is wrong, the operation fails — it doesn't print a warning and continue. `#[allow(warnings)]`-style suppression doesn't exist. (The *compiler* may emit warnings for style/lint issues like redundant imports or naming conventions. These are advisory and do not affect runtime behavior. Runtime behavior is errors or success — never "warning but proceed.")

### The Erasure Lemma

This principle makes the "no meta-annotations" ban operational:

> **Metadata erasure is semantics-preserving:** removing all non-semantic metadata (display names, docs, tags, source spans, visualization hints) does not change the workflow's observable behavior (given the same transport results).

| Class | Allowed? | Rule | Examples |
|---|---|---|---|
| **Descriptive** | Yes | Must be erasable without behavior change | Display names, docs, tags, ownership, version, source spans, logging labels |
| **Optimization hints** | Yes (with rule) | Must not change functional results | Cost estimates, parallelism hints, cache hints |
| **Semantic modifiers** | Banned | Must be modeled structurally | Guards that skip required values, implicit resource edges |

If an annotation can change results, it is not a hint — it must be modeled structurally (nodes/edges/types). This is the test for whether C1 (below) is satisfied.

### Compiler-Enforced Policies

These are normative invariants enforced by the compiler. They are the primary anti-drift mechanism — each prevents a class of gunbc failure mode (see Appendix K).

**C1: Annotations desugar to structure.** Every `@trait` must compile into explicit IR fields/nodes. No annotation may influence runtime behavior without an observable structural representation. (Promotes Appendix K.6 G1.)

**C2: Stable identities.** `TypeId`, `ServiceId`, `OperationId`, and `NodeId` are derived from fully-qualified names and call-site paths, not build order. Reformatting `.dag` files must not change IDs.

**C3: Deterministic compilation.** Given identical `.dag` inputs and compiler version, emitted artifacts are byte-for-byte identical. All discovery and iteration orders are canonicalized (filesystem paths sorted, map keys sorted, node ordering = stable topo with tie-breaker = NodeId).

**C4: Effects are boundary-only.** Runtime side effects occur only at compiler-emitted transport execute nodes. Prepare/parse nodes and `fn` functors are pure by construction.

**C5: Control edges are explicit.** Dataflow edges arise only from value references. Ordering without dataflow must be expressed via `after` dependencies. Guards do not imply ordering.

**C6: Shell is structured.** `@shell` compiles to argv execution (no implicit shell parsing). Placeholder substitution is argument-based. A raw shell mode (`@shell(raw: "...")`) is available as a deliberate, non-hermetic opt-in.

**C7: REST encoding is defined.** Path parameters are URL-encoded. Query parameters are URL-encoded and sorted deterministically. Body serialization uses canonical JSON (stable key ordering) when used for hashing or test comparison.

**C8: Hermeticity is explicit.** Every transport boundary node is classified as `Hermetic` or `External`. This classification is preserved through lowering and visible to executors and test categorization. Defaults: `@rest` → External, `@file` → Hermetic, `@shell` → must declare `@hermetic` or `@external` (no default — forces author decision).

**C9: Secrets are redacted.** `Secret` values never render in progress output, preambles, error boxes, or JSONL events by default. An explicit `reveal(secret)` is available but disallowed in CI mode.

**C10: Repetition is bounded.** `@retry` requires finite `max`. `poll` requires finite `timeout`. `while` is not available as general syntax. The compiler rejects unbounded repetition.

**C11: Compatibility rules.** Adding an optional field with a default is non-breaking. Removing, renaming, or retyping a field is breaking. Changing a refinement constraint is breaking unless it only loosens. Transport bindings (`@rest` path/method, `@shell` argv) are part of the compatibility surface.

### Policy Traceability Matrix

Each policy maps to a compiler pass that enforces it, an artifact that proves it, and a concrete consequence if violated.

| Policy | Enforced in | Artifact | "Free" test / check | If violated |
|---|---|---|---|---|
| **C1** Annotations → structure | Lower | IR fields on nodes | `dag expand` shows structural form | Annotations become opaque magic; runtime behavior diverges from spec |
| **C2** Stable identities | Resolve | `NodeId` derived from fq path | Replay stability test; `dag manifest` diff | Progress replay breaks; generated code diffs are noisy |
| **C3** Deterministic compilation | All passes | Byte-identical output | CI: `dag emit --check` (no diff) | "Why did this file change?" churn; flaky generated tests |
| **C4** Effects boundary-only | Lower + Validate | Execute nodes tagged as boundaries | Bucket A: `TransportInterceptable` | DryRun can't intercept; CI becomes flaky or needs real I/O |
| **C5** Control edges explicit | Resolve + Lower | `after` edges in GraphIR | `dag viz` shows all ordering edges | "Happened to work because of insertion order" bugs; patterns like `upsert` silently wrong |
| **C6** Shell is structured | Lower | `ShellSpec { argv }` in IR | Hermetic shell tests pass cross-platform | Quoting bugs; shell injection; different behavior per OS |
| **C7** REST encoding defined | Emit | Canonical URL + JSON in transport | Mock comparison uses canonical form | Test fixtures break on key reordering; path encoding bugs |
| **C8** Hermeticity explicit | Lower + Validate | `io_scope` field on execute node | Bucket D: test categorization (hermetic vs external) | CI accidentally runs external ops; slow/flaky tests |
| **C9** Secrets redacted | Emit + Runtime | `Secret` type never serialized to output | CI mode rejects `reveal()`; JSONL scan | Credential leakage in logs, progress output, or event streams |
| **C10** Repetition bounded | Validate | Compile error on unbounded loops | Totality check passes | Non-terminating compilation or execution; P9 violated |
| **C11** Compatibility rules | TypeCheck (incremental) | Breaking-change report | `dag compat --check` CI gate | Silent breaking changes; downstream consumers fail at runtime |

---

## 4. Language Constructs

Eight constructs:

```
type        — data shapes
fn          — pure transformations (constrained functor protocol)
resource    — acquirable capabilities with lifecycle
service     — operations with typed I/O and transport annotations
pattern     — reusable DAG shapes with typed slots
journey     — composed flows (main authoring surface)
pipeline    — staged multi-journey workflows
module      — namespace, visibility, discovery metadata
```

### 4.1 Types

```
// Primitives (defined in std/types.dag, composed from structural primitives):
// Unit, Bool, String, Int, Float, Bytes, Json, Secret

// Records
type AccessToken {
  token: Secret
  scheme: AuthScheme
  expires_at: String?         // ? = optional (zero-or-one)
}

// Sum types (tagged unions)
type AuthScheme
  = Bearer
  | Header { name: String }
  | Basic { username: String }

// Enums
type CloudRuntime = GitHubActions | Metadata | LocalDev
type Platform = Linux | MacOS | Windows

// Collections
// List<T>    — zero or more
// Set<T>     — zero or more, unique
// Map<K, V>  — key-value pairs
```

// Refinement types (constraints on primitives — enables auto-fuzzing)
// See Appendix N for full model-based testing implications.
type CommitSha = String @pattern("^[a-f0-9]{40}$")
type RetryCount = Int @range(min: 1, max: 5)
type HttpStatus = Int @range(min: 100, max: 599)
type Email = String @pattern("^[^@]+@[^@]+\\.[^@]+$")
type Port = Int @range(min: 1, max: 65535)
type GistId = String @format(uuid)
type SecretValue = Secret @non_empty

// @pattern(regex)    — string must match regex
// @range(min, max)   — numeric bounds (inclusive)
// @format(preset)    — well-known format (uuid, uri, iso8601, semver)
// @non_empty         — string/list must have length > 0
// @one_of(values)    — value must be one of the listed literals
// @length(min, max)  — string/list length bounds
```

Design choice: the **DSL surface** keeps cardinality simple — `T` is required-one, `T?` is optional, `List<T>` is zero-or-more. Authors never write interval math. But the **compiler's internal type model** retains the full cardinality algebra (`Cardinality { min, max }` with join/meet/product/satisfies) because it powers coercion checking, automatic test generation, and boundary-value coverage. The surface is simple; the compiler is precise.

Design choice: refinement types constrain primitives with structural metadata that the compiler uses for three purposes: (1) validation at type-check time, (2) auto-generation of test inputs at derive time (see Appendix N), and (3) documentation of expected shapes for service consumers. Per Appendix K.6 guardrail G1, refinement annotations desugar to structural constraints — `@pattern` compiles to a validation predicate in the type's DAG representation, not opaque metadata.

#### Types ARE DAGs — same infrastructure, same composition rules

This is not a metaphor. In gunbc, a type like `Url` is literally a `Dag<TypeOp>` — a causal chain of validation operations:

```
String (raw) → [NonEmpty check] → [URL pattern check] → Url (validated)
```

Type validation IS a causal chain. Using `Dag<TypeOp>` makes this explicit and reuses all DAG infrastructure (composition, lowering, validation). The DSL inherits and extends this.

**TypeOp — the operations in a type DAG:**

```
TypeOp = Identity              // pass-through (base of every type)
       | Validate(Predicate)   // check a condition (NonEmpty, Matches, InRange, ...)
       | Transform(Coercion)   // safe type conversion (Int → Json, Url → String)
       | Wrap(WrapperKind)     // Optional, List, Set, NonEmptyList, Map
       | Unwrap(WrapperKind)   // extract from container
```

**Building types from composition:**

```
// Primitives — identity DAGs (single node, no validation)
String  = Dag { Identity("String") }
Int     = Dag { Identity("Int") }
Bool    = Dag { Identity("Bool") }

// Refined types — validation chains
Url     = Dag { Identity("String") → Validate(NonEmpty) → Validate(Matches("^https?://...")) }
Email   = Dag { Identity("String") → Validate(NonEmpty) → Validate(Matches("^[^@]+@...")) }
Port    = Dag { Identity("Int") → Validate(InRange(1, 65535)) }

// Container types — wrapper nodes with inner type DAGs as SubDags
List<Url>       = Dag { Wrap(List) → SubDag(Url) }
Optional<Email> = Dag { Wrap(Optional) → SubDag(Email) }
Set<String>     = Dag { Wrap(Set) → SubDag(String) }
Map<String, V>  = Dag { Wrap(Map) → SubDag(V) }

// Compound types compose freely
NonEmpty<List<Url>> = Dag { Wrap(NonEmptyList) → Validate(NonEmpty) → SubDag(Url) }
```

**Three-level coercion lattice (already implemented in gunbc):**

Every type has three structural properties the compiler reasons about:

| Level | Property | Example | Safe direction (upcast) |
|---|---|---|---|
| L1 | Cardinality | `[1,1]` → `[0,∞)` | Wider interval |
| L2 | Base type | `Int` → `Json`, `Url` → `String` | More general base |
| L3 | Predicates | `NonEmpty ∧ Matches(url)` → `NonEmpty` | Fewer predicates |

A coercion `A → B` is safe when all three levels widen:

```
can_safely_coerce(source, target) =
    source.cardinality ⊆ target.cardinality   -- L1: interval containment
  ∧ source.base_type ≤ target.base_type       -- L2: base type lattice
  ∧ source.predicates ⊇ target.predicates     -- L3: predicate entailment
```

**Base type lattice:**

```
        Json (top — accepts any structured value)
       / | \
    Int Bool String
               |
              Url (refined String)

    Unit (bottom — no value)
```

Upcasts are safe: `Url → String → Json`. Narrowing (e.g., `String → Url`) is never implicit — it requires an explicit validation node. This prevents "meaning leaking" at the type level.

**Cardinality algebra (lattice + semiring):**

Cardinality is modeled as a closed interval `[min, max]` on ℕ ∪ {∞} with full algebraic structure:

- **Join** (least upper bound): `[1,1] ∨ [0,1] = [0,1]` — what can hold either
- **Meet** (greatest lower bound): `[0,1] ∧ [1,∞) = [1,1]` — what satisfies both
- **Product** (nested iteration): `[1,∞) × [1,1] = [1,∞)` — flattened result
- **Satisfies** (subset containment): `[1,1] ⊆ [0,∞)` — can this output feed that input?

These operations are property-tested for algebraic laws (reflexivity, transitivity, commutativity, associativity, idempotence, absorption) via proptest. The five standard cardinalities (`Zero`, `One`, `ZeroOrOne`, `ZeroOrMore`, `OneOrMore`) are named constants, but arbitrary intervals like `[2, 5]` work without code changes.

**Predicate entailment:**

Predicates form their own partial order: `Predicate::entails(&self, other)` checks whether `self` is at least as strict as `other`. Key rules:

- `InRange(0, 100)` entails `InRange(0, 200)` (tighter range)
- `Equals(5)` entails `InRange(0, 10)` (specific value in range)
- `And(A, B)` entails `A` (conjunction is stricter)
- `NonEmpty` entails nothing about `Matches` (unrelated predicates)

**What this gives the DSL compiler:**

1. **Structural subtyping via DAG traversal.** When a `fn` takes `String` and the caller passes `CommitSha`, the compiler walks the type DAG: `CommitSha` → base type is `String` → safe upcast (dropping the `@pattern` predicate). No explicit cast rules needed.

2. **Automatic coercion insertion.** When an edge connects incompatible types, the compiler checks `can_safely_coerce_to()`. If L1/L2/L3 all widen, the coercion is inserted as a `Transform` node. If any level narrows, it's a compile error with a diagnostic: "output might be empty (min=0) but input requires at least 1 element."

3. **Test generation from type structure.** The type DAG tells the fuzzer how to generate values at every level. `Cardinality::boundary_values()` produces edge cases from the interval (e.g., `[2,5]` → test with `{1, 2, 3, 5, 6}`). `Predicate` constraints generate valid/invalid examples. Container types recurse into their element type DAGs.

4. **Cardinality-aware obligation generation.** The testgen system uses `Cardinality::test_cases()` to generate boundary-value coverage for every port: a `ZeroOrOne` port gets tests with 0 and 1 elements; a `OneOrMore` port gets tests with 1 and 2 elements.

**The standard library defines the type foundation.** `std/types.dag` provides primitive types, common refinements (Url, Email, FilePath, Port), and container constructors. User-defined types extend the same DAG. The `TypeRegistry` stores named type DAGs and resolves references during compilation.

**What this is NOT:** This is not a dependent type system or a proof assistant. The type DAG is a finite, acyclic composition graph with no type-level computation. The compiler uses it for structural reasoning (compatibility, coercion, generation), not for theorem proving. This preserves P9 (totality) — type resolution always terminates because the type DAG is finite and acyclic.

### 4.2 Pure Functions (Typed Functor Protocol)

Pure transformation logic is written as `fn` declarations in `.dag` files. The compiler compiles functor bodies to every target language — no host-language stubs needed.

```
fn render_makefile(registry: ToolRegistry) -> String {
  let header = "# Generated by dag compiler\n.PHONY: all\n"
  let targets = registry.tools
    |> map(t => "{t.name}:\n\t{t.command}")
    |> join("\n\n")
  "{header}\n{targets}"
}

fn gist_filename(branch: String, base_ref: String?) -> String {
  let suffix = match base_ref {
    Some(ref) => "-vs-{ref}"
    None      => ""
  }
  "snapshot-{branch}{suffix}.md"
}

fn aggregate_results(results: List<TestResult>) -> Summary {
  let passed = results |> filter(r => r.ok) |> count()
  let failed = results |> filter(r => !r.ok) |> count()
  { total: passed + failed, passed: passed, failed: failed }
}
```

**Constrained by design.** Functors use a strict subset of ~12 constructs that represent the semantic intersection of mainstream languages (Rust, Go, Python, TypeScript) for mechanical, multi-target compilation:

| Construct | Example |
|---|---|
| `let` binding | `let x = expr` |
| String interpolation | `"hello {name}"` |
| `match` / `if-else` | `match x { A => ..., B => ... }` |
| `for` (collection transform) | `list \|> map(x => ...)` |
| Pipe operator | `x \|> f \|> g` |
| Function calls (stdlib) | `join(list, ",")`, `trim(s)` |
| Record construction | `{ field: value }` |
| Field access | `record.field` |
| Arithmetic / comparison | `a + b`, `x > 0` |
| Boolean logic | `a && b`, `!c` |

**Intentionally excluded:** general recursion, mutation, I/O, closures/higher-order functions, exceptions, concurrency, `unsafe`, casts, raw pointers, reflection. If you can't express it with these constructs, it belongs in a service operation (the imperative shell), not a functor (the functional core).

**Why constrained is better:** If functors were arbitrary, the compiler couldn't reason about them — property-based test generation breaks, multi-target emission breaks, dead code detection breaks. Constraint is the feature that makes "for free" possible.

**Standard library** (~30 functions, grows per phase):

| Category | Functions |
|---|---|
| **String** | `join`, `split`, `trim`, `contains`, `starts_with`, `ends_with`, `replace`, `to_upper`, `to_lower`, `regex_match` |
| **Collection** | `map`, `filter`, `fold`, `flat_map`, `count`, `sort_by`, `group_by`, `first`, `last`, `take`, `skip`, `any`, `all` |
| **Encoding** | `base64`, `url_encode`, `json_stringify`, `json_parse` |
| **Math** | `min`, `max`, `abs`, `round`, `floor`, `ceil` |
| **Formatting** | `pad_left`, `pad_right`, `truncate` |

### 4.3 Resources

```
resource Filesystem {
  kind: Capability          // vs Observation
  mode: ReadWrite           // Read | Write | ReadWrite | Exclusive
  acquire {}                // acquisition logic (may be no-op)
  release {}                // release logic (may be no-op)

  capability read {
    input { path: String }
    output { content: String }
    @file(READ, "{path}")
  }

  capability write {
    input { path: String, content: String }
    output { written: Bool }
    @file(WRITE, "{path}")
  }
}

resource Network {
  kind: Capability
  mode: Read
  acquire {}
  release {}
}

resource Clock {
  kind: Observation         // snapshot, no mutation
  mode: Read
  acquire { @hermetic }     // no-op — clock is always available
  release {}

  capability now {
    input {}
    output { timestamp: String }
    @hermetic                // reads system time (no network, no filesystem)
  }
  // Note: Clock.now() is deterministic in tests (seeded from run_id).
  // In production it reads the system clock — a hermetic, read-only
  // side effect. It is NOT @pure (pure nodes have no effects at all).
  // Test mocks override Clock.now() to return a fixed timestamp.
}

resource AuthContext {
  kind: Capability
  mode: Read
  expires: true             // runtime tracks expiry
  acquire {
    // acquisition is itself a journey (see credential_chain pattern)
  }
  release {}
}
```

Lifecycle kinds (from V2 P6):
- `Ephemeral` — created and destroyed within journey scope
- `Persistent` — survives across invocations
- `Borrowed` — referenced but not owned

### 4.4 Services

Declares operations and their transport binding. Inspired by Smithy. Replaces Rust service traits + `MethodMeta` + ops match arms.

```
service gcp.SecretManager {
  @endpoint("https://secretmanager.googleapis.com")
  @auth(BearerToken)                              // requires AuthContext resource

  operation AccessVersion {
    input {
      project: String
      secret: String
      version: String = "latest"
    }
    output {
      payload: Bytes
      name: String
    }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @idempotent
    @readonly
    @permissions(["secretmanager.versions.access"])
  }

  operation CreateSecret {
    input { project: String, secret_id: String }
    output { name: String }
    @rest(POST, "/v1/projects/{project}/secrets")
    @permissions(["secretmanager.secrets.create"])
  }
}

service git.Core {
  operation CurrentBranch {
    input {}
    output { branch: String }
    @shell(["git", "rev-parse", "--abbrev-ref", "HEAD"])
  }

  operation Diff {
    input { base: String, head: String = "HEAD" }
    output { diff: String }
    @shell(["git", "diff", "{base}...{head}"])
  }

  operation LsFiles {
    input {}
    output { files: List<String> }
    @shell(["git", "ls-files"])
  }
}

service github.Gist {
  @endpoint("https://api.github.com")
  @auth(BearerToken)                              // requires AuthContext resource

  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output { url: String, id: GistId }
    @rest(POST, "/gists")
    @permissions(["gist"])
    @mock_response(
      status: 201,
      body: { "html_url": "https://gist.github.com/mock/{id}", "id": "{id}" }
    )
  }
}
```

Key: services are pure declarations. Every service call in a journey compiles to a transport triplet (prepare/execute/parse). The author never sees the triplet — the compiler emits it.

**Authentication model:** Services that need auth declare `@auth(BearerToken)` (or `@auth(ApiKey)`, `@auth(Basic)`, etc.) at the service level. Journeys that call authenticated services must declare `uses auth: AuthContext` — the compiler threads the auth resource to all authenticated service calls automatically. No "magic credential argument" at call sites. Per guardrail G1, `@auth` desugars to a structural `AuthContext` resource requirement on every operation in the service.

**Base URL composition:** `@endpoint` on the service provides the base URL. Operation-level `@rest` paths are relative to the endpoint. An absolute URL on `@rest` overrides the endpoint entirely.

**Shell command safety:** `@shell` takes an argv array, not a freeform string. Each `{placeholder}` is inserted as a single argv element with no shell interpretation. For cases requiring actual shell features, `@shell(raw: "complex | piped | command")` is available as a deliberate opt-in.

**Output parsing defaults:** For `@shell` operations, `stdout`, `stderr`, and `exit_code` are always available. If the output shape is a single `String` field, the default parse is `trim(stdout)`. For `@rest` operations, the default parse is JSON decode with field names matched against the output shape. Field-level mapping uses `@json("response_field_name")` when names differ:

```
output {
  url: String @json("html_url")    // maps from GitHub's "html_url" to our "url"
  id: GistId                        // name matches, no annotation needed
}
```

**REST encoding rules (per C7):** Path parameters (`{project}`) are URL-encoded. Query parameters are URL-encoded and sorted alphabetically for deterministic requests. Body serialization uses canonical JSON (sorted keys) when used for hashing or test fixture comparison.

**Hermeticity classification (per C8):** Every service declares its IO scope, or it is inferred from transport annotations:

| Annotation | Default `io_scope` | Default `effect` |
|---|---|---|
| `@rest` | `External` | from `@readonly`/`@idempotent` or `NonIdempotent` |
| `@file` | `Hermetic` | from mode (`Read` or `ReadWrite`) |
| `@shell` | **must declare** `@hermetic` or `@external` | — |

These become IR fields on the transport execute node and drive: test categorization (hermetic tests run in CI; external tests need mocks), retry eligibility (only `@idempotent`), and resource conflict analysis.

The `@mock_response` annotation is optional. When present, the compiler uses it to auto-generate `MockSpec` boundary values for Bucket A and Bucket C tests. The mock body is parsed as a JSON AST at compile time (not a string template) — the compiler validates that it parses into the operation's output shape and inserts typed values into structural positions, preventing escaping issues with fuzzed inputs. See Appendix N for the full model-based testing design.

### 4.5 Patterns

Reusable DAG shapes with typed slots. Replaces gunbc's `UpsertBuilder`, `ContentUpsertChain`, `BranchBuilder`, etc.

```
pattern upsert<Check, Create, Resolve> {
  node check: Check -> { exists: Bool }
  node create [after check, when !check.exists]: Create
  node resolve [after check, after create]: Resolve -> { handle: String }
}

pattern content_upsert {
  input { content: String, path: String }
  uses fs: Filesystem(mode: ReadWrite)

  node read: fs.read(path: path)
  node equal: eq(a: content, b: read.content) -> { equal: Bool }
  node write [when !equal.equal]: fs.write(path: path, content: content)

  output { written: Bool = !equal.equal }
}

pattern credential_chain {
  input {
    runtime: CloudRuntime
    audience: String
    service_account: String?
    secret_name: String
    project: String
  }
  uses net: Network
  provides auth: AuthContext      // this pattern PRODUCES an auth context

  node token = match runtime {
    GitHubActions => github_oidc(audience: audience)
    Metadata     => metadata_oidc(audience: audience)
    LocalDev     => local_auth()
  }

  node access = gcp.STS.Exchange(
    subject_token: token.token,
    audience: audience
  )

  node impersonated = match service_account {
    Some(sa) => gcp.IAM.GenerateAccessToken(
      access_token: access.token,
      target_sa: sa
    )
    None => access
  }

  node secret = gcp.SecretManager.AccessVersion(
    project: project,
    secret: secret_name
  )

  output { token: AccessToken = build_token(secret.payload) }
}
```

### 4.6 Journeys

The main authoring surface. Composes services, patterns, and other journeys.

```
journey makegen {
  input { registry: ToolRegistry }
  output { written: Bool }

  content = render_makefile(registry: registry)
  result = content_upsert(content: content, path: "Makefile")

  return { written: result.written }
}

journey gist_snapshot {
  input { base_ref: String? }
  output { url: String }
  uses fs: Filesystem(mode: Read)
  uses auth: AuthContext               // threaded to github.Gist.Create via @auth

  branch = git.Core.CurrentBranch()
  files = git.Core.LsFiles()

  contents = for file in files.files {
    fs.read(path: file)
  }

  markdown = render_snapshot(files: contents)
  result = gist_upload(markdown: markdown, branch: branch.branch, base_ref: base_ref)

  return { url: result.url }
}
```

Edges are implicit — references create dependencies. The compiler resolves `branch.branch` to an edge from `git.Core.CurrentBranch`'s `branch` output port. The `uses auth: AuthContext` declaration is threaded by the compiler to any service call with `@auth` — no magic credential argument at call sites.

### 4.7 Pipelines

Stages with ordering constraints, parallel groups, and aggregation.

```
pipeline ci {
  stage codegen {
    codegen_check()
  }

  stage generate [after codegen] {
    parallel {
      bootstrap()
      pragma()
      testgen()
    }
  }

  stage build [after generate] {
    cargo_build()
  }

  stage verify [after build] {
    parallel {
      cargo_test()
      clippy()
    }
  }

  stage report [after verify] {
    aggregate(results: [verify.*])
  }
}
```

---

## 5. Module System and Discovery

### 5.1 Filesystem IS the Registry

```
project/
  dag.toml                    # project manifest
  std/                        # standard library (built-in types, resources, patterns)
    types.dag
    resources.dag
    patterns.dag
  cloud/
    concepts.dag              # SecretStore, VersionControl, etc.
    gcp/
      secret_manager.dag
      iam.dag
      sts.dag
    aws/
      secrets_manager.dag
  services/
    git.dag
    github/
      gist.dag
  tools/
    clippy.dag
    gist.dag
    deps.dag
  pipelines/
    ci.dag
    build.dag
  meta/                       # self-referential tooling
    dag_viz.dag
    makegen.dag
    testgen.dag
```

### 5.2 Project Manifest

```toml
[project]
name = "gunbc"
version = "0.1.0"

[sources]
paths = ["std/", "cloud/", "services/", "tools/", "pipelines/", "meta/"]

[codegen]
backends = ["rust"]
output = "target/generated/"

[progress]
default_mode = "inline"   # plain | inline | tui | jsonl
```

### 5.3 Discovery Rules

1. Every `.dag` file under `paths` is parsed and added to the module graph.
2. Module path = filesystem path: `cloud/gcp/secret_manager.dag` → `cloud.gcp.secret_manager`.
3. Imports are resolved against the module graph.
4. The module graph IS the workspace DAG. No separate `build_workspace_dag()`.
5. `meta/dag_viz.dag` can reference itself because it's in the same module graph.
6. The `module` declaration at the top of each file is a **consistency assertion**: the compiler errors if it doesn't match the derived path. Case normalization: paths are lowercased, `_` separates words. Windows/macOS case-insensitive filesystem quirks are handled by canonical-casing the derived path.
7. **Canonical identity = module path + local name.** The fully qualified identity of any construct is `module.path.LocalName`. A prefix like `gcp.SecretManager` in a journey is an import alias, not an identity source. Import aliases are resolved at compile time: `import cloud.gcp.secret_manager as gcp` then `gcp.SecretManager.AccessVersion(...)`. The compiler warns when a service name duplicates its module prefix (e.g., `service SecretManager` in module `secret_manager`).

### 5.4 What This Replaces

| gunbc system | Replaced by |
|---|---|
| `#[tool_target]` proc macro | filesystem discovery |
| `#[testgen_target]` proc macro | every journey has test obligations |
| `build_workspace_dag()` hardcoded list | module graph |
| `ToolRegistry::default_registry()` | project manifest |
| `inventory` crate | eliminated |
| `derive_tool_defs()` | eliminated |

---

## 6. Terminal Progress Model

### 6.1 Core Invariant: Progress is a View, Not a Constraint

**The progress display must never impose structure on the DAG.** Groups, waves, and sections are rendering decisions derived from DAG topology — they are not metadata that DAG authors must provide or that constrains how implementations are structured.

In gunb.ai, groups were manually specified (`ProgressOptions.Groups`). That worked, but it meant every DAG had to think about display. In the DSL, the progress renderer observes the DAG and infers everything:

- **SubDag boundaries → section headers** (e.g., `› Authentication`, `› Fetching Secrets`)
- **Parallel siblings → grouped counters** (e.g., `[2/5]`)
- **Topological depth → wave columns** (for TUI layout)
- **Loop expansions → scatter groups** (e.g., `read files [8/8]`)

The renderer CAN create arbitrary groupings for visualization (collapsing parallel nodes, grouping by SubDag parent), but it MUST NOT require the DAG to declare them.

**Journey vs Pattern expansion boundaries:**

- **Journey calls** create SubDag boundaries. They appear as expandable/collapsible sections in progress (e.g., `› credential` can expand to show inner nodes). Journey boundaries are meaningful to the user — they represent named workflows.
- **Small patterns** (`content_upsert`, `upsert`) are compile-time expansion and do NOT create runtime SubDag boundaries by default. They expand inline into the calling journey's node list. This keeps progress stable and author-meaningful rather than cluttered with implementation details.
- **Large patterns** that represent significant self-contained workflows (e.g., `credential_chain` — which internally has 8+ transport triplets, branching, and its own resource lifecycle) should be defined as **journeys**, not patterns, precisely because they warrant a progress boundary. The rule: if the expanded nodes are meaningful to the user as a group, it's a journey; if they're implementation detail, it's a pattern.
- A pattern MAY opt into boundary creation via `@boundary` on its definition for edge cases, but this is rare — prefer making it a journey if it needs a boundary.
- `dag expand` tooling can show pattern expansion in its output even though it doesn't create runtime boundaries.

**Pure vs boundary node distinction:** Prepare/parse nodes are pure. Execute nodes are the only transport boundaries where effects occur and the only nodes that receive capture buffers. This is a compiler invariant and simplifies test categorization: only execute nodes need mocking.

### 6.2 Subprocess Output Capture

This is a first-class concern, not an afterthought. Learned from gunb.ai's `CaptureWriter`.

**The problem**: When a transport node executes a shell command, its stdout/stderr must not leak into the progress display. Without capture, output from concurrent nodes interleaves with spinner animations, producing garbage.

**The solution**: Every transport execute node gets a per-node output buffer (like gunb.ai's `CaptureWriter`):

```
Per-node execution:
  1. Allocate CaptureBuffer for this node
  2. Redirect subprocess stdout/stderr → CaptureBuffer
  3. Execute the command
  4. On success: buffer is discarded (progress shows ✓)
  5. On failure: buffer contents shown in error box
  6. On passthrough: buffer is bypassed (see 6.3)
```

This is modeled in the IR. Transport nodes have an output capture mode:

```
type CaptureMode
  = Captured           // default: stdout/stderr → buffer, shown only on error
  | Passthrough        // interactive: stdout/stderr → terminal directly
  | Streamed           // long-running: stdout/stderr → shown live, line by line
```

### 6.3 Passthrough Mode (Interactive Commands)

Some commands need direct terminal access — OAuth flows, `gcloud auth login`, password prompts. The DAG must declare this.

```
// In a service declaration:
service gcloud.Auth {
  operation Login {
    input { update_adc: Bool = true }
    output { ok: Bool }
    @shell(["gcloud", "auth", "login", "--update-adc"])
    @interactive                         // ← marks as passthrough
  }
}

// In a journey:
journey authenticate {
  // ...
  node login [when needs_reauth]: gcloud.Auth.Login()
  // During execution:
  //   1. Progress display pauses (clears spinner line)
  //   2. gcloud auth login runs with stdin/stdout/stderr inherited
  //   3. User sees the OAuth URL, pastes code, etc.
  //   4. Command completes
  //   5. Progress display resumes
}
```

The `@interactive` annotation compiles to `CaptureMode::Passthrough` on the transport execute node. The progress renderer:
1. Clears the current progress line
2. Lets the subprocess own the terminal
3. Resumes progress display when the node completes

This is exactly what gunb.ai did for OAuth — `cmd.Stdout = os.Stdout` instead of `cmd.Stdout = captureWriter`.

### 6.4 What gunb.ai's `make login` Looks Like in the DSL

The terminal output the user showed:

```
› Authentication
   ✓ clear-cache
   ✓ detect-env
   ✓ check-account
   ✓ check-adc
   ✓ check-tokens
   ✓ configure-gcloud

   Sign in with your @gunb.ai account
   Go to the following link in your browser...
   (interactive OAuth flow — subprocess owns terminal)

› Fetching Secrets
   ✓ fetch-secrets
   ⠧ sync-remote-home
   ✓ write-bazelrc
   ✓ clear-prompt-cache
   ○ export-shell-env
   ○ Login complete (as briansrls@gunb.ai)
```

This emerges from a journey like:

```
journey login {
  output { ok: Bool }
  provides auth: AuthContext     // login PRODUCES auth for downstream callers

  // These nodes form a sequential chain → they render as a flat list
  // The journey name "login" doesn't appear; the SubDag names do.

  auth_result = authenticate()   // SubDag → becomes "› Authentication" section
  secrets = fetch_secrets()      // SubDag → becomes "› Fetching Secrets" section
                                 // uses auth: AuthContext threaded from authenticate()

  return { ok: secrets.ok }
}

journey authenticate {
  output { token: AccessToken }
  provides auth: AuthContext     // this journey PRODUCES an auth context

  clear_cache = cache.Clear()
  env = detect_environment()
  account = check_account()
  adc = check_adc()
  tokens = check_tokens(adc: adc)

  // Interactive step — @interactive on the service operation
  node login [when tokens.needs_reauth]: gcloud.Auth.Login()

  configure = configure_gcloud(tokens: tokens)
  return { token: configure.token }
}
```

**How the sections emerge**:
1. `login` journey calls `authenticate()` and `fetch_secrets()` — both are journey calls that expand to SubDags.
2. The progress renderer sees two SubDag nodes at the top level.
3. SubDag boundaries become `›` section headers.
4. Nodes inside each SubDag become the indented status lines.
5. The `@interactive` `gcloud.Auth.Login()` triggers passthrough — its output appears between the progress lines.

No manual `ProgressGroup` declarations. No `Groups: []dag.ProgressGroup{...}`. The structure IS the DAG.

### 6.5 ProgressManifest (Compiler Output)

The compiler derives a manifest from the DAG topology. The manifest is a description of what EXISTS, not a prescription of how to display:

```
type ProgressManifest {
  // Topology (what exists)
  total_nodes: Int
  topology: List<TopologyNode>        // every node with its depth and parent

  // Labels (human-readable, from DSL identifiers)
  labels: Map<NodeId, String>

  // Structural features (for renderers to use as they see fit)
  subdag_boundaries: List<SubDagBoundary>  // journey calls (patterns expand inline unless @boundary)
  parallel_groups: List<ParallelGroup>     // siblings at same depth
  scatter_points: List<NodeId>             // loop expansion points
  interactive_nodes: List<NodeId>          // @interactive transport nodes
  capture_modes: Map<NodeId, CaptureMode>  // per-node output handling

  // Pipeline-specific (present only for pipeline constructs)
  stage_groups: List<StageGroup>           // pipeline stages → collapsible sections

  // Resource context
  resources: Map<NodeId, List<ResourceUsage>>
}

type TopologyNode {
  id: NodeId
  depth: Int                              // topological depth (wave)
  parent: NodeId?                         // SubDag parent, if any
}

type SubDagBoundary {
  node_id: NodeId                         // the SubDag node in the parent
  label: String                           // journey/pattern name
  inner_nodes: List<NodeId>               // nodes inside the SubDag
}

type ParallelGroup {
  nodes: List<NodeId>                     // siblings with same dependencies
  depth: Int
}
```

**Key difference from gunb.ai**: The manifest describes topology. Renderers decide how to present it:

- The `inline` renderer might collapse SubDags into single chips: `[✓ auth] [◐ secrets]`
- The `plain` renderer might expand SubDags into sections: `› Authentication\n   ✓ clear-cache`
- The `tui` renderer might show SubDags as expandable boxes
- All three read the SAME manifest — they just make different rendering choices

### 6.6 JSONL Event Protocol (from the-gunbai `progress-contract.md`)

The `jsonl` rendering mode emits a formal event stream that enables replay, remote rendering, and machine consumption. The protocol is defined here because it is **normative** — all renderers (plain, inline, tui) are pure functions over the same event stream.

**Envelope:** Every event has a versioned envelope:

```
{ schema: "gunbai.progress.v1", seq: Int, ts_ms: Int, run_id: String?, event_type: String, data: {...} }
```

- `seq` is monotonic per run (required for deterministic replay).
- `ts_ms` is informational; ordering uses `seq` only.
- Renderers must ignore out-of-order events (`seq <= last_seq`).

**Event types:**

| Event | Data | Purpose |
|---|---|---|
| `run_started` | `{ graph_id, nodes: Int, edges: Int }` | Begin execution |
| `graph_snapshot` | `{ graph: GraphSnapshot }` | Initial topology (nodes + edges + metadata) |
| `graph_patch` | `{ add_nodes, add_edges, remove_nodes?, remove_edges? }` | Dynamic expansion (scatter/loop fan-out) |
| `node_state` | `{ node_id, state, message?, error? }` | State transition (queued → running → succeeded/failed/skipped/cancelled) |
| `node_progress` | `{ node_id, fraction?, message?, detail? }` | Incremental progress within a node |
| `node_output` | `{ node_id, stream, chunk, truncated? }` | Captured stdout/stderr/log |
| `run_completed` | `{ success: Bool }` | Execution finished |

**Graph patches** handle dynamic expansion: when a `for` loop's collection size is known at runtime, the executor emits a `graph_patch` adding instance nodes. Renderers update their layout incrementally. This is the mechanism behind scatter group progress like `read 8/8`.

**Renderer contract:**

- `apply_event(state, event) → state` — pure, deterministic. Must handle `graph_snapshot` before any `node_state`.
- `render_frame(state, now_ms) → Frame` — pure, deterministic. Tick-driven renderers call this on a timer.

**Output capture policy:** Per-node, determined by `CaptureMode` (§6.2). `Captured` → bounded buffer, shown only on error. `Passthrough` → no buffer, terminal inherits stdout/stderr directly. `Streamed` → bounded buffer + live `node_output` events emitted as chunks arrive. In SubDag contexts, inner nodes inherit the parent's capture mode unless overridden. Runtime enforces bounded buffers and sets `truncated: true` when data is dropped.

### 6.7 Rendering Modes

| Mode | Description | When |
|------|------------|------|
| `plain` | Sections + status lines (gunb.ai style) | CI, non-TTY |
| `inline` | Compact bar + chips (the-gunbai style) | Default TTY |
| `tui` | Full DAG with edge pulses (the-gunbai style) | Explicit opt-in |
| `jsonl` | Structured event stream | Machine consumption |

**Plain** — gunb.ai style with sections:
```
› Authentication
   ✓ clear-cache (1ms)
   ✓ detect-env (2ms)
   ✓ check-account (50ms)
   ✓ check-adc (3ms)
   ✓ check-tokens (5ms)
   ✓ configure-gcloud (10ms)

› Fetching Secrets
   ✓ fetch-secrets (1.2s)
   ⠧ sync-remote-home
   ○ write-bazelrc
   ○ clear-prompt-cache
   ○ export-shell-env
```

**Inline** — the-gunbai compact style:
```
login ─ 8/12 ━━━━━━━━━━━━░░░░ 67% [✓ auth 6/6] [◐ secrets 2/6]
```

**TUI** — full DAG visualization:
```
╭─ auth ─────────────────────╮  ╭─ secrets ─────────────╮
│ ✓ clear-cache     ✓ detect │──│ ✓ fetch     ⠧ sync   │
│ ✓ check-account   ✓ adc   │  │ ○ bazelrc   ○ cache  │
│ ✓ tokens   ✓ configure    │  │ ○ export              │
╰────────────────────────────╯  ╰───────────────────────╯
```

### 6.8 Failure Output

When a node fails, the CaptureBuffer contents are shown in an error box (gunb.ai pattern):

```
› Fetching Secrets
   ✓ fetch-secrets (1.2s)
   ✖ sync-remote-home (3.4s)

   ┌─ Error: sync-remote-home ────────────────────────┐
   │ gsutil rsync returned exit code 1                 │
   │                                                   │
   │ stderr:                                           │
   │   CommandException: No URLs matched: gs://...     │
   │   CommandException: 1 file/object could not be    │
   │   transferred.                                    │
   └───────────────────────────────────────────────────┘

   ○ write-bazelrc (skipped: dependency failed)
   ○ clear-prompt-cache (skipped)
   ○ export-shell-env (skipped)
```

The captured stderr appears ONLY on failure. On success, it's silently discarded. This prevents the double-printing problem where subprocess output would interleave with progress indicators.

### 6.9 Visual Design Specification

The visual design is inherited from gunb.ai and already ported to gunbc's symbol system. The exact values (ANSI 256 color codes, braille spinner frames at 80ms tick, Unicode/ASCII/Emoji icon tiers, box-drawing characters, section marker `›`, completion animals, prompt status icons) are documented in the terminal crate spec. gunbc's `symbols.rs`, `box_draw.rs`, `frame_write.rs`, and `render_ir.rs` (~2,271 lines total, 95% standalone) form the harvestable terminal crate for the new repo. The ratatui-based TUI from the-gunbai is harvested separately behind a `tui` cargo feature flag.

### 6.10 How Progress Compiles

The progress model touches three compiler phases:

1. **Lower**: Transport nodes get `CaptureMode` based on `@interactive` annotations.
2. **Derive**: `ProgressManifest` computed from lowered DAG topology (depths, SubDag boundaries, parallel groups, scatter points).
3. **Emit**: Codegen backend emits:
   - `CaptureBuffer` allocation per transport node
   - Progress observer trait implementation
   - Frame builder that reads the manifest
   - Renderer selection (plain/inline/tui/jsonl)

The runtime needs only:
- The manifest (static, from compiler)
- Node state transitions (Pending → Running → Succeeded/Failed/Skipped)
- CaptureBuffer contents (for error display)
- Interactive node detection (for passthrough pause/resume)

---

## 7. Resource Model

### 7.1 Declaration

```
journey write_config {
  uses fs: Filesystem(mode: Write)
  uses clock: Clock

  timestamp = clock.now()
  content = render_config(timestamp: timestamp)
  fs.write(path: "config.toml", content: content)
}
```

### 7.2 What the Compiler Does

1. **Inserts acquisition nodes** at DAG boundaries (like gunbc's `FsEnv`, `ClockEnv`).
2. **Threads resources** through edges to consuming nodes (like gunbc's `res:*` ports).
3. **Detects conflicts** — parallel Write+Write on same resource with overlapping keys = compile error (see 7.4).
4. **Generates mock specs** — DryRun substitutes resources with mocks.
5. **Derives test obligations** — Bucket D (Resource Hygiene) from testgen.
6. **Tracks lifecycle** — acquire before first use, release after last use. "Last use" is the topologically last node that holds a reference to the resource handle (computed from the edge graph, not declaration order). In conditional DAGs, release runs when the last reachable user completes or is skipped. Release failure is a node failure — it propagates downstream like any other failure (§8.1).

### 7.3 Conflict Detection (Keyed Resources)

Treating `Filesystem` as a single shared resource is too coarse — it would ban harmless parallelism (writing different files). Conflict detection is keyed:

- `Filesystem` conflicts are checked per **path key**. Two `fs.write(path: "a.txt")` and `fs.write(path: "b.txt")` are fine in parallel. Two writes to the same path are a conflict.
- If the path is unknown at compile time (e.g., loop variable), the compiler **conservatively treats as conflicting** unless the loop guarantees unique keys.
- Network resources are non-conflicting by default (HTTP is stateless).
- Custom resources can declare `@exclusive_key(field)` to specify the conflict key.

### 7.4 Resource Mode Inference and Alias Rules

When a journey calls a pattern that declares resources, the caller must declare compatible modes:

- If `content_upsert` declares `uses fs: Filesystem(mode: ReadWrite)`, the calling journey must declare `uses fs: Filesystem(mode: ReadWrite)` (or a superset).
- Mode mismatch is a compile error. The compiler does NOT automatically escalate `Read` to `ReadWrite`.
- The `provides` keyword (used in `credential_chain`) indicates a pattern that *produces* a resource context rather than *consuming* one.

**Alias naming rules** when resources are inferred from callees:

- If the caller does not declare `uses`, inference uses the callee's alias name. `content_upsert` declares `uses fs: Filesystem` → the caller gets `fs` as the inferred alias.
- If two callees use different aliases for the same resource type, the compiler requires the caller to declare `uses` explicitly to resolve the ambiguity.
- If the caller declares `uses fs: Filesystem(mode: Read)` but a callee needs `ReadWrite`, it is a compile error (not automatic escalation). The fix is for the caller to declare the stronger mode.

### 7.5 Resource Handles Are Opaque (Non-Forgeable)

Resource capability handles (`FilesystemHandle`, `NetworkHandle`, `ToolHandle`, `Credential`) are **opaque tokens** that cannot be constructed by user code. This is a normative invariant — it makes the resource model closed.

**Rules:**

1. Only `acquire` nodes (compiler-emitted from resource lifecycle declarations) and environment nodes can mint capability handles. No user-authored node may construct a handle.
2. Handles are opaque at the IR level. The `Value` representation includes a capability marker that is validated on use. Constructing a `Value` that looks like a handle from raw data is a runtime error.
3. In the DSL, handles never appear as user-visible types. The author writes `uses fs: Filesystem(mode: Write)` and calls `fs.write(...)`. The compiler threads the handle through edges. The author never holds, inspects, or constructs the handle.
4. Test mocks for resource handles are provided by the compiler (from `resource` declarations), not hand-constructed. The mock handle satisfies the capability marker check but routes to a test backend.

**Why this matters:** If handles were forgeable, a node could bypass resource acquisition (and its lifecycle, conflict detection, and policy enforcement) by constructing a fake handle from a `Value`. This would undermine I3 (Observable I/O), I6 (No Escape Hatches), and the entire resource conflict model (§7.3).

**gunbc implementation status:** Partially mitigated via a capability marker pattern (`CAPABILITY_MARKER` in `core/ir/src/resource/mod.rs`) and per-process secrets (`PROCESS_SECRET` in `handle.rs`). The DSL compiler should enforce this structurally — handles are never in the user-visible type namespace, so forgery is impossible at the language level.

### 7.6 What This Replaces

```rust
// gunbc today: manual environment nodes + resource wiring
let fs_env = builder.add_root_node(Node::opaque(
    "fs_env", vec![], vec![port("FilesystemHandle", "FilesystemHandle")],
    FsEnv::new(Scope::Write),
));
// ... later, for EVERY node that needs filesystem:
builder.add_edge(fs_env.out("FilesystemHandle"), node.in_port("res:file:Makefile"));
```

```
// DSL: declared once, compiler handles the rest
journey makegen {
  uses fs: Filesystem(mode: Write)
  // fs.write(...) automatically threads the resource
}
```

---

## 8. Error Model and Execution Semantics

### 8.1 Failure Propagation

Failure is out-of-band — nodes either succeed or fail. There is no `Result<T, E>` type in the DAG; failure is a runtime state transition, not a value.

**Rules:**

1. When a node fails, all downstream nodes (reachable via edges) are **skipped**. Skipped nodes receive no inputs and produce no outputs.
2. Independent nodes (no dependency path to the failed node) **may still complete** — failure does not cancel the entire DAG.
3. A `when` guard cannot observe failure state — guards operate on output values, not runtime status.
4. Skipped nodes appear in progress as `○ node-name (skipped: dependency failed)`.
5. The failed node's captured stderr is shown in an error box. Skipped nodes show no output.

### 8.2 Recovery Patterns

Recovery is structural, not exceptional:

- **Retry**: `@retry(max: 3, backoff: exponential(1000))` on a service operation. The compiler emits retry logic in the transport execute node. Retries are transparent to the DAG.
- **Transaction**: `transaction { begin: ..., body: ..., commit: ..., rollback: ... }` pattern — rollback arm runs when body fails.
- **Fallback**: `match` arms on explicit output values — alternative paths based on what a node *returned*, not whether it *failed*. A service that returns `{ ok: Bool, error_code: Int? }` can be matched on `ok` to choose a fallback path. This is value-level branching, not exception catching.

There is no `try/catch`. Runtime failure (node crashed, transport error) is out-of-band — it skips downstream nodes (§8.1). It cannot be "caught." If you need to handle error *types*, the service must model them as output fields, and you branch on those values with `match`. The distinction: **failure is a runtime state transition (invisible to the DAG); error codes are output values (visible and matchable).**

### 8.3 Control Dependencies (`after`)

Dataflow edges arise from value references. When ordering is needed without dataflow, use `after`:

```
node check: tool.Exists()
node install [after check, when !check.exists]: tool.Install()
node resolve [after check, after install]: tool.Version()
```

**Rules:**

- `after x` creates a control edge from `x` to the current node. The current node will not execute until `x` completes (or is skipped).
- `after x` when `x` is skipped (due to a failed or skipped upstream): the current node still runs. The `after` edge only ensures ordering, not that `x` succeeded.
- Guards (`when`) do NOT imply ordering. `node x [when cond]` does not create an edge from the node that produces `cond` unless `cond` is also referenced as a value. In practice, the reference in `when !check.exists` does create a data edge from `check`, but `after` makes the intent explicit and handles cases where the guard references a different node than the ordering source.
- `after` is the **only** non-data dependency mechanism. There is no implicit ordering from declaration position.

**Rule of thumb for `after` vs value edges:** If correctness depends on *what* the upstream node produced (its output value or whether it succeeded), use a value edge (reference the output). Use `after` only when you need ordering for side-effect sequencing (e.g., "install before run," "write before read") where the downstream node doesn't consume the upstream's output. A node with *only* `after` deps and no value deps is a lint warning — it usually means either the ordering is unnecessary or a value dependency was forgotten.

**`when` vs `match` — two different things:**

- **`when` is a guard** (on a node). It skips the node if the condition is false. The node either runs or is skipped — there is no "else" branch. Lowering: the node gets a `GuardedPort` that suppresses execution. Downstream nodes see `Skipped` if the guard is false.
- **`match` is a conditional expression** (produces a value). It selects between branches and always produces exactly one output. Lowering: the compiler emits a `BranchBuilder` with one arm per case and a merge node that collects the result.

`when` never has an `else`. If you need an alternative path, use `match`. This avoids the ambiguity of "guard with fallback" which has unclear lowering semantics.

### 8.4 Scheduling Semantics

Executors must produce results equivalent to some topological order consistent with the edge dependencies (both data edges and `after` control edges). Concretely:

- Two nodes with no dependency path between them **may** run concurrently. The executor is free to parallelize within a wave.
- Side effects occur only at transport execute nodes. Pure `fn` nodes and prepare/parse nodes have no observable side effects.
- Execution is **deterministic** in outcome: given the same inputs and transport responses, the DAG produces the same outputs regardless of scheduling order.
- Scheduling strategy is executor-specific: sequential in simple runners, goroutine pool / `asyncio.gather` / rayon in parallel runners.

### 8.5 Bounded Repetition (upholding P9)

All repetition constructs must be explicitly bounded to preserve totality:

- `for` is bounded by finite collection.
- `@retry(max: N)` must have a compile-time constant `max`. The compiler rejects `@retry` without `max`.
- `poll(timeout: Duration, interval: Duration)` must have finite timeout.
- `while` is **not available** as general syntax. Any "repeat until condition" logic must be expressed as `poll` with a timeout or as a bounded `for` with early exit.

The compiler verifies at compile time that no unbounded repetition exists in the expanded DAG.

### 8.6 Loop Semantics

`for` loops iterate over a collection and return results in **input order**, regardless of whether iteration is parallel:

```
contents = for file in files.files {
  fs.read(path: file)
}
// contents: List<FileContent>, same order as files.files
```

- Execution of loop iterations may be parallel (the executor decides).
- The result `List<T>` preserves input index order — never completion order.
- If any iteration fails, the loop fails. Completed iterations' results are discarded. Remaining iterations may or may not execute (executor-dependent).

### 8.7 NodeId Stability

NodeIds must be stable across compilations of the same source, enabling progress replay and deterministic testing.

**Scheme:** hierarchical path from module + journey/pattern + local name.

```
tools.gist/gist_snapshot/branch          — top-level node
tools.gist/gist_snapshot/loop:contents   — loop expansion point
tools.gist/gist_snapshot/loop:contents[3] — loop instance (runtime, index-based)
tools.gist/gist_snapshot/cred_chain/token — node inside expanded SubDag
```

**Rules:**
- Module path derived from filesystem (`tools/gist.dag` → `tools.gist`)
- Local names are the binding name in the journey (`branch = git.Core.CurrentBranch()` → `branch`)
- Loop instances use deterministic index: `loop:<name>[<index>]` based on input order
- SubDag expansion preserves parent path: `journey/subdag_name/inner_name`
- Collisions are compile errors (two nodes in the same journey cannot have the same binding name)

### 8.8 Fan-In/Fan-Out Semantics

When an output port connects to multiple downstream inputs (**fan-out**), each downstream node receives the same value — broadcast semantics.

When multiple upstream outputs connect to a single downstream input (**fan-in**), behavior depends on the input port's cardinality:

| Scenario | Semantics |
|---|---|
| Fan-out: `One` output → multiple inputs | Broadcast — each input gets the same value |
| Fan-in: multiple `One` outputs → `One` input | **Compile error** — ambiguous which value |
| Fan-in: multiple `One` outputs → `List<T>` input | Collect into list (canonical order) |
| Fan-in: `List<T>` + `One` outputs → `List<T>` | Concatenate (canonical order) |

**Canonical edge ordering** (required for determinism): Collection order must be deterministic across compilations. Sort key: `(from_node_id, from_port_name, edge_index)`. This ensures the same DAG always produces the same collection order, matching gunbc's `canonical_edge_order()`.

**Map merge:** When fan-in produces a `Map<K, V>`, the default merge strategy is `ErrorOnDuplicate` — the compiler rejects fan-in that could produce duplicate keys. `LastWriteWins` and `CombineValues` are available as explicit annotations if the author accepts the semantics.

**Invariant:** Any implicit aggregation must be justified by a derivable combining law; otherwise fan-in is a compile error.

### 8.9 Workflow Signatures

Every journey has a **workflow signature** — a declared contract of typed inputs and outputs — that is verified against the inferred signature (computed from unconnected ports).

```
journey gist_snapshot {
  input { extensions: List<String> }     // declared input
  output { url: String }                 // declared output
  ...
}
```

**Invariant:** `DeclaredSignature == InferredSignature`. This catches:

- **Silent interface drift**: Forgot to wire an edge? Now it's a new public input/output.
- **Wiring bugs**: Intended `A → B` but forgot the edge — validation fails instead of silently exposing ports.
- **Cardinality drift**: Changed `T?` to `T`? Signature check catches it.
- **Dead work**: Pure nodes not contributing to any output can be flagged.

The compiler infers the signature from unconnected ports and compares it to the declared `input`/`output` blocks. Mismatches are compile errors. Tool ports (framework-provided capabilities like `tool:<id>`) are excluded from the user-facing signature.

### 8.10 Recursion

The journey/pattern call graph must be acyclic. The compiler rejects recursive calls at compile time. If a journey `A` calls `B` which calls `A`, this is a compile error. There is no fixpoint construct — unbounded recursion violates P9 (totality).

---

## 9. Compiler Pipeline

```
.dag files (filesystem)
   │
   ▼
[Discover] ──→ Module graph (all .dag files in project)
   │
   ▼
[Parse] ─────→ AST per file (concrete syntax tree)
   │
   ▼
[Resolve] ───→ Resolved AST (imports linked, names resolved against module graph)
   │
   ▼
[TypeCheck] ──→ Typed AST (expressions typed, resource requirements validated)
   │
   ▼
[PatternExpand] → PatternIR (patterns → sub-DAG templates,
   │                          resources → acquire/release nodes)
   ▼
[Lower] ─────→ GraphIR (Node / Dag / Port / Edge)
   │             - service calls → transport triplets
   │             - implicit edges → explicit Edge values
   │             - resource uses → acquire/release lifecycle nodes
   │             - for → LoopBuilder nodes
   │             - match → BranchBuilder nodes
   │             - when → guarded ports
   ▼
[Validate] ──→ Validated GraphIR (SPEC.md invariants + resource conflicts)
   │
   ▼
[Derive] ────→ ProgressManifest + TestObligations + ToolMetadata
   │
   ▼
[Emit] ──────→ Per-backend codegen
                 - Type definitions
                 - Pure function implementations (compiled from `fn` functor bodies)
                 - Transport wiring (HTTP client, shell exec, file I/O)
                 - Test harness (4-bucket testgen)
                 - CLI entrypoint (args from DAG entrypoints)
                 - Progress renderer (manifest-driven)
                 - Makefile / CI YAML
```

---

## 10. Multi-Target Emission

### 10.1 IR Semantics Are Minimal

| IR Concept | Rust | Go | Python | MIPS |
|---|---|---|---|---|
| Node | `fn` | `func` | `def` | `jal label` |
| Edge | variable | variable | variable | register |
| Transport | `reqwest`/`Command` | `net.http`/`exec` | `requests`/`subprocess` | `syscall` |
| Guard | `if` | `if` | `if` | `beq` |
| SubDag | `fn` (inlined) | `func` (inlined) | `def` (inlined) | `jal` (nested) |
| Loop | `for .. in` | `for .. range` | `for .. in` | loop/`beq` |
| Topo schedule | sequential | goroutine pool | `asyncio.gather` | instruction order |

### 10.2 Backend Interface

Each codegen backend implements:

```
trait CodegenBackend {
  fn emit_type(ty: &TypeDef) -> String
  fn emit_fn(func: &FnDef) -> String             // compile functor body to target language
  fn emit_transport(spec: &TransportSpec) -> String
  fn emit_journey(journey: &JourneyDef) -> String // orchestration + wiring
  fn emit_test(obligation: &TestObligation) -> String
  fn emit_cli(entrypoints: &[Port]) -> String
  fn emit_progress(manifest: &ProgressManifest) -> String
}
```

### 10.3 Emission Targets (Unifying gunbc's 13 Rendering Islands)

gunbc's `docs/design/unified-emission.md` catalogued 13 separate rendering systems with 4 different IR/trait patterns. The DSL Emit phase must unify them under one pipeline: `Derive → IR → Renderer → Output`. The table below defines every emission target, its IR, and its renderer trait.

| # | Target | IR produced by Derive | Renderer | Output |
|---|---|---|---|---|
| 1 | **Type definitions** | `TypeDef` (structs, enums, aliases) | `CodegenBackend::emit_type` | Rust structs, Go types, Python dataclasses |
| 2 | **Pure functions** | `FnDef` (functor AST) | `CodegenBackend::emit_fn` | Compiled function bodies per target language |
| 3 | **Transport wiring** | `TransportSpec` (triplet templates) | `CodegenBackend::emit_transport` | HTTP client setup, shell exec, file I/O |
| 4 | **Test harness** | `TestFile` / `TestFn` / `Stmt` / `Assert` | `TestRenderer` trait (per-language) | Rust `#[test]` functions, Python pytest, Go `TestX` |
| 5 | **Mock fixtures** | `MockManifest` (from `@mock_response` + refinement types) | `TestRenderer` (same trait) | MockSpec construction, fixture files |
| 6 | **CLI entrypoints** | `CliSpec` (args from DAG entrypoint ports) | `CodegenBackend::emit_cli` | Clap/argparse/cobra wiring |
| 7 | **Progress renderer** | `ProgressManifest` (topology, boundaries, groups) | `ProgressRenderer` trait | Frame building, JSONL emission |
| 8 | **Makefile** | `MakefileIR` (targets, deps, rules) | `MakefileRenderer` | Makefile + .gitignore |
| 9 | **CI YAML** | `SharedStep[]` (checkout, run, dag-step) | `CiRenderer` trait (per-provider) | GitHub Actions YAML, GitLab CI YAML |
| 10 | **Terminal layout** | `DagLayout` (wave columns, edge routes) | `TerminalRenderer` (standard/compact) | ANSI terminal output |
| 11 | **JSONL events** | Event envelope (§6.6 protocol) | `JsonlRenderer` | Structured event stream |
| 12 | **Content hash manifest** | `ManifestEntry` (input hash, file count) | — (serialized directly) | `.manifest.json` for freshness |
| 13 | **Obligation report** | `ObligationSummary` (discharged/testable counts) | `CodegenBackend::emit_test` header | Comment block in generated test files |

**Key principle (from gunbc testgen, the "gold standard"):** IR before rendering. The Derive phase builds language-neutral IRs. The Emit phase maps IRs to target-specific output through renderer traits. Adding a new language backend requires implementing `CodegenBackend` + `TestRenderer` — zero changes to Derive.

### 10.4 The 100% / 0% Split (Functor Protocol)

With the Typed Functor Protocol (§4.2), the developer writes everything in `.dag` files — including pure transformation logic via `fn` declarations. The compiler generates all host-language code across all 13 emission targets above.

The developer writes nothing in the host language for the common case. A `@custom` transport annotation is available for operations that don't fit `@rest` / `@shell` — the developer implements only the execute step, preserving all structural guarantees.

---

## 11. What to Harvest

### From gunb.ai
- **CaptureWriter** pattern: per-node output buffer, subprocess stdout/stderr captured not printed, shown only on failure in error boxes. Thread-safe (`sync.Mutex` + `bytes.Buffer`). This is THE solution to double-printing.
- **Passthrough mode**: Interactive commands (`gcloud auth login`, OAuth) bypass capture, inherit terminal stdin/stdout/stderr directly. Progress display pauses during passthrough.
- **Section rendering**: `›` section headers from SubDag boundaries (in gunb.ai these were manually grouped via `ProgressOptions.Groups` — we make them emergent from DAG structure)
- **Error boxes**: Failed node output displayed in bordered box with captured stderr. Successful nodes silently discard captured output.
- **Preamble box**: Tool header with name, description, args displayed before execution
- **Emoji prompt icons**: Status indicators exported as shell variables (`GUNB_AUTH_ICON`, etc.) — shows system state at a glance
- Lease/heartbeat execution model (concept for distributed execution)

### From the-gunbai
- TUI progress system (ratatui + crossterm): edge pulses, wave layout, scatter groups
- Progress state machine (`ProgressState`, `NodeProgressState`, `ProgressCounts`)
- Spinner system (deterministic tick-driven, braille frames)
- JSONL event streaming (schema: `gunbai.progress.v1`)
- Inline renderer (compact progress bar + box-drawing DAG)

### From gunbc
- `Node<T>`, `Dag<T>`, `Port`, `Edge` core types (proven correct)
- Pattern builders: `UpsertBuilder`, `BranchBuilder`, `LoopBuilder`, `ContentUpsertChain`
- Execution engine: lowering, topo sort, DryRun
- Testgen obligation model (4 buckets, anti-tautology rule, `ProofObligation`, `DischargeStatus`)
- Transport executor (REST, Shell, File, TCP)
- Resource conflict detection algorithm
- Frame-based display (pure `build_frame()`, `FrameRenderer<M>` trait)
- `OutputMedium` trait hierarchy (AnsiText, PlainText, HtmlText)
- `SemanticColor` / `SymbolId` tier-based symbol resolution

### Redesign
- Decouple `TransportRequest`/`TransportResponse` from `Value` enum
- Eliminate `ValueExpr` (codegen works from IR + types)
- Move transport out of `core/ir/src/transport/` into transport crate
- Replace all registration macros with filesystem discovery
- Unify the-gunbai's TUI + gunbc's FrameRenderer into manifest-driven system

### Known Risk: Transport Executor Is a Testing Blind Spot

The transport executor (`lib/transport/src/executor.rs`) sits **outside** the DAG system — testgen generates tests for Prepare and Parse nodes (pure functions) and DryRun-intercepts Execute nodes with mocks, but nothing tests what happens inside `execute_transport()` itself.

| Function | Lines | Test coverage |
|---|---|---|
| `execute_rest` | ~45 | 0 (via tool integration tests only) |
| `execute_http` | ~55 | 0 (via REST wrapper) |
| `execute_file` | ~180 | Good (30+ unit tests) |
| `execute_tcp` | ~35 | **Zero** |
| `execute_shell` | ~40 | 2 (basic happy-path) |

**Concrete bug this missed:** A swapped-timeout bug in `execute_tcp` — `connect_timeout_ms` was used for `set_read_timeout` and `read_timeout_ms` for `set_write_timeout` — survived because TCP has zero test coverage and the executor sits outside where testgen operates.

**Why this matters for the DSL:** The DSL compiles service declarations to transport triplets (Prepare → Execute → Parse). Prepare and Parse are pure and testable. But Execute is the actual I/O boundary, and its internals remain opaque. "DSL-ifying" workflows while keeping the riskiest wiring outside the model is incomplete victory.

**Migration plan (from `TODO/TODO_transport_dag_migration.md`):**

1. **Phase 1 (immediate):** Fill the testing gap — unit tests for all 5 executor functions, especially TCP.
2. **Phase 2:** Typed port decomposition — decompose opaque `TransportRequest` into named scalar ports so field routing is compiler-verified (prevents the swapped-timeout class of bugs).
3. **Phase 3:** Transport behavioral specs — declarative `TransportBehavior` specs that integrate with testgen to generate behavioral tests.
4. **Phase 4 (if needed):** Full sub-DAG modeling of transport internals.

The DSL does not need to wait for this migration to be useful — the triplet model is correct at the DAG level. But the transport executor should be brought under testgen coverage as a parallel workstream.

---

## 12. Phasing

### Phase 1: Language Core + Module Discovery + Progress Manifest

**Target**: Express `makegen` end-to-end.

```
tools/makegen.dag → discover → parse → typecheck → lower → validate
  → derive ProgressManifest → emit Rust → execute with inline progress
```

Proves: parser, types, patterns (content_upsert), discovery, progress manifest, Rust backend.

### Phase 2: Services + Resources + Cloud Modeling

**Target**: Express `acquire_gcp_secret`.

- Provider-qualified service calls (`gcp.SecretManager.AccessVersion`)
- `@rest` → transport generation
- `resource AuthContext` with lifecycle
- `match` for runtime branching, `when` for guards

### Phase 3: Composition + TUI Progress

**Target**: Express `gist_snapshot`.

- `for` loops, journey composition, SubDag expansion
- TUI progress renderer driven by ProgressManifest
- Static DAG visualization (before execution)

### Phase 4: Pipelines + Second Codegen Backend

**Target**: Express CI pipeline. Add Go or Python backend.

- Pipeline stages, parallel execution, aggregation
- Same `.dag` files → different language output

---

# Appendix A: Content Upsert (Makegen)

The simplest complete graph in gunbc. The canonical "hello world" for the DSL.

## A.1 Today: Rust (gunbc)

### Graph builder (`gunbc-dag/src/makegen/graph.rs` — 137 lines)

```rust
pub fn build_makegen_graph() -> Dag<MakegenGraphOp> {
    let mut builder: DagBuilder<MakegenGraphOp> = DagBuilder::new();

    // Root: filesystem handle
    let fs_env = builder
        .add_root_node(Node::opaque(
            "fs_env", vec![],
            vec![port("FilesystemHandle", "FilesystemHandle")],
            MakegenGraphOp::FsEnv(FsEnv::new(Scope::Write)),
        )).expect("fs_env");

    // Root: load tool registry
    let load_registry = builder
        .add_root_node(Node::opaque(
            "load_registry", vec![],
            vec![
                port("tool_count", "Int"),
                port("tool_names", "NonEmptyStringList"),
                port("registry", "Json"),
            ],
            MakegenGraphOp::Domain(MakegenOp::LoadRegistry),
        )).expect("load_registry");

    // Pure: render Makefile content
    let render_makefile = builder
        .add_node_after(Node::opaque(
            "render_makefile",
            vec![port("registry", "Json")],
            vec![port("makefile_content", "String")],
            MakegenGraphOp::Domain(MakegenOp::RenderMakefile),
        ), &load_registry).expect("render_makefile");

    builder.add_edge(
        load_registry.out("registry"),
        render_makefile.in_port("registry"),
    ).expect("load_registry.registry -> render_makefile.registry");

    // Content upsert chain (5 nodes, 8 edges — added by helper)
    add_content_upsert_chain(
        &mut builder,
        "makegen",
        &render_makefile,
        "makefile_content",
        &fs_env,
        "Makefile",
    );

    builder.build().expect("makegen_graph")
}
```

Plus: operation enum (25 lines), `Executable` impl (40 lines), operation implementations (80+ lines), testgen registration (15 lines), tool registration (15 lines).

**Total: ~200+ lines across 3 files.**

### Resulting IR (8 nodes, 10 edges)

```
Dag {
  nodes: [
    Node { id: "fs_env",                    body: Opaque(FsEnv) }
    Node { id: "load_registry",             body: Opaque(LoadRegistry) }
    Node { id: "render_makefile",           body: Opaque(RenderMakefile) }
    Node { id: "prepare_read_makegen",      body: Opaque(PrepareFileRead) }
    Node { id: "execute_read_makegen",      body: Opaque(Transport::Execute) }
    Node { id: "compare_makegen_content",   body: Opaque(Blob::CompareContent) }
    Node { id: "prepare_write_makegen",     body: Opaque(PrepareFileWrite) }
    Node { id: "execute_makegen_transport", body: Opaque(Transport::Execute) }
  ]
  edges: [
    load_registry.registry         → render_makefile.registry
    render_makefile.makefile_content → compare_makegen_content.expected_content
    render_makefile.makefile_content → prepare_write_makegen.content
    prepare_read_makegen.request    → execute_read_makegen.request
    prepare_read_makegen.skip       → execute_read_makegen.skip
    execute_read_makegen.response   → compare_makegen_content.response
    compare_makegen_content.skip    → execute_makegen_transport.skip
    compare_makegen_content.skip_reason → execute_makegen_transport.skip_reason
    prepare_write_makegen.request   → execute_makegen_transport.request
    fs_env.FilesystemHandle         → execute_read_makegen.res:file:Makefile
    fs_env.FilesystemHandle         → execute_makegen_transport.res:file:Makefile
  ]
}
```

### Generated tests (from testgen — excerpt)

```rust
#[test]
fn test_dryrun_completion() {
    let dag = build_makegen_graph();
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("DryRun should complete");
    assert!(!log.entries.is_empty());
}

#[test]
fn test_transport_interception() {
    let dag = build_makegen_graph();
    let result = assert_boundary_mockable(&dag, mock_spec().to_boundary_mocks());
    assert!(result.is_ok());
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_read_makegen"));
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_makegen_transport"));
}
```

## A.2 DSL

### `tools/makegen.dag` (5 lines of authoring)

```
module tools.makegen

import std.patterns { content_upsert }

journey makegen {
  input { registry: ToolRegistry }
  output { written: Bool }

  content = render_makefile(registry: registry)
  result = content_upsert(content: content, path: "Makefile")

  return { written: result.written }
}
```

## A.3 Compiler Output

### Resulting IR (identical structure — 8 nodes, 10 edges)

The compiler produces the same IR as the hand-wired version. The key differences:

1. `content_upsert` pattern expansion creates the 5-node chain automatically.
2. `render_makefile(registry: registry)` becomes a service call (pure, no transport).
3. `uses fs: Filesystem` is inferred from `content_upsert`'s declaration — the `fs_env` node is inserted automatically.
4. Edges are derived from references: `content_upsert(content: content, ...)` creates the edge from `render_makefile.makefile_content` to the upsert chain.

### Generated test obligations

```
Bucket A (Execution Semantics):
  - DryRunCompletion: full workflow
  - TransportInterceptable: execute_read_makegen
  - TransportInterceptable: execute_makegen_transport

Bucket B (Contract Obligations):
  - NodeContractCompliance: render_makefile

Bucket C (Scenario Coverage):
  - AllTransportsSucceed
  - SingleTransportFailure: execute_read_makegen
  - SingleTransportFailure: execute_makegen_transport
  - GuardBranchCoverage: execute_makegen_transport (skip guard)

Bucket D (Resource Hygiene):
  - TransportResourceDeclared: execute_read_makegen
  - TransportResourceDeclared: execute_makegen_transport
  - ResourceInputConnected: execute_read_makegen.res:file:Makefile
  - ResourceInputConnected: execute_makegen_transport.res:file:Makefile
```

### ProgressManifest

```
ProgressManifest {
  total_nodes: 8
  topology: [
    { id: "fs_env", depth: 0, parent: None }
    { id: "load_registry", depth: 0, parent: None }
    { id: "render_makefile", depth: 1, parent: None }
    { id: "prepare_read_makegen", depth: 1, parent: None }
    { id: "execute_read_makegen", depth: 2, parent: None }
    { id: "compare_makegen_content", depth: 3, parent: None }
    { id: "prepare_write_makegen", depth: 3, parent: None }
    { id: "execute_makegen_transport", depth: 4, parent: None }
  ]
  labels: {
    "fs_env": "fs", "load_registry": "load", "render_makefile": "render",
    "prepare_read_makegen": "read (prepare)", "execute_read_makegen": "read",
    "compare_makegen_content": "compare", "prepare_write_makegen": "write (prepare)",
    "execute_makegen_transport": "write"
  }
  subdag_boundaries: []
  parallel_groups: [{ nodes: ["fs_env", "load_registry"], depth: 0 }]
  scatter_points: []
}
```

### Terminal output (inline mode)

```
makegen ─ 4/4 ━━━━━━━━━━━━━━━━ 100% [✓ load] [✓ render] [✓ compare] [⊘ write]
```

(write skipped because content unchanged — the `[when !equal.equal]` guard evaluated false)

---

# Appendix B: Cloud Credential Acquisition (GCP)

The most complex graph in gunbc. The canonical stress test for the DSL.

## B.1 Today: Rust (gunbc)

### `lib/gcp-ops/src/graph.rs` — 1,688 lines (excerpt: first transport triplet)

```rust
// GitHub OIDC: prepare → execute → parse (one of ~8 such triplets)
let prepare = builder
    .add_root_node(Node::opaque(
        "prepare_github_oidc",
        vec![
            port("audience", "String"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareGitHubOidcRequest),
    )).expect("prepare_github_oidc");

let execute = builder
    .add_node_after(Node::opaque(
        "execute_github_oidc",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ), &prepare).expect("execute_github_oidc");

let parse = builder
    .add_node_after(Node::opaque(
        "parse_github_oidc",
        vec![port("response", "TransportResponse")],
        vec![port("subject_token", "String")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseGitHubOidcResponse),
    ), &execute).expect("parse_github_oidc");

builder.add_edge(prepare.out("request"), execute.in_port("request"));
builder.add_edge(prepare.out("skip"), execute.in_port("skip"));
builder.add_edge(net_env.out(NetEnv::PORT), execute.in_port(RESOURCE_API_NETWORK));
builder.add_edge(execute.out("response"), parse.in_port("response"));
// ... repeat for STS exchange, impersonation, secret access (~30 lines each)
```

**Plus**: `ops.rs` (2,077 lines), service traits (180 lines each), generated tests (157K chars).

**Total: ~4,000+ lines across 6+ files.**

### Service trait (`lib/gcp-ops/src/services/secret_manager.rs` — excerpt)

```rust
pub trait SecretManagerService {
    fn access_secret_version(&self, project: &str, secret: &str, version: &str) -> RestRequest;
    fn get_secret(&self, project: &str, secret: &str) -> RestRequest;
    fn create_secret(&self, project: &str, secret_id: &str) -> RestRequest;
    fn add_secret_version(&self, project: &str, secret: &str, payload_base64: &str) -> RestRequest;
}

pub const ACCESS_SECRET_VERSION_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets/{secret}/versions/{version}:access",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["secretmanager.versions.access"],
    service: "secretmanager",
};
```

## B.2 DSL

### `cloud/gcp/secret_manager.dag`

```
module cloud.gcp.secret_manager

service gcp.SecretManager {
  operation AccessVersion {
    input { project: String, secret: String, version: String = "latest" }
    output { payload: Bytes, name: String }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @idempotent @readonly
    @permissions(["secretmanager.versions.access"])
  }

  operation CreateSecret {
    input { project: String, secret_id: String }
    output { name: String }
    @rest(POST, "/v1/projects/{project}/secrets")
    @permissions(["secretmanager.secrets.create"])
  }

  operation AddVersion {
    input { secret_name: String, payload: Bytes }
    output { name: String }
    @rest(POST, "/v1/{secret_name}:addVersion")
    @permissions(["secretmanager.versions.add"])
  }
}
```

### `cloud/gcp/credential.dag`

```
module cloud.gcp.credential

import cloud.gcp.secret_manager
import cloud.gcp.iam
import cloud.gcp.sts
import std.patterns { credential_chain }

journey acquire_gcp_secret {
  input {
    runtime: CloudRuntime
    project: String
    secret_name: String
    audience: String = "sigstore"
    service_account: String?
  }
  output { token: AccessToken }
  provides auth: AuthContext

  cred = credential_chain(
    runtime: runtime,
    audience: audience,
    service_account: service_account,
    secret_name: secret_name,
    project: project
  )

  return { token: cred.token }
}
```

**Total: ~50 lines across 2 files** (vs. 4,000+ lines across 6+ files).

## B.3 What the Compiler Does

1. `gcp.SecretManager.AccessVersion(...)` expands to a transport triplet:
   - `prepare_access_version` (builds `RestRequest` from `@rest` annotation)
   - `execute_access_version` (transport boundary)
   - `parse_access_version` (extracts `payload`, `name` from response)

2. `credential_chain` pattern expands to:
   - `match runtime` → BranchBuilder with 3 arms
   - `gcp.STS.Exchange(...)` → transport triplet
   - `when service_account` → guarded node (impersonation optional)
   - `gcp.SecretManager.AccessVersion(...)` → transport triplet
   - `build_credential(...)` → pure node

3. Resource `Network` is inferred from service calls with `@rest` — the compiler inserts `net_env` and threads it to all transport execute nodes.

4. Test obligations derived automatically (100+ tests from the graph structure).

## B.4 Generated Tests (from compiler)

Same 4-bucket structure as gunbc testgen, but derived from the DSL rather than hand-wired:

```
Bucket A: DryRunCompletion, TransportInterceptable × 4
Bucket B: EdgePredicateEntailment × 2, NodeContractCompliance × 14, OptionalInputHandling × 8
Bucket C: AllTransportsSucceed, SingleTransportFailure × 4, GuardBranchCoverage × 2
Bucket D: TransportResourceDeclared × 4, ResourceInputConnected × 4
```

---

# Appendix C: Service Composition (Gist Snapshot)

Shows journey composition, loops, and multi-service orchestration.

## C.1 Today: Rust (gunbc)

`lib/tools/gist/src/graph.rs` — 1,449 lines covering 3 modes (snapshot, diff, recent).

The snapshot mode alone involves:
- Git operations (branch resolution SubDag, ls-files)
- Loop over files (LoopBuilder for per-file reads)
- Markdown rendering
- Cloud credential chain (SubDag)
- Gist API call (transport triplet)

## C.2 DSL

### `services/git.dag`

```
module services.git

service git.Core {
  operation CurrentBranch {
    input {}
    output { branch: String }
    @shell(["git", "rev-parse", "--abbrev-ref", "HEAD"])
  }

  operation LsFiles {
    input {}
    output { files: List<String> }
    @shell(["git", "ls-files"])
  }

  operation Diff {
    input { base: String, head: String = "HEAD" }
    output { diff: String }
    @shell(["git", "diff", "{base}...{head}"])
  }

  operation RevList {
    input { since: String }
    output { commits: List<String> }
    @shell(["git", "rev-list", "--since={since}", "HEAD"])
  }
}
```

### `services/github/gist.dag`

```
module services.github.gist

service github.Gist {
  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output { url: String, id: String }
    @rest(POST, "https://api.github.com/gists")
    @permissions(["gist"])
  }
}
```

### `tools/gist.dag`

```
module tools.gist

import services.git
import services.github.gist
import std.patterns { credential_chain }

journey gist_upload {
  input {
    markdown: String
    branch: String
    base_ref: String?
  }
  output { url: String }
  uses net: Network
  uses auth: AuthContext                 // threaded to github.Gist.Create via @auth

  filename = gist_filename(branch: branch, base_ref: base_ref)
  cred = credential_chain(runtime: detect_runtime(), ...)

  result = github.Gist.Create(
    description: "Snapshot from {branch}",
    files: { filename: markdown }
  )

  return { url: result.url }
}

journey gist_snapshot {
  input { base_ref: String? }
  output { url: String }
  uses fs: Filesystem(mode: Read)

  branch = git.Core.CurrentBranch()
  files = git.Core.LsFiles()

  contents = for file in files.files {
    fs.read(path: file)
  }

  markdown = render_snapshot(files: contents)
  result = gist_upload(
    markdown: markdown,
    branch: branch.branch,
    base_ref: base_ref
  )

  return { url: result.url }
}

journey gist_diff {
  input { base_ref: String }
  output { url: String }

  branch = git.Core.CurrentBranch()
  diff = git.Core.Diff(base: base_ref)
  markdown = render_diff(diff: diff.diff)
  result = gist_upload(
    markdown: markdown,
    branch: branch.branch,
    base_ref: base_ref
  )

  return { url: result.url }
}

journey gist_recent {
  input { since: String = "3.days.ago" }
  output { url: String }

  branch = git.Core.CurrentBranch()
  commits = git.Core.RevList(since: since)
  diffs = for commit in commits.commits {
    git.Core.Diff(base: "{commit}~1", head: commit)
  }
  markdown = render_recent(diffs: diffs)
  result = gist_upload(
    markdown: markdown,
    branch: branch.branch
  )

  return { url: result.url }
}
```

**Total: ~80 lines** (vs. 1,449 lines for the Rust graph builder).

## C.3 ProgressManifest for `gist_snapshot`

```
ProgressManifest {
  total_nodes: 12  // includes expanded credential_chain SubDag
  topology: [
    { id: "branch", depth: 0, parent: None }
    { id: "files", depth: 0, parent: None }
    { id: "loop:contents", depth: 1, parent: None }
    { id: "render", depth: 2, parent: None }
    { id: "cred_chain", depth: 3, parent: None }
    { id: "gist_create", depth: 4, parent: None }
  ]
  labels: { "branch": "branch", "files": "ls-files", "loop:contents": "read files",
            "render": "render", "cred_chain": "credential", "gist_create": "upload" }
  subdag_boundaries: [{ node_id: "cred_chain", label: "credential", inner_nodes: [...] }]
  parallel_groups: [{ nodes: ["branch", "files"], depth: 0 }]
  scatter_points: ["loop:contents"]
}
```

**Terminal output (inline)**:
```
gist ─ 4/6 ━━━━━━━━━━━━░░░░ 67% [✓ branch] [✓ ls-files] [✓ read 8/8] [✓ render] [◐ credential] [○ upload]
```

The `read 8/8` is a scatter group — the LoopBuilder expanded to 8 parallel file reads.

---

# Appendix D: CI Pipeline

Shows pipeline construct with stages, parallel groups, and aggregation.

## D.1 Today: Rust (gunbc)

`gunbc-dag/src/ci/graph.rs` — 920 lines.

## D.2 DSL

### `pipelines/ci.dag`

```
module pipelines.ci

import tools.makegen
import tools.bootstrap
import meta.testgen
import meta.codegen

pipeline ci {
  stage codegen {
    codegen.check()
  }

  stage generate [after codegen] {
    parallel {
      bootstrap()
      pragma()
      testgen()
    }
  }

  stage build [after generate] {
    cargo_build()
  }

  stage verify [after build] {
    parallel {
      cargo_test()
      clippy()
    }
  }

  stage report [after verify] {
    aggregate(results: [verify.*])
  }
}
```

## D.3 ProgressManifest

```
ProgressManifest {
  total_nodes: 8
  topology: [
    { id: "codegen.check", depth: 0, parent: None }
    { id: "bootstrap", depth: 1, parent: None }
    { id: "pragma", depth: 1, parent: None }
    { id: "testgen", depth: 1, parent: None }
    { id: "cargo_build", depth: 2, parent: None }
    { id: "cargo_test", depth: 3, parent: None }
    { id: "clippy", depth: 3, parent: None }
    { id: "aggregate", depth: 4, parent: None }
  ]
  // Stage groups are derived from pipeline stage declarations + topology.
  // They are in the manifest (pipeline structure is execution structure, not display metadata).
  subdag_boundaries: []
  parallel_groups: [
    { nodes: ["bootstrap", "pragma", "testgen"], depth: 1 }
    { nodes: ["cargo_test", "clippy"], depth: 3 }
  ]
  scatter_points: []
}
```

**Terminal output (inline)**:
```
ci ─ stage: verify 6/8 ━━━━━━━━━━━━░░░░ 75%
  [✓ codegen] [✓ bootstrap ✓ pragma ✓ testgen] [✓ build] [◐ test ◐ clippy] [○ report]
```

---

# Appendix E: Tool Installation (Upsert)

Shows the upsert pattern for tool installation.

## E.1 Today: Rust (gunbc)

`lib/tools/clippy/src/graph.rs` — 186 lines using `UpsertBuilder`.

```rust
let node = UpsertBuilder::new("install_clippy")
    .with_check(ClippyOp::Check)        // which clippy-driver
    .with_create(ClippyOp::Install)      // rustup component add clippy
    .with_resolve(ClippyOp::Resolve)     // clippy-driver --version
    .build();
```

## E.2 DSL

### `tools/clippy.dag`

```
module tools.clippy

import std.patterns { upsert }

resource Clippy {
  kind: Capability
  mode: Read
  lifecycle: Persistent           // survives across invocations

  acquire {
    upsert {
      check:   shell(["which", "clippy-driver"]) -> { exists: Bool }
      create:  shell(["rustup", "component", "add", "clippy"])
      resolve: shell(["clippy-driver", "--version"]) -> { handle: String }
    }
  }
}

journey clippy_lint {
  input { paths: List<String>? }
  output { clean: Bool, findings: String }
  uses clippy: Clippy             // tool availability as a resource

  result = shell(["cargo", "clippy", "--", "-D", "warnings"])
  return { clean: result.exit_code == 0, findings: result.stdout }
}
```

---

# Appendix F: LLM Review Workflow

Shows cloud credential + LLM service composition.

## F.1 Today: Rust (gunbc)

`lib/review/src/graph.rs` — 1,376 lines with blob acquisition, credential chain, LLM request.

## F.2 DSL

### `tools/review.dag`

```
module tools.review

import services.git
import cloud.gcp.credential
import std.patterns { credential_chain }

service llm.OpenAI {
  operation ChatCompletion {
    input {
      model: String = "gpt-4"
      messages: List<Message>
      temperature: Float = 0.3
    }
    output { content: String, usage: TokenUsage }
    @rest(POST, "https://api.openai.com/v1/chat/completions")
  }
}

type Message {
  role: String        // "system" | "user" | "assistant"
  content: String
}

type TokenUsage {
  prompt_tokens: Int
  completion_tokens: Int
}

journey review_diff {
  input {
    base_ref: String
    system_prompt: String?
  }
  output { review: String }
  uses net: Network
  uses auth: AuthContext                 // threaded to llm.OpenAI via @auth

  diff = git.Core.Diff(base: base_ref)
  cred = credential_chain(runtime: detect_runtime(), ...)

  prompt = build_review_prompt(
    diff: diff.diff,
    system: system_prompt
  )

  result = llm.OpenAI.ChatCompletion(
    messages: prompt
  )

  return { review: result.content }
}
```

---

# Appendix G: Rendering / Emission

gunbc has 13 rendering systems, 5 different traits, 8 with no trait. The DSL eliminates this entirely: **rendering is a pure function node**. No special system, no trait hierarchy. A `fn makefile_render(...)` or `fn ci_yaml_render(...)` is just another functor whose output flows through the DAG like anything else. The `content_upsert` pattern handles writing the rendered output to a file. Polymorphism across formats (Ansi/Plain/HTML/Markdown) is handled by a `RenderFormat` enum parameter to the functor — no concept/trait abstraction needed.

---

# Appendix H: Pattern Catalog

All patterns from gunbc, expressed in DSL syntax.

### Upsert (check → create → resolve)
```
pattern upsert<Check, Create, Resolve> {
  node check: Check -> { exists: Bool }
  node create [after check, when !check.exists]: Create
  node resolve [after check, after create]: Resolve -> { handle: String }
}
```

### Content Upsert (generate → read → compare → skip-if-unchanged write)
```
pattern content_upsert {
  input { content: String, path: String }
  uses fs: Filesystem(mode: ReadWrite)

  node read: fs.read(path: path)
  node equal: eq(a: content, b: read.content) -> { equal: Bool }
  node write [when !equal.equal]: fs.write(path: path, content: content)

  output { written: Bool = !equal.equal }
}
```

### Credential Chain (OIDC → STS → optional impersonation → secret access)
```
pattern credential_chain {
  input { runtime: CloudRuntime, audience: String, service_account: String?, ... }
  uses net: Network
  provides auth: AuthContext

  node token = match runtime { ... }
  node access = gcp.STS.Exchange(subject_token: token.token)
  node impersonated = match service_account { Some(sa) => ... None => access }
  node secret = gcp.SecretManager.AccessVersion(...)

  output { token: AccessToken }
}
```

### Transaction (begin → body → commit/rollback)
```
pattern transaction<Begin, Body, Commit, Rollback> {
  node begin: Begin -> { tx_id: String }
  node body: Body
  node commit [when body.success]: Commit
  node rollback [when !body.success]: Rollback
}
```

### Retry (execute → check → re-execute on failure)
```
pattern retry<Op> {
  input { max_attempts: Int = 3, backoff_ms: Int = 1000 }
  node op: Op
  @retry(max: max_attempts, backoff: exponential(backoff_ms))
}
```

### Loop (iterate over collection)
```
// Expressed inline in journeys:
contents = for file in files.files {
  fs.read(path: file)
}
```

### Branch (conditional routing)
```
// Expressed inline:
node result = match condition {
  A => journey_a(...)
  B => journey_b(...)
}

// Or with when:
node optional_step [when flag] { ... }
```

### Emit (prepare → format → hash → compare → write → record)
```
pattern emit {
  input { content: String, path: String }
  uses fs: Filesystem(mode: ReadWrite)

  node hash: content_hash(content: content)
  node read_existing: fs.read(path: "{path}.hash")
  node equal: eq(a: hash.hash, b: read_existing.content) -> { equal: Bool }
  node write_content [when !equal.equal]: fs.write(path: path, content: content)
  node write_hash [when !equal.equal]: fs.write(path: "{path}.hash", content: hash.hash)

  output { written: Bool = !equal.equal }
}
```

---

# Appendix I: Inspiration Targets

| Source | What to take | What to avoid |
|--------|-------------|---------------|
| **Smithy (AWS)** | `service` + `operation`, `@trait` annotations, resource lifecycle | XML heritage, complex trait algebra |
| **Terraform HCL** | Provider-qualified names, implicit deps from references | Mutable state, plan/apply split |
| **CUE** | Constraints inline, defaults, unification for inheritance | Value lattice complexity |
| **dbt** | `ref()` implicit DAG, model auto-discovery from filesystem | SQL-only, no type system |
| **Concourse CI** | Resource types, pipeline-as-DAG, `passed` constraints | YAML, no composition |
| **Dhall** | Totality, no side effects, imports with integrity checks | Haskell syntax barrier |
| **Protobuf** | Language-agnostic IDL, codegen plugins, evolution rules | No computation, no DAG |
| **the-gunbai TUI** | Inline progress, TUI DAG viz, edge pulses, wave layout | Runtime-only layout |
| **Nix** | Reproducible, declarative, lazy | Complexity, learning curve |

**Anti-inspirations**:
- **Airflow**: Imperative DAG construction (exactly what we're replacing)
- **YAML pipelines**: No type system, stringly-typed (what V2 rejected)
- **Terraform state**: Mutable state management (our model is stateless)
- **Pulumi**: Host-language coupling (defeats language-agnosticism)
- **Helm**: Template-of-a-template layering (complexity without guarantees)

---

# Appendix J: Cross-Repository Capability Matrix

This appendix documents what each generation of the platform (gunb.ai, the-gunbai, gunbc) provides "for free" — meaning what the framework gives you without manual effort — and how capabilities transfer across the lineage as scenarios evolve.

## J.1 The Lineage: What Each Generation Proved

```
gunb.ai (v1, Go)          the-gunbai (v2, Rust)       gunbc (v3, Rust)           DSL (v4, target)
──────────────────         ──────────────────────      ──────────────────         ──────────────────
DAGs work                  Codegen from knowledge      Full IR + proofs           Language-level DAGs
CaptureWriter              TUI progress                Testgen (73% gen)          95% generated code
Lease execution            40+ understandings          Transport boundaries       Filesystem discovery
                           195+ behaviors              DryRun interception        Multi-target emission
```

### What "for free" means at each generation

| Generation | "For free" = | You still write manually |
|---|---|---|
| **gunb.ai** | Parallel execution, output capture, progress sections | Everything: graph wiring, tests, types, discovery, progress groups |
| **the-gunbai** | Integration code from understandings, TUI progress, some contract tests | Graph wiring, most tests, IR is implicit, no structural guarantees |
| **gunbc** | Structural soundness, 73% of tests, DryRun, transport interception, pattern reuse | Graph builders (7,000+ lines), discovery (6 islands), progress rendering |
| **DSL** | Graph authoring (10-100x compressed), discovery, progress manifest, multi-language codegen | Pure transformation logic (~5% of total code) |

## J.2 Capability Transfer Matrix

What each repo contributes to the DSL, organized by concern. Per-scenario detailed comparisons are in Appendices A-F.

### DAG Modeling

| What transfers | From | To DSL as |
|---|---|---|
| `Node<T>`, `Dag<T>`, `Port`, `Edge` types | gunbc | Core IR target (identical structure after lowering) |
| `DagBuilder` with generations | gunbc | Compiler's `Lower` phase produces same builder output |
| `Cardinality` (One, ZeroOrOne, ZeroOrMore, OneOrMore) | gunbc | Simplified to `T`, `T?`, `List<T>` in surface syntax |
| Acyclicity by construction | gunbc | Guaranteed by language (no cycles expressible in `.dag`) |
| Boundary/entrypoint detection | gunbc | Compiler's `Validate` phase (same algorithm) |

### Testing

| What transfers | From | To DSL as |
|---|---|---|
| 4-bucket obligation model | gunbc testgen | Compiler's `Derive` phase produces `TestObligations` |
| Anti-tautology rule | gunbc testgen | Same: only generate tests for Unknown/RuntimeOnly obligations |
| DryRun completion test | gunbc | Generated for every journey |
| Transport interception test | gunbc | Generated for every service call |
| N+1 scenario coverage | gunbc | Generated: all-succeed + per-transport failure |
| Guard branch coverage | gunbc | Generated from `when` / `match` constructs |
| Resource hygiene | gunbc | Generated from `uses` declarations |
| MockSpec infrastructure | gunbc | Compiler generates MockSpec from service declarations |
| `Simulator` / `IoContract` | gunbc | Generated from typed ports |

### Progress & Terminal

| What transfers | From | To DSL as |
|---|---|---|
| `CaptureWriter` pattern | gunb.ai | Per-node `CaptureBuffer` (default for all transport nodes) |
| Passthrough mode | gunb.ai | `@interactive` annotation → `CaptureMode::Passthrough` |
| Section rendering (`›`) | gunb.ai | Inferred from SubDag boundaries in ProgressManifest |
| Error boxes (bordered, captured stderr) | gunb.ai | Same visual design, driven by CaptureBuffer on failure |
| Color palette (ANSI 256) | gunb.ai → gunbc | Identical: `SemanticColor` enum, same codes |
| Spinner (braille, 80ms) | gunb.ai → gunbc | Identical: same frames, same timing |
| TUI with edge pulses | the-gunbai | Optional `tui` renderer reading ProgressManifest |
| Wave-based layout | the-gunbai | `TopologyNode.depth` in ProgressManifest |
| Scatter groups (`[2/5]`) | the-gunbai | `scatter_points` in ProgressManifest |
| Inline progress bar | the-gunbai | `inline` renderer reading ProgressManifest |
| JSONL event streaming | the-gunbai | `jsonl` renderer reading ProgressManifest |
| Frame-based display (`build_frame()`) | gunbc | Manifest-driven frame builder (same pure function concept) |
| `OutputMedium` / `SemanticColor` / `SymbolId` | gunbc | Terminal crate (harvested directly, ~2,271 lines) |

### Services & Resources

| What transfers | From | To DSL as |
|---|---|---|
| Understanding concept (structured external system docs) | the-gunbai | `service` declarations with `@rest`, `@shell` annotations |
| Behavior generation from understandings | the-gunbai | Compiler expands service operations to transport triplets |
| `SecretManagerService` trait + `MethodMeta` | gunbc | `service gcp.SecretManager { operation ... }` |
| Transport triplet (prepare/execute/parse) | gunbc | Compiler generates from service call + `@rest`/`@shell` |
| DryRun interception at transport boundary | gunbc | Same: mock transport executor swapped at execute node |
| `ResourceAccess` / `detect_conflicts()` | gunbc | Compiler's resource conflict check in `Validate` phase |
| Typed resource ports (`res:*`) | gunbc | `uses fs: Filesystem(mode: Write)` — compiler threads edges |
| Lease/heartbeat model | gunb.ai | `resource` with lifecycle (acquire/use/release) |

### Discovery

| What transfers | From | To DSL as |
|---|---|---|
| Manual hardcoded lists | gunb.ai, the-gunbai | Eliminated |
| `#[tool_target]` proc macro | gunbc | Eliminated (filesystem discovery) |
| `#[testgen_target]` proc macro | gunbc | Eliminated (every journey has test obligations) |
| `build_workspace_dag()` | gunbc | Eliminated (module graph IS workspace DAG) |
| `inventory` crate | gunbc | Eliminated |

Each generation's "free" capabilities compound: the DSL gets parallel execution from gunb.ai + codegen from the-gunbai + structural proofs from gunbc + language-level compression from the new compiler.


---

# Appendix K: Root Cause Analysis — Why gunbc Got Out of Control

This appendix documents the precise failure modes that caused gunbc's codebase to accumulate glue, drift, and rework pressure — and traces each failure mode to the DSL construct that eliminates it.

The framing draws on internal postmortem documents: `TODO/TODONE/refactor-pressure.md` (2026-02-05), `TODO/TODONE/architecture-debt.md` (2026-02-05), `docs/design/consolidation-plan.md`, `docs/design/unified-registration.md`, and `docs/design/unified-emission.md`.

## K.1 The Precise Diagnosis

gunbc's IR is strong and ambitious. The design philosophy — "Everything is a DAG," behavior must be representable structurally, semantic meta-annotations are banned — is a very high bar for modeling. The core IR (`Node<T>`, `Dag<T>`, `Port`, `Edge`, cardinality algebra, transport boundaries, pattern library) is proven correct and heavily tested.

**The failure was not "lack of modeling." It was incomplete modeling: the IR layer was modeled aggressively, while the spec/registry/discovery/emission/progress layers often weren't, so meaning leaked into glue.**

From `architecture-debt.md` (2026-02-05 Weekly Signal):

> When a concept lacks a typed, structural home (IR/model/registry/resource), it leaks into templates, env access, string IDs, and ad-hoc rules — and then we refactor later to pull it back into structure.

And from `refactor-pressure.md`:

> We keep refactoring because the system still allows key behavior and meaning to exist outside the model (DAG/resources/types/IR), and the resulting duplicate sources of truth drift until they force a structural cleanup.

## K.2 Four Root Causes (from Internal Postmortem)

### A) Model is not closed — behavior exists outside the DAG

Code reached out to the environment implicitly: `std::env::var()`, `SystemTime::now()`, `Platform::detect()`, `FilesystemHandle::new()`. These calls happened inside opaque nodes, breaking DryRun interception, testability, and dependency reasoning.

**Leak → Fix dynamic (completed 2026-02-05):**

| Leak | Fixed with | Phase |
|---|---|---|
| Inline `SystemTime::now()` | `ClockEnv` node with explicit env port | Resource Phase 2 |
| Inline `Platform::detect()` | `PlatformEnv` node with explicit env port | Resource Phase 3 |
| Inline `FilesystemHandle::new()` | `FsEnv` node with explicit resource port | Resource Phase 1 |
| `std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL")` | Explicit input ports on graph root | Resource Phase 4 |
| `GUNBC_EXEC_MODE` global | Exec mode via DAG edge | Resource Phase 5 |

**DSL fix:** `uses fs: Filesystem(mode: Write)` / `uses clock: Clock` / `uses net: Network`. Resources are declared, not accessed implicitly. The compiler inserts acquisition nodes and threads resources through edges. No code path can reach the environment without a declaration.

### B) Invariants are not enforced by construction — policy exists but allows escape hatches

The system had policies ("all I/O through transport boundaries," "no hidden env access") but allowed escape hatches. Churn happened when code violated policies and only discovered the violation during review, lint, or runtime.

**Examples:**
- `execute_transport()` was public — any opaque node could call it directly, bypassing the transport boundary model. **Fixed:** removed from public API; only `TransportOps::Execute` can perform I/O.
- `clippy.toml` banned `std::fs` and `std::process::Command`, but pragmas could disable the ban. **Fixed:** pragma audit with explicit exception list.
- Generated code could trigger lint warnings. **Policy:** if generated file triggers lint, fix the IR or clippy config — never add `#[allow]` in generated output.

**DSL fix:** The language itself prevents escape hatches. You cannot express I/O in a `.dag` file — the only way to do I/O is through a `service` operation with a `@rest` / `@shell` / `@file` annotation, which the compiler expands to a transport triplet. Escape hatches are not available in the surface syntax.

### C) Semantics are duplicated across layers — same meaning lives in two places

When the same concept exists in two places, they drift until cleanup becomes necessary.

**Documented examples:**

| Duplicate concept | Where it lived | Drift consequence |
|---|---|---|
| Boundary mocks | `registry.rs` AND `graph_mock.rs` | Mock values could disagree; tests might pass with stale mocks |
| Tool definitions | `all_tools()` vec AND `CliToolDef` constants | Forgot to add to `all_tools()` = tool exists but isn't discoverable |
| Hash logic | Per-crate implementations | Different crates computed different hashes for same content |
| Cardinality | Port cardinality AND type contract predicates | `Optional<T>` + `One` cardinality = contradictory claims |
| Graph builder identity | `GraphBuilderId` enum AND string templates | Rename builder function → string silently emits wrong name → runtime failure |

**DSL fix:** Single source of truth by construction. A `service` declaration IS the boundary mock source (the compiler generates MockSpec from `@rest` annotations). A `.dag` file IS the tool definition AND the graph builder AND the registry entry. Filesystem discovery eliminates all manual lists.

### D) Cross-cutting concerns appear before they have a home

Shared concerns (hashing, registry metadata, build artifact policy, resource dependency rules) got wedged into convenient places until third duplication forced a refactor.

**The pattern:** hashing appeared in three crates before `gunbc-infra::hash` was extracted. Freshness checking appeared in two crates before `gunbc-infra::freshness`. Rendering appeared in 13 places before the emission unification design was written.

**DSL fix:** The compiler pipeline has explicit homes for every cross-cutting concern: `Discover` phase for registry, `Derive` phase for progress manifest, `Emit` phase for rendering, `Validate` phase for resource conflicts. Concerns are modeled in the compiler, not discovered ad-hoc across crates.

## K.3 The "Generic IR Chokepoint" Problem

Beyond the four root causes, gunbc forced too much meaning through a generic IR chokepoint, and some semantics weren't preserved across that boundary.

**The canonical example: hermeticity.**

From `TODO/consolidation.md` (Integration Test Gap Analysis):

> **Design Problem: `TransportRequest` doesn't encode hermeticity.**
>
> We want test categories derived from the transport type system: **integration** (hermetic, local-only) vs **external** (non-hermetic, network/auth). But `TransportRequest` variant alone doesn't determine this.
>
> The problem is `Shell`. Higher-level domain types know whether they're hermetic, but that information is erased when they convert to `TransportRequest::Shell`:
>
> ```
> GitRequest::LsFiles.to_shell_request()    → Shell { ... }   // hermetic
> GistRequest::new().to_shell_request()     → Shell { ... }    // non-hermetic
> CargoCommand::Build.to_shell_request()    → Shell { ... }    // hermetic
> ```
>
> After conversion, these are indistinguishable at the transport layer. The executor sees `Shell(ShellRequest {...})` and has no way to know whether it hits the network.

This is the "IR-only" failure mode in a nutshell:
- The IR is too generic at the transport layer
- You lower into `TransportRequest::Shell` early
- You lose distinctions that matter for execution, policy, and test classification

**DSL fix:** Transport is late-bound (Design Principle P5). Service declarations carry semantic metadata (`@rest`, `@shell`, `@permissions`, `@idempotent`, `@readonly`) that the compiler preserves through lowering. The compiler can classify `git.Core.LsFiles()` as hermetic (local shell, no network annotation) vs `github.Gist.Create()` as non-hermetic (`@rest` with `@permissions`) — because the service declaration IS the source of truth, not a generic `Shell` variant.

## K.4 What Is NOT a Failure (Correcting Overstatements)

The internal reconciliation notes (`consolidation-plan.md`) explicitly correct some "redundancy claims" that were wrong or overstated:

| Claimed redundancy | Actual status |
|---|---|
| "Dual-source boundary mocks" | Not actually dual-sourced — MockSpec.to_boundary_mocks() is the single source |
| "CliToolDef/ToolDef duplication" | Intentional separation: platform satisfiability (ToolDef) vs runtime acquisition (CliToolDef) |
| "Type/cardinality duplication" | Already unified via TypeRegistry.infer_cardinality() |

The lesson: not all apparent duplication is harmful. Some "duplicate" structures serve distinct purposes and should remain separate.

## K.5 Root Causes → DSL Features Traceability Matrix

Each gunbc pain point mapped to the DSL construct that eliminates it and the compiler pass that enforces it.

| # | Pain Point | Root Cause | DSL Construct | Compiler Pass | Evidence |
|---|---|---|---|---|---|
| 1 | 7,000+ lines of hand-wired builders | No front-end language | `.dag` files with journey/pattern syntax | Parse → Lower | `dsl-design.md` §1 |
| 2 | 6 registration islands | No unified discovery | Filesystem as registry | Discover | `unified-registration.md` |
| 3 | `all_tools()` hardcoded vec (360 lines) | Manual bottleneck | Every `.dag` file auto-discovered | Discover | `unified-registration.md` §3 |
| 4 | `GraphBuilderId` string coupling | Meaning outside model (C) | Function pointers from module graph | Discover + Resolve | `consolidation-plan.md` §3 |
| 5 | 13 rendering systems, 5 traits | No emission model (D) | `ProgressManifest` + renderer trait | Derive + Emit | `unified-emission.md` |
| 6 | Hidden env access in opaque nodes | Model not closed (A) | `uses` declarations + compiler-inserted env nodes | Lower | `refactor-pressure.md` §A |
| 7 | `execute_transport()` escape hatch | No construction enforcement (B) | Service ops → compiler-emitted triplets | Lower | `refactor-pressure.md` §B |
| 8 | `format!()` codegen constructing source | No emission IR (D) | TestFile IR + TestRenderer trait | Emit | `architecture-debt.md` |
| 9 | Hermeticity erased at transport layer | Generic IR chokepoint | Service annotations preserved through lowering | Lower + Validate | `consolidation.md` §8 |
| 10 | Boundary mocks defined in two places | Semantic duplication (C) | MockSpec derived from service declarations | Derive | `unified-registration.md` §4 |
| 11 | Resource lifecycle implicit | Incomplete resource model | `resource` with acquire/use/release | Lower + Validate | `dsl-design.md` §7 |
| 12 | Progress rebuilt from scratch (lost TUI quality) | No progress model | `ProgressManifest` at compile time | Derive | `dsl-design.md` §6 |
| 13 | New tools require manual addition to `build_workspace_dag()` | Discovery doesn't include meta-tools | Module graph IS workspace DAG | Discover | `dsl-design.md` §5 |
| 14 | Manual MockSpec per tool | Test infrastructure not generated | Compiler generates MockSpec from service declarations | Derive | `dsl-design.md` §8 |
| 15 | `Value`/`ValueExpr` parallel hierarchies | Emission leaked into IR | Codegen works from IR + types; no runtime value expressions | Emit | `architecture-debt.md` |

## K.6 Guardrails for the DSL (Preventing Re-Creation of Failure Modes)

The feedback raises an important concern: how do we prevent re-creating gunbc's failure modes *inside* the DSL? Three specific guardrails:

### G1: Annotations must desugar to structure

Design Principle P9 ("The language is total") and the IR philosophy's ban on semantic meta-annotations must apply to the DSL surface syntax. `@interactive`, `@rest`, `@idempotent`, etc. must **desugar into explicit structural nodes/fields** in the lowered IR, not remain as opaque annotations that modify behavior outside the model.

**Test:** Can you delete the annotation and get a compile error or behavior change that's visible in the IR? If the annotation has no structural representation, it's a semantic meta-annotation and violates P9.

### G2: Preserve producer-level semantics through lowering

Hermeticity is the canonical example. If the DSL keeps a generic transport layer (it does — Design Principle P5), it must carry semantic properties from the service declaration through compilation and execution.

**Options from `consolidation.md`:**
- Field on `TransportRequest` (e.g., `hermetic: bool`)
- Split `Shell` variant into `LocalShell` / `NetworkShell`
- Node-level annotation that survives lowering

**Recommendation:** The DSL compiler should propagate `@rest` / `@shell` / `@idempotent` / `@readonly` / `@permissions` as metadata on the lowered transport node, so the executor and test categorizer can access them without re-deriving them from string inspection.

### G3: Kill manual bottlenecks first

The internal docs identify `all_tools()` as the #1 source of silent omission bugs. The DSL's filesystem discovery eliminates this entirely — but the Phase 1 implementation must verify this is true end-to-end: every `.dag` file in the `paths` directories must appear in the module graph, and the module graph must be the sole source for downstream automation (Makefile targets, CLI generation, testgen registration).

**Metric (from `refactor-pressure.md`):**
- Manual tool registrations → 0
- Stringly `GraphBuilderId` references → 0
- Rendering systems without IR/trait → 0
- `format!()` constructing source code → 0

---

# Appendix L: References

A/B workflow comparisons (imperative/OO/functional vs gunbc DAG) are in `docs/ab-writing-workflows.md`. The handbook pattern catalog and E2E pipeline are in `docs/handbook.md`. Appendices A-F contain per-workflow detailed comparisons. This appendix retains only the consolidation status and file path reference.


## L.1 Consolidation Status (Living Reference)

Current status of the six work streams from `docs/design/consolidation-plan.md`, included here so the DSL design can track which gunbc issues are already fixed vs still need to be addressed by the language.

| Stream | Problem | Status | DSL Eliminates? |
|---|---|---|---|
| **1. Registration Unification** | 6 registration islands, manual `all_tools()` vec | R1-R2 complete (tool-registry crate + annotations) | Yes — filesystem discovery |
| **2. Emission Unification** | 13 rendering systems, 5 traits, 8 with no trait | Design complete, implementation not started | Yes — `Emit` phase with `CodegenBackend` |
| **3. String-Coupled Dispatch** | `GraphBuilderId::as_str()` breaks at runtime | Absorbed by Stream 1 Phase R2 | Yes — function pointers from module graph |
| **4. Documentation Consistency** | Handbook contradictions (e.g., "I/O enforcement complete" vs migration table) | Pending | Yes — this document IS the consolidated doc |
| **5. CI Verification Gaps** | `make verify` not in CI, generated files not verified | `make verify` exists, not yet in CI | Yes — compiler verifies `.dag` → generated output |
| **6. CliToolDef/ToolDef Alignment** | Two tool types with field overlap | Intentional separation, no action | N/A — DSL has `resource` + service ops |

## L.2 Reference: Key File Paths

For navigating between this design doc and the source material it consolidates:

| Document | Path | Key Content |
|---|---|---|
| DSL Design (this doc) | `docs/design/dsl-design.md` | Language spec, all appendices |
| Handbook | `docs/handbook.md` | Pattern catalog, E2E examples, repo map |
| A/B Workflows | `docs/ab-writing-workflows.md` | Imperative/OO/functional vs gunbc DAG comparisons |
| Design Overview | `docs/design/overview.md` | Philosophy, invariants, formal model |
| Testgen | `docs/design/testgen.md` | Obligation model, 4 buckets, anti-tautology rule |
| Unified Registration | `docs/design/unified-registration.md` | 6 registration islands → unified discovery |
| Unified Emission | `docs/design/unified-emission.md` | 13 rendering systems → OutputMedium hierarchy |
| Consolidation Plan | `docs/design/consolidation-plan.md` | 6 work streams, reconciliation status |
| Refactor Pressure | `TODO/TODONE/refactor-pressure.md` | Root causes A-D, decision rules, quick scans |
| Architecture Debt | `TODO/TODONE/architecture-debt.md` | Meta-root-cause, leak→fix table |
| IR Spec | `SPEC.md` | Formal IR specification |
| Agent Guide | `AGENT.md` | Onboarding, guardrails |

---

# Appendix M: Competitive Landscape and Alternatives Analysis

This appendix positions the DSL against the real alternatives people use to avoid hand-wiring graphs and integrations, identifies gaps, and documents paths-not-taken.

## M.1 What the DSL Actually Competes With

The DSL is not competing with Lombok or ORMs directly. It competes with three alternative ways people avoid hand-wiring:

| Category | Examples | What they do |
|---|---|---|
| **Host-language metaprogramming** | Java annotation processors / Lombok, Rust proc-macros, Python decorators | Generate boilerplate within one language |
| **Runtime orchestration frameworks** | LangGraph, LangChain, CrewAI, Temporal, Airflow, Dagster | Execute workflows at runtime with framework conventions |
| **IDLs that generate clients + models** | OpenAPI, Smithy, protobuf/gRPC, Thrift | Describe interfaces, generate multi-language code |

The DSL is a **hybrid of IDL + workflow compilation**: service/operation/type declarations (like Smithy/protobuf) plus workflow compilation into a typed DAG IR with transport boundaries, test derivations, and progress manifests. This hybrid is the distinguishing position — neither pure IDL nor pure orchestrator.

## M.2 What the DSL Provides That Alternatives Don't

Four capabilities that are first-class compiler output, not conventions:

| Capability | What it means | Closest alternative | Gap in alternative |
|---|---|---|---|
| **Transport triplets as structural primitive** | Service calls expand to prepare → execute → parse with skip wiring. Authors never see the triplet. | Smithy generates client stubs | No DAG structure, no skip wiring, no DryRun interception |
| **Proof-obligation test generation** | Tests derived from graph properties (4 buckets), discharged structurally or generated. Anti-tautology rule. | Dagster has `@asset` testing, but manual | No structural obligation model, no mechanical derivation |
| **Progress as derived topology view** | ProgressManifest computed at compile time from DAG structure. Renderers are pluggable views. | LangGraph has runtime streaming/tracing | Runtime-only, no compile-time manifest, no multi-renderer architecture |
| **Debuggable execution modes** | DryRun intercepts transport nodes, Simulate with timing. Step-by-step possible. | LangGraph has interrupt/resume | No compile-time boundary classification, no mock-spec derivation |

## M.3 Other Alternatives (Summary)

**Host-language metaprogramming (Lombok / proc-macros):** Generates boilerplate within one language. A Rust proc-macro alternative was considered (`#[dag] fn makegen(...)`) but rejected — it kills multi-target emission (P10). The DSL steals Lombok's "delombok" idea: `dag expand`, `dag show-triplets`, `dag obligations` commands show what the compiler produced.


**ORMs (Hibernate/JPA):** Share the "declarative metadata → generated behavior" pattern. Key lessons: (1) types/operations need versioning/evolution rules (protobuf-style), (2) unusual APIs need a `@custom` transport escape hatch (see M.6), (3) users need "show me what the compiler meant" tooling.

## M.4 Comparison: LangGraph

LangGraph is the closest real alternative. Both model workflows as graphs with explicit nodes and edges.

### LangGraph `review_diff` (concrete Python)

```python
from typing_extensions import TypedDict
from langgraph.graph import StateGraph, START, END

class State(TypedDict, total=False):
    base_ref: str
    diff: str
    prompt: list[dict]
    review: str

def get_diff(state: State) -> dict:
    return {"diff": run_git_diff(state["base_ref"])}

def build_prompt(state: State) -> dict:
    return {"prompt": [
        {"role": "system", "content": "You are a code reviewer."},
        {"role": "user", "content": state["diff"]},
    ]}

def call_llm(state: State) -> dict:
    resp = ChatOpenAI(model="gpt-4", temperature=0.3).invoke(state["prompt"])
    return {"review": resp.content}

builder = StateGraph(State)
builder.add_node("get_diff", get_diff)
builder.add_node("build_prompt", build_prompt)
builder.add_node("call_llm", call_llm)
builder.add_edge(START, "get_diff")
builder.add_edge("get_diff", "build_prompt")
builder.add_edge("build_prompt", "call_llm")
builder.add_edge("call_llm", END)

graph = builder.compile()
out = graph.invoke({"base_ref": "origin/main"})
```

### DSL `review_diff`

```
journey review_diff {
  input { base_ref: String }
  output { review: String }
  uses net: Network

  diff = git.Core.Diff(base: base_ref)
  prompt = build_review_prompt(diff: diff.diff)
  result = llm.OpenAI.ChatCompletion(messages: prompt)

  return { review: result.content }
}
```

### Where each is stronger

| Dimension | DSL | LangGraph |
|---|---|---|
| Graph typing | Explicit typed ports, cardinality, compile-time validation | Shared state dict with optional type hints (TypedDict) |
| I/O boundaries | Structural: service calls → transport triplets, DryRun intercepts | Convention: side effects live in node functions, mocking is manual |
| Test derivation | Compiler-derived: 4-bucket obligations, MockSpec from service declarations | Manual: write your own tests and mocks |
| Progress | Compile-time ProgressManifest → pluggable renderers | Runtime streaming/tracing callbacks |
| Multi-language | Core goal: Rust/Go/Python/TS backends | Python and TypeScript only |
| **Durability/HITL** | **Not core (yet)** | **Core feature: interrupt/resume, checkpointing, human approval** |
| **Dynamic fan-out** | `for` loops in IR, scatter groups | `Send` objects for map-reduce, reducer annotations |
| **Agentic patterns** | Not modeled (deterministic orchestration) | First-class: tool calling, dynamic routing, memory |

### The durability/HITL gap

LangGraph's first-class durability semantics (interrupt at any node, persist state, resume later, human-in-the-loop approval patterns) represent a genuine capability the DSL does not currently address.

**Options:**

1. **Ignore it.** The DSL targets deterministic workflow orchestration, not agentic HITL systems. Different problems.

2. **Model it structurally.** Add a `@durable` annotation on journeys that compiles to checkpointing infrastructure (state serialization at each transport boundary, resume from checkpoint). This extends the resource model — durable state becomes a resource with lifecycle.

3. **Treat LangGraph as an execution backend.** The DSL's Python codegen backend could emit LangGraph `StateGraph` code instead of raw Python. This borrows LangGraph's runtime (durability, HITL, tracing) while keeping the DSL's compile-time guarantees (typed ports, test derivation, progress manifest). The `.dag` file remains the contract; LangGraph is the execution engine.

**Recommendation:** Option 3 is the strongest near-term path. It preserves the DSL's unique value (typed, portable, test/progress derivations) while borrowing runtime capabilities where LangGraph is genuinely better. This aligns with Design Principle P10 (Language-agnostic) — execution backends are plugins.


## M.5 Side-by-Side Summary

| Dimension | `.dag` DSL + compiler | LangGraph | LangChain | CrewAI |
|---|---|---|---|---|
| Authoring | Declarative `.dag` + patterns | Python/TS graph builder | Python/TS runnable composition | YAML/Python agents + tasks |
| Graph semantics | Explicit typed DAG IR, compile-time invariants | Stateful graph over shared state | Mostly linear chains | Task process model (seq/hierarchical) |
| Dynamic fan-out | `for` loops → scatter groups in IR | `Send` map-reduce patterns | Manual Python loops | `kickoff_for_each` + custom code |
| Progress model | Derived manifest + pluggable renderers | Runtime streaming/tracing | Callbacks/tracing | Verbose logs/streaming |
| Testing | Structural obligations + DryRun intercept | Manual tests + mocks | Manual tests + mocks | Manual tests |
| Multi-language | Explicit goal (Rust/Go/Python/TS/MIPS) | No (Python/TS only) | Partial (Python/JS) | No (Python only) |
| Durability/HITL | Not core (addressable via LangGraph backend) | Core feature set | Not core | Available via processes |
| Agentic patterns | Not modeled (invoke as service boundary) | First-class | First-class | Core design |

## M.6 The "Are We Going in the Right Direction?" Test

**Build the DSL if** the goal is:
- Compress authoring (stop writing 7,000+ lines of builders)
- Keep structural guarantees (acyclicity, typing, saturation)
- Keep transport boundaries mockable (DryRun / interception)
- Keep or improve progress UX (topology-derived rendering)
- Support multi-target emission
- Make the workflow description a portable contract (".dag like .proto")

**Use LangGraph/Temporal directly if** the goal is:
- Long-lived, stateful, human-in-the-loop agent platform
- LLM decides next steps dynamically
- Checkpointing/resume as core requirement
- Python/TS ecosystem integration is more important than multi-language codegen

**The reconciliation:** These are not mutually exclusive. The `.dag` file is the canonical contract. Execution backends are pluggable. A LangGraph backend for Python emission would borrow durability/HITL capabilities while preserving the DSL's compile-time guarantees. A Temporal backend would provide distributed execution. The DSL's value is in what happens *before* execution: structural proof, test derivation, progress manifests, multi-target code generation.

## M.7 Gaps and Tooling Needs Identified

From the competitive analysis, three concrete gaps and tooling needs:

### Gap 1: Durability/HITL

**Current state:** Not addressed.
**Recommendation:** Model as execution backend plugin (M.5 Option 3). LangGraph backend for Python. For Rust, consider Temporal-compatible codegen.
**Timeline:** Phase 4+ (after core language + multi-target emission).

### Gap 2: Schema evolution

**Current state:** Not addressed. ORM lesson (M.4): types and service operations will change.
**Recommendation:** Adopt protobuf-style compatibility rules. Service operations must have stable wire format. Type fields can be added (with defaults) but not removed or retyped without a version bump.
**Timeline:** Phase 2 (when services are introduced).

### Gap 3: "Show me what the compiler meant" tooling

**Current state:** Not addressed. Lombok and ORM lesson: users need to see generated output.
**Recommendation:** First-class CLI commands:

```
dag expand <file.dag>          # show lowered GraphIR (Node/Dag/Port/Edge)
dag show-triplets <file.dag>   # show transport triplet expansion for each service call
dag obligations <file.dag>     # show derived TestObligations by bucket
dag manifest <file.dag>        # show ProgressManifest (waves, groups, scatter points)
dag viz <file.dag>             # ASCII DAG visualization (pre-execution)
```

**Timeline:** Phase 1 (essential for trust and debugging from day one).

### Gap 4: Escape hatch for unusual APIs

**Current state:** Not addressed. ORM lesson: some APIs won't fit `@rest` / `@shell`.
**Recommendation:** A `@custom` transport annotation that tells the compiler "I will implement this transport myself." The compiler still emits the triplet structure (prepare/execute/parse) and generates test obligations, but the execute node delegates to a developer-provided function instead of a generated transport executor.

```
service unusual.Api {
  operation WeirdCall {
    input { payload: Bytes }
    output { result: Json }
    @custom("my_transport_impl")  // developer implements the execute step
  }
}
```

This preserves structural guarantees (the triplet exists, DryRun can intercept it, tests are generated) while allowing escape from the annotation-driven transport generation.

**Timeline:** Phase 2 (when services are introduced).

---

# Appendix N: Model-Based Testing and Auto-Generated Mocks

This appendix describes how the DSL's type system, service declarations, and testgen model combine to enable **model-based testing**: the compiler generates not just test *structure* (which tests to run) but test *data* (what mock values to use), eliminating the manual `MockSpec` fixture burden that accounts for a significant portion of gunbc's per-tool authoring cost.

## N.1 The Current State: MockSpec Is Half-Automated

In gunbc, `extract_mock_requirements()` automatically determines *what needs to be mocked* by analyzing the DAG structure — which nodes are transport boundaries, what output ports they have, what types those ports declare. This is the structural half.

But the developer still supplies the concrete *values*:

```rust
// gunbc today: structure is derived, values are manual
MockSpec::new("gist")
    .boundary("fs_env", "fs:write", mock_fs_handle())          // manual value
    .transport_response("execute_gist", "response",
        TransportResponse::Rest(mock_gist_response_json()))     // manual value
    .boundary_str("parse_gist_response", "url",
        "https://gist.github.com/mock/abc123")                  // manual value
```

For a typical tool with 5 transport nodes, this is 20-40 lines of hand-written mock fixtures. For the full repo (8+ tools), it's hundreds of lines that must be maintained in sync with service APIs.

## N.2 The Goal: Compiler-Generated Mock Values

The DSL should close the gap so the developer workflow becomes:

```
1. Write the .dag file
2. Write the pure transformation logic (5% of code)
3. Stop.
```

The compiler:
- Builds the graph (Parse → Lower)
- Proves structural soundness (Validate)
- Derives test obligations (Derive — already designed)
- **Generates mock values from type constraints and service annotations** (Derive — new)
- Generates test harnesses with those values (Emit — already designed)
- Fuzzes pure nodes with generated inputs (Emit — new)

## N.3 Three Tiers of Auto-Generation

### Tier 1: Type-Driven Generation (Refinement Types)

When a type has refinement constraints, the compiler can generate valid and invalid values mechanically.

```
type CommitSha = String @pattern("^[a-f0-9]{40}$")
type RetryCount = Int @range(min: 1, max: 5)
type GistId = String @format(uuid)
type HttpStatus = Int @range(min: 100, max: 599)
```

The compiler's type-aware generator produces:

| Type | Valid examples | Edge cases | Invalid examples |
|---|---|---|---|
| `CommitSha` | `"a" * 40`, random hex strings | Empty string, 39 chars, 41 chars | `"ZZZZ..."`, non-hex chars |
| `RetryCount` | 1, 3, 5 | 1 (min), 5 (max) | 0, 6, -1, MAX_INT |
| `GistId` | Random UUIDs | Nil UUID (`00000000-...`) | Empty string, malformed |
| `HttpStatus` | 200, 404, 500 | 100 (min), 599 (max) | 99, 600, 0, -1 |

For unconstrained primitives (`String`, `Int`, `Bool`), the compiler uses safe defaults:

| Primitive | Default valid | Default edge cases |
|---|---|---|
| `String` | `"test_value"` | `""`, very long string |
| `Int` | `42` | `0`, `MAX_INT`, `MIN_INT` |
| `Bool` | `true` | `false` |
| `Bytes` | `[0x00]` | `[]`, large buffer |
| `Json` | `{}` | `null`, deeply nested |
| `Secret` | `Secret("mock_secret")` | `Secret("")` |

Records are generated by composing field generators:

```
type AccessToken {
  token: Secret
  scheme: AuthScheme
  expires_at: String?
}

// Compiler generates:
// valid: AccessToken { token: Secret("mock"), scheme: Bearer, expires_at: None }
// valid: AccessToken { token: Secret("mock"), scheme: Bearer, expires_at: Some("2026-01-01") }
// edge:  AccessToken { token: Secret(""), scheme: Header { name: "" }, expires_at: None }
```

### Tier 2: Service-Driven Generation (`@mock_response`)

For transport boundaries, type constraints alone aren't sufficient. A randomly generated JSON string won't parse as a valid GitHub API response. The `@mock_response` annotation provides the semantic template:

```
service github.Gist {
  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output { url: String, id: GistId }
    @rest(POST, "https://api.github.com/gists")
    @mock_response(
      status: 201,
      body: { "html_url": "https://gist.github.com/mock/{id}", "id": "{id}" }
    )
  }
}
```

The compiler:
1. Sees `@mock_response` on the operation
2. Generates a `GistId` value using the refinement type generator (`String @format(uuid)` → `"550e8400-e29b-41d4-a716-446655440000"`)
3. Interpolates into the template: `{ "html_url": "https://gist.github.com/mock/550e8400-...", "id": "550e8400-..." }`
4. Wraps as `TransportResponse::Rest(RestResponse { status: 201, body: ... })`
5. Wires into the generated `MockSpec` for this operation's execute node

The result: **zero hand-written mock fixtures** for operations that have `@mock_response`.

For operations *without* `@mock_response`, the compiler falls back to Tier 1 type-driven generation for the output fields. This works for simple cases (`output { branch: String }` → mock value `"mock_branch"`) but may not produce semantically meaningful responses for complex APIs.

**Design choice:** `@mock_response` is optional. Operations without it still get structurally correct tests (DryRun completion, transport interception) using type-derived fallback values. Operations *with* it get semantically correct tests (the parse node receives a realistic response and can exercise the happy path).

### Tier 3: Property-Based Fuzzing (Pure Nodes)

Pure nodes (prepare, parse, render) are deterministic functions with no side effects. They are ideal targets for property-based fuzzing: generate thousands of valid inputs, verify the node never panics, and optionally verify output invariants.

The compiler can isolate every pure node and fuzz it:

```
// For a pure node:
//   input { diff: String, system: String? }
//   output { messages: List<Message> }

// Compiler generates:
#[test]
fn fuzz_build_review_prompt() {
    use proptest::prelude::*;
    proptest!(|(
        diff in ".*",
        system in proptest::option::of(".*"),
    )| {
        let inputs = inputs! { "diff" => Value::Str(diff), "system" => opt(system) };
        let result = execute_single_node("build_review_prompt", inputs);
        // Property: never panics
        prop_assert!(result.is_ok());
        // Property: output has correct shape
        let outputs = result.unwrap();
        prop_assert!(outputs.contains_key("messages"));
    });
}
```

When the input ports have refinement types, the fuzzer respects them:

```
// input { sha: CommitSha, count: RetryCount }
// CommitSha = String @pattern("^[a-f0-9]{40}$")
// RetryCount = Int @range(min: 1, max: 5)

// Compiler generates:
proptest!(|(
    sha in "[a-f0-9]{40}",          // from @pattern
    count in 1..=5i64,               // from @range
)| {
    // ...
});
```

This closes the gap for **Bucket B (Contract Obligations)**: `NodeContractCompliance` evolves from "does the node produce correct output for one example?" to "does the node produce correct output for *any* valid input?"

## N.4 The Syntactic vs Semantic Fuzzing Boundary

There is a hard line between what auto-fuzzing can and cannot do:

| Tier | What it tests | Fully automated? | Requires |
|---|---|---|---|
| **Tier 1: Type-driven** | Pure nodes don't panic on valid/invalid inputs | Yes | Refinement types on input ports |
| **Tier 2: Service-driven** | Happy-path pipeline with realistic API responses | Yes | `@mock_response` on service operations |
| **Tier 3: Property-based** | Output invariants hold for all valid inputs | Partially | Developer-written output invariants (optional) |

**What auto-fuzzing cannot do:**
- Verify business logic correctness (e.g., "this prompt template produces good reviews") — that requires human judgment
- Generate semantically meaningful API responses without `@mock_response` — a random string won't parse as GitHub JSON
- Test network-level failure modes (timeouts, partial responses, rate limiting) — these require explicit `@error_response` annotations (see N.7)

## N.5 Seed Policy Matrix (Structural vs Semantic Carriers)

The distinction between "type-valid" and "semantically meaningful" test inputs is not just a fuzzing concern — it is a **correctness invariant** that caused a real auth regression in gunbc (see `TODO/TODO_testgen_seed_policy_postmortem.md`).

**The problem:** A generated test seeded a `TransportResponse` input with a shape-valid placeholder (correct type, empty/mock content). The downstream parser expected a real auth token inside the response body. The test passed structurally but produced a semantically wrong execution path.

**Root cause:** The generator conflated structural validity (correct outer type) with semantic validity (meaningful for the operation). For auth/transport carrier types, these are not equivalent.

**The rule (already implemented in gunbc `core/ir/src/types.rs`):**

Every type is classified into one of two seed classes:

| Seed class | Types | Placeholder generation |
|---|---|---|
| `StructuralGeneratable` | Primitives (`String`, `Bool`, `Int`, `Json`), refined primitives (`Url`, `Email`, `FilePath`), common wrappers (`OptionalString`, `StringList`) | Safe — synthetic placeholders produce valid test inputs |
| `SemanticCarrier` | `TransportRequest`, `TransportResponse`, `Credential`, `Secret`, `FilesystemHandle`, `NetworkHandle`, `ToolHandle`, and all unknown types | **Explicit seed required** — synthetic values are shape-valid but semantically wrong |

The policy is **fail-closed**: unknown/new types default to `ExplicitSeedRequired`.

**Seed provenance priority:**

1. **Explicit** — authored mock/example (`@mock_response`, `@node_example`)
2. **Witness** — contract/type-derived (from refinement constraints)
3. **Synthetic fallback** — only for `StructuralGeneratable` types

**Key invariant:** Semantic-carrier inputs are never silently satisfied by synthetic fallback in contexts where behavior correctness is being asserted.

**What this means for the DSL compiler:**

- When generating Bucket B (pure node fuzzing) tests, the compiler classifies each input port's type. `StructuralGeneratable` ports get auto-generated values from refinement constraints. `SemanticCarrier` ports require values from `@mock_response`, `@node_example`, or explicit test fixtures.
- If a semantic-carrier input has no explicit seed source, the compiler emits a **compile error** (not a skipped test, not a synthetic placeholder). This is the "No Skips" policy (N.11) applied at the seed level.
- The `@mock_response` annotation on service operations is the primary mechanism for providing explicit seeds for transport carrier types. Without it, the service's transport boundary tests cannot be generated.

**Target (not yet fully implemented):** A full seed policy matrix keyed by `(type_class, test_context)`:

| | `RealSingleNodeRequiredInput` | `DryRunBoundaryMock` | `LiveFlowInput` |
|---|---|---|---|
| `StructuralGeneratable` | Generated | Generated | Generated |
| `SemanticCarrier` | **Explicit required** | Witness OK | **Explicit required** |

Currently only the `RealSingleNodeRequiredInput` context is implemented. The DSL compiler should implement all three contexts as testgen matures.

## N.6 Integration with Existing Testgen Buckets

The three tiers map to the four testgen buckets:

| Bucket | Current (gunbc) | With auto-generation (DSL) |
|---|---|---|
| **A: Execution Semantics** | DryRun + transport interception. Values from manual MockSpec. | DryRun + transport interception. Values from `@mock_response` or type-driven fallback. **No manual MockSpec needed.** |
| **B: Contract Obligations** | `NodeContractCompliance` with one example input per node. | Property-based fuzzing: thousands of inputs per pure node, crash-freedom and shape-correctness guaranteed. **Bucket B becomes exhaustive.** |
| **C: Scenario Coverage** | All-succeed, per-failure, guard branches. Values from manual MockSpec. | Same scenarios. Values from `@mock_response`. **Per-failure scenarios can also inject `@error_response` templates.** |
| **D: Resource Hygiene** | Resource connectivity, conflict absence. Values from manual MockSpec. | Same checks. Resource mock values generated from `resource` type definitions. **No manual resource MockSpec needed.** |

## N.7 Compiler Pipeline Integration

The auto-generation work happens in two compiler passes:

### Derive phase (existing, extended)

Currently derives `ProgressManifest` and `TestObligations`. Extended to also derive:

```
type MockManifest {
  boundary_mocks: Map<NodeId, Map<PortName, GeneratedValue>>
  transport_mocks: Map<NodeId, GeneratedTransportResponse>
  resource_mocks: Map<ResourceId, GeneratedResourceValue>
  fuzz_targets: List<FuzzTarget>
}

type GeneratedValue {
  source: TypeDriven | MockResponseTemplate | FallbackDefault
  value: Value
  edge_cases: List<Value>
}

type FuzzTarget {
  node_id: NodeId
  input_generators: Map<PortName, Generator>
  output_invariants: List<Invariant>    // from refinement types on output ports
}
```

### Emit phase (existing, extended)

Currently emits type definitions, transport wiring, test harnesses, CLI. Extended to emit:

- `MockSpec` construction from `MockManifest` (replaces hand-written `graph_mock.rs`)
- Property-based test functions for each `FuzzTarget`
- Edge-case test functions for each refined input port

The generated test file gains new sections:

```rust
// === Auto-generated MockSpec (from @mock_response + type generators) ===
fn mock_spec() -> MockSpec {
    MockSpec::new("gist")
        .boundary("fs_env", "fs:write",
            Value::Map(/* generated from Filesystem resource type */))
        .transport_response("execute_gist", "response",
            TransportResponse::Rest(RestResponse {
                status: 201,
                body: json!({"html_url": "https://gist.github.com/mock/550e8400-...", "id": "550e8400-..."}),
            }))
        .boundary("parse_gist_response", "url",
            Value::Str("https://gist.github.com/mock/550e8400-...".into()))
}

// === Property-based fuzz tests (Bucket B, Tier 3) ===
#[test]
fn fuzz_render_snapshot() {
    proptest!(|(
        files in prop::collection::vec((".*", ".*"), 0..20),
    )| {
        let inputs = inputs! { "files" => Value::List(/* ... */) };
        let result = execute_single_node("render_snapshot", inputs);
        prop_assert!(result.is_ok(), "render_snapshot panicked on valid input");
        let outputs = result.unwrap();
        prop_assert!(outputs.contains_key("markdown"), "missing 'markdown' output");
    });
}
```

## N.8 Deterministic Generation for Reproducible Tests

Auto-generated ids, UUIDs, and fuzz inputs must be reproducible across runs and backends to prevent test flakiness.

**Rules:**

- **Stable seed per DAG version**: the default seed is `hash(normalized_ir)` — a hash of the canonicalized IR after all compiler passes. Same source → same seed → same generated values.
- **Deterministic by default in CI**: `dag test` uses the stable seed. No randomness unless explicitly requested.
- **Opt-in randomness locally**: `dag test --fuzz-random` uses a random seed for local exploratory fuzzing. The seed is printed so failures can be reproduced: `dag test --fuzz-seed=0xdeadbeef`.
- **Iteration bounds**: configurable in `dag.toml`:

```toml
[testgen]
fuzz_iterations = 100   # default depth for Tier 3 property tests (proptest)
fuzz_timeout_ms = 5000  # per-property timeout
```

A DAG with 20 pure `fn` nodes at 100 iterations = 2,000 fuzz tests, which is manageable. Default of 100 (not proptest's default 256) balances coverage with CI time.

- **Cross-field invariants**: acknowledged as outside the scope of auto-fuzzing. The compiler generates a note in the test output: `// NOTE: cross-field invariants (e.g., start_date < end_date) require manual test cases or runtime guards`. Authors add these via explicit `@test` annotations on the journey.

## N.9 Error Response Templates (Failure Scenario Mocking)

For Bucket C's per-failure scenarios, the compiler needs to generate realistic error responses. A new `@error_response` annotation provides this:

```
service gcp.SecretManager {
  operation AccessVersion {
    input { project: String, secret: String, version: String = "latest" }
    output { payload: Bytes, name: String }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @mock_response(
      status: 200,
      body: { "payload": { "data": "bW9ja19zZWNyZXQ=" }, "name": "projects/p/secrets/s/versions/1" }
    )
    @error_response(
      status: 404,
      body: { "error": { "code": 404, "message": "Secret not found", "status": "NOT_FOUND" } }
    )
    @error_response(
      status: 403,
      body: { "error": { "code": 403, "message": "Permission denied", "status": "PERMISSION_DENIED" } }
    )
  }
}
```

The compiler uses `@mock_response` for the all-succeed scenario and `@error_response` for per-failure scenarios. When a transport node is the "failing" node in a Bucket C single-failure test, the compiler injects the error response instead of the success response.

Without `@error_response`, the compiler falls back to a generic transport error (connection refused, timeout) — which tests error propagation but not API-specific error handling.

## N.10 The End-State Developer Workflow

### Without auto-generation (gunbc today)

```
1. Define DAG builder               (~200 lines Rust)
2. Write op enum + implementations  (~80 lines)
3. Write MockSpec with manual values (~40 lines, must match API schemas)
4. Register with testgen             (~10 lines)
5. Write pure node logic             (~50 lines)
   Total manual: ~380 lines
   Total generated: ~60 lines (testgen)
   Manual %: ~86%
```

### With auto-generation (DSL target)

```
1. Write .dag file                   (~20 lines)
   - service declarations with @mock_response (if needed)
   - journey with pattern composition
   - refinement types on domain types (if needed)
2. Write pure transformation logic   (~20 lines Rust/Go/Python)
   Total manual: ~40 lines
   Total generated: ~350+ lines (types, transports, MockSpec, tests, CLI, progress)
   Manual %: ~10%
```

The `MockSpec` moves from a per-tool authoring burden to a compiler output.

## N.11 Relationship to gunbc's Existing Simulator Infrastructure

gunbc already has the seeds of this system in `core/test/`:

| Existing infrastructure | How it evolves in the DSL |
|---|---|
| `Simulator { generator, validator }` | Becomes the runtime representation of refinement type generators |
| `IoContract { input: Map<Simulator>, output: Map<Simulator> }` | Compiler-derived from journey port types + refinement constraints |
| `non_empty_string()`, `boolean()`, `exit_code()`, `int_range()` | Become built-in generator presets mapped from `@non_empty`, `@range`, etc. |
| `MockSpec::node_example(NodeExample { inputs, outputs })` | Compiler generates `NodeExample` from refinement types + `@mock_response` |
| `OutputMatcher::Exact`, `Contains`, `NonEmpty` | Compiler generates matchers from output port refinement types |

The DSL compiler doesn't invent new testing infrastructure — it drives the existing `Simulator` / `IoContract` / `MockSpec` / `OutputMatcher` types from declarative metadata instead of manual construction.

## N.12 No-Skips Test Policy (from the-gunbai `mockable-integrations.md`)

If a test is generated, it **must** run. There are no silent skips.

**Rules:**

1. Every transport boundary node must have a `@mock_response` annotation OR the compiler generates a generic transport error mock. Missing mocks for transport boundaries produce a **compile error**, not a skipped test.
2. Generated tests are hermetic — they never require live credentials, network access, or external services.
3. `#[ignore]` / `@skip` is reserved only for explicitly user-marked tests (via `@skip_test` annotation on the journey). The compiler never generates ignored tests.
4. DAG-level failure propagation tests cover every node: for each transport boundary, the compiler generates a scenario where that node fails, verifying that downstream nodes are skipped and no global "success" is reported.
5. If a resource mock is required (e.g., `resource Credential` with `acquire` logic), the compiler derives it from the resource definition's lifecycle specification. Missing resource mocks are compile errors.

This policy ensures that the CI obligation count (e.g., "133 obligations, 58 discharged, 75 testable") is real — every "testable" obligation has a runnable test with no silent gaps.

## N.13 Guardrail Compliance (C1/G1-G3)

Per Appendix K.6:

**G1 (Annotations must desugar to structure):** Refinement annotations desugar to structural predicates. `@pattern("^[a-f0-9]{40}$")` compiles to a `Predicate::Regex` node in the type's DAG representation. `@range(min: 1, max: 5)` compiles to `Predicate::IntRange { min: 1, max: 5 }`. `@mock_response` compiles to a `MockTemplate` in the `MockManifest`. None of these are opaque metadata that survives into the runtime without structural representation.

**G2 (Preserve producer-level semantics):** `@mock_response` and `@error_response` are producer-level annotations that survive through lowering. The compiler preserves them in the `MockManifest` and uses them during the `Emit` phase to generate test fixtures. They are not erased when the service call is lowered to a transport triplet.

**G3 (Kill manual bottlenecks):** The manual MockSpec is one of the three remaining manual bottlenecks (alongside graph builders and registration). Auto-generation eliminates it, reducing the per-tool manual cost from ~380 lines to ~40 lines.

## N.14 Phasing

| Phase | What's automated | Requires |
|---|---|---|
| **Phase 1** (Language Core) | Type-driven fallback values for `MockSpec` (safe defaults for primitives) | Type system (§4.1) |
| **Phase 2** (Services) | `@mock_response` → generated `MockSpec` for transport boundaries | Service declarations (§4.3) |
| **Phase 2** (Services) | `@error_response` → generated failure scenarios for Bucket C | Service declarations (§4.3) |
| **Phase 3** (Composition) | Property-based fuzzing for pure nodes (Bucket B exhaustive) | Refinement types + `execute_single_node` harness |
| **Phase 4** (Multi-target) | Fuzz tests emitted in target language (Rust proptest, Python hypothesis, Go rapid) | Codegen backend trait extended |

The progression is deliberate: Phase 1 eliminates the worst of the manual MockSpec burden (safe defaults). Phase 2 eliminates it entirely for well-annotated services. Phase 3 makes Bucket B exhaustive. Phase 4 makes it multi-language.

---

# Appendix O: Future Considerations (Cross-Repository Review)

This appendix captures features and ideas from the three repos (gunb.ai, the-gunbai, gunbc) that are **not yet incorporated** into the normative DSL spec but should be reviewed before the design is finalized. Each item includes its source, a brief description, and a recommendation.

## O.1 Risk Levels and Envelope Semantics (the-gunbai `spec.md` §1.4)

**What it is:** Every node has a risk level (Low/Medium/High/Critical) that drives policy-driven wrapping. The executor selects an "envelope" (BestEffort, Retryable, ApprovalGate, DryRunFirst, Saga) based on risk + graph policy. Policies are per-environment (CI may accept Medium; production may gate everything above Low).

**Why it matters:** The DSL has `@hermeticity` and `@auth` annotations but no general risk classification. As workflows touch production systems, a risk model prevents "deploy to production" from running without approval gates.

**Recommendation:** Defer to post-Phase 2. Once services and resources are stable, add `@risk(high)` as an annotation that desugars to envelope selection (C1-compliant). The envelope patterns (retry, approval gate, dry-run-first) already exist as DSL patterns — risk classification just selects which one to apply.

## O.2 Caching Model — Lenses, Snapshots, ContextID (the-gunbai `spec.md` §2.3)

**What it is:** A principled caching model where Snapshots represent world-state, Lenses define what you observe, and ContextID = hash(snapshot + lens + params) serves as a cache key. Invalidation is explicit: if lens inputs or logic change, cache is invalidated.

**Why it matters:** The DSL doesn't address caching. As workflows grow, re-executing unchanged subgraphs is wasteful. Deterministic NodeIds (§8.7) and content hashing (`core/infra`) are prerequisites.

**Recommendation:** Defer to post-Phase 3. The DSL's deterministic NodeIds and content hashing infrastructure provide the foundation. A `@cacheable` annotation on journeys (with explicit cache key from input ports) would compose with the freshness model. Key rule: "if you cannot describe what invalidates a result, you must not cache it."

## O.3 Requirements Propagation (the-gunbai `spec.md` §2.4)

**What it is:** A requirement declared by a node propagates to its dependents. Every dependent must acknowledge, exclude, or propagate it. Enforcement levels: Advisory, Warning, Enforced, Deprecated.

**Why it matters:** This enables behavioral contracts to flow from infrastructure to application code. Example: a database service declares `@requires(backup_strategy)` — every journey using it must acknowledge or propagate this requirement.

**Recommendation:** Consider for Phase 4 (pipelines). Requirements are most useful for multi-team workflows where pipeline stages cross ownership boundaries. The DSL's type system handles many requirement-like concerns (e.g., `@auth` is effectively a requirement). A general `@requires` mechanism would complement this.

## O.4 Prerequisites and Auto-Satisfaction (the-gunbai `spec.md` §2.5)

**What it is:** Named conditions that must be true before a node runs. The compiler auto-satisfies prerequisites by finding existing providers, injecting from a registry, or verifying external conditions.

**Why it matters:** The DSL's `resource` construct with `acquire` blocks handles the main case (tool installation, credential acquisition). Prerequisites generalize this: "ensure workspace lock acquired," "ensure schema migration complete."

**Recommendation:** The DSL's resource model covers 80% of prerequisite use cases. The remaining 20% (cross-journey preconditions) could be modeled as `resource` types with `Persistent` lifecycle. Defer general prerequisite syntax unless resource modeling proves insufficient.

## O.5 ExecutionContext — Infrastructure-Derived Requirements (the-gunbai `spec.md` §2.6)

**What it is:** Properties of the execution environment (Preemptible, NetworkPartitionable, DiskEphemeral, CostModel) that automatically derive behavioral requirements. Example: `Preemptible=true` → node must be idempotent and checkpointable.

**Why it matters:** Enables the compiler to insert reliability wrappers (checkpointing, retry, budget tracking) based on where code runs, not what code does.

**Recommendation:** Defer to post-Phase 4. This is a "deployment-time" concern that becomes relevant when the DSL targets multiple execution environments. The DSL's `@hermeticity` annotation is a simple version of this concept.

## O.6 Additional Patterns (the-gunbai `docs/design/behavior-patterns.md`)

Patterns modeled in the-gunbai but not yet in the DSL pattern catalog:

| Pattern | Phases | Use case | Priority |
|---|---|---|---|
| **Watch** | Subscribe → Receive → Unsubscribe | Event streaming, webhooks | Low — requires async/streaming model |
| **Lock** | Acquire → Hold → Release | Mutual exclusion on shared resources | Medium — resource model may suffice |
| **Cache** | Check → Fetch → Store | Read-through caching | Medium — pairs with O.2 |
| **CRUD** | Create, Read, Update, Delete | Resource lifecycle | Low — upsert covers most cases |
| **Saga** | Compensating actions on failure | Distributed transactions | Medium — extends transaction pattern |
| **Circuit Breaker** | Closed → Open → Half-Open | Failure threshold protection | Low — can be modeled as retry variant |

**Recommendation:** Lock and Saga are the highest priority additions. Lock maps to resource conflict detection with time-bounded exclusivity. Saga extends the existing `transaction` pattern with compensation actions. Others can be added as patterns are encountered in real workflows.

## O.7 Demand-Driven Codegen / Consumer Tracking (the-gunbai `docs/design/demand-driven-codegen.md`)

**What it is:** Tracking which downstream targets consume which generated artifacts, with freshness requirements. Enables "regenerate only what changed" instead of "regenerate everything."

**Why it matters:** As the DSL generates more artifacts (13 emission targets per §10.3), full regeneration becomes expensive. Consumer tracking enables incremental compilation.

**Recommendation:** Defer to Phase 3+. The DSL's deterministic compilation (C3) and content hashing provide the foundation. Incremental compilation is an optimization — get correctness first, then speed.

## O.8 the-gunbai Inline DAG Visualization (Implemented, PV1-PV4)

**What it is:** The-gunbai built a complete inline progress visualization system through four phases: compact inline progress (PV2), expanded DAG rendering with box drawing (PV3), and interactive toggle between compact/expanded views (PV4). Design decisions: crossterm for terminal control, hybrid compact/expanded approach, responsive width thresholds, accessibility modes.

**Why it matters:** This is **working code** that should be harvested for the DSL's `inline` and `tui` rendering modes (§6.7). Key files: `gunbai-runtime/src/progress/inline/layout.rs` (wave-based layout), `gunbai-runtime/src/progress/inline/render.rs` (box drawing, edge routing), `gunbai-runtime/src/progress/inline/input.rs` (interactive toggle).

**Recommendation:** Harvest into Phase 1 (§11 "What to Harvest" already lists the-gunbai's terminal crate). This is existing, tested implementation that directly maps to the DSL's `ProgressManifest` → renderer pipeline.

## O.9 gunbc Unified Registration Plan (`docs/design/unified-registration.md`)

**What it is:** gunbc has 6 registration islands (testgen targets, tool definitions, graph builders, boundary mocks, resource defs, graph builder IDs). Only testgen uses the `inventory` auto-discovery pattern; the rest are hardcoded lists.

**Why it matters:** The DSL's filesystem discovery (P7) eliminates ALL registration islands. But during the migration period (when both `.dag` files and Rust builders coexist), the parity harness needs to discover both. The unified registration plan provides the bridge.

**Recommendation:** Relevant only during migration (Workstream A/B from the roadmap). Not a DSL language feature — it's an implementation detail of the parity harness.

## O.10 gunbc Unified Emission Plan (`docs/design/unified-emission.md`)

**What it is:** A 1,400-line audit of gunbc's 13 rendering systems documenting exactly where each one deviates from the "IR → Renderer → Output" gold standard (testgen). Identifies which systems have proper IR, which have renderer traits, and which are raw string concatenation.

**Why it matters:** This is the **map** for implementing §10.3's emission targets. Each entry identifies what IR to create, what renderer trait to define, and what gunbc code to harvest or replace.

**Recommendation:** Use as implementation guide during Phase 1-2. The audit is already done — follow it.

---

**Review checklist:** Before finalizing the DSL spec, revisit each item above and decide: incorporate now, defer with a specific phase target, or explicitly out-of-scope. Items O.1-O.5 are architectural; items O.6-O.10 are implementation guidance.
