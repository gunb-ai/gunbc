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
| 20m / 35m / 60m hard timeouts | Encoded in YAML, not modeled — no per-step `timeout: Duration` carrier | T-24 close gap |
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
| Per-step `timeout` / `worker_count` | Not modeled in `CiUpsertStep<T>` yet |
| `gunbc` content-addressed emit | Compiler doesn't expose `--manifest <path>` or `--cache-key` flags |
| Parallel module emit | Compiler is sequential per ~1.25s/file at 332-source closure |

---

## 4. Elastic CI — target shape under "infinite compute, stateless runners"

### 4.0 Framing: one pattern, every layer; one logical computer, locality-aware below

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
- Per-step `worker_count: Int` (new modeled field on `CiUpsertStep<T>` or derived
  from `CiCommand` variant) → how many parallel workers the step's runner runs
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
- Token cap = `min(host_pool.jobserver_token_cap, step.worker_count)` (derived
  from modeled facts)
- No cross-step, no cross-runner, no host-wide state
- Eliminates the FIFO-race-as-side-channel anti-pattern entirely

`ctrl-build`'s `CARGO_BUILD_JOBS` cap stays — but it's a *floor* (memory safety)
not the *authority* on parallelism (which lives in the modeled
`worker_count`).

### 4.7 Locality is provider-private; the workload only sees an opaque hint

L1 locality knowledge belongs to the `ComputeFabric` and its providers — never
to the workload. The workload ships one *opaque* hint with its request:
`locality_hint: Option<PriorRunIdentity>` — "if you ran this same Node before,
where?" The provider interprets it (or doesn't):

| Provider type | What `locality_hint` means there |
|---------------|----------------------------------|
| Stateful host-pool (srv1+srv2 today) | "prefer the host whose L1 produced this output last; rebuild via L2 if unavailable" |
| Stateless container (Fargate, single-use k8s pods) | Ignored — every slot is cold-start; L2 is the only cache |
| Remote-execution CAS (BuildBuddy RBE) | "warm worker affinity if the CAS scheduler exposes it; otherwise scheduler's choice" |
| GHA hosted runners | Ignored — Microsoft owns scheduling |

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

| Profile bottleneck (§1) | Today | Elastic shape |
|---|---|---|
| 4× full-tree compile redundancy | Sequential gunbc invocations in one job | 1 emit per (source, target) → L2 → 3 consumers read via `UpstreamUpsert` |
| 33m `ci_v4` wall (sequential) | One runner, 26 steps | Each `CiUpsertStep<T>` = own runner; DAG-parallel |
| v3's 5 overlapping cargo builds | Per-job cargo target/ ; clippy & test rebuild same modules | L1-cached target/, shared via cargo-fingerprint content-hashing |
| T-22 cache 9-glob fragility | `hashFiles(...)` over `src/v4/**` etc. | `content_hash(whole CiUpsertStep<T>)` — busts only on actual node mutation |
| Shared `$HOME` race | Per-job env-var workaround | Each step = ephemeral runner; never assumes filesystem persistence |
| srv2 jobserver FIFO incident | Host-wide FIFO in `/var/lib/ctrl/jobserver/` | Per-runner ephemeral jobserver in `$RUNNER_TEMP` |
| 20m / 35m hard timeouts in YAML | Hardcoded `timeout-minutes:` | Per-step modeled `timeout: Duration` field |
| Coarse `if: v4 \|\| testclaim_corpus` bucket gating | Component-level bucket booleans | Per-step `inputs ∩ affected_set` selection (Phase 2.5) |
| Static `Symbol` cache tags (`ci_cache_cmd_m1_probe_tag`) | `cache_digest` is a constant symbol | `cache_digest = content_hash(whole CiUpsertStep<T> node)` projection |
| `continue-on-error: true` on M1 / phase1 | Modeled `non_blocking: true` but YAML drift risk | Single source: `CiGate.run_policy` / step `non_blocking` projected to YAML |

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

**Q4 — Per-step `worker_count` modeling.** Add a field on `CiUpsertStep<T>` (or
on `CiCommand` variants)? Modeling DFS Manager owns the substrate decision per
`docs/planning/v4-ci-overhaul-2026-05-30.md` §7.

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
+ Compiler Spine per planning doc §7. Adds per-step `worker_count`, `timeout`,
content-hash-derived `cache_digest`. Replaces every shell gate with a
`CiUpsertStep<T>` Node.

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
  prerequisite (per-step timeout, worker_count, content-addressed gunbc output)
  is a structural fact that needs modeling, not a heuristic.

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
