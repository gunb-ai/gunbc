DESIGN NOTE — v2-bootstrap 90-minute budget (node adhoc-189dd15f-1a8). Read-only: source + existing CI logs/API. Nothing was built, claimed, regened or run by me; no push. All execution evidence is existing CI receipts. Static analysis was git/grep plus a text-only import-closure resolver over .dag files (no compilation).

== 1. THE COST CHAIN (measured) ==
Authoritative per-step durations, 7 completed main pushes today (gh run view --json jobs), plus the four runs in my brief:

step                    fast mode   slow mode
Build the compiler         226-237s    363-411s
Compile v1 unit-test harness 186-190s  315-330s
emit+build self-host      1085-1231s  1755-1933s
emit+build v2-native-cli  1191-1274s  1774-1983s
Walk parse-population      913-1027s  1323-1462s
Save parse-* (x2)              0-1s       0-1s

The job is BIMODAL, not uniformly overspent. Fast mean = 63.0 min (n=2), slow mean = 98.3 min (n=5). Every independent step is 1.42-1.70x slower in the slow mode — a uniform factor across unrelated steps, so it is fleet speed, not per-step work. runs-on is [self-hosted, linux, arm64]; at 11:20 today 13 PR witnesses runs were in_progress at once. Contention is the likely cause (untested here).

INSIDE each emit+build step (log-measured in BOTH 42339827972 and 42292182286):
compile.frontend ~25s, normalize ~1s, **reconcile 7-8 min**, analyses ~1s, **emit 4 min**, then typechecks (target_model 66s, semantic_decl_emission 4 min), then cargo build. Both steps run every one of those stages a SECOND time, and both emit from the same input: same source_tree hash (1e27f7c.../aace5b42...), same "emitting src/v2/cli/compile_cli.dag", 266 files vs 251 files.
No sharing between them, all deliberate: GUNBC_NATIVE_PRODUCT_REUSE=fresh on both (the seed refuses a restored hit under it — native_lane_runner.rs:77-88), RUSTC_WRAPPER="" and RUSTC_WORKSPACE_WRAPPER="" (sccache is zeroed on purpose: "an ambient value would make the verdict a fact about the runner"), CARGO_INCREMENTAL=0, and each build runs in its own PrivateProbeRoot whose "per-run cargo target dir has no consumer once the copy is verified" (gen_one_executable_keep_seed_growth.dag). The actions/cache Cargo step covers repo target/ + registry only — never the probe roots.

THE WALK — and this is the finding that reframes the task. Population = every bound module under dag/ and src/v2: 10,106 members (8,201 + 1,899 .dag files on disk). Its cost is essentially BINARY, measured from receipts:
  reuse MISSED  : 1366s (42339827972, reused=0 miss_identity_mismatch=10094); 1381s slow-mode mean
  reuse HIT     :    6s  42362849618  reused=10101 miss_digest_unseen=4
                    6s  42362837474  reused=10102 miss_digest_unseen=1
                    5s  42362378722  reused=10102 miss_digest_unseen=2
                   16s  42361491915  reused=10087 miss_digest_unseen=16
Reused members still get read and content-hashed before standing is consulted (population_verbs.dag:429-436), and that whole 10,106-file read+hash costs ~5s. So parsing is ~99.6% of the walk; acquisition is not a floor worth optimizing.

