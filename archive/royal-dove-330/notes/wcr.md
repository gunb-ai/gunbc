## walk-cost model record: read-only receipts, nothing executed

Owner: royal-dove-330 (holder of node adhoc-189dd15f), assigned by silent-bee-845 after silent-koi-402 was archived with no successor. This comment is the durable record; the scratch copy was in ephemeral /tmp.

METHOD AND PROVENANCE, stated once. Everything below comes from retained GitHub Actions logs (`gh run view --log`, read-only), the Actions cache API (`gh api .../actions/cache`, one lookup at 2026-11-07T12:48:41-43Z, repo-wide, `--paginate`, 667 entries, no ref or key-prefix filter, retention-limited so eviction is invisible), committed `.github/workflows/witnesses.yml`, `git`/`grep`, and static reading of the Rust and `.dag` sources. NOTHING was compiled, claimed, tested, regened, run, dispatched, pushed or landed by this session; every RED/GREEN or timing statement here is a measurement of a past CI run or a prediction, never an execution by me.

### 1. The publication-side loop, confirmed at key level

Parse-verdict receipts and parse-compiler identity blobs are published ONLY by the main-push route's save steps ("Save parse-verdict receipts to Actions cache", "Save parse-compiler identity blob"), which sit AFTER the push walk in the job. A run cancelled during its walk therefore publishes nothing, and the next PR walk falls back to an older ancestor.

Key-level state (lookup 12:48:41-43Z):
- `49d4bc36b6bbdf29b539c6b38c37ddbb922633e0` is the last publisher: BOTH artefacts at 11:03:53Z, run 50438 success - receipts `gunbc-materialization-parse-verdict-a5c4d44bebe7a2362cc905df8ecaabb827b2550671a038124ceb6344eea94d1f-49d4bc36...` 102,890 B and identity `gunbc-parse-identity-49d4bc36...` 241 B; re-hit at 12:43:12Z.
- `0cfb85d8`, `f793a798`: 0 keys of either family; their runs (50445, 50451) were CANCELLED, so the save steps were skipped - the cause is on the record for these two.
- `ed007978`, `a7de1afe`, `cce400af`, `2340e0af`, `8d22701a`, `bef04740`: 0 keys of either family, but runs 50453/50459/50460/50461/50462/50470 were not yet concluded at lookup time, so for these the absence means only "no save yet", not a cause.
- All 12 main commits merged during the 120-minute configuration window DO have keys. The gap begins after the revert to the 90-minute budget.
- Distance is currently 8 of the 12-candidate window (`49d4bc36` sits at ordinal 8 of bound 11). The window is NOT exhausted: eventual PR failure is a FORECAST, not a receipt. Mismatch or eviction can miss independently of distance.
- Wording discipline kept: "loop re-arming" is an INFERENCE pending causal joins, not an established mechanism.

### 2. The identity-era failure mode - independent of distance and of publication

Run 42339827972 (PR, the one that PASSED in 89m17s) still paid a full re-parse:
- 05:55:15.527Z `Cache hit for: gunbc-parse-identity-fa354186e431...` - the ancestor WAS published and its identity blob WAS restored (that entry was created 2026-11-06T17:14:14Z).
- 05:55:16.154Z `Cache not found for input keys: gunbc-materialization-parse-verdict-a5c4d44bebe7...-fa354186e431...` - the receipts under the HEAD's identity for that ancestor were absent.
- 06:18:02.969Z `parse-verdict reuse: ancestor=fa354186... reused=0 miss_identity_absent=0 miss_identity_mismatch=10094 miss_entry_absent=0 miss_digest_unseen=0 miss_declared_refused=0` after 22m46s.

The counters are ORDERED, and that is the finding: with the restored ancestor's parse-compiler identity differing from the head's, every member is charged `miss_identity_mismatch` while `miss_entry_absent` stays 0 even though the receipts file was not restored. The identity comparison short-circuits before the receipts are consulted, so a stale-era publication behaves exactly like an absent one.

