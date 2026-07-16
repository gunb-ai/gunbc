# v2 memory control — audit (2026-07-14)

**Question (operator):** switching v2 to Realization; memory management should be *perfectly understood and controlled* at that point. Is it?

**Answer: no — and the reason is not a memory bug.** v2 cannot **observe**, **compose**, or **refuse on** space, and the construction mechanism that the whole model leans on (required record fields) is **not enforced by the compiler**. Every claim below is green-by-execution or refuted-by-execution against `da71e41d0`, with a calibration control. v1 is treated only as displaced-cost evidence.

Receipts: the whole hermetic corpus ran green on a Raspberry Pi 3 (`claim_batch --roster-from-discovery`, 554 entry groups, **1837/1837 PASS**) while the process reached **VmSize 6.25 GB / 6.0 GB swapped**, growing ~8.4 MB per entry, never released — 13 GB of swap to finish; stock config OOMs near entry ~350. Log: `.forensics/claim_full.log` (untracked).

---

## F1 — record-literal field presence and types are UNCHECKED (the keystone; scope is corpus-wide)

This is the largest finding and it is **not** scoped to memory. DESIGN §5's primary move is *"make the bad state **unwritable** — correctness by construction, not validation."* In this tree, the main construction device — a required record field — **does not exist in practice**.