== 2. THE LOOP — silent-koi's lead, CONFIRMED, with two corrections ==
Verified from source: materialization_publish_if() = event push AND refs/heads/main gates BOTH save steps (parse_verdict_publish.dag:82-95), and compiler_gate_required_steps sequences them AFTER both walks. 42292182286 was cancelled inside its walk; both save steps = skipped. Confirmed from the step API, not inferred.
CORRECTION 1: the brief's receipt for this (run 42341242245) is attempt 2 — a 26-second cancelled workflow_dispatch titled "Dispatch: review units...", which proves nothing about walks. The brief's numbers are real but come from --attempt 1. Anyone re-checking it will see all-steps-skipped unless they pass --attempt 1.
CORRECTION 2: the loop is not currently armed, because main has been publishing today. The nearest published ancestor for the 11-07 05:06 PR was fa354186, dated 11-06 12:36:55Z (~17h stale); ordinals 0-10 had no identity blob. Today, after 49d4bc36 (#16029, the 90 revert) finished and published at 11:03:53Z, PRs restore ancestor 49d4bc36 directly and hit ~100%.
The five main pushes that "succeeded" at 96.7-99.0 min did so ONLY under the disputed 120 config (#16019, 05:29-10:01Z). Under 90 the slow mode cannot finish. Those receipts are evidence FOR the loop, purchased with an unauthorized configuration.

== 3. WHY REUSE WAS DEAD, AND WHY IT WAS RIGHT TO BE ==
For 42339827972 I resolved the identity's live import closure statically (96 .dag modules from tokenize.dag + parse.dag + parse_verdict_schema.dag) and intersected it with the real fa354186..ae4bfdf2 diff: exactly ONE closure file changed, dag/std/algebra.dag. The runtime crate and all three entry files were unchanged. So the 10094-way identity mismatch was a legitimate invalidation, not a bug. I checked the other suspects and closed them: the identity blob and i_head are both the same --compiler value verbatim (compile_cli.dag:1063), and the receipts cache key embeds head-identity x ancestor-sha. The gate is correct and must not be weakened; reusing across it would be reusing unverified receipts.

== 4. EARLIEST UNJUSTIFIED BOUNDARY (6b) ==
Slice: source tree -> per-instrument frontend/normalize/reconcile/analyses/emit -> probe crate -> cargo build -> walk.
The boundary is that the required job asks for the same semantic fact twice and, on the push route, asks for it a third time from scratch:
(a) the push route has NO reuse path at all — all 24 ancestor restore steps are SKIPPED on push (verified 42362202248), the reuse walk step is skipped, and parse_verdict_publish_args() passes no --reuse-from (0 occurrences in that run's log). Cause is PR-only authoring: parse_verdict_restore_if() = materialization_scope_guard(GhaCachePullRequestMergeRefScope) and pull_request_base_sha_expr() = github.event.pull_request.base.sha. The push route therefore always pays 913-1462s to re-derive 10,106 verdicts that an ancestor already holds verified.
(b) the two instruments re-derive the shared prefix (reconcile+emit ~12 min fast / ~19 min slow) over an identical source tree — a shared-state least common ancestor, so DESIGN 2 rung 1 applies: share the first value. Not cache the second: "A cache may discharge unavoidable recurrence; it may never excuse authored duplication."
Nothing upstream of these is unjustified: the identity gate, the population, and the fresh/no-sccache bindings each carry a stated reason.

== 5. SMALLEST CHANGE THAT REMOVES DUPLICATED WORK ==
Change 1 — push-route reuse parity (this is the one; smallest surface, largest measured effect):
  dag/gunbc/parse_verdict_restore.dag: parse_verdict_restore_if() must admit the push scope; pull_request_base_sha_expr() needs a push-side carrier for the ancestor root (github.event.pull_request.base.sha does not exist on push — first-parent chain of github.sha is the analogue); parse_verdict_ordinal_output / the --first-parent locator must read from that carrier.
  dag/gunbc/parse_verdict_publish.dag: parse_verdict_publish_args() gains --reuse-from <restore dir>, and parse_verdict_ancestor_dir needs a push-side definition.
  dag/gunbc/witness/compiler_gate_workflow.dag: compiler_gate_required_steps must order the locate/restore steps before the push walk (they exist, they are only gated off).
Semantics preserved exactly: same identity gate, same source-digest key, same declared-symbol-set refusal, no unverified receipt consumed, population unchanged at 10,106, no new job, no dropped test. It is route parity, not a new cache.
Change 2 (needs its own design + review, do not bundle): share the emit materialization between the two instruments so reconcile/emit/the emitted crate are derived once. I have NOT scoped this; the second step emits a different target (266 vs 251 files, different entry), so only the shared prefix is removable and the probe-root independence may be load-bearing for a witness — that needs checking before anyone proposes it.

== 6. EXPECTED RUNTIME, AND HOW DERIVED ==
From Change 1 alone, walk miss -> walk hit (5-16s, measured), holding every other step at its observed value:
  fast mode: 2771s + ~10s = ~46 min
  slow mode: 4657s + ~10s = ~78 min   -> under 90, worst observed margin ~12 min
Both derived by summing observed step durations from the step API and substituting the measured reuse-hit walk time. This covers the common case (a main push that does not touch the parse closure) and it also closes the publication loop, which is what makes PR runs land at ~78 min too.
IT IS NOT SUFFICIENT for a push that DOES change the parse closure: there the walk still costs 913-1462s, and slow mode gives 4657+1462 = 6119s = ~102 min, still over 90. That residual is why Change 2 is needed — removing ~1150s of duplicated reconcile/emit from slow mode puts even a closure-changing push at ~83 min. That Change-2 estimate is an estimate: it assumes the duplicated stages scale with fleet speed and that the whole second instance is removable, neither of which I have verified.

== 7. EVIDENCE NEEDED TO VALIDATE ==
1. A main push with reuse engaged printing "parse-verdict reuse:" with reused ~ 10,100 and a walk near 5-16s (the push route has never printed this line — it has never reused).
2. The two save steps reaching success on that push, and a subsequent PR restoring THAT sha as ancestor 0 (closing the loop with receipts rather than inference).
3. Per-stage durations for the emit steps on a slow-mode run, to separate duplicated reconcile/emit from cargo build before Change 2 is sized.
4. Fleet load correlated with the fast/slow split (concurrent v2-bootstrap count at step time), to test the contention hypothesis rather than assume it.
5. Observability, prerequisite for 1-3 and cheap: the walk prints NOTHING until it finishes — the 18m26s cut in 42292182286 produced no diagnostic and the cancellation named no cause. That is the exact shape dag/gunbc/recurring_failure_mode/fixture_closure_union_emit_suspected_superlinear_cost.dag already describes, and its own next-rung trigger prescribes the fix: print the members at start and give the phase its own budget.

== 8. COORDINATION ==
No parallel owner exists: the work-item roster has exactly one open item matching v2-bootstrap/parse-population/walk-cost/receipts, and it is mine (adhoc-189dd15f-1a8). No open PR touches the walk. Git ownership of parse_verdict_publish.dag, parse_verdict_restore.dag and the workflow authority is gunbai-bot[bot] only — no human owner to join. The nearest existing cursor is the RFM row above, whose subject is the floor's fixture-closure-union-emit phase, not this walk; I have not duplicated it and I propose no implementation there.
CONFIG note, kept separate from runtime as instructed: 49d4bc36 (#16029, merged 10:01:59Z) restored the required-job ceiling to the operator-approved 90. I planned against 90 throughout. The five 96-99 min "successes" were taken under the disputed 120 and are recorded as observation only.

No implementation, push, run or dispatch was made. Node stays open pending your implementation scope.