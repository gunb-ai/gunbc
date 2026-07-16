# Emitted crate partition — derive the crate layout to saturate the CPU

**Status:** design seed (net-new; no prior spec on main as of `744b7e6ccc`). Author handoff artifact — implementation may be owned elsewhere.
**Relates to:** `#6677` (derived crate *registration* — the inner layer this builds on), the DESIGN §2 "one concept, every scale/breadth" Realization pattern, and the session thread on the agnostic *compilation-unit* concept (crate = C translation-unit = Go package; extdeps owns the unit *shape*, workflow owns the *partition policy*).

---

## 1. Problem

The self-hosted compiler emits its own Rust as a **crate layout**. Today that layout is a **hand-drawn 2-crate functional split** in `src/v1/stage0_crates.dag` — `stage0_crate_plan()` hand-assigns ~49 modules to `v1_stage0_core` (CoreCrate) and 7 to `v1_stage0_emit_core` (EmitCoreCrate). `stage0_crates.dag` has **zero** references to the module authority (`frontier.dag`); the membership is a hand ledger.

`#6677` made the frontier the single authority for the **registration** (which modules get a `pub mod` decl, the stage0 file/dir lists) — but *not* the **partition** (which crate a module lands in). So the "single crate-organization authority" is partial: it derives registration *within* a hand-drawn partition.

The partition is where the cost lives. A Rust crate is the unit of **separate compilation, incremental rebuild, and build parallelism** (and a codegen/monomorphization boundary). A hand-drawn 2-crate split means the build serializes on one fat crate — this is the CI-wall symptom ("one giant serial crate is the straggler"). The compiler spends the only thing we value (time, §1) waiting on a partition that was never designed for parallelism.

## 2. Objective (the policy)

Partition the emitted module graph into crates that **saturate the CPU by construction**:

