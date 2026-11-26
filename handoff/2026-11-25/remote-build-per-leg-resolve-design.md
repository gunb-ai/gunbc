STEP 1 (design, no code yet) — remote-build per-leg closure resolve. Evidence is at head 37722a369e47 in my worktree.

=====================================================================
(a) WHERE THE PER-LEG RESOLVE IS, AND WHY A PROBE PAYS IT
=====================================================================
1. One process per leg. sshd runs the remote-build key's forced command, rendered by
   gunbc.compute.remote_build_serve `remote_build_forced_command_words` (line ~450) as
   `<serve_binary> run --source-root <repo>/dag --source-root <repo>/src/v2
    --entry <repo>/dag/gunbc/compute/remote_build_serve.dag --function remote_build_serve_cli
    --arg host=<host>`
   with serve_binary = /opt/gunbc/bin/gunbc (gunbc.host.host_layout srv1_gunbc_serve_binary).
   gunbc.compute.remote_build_client `remote_build_try_host` sends TWO legs per candidate:
   ProbeLeg with the words `remote-build-v1 probe`, then — only on ready — a ProvisioningLeg
   with the request words. Each leg is a fresh exec: nothing is shared between them.

2. The run verb resolves before it runs anything. src/v1/stage0/src/main.rs `run_verb` calls
   `cli_run::resolve_entry_graph_demand_acquired_scoped(source_roots, entry_file, &requested)`
   and then `report_pre_entry_phases`, which is what prints the line calm-wren read
   (`resolve_entry_graph_inclusive`). `--function` scoping narrows which function BODIES get
   typechecked; it cannot narrow the import closure, because the closure is what the entry
   module's `import` lines name. Whole-tree `resolve_entry_graph` is not involved — this is
   the demand-acquired path, and it is still 4 minutes.

3. The 895 modules are the SERVE entry's own closure, not the request's. Static census of the
   `import` graph over this tree (tools/closure_census.py, uncommitted, calibrated):
   gunbc.compute.remote_build_serve = 891 modules; the engine reports closure_files=895
   (+4 prelude). Rate check against the corpus: 895 → 248 s ≈ 278 ms/module; the
   gunbc.plans.closure_consolidation rows give 3,174 modules → 1,492 s ≈ 470 ms/module,
   16.2 GiB. So it is linear-plus, and the fixed floor of a `gunbc run` is unmeasured.

4. WHY a probe resolves anything at all. The probe answer is produced at
   remote_build_serve.dag:131-132 — `if trim(text) == remote_build_probe_text { answer ServeReady }` —
   but it sits INSIDE `remote_build_serve_on`, which the process reaches only after resolve, and
   it is not a free constant: the receipt interpolates `instance.instance_id` (a roster read via
   gunbc.compute.host_capacity `host_capacity_home_for_host`, closure 343), and step 2's principal
   gate asks that same authority for the ledger's service user. So the probe demands, purely by
   its position in the module graph, the 891-module closure that belongs to the REQUEST path
   (fetch, seed build, provider, ledger).

5. THE CONSEQUENCE THAT MAKES THIS A BUG, NOT A TUNE-UP. The probe leg's total-wall ceiling is
   `fleet_ssh_leg_deadline.wall_probe = second(15)` (gunbc.fleet.fleet_known_hosts_anchor:1366),
   and test.claim.live_deploy.deployed_tree_leg_deadline_witness pins the four ceilings as
   grounded literals (15/60/300/900). A 248 s resolve cannot answer inside 15 s: the leg comes
   back SessionOverran → remote_build.remote_build_next_step → HostLegOverran → moves on. Every
   roster host overruns identically, so the walk ends RemoteBuildNoHostAvailable. Either the
   remote-build dispatch (client entry `remote_build_run`, consumed by gunbc.compute.test_run)
   is not dispatching today, or something is widened around it. That is the first thing I want
   your read on, and it is why I am writing before code.

6. For the record on the request leg: it pays the resolve at least twice — the serve entry's,
   then the provider operation's own `gunbc compile` / claim_batch resolve
   (gunbc.compute.work_provider_local `compute_command_argv`).

=====================================================================
(b) THE FIX, JUDGED ON THE MATERIALIZATION LADDER
    (DESIGN `gunbc.design_argument::demand_before_materialization`: authored duplication?
     reuse obligation? provider discharge? — then realization)
