# Elastic CI redesign — exploration

> **Status:** EXPLORATION — operator briefing, not a ratified plan. Builds on existing
> ratified work (T-24 phases, Upsert<T> overhaul); does not redesign it. Identifies
> infra-layer + compiler-layer + cache-layer prerequisites for "infinite-elastic"
> end-state.
> **Date:** 2026-05-31
> **Authority anchor:** PR #4074 run 26702944004 (the 33m46s `ci_v4` profile that
> prompted the exploration).

---

## 0. What this doc is — and is NOT

**Is:** a synthesis layer over (a) the current run profile, (b) the modeled CI
substrate in `src/v4/workflow/ci.dag`, (c) the ratified T-24 phased overhaul, and
(d) the operator's framing constraint: *"simulate a perfectly elastic CI/build
environment using srv1/srv2 — given infinite resources, design without
hardcoding constraints to our specific compute… treat servers as stateless;
caching is either Actions Cache or per-host logical caches, never relying on
filesystem state."*

**Is NOT:**
- A re-ledger of `ci.dag` substrate (references by line, doesn't duplicate per the
  ledger-standing principle, operator 2026-05-19).
- A redesign of T-24 phases — Phase 1a / 1.4 / 1.5 / 1b / 2 / 2.5 are ratified.
  See `docs/planning/v4-ci-overhaul-2026-05-30.md` §6.
- A vendor commitment (Actions Cache vs BuildBuddy vs local CAS) — operator
  decision territory.
- A new policy / mode / run-class enum. Per
  `feedback_heuristics_recoverable_to_substrate`, heuristic enums are
  forbidden in a closed system; everything below either reads existing modeled
  facts or names a structural fact that needs modeling.

---

## 1. Profile snapshot — what motivates the redesign

Source: PR #4074 run 26702944004 (`gh run view 26702944004`).

### 1.1 Wall-time shape

| Job | Wall | Critical-path role |
|-----|------|--------------------|
| `fmt` | 24s | parallel start |
| `affected` | 23s | parallel start |
| `discipline` | 45s | parallel start |
| `ci_integration` | 4m 16s | parallel lane |
| `v3` | 16m 41s | parallel lane (gated on `v3=true`) |
| `self_host_ratchet` | 7s | post-v3 |
| **`ci_v4`** | **33m 46s** | **long pole** |
| `ci` | 5s | aggregator stub |
| `v4` | 5s | aggregator stub |

Wall-clock critical path = `ci_v4`. Total observed run was 1h 15m only because the
run was `run_attempt=2` and `ci_v4` was specifically rerun ~24 min after first-pass
completion. **First-attempt critical path ≈ `ci_v4` time, ~34 min.**

### 1.2 Inside `ci_v4` (33m 46s) — the four-compile redundancy

Extracted from per-step timestamps (`gh api .../jobs/78700915985`):

| Step | Wall | What it actually does |
|------|------|----------------------|
| Setup (toolchain + caches) | 26s | gunbc binary cache hits — no v2 rebuild |
| MVP-1 e2e (add.dag) | 12s | tiny fixture |
| Lens-CI semantic compile | 30s | 54-file partial closure |
| **M1 v4 full-tree rust emit probe** | **10m 4s** | 7m 4s gunbc compile (332→336 files) + 2m 59s `cargo check` |
| `Detect phase1_nat_semiring` | <1s | |
| **v2→v4 bootstrap compile (dag)** | **7m 21s** | gunbc compile (332→1 dag artifact) |
| Cache cleanup (gates 3s) | 3s | |
| **T-22 corpus rust+dag** | **14m 44s** | gunbc compile rust (7m 18s) + gunbc compile dag (7m 22s) + py/jq receipts (<5s) |
| Post-caches + teardown | 24s | |

**The compiler runs on the same 332-source `src/v4` closure four times in one job:**
- A: M1 rust emit (7m 4s)
- B: v2→v4 bootstrap dag (7m 21s)
- C: T-22 corpus rust (7m 18s) — **identical input to A**
- D: T-22 corpus dag (7m 22s) — **identical input to B**

Total compiler-runtime in `ci_v4` = 32m 4s (95% of wall). 14m 40s of that
(C + D) is byte-for-byte duplication of A + B.

### 1.3 `v3` (16m 41s) — 5 cargo invocations on overlapping crate state

| Step | Wall | Notes |
|------|------|-------|
| Prebuild integration test bin | 2m 20s | `cargo test --test integration --no-run` |
| `cargo test --lib --bins` (457 tests in 342s) | 7m 6s | 1m 24s compile + 5m 42s execute |
| determinism + doc + integration zero-filter | ~76s | mostly libtest setup overhead |
| `cargo clippy --all-targets` | 1m 45s | separate build state from `cargo test` |
| `cargo clippy --features bootstrap-regen-fresh` | 1m 2s | feature-different incremental |
| `cargo test --no-run --features bootstrap-regen-fresh` | 2m 23s | yet another feature-different build |

### 1.4 Cache fragility

T-22 cache key (`ci.yml:408`):
```
hashFiles('.github/workflows/ci.yml', 'scripts/v4-testclaim-corpus-gate.sh',
          'src/v2/stage0/src/**', 'src/v2/stage0/Cargo.toml', 'Cargo.toml',
          '**/Cargo.lock', 'rust-toolchain.toml', 'dsl/std/**', 'src/v4/**')
```

For the active sprint, `src/v4/**` mutates in nearly every PR, `ci.yml` mutates
weekly. Effective hit rate ≈ 0%. The receipt cached is exactly the duplication in
§1.2, so when it hits it saves 14m 44s — but it rarely hits.

### 1.5 Deeper artifacts (for cross-reference)

- Anatomy + Table A (redundancy) / Table B (modeled-fix) inventory:
  `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md`
- Warm-cache wall measurement (12m31s v3 baseline):
  `docs/audit/ci-warm-cache-wall-measurement-2026-05-29.md`

---

## 2. Infra layer — what's actually under us

### 2.1 Modeled fleet (`src/v4/workflow/ci.dag:364-423`)

```
SelfHostedRunnerPool { host, arch, core_count, runner_count, jobserver_token_cap }

ci_srv1_pool = { srv1_host, arm64, 128 cores, 20 runners, 25 jobserver tokens }
ci_srv2_pool = { srv2_host, arm64, 128 cores, 30 runners, 36 jobserver tokens }
ci_self_hosted_runner_pools = [ci_srv1_pool, ci_srv2_pool]
```

Total: **50 runners across 2 boxes, 256 cores, 61 jobserver tokens**. Substantial
capacity — bottleneck is not raw compute, it's how the work is shaped against it.

### 2.2 `ctrl-build` wrapper (`/usr/local/bin/ctrl-build`)

Every cargo invocation in container sessions is routed through `ctrl-build`:

| Concern | What it does |
|---------|--------------|
| **sccache** | Per-host compilation cache (lifetime/identity not modeled) |
| **`CARGO_BUILD_JOBS`** | Dynamic cap based on container memory + concurrent-job count |
| **Memory caps** | Cgroup-style; prevents swap on shared host |
| **Local default** | `--local` is implicit; explicit `--remote` opt-in for BuildBuddy |
| **BuildBuddy** | Remote execution via `bb` CLI; `CTRL_BUILD_REMOTE_IMAGE` defaults to `ctrl-session:latest` |

**What's not yet declared as substrate:** sccache identity (per-host? per-runner?
shared?), token semantics, or the precise relationship between `ctrl-build`'s caps
and `SelfHostedRunnerPool.jobserver_token_cap` in `ci.dag`. These are operator
infra facts; modeling them is on the table for Phase 1.5+.

### 2.3 Shared `$HOME` isolation pattern (`ci.yml:27-35,42-45`)

srv1/srv2 expose the same `$HOME=/home/briansrls/` across all ephemeral GHA worker
instances per host. `actions-rust-lang/setup-rust-toolchain@v1.16.0` concurrently
clobbers `~/.cargo/bin/rustc` (`Text file busy`) and the default-toolchain pointer.

Workaround: every Rust-using job's first step writes
`CARGO_HOME=$RUNNER_TEMP/cargo` and `RUSTUP_HOME=$RUNNER_TEMP/rustup` to
`$GITHUB_ENV`. Brittle:
- Any new action that writes outside the isolated dirs could regress.
- `Post Setup Rust` steps add 15-22s of cache-save per job (six Rust-bearing
  jobs ⇒ ~90s aggregate).

This is the "filesystem state as side-channel" anti-pattern the operator's framing
explicitly rules out.

### 2.4 Today's cache layer

| Cache | Backend | Key fragility | Hit observed in PR #4074 |
|-------|---------|---------------|-----|
| Cargo (per job) | `actions/cache@v4` to `$RUNNER_TEMP/cargo/**` + `target/` | Cargo.lock + v3 sources | Restore ~6s (warm); save ~15-22s |
| gunbc binary | `actions/cache@v4` to `target/release/gunbc` | v2 sources + Cargo files | **HIT** (skipped rebuild) |
| T-22 receipt | `actions/cache@v4` (small receipt file) | 9 broad globs | **MISS** (introducing PR) |
| sccache | `ctrl-build`-managed | per-host, not modeled | not visible in CI log |
| Compiler emit output | **none** | — | every gunbc compile re-emits from scratch |

The fourth row is the elastic-design gap: there is no content-addressed cache of
the v2-compiler's emitted output. Every `gunbc compile` writes 336 files from
scratch even when the input + binary are byte-identical to a prior run.

### 2.5 Known infra gotchas (load-bearing-by-luck)

| Gotcha | Where it bit | Owner |
|--------|--------------|-------|
| srv2 `ctrl-jobserver` FIFO race | `/var/lib/ctrl/jobserver/host.fifo` is a directory on srv2 (FIFO on srv1) → daemon crash-loop → emit stalls until step timeout. M1 20m timeout / exit 143 (audit §5.1) | Operator infra |
| Concurrent jobs share `$HOME` registry | Race avoidance via per-job `CARGO_HOME` indirection; brittle to new actions | Workflow author |
| `$RUNNER_TEMP` per-job lifetime | Caches must be re-fetched every job — no cross-job state | GHA-side, not local |
| 20m / 35m / 60m hard timeouts | Encoded in YAML, not modeled — no `ExecutionBudget` splitting semantic `expected_cost` from operational `watchdog` (see §4.0c) | T-24 close gap |
| `continue-on-error: true` on Tier-0 probes (M1, phase1) | Modeled `non_blocking: true` in `ci.dag:451-457`, but YAML divergence possible | Phase 1.5 W2.3 |

---

## 3. In-flight Upsert<T> CI rework — what's already landing

