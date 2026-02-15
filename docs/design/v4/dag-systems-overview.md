# DAG-Based Systems Overview

> A presentation document covering gunb.ai's DAG-driven architecture, current implementations, and the vision for declarative reliability.

**How to read this**: this is a reference snapshot of Go-era DAG systems.
It is **not** the V2 contracts spec (see
[`v2-contracts-design.md`](v2-contracts-design.md)). The important
transferable ideas are: contracts on executable code, typed dataflow,
resource claims, and the compile/validate pipeline.

### Vocabulary mapping to V2

| Go-era term | V2 term |
|---|---|
| NodeContract | `PropertyClaim` + prereq/import/export/claim equivalents |
| Task / Operation | Block (execution layer) + instantiated behavior (modeling layer) |
| Patterns (retry, lease, two-phase) | Candidate `PatternDef`s — only after census gate (P3) |
| `CompileAndValidate` | Lowering phase (validation + unrolling) |
| `func() error` | No direct equivalent — V2 models external tools, not executable code |

---

## Executive Summary

gunb.ai uses a **unified DAG framework** (`OaaS_v2/pkg/dag`) to orchestrate complex, multi-step workflows across four major systems:

| System | Purpose | DAG Patterns Used |
|--------|---------|-------------------|
| **make heal** | Code generation & linting | Resource locks, wave execution, contracts |
| **make login** | Authentication & secrets | Two-phase DAG, interactive step separation |
| **infra apply** | Infrastructure automation | Source resolution, state propagation, lock serialization |
| **OaaS/triage** | LLM task orchestration | Ticket dependencies, lease-based execution, budget flow |

Each system started as "fragile imperative code" and evolved to use DAG patterns for reliability. This document captures the current state and the vision for **declarative reliability** where infrastructure context automatically derives behavioral requirements.

---

## Part 0: The DAG Contract System

Before diving into specific systems, here's the unified contract system from `OaaS_v2/pkg/dag/` that all systems build upon.

### Core Interface: Contractor

Every DAG node implements a single interface:

```go
type Contractor interface {
    Contract() NodeContract
}
```

### NodeContract: The Unified Contract

```go
type NodeContract struct {
    // ══════════════════════════════════════════════════════════════
    // Prerequisites (auto-derived from Exports/Imports + manual)
    // ══════════════════════════════════════════════════════════════
    Provides []PrerequisiteID  // What this node establishes
    Requires []PrerequisiteID  // What this node needs before running

    // ══════════════════════════════════════════════════════════════
    // Resource Claims (capacity-based scheduling)
    // ══════════════════════════════════════════════════════════════
    Claims []Claim

    // ══════════════════════════════════════════════════════════════
    // Typed Data Flow (imports/exports between nodes)
    // ══════════════════════════════════════════════════════════════
    Exports []DataRef  // Data this node produces
    Imports []DataRef  // Data this node consumes

    // ══════════════════════════════════════════════════════════════
    // Runner State Management (cleanup orchestration)
    // ══════════════════════════════════════════════════════════════
    Invalidates            []RunnerResourceRef  // State this node removes
    RequiresRunnerResource []RunnerResourceRef  // State this node needs

    // ══════════════════════════════════════════════════════════════
    // Integration Capabilities (external service access)
    // ══════════════════════════════════════════════════════════════
    RequiresIntegration []IntegrationCapabilityRef
}
```

### PrerequisiteID: Unified Namespace

Prerequisites use "namespace:path" format for type-safe identifiers:

```go
type PrerequisiteID string

// Constructors
dag.Data("CI_EXPORT_PATH")           // "data:CI_EXPORT_PATH"
dag.Res("go-ctrl")                   // "resource:go-ctrl"
dag.Cap("bazel")                     // "cap:bazel"
dag.Integration("github/actions/read") // "integration:github/actions/read"
dag.State("runner/android-sdk")      // "state:runner/android-sdk"
dag.File("/workspace/config.json")   // "file:/workspace/config.json"
```

**Auto-derivation**: Prerequisites are automatically derived from typed contracts:
- Each `Export` → `"data:{name}"` prerequisite (provided)
- Each required `Import` → `"data:{name}"` prerequisite (required)
- Manual `Provides`/`Requires` for non-data prerequisites

### DataRef: Typed Data Flow

```go
type DataRef struct {
    Name        string    // "CI_EXPORT_BAZELISK_HOME"
    Description string    // Required: what this data represents
    Type        DataType  // Coarse-grained type checking
    Required    bool      // Hard requirement vs optional
}

// Coarse-grained "ish" types (intentionally permissive)
const (
    TypeUnknown      DataType = iota
    TypeStringish             // Paths, names, messages
    TypeIntish                // Counts, sizes, exit codes
    TypeBoolish               // Flags, success/failure
    TypeFloatish              // Percentages, durations
    TypeStringListish         // File paths, targets
    TypeJSONish               // Structured configs
    TypeBinaryish             // Path to executable
)

// Constructor helpers
dag.StringRef("OUTPUT_PATH", "Build output location", true)
dag.BoolRef("SUCCESS", "Whether build succeeded", true)
dag.JSONRef("CONFIG", "Build configuration", false)
```

### Claim: Resource Capacity

```go
type Claim struct {
    ResourceID  string  // "ci/apt", "go-ctrl", etc.
    Slots       uint32  // How many slots to acquire
    Priority    int32   // Queue ordering (higher = more urgent)
    NonBlocking bool    // Fail immediately if unavailable
}

// Capacity constants
const (
    CapacityUnlimited uint32 = 0  // Value semantics (infinite concurrent)
    CapacityMutex     uint32 = 1  // Exclusive access (mutex)
)
```

### IntegrationCapabilityRef: External Access

```go
type IntegrationCapabilityRef struct {
    Provider    string  // "github", "openai", "cursor"
    Capability  string  // "actions:read", "chat:completion"
    Description string  // Why this is needed
    Resource    string  // Optional: specific resource
}
```

### RequirementRef: Behavioral Contracts

```go
type RequirementRef struct {
    ID          string            // "output.deterministic"
    Name        string            // Human-readable
    Description string
    Enforcement EnforcementLevel  // Advisory/Warning/Enforced/Deprecated
}

type RequirementAcknowledgment struct {
    RequirementID string
    Method        VerificationMethod  // Test/Attestation/Inherited
    Reasoning     string
    TestTarget    string              // If Test method
    Expires       string              // If Attestation (format: "2006-01-02")
}
```

### ContractBuilder: Fluent API

```go
contract := dag.NewContract().
    Provides(dag.Cap("bazel")).
    Requires(dag.Data("config")).
    Claims(dag.Claim{ResourceID: "ci/apt", Slots: 1}).
    Exports(dag.StringRef("CI_EXPORT_PATH", "Build output", true)).
    Imports(dag.StringRef("CI_INPUT_CONFIG", "Build config", true)).
    RequiresIntegration(dag.IntegrationCapabilityRef{
        Provider: "github", Capability: "actions:read",
    }).
    Build()
```

### Compile & Validate Pipeline

```go
result := dag.CompileAndValidate(d, dag.CompileAndValidateOptions{
    ResourcePool:        pool,
    IntegrationCheckers: checkers,
    IncludeOrphanChecks: true,
})

// Validation passes (in order):
// 1. Structural - cycles, missing deps, self-deps
// 2. Contractor - all nodes implement Contractor interface
// 3. Contract compilation - derive prerequisites, wave-aware
// 4. Data flow - exports/imports match, type checking
// 5. Resource conflicts - capacity violations within waves
// 6. Runner resource conflicts - cleanup vs usage
// 7. Integration capabilities - external access verification
// 8. Orphan checks - unused exports/invalidations (warnings)

if result.HasErrors() {
    for _, err := range result.AllErrors() {
        log.Error(err)
    }
}
```

---

## Part 1: make heal

### Problem Statement

Running code generation and linting involves:
- Multiple tools with interdependencies (protos → Go code → linting)
- Concurrent access to shared resources (go.mod, bazel)
- Need for determinism (CI must produce identical results)
- Performance (avoid repeated Bazel analysis overhead)

### Solution: DAG with Resource Locks & Contracts

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 0: No Tools (bootstrap)                               │
│   bazelrc-extdeps (Pure Go regex parser)                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Python Only                                        │
│   module-bazel-versions (sync MODULE.bazel)                 │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Bazel Required                                     │
│   versions → protos                                         │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3+: Parallel Generators                               │
│   workflows | scripts | devcontainer | dockerfile | ...     │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Lint Wave (parallel with resource locks)                    │
│   gofmt | biome | buildifier | golangci-lint | ...          │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

**1. Resource-Based Scheduling**
```go
// Unix-style permissions for mutual exclusion
resources.Write("go-source")  // Exclusive: blocks all others
resources.Read("go-source")   // Shared: multiple concurrent OK

// Maps to DAG capacity
func (r ResourceRef) ToUnified() *dag.Resource {
    capacity := dag.CapacityUnlimited
    if r.Mode.IsExclusive() {
        capacity = dag.CapacityMutex  // Capacity = 1
    }
    return &dag.Resource{ID: r.Name, Capacity: capacity}
}
```

**2. Contract System (Behavioral Requirements)**
```go
type Task struct {
    id        string
    dependsOn []string
    resources []resources.ResourceRef
    contracts []Contract  // MUST declare: Deterministic, Idempotent
    run       func() error
}

// Required contracts enforced at validation
var RequiredContracts = []Contract{
    ContractOutputDeterministic,  // Same input → byte-identical output
}
```

**3. Contract Registry with Verification**
```go
// Each task has a registered contract with proof
func BazelrcExtdepsContract() GeneratorContract {
    return GeneratorContract{
        ID: "bazelrc-extdeps",
        Compliance: []ComplianceDeclaration{{
            RequirementID: "output.deterministic",
            Verification: Verification{
                Test: &TestReference{
                    Target:       "//tools/heal:determinism_test",
                    TestFunction: "TestBazelConfigGeneration_Deterministic",
                    Description:  "Runs 100 times, verifies byte-identical output",
                },
            },
        }},
    }
}
```

### Actual DAG Definition

From `tools/heal/main.go` - the complete task list:

```go
tasks := []*Task{
    // ══════════════════════════════════════════════════════════════
    // Layer 0: No tools required (bootstrap)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "bazelrc-extdeps",
        name:      "Generate .bazelrc.extdeps",
        resources: []resources.ResourceRef{resources.Write("bazel-ctrl")},
        requires:  []string{},  // Layer 0: no tools
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        failFast:  true,
        run:       func() error { return generateBazelrcExtdeps(repoRoot) },
    },

    // ══════════════════════════════════════════════════════════════
    // Layer 1: Python only (no Bazel)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "module-bazel-versions",
        name:      "Sync MODULE.bazel versions",
        resources: []resources.ResourceRef{resources.Write("bazel-ctrl")},
        requires:  []string{"python3"},  // Layer 1: Python only
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        failFast:  true,
        run:       func() error { return syncModuleBazelVersions(repoRoot) },
    },

    // ══════════════════════════════════════════════════════════════
    // Layer 2: Bazel required
    // ══════════════════════════════════════════════════════════════
    {
        id:        "versions",
        name:      "Generate version constants",
        dependsOn: []string{"module-bazel-versions"},
        requires:  []string{"bazel"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        failFast:  true,
        run:       func() error { return generateVersions(repoRoot) },
    },
    {
        id:        "protos",
        name:      "Generate proto files",
        dependsOn: []string{"versions"},
        requires:  []string{"bazel"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        failFast:  true,
        run:       func() error { return generateProtos(repoRoot) },
    },

    // ══════════════════════════════════════════════════════════════
    // Layer 3+: Parallel generators (all depend on protos)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "workflows",
        name:      "Generate GitHub workflows",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateWorkflows(repoRoot) },
    },
    {
        id:        "scripts",
        name:      "Generate shell scripts",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateScripts(repoRoot) },
    },
    {
        id:        "devcontainer",
        name:      "Generate devcontainer scripts",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateDevcontainer(repoRoot) },
    },
    {
        id:        "dockerfile",
        name:      "Generate Dockerfile snippets",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateDockerfile(repoRoot) },
    },
    {
        id:        "dagdiagrams",
        name:      "Generate DAG diagrams",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateDAGDiagrams(repoRoot) },
    },
    {
        id:        "compute",
        name:      "Generate compute config",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateComputeConfig(repoRoot) },
    },
    {
        id:        "bazelconfig",
        name:      "Generate Bazel config",
        dependsOn: []string{"protos"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return generateBazelConfig(repoRoot) },
    },

    // ══════════════════════════════════════════════════════════════
    // Sync phase: go mod tidy → go deps sync
    // ══════════════════════════════════════════════════════════════
    {
        id:        "go-mod-tidy",
        name:      "Sync go.mod",
        dependsOn: []string{"protos"},
        resources: []resources.ResourceRef{resources.Write("go-ctrl")},
        requires:  []string{"go"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runGoModTidy(repoRoot) },
    },
    {
        id:        "go-deps-sync",
        name:      "Sync go_deps modules",
        dependsOn: []string{"go-mod-tidy"},
        resources: []resources.ResourceRef{resources.Write("bazel-ctrl")},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return syncModuleBazelGoDeps(repoRoot) },
    },

    // ══════════════════════════════════════════════════════════════
    // Lint phase (depend on generators + go-deps-sync)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "shell-executable-fix",
        name:      "Fix shell script permissions",
        dependsOn: generatorDeps,
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.ShellExecutableFix{}) },
    },
    {
        id:        "crlf-fix",
        name:      "Fix line endings",
        dependsOn: generatorDeps,
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.CRLFFix{}) },
    },
    {
        id:        "gofmt",
        name:      "Format Go code",
        dependsOn: []string{"crlf-fix", "go-test-macros"},
        resources: []resources.ResourceRef{resources.Write("go-source")},
        requires:  []string{"gofmt"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.Gofmt{}) },
    },
    {
        id:        "clang-format",
        name:      "Format proto files",
        dependsOn: []string{"crlf-fix"},
        resources: []resources.ResourceRef{resources.Write("proto")},
        requires:  []string{"clang-format"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.ClangFormat{}) },
    },
    {
        id:        "biome",
        name:      "Format JS/TS/JSON",
        dependsOn: []string{"crlf-fix"},
        resources: []resources.ResourceRef{resources.Write("frontend")},
        requires:  []string{"biome"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.Biome{}) },
    },
    {
        id:        "gazelle",
        name:      "Update BUILD files",
        dependsOn: []string{"gazelle-directives"},
        resources: []resources.ResourceRef{
            resources.Write("bazel-ctrl"),
            resources.Read("go-source"),
        },
        requires:  []string{"bazel"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.Gazelle{}) },
    },
    {
        id:        "buildifier",
        name:      "Format BUILD files",
        dependsOn: []string{"gazelle"},
        resources: []resources.ResourceRef{resources.Write("bazel-ctrl")},
        requires:  []string{"buildifier"},
        contracts: []Contract{ContractOutputDeterministic, ContractOutputIdempotent},
        run:       func() error { return runTool(&linttools.Buildifier{}) },
    },
    {
        id:        "golangci-lint",
        name:      "Lint Go code",
        dependsOn: []string{"gofmt"},
        resources: []resources.ResourceRef{resources.Read("go-source")},
        requires:  []string{"golangci-lint"},
        contracts: []Contract{ContractOutputDeterministic},  // Read-only
        run:       func() error { return runTool(&linttools.GolangciLint{}) },
    },

    // ══════════════════════════════════════════════════════════════
    // Verify phase
    // ══════════════════════════════════════════════════════════════
    {
        id:        "orphan-check",
        name:      "Check for orphan files",
        dependsOn: []string{"buildifier"},
        contracts: []Contract{ContractOutputDeterministic},
        run:       func() error { return runOrphanCheck(repoRoot) },
    },

    // ══════════════════════════════════════════════════════════════
    // Completion marker
    // ══════════════════════════════════════════════════════════════
    {
        id:        "complete",
        name:      "Heal complete",
        dependsOn: []string{"orphan-check", "golangci-lint", "clang-format", "biome"},
        contracts: []Contract{ContractOutputDeterministic},
        run:       func() error { return nil },  // No-op marker
    },
}
```

**Dependency Chain Summary:**
```
Layer 0: bazelrc-extdeps (no deps)
    ↓
Layer 1: module-bazel-versions
    ↓
Layer 2: versions → protos
    ↓
Layer 3: workflows | scripts | devcontainer | dockerfile | dagdiagrams | compute | bazelconfig
         go-mod-tidy → go-deps-sync
    ↓
Lint:    crlf-fix → gofmt → golangci-lint
         crlf-fix → clang-format
         crlf-fix → biome
         gazelle-directives → gazelle → buildifier → orphan-check
    ↓
Complete: depends on all terminal lint tasks
```

### Contract Usage (pkg/dag patterns)

**Resources as Claims:**
```go
// tools/heal/resources/unified.go
func (r ResourceRef) ToUnified() *dag.Resource {
    capacity := dag.CapacityUnlimited
    if r.Mode.IsExclusive() {
        capacity = dag.CapacityMutex
    }
    return &dag.Resource{ID: r.Name, Capacity: capacity}
}

// Example claims in tasks:
resources.Write("go-source")   // → dag.Claim{ResourceID: "go-source", Slots: 1}
resources.Write("bazel-ctrl")  // → dag.Claim{ResourceID: "bazel-ctrl", Slots: 1}
resources.Read("go-source")    // → dag.Claim{ResourceID: "go-source", Slots: 0} (unlimited)
```

**Behavioral Contracts (Heal-specific, not pkg/dag):**
```go
// tools/heal/contracts/generators.go
type GeneratorContract struct {
    ID          string
    Name        string
    Description string
    Compliance  []ComplianceDeclaration
}

type ComplianceDeclaration struct {
    RequirementID string  // "output.deterministic", "output.idempotent"
    Verification  Verification
}

type Verification struct {
    Test        *TestReference        // Test proves compliance
    Attestation *AttestationReference // Human attests compliance
    Inherited   *InheritedReference   // Inherits from dependency
}

// Example: protos generator contract
func ProtosContract() GeneratorContract {
    return GeneratorContract{
        ID:   "protos",
        Name: "Proto Generator",
        Compliance: []ComplianceDeclaration{
            {
                RequirementID: "output.deterministic",
                Verification: Verification{
                    Test: &TestReference{
                        Target:       "//tools/heal:determinism_test",
                        TestFunction: "TestProtoGeneration_Deterministic",
                    },
                },
            },
            {
                RequirementID: "output.idempotent",
                Verification: Verification{
                    Inherited: &InheritedReference{
                        Source: "output.deterministic",
                        Reason: "Deterministic output implies idempotent",
                    },
                },
            },
        },
    }
}
```

**Data Flow (implicit via file diffs):**
```go
// Each generator returns []string of modified files
// These become implicit exports for downstream consumers
diffs, err := generateProtos(repoRoot)
addDiffs(diffs)  // Tracked for -check mode validation
```

### Code Location

- Entry: `tools/heal/main.go`
- Task model: `tools/heal/task.go`
- Resources: `tools/heal/resources/`
- Contracts: `tools/heal/contracts/`

---

## Part 2: make login

### Problem Statement

Authentication involves:
- Multiple sequential steps (check credentials → login → configure)
- Interactive prompts (browser auth) mixed with background work
- Multiple services to configure (GCP, secrets, shell env)
- Need for freshness checking and retry on token expiry

### Solution: Two-Phase DAG with Interactive Separation

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: Authentication DAG (login-auth)                    │
├─────────────────────────────────────────────────────────────┤
│ Preflight (parallel):                                       │
│   clear-cache | detect-env | check-account | check-adc      │
├─────────────────────────────────────────────────────────────┤
│ Interactive (sequential):                                   │
│   auth (gcloud login - browser prompt)                      │
├─────────────────────────────────────────────────────────────┤
│ Configuration:                                              │
│   configure-gcloud (set project defaults)                   │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: Secrets Fetch DAG (login-fetch)                    │
├─────────────────────────────────────────────────────────────┤
│   fetch-secrets                                              │
│      ↓                                                       │
│   sync-remote-home | write-bazelrc (parallel)               │
│      ↓                                                       │
│   export-shell-env (depends on both)                        │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

**1. Interactive vs Background Separation**
```go
func runLoginDAG(steps []*loginStep, ctx *loginContext) error {
    // Separate interactive steps from background steps
    var interactiveSteps, backgroundSteps []*loginStep
    for _, step := range steps {
        if step.interactive {
            interactiveSteps = append(interactiveSteps, step)
        } else {
            backgroundSteps = append(backgroundSteps, step)
        }
    }

    // Execute background steps using DAG executor (parallel)
    d.Execute(ctx, dag.ExecuteOptions{MaxConcurrency: 4}, ...)

    // Run interactive steps sequentially (user prompts)
    for _, step := range interactiveSteps {
        step.run(ctx)
    }
}
```

**2. Freshness with Retry Loop (DAG Pattern)**
```go
func EnsureFresh(ctx context.Context, cfg RefreshConfig) *RefreshResult {
    // Quick check: if already fresh, return immediately
    if !fresh.RefreshNeeded {
        return &RefreshResult{Success: true}
    }

    // Use Loop pattern for bounded refresh attempts
    loopCfg := patterns.LoopConfig[*Freshness]{
        MaxAttempts: 3,
        Interval:    2 * time.Second,
        Action: func(ctx context.Context, state patterns.LoopState) (*Freshness, patterns.LoopOutcome, error) {
            if err := doRefresh(ctx, cfg); err != nil {
                if isPermanentAuthFailure(err) {
                    return nil, patterns.LoopFatal, err
                }
                return nil, patterns.LoopContinue, nil  // Retry
            }
            return newFresh, patterns.LoopSuccess, nil
        },
    }
    return patterns.Loop(ctx, loopCfg)
}
```

**3. Non-Fatal Error Handling**
```go
// Remote home sync is non-fatal (graceful degradation)
if err := syncer.SyncFromRemote(ctx, account); err != nil {
    fmt.Fprintf(os.Stderr, "[home] warning: %v\n", err)
}
return nil  // Continue execution
```

### Actual DAG Definition

From `tools/secrets/cmd/secrets/main.go` - the two-phase login DAGs:

**Phase 1: Authentication Steps (`authSteps`)**

