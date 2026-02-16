# DSL Design Digest

**What this is:** A concise walkthrough of the gunbc v4 DSL — what it does, how it compiles, and what you get for free at each stage. Intended as a shareable overview for someone who hasn't read the 4,000+ line spec.

**Source material:** `dsl-design.md`, `dsl-roadmap.md`, and the v2/v3 contract documents in `docs/design/v4/`.

---

## 1. The Problem in One Paragraph

gunbc builds workflows as typed DAGs (directed acyclic graphs). Today those DAGs are hand-wired in Rust: 7,000+ lines of builder code, 6 separate registration islands, and per-tool boilerplate for types, tests, progress rendering, and CLI. Adding a new tool costs ~200 lines across 3 files. The DSL replaces all of that with `.dag` files that compile to the same IR, targeting ~20 lines in 1 file per tool, with 100% code generation.

---

## 2. The Language at a Glance

Eight constructs, each with a clear role:

| Construct | Purpose | Example |
|-----------|---------|---------|
| `type` | Data shapes (records, enums, refinements, brands) | `type Port = Int @range(1, 65535)` |
| `fn` | Pure transformations (12 portable constructs, not Turing-complete) | `fn render(r: Registry) -> String { ... }` |
| `resource` | Capabilities with acquire/use/release lifecycle | `resource Filesystem { capability read { ... } }` |
| `service` | External operations with typed I/O and transport annotations | `service gcp.SecretManager { operation AccessVersion { ... } }` |
| `pattern` | Reusable sub-DAG templates with typed slots | `pattern content_upsert { node read ... node write ... }` |
| `func` | Composed workflows — the main authoring surface | `func makegen(...) { ... }` |
| `pipeline` | Staged multi-func workflows | `pipeline ci { stage build { ... } stage test { ... } }` |
| `module` | Namespace, visibility, discovery metadata | `module tools.makegen` |

The `fn` body language is deliberately constrained: `let`, `if/else`, `match`, `for`, `|>`, string interpolation, arithmetic, comparison, boolean logic, field access, record construction, and function calls. That's it. The compiler sees all code, which is what makes proof and test generation possible.

---

## 3. The Compilation Pipeline

Nine passes, each producing a concrete artifact. The key insight: **every pass earns you something** — either a structural proof or generated test material.

```
.dag files (filesystem)
   │
   ▼
┌─────────────┐
│ 1. Discover  │──→ ModuleGraph (all .dag files auto-found)
└──────┬──────┘
       ▼
┌─────────────┐
│ 2. Parse     │──→ AST per file
└──────┬──────┘
       ▼
┌─────────────┐
│ 3. Resolve   │──→ Resolved AST (imports linked, names bound)
└──────┬──────┘
       ▼
┌─────────────┐
│ 4. TypeCheck │──→ Typed AST (expressions typed, coercions inserted)
└──────┬──────┘
       ▼
┌──────────────────┐
│ 5. PatternExpand │──→ PatternIR (patterns → sub-DAG templates,
└──────┬───────────┘    resources → acquire/release nodes)
       ▼
┌─────────────┐
│ 6. Lower     │──→ GraphIR (flat Node/Edge/Port graph)
└──────┬──────┘
       ▼
┌─────────────┐
│ 7. Validate  │──→ Validated GraphIR (invariants checked)
└──────┬──────┘
       ▼
┌─────────────┐
│ 8. Derive    │──→ ProgressManifest + TestObligations + MockSpecs
└──────┬──────┘
       ▼
┌─────────────┐
│ 9. Emit      │──→ Target-language code (Rust, Go, Python, TS)
└─────────────┘
```

---

## 4. What Each Stage Buys You

### Stage 1: Discover
**Input:** Filesystem scan of project directory.
**Output:** `ModuleGraph` — a complete catalog of every `.dag` file.

**What you get for free:**
- **Proof: no registration gaps.** Every `.dag` file is auto-discovered. No manual `all_tools()` vec, no hardcoded lists. If the file exists, the compiler knows about it. This eliminates gunbc's "6 registration islands" problem where dag-viz couldn't even see itself.

### Stage 2: Parse
**Input:** `.dag` source text.
**Output:** Concrete syntax tree (AST) per file.

**What you get for free:**
- **Proof: syntactic well-formedness.** Every construct parses or you get a line-numbered error. No runtime "oops, this builder was malformed."

### Stage 3: Resolve
**Input:** ASTs + ModuleGraph.
**Output:** Resolved AST with all imports linked, all names bound to definitions.