Executed against `gunbc compile --target dag` (the CI compile-clean gate's own path):

| probe | expected | actual |
|---|---|---|
| `CacheProvider` — all fields, well-typed | compiles | `0 diagnostics` ✓ |
| `CacheProvider` — **`eviction` field omitted** | refuse | **`0 diagnostics`** ✗ |
| `CacheProvider` — **`eviction: 42`** (Int vs `EvictionClass`) | refuse | **`0 diagnostics`** ✗ |
| `CostAccount` — **3 of 4 fields omitted** | refuse | **`0 diagnostics`** ✗ |
| `DependencyView` — **3 of 4 omitted + `kind: 999`** | refuse | **`0 diagnostics`** ✗ |
| **calibration:** bogus field *name* | refuse | **hard error** ✓ — `field 'bogus_field_xyz' not found in type 'CacheProvider'` |

The calibration proves the typechecker runs on this path. It checks field **names** only — never **presence**, never **types**. (`tier: 42` even survives typecheck and dies at *runtime*: `non-exhaustive pattern match on: 42`.)

**Consequence.** `dag/std/materialization_ladder.dag:28` asserts *"Rule 3 (eviction) is by construction, not by check: `CacheProvider` cannot be written without an `EvictionClass`."* That sentence is **false by execution**. More broadly: **every "by construction, it's a required field" claim in the corpus rests on a property the compiler does not enforce.** This is worse than the §5 tell the operator named ("satisfied by editing the declaration while the realization lies") — here the *declaration itself* is unchecked.

This belongs **on** ROADMAP ④'s path, not after it: Realization's guarantees are declared as record fields.

→ **Dissolve-on:** field-presence + field-type checking in `04_infer`'s record-literal arm, with a discriminating RED per arm (omitted field; wrong-typed field), and a corpus sweep for literals currently relying on the hole.

## F2 — v2 has no effect that can observe space

`dag/std/realization_measurement.dag:25` — the **entire** realization-measurement effect coproduct:

```
type RealizationMeasureEffect
  = ObserveElapsedAtSubject { subject: ContentHash, work_shape: String }
```

One variant. Time. `grep -rniE 'ObservePeak|PeakResident|peak_rss|rusage' dag src/v2` → prose only, zero carriers. The producing seed agrees: `PerformanceReceipt` (`src/v1/stage0/src/v1_interpreter.rs:8336`) is `{subject_key, work_shape, wall_nanos, eval_self_nanos, sample_count}` — **no memory field**.

So **`CostAccount.space` with `basis: Measured` is not producible in v2.** Every space value in the tree is a literal or a passed-in parameter (`fleet_host_budget.dag:66` `space: cfg.mib_per_job`, `basis: Predicted`).

This is why the 6.25 GB was *invisible to the model* rather than merely unfixed. Per §5, a deficit whose frequency is zero by construction never ranks for fixing.

→ **Dissolve-on:** `ObservePeakResidentAtSubject` as a second variant on the existing effect (extends an existing authority — **not** a new one).

## F3 — the space axis is modeled shallower than time, and cannot express retention

`dag/std/realization_schedule.dag:23`:

```
type CostAccount<S> {
  time:  Measure<Time, S, Nat>   // full Measure: basis, scale, unit
  space: ByteSize                // naked scalar
  power: Watt                    // naked scalar — watt(0) at 100% of construction sites
  basis: CostBasis
}
```

DESIGN §2 cites `Cost = Time|Space|Energy → a record (every cost has all three)` as a *worked example of good decomposition*. In the tree only time is decomposed to depth; the record is currently the strongest evidence **for** the claim it falsifies.

A scalar `ByteSize` **structurally cannot express the actual bug**: it cannot distinguish "allocated 8.4 MB and freed it" from "allocated 8.4 MB per entry and never released." CLAUDE.md's own thread says the failure axis is *"retention, not footprint"* — and the type has no way to say *retained*.

Two §3 notes: the carrier says `Power`, DESIGN says `Energy`, and `Energy` is not a `Quantity` at all (`dag/std/measure.dag:6-31`) — the **doc** is the error (energy = power × time; time is already a field). And `power` is a phantom: `watt(count: 0)` everywhere.

→ **Dissolve-on:** a retention/residency axis expressed as the **ladder's existing** eviction concept applied to the space quantity (`ScopeExit`/`SpacePacked`) — **not** a second retention taxonomy (that would be the §3 fork).

## F4 — space has no algebra, and the one rollup applies time's rule (asserted as contract)

`dag/std/realization_measurement.dag:74,86` give Time two composition rules — `time_measure_seq` (**add**) and `time_measure_par` (**max**). **Space has no peer.**

`dag/gunbc/fleet_intent.dag:168` **sums** space across a receipt list:

```
space: byte_size(count: fold(receipts, init: 0,
  f: (acc, receipt) => acc + byte_size_count(receipt.cost.space)))
```

Space is the **dual** of time: sequential steps release (peak = **max**), concurrent steps co-reside (peak = **add**). This applies the parallel rule to the serial axis. `dag/test/claim/realization_measurement_keystone_test.dag:63` asserts the sum **as the contract** — the §5 "enshrined degradation" pattern.

→ **Dissolve-on:** `space_measure_seq` (max) / `space_measure_par` (add) as peers of the time rules; flip the witness.

## F5 — eviction is a write-only label; no reader, no RED

`eviction` appears at 5 sites in the ladder (type `:55`, field `:75`, ctors `:133`, `provider_row` param/passthrough `:146/:148`). **No reader anywhere in the corpus.**

Executed perturbation: **every eviction class deliberately inverted** (CAS persistent → `ScopeExit`; in-process reference/copy/memo → `SpacePacked{"NONSENSE-no-such-budget"}`) → **24/24 witnesses still PASS**, identical to baseline. There is no discriminating RED for eviction. No function relates `tier`/`placement` to `eviction` (the "ScopeExit for scoped, SpacePacked for persistent" coherence rule is **not modeled at all**).

**Blast radius is specific — the ladder is not vaporware.** Its *verdict* logic (plurality → LCA → discharge/refuse/accept) is genuinely inhabited and runs over the real emitted `ci.yml`, with real RED controls (`dag/gunbc/ci_materialization_gate.dag:45,61`; `floor_materialization.dag:89,98` with RED `floor_index_build_materialization_without_provider`). The **eviction dimension specifically** is hollow.

## F6 — `run ≜ realize ∘ materialize ∘ dependency_view`: two of three do not exist

```
fn realize          -> 0 definitions
fn dependency_view  -> 0 definitions
fn run              -> 0 definitions
fn materialize      -> 1   (analysis-only; nothing executes off its verdicts)
```

`src/v2/std/spine.dag` **does not import `v2.std.dependency`** — its `critical_path_depth`/`independence_width` fold the Node *containment tree*, not the `DependencyView`. `DependencyView` (`src/v2/std/dependency.dag:36`) is flat — `{source, dependent, kind, usage_site}` — **no frames, no nesting**. ROADMAP ④'s "ONE *nested* DependencyView" is not inhabited; the tree already says so (`src/v2/compiler/materialization_carriers.dag:41` dissolve-on).

Frames **are** inhabited (`materialization_ladder.dag:30-47`) and an LCA **is** computed (`frame_path_lca:220`, `group_obligation_lca:347`) — but at **outer** scopes only (CI run / shell / process), from hand-supplied `site: List<Frame>`. **The eval frame — where the memory actually lives — has no `Frame` carrier.**

## F7 — F2 blocks `realize`; the two findings are one keystone

`execution-spine-design.md:146` (**FLAG D**) assigns memory-packing to `realize`, and is explicit that memory does **not** become the OS's problem: budget/packing is *"`realize`'s packing over a modeled host-fact set (budget + per-node **measured** peak RSS + placement)"*, `achievable_concurrency = min(independence_width, budget / peak)` (`:97`). §6-1 (`:100`) names the failure by exit code: *"**pack on measured peaks, never guessed** — a memory-blind scheduler that packs by count OOMs (the exit-137 failure mode)."*

**`realize` requires measured peak. F2 says no effect can produce it.** `ObservePeakResidentAtSubject` is therefore the single blocking keystone: without it there is no measured space → no `realize` packing → no spine, and the `memory_governor`'s own dissolve-on (*"graph-derived per-node demand (`CostAccount.space` measured) replaces the reactive estimator"*, `src/v2/workflow/ci_floor_plan.dag:225`) **can never fire**.

## F8 — "memory dissolves into process frames" is half true; the wrong half is the actionable one

- **True:** no GC/arena/bytecode-VM is planned or wanted (`v1-run-stability-throughline.md:174`), and emit-on-demand **is** the declared exit for the interpreter's retention class (`:3`). Frames-as-processes bounds **lifetime** — that is exactly what `ScopeExit` ("eviction = frame exit, derived") names.
- **False:** it does not bound **footprint**, and the design refuses to let it. Already falsified in-tree: the floor's frames **are** processes today (`floor_materialization.dag:29` `claim_executor-process`), and the whole-corpus run still died — the governor packs to `memory.high` *by construction*, was forced to `forced_serial=1` with 474 back-offs, and *"dies by step-cap — **a time death caused by a memory shape**"* (`v1-run-stability-throughline.md:15`).

**Process isolation bounds lifetime, not footprint.** The `min(width, budget/peak)` half of `realize` is precisely the part that does not dissolve — and it is unbuilt.

## F9 — emit-on-demand: transport landed, artifact path absent

ROADMAP.md:37 is unchecked. **(a) agreement**: real and good — `src/v2/compiler/emit_host.dag` does emit→build→run (`run_host_process:245`, `run_test_claim_emit_vs_eval:459`), 15 `emit_host_*_equals_eval` witnesses each with a discriminating RED, CI-gated. **(b) content-hash reuse: does not exist** — `TestClaimCacheKey` (`src/v2/compiler/05_eval.dag:925`) is a key with no store, no content hash, no reuse; `emit_on_demand` appears in **one** plan doc and **zero** carriers. The named first customer (parse census) is absent from code.

## F10 — ORDERING CONSTRAINT: deleting v1 zeroes v2's realized memory control

`claim_executor` imports the v1 tree-walker directly (`src/v1/stage0/src/bin/claim_executor.rs:26`, `make_eval_context:613`). **Every v2 witness, lens, `materialize` and `spine_receipt` included, evaluates on it.** v2 inherits `InterpContext` retention (`v1_interpreter.rs:1169`) wholesale.

The **only** realized eviction in the tree is two hand-written v1 Rust mechanisms the ladder does **not** dispatch:
- `eval_call_memo_frame_exit` (`v1_interpreter.rs:900`), hand-called at 4 sites (`claim_batch.rs:458,638`; `claim_executor.rs:623,691`) — `dag/extdeps/realization/eval_memo.dag:28` self-declares `Scaffold { dissolves_to: RealizationDispatch }`, i.e. *the ladder does not drive me yet*. Its receipt records the regression it fixed: single-witness plateau ~3.4 GiB vs six-witness climb past ~20 GiB to **SIGKILL**.
- `enforce_size_bound` (`src/v1/stage0/src/resolved_graph_cache.rs:346`).

> **On the day the seed is deleted, the tree's *realized* eviction goes to zero while its *declared* eviction stays green — and the ladder will not notice, because it never reads the field.**

Compounding: `duplicate-work-graph-lens-design.md` **cancelled C3** — described verbatim as *"the resolve-store scope fix (first **derived** eviction; the 9 GB receipt)"* — on the grounds that *"the W4 9 GB resolve-store pain… is an ops fix, not a modeling target."* That was the one lane that would have made eviction derived. And `DESIGN.md:91` records eviction as *"(M2, the shelved #5886 projection) … staged behind measured triggers"* — but the trigger is a measurement (F2) that cannot be taken.

---

## Ranked work (each denominated in displaced cost, §6)

| # | Item | Authority | Displaced cost |
|---|---|---|---|
| **1** | **Field presence + type checking on record literals** (F1) | Existing — `04_infer` record-literal arm | Every "by construction" guarantee in the corpus, including all of ④'s. Cheapest known path to making the ladder's own claim true. |
| **2** | **`ObservePeakResidentAtSubject`** (F2) | Existing — extends `RealizationMeasureEffect` | The keystone. Unblocks `CostAccount.space: Measured` → `realize`'s `min(width, budget/peak)` → the governor's dissolve-on. Without it F7 is unbuildable. |
| **3** | **`space_measure_seq`/`_par`** (F4) | Existing — peers of the time rules | Fixes a live bug asserted as contract; any width fold reading the rollup mis-predicts peak. |
| **4** | **Retention axis** — peak vs steady-state (F3) | Existing — the ladder's eviction concept, **not** a new taxonomy | The 9 GB retain-forever store *is* this concept missing. A scalar cannot express it. |
| **5** | **`MemoryBudgetExceeded { observed, budget, subject }`** | Existing pattern — mirrors `EvalBudgetExceeded` (`v1_interpreter.rs:660`) | Today OOM is untyped, uncounted death. Time got its tourniquet (270 min of billed silence → named red). Space still returns exit 137. |
| **6** | **Make the ladder *read* `eviction`** (F5) | Existing | A required field you never read controls nothing. Needs a tier↔eviction coherence rule + a RED. |
| **7** | **v1-deletion ordering** (F10) | Sequencing | Either 1–4 land first, or `memory_governor` migrates to a carrier, or v1 deletion is blocked. |

**Not recommended:** a GC, an arena, or a bytecode VM. The design's rejection of these is correct and should stand.

## Live §5 violation found in passing (unrelated to memory, same audit)

`dag/std/realization_width.dag:108-111` and `:140-144`: budget unreadable → **fabricate a width** (`conservative_fallback_width`). ⊤-as-ignorance answered with a number — untyped, uncounted. Enshrined by two witnesses (`:271` `witness_stamp_unreadable_budget_falls_back_conservative`, `:276`). Should be `WidthUnknown { cause: BudgetUnreadable }` (the established third-arm pattern: `DescentUnknown`, `IdentityUnknown`).