```go
authSteps := []*loginStep{
    // ══════════════════════════════════════════════════════════════
    // Preflight checks (parallel, no dependencies)
    // ══════════════════════════════════════════════════════════════
    {
        id:    "clear-cache",
        group: "preflight",
        run: func(ctx *loginContext) error {
            if mgr, err := cache.NewManager(cacheConfig); err == nil {
                _ = mgr.ClearAccountCache()
            }
            return nil
        },
    },
    {
        id:    "detect-env",
        group: "preflight",
        run: func(ctx *loginContext) error {
            ctx.setIsContainer(isContainerEnvironment())
            ctx.setCanOpenBrowser(canOpenBrowser())
            return nil
        },
    },
    {
        id:    "check-account",
        group: "preflight",
        run: func(ctx *loginContext) error {
            account, err := client.GetAccount()
            ctx.setAccount(account)
            return err
        },
    },
    {
        id:    "check-adc",
        group: "preflight",
        run: func(ctx *loginContext) error {
            ctx.setHasADC(client.HasADC())
            return nil
        },
    },
    {
        id:    "check-tokens",
        group: "preflight",
        run: func(ctx *loginContext) error {
            ctx.setHasTokens(client.HasValidTokens())
            return nil
        },
    },

    // ══════════════════════════════════════════════════════════════
    // Interactive auth (depends on ALL preflight checks)
    // ══════════════════════════════════════════════════════════════
    {
        id:          "auth",
        group:       "auth",
        interactive: true,  // <-- Separated from parallel execution
        dependsOn:   []string{"clear-cache", "detect-env", "check-account", "check-adc", "check-tokens"},
        run: func(ctx *loginContext) error {
            return runAuthStep(ctx, desiredDomain)
        },
    },

    // ══════════════════════════════════════════════════════════════
    // Configure gcloud (depends on auth)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "configure-gcloud",
        group:     "config",
        dependsOn: []string{"auth"},
        run: func(ctx *loginContext) error {
            if !ctx.needLogin() {
                return nil  // Skip if already logged in
            }
            _ = client.SetProject(*project)
            _ = client.SetQuotaProject(*project)
            return nil
        },
    },
}
```

**Phase 2: Secrets Fetch Steps (`fetchSteps`)**

```go
fetchSteps := []*loginStep{
    // ══════════════════════════════════════════════════════════════
    // Fetch secrets from Secret Manager
    // ══════════════════════════════════════════════════════════════
    {
        id:    "fetch-secrets",
        group: "fetch",
        run: func(ctx *loginContext) error {
            fetcher := fetch.NewFetcher(client, fetchConfig)
            ctx.setFetchResult(fetcher.FetchDevCI())
            return nil
        },
    },

    // ══════════════════════════════════════════════════════════════
    // Parallel: sync home + write bazelrc (both depend on fetch)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "sync-remote-home",
        group:     "home",
        dependsOn: []string{"fetch-secrets"},
        run: func(ctx *loginContext) error {
            syncer := home.NewSyncer(home.DefaultConfig())
            if !syncer.IsAvailable(context.Background()) {
                return nil  // GCS bucket not available - skip silently
            }
            if err := syncer.SyncFromRemote(ctx, ctx.account()); err != nil {
                // Non-fatal: log warning but don't fail login
                fmt.Fprintf(os.Stderr, "[home] warning: %v\n", err)
            }
            return nil
        },
    },
    {
        id:        "write-bazelrc",
        group:     "config",
        dependsOn: []string{"fetch-secrets"},
        run: func(ctx *loginContext) error {
            result := ctx.fetchResult()
            return writeBazelrcSecrets(result.Values, "", "")
        },
    },
    {
        id:        "clear-prompt-cache",
        group:     "config",
        dependsOn: []string{"fetch-secrets"},
        run: func(ctx *loginContext) error {
            return clearPromptCache()
        },
    },

    // ══════════════════════════════════════════════════════════════
    // Export shell env (depends on BOTH fetch + home sync)
    // ══════════════════════════════════════════════════════════════
    {
        id:        "export-shell-env",
        group:     "env",
        dependsOn: []string{"fetch-secrets", "sync-remote-home"},
        run: func(ctx *loginContext) error {
            result := ctx.fetchResult()
            mgr, _ := cache.NewManager(cacheConfig)

            exports := result.ToExports()
            exports += generatePromptIcons(result.Values, ctx.account())

            syncer := home.NewSyncer(home.DefaultConfig())
            exports += syncer.GetEnvSetup(ctx.account())

            return mgr.WriteSecretsCache(exports)
        },
    },

    // ══════════════════════════════════════════════════════════════
    // Completion marker
    // ══════════════════════════════════════════════════════════════
    {
        id:        "Login complete",
        group:     "complete",
        dependsOn: []string{"export-shell-env", "write-bazelrc", "clear-prompt-cache"},
        run:       func(_ *loginContext) error { return nil },
    },
}
```

**Dependency Chain Summary:**
```
Phase 1 (login-auth):
  Preflight (parallel): clear-cache | detect-env | check-account | check-adc | check-tokens
      ↓
  Interactive: auth (depends on ALL preflight)
      ↓
  Config: configure-gcloud

Phase 2 (login-fetch):
  fetch-secrets
      ↓
  sync-remote-home | write-bazelrc | clear-prompt-cache (parallel)
      ↓
  export-shell-env (depends on fetch + home sync)
      ↓
  Login complete
```

### Contract Usage (pkg/dag patterns)

**Implicit Data Flow via Context:**
```go
// loginContext acts as the data store between steps
type loginContext struct {
    mu           sync.RWMutex
    account      string
    isContainer  bool
    canBrowser   bool
    hasADC       bool
    hasTokens    bool
    fetchResult  *fetch.Result
}

// Steps "export" by setting context
func (ctx *loginContext) setAccount(a string)         { ctx.account = a }
func (ctx *loginContext) setFetchResult(r *fetch.Result) { ctx.fetchResult = r }

// Steps "import" by reading context
func (ctx *loginContext) account() string             { return ctx.account }
func (ctx *loginContext) fetchResult() *fetch.Result  { return ctx.fetchResult }

// This is equivalent to:
// dag.Export("account", ...)
// dag.Import("account", Required: true)
```

**Prerequisites via dependsOn:**
```go
// Explicit dependency declaration
{
    id:        "export-shell-env",
    dependsOn: []string{"fetch-secrets", "sync-remote-home"},
    // ...
}

// Equivalent to pkg/dag:
// dag.Requires(dag.Data("fetch-secrets.result"))
// dag.Requires(dag.Data("sync-remote-home.complete"))
```

**Interactive Flag (Custom Pattern):**
```go
// Login separates interactive from background steps
// This is a domain-specific pattern, not in pkg/dag
{
    id:          "auth",
    interactive: true,  // Runs sequentially, not in DAG executor
    dependsOn:   []string{"clear-cache", "detect-env", ...},
}
```

**Retry Pattern (pkg/dag/patterns):**
```go
// auth/refresh.go uses the Loop pattern
loopCfg := patterns.LoopConfig[*Freshness]{
    MaxAttempts: 3,
    Interval:    2 * time.Second,
    Action: func(ctx context.Context, state patterns.LoopState) (*Freshness, patterns.LoopOutcome, error) {
        if err := doRefresh(ctx, cfg); err != nil {
            if isPermanentAuthFailure(err) {
                return nil, patterns.LoopFatal, err  // Don't retry
            }
            return nil, patterns.LoopContinue, nil   // Retry
        }
        return newFresh, patterns.LoopSuccess, nil
    },
}
result := patterns.Loop(ctx, loopCfg)
```

### Code Location

- Entry: `tools/secrets/cmd/secrets/main.go`
- Auth freshness: `tools/secrets/auth/freshness.go`
- Refresh loop: `tools/secrets/auth/refresh.go`

---

## Part 3: infra apply

### Problem Statement

Infrastructure automation involves:
- Source artifacts (containers, scripts) must be built before infra DAG runs
- GCP APIs have 409 conflicts on concurrent IAM updates
- State must be checked before deciding to create/update
- Dependencies propagate (if template changes, MIG must update)

### Solution: Two-Phase Apply with Lock Serialization

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: Source Resolution                                  │
├─────────────────────────────────────────────────────────────┤
│ bazel://containers:prod  →  Build, push, get digest         │
│ file://scripts/startup.sh → Upload to GCS, get URL          │
│ registry://image:v1.2.3  →  Fetch digest                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
              (Inject resolved values)
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: Plan (Concurrent State Checking)                   │
├─────────────────────────────────────────────────────────────┤
│ For each operation: Check() → EXISTS | CREATE | UPDATE      │
│ Propagate BLOCKED from failed dependencies                   │
│ Propagate UPDATE from changed dependencies                   │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 3: Apply (Wave-Based Parallel Execution)              │
├─────────────────────────────────────────────────────────────┤
│ Wave 0: Enable APIs (root)                                  │
│ Wave 1: Create SAs, create secrets (parallel)               │
│ Wave 2: Bind CI SA (parallel, respecting locks)             │
│ Wave 3: Instance template, MIG, backend service...          │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

**1. Source Resolution (Holistic Apply)**
```go
type SourceRef string  // "bazel://target", "file://path", "registry://image"

func (r *SourceResolver) Resolve(ctx context.Context, ref SourceRef) (*ResolvedSource, error) {
    switch scheme(ref) {
    case "bazel":
        // 1. bazel build target
        // 2. oras cp to registry
        // 3. Get digest
        return &ResolvedSource{ImageTag: tag, ImageDigest: digest}
    case "file":
        // 1. Hash file content
        // 2. Upload to GCS with hash in name
        return &ResolvedSource{URL: gcsURL, FileHash: hash}
    case "registry":
        // Just fetch existing digest
        return &ResolvedSource{ImageTag: image, ImageDigest: digest}
    }
}
```

**2. Lock-Based Serialization (Preventing 409 Conflicts)**
```go
type Operation struct {
    ID        string
    DependsOn []string
    Locks     []string  // Mutual exclusion keys
    Check     func() (OpState, error)
    Execute   func() error
}

// Locks prevent concurrent GCP API conflicts
op.Locks = []string{
    "lock:iam-policy:project:gunbai-auto",      // Project IAM
    "lock:sa-iam:vm@project.iam.gserviceaccount.com",  // SA IAM
}

// DAG executor converts locks to claims
func (op *Operation) AcquireResources() []dag.Claim {
    for _, lock := range op.Locks {
        claims = append(claims, dag.Claim{
            ResourceID: "lock/" + lock,
            Slots: 1,  // Mutex semantics
        })
    }
}
```

**3. State Propagation**
```go
func PlanWithProgress(d *dag.DAG[*Operation], client gcloud.Admin) *PlanResult {
    // Pass 1: Run Check() for each operation (concurrent)
    d.Execute(ctx, opts, func(ctx context.Context, op *Operation) error {
        op.State, _, _ = op.Check(client)
        return nil
    })

    // Pass 2: Propagate BLOCKED from dependencies
    for _, op := range d.TopologicalOrder() {
        for _, depID := range op.DependsOn {
            if dep.State == StateBlocked {
                op.State = StateBlocked
                op.StateReason = fmt.Sprintf("blocked by %s", depID)
            }
        }
    }

    // Pass 3: Propagate UPDATE from changed dependencies
    for _, op := range d.TopologicalOrder() {
        if op.State != StateExists {
            continue
        }
        for _, depID := range op.DependsOn {
            if dep.State == StateCreate || dep.State == StateUpdate {
                op.State = StateUpdate
                op.StateReason = fmt.Sprintf("dependency %s changed", depID)
            }
        }
    }
}
```

