# srv1 deploy — the concurrency race, the false greens, and the misattributed refusal

Status: PHASE A LANDED, ACTIVATION UNVERIFIED, ORIGINAL DEFECT OPEN (see [Status — 2026-08-07](#status--2026-08-07) below). Diagnosis dated 2026-07-29, session sleek-lynx-322. Origin: operator asked to root-cause a `deploy_dashboard_srv1` failure reporting `ServedSurfaceStale`, then directed that the **false greens and the misattributed errors matter as much as the flake**.

**Epistemic status, stated up front** — an earlier revision of this line claimed "nothing here is inferred from the shape of the code alone", and that was false of its own contents (review 44766 caught it). Three tiers are used deliberately and marked where they appear:

- **Observed** — digests, commit shas, job windows, step conclusions, and the emitter/roster contents in §4. Reproducible from §9.
- **Reconstructed** — the §1 step ordering. The endpoints are observed; the interleaving *between* them is the simplest ordering consistent with them, not a logged sequence.
- **Possible-by-construction** — hazards derived from code shape (§2.2). These say a state is *writable*, not that it *occurred*. One claim previously stated at this tier as though observed (two interleaved rsyncs) was wrong and has been removed.

One sentence: **`deploy_dashboard_srv1` has no mutual exclusion, so several main-push deploys mutate one srv1 host at once; the loser refuses correctly (red), the winners green without establishing which tree is live, and the refusal's own diagnostic is overwritten by an unrelated rendering complaint.**

## 0. Displaced cost (§6 — the pain this removes)

Measured over the last 25 pushes to main, all 25 of which ran a deploy job (§9.3):

| | count | what it costs |
|---|---|---|
| red main deploys, every one inside an overlap window | 3 / 25 (12%) | a red main that has nothing to do with the commit; per `deploy-srv1-job-masked-by-skip-on-red-ci`, per-job faults here stay latent |
| **identity-unproven greens** — successes inside an overlap window | **8 / 11 overlapping jobs** | success is reported without establishing that this run's source closure and binary are the live ones. Not "8 wrong-tree deployments" — see §3.2 on what is and is not measured |
| refusals naming the wrong cause | every refusal carrying a captured multi-line diagnostic | the mechanism added *specifically* to locate this failure is defeated at the moment it fires |

The middle row is the load-bearing one. The flake is loud and self-heals on the next push; the identity-unproven green is the §5 absorbing-fallback shape moved onto the *success* arm — the contract's claim is narrower than the obligation it is trusted for, and nothing counts the gap. The honest quantity is **exposure concealed by the success contract**, not proven lies.

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

**Precisely what this establishes (observed):** the run for `b61ed50962` observed a served-surface fingerprint equal to `ab58f6bca4`'s *expected* fingerprint. It does **not** establish that `ab58f6bca4`'s *tree* was live — the health endpoint identifies five rendered bodies, while the deploy mutates a source tree and a binary. Saying "observed `ab58f6bca4`'s tree" would contradict §3.2's own finding, and an earlier revision of this doc did exactly that (review 44766).

**Reconstructed timeline** — the endpoints below are observed (job windows, the unit diagnosis `Active: since 21:32:26; 2min 12s`, the probe transcript); the ordering between them is the simplest sequence consistent with those endpoints, not a logged trace:

1. both runs drive the tree sync; the effective source selector ends up pointing at `ab58f6bca4`'s checkout (see §2.2 — the mechanism is the shared env file, not two concurrent rsyncs)
2. `21:32:26` — a `systemctl restart gunbc-roadmap.service` wins; the new process begins loading whatever is on disk
3. `~21:32:38` — the loser's poll starts (bound 120 s, cadence 1 s)
4. `~21:33:3x` — the process finishes load-plus-compile and binds; this is the ~70 straight `curl` exit=7 probes, *not* a crash — a restart reset the load clock mid-poll
5. `21:33:48` — the winner's poll sees a matching fingerprint, converges, job green
6. `21:34:38` — the loser exhausts its bound against a fingerprint that is not its own, refuses `ServedSurfaceStale`

The refusal is **correct**. `gunbc serve` binds its graph once at process start (`service_ready_means_serving_this_tree_note`), the unit is a singleton, so at most one concurrent deploy can hold a matching fingerprint.

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

Every concurrent deploy mutates the same singleton host state, unlocked (`.github/live-deploy-srv1-apply.sh`). These are **possible-by-construction** hazards — each names a state the code permits to be written, not an event observed in the incident:

| state | hazard |
|---|---|
| `/etc/gunbc-tree-sync.env` — holds `GUNBC_TREE_SRC=$PWD` | a **global last-writer-wins source selector**. The CI deploy uses `LocalShell`, so `$PWD` really is the individual runner slot's checkout. Run X's `systemctl restart gunbc-tree-sync.service` can therefore sync run **Y's** checkout, and X has no way to tell |
| `/opt/gunbc/gunbc` via `rsync -rlpt --delete` | the tree is replaced in place, so a serve process can begin loading while replacement is in flight. **Not** claimed: two simultaneous rsync processes — the sync is one `Type=oneshot` unit and systemd holds at most one job per unit, merging or replacing conflicting queued jobs, so concurrent callers get a merged/replaced job rather than parallel rsyncs (correction from review 44766; the earlier "mixed tree that never existed in git" claim was asserted from code shape and is withdrawn) |
| `/opt/gunbc/bin/gunbc` | last writer wins, and it is installed into the live path rather than activated atomically; the running binary and the loaded tree can come from different commits |
| `gunbc-roadmap.service` (one unit) | last restarter decides the served graph for everyone |

Two consequences worth stating separately. First, more than one caller can believe it initiated its own sync when systemd actually ran a merged job against another caller's env file — so even the *winner* has not established that it deployed its own tree; it is the run whose expected fingerprint happened to match. Second, `live_deploy_apply_via_transport` performs mutation, readiness polling, and digest readback as three separate phases, so an exclusion boundary that does not span **all three** lets another deploy change the host while the previous run is still proving readiness.

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

### 3.2 The identity-unproven green — the dangerous half

**What the sample establishes.** In the 25-run sample there were 5 overlap events covering 11 deploy jobs; 3 refused and **8 succeeded**. The success contract identifies only five rendered bodies. Therefore those 8 successes **do not establish that the run's source closure and binary were the live ones**. The sample demonstrates 8 *identity-unproven* overlapping greens and a real false-green channel.

**What it does not establish**, and an earlier revision of this section wrongly implied (review 44766): that all 8 deployed the wrong source or binary. The missing discriminators are specific and worth naming, because each is a thing a follow-up could actually measure:

- job-window overlap does not prove the *mutation* phases overlapped — checkout, artifact download and verification all precede the deploy step;
- in a two-way overlap with one green and one red, the green may simply be the correct final winner;
- equal `/ROADMAP.md` digests do not imply all five healthz bodies were equal (that axis is just the one cheap enough to compute across history — §9.1);
- even equal five-path maps prove only that the contract *cannot distinguish* the revisions, not that the wrong one was live;
- the global source selector (§2.2) makes wrong-tree success *possible*; possibility is not a receipt that it happened in any particular green run.

**The defensible statement:** of 11 deploy jobs whose windows overlapped, 8 completed successfully, and because the contract identifies only five rendered bodies none of those 8 established that its own source closure and binary were live.

Two distinct gaps compose to produce that:

1. **Coincidence can satisfy the check.** When the fingerprint of tree A equals that of tree B, the contract is satisfied without discriminating them — and the `/ROADMAP.md` block `a8a773489d754237` spans ten consecutive commits, so such windows demonstrably exist. The check is honest about what it claims ("the surface I can observe matches mine"); that claim is simply narrower than the deploy's obligation.
2. **The fingerprint covers less than the deploy mutates.** `roadmap_site_healthz_body()` publishes digests for exactly five URL paths. The deploy also installs the whole source tree and the `gunbc` binary, which the dispatch path and the serve closure both depend on. So `green` means *those five bodies match*, never *my tree and binary are what is deployed*.

The §5 reading: whenever the fingerprints coincide, the underlying fault cannot be *observed at all*, so its frequency is zero by construction and the deficit never ranks for fixing (§6 prices by displaced cost, and a masked cost displaces nothing). That is the absorbing fallback's concealment moved onto the success arm. The measured quantity is exposure, and the remedy is to widen what the contract identifies (§7 item 3) so the fault becomes countable rather than to assert how often it has already bitten.

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
| a **dedicated single runner label** (e.g. `runs-on: [self-hosted, linux, arm64, srv1, srv1-deploy]` with exactly one runner carrying `srv1-deploy`) | one-at-a-time execution | **no ordinary pending-replacement** | a self-hosted runner executes one job at a time, so extra deploys wait for the runner. Needs no emitter change — a runner registration and a label row |

`concurrency` **is** supported at job level, so this can be scoped to the deploy without touching the `ci` job's group.

The third option is the closest to "don't cancel any deploy, block until the previous completes (pass or fail)", and the cheapest to land given §4. Four limits, because an earlier revision called it "strict FIFO" and that is too strong (review 44766):

- GitHub documents **no commit-order FIFO guarantee** for self-hosted runner routing;
- GitHub's own concurrency queue orders by *when a job begins waiting* — after its dependencies resolve — not by push or dispatch order, and the docs say ordering is not guaranteed;
- §2.3 already proves deploy eligibility inverts relative to commit order in practice, so whatever order emerges is not commit order;
- a queued self-hosted job can eventually fail on the platform's queue timeout, so "never cancels anything" is operational, not absolute. Separately, a single slot means a hung deploy blocks the rest until its `timeout-minutes: 10` backstop fires — bounded and loud.

The accurate claim is therefore: *exactly one matching runner gives one-at-a-time execution without ordinary pending-job replacement.* It is not an ordering guarantee, and §7 item 1 explains why it must not be the mutual-exclusion authority either.

## 6. Why serialization alone is not the fix — it would make things worse

Given §2.3, serialization by arrival will regularly run an **older** commit's deploy after a newer one. That older run then rsyncs its older tree, restarts, polls, sees its own fingerprint, **converges, and greens** — silently regressing the live dashboard to a superseded commit. Applied to the reported pair, queueing alone would have produced: `ab58f6bca4` deploys, then `b61ed50962` deploys over it and greens, leaving main serving the older tree with two green jobs. That is strictly worse than today's honest red.

So serialization is necessary and not sufficient. The missing half is a **freshness precondition**: a deploy is convergence toward main's tip, not a per-commit obligation, so a deploy whose commit is no longer main's tip has *nothing to do* and must say so rather than act.

## 7. Proposed shape (for operator ruling — nothing landed)

1. **Exclusion, in two layers — the scheduler is mitigation, the host is the authority.** A runner label or a `concurrency` group only constrains *GitHub jobs routed through it*. The repo also exposes `live_deploy_apply_srv1_operator_wet` (`apply.dag`) on a different access path, and neither mechanism can exclude that caller, another workflow, or an accidental second runner picking up the label. So:
   - **(a) scheduler pressure control** — job-level `concurrency` with `queue: max` once §4's emitter drops are closed, or one matching runner as the immediate mitigation (a runner spec row on `gunbc_ci_deploy_srv1_stage`);
   - **(b) the actual invariant: a typed host-side lease on the srv1 deployment resource**, held from source synchronisation through binary activation, service restart, readiness, *and* the final identity readback. §2.2 is why the span matters: `live_deploy_apply_via_transport` has three phases, so releasing at restart lets the next deploy mutate the host while this one is still proving readiness — the same race one phase later.

   Two further properties belong with (b), and neither is expressible today:
   - **atomic activation** — build a versioned release directory and switch one active pointer, instead of rsyncing and installing into live mutable paths (which is what makes "loading while the tree is replaced" writable at all);
   - **monotonicity** — refuse a candidate that would move the deployed generation *backward*. This is the construction-side form of §6's regression, and it holds even if the freshness check in item 2 is wrong or unavailable.
2. **Superseded-tip refusal** — before mutating, the deploy establishes that its revision is main's tip. If not, it emits a typed, located, **counted** `SupersededByNewerTip` and mutates nothing. This is what keeps §6 from biting. It also subsumes the flake: the loser stops racing instead of refusing after 120 s.
3. **Deployment identity in the fingerprint** — attack the false-green class at its root by having healthz publish the identity of what is *deployed*, not only what is *rendered*. That identity is a **pair**, because §2.2 and §3.2 establish that the deploy mutates two things:
   - **(a) the source closure the serve process actually loaded** — a content hash over it makes `observed == expected` decidable regardless of whether the rendered pages coincide. Precedent exists (`resolved_graph_cache` already combines per-module content hashes; `std.content_hash` is the authority). It must stay a pure function of the loaded sources — not a `git rev-parse`, which would make healthz wet.
   - **(b) the running `gunbc` binary** — without it, a correct source hash can still green while the installed `/opt/gunbc/bin/gunbc` is from another commit, since the deploy installs the binary as well as the tree.

   **(a) alone does not close the class.** It is a strict improvement over five rendered bodies, but describing it as closing the false-green class would repeat §3.2's own error one level up — a check trusted for an obligation wider than the claim it actually makes. The mechanism for (b) is a genuine choice and is deferred to §8 Q5; until that is ruled on, this item is "identify the source closure, and name the binary as still-unidentified", not "the class is closed".
4. **Close the two emitter drops** (§4) with RED controls, whether or not option 1 needs them — a declared field the serializer discards is a fail-open in every future consumer.
5. **Retire the `/etc/gunbc-tree-sync.env` channel** — the rsync source belongs in the unit invocation, not in a globally-mutable env file. Exclusion closes the window, but the misattribution hazard is independent and cheap to remove.
6. **Make the control-byte admission cursor-action aware, and make the diagnosis survive regardless.** Two separate repairs, and the first is *not* "allow LF and TAB everywhere" (review 44766): `serialize_frame` deliberately joins lines with `\n`, so `Append` can legitimately carry a multi-line frame — but `Overwrite` models a single open dynamic line, and permitting embedded line feeds there would leave earlier lines uncleared on the next overwrite. So admission must depend on the requested `CursorAction` and the `DynamicLineState`, not on one global byte roster. Independently, the refusal-write path must **preserve the decl's located diagnosis** and report a render failure as a *second* fact, so the diagnosis survives whether or not its multi-line detail renders. The invariant: **a diagnostic channel may not replace a located diagnosis with a complaint about itself.**

## 8. Open questions the operator should rule on

1. **Disposition of a superseded deploy.** `SupersededByNewerTip` is neither `Converged` nor a failure — the desired state is already at-or-ahead of this run's tree. Folding it into `Converged` is a state-space conflation (§3) and re-opens the false green; folding it into a red makes ordinary rapid pushes red main. A third disposition looks right, but it must be *counted* so the rate stays observable. **Which arm, and does it exit 0?**

   Receipt that this question is not academic, collected while this doc was being written: flipping this PR out of draft fired `ready_for_review` at `22:48:46`, `ci.yml`'s own `cancel-in-progress` group superseded the 80-second-old draft-era run, and all five of its jobs concluded **`cancelled`**. The dashboard read that as *"CI FAILING (5 failing) — this blocks merge"* and demanded a fix for a non-defect. Superseded work already exists one layer out, GitHub already gives it a distinct conclusion, and a consumer still collapsed it into failure. That is §3.3's misattribution in a second location, and it is exactly what giving `Superseded` a failure-shaped disposition would institutionalise inside the deploy.
2. **Does `Superseded` skip the mutation entirely, or still install the binary?** If a newer tree is live, an older binary install is itself a regression — argues for skipping everything.
3. **Scope of the healthz fingerprint (item 3).** Widening it to a source-closure hash makes the readiness contract strictly stronger and will red any deploy whose sync was partial — including cases that green today. That is the point, but it is a behavioural change to a live gate and wants an explicit go.
4. **Is `queue: max` worth modelling now**, or is the single-runner label the terminus? The label route makes the `ConcurrencyMappingQueueMax` variant's missing `queue:` emission a latent bug rather than a blocker.
5. **How is binary identity established (§7 item 3b)?** A source-closure hash leaves `/opt/gunbc/bin/gunbc` unidentified, so source and binary can come from different commits and still green.

   **Provenance is not identity** (review 44766). A build-time *source* stamp proves only what source the binary claims to have been built from: two binaries from identical source but different compiler version, feature set, linker inputs — or a tampered one — carry the same stamp. Binary identity wants the artifact itself: a post-link digest, an attested artifact digest, or a linker build ID. So the honest shape is a versioned manifest rather than one field —

   ```
   DeploymentIdentity { source_closure_digest, binary_artifact_digest_or_build_id, source_commit, schema_version }
   ```

   — with the installed artifact verified against the manifest *before* activation, and the running process exposing the immutable identity it was launched with.

   One more constraint on the carrier: today's `ContentHash` path bottoms out in **64-bit FNV-1a**. That is adequate for correlating this bounded incident (§9.1 relies on it), but a 64-bit value cannot serve as collision-resistant deployment identity over arbitrary artifacts, so item 3 needs a decision about the digest carrier too, not only about what is digested.

   **Which mechanism — or is the obligation explicitly narrowed to source identity, with binary identity discharged by a named mechanism elsewhere?** Until this is answered, item 3 must not be described as closing the false-green class.
6. **What is the desired revision authority (§7 item 2)?** "A deploy is convergence toward main's tip" is a proposed *policy*, not a derived fact, and there are at least three candidates with different behaviour: the **raw current main tip**; the **newest main revision that passed deployment admission**; or merely a **monotonic no-rollback relation** against the currently active deployment. The discriminating case: commit B lands after A but B's CI fails. A raw-tip check makes A superseded and leaves the *previously* deployed version live; a latest-admitted-green policy deploys A; a monotonicity-only policy also deploys A but tolerates B never arriving. All three are defensible and they are different semantics — so this belongs **before** `SupersededByNewerTip` is named as the carrier, since the name presupposes the first answer.

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

Denominator, stated explicitly because an earlier revision said "25 pushes" over a 23-row table (review 44766): **all 25 push runs ran a deploy job.** The two absent rows were `042a35fe86` and `d6ec32b98f`, whose deploy jobs had not yet been created when the sample was taken; both have since concluded `success` and neither overlaps another window. They are included below.

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
042a35fe86 success 22:47:16 -> 22:50:53
d6ec32b98f success 23:12:52 -> 23:16:27
```

25 deploy jobs; 5 overlap events involving 11 of them; 3 refusals, every one inside an overlap window, and no non-overlapping deploy failed. The other 8 overlapping jobs are the **identity-unproven greens** of §3.2 — successes whose own source closure and binary were never established as live, which is not the same claim as 8 wrong-tree deployments.

---

## Status — 2026-08-07

### Delivered

- **#7909 / Phase A merged as `d409b75f7fcf1aeb8c3a52744734945c35b7ae46`:**
  - `gunbc serve` binds one release revision at process launch, validated (40 lowercase hex) before the listener binds;
  - `/healthz` publishes `revision` and `surface_bundle_identity` as separate members, because they fail independently and their remedies differ;
  - apply admits the candidate checkout before any host mutation — `CandidateCheckoutClean | CandidateCheckoutDirty { tracked_changes, staged_changes, untracked_deployed_paths, ignored_deployed_paths } | CandidateCheckoutUnobservable`, and construction proceeds only from `Clean`;
  - readiness compares the running process against the *admitted candidate*, not against a freshly re-rendered expectation.

  The ignored-path axis was added in review and is load-bearing: `git status --porcelain=v1 -z --untracked-files=all` does **not** report ignored files, while the source sync excludes only `target` and `.git`. Ignored paths outside those two (this repo ignores `.env`, `/.gunbc/`, `.claude/`, `/target-codex/`, `/.cargo-home/`, and `src/v2/test/claim/manual/sg2_scratch_probe.dag` — a `.dag` inside a source root) were copied by rsync while the admission called the tree clean, producing a self-consistent false identity invisible to every downstream comparison. Observed through `git ls-files --others --ignored --exclude-standard -z` rather than `--ignored=matching`, because the latter reports a matching directory as one folded entry (`!! ign/`) and never names the file inside it.

- **#7990 merged as `36dbd71a31d8a139a2f51a2bcde3320458087015`:** removed the standing `ensure_is_converged` declaration homonym. This is namespace hygiene, **not** part of deploy correctness.

### Operational activation still unproven

Phase A is in main but is **not** yet established as live on srv1. Activation is complete only when:

1. every deploy job created from a commit older than `d409b75f7fc` has finished or been cancelled;
2. a main-push deploy at `d409b75f7fc` or a descendant succeeds;
3. the public routed `/healthz` reports that deploy job's exact revision;
4. its `surface_bundle_identity` and served artifact projection match the same candidate;
5. no older queued deployment subsequently removes the identity channel.

Until this receipt exists, `ReleaseRevisionAbsent` remains the honest observation of the live process.

**Measured 2026-08-08, and it changes the shape of this gate: the deploy path is inert, not backed up.** Across the last **40 main-push runs, `deploy_dashboard_srv1` was `skipped` 38 times and ran 0 times** — no successes and no failures in the sample. The skipped job records `0` steps with `started_at == completed_at`, which is the signature of a job-level `if:` evaluating false rather than of a queued or cancelled job.

That is not explained by the two conditions one would reach for first, both checked:

- **`ci` failing is not the cause.** Runs `4d8234db3ba` and `c3b7d4b5162` are `event=push`, `head_branch=main`, `ci: success` — and deploy is still `skipped`.
- **The event and ref are not the cause.** The GitHub API reports `event=push`, `head_branch=main` for those runs, and the job condition at each of `4d8234db3ba`, `6d077293a78`, and `d409b75f7fc` is byte-identically `if: github.ref == 'refs/heads/main' && github.event_name == 'push'`, which is true for them on its face.

So the activation gate is **not** "wait for the pre-Phase-A queue to drain". The queue is not the obstacle; the deploy job is not executing at all, and has not been for at least 40 main pushes. **The cause is not established here and must not be guessed** — a wrong root cause is exactly what this document was written to stop. What is established is the observation and the two eliminations above.

Consequence for Phase A: it cannot activate by waiting. Until `deploy_dashboard_srv1` executes, srv1 keeps serving a pre-Phase-A process, the identity channel is absent from the routed `/healthz`, and Phase B has no observable baseline to recut against. Diagnosing why this job is skipped is therefore the **first** Phase B prerequisite, ahead of any code work. Note the workflow shape that makes this non-automatic: `deploy_dashboard_srv1` declares `needs: [ci]` and `if: github.ref == 'refs/heads/main' && github.event_name == 'push'`, so a failing `ci` on main skips the deploy, and a `workflow_dispatch` run **cannot** activate Phase A at all.

The receipt to append when it arrives:

```text
workflow run id
deploy job id
deploy head SHA
deployment completion time
public healthz revision
public surface_bundle_identity
expected surface_bundle_identity
older queued/running deployment count after completion
```

The last field is not decoration: one successful identity-capable deploy is not durable activation while an older pre-Phase-A job remains eligible to run later and reinstall the old unit.

### Original defect remains open

Phase A makes deployed identity **observable**. It does not serialize deployment, retain every candidate, prevent rollback, or establish eventual tip convergence. Still unresolved:

- pending GitHub deployment work can still be dropped;
- deployment arrival order can differ from commit ancestry;
- the host has no target-scoped lease shared by every mutator (a runner label or concurrency group is mitigation only — neither excludes the operator apply path);
- the actuator has no production monotonic ancestry decision;
- the original diagnostic-cause substitution (§3.3) remains a separate follow-up unless independently closed elsewhere.

### Phase B — deferred monotonic reconciliation

Start from then-current main **after** the Phase A activation receipt. Do **not** continue #7802 as a merge candidate; it is closed unmerged as a quarry.

Quarry #7802 at exact head `b96cff886b1a70119070d7256366bb4745877ab4` for:

- `dag/extdeps/linux/flock.dag` (118 lines);
- `gunbc.live_deploy.lease` (600 lines);
- `gunbc.live_deploy.coordination` (322 lines);
- the locus / extent / participant / exit-domain witness matrix (most of `ci_deploy_witness_test.dag`, 918 lines);
- the queue-max workflow projection;
- the capability-gated cutover ordering.

Dissolve rather than transplant:

- receipt-authoritative active-revision observation;
- `ObservedDeployedRevision`;
- the standalone `live_deploy/revision.dag` decision vocabulary where it duplicates Phase A;
- `RecordCandidateRevision` as a prerequisite for routing;
- the `ActiveRevisionUngrounded` stopped-line architecture.

Required production path:

```text
lease target
→ observe admitted candidate
→ observe routed process identity
→ compare routed revision × candidate by Git ancestry
→ stale/equal: counted zero-mutation no-op
→ unrelated/unverifiable: refuse before mutation
→ descendant: prepare and verify idle process
→ flip route
→ re-observe routed process identity and surface
→ retire former active process
```

A filesystem receipt may remain as audit or recovery evidence. It is **not** the authority for what the public route serves.

### Phase B acceptance gates

- real `free / contended / free` lease probe as `ghrunner` on srv1;
- real ancestry probes against the runner's object store;
- active C plus older candidate B performs zero host mutations;
- equal candidate is idempotent;
- unrelated or unreadable ancestry refuses before mutation;
- candidate revision and surface verify on the idle slot;
- routed revision and surface verify again after the flip;
- former active process retires only after routed verification;
- first public cutover executes from merged main, never from an unmerged PR head.

**Resume trigger:** the routed srv1 process reports an exact merged-main revision through the Phase A health channel, and the pre-Phase-A deployment queue has drained.
