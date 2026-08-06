# Frontier probe runs interrupted — exact-head survey did not complete (2026-08-06)

**Status:** EXECUTED RECEIPT OF AN INTERRUPTED RUN. This is not a frontier measurement and contains no blocker-class counts. The authoritative 27/27 survey **did not run**: every probe invocation was SIGKILLed before emitting a receipt for any module.

**Do not read any frontier count out of this document.** The only numbers here are about the instrument, not about the compiler frontier.

**Do not read a probe memory requirement out of it either.** The cause of the kills is **not attributable from inside this container**, and this receipt was renamed from `frontier_probe_memory_refusal_2026-08-06` precisely because that name asserted a memory-refusal conclusion the evidence does not support. Probe memory cost is **unmeasured**, not high.

## What was attempted

The Lane 1 exact-head closeout recipe (`gunbc.ci_layer_roots` `compiler_frontier_per_module_probe_exclusion_note`), rehearsed end to end. Predecessors #7776, #7762 and #7767 are merged on main.

- Subject head (rehearsal only, **not** a pin): `2be88489b1303357a10b135dd90704103ea93b0e`.
- Build: clean detached worktree, `cargo build --release -p v1-compiler --bin frontier_probe_survey`.
- Provenance: `build.rs` stamps `BUILD_COMMIT` / `BUILD_TREE` / `BUILD_DIRTY`; `verify_build_provenance` refuses stale binary, commit mismatch, tree mismatch and dirty-worktree builds. All gates passed — the binary is genuinely from that tree.

## The refusal

Every probe invocation was SIGKILLed by an OOM killer. Host `memory.max` = 33578549248 bytes (31.3 GiB / 32023 MiB) — but see the counter analysis below: **this cgroup's cap was never the binding constraint.**

| # | Invocation | Source roots | Result |
|---|------------|--------------|--------|
| 1 | one-shot, all 27 modules | `src/v2` + `dag` | killed **during module 1 of 27**, no manifest emitted |
| 2 | `03_normalize`, fresh process | `src/v2` + `dag` | exit 137 |
| 3 | `03_resolve`, fresh process | `src/v2` + `dag` | exit 137 |
| 4 | `01_tokenize`, fresh process | `src/v2` + `dag` | exit 137 |
| 5 | `03_normalize`, fresh process | `src/v2` only | exit 137 |
| 6 | `01_tokenize` retry, low host load | `src/v2` + `dag` | exit 137 — survived to 3.5 min at ~2.0 GB, past every earlier run, then killed anyway without raising `memory.peak` |

Six for six. Zero receipts produced.

**Located identically in all five:** death occurs immediately after the `probing <module>` line, inside the interpreted `frontier_probe_emit_from_ingest` evaluation. The ingest closure is read to completion first (the `[file] read …` lines), so this is not an ingest or discovery failure.

### Instrument check — and the misattribution it caught