**4. Idempotency via Check-Before-Act**
```go
Execute: IdempotentExecute(
    func(client gcloud.Admin) (bool, error) {
        return client.SecretExists(secretID, project)  // Check
    },
    func(client gcloud.Admin) error {
        return client.CreateSecret(secretID, project)  // Act
    },
),
```

### Actual DAG Definition

From `tools/infra/dag/builder.go` - the Operation struct and Build() method:

**Operation Struct (the DAG node type)**

```go
type Operation struct {
    ID          string      // "sa:gunbai-vm", "mig:gunbai-dev", etc.
    Type        OpType      // OpServiceAccount, OpMIG, OpHealthCheck, etc.
    DependsOn   []string    // DAG edges to other operation IDs
    Locks       []string    // Mutex keys for serialization
    Description string      // Human-readable for logging

    State       OpState     // unknown → blocked/exists/create/update
    StateReason string      // Why in this state

    CheckPermissions  []Permission  // IAM perms for Check()
    ApplyPermissions  []Permission  // IAM perms for Execute()

    Check   func(client gcloud.Admin) (OpState, string, error)
    Execute func(client gcloud.Admin) error

    Idempotency IdempotencyBehavior  // "idempotent" or "intentionally_stateful"
    Service     string               // "iam", "compute" - for rate limiting
}

type OpState string
const (
    StateUnknown  OpState = "unknown"
    StateBlocked  OpState = "blocked"   // Manual prereq not met
    StateExists   OpState = "exists"    // No changes needed
    StateCreate   OpState = "create"    // Will create
    StateUpdate   OpState = "update"    // Will update
)
```

**Build() Method - Constructing the Full DAG**

```go
func (b *Builder) Build() *coredag.DAG[*Operation] {
    var ops []*Operation
    cfg := b.spec.Config

    // ══════════════════════════════════════════════════════════════
    // Manual prerequisites (foundation - no auto deps)
    // ══════════════════════════════════════════════════════════════
    for _, prereq := range b.spec.ManualPrereqs {
        ops = append(ops, b.buildManualOp(prereq))
    }

    // Org policy (if any resources need public access)
    if b.hasPublicAccessResources() {
        ops = append(ops, b.buildOrgPolicyOp(cfg))
    }

    // ══════════════════════════════════════════════════════════════
    // Secrets (created early - other resources may depend on them)
    // ══════════════════════════════════════════════════════════════
    for _, secret := range b.spec.Secrets {
        ops = append(ops, b.buildSecretOps(secret, cfg)...)
    }

    // ══════════════════════════════════════════════════════════════
    // Service accounts (depend on API enablement)
    // ══════════════════════════════════════════════════════════════
    for _, sa := range b.spec.ServiceAccounts {
        ops = append(ops, b.buildServiceAccountOps(sa, cfg)...)
    }

    // ══════════════════════════════════════════════════════════════
    // Health checks
    // ══════════════════════════════════════════════════════════════
    for _, hc := range b.spec.HealthChecks {
        ops = append(ops, b.buildHealthCheckOp(hc, cfg, prefix))
    }

    // ══════════════════════════════════════════════════════════════
    // Instance templates (depend on service accounts)
    // ══════════════════════════════════════════════════════════════
    for _, tmpl := range b.spec.InstanceTemplates {
        ops = append(ops, b.buildInstanceTemplateOp(tmpl, cfg, prefix))
    }

    // ══════════════════════════════════════════════════════════════
    // MIGs (depend on instance templates)
    // ══════════════════════════════════════════════════════════════
    for _, mig := range b.spec.MIGs {
        ops = append(ops, b.buildMIGOps(mig, cfg, prefix)...)
    }

    // ══════════════════════════════════════════════════════════════
    // Cloud Run, Cloud Functions, Serverless NEGs
    // ══════════════════════════════════════════════════════════════
    for _, cr := range b.spec.CloudRunServices {
        ops = append(ops, b.buildCloudRunOps(cr, cfg)...)
    }
    for _, cf := range b.spec.CloudFunctions {
        ops = append(ops, b.buildCloudFunctionOps(cf, cfg)...)
    }
    for _, neg := range b.spec.ServerlessNEGs {
        ops = append(ops, b.buildServerlessNEGOp(neg, cfg))
    }

    // ══════════════════════════════════════════════════════════════
    // Backend services (depend on MIGs/NEGs + health checks)
    // ══════════════════════════════════════════════════════════════
    for _, bs := range b.spec.BackendServices {
        ops = append(ops, b.buildBackendServiceOps(bs, cfg, prefix)...)
    }

    // ══════════════════════════════════════════════════════════════
    // URL maps, proxies, forwarding rules (depend on backend services)
    // ══════════════════════════════════════════════════════════════
    for _, um := range b.spec.URLMaps {
        ops = append(ops, b.buildURLMapOp(um, cfg, prefix))
    }
    for _, fr := range b.spec.ForwardingRules {
        ops = append(ops, b.buildForwardingRuleOps(fr, cfg, prefix)...)
    }

    // ══════════════════════════════════════════════════════════════
    // IAP (depends on oauth consent screen + backend service)
    // ══════════════════════════════════════════════════════════════
    for _, iap := range b.spec.IAPConfigs {
        ops = append(ops, b.buildIAPOps(iap, cfg, prefix)...)
    }

    // ══════════════════════════════════════════════════════════════
    // GCS buckets
    // ══════════════════════════════════════════════════════════════
    for _, bucket := range b.spec.GCSBuckets {
        ops = append(ops, b.buildGCSBucketOps(bucket, cfg)...)
    }

    // Auto-inject API enablement based on what we're creating
    ops = b.injectAPIEnablement(ops, cfg, apisNeeded)

    return coredag.New("infra-"+string(b.spec.Environment), ops)
}
```

**Example Operation: Service Account with IAM**

```go
// Create SA operation
createOp := &Operation{
    ID:          fmt.Sprintf("sa:%s", saName),
    Type:        OpServiceAccount,
    Description: fmt.Sprintf("Create service account %s", saName),
    Locks:       []string{fmt.Sprintf("lock:sa:%s", saEmail)},
    CheckPermissions: []Permission{
        {Permission: "iam.serviceAccounts.get", Project: saProject},
    },
    ApplyPermissions: []Permission{
        {Permission: "iam.serviceAccounts.create", Project: saProject},
    },
    Check: func(client gcloud.Admin) (OpState, string, error) {
        exists, err := client.ServiceAccountExists(saEmail, saProject)
        if exists { return StateExists, "already exists", nil }
        return StateCreate, "will create", nil
    },
    Execute: func(client gcloud.Admin) error {
        exists, _ := client.ServiceAccountExists(saEmail, saProject)
        if exists { return nil }  // Idempotent
        return client.CreateServiceAccount(saName, saProject, displayName, desc)
    },
    Idempotency: BehaviorIdempotent,
}

// IAM binding operation (depends on SA creation)
bindOp := &Operation{
    ID:          fmt.Sprintf("iam:%s:%s", saName, sanitizeRole(role)),
    Type:        OpProjectIAM,
    DependsOn:   []string{createOp.ID},  // <-- DAG edge
    Description: fmt.Sprintf("Grant %s to %s", role, saName),
    Locks:       []string{fmt.Sprintf("lock:iam-policy:project:%s", project)},
    Check: func(client gcloud.Admin) (OpState, string, error) {
        has, _ := client.HasProjectIAMBinding(project, member, role)
        if has { return StateExists, "binding exists", nil }
        return StateCreate, "will add binding", nil
    },
    Execute: IdempotentExecute(
        func(c gcloud.Admin) (bool, error) { return c.HasProjectIAMBinding(project, member, role) },
        func(c gcloud.Admin) error { return c.AddProjectIAMBinding(project, member, role) },
    ),
    Idempotency: BehaviorIdempotent,
}

// WIF operation (depends on SA + manual WIF pool)
wifOp := &Operation{
    ID:          fmt.Sprintf("sa-wif:%s:%s", saName, wifAttr),
    Type:        OpServiceAccountIAM,
    DependsOn:   []string{createOp.ID, "manual:ci-wif-pool-provider"},  // Multiple deps
    Description: fmt.Sprintf("Grant WIF access to %s", saName),
    Locks:       []string{fmt.Sprintf("lock:sa-iam:%s", saEmail)},
    // ...
}
```

**Example: MIG with Intentionally Stateful Rolling Update**

```go
createOp := &Operation{
    ID:          fmt.Sprintf("mig:%s", name),
    Type:        OpMIG,
    DependsOn:   []string{fmt.Sprintf("template:%s", templateName)},
    Idempotency: BehaviorIdempotent,
    // ...
}

// Rolling update MUST run when template changes
rollOp := &Operation{
    ID:          fmt.Sprintf("mig-roll:%s", name),
    Type:        OpMIGRollingUpdate,
    DependsOn:   []string{createOp.ID, fmt.Sprintf("template:%s", templateName)},
    Check: func(client gcloud.Admin) (OpState, string, error) {
        return StateExists, "will trigger if template changed", nil
    },
    Execute: func(client gcloud.Admin) error {
        return client.TriggerMIGRollingUpdate(name, zone, project, templateName)
    },
    Idempotency: BehaviorIntentionallyStateful,  // <-- MUST run on UPDATE
}
```

**Dependency Chain Summary:**
```
Wave 0: api-enable:compute | api-enable:iam | manual:oauth-consent
    ↓
Wave 1: sa:gunbai-vm | sa:gunbai-deploy | secret:api-key
    ↓
Wave 2: iam:gunbai-vm:roles/... | sa-wif:gunbai-vm:...
        health-check:web
    ↓
Wave 3: template:gunbai-dev-tpl (depends on SA)
    ↓
Wave 4: mig:gunbai-dev (depends on template)
        mig-roll:gunbai-dev (intentionally stateful)
    ↓
Wave 5: backend-service:web (depends on MIG + health-check)
    ↓
Wave 6: url-map:main | iap:web
    ↓
Wave 7: forwarding-rule:https
```

### Contract Usage (pkg/dag patterns)

**Operations implement Contractor interface:**
```go
// tools/infra/dag/builder.go
func (op *Operation) Contract() dag.NodeContract {
    contract := dag.NewContract()

    // Convert Locks to Claims (mutex semantics)
    for _, lock := range op.Locks {
        contract.Claims(dag.Claim{
            ResourceID: "lock/" + lock,
            Slots:      1,  // Mutex
        })
    }

    // Service-based rate limiting
    if op.Service != "" {
        contract.Claims(dag.Claim{
            ResourceID: "rate/" + op.Service,
            Slots:      1,
        })
    }

    return contract.Build()
}
```

**Lock Patterns (mapping to Claims):**
```go
// Different lock scopes prevent 409 conflicts
op.Locks = []string{
    "lock:iam-policy:project:gunbai-auto",      // Project-wide IAM (coarse)
    "lock:sa-iam:vm@project.iam.gserviceaccount.com",  // SA-specific (fine)
    "lock:backend-service:gunbai-auto:web",     // Backend service config
}

// Maps to dag.Claim:
dag.Claim{ResourceID: "lock/lock:iam-policy:project:gunbai-auto", Slots: 1}
dag.Claim{ResourceID: "lock/lock:sa-iam:vm@...", Slots: 1}
```

