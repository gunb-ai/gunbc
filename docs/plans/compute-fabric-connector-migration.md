# compute_fabric connector — vocabulary relocation tracking

> Plan doc. Tracks every symbol removed from `product.compute_fabric` in the connector remodel ([PR #5889](https://github.com/gunb-ai/gunbc/pull/5889)). Framing: the connector is the thin matching law (need × opportunity → connection); the removed vocabulary becomes the connector's grounding once it relocates to its single-authority home. Disposition column: **RELOCATE** = has live consumers, needs a new home before #5889 can merge; **DELETE** = fabric-internal with no external consumers. Status checkboxes track relocation/deletion of each group.
>
> Owner: **keen-dove-772** (sequencing decision). Dissolution trigger: all rows resolved (every RELOCATE landed at its proposed home, every DELETE confirmed removed).

---

## Network / locality

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `NetworkLocality` (+ `Loopback`, `Lan`, `Internet`, `WslNat`, `WslBridged`, `DockerBridge`, `TailscaleOverlay`) | `fleet_intent_network.dag`, `network_topology.dag`, `access_iam_validation_test.dag` | RELOCATE | `extdeps/network/topology.dag` or `product/network_topology.dag` | [ ] |
| `NetworkAddressability` | `fleet_intent_network.dag` | RELOCATE | Same as `NetworkLocality` | [ ] |
| `NetworkEgressClass` | `fleet_intent_network.dag` | RELOCATE | Same as `NetworkLocality` | [ ] |
| `NetworkInterface` | `fleet_intent_network.dag` | RELOCATE | Same as `NetworkLocality` | [ ] |

---

## Isolation / container

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `IsolationBoundary` (+ `SharedHostHome`, `PerJobFilesystem`, `ContainerHermetic`, `Vm`) | `fleet_intent.dag`, `fleet_container.dag`, `ci_fleet.dag` | RELOCATE | `product/fleet_container.dag` or `extdeps/container/types.dag` | [ ] |
| `ContainerRuntime` (`Docker`, `Podman`, `Gvisor`, `Ubicloud`, `GcloudRun`) | `fleet_intent.dag`, `fleet_container.dag` | RELOCATE | `extdeps/container/types.dag` | [ ] |
| `PersistenceKind` (`Ephemeral`, `Persistent`, `CachedMirror`) | `fleet_intent.dag` (via `StorageDevice`) | RELOCATE | `product/fleet_intent.dag` or `extdeps/storage/types.dag` | [ ] |

---

## Storage

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `StorageDevice` (+ `NvmeDevice`, `BareDevice` variants) | `fleet_intent.dag`, `fleet_intent_storage_witness_test.dag` | RELOCATE | `extdeps/storage/types.dag` (catalog rows already there) | [ ] |
| `StorageMedium` (`Nvme`, `Ssd`, `Hdd`, `NetworkAttached`, `EphemeralFs`) | `fleet_intent.dag` | RELOCATE | `extdeps/storage/types.dag` | [ ] |
| `StorageMount` | `fleet_intent.dag` | RELOCATE | `product/fleet_intent.dag` | [ ] |
| `storage_device_capacity`, `storage_device_medium`, `storage_device_read_bandwidth`, `storage_device_write_bandwidth`, `storage_device_pcie_link`, `storage_device_serial`, `nvme_storage_device` | `fleet_intent.dag`, `fleet_intent_storage_witness_test.dag` | RELOCATE | Co-locate with `StorageDevice` | [ ] |

---

## Memory / GPU / accelerator

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `MemoryKind` (`Dram`, `Hbm`, `UnifiedShared`) | `ci_fleet.dag` | RELOCATE | `extdeps/memory/types.dag` | [ ] |
| `MemoryFacts`, `MemoryDevice` | `ci_fleet.dag`, `fleet_intent.dag` | RELOCATE | `extdeps/memory/types.dag` | [ ] |
| `GpuFacts`, `GpuComputeCapability`, `GpuRuntime`, `gpu_compute_capability_sm_label` | `hardware_selection.dag` | RELOCATE | `extdeps/gpu/types.dag` | [ ] |
| `AcceleratorFacts` | `hardware_selection.dag` | RELOCATE | `extdeps/gpu/types.dag` | [ ] |
| `LatencyClass` | `fleet_intent.dag` | RELOCATE | `extdeps/network/topology.dag` or `std/measure` | [ ] |

---

## Money / cost

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `MoneyMicros`, `money_micros`, `money_micros_count` | `hardware_selection.dag` | RELOCATE | `std/measure.dag` (`MoneyAmount<Micro>` alias already there; promote constructor/accessor) | [ ] |
| `CostClass` (`OwnedMarginalZero`, `PerSecondBilled`, `Metered`, `DeveloperAttention`) | `ci_fleet.dag` | RELOCATE | `product/ci_fleet.dag` or `extdeps/provider/cost.dag` | [ ] |
| `ProviderIdentity` | `ci_fleet.dag`, `ci_budget_tree.dag`, `runner_spec_from_offer.dag` | RELOCATE | `product/ci_fleet.dag` or `extdeps/provider/identity.dag` | [ ] |
| `CostModel`, `ProviderCostModel`, `CostEstimate`, `AvailabilityWindow` | `ci_fleet.dag` | RELOCATE | `product/ci_fleet.dag` | [ ] |

---

## Compute host / offer / supply

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `ProcessorKind` (`CpuProcessor`, `GpuProcessor`, `AcceleratorProcessor`) | `ci_fleet.dag`, `fleet_intent.dag` | RELOCATE | `extdeps/cpu/types.dag` | [ ] |
| `ExecutionSurface` | `ci_fleet.dag`, `fleet_intent.dag`, `host_effect.dag` | RELOCATE | `product/fleet_intent.dag` | [ ] |
| `ComputeHost` | `host_effect.dag`, `host_effect_realize.dag`, `ci_fleet.dag`, `fleet_intent.dag`, `ci_runner_placement.dag`, `fleet_host_budget.dag`, `runner_spec_from_offer.dag`, `ci_budget_tree.dag` | RELOCATE | `product/fleet_intent.dag` (the host-model authority) | [ ] |
| `compute_host_primary_cpu`, `compute_host_ram_bytes_total` | `rust_gates_ci.dag`, `ci_runner_placement.dag`, `fleet_host_budget.dag`, `ci_budget_tree.dag`, `runner_spec_from_offer.dag` | RELOCATE | Co-locate with `ComputeHost` | [ ] |
| `ComputeSupplyFacts` | `ci_fleet.dag`, `fleet_intent.dag` | RELOCATE | `product/ci_fleet.dag` | [ ] |
| `ComputeOffer` | `ci_fleet.dag`, `fleet_intent.dag`, `ci_runner_placement.dag`, `fleet_host_budget.dag`, `runner_spec_from_offer.dag`, `ci_budget_tree.dag` | RELOCATE | `product/ci_fleet.dag` (the offer authority) | [ ] |
| `placement_supply_row` | `ci_runner_placement.dag`, `fleet_host_budget.dag`, `ci_budget_tree.dag`, `runner_spec_from_offer.dag` | RELOCATE | `product/placement_supply.dag` (already exists as `PlacementSupplyRow`) | [ ] |

---

## Resource envelope / requirements

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `ResourceEnvelope` | `fleet_container.dag`, `src/v2/workflow/ci_floor_plan.dag`, `src/v1/stage0/inert_carrier_project.rs` | RELOCATE | `product/fleet_container.dag` or `std` (scheduler authority) | [ ] |
| `CpuRequirement`, `GpuRequirement`, `MemoryRequirement`, `StorageRequirement`, `NetworkRequirement` | `fleet_container.dag`, `src/v2/workflow/ci_floor_plan.dag` | RELOCATE | Co-locate with `ResourceEnvelope` | [ ] |
| `ToolchainCapability`, `ToolchainRequirement`, `ToolchainEnvIsolation`, `BuildWrapperKind` | `fleet_intent.dag`, `fleet_container.dag`, `ci_fleet.dag` | RELOCATE | `extdeps/toolchain/types.dag` (types already there) | [ ] |

---

## Work demand / parallelism

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `WorkDemand`, `ParallelismShape`, `ReducerLaws` | `src/v2/workflow/ci_floor_plan.dag`, `dsl/gunbc/ci_floor_measurement.dag`, `src/v1/stage0/inert_carrier_project.rs` | RELOCATE | `std/realization_schedule.dag` or `product/placement_supply.dag` | [ ] |
| `WorkUnitId`, `WorkUnit` | `execution_receipt_digest_test.dag`, `realization_measurement_keystone_test.dag` | RELOCATE | `std/realization_schedule.dag` | [ ] |

---

## Admission / input envelope

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `InputEnvelope`, `InputBound`, `InputSizeAxis`, `AdmissionVerdict` | `input_envelope_admission_test.dag`, `gunbc_ci_corpus_envelope.dag` | RELOCATE | `product/ci_floor_measurement.dag` or `std/realization_schedule.dag` | [ ] |
| `input_admitted`, `gunbc_ci_corpus_envelope`, `parallel_run_demand_envelope`, `parallel_run_demand_envelope_shard_count` | `input_envelope_admission_test.dag`, `ci_floor_measurement.dag` | RELOCATE | Co-locate with `InputEnvelope` | [ ] |

---

## Performance / execution receipts

| Symbol | External consumers | Disposition | Proposed home | Status |
|--------|-------------------|-------------|---------------|--------|
| `PerformanceReceipt`, `ExecutionReceipt`, `CostReceipt`, `PricingSource`, `AmortizationScope` | `execution_receipt_digest_test.dag`, `realization_measurement_keystone_test.dag` | RELOCATE | `std/realization_measurement.dag` (receipt accessors already there) | [ ] |
| `execution_receipt_subject_digest`, `execution_receipt_performance_digest`, `execution_receipt_cost_digest`, `execution_receipt_full_digest` | `execution_receipt_digest_test.dag` | RELOCATE | Co-locate with `ExecutionReceipt` | [ ] |
| `performance_receipt_host_single_sample`, `cost_account_from_performance_receipts`, `performance_receipts_total_time` | `realization_measurement_keystone_test.dag` | RELOCATE | `std/realization_measurement.dag` | [ ] |
| `MeasurementConfidence` | `realization_measurement_keystone_test.dag` | RELOCATE | `std/realization_measurement.dag` | [ ] |

---

## Fabric-internal stubs (DELETE)

No external consumers confirmed. Safe to drop once #5889 lands.

| Symbol | Disposition | Status |
|--------|-------------|--------|
| `ComputeWitness<C>`, `Outcome<T>`, `UpsertInputRef`, `ChangeSet`, `ArtifactRef<T>`, `ArtifactSpec<T>`, `WorkAction`, `WorkGraph`, `Partitioner`, `Reducer`, `EffectBoundary`, `SymbolicCost`, `Associativity`, `Commutativity`, `Idempotency`, `IdentityElement`, `InstructionSet` | DELETE (opaque Node stubs, zero consumers) | [ ] |
| `ComputeLease`, `ComputeLeaseWitness`, `ComputeLeaseEligibility`, `AllocationReceipt` | DELETE (lease machinery, zero external consumers) | [ ] |
| `MachineView`, `MissingDemandFact`, `DemandDimension` | DELETE (old matcher internals) | [ ] |
| `OomSignalClass`, `ProviderConstraint`, `ExecutionBudget`, `WatchdogLimit` | DELETE (no external consumers found in tree scan) | [ ] |
| `ComputeArtifactLocality` | DELETE (no external consumers found) | [ ] |
| `satisfies`, `isolation_satisfies`, `isolation_digest`, old `example_*` data, old witnesses | DELETE (replaced by connector model) | [x] (landed in #5889) |
