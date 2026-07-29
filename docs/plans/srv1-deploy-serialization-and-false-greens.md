# srv1 deploy — the concurrency race, the false greens, and the misattributed refusal

Status: DIAGNOSIS, receipts complete, no fix landed (2026-07-29, session sleek-lynx-322). Origin: operator asked to root-cause a `deploy_dashboard_srv1` failure reporting `ServedSurfaceStale`, then directed that the **false greens and the misattributed errors matter as much as the flake**. Every claim below is backed by a receipt in §9; nothing here is inferred from the shape of the code alone.

One sentence: **`deploy_dashboard_srv1` has no mutual exclusion, so several main-push deploys mutate one srv1 host at once; the loser refuses correctly (red), the winners green whether or not they deployed their own tree, and the refusal's own diagnostic is overwritten by an unrelated rendering complaint.**

## 0. Displaced cost (§6 — the pain this removes)

Measured over the last 25 pushes to main (§9.3):

| | count | what it costs |
|---|---|---|
| red main deploys, all from the race | 3 / 23 (13%) | a red main that has nothing to do with the commit; per `deploy-srv1-job-masked-by-skip-on-red-ci`, per-job faults here stay latent |
| **silent false greens** — overlapping deploys that greened | **8 / 11 overlapping runs** | a deploy reports success without establishing that its tree is live; the live dashboard can be another commit's |
| refusals naming the wrong cause | every refusal carrying a captured diagnostic | the mechanism added *specifically* to locate this failure is defeated at the moment it fires |

The false-green row is the load-bearing one. The flake is loud and self-heals on the next push; the false green is the §5 absorbing-fallback shape one level up — the deploy's *success* signal does not mean what it says, and nothing counts how often it lies.

## 1. The incident, with receipts

The refusal published both fingerprints, and both pin to exact commits. `site_artifact_digest` is `content_hash_atom` = fnv1a64 rendered `{:016x}`; `/ROADMAP.md`'s served body is `expected_roadmap_md()`, byte-equal to the committed file on main. So fnv1a64 over `git show <rev>:ROADMAP.md` across recent main commits identifies both sides (§9.1):

| digest in the refusal | commit | pushed |
|---|---|---|
| `8759d254e4097c7f` — `expected=` | `b61ed50962` "WIP: affected set (#7424)" | 16:44:53 |
| `de356553a2f5fad9` — `observed=` | `ab58f6bca4` "Extract plural SCM compatibility realization shape (#7405)" | 16:45:20 |

Two pushes **27 seconds apart**. Their deploy jobs ran on srv1 simultaneously (§9.2):

```
ab58f6bca4   success   21:30:00 -> 21:33:48
b61ed50962   failure   21:30:12 -> 21:34:42     <- the reported run
```

So the run for `b61ed50962` observed `ab58f6bca4`'s surface. Timeline, reconciled against the unit diagnosis in the refusal (`Active: since 21:32:26; 2min 12s`) to within seconds:

1. both runs rsync into `/opt/gunbc/gunbc`; the last writer is `ab58f6bca4`'s tree
2. `21:32:26` — a `systemctl restart gunbc-roadmap.service` wins; the new process begins loading whatever is on disk
3. `~21:32:38` — the loser's poll starts (bound 120 s, cadence 1 s)
4. `~21:33:3x` — the process finishes load-plus-compile and binds; this is the ~70 straight `curl` exit=7 probes, *not* a crash — the second restart reset the load clock mid-poll
5. `21:33:48` — the winner's poll sees a current surface, converges, job green
6. `21:34:38` — the loser exhausts its bound against a surface that is genuinely not its tree, refuses `ServedSurfaceStale`

The refusal is **correct**. `gunbc serve` binds its graph once at process start (`service_ready_means_serving_this_tree_note`), the unit is a singleton, so exactly one concurrent deploy can be right about the served surface.

## 2. The mechanism

### 2.1 Nothing serializes main-push deploys

`ci.yml`'s only concurrency control is workflow-level:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.run_id }}
  cancel-in-progress: ${{ github.event_name != 'merge_group' }}
