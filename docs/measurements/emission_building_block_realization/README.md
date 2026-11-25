# emission-building-block-realization — the measurement, not the specimens

Roadmap node: `emission-building-block-realization`, declared as a row of
`gunbc.roadmap.roadmap_authority::declared_roadmap_nodes` (rows carry no symbol of their own, so the row `id:` is the
name; the file position of that row is carried only beside it, never instead of it).
Lane: v2 self-host and native emission. Manager: warm-fox-517. Planning owner: still-gull-398.
Session: tidy-hawk-691. Manager's ruling on shape (2026-11-24): **one PR, one leaf**, the battery carried as
commits behind this report rather than as a second PR.

## Scope of THIS PR, stated so a reader is not misled

The paragraphs below describe **the measurement commit**, not the whole branch, and the two are not the
same thing. Concretely:

| Commits | What they add |
|---|---|
| `8e3b2847a1` (amended as `ec2ba22b`, then `82ecabc035566df010b510795b0370d783969a34`) | this report, and one comment in `dag/gunbc/emit_state_threading_specimen.dag` |
| `b30632eda1c`, then `187b6f719fc` and later battery commits | `dag/gunbc/emit_building_block_contracts.dag` (the 23 building-block contracts), the instrument batteries `dag/gunbc/instruments/emit_*_observation.dag`, and their target rows in `dag/gunbc/instrument_targets.dag` |

So the sentence "this report repairs nothing, emits nothing new, and adds no claim rows" is true **of the
measurement commit** and was, until this section, being read as a statement about the PR. It is not: the PR
now carries the first batteries of deliverable two, and they add claim rows and a target. Reading the
earlier wording as the PR's file list will get the change's scope wrong, which is why it is now scoped to
the commit that earned it.

**Why deliverable two lands while the parse-population profile still does not exist.** The profile is still
absent at this revision (§1 is unchanged on that point, and the specimen comment still says so). The
precondition was not quietly dropped; it was **replaced by a recorded basis change**. The planning owner's
ruling (neat-badger-614, relayed by warm-fox-517) permits this row's observations to be selected from this
row's own pinned allocation-counter data, with the profile cited as corroboration only, provided the reason
is stated with numbers and the basis's limitation is stated with it. That ruling is recorded on the roadmap
row itself, as an `updated(t:, note:)` in the updates layer of `dag/gunbc/roadmap/roadmap_authority.dag` —
the note is 1974 characters, sha256 `9dd0c6865134a315b1828b55d3c8f3f649b7522bcdd5cf53a441de2088d59d88`,
verified by extraction from the file at the head of the batched planning PR, not from memory. What the basis
can see is operations and bytes only: no traversal count, no per-phase lifetime, and those two gaps remain
owed by the vocabulary work rather than being claimed here
(`gunbc.emit_copy_qualification copy_work_dimension_unavailable_cause`). A reader who wants the profile's own
numbers is still waiting on that row; nothing in this PR supplies them.

This report is a **measurement**. It repairs nothing and changes no emitter: the only file the measurement
commit changes is `dag/gunbc/emit_state_threading_specimen.dag`, whose provenance sentence pointed at an
artifact that does not exist (§1).

## Evidence header