**What you get for free:**
- **Proof: stable identities (C2).** Every `NodeId`, `TypeId`, `ServiceId` is derived from the fully-qualified module path — not build order, not insertion order. This means progress replay is stable across recompilation, and generated code diffs are clean.
- **Proof: no dangling references.** Every `import` resolves to a real module. Every name resolves to a real definition.

### Stage 4: TypeCheck
**Input:** Resolved AST.
**Output:** Typed AST with validated expressions, inserted coercions, checked resource requirements.

**What you get for free:**
- **Proof: type safety.** Port connections are checked: you can't wire a `String` output to an `Int` input. Refinement types (`@range`, `@pattern`) are validated at compile time where possible.
- **Proof: coercion safety.** Safe upcasts (e.g., `Url → String`) are inserted automatically using a three-level coercion lattice. Unsafe direction (narrowing) is rejected.
- **Proof: compatibility (C11).** Adding an optional field with a default is non-breaking. Removing a field is breaking. The compiler can generate a breaking-change report.
- **Generated tests: type-driven boundary values.** From refinement types, the compiler derives valid inhabitants and invalid boundary values. `Port @range(1, 65535)` → valid: `{1, 1000, 65535}`, invalid: `{0, -1, 65536}`. These feed Bucket B tests.

### Stage 5: PatternExpand
**Input:** Typed AST with pattern references.
**Output:** `PatternIR` — patterns expanded to sub-DAG templates, resources to acquire/release nodes.

**What you get for free:**
- **Proof: pattern completeness.** When a `content_upsert` pattern is referenced, the compiler expands it to the full 5-node chain (read → compare → conditional write). You can't forget a step — the pattern defines all of them.
- **Proof: resource lifecycle.** `uses fs: Filesystem(mode: Write)` causes the compiler to insert `fs_env` (acquire) and release nodes automatically, thread resource handles to all consuming nodes, and detect conflicts (e.g., two parallel writes to the same file path = compile error).

### Stage 6: Lower
**Input:** PatternIR.
**Output:** `GraphIR` — flat graph of `Node`, `Edge`, `Port` values. Same structure as gunbc's hand-wired builders.

**What the compiler does here:**
- Service calls → **transport triplets** (prepare → execute → parse)
- `match` → **BranchBuilder** nodes
- `when` → **guarded ports** (skip wiring)
- Implicit ordering → **explicit `after` edges**
- Collection ops (`map`, `filter`, `fold`) → **IR-level nodes** (enabling data-parallel execution)

**What you get for free:**
- **Proof: annotations are structure (C1).** Every annotation (`@rest`, `@idempotent`, `@mock_response`) desugars to an IR field — not opaque metadata. `dag expand` shows you the structural form.
- **Proof: effects are boundary-only (C4).** I/O only happens at `Transport::Execute` nodes. Everything else is pure. This is what makes DryRun interception work.
- **Proof: shell is structured (C6).** Shell commands are `ShellSpec { argv: [...] }`, not strings. No quoting bugs, no injection.
- **Proof: hermeticity is explicit (C8).** Every transport node is tagged `Hermetic` or `External`. The compiler forces the decision for `@shell` (no default).

### Stage 7: Validate
**Input:** GraphIR.
**Output:** Validated GraphIR (or compile errors).

**What you get for free:**
- **Proof: acyclicity.** The graph has no cycles. This is a DAG — guaranteed by construction in the language (no cycles expressible in `.dag` syntax) and verified here.
- **Proof: port saturation.** Every required input port is connected. No "forgot to wire that edge" bugs.
- **Proof: resource conflict absence.** Parallel writes to the same keyed resource are detected and rejected.
- **Proof: bounded repetition (C10).** No unbounded loops. `@retry` requires finite `max`. The language is total (P9).
- **Proof: control edges are explicit (C5).** Ordering dependencies are real edges, not insertion-order accidents.

### Stage 8: Derive
**Input:** Validated GraphIR.
**Output:** Three artifacts: `ProgressManifest`, `TestObligations`, `MockSpecs`.

This is where the compiler extracts *obligations from the graph's structure*:

**ProgressManifest** — the static topology of the workflow:
- Total node count, wave depths, parallel groups
- SubDag boundaries (where funcs call other funcs)
- Scatter points (loop expansion)
- Capture modes per node
- Labels derived from DSL identifiers

All four rendering backends (plain, inline, TUI, JSONL) consume the same manifest. Progress rendering is a view of the graph, not a hand-coded display.