=====================================================================
CANDIDATE 1 — the probe answers from a tiny front door.  RECOMMENDED; the only one that makes
the 15 s ceiling honest.
  Ladder step 1 decides it: the probe's demand for the 891-module closure is AUTHORED
  DUPLICATION, not a reuse obligation — the answer is a constant plus a principal reading.
  Caching it (candidate 2) would "make redundant work cheap and suppress the signal that would
  rank it for deletion". Delete the demand.
  Shape: a leaf entry module `gunbc.compute.remote_build_serve_probe` (the front door) that reads
  SSH_ORIGINAL_COMMAND; if it is exactly the probe word it admits the principal and answers
  ready; otherwise it execs the EXISTING serve entry (`<same binary> run --entry
  .../remote_build_serve.dag --function remote_build_serve_cli`) and streams that answer. The
  forced command names the front door; the request path is untouched and pays what it pays today
  against a build. Entry selection by declared binding follows the existing precedent in
  gunbc.roadmap.roadmap_serve / roadmap_ingress ("an instance is a handler, not an argument").
  Authority it reuses (single_authority): the wire vocabulary stays gunbc.compute.remote_build's
  (`remote_build_probe_text`, the serve header/ending words) — so those constants MOVE to a leaf
  both sides import; that is a transparent relocation of an existing concept
  (`minimize_redundancy`), not a new authority. The principal judgment stays
  gunbc.fleet.fleet_job_principal `job_principal_reading` verbatim. One contract, one binary, one
  vocabulary; the front door is a second REALIZATION at a different demand, which DESIGN allows.
  What the probe CLAIMS narrows, said out loud: today the probe text carries the instance id and
  the gate includes ledger-group membership — both are ledger facts, needed before a reservation,
  not before "ready". The front door attests *principal + contract word*; the roster/ledger checks
  stay where they are already paid (the request path). Narrowing the claim, not widening admission.
  fail_closed: an unrecognized command never answers ready (it hands off; a bad hand-off is the
  serve's own typed refusal); a principal the rows do not admit refuses exactly as now; the
  closure budget becomes a CHECKED fact — a witness over the front door's declared admissible
  import set goes red the moment a heavy module is imported into it (the red is authorable, which
  is the guarantee-ladder precondition), plus ONE inhabitance claim running the real leg, so
  deleting the hand-off makes the request-path control fail (`witness_one_interface`).
  Memory under the seat ceiling: measured union for the front door's present import set = 136
  modules (std types/optional/algebra/measure + gunbc.cli_wire 38 + fleet_job_principal 21 +
  fleet_posix_accounts 107 + filesystem_io 48 + extdeps.exec.command 45, deduped) against 891
  today; reducible to ~25 with two leaf cuts (account rows out of fleet_posix_accounts, wire words
  out of remote_build) — closure_consolidation's own move. At 4–5 MiB/module the probe leg's peak
  lands under ~0.5 GiB. And note: a leg runs OUTSIDE any compute seat, so all of its bytes are
  unappropriated against the host pool — an 8x cut is a seat-ceiling win on its own. No resident
  bytes added.
  THE UNKNOWN I WILL MEASURE BEFORE BUILDING, on srv1, resolve-only (the
  `--function __no_such_function__` trick the entry-closure integration concern used): if the
  FIXED floor of a `gunbc run` alone exceeds 15 s, a front door is not enough and the probe's
  content must come from the client-side standing read (the stale-serving-host failure mode
  already says a client's standing against a host must come from a converge-written record, not
  from the host's answers), with the leg kept only for reachability. Measurement picks, not taste.

CANDIDATE 2 — content-addressed resolved-closure cache keyed by (revision, entry) in the fabric store.
  Ladder: legitimate for the REQUEST leg only — N legs of one host resolve the same fixed 891
  modules of the deployed checkout, so there IS a reuse obligation at their least common visible
  ancestor (the host's checkout/serve home). It must never be the probe's answer (step 1 above).
  Key: the engine already owns this identity — src/v1/stage0/src/closure_identity.rs
  `KeyInputMaterials { closure_subject: subject_digest_for_closure(sources),
  transform_content: transform_content_digest() }`, plus the requested function names. Reuse it.
  A hand-spelled `(revision, entry)` string beside it is a fork, and it misses the one axis that
  matters for "engine built at head": the engine's own content digest.
  Store: the instance's declared `materialization_store_root` (srv1: DurableHostVolume) through
  gunbc.durable_cas_file_store (VerifiedCasAttempt + flock exclusion) — never a hand-built path
  (a_slot_store_is_built_over_a_literal_path_by_any_module).
  Staleness refuses: a hit requires the exact key AND a verified artifact readback; a missing
  sidecar, a digest mismatch or an unreadable artifact RECOMPUTES — it never trusts a stale graph
  and never widens — and the refusal is typed with which axis of the key missed. Retention must
  span the obligated lifetime or the provider refuses (ladder step 3).
  Memory: wall win is real, RSS win is not — the process still needs a resolved graph in hand to
  run, so ~3.8 GiB peak stays. Engineering cost is the highest of the three: there is no
  serialization of ResolvedGraph + source indices today and no engine cache at all.
  Verdict: right second step, wrong first step. Cite your own ledger back to me here —
  gunbc.plans.debt_payoff §3: "a count floor, a widened budget or a cache is never payoff"; if
  this lands, it lands on a measured wall figure with a witness, not on "a cache exists".

CANDIDATE 3 — resident serve process per host.
  It has an existing home, so it does not need a new mechanism to be admissible: the dashboard
  already runs a resident serve (instance `serve_binary` + `serve_function` + listener, converged
  by gunbc.live_deploy.spec / tmux).
  Authority it reuses: DESIGN's Fabric/compute boundary — "a compute cell is stateless between
  grants, while a serving seat holds a loaded, quality-conditioned replica" (serving-capacity-home
  ruling). A resident remote-build serve is therefore a SERVING SEAT: it must hold its bytes
  through product.capacity.pool / pool_events `SeatRequest` and gunbc.compute.host_capacity (which
  pool, where the ledger lives), and appear in the host's declared shape
  (gunbc.runner.runner_slot_desired / gunbc.host.memory_envelope).
  Why I refuse it now, on the seat ceiling: srv1's build pool is authored at 6 × 22 GiB
  (`compute_build_memory_ceiling_default` = 23622320128; the row's note says 6 fits, 7 does not),
  and a resident warm graph is ~2–4 GiB held PERMANENTLY, outside that appropriation. The live
  fleet's runner units already drift from the declared slot shape
  (recurring_failure_mode::the_live_runner_unit_memory_shape_drifts_from_the_declared_slot, whose
  trigger — a scheduled live read of every fleet slot refusing on drift — has not run), so the
  permanent reservation would not be enforced where it is checked; and
  recurring_failure_mode::a_held_compute_seat_has_no_in_corpus_discharge refuses exactly this.
  Staleness: a resident process pins a revision by construction, which is
  recurring_failure_mode::a_stale_serving_host_re_reads_a_verb_it_does_not_know_as_another_operations_operand;
  so C owes the serve-identity / admitted-revision check per request regardless.
  Verdict: biggest steady-state win (both legs warm), refused for now with named triggers — the
  serving-seat home installed, and the fleet slot-shape drift read running.

WHAT I WILL NOT DO, stated so nobody has to ask: no Rust-side probe verb (it moves the contract
vocabulary away from gunbc.compute.remote_build — a second authority for the wire, exactly the
stale-verb failure mode); and no raising of `wall_probe` to ~300 s to make a 4-minute constant
"fit" — that makes every unreachable host cost five minutes of the roster walk and asks
deployed_tree_leg_deadline_witness to defend a number nobody can ground.

=====================================================================
(c) INTENTION AND ROADMAP NODE (proposed text only — I will not touch plans while #16859 is open)
=====================================================================
No intention covers this today: nothing under dag/gunbc/plans states remote-build serving cost.
The adjacent authority is gunbc.plans.closure_consolidation ("a consumer reaches only the domain
declarations and host authorities it actually needs"), which owns the two LEAF CUTS. I propose a
new intention for the route itself, rostered in gunbc.plan_registry and bound in
gunbc.roadmap.roadmap_intention_binding as node `remote-build-serving-bandwidth`:

  module gunbc.plans.remote_build_serving_bandwidth
  data remote_build_serving_bandwidth_plan: Plan = Plan {
    projection: PlanIsAuthorityOnly,
    slug: "remote-build-serving-bandwidth",
    title: "Remote-build serving answers what each leg asks, and only that",
    retirement: PlanRetiresWhen { condition: unbound_dissolution(description:
      "Every remote-build leg completes inside its declared per-kind ceiling on a serving host,
       with the probe's closure budget a checked fact and the request leg's resolve discharged by
       a provider keyed on the engine's own closure subject digest — or the route is retired by
       the fabric dispatch it is converging on.") },
    body: ...
  }

  GOAL. A remote-build leg costs its own demand. A probe is answered from the facts a probe can
  see; a request pays the resolve a request needs; nothing pays a closure it does not consume.

  WHY (displaced cost, dated). srv1, 2026-11-25 (calm-wren-314): every leg — including a probe
  that answers a constant — pays `resolve_entry_graph_inclusive` 248,554 ms over 895 modules at
  max_rss 3.8 GiB, because the serve entry module's import closure (891 modules) is resolved
  before any function runs. The probe leg's ceiling is 15 s
  (gunbc.fleet.fleet_known_hosts_anchor fleet_ssh_leg_deadline wall_probe), so a probe cannot
  answer inside its own deadline and the roster walk ends RemoteBuildNoHostAvailable. This is the
  largest single bandwidth cost on the remote-build route.

  DONE-WHEN.
   1. A probe leg over the remote-build key completes inside the declared ProbeLeg ceiling on a
      serving host, witnessed by a claim that runs the REAL leg, with the pre-entry receipt
      printed and closure_files under the front door's declared budget.
   2. The serve contract's vocabulary has one home: probe word, header and endings read from one
      leaf by client and serving side alike; a witness refuses a re-spelling.
   3. The serving front door's import closure is a declared budget with a discriminating red
      control (import a heavy module into it and the claim goes red), not a number in prose.
   4. Whatever reuse exists on the request leg is discharged by a provider whose key is the
      engine's closure subject digest + transform content digest, never a hand-spelled
      (revision, entry); miss, mismatch or unreadable artifact recomputes with a typed cause.
   5. Each leg's peak RSS is reported by the pre-entry `process_cpu ... max_rss_kib` line the leg
      already prints, so the seat-ceiling reading stays an observation, not a claim.

  NOT THIS INTENTION: the resident serving seat (fabric's, and refused until the seat home and the
  slot-shape drift read land), and the engine's serialized-graph provider (closure_consolidation's).

=====================================================================
STEP 2, ON YOUR GO
=====================================================================
Order, with witnesses + red controls, engine built at head, tallies posted:
  1. srv1 resolve-only baselines (serve entry, candidate front door, std-only control) → decides
     front-door-fits vs client-side-standing. ~3 measurements, each under a minute.
  2. Wire-vocabulary leaf move + front door module + closure-budget ledger + witnesses
     (probe answers ready in-budget; principal refusal still fires; hand-off runs the real serve —
     deleting the hand-off must fail the request-path control).
  3. Forced command points at the front door; the serving converge re-installs authorized_keys
     through the same authority (remote_build_authorized_keys_lines); witness that a stale front
     door cannot answer ready for a request it cannot hand off.
  4. Before/after tallies (wall + max_rss + closure_files) posted to the work item, and a note to
     calm-wren-314 so wall_probe can shrink later rather than grow now.
Two things I need from you: (i) the go; (ii) whether the probe's claim may narrow to
"principal + contract word" as argued above — that is the one judgment call in here that is yours
rather than the model's. If you would rather the probe keep the roster read, the honest cost is
~343 modules and I will need the standing-read variant instead.

## Pause state (2026-11-26 ~07:40Z), after #17114 merged at 24ce2b4291b2

DONE: PR #17114 merged (door). srv3 OBSERVE succeeded twice; srv3 CONVERGE applied all three
arms (forced command, serve_binary=160f8a56 = that run's release-bins gunbc hash, checkout=a8034c1)
then exit-1ed on the recover-catalog seat census (unrelated fleet-seat defect, routed to
crisp-pike-295). srv3 therefore HAS THE DOOR.

HELD BY PARENT PAUSE (clever-raven-401, msg_fec683a2): no fleet-converge dispatches, no new
one-issue PRs, srv4 observe + shadow-gate probe read on hold. Blocker list B1..B12 sent to parent
(full text in the message log). Key entries:
 - B1 the census refusal (dag/gunbc/compute/attempt_lifecycle.dag) blocks every entry importing
   gunbc.compute.work_provider_local -- including the REQUEST arm. The door's probe arm imports
   none of it, so probes are unaffected; srv3 compiles still refuse.
 - B4 hosted-runner-shadow-gate.yml remote steps are broken: GIT_CONFIG_KEY_* =
   credential.https://github.com/helper is a malformed git key (must be
   credential.https://github.com/.helper); 6 occurrences (lines 110/112/134/136/157/159); the
   client dies at snapshot `git add -A` before any leg. Verified locally with git 2.47.3.
   Evidence: run 45774944663 step 7. No earlier workflow_dispatch exists in the last 100 runs.
 - B6 srv4 never observed: "hostname converge dispatch admission" step dies at its 5-min timeout
   resolving 1585 modules (ms=301548, max_rss 6,084,572 KiB) -- twice (45770097040, 45773631715).
 - B9 the CLIENT entry pays the same class per process (906 files / 68 s / 3.66 GiB; calm-wren:
   832,637 ms). Nobody owns the client-side cut.

REMAINING FOR MY ITEM (waiting on the plan):
 1. srv3 probe read behind the door (wall, closure_files, max_rss) -> the figure for
    serving_probe_cold_answer_read (fleet_known_hosts_anchor.dag:1411, Second=240) and the
    ServingProbeLeg 600 s shrink that calm-wren-314 lands. Blocked by B4; dispatch form is proven:
    gh workflow run hosted-runner-shadow-gate.yml -R gunb-ai/gunbc --ref main -f pattern=...
 2. srv4 observe + converge (door not installed there).
 3. Tell crisp-pike-295 when srv3's converge settles (told them the door is installed + the ask for
    host-side [pre-entry] lines; they still need B1 fixed for their compile).

## Addendum 2026-11-26 (after the regroup PAUSE)

PARENT DIRECTIVES NOW IN FORCE (clever-raven-401):
 - (a) GO  shadow-gate git-config survey-infrastructure PR  -> gunb-ai/gunbc#17129, parent merges.
 - (b) HOLD the srv3 probe dispatch: orphan on srv3 (unit gunbc-compute-d6481839…, since 06:07,
   20 GB of the 25 GiB slice) + census bug A (refused reservations recorded terminal:recovered
   with granted_kib, then charged). Dispatch only after the orphan ends AND bug A is fixed.
   crisp-pike-295: the census defect (never-charged terminal slots folding the census to unknown)
   is gunb-ai/gunbc#17136 (approved). After it merges and srv3 converges, the door answers the
   probe with no census in the way.
 - (c) srv4 observe/converge HOLD. (d) srv1 HOLD.
 - PLAN ITEM accepted, NOT started (wait for the plan PR): there is NO host-side wall-clock bound
   at all -- neither before start (srv3's serve process waited 17 h: SSH session from srv2
   192.168.1.201 opened 2026-11-25 12:46 still established; serve started 05:44 on 11-26) nor on
   run (4 h 41 m orphan). Shape agreed: the ceiling rides in the wire words and is enforced at
   remote_build_serve_hand_off, plus a bound on how long a request may wait before starting.
   Witness = green claim that an overrunning child ends; red control = removing the bound
   reproduces the orphan.
 - ServingProbeLeg shrink (the 240 s read at fleet_known_hosts_anchor.dag:1411 -> 600 s ceiling)
   will be MINE as a plan item, sized from a SERVING-HOST read, never from my box.

MERGED SINCE THE DESIGN REPORT:
 - #17114 (the front door) at 24ce2b4291b2. srv3 has the door; srv1/srv4 do not.
 - #17132 at d20fa377f6c (parent merged): dropped `claim_run: false` at
   dag/gunbc/site/gunb_ai_pages_workflow.dag:88 -- that one stale argument made
   gunbc.instruments.generated_artifact_gate refuse at resolve (20 min to fail), so no generated
   projection had a working declared repair route. Red control (refusal at a93c6559c97) and green
   arm (main_wet_one exit 0 at head) are both posted on the PR.

#17129 AT NOW (branch session/loyal-badger-240-shadow-gate-git-config):
 - 70c008ca150 fix(witness): the GENERATOR owns the key (review 98880's point: the first commit
   had patched only the generated projection) -- both GIT_CONFIG_KEY_* literals in
   dag/gunbc/witness/hosted_runner_shadow_gate_workflow.dag::shadow_gate_push_credential_env now
   read credential.https://github.com/.helper.
 - ff6176950fa test(witness): new claim
   the_push_credential_helper_key_is_the_spelling_git_accepts (module now 11 test fns). No claim
   in the repo asserted the key at all (git grep credential.https -- dag/test = empty); this one
   counts the rendered workflow: 3x KEY_0 + 3x KEY_1 of the good spelling, 0 of the old.
   Counts pre-verified against the committed projection with grep -o -F (3 / 3 / 0).
 - Declared-route regen evidence already posted: generated_artifact_gate main_wet_one --arg
   path=.github/workflows/hosted-runner-shadow-gate.yml -> exit 0, Filesystem.Write 11136 bytes,
   git status clean; resolve ms=1391228, closure_files=3706, max_rss 13.3 GiB.
 - PENDING: green tally (11 claims) at ff6176950fa with the engine built at that head
   (gunbc ff6176950face...-dirty), then the red arm (revert the two literals -> the new claim must
   NOT hold), then post held/did-not-hold/unobserved on the PR and tell parent.

CI READING NOTE (cost me time): on a branch push the hosted-runner-shadow-gate "configuration" job
runs ONLY its guard step (sets configured=false; remote steps skipped) -- its log is ~3.7 KB and has
no claim output. The claim lane in CI is the separate "witnesses" workflow. gh run view --log can
also return a truncated log; the jobs/<id>/logs endpoint is the reliable one.