| | |
|---|---|
| Base, recorded by my own fetch | `origin/main = 56d460116d6252f6ec8dfe889ea358c804685861` |
| Base tree | `9096caefc17d90bc7edddf5c565e3fa57daa4afb` (`git rev-parse origin/main^{tree}`) |
| Checkout the run used | HEAD `56d460116d6252f6ec8dfe889ea358c804685861`, TREE `9096caef...`, `git status --porcelain` empty (0 bytes) |
| HOST | `srv1` |
| BOOT (srv1's own `/proc/1` mtime) | `2026-11-22 22:51:15.200000000 +0000` |
| Toolchain | `rustc 1.93.0 (254b59607 2026-01-19) aarch64-unknown-linux-gnu`, LLVM 21.1.8 |
| CLI binary | `sha256 32c3a544d1065ef134ede76cb48e5836b54405952380ba7764a79e5255071dc8`; `./gunbc --version` prints `gunbc 56d460116d6252f6ec8dfe889ea358c804685861` |
| Binary provenance | **Not the released runner.** Built by me at the base sha (§0). The released `/opt/gunbc/bin/gunbc` self-reports `tree:sha1:09fd98d09d6ba92d75c80145e97c4f7b2aa5a9ed`, which is not main's tree `9096caef...`, so it is not source-equal to this base and is never described as such here. |
| Limiter and slots, as printed by the tool | `HEAVY_OWNER=tidy-hawk-691`. `heavy-slot: slot 3/12 acquired by tidy-hawk-691 at 2026-11-24T11:38:59Z: cargo build --release -j 12 --bin gunbc -p v1-compiler` and `heavy-slot: slot 2/12 acquired by tidy-hawk-691 at 2026-11-24T11:48:13Z: .../release/gunbc test //gunbc/instruments:native-allocation-observation`. Nothing was built in the session container. |
| Emitted specimen crate | 56 files emitted, source closure `4a13972596359701cd4f879afbaf1bc7c609bf495fb8db69a71749f5c1f59569` (identical across both runs); built `cargo build --release --offline --message-format json` with `RUSTFLAGS="-D warnings"`, `exit_status=0 warning_count=0`; binary `.../target/release/gunbc-emitted-closure-dag-gunbc-instruments-native-allocation-observation-dag`. The binary is **not byte-reproducible** across runs: `sha256 7bce1a57d7bd67ba87a603d674fa56950d72723e6fcdfbfde80aff2fb6a0876f` (run at 11:48:13Z) and `sha256 62acb0fa9ae9461e9e20d75abf9fb49e2bc9761fe26aa813df85d87f52eebb7f` (run at 13:14:50Z). Cause UNKNOWN. |

Witness labels, stated per claim as the lifecycle requires:

* **"green by execution in run <run-link comment on this PR>"** — I ran it; raw lines pasted in §4.
* **"parses + required lane"** — typechecked by CI, not executed by me. gunbc CI executes no claims.
* **UNKNOWN** — I could not observe it.

**Honesty note, because this row exists to catch exactly this.** An earlier draft of this report
pasted a witness block from memory rather than from the log file. It contained lines that the log does
not contain (a `PASS` line I had not seen, per-case `ops=`/`bytes=` numbers on the four dying-base
cases, a `specimen_main` path, a `closure=sha256:4a13972575cb280d…` value, and a cardinality-0
`256 ops / 8584 bytes` reading), and two citations that do not exist (`ExpectedCopyWork` in the
instrument, and an "exit code contract" in `docs/measurements/roster_import_sweep_native/README.md`).
Every number in §4 below was read back off the log file on disk. The correction is recorded here
rather than edited silently, because a measurement report that quietly restates itself is worthless.

## 0. Reproducing the executed witness

```
# my worktree: bundle the exact base commit, transfer it, check it out
git update-ref refs/tmp/base 56d460116d6252f6ec8dfe889ea358c804685861
git bundle create base-full.bundle refs/tmp/base
scp base-full.bundle briansrls@srv1.tailecbe08.ts.net:~/scratch/tidy-hawk-691/
ssh briansrls@srv1.tailecbe08.ts.net
  git init tree-exact && git remote add origin ../base-full.bundle
  git fetch origin refs/tmp/base:refs/heads/base && git checkout base
  # HEAD = 56d460116d6252f6ec8dfe889ea358c804685861, TREE = 9096caef..., status clean

# build the CLI at the base sha (never in the session container)
export PATH=$HOME/.cargo/bin:$PATH CARGO_NET_OFFLINE=true \
       CARGO_TARGET_DIR=$HOME/scratch/tidy-hawk-691/shared-target
HEAVY_OWNER=tidy-hawk-691 /home/briansrls/bin/heavy-slot \
  cargo build --release -j 12 --bin gunbc -p v1-compiler        # BUILD_EXIT=0

# execute the instrument, exit status captured into the same evidence file
HEAVY_OWNER=tidy-hawk-691 /home/briansrls/bin/heavy-slot \
  $CARGO_TARGET_DIR/release/gunbc test //gunbc/instruments:native-allocation-observation
```

Why a bundle and not `git archive`: `git archive` drops `export-ignore` paths. My first tree from
`git archive` hash-checked as tree `c4f1317147c1e903b7586cd28bd80002512fec63` against main's
`9096caefc17d90bc7edddf5c565e3fa57daa4afb`, i.e. it was **not** content-equal to main, and was
discarded. The bundle route reproduces main's tree exactly, which is what the identity claim in the
header depends on.

## 1. The parse-population profile is absent at this base

The first_slice says specimens are "chosen from the parse-population profile"
(`gunbc.roadmap.roadmap_authority::declared_roadmap_nodes`, the row whose `id` is
`emission-building-block-realization`). No such durable artifact exists here.

| Command, at `origin/main` = `56d460116d` | Hits |
|---|---|
| `git grep -inE "population.profile" origin/main -- .` | **2 hits, 2 files**: `dag/gunbc/emit_state_threading_specimen.dag`, `dag/gunbc/roadmap/roadmap_authority.dag` — both references, neither a profile |
| `git grep -icE "perf self time" origin/main` | 1 — inside the `first_slice` field of the `emission-real-consumer-profiles` row of `gunbc.roadmap.roadmap_authority::declared_roadmap_nodes` |
| `git grep -icE "collection, allocation, provenance index, lexer and parser"` | 1 — same line |
| `git grep -icE "165-module" origin/main` | 4 — that same `first_slice` field plus `dag/gunbc/recurring_failure_mode/census_rows_read_the_loaded_population.dag`, where 165 is the fleet shard count, not a profile |
| `git grep -icE "five largest dag" origin/main` | 0 |
| `git ls-tree -r --name-only origin/main \| grep -iE "population\|profile"` | no parse-population baseline artifact among the hits |

Consequence, and the reason this report carries no specimens: choosing a specimen shape today is
choosing by taste, which `census_rows_read_the_loaded_population.dag` names as the failure mode this
project exists to avoid. Specimen design is paused; the profile is `emission-real-consumer-profiles`'s
deliverable, not this row's.

**One consequence for a file that already exists.** `dag/gunbc/emit_state_threading_specimen.dag`
asserted at line 6 that its shape was "taken from the parse-population profile". No such artifact
exists at this sha, so that sentence cited nothing. It now cites the roadmap first_slice wording it
was built from, and states that it is not evidence about the real consumer population. The specimen's
shape is unchanged, so §4's executed witness still covers the same code.

## 2. The contract vocabulary as it exists today

Carrier: `dag/gunbc/emit_copy_qualification.dag`, 914 lines, **0 `test fn` rows** — nothing in this
module executes by itself. Every line below was read off that file at the base sha.

The grid is **five axes**, not four — the ticket names four; `InputSizeBasis` is the fifth.

| Name | Line | Content |
|---|---|---|
| `GridAxis` | 28 | the five axis tags |
| `RulingShapeAssignment` / `ruling_shape_assignments` | 35 / 41 | 10 named ruling shapes ("linear use", "fold/map", "closure capture", "loop-carried accumulator"…), each bound to one axis member |
| **`OwnershipPattern`** | 55 | 10 members: `ConsumedOnce` 56, `BorrowedReadOnly` 57, `ByValueThenReused` 58, `BranchExclusive` 59, `ReuseAcrossJoin` 60, `AliasThenMutate` 61, `MoveIntoClosure` 62, `BorrowIntoClosure` 63, `LoopCarriedAccumulator` 64, `FieldExtractThenReuseEnclosing` 65 |
| **`CarrierRealization`** | 67 | 7 members: `CopyScalarControl` 68, `OwnedStringCarrier` 69, `DeepCloningSequenceCarrier` 70, `SharedRecordCarrier` 71, `PersistentSequenceCarrier` 72, `PersistentMapCarrier` 73, `NestedCheapDeepRecordCarrier` 74 |
| **`ControlShape`** | 76 | 4 members: `StraightLine` 77, `IfSplit` 78, `MatchBinding` 79, `LoopRecursion` 80 |
| **`ConsumerContract`** | 82 | 2 members: `ConsumesByValue` 83, `BorrowsByRef` 84 |
| `InputSizeBasis` | 86 | 2 members: `ScalarUnitBasis` 87, `SizedCardinalityBasis` 88 |
| `declared_cardinalities` | 93 | `[0, 4, 16, 64]` — the sizes symbolic expectations are evaluated at |
| `EmitCopyQualificationCaseId` | 97 | the five-tuple case record |
| slug fns | 106, 121, 133, 142, 149, 156 | `ownership_pattern_slug`, `carrier_realization_slug`, `control_shape_slug`, `consumer_contract_slug`, `input_size_basis_slug`, `case_slug` |
| `scalar_cardinality` | 95 | `1` |
| enumerations | 165, 171, 176, 178, 180 | `all_ownership_patterns` … `all_input_size_bases` |
| `StructuralInapplicability`, `CaseApplicability` | 191, 202 | why a grid cell cannot exist |
| gating fns | 223, 250, 259, 268, 280, 299, 334, 345, 357, 369, 376 | `pattern_admits_shape`, `shape_admits_only_loop`, `shape_admits_only_straight_line`, `carrier_is_record`, `pattern_admits_carrier`, `pattern_admits_contract`, `basis_admits_carrier`, `carrier_is_sized`, `carrier_realized`, `first_refusal`, `case_applicability` |
| roster | 385, 390, 411, 415, 422, 426 | `full_case_roster`, `roster_denominator`, `row_is_applicable`, `applicable_case_rows`, `applicable_case_count` |
| cost vocabulary | 433, 440, 451, 456, 460, 486, 493, 508, 519, 525 | `CopyCostClass`, `CloneMinimality`, `CloneUnjudgeableGround`, `CloneLintDiscrimination`, `clone_lint_discrimination`, `CloneRealizationFact`, `carrier_clone_realization`, `AllocGrowthClass`, `ExpectedCopyCost`, `ExpectationDerivation` |
| derivation | 532, 555, 565, 586, 622, 656 | `required_whole_value_materializations`, `materialization_cost_class`, `materialization_alloc_growth`, `derive_expectation`, `derive_materialized_expectation`, `linear_alloc_growth_bound_bytes_per_element` |
| observation | 659, 664, 673, 682, 684, 695, 705, 709, 713, 717, 724 | `StaticCloneObservation`, `CopyWorkDimension`, `ObservedCopyWork`, `copy_work_dimension_unavailable_cause`, `observed_copy_work_from_allocator`, `AllocObservationRow`, `alloc_reading_nat`, `allocated_byte_size`, `byte_size_gt`, `byte_size_ne`, `RuntimeObservation` |
| verdict | 729, 745, 757, 769, 783, 787, 800, 846, 851, 884 | `EmitCopyQualification`, `qualification_is_qualified`, `qualification_case`, `qualification_slug`, `GrowthJudgment`, `ConsecutivePairScan`, `judge_alloc_growth`, `BehaviorAgreement`, `judge_case`, `judge_growth_arm` |

Who consumes it: `emit_copy_qualification_fixture_gen.dag` (roster → fixtures),
`instruments/emit_copy_qualification_transport.dag` (`judge_case`, `qualification_is_qualified`,
`carrier_clone_realization`, `alloc_reading_nat`), and `dag/test/claim/emit_copy_qualification_witness.dag`
(**35 `test fn` rows**, parses + required lane; see §6c).

## 3. What the vocabulary can and cannot state

Against the conditionals the ticket requires every contract to carry. Counts are
`grep -ic <term> dag/gunbc/emit_copy_qualification.dag` at the base sha.

| Conditional the ticket asks for | State today | Evidence |
|---|---|---|
| the state it runs in | **partial** — no state axis; nearest are `OwnershipPattern` members (`BorrowedReadOnly` 57, `AliasThenMutate` 61) and the wet battery's own axes | `observation_state`: 0 |
| input size | **yes** — `InputSizeBasis` (86) × `declared_cardinalities [0,4,16,64]` (93) | — |
| payload shape | **no axis** | `payload`: 2 hits, both prose about fixture element bytes (646, 649); no member, no field |
| observation versus consumption | **no** — `ConsumerContract` (82) distinguishes *by-value* from *by-ref*, which is not observation vs consumption | `Observe`: no member on either axis |
| live aliases / retained versions | **no** | `live_aliases`: 0, `retained_versions`: 0; `alias`: 10 hits, all `AliasThenMutate` and prose |
| escape | **no axis** — escape appears only as two pattern members (`MoveIntoClosure` 62, `BorrowIntoClosure` 63) | `escape`: 0 |
| representation entering and leaving | **no** | `representation`: 0 |
| construction / steady-state / release / teardown, separately and in total | **no** — what exists is `AllocGrowthClass` (508) + `AllocObservationRow` (695) + `judge_alloc_growth` (800), an asymptotic judgment over consecutive size steps, not a per-phase report | `steady`: 0, `teardown`: 0, `release`: 0, and all 4 `construction` hits (27, 50, 507, 704) are prose, not a field or member |
| traversal count where the claim depends on it | **no** — not expressible in this carrier at all | `traversal`: 0 |

`v2.lens.cost` (`src/v2/lens/cost.dag`) cannot carry the rest. Its one meaning is complexity:
`SymbolicCost` (55) and `AsymptoticClass` (65: `O1`, `LogN`, `Linear`, `LinearNLogN`, `Polynomial`,
`Exponential`, `Unbounded`, `Unknown`), with `add`/`mul`/`poly`/`eval` over the axes
`size:n, aux:m, depth:d, leaves:l, nodes:u, height:h`. Hit counts at the base sha:
`ownership` 0, `alias` 0, `escape` 0, `borrow` 0, `clone` 0, `traversal` 0, `alloc` **1** — and that
one hit is `Alloca { result: _, allocated_type: _, element_count: _ } => 1` inside
`v2.lens.cost::llvm_instruction_cost` (file `src/v2/lens/cost.dag`, fn declared at 1020, the arm at 1027),
an entry in a per-instruction size table, not an allocation-count axis. **There is no ownership axis and no allocation-count axis.** The lens is
not stretched to fit; a stated absence is a result. Adding such axes is a design question for
neat-badger (§7 proposal, unimplemented), not something to bolt onto a lens in a measurement PR.

The carrier also limits "the counters and the emitted operations are both read" by its own admission,
line 682, `copy_work_dimension_unavailable_cause`: the counting allocator observes allocation
operations and bytes only, and planting per-clone counters would require hooks in production carriers.
So today the counter arm reads ops+bytes, and the operation arm is the wet transport's clippy clone
lint, whose executing consumer is deliberately absent (§6b).

## 4. The executing denominator: one run, 11 cases, 2 specimen modules

This is the **only** executing specimen machinery in this area of the tree: the instrument
`dag/gunbc/instruments/native_allocation_observation.dag`, registered at
`dag/gunbc/instrument_targets.dag` (label `native_allocation_observation_label` 494-495, entry
498-499, binding row 948-949 with `NativeClaimProgramProducer`, and the binding is listed in the bound
population at 1196), invoked as
`gunbc test //gunbc/instruments:native-allocation-observation`.

**Case split, read off the source and the printed roster: 7 + 4 = 11.**
State-threading case identities at `native_allocation_observation.dag:217-224` (7, the last being
`subject_reading_taken`), inside `fn allocation_cases()` (207); dying-base identities at `:243-246`
(4), inside `fn dying_base_cases()` (233). The "5 and 6" split quoted at me from the lane is wrong on
both sides; the figures here are from the file and the run.

Raw lines, read back from the log file on srv1 (`~/scratch/tidy-hawk-691/run3.log`), the run-link
comment on this PR carries the same block:

```
HOST=srv1 BOOT=2026-11-22 22:51:15.200000000 +0000
BASE_SHA=56d460116d6252f6ec8dfe889ea358c804685861 BASE_TREE=9096caefc17d90bc7edddf5c565e3fa57daa4afb
BINARY_SHA256=32c3a544d1065ef134ede76cb48e5836b54405952380ba7764a79e5255071dc8  .../shared-target/release/gunbc
heavy-slot: slot 5/12 acquired by tidy-hawk-691 at 2026-11-24T13:14:50Z: /home/briansrls/scratch/tidy-hawk-691/shared-target/release/gunbc test //gunbc/instruments:native-allocation-observation
v2-native-route: emitting dag/gunbc/instruments/native_allocation_observation.dag (seed, in-process)
v2-native-route: emitted 56 files into /tmp/gunbc-emit-compile-266940-1795526319337556744-0/dag_gunbc_instruments_native_allocation_observation_dag (closure 4a13972596359701cd4f879afbaf1bc7c609bf495fb8db69a71749f5c1f59569); emission arena released (rss_kb_before=Some(7560004) trim_reclaimed_kb=Some(111644) rss_kb_after=Some(7448360)); cargo build
v2-native-route: emitted crate built — argv=["cargo", "build", "--release", "--offline", "--message-format", "json", "--color", "never", "--manifest-path", "/tmp/gunbc-emit-compile-266940-1795526319337556744-0/dag_gunbc_instruments_native_allocation_observation_dag/Cargo.toml"] RUSTFLAGS="-D warnings" compiler=/home/briansrls/.cargo/bin/rustc exit_status=0 warning_count=0 cargo_network_retries=0 rustc=rustc 1.93.0 (254b59607 2026-01-19); binary: rustc; commit-hash: 254b59607d4417e9dffbc307138ae5c86280fe4c; commit-date: 2026-01-19; host: aarch64-unknown-linux-gnu; release: 1.93.0; LLVM version: 21.1.8
v2-native-route: emitted compiler at /tmp/gunbc-emit-compile-266940-1795526319337556744-0/target/release/gunbc-emitted-closure-dag-gunbc-instruments-native-allocation-observation-dag (sha256 62acb0fa9ae9461e9e20d75abf9fb49e2bc9761fe26aa813df85d87f52eebb7f)
v2-native-route: discriminating red established — Discriminated subject=EntryOwnModule module=gunbc_instruments_native_allocation_observation red=338 | pub const EMIT_COMPILE_MUTATION_PROBE: u8 = "the phase's own discriminating red";
roster availability_provider_registered_by_the_selecting_main,observation_empty_window_reads_zero,observation_known_allocation_reads_both_axes,observation_walks_return_the_given_sequence,observation_negative_head_stops_specimen_and_reference_alike,calibration_cloning_control_reads_above_reference,subject_reading_taken,dying_base_unique_outer_computes_from_old_fields,dying_base_retained_outer_is_unchanged,dying_base_retained_nested_is_preserved,dying_base_optional_nested_slot_absent_and_present
case availability_provider_registered_by_the_selecting_main held observed=available=true
case observation_empty_window_reads_zero held observed=empty_ops=0 empty_bytes=0
case observation_known_allocation_reads_both_axes held observed=known_4096_text_ops=4107 known_4096_text_bytes=12360
case observation_walks_return_the_given_sequence held observed=tokens=2000
case observation_negative_head_stops_specimen_and_reference_alike held observed=specimen_tokens=1 reference_tokens=1
case calibration_cloning_control_reads_above_reference held observed=cloning_ops=43711 cloning_bytes=21853472 reference_ops=6112 reference_bytes=1401008
case subject_reading_taken held observed=n=2000 producer=emitted-native-counting-allocator specimen_ops=4116 specimen_bytes=1322440 reference_ops=6112 reference_bytes=1401008 subject=WITHIN_DECLARED_REFERENCE_OPERATION_COUNT
case dying_base_unique_outer_computes_from_old_fields held observed=k=8 n=4
case dying_base_retained_outer_is_unchanged held observed=retained_k=5 result_k=8
case dying_base_retained_nested_is_preserved held observed=kept_k=1 result_k=4
case dying_base_optional_nested_slot_absent_and_present held observed=absent_k=3 present_k=3
native-claim: entry=dag/gunbc/instruments/native_allocation_observation.dag closure=4a13972596359701cd4f879afbaf1bc7c609bf495fb8db69a71749f5c1f59569 binary=62acb0fa9ae9461e9e20d75abf9fb49e2bc9761fe26aa813df85d87f52eebb7f seed=32c3a544d1065ef134ede76cb48e5836b54405952380ba7764a79e5255071dc8 warning_count=0 status=0 child_wall_ms=40 deadline_ms=600000 -- held
REAL_EXIT_STATUS_OF_gunbc_test=0
```

**Reproducibility across two runs of the same command at the same sha.** The emitted source closure is
byte-stable: `4a13972596359701cd4f879afbaf1bc7c609bf495fb8db69a71749f5c1f59569` in both runs, and all 11
case readings are identical in both. The emitted **binary** is not: run 1 at 11:48:13Z (slot 2/12) produced
`sha256 7bce1a57d7bd67ba87a603d674fa56950d72723e6fcdfbfde80aff2fb6a0876f`, run 2 at 13:14:50Z (slot 5/12)
produced `sha256 62acb0fa9ae9461e9e20d75abf9fb49e2bc9761fe26aa813df85d87f52eebb7f`. The counters therefore
reproduce; a sha256 of an emitted crate binary pins the build, not the measurement. Cause not
investigated — UNKNOWN.

What those numbers actually say, and what they do not:

* **Only 4 of the 11 cases carry allocation counters.** `observation_empty_window_reads_zero`
  (0 ops / 0 bytes, the empty-window floor), `observation_known_allocation_reads_both_axes`
  (**4107 ops / 12 360 bytes** for a 4096-byte text allocation — the positive control that proves the
  counter, so a 0 above means "no allocation", not "no counter"),
  `calibration_cloning_control_reads_above_reference` and `subject_reading_taken`.
* **Observation and state threading at cardinality 2000:** the specimen walk reads **4116 ops /
  1 322 440 bytes** against the reference walk's **6112 ops / 1 401 008 bytes**, verdict
  `WITHIN_DECLARED_REFERENCE_OPERATION_COUNT`. This is the ticket's "small nonescaping result and
  cursor advanced in a repeated walk", measured.
* **The one violating variant that exists:** the cloning control reads **43 711 ops / 21 853 472
  bytes** against that same reference — ~7.2× the operations and ~15.6× the bytes on the same path,
  asserted as `calibration_cloning_control_reads_above_reference`. It is the ticket's "per-step
  allocation of the accumulator" red control, and it is the **only** red control executing today.
* **5 cases are availability or token-count assertions** (`available=true`, `tokens=2000`,
  `specimen_tokens=1 reference_tokens=1`) — they establish that the counter is wired and that the
  walks agree on shape, not what a step costs.
* **The 4 dying-base cases observe field values only** — `k=8 n=4`, `retained_k=5 result_k=8`,
  `kept_k=1 result_k=4`, `absent_k=3 present_k=3`. **No allocation counter is read in any of them.**
  So "a retained record is not re-copied" is asserted today by field identity, not by an allocation
  count; the ticket's "owning retains, payload copies and collection copy-on-write are distinct
  observations" is not yet observable in the executing witness.
* `dying_base_unique_outer_computes_from_old_fields` holds with `k=8`. Whether reading from the old
  fields is the contract or the lowering defect owned by `emission-ownership-facts-survive-lowering`
  is that row's call; it is recorded here because the executing witness cannot tell the two apart, and
  `held` must not be read as endorsement.

## 5. The building-block map

Boundary list, against the executed denominator. "Unqualified" is the row's own word for *did not
execute*, and per the ticket's red_control it means *does not count*.

| Building block | Executing specimen? | Contract in `emit_copy_qualification` terms | Status |
|---|---|---|---|
| Value — scalar | exercised as `Int` throughout | no per-block contract | **unqualified** |
| Value — record | `RefAcc` (instrument 58), `DbInner`/`DbOuter` (specimen) execute | none attached | **unqualified** |
| Value — nonrecursive variant | `DbOpt` absent/present executes | none attached | **unqualified** |
| Generic instance over a scalar | none | `PersistentSequenceCarrier` is an axis member (72) with no executing case | **unqualified** |
| Generic instance over a nontrivial payload | none | same | **unqualified** |
| Transform | none | — | **unqualified** |
| Branch | the `if h < 0` guard executes inside `reference_walk` (76) and `spec_walk_tokens` | `IfSplit` (78) never bound to a case | **unqualified** |
| Loop with a record accumulator | **yes** — `spec_walk_tokens` / `reference_walk`, 4116 ops vs 6112 reference | `LoopCarriedAccumulator` (64) × `LoopRecursion` (80) never instantiated as an `EmitCopyQualification` | **measured, uncontracted** |
| Bind | incidental (`let`) | none | **unqualified** |
| Match on an owned temporary | `dying_base_optional_nested_slot_absent_and_present` executes (field values only) | `MatchBinding` (79) never instantiated | **unqualified** |
| Match on a borrowed shared sum | none | — | **unqualified** |
| Field read | executes inside the rebuild cases | no contract, no separate count | **unqualified** |
| Record rebuild, one field changed | **yes**, but observes field values only — no counter read | none attached | **measured as behavior, unmeasured as cost** |
| Local closure | `map(x => x)` executes | none | **unqualified** |
| Escaping closure | none | `MoveIntoClosure`/`BorrowIntoClosure` are members (62-63) with no case | **unqualified** |
| Sequence build | **yes** — `list_push`/`concat` inside both walks | none | **measured, uncontracted** |
| Element read | **yes** — `get(i, seq)` inside the walks | none | **measured, uncontracted** |
| Sequential traversal | **yes** — `fold`/`map` in both walks, counted at cardinality 2000 | none | **measured, uncontracted** |
| `PersistentSequenceCarrier` | none | axis member (72), zero executing cases | **unqualified** |
| Index build / lookup / traversal | none | — | **unqualified** |
| `PersistentMapCarrier` | none | axis member (73), zero executing cases | **unqualified** |

Read plainly: of 22 named blocks, **5 execute**; **1** of those carries a violating variant with a
discriminating run (the accumulator cloning control). **None carries a contract**, because no
`EmitCopyQualification` record is constructed on the executing path: the instrument builds
`AllocationCase` records (124) and judges them with `span_at_least` (155), `operations_above` (164)
and `operations_within` (174), while the qualification judges `gunbc.emit_copy_qualification::judge_case` (851) and
`gunbc.emit_copy_qualification::judge_growth_arm` (884) are reached only by the wet transport, whose consumer is deliberately absent.

## 6. Findings

a. **Parse-population profile absent** — §1, with the greps. Fixed where it had already caused a
   false citation: `emit_state_threading_specimen.dag` line 6 now cites its real source.

b. **The wet battery's absence is an absence wall, correct as written** — checked by me, not taken on
   anyone's word. `dag/test/claim/deleted_lanes_countable_witness.dag:30`
   `deleted_lane_job_ids_are_absent` returns true only when none of `fabric-evidence`,
   `rust-unit-tests`, `emit-copy-qualification-battery`, `required-v2-native` appears among the floor
   lane job ids; `:39` `w_RED_the_deleted_lanes_do_not_return` asserts both directions, including that
   supplying a deleted id authors RED, with a positive-control seam.
   `gunbc.rung_drop.emit_copy_qualification_without_a_consumer` (`declared: 2026-09-04`, standing
   `DeletedWithoutReplacement`) states the battery **has never executed**. So any block qualified by
   that battery reads unqualified, and re-adding the lane without a consumer is meant to go red there.
   No file in that wall was touched by this PR.

c. **The qualification vocabulary's own 35 executable rows are not executed here.**
   `dag/test/claim/emit_copy_qualification_witness.dag` has 35 `test fn` rows over the roster,
   denominator, derivation and growth judges. They are **parses + required lane**. Whether they pass is
   **UNKNOWN** to this report.

d. **`v2.lens.cost` cannot carry this row** — §3, with the hit counts. Stated absence, not stretched.

e. **The route named "v2" emits with the seed emitter.** The log line reads
   `v2-native-route: emitting dag/gunbc/instruments/native_allocation_observation.dag (seed,
   in-process)`, and the emitter selection is the `eprintln!` inside
   `prepare_emitted_compiler_for_entry` (`src/v1/stage0/src/cli_run/native_lane_runner.rs`, fn declared at
   412, the line at 587). There is no v2-emitter selection on that
   path, so every v2 result today reads **unqualified**, which is what the first_slice anticipated
   ("where v2 cannot yet execute it, the exact blocker is named"). The named blocker is that line and
   that route. I did not attempt a v2-emitter-only execution and make no claim about whether it could.

f. **Released-runner mismatch.** `/opt/briansrls`-adjacent released binary `/opt/gunbc/bin/gunbc`
   reports `tree:sha1:09fd98d09d6ba92d75c80145e97c4f7b2aa5a9ed` ≠ main's tree `9096caef...`. Not used.
   My binary is **source-equal to `56d460116d`** (its `--version` prints that commit) and is never
   called a released runner.

## 7. Grounding search, then a proposal to be ruled on — nothing implemented

Per DESIGN, ownership and realization facts need their own carrier, and one must not reinvent what
std already has. Grounding search over the *actual* std and extdeps trees — `dag/std` and
`dag/extdeps`; there is no `src/std` at this sha — in declared-type form
`^type [A-Za-z_]*(Owned|Borrow|Alias|Escape|Owner|Realization|Mutab|Share)`. Noise first: case-insensitive
substring searches over the same trees return `Rc` 12 590 hits across 925 files, which is substring
noise and is recorded only to be dismissed. The declared-type hits:

| Existing carrier | Line | What it already means |
|---|---|---|
| `extdeps/languages/rust/types.dag` `OwnershipKind` | 133 | Rust surface spelling: `Owned \| SharedRef \| MutRef \| RawPtr` (134-137), with `BoxPtr \| RcPtr \| ArcPtr` and `rc_template = "Rc<…>"` data in the same module |
| `extdeps/languages/rust/representation.dag` `RustRepresentationRealization` | 58 | how a `.dag` shape is spelled as Rust |
| `std/realization_cost_fidelity.dag` `RealizationCostFact` / `RealizationCostFidelityRow` / `RealizationCostLookup` / `RealizationCostFidelityReport` | 156 / 361 / 378 / 483 | realized-cost facts and their fidelity report — the closest existing meaning to what this row wants |
| `std/reference_realization.dag`, `std/operator_realization.dag`, `std/native_test_realization.dag`, `std/realization_inherent.dag`, `std/realization_transport.dag`, `std/multi_axis_realization_fidelity.dag`, `std/realization_cost_fidelity.dag`, `std/realization_cost_fidelity.dag` | module-level | the realization authorities the carriers already import |

**Proposal, for a ruling by neat-badger. Nothing here is implemented.** A carrier of ownership and
realization facts — call it `gunbc.emit_ownership_facts` — owning the five axes the current
vocabulary cannot state, attached to what the search found rather than invented:

```
// proposed sketch ONLY — no such file exists in this PR
type FactPhase          = Construction | SteadyUse | Release | Teardown   // carrier has none of these as members
type FactRepresentation = ByValue | BySharedRef | ByMutRef | ByRc          // attaches to extdeps OwnershipKind 133
type FactAliasState     = NoLiveAlias { } | LiveAlias { readers: Int } | RetainedVersion { count: Int }
type FactEscape         = DoesNotEscape | EscapesViaClosure | EscapesViaReturn
type FactOperationAccount { traversals: Int, element_reads: Int, rebuilds: Int }  // the traversal count the carrier cannot hold
type OwnershipFact {
  pattern: OwnershipPattern            // reused, emit_copy_qualification.dag:55
  realization: CarrierRealization      // reused, :67
  control: ControlShape                // reused, :76
  consumer: ConsumerContract           // reused, :82
  phase: FactPhase
  entering: FactRepresentation
  leaving: FactRepresentation
  alias: FactAliasState
  escape: FactEscape
  operations: Optional<FactOperationAccount>
}
```

It reuses the four existing terms instead of duplicating them, adds the missing axes, and leaves
`v2.lens.cost` alone as the single-meaning complexity authority. Whether this should be a new module,
extra members on the existing five axes, or fields on `EmitCopyQualification` is the ruling asked for.

## 8. Review round 1, and the citation audit it prompted

A review on this PR (review artifact for the 14:46:34Z REQUEST_CHANGES) caught that the provenance
sentence I had just repaired cited by position — `roadmap_authority.dag:3126` — which DESIGN forbids:
`gunbc.design_argument::single_authority` says "Cite the symbol, not the position", a `file:line` pointer
being "a second, positional naming scheme that any edit above it silently invalidates", with a position
carried only where no symbol exists and then beside the symbol. I verified that text in DESIGN.md rather
than taking the review's word. The specimen comment now cites
`#gunbc.roadmap.roadmap_authority::declared_roadmap_nodes` and names the row by its `id`, and every
position left in this report sits beside the symbol that owns it. The report cites symbols in
`module::decl` spelling without the `#` marker, because markdown comments are not what the reference
checker reads.

That review also prompted a full audit of every `file:line` in this report against the files. It found one
more of my own errors: I had written that the instrument is registered at `instrument_targets.dag:59`,
which is a comment; the rows that bind `//gunbc/instruments:native-allocation-observation` are
`native_allocation_observation_label` (494-495), the entry (498-499), the `TargetBinding` naming
`NativeClaimProgramProducer` (948-949), and its listing in the bound population (1196). That citation had
come from the same kind of elided tool read as the numbers in the honesty note above.

## 9. What this PR deliberately does not do

* no new specimens, specimen claim files, or contracts (deliverable two, gated on the profile);
* no emitter change and nothing under `src/v2` (the blocker in §6e is named, not fixed);
* no composite patterns (`emission-composite-patterns`);
* no claim rows added anywhere, so no new claim can be reported as executing when it does not;
* no edit to `deleted_lanes_countable_witness.dag`, `instrument_targets.dag`, the wet battery, the
  transport, or any file held by another live worker in this lane.