**TestObligations** — the four-bucket model:

| Bucket | What it covers | How obligations are derived |
|--------|---------------|---------------------------|
| **A: Execution Semantics** | Can the workflow run at all? | `DryRunCompletion` for every func. `TransportInterceptable` for every transport node. |
| **B: Contract Obligations** | Do pure nodes produce correct output? | `NodeContractCompliance` for every `fn` node. Property-based fuzzing from refinement types. |
| **C: Scenario Coverage** | What happens when things fail? | `AllTransportsSucceed` + `SingleTransportFailure` per transport node. `GuardBranchCoverage` for every `when`/`match`. |
| **D: Resource Hygiene** | Are resources properly wired? | `TransportResourceDeclared` and `ResourceInputConnected` for every transport node that uses a resource. |

**The anti-tautology rule:** Only generate tests for obligations with status `Unknown` or `RuntimeOnly`. If the compiler can prove an obligation structurally (e.g., "this port is always connected"), it discharges it — no test emitted, because the test would be tautological.

**MockSpecs** — generated from `@mock_response` annotations on service operations. Eliminates the ~380 lines of hand-written MockSpec per tool.

### Stage 9: Emit
**Input:** All derived artifacts + GraphIR.
**Output:** Complete target-language code.

**What gets generated (13 emission targets):**

| # | Target | What it is |
|---|--------|------------|
| 1 | Type definitions | Structs, enums, aliases |
| 2 | Pure functions | `fn` bodies compiled to target language |
| 3 | Transport wiring | HTTP clients, shell exec, file I/O |
| 4 | Orchestration | DAG execution code (topologically scheduled) |
| 5 | Test harness | 4-bucket tests from TestObligations |
| 6 | CLI entrypoint | Argument parsing from entry port types |
| 7 | Progress renderer | Manifest-driven display code |
| 8 | Makefile targets | Per-tool make targets |
| 9 | CI YAML | Pipeline stage definitions |
| 10 | Mock fixtures | From `@mock_response` / `@error_response` |
| 11 | DAG visualization | Static graph rendering data |
| 12 | Content hash manifest | For freshness checking |
| 13 | Obligation report | Discharged/testable counts per bucket |

**Proof: deterministic compilation (C3).** Byte-identical output given identical inputs. CI can run `dag emit --check` to verify no drift.

---

## 5. End-to-End Example: `makegen` (The "Hello World")

### What it does

Generates a `Makefile` from a tool registry. If the content hasn't changed, skip the write.

### The DSL (5 lines of authoring)

```
module tools.makegen

import std.patterns { content_upsert }

func makegen(registry: ToolRegistry) -> { written: Bool } {
  content = render_makefile(registry: registry)
  result = content_upsert(content: content, path: "Makefile")
  return { written: result.written }
}
```

### What the compiler produces

**After PatternExpand:** The `content_upsert` reference expands to a 5-node sub-DAG (read current file → compare → conditional write). The `uses fs: Filesystem` is inferred from the pattern's declaration.

**After Lower:** 8 nodes, 10 edges — identical structure to the hand-wired Rust builder (137 lines + 200 lines of supporting code):

```
Nodes:
  fs_env                    Opaque(FsEnv)               [] → [FilesystemHandle]
  load_registry             Opaque(LoadRegistry)         [] → [ToolRegistry]
  render_makefile           Fn(render_makefile)          [ToolRegistry] → [String]
  prepare_read_makegen      Opaque(PrepareFileRead)      [FilesystemHandle] → [ReadSpec]
  execute_read_makegen      Transport(Execute)           [ReadSpec] → [FileContent]
  compare_makegen_content   Fn(compare_content)          [String, FileContent] → [Bool]
  prepare_write_makegen     Opaque(PrepareFileWrite)     [String, FilesystemHandle] → [WriteSpec]
  execute_makegen_transport Transport(Execute)           [WriteSpec] → [Written]
```

**After Validate:** All invariants hold — acyclic, all ports connected, resource handles threaded, no conflicts.

**After Derive — TestObligations (12 obligations across 4 buckets):**

