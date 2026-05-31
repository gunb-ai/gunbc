# v4 Modeling DFS Worksheet B — Elastic cache interface (fractal Upsert chain)

> **Status:** WORKSHEET DRAFT — §8 **pending** (prior combined-doc §8 withdrawn 2026-05-31; proud-pike-680). Pair with **Worksheet A** (`v4-elastic-compute-fabric-worksheet-2026-05-30.md`).
> **Date:** 2026-05-30 (split amend 2026-05-31)
> **Author:** sharp-wolf-824 (worker under proud-pike-680)
> **Dispatch anchor:** node://adhoc-2e6e2313-8a5 — exploration §4.0f **Worksheet B** (cases 9–19) + §4.0g
> **Authority doc:** `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0g–§4.0f Worksheet B (main)
> **Prerequisite:** Worksheet A §8 (for `ExecutionReceipt<T>` composition edge only). **No `.dag` landing in this slice.**

---

## Composition boundary (no cross-import with Worksheet A)

Compose with compute fabric **only** through:

- `CachedArtifactReceipt<T>.producer: ExecutionReceipt<T>` (cache hit must cite compute proof)
- Semantic artifacts use `ArtifactIdentity<T>` projected from Upsert subjects (same `content_hash` discipline as compute `ArtifactRef` ingress — identities must align at harness boundary)

**Forbidden:** `dsl/std/cache_interface.dag` importing `compute_fabric.dag`. **`CacheStore` is not first authority** — use `CacheInterfaceFacts` rows + derived `CacheStoreView`.

---

## Mechanical dispatch rule

> **No cache-interface implementation worker until Worksheet B is Modeling DFS Manager–approved.**

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
  evidence: CacheRowEvidence              // case 19 — vendor or observed
}

// Derived projection — NEVER first authority.
type CacheStoreView {
  facts: CacheInterfaceFacts
}

type CacheRowEvidence
  = VendorCitation { source: NonEmptyStr, url: NonEmptyStr }
  | RunnerObserved { receipt: NonEmptyStr, observed_at: LogicalTime }
```

### 1.2 Identity vs transport

```dag
type ArtifactIdentity<T> {
  subject_digest: ContentHash              // content_hash(canonical Upsert<T> subject)
  artifact_kind: NonEmptyStr               // branded kind — align with v4.std.artifact at harness
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
  | MissThenFallback { lower_tier: CacheInterfaceId }
  | MissIsDiagnostic
```

### 1.4 KeyDerivationFacts (per-backend concrete fields)

```dag
type KeyDerivationClass
  = ContentAddressedByValue
  | HandAuthoredString
  | NativeInternalHash

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

### 1.5 Cache hit receipt (composition with Worksheet A)

```dag
// ExecutionReceipt<T> — defined in compute_fabric.dag; referenced here only.
type CachedArtifactReceipt<T> {
  artifact: ArtifactIdentity<T>
  backend_key: BackendCacheKey
  producer: ExecutionReceipt<T>            // COMPOSITION EDGE — no compute import in .dag
  verified_subject_digest: ContentHash
  content_digest: ContentHash
}

type CacheLookupResult<T>
  = Hit { receipt: CachedArtifactReceipt<T> }
  | Miss
  | RejectedHit { reason: NonEmptyStr }
```

### 1.6 Dimension enums + StorageSurface

```dag
type ValueShape = RawBytes | StructuredArtifact | TarArchive | FileTree
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
| `buildbuddy_cas_facts` | `ContentAddressLookup` | `MissThenFallback` | 11 |
| `cargo_target_dir_facts` | `NativeInternalLookup` | `MissThenCreate` | 12 |
| `rustup_toolchain_store_facts` | `NativeInternalLookup` | `MissThenCreate` | 13 |

Each row must include `evidence: CacheRowEvidence` before ratification (case 19).

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
| Compute proof | `ExecutionReceipt<T>` in **compute_fabric** (composition) |

---

## §4 Spot-fix register (Worksheet B)

| Pattern | Why forbidden |
| ------- | ------------- |
| `CacheKind` coproduct | §4.0d / case 16 |
| `type CacheStore` as first authority | Use `CacheInterfaceFacts` + `CacheStoreView` |
| `hashFiles(...)` in workflow authority | case 9 |
| Bare blob without `CachedArtifactReceipt` | case 15 |
| `import compute_fabric` | Composition boundary |
| Row without `CacheRowEvidence` | case 19 |

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
| 15 | Wrong-cache-hit | `CachedArtifactReceipt`, `RejectedHit` | `verified_subject_digest` + `content_digest` match |
| 16 | Orthogonal facts | dimension enums + rows | 4/4 grid; no `CacheKind` |
| 17 | Same identity, different transport | `cache_key_projection` | Different `BackendCacheKey`, same `ArtifactIdentity` |
| 18 | Toolchain bump | `InvalidationTrigger::ToolchainChange` | Output-affecting bump → new `subject_digest` |
| 19 | Vendor evidence | `CacheRowEvidence` on every row | Ratification rejects rows without evidence |

---

## §6 Landing order (Worksheet B)

```text
B1. §1.6 dimension enums + KeyDerivationFacts
B2. §1.3 semantics coproducts
B3. §1.1 CacheInterfaceFacts record
B4. data rows (gha, sccache, buildbuddy, cargo_target, rustup) + evidence
B5. ArtifactIdentity / BackendCacheKey / CacheKeyProjection
B6. CachedArtifactReceipt + CacheLookupResult
B7. CacheStoreView projection fn (optional)
B8. v4.std mirror (ExecutionReceipt reference by name)
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

## §10 Manager approval checklist (Worksheet B) — OPEN

- [ ] `CacheInterfaceFacts` before `CacheStoreView` discipline accepted
- [ ] Identity vs transport (`ArtifactIdentity` / `BackendCacheKey` / `CacheKeyProjection`) accepted
- [ ] Lookup/write/miss semantics + `KeyDerivationFacts` accepted
- [ ] §5 cases 9–19 accepted
- [ ] Vendor evidence discipline (case 19) accepted
- [ ] Composition boundary with Worksheet A accepted
- [ ] Implementation dispatch authorized (cache PR only)

---

## Related artifacts

- `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` §4.0g + §4.0f Worksheet B
- `docs/planning/v4-elastic-compute-fabric-worksheet-2026-05-30.md` (pair)
- `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`
