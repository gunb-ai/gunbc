# Where `build_symbol_index_census` is actually paid — and why the reuse lane cannot serve it

Compiler reuse series, third change (`second worker reuses one real v1 front-end result through the
existing store`). This is the finding row: it records what was measured, and the measurement says
that on the only route driven here which asks the candidate question at an arm this PR may serve, an
answer served from the store would cost **more than an answer computed** — 14.2× and 2.14× more, on
the key alone, before the payload moves at all. Standing:
`gunbc.v1.v1_maintenance_standing`, lane `consumers-on-canonical-dependencies` of
`gunbc.plans.demand_engine_program`.

Conditions named, because a measurement without its commit and its image describes nothing:

- probes 1–7: source commit `ebc880f930e458c2fb998e1eb1ce71dc3bc1a4a6` (main, with #17126 merged as
  `16dfccbdf902`) plus this lane's instruments only, built by
  `/opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc` into a private target dir. That
  source was rebuilt more than once across the probe series and the run logs carry a re-hashed digest
  for only one of those images, so those probes name their source, their instrument set and their
  image prefix rather than a digest re-hashed before and after; the per-image list below says exactly
  which image fed which reading, and a reader re-taking them rebuilds that source with the instruments
  below and compares against the counts here.
- host: one machine, one session slice; the two invocations of a pair ran **one at a time**
- nothing was planted at `/tmp/fleet-converge-plan/`

Each image below names the source it was built from, the instruments it carried, which probes it ran
and what it printed — a row that does not name its commit and its image describes an unknown front
end:

- what each image printed, per probe — no row borrows its reading from another image's binary, and
  `a9085c43…` and `fc37c6eb…` are two builds of one source with different instruments, not one
  image remembered two ways:
  - `fc37c6eb8680c5821cdce2656deaaef92a29c8b3c2874dbd8fe671e4cb1a43c5` — the instrumented observer
    build at `ebc880f930e`, whose front door emits an `outcome: "asked"` line before any key or store
    decision, re-hashed after the runs and unchanged — ran **probes 1 through 7** and printed
    **zero** such lines: on those routes the function was never entered. That image carried no
    per-site trace lines, so it is evidence about the front door and the exit codes only.
  - `a9085c43…` — the same source plus the four per-site trace lines, recorded in the run notes by
    this short prefix only — re-ran the probe 2 and probe 3 shapes to separate "not entered" from
    "instrument dead". Each printed one `census-enter` and one `site=infer_graph_typecheck` line
    (`modules=589` on probe 2, `modules=2` on probe 3) with `site=cli_run_reconcile` and
    `site=infer_global_bare_wrapper` silent in the same process: a live trace beside a zero, which is
    what lets that zero be read as "not entered" rather than "trace lost".
  - `49bcf765…` — source `155d4f445ba` (its `src/` tree is `ebc880f930e`'s plus one unrelated
    `src/v2/workflow/local_repo_wet_terminal.dag` deletion) with the four gated trace lines; binary
    `sha256 49bcf7658146cfcdca304195d493daf242d77c74702e51233e9f516b2c84856b`, hashed at build time
    and again before the run, identical — ran probe 8, and is the only image that ever printed at the
    wrapped arm: two `census-enter` lines and two `site=cli_run_reconcile` lines (`modules=15`,
    `modules=78`), zero at the other two sites. The probe 8 `2 / 2` row rests on this image alone.