**Prerequisites via DependsOn:**
```go
// Explicit DAG edges
bindOp := &Operation{
    ID:        "iam:gunbai-vm:roles/compute.admin",
    DependsOn: []string{"sa:gunbai-vm"},  // Must create SA first
}

// With manual prerequisites
wifOp := &Operation{
    ID:        "sa-wif:gunbai-vm:github",
    DependsOn: []string{
        "sa:gunbai-vm",                    // Auto dependency
        "manual:ci-wif-pool-provider",     // Manual prerequisite (human setup)
    },
}
```

**Permission Requirements (maps to RequiresIntegration):**
```go
op := &Operation{
    CheckPermissions: []Permission{
        {Permission: "iam.serviceAccounts.get", Project: project},
    },
    ApplyPermissions: []Permission{
        {Permission: "iam.serviceAccounts.create", Project: project},
    },
}

// Conceptually equivalent to:
dag.RequiresIntegration(dag.IntegrationCapabilityRef{
    Provider:   "gcp",
    Capability: "iam.serviceAccounts.get",
    Resource:   project,
})
```

**Idempotency as Behavioral Requirement:**
```go
type IdempotencyBehavior string

const (
    BehaviorIdempotent           IdempotencyBehavior = "idempotent"
    BehaviorIntentionallyStateful IdempotencyBehavior = "intentionally_stateful"
)

// Maps to dag.RequirementAcknowledgment:
dag.RequirementAcknowledgment{
    RequirementID: "execution.idempotent",
    Method:        dag.VerificationAttestation,
    Reasoning:     "Uses check-before-act pattern",
}
```

**State Propagation (custom infra pattern):**
```go
// Plan phase propagates state through DAG
// Pass 1: Check each operation
// Pass 2: Propagate BLOCKED from failed dependencies
// Pass 3: Propagate UPDATE from changed dependencies

// Conceptually similar to dag.Invalidates/RequiresRunnerResource:
// If template changes → MIG must update
op.Invalidates = []dag.RunnerResourceRef{{Name: "mig-state"}}
op.RequiresRunnerResource = []dag.RunnerResourceRef{{Name: "template-state"}}
```

### Code Location

- Entry: `tools/infra/cmd/infra/main.go`
- Source resolution: `tools/infra/dag/source.go`
- DAG builder: `tools/infra/dag/builder.go`
- Design doc: `docs/design/infra-holistic-apply.md`

---

## Part 4: OaaS / oaas-triage

### Problem Statement

LLM-backed task orchestration involves:
- Long-running operations (minutes to hours)
- Expensive API calls with cost tracking
- Dependencies between tickets (analysis → fix → test)
- Need for interruption handling (approval workflows)
- Concurrent execution across a worker pool

### Solution: Lease-Based DAG Execution with Budget Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Planner: Natural Language → Project+Ticket DAG              │
├─────────────────────────────────────────────────────────────┤
│ "Fix the flaky test in pkg/dag"                             │
│     ↓                                                        │
│ Project: "Fix flaky test"                                   │
│   ├─ Ticket 1: Analyze failure (no deps)                    │
│   ├─ Ticket 2: Draft fix (depends on 1)                     │
│   └─ Ticket 3: Verify fix (depends on 2)                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Engine: Polling Loop + Flat Worker Pool                     │
├─────────────────────────────────────────────────────────────┤
│ Every tick:                                                  │
│   1. ClaimRunnableTicketsGlobal(runnerID, limit, lease)     │
│   2. Dispatch each to worker goroutine                      │
│   3. Maintain heartbeat during execution                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Runner: Ticket Execution with Lease Maintenance             │
├─────────────────────────────────────────────────────────────┤
│ 1. Verify dependencies satisfied                            │
│ 2. Select worker (BasicLLM, Agentic, SystemOp)             │
│ 3. Execute with heartbeat loop (extend lease every 30-45s) │
│ 4. Handle outcome: Success | Failed | Interrupted          │
│ 5. Append cost transaction to ledger                       │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

**1. Lease-Based Exclusive Execution**
```go
// Atomic claim: only one runner can hold a ticket
func ClaimRunnableTicketsGlobal(ctx, runnerID, limit, duration) ([]*Ticket, error) {
    // Query: phase=READY AND lease_expires_at < now
    // Atomically set: lease_owner_id, lease_expires_at
    // Return claimed tickets
}

// Heartbeat keeps lease alive during long execution
func (r *TicketRunner) executeTicket(ctx context.Context, ticket *Ticket) error {
    outcomeCh := make(chan result, 1)
    go func() { outcomeCh <- worker.Execute(ctx, ticket) }()

    heartbeat := r.nextHeartbeatTimer()
    for {
        select {
        case outcome := <-outcomeCh:
            return r.persistOutcome(ctx, ticket, outcome)
        case <-heartbeat:
            r.store.ExtendTicketLease(ctx, r.runnerID, ticketID, r.leaseDuration)
            heartbeat = r.nextHeartbeatTimer()
        }
    }
}
```

**2. Dependency Resolution (DAG Semantics)**
```go
func areDependenciesSatisfied(deps []*Dependency, resolver) (bool, error) {
    for _, dep := range deps {
        // Ticket dependency: ticket must be COMPLETED with SUCCESS
        // Project dependency: ALL tickets must be COMPLETED with SUCCESS
        if !isDependencySatisfied(dep, resolver) {
            return false, nil  // Blocked, not error
        }
    }
    return true, nil
}
```

**3. Budget Flow (Hierarchical Cost Accounting)**
```
Root Project: Seed initial budget
  ↓
Parent spawns child: RESERVE transaction
  - Parent's available ↓, reserved ↑
  - Child ceiling set
  ↓
Child executes: DRAW transaction
  - Parent's reserved ↓, child's available ↑
  ↓
Child completes: RELEASE transaction
  - Unused reservation returns to parent
```

**4. Interruption & Decision Framework**
```go
// Worker can pause for human decision
type TicketInterruption struct {
    Kind    InterruptionKind  // APPROVAL, GUIDANCE, INPUT
    Summary string
    PlannedEffects []*PlannedSideEffect
}

// Flow:
// 1. Worker returns Interrupted=true
// 2. PauseTicket(): phase=BLOCKED, lease released
// 3. User reviews, makes decision
// 4. ResolveInterruption(decision): RESUME or ABORT
// 5. Engine reclaims and resubmits (if RESUME)
```

### Triage Workflow

```bash
oaas-triage flake --runs=50
  ├─ Run each test 50 times
  ├─ Collect pass/fail per run
  ├─ Compute failure rates
  ├─ Build context payload (evidence)
  └─ Dispatch to OaaS:
       └─ Template routes to BackendChangeV4 for fix

Evidence markers:
  - "both passes AND failures" → Race condition (not flaky test, but flaky CODE)
  - "main branch presubmit failed" → Deterministic bug
  - "coverage below threshold" → Test gap
```

### Actual DAG Definition (Proto)

From `OaaS_v2/proto/` - the Ticket and Dependency definitions:

**ticket.proto - Ticket Message**

```protobuf
message Ticket {
    // ══════════════════════════════════════════════════════════════
    // Meta (flat fields)
    // ══════════════════════════════════════════════════════════════
    optional string ticket_id = 1;
    optional string parent_project_id = 2;
    optional int64 spec_version = 3;
    optional google.protobuf.Timestamp created_at = 4;
    optional google.protobuf.Timestamp updated_at = 5;
    optional google.protobuf.Timestamp ticket_completed_at = 6;
    optional string boss_ticket_id = 7;
    optional bool is_infrastructure = 8;

    // ══════════════════════════════════════════════════════════════
    // Spec (intent - what to do)
    // ══════════════════════════════════════════════════════════════
    optional TicketSpec spec = 20;

    // ══════════════════════════════════════════════════════════════
    // Status (result - what happened)
    // ══════════════════════════════════════════════════════════════
    optional TicketStatus status = 30;
}

message TicketSpec {
    optional string title = 1;
    optional string instructions = 2;
    optional double budget_credits = 3;

    // ──────────────────────────────────────────────────────────────
    // DAG EDGES: Dependencies within or across projects
    // Scheduler reads each Dependency and decides when this ticket is unblocked
    // ──────────────────────────────────────────────────────────────
    repeated Dependency dependencies = 4;

    optional google.protobuf.Duration max_run_duration = 5;
    optional google.protobuf.Duration max_idle_duration = 6;
    repeated string preferred_models = 7;
    optional string model_policy = 8;

    enum ExecutionStyle {
        EXECUTION_STYLE_UNSPECIFIED = 0;
        EXECUTION_STYLE_BATCH = 1;      // Background processing
        EXECUTION_STYLE_ACTIVE = 2;     // Interactive/real-time
    }
    optional ExecutionStyle execution_style = 9;

    // Worker type selection
    oneof execution_profile {
        BasicLLMExecution basic_llm = 20;
        AgenticExecution agentic = 21;
        SystemOpExecution system_op = 22;
        PerfTestExecution perf_test = 23;
    }

    optional OutputContract output_contract = 30;
    repeated DerivedOutput derived_outputs = 31;
}

message TicketStatus {
    TicketPhase phase = 1;      // PENDING → READY → RUNNING → COMPLETED
    TicketOutcome outcome = 2;  // SUCCESS | FAILED | CANCELLED
    optional string notes = 3;
    optional string error_message = 4;

    oneof execution_result {
        BasicLLMResult basic_llm_result = 10;
        AgenticResult agentic_result = 11;
        SystemOpResult system_op_result = 12;
        PerfTestResult perf_test_result = 13;
    }

    repeated ExecutionEvent events = 15;
    repeated CostTransaction cost_transactions = 16;
    repeated string child_project_ids = 17;

    // Blocking model for approvals/guidance
    optional TicketInterruption interruption = 18;
    optional TicketDecision decision = 19;
}

enum TicketPhase {
    TICKET_PHASE_UNSPECIFIED = 0;
    TICKET_PHASE_PENDING = 1;    // Awaiting dependency satisfaction
    TICKET_PHASE_READY = 2;      // Dependencies met, eligible for claiming
    TICKET_PHASE_RUNNING = 3;    // Claimed by runner, executing
    TICKET_PHASE_COMPLETED = 4;  // Terminal: succeeded or failed
    TICKET_PHASE_CANCELLED = 5;  // Terminal: explicitly cancelled
    TICKET_PHASE_BLOCKED = 6;    // Non-terminal: awaiting decision
}

enum TicketOutcome {
    TICKET_OUTCOME_UNSPECIFIED = 0;
    TICKET_OUTCOME_SUCCESS = 1;
    TICKET_OUTCOME_FAILED = 2;
    TICKET_OUTCOME_CANCELLED = 3;
}
```

**dependency.proto - Dependency Edge**