```
Bucket A (Execution Semantics):
  ✓ DryRunCompletion: full workflow
  ✓ TransportInterceptable: execute_read_makegen
  ✓ TransportInterceptable: execute_makegen_transport

Bucket B (Contract Obligations):
  ✓ NodeContractCompliance: render_makefile

Bucket C (Scenario Coverage):
  ✓ AllTransportsSucceed
  ✓ SingleTransportFailure: execute_read_makegen
  ✓ SingleTransportFailure: execute_makegen_transport
  ✓ GuardBranchCoverage: execute_makegen_transport (skip guard)

Bucket D (Resource Hygiene):
  ✓ TransportResourceDeclared: execute_read_makegen
  ✓ TransportResourceDeclared: execute_makegen_transport
  ✓ ResourceInputConnected: execute_read_makegen.res:file:Makefile
  ✓ ResourceInputConnected: execute_makegen_transport.res:file:Makefile
```

**After Derive — ProgressManifest:**

```
total_nodes: 8
parallel_groups: [{ nodes: ["fs_env", "load_registry"], depth: 0 }]
topology: [fs_env(0), load_registry(0), render_makefile(1),
           prepare_read(1), execute_read(2), compare(3),
           prepare_write(3), execute_write(4)]
```

**Terminal output (inline renderer, from manifest):**

```
makegen ─ 4/4 ━━━━━━━━━━━━━━━━ 100% [✓ load] [✓ render] [✓ compare] [⊘ write]
```

(Write skipped because content was unchanged — the `when !equal.equal` guard fired.)

**After Emit:** Complete Rust code — types, functions, transport wiring, CLI, tests, Makefile target. Zero manual wiring.

### The compression

| Metric | Hand-wired (today) | DSL |
|--------|-------------------|-----|
| Lines of graph builder | 137 | 0 (compiler generates) |
| Lines of ops + traits | 80+ | 0 (compiler generates) |
| Lines of registration | 15 | 0 (auto-discovered) |
| Lines of test setup | 15+ | 0 (compiler generates) |
| **Lines authored** | **~200+ across 3 files** | **~5 in 1 file** |

---

## 6. End-to-End Example: GCP Credential Chain (The Stress Test)

### What it does

Acquires a GCP access token through a multi-step chain: GitHub OIDC → STS token exchange → (optional) service account impersonation → Secret Manager access.

### The DSL (~50 lines across 2 files)

**Service declaration** (`cloud/gcp/secret_manager.dag`):

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

  operation CreateSecret { ... }
  operation AddVersion { ... }
}
```

**Workflow** (`cloud/gcp/credential.dag`):

```
module cloud.gcp.credential

import cloud.gcp.secret_manager
import cloud.gcp.iam
import cloud.gcp.sts
import std.patterns { credential_chain }