Exit 137 alone does not establish an OOM — it is also the shape of a harness timeout or an external kill (see the repo's standing `exit 137 != red` caution). cgroup `memory.events` `oom_kill` incremented **1 → 2 → 3 → 4 → 5**, one per killed run, in exact lockstep, which establishes that these were genuine kernel kills of processes in this cgroup.

**It does not establish what killed them, and an earlier revision of this receipt drew exactly that wrong conclusion.** The full counter set:

```
memory.max     33578549248   (31.3 GiB)
memory.peak     9136902144   ( 8.5 GiB)  <- cgroup high-water across ALL runs
memory.events   max 0   oom 0   oom_kill 8
```

- `max 0` — allocation in this cgroup **never** reached `memory.max`.
- `oom 0` — the **cgroup** OOM killer was **never invoked**.
- `oom_kill 8` — eight processes in this cgroup were nonetheless killed. In cgroup v2 this counter also counts tasks in the cgroup reaped by the **global** OOM killer.
- `memory.peak` **8.5 GiB against a 31.3 GiB cap** — the cgroup never exceeded 27% of its limit at any point tonight.

**Conclusion: these kills came from host-level memory pressure on a shared machine, not from this cgroup's limit and not from the probe's own working set.** The probe was killed at or below 8.5 GiB cgroup-wide, which is *not* the signature of a runaway.

Two claims an earlier revision of this receipt asserted are therefore **withdrawn as refuted**:

1. ~~"the probe requires more than 31 GiB"~~ — refuted by `memory.peak` = 8.5 GiB.
2. ~~"this host cannot cold-build `v1-compiler` at all"~~ — refuted by an independent control on the same fleet under a **byte-identical** `memory.max` of 33578549248: a genuine cold build (sccache cache hits 0, 1 compilation, 163 s of rustc) completed in 2m59s at a **3.26 GiB** peak with `oom_kill 0`. The crate needs ~3.3 GiB, not 31.

The `oom_kill` counter incrementing in lockstep is a sound control for *"was this a kill?"* and a worthless one for *"what hit the limit?"*. Reading the second off the first is the mistake this section now records. `memory.events max` / `oom` and `memory.peak` are the fields that answer it, and they were available the entire time.

## Two hypotheses raised and refuted by execution

Both were plausible, both were wrong, and both were killed by a control before being reported as causes.

**H1 — "the 10 never-measured modules are the ones that exhaust memory."** The interim TSV holds exactly 17 rows, and the 10 absent ones are sweep-order positions **1–9 and 11** — a near-prefix, with only `source_authority` (position 10) interrupting it. That clustering made a module-specific property look likely. **Refuted:** `01_tokenize` is one of the 17 that *did* measure, and it OOMs now (run 4).

**H2 — "the widened root set (`src/v2` + `dag`) is the cause."** The interim audit probed with `src/v2` only; the authoritative closeout mandates both roots, which genuinely widens the closure. **Refuted:** `03_normalize` OOMs with `src/v2` alone (run 5).

The cause is therefore *not* module identity and *not* the root set. It remains open.

## Open: regression vs host capacity

The decisive experiment is to build the probe at the interim audit head `9f978aa8df` — where 17 modules measured cleanly — and run it on this host. Success would mean a regression landed since; failure would mean the host cap is simply too small for this probe.

That experiment did not complete here. Three attempts to build `v1-compiler` at the old head were all killed (`sccache: Compiler killed by signal 9`):

| Attempt | Configuration | Result |
|---|---|---|
| 1 | release, `CARGO_BUILD_JOBS=15` (default) | killed, `oom_kill` → 6 |
| 2 | release, `CARGO_BUILD_JOBS=1` | killed, `oom_kill` → 7 |
| 3 | debug profile (`opt-level=2` per dev profile) | killed, `oom_kill` → 8 |

**These are victims, not diagnoses.** Per the counter analysis above, none of them hit this cgroup's limit, and an independent same-cap control cold-builds this crate at a 3.26 GiB peak. So the builds died to host-level pressure they did not cause, and they say nothing about the crate's requirements. The tempting inference — one rustc died, therefore one rustc needs the whole cap — is invalid, because a process killed by an external OOM killer tells you about the machine at that moment, not about itself.

Consequence: **regression vs host capacity is UNRESOLVED**, but the reason is contention on a shared host rather than any established capacity limit, and it is retryable rather than blocking. Per operator direction it is reported, not chased, and the probe is not trimmed to fit.

### One candidate, not a diagnosis

#7762 closed the manifest-elision hole — the silent-narrow the survey carrier describes. Before it the probe elided closure rows; after it, it admits the full closure. Closing a narrowing that had been *implicitly* bounding memory would produce this shape, and it has a named precedent in DESIGN: removing the scan-all-keys fail-open exposed 8 real deficits its silence had hidden.

This is a hypothesis with a mechanism, **not** a finding. It has not been tested, and it should be assessed by the owner of that change rather than diagnosed from outside.

## Consequence for the closeout

27/27 **did not complete in this window** on either documented path — the one-shot or the per-module fallback (`docs/probes/run_frontier_probe_survey_per_module.sh`).

**"Unreachable" is not established, and an earlier revision of this receipt wrongly claimed it.** Since no run reached the cgroup limit, nothing here shows the probe cannot fit; the runs were interrupted by external pressure they did not cause. The correct reading is *interrupted and retryable*, not *impossible*.

Any per-module cost figure for the probe remains **unmeasured**. It becomes measurable only when a run completes, or when a kill is attributable to this cgroup — `memory.events` `max` or `oom` incrementing, rather than `oom_kill` alone.

Per operator direction the probe must **not** be trimmed to fit the host; a probe shrunk to fit a box measures a different question. On this evidence there is also nothing to shrink it *to*.

## Recipe defects found while rehearsing

**1 — the mandated detached worktree and the default build path are mutually exclusive.** `ctrl-build` defaults to remote (`CTRL_BUILD_MODE=remote`) in these sessions, and BuildBuddy refuses a detached worktree: `remote config: get base branch and commit: unexpected branch state * (no branch)`. The closeout recipe requires a clean *detached* worktree, so `ctrl-build --local` is **required**, not a preference. Anyone following the recipe as written hits this first.

**2 — WITHDRAWN: the suspected module collision is refuted by execution.** I predicted that the recipe's `--source-root target/frontier-probe-survey`, passed alongside `src/v2`, would refuse: both declare module `v2.test.workflow.host_frontier_probe_survey_manifest`, and the seed carries `duplicate_module_path_across_roots_refuses_loudly` (`v1_compiler.cli_run`).

Two executed runs say otherwise. A duplicate of the manifest module was placed in a third source root and `claim_batch` run over `compiler_frontier_per_module_probe_test.dag`:

| Third root | Location | Result |
|---|---|---|
| `target/collision-test` | inside `target/` | exit 0, witness PASS, zero refusal messages |
| `dup_root_probe` | outside `target/` | exit 0, zero refusal messages |

Duplicates across roots are tolerated on this path, so **the recipe does not collide** and the prediction is withdrawn. The seed wall evidently governs a different index path than the one `claim_batch` takes — worth knowing, but not a defect in this recipe.

**2b — ALSO WITHDRAWN: `target/`-resident source roots are read.** The collision runs could not distinguish "read" from "ignored", because the stub and the duplicate carry identical content and both produce the same outcome either way. That mattered, because the seed skips `target/` beside a `Cargo.toml` as build output (`cargo_target_dir_output_never_enters_the_module_index`); had that applied to an explicitly-passed root, the recipe's emitted manifest would be **silently ignored** and the witnesses would read the stub — worse than a collision, since several of them green **vacuously** on `Empty` receipts (`compiler_frontier_wave2_blocker_partition_totality_holds` did exactly that in both runs above, a green asserting nothing).

Settled by a discriminating probe rather than by reasoning: a syntactically invalid `.dag` placed in a third root under `target/` is loud if the root is indexed and silent if skipped. It was **loud** —

```
for_each_parsed_module_binding: parse error in .../target/broken_root/b.dag: expected item declarat…
exit 101
```

So an explicitly-passed source root under `target/` **is** indexed, the recipe's emitted manifest **will** be read, and the vacuous-green risk does not arise from this mechanism.

**Net: no recipe defect survives except item 1.** Both predictions here were wrong, and both were killed by execution rather than argument. The vacuous-green *property* is nonetheless real and worth remembering independently: `compiler_frontier_wave2_blocker_partition_totality_holds` passes on an empty manifest, so it must never be read as evidence that a survey ran.

## Incidental: 4 hard diagnostics on this head, unrelated to this work

While verifying that this PR's `.dag` edits compile, all three edited modules returned **zero diagnostics in their own files** — but every compile still exits 1, on 4 hard diagnostics in `dag/extdeps/systems/nvidia.dag`:

```
source annotation sits inside a declaration body. Only module-item grain is modeled;
move it above the declaration it describes.  (dag/extdeps/systems/nvidia.dag:4538-4620, :4625-4701, :4706-4788, :4793-4869)
```

The cited offsets exceed the file's 120 lines but fit its 5555 bytes, so they are **byte** offsets. The refusal is exactly what DESIGN §4c specifies: the initial `.dag` realization admits only standalone leading `//` blocks on module-scope declarations, and body-grain forms refuse until separately modeled.

Attribution: that file arrives with SPARK-0 (`2be88489b13`) and is **not touched by this branch** — this diff is four files and none is `nvidia.dag`. The diagnostics also appear when compiling a 26-source closure that does not contain the file, so they surface tree-wide rather than closure-scoped.

Recorded as an observation for routing, not diagnosed here. Whether the compile-clean gate scopes around it has not been investigated.

## Roster provenance finding (independent of the OOM)

Separate from the memory refusal, and not caused by it.

`v2.compiler.self_host.frontier` declares `knowledge_attributed_blocker_class_seed_retained_row` — the constructor whose name honestly marks a row as knowledge-attributed rather than measured. It has **zero call sites**: repo-wide, the only occurrence of that symbol is its own definition.

Meanwhile all **10** roster modules that have never been execution-measured at any head are authored through `execution_measured_seed_retained_row`, the constructor whose name asserts execution measurement — 8 directly, and 2 (`03_normalize`, `03_body_producer`) via `quarantine_seed_retained_row_from_oracle`, which itself delegates to it.

The never-measured 10, derived by joining `compiler_frontier_sweep_order` against the interim TSV's module column:

`03_body_producer`, `03_name_resolve`, `03_normalize`, `03_resolve`, `05_emit_orchestration`, `emit_host`, `emit_module`, `emit_produced`, `emit_semantic_decl`, `program_assembly`

This is DESIGN §4b **rung inflation**: a carrier named for the top rung while occupying the bottom, and "an inflated class never ranks for climbing."

**The mechanism matters more than the count.** `execution_measured_seed_retained_row` takes `measured_blocker`, `located_stage` and `located_reason` as ordinary parameters. It therefore *cannot* distinguish a measured row from an attributed one — the constructor's **name** asserts a provenance the **type** does not carry, so no amount of care at the call site can make the claim checkable. The §5 construction fix is a provenance field on the row, making an attributed row unrepresentable as measured; the honest-constructor-plus-discipline approach has already been tried here and produced zero adoption.

No roster row is edited by this receipt. `frontier.dag` is load-bearing and the change above is a model change, flagged for routing rather than landed.

## What is still true and unaffected

The interim audit at `9f978aa8df` (`frontier_probe_exact_head_survey_2026-08-03`) stands exactly as written: 17 modules, 14 `^parse_grammar_choice_overlap_residue` and 3 `^resolve_module_not_found`, all at `ProbeStageAssemble`, at that head with `src/v2`-only roots. Verified directly from the TSV for this receipt.

It remains **not combinable** with any later head or changed source-root set, and nothing in this document changes or extends it.