```protobuf
enum DependencyKind {
    DEPENDENCY_KIND_UNSPECIFIED = 0;
    DEPENDENCY_KIND_TICKET = 1;   // Depends on another ticket
    DEPENDENCY_KIND_PROJECT = 2;  // Depends on ALL tickets in a project
}

message Dependency {
    optional DependencyKind kind = 1;

    oneof target {
        string ticket_id = 2;   // For DEPENDENCY_KIND_TICKET
        string project_id = 3;  // For DEPENDENCY_KIND_PROJECT
    }

    // Which output slot to consume (defaults to "primary")
    // Examples: "primary", "humanized", "summary"
    optional string output_slot = 4;
}
```

**project.proto - Project Container**

```protobuf
message Project {
    optional string project_id = 1;
    optional string parent_project_id = 2;
    optional int64 spec_version = 3;
    optional google.protobuf.Timestamp created_at = 4;
    optional google.protobuf.Timestamp updated_at = 5;
    optional bool archived = 6;
    optional string parent_ticket_id = 7;
    optional string root_project_id = 8;

    optional ProjectSpec spec = 20;
    repeated ExecutionEvent events = 15;
}

message ProjectSpec {
    optional string goal = 1;
    optional double budget_credits = 2;

    // Policy for scary side effects
    SideEffectPolicy side_effect_policy = 3;

    // Requirements/capabilities for matching
    repeated string requirements = 4;
    repeated string capabilities = 5;

    // Workspace locks for concurrent access control
    repeated WorkspaceLock workspace_locks = 6;
}

enum ProjectPhase {
    PROJECT_PHASE_UNSPECIFIED = 0;
    PROJECT_PHASE_ACTIVE = 1;     // Work in progress
    PROJECT_PHASE_COMPLETED = 2;  // All tickets completed successfully
    PROJECT_PHASE_HALTED = 3;     // Blocked by failed ticket(s)
}

enum SideEffectPolicy {
    SIDE_EFFECT_POLICY_UNSPECIFIED = 0;
    SIDE_EFFECT_POLICY_ALLOW = 1;               // Commit immediately
    SIDE_EFFECT_POLICY_REQUIRE_APPROVAL = 2;    // Pause at scary effects
    SIDE_EFFECT_POLICY_SIMULATE = 3;            // Never commit scary effects
    SIDE_EFFECT_POLICY_SIMULATE_PERMISSIVE = 4; // SIMULATE without preflight
}
```

**Example: Triage DAG**

```
Project: "Fix main branch CI failure"
├─ Ticket 1: "Analyze CI logs"
│     spec:
│       title: "Analyze CI logs"
│       dependencies: []  // No deps - root node
│       execution_profile: basic_llm
│
├─ Ticket 2: "Draft fix"
│     spec:
│       title: "Draft fix for root cause"
│       dependencies:
│         - kind: DEPENDENCY_KIND_TICKET
│           ticket_id: <ticket_1_id>
│           output_slot: "primary"  // Consumes analysis
│       execution_profile: agentic
│
└─ Ticket 3: "Verify fix"
      spec:
        title: "Run tests to verify fix"
        dependencies:
          - kind: DEPENDENCY_KIND_TICKET
            ticket_id: <ticket_2_id>
        execution_profile: system_op
```

**Key Invariants:**

```
Dependency satisfaction:
  TICKET dep → ticket.ticket_completed_at IS SET AND outcome == SUCCESS
  PROJECT dep → ALL tickets in project are COMPLETED with SUCCESS

Lifecycle:
  PENDING: dependencies not yet satisfied
  READY: all dependencies satisfied, no lease held
  RUNNING: lease held by a runner
  COMPLETED: terminal (success/failed)
  BLOCKED: awaiting human decision (interruption)

Terminal state invariant:
  if phase in {COMPLETED, CANCELLED}:
    ticket_completed_at IS SET
    outcome != UNSPECIFIED
```

### Contract Usage (pkg/dag patterns)

**Dependencies as Prerequisites:**
```protobuf
// ticket.proto: Dependency maps to dag.Requires
message Dependency {
    DependencyKind kind = 1;
    oneof target {
        string ticket_id = 2;   // dag.Data("ticket:{id}:output")
        string project_id = 3;  // dag.Data("project:{id}:complete")
    }
    string output_slot = 4;     // Which export to consume
}

// Equivalent pkg/dag contract:
dag.NewContract().
    Requires(dag.Data("ticket:abc123:primary")).
    Build()
```

**Output Contract as Exports:**
```protobuf
// ticket.proto: OutputContract defines exports
message OutputContract {
    repeated OutputSlot slots = 1;
}

message OutputSlot {
    string name = 1;         // "primary", "humanized", "summary"
    string description = 2;
    OutputType type = 3;     // TEXT, JSON, PATCH, etc.
}

// Equivalent pkg/dag contract:
dag.NewContract().
    Exports(dag.DataRef{
        Name:        "TICKET_OUTPUT_PRIMARY",
        Description: "Main ticket output",
        Type:        dag.TypeStringish,
        Required:    true,
    }).
    Build()
```

**Budget as Resource Claim:**
```go
// Budget flow is conceptually a resource claim
// Parent reserves budget → Child claims from reservation

// pkg/dag equivalent:
dag.NewContract().
    Claims(dag.Claim{
        ResourceID: "budget/project:parent123",
        Slots:      100,  // Credits requested
    }).
    Build()
```

**Integration Capabilities:**
```go
// Tickets declare what integrations they need
type AgenticExecution struct {
    // ...
    RequiredCapabilities []string  // "github:repo:write", "cursor:agent"
}

// pkg/dag equivalent:
dag.NewContract().
    RequiresIntegration(dag.IntegrationCapabilityRef{
        Provider:    "github",
        Capability:  "repo:write",
        Description: "Push commits to repository",
    }).
    RequiresIntegration(dag.IntegrationCapabilityRef{
        Provider:    "cursor",
        Capability:  "agent",
        Description: "Run agentic coding session",
    }).
    Build()
```

**Workspace Locks as Claims:**
```protobuf
// project.proto: WorkspaceLock for concurrent access
message WorkspaceLock {
    string workspace_id = 1;  // "repo:gunb-ai/gunb.ai"
    LockMode mode = 2;        // SHARED or EXCLUSIVE
}

// pkg/dag equivalent:
dag.NewContract().
    Claims(dag.Claim{
        ResourceID: "workspace/repo:gunb-ai/gunb.ai",
        Slots:      1,  // EXCLUSIVE
    }).
    Build()
```

**Side Effect Policy as Behavioral Requirement:**
```protobuf
enum SideEffectPolicy {
    SIDE_EFFECT_POLICY_ALLOW = 1;
    SIDE_EFFECT_POLICY_REQUIRE_APPROVAL = 2;
    SIDE_EFFECT_POLICY_SIMULATE = 3;
}

// Maps to requirement acknowledgment:
dag.RequirementAcknowledgment{
    RequirementID: "sideeffect.approval_required",
    Method:        dag.VerificationAttestation,
    Reasoning:     "Project policy requires human approval for scary effects",
}
```

**Lease as Resource Claim:**
```go
// Ticket lease is a mutex resource
// Only one runner can hold a ticket at a time

// pkg/dag equivalent:
dag.NewContract().
    Claims(dag.Claim{
        ResourceID: "lease/ticket:abc123",
        Slots:      1,  // Mutex
    }).
    Build()

// Heartbeat extends the lease duration
store.ExtendTicketLease(ctx, runnerID, ticketID, 10*time.Minute)
```

**Cost Transactions as Audit Trail:**
```protobuf
// cost.proto: Append-only ledger
message CostTransaction {
    string provider = 1;      // "openai", "cursor"
    string operation = 2;     // "chat_completion"
    CostReport cost = 3;
    Receipt receipt = 4;      // provider_request_id for audit
}

// This is an appendage pattern (L1) - not a contract,
// but attached to ticket execution for observability
```

### Code Location

- Engine: `OaaS_v2/internal/engine/`
- Runner: `OaaS_v2/internal/runner/`
- Store: `OaaS_v2/internal/store/`
- Planner: `OaaS_v2/internal/planner/`
- Triage CLI: `tools/ci/cmd/oaas-triage/`
- Protos: `OaaS_v2/proto/ticket.proto`, `dependency.proto`, `project.proto`

---

## Part 5: The SPOT Runner Problem

### Current Issue

GitHub Actions with SPOT runners introduces reliability challenges:

1. **No auto-restart on preemption** - Job hangs instead of retrying
2. **Non-idempotent jobs can't safely restart** - e.g., oaas-triage could run indefinitely, costing money
3. **Confusing progress UI** - User sees 90% → 0% on restart

### Why This Matters

Today, making a job SPOT-safe requires:
- Manual retry logic
- Progress checkpointing
- Idempotency enforcement
- Budget caps

This is the **"fragile process, add reliability"** pattern we keep repeating.

### The Vision: Declarative Reliability

Instead of writing reliability code manually, **derive it from execution context**:

```go
// Hypothetical: declare execution context, derive requirements
dag.Task("deploy",
    dag.RunsOn(Spot),  // Framework derives: must be idempotent, checkpointable
    dag.Do(actualDeployLogic),
)

// Framework auto-injects:
// - Checkpoint loading (resume from last state)
// - Checkpoint saving (after each step)
// - Idempotency key generation
// - Progress persistence (UI shows "resumed from 47%")
// - Budget caps (no infinite restart loops)
```

### What's Missing from pkg/dag

Current model describes **what a node needs**:
```go
node.Contract().
    Requires(dag.Cap("bazel")).
    Claims(dag.Claim{ResourceID: "ci/apt", Slots: 1}).
    Exports(dag.StringRef("OUTPUT_PATH", "Build output", true))
```

Missing: **where the node runs** and its implications:
```go
// L1 candidate: Execution environment properties
type ExecutionContext struct {
    Preemptible          bool   // SPOT instances
    NetworkPartitionable bool
    CostModel            CostModel
}

// L1 candidate: Derived requirements
type DerivedRequirements struct {
    MustBeIdempotent   bool  // derived from Preemptible=true
    MustCheckpoint     bool  // derived from Preemptible=true
    MustBeBudgetCapped bool  // derived from CostModel
}
```

### Two Implementation Approaches

**1. Compile-Time Enforcement (Current Style)**
```go
// Node declares it CAN run preemptibly
node.Contract().
    SatisfiesRequirement(req.Idempotent, "uses content-addressed outputs")

// Context declares it REQUIRES preemptibility support
ctx := dag.SpotContext()

// Validation fails if node doesn't satisfy context
dag.CompileAndValidate(d, dag.WithExecutionContext(ctx))
```

**2. Runtime Wrapping (New Capability)**
```go
// Framework automatically wraps non-idempotent work
dag.Task("deploy",
    dag.RunsOn(Spot),
    dag.Do(func(ctx dag.Context) error {
        // Framework auto-wraps with:
        // - Checkpoint loading
        // - Checkpoint saving
        // - Progress persistence
        return actualDeployLogic(ctx)
    }),
)
```

### End State Vision

```go
// 95% of work is DAG structure + contracts
pipeline := dag.Pipeline(
    dag.Stage("fetch", dag.Imports("url"), dag.Exports("raw")),
    dag.Stage("transform", dag.Imports("raw"), dag.Exports("processed")),
    dag.Stage("validate", dag.Imports("processed"), dag.Exports("result")),
)

// 5% is actual custom logic (pure functions)
pipeline.Stage("transform").Do = func(ctx dag.Context, raw []byte) ([]byte, error) {
    return json.Marshal(processData(raw))
}

// Framework provides:
// - I/O (fetch URLs, read files, write outputs)
// - Retry logic
// - Progress tracking
// - Checkpointing
// - Resource management
// - Cost accounting

// User provides:
// - Transformation logic (pure functions)
// - Domain-specific validation rules
```