Key composition (committed YAML): the receipts key is `gunbc-materialization-parse-verdict-<HEAD's compiler_identity>-<ancestor sha>` (e.g. witnesses.yml:323 and the matching save lines) while standing compares the restored `gunbc-parse-identity-<ancestor>` blob against the head's identity, per member by exact recomputed source digest plus head/base compiler identity (`parse_verdict_standing`, `dag/gunbc/parse_verdict_schema.dag:81-103`; `population_parse_standing`, `src/v2/cli/population_verbs.dag:415-436`). So a walk can only hit an ancestor whose publication carries the head's CURRENT identity. Across the 296 retained receipt entries there are 16 distinct identity eras in about two days: era turnover is ordinary, not exceptional.

CONSEQUENCE: fixing publication alone (a main push that finishes and saves) does not prevent a full 10,094-member re-parse (measured 1,366 s) whenever the tokenize+parse+schema closure has changed since the nearest published ancestor. TWO gates must both hold for a cheap walk: something published, and the published era equal to the head's era. A model with only the first under-predicts full walks on parse-path commits, which is exactly the population that matters for this budget.

### 3. The hit join - the whole mechanism working, end to end

Run 42362849618, v2-bootstrap job 128374654255 (read-only log fetch):
- 11:19:47.398Z `Cache hit for: gunbc-parse-identity-49d4bc36b6bbdf29b539c6b38c37ddbb922633e0`, restored 11:19:47.645Z
- 11:19:48.026Z `Cache hit for: gunbc-materialization-parse-verdict-a5c4d44bebe7a2362cc905df8ecaabb827b2550671a038124ceb6344eea94d1f-49d4bc36...`, restored 11:19:48.270Z
- 11:19:54.346Z `parse-verdict reuse: ancestor=49d4bc36... reused=10101 miss_identity_absent=0 miss_identity_mismatch=0 miss_entry_absent=0 miss_digest_unseen=4 miss_declared_refused=0`

Both artefacts restored, walk about 7 seconds against 10,106 bound modules (8,201 `dag/*.dag` + 1,899 `src/v2/*.dag`). Other observed hits in the retained set: reused 10102/1 miss (6 s), 10102/2 (5 s), 10087/16 (16 s). This is the ceiling for the reuse route, measured.

### 4. Emit-side cost chain, with corrected citations

From the `[pre-entry]` rows and route lines of main run 42292182286:
- **self-host** (seed `src/v2/compiler/source_root_eval.dag`): process_invocation_total 1,651,464 ms = 27m31s. Inside it `compile_transaction` 1,218,660 ms = 20m19s (21:05:44.8 -> 21:26:03.5); `emitted 266 files` at 21:26:04.9; cargo build returned 21:30:44.0 (about 4m39s); the remaining `process_unattributed` 432,804 ms = 7m13s covers door/refusal controls, fault/restore/spawn and the 5-test runtime calibration suite. Closure `8c9d036f6c79...`, binary `8dacbad4...`.
- **v2-native-cli** (seed `src/v2/cli/compile_cli.dag`): process total 1,936,227 ms = 32m16s. `compile_transaction` 1,349,729 ms = 22m30s; `emitted 251 files`; cargo about 4m24s; floor 586,498 ms = 9m46s. Closure `09bd8860...`, binary `2c277246...`.
- So of the 59m47s the two steps cost: about **42m49s is the two in-process compile transactions**, about 9m cargo, about 7m post-build controls. The transactions are the cost centre.