func acquire_gcp_secret(
  runtime: CloudRuntime,
  project: String,
  secret_name: String,
  audience: String = "sigstore",
  service_account: String?
) -> { token: AccessToken }
  provides auth: AuthContext
{
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

### What the compiler does

1. Each `service.operation(...)` call expands to a **transport triplet**:
   - `prepare_*` — builds the REST request from `@rest` annotation
   - `execute_*` — transport boundary (the only I/O node)
   - `parse_*` — extracts typed outputs from the response

2. `credential_chain` pattern expands to:
   - `match runtime` → BranchBuilder with 3 arms (GitHub Actions, GCP Metadata, Local)
   - STS exchange → transport triplet
   - `when service_account` → guarded impersonation (optional step)
   - Secret Manager access → transport triplet

3. `Network` resource is inferred from `@rest` calls. The compiler inserts `net_env` and threads it to all execute nodes.

### Test obligations (derived, not written)

```
Bucket A: DryRunCompletion, TransportInterceptable × 4
Bucket B: EdgePredicateEntailment × 2, NodeContractCompliance × 14,
          OptionalInputHandling × 8
Bucket C: AllTransportsSucceed, SingleTransportFailure × 4,
          GuardBranchCoverage × 2
Bucket D: TransportResourceDeclared × 4, ResourceInputConnected × 4
```

### The compression

| Metric | Hand-wired (today) | DSL |
|--------|-------------------|-----|
| Graph builder | 1,688 lines | 0 |
| Ops + service traits | 2,077 + 180 lines | 0 |
| Generated test chars | 157K (from testgen) | Same (from compiler) |
| **Lines authored** | **~4,000+ across 6+ files** | **~50 across 2 files** |

---

## 7. The Type System and Test Generation

Types are themselves DAGs of validation operations. This is what powers automatic test generation.

### How types become tests

A refined type like `Port = Int @range(1, 65535)` compiles to:

```
Dag { Identity("Int") → Validate(InRange(1, 65535)) }
```

The set of valid values: `⟦Port⟧ = { n ∈ ℤ | 1 ≤ n ≤ 65535 }`.
The complement: `⟦Port⟧ᶜ ∩ ⟦Int⟧ = { n ∈ ℤ | n < 1 ∨ n > 65535 }`.

From this the compiler automatically generates:
- **Valid inhabitants:** `{1, 1000, 65535}` (boundary + interior)
- **Invalid boundary values:** `{0, -1, 65536, MAX_INT}`

These feed into Bucket B (Contract Obligations) — every pure `fn` node gets tested with valid and invalid inputs derived mechanically from the type DAG.

### Branded types prevent cross-domain confusion

```
type UserId  = String @brand("UserId")
type TeamId  = String @brand("TeamId")
```

Both are strings, but the `Brand` node makes them structurally incompatible. You can't pass a `TeamId` where a `UserId` is expected — caught at compile time, not runtime.

### Refinement + brand composition

```
type Milliseconds = Int @range(0, ∞) @brand("Milliseconds")
type Seconds      = Int @range(0, ∞) @brand("Seconds")
```

Different brands, different types. The test generator produces distinct value sets for each.

---

## 8. Compiler-Enforced Policies (C1–C11)

These are invariants the compiler proves on every compilation. Each maps to a pass, an artifact, and a "free" check:

| Policy | What it means | Pass | Free check |
|--------|--------------|------|------------|
| **C1** Annotations → structure | No opaque `@trait` magic | Lower | `dag expand` shows structural form |
| **C2** Stable identities | NodeIds from fq paths, not build order | Resolve | Progress replay stability |
| **C3** Deterministic compilation | Same input → byte-identical output | All | `dag emit --check` in CI |
| **C4** Effects boundary-only | I/O only at Execute nodes | Lower + Validate | DryRun interception works |
| **C5** Control edges explicit | No implicit ordering | Resolve + Lower | `dag viz` shows all edges |
| **C6** Shell is structured | `argv` array, no string parsing | Lower | Cross-platform shell tests |
| **C7** REST encoding defined | Canonical URL + JSON serialization | Emit | Mock comparison uses canonical form |
| **C8** Hermeticity explicit | Every transport tagged Hermetic/External | Lower + Validate | Bucket D test categorization |
| **C9** Secrets redacted | `Secret` type never rendered to output | Emit + Runtime | CI rejects `reveal()` |
| **C10** Bounded repetition | No unbounded loops | Validate | Totality check passes |
| **C11** Compatibility rules | Optional field + default = non-breaking | TypeCheck | `dag compat --check` gate |

---

## 9. Summary: What the DSL Earns at Each Level

| Level | What you write | What the compiler proves | What the compiler generates |
|-------|---------------|------------------------|---------------------------|
| **Types** | `type Port = Int @range(1, 65535)` | Set containment, coercion safety, brand disjointness | Valid/invalid boundary values for Bucket B |
| **Functions** | `fn render(r: Registry) -> String { ... }` | Totality (12 constructs, no general recursion), type correctness | Target-language function bodies, property-based fuzz tests |
| **Services** | `service gcp.SM { operation ... @rest ... }` | Transport classification (hermetic/external), permission declarations | Transport triplets, MockSpecs, Bucket A interception tests |
| **Resources** | `resource Filesystem { ... }` | Lifecycle correctness, conflict absence | Acquire/release nodes, Bucket D hygiene tests |
| **Patterns** | `pattern content_upsert { ... }` | Sub-DAG completeness (all nodes + edges present) | Expanded node chains, guard/skip wiring |
| **Funcs** | `func makegen(...) { ... }` | Acyclicity, port saturation, resource threading | Full GraphIR, ProgressManifest, all 4 test buckets, CLI, Makefile |
| **Pipelines** | `pipeline ci { stage ... }` | Stage ordering, inter-stage type compatibility | Stage groups, CI YAML, pipeline-level tests |

---

## 10. How to Read the Full Spec

The design documents form an arc. If you want to go deeper:

1. **`bl1-retrospective.md`** — Why: what went wrong with hand-wired builders
2. **`v2-contracts-design.md`** + **`v2-worked-examples.md`** — The pattern/contract foundation
3. **`v3-contracts-minimal.md`** + **`v3-worked-examples.md`** — The key insight: one recursive type (`Node<T>/Dag<T>`)
4. **`dsl-design.md`** — The full spec (start at §1 and §4 for the language, §9 for the pipeline, appendices A–C for complete examples)
5. **`dsl-roadmap.md`** — How to build it (5 phases, acceptance gates, worker assignments)
