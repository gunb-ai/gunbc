# v4 Modeling DFS Worksheet B — Elastic cache interface (fractal Upsert chain)

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §8 sign-off 2026-05-31 (proud-pike-680; PR #4095). **Pre-implementation schema amendments** 2026-05-31 (operator review; §1.9). **READY-FOR-IMPLEMENTATION-DISPATCH** once #4095 merges to main. Pair with **Worksheet A** (`v4-elastic-compute-fabric-worksheet-2026-05-30.md`).
> **Date:** 2026-05-30 (split amend 2026-05-31)
> **Author:** sharp-wolf-824 (worker under proud-pike-680)
> **Dispatch anchor:** node://adhoc-2e6e2313-8a5 — exploration §4.0f **Worksheet B** (cases 9–19) + §4.0g
> **Authority doc:** `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0g–§4.0f Worksheet B (main)
> **Prerequisite:** Worksheet A §8 (for `ExecutionReceipt<T>` composition edge only). **No `.dag` landing in this slice.**

---

## Composition boundary (no cross-import with Worksheet A)

Compose with compute fabric **only** through:

- `CachedArtifactReceipt<T>.producer: ProducerReceipt<T>` where internal hits cite `ExecutionReceiptRef<T>` (neutral digest ref — **no** `import compute_fabric`)
- Worksheet A still exposes full `ExecutionReceipt<T>` on `ExecutionReceipt.output` (compute → artifact handle)
- Semantic artifacts use `ArtifactIdentity<T>` projected from Upsert subjects (same `content_hash` discipline as compute `ArtifactRef` ingress — identities must align at harness boundary)

**Forbidden:** `dsl/std/cache_interface.dag` importing `compute_fabric.dag`. **`CacheStore` is not first authority** — use `CacheInterfaceFacts` rows + derived `CacheStoreView`.

**Staged target (post–first landing):** hoist `ProducerReceiptRef<T>` / shared provenance carriers to `dsl/std/artifact.dag` (Option B); first implementation uses Option A in §1.5.

---

## Mechanical dispatch rule

> **Cache-interface implementation workers may dispatch per §6 after PR #4095 merges (Worksheet B §8 closed 2026-05-31).**

Land concrete `CacheInterfaceFacts` **data rows first**; `CacheStoreView` is a projection only (same posture as `MachineView` in Worksheet A).

---

## §10.0-adapted worksheet (Worksheet B only)

```text
Substrate class:        CACHE-INTERFACE (dsl/std/cache_interface.dag)
Representative failure:  CacheKind enum; CacheStore as first authority; hashFiles in YAML;
                         bare blob hits without producer ExecutionReceipt.
Immediate local patch:   CacheKind = GHA | sccache; hand-authored cache_key on steps.
Why forbidden:           §4.0d; P2 — backend rows must land as CacheInterfaceFacts first.
DFS path:
  NEW: dsl/std/cache_interface.dag
  CONSUME: types.dag ContentHash, patterns.dag Upsert<T> (projection discipline only)
  COMPOSITION: ExecutionReceipt<T> from compute_fabric (type name only — no import)
Falsification probe:   §5 cases 9–19 only.
```

---

## §1 Authoritative substrate catalog (Worksheet B)

### 1.1 Authority order: facts row first, view second

```dag
type CacheInterfaceId = NonEmptyStr where brand("CacheInterfaceId")

// Authority — one row per concrete backend. Lands FIRST.
type CacheInterfaceFacts {
  identity: CacheInterfaceId
  backing_surface: StorageSurface
  key_derivation: KeyDerivationFacts
  lookup_semantics: CacheLookupSemantics
  write_semantics: CacheWriteSemantics
  miss_semantics: CacheMissSemantics
  value_shape: ValueShape
  locality: PersistenceLocality
  eviction: EvictionPolicy
  atomicity: AtomicityModel
  auth: AuthScope
  read_latency: ReadLatencyClass
  consistency: ConsistencyModel
  evidence: List<CacheEvidence>           // case 19 — plural, field-scoped when needed
}

// Derived projection — NEVER first authority.
type CacheStoreView {
  facts: CacheInterfaceFacts
}

type CacheRowEvidence
  = VendorCitation { source: NonEmptyStr, url: NonEmptyStr }
  | RunnerObserved { receipt: NonEmptyStr, observed_at: LogicalTime }
  | OperatorObserved { note: NonEmptyStr, observed_at: LogicalTime }
  | CiReceiptCitation { workflow_run_id: NonEmptyStr, step_id: NonEmptyStr }

// Field-scoped evidence when vendor docs, runner observation, and operator notes
// support different coordinates on the same row (case 19).
type CacheEvidence {
  field: Symbol                            // e.g. eviction, read_latency, key_derivation
  evidence: CacheRowEvidence
}
```

### 1.2 Identity vs transport

```dag
type ArtifactKindId = NonEmptyStr where brand("ArtifactKindId")  // stage → v4.std.artifact at harness

type ArtifactIdentity<T> {
  subject_digest: ContentHash              // content_hash(canonical Upsert<T> subject)
  artifact_kind: ArtifactKindId            // not bare NonEmptyStr — prevents copy-paste drift
}

type ProviderKey
  = OpaqueStringKey { key: NonEmptyStr }
  | SccacheInternalHash { digest: ContentHash }
  | CasContentDigest { digest: ContentHash }
  | CargoFingerprint { digest: ContentHash }

type BackendCacheKey {
  store: CacheInterfaceId
  key: ProviderKey
}

type CacheKeyProjection<T> {
  artifact: ArtifactIdentity<T>
  backend: CacheInterfaceFacts
  transport: BackendCacheKey
}
```

### 1.3 Lookup / write / miss semantics

```dag
type CacheLookupSemantics
  = ExactKeyOnly
  | PrefixFallback { ordered_prefixes: List<NonEmptyStr> }
  | NativeInternalLookup
  | ContentAddressLookup

type CacheWriteSemantics
  = WriteOnce
  | OverwriteAllowed
  | WriteThenCommit
  | WriteThenRename
  | ProviderInternal

type CacheMissSemantics
  = MissThenCreate
  | MissIsDiagnostic
  | ProviderNativeFallback                  // intrinsic to backend (e.g. GHA restore-key chain)

// Cross-store L1→L2 fallback is composition, NOT a field on CacheInterfaceFacts.
type CacheLayerPlan {
  primary: CacheInterfaceId
  fallback: Option<CacheInterfaceId>
}
```

### 1.4 KeyDerivationFacts (per-backend concrete fields)

```dag
type KeyDerivationClass
  = ContentAddressedByValue
  | HandAuthoredString
  | NativeInternalHash

// Preliminary classifier only — concrete rows MUST still spell field-specific hit/miss
// inputs in KeyDerivationFacts (do not let RustcInvocation become a black box).
type InputSurface
  = UpsertSubject
  | FilePaths
  | RustcInvocation
  | ToolchainSpec

type VisibilityScope = Repo | Org | Network | World

type InvalidationTrigger
  = TtlExpiry
  | ManualPurge
  | ToolchainChange
  | DependencyChange

type KeyDerivationFacts {
  classification: KeyDerivationClass
  inputs_considered: List<InputSurface>
  overwritable: Bool
  prefix_fallback_allowed: Bool
  content_verified_on_read: Bool
  visibility_scope: VisibilityScope
  invalidation_triggers: List<InvalidationTrigger>
}
```

### 1.5 Cache hit receipt + producer composition (Worksheet A)

```dag
// Option A (first landing): neutral ref — cache_interface.dag does NOT import compute_fabric.
type ProducerKind
  = InternalExecution
  | ExternalVendor
  | RunnerObserved

type ExecutionReceiptRef<T> {
  receipt_digest: ContentHash              // digest of full ExecutionReceipt<T> in compute_fabric
  producer_kind: ProducerKind
}

type ProducerReceipt<T>
  = InternalExecution { receipt: ExecutionReceiptRef<T> }   // our CI compute path
  | VendorArtifact { evidence: CacheRowEvidence, content_digest: ContentHash }
  | RunnerObservedArtifact { evidence: CacheRowEvidence, content_digest: ContentHash }

type CachedArtifactReceipt<T> {
  artifact: ArtifactIdentity<T>
  backend_key: BackendCacheKey
  producer: ProducerReceipt<T>             // not bare ExecutionReceipt<T> — see §1.9
  verified_subject_digest: ContentHash
  content_digest: ContentHash
}

type CacheRejectReason
  = SubjectDigestMismatch
  | ContentDigestMismatch
  | ProducerReceiptMissing
  | BackendKeyMalformed
  | BackendUnauthorized
  | BackendUnavailable

type CacheLookupResult<T>
  = Hit { receipt: CachedArtifactReceipt<T> }
  | Miss
  | RejectedHit { reason: CacheRejectReason }
```

### 1.6 Dimension enums + StorageSurface

```dag
type ValueShape = RawBytes | StructuredArtifact | TarArchive | FileTree
// Cache-store placement only — compute demand uses `ComputeArtifactLocality` (Worksheet A).
type PersistenceLocality = InProcess | PerRunnerFilesystem | PerHostFilesystem | CrossHostNetwork
type EvictionPolicy = Ttl { days: Int } | Lru | SizeBounded { cap_bytes: ByteSize } | Never | Manual
type AtomicityModel = PerFile | WriteThenRename | WriteThenCommit | TwoPhase
type AuthScope = None | FilesystemPerms | ApiKey | NetworkAcl
type ReadLatencyClass = InProcessNs | LocalDiskUs | LanMs | WanTensMs
type ConsistencyModel = Strong | Eventual | ReadYourWrites

type StorageSurface = NonEmptyStr where brand("StorageSurface")  // NVMe path, GHA API, BB CAS, …
type LogicalTime = NonEmptyStr where brand("LogicalTime")
type ByteSize = Measure<Memory, _>
```

### 1.7 Projection function

```dag
fn cache_key_projection<T>(
  artifact: ArtifactIdentity<T>,
  backend: CacheInterfaceFacts,
) -> CacheKeyProjection<T>
// Harness derives BackendCacheKey from subject_digest + backend.key_derivation (case 9–11).
```

### 1.8 Canonical data rows (land before views)

| Row id | `lookup_semantics` | `miss_semantics` | Case |
|--------|-------------------|------------------|------|
| `gha_actions_cache_facts` | `PrefixFallback` | `MissThenCreate` | 9 |
| `sccache_local_facts` | `NativeInternalLookup` | `MissThenCreate` | 10 |
| `buildbuddy_cas_facts` | `ContentAddressLookup` | `MissThenCreate` | 11 |
| `cargo_target_dir_facts` | `NativeInternalLookup` | `MissThenCreate` | 12 |
| `rustup_toolchain_store_facts` | `NativeInternalLookup` | `MissThenCreate` | 13 |

Each row must include non-empty `evidence: List<CacheEvidence>` before ratification (case 19). Cross-store fallback (e.g. sccache L1 → remote CAS) is modeled with `CacheLayerPlan`, not `MissThenFallback` on a single row.

### 1.9 Implementation notes (pre–`cache_interface.dag` landing)

```text
1. CachedArtifactReceipt.producer uses ProducerReceipt<T>, not a direct
   ExecutionReceipt<T> import. First landing: ExecutionReceiptRef<T> digest ref
   (Option A). Later: shared provenance in artifact.dag (Option B).

2. CacheInterfaceFacts.evidence is List<CacheEvidence> — field-scoped when vendor
   docs, runner observation, operator notes, or CI receipts support different coords.

3. Cross-store fallback belongs in CacheLayerPlan, not CacheInterfaceFacts, unless
   the provider itself implements fallback (ProviderNativeFallback on miss_semantics).

4. ArtifactKindId and CacheRejectReason are branded/typed carriers — not prose NonEmptyStr.

5. InputSurface is a coarse classifier; each concrete row still documents what the
   backend actually considers for hit/miss, verification, and invalidation.

Implementation guardrails (dispatch):
  - Land concrete CacheInterfaceFacts rows first.
  - No CacheKind; no import compute_fabric; CacheStoreView is never first authority.
  - No CI/workflow emission in cache_interface PR.
  - Model GHA / sccache / BuildBuddy / Cargo / rustup as distinct rows.
```

---

## §2 Parser gates (Worksheet B)

| Gate | Requirement |
| ---- | ----------- |
| **P-CI-TYPE** | `cache_interface.dag` parses §1 |
| **P-CI-GENERIC** | `ArtifactIdentity<T>`, `CachedArtifactReceipt<T>` |
| **P-CI-NO-IMPORT** | Module does not `import` compute_fabric |
| **P-CI-EVIDENCE** | `data` rows require `evidence` field at parse time (or 🟡 gate until data-body syntax lands) |

---

## §3 M9 concept-home map (cache only)

| Concept | Home |
| ------- | ---- |
| Upsert cache discipline | `dsl/std/patterns.dag` (content_hash canon) |
| Content hash | `dsl/std/types.dag` |
| Backend facts | **`dsl/std/cache_interface.dag`** |
| Producer proof | `ProducerReceipt<T>` in **cache_interface**; `ExecutionReceipt<T>` in **compute_fabric** via `ExecutionReceiptRef` digest |
| Layered cache plans | **`CacheLayerPlan`** (harness / planner — not per-row miss field) |

---

## §4 Spot-fix register (Worksheet B)

| Pattern | Why forbidden |
| ------- | ------------- |
| `CacheKind` coproduct | §4.0d / case 16 |
| `type CacheStore` as first authority | Use `CacheInterfaceFacts` + `CacheStoreView` |
| `hashFiles(...)` in workflow authority | case 9 |
| Bare blob without `CachedArtifactReceipt` | case 15 |
| `import compute_fabric` | Composition boundary |
| Row without `List<CacheEvidence>` | case 19 |
| `MissThenFallback { lower_tier }` on `CacheInterfaceFacts` | Use `CacheLayerPlan` + `ProviderNativeFallback` |
| `RejectedHit { reason: NonEmptyStr }` | Use `CacheRejectReason` |

---

## §5 Falsification cases 9–19

| # | Case | Proving types | Pass condition |
|---|------|---------------|----------------|
| 9 | GHA Actions Cache | `data gha_actions_cache_facts` | No `hashFiles` authority; `cache_key_projection` → `BackendCacheKey` |
| 10 | sccache | `sccache_local_facts`, `NativeInternalLookup` | Internal hash opaque at L1 |
| 11 | BuildBuddy CAS | `buildbuddy_cas_facts`, `ContentAddressLookup` | `ctrl-build --remote`; digest 1:1 |
| 12 | Cargo target/ L0 | `cargo_target_dir_facts`, `PerRunnerFilesystem` | Ephemeral; cleared on lease end |
| 13 | rustup store | `rustup_toolchain_store_facts` | Distinct `KeyDerivationFacts` vs CAS |
| 14 | New backend | one new `CacheInterfaceFacts` row | No `Upsert<T>` / `WorkUnit` change |
| 15 | Wrong-cache-hit | `CachedArtifactReceipt`, `RejectedHit { CacheRejectReason }` | Digest mismatch → typed reject reason |
| 16 | Orthogonal facts | dimension enums + rows | 4/4 grid; no `CacheKind` |
| 17 | Same identity, different transport | `cache_key_projection` | Different `BackendCacheKey`, same `ArtifactIdentity` |
| 18 | Toolchain bump | `InvalidationTrigger::ToolchainChange` | Output-affecting bump → new `subject_digest` |
| 19 | Vendor evidence | `List<CacheEvidence>` on every row | Ratification rejects empty evidence; field-scoped allowed |

---

## §6 Landing order (Worksheet B)

```text
B1. §1.6 dimension enums + ArtifactKindId + CacheRejectReason
B2. §1.3 semantics coproducts + CacheLayerPlan
B3. §1.1 CacheInterfaceFacts + List<CacheEvidence>
B4. data rows (gha, sccache, buildbuddy, cargo_target, rustup) + evidence lists
B5. ArtifactIdentity / BackendCacheKey / CacheKeyProjection
B6. ProducerReceipt / ExecutionReceiptRef + CachedArtifactReceipt + CacheLookupResult
B7. CacheStoreView projection fn (optional)
B8. v4.std mirror (ProducerReceiptRef if shared module not yet landed)
```

---

## §8 Out of scope (Worksheet B)

- `dsl/std/compute_fabric.dag` (Worksheet A)
- `ci.dag` / workflow emission (downstream)
- INVARIANTS / THESIS / MODELING edits

---

## §9 Downstream brief (after §10 B)

Land `dsl/std/cache_interface.dag` per §6. **MUST NOT** import compute module. Prove cases 9–19.

---

## §10 Manager approval checklist (Worksheet B) — CLOSED 2026-05-31

- [x] `CacheInterfaceFacts` before `CacheStoreView` discipline accepted
- [x] Identity vs transport (`ArtifactIdentity` / `BackendCacheKey` / `CacheKeyProjection`) accepted
- [x] Lookup/write/miss semantics + `KeyDerivationFacts` accepted
- [x] §5 cases 9–19 accepted
- [x] Vendor evidence discipline (case 19) accepted
- [x] Composition boundary with Worksheet A accepted (`ProducerReceipt` / `ExecutionReceiptRef`, no cross-import)
- [x] Pre-implementation amendments (§1.9) accepted for dispatch guardrails
- [x] Implementation dispatch authorized (cache_interface PR only, post-merge)

---

## Related artifacts

- `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0g + §4.0f Worksheet B
- `docs/planning/v4-elastic-compute-fabric-worksheet-2026-05-30.md` (pair)
- `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`
