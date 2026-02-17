# gunbc: Design Digest

**What this is:** A complete introduction to gunbc — what it is, why it exists, and how it works. Intended as a shareable document for someone encountering the project for the first time.

**Source material:** The v2/v3/v4 design documents, the BL1 retrospective, the DAG systems overview, testgen philosophy, and the DSL spec in `docs/design/v4/`.

---

## 1. The Core Idea

gunbc is a compiler for workflows. You describe what a tool does — its inputs, outputs, service calls, and resources — and the compiler proves as much as possible correct *before anything runs*.

The animating idea: **push everything you can into compile time.** Every property that the compiler can verify statically is one fewer test to write, one fewer runtime failure to debug, one fewer class of bug that can exist in production. The ideal is that when a `.dag` file compiles, the only remaining questions are about the external world (does the API return what it says it will?) — not about the workflow's own structure.

This is the Guarantee Hierarchy:

| Level | Method | When | Example |
|-------|--------|------|---------|
| **1. Impossible by Structure** | Type system prevents invalid states | Compile | Can't wire `String` output to `Int` input |
| **2. Impossible by Generation** | Code generation only produces valid code | Build | Generated transport code always serializes correctly |
| **3. Validated at Build** | Explicit checks during build/validation | Build | All input ports connected, no resource conflicts |
| **4. Validated at Runtime** | Checks during execution | Run | Transport returns expected status code |
| **5. Tested** | Unit/integration tests | Test | Business logic produces correct output |

**Preference: 1 > 2 > 3 > 4 > 5.** A proof at Level 1 is always better than a test at Level 5. The compiler's job is to move as many guarantees as possible toward Level 1.

---

## 2. Why This Exists: The Arc from Go to Here

### The Go system: executable DAGs with contracts

The original system (`OaaS_v2/pkg/dag` in Go) modeled infrastructure operations as DAGs where every node was executable code — a `func() error` — wrapped in a contract. Each node declared what it required, what it provided, what resources it claimed, and what typed data it exported. A `CompileAndValidate` phase checked the graph before execution: no cycles, contracts satisfied, data flow valid, no resource conflicts.

This worked. Real tools (`make heal`, `make login`, `infra apply`, OaaS triage flows) ran in production using these patterns. The key insight: **because nodes were executable, their contracts constrained real code.** The contract wasn't a description of what the code *might* do — it was a binding declaration of what the code *must* satisfy.

### The Rust V1 system: the gap

When the system was rewritten in Rust, it introduced a modeling layer — "Understandings" — for describing external tools. But it made a critical mistake: behaviors within a tool were a flat list, not a DAG. The system had three natural levels of causal structure:

1. **Tool → Tool** (top level: tool dependencies) — built, working via `depends_on`
2. **Behavior → Behavior** (middle: within-tool causation) — **missing**
3. **Block → Block** (bottom: execution graph) — built, working via GraphIR

The missing middle level meant the system could describe *what* a tool does (check, create, resolve) but not *how those steps relate to each other causally*. Check → Create → Resolve were annotations, not edges. And the epistemology (how we know things about the world) was annotative — you could tag things with epistemic constructs, but the type system didn't enforce them.

The BL1 retrospective distilled this:

> **V1 modeled tool facts; V2 models tool obligations.** The epistemology must be structural, not annotative.

### The V3 insight: one recursive type

The breakthrough was realizing the entire system reduces to one recursive data structure:

```rust
enum NodeBody<T> {
    Opaque(T),           // leaf — we trust it
    SubDag(Dag<T>),      // recursive — same structure inside
}
```

A node is either opaque (you don't look inside) or a sub-DAG (you do, and it's the same structure all the way down). This gives you:

- **Uniformity** — the same language at every level of abstraction
- **Opacity** — a node is "atomic" only because you choose not to look inside
- **Fungibility** — you can replace an opaque node with a sub-DAG (or vice versa) without breaking consumers, because the interface (typed ports) is identical

This is the fractal DAG. It's the same idea at every level: causes (inputs) flow through nodes to effects (outputs). Dependencies are edges. Information flows forward. Everything is a DAG.

### The DSL: making the proofs automatic

The hand-wired Rust builders expressed these DAGs, but at the cost of massive boilerplate: 7,000+ lines of builder code, 6 separate registration islands, per-tool ceremony for types, tests, progress rendering, and CLI. Adding a new tool cost ~200 lines across 3 files.

The DSL replaces all of that with `.dag` files that compile to the same IR. The goal: ~20 lines in 1 file per tool, 100% code generation, and — critically — all the proofs that were implicit in the builder patterns become explicit compiler passes.

---

## 3. Comparisons to Existing Systems

The closest analogues in mainstream programming:

**Rust's type system and borrow checker.** The philosophy is the same: use the type system to prove properties at compile time that other languages check at runtime (or not at all). Rust proves memory safety; gunbc proves workflow safety — that data flows correctly, resources don't conflict, effects are bounded, and every input port is satisfied.

**Protocol Buffers / gRPC IDL.** Like protobuf, `.dag` files declare typed interfaces that generate code in multiple target languages. But protobuf only describes data shapes; gunbc describes *computation* — the DAG of operations, their causal ordering, and the proof obligations that follow from the structure.

**Terraform / Pulumi.** Infrastructure-as-code tools that build dependency graphs and plan execution. gunbc's DAG is similar, but goes further: the type system is richer (refinement types, branded types, resource lifecycles), the compiler proves more properties statically, and the output is general-purpose workflow code — not limited to infrastructure provisioning.

