# The frontier cohort board, and what a root weighs across fifteen doors (2026-08-24)

**Session:** `quiet-eagle-429`. **Work item:** `node://adhoc-8e1904e7-24d`.

Every mechanism root the self-host repair queue is ranked on was traced from **one** entry,
`src/v2/compiler/03_ingest.dag`. This is the other fifteen, boarded in one cohort at one ref, and
the join over them. It is a measurement, not a repair and not a gate: no count here may be quoted
as a merge-blocking literal (DESIGN.md §5 — a figure copied from the current tree is not an
oracle).

## The sizing rule this cohort establishes

**Read this before sizing any emitter root, from this board or from a future one.**

- **Size by DISTINCT SITES, never by manifestations.** A manifestation is one rustc diagnostic; a
  site is one thing a repair edits. The ratio between them is not 1 and is not constant.
- **Never sum per-module figures.** Module closures overlap almost completely, so a total added
  across boards counts one emitted defect once per door it is visible through. Measured here, that
  sum overstates by **7.0x** on the shim-free view — and the multiplier is a property of how many
  entries someone happened to board, so it rises with no change to the tree.
- **Never extrapolate one module across sixteen.** `00_compile` boards 189 distinct sites and not
  one of them is outside `03_ingest`'s set; `discovery_enumeration` boards two. Neither is a
  sixteenth of anything.

The falsifying example is two rows of this board's own weighting, and it is the whole argument:

| root | distinct sites | manifestations | diagnostics per site |
|---|---:|---:|---:|
| `PRIMITIVE_REPR_FORK` | **4** | 104 | 26.0 |
| `ABSENT_CLONE_BOUND` | **22** | 129 | 5.9 |

Ranked by diagnostics these two are comparable and would be staffed comparably. Ranked by what a
repair actually edits they are five-fold apart, and the smaller one is the one to take first. So
the inflation above is not a constant anyone can divide out afterwards: **it varies per root, and
it varies in the direction that reverses the ranking.**