- the cost numbers came from `eb84380429e` (with `origin/main` `bfb3b50b057` merged in, which contains
  #17191 `cb0ad214441`) with one gated measurement block inside `reconcile_with_typed_cache`; binary
  `sha256 24b7f70531de7a3e0f348c6d4d7b399e2351f4802db7a6be20c0bd32233925e4`, hashed before the run and
  re-hashed after it, identical both times; its replicate, an earlier build of the same source, is
  `sha256 81e66d1cd4189fbff8fdb649440c66db94e61737b1f24b55794a2cb489a48c5b`
- the two arm-attribution lines came from `eb84380429e` with two gated `eprintln!`s and no timing
  block; binary `sha256 6887c269b6f63b76b22ca7f3b7e3a0763b4deee5794ae49c4316d8135a88f5a7`, hashed
  before and after that run, identical
- #17177 (the v1 source-to-chars change that feeds the newline index) is in **neither** tree — no
  `#17177` commit exists in `origin/main` at either reading — so no row here splices readings across
  that change. If it lands, the affected rows are re-taken at the new sha.

## How the counts were taken

Four `eprintln!` traces, each gated on `GUNBC_FRONT_END_RESULT_SERVING=1`, at:

1. the entry of `v1_compiler_infer::build_symbol_index_census`
   (`src/v1/stage0/src/v1_compiler_infer.rs:35050`),
   printing `census-enter modules=<n>`;
2. `cli_run.rs::build_symbol_index_for_reconcile` — the one call site the reuse lane wrapped —
   printing `site=cli_run_reconcile modules=<n>`;
3. the graph-typecheck call inside `v1_compiler_infer::typecheck_scoped`
   (`src/v1/stage0/src/v1_compiler_infer.rs:42382`,
   the emitted mirror of `src/v1/04_infer.dag:23044`), printing `site=infer_graph_typecheck modules=<n>`;
4. the `.global_bare` wrapper `v1_compiler_infer::build_global_bare_census`
   (`src/v1/stage0/src/v1_compiler_infer.rs:37039`, mirror of `src/v1/04_infer.dag:20693`), printing
   `site=infer_global_bare_wrapper modules=<n>`.

Sites 3 and 4 live in a generated mirror ("Generated by v1 compiler — do not edit"); the traces were
measurement only and were reverted with `git checkout` after the runs. No mirror edit is part of any
PR here.

## How to re-take these counts

The traces existed only in this measurement build and were reverted; nothing is committed that emits
them. They can be reproduced from a private build in four steps, at the commit named above:

1. Add one `eprintln!` at the top of `build_symbol_index_census`
   (`src/v1/stage0/src/v1_compiler_infer.rs:35050`), printing `modules.len()`.
2. Add one at each of the three production call sites:
   `src/v1/stage0/src/cli_run.rs:14653` (inside `build_symbol_index_for_reconcile`, just before the
   call), `src/v1/stage0/src/v1_compiler_infer.rs:42382` (inside `typecheck_scoped`, just before the
   call) and `src/v1/stage0/src/v1_compiler_infer.rs:37039` (the `global_bare` wrapper), plus one at
   the reuse lane's own serve decision, if that code is present.
3. Gate every one of them on `std::env::var("GUNBC_FRONT_END_RESULT_SERVING").is_ok()`, so an unarmed
   run stays byte-identical to an unmodified binary. Do not commit the edits: sites 3 and 4 sit in
   generated mirrors of `src/v1/04_infer.dag`, and a hand edit there would be a mirror drift.
4. Build `cargo build --release -p v1-compiler --bin gunbc --target-dir <private>` (release, not
   debug: the debug binary did not finish the fleet entry inside ten minutes), record the binary's
   SHA-256 before and after the run, and drive each probe from a different checkout path with its own
   `GUNBC_WORKER_ID` and a fresh `GUNBC_NATIVE_CACHE_ROOT` under `GUNBC_FRONT_END_RESULT_SERVING=1`.

Two further instruments were used on probe 8 and are worth re-creating if the claim-route reading is
ever challenged. To attribute a census call to an arm, put a gated `eprintln!("[fes-arm] arm=…
entry={entry}")` immediately before the `resolve_workspace_entry` call (`cli_run.rs:20575`) and before
`run_claim_route`'s `resolve_entry_with_index_typed` call (`cli_run.rs:20972`). To get the cost
figures, put one gated measurement block inside `reconcile_with_typed_cache` right after
`build_symbol_index_for_reconcile` returns, timing four things separately with
`std::time::Instant`: the census call; building the `(repository-relative file path, module path,
source bytes)` pool and the sorted roster the merged key wants (`workspace_relative_repo_path` for the
path, the parse binding's `module_path` for the module path); `closure_identity::symbol_census_key`
over that pool and roster; and a `serde_json::to_vec` + `from_slice::<SymbolIndex>` round-trip of the
census value, printing `json_bytes` and whether the restored value is `==` the produced one. The
`SymbolIndex` type already derives `Serialize`, `Deserialize` and `PartialEq`
(`src/v1/stage0/src/v1_compiler_infer_env.rs:356`), so no seed declaration is needed to take this
reading — which is the point: it is a measurement, not a feature.

## Per-site counts

Every probe armed with `GUNBC_FRONT_END_RESULT_SERVING=1`, a fresh `GUNBC_NATIVE_CACHE_ROOT` and its
own `GUNBC_WORKER_ID`. Only the probes whose per-site counts bear on the placement question are
rowed here; the full probe set, with what each one is evidence for, is in the probe table below.

| probe | invocation | exit | `census-enter` | site 2 (`cli_run`) | site 3 (`typecheck_scoped`) | site 4 (`global_bare`) | `[front-end-result]` events |
|---|---|---|---|---|---|---|---|
| 3 | `gunbc compile --source-root target/fers-small/src --entry …/entry.dag --target dag --output-dir …`, 2-module closure | 0 | 1 | 0 | 1 (`modules=2`) | 0 | 0 |
| 2 | `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/fleet/fleet_converge_plan_revision.dag --function verify_checkout_at_plan_revision`, 589 closure files | 1 | 1 | 0 | 1 (`modules=589`) | 0 | 0 |
| 8 | `gunbc test //test/claim/site_gunb_ai_css_witness:witness_decimal_wire_is_the_shortest_decimal --source-root dag --source-root src/v2`, the CLAIM route | 0 | **2** | **2** (`modules=15`, `modules=78`) | 0 | 0 | 0 |

On `gunbc run` and `gunbc compile` the census is paid **exactly once per process, over the whole
graph**, and the caller is the graph typecheck inside the seed's infer stage:

```
gunbc run
  → v1_compiler_compile::compile_to_resolved_with_options
  → v1_compiler_infer::reconcile_scoped   (src/v1/stage0/src/v1_compiler_compile.rs:3567)
  → v1_compiler_infer::typecheck_scoped   (src/v1/04_infer.dag:23044)
  → v1_compiler_infer::build_symbol_index_census
```

## The claim route pays it twice, at the arm this lane wrapped

Probe 8 drove `gunbc test` over one claim. Both census calls in that process arrived at
`v1_compiler.cli_run::build_symbol_index_for_reconcile` — the site the reuse lane wrapped — and
neither arrived at `typecheck_scoped` or at the `.global_bare` wrapper. The chain:

```
gunbc test <claim>
  → v1_compiler.target_invocation_host::run_claim_selection
      (src/v1/stage0/src/target_invocation_host.rs:2499; the call at :2528)
  → v1_compiler.cli_run::run_claim_route                (src/v1/stage0/src/cli_run.rs:20858)
      ├─ resolve_workspace_entry            (cli_run.rs:20564; called at :20930, the call at :20575)
      │    → v1_compiler_entry_resolve::resolve_entry_graph_shared  (cli_run/entry_resolve.rs:1493)
      │    → resolve_entry_with_index                              (…:1820; the call at :1513)
      │    → resolve_entry_with_parse_cache                        (…:2016; the call at :1830)
      │    → resolve_entry_with_parse_cache_typed  (…:2029; the call at :2026, then `…_inner` :2046)
      │    → resolved_graph_from_sources_with_index  (…:2138; the call at :2062)
      │    → resolved_graph_from_sources_with_index_typed (…:2154; the call at :2151)
      │    → reconcile_with_typed_cache     (cli_run.rs:14660; the call at entry_resolve.rs:2278)
      │    → build_symbol_index_for_reconcile (cli_run.rs:14647) … census #1  (modules=15)
      └─ resolve_entry_with_index_typed     (cli_run.rs:20972; defined entry_resolve.rs:1890)
           → resolve_entry_with_parse_cache_typed (the call at :1902) → … the same tail
           → build_symbol_index_for_reconcile (cli_run.rs:14653) … census #2  (modules=78)
```

The first arm reaches the census through `resolve_entry_graph_shared`, the second through
`resolve_entry_with_index_typed`; both land in the same reconcile arm, which is why one process pays
the census twice and why the row cannot be read as one call.

That chain is read from the source; it was also **observed**. A third measurement build — source
`eb84380429e` plus one gated `eprintln!` before the `resolve_workspace_entry` call
(`cli_run.rs:20575`) and one before `run_claim_route`'s `resolve_entry_with_index_typed` call
(`cli_run.rs:20972`), binary `sha256
6887c269b6f63b76b22ca7f3b7e3a0763b4deee5794ae49c4316d8135a88f5a7` hashed before the run and
re-hashed after it, identical — printed exactly these two lines, in this order, and exited 0 in
328.5 s:

```
[fes-arm] arm=resolve_workspace_entry entry=src/v2/workflow/floor_discovery_source_authority.dag
[fes-arm] arm=run_claim_route_with_index_typed entry=dag/test/claim/site_gunb_ai_css_witness.dag
```

One entry per arm, in that order: the 15-module census arrives with the FLOOR-DISCOVERY authority the
claim route resolves first, and the 78-module census with the claim's own entry module. These trace
lines, like the others here, were reverted with `git checkout` and are not part of any PR.

So: `gunbc run` and `gunbc compile` pay the census **once**, inside the seed's typecheck stage;
`gunbc test <claim>` pays it **twice**, both times at the arm this lane wrapped. That is the only
route driven here on which a second worker actually asks the question this lane could answer.

### What a serve would cost on that route

`gunbc.design_argument::demand_before_materialization` admits a provider only when the measured serve
cost is below the recompute cost, so the same instrument timed, around each census call: the census
itself, `v1_compiler.closure_identity::symbol_census_key` over the same inputs (the merged #17191
authority, fed `(repository-relative file path, module path, source bytes)` for the 10,660-file
process-shared pool plus the closure roster), and a `serde_json` `to_vec` + `from_slice` round-trip of
the resulting `SymbolIndex`.

**The stop decision rests on the key alone, and the round-trip is context rather than a bound.** A
warm consumer does not pay an encode: the bytes it reads were encoded by whoever published them. What
a consumer must pay on every hit is the key (it is the address; there is no read before it), the store
read, the digest check and one decode. So the ratio that decides is key-over-census; the `serde_json`
figures are recorded because the payload's size is a real property of this answer — 63 MB of JSON for
78 modules, which is what a carrier would have to move — and because the decode half of a round-trip
is part of hit latency, but neither figure is claimed to be a lower bound on it.

| call | closure | census | key | serde round-trip | payload JSON | restored == produced |
|---|---|---|---|---|---|---|
| #1 | 15 modules | **57.686 ms** | 819.519 ms | 123.652 ms | 7,417,995 B | yes |
| #2 | 78 modules | **383.369 ms** | 820.601 ms | 938.592 ms | 63,393,360 B | yes |

The table is the timed invocation: the whole claim route in **332.4 s** wall, exit 0
(`✓ typecheck gunbc.site.gunb_ai_css done in 14 seconds`;
`1 claim(s): 1 held, 0 did not hold, 0 unobserved`), on the image whose `sha256` is named above
(`24b7f70531de…`), re-hashed after the run and unchanged. One earlier invocation on a first build of
the same source (`81e66d1cd418…`) is the replicate: census 57.599 ms and 382.931 ms, key 828.275 ms
and 822.219 ms, `restored == produced` on both calls. The two figures that decide the finding
reproduce — census and key agree between the images within **1.1%** (census 0.15% and 0.11%, key 1.07%
and 0.20%). The JSON timings are looser, differing by **3.1%** and **4.2%**, which is another reason
they are context here and not the deciding ratio.

The reading:

- **The key alone costs more than the census it would replace**: 819.519 ms against 57.686 ms, a
  **14.2× ratio** on the 15-module call, and 820.601 ms against 383.369 ms, a **2.14× ratio** on the
  78-module call. Nothing on the hit path can be cheaper than this, because the key is the address.
  That is the decision, and it is the wrong side of the 2× bar on both calls.
- Context, not the bound: the `serde_json` round-trip measured 123.7 ms and 938.6 ms, and the
  module-pool build a consumer without a retained pool vector must first perform to feed the key at
  all measured 603 ms and 610 ms. Taken as the shape of a full request-response cycle those sit on
  top of the ratio above; taken as the producer's publish cost they are not the consumer's hit cost.
- The key's cost is **pool-bound, not closure-bound** — 819.5 ms and 820.6 ms for closures of 15 and
  78 modules, because #17191's preimage binds every discovered module's full source bytes (10,660
  files under `--source-root dag --source-root src/v2`) plus the compiler-identity term. That
  provider-sensitivity is what makes the key correct, and it is also what makes it cost more than a
  78-module census.
- The payload is the other half: 63 MB of JSON for a 78-module census. `restored == produced` held on
  both calls (structural `PartialEq` after `from_slice`, not a bytes comparison), so the carrier is
  faithful; it is simply expensive to fill and empty.
- `key16` differed between the two runs of the same source at the same commit
  (`b1988e5f…`/`af37016f…` for one image, `539ef3c1…`/`7728f7f2…` for the rebuilt one), which is the
  key doing its job: it binds the running compiler image through
  `closure_identity::transform_content_digest()`, so a rebuilt compiler cannot be served a stale
  census.

**On this route, serving would cost more than refusing to serve.** That is the
`demand_before_materialization` test, run and failed, not assumed.

## This is #17126's 10.18 seconds

#17126's cold profile attributed 10.18 inclusive sampled CPU seconds to
`v1_stage0_v1_infer::v1_compiler_infer::build_symbol_index_census` and said nothing about which
caller reached it. That figure corresponds to this one whole-graph call — probe 2's single `modules=589` entry —
and it happens **before** the checkout guard reads its receipt: probe 2 printed
`compile.frontend` / `compile.normalize` / `compile.reconcile` / `compile.analyses`, then the
`[pre-entry]` rows, then `◐ started Filesystem.Read /tmp/fleet-converge-plan/plan_receipt.json`, then
the refusal. So #17126's profile and this instrument do not contradict each other; they were never
measuring the same call. #17126's own `observed/checkout-guard-release.json` rows have the same shape
as probe 2 (`closure_*` rows all 0, `resolve_entry_graph_inclusive` ≈ 107–114 s).

## Why the wrapped arm stayed silent on `gunbc run` and `gunbc compile`

The lane wrapped `v1_compiler.cli_run::build_symbol_index_for_reconcile`
(`src/v1/stage0/src/cli_run.rs:14647`). It is reached only from `reconcile_with_typed_cache`
(`src/v1/stage0/src/cli_run.rs:14660`, the call at `:14717`), whose only call site in the crate sits
inside `resolved_graph_from_sources_with_index_typed`
(`src/v1/stage0/src/cli_run/entry_resolve.rs:2278`). So everything that asks the census at that arm
arrives through one of eight entry-resolve authorities, which delegate to one another like this:

| link | position at `eb84380429e` | the call it makes |
|---|---|---|
| `resolve_entry_graph` | `cli_run/entry_resolve.rs:1024` | `resolve_entry_with_index` at `:1041` |
| `resolve_entry_graph_shared` | `…:1493` | `resolve_entry_with_index` at `:1513` |
| `resolve_entry_with_index` | `…:1820` | `resolve_entry_with_parse_cache` at `:1830` |
| `resolve_entry_with_index_typed` | `…:1890` | `resolve_entry_with_parse_cache_typed` at `:1902` |
| `resolve_entry_with_parse_cache` | `…:2016` | `resolve_entry_with_parse_cache_typed` at `:2026` |
| `resolve_entry_with_parse_cache_typed` | `…:2029` | `resolve_entry_with_parse_cache_inner` at `:2043`, which calls `resolved_graph_from_sources_with_index` at `:2062` |
| `resolved_graph_from_sources_with_index` | `…:2138` | `resolved_graph_from_sources_with_index_typed` at `:2151` |
| `resolved_graph_from_sources_with_index_typed` | `…:2154` | `reconcile_with_typed_cache` at `:2278`; it recurses to itself at `:2352` |

An earlier version of this document listed 17 such sites and called them "eleven production arms and
six `#[cfg(test)]` arms". That scan looked at two names — `resolve_entry_graph` and
`resolve_entry_graph_shared` — because those were the two the first instrument sat under. The other
six authorities were never scanned, which is exactly how the claim route (which enters at
`resolve_entry_with_index_typed`, and so never touches either scanned name) could contradict the
table. Corrected: a scan of all eight names over every `.rs` under `src/v1/stage0/src` at
`eb84380429e`, with comments and string literals blanked first so a mention in prose does not count
as a call, and `#[cfg(test)]` decided from the enclosing item's attributes, finds **64 call sites:
30 outside `entry_resolve.rs`, the 7 delegations in the table above, and 27 inside `#[cfg(test)]`
modules**.

The 30 arms outside `entry_resolve.rs`, each named by its enclosing function, with the position of
that enclosing function's own production call site:

| arm (file:line) | enclosing function | the authority it calls | reached from |
|---|---|---|---|
| `src/v1/stage0/src/cli_run.rs:5345` | `call_compile_clean_bool_list_fn` (`:5339`) | `resolve_entry_graph_shared` | `cli_run/compile_clean.rs:181`, `:191`, `:202` |
| `src/v1/stage0/src/cli_run.rs:11420` | `build_live_read_selection_manifest` (`:11414`) | `resolve_entry_with_index` | `cli_run.rs:11550` |
| `src/v1/stage0/src/cli_run.rs:15191` | `render_batch_summary_line` (`:15174`) | `resolve_entry_graph_shared` | no call site anywhere in the repo |
| `src/v1/stage0/src/cli_run.rs:16079` | `install_output_policy` (`:16077`) | `resolve_entry_graph_shared` | no call site; its `install_output_policy_in` sibling is called at `:16092` |
| `src/v1/stage0/src/cli_run.rs:16311` | `install_group_syntax` (`:16305`) | `resolve_entry_graph_shared` | no call site anywhere in the repo |
| `src/v1/stage0/src/cli_run.rs:17675` | `fn resolve` of `impl ScopedRunObservation` (`:17658`) | `resolve_entry_graph_shared` | no construction of `ScopedRunObservation` outside `:17645`/`:17658` |
| `src/v1/stage0/src/cli_run.rs:19291` | `project_witness_cost_receipt` (`:19267`) | `resolve_entry_graph_shared` | no call site in production; the string at `:19528` names the `.dag` routine this host evaluates, and the Rust fn is imported only by the test list at `:23250` |
| `src/v1/stage0/src/cli_run.rs:20575` | `resolve_workspace_entry` (`:20564`) | `resolve_entry_graph_shared` | `cli_run.rs:20930`, inside `run_claim_route` — **one of probe 8's two arms** |
| `src/v1/stage0/src/cli_run.rs:20972` | `run_claim_route` (`:20858`) | `resolve_entry_with_index_typed` | `target_invocation_host.rs:2528`, inside `run_claim_selection` (`:2499`), called at `:3918` — **probe 8's other arm** |
| `src/v1/stage0/src/cli_run.rs:25042` | `ci_floor_commit_witness_claim_pairs` (`:25039`) | `resolve_entry_with_index` | `cli_run/witness_gates.rs:753` |
| `src/v1/stage0/src/cli_run.rs:25089` | `roster_entry_registry_cache` (`:25078`) | `resolve_entry_with_index` | `cli_run/witness_gates.rs:694` |
| `src/v1/stage0/src/cli_run/checker_dependency.rs:291` | `evaluate_application` (`:272`) | `resolve_entry_graph_shared` | `checker_dependency.rs:418`, and `:264` through `checker_subject_application` (`:260`) |
| `src/v1/stage0/src/cli_run/emit_host.rs:1156` | `compile_dag_candidate_resolved_call_edges_uncached` (`:1033`) | `resolve_entry_graph_shared` | `emit_host.rs:837` |
| `src/v1/stage0/src/cli_run/emit_host.rs:1167` | same function, the resolved-call-edge arm | `resolve_entry_with_index` | `emit_host.rs:837` |
| `src/v1/stage0/src/cli_run/reach_base_standings.rs:44` | `reach_planned_for_event` (`:30`) | `resolve_entry_graph_shared` | no call site; the `.dag` routine of this name (`src/v2/workflow/floor_subject_seed.dag:508`, called from `src/v2/test/claim/workflow/floor_subject_seed.dag:411-414`) is the routine this host function *evaluates*, not a caller of it |
| `src/v1/stage0/src/cli_run/reach_base_standings.rs:165` | `reach_base_standings` (`:125`) | `resolve_entry_with_index` | no call site; the module is `pub mod` (`cli_run.rs:98`) and these two hosts are named as seed growth (`dag/gunbc/floor_arm_set_consumer_seed_growth.dag:88`) |
| `src/v1/stage0/src/cli_run/required_floor_runner.rs:217` | `floor_git_diff_range` (`:213`) | `resolve_entry_graph_shared` | `required_regen_host.rs:6972`, `:13630` |
| `src/v1/stage0/src/cli_run/required_floor_runner.rs:285` | `floor_diff_comparison_readout` (`:281`) | `resolve_entry_graph_shared` | `cli_run.rs:20098` |
| `src/v1/stage0/src/cli_run/required_floor_runner.rs:364` | `floor_diff_baseline_readout` (`:360`) | `resolve_entry_graph_shared` | `cli_run.rs:26575`, `:26582` — both inside `#[cfg(test)]`, so no CLI route reaches it |
| `src/v1/stage0/src/cli_run/required_floor_runner.rs:406` | `floor_git_diff_name_status_range` (`:402`) | `resolve_entry_graph_shared` | `required_regen_host.rs:6965` |
| `src/v1/stage0/src/cli_run/test_module_hygiene_bridge.rs:81` | `resolve_hygiene_ctx` (`:79`) | `resolve_entry_graph_shared` | `test_module_hygiene_bridge.rs:190`, `:139`, `:100` |
| `src/v1/stage0/src/generated_artifact_boundary_host.rs:64` | `run_docs_projection_agreement` (`:63`) | `resolve_entry_graph_shared` | no Rust call site; re-exported at `cli_run.rs:32515` and named in `dag/gunbc/generated_artifact_boundary_seed_growth.dag` |
| `src/v1/stage0/src/generated_artifact_boundary_host.rs:130` | `generated_artifact_ctx` (`:118`) | `resolve_entry_graph_shared` | `generated_artifact_boundary_host.rs:365`, inside `run_generated_artifact_boundary` (`:361`), itself only re-exported (`cli_run.rs:32516`) with no in-tree call site |
| `src/v1/stage0/src/namespace_baseline.rs:2214` | `evaluate_owned_item_in` (`:2191`) | `resolve_entry_with_index` | `namespace_baseline.rs:2467`, `:2183` |
| `src/v1/stage0/src/namespace_baseline.rs:2338` | `closure_paths_of` (`:2331`) | `resolve_entry_with_index` | `namespace_baseline.rs:2294`, `:2390` |
| `src/v1/stage0/src/required_regen_host.rs:13074` | `regen_generation_role_population` (`:13058`) | `resolve_entry_with_index` | `required_regen_host.rs:8551` |
| `src/v1/stage0/src/required_regen_host.rs:13275` | `render_affected_set_bound` (`:13262`) | `resolve_entry_with_index` | `required_regen_host.rs:13459` |
| `src/v1/stage0/src/required_regen_host.rs:13494` | `render_scope_selection` (`:13486`) | `resolve_entry_with_index` | `required_regen_host.rs:13818` |
| `src/v1/stage0/src/required_regen_host.rs:13576` | `regen_scope_worktree_admitted` (`:13561`) | `resolve_entry_with_index` | `required_regen_host.rs:13781` |
| `src/v1/stage0/src/target_invocation_host.rs:2528` | `run_claim_selection` (`:2499`) | `run_claim_route` | `target_invocation_host.rs:3918` |

The 27 `#[cfg(test)]` sites are not listed row by row: `cli_run.rs` 16 (`:5918`, `:9219`–`:9601`,
`:11622`, `:21930`, `:29100`, `:29101`), `cli_run/demand_acquire.rs` 2 (`:924`, `:1049`),
`cli_run/entry_resolve.rs` 2 (`:3954`, `:3960`), `cli_run/http_ingress_host.rs` 1 (`:1165`),
`cli_run/reach_base_standings.rs` 1 (`:332`), `required_regen_host.rs` 2 (`:15674`, `:16238`),
`target_invocation_host.rs` 2 (`:4882`, `:6110`), `v1_interpreter.rs` 1 (`:28419`). They compile into
the test binary only, so no `gunbc` invocation can reach them.

Three things fall out of the corrected table that the 17-row version hid. First, the claim route's
two arms are not symmetric with respect to the old scan. `resolve_workspace_entry`
(`cli_run.rs:20575`) delegates to `resolve_entry_graph_shared` — one of the two names the old scan
*did* look at, so this arm was in its scope and simply never made it into the list that was written
down; the verdict "no route entered any of them" was reached without asking what calls each arm's
enclosing function. Probe 8's `modules=15` census arrives through exactly that arm. The genuinely invisible arm is the other one: `run_claim_route` calls
`resolve_entry_with_index_typed` (`cli_run.rs:20972`), a name outside the two the old scan looked at,
and that is where probe 8's `modules=78` census arrives. Second, of the 30 arms, **21 have a production call site**, **1 is reachable only from
`#[cfg(test)]` code** (`floor_diff_baseline_readout`), and **8 have no Rust call site at all** — they
are reached, if at all, through the seed's name-keyed dispatch or not at all. A route that never
enters one of the 21 cannot pay this census, which is the whole of probes 1–4. Third, the claim route
contributes **one** chain, not two independent entries, and `target_invocation_host.rs:2528` is
counted once: `target_invocation_host.rs:3918` calls `run_claim_selection`, whose call at `:2528` is
itself one of the 30 arms (its authority is `run_claim_route`), and `run_claim_route`'s own arm at
`cli_run.rs:20972` (its authority is `resolve_entry_with_index_typed`) is the second. The `:2528`
position appears in two rows because it is simultaneously an arm and the recorded caller of the arm
below it; the call-site count behind the 30 is 30 distinct positions, and the two claim-route rows are
two of them.

The all-cache-hits shortcut is not the explanation: `typed_module_cache` is written only by
`index_insert_typed` (`src/v1/stage0/src/cli_run.rs:12495`) from the dispatch loop that runs *after*
the census, so a first reconcile of a closure always misses and always asks. And the zeroes are
readable, per image as named above: on the trace-bearing build, probes 2 and 3 printed one census
entry each at `typecheck_scoped` (`modules=589`, `modules=2`) while the wrapped-arm and `.global_bare`
lines stayed silent in those same processes — the trace was live and the function was simply not
entered — and the image that printed two entries at the wrapped arm is a different build
(`49bcf765…`, probe 8). (The front-door `outcome: "asked"` event the reuse lane's serve instrument emits belongs
to the serving code this finding record does not carry; see the PR body.)

The probes, each armed with `GUNBC_FRONT_END_RESULT_SERVING=1`, a fresh store and worker id, driven
one at a time. Each column is fed by one named image: the `exit` and `[front-end-result]` columns for
probes 1–7 come from `fc37c6eb…`; the four per-site columns for probes 2 and 3 come from `a9085c43…`,
which re-ran those two shapes; probe 8's per-site columns come from `49bcf765…` and its timings and
wall figure from `24b7f705…`. Every one is named with its own `sha256` above:

| # | invocation | exit | events | evidence? |
|---|---|---|---|---|
| 1 | `gunbc run`, 2-module closure (`reuse.helper` / `reuse.entry`), `resolve_entry_graph_inclusive` 14 ms | 0 | 0 | yes |
| 2 | `gunbc run`, whole-pool fleet entry `verify_checkout_at_plan_revision`, 589 files, `resolve_entry_graph_inclusive` 107.6 s, `compile.reconcile` 12 s | 1 | 0 | yes |
| 3 | `gunbc compile --target dag`, 2-module closure (`dag-artifact.json` emitted) | 0 | 0 | yes |
| 4 | `gunbc compile --target rust`, 2-module closure (6 files emitted) | 0 | 0 | yes |
| 5 | `gunbc compile --target dag`, whole-pool fleet entry, ended by a 400 s own-timeout inside the compile transaction after `compile.frontend` 37 s | — | 0 so far | no — not run to completion |
| 6 | `gunbc test //gunbc/instruments:heads-reading-differential`, ended by a 260 s own-timeout after the 3 header lines | — | 0 so far | **no — never reached the claim step**, so it says nothing about it |
| 7 | `gunbc test //dag/test/claim:fleet_converge_plan_revision_witness`, printed only the 3 header lines and exited 0 | 0 | 0 | **no — the claim was never executed**, so its zero is not a measurement of the claim route |
| 8 | `gunbc test //test/claim/site_gunb_ai_css_witness:witness_decimal_wire_is_the_shortest_decimal`, the claim route, 332.4 s wall | 0 | 2 census calls at the wrapped arm | yes |

Probes 1–4 really drive their front end — 1 and 2 through the typecheck, 3 and 4 through an emit —
and none of them enters the function the lane wrapped. Probes 6 and 7 are withdrawn as evidence: 6 was ended by its own timeout before the
claim step and 7 exited 0 without executing the claim at all, so neither one ever reached the place
their zero would be about. Probe 8 is the shape they were meant to be, and it answered: 2 census calls,
both at the wrapped arm.

## The checkout guard arm, verbatim

Same image, checkout root, one invocation at a time, otherwise-empty environment, fresh store each.
This host publishes no fleet plan receipt and nothing was planted at `/tmp/fleet-converge-plan/`.

| arm | env | exit | guard refusal, verbatim | stdout |
|---|---|---|---|---|
| unarmed | serving variable absent | `1` | `fleet_converge_plan_revision: PlanRunUnreadable — plan receipt could not be read at /tmp/fleet-converge-plan/plan_receipt.json: No such file or directory (os error 2)` | 0 bytes, sha256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| armed | `GUNBC_FRONT_END_RESULT_SERVING=1`, fresh `GUNBC_NATIVE_CACHE_ROOT`, `GUNBC_WORKER_ID=worker-A-bright-pike-763` | `1` | identical, byte for byte | identical |

Both arms: 30 stderr lines, no entry and no receipt written under the store root. The guard sits on a
route reuse does not touch, so this shows non-interference. The guard's own refusal is held by the
committed control `checkout_that_landed_on_the_event_sha_refuses` in
`//dag/test/claim:fleet_converge_plan_revision_witness` (`admit_checkout_head(expected, observed)` on
a mismatched pair must yield the located `CheckoutHeadMismatch` naming both revisions), and #17126's
four-case release table (`observed/checkout-guard-release.json`, 587 probes per case, cold and warm
refusals naming the same expected and observed revisions) stays the live process-level evidence.
Nothing here changes either.

## The finding, in one sentence

The candidate question is real, its producer call is real, and on the one route driven here that asks
it at the arm this lane could serve — `gunbc test <claim>`, twice per process, 57.7 ms and 383.4 ms —
**answering it from the store would cost more than computing it**: the merged key alone, which every
hit must compute before it can even address the store, is 819.5 ms and 820.6 ms, **14.2× and 2.14× the
census it would replace** (it binds the full bytes of all 10,660 pool modules, so its cost is
pool-bound and not closure-bound); the `serde_json` round-trip measured a further 123.7 ms and
938.6 ms, and a consumer without a retained pool vector must first spend 603 ms and 610 ms building
the pool the key reads. On `gunbc run` and `gunbc compile` the census is paid once per process,
and it is paid somewhere else entirely — inside the seed's graph typecheck
(`v1_compiler_infer::typecheck_scoped`, the emitted mirror of `src/v1/04_infer.dag:23044`) — a
load-bearing seed stage this lane may not serve from, and the 10.18 s #17126 attributed to the census
is that call, made before the checkout guard ever reads its receipt. So the item's own escape clause
applies: **this answer should stay fresh**, by the `demand_before_materialization` test measured and
not assumed, and this PR carries no serving code, no entry-kind row, no seed-growth row and no store
carrier. The checkout guard's refusal is unchanged by anything here (its arm is above, verbatim, armed
and unarmed), and the probes that finished without reaching the wrapped arm — 1 through 4 — say only
that those routes never enter that function, not that no route can.