```

On a **push**, `github.event.pull_request.number` is empty, so the group falls back to `github.run_id` — unique per run. Every main-push run is therefore alone in its own group. That is correct for the `ci` job (a main build must not be cancelled by the next push) but it means the deploy inherits *no* exclusion. `DeployStage` (`dag/gunbc/ci_spec.dag`) has no concurrency field at all, so today the model cannot express one.

srv1 carries at least three runner slots eligible for this job — observed empirically as three deploys running together at 16:57–16:59 (§9.3).

### 2.2 What is being raced

Every concurrent deploy mutates the same singleton host state, unlocked (`.github/live-deploy-srv1-apply.sh`):

| state | hazard |
|---|---|
| `/etc/gunbc-tree-sync.env` — holds `GUNBC_TREE_SRC=$PWD` | a **global last-writer-wins channel**: run X's `systemctl restart gunbc-tree-sync.service` can rsync run **Y's** checkout, so a run can sync a tree that is not its own while believing otherwise |
| `/opt/gunbc/gunbc` via `rsync -rlpt --delete` | two interleaved `--delete` rsyncs into one destination can leave a **mixed tree that never existed in git** |
| `/opt/gunbc/bin/gunbc` | last writer wins; the running binary and the tree can come from different commits |
| `gunbc-roadmap.service` (one unit) | last restarter decides the served graph for everyone |

The env-file channel means even the *winner* is not guaranteed to have deployed its own tree — it is only the run whose expected digest happened to match what got served.

### 2.3 Arrival order at the deploy stage is already inverted relative to commit order

This is not hypothetical and it is the crux of §6. The deploy is gated on `needs: [ci]`, and `ci` duration varies with the affected set. In the reported incident the **earlier** commit's deploy started **later**:

```
b61ed50962  pushed 16:44:53  ->  deploy started 21:30:12   (earlier commit, later deploy)
ab58f6bca4  pushed 16:45:20  ->  deploy started 21:30:00   (later commit, earlier deploy)
```

Any fix that orders deploys by *arrival* therefore orders them wrongly with respect to *content*.

## 3. Three failure classes, one cause

### 3.1 The flake — a correct refusal on a red main

The loser burns its full 120 s bound and refuses. Nothing is wrong with the refusal: it is typed, located, and true. It is simply attributable to a scheduling defect rather than to the commit under test. Cost: a red main deploy, plus 120 s of poll per occurrence.

### 3.2 The false green — the dangerous half

In the 25-run sample there were 5 overlap events covering 11 runs, but only 3 failed. **The other 8 greened.** They greened because the racing commits' served surfaces happened to be byte-identical: the whole `/ROADMAP.md` block `a8a773489d754237` spans ten consecutive commits, so during that window a stale surface is *indistinguishable* from a current one.

Two distinct gaps compose here:

1. **Coincidence hides the race.** When the fingerprint of tree A equals that of tree B, the readiness contract is satisfied by the wrong tree. The check is honest about what it claims ("the surface I can observe is my surface") but that claim is weaker than the deploy's actual obligation.
2. **The fingerprint covers less than the deploy mutates.** `roadmap_site_healthz_body()` publishes digests for exactly five URL paths. The deploy also installs the whole source tree and the `gunbc` binary, which the dispatch path and the serve closure both depend on. So `green` means *those five bodies match*, never *my tree is what is deployed*.

The §5 reading: the frequency of the underlying fault is **zeroed by construction** whenever the surfaces coincide, so the deficit never ranks for fixing. That is the same concealment the absorbing fallback performs, sitting on the success arm instead of the failure arm.

### 3.3 The misattributed refusal — the diagnostic overwrites itself

The reported run's headline line was:

```
⛔ decl=gunbc.live_deploy.apply::live_deploy_apply_srv1_wet#whole refused: rendered text cannot contain terminal cursor-control bytes
```

That is not the deploy's reason. It is `extdeps.render.terminal`'s `TerminalWriteRefused` cause, substituted for it. Chain:

1. `ansi_terminal_controls = ansi_c0_controls ++ ansi_c1_controls`, and `ansi_c0_controls` includes `\x09` (TAB) and `ansi_line_feed`. So `ansi_text_contains_terminal_control` returns true for **any multi-line text**, which is not what "cursor-control bytes" means — the wall exists so callers cannot smuggle their own overwrite/erase payloads, and `project_terminal_write_admitted` legitimately owns CR + erase-line itself.
2. `ServiceNotReady.unit_diagnosis` is captured `systemctl status` output — multi-line by construction. `service_ready.dag`'s `live_deploy_service_ready_unit_diagnosis_note` records that this field was added *precisely* so this failure would stop being unlocatable.
3. The write refuses, and `cli_run.rs` (`scoped_run_projection_refusal_write`) re-emits with the **render cause** as the `diagnostic`, dropping the decl's reason. `observation_ci_render.dag` renders `Refused { diagnostic }` as `concat("refused: ", d)`, so the observation line reports a rendering complaint.
4. The real reason survived only because a separate path printed `ExitFailure { reason: ServiceNotReady … }`.

So the located diagnosis was recoverable, but the primary channel actively misdirected: "cursor-control bytes" reads as corrupt output, not as "srv1 is serving another commit." This fires on **every** refusal whose reason carries a captured multi-line diagnostic, which is the entire class the field was introduced for.

## 4. A fourth finding: the fix is currently inexpressible

Two silent drops in the emitter mean the obvious fix would typecheck and change nothing — the §5 specification-without-execution trap:

- **`job_yaml` never emits `concurrency`.** `Job.concurrency: ConcurrencySpec?` exists on the type (`dag/extdeps/github/actions.dag:120`), but `job_yaml` (`dag/extdeps/languages/yaml/gha_workflow.dag:324`) emits `runs-on`, `needs`, `timeout-minutes`, `if`, `permissions`, `steps`, `env`, `continue-on-error` — and drops `concurrency` (along with `name`, `strategy`, `outputs`). Setting the field on the deploy job today is a no-op.
- **`queue` is never emitted.** `ConcurrencySpec` distinguishes `ConcurrencyMappingQueueMax` from `ConcurrencyMappingQueueNotMax`, and `CancelInProgressWhenQueueMax` correctly models GitHub's constraint that `queue: max` forces `cancel-in-progress: false`. But `concurrency_mapping_queue_max` emits only `group` and `cancel-in-progress` — no `queue` key — so the two variants serialize **identically**. A modeled distinction with no realization difference.

Both must be closed before any concurrency row can be believed, and both want a discriminating RED (a witness asserting the emitted YAML *contains* the key).

## 5. What GitHub can and cannot do

The operator's ask — *each srv1 deploy blocks on the previous one, and no deploy is cancelled* — is achievable, but not exactly with the default `concurrency` behavior. Three mechanisms:

| mechanism | serializes? | cancels anything? | notes |
|---|---|---|---|
| job-level `concurrency: {group: deploy-srv1, cancel-in-progress: false}` | yes | **yes** — pending depth is 1 | documented: *"any existing `pending` job or workflow in the same concurrency group will be canceled and the new queued job or workflow will take its place."* The in-progress run is untouched; intermediate **pending** runs are dropped as each new one arrives |
| job-level `concurrency` with `queue: max` | yes | only past the cap | permits multiple pending (documented cap 100); `cancel-in-progress` must be false. Closest to literal "queue, don't cancel" — but the emitter cannot emit `queue` today (§4) |
| a **dedicated single runner label** (e.g. `runs-on: [self-hosted, linux, arm64, srv1, srv1-deploy]` with exactly one runner carrying `srv1-deploy`) | yes, strict FIFO | **no** | a self-hosted runner executes one job at a time, so extra deploys simply wait for the runner. Nothing is cancelled, no GitHub-side queue semantics involved, and it needs no emitter change — only a runner registration and a label row |

`concurrency` **is** supported at job level, so this can be scoped to the deploy without touching the `ci` job's group.

The third option is the only one that literally satisfies "don't cancel any deploy, block until the previous completes (pass or fail)", and it is also the cheapest to land given §4. It has one operational cost worth naming: a single deploy slot means a hung deploy blocks all subsequent deploys until its `timeout-minutes: 10` backstop fires — bounded, and loud.

## 6. Why serialization alone is not the fix — it would make things worse

Given §2.3, strict FIFO by arrival will regularly run an **older** commit's deploy after a newer one. That older run then rsyncs its older tree, restarts, polls, sees its own surface, **converges, and greens** — silently regressing the live dashboard to a superseded commit. Applied to the reported pair, queueing alone would have produced: `ab58f6bca4` deploys, then `b61ed50962` deploys over it and greens, leaving main serving the older tree with two green jobs. That is strictly worse than today's honest red.

So serialization is necessary and not sufficient. The missing half is a **freshness precondition**: a deploy is convergence toward main's tip, not a per-commit obligation, so a deploy whose commit is no longer main's tip has *nothing to do* and must say so rather than act.

## 7. Proposed shape (for operator ruling — nothing landed)

1. **Exclusion** — one deploy at a time on srv1. Recommend the dedicated-runner-label route (§5, option 3): no cancellation, no emitter dependency, expressible as a runner spec row on `gunbc_ci_deploy_srv1_stage`.
2. **Superseded-tip refusal** — before mutating, the deploy establishes that its revision is main's tip. If not, it emits a typed, located, **counted** `SupersededByNewerTip` and mutates nothing. This is what keeps §6 from biting. It also subsumes the flake: the loser stops racing instead of refusing after 120 s.
3. **Graph identity in the fingerprint** — kill the false-green class at its root by publishing, in the healthz artifacts map, a content hash over the **source closure the serve process actually loaded**. Then `observed == expected` is decidable regardless of whether the rendered pages coincide, and the check finally claims what the deploy owes. Precedent exists (`resolved_graph_cache` already combines per-module content hashes; `std.content_hash` is the authority). Keep it a pure function of the loaded sources — not a `git rev-parse`, which would make healthz wet.
4. **Close the two emitter drops** (§4) with RED controls, whether or not option 1 needs them — a declared field the serializer discards is a fail-open in every future consumer.
5. **Retire the `/etc/gunbc-tree-sync.env` channel** — the rsync source belongs in the unit invocation, not in a globally-mutable env file. Exclusion closes the window, but the misattribution hazard is independent and cheap to remove.
6. **Narrow the control-byte wall** — `ansi_text_contains_terminal_control` should test for the bytes the wall is actually defending against (ESC/CSI, CR, and the C1 range), not LF and TAB, which are ordinary content in any captured diagnostic. Alternatively the refusal-write path must preserve the decl's reason and report the render failure as a *second* fact rather than a substitution. Either way the invariant is: **a diagnostic channel may not replace a located diagnosis with a complaint about itself.**

## 8. Open questions the operator should rule on

1. **Disposition of a superseded deploy.** `SupersededByNewerTip` is neither `Converged` nor a failure — the desired state is already at-or-ahead of this run's tree. Folding it into `Converged` is a state-space conflation (§3) and re-opens the false green; folding it into a red makes ordinary rapid pushes red main. A third disposition looks right, but it must be *counted* so the rate stays observable. **Which arm, and does it exit 0?**
2. **Does `Superseded` skip the mutation entirely, or still install the binary?** If a newer tree is live, an older binary install is itself a regression — argues for skipping everything.
3. **Scope of the healthz fingerprint (item 3).** Widening it to a source-closure hash makes the readiness contract strictly stronger and will red any deploy whose sync was partial — including cases that green today. That is the point, but it is a behavioural change to a live gate and wants an explicit go.
4. **Is `queue: max` worth modelling now**, or is the single-runner label the terminus? The label route makes the `ConcurrencyMappingQueueMax` variant's missing `queue:` emission a latent bug rather than a blocker.

Related lane: [srvN build-cache provisioning](srvn-buildcache-provisioning-design.md) shares the srv1 host-effect surface and the same "absorbing fallback masks the deficit" shape on `ci_release_build_script`.

## 9. Receipts

### 9.1 Pin a healthz digest to a commit

```python
def fnv1a64(b):
    h = 0xcbf29ce484222325
    for x in b:
        h ^= x
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return "%016x" % h
# compare against `git show <rev>:ROADMAP.md` for each recent main commit;
# both `observed=` and `expected=` in the refusal land on named commits.
```

Valid because `roadmap_site_roadmap_md_body() == expected_roadmap_md()` and the generated-artifact gate keeps that byte-equal to the committed file on main.

### 9.2 Confirm the overlap

```
gh run list --workflow=ci.yml --commit=<sha> --json databaseId,conclusion,createdAt
gh run view <id> --json jobs \
  -q '.jobs[]|select(.name=="deploy_dashboard_srv1")|"\(.conclusion) \(.startedAt) -> \(.completedAt)"'