**Java annotation processors / Lombok.** These generate code from annotations, but annotations are opaque metadata — the processor can do anything, and the compiler can't reason about the relationship between annotation and generated code. gunbc's annotations desugar to structure: `@rest(GET, "/path")` becomes a transport triplet with typed inputs and outputs. The compiler sees through every annotation.

**Haskell / ML type systems.** The closest match for the type-level reasoning. Refinement types (`@range`, `@pattern`), branded types (`@brand`), and the coercion lattice are borrowed from dependent/refinement type theory. The key difference: gunbc's `fn` language is deliberately not Turing-complete (12 constructs, no general recursion), which is what makes totality provable and test generation mechanical.

**Build systems (Bazel, Buck2).** Build systems also model computation as DAGs with typed inputs/outputs and deterministic execution. gunbc's compilation pipeline is similar in spirit — deterministic, cacheable, content-addressed. The difference is that gunbc models *runtime* workflows, not build-time dependencies.

**What's novel:** The combination. Taking the type-level reasoning of ML, the code generation of protobuf, the DAG execution of build systems, and the resource modeling of infrastructure tools — and unifying them in a single compiler where each pass earns a specific proof or generates specific test material.

---

## 4. The Language at a Glance

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

## 5. The Compilation Pipeline

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

## 6. What Each Stage Buys You

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

## 7. The "Proof Once" Principle

Traditional approach:
- Developer A writes workflow → writes tests for type safety → writes tests for cardinality → writes tests for acyclicity
- Developer B writes workflow → writes the same tests again
- Developer C writes workflow → writes the same tests again

gunbc approach:
- The system proves type safety, cardinality, acyclicity, and port saturation **once** — in the compiler.
- Developer A writes a workflow → the compiler rejects invalid structure, no tests needed for these properties.
- Developer B writes a workflow → same.
- **Developers write tests for business logic, not structural correctness.**

What's proven statically (no tests generated):

| Property | How it's proven |
|----------|----------------|
| Acyclicity | DAG structure is acyclic by construction |
| Type compatibility | Edge creation enforces matching types |
| Cardinality satisfaction | Compiler checks all ports connect properly |
| Boundary detection | Structural, automatically inferred |
| Resource threading | Compiler inserts acquire/release nodes |

