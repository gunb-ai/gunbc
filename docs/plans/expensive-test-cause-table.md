# Expensive-test cause table (ROADMAP §1 — root-cause before any lane)

*Owner: proud-deer-709 under §1 / quick-ant-298. Operator directive (2026-06-21, via
bright-stag): a nightly lane that merely schedules expensive tests away **accepts the expense
as given** = defers cost into the future = the §6 local-patch trap. Root-cause each expensive
test FIRST — most causes are **defects**, not intrinsic cost. The nightly lane is the fallback
for the irreducible remainder only, built AFTER this table.*

## Method (attribution, not measurement)

The **bucket is a property of mechanism** — what the test physically does — read directly from
the test source (subprocess/cargo/network/disk vs in-process `compile_dag_named`/`compile_multi`).
That is more reliable than timing; the parent's user-vs-wall heuristic (wall≫user ⇒
IO/subprocess ⇒ (a); user≈wall ⇒ real seed compute ⇒ (c)) **corroborates** it. Per-test
cost(s) reuse fierce-hawk-540's clean **single-threaded** run (user-CPU ≈ wall; proof:
`a4_opacity` isolated = 108s wall = 108s user). Parallel runs inflate user-CPU ~6× via
memory-bandwidth contention and are ignored. No re-measurement by me.

Buckets (operator's):
- **(a) NON-HERMETIC** — real IO/network/subprocess/disk-rebuild. A hermeticity bug to FIX;
  once hermetic it is not expensive and should not be `#[ignore=expensive]` at all.
- **(b) REDUNDANT RECOMPUTE** — recomputes a cacheable input with no memo (§2). Fix = memoize.
- **(c) V1-SEED STRUCTURAL** — the cost IS the slow hand-Rust seed compiler running in-process
  (tens of CPU-sec even for trivial input). Structural; the self-host shrink's job;
  interim-periodic only, dissolve-on tied to seed-shrink.
- **(d) GENUINELY LARGE input** — the only bucket where "periodic" is the honest permanent
  answer.

## Headline finding — two disjoint populations, two causes

There are **two separate expensive populations**, and conflating them was the trap:

- **Population A — fierce-hawk's measured cost≥3s set (currently GREEN, ~29 tests, 28–118s
  each).** These are the tests #5427's gate-widening will newly `#[ignore="expensive"]`. **All
  (c) v1-seed-structural** — every one is an in-process full-pipeline test
  (`compile_multi`/`compile_dag_named`/typecheck/interpreter over real or inline `.dag`), zero
  subprocess. This is the cost-dominant population and the known seed-perf root.
- **Population B — the already-`#[ignore]`'d subprocess/network set (~16 tests).** **Mostly
  (a)/(b)** — `cargo build`/`cargo check` subprocesses, disk rebuilds, two live-network tests.
  These are *fixable defects*, not intrinsic cost.

So the operator's instinct holds both ways: the big cost is (c) (seed compiler, Population A),
and Population B is largely (a)/(b) that should drain back to the per-PR gate, leaving a small
nightly residue.

### Bimodal evidence + the isolation-vs-marginal caveat (Population A)

fierce-hawk's single-threaded set has **nothing between 1s and 28s** — tests are either ~instant
(pure unit) or ≥28s (whole-pipeline). A 2-line `module clean\ntype Widget {…}` snippet
(`compile_multi`, **no imports**, so the transitive closure is just itself) still costs **69s**.
The cost is therefore a **fixed floor, not input-proportional**. Two clusters: ~28–39s
(`cross_representation_equality`, `coproduct_reflection` — which never call `compile_multi`) and
~66–118s (the `compile_multi`/`diagnostics` tests). Two constants stacked.

**Critical caveat — these are ISOLATION numbers, not marginal suite cost.** fierce-hawk measured
each test **run alone** (`a4_opacity isolated = 108s`). But:
- `module_index()` (`helpers.rs:154`) is `MODULE_INDEX.get_or_init(build_module_index)` — a
  **once-per-process** static; `build_module_index` scans the whole `dsl/`+`src/v2` tree and
  parses every `.dag`'s module declaration. In an isolated single-test process **each test pays
  this once**; in a real `cargo test` run (all tests share one process) it is paid **once total**.