---

## Summary: Evolution Path

| Stage | Description | Where We Are |
|-------|-------------|--------------|
| **1. Fragile** | Imperative code, no retry/checkpoint | Legacy code |
| **2. Wrapped** | Add DAG patterns for reliability | **make heal**, **make login** |
| **3. Structured** | DAG drives execution, state propagation | **infra apply**, **OaaS** |
| **4. Declared** | Context derives requirements | **Vision** (SPOT handling) |
| **5. Generated** | DAG does most I/O, user fills pure logic | **Future** |

The architectural question: Should the framework **validate** (compile-time), **enforce** (runtime wrapping), or **both**?

Current L2 patterns (RetryUntilSuccess, LeaseScope, BudgetFlow) already do some runtime enforcement. The SPOT runner problem suggests pushing this further: infrastructure context should automatically derive and enforce behavioral requirements.

---

## Part 6: Future Evolution — The Inverted Model

### The Core Problem Today

Today's development pattern is:

```
1. Write fragile imperative code
2. Discover it fails in production (SPOT preemption, network errors, etc.)
3. Wrap it with reliability patterns (retry, checkpoint, idempotency)
4. Repeat for every new system
```

This creates **far-reaching dependencies** — the fragile code reaches into infrastructure concerns, and we keep "joining" work to lock it down.

### The Inverted Model

What we want instead:

```
1. Design e2e causal flows (what data flows where)
2. Framework generates all boilerplate (I/O, retry, checkpoint, validation)
3. Contracts are satisfied by generation
4. Developer fills in pure helper functions
```

The key insight: **contracts can be satisfied by generation, not implementation**. If the framework generates checkpoint-aware wrappers, the developer never writes checkpoint code — they just write pure transformation logic.

### ExecutionContext: The Missing L1 Primitive

Currently, NodeContract describes **what a node needs**:

```go
type NodeContract struct {
    Provides []PrerequisiteID
    Requires []PrerequisiteID
    Claims   []Claim
    Exports  []DataRef
    Imports  []DataRef
    // ...
}
```

Missing: **where the node runs** and what that implies:

```go
// Proposed L1 addition
type ExecutionContext struct {
    // Infrastructure properties
    Preemptible          bool      // SPOT instances, serverless with timeout
    NetworkPartitionable bool      // Edge compute, mobile
    DiskEphemeral        bool      // Containers, serverless

    // Cost model
    CostModel struct {
        CostPerSecond    float64
        CostPerRequest   float64
        MaxCost          float64   // Budget cap
    }

    // Reliability profile
    ExpectedFailureRate float64   // 0.05 = 5% preemption rate
    MaxRetries          int
}
```

### DerivedRequirements: Auto-Computation from Context

Instead of manually declaring "this must be idempotent", the framework **derives** requirements:

```go
// Proposed L1 addition
type DerivedRequirements struct {
    // Derived from Preemptible=true
    MustBeIdempotent   bool
    MustCheckpoint     bool
    MustSupportResume  bool

    // Derived from CostModel.MaxCost > 0
    MustBeBudgetCapped bool
    MustTrackCost      bool

    // Derived from DiskEphemeral=true
    MustExternalizeState bool

    // Derived from NetworkPartitionable=true
    MustHandleOffline bool
}

func DeriveRequirements(ctx ExecutionContext) DerivedRequirements {
    req := DerivedRequirements{}

    if ctx.Preemptible {
        req.MustBeIdempotent = true
        req.MustCheckpoint = true
        req.MustSupportResume = true
    }

    if ctx.CostModel.MaxCost > 0 {
        req.MustBeBudgetCapped = true
        req.MustTrackCost = true
    }

    if ctx.DiskEphemeral {
        req.MustExternalizeState = true
    }

    return req
}
```

### Generated Wrappers: The Key Innovation

The framework can **generate wrappers** that satisfy derived requirements:

```go
// Developer writes pure logic
func transform(input []byte) ([]byte, error) {
    // Pure function - no I/O, no state
    return json.Marshal(processData(input))
}

// Framework generates checkpoint-aware wrapper
func (g *GeneratedWrapper) Execute(ctx dag.Context) error {
    // Auto-generated: load checkpoint
    checkpoint, _ := ctx.LoadCheckpoint()
    if checkpoint != nil {
        g.state = checkpoint.State
        g.step = checkpoint.Step
    }

    // Auto-generated: step through with checkpoint saves
    for g.step < len(g.steps) {
        result, err := g.steps[g.step](g.state)
        if err != nil {
            return err
        }
        g.state = result
        g.step++

        // Auto-generated: save checkpoint after each step
        ctx.SaveCheckpoint(Checkpoint{State: g.state, Step: g.step})
    }

    return nil
}
```

The developer **never writes checkpoint code** — they declare the pipeline structure and fill in pure functions.

### Migration Path: Each System

#### make heal

**Today**: Tasks declare `contracts` manually, executor checks them.

**Future**:
```go
// Instead of declaring contracts...
task := &Task{
    id:        "protos",
    contracts: []Contract{ContractOutputDeterministic}, // Manual
    run:       generateProtos,
}

// ...declare causal flow, derive contracts
flow := dag.Flow("heal").
    Stage("protos").
        Imports(dag.File("MODULE.bazel")).
        Exports(dag.FilePattern("**/*.pb.go")).
        Do(generateProtos)  // Pure: files in → files out

// Framework derives: file I/O is deterministic if function is pure
// Framework generates: diff-based idempotency check
```

#### make login

**Today**: `loginContext` stores inter-step data, `interactive` flag separates execution.

**Future**:
```go
flow := dag.Flow("login").
    Stage("preflight").
        Exports(dag.Data("account"), dag.Data("env")).
        Parallel(
            dag.Do(detectEnv),
            dag.Do(checkAccount),
            dag.Do(checkADC),
        ).
    Stage("auth").
        Interactive(true).  // Framework separates from parallel execution
        Requires(dag.Data("env")).
        Exports(dag.Data("tokens")).
        Do(runAuth).
    Stage("fetch").
        Requires(dag.Data("tokens")).
        Exports(dag.Data("secrets")).
        Do(fetchSecrets)

// Framework generates:
// - Context struct with thread-safe getters/setters
// - Interactive step separation
// - Retry logic for auth refresh
```

#### infra apply

**Today**: Operations have `Locks`, `Check`, `Execute`, state propagation logic.

**Future**:
```go
flow := dag.Flow("infra").
    Stage("sa:gunbai-vm").
        Claims(dag.Lock("sa:gunbai-vm@project")).
        Check(saExists).       // Pure: returns bool
        Create(createSA).      // Framework wraps with idempotency

    Stage("iam:gunbai-vm").
        Requires(dag.State("sa:gunbai-vm")).
        Claims(dag.Lock("iam-policy:project")).
        Check(bindingExists).
        Create(addBinding)

// Framework generates:
// - State propagation (if SA changes, IAM updates)
// - Lock acquisition/release
// - Check-before-act idempotency
// - Plan vs Apply mode switching
```

#### OaaS

**Today**: Ticket proto has `Dependency`, `TicketPhase`, `CostTransaction`.

**Future**:
```go
flow := dag.Flow("fix-flaky-test").
    Context(dag.Preemptible(true)).  // SPOT runner

    Stage("analyze").
        Imports(dag.Data("test_logs")).
        Exports(dag.Data("root_cause")).
        Budget(10.0).
        Do(analyzeWithLLM).

    Stage("fix").
        Requires(dag.Data("root_cause")).
        Exports(dag.Patch("fix.patch")).
        Budget(20.0).
        Do(generateFix).

    Stage("verify").
        Requires(dag.Patch("fix.patch")).
        Exports(dag.Bool("success")).
        Do(runTests)

// Framework generates:
// - Preemptible=true → checkpoint after each LLM call
// - Budget tracking and caps
// - Lease management and heartbeat
// - Output slot contracts
```

### The End State: 95% Structure, 5% Logic

```go
// Developer defines structure (95% of "code")
pipeline := dag.Pipeline("process-data",
    dag.Context(dag.Preemptible(true), dag.Budget(100.0)),

    dag.Stage("fetch",
        dag.Imports(dag.URL("input_url")),
        dag.Exports(dag.Data("raw_data")),
    ),
    dag.Stage("transform",
        dag.Imports(dag.Data("raw_data")),
        dag.Exports(dag.Data("processed")),
    ),
    dag.Stage("validate",
        dag.Imports(dag.Data("processed")),
        dag.Exports(dag.Bool("valid"), dag.Data("errors")),
    ),
    dag.Stage("store",
        dag.Imports(dag.Data("processed"), dag.Bool("valid")),
        dag.Exports(dag.Data("storage_key")),
    ),
)

// Developer fills in pure functions (5% of code)
pipeline.Stage("transform").Do = func(raw []byte) ([]byte, error) {
    return json.Marshal(processData(raw))  // Pure transformation
}

pipeline.Stage("validate").Do = func(data []byte) (bool, []string) {
    return validateData(data)  // Pure validation
}

// Framework provides EVERYTHING ELSE:
// - HTTP fetch with retry
// - Checkpoint save/load
// - Budget tracking
// - Progress reporting
// - Error handling and retry
// - Storage writes
// - Cost accounting
// - Resume from preemption
```

### Open Questions

**1. Compile-Time vs Runtime vs Both?**

| Approach | Pros | Cons |
|----------|------|------|
| **Compile-time** | Catches errors early, no runtime overhead | Can't handle dynamic contexts |
| **Runtime** | Flexible, handles dynamic requirements | Errors at runtime, overhead |
| **Both** | Best of both worlds | Complexity, potential conflicts |

Current thinking: **Both**, with compile-time as the primary and runtime as a safety net:
- Compile-time: Validate that declared contracts match context
- Runtime: Auto-wrap to satisfy derived requirements

**2. How Much Can Be Generated?**

| What | Generation Feasibility |
|------|----------------------|
| Checkpoint save/load | High — framework knows state shape |
| Retry logic | High — framework knows error types |
| Budget tracking | High — framework knows cost model |
| Progress UI | High — framework knows step count |
| Idempotency | Medium — needs domain hints |
| Validation | Low — domain-specific |
| Transformation | Low — pure business logic |

**3. Where's the Boundary?**

The framework should handle **infrastructure concerns**:
- I/O (network, disk, external services)
- Reliability (retry, checkpoint, resume)
- Resources (locks, budgets, leases)
- Observability (progress, cost, logs)

The developer handles **domain concerns**:
- Transformation logic (pure functions)
- Validation rules (domain-specific)
- Business decisions (what to do on failure)

**4. Migration Strategy**

Phased approach:
1. **Phase 1**: Add `ExecutionContext` and `DerivedRequirements` to pkg/dag
2. **Phase 2**: Build `dag.Flow()` DSL that generates contracts
3. **Phase 3**: Add generated wrappers for common patterns (checkpoint, retry, budget)
4. **Phase 4**: Migrate existing systems one at a time

