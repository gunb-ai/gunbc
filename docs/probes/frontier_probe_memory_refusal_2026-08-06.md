# Frontier probe memory refusal — exact-head survey blocked (2026-08-06)

**Status:** EXECUTED REFUSAL RECEIPT. This is not a frontier measurement and contains no blocker-class counts. The authoritative 27/27 survey **did not run**: the probe is OOM-killed before it produces a receipt for any module.

**Do not read any frontier count out of this document.** The only numbers here are about the instrument, not about the compiler frontier.

## What was attempted

The Lane 1 exact-head closeout recipe (`gunbc.ci_layer_roots` `compiler_frontier_per_module_probe_exclusion_note`), rehearsed end to end. Predecessors #7776, #7762 and #7767 are merged on main.

- Subject head (rehearsal only, **not** a pin): `2be88489b1303357a10b135dd90704103ea93b0e`.
- Build: clean detached worktree, `cargo build --release -p v1-compiler --bin frontier_probe_survey`.
- Provenance: `build.rs` stamps `BUILD_COMMIT` / `BUILD_TREE` / `BUILD_DIRTY`; `verify_build_provenance` refuses stale binary, commit mismatch, tree mismatch and dirty-worktree builds. All gates passed — the binary is genuinely from that tree.

## The refusal

Every probe invocation was killed by the cgroup OOM killer. Host cap `memory.max` = 33578549248 bytes (31.3 GiB / 32023 MiB).

| # | Invocation | Source roots | Result |
|---|------------|--------------|--------|
| 1 | one-shot, all 27 modules | `src/v2` + `dag` | killed **during module 1 of 27**, no manifest emitted |
| 2 | `03_normalize`, fresh process | `src/v2` + `dag` | exit 137 |
| 3 | `03_resolve`, fresh process | `src/v2` + `dag` | exit 137 |
| 4 | `01_tokenize`, fresh process | `src/v2` + `dag` | exit 137 |
| 5 | `03_normalize`, fresh process | `src/v2` only | exit 137 |

Five for five. Zero receipts produced.

**Located identically in all five:** death occurs immediately after the `probing <module>` line, inside the interpreted `frontier_probe_emit_from_ingest` evaluation. The ingest closure is read to completion first (the `[file] read …` lines), so this is not an ingest or discovery failure.

### Instrument check