**Scope of the rule.** It is a *planning* rule — it decides which root you pick and how you rank
roots against each other. It does not reach a lane already on one traced root verifying with a
two-arm before/after measurement, because such a measurement is unit-agnostic: whatever it counts,
it counts the same way on both arms. Re-quoting a repair already past planning in distinct sites
would change no decision it is still making. The exception is a lane whose acceptance test is an
exact count, where the unit still decides something (`eager-lark-892`'s is one).

## Subject, ref, provenance

| | |
|---|---|
| roster authority | `tools.self_host_module_behavioral_transport_roster` — **16** `module_path` rows, **4** with a non-empty `shim_lib_rel` |
| boarded | **15** (the 16th, `program_assembly`, refused before emitting — see below) |
| ref | `6a59c1a549bee1ba01e59e83d30d1a0d88c5b9b3` |
| ref contents | `origin/main` `1ed02057a5fac683893afcbb427fa8933cc0f2a4` (which carries #9027) merged into the session branch; `git diff --stat 1ed02057a5..6a59c1a549` is exactly three files — `.gitignore`, and this probe's two instruments. None is a `.dag` or `.rs` source and none is reachable by the compiler |
| instrument | `docs/probes/curated_cargo_board_cohort.sh` → `curated_cargo_probe_one.sh`, `CSSL_STD_SEED_LINK=1` |
| join | `docs/probes/cross_module_mechanism_weighting.py` |
| dispatch | four parallel `ctrl-build --remote` runs; every board's own row carries the ref in column 9, and the join **refuses** unless all fifteen agree with the ref declared on its command line |

**Enumeration is from the authority.** The module list and each row's `shim_lib_rel` are read out
of the roster by the cohort runner, not from a filename glob, a grep on a field name, or a hand
list — each of which produced a wrong-but-plausible population in this fleet inside one evening.
`shim_lib_rel` is not defaultable: 12 rows are empty and 4 are not, so guessing empty is right 12
times and silently wrong 4, which is worse than being uniformly wrong.

**Instrument continuity, measured rather than assumed.** The reference entry re-boarded here
gives **177 files emitted / 503 diagnostics / 0 blocking**, identical to the certified
`98b18cdc81e` board. The emit side of the instrument therefore did not move across #8919's
annotation channel, #9027, or this merge — so a per-module difference below is a property of the
module, not of a compiler change in between.

**The rustc side did move, and it is not this lane's doing.** At the same entry: coded rows 305
here against 316 at `98b18cdc81e`, E0308 123 against 128, E0277 18 against 23,
`CARGO_ERROR_TOTAL` 319 against 330. Any repair sizing quoted from the 2026-08-23 board is
already stale by roughly a dozen diagnostics.

## The unit, and why nothing here is a sum across modules

Module closures overlap almost completely: `src/v2_std_algebra.rs` and its neighbours are emitted
into essentially every crate. So **one emitted defect is counted once per door it is visible
through**, and a total summed across boards rises when you board more entries with nothing
changed in the tree. Three quantities, and the first is never added across modules:

- **manifestations** — the existing per-board unit (`rustc_mechanism_classify`'s grain).
- **distinct sites** — manifestations deduplicated by site identity across the whole cohort.
- **breadth** — how many boards a distinct site appears on.

Site identity is published at two strictnesses because only one of them can be wrong quietly.
Strict keys on `(file, line, col, code, type pair)`; loose drops line and col and keys on the
message instead. Strict assumes an emitted file has the same bytes in every closure — the natural
assumption, and exactly the one that is unsafe, since a closure-dependent emit decision (the
primitive-representation switch is one that provably exists) shifts line numbers without changing
the defect. **Measured here: 598 strict against 298 loose.** That two-to-one gap is the reading:
emitted line numbers are *not* stable across closures, so the loose figure is the one to use and
the strict figure is published only to show why.

## The board

```
15 boards · 1709 manifestations summed · 298 distinct sites (loose) · 205 shared (68.8%)
overcount from summing across boards: 5.7x
```

| module | shim | emitted | advisory | cargo total | manifestations | distinct sites | unique to it | not visible from 03_ingest |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `00_compile` | — | 175 | 503 | 318 | 329 | 189 | 0 | 0 |
| `01_tokenize` | — | 26 | 31 | 18 | 19 | 11 | 0 | 0 |
| `02_parse` | — | 100 | 188 | 134 | 141 | 82 | 0 | 0 |
| `03_body_producer` | yes | 16 | 22 | 16 | 17 | 13 | 1 | 10 |
| `03_ingest` | — | 177 | 503 | 319 | 330 | 190 | 1 | 0 |
| `03_normalize` | yes | 80 | 163 | 195 | 198 | 89 | 72 | 83 |
| `03_resolve` | — | 81 | 163 | 89 | 96 | 58 | 0 | 0 |
| `04_infer` | — | 86 | 179 | 98 | 106 | 66 | 0 | 2 |
| `07_target_carriers` | — | 68 | 136 | 79 | 82 | 46 | 9 | 9 |
| `discovery_enumeration` | — | 15 | 22 | 2 | 2 | 2 | 1 | 1 |
| `materialization_carriers` | — | 53 | 146 | 25 | 20 | 15 | 0 | 0 |
| `parse_engine_hooks` | yes | 14 | 22 | 10 | 10 | 7 | 0 | 6 |
| `program_partition` | — | 99 | 179 | 121 | 131 | 77 | 5 | 7 |
| `source_authority` | — | 130 | 244 | 185 | 195 | 113 | 0 | 0 |
| `use_site_verdict` | yes | 20 | 22 | 32 | 33 | 22 | 4 | 18 |

## The weighting

Ranked by distinct sites, never by summed manifestations. `SHIM_CLOSURE_GAP_instrument` is
listed **and is not a root** — see the quarantine below.

| root | distinct sites | shared | max breadth | manifestations | codes |
|---|---:|---:|---:|---:|---|
| UNCLASSIFIED | 217 | 151 | 15 | 1259 | 20 codes |
| `SHIM_CLOSURE_GAP_instrument` | 37 | 12 | 4 | 97 | E0432, E0433, E0608 |
| `ABSENT_CLONE_BOUND` | 22 | 20 | 13 | 129 | E0277, E0308, E0599 |
| `MAP_CARRIER_FORK` | 18 | 18 | 9 | 120 | E0308, E0560 |
| `PRIMITIVE_REPR_FORK` | 4 | 4 | 8 | 104 | E0308, E0369 |

**The ratio between the last two columns is the finding a per-module board cannot produce.**
`PRIMITIVE_REPR_FORK` is **4 distinct sites** carrying **104 manifestations** — 26 diagnostics per
site. `ABSENT_CLONE_BOUND` is 22 sites carrying 129. Ranked by diagnostics the two look
comparable; ranked by the thing a repair actually edits they differ by a factor of five. A root
size measured in diagnostics is an upper bound on defects with an unmeasured fan-out, and here
the fan-out is measured: it is not 1 and it is not constant.

Two cross-code roots are established by this cohort and did not exist as roots before it, because
each is only visible from a join:

- **`PRIMITIVE_REPR_FORK`** — the `rust_corpus_repr` HostNative/FaithfulFreeMonoid switch DESIGN
  already carries. rustc reports the same emitted decision as a type mismatch (E0308) where a
  value crosses the seam and as an unsupported operator (E0369) where an operator is applied to
  the modeled carrier. A per-code partition ranks it twice at part size.
- **`MAP_CARRIER_FORK`** — one map concept realized two ways, modeled `PartialFunction` against
  `HashMap`. It reaches rustc as a mismatch (E0308) and as a missing struct field (E0560) where
  the emitter writes a record literal whose field names belong to the other realization. The
  E0560 half is the load-bearing part: read per code it looks like an unrelated
  "struct has no field" family that no E0308 repair would close.

`ABSENT_CLONE_BOUND` keeps the discriminator `rustc_mechanism_classify` established (rustc's own
explanation text, not a type shape), so nothing here re-labels a row it claims.

**What is deliberately not classified.** E0004 non-exhaustive patterns, E0425 unresolved names,
and the `K: Hash` / `K: Eq` bound family are each large and each plausibly one root, and none has
a discriminator this cohort establishes. They stay UNCLASSIFIED. Filling them in from their error
code is the move the whole instrument exists to refuse, and 217 of 298 sites staying unclassified
is the honest state rather than a gap in the write-up.

### The most-shared sites

| breadth | code | emitted file | rustc says | root |
|---:|---|---|---|---|
| 15 | E0599 | `src/v2_std_node.rs` | the method `partial_cmp` exists for reference `&T`, but its trait bounds were not satisfied | UNCLASSIFIED |
| 13 | E0308 | `src/v2_std_algebra.rs` | mismatched types `T` vs `&T` | ABSENT_CLONE_BOUND |
| 13 | E0308 | `src/v2_std_algebra.rs` | mismatched types `_` vs `&_` | ABSENT_CLONE_BOUND |
| 11 | E0277 | `src/v2_std_collection.rs` | the trait bound `K: std::hash::Hash` is not satisfied | UNCLASSIFIED |
| 9 | E0425 | `src/v2_std_text.rs` | cannot find type `Int` in this scope | UNCLASSIFIED |
| 9 | E0308 | `src/v2_std_text.rs` | mismatched types `i64` vs `Box<i64>` | UNCLASSIFIED |
| 9 | E0560 | `src/v2_std_collection.rs` | struct `Rc<PartialFunction<_, _>>` has no field named `lookup` | MAP_CARRIER_FORK |
| 9 | E0277 | `src/v2_std_collection.rs` | the trait bound `K: Eq` is not satisfied | UNCLASSIFIED |
| 9 | E0308 | `src/v2_std_collection.rs` | mismatched types `HashMap<K, V>` vs `Rc<PartialFunction<_, _>>` | MAP_CARRIER_FORK |
| 9 | E0308 | `src/v2_std_compilers_lexing.rs` | mismatched types `Vector<Rc<Token>>` vs `Rc<Vector<Rc<Token>>>` | UNCLASSIFIED |
| 9 | E0004 | `src/v2_std_integer.rs` | non-exhaustive patterns: `&Some(_)` not covered | UNCLASSIFIED |
| 8 | E0308 | `src/v2_std_passing_candidate_fold.rs` | mismatched types `T` vs `&T` | ABSENT_CLONE_BOUND |


### Breadth distribution

| breadth | distinct sites |
|---:|---:|
| 1 | 93 |
| 2 | 86 |
| 3 | 9 |
| 4 | 33 |
| 5 | 19 |
| 6 | 8 |
| 7 | 14 |
| 8 | 25 |
| 9 | 7 |
| 11 | 1 |
| 13 | 2 |
| 15 | 1 |

## The quarantine: four boards carry a lane shim, and it shows

`03_body_producer`, `03_normalize`, `parse_engine_hooks` and `use_site_verdict` are boarded with
their roster-declared `shim_lib_rel`. That shim's hand-written `lib.rs` **replaces** the assembled
one and declares only the modules its own lane needed, so every other module of the closure
refuses as an unresolved crate-root path. `curated_cargo_probe_one.sh` names this the FALSE-RED
hazard; this cohort measures it: **all 60 E0432 and all 16 E0608 manifestations sit on those four
boards**, along with 41 of 57 E0433. It is attributed as `SHIM_CLOSURE_GAP_instrument` rather than
left UNCLASSIFIED because the two are different claims — UNCLASSIFIED says no root is established,
this says the population is a property of how the board was taken.

**Consequence for the headline, stated before the headline is used.** The cohort splits:

| view | boards | manifestations | distinct sites | shared | overcount | visible from `03_ingest` | invisible |
|---|---:|---:|---:|---:|---:|---:|---:|
| all boards | 15 | 1709 | 298 | 68.8% | 5.7x | 190 (63.8%) | 108 |
| shim-free only | 11 | 1451 | 207 | **92.3%** | **7.0x** | 190 (**91.8%**) | **17** |

The four shim boards contribute 91 of the 108 sites that `03_ingest` cannot see, and 37 of those
are instrument-attributable outright. **The remainder of their unique population is not a finding
until those modules are re-boarded shim-free** — it is not established to be a module property,
and this document does not claim it is.

## What this says about a queue ranked on a sample of one

The lane's premise was that a repair queue traced from one entry might be ranked on an
unrepresentative sample. Measured, the answer has two halves and they point opposite ways.

**The ranking is largely sound.** Across the eleven shim-free boards, **92.3% of distinct sites
appear on two or more**, and boarding ten additional modules added only **17 distinct sites** that
`03_ingest` could not already see — 8.2% of that view's population. `00_compile` boards 189
distinct sites and **none** of them is outside `03_ingest`'s set; `03_ingest` itself has exactly
one site no other board sees. There is a shared floor, it is most of the volume, and it is what
the queue has been working. This confirms the standing H1 hypothesis in
[`self-host-cargo-refusal-root-partition.md`](../../plans/self-host-cargo-refusal-root-partition.md)
by the exact falsifier that document named — real error texts, more than two modules, intersection
measured rather than inferred from histogram similarity.

**The sizes are not.** Summing per-module figures overstates by **7.0x** on the shim-free view.
Any root sized by adding boards together, or by extrapolating one module's count to a roster of
sixteen, is inflated by most of an order of magnitude. Per-root fan-out varies by a factor of five
on top of that, so the inflation is not even uniform across roots.

**Where a genuinely new root would come from.** The seventeen shim-free sites invisible from
`03_ingest` sit almost entirely on `07_target_carriers` (9) and `program_partition` (7), and every
one of them is currently UNCLASSIFIED. That is the only place in this cohort where a root nobody
has seen could still be hiding, and it is small.

## `program_assembly` — the sixteenth row, and why it is absent rather than zero

`gunbc compile` refuses on this entry before emitting: `emit_fail`, `EMIT_REFUSE`, no cargo log,
`CARGO_ERROR_TOTAL` `n/a` (deliberately not `0`, which would read as a clean build to anything
summing the column). Its closure resolves 78 sources against `03_ingest`'s 76, so it is not an
enumeration failure.

The absence is **declared into the join**, not filtered out of it: the population check refuses
unless every board that produced a row also produced a log, and `--declared-absent` requires a
reason string. A cohort that quietly boards 15 of 16 and reports a weighting over "the frontier"
is the empty-observation narrow DESIGN.md names — ⊥-as-answer conflated with ⊥-as-ignorance.

**The reason, measured.** A separate single-entry dispatch — markers authored and read rather
than the harness's exit status: `BUILD_EXIT=0`, `RUN_EXIT=1`, `EMITTED=0` — shows the compile
reaching `compile.analyses` and then producing **26 hard diagnostics**, all name resolution in the
module's own source:

```
unresolved type 'DagSourceReadWitness'      (program_assembly.dag:540-560, and 3 more sites)
unresolved type 'PreparedGrammar'           (:595-610, :1881-1896)
unresolved type 'SourceRootIngest'          (:4539-4555, :5425-5441, :6156-6172)
unresolved type 'Admission'                 (:5844-5853, :6187-6196)
no field 'artifact' / 'source' / 'compilation_unit' / 'source_root' on 'DagSourceReadWitness'
function 'tokenize' / 'parse_module_prepared' / 'normalize' / 'prepare_grammar' not found in scope
function 'source_authority_diagnostic' not found in scope
no field 'tree' on type 'T'
```

This is a **per-module blocker in the `.dag` source, not a harness artifact and not an emitter
defect**: the entry never reaches emission, so it contributes no rustc population at all and its
absence from the weighting removes nothing from the root ranking. It is also the one board that
would be cheap to unblock — the failures are located, typed, and confined to one file's names.

That measurement was taken at `617201405d00379e726933e2bcd2bb5c9fa100a5`, one commit past the
board ref; `git diff --stat 6a59c1a549..617201405d` is a single file,
`docs/probes/cross_module_mechanism_weighting.py`, which the compiler does not read. The ref is
named rather than rounded to "HEAD" because a different ref is a different subject even when the
difference cannot matter.

## Reproduce, without a rebuild

The fifteen cargo logs are committed under `logs/` with their sha256s, exactly so this population
can be re-partitioned by a future classifier without re-running the sweep — the property whose
absence blocked a cross-code classifier at an earlier ref.

```sh
gunzip -k docs/probes/frontier_cohort_board_2026-08-24/logs/*.gz   # into a scratch dir
python3 docs/probes/cross_module_mechanism_weighting.py <log-dir> <out-prefix> \
  --rows docs/probes/frontier_cohort_board_2026-08-24/probe_rows.tsv \
  --expect-sha 6a59c1a549bee1ba01e59e83d30d1a0d88c5b9b3 \
  --declared-absent "program_assembly=EMIT_REFUSE: refused before emitting" \
  --shim-board 03_body_producer --shim-board 03_normalize \
  --shim-board parse_engine_hooks --shim-board use_site_verdict
```

**One caveat on the second command below.** `curated_cargo_board_cohort.sh` is a hand-authored
shell runner and therefore a DESIGN §6 scaffold awaiting an operator verdict (raised by
review 55258 on gunbc#9064; its own header carries the corrected classification and the
dissolution trigger). If that verdict refuses it, the file is deleted and nothing in this board
changes: the fifteen logs, their sha256s, the probe rows and the join are all committed here, and
`curated_cargo_probe_one.sh` — already in tree — takes a board on its own. What deletion costs is
the mechanised roster read, i.e. the defence against the denominator failure described above.

To take the boards again at a new ref:

```sh
PROBE_KEEP_LOG_DIR=<dir> docs/probes/curated_cargo_board_cohort.sh   # no args: the whole roster
```

## Scope

Fifteen boards at one ref with one instrument. Nothing here measures a delta, and no board was
taken twice: the probe's binary stamp keys on `git rev-parse HEAD` alone, which is a complete key
at a clean single-arm ref and is not one for a two-arm dispatch from a single branch.