- **Count ≈ cores.** Emit `K ≈ n_cores` crates (interim policy). Cores is the count knob; it is a machine fact we already model (the `CARGO_BUILD_JOBS` derivation; the adaptive governor reads cgroup/CPU).
- **Even.** Crates should be **equal in compile-cost**. The parallel build is only as fast as its slowest dependent chain, so *balance* — not min-cut — is the primary term. An even partition minimizes the max-crate compile time.
- **Flat.** The induced **crate-level DAG should be shallow** (short critical path). Dependent crates serialize (a crate can't compile until its dependency crates finish), so depth, not just count, gates wall-clock.
- **Acyclic (hard constraint).** Rust crates cannot form a dependency cycle. This is not negotiable and it constrains everything below.

Objective, precisely: **minimize the parallel critical path through the crate DAG**, subject to acyclicity, targeting `K ≈ n_cores` even-cost crates.

## 3. The lever: we *emit* the crates

This is not generic graph-partitioning of a fixed input. **Because we emit the crate sources, we control the module boundaries themselves.** We can split a lumpy module, coalesce small ones, and choose cut points — co-designing the module boundaries *and* the partition to hit evenness and flatness. Generic partitioning is stuck with whatever lumpy shape the modules happen to have; we are not. This makes "even" *achievable* rather than aspirational, and it is the reason the interim policy is realistic.

## 4. Constraints

- **Acyclicity → SCCs are atoms.** Any set of modules that mutually depend (directly or through a cycle) must live in the *same* crate, or the crate DAG cycles. So the real object to partition is the **strongly-connected-component condensation** of the module dependency graph — which is acyclic by construction — not the raw module list. `dag/std/graph.dag` already provides graph primitives to build the SCC condensation on.
- **Monomorphization boundary.** Each crate re-monomorphizes the generics it instantiates. Splitting a hot generic across a crate boundary duplicates codegen. A cost to weigh (keep tightly-coupled generic-heavy modules together), not a hard rule — noted so the cost proxy in §5 can eventually price it.
- **Emit-time only.** The partition is a *derivation*, computed at emit time from the module graph + core count. It is not a runtime concern and it is not hand-authored.

## 5. Inputs (authorities — no new ledgers)

| Input | Authority (existing) |
|---|---|
| Module roster + dependency edges | `frontier.dag` (the module authority `#6677` established) |
| Core count | the `CARGO_BUILD_JOBS` derivation / adaptive governor CPU fact |
| Per-atom compile-cost proxy | **interim:** node/LOC count per module; **later:** measured compile time |
| Graph primitives (SCC, condensation) | `dag/std/graph.dag` |

The partition **replaces** `stage0_crate_plan().modules` — it does not sit beside it. `stage0_crates.dag`'s hand membership list is deleted and derived from the frontier, closing the last hand ledger (§3: no parallel representation).

## 6. Algorithm sketch

1. **Build the module dependency graph** from the frontier (nodes = self-host modules, edges = imports).
2. **SCC-condense** it → an acyclic DAG of atoms (`std/graph.dag`).
3. **Estimate per-atom cost** (interim proxy: node count; the shape is `Measure`, so the proxy is swappable for measured compile-time later).
4. **Partition the condensed DAG into `K ≈ n_cores` groups**, minimizing the critical-path objective (§2) under the balance target. This is a balanced acyclic graph partition — NP-hard in general, so a **heuristic** (e.g. topological-layer banding + greedy balance, or recursive bisection along min-cut seams with a balance penalty). The heuristic is fine; the *policy* is the fixed part.
5. **Emit `K` `Stage0CrateSpec` rows** from the partition. `stage0_crates.dag`'s renderers (`emit_stage0_crate_manifest`, `emit_stage0_crate_lib`) already turn a `Stage0CrateSpec` into `Cargo.toml` + `lib.rs`; they are reused unchanged. Only `stage0_crate_plan()` changes — from hand list to derived partition.

## 7. Fail-closed (§5)

The derivation must **refuse, never widen**:
- A partition that would induce a **crate cycle** is a typed, located refusal — never silently merged-into-one or emitted broken.
- An atom whose cost exceeds a single crate's balance budget (an un-splittable SCC bigger than `total/K`) is a **located refusal naming the SCC** — the honest signal that a module boundary needs manual splitting, not a silent lopsided crate.
- The core-count fact being unavailable is a refusal, not a fabricated default `K`.

The bad state (cyclic or wildly-unbalanced partition) should be **unwritable by construction** where possible (the SCC-condensation makes cyclic partitions structurally impossible), and a typed refusal for the residue (un-splittable oversized atom).

## 8. § alignment

- **§1 (time):** the whole point — build parallelism is time saved; the partition is priced in wall-clock, not elegance.
- **§2 (Realization, one kernel N uses):** "partition a DAG into time-minimal units" is the *same* kernel behind CI floor batches (`ci_floor_plan`), sccache units, and the affected-set closure. This design should expose that kernel, not a crate-only one — the crate partition is one realization of it.
- **§3 (single authority):** derives from the frontier; deletes the hand `stage0_crate_plan()` membership ledger (the last parallel representation the `#6677` "single authority" claim didn't cover).
- **§7 (self-host):** the compiler decides its own emitted crate structure from its own module graph — the recursion applied to the build itself.

## 9. Non-goals / staging

- **Full cost-model optimization is later.** Interim proxy = node count; interim policy = even/≈cores/flat/acyclic. Measured compile-time, LTO tradeoffs, and monomorphization-cost pricing are a follow-on once the interim partition is green-by-execution.
- **The cross-language agnostic unit shape** (crate = C TU = Go package = assembly) is deferred to a 2nd active realization (per the session thread + `[[compilation-unit-agnostic-concept]]` memory) — this doc is the **Rust realization's partition policy**, not the agnostic shape. It should be *structured* so the eventual agnostic factoring can lift the policy (extdeps owns the crate *unit shape*; this partition is the *workflow-layer policy*).

## 10. Open questions (for the implementing owner)

1. **Core-count target:** fixed `K = n_cores`, or adaptive to the runner (the governor already varies width)? A fixed `K` is simpler and deterministic (determinism gate); an adaptive `K` couples the emitted sources to the build machine, which is a determinism hazard — **lean fixed `K`, chosen from a declared target, not the live host.**
2. **Compile-cost proxy fidelity:** is node count good enough for evenness, or is a cheap measured proxy needed up front?
3. **Interaction with the Wave-2 self-emit fan-out:** partition (crate membership) is *orthogonal* to self-emit (a module producing its own Rust), so it does not block the fan-out — but re-partitioning while modules flip `SelfEmitted` creates churn. Sequence the partition derivation to land at a quiescent point (e.g. after Band A, before/with Band B/C), or make it robust to membership changing under it.
4. **Monomorphization across boundaries:** does the interim proxy need to penalize splitting generic-heavy coupled modules, or is that a later refinement?

## 11. Interface summary (the handoff)

- **Touch:** `src/v1/stage0_crates.dag` — replace `stage0_crate_plan()` (hand list) with a partition derived from the frontier + core count. Reuse the existing `Stage0CrateSpec` renderers unchanged.
- **Read:** `frontier.dag` (module graph), the `CARGO_BUILD_JOBS`/governor core fact, `dag/std/graph.dag` (SCC/condensation).
- **Do not:** create a structure parallel to `stage0_crate_plan()`; hand-author the partition; couple `K` to the live host.
- **Prove by execution (§5/§7):** the emitted `K`-crate structure **compiles** and the **`regen_verify` self-host fixed-point is green**, and a discriminating input (a forced cyclic or oversized-atom partition) **refuses** — not just a structural check.