### Free Testability: A Natural Consequence

A major benefit of the inverted model: **testability comes for free** from the DAG structure itself.

#### DAG-Level Mocking and Simulation

Because the DAG explicitly declares all I/O boundaries, testing becomes trivial:

```go
// Production: full DAG with real I/O
fullDAG := dag.Flow("process").
    Stage("fetch", dag.Imports(dag.URL("input"))).
    Stage("transform", ...).
    Stage("store", dag.Exports(dag.Storage("output")))

// Test: subset of DAG with mocked I/O
testDAG := fullDAG.
    Subset("transform").              // Only test transform stage
    MockImport("raw_data", testData). // Inject test input
    CaptureExport("processed")        // Capture output for assertions

result := testDAG.Execute(ctx)
assert.Equal(t, expected, result.Captured["processed"])
```

**What you get for free:**

| Capability | How It Works |
|------------|--------------|
| **Subset execution** | DAG knows dependencies — extract any subgraph |
| **Mock imports** | Imports are declared — inject fake data at boundaries |
| **Simulate exports** | Exports are declared — capture without real I/O |
| **Deterministic replay** | Record real I/O, replay in tests |
| **Chaos testing** | Inject failures at any declared I/O point |

```go
// Chaos testing: inject failures at declared boundaries
chaosDAG := fullDAG.
    InjectFailure("fetch", dag.NetworkError{}).  // Fail on first fetch
    InjectDelay("store", 5*time.Second)          // Slow storage

// Test retry behavior, timeout handling, etc.
```

#### Contract-Generated Tests

Since every node declares its contracts, **tests can be generated from the contracts themselves**:

```go
// Node declares contracts
stage := dag.Stage("protos",
    dag.Exports(dag.FilePattern("**/*.pb.go")),
    dag.Contract(dag.Deterministic, dag.Idempotent),
)

// Framework generates tests automatically:

// 1. Determinism test (flake detection)
func TestProtos_Deterministic(t *testing.T) {
    for i := 0; i < 100; i++ {
        result1 := stage.Execute(testInput)
        result2 := stage.Execute(testInput)
        assert.Equal(t, result1, result2, "run %d: output must be deterministic", i)
    }
}

// 2. Idempotency test
func TestProtos_Idempotent(t *testing.T) {
    stage.Execute(testInput)  // First run
    stage.Execute(testInput)  // Second run (should be no-op or same result)
    // Assert no side effects accumulated
}

// 3. Contract satisfaction test
func TestProtos_Contracts(t *testing.T) {
    contract := stage.Contract()
    assert.Contains(t, contract.Exports, dag.FilePattern("**/*.pb.go"))
    assert.True(t, contract.Satisfies(dag.Deterministic))
}
```

**Contract → Test mappings:**

| Contract | Generated Test |
|----------|----------------|
| `Deterministic` | Run N times, assert byte-identical output |
| `Idempotent` | Run twice, assert no accumulated side effects |
| `Preemptible` | Kill mid-execution, resume, assert correct completion |
| `BudgetCapped` | Run to budget exhaustion, assert graceful stop |
| `Checkpoint` | Kill at each step, resume, assert correct state |

```go
// Generated: Preemptibility test
func TestStage_Preemptible(t *testing.T) {
    ctx, cancel := context.WithCancel(context.Background())

    go func() {
        time.Sleep(randomDuration())  // Kill at random point
        cancel()
    }()

    stage.Execute(ctx, testInput)  // Interrupted

    // Resume from checkpoint
    result := stage.Execute(context.Background(), testInput)
    assert.Equal(t, expected, result)
}
```

#### Property-Based Testing from Contracts

Contracts become **property specifications** for property-based testing:

```go
// Contract declares: Exports(dag.Data("count", TypeIntish))
// Property: output is always an integer

func TestTransform_Properties(t *testing.T) {
    rapid.Check(t, func(t *rapid.T) {
        input := rapid.SliceOf(rapid.Byte()).Draw(t, "input")

        result := stage.Execute(input)

        // Property derived from contract
        _, ok := result.Export("count").(int)
        assert.True(t, ok, "count must be intish per contract")
    })
}
```

#### Integration Test Generation

The DAG structure enables **automatic integration test generation**:

```go
// Given a complete flow definition...
flow := dag.Flow("login",
    dag.Stage("auth", dag.Exports(dag.Data("tokens"))),
    dag.Stage("fetch", dag.Requires(dag.Data("tokens")), dag.Exports(dag.Data("secrets"))),
    dag.Stage("write", dag.Requires(dag.Data("secrets"))),
)

// Framework generates integration tests:

// 1. Happy path: all stages succeed
func TestLogin_HappyPath(t *testing.T) { ... }

// 2. Each stage failure: verify downstream handling
func TestLogin_AuthFails(t *testing.T) { ... }
func TestLogin_FetchFails(t *testing.T) { ... }
func TestLogin_WriteFails(t *testing.T) { ... }

// 3. Dependency satisfaction: verify data flows correctly
func TestLogin_TokensFlowToFetch(t *testing.T) { ... }
func TestLogin_SecretsFlowToWrite(t *testing.T) { ... }
```

#### The Testing Inversion

**Today**: Write code → figure out how to test it → write mocks manually

**Future**: Declare structure → tests are derived from contracts → fill in assertions

```go
// Developer declares (5 lines)
stage := dag.Stage("transform",
    dag.Imports(dag.Data("raw")),
    dag.Exports(dag.Data("processed")),
    dag.Contract(dag.Deterministic, dag.Idempotent),
)

// Framework generates (50+ lines of tests):
// - TestTransform_Deterministic (100 runs)
// - TestTransform_Idempotent (double-run)
// - TestTransform_ImportsRaw (contract check)
// - TestTransform_ExportsProcessed (contract check)
// - TestTransform_MockedInput (with test data)
// - TestTransform_CapturedOutput (output assertions)

// Developer adds domain-specific assertions (5 lines)
func TestTransform_BusinessLogic(t *testing.T) {
    result := generatedTestHarness.Execute(businessTestCase)
    assert.Equal(t, expectedBusinessResult, result)
}
```

This is the **testing inversion**: instead of writing tests that probe implementation details, you declare contracts and the framework generates tests that verify the contracts hold. Domain-specific tests become small additions on top of a generated foundation.

### Relation to Current Architecture

```
Current:
  L0 (DAG core) → L1 (Appendages) → L2 (Patterns) → Domain Code

Future:
  L0 (DAG core) → L1 (Appendages + ExecutionContext) → L1.5 (Flow DSL)
                                                          ↓
                                              ┌───────────┴───────────┐
                                              ↓                       ↓
                                    Generated L2 Patterns    Generated Tests
                                              ↓
                                    Pure Domain Functions
```

The key change: **L1.5 (Flow DSL)** sits between appendages and patterns, generating both the glue code that satisfies contracts AND the tests that verify those contracts hold.

---

## Appendix: DAG Framework Reference

> See **Part 0: The DAG Contract System** for full contract definitions.

### Layered Architecture (L0 → L1 → L2)

```
┌─────────────────────────────────────────────────────────────────┐
│ L2: Composed Patterns                                           │
│   Pipeline, RetryUntilSuccess, SpawnAndAwait, LeaseScope,      │
│   BudgetFlow, Freshen, Transform, Creation, Poll               │
├─────────────────────────────────────────────────────────────────┤
│ L1: Composable Appendages (Proto-defined)                      │
│   StateMachine, Lease, Phase, Resource/Claim                   │
├─────────────────────────────────────────────────────────────────┤
│ L0: Minimal Foundation                                          │
│   Node interface, DAG[T], Execute, ComputeWaves, Validate      │
└─────────────────────────────────────────────────────────────────┘
```

### Core Abstractions (L0)

```go
type Node interface {
    NodeID() string
    NodeDependsOn() []string
}

type DAG[T Node] struct { ... }
func (d *DAG[T]) Execute(ctx, opts, fn) error
func (d *DAG[T]) ComputeWaves() []Wave
func (d *DAG[T]) TopologicalOrder() []T
func (d *DAG[T]) Validate() []ValidationError
```

### Contract System (L0 extension)

```go
type Contractor interface {
    Contract() NodeContract
}

type NodeContract struct {
    Provides               []PrerequisiteID
    Requires               []PrerequisiteID
    Claims                 []Claim
    Exports                []DataRef
    Imports                []DataRef
    Invalidates            []RunnerResourceRef
    RequiresRunnerResource []RunnerResourceRef
    RequiresIntegration    []IntegrationCapabilityRef
}
```

### Composable Appendages (L1)

| Appendage | Purpose | Proto Location |
|-----------|---------|----------------|
| **StateMachine** | State tracking with transitions | `pkg/dag/state_machine.proto` |
| **Lease** | Ownership/claim tracking | `pkg/dag/lease.proto` |
| **Phase** | Lifecycle position | `pkg/dag/phase.proto` |
| **Resource/Claim** | Capacity semantics | `pkg/dag/resource.go` |

### Composed Patterns (L2)

| Pattern | Purpose | File |
|---------|---------|------|
| **Pipeline** | Fetch → Transform → Validate | `patterns/pipeline.go` |
| **RetryUntilSuccess** | Bounded retry with backoff | `patterns/retry.go` |
| **SpawnAndAwait** | Child project management | `patterns/spawn.go` |
| **LeaseScope** | Resource lifecycle (acquire/release) | `patterns/lease_scope.go` |
| **BudgetFlow** | Cost accounting across DAG | `patterns/budget.go` |
| **Loop** | Polling with outcomes | `patterns/loop.go` |

### Contract → System Mapping

| Contract Feature | make heal | make login | infra apply | OaaS |
|------------------|-----------|------------|-------------|------|
| **Prerequisites (Requires)** | `dependsOn` | `dependsOn` | `DependsOn` | `Dependency` proto |
| **Prerequisites (Provides)** | implicit | implicit | implicit | `OutputContract` |
| **Claims** | `resources.Write/Read` | - | `Locks` | `WorkspaceLock`, lease |
| **Exports** | file diffs | context setters | - | `OutputSlot` |
| **Imports** | - | context getters | - | dependency `output_slot` |
| **RequiresIntegration** | `requires` (tools) | - | `Permissions` | `RequiredCapabilities` |
| **Behavioral Requirements** | `contracts` (custom) | - | `Idempotency` | `SideEffectPolicy` |

### Documentation

- **Primary Reference**: `docs/pkg-dag.md`
- **Unified Spec**: `docs/UNIFIED_DAG_PHASE1_SPEC.md`
- **Patterns Guide**: `docs/architecture/dag-patterns.md`
- **Migration**: `docs/dag-migration-candidates.md`
- **Contract Enforcement**: `docs/dag-contract-enforcement.md`

### Code Locations

| Component | Path |
|-----------|------|
| Core DAG | `OaaS_v2/pkg/dag/dag.go` |
| Contracts | `OaaS_v2/pkg/dag/contract.go` |
| Data Flow | `OaaS_v2/pkg/dag/data_flow.go` |
| Prerequisites | `OaaS_v2/pkg/dag/prerequisite.go` |
| Resources | `OaaS_v2/pkg/dag/resource.go` |
| Requirements | `OaaS_v2/pkg/dag/requirements.go` |
| Patterns | `OaaS_v2/pkg/dag/patterns/` |