- So the per-test isolation time **double-counts** every once-per-process cost (the module-index
  whole-tree parse, lazy statics, binary load). The **marginal** per-PR cost of adding these 29
  tests to the suite is plausibly **far below** the 29×(28–118s) isolation sum.

This bifurcates the root cause and is the pivotal verification:
- **If the floor is once-per-process setup** (module-index parse / lazy statics): the real per-PR
  impact is small → these tests may **not need `#[ignore=expensive]` at all** (the "expensive"
  reading is a measurement artifact of isolation). That dissolves the lane question for Pop A.
- **If the floor is genuine per-call `compile_sources` cost** (the seed compiler is slow even on
  2 lines): it is real (c) per-test work that sums in the suite → interim-periodic, dissolve-on
  self-host.
- A **suite-level (b)** (a cacheable closure recompiled per `compile_multi` call) sits between:
  amortizable by a shared compile-fixture / memoized resolve (§2, the dormant resolve-cache).

**The discriminating experiment** (parent's call — it is measurement, which the directive
reserves): compare *single-test-isolation* time vs *full-suite-marginal* time (suite-with-test
minus suite-without). Cheap, decisive, and host-load-tolerant if run carefully. NOT run here.

## Verify experiment — Pop-A floor decomposition (measured, this branch)

Parent-authorized discriminating measurement (warm build, `--test-threads=1`, **box under load
~2× vs fierce-hawk's clean run** — use ratios, not absolutes):

| run | tests | wall | user |
|---|---|--:|--:|
| A | `clean_compile_produces_zero_diagnostics` (×1) | 138.10s | 138.18s |
| B | + `unresolved_type_in_field` + `bare_container_type_detected` (×3) | 234.96s | 233.98s |

Decomposition: marginal per added test = (234.96 − 138.10)/2 = **~48s/test** (loaded; ≈24s
deloaded). Implied once-floor = 138.10 − 48 = **~90s** (loaded; ≈45s deloaded). **user ≈ wall**
throughout (confirms fierce-hawk's single-threaded property). So Pop-A is **TWO costs stacked**:

1. **A ~90s once-per-process floor** = the test-helper `module_index` OnceLock (whole-`dsl/`+
   `src/v2` `parse_source`). Paid by the first `compile_multi` in a process → each *isolated*
   test pays it (inflating fierce-hawk's per-test numbers), but a real `cargo test` suite
   amortizes it **once**. This part *is* an isolation artifact.
2. **A ~48s/test (≈24s deloaded) genuine per-test residue** that **sums in a real suite**. The
   three snippets are 2–3 lines; two import **nothing**, one imports `std.types`, yet the
   marginal was **uniform** → the residue is **import-invariant**, i.e. a *fixed per-
   `compile_sources` call cost*, NOT a memoizable import-closure recompute.

**Conclusions:**
- **The pure-isolation-artifact hypothesis is REFUTED.** The floor is an artifact, but the
  ~24s-deloaded per-test residue is real: ~21 compile tests ⇒ ~45s floor + ~21×24s ≈ **~9min
  real per-PR suite cost**. So Pop-A genuinely merits cadence-decoupling — #5427's
  `ignore=expensive` on Pop-A is justified, and the nightly-residue from Pop-A is **non-zero**.
- **Resolve-path annotation (parent's ask):** the residue is on the **harness-other in-process
  path** — `compile_multi → compile_sources → front_end_sources → resolve_modules` — which is
  **cache-blind**: it routes through **neither** discovery (`cli_run.rs:~547`, cache-consulting)
  **nor** SingleClaim (`cli_run.rs:~505`). Confirmed independently by keen-otter-380 (the only
  test touching `GUNBC_RESOLVED_GRAPH_CACHE_DIR` is `resolve_cross_process_cache_test.rs`).
- **§2 verdict for Pop-A: the disk resolve-cache does NOT drain it** — wrong path (in-process,
  cache-blind) AND wrong cost (the residue is a fixed per-call seed cost, import-invariant, not
  an import-resolve recompute). So **neither** the zero-code cache-dir **nor** stern-otter's
  SingleClaim routing helps Pop-A. §2's lever lands on the *CLI/floor* resolve, not these tests.
- **Two cheap wins surfaced anyway:** (i) the ~45–90s `module_index` floor is a **test-helper
  inefficiency** — it `parse_source`s every `.dag` just to read module *names*; a light
  module-decl scan (or sharing the compiler's index) cuts it. Test-infra fix, not §2.
  (ii) Pop-B build-once (queued).

### Handoff: Pop-A part-(a) fix (module_index light-scan) — for the test-infra fix-worker

*Diagnosis is mine (proud-deer-709); the FIX is shed to a separate test-infra worker (parent
2026-06-21) so it doesn't block the two gating measurements. Notes for the picker-up:*
- **Location:** `src/v1/tests/src/helpers.rs` — `MODULE_INDEX: OnceLock` (`:152`), built by
  `build_module_index()` (`:124`) → `scan_dag_files` (`:128`) → `extract_module_declaration`
  (`:146`), which calls **`parse_source(&content)`** on **every** `.dag` under the source roots
  just to read the `module x.y` name.
- **Fix:** replace the full `parse_source` in `extract_module_declaration` with a **light
  module-decl line scan** (read the leading `module <name>` declaration only — no full parse),
  or share the compiler's own module index. Keep it the single authority for the test module
  map (it overlays co-roots: "later roots win").
- **Win:** the `OnceLock` builds on the **first `compile_multi` call in every rust-gate
  process**, so the floor (~45s deloaded / ~90s loaded here) is paid on **every per-PR rust-gate
  run regardless of which tests run** → a recurring per-PR CI win, not just a Pop-A one.
- **Proof by execution (DESIGN §5):** time the floor before/after (e.g. a single `compile_multi`
  test's wall, or instrument `module_index()` build time); the build-time must drop from ~tens
  of seconds to sub-second, with the test map **byte-identical** (same module→path entries).
- **Pop-A bucket stands (c)**, but split: floor = fixable test-helper artifact; per-test residue
  = genuine seed-perf, **dissolve-on self-host shrink, interim-periodic**. The operator's
  "most-expensive-is-a-fixable-defect" thesis holds for **Pop-B** (subprocess/network (a)/(b)),
  **not** for Pop-A's per-test residue.
- **Open (cheap profiling follow-up):** the ~24s-deloaded fixed per-`compile_sources` cost on a
  2-line file is unexplained at the function level (a constant pass in resolve/typecheck/emit?).
  Worth a flamegraph for whoever owns seed-perf — it is the real recurring Pop-A cost.

## Population A — fierce-hawk's measured (c) set (single-threaded user-CPU ≈ wall)

All bucket **(c) v1-seed-structural**, mechanism = in-process full pipeline. Decision for every
row: **interim-periodic only, dissolve-on = v1 seed shrinks (§7); verify suite-level (b)
amortization first.** Costs are her clean numbers (seconds):

| cost(s) | test |
|--:|---|
| 118.4 | `a4_opacity::opacity_byteoffset_for_charoffset_must_reject` |
| 106.4 | `diagnostics::multiple_missing_exports_each_have_own_span` |
| 100.5 | `cron_tag_test::cron_tag_upsert_protocol_keystone_holds_via_interpreter` |
| 95.1 | `diagnostics::unresolved_type_in_field` |
| 93.0 | `data_cache_scoping_test::data_cache_does_not_leak_across_graphs_on_one_thread` |
| 92.0 | `a4_opacity::opacity_charoffset_for_byteoffset_must_reject` |
| 84.0 | `cron_tag_test::cron_tag_upsert_protocol_witness_discriminates_on_mutation` |
| 81.5 | `diagnostics::parameterized_container_no_false_positive` |
| 81.1 | `a4_opacity::opacity_same_byteoffset_must_accept` |
| 80.6 | `diagnostics::variant_not_reexported_through_type_only_import` |
| 77.7 | `diagnostics::empty_list_wrong_expected_type` |
| 74.8 | `diagnostics::missing_export_points_at_name` |
| 74.5 | `diagnostics::unknown_type_name_no_arity_false_positive` |
| 74.4 | `diagnostics::unresolved_import_names_module` |
| 72.8 | `body_producer_infer_perf_witness_test::…_resolves_clean` |
| 71.0 | `body_producer_infer_perf_witness_test::…_wrong_type_still_rejects` |
| 69.2 | `diagnostics::clean_compile_produces_zero_diagnostics` |
| 67.8 | `diagnostics::empty_list_with_type_context_no_false_positive` |
| 67.5 | `diagnostics::bare_container_type_detected` |
| 67.5 | `data_cache_scoping_test::data_value_is_shared_across_runs_in_one_context` |
| 66.4 | `data_cache_scoping_test::fresh_context_reevaluates_data_independently` |
| 38.9 | `coproduct_reflection_conformance_test::…_behavior_arm_sets_are_distinct` |
| 30.9 | `coproduct_reflection_conformance_test::…_path3_witness_fails_on_dropped_disj_arm` |
| 30.5 | `coproduct_reflection_conformance_test::…_path3_connective_behavior_conformance_holds` |
| 29.5 | `coproduct_reflection_conformance_test::…_path3_pair_witness_fails_on_perturbed_atom_payload_type` |
| 29.4 | `cross_representation_equality_test::genuine_inequalities_stay_false_not_errors` |
| 29.3 | `cross_representation_equality_test::reconciled_and_native_equality_still_true` |
| 28.8 | `cross_representation_equality_test::cross_representation_forks_fail_closed` |
| 28.3 | `coproduct_reflection_conformance_test::…_connective_reflection_pairs_match_syntactic` |

*(g–z tail: fierce-hawk's uncapped single-threaded run is host-saturated (~2.5hr, load 22); a
cap-6s pass gives the ≥6s NAMES — same full-pipeline (c) class — without true seconds. Names to
be appended when her pass finishes; structurally identical to the above.)*

## Population B — already-`#[ignore]`'d subprocess/network set (mostly (a)/(b))

`build_stage0()` (`bootstrap.rs:12`) = `cargo build -p v1-compiler --release` subprocess;
~6 tests each rebuild the **same** binary (a disk-rebuild = (a); duplicated = (b)).

| test (file:line) | mechanism | bucket | decision |
|---|---|---|---|
| `strict_compile_diagnostic_count` (bootstrap:307) | `build_stage0` | (a)+(b) | FIX: consume the floor's prebuilt release bin / build-once fixture |
| `stage0_compile_accepts_dag_target` (bootstrap:331) | `build_stage0` + fs::write | (a)+(b) | FIX: build-once |
| `stage0_compile_imports_ephemeral_generated_source_root` (bootstrap:374) | `build_stage0` | (a)+(b) | FIX: build-once |
| (bootstrap:659) | `build_stage0` | (a)+(b) | FIX: build-once |
| `bootstrap_stage0_to_stage1` (bootstrap:483) | `build_stage0` + `cargo` | (a)+(b) | FIX: build-once; the cargo-check is the real (a) residue |
| `bootstrap_fixed_point` (bootstrap:578) | `build_stage0`×2 + `cargo` | (a)+(b) | FIX: build-once; fixed-point intent is (c) but the rebuild is (a)+(b) |
| `bootstrap_l4_structural` (bootstrap:976) | in-proc compile + `cargo test` | (a)+(c) | FIX the (a): emitted-crate `cargo test` is the cost |
| `complexity_self_analysis_subset` (pipeline:3205) | runs **prebuilt** bin subprocess | (c) via bin | interim-periodic (no rebuild; cost is seed compute) |
| `v1_emits_v2_scoped_compiler_closure_cargo_check_error_count` (pipeline:8043) | in-proc emit + `cargo check` | (a)+(c) | FIX the (a) cargo-check, or keep as boundary diagnostic |
| `v2_trivial_import_emits_rust_that_cargo_checks` (pipeline:8135) | fs::write + `cargo check` | (a) | FIX: cargo-check on emitted output is the defect |
| `review_dag_compiles_to_rust` (pipeline:8173) | in-proc compile + `cargo check` | (a)+(c) | FIX the (a) |
| `review_dag_has_review_subcommand` (pipeline:8273) | in-proc `compile_dag_named` | (c) | interim-periodic (transitive-resolve; verify (b)) |
| `review_dag_emits_cargo_with_deps` (pipeline:8291) | in-proc `compile_dag_named` | (c) | interim-periodic |
| `anthropic_dag_compiles_to_rust` (pipeline:10333) | in-proc `compile_dag_named` | (c) | interim-periodic |
| `full_dsl_compiles` (pipeline:21) | in-proc whole-dsl compile | (c) | interim-periodic; canonical (c); verify (b) resolve redundancy |
| `review_dag_builds_and_runs_dry_run` (pipeline:8310) | `cargo build` + run | (a) → (d) | genuine end-to-end build smoke; periodic if kept |
| `anthropic_live_e2e` (pipeline:10510) | `cargo` + **live Anthropic API** | (a) → (d) | live API can't be hermetic ⇒ genuine periodic residue (or hermetic-replay) |
| `http_pilot_rest_record_then_hermetic_replay_holds` (interp_recorded_fixture:1103) | subprocess + **live jsonplaceholder** | (a) → (d) | replay half already hermetic (floor); wet record-capture is genuine periodic residue |

*(KEEP-PLAIN, not in any green lane: `dump_complexity_report` pipeline:12084 = `--nocapture`
diagnostic dump, not a pass/fail test.)*

## Decision flow (operator's)

1. **(a)/(b) → dispatch fixes, drain, promote back to per-PR gated** (Population B + the
   suite-level (b) on Population A):
   - **Build-once / consume-the-floor-binary**: the ~6 `build_stage0` tests rebuild the same
     `v1-compiler --release` binary the **CI floor already builds**
     (`ci_floor_required_artifacts` = `gunbc` + `claim_executor`). Share one build → (a)+(b)
     drained. Highest single leverage in Population B.
   - **Hermeticize the cargo-check boundary tests** (`v2_trivial_import…`, `review_dag_compiles…`,
     `v1_emits_v2_scoped…`, `bootstrap_l4_structural`): replace the `cargo check`/`cargo test`
     subprocess on emitted output with a cheaper in-process structural check where the intent is
     "emitted Rust well-formed"; else they are genuine (a) build smokes → periodic.
   - **Verify + drain the Population-A suite-(b)**: confirm the ~28–118s fixed floor is a constant
     prelude/closure recompiled per test; if so, a shared compile-fixture / memoized resolve
     (§2; keen-otter-380 one-door `realize`, stern-otter-43 warm==cold) amortizes ~21 tests at
     once — the single highest-leverage fix in the whole set. **This is a hypothesis to verify,
     not a proven defect.**
2. **(c) → dissolve-on tied to self-host shrink** (Population A residue after the (b) check, +
   the in-process Population-B compiles). Interim-periodic ONLY; dissolve-on = seed shrinks (§7).
3. **(d) + interim-(c) → THAT residue earns the nightly lane**, built with the parked cfg_attr
   mechanism (`docs/plans/nightly-ignored-lane.md`). Residue = interim-(c) seed compiles + the
   two genuine-external wet captures (`anthropic_live_e2e`, `http_pilot…`).

## Next / open

- Append Population A's g–z names from fierce-hawk's cap-6s pass (~incoming).
- **Verify the Population-A suite-(b) hypothesis** (does the 28–118s floor amortize under a
  shared/memoized prelude-resolve?) — the pivotal unknown; if true it reclassifies most of the
  cost from (c)-permanent to (b)-fixable. Ties to §2 (sunny-bee subtree).
- Escalate this table to quick-ant-298 for the dispatch decision (which (a)/(b) fixes to spawn,
  and whether to verify the suite-(b) now).