**The measurement boundary.** Each transaction reports `attribution=floor` with `raw_ms` 27,407 and 27,278: the phases that actually register timings sum to about 27 s (`pool_source_newline_index` 493/454, `pool_source_tokenize` 14,295/14,270, `pool_heads_parse` 1,622/1,528, `pool_census_parse` 2,229/2,119, `closure_name_census_build` 710/703, `selection_tier_closure_files` 8,110/8,115, `reference_pool_names_from_census` 16/16 ms; `demand_layout_acquire` inclusive 22,915/22,509 ms), so 1,326,813 and 1,196,151 ms - about 98% - sit outside every recorded phase. Instrumentation is thin by construction: the entire map is 12 call sites (`cli_run/entry_resolve.rs` 6 records + 2 timed, `cli_run.rs` 3 records, `cli_run/pool_acquire.rs` 1, `demand_acquire.rs` 1 timed, `main.rs` 1 timed), and `src/v1/stage0/src/cli_run/pre_entry_phase.rs:217-219` states plainly that the process floor is NOT "everything outside the transaction" and that an inclusive ancestor may hide a slow child. Nothing is broken; the largest cost in the job simply has no phase boundary, so any claim about what those ~21 minutes do is inference, not measurement.

CITATION CORRECTION: an earlier working note of mine carried "compile_telemetry.dag:1712/:1781" as the emitter of these rows. No such file exists (find plus a repo-wide grep return nothing). The real emitter is `src/v1/stage0/src/cli_run/pre_entry_phase.rs:302` and `:309` (the transaction and its floor) and `:368` and `:378` (the process total and its floor), with the boundary documentation at `:201-221`. The numbers themselves are unchanged from the retained log.

**Why the two emit steps share nothing - policy, not oversight.** Each preparation makes a root private to the run (`cli_run/native_lane_runner.rs:417-425`, "No other run can name it, so nothing here needs excluding a peer"), and the workflow sets `GUNBC_NATIVE_PRODUCT_REUSE: fresh` on both steps (`witnesses.yml:251`, `:279`). The code documents what that forgoes: a verified product-store hit "stands in for reconcile + emit + build + the red that admitted the entry", with the caller's door and refusal controls then running against the reused executable (`native_lane_runner.rs:427-430`). Under `fresh` the store is neither consulted nor committed, and if `GUNBC_NATIVE_CACHE_ROOT` binds a store the run REFUSES (`:434-443`: "a restored final product may not stand in for the build this run owes"); under `fresh` not even a product key is derived, because `derive_key` would re-ingest the corpus index and reload the closure just to hash it - "the same work twice (DESIGN section 2)" (`:451-454`). There is no sccache, and the cargo build runs `--offline` per crate directory (`native_lane_runner.rs:649-657`, via `emitted_closure_compile_host::run_cargo`); dependency downloads are cached by the separate "Cache Cargo" step. The two steps are genuinely different subjects (266 vs 251 files, closures `8c9d036f` vs `09bd8860`), so they are not literally the same work; the overlap is the shared compiler closure, and the probe-root privacy that keeps them apart is the same isolation the gate depends on.

Recording, NOT recommending: turning on the existing verified-hit path in CI is the only known way to remove the ~43 minutes of compile transactions, and it changes what the gate proves. That is a semantics decision for the owners, and proposing a cache to hide that cost is what DESIGN 2 forbids. The consequence stands either way: the runtime headroom under 90 minutes has to come from the parse-population side unless that ruling is revisited.

### 5. What follows for the budget, and what is still unexecuted

Change 1 (the reuse route, source READY at commit `e3ba345342f0046711b904adc71ab2964de3a3d2`, tree `8dedb58c302b982654b5c6f4fbd0acdb31dd879a`, base `ed007978dc2a`) removes the route asymmetry so a main-push walk can reuse a published ancestor at all - today the CLI refused reuse together with publish, which is why the push route always paid a full walk. It changes no standing rule and drops no test; population semantics are preserved.

STILL UNEXECUTED AND WITHHELD (unchanged, do not read readiness as evidence): the claims have never run; real-producer inhabitance (lookup, CLI seam, published artefact) is OPEN; same-subject cold-versus-reused artefact equality is OPEN; `witnesses.yml` is ungenerated and its stamp must be regenerated before any LAND; generation, push and LAND are withheld by keen-otter. The honest next evidence for the emit side is a single instrumented run that registers boundaries around the compile-transaction body, so the ~21 unattributed minutes become attributable - that is a workload and needs its own authorization.