### 3.1 Substrate (current state)

`src/v4/workflow/ci.dag` (≈3100 lines, expanding):

```
type CiPipeline { jobs: List<CiJob>, gates: List<CiGate> }                 // :157-160
type CiJob { id, command, needs }                                          // :140-144
type CiGate { id, job, run_policy }                                        // :146-150
type CiUpsertStep<T> { inputs, verify, create, resolve, payload_type }     // :173-179
type UpsertInputRef                                                        // :188-195
  = FileSet { selector: FileSetSelector }
  | SubstrateNodeSet { selector: NodeQuery }
  | LensOutputRef { lens, ports }
  | TestClaimRef { claim_id }
  | UpstreamUpsert { step_id }
type CiSelectionReceipt { pr, affected, decisions }                        // :320-324
type SelectionDecision = Run | Skip | CarvedOut { reason }                 // :336-340
type CiCarveout { step_id, reason_code, reason_detail, dissolution_target }// :269-274
```

**Active landing (W2.3 / commit 2026-05-30):** five `GateStep` `CiUpsertStep`
rows landed in the most recent merge ("W2.3 Bucket E: five GateStep CiUpsertStep
rows (full ci_pipeline shadow bijection)"). Bijection with the shadow
`ci_pipeline_step_ids_shadow` universe is now complete (per
`ci_upsert_steps_full_in_scope_step_ids`, `ci.dag:955-958`).

### 3.2 Phase sequence (per `docs/planning/v4-ci-overhaul-2026-05-30.md` §6)

| Phase | Scope | T-24 effect |
|-------|-------|------------|
| **1a** | `ci.dag` sole policy authority for I0–I8 integrity; T-22 interpreter on `ci_pipeline` (S2′); coarse bucket `if:` dissolved | OPEN |
| **1.4** | Land `Upsert<T>` as a usable substrate primitive (`dsl/std/patterns.dag` UPSERT<T> section currently has commented stubs — blocked on parser generics) | substrate landing |
| **1.5** | Every CI step becomes a `CiUpsertStep<T>` Node — verify-first / create / resolve / cache-derived-from-content-hash. `inputs` is typed (no bare-Symbol globs). **No `Always` policy variant** — always-run steps live in explicit `ci_always_run_carveouts` data, each with `reason_code` + `dissolution_target` | OPEN |
| **1b** | Atoms A3–A14 promoted opt-in (`docs/design-ci-dag-overhaul.md`); A6–A8 delete `scripts/check-*` in same PR as TestClaim port | OPEN |
| **2** (A15) | Shape-B `ci.yml` emitted from `CiPipeline`; **all hand-authored YAML deleted** | T-24 **[DONE]** |
| **2.5** | Receipt's `selected: List<CiStepSelection>` actively drives runner fanout; shadow → active | post-T-24 (proposed) |

### 3.3 The IRT-1 / IRT-4 discipline (T-21 / T-24)

Two distinct cache-key concerns, **must not conflate** (per
`src/v4/TASKS.md` ~L1120 and `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` §6):

```
# IRT-1 — schedule (which steps re-run given this diff)
ci_select_from_affected_set(roster, affected) → subset whose declared input
  subgraph intersects the rerun frontier

# IRT-4 — verdict cache (when a scheduled step's verdict is reusable)
verdict(step) = cached_verdict(content_hash(whole CiUpsertStep<T> node))
  # whole node = inputs + verify/create/resolve + tool versions + extdeps
  # NOT content_hash(inputs alone) — that admits stale reuse when oracle drifts (P2)
```

### 3.4 What's NOT yet landed (gaps from elastic ideal)

| Gap | What's missing |
|-----|----------------|
| Active selection | Receipt decisions don't drive runner fanout (still shadow) |
| Cache-key derivation | Non-TestClaim commands still use static `Symbol` tags as cache identity (e.g. `ci_cache_cmd_m1_probe_tag`); must dissolve to `content_hash(whole node)` |
| Output sharing | Bootstrap → consumer fan-out has no L2 cache primitive; the four-compile redundancy in §1.2 is exactly this gap |
| Declared parallelism + execution budget | `ParallelismShape` + `ReducerLaws` (§4.0b) and `ExecutionBudget { expected_cost, provider_model, watchdog }` (§4.0c) not yet landed; authored `worker_count` / `timeout` fields are explicitly **forbidden** (§4.0d) — they bake provider capacity into the step |
| `gunbc` content-addressed emit | Compiler doesn't expose `--manifest <path>` or `--cache-key` flags |
| Parallel module emit | Compiler is sequential per ~1.25s/file at 332-source closure |

---

## 4. Elastic CI — target shape under "infinite compute, stateless runners"

### 4.0 Methodology: model srv1/srv2 intricately first; the abstraction emerges

**Bottom-up, not top-down.** Per MODELING.md M9 (DFS the concept DAG before
defining new types) and CLAUDE.md "Cost of Change = 1" — abstractions are
*emergent from concrete instances*, not pre-designed against one. The
`ComputeFabric` / `ComputeProvider` shape sketched in this section is
**provisional**: it's what the abstraction *would* look like if today's srv1/srv2
were the right factoring. It probably isn't — the right factoring becomes visible
only when ubicloud, gcloud, and other providers are modeled concretely alongside.

**Concrete provider roster (operator roadmap, 2026-05-31):**

| Provider | Footprint class | Why it matters for the abstraction |
|----------|----------------|-----------------------------------|
| **srv1** / **srv2** | Owned bare-metal Ampere Altra arm64 Linux, pooled host | Today's reality; shared `$HOME`, host-wide FIFO, locality-rich L1 |
| **ubicloud containers** | Managed Ubicloud container, ephemeral hermetic Linux | Stateless L1; per-invocation lifecycle; tests the "no shared host state" path |
| **gcloud containers** | GKE / Cloud Run, ephemeral hermetic Linux, per-second billed | Same statelessness; introduces the per-second cost model |
| **Mac mini** | Owned single-host Apple Silicon arm64 **Darwin/macOS** | **The abstraction stress test.** Different OS (not Linux), different FS (APFS, case-sensitivity quirks), different process model (no cgroups, launchd not systemd, SIP/codesigning constraints), no shared-`$HOME` pattern unless explicitly configured. If the `ComputeFabric` interface accommodates macOS cleanly, it's actually general. |
| **WSL** | Linux-on-Windows-host, developer-machine | Distinct storage performance topology (DrvFs penalties on Windows-mounted paths); developer-attention cost class (the human owns the box); irregular availability |

**Five concrete providers, four distinct footprint classes** (bare-metal pool /
ephemeral container / macOS host / WSL developer). The cross-provider
`ComputeFabric` shape is canonical only after **all five are modeled
concretely** — not after one. The user-named insight: **Mac mini's footprint is
deliberately different to force hard thinking about how this fabric actually
works** — without it, the abstraction risks being "two Linux self-hosted pools
with thin wrapping" rather than a genuine compute substrate.

Anything before the five-provider concrete model lands is candidate-vocabulary,
not authority.

**Performance-per-cost scheduling — separate concern.** A meta-scheduler over
`ComputeFabric` providers can pick the best perf-per-cost slot per workload.
Cost models vary sharply: srv1/srv2 + Mac mini are owned (marginal cost ≈ 0,
fixed throughput); gcloud is per-second billed (high throughput, real cost);
ubicloud is metered; WSL is developer-machine attention (cost = "is the human
running CI right now?"). The scheduler's choice function reads
`cost_class: CostClass` + `advertised_throughput` from each provider's slot
offer. **Out of scope for the core abstraction**; lands as a separate Node once
providers are concretely modeled. Naming the meta-scheduler now risks
prematurely shaping it; the abstraction has to be right first.

**What "model srv1/srv2 intricately" means concretely (the agenda):** today's
`SelfHostedRunnerPool` (`ci.dag:364-370`) captures 5 fields — `host`, `arch`,
`core_count`, `runner_count`, `jobserver_token_cap`. That's the tip. The rest of
the structural reality is unmodeled and lives in shell-comments, audit docs, and
incident postmortems. Land it as substrate:

| Fact | Today's authority | Why it matters for the abstraction |
|------|-------------------|-----------------------------------|
| **Shared `$HOME=/home/briansrls/`** across ephemeral worker instances per host | `ci.yml:27-35` comments | srv1/srv2 architectural quirk; ubicloud containers won't share `$HOME` → abstraction must accommodate "no shared root" |
| **Per-job `CARGO_HOME` / `RUSTUP_HOME` indirection** (the rustup race workaround) | `ci.yml:42-45` first step of every Rust job | A workaround for the prior row. In hermetic-container providers this just disappears — the abstraction needs to express "Yes/No: filesystem isolation between jobs is automatic" |
| **`ctrl-build` wrapper** (sccache, dynamic `CARGO_BUILD_JOBS`, memory caps, BuildBuddy opt-in) | `/usr/local/bin/ctrl-build`; session brief | Provider-specific build-environment wrapper. Other providers may have analogues (Cloud Run container image bakes them in; ubicloud may be similar). |
| **`ctrl-jobserver` daemon + host-wide FIFO** at `/var/lib/ctrl/jobserver/host.fifo` | Operator infra; srv2 incident audit doc §5.1 | Currently a host-wide singleton — exactly the "load-bearing-by-luck" pattern (one FIFO misconfigured → silent 20m hang). Stateless providers don't have this layer at all. |
| **sccache identity & location** | Implicit in `ctrl-build` | Per-host? Per-runner? Where exactly? This is L1-cache substrate; needs naming. |
| **Storage topology** (local NVMe? network FS? per-runner `$RUNNER_TEMP` lifecycle?) | Implicit | Determines L1 affinity semantics. Containers' ephemeral FS makes the answer different. |
| **Runner instance lifecycle** (named like `srv2-27-1780203011-420094` — ephemeral?) | GHA actions-runner config | Differs sharply across providers (long-lived pool vs one-shot container). |
| **Memory pressure / cgroup model** | `ctrl-build`'s "container memory caps" mention | What constitutes overcommit, what triggers OOM, what observable signal does the workload get? |
| **OS / kernel / glibc / arch** | implicit; runner labels `[self-hosted, linux, arm64]` | For srv1/srv2, fixed. For container providers, this is *data* (image declares it). |
| **Failure modes catalog** | scattered (audit §5; incidents) | FIFO race, $HOME rustup clobber, swap-on-cap. Each is a `SelfHostedFailureMode` substrate row; the abstraction needs to express which failure modes a provider can exhibit. |

Once these land for srv1/srv2 as `.dag` data — *not as shell comments or audit
prose* — the same exercise on ubicloud and gcloud will surface what's stable
(input declaration, content-hash addressing, slot lifecycle) vs accidental
(shared `$HOME`, host-wide FIFO, locality-aware L1). The abstraction's field
set is *the intersection that turns out to actually matter*.

**Until then, treat §4.0–§4.10 below as candidate-vocabulary.** The
`ComputeFabric` Node + `ComputeProvider` interface + `locality_hint` are useful
sketches for shared understanding, not modeled authority. Authority belongs to
the intricate srv1/srv2 substrate, landed first.

### 4.0a Canonical schema: the five-layer execution chain

The abstraction is **five named layers**, each with a distinct type and a single
job. The chain is unidirectional (each layer projects to the next); reasoning
about cost, parallelism, or scheduling at the wrong layer is the root cause of
most CI design drift.

```text
CiUpsertStep<T>      — what RESULT is needed (CI semantics)
       │
       ▼
WorkUnit<T>           — what must EXECUTE to produce that result
       │
       ▼
ComputeDemand         — what RESOURCES + LAWS the execution requires
       │
       ▼
ComputeProvider       — what a backend (srv1/srv2, Mac mini, gcloud, …) can OFFER
       │
       ▼
ComputeLease          — the chosen allocation (provider + envelope + scope)
       │
       ▼
ExecutionReceipt<T>   — what HAPPENED (output, verdict, cost, perf)
```

Each layer is itself an `Upsert<T>` (fractal — §4.0 methodology applies
recursively): `verify` reads the layer's cache identity; `satisfy` recurses to
the layer above; `create` performs the layer-specific action; `resolve` returns
a stable handle the next layer hashes.

#### Layer 1 — `CiUpsertStep<T>` (CI semantics)

Already landed in `ci.dag:173-179` and being rolled out across W2.3 buckets A–E:

```dag
type CiUpsertStep<T> {
  inputs: List<UpsertInputRef>
  verify: VerifyCheck
  create: CreateAction
  resolve: ResolveExpr<T>
  payload_type: Node
}
```

**What it says:** "this CI gate is satisfied when these declared inputs produce
this verified outcome." Nothing about hosts, cores, runners, or wall-time. CI
authors write at this layer and only this layer.

#### Layer 2 — `WorkUnit<T>` (executable unit)

```dag
type WorkUnit<T> {
  id: WorkUnitId
  inputs: List<ArtifactRef>
  action: WorkAction
  output: ArtifactSpec<T>
  requirements: ComputeDemand
}
```

A `WorkUnit` is what a `CiUpsertStep<T>` projects to when scheduled. Inputs are
content-addressed `ArtifactRef`s (not file paths). Outputs are typed
`ArtifactSpec<T>` claims. The same `WorkUnit` produced by two different
`CiUpsertStep<T>`s can share results — this is the dedup substrate.

**Ingress-boundary discipline.** `ArtifactRef` is the *scheduled* form — but
the CI dependency model still starts from real changed paths and substrate
queries at ingress (`UpsertInputRef` → `ChangeSet` → `FileSetSelector` /
`NodeQuery`). Don't erase that boundary too early; the harness needs to
explain *why this work unit selected this path change*. The materialization
chain is:

```
UpsertInputRef (CiUpsertStep<T>.inputs — typed substrate selector)
  → ChangeSet ∩ FileSetSelector / NodeQuery  (ingress: real paths and node IDs)
  → materialized ArtifactRef                  (content-addressed snapshot)
  → WorkUnit<T>.inputs                        (scheduled form)
```

Scheduled work never re-asks "which files matched?" — that's resolved at
ingress and baked into the `ArtifactRef`. But the chain is auditable end to
end: a path change can be traced through to the work units it selected.

#### Layer 3 — Demand: orthogonal coordinates, not modes

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
  effects: List<EffectBoundary>
}

type ResourceEnvelope {
  cpu: Option<CpuRequirement>
  gpu: Option<GpuRequirement>
  memory: Option<MemoryRequirement>
  storage: Option<StorageRequirement>
  network: Option<NetworkRequirement>
}
```

**Crucial discipline (per MODELING.md coproduct-vs-coordinate check):**
resources are **orthogonal coordinates of a record**, not variants of a sum.
A real workload inhabits multiple dimensions simultaneously — a GPU training
job needs `gpu + cpu + memory + storage + network`; a Rust build job needs
`cpu + memory + filesystem + artifact cache`; a data-transfer job needs
`network + storage`. Modeling resources as
`ComputeKind = CPU | GPU | Storage | Network` (an exclusive variant) is
**forbidden** (§4.0d): a single inhabitant carries values across all
dimensions, so they're record coordinates, not sum cases.

The demand says **what the work requires**, never **what hardware to use**.
"arm64 Linux + Rust toolchain + ≥4GiB RAM + hermetic-fs isolation + no
ambient network" — yes. "srv1, 20 workers" — no (authored tuning, forbidden).

#### Layer 4 — Supply: facts, not "machines"

`Machine` is a leaky abstraction — it collapses orthogonal axes (hostname,
physical box, CPU package, OS install, container runtime, network identity,
runner registration, cache locality, scheduler slot). Modeling discipline:
**facts first, projections later**. The primitive types are factored along
those axes:

```dag
// Processor: CPU / GPU / accelerator are MUTUALLY EXCLUSIVE device kinds —
// this IS a coproduct (a single processor is one of these, not several).
// Resource REQUIREMENTS (Layer 3) are different — those are coordinates.
type ProcessorKind
  = CpuProcessor { cpu: CpuFacts }
  | GpuProcessor { gpu: GpuFacts }
  | AcceleratorProcessor { accelerator: AcceleratorFacts }

type CpuFacts {
  architecture: CpuArchitecture
  vendor: Symbol          // ampere, apple, intel, amd, …
  model: Symbol           // altra, m2_pro, xeon, epyc, …
  cores: Int
  threads: Int
  instruction_sets: List<InstructionSet>
}

type GpuFacts {
  vendor: Symbol
  model: Symbol
  compute_capability: Option<Symbol>
  memory: MemoryFacts
  supported_runtimes: List<GpuRuntime>
}

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

type OperatingSystemSurface {
  kernel: KernelFamily       // linux, darwin, windows_nt, linux_guest_on_windows
  distro_or_product: Symbol  // ubuntu, macos, windows, wsl_ubuntu
  version: Symbol
  filesystem_semantics: FileSystemSemantics
  process_semantics: ProcessSemantics
}

type ExecutionSurface {
  os: OperatingSystemSurface
  isolation: IsolationBoundary
  container_runtime: Option<ContainerRuntime>
  toolchains: List<ToolchainCapability>
  mounted_storage: List<StorageMount>
  network: List<NetworkInterface>
}

type ComputeHost {
  identity: HostIdentity
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
```

**"srv1" / "srv2" / "Mac mini" / "WSL guest" are not primitives.** They are
named projections over the factored facts above. A GitHub self-hosted runner,
container, WSL shell, SSH session, Fargate task, or Mac mini user session is
**an `ExecutionSurface` realized on a `ComputeHost`** — never the same thing as
the host. If `Machine` exists at all, it's a convenience projection for humans
and dashboards:

```dag
fn machine_view(host: ComputeHost, surface: ExecutionSurface) -> MachineView
```

Authority remains in the factored facts, not in the projection.

#### Layer 4 (continued) — Offer / Lease over fact bundles

```dag
type ComputeOffer {
  provider: ProviderIdentity
  supply: ComputeSupplyFacts
  available_window: AvailabilityWindow
  cost_quote: Option<CostEstimate>
  constraints: List<ProviderConstraint>
}

type ComputeLease {
  offer: ComputeOffer
  demand: WorkDemand
  allocation: AllocationReceipt
  eligibility: Witness<ComputeLeaseEligibility>
}
```

#### Matching: homomorphism over fact bundles

Eligibility is structural — a `satisfies` function maps demand dimensions to
supply dimensions:

```dag
fn satisfies(
  supply: ComputeSupplyFacts,
  demand: WorkDemand
) -> Witness<ComputeLeaseEligibility>
```

The scheduler does NOT say *"choose srv1 because fast."* It says:

> "`WorkDemand` requires arm64 Linux + Rust toolchain + ≥16GiB RAM +
> filesystem + shared CAS. srv1's `ExecutionSurface` satisfies those
> dimensions. srv2 also satisfies them. gcloud container satisfies them if
> image has Rust toolchain. Mac mini does **not** satisfy Linux requirement
> unless demand accepts macOS or containerized Linux. WSL satisfies Linux-ish
> process surface but has path/cache/network boundary facts."

Performance-per-cost selection is downstream optimization **over eligible
offers and observed receipts**, never an eligibility heuristic. Absent receipt
evidence → scheduler reports `insufficient evidence`, not a silent guess.

#### Layer 5 — `ExecutionReceipt<T>` (what happened)

```dag
type ExecutionReceipt<T> {
  work: WorkUnit<T>
  lease: ComputeLease
  output: Outcome<ArtifactRef<T>>
  performance: PerformanceReceipt
  cost: CostReceipt
  started_at: LogicalTime
  finished_at: LogicalTime
}

// Receipt quality matters — perf-per-cost selection must read measurement
// confidence, not just point values. Otherwise "highest performance per cost"
// degrades into a hidden heuristic.
type PerformanceReceipt {
  work_shape: WorkShape
  provider: ProviderIdentity
  duration: Duration
  cache_state: CacheState                  // cold / L1-warm / L2-hit / etc.
  sample_count: Int                         // 1 measurement vs N samples
  measurement_context: ExecutionSurface     // where it ran — drives generalizability
  confidence: MeasurementConfidence         // point / range / distribution
}

type CostReceipt {
  provider: ProviderIdentity
  billable_units: Cost
  pricing_source: PricingSource             // vendor docs / observed bill / negotiated
  amortization_scope: Option<AmortizationScope>  // for owned hardware: how is fixed cost split?
}
```

Receipts are the **only substrate the scheduler reads** to make
perf-per-cost decisions in the future. "This shape of work on srv1 took X;
on gcloud took Y; cache hit on srv2 saved Z" — those facts come from receipts,
not from heuristics. If no receipt evidence exists for a class of work, or if
existing receipts have `confidence: SingleSample`, the scheduler reports
`insufficient evidence` rather than silently guessing. Mechanically-checkable
acceptance: "perf-per-cost ranking is computable only if all candidate offers
have at least one receipt with `confidence ≥ Range`" (or similar threshold —
exact bar is operator-bar).

### 4.0b Parallelism is algebraic, not numerical

Per-step `worker_count: Int` as an authored field is the wrong shape — it bakes
physical-capacity assumptions into the step and forces re-tuning whenever
providers change. Parallelism belongs in `ComputeDemand` as a **declared
algebra**:

```dag
type ParallelismShape
  = SingleWorkItem
  | IndependentShards { shard_count: Int }
  | DependencyGraphParallel { graph: WorkGraph }
  | PartitionedReduce {
      partitioner: Partitioner
      map: WorkAction
      reduce: Reducer
      laws: ReducerLaws
    }

type ReducerLaws {
  associative: Witness<Associativity>
  commutative: Option<Witness<Commutativity>>
  identity: Option<IdentityElement>
  idempotent: Option<Witness<Idempotency>>
}
```

The scheduler derives worker count from `(parallelism_shape × provider_capacity ×
cache_locality)`. MapReduce / batch fan-out is *only* available when
`ReducerLaws.associative` (at minimum) is witnessed; absent laws mean the
scheduler must run a narrower plan or fail closed. This matches the lawful-
rewrite discipline elsewhere in the codebase (parallel-map / tree-reduce /
CUDA lowerings all require `LawfulRewriteWitness`).

### 4.0c Watchdog ≠ cost (operational vs semantic)

The doc previously named per-step `timeout: Duration` as a missing field. That
conflates two distinct concerns:

```dag
type ExecutionBudget {
  expected_cost: SymbolicCost       // semantic — derived from work complexity
  provider_model: ProviderCostModel  // per-provider rate / capacity
  watchdog: WatchdogLimit            // operational kill-switch, conservative
}
```

`watchdog` is the runaway-process kill-switch — operational safety, not
program complexity. `expected_cost` is the modeled complexity claim. Mixing
the two (today's hardcoded `timeout-minutes: 35` in YAML) is what makes
timeouts feel arbitrary and brittle.

### 4.0d Forbidden — substrate violations that re-introduce the old shape

The closed-system discipline (per
`feedback_heuristics_recoverable_to_substrate`) names the failure mode: when
a heuristic or shortcut is added in place of a structural fact, future
divergence is silent. The following are forbidden anywhere in the chain:

- **Host-specific fields on `CiUpsertStep<T>`** (`host: srv1`, `runner_label: arm64-self-hosted`). CI steps describe results, not placement.
- **`worker_count: Int` as authored tuning metadata**. Parallelism is declared as `ParallelismShape`; counts are scheduler-derived.
- **Scheduler-policy / run-mode enums** that compress unknowns into named cases (`CiRunPolicy = Eager | Conservative | Aggressive`). Heuristics are symptoms of missing structural facts.
- **Cache identity from filesystem residue** ("the file was at this path last run"). Cache identity is `content_hash(complete subgraph)`, projected at lookup time.
- **Provider-specific assumptions outside `ComputeProvider` rows** (e.g., `ctrl-jobserver` FIFO path hardcoded in a step's `create`). Provider-private facts stay in provider rows.
- **`timeout: Duration` as the sole budget carrier**. Split into modeled `ExecutionBudget` (`expected_cost`, `provider_model`, `watchdog`).
- **Authored cache keys** (`cache_key: "..."`). Already forbidden in `ci.dag:163-166` — cache identity is derived from `content_hash(complete CiUpsertStep<T>)`.
- **`ComputeKind = CPU | GPU | Storage | Network` as exclusive variants.** Resource *requirements* (Layer 3) are orthogonal coordinates — a single workload inhabits multiple dimensions. Coproduct here would deny GPU-trains-with-CPU-orchestration and storage-heavy-with-network-egress jobs. (Processor *devices* in Layer 4 ARE a coproduct because a single device IS one kind; the coproduct-vs-coordinate check is contextual.)
- **`host: Symbol` or `machine: Symbol` as eligibility authority.** Eligibility is structural over factored facts; "this work runs because hostname matches" is a heuristic for missing demand/supply dimensions.
- **`Machine` as a primitive concept.** Use `ComputeHost` (composition of processors / memory / storage / network) + `ExecutionSurface`; `MachineView` is at most a convenience projection for dashboards.
- **Scheduler risk / fallback heuristic enums** (`if no receipts, choose cheapest`). Absent receipts → `insufficient evidence`, fail-loud, not silent guess.
- **`CacheKind = GHA | sccache | BuildBuddy | Local | …` as exclusive variants.** Cache facts (key derivation, value shape, locality, eviction, atomicity, auth, latency) are orthogonal coordinates — same coproduct-vs-coordinate trap as `ComputeKind`. Cache backends are modeled as `CacheStore` *records* whose fields take values from each dimension.
- **GHA-specific / Rust-specific / sccache-specific cache semantics above `CacheStore`.** Existing caches are *storage backends*, not separate cache designs. The Upsert<T> contract is invariant; only `CacheStore` rows know about backend specifics.
- **Hand-authored cache keys at any layer** (`hashFiles(...)` in YAML, `cache_key:` strings in step definitions, sccache `--cache-mode-key` overrides). Cache identity is always `content_hash(canonical Upsert<T> subject)`, projected into the backend's native key format at lookup time.
- **Bare-blob cache hits without `CachedArtifactReceipt`.** A cache must prove the artifact was produced by an `ExecutionReceipt` whose `WorkUnit` projects to the same `content_hash`; "this file existed at this key" is not sufficient (kills wrong-cache-hit bug class).
- **Filesystem residue as proof of cache state.** "The file was at this path last run" must be verified by content-hash match, not assumed.

Anything in this list is a re-introduction of the substrate gap the chain is
meant to dissolve.

### 4.0g Caching: model each concrete cache interface first; one discipline emerges

**Same methodology as §4.0 — bottom-up.** Do NOT pre-design a generic cache
abstraction (`CacheKind = GHA | sccache | BuildBuddy | Local | …` is exactly
the compressive-coproduct anti-pattern §4.0d forbids for `ComputeKind`). Model
each existing cache interface as concrete orthogonal facts; the common
abstraction emerges from comparing the rows.

#### Concrete cache-interface roster (model each as a fact row in `dsl/std/`)

| Cache interface | Where it lives | Native key format | Value shape | Persistence | Eviction | Atomicity |
|---|---|---|---|---|---|---|
| **GHA Actions Cache** | GitHub-managed network | hand-authored string (today: `hashFiles(...)`) | tar archive of paths | cross-host network | 7-day TTL + ~10GB/repo cap | write-then-commit |
| **sccache** | configurable backend (local disk / S3 / Redis / BuildBuddy) | hash(rustc command + inputs + env) | rustc output blob (object/rmeta) | backend-specific (per-host typical) | backend-specific | write-then-rename |
| **BuildBuddy CAS** | BuildBuddy server (network), via `ctrl-build --remote` | `content_hash(value)` — natively content-addressed | arbitrary action-result blob | cross-host network | BuildBuddy-managed | write-then-commit |
| **Cargo `target/` (incremental)** | per-`CARGO_TARGET_DIR` filesystem | Cargo internal fingerprint (content-hash of inputs + features + flags) | object files + dep files + metadata | per-runner filesystem | manual (`cargo clean`) | per-file |
| **`~/.cargo/registry/`** | per-`CARGO_HOME` filesystem | crate-name + version + source checksum | downloaded crate sources | per-host filesystem | rarely | atomic-extract |
| **rustup toolchain store** | per-`RUSTUP_HOME` filesystem | toolchain name + version + arch | toolchain binary tree | per-host filesystem | manual | atomic-extract |
| *(future)* Mac mini local sccache | local APFS | sccache-native | sccache-native | per-host filesystem | sccache config | write-then-rename |
| *(future)* S3-backed sccache | AWS S3 | sccache-native (over S3 keys) | sccache-native | cross-host network | S3 lifecycle | S3 object PUT atomicity |

#### Orthogonal fact dimensions (emerge from comparing the rows)

| Dimension | Range of values across the roster |
|---|---|
| **Key derivation** | content-addressed-by-value / hand-authored / native-internal-hash |
| **Value shape** | bytes / structured artifact / tar archive / file tree |
| **Persistence locality** | in-process / per-runner filesystem / per-host filesystem / cross-host network |
| **Eviction policy** | TTL / LRU / size-bounded / never / manual |
| **Atomicity** | per-file / write-then-rename / write-then-commit / two-phase |
| **Auth scope** | none / filesystem perms / API key / network ACL |
| **Read latency class** | ns (in-process) / µs (local disk) / ms (LAN) / 10s ms (WAN) |

**Independence proof.** The roster occupies 4 of 4 possible
`(content-addressed × locality)` combinations:

| | Content-addressed | Hand-keyed |
|---|---|---|
| **Cross-host network** | BuildBuddy CAS | GHA Actions Cache |
| **Per-host filesystem** | sccache, Cargo target/ | rustup toolchain store |

Dimensions are orthogonal *in practice*, not just on paper. Modeling them as a
`CacheKind` variant would deny these inhabited combinations.

#### The fractal discipline emerges (don't pre-design — concrete row first, view second)

Once each cache is modeled as orthogonal facts, the **concrete fact row** is:

```dag
// Authority — one row per concrete backend (GHA Actions Cache, sccache, …).
// Lands FIRST. Consumers may project a view from it; this is not the view.
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
}

// Derived projection — convenience view for consumers that want a uniform
// handle (e.g., the scheduler picking among offers). NEVER the first modeled
// authority; always derived from CacheInterfaceFacts rows. Same shape rule as
// MachineView (§4.0a Layer 4) — a projection, not a primitive.
type CacheStoreView {
  facts: CacheInterfaceFacts
}
```

— a record of orthogonal facts, not an enum-of-cases. Each concrete backend
gets one `CacheInterfaceFacts` data row (`gha_actions_cache_facts`,
`sccache_local_facts`, `buildbuddy_cas_facts`, `cargo_target_dir_facts`,
`rustup_toolchain_store_facts`, …); the view follows.

**Vendor-fact discipline.** `CacheInterfaceFacts` rows assert claims about
external services (TTL, size cap, atomicity, restore behavior, key
semantics). Exploration-grade rows like the roster above are *candidate-
vocabulary*. Before ratification, each row must carry either (a) a cited
source (vendor docs, RFC) or (b) a runner-observed receipt that verifies the
claim against actual behavior. Asserted-without-evidence rows are forbidden
at ratification.

#### Identity vs transport: separate `ArtifactIdentity<T>` from `BackendCacheKey`

A cache hit involves two distinct identities — conflating them re-introduces
the "backend invents the identity" failure mode:

```dag
// Semantic — what artifact this IS, projected from the complete Upsert<T> subject.
// Invariant across backends; defines the artifact's true identity.
type ArtifactIdentity<T> {
  subject_digest: Hash          // content_hash(canonical Upsert<T> subject)
  artifact_kind: ArtifactKind<T>
}

// Transport — how a specific backend addresses this artifact internally.
// Backend-specific; varies across CacheInterfaceFacts rows; never authority.
type BackendCacheKey {
  store: CacheInterfaceId
  key: ProviderKey              // tagged union per backend: opaque string for
                                 // GHA, sccache-hash for sccache, CAS digest for
                                 // BuildBuddy, Cargo fingerprint for target/
}

// The projection from semantic identity to backend transport — explicit,
// auditable, replaceable per backend without changing semantic identity.
type CacheKeyProjection<T> {
  artifact: ArtifactIdentity<T>
  backend: CacheInterfaceFacts
  key: BackendCacheKey
}
```

This makes the layering explicit: `ArtifactIdentity` is the cross-backend
semantic carrier; `BackendCacheKey` is the transport per backend; the
projection is the function that maps one to the other. GHA, sccache, Cargo,
and BuildBuddy never become identity authorities — they only host transport.

#### Cache hit must return a receipt, not just bytes

```dag
// Required result of a successful cache lookup at any tier. A bare blob hit
// without this receipt is rejected (§4.0d). Kills the wrong-cache-hit bug class.
type CachedArtifactReceipt<T> {
  artifact: ArtifactIdentity<T>
  backend_key: BackendCacheKey
  producer: ExecutionReceipt<T>       // who originally produced this artifact
  verified_subject_digest: Hash       // matched against artifact.subject_digest
  content_digest: Hash                // matched against the bytes received
}
```

The cache proves *this artifact was produced by an `ExecutionReceipt` whose
`WorkUnit` projects to the same `content_hash`*. "A file existed at this
key" is not sufficient — the producer-chain receipt is required.

#### Lookup / write / miss semantics — restore is not just key derivation

Cache backends differ sharply in *behavior*, not just key format. Model
those behaviors as orthogonal coordinates too:

```dag
type CacheLookupSemantics
  = ExactKeyOnly                                       // BuildBuddy CAS
  | PrefixFallback { ordered_prefixes: List<KeyPrefix> }  // GHA actions/cache@v4 restore-keys
  | NativeInternalLookup                                // sccache, Cargo target/
  | ContentAddressLookup                                // CAS-native

type CacheWriteSemantics
  = WriteOnce
  | OverwriteAllowed
  | WriteThenCommit          // GHA Cache: PUT then commit; visible after commit
  | WriteThenRename          // sccache: write to tmp, rename atomically
  | ProviderInternal         // Cargo target/, rustup

type CacheMissSemantics
  = MissThenCreate                                  // L0/L1 typical
  | MissThenFallback { lower_tier: CacheInterfaceId }  // L0→L1→L2 cascade
  | MissIsDiagnostic                                // verify-only stores
```

Concrete row examples:
- **GHA Actions Cache:** `lookup_semantics: PrefixFallback`, `write_semantics: WriteThenCommit`, `miss_semantics: MissThenCreate`
- **BuildBuddy CAS:** `lookup_semantics: ContentAddressLookup`, `write_semantics: WriteOnce`, `miss_semantics: MissThenFallback { lower_tier: external_source }`
- **sccache:** `lookup_semantics: NativeInternalLookup`, `write_semantics: WriteThenRename`, `miss_semantics: MissThenCreate`
- **Cargo `target/`:** `lookup_semantics: NativeInternalLookup`, `write_semantics: ProviderInternal`, `miss_semantics: MissThenCreate`

These facts must be modeled per row before the abstraction is ratified.

#### `KeyDerivation` — concrete facts, not just three-way classification

Don't stop at `KeyDerivation = ContentAddressed | HandAuthored | NativeInternal`.
That's an opening sketch. The substrate needs concrete fields per backend:

```dag
type KeyDerivationFacts {
  classification: KeyDerivationClass         // ContentAddressed | HandAuthored | NativeInternal
  inputs_considered: List<InputSurface>      // what does the backend hash?
  overwritable: Bool                          // can a put with same key replace?
  prefix_fallback_allowed: Bool               // can a non-exact match return?
  content_verified_on_read: Bool              // does the backend verify bytes?
  visibility_scope: VisibilityScope           // repo / org / network / world
  invalidation_triggers: List<InvalidationTrigger>  // what makes a key go stale?
}
```

Each `CacheInterfaceFacts` row's `key_derivation` must answer those questions
concretely. "GHA Cache uses hand-authored strings" is insufficient — the
substrate needs the full fact set so the harness can audit projections.

Then the `Upsert<T>` fractal discipline applies uniformly: at every layer of
the DAG, the artifact's identity is `ArtifactIdentity<T> { subject_digest:
content_hash(canonical subject) }`; the backend transport is computed by the
`CacheKeyProjection` function for that backend's `KeyDerivationFacts`:

| Backend `key_space` | Projection |
|---|---|
| Content-addressed (BuildBuddy CAS) | identity — already matches |
| Hand-authored string (GHA Actions Cache) | emit `content_hash` as the opaque key string |
| Native-internal-hash (sccache) | sub-layer; opaque inside the coarser Upsert<T>'s `create` |
| Cargo fingerprint (Cargo `target/`) | L0 internal to a cargo invocation; opaque to fractal layers above |

The fractal model **does not replace** sccache's or Cargo's internal hashing.
It wraps them: a `ModuleEmitStep<T>` Upsert<T> at the fractal level can use
sccache as L1 (the storage backend), with sccache's internal keying private to
its implementation.

#### Derived: layer-to-store mapping (worked projection — not the design)

| DAG layer | Today's backing | Canonical `CacheInterfaceFacts` row (fractal) |
|---|---|---|
| `CiUpsertStep<T>` verdict | T-22 receipt cache; M1 absent | L2 (`gha_actions_cache_facts` / `buildbuddy_cas_facts`) |
| `WorkUnit<T>` output (gunbc emit) | none — the 4× redundancy gap | L2 (`buildbuddy_cas_facts`) |
| `ModuleEmitStep<T>` output | sccache | L1 (`sccache_local_facts`) |
| rustc invocation (CGU-level) | Cargo `target/` | L0 (`cargo_target_dir_facts`, scoped to `$RUNNER_TEMP/target/`) |
| Crate download | `~/.cargo/registry/cache/` | L1 + optional L2 mirror |
| Toolchain | rustup-managed | L1 (`rustup_toolchain_store_facts`) |
| Compute lease | scheduler-internal | L1 lease-state cache |

#### Derived: cascade (one shape, every layer)

```
Upsert<T>.verify  → L0 (in-runner ephemeral target/, sccache local)
                  │  miss
                  ▼
                  → L1 (srv1-cache / srv2-cache / Mac mini local / sccache shared)
                  │  miss
                  ▼
                  → L2 (BuildBuddy CAS / GHA Actions Cache)
                  │  miss
                  ▼
                  → satisfy (recurse into upstream Upsert<T>s, then create)
```

#### Two pairings that need explicit substrate modeling

**`ctrl-build` ↔ sccache.** `ctrl-build` is a runner-side wrapper that sets
up the cargo execution environment (`CARGO_TARGET_DIR`, sccache backend
config, `CARGO_BUILD_JOBS` cap, memory caps). It is NOT a separate cache
layer — it's part of the `ComputeProvider.execution_surface` definition
(toolchain capability + env wiring) and wires existing `CacheInterfaceFacts`
rows (`sccache_local_facts`, `cargo_target_dir_facts`) into the runner
instance.

**Cargo registry ↔ optional L2 mirror.** Today `~/.cargo/registry/` is
per-host; cache miss = re-download from crates.io (external network). A
modeled L2 backing means the crate-download Upsert<T>'s `verify` first
queries the L2 mirror (keyed by
`content_hash(crate-name, version, source-hash)`), falls back to crates.io,
then puts to L2. No per-host filesystem assumption; ground truth is
canonical regardless of whether srv1, Mac mini, or a fresh container made
the request.

### 4.0f Modeling DFS worksheets — acceptance criteria (TWO worksheets, not one)

The substrate is **ratified only when these falsification cases pass** — they
prove the abstraction is real, not srv1/srv2-shaped with thin wrapping.
**Compute and caching are orthogonal concerns; dispatch as two distinct
worksheets** to proud-pike-680 (per §7 owner-table). Compose only through
`ExecutionReceipt`.

#### Worksheet A — Compute fabric facts

| # | Case | Pass condition |
|---|------|---------------|
| 1 | srv1/srv2 as `ComputeSupplyFacts` rows | No host-specific `CiUpsertStep<T>` field anywhere in pipeline |
| 2 | Mac mini as `ComputeSupplyFacts` row | Apple Silicon `CpuFacts` + macOS `OperatingSystemSurface` lands without changing CI schema |
| 3 | WSL as `ComputeSupplyFacts` row | Host/guest/path/network facts explicit (`linux_guest_on_windows` kernel, `wsl_path_translation` filesystem semantics, `wsl_nat_or_bridged` network) |
| 4 | gcloud / ubicloud container provider | New `ComputeSupplyFacts` row with no new run-mode enum anywhere in the chain |
| 5 | GPU-capable provider | `ResourceEnvelope.gpu: Some(...)` coordinate; **never** `ComputeKind = GPU` variant |
| 6 | Storage-heavy provider | `storage + network` coordinates in demand; provider's `StorageDevice[]` + `NetworkInterface[]` match |
| 7 | Single `WorkDemand` matches multiple providers | `satisfies(supply, demand)` returns `Witness<ComputeLeaseEligibility>` for srv1, srv2, gcloud-container, *and* (where compatible) Mac mini / WSL |
| 8 | Ineligible provider fails closed | `satisfies` returns `Rejected { reason: missing_fact(...) }` naming the specific missing demand dimension; no silent fallback |

**Worksheet A type outputs** (land in `dsl/std/` / `src/v4/std/`):

```text
ProcessorKind / CpuFacts / GpuFacts / AcceleratorFacts
MemoryDevice / MemoryFacts
StorageDevice
NetworkInterface
OperatingSystemSurface
ExecutionSurface
ComputeHost
ComputeSupplyFacts
WorkDemand
ResourceEnvelope
ComputeOffer
ComputeLease
ExecutionReceipt
PerformanceReceipt (with sample_count, measurement_context, confidence)
CostReceipt (with pricing_source, amortization_scope)
ProviderCostModel
ParallelismShape
ReducerLaws
ExecutionBudget (with WatchdogLimit separated from SymbolicCost)
```

#### Worksheet B — Cache interface facts

| # | Case | Pass condition |
|---|------|---------------|
| 9 | GHA Actions Cache as `CacheInterfaceFacts` row | Hand-authored `hashFiles(...)` patterns disappear from emitted workflow; harness derives `BackendCacheKey` via `CacheKeyProjection` from canonical `ArtifactIdentity<T>` |
| 10 | sccache as `CacheInterfaceFacts` row | sccache-native keying stays internal (`NativeInternalLookup`); the coarser Upsert<T> grain (module-emit / cargo-invocation) is what the fractal model addresses; sccache is L1 backing |
| 11 | BuildBuddy CAS as `CacheInterfaceFacts` row | Routed via `ctrl-build --remote`; canonical `ArtifactIdentity<T>.subject_digest` matches BuildBuddy's native CAS key 1:1 (`ContentAddressLookup`) |
| 12 | Cargo `target/` as `CacheInterfaceFacts` row | Modeled as L0 (per-runner ephemeral); never authority; cleared with the runner lease |
| 13 | rustup toolchain store as `CacheInterfaceFacts` row | Modeled with `Symbol` keying (toolchain name + version + arch) — different `KeyDerivationFacts` than other rows; proves orthogonality |
| 14 | Adding a new cache backend (Mac mini local sccache, S3 mirror, etc.) | One new `CacheInterfaceFacts` fact row; no change to higher-layer `Upsert<T>` definitions or `WorkUnit<T>` types |
| 15 | Wrong-cache-hit protection | A bare-blob hit without a matching `CachedArtifactReceipt { producer: ExecutionReceipt { work: WorkUnit<T> } }` is rejected; cache must prove `verified_subject_digest == artifact.subject_digest` AND `content_digest == hash(bytes)` |
| 16 | Cache backends are orthogonal-fact composition, not a kind enum | The 4 of 4 `(content-addressed × locality)` combinations from §4.0g are all representable; no `CacheKind` coproduct anywhere |
| 17 | Same `ArtifactIdentity<T>` projects into different backends differently | `CacheKeyProjection(ArtifactIdentity<X>, gha_actions_cache_facts)` ≠ `CacheKeyProjection(ArtifactIdentity<X>, buildbuddy_cas_facts)` as `BackendCacheKey` (different transport), but both resolve back to the same `ArtifactIdentity<X>` (identity invariant under transport) |
| 18 | Toolchain version change changes `ArtifactIdentity` when output-affecting | Rust toolchain bump → new `subject_digest` → cache miss → re-emit; non-output-affecting host change → no identity change |
| 19 | Vendor row evidence | Each `CacheInterfaceFacts` row carries either a cited vendor source OR a runner-observed verification receipt — asserted-without-evidence rows are rejected at ratification |

**Worksheet B type outputs** (land in `dsl/std/`):

```text
CacheInterfaceFacts (concrete row — lands first)
CacheStoreView (derived projection — never first authority)
KeyDerivationFacts (with concrete fields per backend)
KeyDerivationClass
CacheLookupSemantics
CacheWriteSemantics
CacheMissSemantics
ConsistencyModel
StorageSurface
ValueShape / PersistenceLocality / EvictionPolicy / AtomicityModel / AuthScope / ReadLatencyClass
ArtifactIdentity<T>
ArtifactSpec<T>
BackendCacheKey
ProviderKey
CacheKeyProjection<T>
CacheLookup<T>
CacheEntry<T>
CachedArtifactReceipt<T>
InputSurface
VisibilityScope
InvalidationTrigger
```

**Independence acceptance.** Worksheets A and B compose only through
`ExecutionReceipt<T>.output: Outcome<ArtifactRef<T>>` and
`ExecutionReceipt<T>` consumption by the `producer:` field of
`CachedArtifactReceipt<T>`. No type in Worksheet A imports a type from
Worksheet B except via these two interfaces; vice versa. Cross-coupling
beyond those interfaces is a worksheet-scope violation — escalate.

Per `feedback_eliminate_x_directives`: this isn't an "eliminate srv1/srv2
specialness" directive — it's a positive substrate-landing for orthogonal
compute and cache facts. The dissolution of srv1/srv2-specific CI semantics
is the *downstream consequence* of the substrate landing, not the goal itself.

### 4.0e (provisional, superseded by §4.0a above) Frame: one pattern, every layer

> Retained as historical context — §4.0a is the canonical schema.

**The pattern is Upsert<T>.** Not "Upsert<T> for CI steps, plus a separate cache
design, plus a separate runner-pool design, plus a separate compiler-internals
design." Same shape, every layer:

```
verify  → "is the desired output already at this layer's address?"
satisfy → "if not, recursively upsert the inputs at the layer below"
create  → "execute the layer's specific action"
resolve → "return a stable handle / verdict the layer above can hash"
```

Concrete instances (each is one `Upsert<T>` row, not a bespoke subsystem):

| Layer | Upsert<T> instance | `verify` reads | `satisfy` recurses into | `create` does |
|-------|-------------------|----------------|------------------------|---------------|
| **CI step** | `CiUpsertStep<T>` (`ci.dag:173`) | step verdict cache (L2 by `content_hash(node)`) | upstream `CiUpsertStep<T>` per `UpstreamUpsert` | runner dispatch |
| **Compiler module emit** | per-module emit step | per-module output content-hash cache | module's dep closure (other module emits) | compile that module |
| **Cache fetch** (L2 hit) | L2 read | L2 store for this key | — | network/disk read |
| **Cache miss → fall to L1** | L1 read | L2 upsert | — | local read |
| **Runner allocation** | resource-manager upsert | "is there a free runner with affinity for this step?" | jobserver-token upsert (acquire) | spawn runner instance |
| **Jobserver token** | token upsert | per-runner ephemeral pool | — | hand out a token |
| **Bootstrap stage** | `BootstrapStageCompile` (`ci.dag:113`) | stage artifact content-hash | prior stage output | run that stage's compile |

This isn't an analogy — it's the same `Upsert<T>` Node, instantiated at different
levels of the DAG. The `dsl/std/patterns.dag` UPSERT<T> canon (per
`docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`) is meant to be applied
recursively. "Don't reinvent the wheel per layer" = don't write a parallel mental
model for caches vs runners vs compiler internals; each is a row in a Node graph
whose shape is fixed.

**Architectural payoff of fractal Upsert<T>:**
- One implementation, not N. The interpreter that walks `ci_pipeline` is the same
  interpreter that walks an emit step's per-module dep closure (just rooted lower).
- One cache discipline (`content_hash(whole node)`) — no per-layer ad-hoc keying.
- One verdict shape (`Outcome<...>`) — aggregator at any level is a fold.
- One escape valve (`ci_always_run_carveouts`-style explicit data) when the
  layer below isn't yet modeled.

**Workload view: ask the network for compute; don't name machines.**
At the modeling layer, the workload (a `CiUpsertStep<T>`) declares only its
*needs*: deterministic ID (content hash), inputs (content-addressed), and
optional hints (`max_useful_parallelism`, `min_memory_bytes`, `arch_constraint`,
`locality_hint: Option<PriorRunIdentity>`). The workload does **not** name srv1,
srv2, a runner pool, or any other physical fact. The request to the network is
"give me a compute slot to run this Upsert<T>" — the network decides what to
hand back, and the workload adapts (jobserver runs at whatever parallelism the
slot exposes; no step pins to "N cores").

**Network view: `ComputeFabric` is a pluggable provider `Upsert<T>`.** The
fabric is one Node whose `create` phase resolves an abstract request to a
concrete runner instance. Concrete providers plug into it. Today's instance is
srv1+srv2 self-hosted (`ci_self_hosted_runner_pools`, `ci.dag:420-423`);
swapping to Fargate, k8s, BuildBuddy RBE, or GHA hosted runners is a new
provider Node, not a workload-layer change. Each provider declares its own:

- **capacity** — `advertised_slots: AdvertisedSlots` (what the provider will hand
  out right now: count, core/mem profile, arch). Dynamic, not pinned.
- **locality model** — provider-private. Stateful providers (srv1/srv2,
  long-lived k8s pools) advertise per-instance L1 affinity; stateless providers
  (Fargate, single-use containers) advertise "no L1 — every cold start hits L2."
- **cost class** — for future scheduling: cheap-and-slow (queue tolerant) vs
  fast-and-expensive (latency-sensitive critical path).
- **boundary penalties** — what crossing this provider's physical fault lines
  costs (e.g., for srv1↔srv2: a network round-trip on L1 miss; for Fargate:
  always cold L1).

**Splitting work across boundaries is the provider's no-go, not the workload's.**
One step = one slot, by `ComputeFabric` invariant. If the workload could fit on
srv1 *or* srv2, the fabric picks one (locality-warm preferred) and that's the
slot. The workload never splits its working set across provider boundaries —
that decision is made one layer down, transparent to the workload.

**Decoupling payoff.** If srv2 dies, the fabric's `verify` for srv2-pinned
affinity misses; `satisfy` falls back to srv1 or to L2-cold; workload sees a
slower step, never a broken step. If we add a Fargate provider tomorrow, the
fabric grows by one row in its provider list; workload code does not change.
"Give me as much compute as the network wants to" is literal: the fabric
returns a slot, the workload runs in it, the jobserver scales to it.

The subsections below (§4.1–§4.10) are each one instance of this pattern at a
specific layer. Treat them as worked examples of §4.0, not as parallel designs.

### 4.1 First principle: one Node = one schedulable work unit

Today: `ci_v4` runs 26 sequential steps in one runner process. Steps that don't
depend on each other (M1 emit ⊥ bootstrap compile) still serialize because they
share a single runner's working directory.

Elastic: every `CiUpsertStep<T>` row in `ci_pipeline` = an atomic work unit.
Sibling steps with no `needs` edge run on **independent runner instances** in
parallel. The job DAG = the DAG of `CiUpsertStep<T>` nodes; there's no enclosing
"`ci_v4` job" container.

Consequence: the wall-time floor for a workflow run is the longest *chain* of
`needs` edges through `ci_pipeline`, not the sum of step times. For PR #4074's
shape, the chain is `v2_compile_src_v4 → {M1, T-22, bootstrap, phase1, lens-ci}`
fan-out — at most **2 hops** from witness to verdict.

### 4.2 Cache as nested Upsert<T> — not a "designed hierarchy"

**Frame (§4.0 application):** each cache tier is an `Upsert<T>` whose
`satisfy`-phase upserts the tier below. There is no separately-designed cache
hierarchy; there's one pattern recursed three deep. The names L0/L1/L2 are
identities for the underlying *storage*, not separate cache designs.

Per the operator's constraint *"either Actions Cache or treating srv1/srv2 as
separate caches"*:

| Tier | Identity | Scope | Authority? |
|------|----------|-------|-----------|
| **L0** in-runner ephemeral | per-runner-instance `$RUNNER_TEMP` | one runner's lifetime | **Never** — discarded with runner |
| **L1** per-host logical | `srv1-cache`, `srv2-cache` as logically distinct Actions Cache scopes (or content-addressed sccache + key namespace per host) | one host's lifetime | Yes for host-local reuse |
| **L2** cross-host network | Actions Cache (GitHub-managed) or BuildBuddy CAS (via `ctrl-build --remote`) | global | Yes for cross-host sharing |

**Cache key for every step = `content_hash(complete CiUpsertStep<T> subgraph)`.**
This is IRT-4 applied uniformly — not just to `TestClaim` rows.

**No reliance on filesystem state** between runs. Every step's runner starts cold;
inputs come from L2 (or L1 for host-local repeats) by content hash, not by
"the file was left here last time."

### 4.3 The four-compile redundancy collapses

In elastic shape, the `gunbc compile --target rust src/v4` invocation becomes a
single Node with:
- inputs = `content_hash(src/v4/**)` + `content_hash(target/release/gunbc)` + `target=rust`
- verify = L2 lookup
- create = run the compiler, L2 put
- resolve = the output directory handle (content-addressed)

Then `M1 cargo check`, `T-22 rust receipt walk`, and any future rust-output
consumer each declare `UpstreamUpsert { step_id: <the_emit_step> }` as input —
they receive the resolved output directory without re-running the emit.

Same shape for `--target dag`: one node, three consumers (bootstrap viability,
T-22 dag inspection, future v4-bootstrapped consumers).

**Wall-time effect:** A + B parallel (~7m each), C and D collapsed to L2 hits
(<1s plus python receipt). `ci_v4` work-content drops from 32m to ~10m, ideal
(parallel) wall to ~7m + a bit of receipt parsing.

### 4.4 Compiler-side prerequisites

The cache-tier discipline only delivers wins if `gunbc` exposes content-hash
identity:

1. **`gunbc emit --manifest <path>`** — writes:
   ```json
   {
     "gunbc_binary_hash": "...",
     "source_root_hash": "...",
     "target": "rust" | "dag",
     "output_files": [{"path": "...", "hash": "..."}],
     "elapsed_ms": ...
   }
   ```
   The harness uses this to populate L2; consumers read it to verify cache content.

2. **Parallel module emit.** At ~1.25s/file × 332 files = ~7m today. Parallel
   emission (compiler-internal jobserver participation; honors
   `CARGO_BUILD_JOBS`-style cap declared per step) should ~halve this on the
   srv1/srv2 fleet.

3. **Per-file content-addressed skip.** For the long-tail case (1 of 334 .dag
   files edits): the per-module emit cache key is
   `content_hash(module_inputs + module_dep_closure + gunbc_binary)`. Most modules
   hit cache; only the touched module + its dependents re-emit.

4. **Decoupled output identity.** Today the output dir is path-keyed
   (`$RUNNER_TEMP/v4-rust-emit/...`). Elastic: output dir is
   content-addressed-named (`...-emit-<hash>/`); harness symlinks consumers' input
   paths to the cached output. No filesystem "did the previous job leave it
   here" — purely L1/L2 read.

These three are real compiler engineering (sequencing into Phase 1b or beyond),
not workflow YAML edits.

### 4.5 Compute fabric — workloads declare needs, the network decides

**Frame (§4.0 application):** compute allocation is the `ComputeFabric`
`Upsert<T>` — a pluggable provider abstraction. The workflow harness never
names srv1, srv2, or any other physical fact. It declares the step's resource
*request* (max useful parallelism, min memory, arch, locality hint) and the
fabric returns a slot. Whatever the fabric returns, the step adapts to it
(jobserver scales to slot capacity, not to a pinned core count).

The fabric is the *only* layer that reads `ci_self_hosted_runner_pools` today,
and the *only* layer that would read a hypothetical `fargate_provider`,
`buildbuddy_rbe_provider`, or `k8s_provider` tomorrow. Providers plug into the
same interface — workload-layer code is provider-agnostic.

Today: pool capacity (50 runners) is implicit; jobs race for them under GHA's
runner-selection logic.

Modeled: `SelfHostedRunnerPool { runner_count, jobserver_token_cap }` is data
(`ci.dag:404-418`) but **no consumer reads it** at workflow generation time.

Elastic:
- Each scheduled step → fresh runner instance, pulled from `ci_self_hosted_runner_pools`
- Pool exhaustion = FIFO queue, not capacity-aware backoff tricks
- Per-step parallelism comes from the step's declared `ParallelismShape` (§4.0b),
  not an authored `worker_count` field (forbidden §4.0d). The scheduler derives
  the concrete worker count from `(parallelism_shape × provider_capacity ×
  cache_locality)` at lease time.
- Cross-step parallelism is unlimited up to pool size; over-subscription queues
- Per-host affinity (srv1 vs srv2) is hash-driven for L1 cache locality, not for
  load-balancing — see §4.7

### 4.6 Jobserver redesign — per-runner ephemeral (Upsert<T> over token slots)

**Frame (§4.0 application):** jobserver-token acquisition is one more
`Upsert<T>` recursion below runner allocation. Token-pool identity = the runner's
identity; tokens live exactly as long as the runner.

Today: `MAKEFLAGS=--jobserver-auth=fifo:/var/lib/ctrl/jobserver/host.fifo`. Host-wide
FIFO; the srv2 incident (audit §5.1) demonstrated that a single misconfigured FIFO
silently turns full-tree emit into a 20m hang.

Elastic:
- Each runner instantiates its own jobserver in `$RUNNER_TEMP` (ephemeral, lives
  for the runner's lifetime)
- Token cap is **derived**, not authored: scheduler computes it from
  `(step.ParallelismShape × provider_capacity × cache_locality)` at lease time,
  bounded above by `host_pool.jobserver_token_cap`. No `step.worker_count` field
  (forbidden §4.0d).
- No cross-step, no cross-runner, no host-wide state
- Eliminates the FIFO-race-as-side-channel anti-pattern entirely

`ctrl-build`'s `CARGO_BUILD_JOBS` cap stays — but it's a *floor* (memory safety)
not the *authority* on parallelism (which lives in the modeled `ParallelismShape`).

### 4.7 Locality is provider-private; the workload only sees an opaque hint

L1 locality knowledge belongs to the `ComputeFabric` and its providers — never
to the workload. The workload ships one *opaque* hint with its request:
`locality_hint: Option<PriorRunIdentity>` — "if you ran this same Node before,
where?" The provider interprets it (or doesn't):

| Provider (concrete roster, §4.0) | What `locality_hint` means there |
|---------------------------------|----------------------------------|
| **srv1 / srv2** (owned arm64 Linux pool) | "prefer the host whose L1 produced this output last; rebuild via L2 if unavailable" |
| **Mac mini** (owned single Apple Silicon host) | "this is the only host — L1 is always 'this box' or nothing"; locality hint collapses to "do you have it in your cache?" |
| **WSL** (developer-machine Linux-on-Windows) | "if the human's machine is online and has L1, use it; else L2." Provider's availability is itself dynamic. |
| **ubicloud / gcloud containers** (ephemeral hermetic) | Ignored — every slot is cold-start; L2 is the only cache |
| **BuildBuddy RBE** (remote-execution, via `ctrl-build --remote`) | "warm worker affinity if the CAS scheduler exposes it; otherwise scheduler's choice" |

The workload doesn't switch on provider type. It always sends the same hint;
each provider decides what to do with it. The "intelligent enough not to split
storage across boundaries" property is a provider-side invariant
(`ComputeFabric` returns one slot per step; never partial); not a workload
constraint.

If srv2 dies, the host-pool provider's `verify` misses every srv2-affinity hint
in its `locality_hint` resolution; falls back to srv1 or to L2-cold; workload
sees a slower step, never a broken step. If we add a Fargate provider and
schedule a step there, the same `locality_hint` is sent and silently ignored;
the step runs cold via L2. No workload-layer change. **This is the elasticity
knob: providers expose what they can do with locality; workload sends one hint
and adapts to whatever it gets.**

### 4.8 Cache invalidation discipline

Today: `hashFiles('src/v4/**', '.github/workflows/ci.yml', 'dsl/std/**', ...)` —
broad path globs that bust on irrelevant changes.

Elastic: every step's cache key is **derived** from
`content_hash(complete CiUpsertStep<T> subgraph)`. The subgraph includes:
- inputs (resolved via `UpsertInputRef`)
- verify / create / resolve identity
- tool versions (gunbc binary hash, rustc version, etc.)
- extdeps (declared, not implicit)

A docs-only edit (e.g., `src/v4/TASKS.md`) does not bust the M1 emit cache because
TASKS.md is not in any `CiUpsertStep<T>.inputs` of that step. The "what affects
what" knowledge lives in the modeled inputs, not in YAML `hashFiles()`.

`cache_key` is **never** a payload field (per `ci.dag:163-166` design note +
Practice 11 row 5). Derived, projected — same as `test_claim_interpretation_cache_digest`
for TestClaim rows.

### 4.9 Selection receipt: shadow → active

Today: receipt computed (Phase 1.5 shadow rows landed in W2.3) but does not gate
execution.

Elastic active mode:
1. Workflow harness's first runner: compute `CiSelectionReceipt` for this PR.
2. For each step where `decision = Skip`: emit `Verdict::Pass` from L2 cached
   verdict; no runner allocated.
3. For each `Run` or `CarvedOut`: dispatch a fresh runner; runner reads inputs
   from L1/L2 cache (or computes them from upstream).
4. Pool fanout = parallel-execute over (Run ∪ CarvedOut), respecting `needs`.

This is Phase 2.5 — the operator's "minimal CI per PR" delivery.

### 4.10 Aggregator = modeled Verdict consumer

Today: `ci` aggregator job (`ci.yml:418-455`) is a fail-closed stub that
inspects `needs.*.result` strings.

Elastic: each step's verdict is `Outcome<StepVerdict>` (`Accepted` | `Rejected`).
Aggregator = modeled fold over the verdict list. Branch-protection `ci` check name
is the projection of that fold. No string-comparison shell.

---

## 5. Mapping today → elastic

| Profile bottleneck (§1) | Today | Elastic shape | Required substrate fact |
|---|---|---|---|
| 4× full-tree compile redundancy | Sequential gunbc invocations in one job | 1 emit per (source, target) → L2 → 3 consumers read via `UpstreamUpsert` | `ArtifactRef` + `content_hash` output identity (§4.0a Layer 2) |
| 33m `ci_v4` wall (sequential) | One runner, 26 steps | Each `CiUpsertStep<T>` = own `WorkUnit`; DAG-parallel | `CiStepId → WorkUnit` projection (§4.0a Layer 1→2) |
| v3's 5 overlapping cargo builds | Per-job cargo target/ ; clippy & test rebuild same modules | Shared `ArtifactRef` via cargo-fingerprint content-hashing | `CacheSurface` declared on provider; `ArtifactRef` content-addressing |
| T-22 cache 9-glob fragility | `hashFiles(...)` over `src/v4/**` etc. | `content_hash(whole CiUpsertStep<T>)` — busts only on actual node mutation | Derived cache-key projection (already in `ci.dag:163-166`) |
| Shared `$HOME` race | Per-job env-var workaround | `IsolationContract = HermeticPerWork` in `ComputeDemand` | `IsolationContract` substrate; provider declares `IsolationModel` capability |
| srv2 jobserver FIFO incident | Host-wide FIFO in `/var/lib/ctrl/jobserver/` | Per-runner ephemeral jobserver; provider-private | `ExecutionSurface.process_model` substrate; provider isolates jobserver per-lease |
| 20m / 35m hard timeouts in YAML | Hardcoded `timeout-minutes:` | `ExecutionBudget { expected_cost, provider_model, watchdog }` (§4.0c) | `SymbolicCost` + `ProviderCostModel` substrate; `WatchdogLimit` separate carrier |
| Coarse `if: v4 \|\| testclaim_corpus` bucket gating | Component-level bucket booleans | Per-step `inputs ∩ affected_set` selection (Phase 2.5) | `UpsertInputRef` typed inputs (Phase 1.5 — landing) |
| Static `Symbol` cache tags (`ci_cache_cmd_m1_probe_tag`) | `cache_digest` is a constant symbol | `cache_digest = content_hash(whole CiUpsertStep<T> node)` projection | Dissolve static cache-tag symbols; project from whole-node hash |
| Authored `worker_count: 20` per step | Would-be hardcoded tuning | `ParallelismShape = IndependentShards / DependencyGraphParallel / PartitionedReduce { laws }` | `ParallelismShape` coproduct + `ReducerLaws` (§4.0b) |
| MapReduce / batch fan-out (future) | Not yet attempted | Scheduler fans out when `ReducerLaws.associative` is witnessed; falls back to narrow plan otherwise | `Witness<Associativity>` + `Witness<Idempotency>` substrate |
| `continue-on-error: true` on M1 / phase1 | Modeled `non_blocking: true` but YAML drift risk | Single source: `CiGate.run_policy` / step `non_blocking` projected to YAML | `CiGateRunPolicy` already typed (`ci.dag:153-156`); dissolve YAML drift via Phase 2 Shape-B emission |

---

## 6. Open questions worth surfacing for operator

**Q1 — L1 backend identity.** "srv1/srv2 as logically separate caches" — does the
implementation:
- (a) Use distinct Actions Cache key namespaces (e.g., `srv1-...` / `srv2-...`
  prefixes), with GHA backend providing the storage?
- (b) Run a per-host local CAS (e.g., a `/var/cache/ci/cas/` content-addressed
  store, sccache-style), with the harness picking the host?
- (c) Treat sccache as L1 (already per-host) and Actions Cache as L2?

The operator framing says "logically separate" — argues against treating srv1/srv2
as fungible. (b) gives the most explicit identity; (a) leans on existing GHA
infra; (c) reuses what `ctrl-build` already does.

**Q2 — L2 backend.** Actions Cache (network-bounded, ~5 GB ceiling per
repository, eviction policy not under our control) vs BuildBuddy CAS (already
integrated via `ctrl-build --remote`, more capacity, more control). The
compiler-output cache is going to be larger than typical cargo caches —
each `src/v4` rust emit is ~336 files × small. Not huge per emit, but
multiplied across PRs.

**Q3 — Compiler-side investment.** §4.4 names three real compiler changes
(`--manifest`, parallel emit, per-file skip). These are Phase 1b+ work, not
workflow YAML edits. Sequencing question: do these block elastic CI, or does
elastic CI deliver some wins (DAG parallelism, cache-key tightness) even before
the compiler changes?

Tentative answer: most of §4.1 + §4.2 + §4.7 + §4.8 + §4.9 wins are achievable
*without* compiler changes — they're harness + modeling work. Compiler changes
are the bottom-up throughput multiplier. So elastic CI can ship in stages, with
compiler work as a parallel lane.

**Q4 — `ParallelismShape` × provider capacity interaction.** Given §4.0b's
algebraic parallelism (`SingleWorkItem` / `IndependentShards` /
`DependencyGraphParallel` / `PartitionedReduce { laws }`) and §4.0d's
prohibition on authored `worker_count`, the scheduler derives concrete worker
count from `(parallelism_shape × provider_capacity × cache_locality)`. Open:
when `ParallelismShape = IndependentShards { shard_count: N }` and provider has
M < N free slots, does the scheduler (a) run N sequentially in M, (b) queue
extra shards, (c) split across providers (forbidden — §4.0a/§4.7 one-step =
one-slot), or (d) something else? Modeling DFS Manager owns the substrate
decision per `docs/planning/v4-ci-overhaul-2026-05-30.md` §7.

**Q5 — Determinism as effect.** Per `project_determinism_as_effect` memory: a
non-deterministic step's verdict cannot safely cache. Does each `CiUpsertStep<T>`
declare a deterministic effect, with non-deterministic steps either
(a) excluded from IRT-4 reuse, or (b) routed through a determinism-witness
sub-step? This is orthogonal to elasticity but conflicts at the cache layer.

**Q7 — Provider plurality.** When (not if) we run with multiple `ComputeFabric`
providers — e.g., srv1+srv2 + a Fargate burst-pool for CI bursts — what's the
cross-provider L2 cache discipline? L2 must be the single source of truth all
providers read from. Concretely: if step S runs on Fargate (cold L1), produces
output, puts to L2; next run, srv1 picks up step S, its L1 is empty for this
hash, falls through to L2 → hit. That's the design contract. Question for
operator: is there a preferred L2 backend that all providers should target
(Actions Cache? BuildBuddy CAS? S3?), or does each provider get its own L2
identity?

**Q8 — Locality hint surface.** The opaque `locality_hint: Option<PriorRunIdentity>`
shape works for stateful providers and is ignored by stateless ones. Open: should
this be richer (e.g., the workload also declares its L1 *working-set size* so
stateful providers can decide whether L1 affinity is worth the queue wait)? Or
is the simpler "opaque hint, provider decides" interface load-bearing for the
provider abstraction?

**Q6 — Active gate condition.** Phase 2.5 turns receipt decisions into runner
dispatch. The operator's framing says "absolutely minimal set of functionality
that gives us the highest confidence." That's a structural correctness criterion
(every decision is justified by `inputs ∩ affected_set` evidence), not a
wall-clock criterion. The wall-clock win is downstream. Acceptance for Phase 2.5
should be the structural criterion (per planning doc §5).

---

## 7. Concrete program shape (not a dispatch — operator-bar)

This isn't a single PR. It's a multi-quarter program with three parallel lanes:

**Lane M (modeling).** Land Phase 1.4 → 1.5 → 1b → 2 → 2.5. Owner: Modeling DFS
+ Compiler Spine per planning doc §7. Lands the §4.0a canonical schema
(`WorkUnit<T>`, `ComputeDemand`, `ComputeSupplyFacts`, `ComputeOffer`,
`ComputeLease`, `ExecutionReceipt`, `ParallelismShape` + `ReducerLaws`,
`ExecutionBudget`) plus content-hash-derived `cache_digest`. Replaces every
shell gate with a `CiUpsertStep<T>` Node. Authored `worker_count` / `timeout`
fields are forbidden (§4.0d); parallelism + budget are derived.

**Lane C (compiler).** `gunbc --manifest` output, parallel module emit,
per-file content-addressed skip. Owner: Compiler Spine. Independently
schedulable.

**Lane I (infra / harness).** Per-runner ephemeral jobserver, L1/L2 cache
backend integration (Q1/Q2 answers), shadow→active receipt cutover. Owner:
clever-cat-115's existing-shell-retirement lane + operator infra.

Each lane has independent acceptance; the elastic-CI **emergent property** is
their conjunction.

---

## 8. What this doc does NOT propose

- Not a new policy enum / run-mode coproduct (forbidden per
  `feedback_heuristics_recoverable_to_substrate`).
- Not new shell scripts (per `project_no_new_shell` — operator 2026-05-29:
  every CI step models as `Upsert<T>` Node, not as `bash`).
- Not redesigning T-24 phases (ratified; this doc identifies prerequisites,
  not replacements).
- Not committing to a cache backend (Q1/Q2 are operator-bar).
- Not bridging missing modeling with a "temporary" carrier — every named
  prerequisite (`ExecutionBudget` / `ParallelismShape` + `ReducerLaws` per
  §4.0b/c, content-addressed gunbc output) is a structural fact that needs
  modeling, not a heuristic. Authored `worker_count` / `timeout: Duration`
  fields are explicitly forbidden (§4.0d).

---

## 9. Authoritative anchors

| Artifact | Why it's relevant here |
|----------|----------------------|
| `src/v4/workflow/ci.dag` | Modeled `CiPipeline`, `CiUpsertStep<T>`, `SelfHostedRunnerPool`, gates, carveouts, cache tags |
| `docs/design-ci-dag-overhaul.md` (PR #3886) | Design canvas A0–A14 — the original atom-by-atom plan |
| `docs/design-ci-bankruptcy-rebuild.md` | Tier-0 rebuild B0–B3; ratified D1–D5 operator decisions |
| `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` | Anatomy + Table A redundancy + Table B modeled fix + operator-aligned IRT-1/IRT-4 framing |
| `docs/planning/v4-ci-overhaul-2026-05-30.md` | Phase 1a/1.4/1.5/1b/2/2.5 plan + 6 operator decisions |
| `docs/planning/v4-w2.3-ci-upsert-step-migration-worksheet-2026-05-30.md` | Active W2.3 Upsert-step migration buckets A–E |
| `docs/planning/compiler-spine-ci-selection-receipt-shadow-2026-05-30.md` | Selection receipt shadow → active cutover plan |
| `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md` | UPSERT<T> canon + stray scan |
| `docs/audit/ci-warm-cache-wall-measurement-2026-05-29.md` | Warm-cache baseline (12m31s v3 job) for comparison |
| PR #4074 / run 26702944004 | This profile's data source (the 33m46s `ci_v4` job and per-step timestamps) |

---

*End of exploration. Per the ledger-standing principle (operator 2026-05-19):
this doc is synthesis, not a parallel ledger. If a fact below contradicts the
substrate in `ci.dag`, trust `ci.dag`. Updates: amend this file in place; do not
fork a v2.*
