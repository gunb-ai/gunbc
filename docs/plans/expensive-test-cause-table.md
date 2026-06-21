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

### Bimodal evidence (Population A)

fierce-hawk's single-threaded set has **nothing between 1s and 28s** — tests are either ~instant
(pure unit) or ≥28s (whole-pipeline). A 2-line `module clean\ntype Widget {…}` snippet
(`compile_multi`, **no imports**, so the transitive closure is just itself) still costs **69s**.
The cost is therefore a **fixed whole-pipeline floor, not input-proportional**. PRIMARY cause is
(c) (the seed compiler's inherent per-compile cost). But that fixed ~28–118s floor is paid
**identically across ~21 `diagnostics`/`a4_opacity`/`data_cache_scoping`/`cron_tag` tests** —
so there is a **secondary (b) suspicion at the suite level**: a large constant compiled per
test (an implicit prelude / a constant analysis pass) that a shared compile-fixture or memoized
resolve could amortize. This is the exact "re-does work that could be cached" the parent flagged
(§2, the floor's dormant resolve-cache). Flagged for verification — NOT fixed here.

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
