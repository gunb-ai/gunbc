# Emitted crate partition — derive the crate layout to saturate the CPU

**Status:** design, interface-locked with the language-consolidation lane (eager-ferret-110) 2026-07-16. This is the **Rust realization's workflow-layer half** of a shared, target-agnostic contract; the shared *home* (the `CompilationUnit` type + `partition_fold` kernel + the per-target validity interface) is authored by the language lane, and C is the second realization staffed against the same home. Implementation owner TBD (operator considering placement).
**Relates to:** `#6677` (derived crate *registration* — the inner layer this builds on), DESIGN §2 "one concept, every scale/breadth" Realization, §3 single-authority / import-arrow, §5 fail-closed + no-dual-representation, §7 self-host. Design record + anti-fork constraints: the `compilation-unit-agnostic-concept` session memory.

---

## Decisions (operator, 2026-07-18)

- **Deferred** behind the active self-host critical path (Gate-A emit-coherence + body-emit). Resume when those lanes settle; do **not** dispatch ahead of them. Tracked here + in session memory so it is not forgotten.
- **Cost proxy = LOC, made EXPLICIT in the interface.** The balance term uses lines-of-code as the compile-cost stand-in *for now*, but it must be a **named, typed proxy** in the policy interface (e.g. `CompileCostProxy = LocProxy { … } | …`), never an implicit `LOC == cost` equation buried in the fold. Swapping in a real compile-cost / monomorphization model later is then a typed substitution at one declared point, not a hidden refactor (§3 single-authority, §5 no-fabricated-default).
- **Tests are out of the partition.** The `.dag` test corpus runs on the floor (`claim_executor`), not as emitted Rust `#[test]`, so test modules are not in the frontier roster / module graph and never enter the partition. The even-cost balance is over compiler modules only; there is no test-crate story to design. (The floor still *benefits* — a better-parallelized crate build speeds the witness runs — it just is not *part* of the partitioned input.)

---

## 1. Problem

The self-hosted compiler emits its own Rust as a **crate layout**. Today that layout is a **hand-drawn 2-crate functional split** in `src/v1/stage0_crates.dag` — `stage0_crate_plan()` hand-assigns ~49 modules to `v1_stage0_core` and 7 to `v1_stage0_emit_core`. `stage0_crates.dag` has **zero** references to the module authority (`frontier.dag`); the crate **membership** is a hand ledger.

`#6677` made the frontier the single authority for module **registration** (which modules get a `pub mod` decl, the stage0 file/dir lists) — but *not* the **partition** (which crate a module lands in). So the "single crate-organization authority" is partial: it derives registration *within* a hand-drawn partition. The membership ledger is the last uncovered fork — and it is a §3 hazard for the same reason v1's became "the last uncovered fork": a private ledger with no dissolution path.

The partition is where the cost lives. A Rust crate is the unit of **separate compilation, incremental rebuild, and build parallelism** (and a codegen/monomorphization boundary). A hand-drawn 2-crate split serializes the build on one fat crate — the CI-wall symptom ("one giant serial crate is the straggler"). The compiler spends the only thing we value (time, §1) waiting on a partition never designed for parallelism.

## 2. The concept is agnostic; this doc is one realization of it

A Rust crate = C translation-unit = Go package = .NET assembly = Java jar are §3 **nicknames** for one concept: the target's **unit of separate compilation & linkage**. So the shape must not be minted Rust-only (that would cement crate accidents as universal — the §7 byte-fixpoint-trap analog). The consolidation is done as a **shared home + two realizations staffed together** (Rust here, C in the language lane), so the agnostic shape is factored with *two* axis-sets present, not extrapolated from one.

**The layer split (locked 2026-07-16):**

