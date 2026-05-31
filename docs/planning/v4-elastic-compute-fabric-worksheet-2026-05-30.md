# v4 Modeling DFS Worksheet A — Elastic compute fabric (fractal Upsert chain)

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §8 sign-off 2026-05-31 (proud-pike-680; PR #4095 split amend). **READY-FOR-IMPLEMENTATION-DISPATCH** once #4095 merges to main. Pair with **Worksheet B** (`v4-elastic-cache-interface-worksheet-2026-05-30.md`).
> **Date:** 2026-05-30 (split amend 2026-05-31)
> **Author:** sharp-wolf-824 (worker under proud-pike-680)
> **Dispatch anchor:** node://adhoc-2e6e2313-8a5 — exploration §4.0f **Worksheet A** (cases 1–8)
> **Authority doc:** `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0a–§4.0d (main)
> **Prerequisite:** Phase 1.4 `Upsert<T>` + Phase 1.5 CI schema worksheets §8 CLOSED. **No `.dag` landing in this slice.**

---

## Composition boundary (no cross-import with Worksheet B)

Compose with cache substrate **only** through:

- `ExecutionReceipt<T>.output: Outcome<ArtifactRef<T>>` (compute → artifact handle)
- Worksheet B's `CachedArtifactReceipt<T>.producer: ProducerReceipt<T>` citing `ExecutionReceiptRef<T>` on internal hits (no `import` of compute module in cache_interface)

**Forbidden:** `dsl/std/compute_fabric.dag` importing `cache_interface.dag` (or vice versa). `PerformanceReceipt` must not name `CacheInterfaceFacts` / `CacheStoreView` — cache state is an opaque summary string or lands in Worksheet B receipts.

---

## Mechanical dispatch rule

> **Compute-fabric implementation workers may dispatch per §6 after PR #4095 merges (Worksheet A §8 closed 2026-05-31).**

Worksheet B is a separate §8 gate. Do **not** merge either worksheet PR until both checklists match exploration §4.0f split on **main**.

---

## §10.0-adapted worksheet (Worksheet A only)

```text
Substrate class:        COMPUTE-FABRIC (dsl/std/compute_fabric.dag + v4.std mirror)
Representative failure:  ci.dag SelfHostedRunnerPool + host-named YAML; no WorkDemand /
                         satisfies; perf/cost receipts without confidence metadata.
Immediate local patch:   host: on CiUpsertStep; ComputeKind enum; worker_count on steps.
Why forbidden:           §4.0d + P2; parallel cache authority belongs in Worksheet B only.
DFS path:
  NEW: dsl/std/compute_fabric.dag
  CONSUME: patterns.dag Upsert<T>, types.dag, measure.dag, effects.dag
  CONSUME: v4.ci-schema UpsertInputRef, v4.change ChangeSet, v4.artifact ArtifactRef
  OUT OF SCOPE: cache_interface.dag, ci.dag migration
Falsification probe:   §5 cases 1–8 only.
```

---

## §1 Authoritative substrate catalog (Worksheet A)

**Symbol discipline:** `NonEmptyStr` brands in `dsl/std/`; `Symbol` in `v4.std` mirror only.

### 1.1 Layer 2 — executable unit + ingress chain

`ArtifactRef` is the scheduled form; ingress must remain auditable (exploration §4.0a Layer 2).

```dag
// v4.workflow.ci — CONSUME (Phase 1.5); not re-defined here.
// type UpsertInputRef = FileSet | SubstrateNodeSet | LensOutputRef | TestClaimRef | UpstreamUpsert

// v4.std.change — CONSUME
// type ChangeSet { ... }

// Ingress materialization (Layer 2 landing family):
fn artifact_refs_from_upsert_inputs(
  inputs: List<UpsertInputRef>,
  change: ChangeSet,
) -> List<ArtifactRef>
// Chain (auditable):
//   UpsertInputRef
//     → ChangeSet ∩ FileSetSelector / NodeQuery   (ingress: paths + node IDs)
//     → ArtifactRef                               (content-addressed snapshot)
//     → WorkUnit<T>.inputs

type WorkUnitId = NonEmptyStr where brand("WorkUnitId")

type WorkUnit<T> {
  id: WorkUnitId
  inputs: List<ArtifactRef>           // v4.std.artifact — post-ingress only
  action: WorkAction                  // Node until carrier lands
  output: ArtifactSpec<T>
  requirements: WorkDemand
}
```

### 1.2 Layer 3 — demand

**P2:** `ResourceEnvelope` is sole authority for cpu/gpu/memory/storage/network coordinates.

```dag
type ResourceEnvelope {
  cpu: Option<CpuRequirement>
  gpu: Option<GpuRequirement>
  memory: Option<MemoryRequirement>
  storage: Option<StorageRequirement>
  network: Option<NetworkRequirement>
}

type WorkDemand {
  resources: ResourceEnvelope
  os: Option<OperatingSystemRequirement>
  isolation: IsolationRequirement
  toolchains: List<ToolchainRequirement>
  parallelism: ParallelismShape
  data_locality: List<ArtifactLocalityRequirement>
  effects: List<EffectBoundary>
}

type CpuRequirement { min_cores: Int, architecture: Option<CpuArchitecture> }
type GpuRequirement { min_vram: ByteSize, runtimes: List<GpuRuntime> }
type MemoryRequirement { min_bytes: ByteSize }
type StorageRequirement { min_bytes: ByteSize, persistence: PersistenceKind }
type NetworkRequirement { egress: Option<NetworkEgressClass>, ambient_allowed: Bool }
type OperatingSystemRequirement { surface: OperatingSystemSurface }
type IsolationRequirement { boundary: IsolationBoundary }
type ToolchainRequirement { name: NonEmptyStr, version: NonEmptyStr }

// Compute-owned demand locality — NOT `PersistenceLocality` (Worksheet B / cache_interface.dag).
// Harness aligns demand vs cache row locality at schedule time; no cross-module type import.
type ComputeArtifactLocality
  = InProcess
  | PerRunnerColocation
  | PerHostColocation
  | CrossHostFetch

// Forward reference — same brand as Worksheet B `ArtifactIdentity.artifact_kind` (harness-aligned).
type ArtifactKindId = NonEmptyStr where brand("ArtifactKindId")

type ArtifactLocalityRequirement { artifact_kind: ArtifactKindId, locality: ComputeArtifactLocality }

type ToolchainCapability {
  name: NonEmptyStr
  version: NonEmptyStr
  env_isolation: ToolchainEnvIsolation
  build_wrapper: Option<BuildWrapperKind>
  linked_cache_interface: Option<CacheInterfaceId>  // opaque id — row lives in Worksheet B
}

type ToolchainEnvIsolation
  = SharedHomeAcrossJobs
  | PerJobCargoHome { cargo_home_var: NonEmptyStr, rustup_home_var: NonEmptyStr }
  | HermeticContainer

type BuildWrapperKind = CtrlBuild | BakedInImage | NoBuildWrapper

// Opaque forward reference — defined only in Worksheet B / cache_interface.dag.
type CacheInterfaceId = NonEmptyStr where brand("CacheInterfaceId")
```

### 1.3 Parallelism + budget

```dag
type ParallelismShape
  = SingleWorkItem
  | IndependentShards { shard_count: Int }
  | DependencyGraphParallel { graph: WorkGraph }
  | PartitionedReduce { partitioner: Partitioner, map: WorkAction, reduce: Reducer, laws: ReducerLaws }

type ReducerLaws {
  associative: Witness<Associativity>
  commutative: Option<Witness<Commutativity>>
  identity: Option<IdentityElement>
  idempotent: Option<Witness<Idempotency>>
}

type ExecutionBudget {
  expected_cost: SymbolicCost
  provider_model: ProviderCostModel
  watchdog: WatchdogLimit
}

type ProcessorCostModel { /* data rows per provider */ }
type ProviderCostModel { /* provider-indexed */ }
type CostModel { cost_class: CostClass, rates: ProviderCostModel }
type WatchdogLimit { max_wall: Duration }
```

### 1.4 Layer 4 — supply

```dag
type ProcessorKind
  = CpuProcessor { cpu: CpuFacts }
  | GpuProcessor { gpu: GpuFacts }
  | AcceleratorProcessor { accelerator: AcceleratorFacts }

type CpuFacts { architecture: CpuArchitecture, vendor: NonEmptyStr, model: NonEmptyStr, cores: Int, threads: Int, instruction_sets: List<InstructionSet> }
type GpuFacts { vendor: NonEmptyStr, model: NonEmptyStr, compute_capability: Option<NonEmptyStr>, memory: MemoryFacts, supported_runtimes: List<GpuRuntime> }
type AcceleratorFacts { vendor: NonEmptyStr, model: NonEmptyStr }
type MemoryDevice { capacity: ByteSize, memory_kind: MemoryKind, bandwidth: Option<Bandwidth> }
type StorageDevice { capacity: ByteSize, medium: StorageMedium, read_bandwidth: Option<Bandwidth>, write_bandwidth: Option<Bandwidth>, persistence: PersistenceKind }
type NetworkInterface { addressability: NetworkAddressability, bandwidth: Option<Bandwidth>, latency_class: Option<LatencyClass>, locality: NetworkLocality }
type OperatingSystemSurface { kernel: KernelFamily, distro_or_product: NonEmptyStr, version: NonEmptyStr, filesystem_semantics: FileSystemSemantics, process_semantics: ProcessSemantics }
type ExecutionSurface { os: OperatingSystemSurface, isolation: IsolationBoundary, container_runtime: Option<ContainerRuntime>, toolchains: List<ToolchainCapability>, mounted_storage: List<StorageMount>, network: List<NetworkInterface> }
type ComputeHost { identity: HostIdentity, processors: List<ProcessorKind>, memory: List<MemoryDevice>, storage: List<StorageDevice>, network_interfaces: List<NetworkInterface> }
type ComputeSupplyFacts { physical: ComputeHost, execution: ExecutionSurface, cost: Option<CostModel>, observed_performance: List<PerformanceReceipt> }
type ComputeOffer { provider: ProviderIdentity, supply: ComputeSupplyFacts, available_window: AvailabilityWindow, cost_quote: Option<CostEstimate>, constraints: List<ProviderConstraint> }

type ProviderConstraint
  = HostJobserverFifo { fifo_path: NonEmptyStr, token_cap: Int }
  | SharedHomeRoot { path: NonEmptyStr }
  | MaxConcurrentRunners { cap: Int }
  | OomBehavior { signal: OomSignalClass }

type AvailabilityWindow { open: LogicalTime, close: Option<LogicalTime> }
type CostEstimate { model: CostModel, expected_units: Float, currency: NonEmptyStr }

type MachineView { label: NonEmptyStr, host: HostIdentity }
fn machine_view(host: ComputeHost, surface: ExecutionSurface) -> MachineView
```

### 1.5 Eligibility

```dag
type DemandDimension
  = DemandCpu | DemandGpu | DemandMemory | DemandStorage | DemandNetwork
  | DemandOs | DemandIsolation | DemandToolchains | DemandParallelism
  | DemandDataLocality | DemandEffects

type MissingDemandFact { dimension: DemandDimension, required: NonEmptyStr }

type ComputeLeaseEligibility
  = Eligible { witness: ComputeLeaseWitness }
  | Rejected { reason: MissingDemandFact }

type ComputeLeaseWitness { provider: ProviderIdentity, satisfied: List<DemandDimension> }
type AllocationReceipt { provider: ProviderIdentity, scope: NonEmptyStr, acquired_at: LogicalTime }
type ComputeLease { offer: ComputeOffer, demand: WorkDemand, allocation: AllocationReceipt, eligibility: Witness<ComputeLeaseEligibility> }

fn satisfies(supply: ComputeSupplyFacts, demand: WorkDemand) -> Witness<ComputeLeaseEligibility>
```

### 1.6 Layer 5 — execution + receipts (confidence fields)

```dag
type ExecutionReceipt<T> {
  work: WorkUnit<T>
  lease: ComputeLease
  output: Outcome<ArtifactRef<T>>     // composition edge to Worksheet B
  performance: PerformanceReceipt
  cost: CostReceipt
  started_at: LogicalTime
  finished_at: LogicalTime
}

type MeasurementConfidence
  = SingleSample
  | Range { low: Float, high: Float }
  | DistributionSummary { mean: Float, variance: Float }

type PerformanceReceipt {
  provider: ProviderIdentity
  work_shape: WorkUnitId
  wall_duration: Duration
  cpu_seconds: Option<Float>
  sample_count: Int
  measurement_context: ExecutionSurface
  confidence: MeasurementConfidence
  cache_state_summary: Option<NonEmptyStr>   // opaque — no CacheInterface import
}

type PricingSource
  = VendorDocument { citation: NonEmptyStr }
  | ObservedBill { receipt_id: NonEmptyStr }
  | NegotiatedRate { contract_ref: NonEmptyStr }

type AmortizationScope
  = PerLease
  | PerCalendarMonth
  | PerWorkShape

type CostReceipt {
  provider: ProviderIdentity
  cost_class: CostClass
  amount: Option<Float>
  pricing_source: PricingSource
  amortization_scope: Option<AmortizationScope>
}
```

### 1.7 Supporting enums (compute module)

```dag
type CpuArchitecture = X86_64 | Aarch64 | Armv7 | Riscv64 | Wasm32
type KernelFamily = Linux | Darwin | WindowsNt | LinuxGuestOnWindows
type FileSystemSemantics = Posix | Ntfs | Apfs | WslPathTranslation
type ProcessSemantics = PosixProcess | WindowsProcess | HybridWslGuest
type NetworkLocality = Loopback | Lan | Internet | WslNat | WslBridged
type NetworkAddressability = Unicast | Multicast | Broadcast
type NetworkEgressClass = None | CratesIo | InternalMirror | Unrestricted
type IsolationBoundary = SharedHostHome | PerJobFilesystem | ContainerHermetic | Vm
type PersistenceKind = Ephemeral | Persistent | CachedMirror
type MemoryKind = Dram | Hbm | UnifiedShared
type StorageMedium = Nvme | Ssd | Hdd | NetworkAttached | EphemeralFs
type LatencyClass = UltraLow | Low | Medium | High
type GpuRuntime = Cuda | Rocm | Metal | OpenCl
type ContainerRuntime = Docker | Podman | Gvisor | Ubicloud | GcloudRun
type CostClass = OwnedMarginalZero | PerSecondBilled | Metered | DeveloperAttention
type OomSignalClass = KillProcess | Throttle | ReportOnly
type LogicalTime = NonEmptyStr where brand("LogicalTime")
type ByteSize = Measure<Memory, _>
type Bandwidth = Measure<DataRate, _>
type Duration = Measure<Time, _>
type HostIdentity = NonEmptyStr where brand("HostIdentity")
type ProviderIdentity = NonEmptyStr where brand("ProviderIdentity")
type StorageMount { mount_path: NonEmptyStr, device: StorageDevice, lifecycle: PersistenceKind }
```

---

## §2 Parser gates (Worksheet A)

| Gate | Requirement |
| ---- | ----------- |
| **P-CF-TYPE** | `dsl/std/compute_fabric.dag` parses §1 |
| **P-CF-WITNESS** | `Witness<C>` from v4 mirror; dsl uses 🟡 where needed |
| **P-CF-INGRESS** | `artifact_refs_from_upsert_inputs` types check against landed `UpsertInputRef` / `ChangeSet` |
| **P-CF-GENERIC** | `WorkUnit<T>`, `ExecutionReceipt<T>` generics |

---

## §3 M9 concept-home map (compute only)

| Concept | Home |
| ------- | ---- |
| Upsert canon | `dsl/std/patterns.dag` |
| Ingress selectors | `v4.workflow.ci` `UpsertInputRef` |
| Change frontier | `v4.std.change` `ChangeSet` |
| Artifact identity | `v4.std.artifact` `ArtifactRef` |
| Compute facts | **`dsl/std/compute_fabric.dag`** (NEW) |
| Demand artifact locality | **`ComputeArtifactLocality`** in compute_fabric (not `PersistenceLocality`) |
| Artifact kind on demand | **`ArtifactKindId`** (shared with Worksheet B / harness — not bare `NonEmptyStr`) |

---

## §4 Spot-fix register (Worksheet A)

| Pattern | Why forbidden |
| ------- | ------------- |
| `PersistenceLocality` on `WorkDemand` / `ArtifactLocalityRequirement` | Cache-owned enum (Worksheet B); use `ComputeArtifactLocality` |
| `artifact_kind: NonEmptyStr` on `ArtifactLocalityRequirement` | Use `ArtifactKindId` (shared with Worksheet B identity) |
| `ComputeKind` coproduct | §4.0d |
| `host:` on `CiUpsertStep` | Placement on result layer |
| `worker_count` on CI step | Use `ParallelismShape` |
| `import cache_interface` in compute_fabric | Composition boundary |
| `CacheStore` as compute authority | Worksheet B only |

---

## §5 Falsification cases 1–8

| # | Case | Proving types | Pass condition |
|---|------|---------------|----------------|
| 1 | srv1/srv2 rows | `data supply_srv1`, `supply_srv2: ComputeSupplyFacts` | No `host` on `CiUpsertStep` |
| 2 | Mac mini | `CpuFacts` + Darwin `OperatingSystemSurface` | Row without CI schema change |
| 3 | WSL | `LinuxGuestOnWindows`, `WslPathTranslation`, `WslNat`/`WslBridged` | `data supply_wsl` |
| 4 | gcloud/ubicloud | `ComputeSupplyFacts` + `container_runtime` | No run-mode enum |
| 5 | GPU | `WorkDemand.resources.gpu` | No `ComputeKind` |
| 6 | Storage-heavy | `resources.storage` + `resources.network` vs devices | `satisfies` Eligible |
| 7 | Multi-provider | `satisfies` | Same demand → multiple `Eligible` |
| 8 | Fail-closed | `Rejected { MissingDemandFact }` | Named dimension |

---

## §6 Landing order (Worksheet A)

```text
A1. §1.7 enums + measure carriers
A2. §1.4 supply records
A3. data supply_srv1 / supply_srv2 (§7)
A4. §1.2 WorkDemand + ResourceEnvelope
A5. §1.5 satisfies
A6. §1.1 ingress fn + WorkUnit<T>
A7. §1.6 ExecutionReceipt + confidence fields
A8. v4.std mirror
```

---

## §7 srv1/srv2 agenda

| Fact | Target |
|------|--------|
| Shared `$HOME` | `IsolationBoundary::SharedHostHome` |
| CARGO_HOME indirection | `ToolchainCapability.env_isolation` |
| ctrl-build | `ToolchainCapability.build_wrapper` |
| ctrl-jobserver FIFO | `ProviderConstraint::HostJobserverFifo` |
| sccache link | `ToolchainCapability.linked_cache_interface` → Worksheet B row id |
| Storage / `$RUNNER_TEMP` | `StorageDevice` + `StorageMount` |

---

## §8 Out of scope (Worksheet A)

- `dsl/std/cache_interface.dag` (Worksheet B)
- `ci.dag` / `ci.yml` migration
- INVARIANTS / THESIS / MODELING edits

---

## §9 Downstream brief (after §10 A)

Land `dsl/std/compute_fabric.dag` per §6. **MUST NOT** import cache module. Prove cases 1–8.

---

## §10 Manager approval checklist (Worksheet A) — CLOSED 2026-05-31

- [x] §1 signatures (ingress chain, ResourceEnvelope, confidence fields)
- [x] §5 cases 1–8
- [x] §6 landing order
- [x] Composition boundary with Worksheet B accepted
- [x] Implementation dispatch authorized (compute_fabric PR only, post-merge)

---

## Related artifacts

- `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0f Worksheet A
- `docs/planning/v4-elastic-cache-interface-worksheet-2026-05-30.md` (pair)
- `docs/planning/v4-ci-schema-worksheet-2026-05-30.md` (`UpsertInputRef`)