Exit 137 alone does not establish an OOM — it is also the shape of a harness timeout or an external kill (see the repo's standing `exit 137 != red` caution). The discriminating control:

- cgroup `memory.events` `oom_kill` incremented **1 → 2 → 3 → 4 → 5**, one per killed run, in exact lockstep. These are genuine kernel kills in the cgroup the probe runs in, not misread exit codes.
- A 15-second RSS sampler observed a peak of only ~2.6 GB. The climb from there to the 31.3 GiB cap happens between samples, so the growth is a **fast runaway**, not a slow accumulation. The low sampled peak is an artifact of the sampling period and must not be reported as the probe's working set.

## Two hypotheses raised and refuted by execution

Both were plausible, both were wrong, and both were killed by a control before being reported as causes.

**H1 — "the 10 never-measured modules are the ones that exhaust memory."** The interim TSV holds exactly 17 rows, and the 10 absent ones are sweep-order positions **1–9 and 11** — a near-prefix, with only `source_authority` (position 10) interrupting it. That clustering made a module-specific property look likely. **Refuted:** `01_tokenize` is one of the 17 that *did* measure, and it OOMs now (run 4).

**H2 — "the widened root set (`src/v2` + `dag`) is the cause."** The interim audit probed with `src/v2` only; the authoritative closeout mandates both roots, which genuinely widens the closure. **Refuted:** `03_normalize` OOMs with `src/v2` alone (run 5).

The cause is therefore *not* module identity and *not* the root set. It remains open.

## Open: regression vs host capacity

The decisive experiment is to build the probe at the interim audit head `9f978aa8df` — where 17 modules measured cleanly — and run it on this host. Success would mean a regression landed since; failure would mean the host cap is simply too small for this probe.

**That experiment could not be executed on this host, and the reason is itself the answer to a different question.** Three attempts to build `v1-compiler` at the old head were all OOM-killed (`sccache: Compiler killed by signal 9`):

| Attempt | Configuration | Result |
|---|---|---|
| 1 | release, `CARGO_BUILD_JOBS=15` (default) | killed, `oom_kill` → 6 |
| 2 | release, `CARGO_BUILD_JOBS=1` | killed, `oom_kill` → 7 |
| 3 | debug profile (`opt-level=2` per dev profile) | killed, `oom_kill` → 8 |

Attempt 2 is the informative one: with `CARGO_BUILD_JOBS=1` there is a **single** rustc process, and it still exceeded the cap compiling the `v1-compiler` lib. So this is not a parallelism problem — **this host cannot cold-build `v1-compiler` at all**. The current-head build succeeded only because sccache served the lib; it never invoked rustc on it.

Consequence: **regression vs host capacity is UNRESOLVED and not resolvable here.** Answering it requires a host that can cold-build the crate, or a warm sccache for the old head's sources. Per operator direction, this is reported rather than chased.

A distinction worth preserving when this is assessed: **15 parallel rustc processes collectively reaching 32 GB is ordinary. One probe process reaching 32 GB alone, for a single module, is not.** The two kills share a signal but not a significance, and only the second is evidence about the probe. Attempt 2 sits in between and is evidence about the crate, not the probe.

### One candidate, not a diagnosis

#7762 closed the manifest-elision hole — the silent-narrow the survey carrier describes. Before it the probe elided closure rows; after it, it admits the full closure. Closing a narrowing that had been *implicitly* bounding memory would produce this shape, and it has a named precedent in DESIGN: removing the scan-all-keys fail-open exposed 8 real deficits its silence had hidden.

This is a hypothesis with a mechanism, **not** a finding. It has not been tested, and it should be assessed by the owner of that change rather than diagnosed from outside.

## Consequence for the closeout

27/27 cannot be produced right now by either documented path — neither the one-shot nor the per-module fallback (`docs/probes/run_frontier_probe_survey_per_module.sh`), since a single module in a fresh process exhausts the cap on its own. Fanning the 27 modules across workers does not address this: the failure is per-module, not per-batch.

Per operator direction, the probe must **not** be trimmed to fit the host. A probe shrunk to fit a box measures a different question.

## Recipe defects found while rehearsing

**1 — the mandated detached worktree and the default build path are mutually exclusive.** `ctrl-build` defaults to remote (`CTRL_BUILD_MODE=remote`) in these sessions, and BuildBuddy refuses a detached worktree: `remote config: get base branch and commit: unexpected branch state * (no branch)`. The closeout recipe requires a clean *detached* worktree, so `ctrl-build --local` is **required**, not a preference. Anyone following the recipe as written hits this first.

**2 — suspected module collision in the documented `claim_batch` step.** The recipe passes `--source-root target/frontier-probe-survey` alongside `--source-root src/v2`, but `src/v2/test/claim/workflow/host_frontier_probe_survey_manifest.dag` already declares module `v2.test.workflow.host_frontier_probe_survey_manifest`, and the emitted manifest declares the same module. The seed carries an explicit wall (`duplicate_module_path_across_roots_refuses_loudly` in `v1_compiler.cli_run`). `v2.workflow.frontier_probe_survey_transport` `survey_manifest_emit_args` composes the same three roots via `gunbc.ci_layer_roots` `witness_layer_roots`.

Marked **suspected**, not confirmed: the first attempt to execute it hit an unrelated prior refusal (source roots must sit under the compiled-in workspace root) and the retry was abandoned to avoid contending with a running probe for memory.

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