- **Shared home (language lane owns; import target, never imports upward):**
  - `CompilationUnit` / `CompilationUnitId` — the agnostic unit. Fields: `members` (module-DAG nodes in this unit — **input**), `interface` (exported surface other units may reference — **R5-derived output**), `deps` (units this depends on — **R5-derived output**), `artifact` (an **opaque agnostic identity**; the crate-name/`.o`/package *spelling* is realization-derived, never stored here).
  - `partition_fold(module_dag, Map<ModuleId, CompilationUnitId>) -> List<CompilationUnit>` — the **shared kernel**. Pure; reads which edges cross the partition boundary to compute `interface` + `deps`. It is the **sole producer** of those two fields — there is no hand-set path (a field both derived and independently settable is a dual-representation that fails open, §5 / the F7 lesson).
  - the per-target **validity-predicate interface** (R4), and an agnostic `Provenance = HandExplicit | PolicyDerived { policy: DeclarationRef }` tag (references a policy by `DeclarationRef` only — the policy *type* never enters the shared layer).
- **Rust workflow-layer (this doc owns):**
  - `PartitionParameter = ExplicitPartition { assignment } | PolicyPartition { policy: PartitionPolicy }`,
  - `PartitionPolicy` (the crates≈cores balanced derivation — §3(c) business/derived, workflow-layer),
  - `resolve_partition(PartitionParameter, module_dag) -> Map<ModuleId, CompilationUnitId>` (Explicit returns its assignment as-is; Policy runs the derivation) — **the `Map` is the only thing that crosses into the shared fold**,
  - `R3` Rust **linkage** (cross-unit ref spelling: `pub` + `use`/path; Cargo dep decl; per-crate re-monomorphization),
  - `R4` Rust **validity** (SCCs are atoms; partition the SCC-condensation; a cyclic condensation or SCC-split refuses).

