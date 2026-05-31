# v4 Modeling DFS Worksheet — Elastic compute fabric + cache substrate (fractal Upsert chain)

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §8 sign-off 2026-05-31 (proud-pike-680; PR #4095). **READY-FOR-IMPLEMENTATION-DISPATCH** once #4095 merges to main.
> **Date:** 2026-05-30
> **Author:** sharp-wolf-824 (worker under proud-pike-680)
> **Dispatch anchor:** node://adhoc-2e6e2313-8a5 — PR #4091 §4.0f / §4.0g acceptance (worksheet-only slice)
> **Authority doc:** `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0–§4.0g (read on branch `session/deep-badger-883` until merged to main)
> **Prerequisite:** Phase 1.4 `Upsert<T>` type substrate (`docs/planning/v4-upsert-t-substrate-worksheet-2026-05-30.md` §8 CLOSED). Phase 1.5 CI schema (`docs/planning/v4-ci-schema-worksheet-2026-05-30.md` §8 CLOSED). **No substrate `.dag` landing in this slice.**

---

## Mechanical dispatch rule

> **Compute-fabric / cache-substrate implementation workers may dispatch per §6 Phase A–E after PR #4095 merges (§8 closed 2026-05-31).**

Same discipline as PR #3938 §10.0 / §11.1: the worksheet is reviewable authority; implementation PRs are downstream. Acceptance of the **implementation** lane is falsification cases 1–15 passing against landed types — not YAML wall-clock wins.

**This PR slice:** worksheet document only. **Escalate** proud-pike-680 on enum pressure (`ComputeKind`, `CacheKind`, run-mode coproducts) or any requirement to touch `ci.dag` / `INVARIANTS.md` / `THESIS.md` / `MODELING.md`.

---

## §10.0-adapted worksheet

```text
Substrate class:        COMPUTE-FABRIC + CACHE (general-purpose dsl/std + v4.std mirror;
                         fractal Upsert<T> chain layers 2–5 + cache rows per exploration §4.0a/§4.0g)
Representative failure:  ci.dag SelfHostedRunnerPool { host, arch, core_count, runner_count,
                         jobserver_token_cap } (L364–370) + shell-comment facts ($HOME, FIFO,
                         ctrl-build) are the only supply authority; GHA hashFiles(...) and
                         worker_count / timeout-minutes in YAML are heuristic placement/tuning;
                         no WorkDemand / CacheStore / satisfies homomorphism.
Immediate local patch:   Add host: Symbol on CiUpsertStep; extend SelfHostedRunnerPool with
                         cache_key; introduce ComputeKind = CPU | GPU | … or CacheKind = GHA |
                         sccache | …; hardcode srv1 in step create actions.
Why forbidden:           Re-introduces §4.0d forbidden shapes (host on CI step, kind enums,
                         authored cache keys, Machine primitive, silent scheduler fallback);
                         parallel authority vs content_hash + patterns.dag Upsert canon (P2).
DFS path:
  std/ authority (NEW — after worksheet §8):
    - dsl/std/compute_fabric.dag   — host/device/OS/execution/supply/demand/lease/receipt carriers
    - dsl/std/cache_substrate.dag  — CacheStore rows + cache dimension enums + CachedArtifactReceipt
    - dsl/std/patterns.dag         — Upsert<T> canon (CONSUME only)
    - dsl/std/types.dag            — ContentHash, branded ids
    - dsl/std/measure.dag          — ByteSize / bandwidth carriers (Quantity=Memory|Information)
    - dsl/std/effects.dag          — EffectBoundary (CONSUME)
  v4/ authority (CONSUME + mirror, not redefine):
    - src/v4/std/platform.dag      — Architecture, OperatingSystem (emit axis — NOT OS surface)
    - src/v4/std/witness.dag       — Witness<C>, Holds | Violates
    - src/v4/std/node.dag          — Symbol, content_hash projections
    - src/v4/lens/cost.dag         — SymbolicCost (ExecutionBudget.expected_cost)
    - src/v4/workflow/ci.dag       — CiUpsertStep<T> (Layer 1 — OUT OF SCOPE for landing PR)
  exploration / audit:
    - docs/planning/elastic-ci-redesign-exploration-2026-05-31.md §4.0–§4.0g
    - docs/audit/upsert-pattern-compiler-stray-2026-05-29.md
Deepest unsound boundary:
  Supply and cache are prose + host-named pool records; eligibility and cache identity are
  filesystem residue and hand-authored YAML — not structural facts over orthogonal coordinates.
Systemic fix:
  Bottom-up land §1 type catalog in dsl/std/ (+ v4.std mirror); populate srv1/srv2 as
  ComputeSupplyFacts data rows; populate cache roster as CacheStore data rows; land satisfies
  returning Witness<ComputeLeaseEligibility> with fail-closed Rejected; wire cache proof via
  CachedArtifactReceipt — then project ci.dag pools and workflow emission (later PRs).
Non-goals (§9):
  - ci.dag / ci.yml migration in substrate landing PR
  - INVARIANTS.md / THESIS.md / MODELING.md edits
  - Meta-scheduler / perf-per-cost optimizer Node
  - ComputeKind / CacheKind exclusive kind enums
Falsification probe:
  §5 table — cases 1–15 each name a proving type + observable pass condition.
Metric allowed only as secondary:
  Skipped CI minutes / wall-clock — after receipt + eligibility correctness, not worksheet acceptance.
```

---

## §1 Authoritative substrate catalog (canonical type signatures)

**Naming discipline:** Layer-3 demand carrier is **`WorkDemand`** (exploration §4.0f worksheet outputs). Exploration §4.0a `WorkUnit.requirements: ComputeDemand` is **naming drift** — landing PR renames to `WorkDemand` for single authority.

**Symbol discipline:** At `dsl/std/` landing, use `NonEmptyStr` brands or `String` where `v4.std.node.Symbol` is not yet importable cross-tree; `src/v4/std/compute_fabric.dag` mirror may use `Symbol`. Do not introduce a second `Symbol` authority in dsl/std.

**Coproduct discipline (MODELING.md contextual check):**
- **Allowed coproducts:** `ProcessorKind` (one physical device is exactly one kind), `ParallelismShape`, `KeyDerivation` / `PersistenceLocality` / … (each dimension is its own closed enum — not a `CacheKind` mega-enum), `ComputeLeaseEligibility` outcome partition.
- **Forbidden coproducts:** `ComputeKind`, `CacheKind`, `CiRunPolicy`, scheduler risk enums — resource *requirements* and cache *backends* are **record coordinates**.

### 1.1 Layer 2 — executable unit (projects from CI; lands after supply/demand)

```dag
// dsl/std/compute_fabric.dag (or v4.std — Layer 2 may stay v4-only until ArtifactRef substrate exists)

type WorkUnitId = NonEmptyStr where brand("WorkUnitId")

type WorkUnit<T> {
  id: WorkUnitId
  inputs: List<ArtifactRef>           // v4.std.artifact — content-addressed; not FilePath
  action: WorkAction                  // Node until WorkAction carrier lands
  output: ArtifactSpec<T>             // Node until ArtifactSpec<T> lands
  requirements: WorkDemand
}
```

**Gate:** `ArtifactRef` / `ArtifactSpec<T>` are **blocked** on v4 artifact identity worksheet if not already usable — Layer 2 types may land as 🟡 `Node` placeholders with dissolution trigger (same posture as `VerifyCheck = Node` in Upsert worksheet).

### 1.2 Layer 3 — demand (orthogonal coordinates)

```dag
type WorkDemand {
  compute: ComputeRequirement
  memory: MemoryRequirement
  storage: StorageRequirement
  network: NetworkRequirement
  os: Option<OperatingSystemRequirement>
  isolation: IsolationRequirement
  toolchains: List<ToolchainRequirement>
  parallelism: ParallelismShape
  data_locality: List<ArtifactLocalityRequirement>
  effects: List<EffectBoundary>       // dsl/std/effects.dag — CONSUME
}

// Scheduler-facing projection of multi-dimensional demand (exploration §4.0a).
// Coordinates — NOT ComputeKind coproduct.
type ResourceEnvelope {
  cpu: Option<CpuRequirement>
  gpu: Option<GpuRequirement>
  memory: Option<MemoryRequirement>
  storage: Option<StorageRequirement>
  network: Option<NetworkRequirement>
}

// Per-dimension requirement records (minimal landing — extend only with named consumer):
type ComputeRequirement { min_cores: Int, architecture: Option<CpuArchitecture> }
type MemoryRequirement { min_bytes: ByteSize }
type StorageRequirement { min_bytes: ByteSize, persistence: PersistenceKind }
type NetworkRequirement { egress: Option<NetworkEgressClass>, ambient_allowed: Bool }
type OperatingSystemRequirement { surface: OperatingSystemSurface }
type IsolationRequirement { boundary: IsolationBoundary }
type ToolchainRequirement { name: NonEmptyStr, version: NonEmptyStr }
type ArtifactLocalityRequirement { artifact_kind: NonEmptyStr, locality: PersistenceLocality }
```

### 1.3 Layer 3b — parallelism + budget (algebraic, not numeric tuning)

```dag
type ParallelismShape
  = SingleWorkItem
  | IndependentShards { shard_count: Int }   // count is declared shape, NOT worker_count on CI step
  | DependencyGraphParallel { graph: WorkGraph }  // WorkGraph = Node until graph substrate lands
  | PartitionedReduce {
      partitioner: Partitioner               // Node placeholder
      map: WorkAction
      reduce: Reducer
      laws: ReducerLaws
    }

type ReducerLaws {
  associative: Witness<Associativity>         // v4.std.witness — import in v4 mirror
  commutative: Option<Witness<Commutativity>>
  identity: Option<IdentityElement>
  idempotent: Option<Witness<Idempotency>>
}

type ExecutionBudget {
  expected_cost: SymbolicCost                 // v4.lens.cost — dsl landing uses Node + 🟡 until import path exists
  provider_model: ProviderCostModel
  watchdog: WatchdogLimit
}

type ProcessorCostModel { /* per-provider rate table — Node or data rows in landing PR */ }
type ProviderCostModel { /* provider-indexed ProcessorCostModel */ }
type WatchdogLimit { max_wall: Duration }    // Duration = Measure<Time, _> from std.measure
```

### 1.4 Layer 4 — supply (facts, not machines)

```dag
// Device-kind coproduct — ALLOWED (one processor ∈ one kind).
type ProcessorKind
  = CpuProcessor { cpu: CpuFacts }
  | GpuProcessor { gpu: GpuFacts }
  | AcceleratorProcessor { accelerator: AcceleratorFacts }

type CpuFacts {
  architecture: CpuArchitecture             // align with v4.std.platform.Architecture where possible
  vendor: NonEmptyStr
  model: NonEmptyStr
  cores: Int
  threads: Int
  instruction_sets: List<InstructionSet>     // closed enum per landing PR
}

type GpuFacts {
  vendor: NonEmptyStr
  model: NonEmptyStr
  compute_capability: Option<NonEmptyStr>
  memory: MemoryFacts
  supported_runtimes: List<GpuRuntime>
}

type AcceleratorFacts { vendor: NonEmptyStr, model: NonEmptyStr }

type MemoryFacts { devices: List<MemoryDevice> }
type MemoryDevice {
  capacity: ByteSize
  memory_kind: MemoryKind
  bandwidth: Option<Bandwidth>
}

type StorageDevice {
  capacity: ByteSize
  medium: StorageMedium
  read_bandwidth: Option<Bandwidth>
  write_bandwidth: Option<Bandwidth>
  persistence: PersistenceKind
}

type NetworkInterface {
  addressability: NetworkAddressability
  bandwidth: Option<Bandwidth>
  latency_class: Option<LatencyClass>
  locality: NetworkLocality
}

// Richer than v4.std.platform.OperatingSystem — kernel + FS + process semantics.
type OperatingSystemSurface {
  kernel: KernelFamily
  distro_or_product: NonEmptyStr
  version: NonEmptyStr
  filesystem_semantics: FileSystemSemantics   // includes wsl_path_translation row for case 3
  process_semantics: ProcessSemantics
}

type ExecutionSurface {
  os: OperatingSystemSurface
  isolation: IsolationBoundary
  container_runtime: Option<ContainerRuntime>
  toolchains: List<ToolchainCapability>
  mounted_storage: List<StorageMount>
  network: List<NetworkInterface>
  // ctrl-build / ctrl-jobserver wiring lives HERE as ToolchainCapability + env facts,
  // NOT as a separate cache layer (exploration §4.0g pairing).
}

type ComputeHost {
  identity: HostIdentity                     // branded NonEmptyStr — NOT eligibility authority
  processors: List<ProcessorKind>
  memory: List<MemoryDevice>
  storage: List<StorageDevice>
  network_interfaces: List<NetworkInterface>
}

type ComputeSupplyFacts {
  physical: ComputeHost
  execution: ExecutionSurface
  cost: Option<CostModel>
  observed_performance: List<PerformanceReceipt>
}

type ComputeOffer {
  provider: ProviderIdentity
  supply: ComputeSupplyFacts
  available_window: AvailabilityWindow
  cost_quote: Option<CostEstimate>
  constraints: List<ProviderConstraint>
}
```

**Projection only (dashboards):**

```dag
type MachineView { label: NonEmptyStr, host: HostIdentity }
fn machine_view(host: ComputeHost, surface: ExecutionSurface) -> MachineView
```

### 1.5 Matching + lease (structural eligibility)

```dag
type ComputeLeaseEligibility
  = Eligible { witness: ComputeLeaseWitness }
  | Rejected { reason: MissingDemandFact }    // fail-closed — case 8

type MissingDemandFact {
  dimension: DemandDimension                 // closed enum of WorkDemand field names
  required: NonEmptyStr                      // human-readable fact description
}

type ComputeLease {
  offer: ComputeOffer
  demand: WorkDemand
  allocation: AllocationReceipt
  eligibility: Witness<ComputeLeaseEligibility>
}

fn satisfies(
  supply: ComputeSupplyFacts,
  demand: WorkDemand
) -> Witness<ComputeLeaseEligibility>
```

**`satisfies` landing posture:** Phase-1 implementation may return `Rejected` stubs for unimplemented dimensions; **must not** return silent `Eligible` on missing facts. Full homomorphism is incremental; falsification case 8 is the gate.

### 1.6 Layer 5 — execution + performance receipts

```dag
type ExecutionReceipt<T> {
  work: WorkUnit<T>
  lease: ComputeLease
  output: Outcome<ArtifactRef<T>>            // Outcome from std — or v4 verdict carrier
  performance: PerformanceReceipt
  cost: CostReceipt
  started_at: LogicalTime
  finished_at: LogicalTime
}

type PerformanceReceipt {
  provider: ProviderIdentity
  work_shape: WorkUnitId
  wall_duration: Duration
  cpu_seconds: Option<Float>
  cache_outcome: Option<CacheOutcomeSummary>
}

type CostReceipt {
  provider: ProviderIdentity
  cost_class: CostClass
  amount: Option<Float>
}
```

### 1.7 Cache substrate (`dsl/std/cache_substrate.dag`)

**Dimension enums** — each is its own closed set (NOT rolled into `CacheKind`):

```dag
type CacheStoreId = NonEmptyStr where brand("CacheStoreId")

type KeyDerivation
  = ContentAddressedByValue
  | HandAuthoredString                        // GHA today — projection target for content_hash
  | NativeInternalHash                        // sccache / Cargo fingerprint — opaque at L0

type ValueShape
  = RawBytes | StructuredArtifact | TarArchive | FileTree

type PersistenceLocality
  = InProcess
  | PerRunnerFilesystem
  | PerHostFilesystem
  | CrossHostNetwork

type EvictionPolicy
  = Ttl { days: Int }
  | Lru | SizeBounded { cap_bytes: ByteSize }
  | Never | Manual

type AtomicityModel
  = PerFile | WriteThenRename | WriteThenCommit | TwoPhase

type AuthScope
  = None | FilesystemPerms | ApiKey | NetworkAcl

type ReadLatencyClass
  = InProcessNs | LocalDiskUs | LanMs | WanTensMs

// Orthogonal-fact record — NOT CacheKind coproduct.
type CacheStore {
  identity: CacheStoreId
  key_space: KeyDerivation
  value_space: ValueShape
  locality: PersistenceLocality
  eviction: EvictionPolicy
  atomicity: AtomicityModel
  auth: AuthScope
  read_latency: ReadLatencyClass
}

// Wrong-cache-hit protection (case 14).
type CachedArtifactReceipt<T> {
  store: CacheStoreId
  subject_digest: ContentHash                  // std/types.dag
  value_digest: ContentHash
  producer: ExecutionReceipt<T>
}

type CacheLookupResult<T>
  = Hit { receipt: CachedArtifactReceipt<T> }
  | Miss
  | RejectedHit { reason: NonEmptyStr }       // bare blob / digest mismatch — case 14
```

**Canonical data rows (landing PR family — names are projections, not primitives):**

| `CacheStoreId` | `key_space` | `locality` | Falsification case |
|----------------|-------------|------------|-------------------|
| `gha_actions_cache` | `HandAuthoredString` | `CrossHostNetwork` | 9 |
| `sccache_l1` | `NativeInternalHash` | `PerHostFilesystem` | 10 |
| `buildbuddy_cas` | `ContentAddressedByValue` | `CrossHostNetwork` | 11 |
| `cargo_target_l0` | `NativeInternalHash` | `PerRunnerFilesystem` | 12 |
| `cargo_registry_l1` | `NativeInternalHash` | `PerHostFilesystem` | 13 (partial) |
| `rustup_toolchain_l1` | `HandAuthoredString` | `PerHostFilesystem` | 13 (partial) |

**Fractal key projection (all layers):**

```dag
fn cache_key_for_upsert<T>(subject: Node, store: CacheStore) -> ContentHash
// Authority: content_hash(canonical Upsert<T> subject) projected per store.key_space (§4.0g table).
```

### 1.8 Supporting closed enums (landing PR bundles with §1.4–1.7)

```dag
type CpuArchitecture = X86_64 | Aarch64 | Armv7 | Riscv64 | Wasm32   // map to/from v4.std.platform.Architecture
type KernelFamily = Linux | Darwin | WindowsNt | LinuxGuestOnWindows
type FileSystemSemantics = Posix | Ntfs | Apfs | WslPathTranslation
type ProcessSemantics = PosixProcess | WindowsProcess | HybridWslGuest
type NetworkLocality = Loopback | Lan | Internet | WslNat | WslBridged
type IsolationBoundary = SharedHostHome | PerJobFilesystem | ContainerHermetic | Vm
type InstructionSet = /* landing PR: avx2, neon, … as needed for srv1/srv2 + Mac */
type ByteSize = Measure<Memory, _>           // std.measure — single numeric authority
type Bandwidth = Measure<DataRate, _>
type Duration = Measure<Time, _>
type HostIdentity = NonEmptyStr where brand("HostIdentity")
type ProviderIdentity = NonEmptyStr where brand("ProviderIdentity")
```

---

## §2 Parser / substrate prerequisites (explicit gates)

| Gate | Owner | Requirement |
| ---- | ----- | ------------- |
| **P-ECF-TYPE** | Modeling DFS + landing PR | §1 types parse in `dsl/std/compute_fabric.dag` + `dsl/std/cache_substrate.dag` |
| **P-ECF-WITNESS** | v4 substrate | `Witness<C>` used only from `v4.std.witness` in v4 mirror; dsl/std uses `Node` + 🟡 for proof fields until cross-import lands |
| **P-ECF-GENERIC** | Parser | `WorkUnit<T>`, `ExecutionReceipt<T>`, `CachedArtifactReceipt<T>` require generic type params on records — same gate family as `Upsert<T>` |
| **P-ECF-MEASURE** | std.measure | `ByteSize` / `Duration` via `Measure<Q,S>` — no parallel `Int` byte fields |
| **P-ECF-SYMBOLIC-COST** | T-12 / lens | `ExecutionBudget.expected_cost: SymbolicCost` — v4 mirror only until dsl import path exists |
| **P-ECF-DATA-ROWS** | Modeling | `data ci_srv1_supply: ComputeSupplyFacts` style rows parse as module-level data (existing ci.dag pattern) |

**Manager ruling (proposed):** Substrate landing **DONE** when **P-ECF-TYPE** + falsification probes §5 (1–15) pass on landed types/data rows. `satisfies` may be partial if case 8 fail-closed is proven for at least one dimension; case 7 may use stub offers until all five providers are populated.

---

## §3 DFS concept-home map (M9)

```text
Concept                    | Home (authoritative)                 | Action
---------------------------|--------------------------------------|----------------------------------
Upsert fractal canon         | dsl/std/patterns.dag                 | CONSUME — verify/satisfy/create/resolve
Cache identity digest        | dsl/std/types.dag ContentHash          | CONSUME — subject_digest
Effect boundaries            | dsl/std/effects.dag                  | CONSUME — WorkDemand.effects
Dimensional bytes/time       | dsl/std/measure.dag                  | CONSUME — ByteSize, Duration
Compute device facts         | dsl/std/compute_fabric.dag           | DEFINE §1.4
Work demand / envelope       | dsl/std/compute_fabric.dag           | DEFINE §1.2
Eligibility homomorphism     | dsl/std/compute_fabric.dag satisfies | DEFINE §1.5
Cache store rows             | dsl/std/cache_substrate.dag          | DEFINE §1.7
Cache proof receipt          | dsl/std/cache_substrate.dag          | DEFINE §1.7
Proof carrier                | v4.std.witness Witness<C>            | IMPORT in v4 mirror only (Phase 1)
Rust target arch/OS enums    | v4.std.platform Architecture/OS      | PROJECT — do not duplicate as sole OS model
CI step semantics            | v4.workflow.ci CiUpsertStep<T>       | OUT OF SCOPE — consume later
Runner pool prose            | ci.dag SelfHostedRunnerPool          | DISSOLVE after supply rows land
Symbolic cost                | v4.lens.cost SymbolicCost            | CONSUME for ExecutionBudget
```

**New files (worksheet authority):**

1. `dsl/std/compute_fabric.dag` — §1.2–1.6, §1.8
2. `dsl/std/cache_substrate.dag` — §1.7
3. `src/v4/std/compute_fabric.dag` — mirror + `Witness`/`SymbolicCost` wired types
4. `src/v4/std/cache_substrate.dag` — mirror + `CachedArtifactReceipt` proof wiring

---

## §4 Spot-fix register (forbidden — grep gate)

| Pattern | Why forbidden | Escalate if pressured |
| ------- | ------------- | --------------------- |
| `ComputeKind = CPU \| GPU \| …` | §4.0d — denies multi-coordinate workloads | proud-pike-680 |
| `CacheKind = GHA \| sccache \| …` | §4.0d — denies 4/4 locality×keying grid | proud-pike-680 |
| `host:` / `runner_label:` on `CiUpsertStep<T>` | Placement on result layer | proud-pike-680 |
| `worker_count: Int` on CI step | Authored tuning — use `ParallelismShape` | — |
| `CiRunPolicy` / run-mode enums | Heuristic for missing facts | proud-pike-680 |
| `cache_key:` / `hashFiles(...)` authority | Must be `content_hash` projection | — |
| `type Machine` primitive | Use `ComputeHost` + `ExecutionSurface` | — |
| `host: Symbol` as satisfies input | Eligibility over fact bundles only | — |
| Bare cache hit without `CachedArtifactReceipt` | Case 14 wrong-hit class | — |
| `Machine` / pool-only supply in ci.dag | Parallel authority post-substrate | ci.dag touch = escalate |
| Substrate types only in `ci.dag` | CI cannot own general compute/cache canon | proud-pike-680 |
| Edits to INVARIANTS / THESIS / MODELING | Out of scope for this lane | proud-pike-680 |

---

## §5 Falsification cases 1–15 → proving types

| # | Case | Proving types / functions | Pass condition (observable) |
|---|------|---------------------------|----------------------------|
| 1 | srv1/srv2 as supply rows | `ComputeSupplyFacts`, `data supply_srv1`, `data supply_srv2` | Rows parse; **grep** `CiUpsertStep` has no `host` / pool fields; pools become projections |
| 2 | Mac mini row | `CpuFacts` (Apple Silicon), `OperatingSystemSurface` (Darwin) | `data supply_mac_mini: ComputeSupplyFacts` without `CiUpsertStep` schema change |
| 3 | WSL row | `KernelFamily::LinuxGuestOnWindows`, `FileSystemSemantics::WslPathTranslation`, `NetworkLocality::WslNat \| WslBridged` | `data supply_wsl: ComputeSupplyFacts` encodes explicit guest facts |
| 4 | gcloud / ubicloud container | `ComputeSupplyFacts` + `ExecutionSurface.container_runtime` | New `data` rows; **grep** no `CiRunPolicy` / run-mode enum in chain |
| 5 | GPU provider | `ResourceEnvelope.gpu: Option<GpuRequirement>` | GPU demand expressible; **grep** no `ComputeKind` |
| 6 | Storage-heavy provider | `WorkDemand.storage`, `WorkDemand.network`, `StorageDevice[]`, `NetworkInterface[]` on supply | `satisfies` Eligible when devices meet thresholds |
| 7 | One demand, many providers | `satisfies`, `Witness<ComputeLeaseEligibility>` | Same `WorkDemand` → `Eligible` for srv1, srv2, gcloud rows where facts match |
| 8 | Ineligible fail-closed | `Rejected { reason: MissingDemandFact }` | No `Eligible` without witness; reason names dimension |
| 9 | GHA Actions Cache row | `CacheStore` id `gha_actions_cache`, `cache_key_for_upsert` | Emitted workflow has no `hashFiles(...)` authority (downstream emit PR) |
| 10 | sccache row | `CacheStore` `sccache_l1`, `NativeInternalHash` | Fractal Upsert at module-emit grain; sccache internal hash not exported |
| 11 | BuildBuddy CAS | `buildbuddy_cas`, `ContentAddressedByValue` | `ctrl-build --remote` path uses store row; key = content hash 1:1 |
| 12 | Cargo `target/` L0 | `cargo_target_l0`, `PerRunnerFilesystem` | Ephemeral; lease teardown clears; never cache authority |
| 13 | New backend = one row | `CacheStore` data table | Add `data cache_mac_mini_sccache: CacheStore` — no `Upsert<T>` / `WorkUnit` type change |
| 14 | Wrong-cache-hit | `CachedArtifactReceipt`, `CacheLookupResult::RejectedHit` | Lookup without matching `ExecutionReceipt.work` digest → rejected |
| 15 | Orthogonal cache facts | `CacheStore` record + dimension enums | All four §4.0g grid cells inhabited by rows; **grep** no `CacheKind` |

---

## §6 Landing order (bottom-up — mandatory)

```text
Phase A — foundations (no provider rows yet)
  A1. measure-backed ByteSize, Bandwidth, Duration in compute_fabric imports
  A2. Closed enums: KernelFamily, FileSystemSemantics, CpuArchitecture, …
  A3. Device records: CpuFacts, MemoryDevice, StorageDevice, NetworkInterface
  A4. ProcessorKind coproduct + ComputeHost + OperatingSystemSurface + ExecutionSurface

Phase B — srv1/srv2 concrete supply (agenda §7)
  B1. data supply_srv1: ComputeSupplyFacts — all facts from §7 table
  B2. data supply_srv2: ComputeSupplyFacts
  B3. machine_view projections (optional fn)

Phase C — demand + eligibility
  C1. WorkDemand + ResourceEnvelope + requirement records
  C2. ParallelismShape + ReducerLaws (Witness fields 🟡 in dsl)
  C3. satisfies + ComputeLeaseEligibility + MissingDemandFact
  C4. ExecutionBudget (SymbolicCost in v4 mirror)

Phase D — cache substrate
  D1. Cache dimension enums (§1.7)
  D2. CacheStore record
  D3. data rows: gha, sccache, buildbuddy, cargo_target, registry, rustup
  D4. CachedArtifactReceipt + CacheLookupResult + cache_key_for_upsert

Phase E — chain completion (may trail B–D)
  E1. ComputeOffer, ComputeLease, PerformanceReceipt, CostReceipt
  E2. WorkUnit<T>, ExecutionReceipt<T> (ArtifactRef gates)
  E3. v4.std mirror files + import wiring

Phase F — downstream (separate PRs — NOT this worksheet slice)
  F1. ci.dag pool dissolution → supply row projection
  F2. workflow emission: cache keys, worker_count removal
  F3. satisfies tests in v4/test/claim/
```

---

## §7 Bottom-up srv1/srv2 agenda (concrete facts before abstraction)

Land **`data supply_srv1`** / **`data supply_srv2`** only after the §1.4 carriers exist. Each row must encode (from exploration §4.0 intro table + operator incidents):

| Fact | `ComputeSupplyFacts` field target |
|------|-----------------------------------|
| Shared `$HOME` across ephemeral workers | `ExecutionSurface` + `IsolationBoundary::SharedHostHome` (srv1/srv2 only) |
| Per-job `CARGO_HOME` / `RUSTUP_HOME` indirection | `ToolchainCapability.env_isolation` fact |
| `ctrl-build` wrapper | `ToolchainCapability` row (sccache, jobs cap, memory caps, BuildBuddy opt-in) |
| `ctrl-jobserver` + host FIFO | `ExecutionSurface` provider-private `ProviderConstraint` data — path in row, not in step `create` |
| sccache identity / location | `CacheStore` row `sccache_l1` + link from `ExecutionSurface` |
| Storage topology / `$RUNNER_TEMP` | `StorageDevice` + `StorageMount` lifecycle facts |
| Runner ephemeral naming | `ProviderConstraint` or `AllocationReceipt` — not `host` on CI step |
| Memory / cgroup | `MemoryDevice` + `ExecutionSurface` OOM semantics fact |
| OS / kernel / arch | `OperatingSystemSurface` + `CpuFacts.architecture` |
| Failure modes (FIFO race, rustup clobber, swap) | `data self_hosted_failure_mode_*` rows referenced from supply |

**Cross-provider rule:** Fields that exist only on srv1/srv2 (shared `$HOME`, host FIFO) **must not** appear on generic `WorkDemand` or `CiUpsertStep` — they are supply-side facts (cases 1, 4).

---

## §8 Out of scope (explicit)

| Item | Reason |
| ---- | ------ |
| `src/v4/workflow/ci.dag` edits | CI migration is Phase F — separate dispatch after substrate |
| `ci.yml` / GHA YAML | Emission consumer — case 9 proven in emit PR |
| `INVARIANTS.md`, `THESIS.md`, `MODELING.md` | Operator/doc lane — escalate |
| Meta-scheduler / perf-per-cost optimizer | Exploration §4.0 — separate Node after providers modeled |
| `ComputeFabric` Node + `locality_hint` sketches | Candidate vocabulary only until five providers landed |
| Substrate implementation in worksheet PR | Manager GO: worksheet-only slice |
| SG-class emit / M1 rustc | Separate DFS worksheets |
| `ComputeKind` / `CacheKind` enum pressure | Escalate manager — forbidden §4.0d |

---

## §9 Downstream worker brief (after §10 approval)

```text
Land §6 Phase A→D in dsl/std/ (+ v4 mirror per §3).

MUST:
  - Follow §1 signatures; §6 landing order
  - Populate srv1/srv2 + cache roster data rows before satisfies claims completeness
  - Prove §5 cases with tests or grep gates as indicated
  - Mark 🟡 any Node-placeholder fields with named dissolution triggers

MUST NOT:
  - Any §4 forbidden pattern
  - ci.dag / ci.yml / INVARIANTS / THESIS / MODELING in substrate PR
  - ComputeKind / CacheKind / run-mode enums

Escalate proud-pike-680:
  - Enum pressure for resource or cache "kinds"
  - Required ci.dag change in same PR as substrate
  - Parser gate P-ECF-* cannot clear without load-bearing stage touch
```

---

## §10 Manager approval checklist (proud-pike-680) — CLOSED 2026-05-31

- [x] §1 canonical signatures approved (WorkDemand naming, ProcessorKind vs ResourceEnvelope discipline)
- [x] §4 forbidden register accepted (ComputeKind / CacheKind escalation path)
- [x] §5 falsification 1–15 mapping accepted
- [x] §6 landing order accepted (bottom-up srv1/srv2 before abstraction consumers)
- [x] §8 out-of-scope boundary accepted
- [x] Parser gates §2 split (P-ECF-TYPE vs Witness/SymbolicCost deferral) accepted
- [x] Substrate implementation worker dispatch authorized (separate PR per §6 A–E)

---

## Related artifacts

- `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0–§4.0g (PR #4091)
- `docs/planning/v4-upsert-t-substrate-worksheet-2026-05-30.md`
- `docs/planning/v4-ci-schema-worksheet-2026-05-30.md`
- `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`
- `dsl/std/patterns.dag` — Upsert\<T\> canon
- `src/v4/workflow/ci.dag` L364–420 — `SelfHostedRunnerPool` (dissolution target)
- `src/v4/std/platform.dag` — Architecture / OperatingSystem (projection source, not duplicate OS surface)