```

### 9.3 The 25-push sample (deploy job windows, sorted by start)

```
76ddfd8c75 success 03:36:38 -> 03:40:21
4b54914cc2 success 04:58:51 -> 05:02:21   ┐ overlap, both green
3f2174614d success 05:01:42 -> 05:05:05   ┘ (identical ROADMAP digest a8a773489d754237)
210696d72e success 05:03:47 -> 05:07:54   ┐ overlap, both green
da554826a5 success 05:04:15 -> 05:07:53   ┘
f014f275b6 success 06:13:34 -> 06:17:12
6e5816f3b5 success 06:19:23 -> 06:22:53
4c7312c0ef success 07:15:45 -> 07:19:08
8096314526 success 07:23:10 -> 07:26:42
542652f326 success 07:48:09 -> 07:51:45
203916e095 FAILURE 16:57:50 -> 17:02:14   ┐
79472b2d1f FAILURE 16:58:39 -> 17:03:14   ├ three-way overlap, exactly one green
f3440704e6 success 16:59:26 -> 17:02:52   ┘
749ca01e73 success 17:13:04 -> 17:16:23
d95a4ebcf2 success 18:53:54 -> 18:57:24
09596e5d86 success 19:08:10 -> 19:11:29
b9b8290f8b success 20:01:03 -> 20:04:31
6fa6577ca4 success 20:37:07 -> 20:40:30
f67a0c644e success 21:00:09 -> 21:04:43   ┐ overlap, both green
697135030c success 21:01:11 -> 21:04:43   ┘ (identical ROADMAP digest 8759d254e4097c7f)
ab58f6bca4 success 21:30:00 -> 21:33:48   ┐ overlap, exactly one green
b61ed50962 FAILURE 21:30:12 -> 21:34:42   ┘ (the reported run)
2ee7a5c40a success 21:51:51 -> 21:55:24
```

Every failure is in an overlap window; no non-overlapping deploy failed. Five overlap events, 11 runs involved, 3 refusals — the other 8 are the false greens of §3.2.