Because only the resolved `Map` crosses the boundary, the shared shape never depends upward on `PartitionPolicy` (the §3 import-arrow inversion is avoided — the same class as F7's compiler→extdeps.rust catch). The C realization (link-cyclic allowed, include-closure TU, `.h`/`.c` split, permissive R4) codes against the *same* `CompilationUnit` + `partition_fold` + R4-interface; only its R3/R4 differ.

## 3. Objective (the Rust partition policy)

Partition the emitted module graph into crates that **saturate the CPU by construction**:

- **Count ≈ cores.** Emit `K ≈ n_cores` crates (interim policy). Cores is a machine fact we already model (the `CARGO_BUILD_JOBS` derivation; the adaptive governor reads cgroup/CPU).
- **Even.** Crates equal in compile-cost. A parallel build is only as fast as its slowest dependent chain, so **balance is the primary term** — an even partition minimizes the max-crate compile time.
- **Flat.** The induced crate-level DAG is shallow (short critical path). Dependent crates serialize, so depth — not just count — gates wall-clock.
- **Acyclic (hard constraint, Rust-specific).** Rust crates cannot form a dependency cycle. This lives in the Rust **R4**, *not* the shared shape (C permits link cycles).

Objective, precisely: **minimize the parallel critical path through the crate DAG**, subject to Rust-acyclicity, targeting `K ≈ n_cores` even-cost crates.

## 4. The lever: we *emit* the crates

This is not generic graph-partitioning of a fixed input. **Because we emit the crate sources, we control the module boundaries themselves** — split a lumpy module, coalesce small ones, choose cut points — co-designing module boundaries *and* partition to hit evenness and flatness. Generic partitioning is stuck with whatever lumpy shape the modules happen to have; we are not. This makes "even" *achievable*, and it is why the interim policy is realistic.

## 5. Constraints (Rust R4)

- **Acyclicity → SCCs are atoms.** Any modules that mutually depend must live in the *same* crate or the crate DAG cycles. So the object to partition is the **strongly-connected-component condensation** of the module dependency graph — acyclic by construction — not the raw module list. `dag/std/graph.dag` provides the SCC/condensation primitives.
- **Monomorphization boundary.** Each crate re-monomorphizes the generics it instantiates; splitting a hot generic across a boundary duplicates codegen. A cost to weigh (keep tightly-coupled generic-heavy modules together), not a hard rule — noted so the §6 cost proxy can eventually price it.
- **Emit-time only.** The partition is a *derivation* computed at emit time from the module graph + core count. Not a runtime concern, not hand-authored.

## 6. Inputs (authorities — no new ledgers)

| Input | Authority (existing) |
|---|---|
| Module roster + dependency edges | `frontier.dag` (the module authority `#6677` established) |
| Core count | the `CARGO_BUILD_JOBS` derivation / adaptive-governor CPU fact |
| Per-atom compile-cost proxy | **interim:** node/LOC count per module; **later:** measured compile time |
| Graph primitives (SCC, condensation) | `dag/std/graph.dag` |
| Unit type + partition fold + R4 interface | the shared home (language lane) |

The partition **replaces** `stage0_crate_plan()`'s hand membership list — it does not sit beside it. The list is deleted and derived, closing the last hand ledger (§3: no parallel representation). The renderers (`emit_stage0_crate_manifest`, `emit_stage0_crate_lib`) that turn a unit into `Cargo.toml` + `lib.rs` are reused unchanged.

## 7. Algorithm sketch (Rust realization)

1. **Resolve the parameter:** `resolve_partition(PartitionParameter, module_dag) -> Map<ModuleId, CompilationUnitId>`. Explicit returns its assignment; Policy runs steps 2–5.
2. **Build the module dependency graph** from the frontier (nodes = self-host modules, edges = imports).
3. **SCC-condense** it → an acyclic DAG of atoms (`std/graph.dag`).
4. **Estimate per-atom cost** (interim proxy: node count; the shape is `Measure`, so swappable for measured compile-time later).
5. **Partition the condensed DAG into `K ≈ n_cores` groups**, minimizing the critical-path objective (§3) under the balance target. This is a balanced acyclic graph partition — NP-hard in general, so a **heuristic** (topological-layer banding + greedy balance, or recursive bisection along min-cut seams with a balance penalty). The heuristic is fine; the *policy* is the fixed part. Output: the `Map`.
6. **Shared fold:** `partition_fold(module_dag, Map) -> List<CompilationUnit>` computes `interface` + `deps` from boundary-crossing edges (shared kernel, not Rust-specific).
7. **Rust R3 render:** each `CompilationUnit` → `Cargo.toml` + `lib.rs` via the existing renderers; `artifact` identity → crate-name spelling here.

**Canonicalization (R1 guard):** `members`/`deps` are set-semantics. If emitted as `List`, the order MUST be a **canonical sort**, never order-as-identity — otherwise two equal partitions with different member order are spuriously distinct, and that would trip the determinism gate (§8) on our *own* ordering rather than a real leak.

## 8. Acceptance = determinism (not a new "partition-invariance" concept)

The crate partition is a **non-semantic perturbation axis**: the crate breakdown has no bearing on what the code *does* — it is not an input to behavior. So acceptance is grounded on the **existing** determinism authority, not a new harness:

- Ground on `v2.std.determinism` / `std.perturbation`. Partition = a new **perturbation axis**; a behavior that depends on the partition = a determinism **leak** carrying a `DeclarationRef` to the root reached. The receipt is the **existing perturbation receipt**.
- **Discriminating witness:** emit a fixed module DAG under ≥2 partitions (1-crate, N-crate, one hand grouping) → all **build** and are **behaviorally equivalent**; an **invalid** partition (forced Rust cycle / SCC-split) → **refuse**, typed + located. Green-by-execution + a discriminating RED (§5/§7), not a structural check.

## 9. Fail-closed (§5)

The derivation must **refuse, never widen**:
- A partition that would induce a **Rust crate cycle** is a typed, located refusal — never silently merged-into-one or emitted broken. The SCC-condensation makes cyclic partitions **unwritable by construction**; the residue (an un-splittable oversized SCC) is a typed refusal.
- An atom whose cost exceeds a single crate's balance budget (an un-splittable SCC bigger than `total/K`) is a **located refusal naming the SCC** — the honest signal that a module boundary needs manual splitting, not a silent lopsided crate.
- The core-count fact being unavailable is a **refusal**, not a fabricated default `K`.

## 10. Staging — Phase 0 (usable now) → Phase 1 (dissolve)

The anti-v1-forgetting move: the interim hardcoded crate list is **not a separate ledger** — it is the R2 parameter with an **explicit value**.

- **Phase 0 (unblocks v2 multi-crate self-host):** implement `resolve_partition` + supply an **`ExplicitPartition`** interim Rust grouping. Land it as a **declared scaffold** — the `HandExplicit` provenance, a typed frontier-retained row with a **dissolution trigger** ("explicit → replaced by derived balanced partition when the policy lands"). Countable, tracked, impossible to forget. This is the "known list we hardcode while we wait to dissolve it" — expressed through the *real* parameter, so it is not a fork.
- **Phase 1 (dissolve):** the derived balanced `PolicyPartition` (§3, crates≈cores/even/flat/acyclic, min parallel critical path) **replaces** the explicit value. Same parameter, derived instead of hand-set; the Phase-0 dissolution trigger fires here. Provenance flips `HandExplicit → PolicyDerived`.

The determinism gate (§8) protects the swap: if v2 self-host behavior ever depended on the crate layout, it reds — so the interim explicit partition is **provably safe** to swap for the derived one.

**Interim `K`:** fixed `K = n_cores` from a **declared target**, not the live host. An adaptive `K` couples the emitted sources to the build machine — a determinism hazard. Lean fixed.

## 11. § alignment

- **§1 (time):** the point — build parallelism is time saved; priced in wall-clock, not elegance.
- **§2 (Realization):** "partition a DAG into time-minimal units" is the *same* kernel behind CI floor batches (`ci_floor_plan`), sccache units, and the affected-set closure. `partition_fold` should be that kernel, one realization being the crate partition.
- **§3 (single authority + import arrow):** derives from the frontier; deletes the hand `stage0_crate_plan()` membership ledger; the shared shape never imports the workflow `PartitionPolicy` (only the resolved `Map` crosses).
- **§5 (fail-closed + no dual representation):** cyclic/oversized partitions refuse; `interface`/`deps` are derived-only (no independently-settable derived field).
- **§7 (self-host):** the compiler decides its own emitted crate structure from its own module graph — the recursion applied to the build itself.

## 12. Gating / sequencing (not blocking the design)

1. **main-green.** main is currently red on a compile-clean std-typecheck break (two `algebra.dag` files: `dag/std/algebra.dag` flat vs `src/v2/std/algebra.dag` grounded; record-lit field authority binds to the flat def; trigger #6686 cross-root import). Fixes in flight: #6705 (the real fix), #6701 (revert #6686), #6696 (flatten literals). Nothing self-host-adjacent goes green until one lands. **Do not scope-creep into the full flat-vs-grounded algebra unification** — that rides the v1→v2 migration; we wait only for the acute break to clear.
2. **`src/v1/stage0_crates.dag` is load-bearing** (DESIGN) — confirm the touch with the operator before cementing (escalation bar). Flag it as "realizes the shared `CompilationUnit` shape (`HandExplicit` provenance, dissolves to `PolicyDerived`)", not a fresh hand ledger.
3. **Band A churn is a non-issue.** The assignment is `Map<ModuleId, CompilationUnitId>`; a module flipping `SelfEmitted` does not move its unit. Phase 0 need not wait for Band A — only for main-green + the sign-off.

## 13. Interface summary (the handoff)

- **Code against (shared home, language lane authors):** `CompilationUnit` + `partition_fold(module_dag, Map) -> List<CompilationUnit>` + the R4 validity interface + `Provenance`. The `partition_fold`/R4 signatures are the contract; the lane flags any adjustment before cementing.
- **Own (this doc / Rust workflow layer):** `PartitionParameter` + `PartitionPolicy` + `resolve_partition -> Map` + R3 Rust linkage + Rust R4 (SCC-atoms/acyclic). The `Map` is the only thing that crosses into the shared fold.
- **Read:** `frontier.dag` (module graph), the `CARGO_BUILD_JOBS`/governor core fact, `dag/std/graph.dag` (SCC/condensation).
- **Do not:** create a structure parallel to `stage0_crate_plan()`; hand-author the partition; hand-set `interface`/`deps`; put a Rust crate name in the shared `artifact` field; embed `PartitionPolicy` in the shared parameter; couple `K` to the live host.
- **Prove by execution (§5/§7/§8):** the emitted `K`-crate structure **builds** and is **behaviorally equivalent under ≥2 partitions** (perturbation receipt), and a forced-invalid partition **refuses** — not just a structural check.
