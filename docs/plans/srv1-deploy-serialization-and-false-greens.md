# srv1 deploy — the concurrency race, the false greens, and the misattributed refusal

Status: DIAGNOSIS (2026-07-29, session sleek-lynx-322), **partially superseded 2026-08-04 — see §10**. Origin: operator asked to root-cause a `deploy_dashboard_srv1` failure reporting `ServedSurfaceStale`, then directed that the **false greens and the misattributed errors matter as much as the flake**.

**What changed since this was written.** Serialization landed (#7551) as job-level `concurrency` with `cancel-in-progress: false` — the §5 first-row mechanism — but **without** the §6 freshness precondition, which is precisely the combination §6 predicted would be *strictly worse than today's honest red*. #7462 had landed the freshness classifier two days earlier with **no production consumer**. §10 records the resulting live incident and the operator ruling that supersedes §7's proposed shape.

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

## 4. A fourth finding: the fix was inexpressible — CLOSED 2026-08-04

**Both emitter drops are closed; this section is retained as the record of what they were, not as an open blocker.** As written on 2026-07-29 it said:

- **`job_yaml` never emits `concurrency`.** `Job.concurrency: ConcurrencySpec?` existed on the type but `job_yaml` dropped it (along with `name`, `strategy`, `outputs`), so setting the field on the deploy job was a no-op.
- **`queue` is never emitted.** `concurrency_mapping_queue_max` emitted only `group` and `cancel-in-progress`, so `ConcurrencyMappingQueueMax` and `ConcurrencyMappingQueueNotMax` serialized **identically** — a modeled distinction with no realization difference.

Both were subsequently repaired with the discriminating REDs this section asked for. `extdeps.languages.yaml.gha_workflow` `optional_job_concurrency_kv` now projects the field (its `job_field_projection_completeness_note` records the per-field disposition, including `strategy` being deleted rather than projected), and `concurrency_mapping_queue_max` now emits a real `queue` key (`concurrency_queue_max_emission_note`), witnessed by `gha_job_projection_witness_test` asserting the emitted mapping contains it and that the two variants no longer serialize identically.

**Consequence for §5 and §8 Q4:** `queue: max` is a live option, not a latent bug, so the single-runner-label route is no longer the cheapest path by default.

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

## 10. The predicted regression, observed (2026-08-04, session still-hawk-637)

§6 said serialization without a freshness precondition would let an older deploy run second, converge, and green — silently regressing the live dashboard. That is now observed rather than predicted.

### 10.1 The incident

Operator-reported run [30863337319](https://github.com/gunb-ai/gunbc/actions/runs/30863337319). The linked `deploy_dashboard_srv1` job did **not** run and fail: it concluded **`cancelled` with an empty step list** (started `00:37:03`, completed `00:37:39`, 0 steps). Its prerequisite `ci` job succeeded. It was a *pending* entry evicted from the concurrency group.

Three main pushes, and the deploy order inverted against commit order in both possible directions at once:

| commit | pushed | deploy window | outcome |
|---|---|---|---|
| `6d46cdc687` | 23:39:24 (**oldest**) | 00:37:44 → 00:41:22 | **success — ran last** |
| `ed9785817e` | 23:39:49 | 00:34:12 → 00:37:43 | success |
| `1a984a151d` | 23:45:23 (**newest**) | — | **cancelled, 0 steps — never deployed** |

Ancestry-verified with `git merge-base --is-ancestor`: `6d46cdc687` is an ancestor of `ed9785817e`, so the final host state is the **oldest** of the three trees, installed by a **green** job, while the newest commit's deploy was discarded.

### 10.2 The rate

Over the 44 most recent main pushes (34 deploys executed; ancestry-verified, not timestamp-inferred): **6 (18%) deployed an older tree over a newer one**, 5 of the 6 reporting `success`; **4 more were evicted while pending and never deployed at all**. Every inversion pair was confirmed by `merge-base --is-ancestor`.

*Epistemic note:* this census was collected by this session from the GitHub API and is **not** an executable receipt in the repository. It is reported as motivating evidence, not as an acceptance basis — the acceptance is the executed permutation law in §10.4. The linked cancellation and the queue semantics alone establish the design defect.

### 10.3 Why the queue could never have been right

GitHub orders concurrency entries by *when they begin waiting* — after `needs:` resolves — never by push order or commit ancestry, and documents that ordering is not guaranteed. The queue knows which job started waiting most recently; it does not know which content is newest, and §2.3 already showed arrival is inverted against commit order in practice.

The deeper defect: each deploy job owns one immutable checkout and one verified release artifact, and no durable register records `desired[srv1-dashboard] = <revision>`. So the concurrency queue was acting as a **lossy desired-state store**, and cancelling a pending job deleted the system's only actionable carrier for that revision.

A scheduler cancellation is also not a deployment verdict: `SchedulerCancelled(candidate)` ≠ `Keep{CandidateSuperseded}`. The second is an observed ancestry fact; the first means only that GitHub removed a job.

### 10.4 The ruling (operator, 2026-08-04) — supersedes §7's proposed shape

> Recut srv1 deployment as a monotonic reconciliation of one public `Srv1Dashboard` target. Immediately use `queue: max` so admitted release carriers are not discarded. Under a target-scoped exclusive lease, compare the currently routed slot's revision with the explicit candidate revision: stale/equal is a counted exit-zero no-op, descendant advances through the existing verified blue/green cutover, and unrelated/unverifiable refuses before mutation. Mutation must require the advance admission. Preserve every CI verdict; do not require every commit to actuate.

The guarantee, stated once: *after admitted work settles, the public srv1 dashboard serves the newest deploy-admitted main revision; during convergence the public route never moves backward and never points at an unverified candidate.* Safety and liveness are halves of one guarantee — wiring only the freshness guard can leave srv1 permanently on an intermediate revision when the newest job has already been cancelled, so `queue: max` and monotonicity must land together.

The ordering law is a join, `next_active = active ⊔ candidate`, which on a linear main history is idempotent, order-independent and monotonic. That is why the scheduler no longer has to preserve commit order — and why the acceptance is *every permutation of three ordered revisions converges to the newest*, not a substring pin on the emitted concurrency block. The old witness asserted that serialization **text** existed and greened in the exact state that cancelled the newest candidate.

Two carrier corrections the ruling forces, beyond the ordering law:

- **The target is the public service, not one of its alternating units.** §7's `DeployTargetId` named `service_unit: gunbc-dashboard` — a unit that exists on no machine, on a model predating blue/green. Production runs `gunbc-roadmap.service`, green carries its own derived names, and both share one root route. The lock, queue group, slots, route and active-revision observation all derive from one `Srv1Dashboard` target.
- **The active revision is read from the routed slot**, via a per-slot receipt outside the rsynced tree — not from §7's global marker keyed by one service unit, which under blue/green can describe a slot that is not the one serving.

### 10.5 The lease and the admission wall, as landed

An earlier revision of this section said the **host-side exclusive lease** (§7 item 1b) was "not yet landed", and that the observe-then-mutate race between the CI and operator entrypoints "remains open — as it is today". Both statements are now false, and both were still true of the first three commits on this branch: the lease, the monotonic decision, and the `RevisionAdvance` admission were modeled with **no production consumer**, which is precisely the #7462 failure this document's own §7 item 1 warns about. Three independent reviews (`review 48283`, `review 48288`, `review 48290`) reported it before the wiring landed, which is the honest provenance of this correction.

What is now wired, and where the evidence is:

- **The lease is the entry, not a step.** `live_deploy_apply_srv1_wet` and `live_deploy_apply_srv1_operator_wet` are both *launchers* (`gunbc.live_deploy.lease` `live_deploy_launch_under_lease`). Each observes its own checkout's HEAD, then re-invokes an inner `_leased_wet` entry as the command `flock --exclusive` wraps, passing the candidate as `--arg candidate=<sha>`. The lock is therefore held across route observation, slot preparation, readiness, readback, route flip, receipt write and retirement — the whole transaction — because it is the *process* that is wrapped. Both public entries route through it, which is what discharges flock(2)'s advisory scope; the obligation is asserted by `ci_invokes_the_launcher_and_not_an_inner_leased_entry` rather than trusted.
- **Mutation requires the admission.** `live_deploy_blue_green_cutover_with_selection` now takes a `RevisionAdvance`. That type is `sole_constructor` in `gunbc.live_deploy.revision`, so the only values in the corpus come from `decide_deploy_for_observation`'s `Advance` arm — an unguarded cutover does not typecheck. The token is *consumed*, not merely accepted: the `RecordCandidateRevision` stage records `admission.to`.
- **The decision starts from an observation, not a sha somebody found.** `ObservedDeployedRevision` splits `Recorded | Unrecorded | Unreadable`; the routed slot's receipt is read through `LiveDeploySlotRevisionRead` before any mutation, and ancestry is probed with two `git.Inspect.MergeBaseIsAncestor` calls under `OutcomeIsData` (exit 1 is the answer "no"; exit 128 is *not* folded into it and refuses).
- **An unrecorded routed slot refuses; it does not bootstrap.** An earlier revision of this branch let `Unrecorded` advance from a synthetic predecessor, on the reasoning that monotonicity is vacuous with no prior revision. That reasoning is wrong wherever it matters: the routed slot is *demonstrably serving something*, so "no receipt" means the revision is unknown, not absent, and advancing from it authorizes an arbitrary — possibly backward — move. Both `Unrecorded` and `Unreadable` are now `Refuse` arms (`ActiveRevisionUngrounded`, `RevisionRelationUnverifiable`), and the synthetic predecessor constructors are deleted rather than left unreachable. Bootstrap is therefore an operator action that records the first receipt, not something a deploy performs on its own authority.
- **The receipt is written before the route flips, and every prefix of the cutover is checked.** `live_deploy_cutover_stage_order` is a row the actuator folds, so the ordering is data rather than nesting. The invariant it carries — at every prefix, the routed slot's recorded revision equals what that slot serves — is executed by `every_prefix_of_the_landed_cutover_order_holds_the_receipt_invariant`, with the flip-before-record order as the discriminating RED and `the_violated_prefix_authorizes_a_backward_public_move` carrying that violation through the real decision fold to the harm: a routed slot serving C whose receipt still says stale admits a later, *older* candidate as an advance.

**Still open, and named rather than papered over:** flock(2) is advisory, so the lease excludes only callers that take the same lock. All four *named public* mutation entries — CI apply, operator apply, CI retract, operator retract — are launchers, and `participant_completeness_reds_when_any_mutator_is_uncovered` reds if a fifth mutator appears uncovered; but the inner `_leased_wet` entries remain callable in principle by anyone who invokes them directly. Closing that by construction needs the reference-level visibility the meta-exec confinement milestone tracks, not another runtime check; a check here would concede the bad state is writable, which is the shape §5 rejects. The residual risk is therefore one an operator has to *choose* to take, not one an ordinary deploy can stumble into.

### 10.6 The crash-prefix control, and its falsification receipt

The ordering property in §10.5 is only as good as the thing that would notice its loss. `live_deploy_cutover_stage_order` (`gunbc.live_deploy.apply`) is a row the actuator folds — `live_deploy_run_cutover_stages` short-circuits on the first non-converged stage and reports the flip's reconciliation on full convergence — and the control folds the same row rather than a copy of it.

Executed 2026-08-05, on this branch:

- With the landed order, `every_prefix_of_the_landed_cutover_order_holds_the_receipt_invariant` and `the_violated_prefix_authorizes_a_backward_public_move` both **PASS**, alongside the other 52 deploy witnesses.
- With the row edited in place to `[Prepare, Flip, Record, Retire]` — the historical order — both **FAIL**, and `the_flip_before_record_order_violates_the_receipt_invariant` (which folds its own authored copy of the rejected order) continues to pass, as its job is to prove the fixture can express the violation at all.

That is the discriminating red, taken against the production row rather than against a restatement of it. A future author who reverts the order edits one list and the control reds; a future author who adds a fifth stage gets its prefix checked for free.

A stage order that never flips is refused (`RouteNotFlipped`), not reported converged — `CutoverRouteOutcome` is its own coproduct rather than `Optional<Reconciliation>` precisely so "never published" and "the flip failed" stay separable.

### 10.7 Second operator review: two P0s and four P1s, and what closed them

Review of head `b7c36b7` (2026-08-05). Every item below is closed in the same document section that describes it, so the design note stops being a place where a superseded claim can survive.

**P0 — the first safe deployment was impossible.** Making an unrecorded routed slot refuse closed the rollback hole and closed liveness with it: no slot carries a receipt, receipts are written only after an admitted advance, and admission required a grounded receipt, so the first deploy after merge would refuse and so would every deploy after it. The repair attempted here was a grounding producer inside the leased apply path (`ground_unrecorded_slot_revision`) reading the slot's own synced git `HEAD`. **That repair was withdrawn by the third review and is recorded in §10.8** — the observation is unbound to the running process, so it manufactured a false predecessor and readmitted the backward move. The liveness debt is now held open deliberately rather than closed by an unsound producer.

**P0 — `0777` made the lock object replaceable while held.** Reproduced with util-linux `flock` before repairing: process A locks path P (inode I₁), another principal unlinks P, process B locks P — which recreates it as inode I₂ — and B acquires immediately. B exited 0 while A still held its lock, against a same-path control that correctly returned 111. The lock identity was mutable underneath a live lease, which is the absence of the guarantee rather than a permissions smell. The lock object is now the target-specific **directory** `/var/lib/gunbc/deploy-lease/srv1-dashboard`, provisioned once by root at 0755. Verified: two `flock`s on one directory contend and the loser returns the conflict code; and `rmdir`/`rm -rf` both fail EACCES under a parent the caller cannot write. Stable inode, no world-writable namespace, no shared group to invent, no lock-file lifecycle.

**P1 — no post-flip route verification.** A zero exit from `tailscale serve` is not an observation that the public mapping moved, and retirement followed it directly. `VerifyRoutedCandidate` now sits between the flip and the retirement, re-reads Tailscale status, and requires the root mount to carry exactly the candidate's proxy. Falsified: deleting the stage from the order reds `the_routed_candidate_is_verified_before_the_old_runtime_retires`.

**P1 — `VerifiedRecordedCandidate` was not produced by recording.** The type was `sole_constructor`, but the flip arm built one from values it already had, so safety rested on stage order rather than on the capability. Each stage now returns a capability its successor consumes: record produces `CandidateRevisionRecorded`, flip consumes it and produces `RoutePublished`, verify consumes that and produces `RoutedCandidateVerified`, and retirement requires it. Each missing capability is a typed refusal naming what did not happen.

**P1 — the remote-session path was unreachable.** `coordination_locus_covers` refused every `NamedHost` lease locus before consulting extent, so no `SessionLifetime` value could ever admit the future remote path and enabling it would have required editing the policy match — exactly what transports-project-facts exists to avoid. The fold is now symmetric on machine identity; today's `SshExec` reaches the correct host and is refused on **extent**, because `ssh host flock … cmd` releases when the command returns. Deferred and unobserved loci keep distinct refusal causes: "handed to another process" and "could not determine" have different remedies.

**P1 — population was joined at kind, not participant.** Both sides listed the same four kinds, so a fifth entrypoint performing an existing kind stayed green. Participants are now `DeclarationRef × kind` joined on the declaration, with `a_new_entrypoint_performing_a_covered_kind_still_refuses` as the discriminating control. Stated at its real rung: this joins two **hand-authored** lists at identity grain — it does not discover a participant nobody listed, and the dissolve-on names the lens that would.

**P1 — the wet probe could false-green.** An unrelated holder acquiring between steps A and B makes B's *outer* `flock` return 111 without ever running the nested attempt; if that holder exits before C, the probe observes 0/111/0 and passes having established nothing about self-contention. Step B now writes a marker after acquiring and before the nested attempt, and the outcome requires both the marker and the conflict code. Step D additionally requires the lock object to be a directory whose parent this principal cannot write — at its real rung: this proves *this* principal cannot replace it, not that no principal can.

Deploy witnesses: 58 green by execution at this head.

### 10.8 Third operator review: the grounding withdrawal, and four narrowed claims

Review of head `6c1aa8a1` (2026-08-05). The theme of this round is different from the previous two: nothing here was found *missing*. Every item is a mechanism that existed and worked, described by a sentence that claimed more than it delivered — the §3 stale-claim class applied to this branch's own carriers. Two of the six were defects introduced by the previous round's repairs.

**P0 — the grounding observation was unbound, so the migration repair was less safe than the deadlock it replaced.** §10.7's repair read `git rev-parse HEAD` in the slot's tree and wrote it as the slot's first receipt. The observation describes the **disk**; the receipt must describe what the routed **process** is serving, and `gunbc serve` binds its graph once at process start — so the two diverge exactly when a sync is interrupted or lands out of order. Recording the tree's HEAD then manufactures a predecessor the public service is not at, and a later candidate B with `grounded < B < served` is admitted as an advance: the public service moves **backward**, through the receipt that exists to forbid it.

The producer is **deleted, not unwired-and-retained**. An earlier draft of this repair kept it with a note explaining that it was the input half of the eventual binding; that is exactly the shape this branch already rejected when it deleted `AdvanceFromUnrecordedSlot` rather than leaving an unconstructable arm — an unreferenced producer of precisely the unsafe value reads as a supported path, and the next author meeting the deadlock reaches for it instead of building the binding. `LiveDeploySlotTreeHeadRead`, its realizer arms, its script builder, and the `SourceTreeRoot` accessor added to carry it are all deleted with it; `dag/gunbc/live_deploy/spec.dag` returns byte-identical to `main`.

A second defect in that same draft is worth recording because it is the branch's recurring failure mode caught in its own repair: the draft rewrote `Unrecorded` into `Unreadable` at the observation site. That degraded a precise typed cause (`ActiveRevisionUngrounded`) into a vague one (`RevisionRelationUnverifiable`), **and** it made the witness asserting the `Unrecorded` refusal describe a path production could no longer reach — specification-without-execution, one level up. The observation is now passed through unmodified and the decision's own arm refuses.

**The liveness consequence is real and is held open deliberately.** No slot carries a receipt, so the first deploy after this merges refuses, and so does every deploy after it, until a receipt is grounded through evidence that binds the process to a revision. That is a stopped line, not a working deploy — stated here rather than left for the operator to discover. The dissolve-on is named on the carrier: a slot-local expected surface rendered from the slot's own tree, or a commit identity the served process publishes about itself.

**P0 — the receipt was writable by the service user.** `live_deploy_slot_revision_record_script` installed the receipt `-o <service_user> -g <service_user>` at `0644`, and `0644` grants the **owner** write. The service user is the principal the serve unit and belt timer run as — a long-lived process the model does not designate as the receipt's writer — so the monotonicity authority was writable by something that could overwrite a routed slot's receipt with an older sha and readmit a superseded candidate as an advance. The file is now placed by root through the granted `sudo install` with no owner flags at all; everyone reads it at `0644`, nobody but root writes it. The witness asserts the **absence of the capability**, not a policy about who should use it.

**P0 — the covered participant list was an alias of the population it was joined against.** `live_deploy_leased_participants()` returned `deploy_mutation_population()` verbatim, so the production admission was tautological: every participant added to the population was automatically declared covered, and the one subject that actually runs could not refuse for any input. The drop-one witnesses were real, but they exercised the pure fold with test-supplied lists — the axis they were trusted for was inert exactly where it mattered. The covered list is now authored independently as what the **launcher** reaches, and `the_covered_population_is_not_an_alias_of_the_mutation_population` feeds the production covered list a population containing one extra participant and requires a refusal; if the alias is ever restored, that half greens vacuously and the witness reds. Stated at its real rung: two hand-authored lists compared at identity grain is a debt contract over a roster, not a closed subject universe.

**P1 — the capability payloads were forgeable across modules.** The stage-sequencing note claimed only the predecessor stage could produce each capability. The production fold does respect the sequencing, but the outcome coproducts were ordinary, so any module holding a genuine `RevisionAdvance` could construct `CandidateRevisionRecorded` itself. The payloads (`RecordedCandidateEvidence`, `RoutedCandidateEvidence`) are now `sole_constructor`, which closes the cross-module half. What remains true and is now said: within this module the evidence is still derived from `reconciliation_converged` on the stage's own result rather than returned *by* the effect, so this is mechanical prevention against other modules plus ordinary sequencing discipline within one — not the structural guarantee that the effect yields its own evidence.

**P1 — the trailing-whitespace refusal was asserted where the bytes never arrive.** A witness asserted `parse_slot_revision_stdout(good + LF)` is `Unreadable`. It refuses — 41 characters are not 40 hex — but that state is unreachable in production: the wet shell transport applies `.trim_end()` before any decoder runs, so a receipt of 40 hex bytes plus `LF`/`CR`/space/`TAB` arrives already canonicalized and is **accepted**. The assertion passed while the property it was trusted for was false. Exactness is now enforced where it executes — the read script measures the file with `wc -c` and exits nonzero unless it is exactly 40 bytes — and the witness asserts that gate is in the script. Internal corruption stays a parser-level claim, because trimming does not touch it.

**P1 — step D claimed more than a probe run as that principal can establish.** The step proves the lock object's DAC permissions deny unlink and rename to this principal *acting as itself*. It does not establish that this principal cannot replace the object: `gunbc.fleet_posix_accounts` grants `ghrunner` NOPASSWD `rm` and `install`, and the generated sudoers lines scope the command **path**, not its arguments, so `sudo rm -rf <lock object>` followed by `sudo install -d` is within the grant. The honest scope is a **cooperative-caller** model — the object cannot be replaced by accident, by an unprivileged co-tenant, or by ordinary deploy machinery — and the dissolve-on is argument-scoped privilege for the deploy principal.

Deploy witnesses: 59 green by execution at this head (38 + 21).