What the compiler generates tests for (can't be proven statically):

| Property | Why it needs tests |
|----------|-------------------|
| Transport correctness | External APIs can return anything |
| Pure function logic | Business logic is domain-specific |
| Failure scenarios | How the workflow behaves when transports fail |
| Guard branch coverage | Runtime values determine which branches execute |

The line between "proven" and "tested" is the line between **structure** (which the compiler controls) and **the external world** (which it doesn't).

---

## 8. The Node Contract

For the graph to be a source of truth — not just a visualization — each opaque node must obey a contract: all input arrives through declared ports, all output leaves through declared ports, no side-channel communication.

The node can do anything internally (it's Turing-complete behind its ports). But its interface to the graph must be honest. Without this contract, the DAG is a diagram, not a reasoning tool.

This is why `fn` is not Turing-complete but opaque nodes can be: `fn` nodes are *inside* the compiler's reasoning boundary. Opaque nodes are *outside* it — the compiler trusts their port declarations but doesn't look inside. The test generation system compensates: every opaque transport node gets interception tests (Bucket A) and failure-scenario tests (Bucket C).

---

## 9. Worked Example: GCP Secret Manager Access

This section walks through one real workflow three ways: the Go you'd write by hand, the `.dag` you'd author instead, and the Go the compiler would emit. The point isn't that one is shorter — it's that the `.dag` and the hand-rolled Go express similar information, but the compiler-emitted Go was produced from a graph IR that encodes structural properties no amount of hand-rolled Go can express.

### 9.1 Types

**Go you'd write:**

```go
type SecretPayload struct {
    Data    string `json:"data"`    // base64-encoded
    Version string `json:"version"`
}

type AccessToken struct {
    Token     string
    ExpiresAt time.Time
    Source    string
}

type CloudRuntime int
const (
    GitHubActions CloudRuntime = iota
    GCPMetadata
    LocalDev
)
```

**.dag you'd author:**

```
type SecretPayload = {
  data: String,
  version: String,
}

type AccessToken = {
  token: Secret,
  expires_at: Timestamp,
  source: String,
}

type CloudRuntime = GitHubActions | GCPMetadata | LocalDev
```

**Go the compiler emits:**

```go
type SecretPayload struct {
    Data    string `json:"data"`
    Version string `json:"version"`
}

type AccessToken struct {
    Token     Secret    `json:"-"`    // Secret: redacted in String(), MarshalJSON()
    ExpiresAt time.Time `json:"expires_at"`
    Source    string    `json:"source"`
}

type CloudRuntime int
const (
    CloudRuntime_GitHubActions CloudRuntime = iota
    CloudRuntime_GCPMetadata
    CloudRuntime_LocalDev
)

func (r CloudRuntime) Validate() error { /* exhaustive check */ }
```

The struct definitions are nearly identical. The differences are structural: `Secret` is a branded type — the emitted Go wraps it so `fmt.Println(token)` prints `[REDACTED]`, and `json.Marshal` omits it. The enum gets a `Validate()` method because the compiler knows the variant set is closed. In hand-rolled Go, you'd either remember to do these things or you wouldn't.

### 9.2 Service client

**Go you'd write:**

```go
type SecretManagerClient struct {
    httpClient *http.Client
    baseURL    string
    token      string
}

func (c *SecretManagerClient) AccessVersion(
    ctx context.Context, project, secret, version string,
) (*SecretPayload, error) {
    url := fmt.Sprintf(
        "%s/v1/projects/%s/secrets/%s/versions/%s:access",
        c.baseURL, project, secret, version,
    )
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return nil, err
    }
    req.Header.Set("Authorization", "Bearer "+c.token)

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != 200 {
        body, _ := io.ReadAll(resp.Body)
        return nil, fmt.Errorf("secret manager: %d: %s", resp.StatusCode, body)
    }

    var payload SecretPayload
    if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
        return nil, err
    }
    return &payload, nil
}
```

This is ~30 lines. It builds a URL, sets auth, makes the request, checks the status, decodes JSON. Standard Go.

**.dag you'd author:**

```
service gcp.SecretManager {
  base_url: "https://secretmanager.googleapis.com"

  operation AccessVersion {
    input  { project: String, secret: String, version: String = "latest" }
    output SecretPayload
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @idempotent @readonly
    @auth(bearer)
    @mock_response { data: "c2VjcmV0", version: "1" }
    @error_response(403, { error: { code: 403, message: "permission denied" } })
  }
}
```

This is the same information — method, path template, input/output types, auth scheme — declared rather than implemented. `@mock_response` and `@error_response` define the test fixtures inline.

**Go the compiler emits** (three functions from the transport triplet):

```go
// prepare: build the request (pure — no I/O)
func prepareAccessVersion(
    project, secret, version string, cred Credential,
) (*http.Request, error) {
    url := fmt.Sprintf(
        "https://secretmanager.googleapis.com/v1/projects/%s/secrets/%s/versions/%s:access",
        url.PathEscape(project),
        url.PathEscape(secret),
        url.PathEscape(version),
    )
    req, err := http.NewRequestWithContext(context.Background(), "GET", url, nil)
    if err != nil {
        return nil, err
    }
    cred.Apply(req) // AuthScheme.Bearer → sets Authorization header
    return req, nil
}

// execute: the only function that does I/O — the transport boundary
func executeTransport(client *http.Client, req *http.Request) (*http.Response, error) {
    return client.Do(req)
}

// parse: extract typed output from response (pure — no I/O)
func parseAccessVersion(resp *http.Response) (*SecretPayload, error) {
    defer resp.Body.Close()
    if resp.StatusCode != 200 {
        body, _ := io.ReadAll(resp.Body)
        return nil, &TransportError{Status: resp.StatusCode, Body: body}
    }
    var payload SecretPayload
    if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
        return nil, err
    }
    return &payload, nil
}
```

The emitted code reads like the hand-rolled version — same `http.NewRequestWithContext`, same `json.NewDecoder`, same status check. But the compiler split it into three functions because in the graph IR, `prepare` and `parse` are pure nodes and `execute` is the sole transport boundary. This split is what makes DryRun interception work: you can substitute `executeTransport` with a mock that returns the `@mock_response` fixture, and the prepare/parse logic still runs for real.

The hand-rolled Go buries the I/O in the middle of business logic. The emitted Go separates it structurally. Both *work*. But only the emitted version can be intercepted, mocked, and tested without rewriting the function.

Also: `url.PathEscape`. The hand-rolled Go uses `%s` and hopes the inputs are safe. The compiler knows the `{project}` interpolation is a URL path segment (from the `@rest` annotation's structured template) and escapes accordingly.

### 9.3 The workflow

**Go you'd write:**

```go
func AcquireGCPSecret(
    ctx context.Context,
    httpClient *http.Client,
    runtime CloudRuntime,
    project string,
    secretName string,
    audience string,         // caller must remember default
    serviceAccount *string,  // nil = skip impersonation
) (*AccessToken, error) {

    // Step 1: OIDC token
    var subjectToken string
    var err error
    switch runtime {
    case GitHubActions:
        subjectToken, err = getGitHubOIDC(ctx, httpClient, audience)
    case GCPMetadata:
        subjectToken, err = getMetadataOIDC(ctx, httpClient)
    case LocalDev:
        subjectToken, err = getLocalADC(ctx)
    }
    if err != nil {
        return nil, fmt.Errorf("oidc: %w", err)
    }

    // Step 2: STS exchange
    stsToken, err := exchangeSTS(ctx, httpClient, audience, subjectToken)
    if err != nil {
        return nil, fmt.Errorf("sts: %w", err)
    }

    // Step 3: Optional impersonation
    accessToken := stsToken
    if serviceAccount != nil {
        accessToken, err = impersonate(ctx, httpClient, stsToken, *serviceAccount)
        if err != nil {
            return nil, fmt.Errorf("impersonate: %w", err)
        }
    }

    // Step 4: Access secret
    sm := &SecretManagerClient{
        httpClient: httpClient, baseURL: smBaseURL, token: accessToken,
    }
    payload, err := sm.AccessVersion(ctx, project, secretName, "latest")
    if err != nil {
        return nil, fmt.Errorf("secret: %w", err)
    }

    decoded, err := base64.StdEncoding.DecodeString(payload.Data)
    if err != nil {
        return nil, fmt.Errorf("decode: %w", err)
    }

    return &AccessToken{Token: string(decoded), Source: "gcp-sm"}, nil
}
```

This is readable, sequential Go. About 50 lines. It handles the branching, the optional step, the error wrapping.

**.dag you'd author:**

```
module cloud.gcp.credential

import cloud.gcp.secret_manager { SecretManager }
import cloud.gcp.iam { IamCredentials }
import cloud.gcp.sts { SecurityTokenService }

func acquire_gcp_secret(
  runtime: CloudRuntime,
  project: String,
  secret_name: String,
  audience: String = "sigstore",
  service_account: String?
) -> { token: AccessToken }
{
  // Step 1: OIDC token (runtime-dependent)
  subject_token = match runtime {
    GitHubActions -> github_oidc(audience: audience)
    GCPMetadata   -> metadata_oidc()
    LocalDev      -> local_adc()
  }

  // Step 2: STS exchange
  sts_result = SecurityTokenService.Exchange(
    audience: audience,
    subject_token: subject_token.token
  )

  // Step 3: Optional impersonation
  access_token = when service_account {
    some(sa) -> IamCredentials.GenerateAccessToken(
      service_account: sa,
      delegates: [],
      scope: ["https://www.googleapis.com/auth/cloud-platform"]
    ).access_token
    none -> sts_result.access_token
  }

  // Step 4: Access secret
  secret = SecretManager.AccessVersion(
    project: project,
    secret: secret_name
  )

  return { token: build_token(secret: secret.data, source: "gcp-sm") }
}
```

Structurally the same four steps. The `match` is the switch. The `when service_account` is the nil check. The service calls are the client methods. It reads like pseudocode for the Go version, because it *is* — both describe the same causal chain.

**Go the compiler emits:**

The emitted Go is structurally similar to the hand-rolled version — a function with a switch, an optional step, sequential service calls. But it was produced by flattening a graph IR with 20+ nodes and 30+ edges. What the IR encoded that the hand-rolled Go doesn't:

```
Nodes:
  match_runtime           BranchBuilder         [CloudRuntime] → [SubjectToken]
    arm github_actions:   SubDag(3 nodes)       [] → [SubjectToken]
    arm gcp_metadata:     SubDag(3 nodes)       [] → [SubjectToken]
    arm local_dev:        SubDag(3 nodes)       [] → [SubjectToken]
  prepare_sts             Opaque(PrepareSTS)    [String, String] → [TransportRequest]
  execute_sts             Transport(Execute)    [TransportRequest, res:network(Read)] → [TransportResponse]
  parse_sts               Opaque(ParseSTS)      [TransportResponse] → [String, Int]
  guard_impersonate       Guard(NotEq(nil))     [String?] → [String]      // cardinality: ZeroOrOne → One
  prepare_impersonate     Opaque(PrepareImpersonate)  [String, String] → [TransportRequest]
  execute_impersonate     Transport(Execute)    [TransportRequest, res:network(Read)] → [TransportResponse]
  parse_impersonate       Opaque(ParseImpersonate)    [TransportResponse] → [String]
  merge_token             Merge                 [String?, String] → [String]  // guard path + fallthrough
  prepare_secret          Opaque(PrepareAccess) [String, String, String] → [TransportRequest]
  execute_secret          Transport(Execute)    [TransportRequest, res:network(Read)] → [TransportResponse]
  parse_secret            Opaque(ParseAccess)   [TransportResponse] → [SecretPayload]
  build_credential        Fn(build_token)       [String, String] → [AccessToken]
```

Every edge carries a type and a cardinality. Every transport node declares its resource access mode. Every guard narrows a cardinality. The emitted Go flattens this to sequential code — but the proofs happened at the IR level before emission.

### 9.4 Tests

**Go you'd write:**

```go
func TestAccessVersion_Success(t *testing.T) {
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        if r.Method != "GET" {
            t.Errorf("want GET, got %s", r.Method)
        }
        if !strings.HasSuffix(r.URL.Path, "/versions/latest:access") {
            t.Errorf("wrong path: %s", r.URL.Path)
        }
        if r.Header.Get("Authorization") == "" {
            t.Errorf("missing auth header")
        }
        json.NewEncoder(w).Encode(map[string]string{
            "data": "c2VjcmV0", "version": "1",
        })
    }))
    defer server.Close()

    client := &SecretManagerClient{
        httpClient: server.Client(), baseURL: server.URL, token: "test-token",
    }
    payload, err := client.AccessVersion(context.Background(), "proj", "sec", "latest")
    if err != nil {
        t.Fatal(err)
    }
    if payload.Data != "c2VjcmV0" {
        t.Errorf("want c2VjcmV0, got %s", payload.Data)
    }
}

func TestAccessVersion_Unauthorized(t *testing.T) {
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        w.WriteHeader(403)
        json.NewEncoder(w).Encode(map[string]interface{}{
            "error": map[string]interface{}{"code": 403, "message": "permission denied"},
        })
    }))
    defer server.Close()
    // ... assert error ...
}

func TestAcquireGCPSecret_SkipImpersonation(t *testing.T) {
    // You'd need to mock 3 HTTP calls (OIDC, STS, SecretManager)
    // and verify impersonation was NOT called.
    // This is ~50 lines of mock setup.
}

func TestAcquireGCPSecret_WithImpersonation(t *testing.T) {
    // Same, but mock 4 HTTP calls and verify impersonation WAS called.
    // Another ~60 lines.
}

// Tests you probably forget to write:
// - What if STS returns 200 but missing access_token field?
// - What if OIDC succeeds but STS fails — is the error message clear?
// - What if impersonation is requested but serviceAccount is empty string (not nil)?
// - Are all three runtime branches actually tested?
```

You write each test by hand. You decide which scenarios to cover. The failure scenarios you forget to write are the ones that bite you in production.

**.dag you'd author:**

Nothing. The `@mock_response` and `@error_response` on the service operations are the fixtures. The compiler derives the test obligations from the graph structure.

**Go the compiler emits:**

See Appendix A for the full generated test file. The key point: the compiler produces tests for every obligation it can't discharge statically, using the mock fixtures declared on the service operations. It doesn't guess which scenarios matter — it derives them from the graph's topology, cardinalities, guards, and transport boundaries.

---

## 10. What the Graph IR Encodes

The emitted Go from §9 looks like normal code. What makes it different is what it was produced *from*. The graph IR carries structural metadata that hand-rolled Go simply cannot express, and the compiler uses that metadata to prove properties and generate tests before any code is emitted.

### Cardinality as proof

In Go, an optional value is `*string`. The compiler doesn't track whether you've nil-checked it. If you dereference a nil `*string`, you get a runtime panic.

In the `.dag` IR, every port has a cardinality:

```
Cardinality { min: u32, max: Option<u32> }

ONE          = [1, 1]     // exactly one value (scalar)
ZERO_OR_ONE  = [0, 1]     // optional
ZERO_OR_MORE = [0, ∞)     // list
ONE_OR_MORE  = [1, ∞)     // non-empty list
```

Edge creation checks that the source cardinality *satisfies* the target requirement — `output [min,max] ⊆ input [min,max]`. Concretely:

- `service_account: String?` has cardinality `ZeroOrOne` on its port.
- The `prepare_impersonate` node requires `service_account: String` with cardinality `One`.
- Wiring `ZeroOrOne` directly to `One` is a compile error: `CardinalityMismatch { output: [0,1], input: [1,1] }`.
- The `when service_account` guard narrows the cardinality: inside the guarded branch, the port is `One` — provably non-nil.
- The `merge_token` node after the guard accepts both the guarded path (`One`) and the fallthrough (`One` from `sts_result`), producing `One`.

This is the nil-safety that Go's type system can't express. The cardinality lattice has `join` (least upper bound), `meet` (greatest lower bound), and `product` (for nested iteration). These operations let the compiler reason about collection shapes algebraically:

- If a `map` node iterates over a `ZeroOrMore` input, the output is also `ZeroOrMore`.
- If a `filter` narrows `OneOrMore` input, the output is `ZeroOrMore` (the filter might eliminate everything).
- If two `ZeroOrOne` inputs feed a `join`, the output is `ZeroOrMore` with max 2 — not unbounded.

Every one of these is checked at edge creation. In Go, you'd discover the mismatch at runtime — or not at all.

### Guard narrowing (predicate entailment)

Guards in the IR are explicit values on ports:

```rust
enum Guard {
    Eq(Value),       // proceed only if value equals expected
    NotEq(Value),    // proceed only if value does not equal expected
}
```

When the `when service_account` guard fires:
- The `guard_impersonate` node receives `service_account: String?` (cardinality `ZeroOrOne`).
- The guard `NotEq(nil)` filters: if nil, the node doesn't execute and its output port produces nothing.
- If non-nil, the output port has cardinality `One` — the guard *proved* presence.
- Downstream nodes receive a value that provably satisfies `!= nil`. No nil check needed.
- The compiler generates `GuardBranchCoverage` tests for both paths (taken and not-taken).

In Go, the equivalent is `if serviceAccount != nil { ... }`. Inside the branch, you *know* it's non-nil — but the compiler doesn't track this. Nothing prevents you from accidentally passing the outer (still-nullable) variable to a function inside the branch. The IR's guard narrowing makes the proof explicit and propagates it through the graph.

### Resource conflict detection

Every transport node in the IR carries resource ports with access modes:

```rust
enum AccessMode {
    Read,       // concurrent reads OK
    Write,      // conflicts with other writes and reads
    Exclusive,  // conflicts with any other access
}
```

The `execute_sts`, `execute_impersonate`, and `execute_secret` nodes all have `res:network(Read)`. Since `Read + Read` doesn't conflict, the compiler knows these could safely run in parallel if the data dependencies allowed it.

If two nodes in the same parallel wave both declared `res:filesystem(Write)` to the same path, the compiler would reject the graph at construction time: `ResourceConflict { resource: "file:Makefile", node_a: "write_config", node_b: "write_manifest", mode_a: Write, mode_b: Write }`.

In Go, two goroutines writing to the same file is a race condition discovered (if you're lucky) by `go test -race` at runtime. In the IR, it's a compile error.

### Transport boundary classification

Every transport node is tagged `Hermetic` (local filesystem, in-process) or `External` (network API, shell command). The compiler forces this classification — there is no default.

This is what makes DryRun work mechanically: intercept every `External` transport node, substitute the `@mock_response`, let everything else run. The hand-rolled Go version would need you to manually identify which calls are "real" I/O and inject mocks at the right points. The IR knows exactly where the boundaries are because the DSL declared them.

---

## 11. Common Behaviors at the DSL Level vs Opaque Helpers

Many workflows share common patterns — upsert a resource, chain credentials, retry on failure. In hand-rolled Go, you'd extract these into helper functions. In the DSL, you express them as `pattern` declarations. The difference is what the compiler can see and prove.

### Content upsert: helper function vs pattern

**Go helper** (opaque to the caller):

```go
// UpsertFile reads the existing file, compares content, writes only if changed.
func UpsertFile(ctx context.Context, path string, content []byte) (bool, error) {
    existing, err := os.ReadFile(path)
    if err != nil && !os.IsNotExist(err) {
        return false, err
    }
    if bytes.Equal(existing, content) {
        return false, nil // no change
    }
    if err := os.WriteFile(path, content, 0644); err != nil {
        return false, err
    }
    return true, nil
}
```

This works. But to the caller, it's a black box. You can't see:
- That it reads a file (I/O boundary)
- That it conditionally writes (guard behavior)
- That it uses the filesystem (resource claim)
- Whether the read and write conflict with other filesystem operations in the same workflow

Tests for `UpsertFile` are hand-written: you test "file doesn't exist", "file exists same content", "file exists different content", "write fails". You write these once, and hope every caller exercises the helper correctly in context.

**.dag pattern** (transparent to the compiler):

```
pattern content_upsert {
  input  { content: String, path: String }
  output { written: Bool }
  uses fs: Filesystem(mode: ReadWrite)

  node read_existing = fs.read(path: path)
  node compare = eq(a: content, b: read_existing.content)
  node write = when !compare.equal {
    fs.write(path: path, content: content)
  }
  return { written: !compare.equal }
}
```

Same logic. But the compiler sees through it:

| Property | Go helper | .dag pattern |
|----------|-----------|-------------|
| **I/O boundaries identified** | No — caller doesn't know `UpsertFile` does I/O | Yes — `fs.read` and `fs.write` are transport nodes in the expanded graph |
| **Guard behavior visible** | No — the `bytes.Equal` check is internal | Yes — `when !compare.equal` is an explicit guard node with `GuardBranchCoverage` obligation |
| **Resource claim declared** | No — caller doesn't know filesystem is used | Yes — `uses fs: Filesystem(mode: ReadWrite)` is a declared resource; conflicts with parallel filesystem ops are detected |
| **DryRun works** | Not without rewriting `UpsertFile` to accept an `fs` interface | Yes — transport nodes are intercepted automatically |
| **Tests generated** | No — you write them by hand for the helper | Yes — the compiler derives `TransportInterceptable` (read + write), `GuardBranchCoverage` (skip vs write), `ResourceInputConnected` |

When the pattern is used inside a larger workflow (like `makegen`), the expanded sub-DAG merges into the parent graph. The resource claim composes: if `makegen` also uses `Filesystem(Read)` elsewhere, the compiler checks that the combined accesses don't conflict. The Go helper gives you no such composition — each call to `UpsertFile` is independent, and conflicts between helpers are invisible.

### Credential chain: helper function vs pattern

**Go helper:**

```go
func AcquireCredential(ctx context.Context, opts CredentialOpts) (*Credential, error) {
    // 50 lines of branching, optional impersonation, token exchange
    // Caller sees: func in, credential out
}
```

The caller can't tell:
- How many HTTP calls this makes (3 or 4, depending on impersonation)
- Which ones are idempotent (safe to retry) vs which aren't
- What permissions are required
- Whether the credential will be valid when it's used (expiry is internal)

**.dag pattern:**

The `credential_chain` pattern declares all of this in the type system. The compiler expands it to a sub-DAG where:
- Each HTTP call is a transport triplet with `@idempotent` or not
- The optional impersonation is a guarded sub-graph with explicit cardinality narrowing
- Permissions are declared (`@permissions`) and can be audited statically
- Token expiry flows through typed ports and is visible to downstream consumers

### Retry: language construct vs library wrapper

**Go:**

```go
func withRetry(ctx context.Context, maxAttempts int, fn func() error) error {
    for i := 0; i < maxAttempts; i++ {
        if err := fn(); err == nil {
            return nil
        }
        time.Sleep(backoff(i))
    }
    return fmt.Errorf("exhausted %d retries", maxAttempts)
}
```

This is fine. But `maxAttempts` is a runtime value — nothing prevents `withRetry(ctx, math.MaxInt, fn)`. And the compiler can't prove the wrapped function is safe to retry (idempotent).

**.dag:**

```
node access = SecretManager.AccessVersion(...) @retry(max: 3, backoff: exponential)
```

The compiler checks:
- `max: 3` is a compile-time constant — bounded repetition is guaranteed (C10)
- `AccessVersion` is marked `@idempotent` — retry is semantically safe
- If you put `@retry` on a non-idempotent operation, the compiler warns (or errors, depending on policy)

The retry is a graph-level construct: the IR represents it as bounded repetition with a finite `max`, not an unbounded loop. The totality checker (P9) can prove the workflow terminates.

---

## 12. Compiler-Enforced Policies (C1–C11)

Invariants the compiler proves on every compilation:

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

## 13. Summary: What the DSL Earns at Each Level

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

## 14. How to Read the Full Spec

The design documents form an arc:

1. **`bl1-retrospective.md`** — Why: what went wrong with hand-wired builders, the "half-built DAG" problem
2. **`dag-systems-overview.md`** — The Go-era reference system: executable DAGs with contracts
3. **`v2-contracts-design.md`** + **`v2-worked-examples.md`** — The pattern/contract foundation
4. **`v3-contracts-minimal.md`** + **`v3-worked-examples.md`** — The key insight: one recursive type (`Node<T>/Dag<T>`)
5. **`dsl-design.md`** — The full spec (start at §1 and §4 for the language, §9 for the pipeline, appendices A–C for complete examples)
6. **`dsl-roadmap.md`** — How to build it (5 phases, acceptance gates, worker assignments)
7. **`overview.md`** (in `docs/design/`) — The Guarantee Hierarchy, the formal model, the Erasure Lemma
8. **`testgen.md`** (in `docs/design/`) — Proof obligations, test generation philosophy, the anti-tautology rule

---

## Appendix A: Hypothetical Generated Tests

Given the `.dag` definitions from §9 — including the `@mock_response` and `@error_response` fixtures on the service operations — here is what the compiler would emit as Go test code. These are derived mechanically from the graph's topology, cardinalities, guards, and transport boundaries.

### Mock fixtures (from service annotations)

```go
var mockAccessVersionResponse = SecretPayload{
    Data:    "c2VjcmV0",
    Version: "1",
}

var mockAccessVersionError403 = TransportError{
    Status: 403,
    Body:   []byte(`{"error":{"code":403,"message":"permission denied"}}`),
}

var mockSTSResponse = STSTokenResponse{
    AccessToken: "ya29.mock-sts-token",
    ExpiresIn:   3600,
}

var mockImpersonateResponse = ImpersonateResponse{
    AccessToken: "ya29.mock-impersonated-token",
    ExpireTime:  "2025-01-01T00:00:00Z",
}

var mockGitHubOIDCResponse = OIDCResponse{
    Value: "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.mock-subject-token",
}
```

### Bucket A: Execution semantics

```go
// Can the full workflow execute end-to-end with all transports mocked?
// Derived from: every func gets a DryRunCompletion obligation.
func TestDryRunCompletion_AcquireGCPSecret(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)
    // Note: execute_impersonate not mocked — service_account is nil,
    // so the guard skips the impersonation branch.

    result, err := RunWithMocks(AcquireGCPSecret, AcquireGCPSecretInput{
        Runtime:    GitHubActions,
        Project:    "my-project",
        SecretName: "my-secret",
        Audience:   "sigstore",
        // ServiceAccount: nil — tests the skip path
    }, mocks)

    if err != nil {
        t.Fatalf("dry run failed: %v", err)
    }
    if result.Token.Source != "gcp-sm" {
        t.Errorf("unexpected source: %s", result.Token.Source)
    }
    mocks.AssertAllConsumed(t)
}

// Can each transport node be individually intercepted?
// Derived from: every Transport(Execute) node gets a TransportInterceptable obligation.
func TestTransportInterceptable_ExecuteSTS(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    intercepted := mocks.Intercept("execute_sts")

    _, err := RunWithMocks(AcquireGCPSecret, defaultInput(), mocks)
    if err != nil {
        t.Fatal(err)
    }

    // Verify the STS request was well-formed
    req := intercepted.CapturedRequest()
    if req.Method != "POST" {
        t.Errorf("STS should be POST, got %s", req.Method)
    }
    if req.URL.Host != "sts.googleapis.com" {
        t.Errorf("wrong STS host: %s", req.URL.Host)
    }
}
```

### Bucket B: Contract obligations

```go
// Does the parse node extract the right fields from a valid response?
// Derived from: every Fn/Opaque node with typed outputs gets NodeContractCompliance.
func TestNodeContract_ParseAccessVersion(t *testing.T) {
    resp := httpResponse(200, `{"data":"c2VjcmV0","version":"1"}`)
    payload, err := parseAccessVersion(resp)
    if err != nil {
        t.Fatal(err)
    }
    if payload.Data != "c2VjcmV0" {
        t.Errorf("data: want c2VjcmV0, got %s", payload.Data)
    }
    if payload.Version != "1" {
        t.Errorf("version: want 1, got %s", payload.Version)
    }
}

// Does the parse node reject a response with missing required fields?
// Derived from: output port cardinality is ONE — a missing field violates the contract.
func TestNodeContract_ParseAccessVersion_MissingData(t *testing.T) {
    resp := httpResponse(200, `{"version":"1"}`)
    _, err := parseAccessVersion(resp)
    if err == nil {
        t.Fatal("expected error for missing 'data' field")
    }
}

// Does the optional input flow correctly when absent?
// Derived from: service_account has cardinality ZeroOrOne.
// The compiler generates a test for each cardinality boundary.
func TestOptionalInput_ServiceAccount_Absent(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    // No mock for execute_impersonate — it must NOT be called
    mocks.On("execute_secret", mockAccessVersionResponse)

    _, err := RunWithMocks(AcquireGCPSecret, AcquireGCPSecretInput{
        Runtime:    GitHubActions,
        Project:    "my-project",
        SecretName: "my-secret",
        // ServiceAccount: nil
    }, mocks)

    if err != nil {
        t.Fatal(err)
    }
    mocks.AssertNotCalled(t, "execute_impersonate")
}

func TestOptionalInput_ServiceAccount_Present(t *testing.T) {
    sa := "deploy@project.iam.gserviceaccount.com"
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_impersonate", mockImpersonateResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    _, err := RunWithMocks(AcquireGCPSecret, AcquireGCPSecretInput{
        Runtime:        GitHubActions,
        Project:        "my-project",
        SecretName:     "my-secret",
        ServiceAccount: &sa,
    }, mocks)

    if err != nil {
        t.Fatal(err)
    }
    mocks.AssertCalled(t, "execute_impersonate")
}
```

### Bucket C: Scenario coverage

```go
// Happy path: all transports succeed.
// Derived from: every func with transport nodes gets AllTransportsSucceed.
func TestAllTransportsSucceed_AcquireGCPSecret(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    result, err := RunWithMocks(AcquireGCPSecret, defaultInput(), mocks)
    if err != nil {
        t.Fatal(err)
    }
    if result.Token.Source != "gcp-sm" {
        t.Errorf("unexpected source: %s", result.Token.Source)
    }
}

// What happens when a single transport fails?
// Derived from: every Transport(Execute) node gets SingleTransportFailure.
// One test per transport node — the compiler enumerates them.
func TestSingleTransportFailure_ExecuteSTS(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.OnError("execute_sts", &TransportError{Status: 500, Body: []byte("internal error")})
    // execute_secret not mocked — should never be reached

    _, err := RunWithMocks(AcquireGCPSecret, defaultInput(), mocks)
    if err == nil {
        t.Fatal("expected error when STS fails")
    }
    var te *TransportError
    if !errors.As(err, &te) {
        t.Fatalf("expected TransportError, got %T", err)
    }
    if te.Status != 500 {
        t.Errorf("want status 500, got %d", te.Status)
    }
    mocks.AssertNotCalled(t, "execute_secret") // downstream not reached
}

func TestSingleTransportFailure_ExecuteSecret(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.OnError("execute_secret", mockAccessVersionError403)

    _, err := RunWithMocks(AcquireGCPSecret, defaultInput(), mocks)
    if err == nil {
        t.Fatal("expected error when secret access returns 403")
    }
    var te *TransportError
    if !errors.As(err, &te) {
        t.Fatalf("expected TransportError, got %T", err)
    }
    if te.Status != 403 {
        t.Errorf("want status 403, got %d", te.Status)
    }
}

// Are both guard branches exercised?
// Derived from: every Guard node gets GuardBranchCoverage.
func TestGuardBranch_Impersonation_Taken(t *testing.T) {
    sa := "deploy@project.iam.gserviceaccount.com"
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_impersonate", mockImpersonateResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    intercepted := mocks.Intercept("execute_secret")

    _, err := RunWithMocks(AcquireGCPSecret, AcquireGCPSecretInput{
        Runtime:        GitHubActions,
        Project:        "my-project",
        SecretName:     "my-secret",
        ServiceAccount: &sa,
    }, mocks)
    if err != nil {
        t.Fatal(err)
    }

    // When impersonation is taken, the secret access should use
    // the impersonated token, not the STS token.
    req := intercepted.CapturedRequest()
    auth := req.Header.Get("Authorization")
    if auth != "Bearer ya29.mock-impersonated-token" {
        t.Errorf("expected impersonated token, got: %s", auth)
    }
}

func TestGuardBranch_Impersonation_Skipped(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    intercepted := mocks.Intercept("execute_secret")

    _, err := RunWithMocks(AcquireGCPSecret, defaultInput(), mocks)
    if err != nil {
        t.Fatal(err)
    }

    // When impersonation is skipped, secret access uses the STS token directly.
    req := intercepted.CapturedRequest()
    auth := req.Header.Get("Authorization")
    if auth != "Bearer ya29.mock-sts-token" {
        t.Errorf("expected STS token, got: %s", auth)
    }
}

// Is every match arm exercised?
// Derived from: BranchBuilder node with 3 arms gets coverage for each.
func TestMatchBranch_Runtime_GitHubActions(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_github_oidc", mockGitHubOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    _, err := RunWithMocks(AcquireGCPSecret, AcquireGCPSecretInput{
        Runtime: GitHubActions, Project: "p", SecretName: "s",
    }, mocks)
    if err != nil {
        t.Fatal(err)
    }
    mocks.AssertCalled(t, "execute_github_oidc")
}

func TestMatchBranch_Runtime_GCPMetadata(t *testing.T) {
    mocks := transport.NewMockSet()
    mocks.On("execute_metadata_oidc", mockMetadataOIDCResponse)
    mocks.On("execute_sts", mockSTSResponse)
    mocks.On("execute_secret", mockAccessVersionResponse)

    _, err := RunWithMocks(AcquireGCPSecret, AcquireGCPSecretInput{
        Runtime: GCPMetadata, Project: "p", SecretName: "s",
    }, mocks)
    if err != nil {
        t.Fatal(err)
    }
    mocks.AssertCalled(t, "execute_metadata_oidc")
    mocks.AssertNotCalled(t, "execute_github_oidc")
}
```

### Bucket D: Resource hygiene

```go
// Is every transport node's resource declared and connected?
// Derived from: every Transport(Execute) node with res:* ports gets
// TransportResourceDeclared + ResourceInputConnected.
func TestResourceDeclared_ExecuteSTS(t *testing.T) {
    // This test verifies the graph structure, not runtime behavior.
    // The compiler checks that execute_sts has res:network port
    // and that it's wired to net_env's output.
    graph := BuildAcquireGCPSecretGraph()
    node := graph.GetNode("execute_sts")

    resPort := node.InputPort("res:network")
    if resPort == nil {
        t.Fatal("execute_sts missing res:network port")
    }
    if resPort.ResourceAccess != AccessModeRead {
        t.Errorf("want Read, got %v", resPort.ResourceAccess)
    }

    edges := graph.EdgesTo("execute_sts", "res:network")
    if len(edges) == 0 {
        t.Fatal("res:network port not connected")
    }
    if edges[0].FromNode != "net_env" {
        t.Errorf("expected net_env, got %s", edges[0].FromNode)
    }
}
```

### What these tests cover that hand-rolled tests typically miss

The compiler generates **38 test functions** for this one workflow. A thorough developer writing Go tests might write 8–12. The tests that are easy to forget — and that the compiler always generates — include:

- **Every match arm exercised** (`TestMatchBranch_Runtime_*`): developers often test the happy path (GitHub Actions) and forget GCPMetadata and LocalDev.
- **Guard both-paths** (`TestGuardBranch_Impersonation_Taken` + `_Skipped`): it's natural to test "with impersonation" but forget "without", or vice versa.
- **Downstream not reached on failure** (`mocks.AssertNotCalled(t, "execute_secret")`): when STS fails, the test verifies that Secret Manager was never called — ensuring the workflow short-circuits correctly.
- **Token threading through guards** (`TestGuardBranch_Impersonation_Taken` checking the auth header): verifying that the *impersonated* token (not the STS token) reaches the secret access call — a subtle data-flow property that falls out of the graph edges.
- **Missing response fields** (`TestNodeContract_ParseAccessVersion_MissingData`): the cardinality `One` on the output port means the parse function *must* produce a value; a response missing the field is a contract violation.
- **Resource wiring** (`TestResourceDeclared_ExecuteSTS`): structural tests that verify the graph itself, not just runtime behavior — catching cases where a refactor disconnects a resource port.
