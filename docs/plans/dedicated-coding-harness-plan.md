# Dedicated Coding Harness — scope

Project ID: DCH

Repository anchor: gunb-ai/gunbc

Baseline: main@de2f5f86346, read 2026-09-01.

Operator direction (2026-09-01): serve a large open model on the DGX Sparks and drive a minimal
coding harness against it, retiring the Claude, Codex and Cursor provider runtimes. Written from
the ground up in `.dag` and Rust. **DCH and the dedicated harness must not import, vendor, depend
on, or cite `ctrl` as authority.** Stated at that grain deliberately: the wider claim would be false
of the repository today, because `extdeps.ctrl.gunbc_pin` already declares an `ExternalAuthority`
pointing into `gunb-ai/ctrl` and compares the host pin against the ctrl pin. That row is out of this
lane's scope and is not to be touched here. Spark parity is to be handled by fleet convergence
rather than by hand. RLM closes first; this lane starts after it.

## 0. What is measured, and what is assumed

Every claim below is either a read of this tree at the stated baseline or a live probe taken
2026-09-01. Where a fact came from another session it is attributed, and where it is unverified it
says so. Three earlier readings in this lane's own correspondence were stale or truncated, so the
provenance is part of the scope rather than beside it.

## 1. Terminal

Success is the RLM terminal procedure, unchanged, executed with our harness selected as the
provider realization:

> From a clean revision R on main, clicking Launch on the roadmap canary creates exactly one
> attempt, one worktree, **one harness process of ours**, and one durable attempt record; the
> worker performs only the canary's exact change; verification evaluates the pinned oracle against
> the exact attempt head; publication records a receipt; the child unblocks on acceptance.

This lane authors **no new acceptance procedure**. RLM already owns a 14-step terminal that is
provider-agnostic in every step but one, and reusing it is what keeps this from being graded on its
own homework. `docs/plans/roadmap-launch-mvp-plan.md` is that procedure.

No substitute terminal counts. A harness that completes a turn against a Spark in isolation is a
component probe, not this terminal.

**And this terminal is necessary without being sufficient.** The canary is one small change: it
will not fill a context window, will not run a build long enough to meet a tool timeout, and will
not sit through a convergence of the serving unit. So the classes in §5 cannot be discharged by
passing it, and a green on the canary is a green on the easy path. Each class in §5 owes its own
evidence, named there. Stating this is the difference between a terminal and a demonstration.

## 2. Current-state ruling

### Already usable

- **The seam exists and is a single-variant coproduct.** `gunbc.provider_interface_binding`
  declares `ProviderControlInterface = CodexAppServer {}`. Adding a harness is a variant addition
  at a modeled seam. That module's own annotation already separates the two senses of "provider" —
  the runtime we install and control, versus the inference backend it sends requests to — and names
  `gunbc.spark.serving_desired` `spark_serving_local_endpoint_url` as the seam where the two would
  silently fuse.
- **The serving runtime speaks the shape the harness needs, natively.** Probed against
  192.168.1.225 on 2026-09-01: Ollama 0.32.9 answers `POST /v1/messages` with an Anthropic-shaped
  response — content blocks including `thinking` and `tool_use`, `stop_reason: "tool_use"`, and a
  `usage` object — for a request carrying a tool with a JSON input schema. `/v1/chat/completions`
  and `/api/chat` also route. So no translating proxy is required for gunbc to drive it.
- **Some of the wire shape is already modeled.** `extdeps.llm.anthropic` (378 lines) carries tool
  result blocks, image sources and model specs; `extdeps.llm.anthropic_rest` names the `/v1/messages`
  path and body.
- **The convergence spine reaches spark serving.** `gunbc.fleet.fleet_converge_plan` imports
  `gunbc.spark.serving_membership`, `serving_realization` and `serving_execution_schedule`, and
  declares `fleet_converge_spark_serving_row_executions` / `_row_effects`.
- **Both Sparks are reachable and serving.** `/api/version` answers 0.32.9 on 192.168.1.225 and
  192.168.1.226; `/v1/models` returns the same seven ids on each.

### Prevents it today

- **The Sparks are not enrolled in the fleet.** `fleet_intent_network.endpoints` lists srv1–srv4
  only, and `srv5`/`srv6` appear in neither it nor `gunbc.fleet_intent`'s `ComputeHost` list. In
  that module **membership IS enrollment**, so this is not an omission to be patched around: it is
  the reason parity is managed by hand. The module's own note records that the standing reason has
  already changed from "there is nothing to enroll" to "enrolling is a decision nobody has taken
  yet."
- **"The models resident" is not representable.** `spark_serving_desired_manifest` governs exactly
  one member; the other six resident models are outside that authority and nothing in gunbc owns
  them. The structural tell, from eager-pike-541: the materializer carries a single manifest, digest
  and blob closure, so no carrier could hold seven. Nobody can state, check, or drift-detect the
  difference between the model we converge and the models the hosts hold.
- **Desired and observed disagree on both hosts.** The deployed user unit still describes
  `gpt-oss:20b`, predating the #9859 amendment to 120b, and carried no context-length line at all.
  Two independently declared desired values, un-rendered on both hosts. **This survives #9960**
  (see §2.1): that PR completed the DESIRED side, and a complete desired value is not a converged
  host.
- **There is no generic inference interface.** `extdeps/llm/llm.dag` is 11 lines and
  `llm_contracts.dag` is 14 — anchors only. Every wire fact lives in a per-vendor module, and
  tool-calling is modeled only inside `openai.dag` and `cursor_stream.dag`. What exists in
  `anthropic.dag` fuses **shape** with **vendor**, which DESIGN's external-upstream decomposition
  separates: Ollama implementing the Anthropic wire shape is precisely the shared-standard case, so
  the shape belongs in its own module with the two implementations citing it.
- **Whether the spine can APPLY a serving row is unproven.** The `converge` CLI verb is unwired
  deliberately — rebuilding it would make the interpreter load-bearing for a new capability while
  two lanes delete it — and its refusal names the live successor route. But this lane's own wet plan
  run came back `PartiallyApplied` with apply refused. That is what RLM-2b is currently measuring.

### 2.1 Changes since the baseline — the concurrency axis is RESOLVED ON MAIN

The baseline above is fixed at `main@de2f5f86346` and is not rewritten as main moves; completed work
is dispositioned here instead. Recorded 2026-09-01, after the controlling reviewer identified that
this plan's active claims had been falsified by a merge that landed while it was in review.

**#9960 (`7810e68b3ea`, ancestor of current main) resolves the concurrency-axis item outright**, and
verified here rather than relayed:

- `extdeps.ollama.server_env` declares `OllamaNumParallel` → `OLLAMA_NUM_PARALLEL`, and
  `spark.serving_unit_render` now emits **four** environment axes, binding
  `ollama_num_parallel_env_assignment` to the spec's slot count. The axis is no longer absent.
- `spark_serving_desired_serving_slots_value` carries a desired count of 4, so the axis has a
  desired value and is drift-reportable — the specific consequence this plan said was missing.
- The false cost model is corrected at the desired value, in the direction this plan predicted:
  the count does **not** divide the window, each slot receives its own full window, and the
  per-slot price is recorded as a property of the **realization** rather than a fleet-wide constant
  — the per-realization objection this plan raised is answered by a declared §4b drop naming that
  exact subject.

**One defect found while verifying it, and it is not #9960's to fix by this lane.**
`gunbc.spark.serving_desired` still carries an *earlier* annotation block stating in the present
tense that concurrency "is not modeled anywhere in desired state", that the two facts trade "because
the slot count divides the context window", and that the concurrency row "is P1b and is deliberately
not smuggled in here". All three are now false, and they contradict that same module's own value and
annotation roughly 140 lines below. That is a §3 meaning fork inside one authority — a stale
annotation is data the substrate cannot check, so nothing reds. **Reported, not repaired here**, and
out of this lane's scope.

**What this does NOT resolve.** #9960 made the state representable, declared and renderable. It did
not read either host back. The observed-versus-desired receipt below stays open, and so do the two
outstanding operator hand-edits it must account for.

### Explicitly off the critical path

`ctrl`'s harness and router are **prior art, not a dependency and not a citation**. Two of their
findings are worth re-deriving deliberately rather than inheriting, because gunbc must establish
them on its own evidence: that a cold large model can exceed a default HTTP client's header/body
timeout, and that an assistant turn must be echoed back verbatim — thinking blocks included — with
all tool results for one turn batched into a single user message. Treat both as **hypotheses with a
predicted failure**, not as facts on loan.

Two more joined that list on 2026-09-01 from the transcript §5 reads, on the same terms: that a
streaming request may terminate a response early where an otherwise identical non-streaming request
returns it complete, and that an assistant turn may stop at `end_turn` having announced work it did
not perform. Neither is established. The second has exactly one observation and the first has none
— their session's last planned step, isolating streaming as the single variable, was never run.

## 3. Serial gate chain

Only one gate mutates at a time. A later gate may author inert types and fixtures but may not
activate production rows before its predecessor's receipt is accepted.

**A gate does not complete while a §5.2 question that owns one of its dispositions is open.** This
is a precondition of the gate, not a note beside it: §5 exists to answer these classes *in advance*,
and a plan that lets a gate finish while its own identified fail-open is unadjudicated has recorded
the class instead of handling it. The bindings, and each is repeated in the gate itself:

| Question | Gate it blocks | What the gate may not do until it returns |
|---|---|---|
| Q3 — converge restarts the serving process mid-turn | **DCH-0** | complete without a drain/lease, or without a declared §4b drop naming the restoration trigger |
| Q1 — stop-the-line boundary | **DCH-2** | fix the failure-arm partition |
| Q2 — typed disposition for a degenerate turn | **DCH-2** | claim a terminal disposition is total |
| Q5 — is a process exit status a carrier | **DCH-2** | fix where the attempt record is written relative to process death |
| Q4 — what n=1 under survivorship licenses | none | it governs how §2's hypothesis list is read, not a gate's completion |

An open question is not a reason to stop work inside a gate; it is a bar on that gate's **receipt**.

### DCH-0 — Enrol the Sparks, and reconcile the rendered unit against the live hosts

**Question owned:** what does the fleet know about these two machines, who says so, and does either
host actually carry what desired state now declares?

The parallel axis and the cost model **left this gate on 2026-09-01**; #9960 performed both, and
§2.1 records the verification. What remains is enrolment and the live reconciliation.

- Enrol srv5/srv6 in `fleet_intent_network.endpoints` and `fleet_intent`'s `ComputeHost` list — the
  act the address correction deliberately did not perform. (The `ComputeHost` half is DCH-0c; see
  below.)
- Reconcile the rendered unit on both hosts against desired state, which is now complete on four
  axes where it was complete on three.
- **Answer Q3 before this gate's receipt is accepted, and carry the answer in a row.** Placing the
  serving unit under convergence means a converge can restart the serving process, and a restart
  mid-turn is the failure that dominated the relayed transcript in §5. This gate may not complete
  while that is neither prevented nor declared. The admissible endings are a drain or lease so a
  converge cannot land against an in-flight turn; or, if the drain is deferred, a §4b(3) row whose
  trigger names the drain capability — never silence, and never a note that a restart is unlikely.

**Receipt:** an observed-vs-desired comparison of the rendered unit on both hosts, not a return
code. The rendered unit is the only member of the must-move-together set that produces **no failure
signal** when stale — ref, manifest, digest and closure each raise a checksum event, while a stale
unit is silent, which is how both hosts drifted on two axes unreported. Two hand-edits are
outstanding against that receipt: the context length raised today, and `OLLAMA_NUM_PARALLEL=4` set
on both hosts, each under operator instruction. **#9960 does not discharge this and it sharpens
it**: a hand-set value on a host and a declared value in the corpus agreeing by coincidence is
exactly what a silent carrier cannot distinguish from convergence, so the readback is the whole
receipt.

**Enrolment is inert today, and that is the risk rather than the reassurance.**
eager-pike-541 looked for the executor rather than arguing from principle: no
`ctrl-fleet-converge` timer or service exists on either Spark in either scope, and
`gunbc.fleet_converge_timer` says of itself that it "remains the naming and cadence authority; it is
no longer a renderer" — its installer chain died with `ci.yml` in #8283. So enrolment cannot cause
an apply, because nothing on those hosts is scheduled to run one. The failure mode to guard is
therefore not prematurity but **a row that reads as an outcome**.

**The consumer census, which was the open item on that reasoning, is done and it sharpens the gate.**
Of 57 non-test modules importing `fleet_intent_network`, the number that read the `endpoints` list
or `fleet_intent_network_topology()` is **zero** — production modules consume individual endpoint
rows *by name* (`bmc_virtual_media`, `host_identity_access` take `srv1_host_lan_endpoint` and
friends). The topology function has exactly one consumer in the tree:
`test.claim.fleet.fleet_intent_network_witness_test`, asserting
`list_length(items: fleet_intent_network.endpoints) == 11`.

Two consequences. First, **list membership is not the load-bearing act** — authoring the srv5/srv6
endpoint rows that named consumers can reach is. Second, enrolment will turn that witness red at
11 → 13, and the right response is not to bump the literal: a count copied from the current tree is
the change-detector DESIGN §5 names, and completeness here is an identity join, not a count
equality. Enrolment is the occasion to fix the oracle, and that repair belongs in this gate.

**ENROLMENT IS TWO ACTS, NOT ONE, AND ONLY ONE OF THEM IS INERT.** The census above covers
`fleet_intent_network.endpoints`. `fleet_intent`'s `fleet_intent_known_hosts` is its opposite —
censused by eager-pike-541 and verified here — with three production consumers:

- `gunbc.generated_artifact` maps the list to `RunnerHostSudoersArtifact`, so enrolment **mints two
  new generated artifacts**; the rows cannot land without a regeneration in the same commit or the
  drift gate reds. Mechanical consequence, mechanical receipt.
- `gunbc.runner.runner_host_deploy` `admit_runner_host` returns `RunnerHostUnenrolled` for any host
  absent from the list. Its annotation calls itself **the enrollment wall** and is explicit that it
  is construction rather than a check: no function in the module takes a bare `RunnerHostDeploy` and
  yields a command, so "run the installer on an unenrolled host" is unrepresentable rather than
  discouraged. It prices the srv4 host-convergence OOM — converge against an unenrolled host derives
  nothing for it, or stamps drop-ins carrying another host's widths — and names the enrolled row as
  exactly where `runner_deployment_plan`'s conservation wall gets the host RAM to check slot caps
  against. **So enrolling srv5/srv6 removes a fail-closed refusal that currently protects them.**
- `gunbc.fleet_converge_apply` finds hosts by identity in that list, so enrolment is precisely what
  makes a Spark reachable by the apply path this lane measured as `PartiallyApplied` / refused.

The inertness argument had two independent legs — no executor, and no consumer. `ComputeHost`
knocks out the second. **Only the executor leg survives**, and it is the leg that changes the moment
anyone installs the timer.

**So DCH-0 lands the endpoints half only.** Author `srv5_host_lan_endpoint` / `srv6_host_lan_endpoint`
as rows the named-import consumers can reach, add them to the list as the authority's completeness
statement, repair the count oracle. The `ComputeHost` half is **DCH-0c**, and it is a decision
rather than a row: it needs the host facts the conservation wall consumes (RAM at minimum), the two
sudoers artifacts regenerated in the same commit, and someone stating out loud that the enrollment
wall is coming down for these two machines on purpose.

Note the row/list split holds on both sides and inverts between them: `srv1_host` and `srv2_host` are
imported by name in at least `os_install_actuator_selection`, `deployed_intent_v1`,
`nbd_proxy_virtual_media_install`, `ci_deploy_access` and `srv3_install_media_fetch`, so authoring
`srv5_host`/`srv6_host` rows is separable from list membership here too — except that here the LIST
is the load-bearing half and the row is the quiet one.

**Owner: eager-pike-541**, agreed 2026-09-01.

### DCH-0b — Make residency representable

**Question owned:** what is a resident-model fact, and who owns the six nobody converged?

Split from DCH-0 at eager-pike-541's request, and the reason is the right one: enrolment and the
parallel axis both write a value into a carrier that already exists and has a shape, whereas the
resident set has **no carrier at all**. Someone must first decide whether a resident-model fact is
desired state or observation, whether it is per host or per fleet, what its identity is when the
same weights appear under two refs, and what a drift between the converged member and the resident
set *means* when nothing put six of those models there through gunbc. That is design, and folding
it into a gate of row-writes would make the row-writes wait on it.

The structural tell that no carrier exists: the materializer carries a single manifest, digest and
blob closure, so nothing in it could hold seven.

### DCH-1 — Hoist the wire shape off the vendor

**Question owned:** what is the interface, and which module owns it?

- One module for the messages/tool-use shape, cited to its specification; `extdeps.llm.anthropic`
  and the Ollama serving surface become two implementations that reference it.
- Tool-calling gets a shape here rather than a third copy inside a vendor module.
- This is a replacement migration at the root, not a new fork: the vendor module's shape rows move,
  they are not duplicated.

### DCH-2 — The harness, in `.dag` and Rust

**Question owned:** can we drive a turn to a terminal disposition and report it?

- The loop: post a request, and while the response stops on tool use, execute the tools and post the
  results back. Four tools is the starting surface: run a command, read, write, edit.
- **The tool surface is where §5.1 row 7 lands, so it is specified here rather than left to the
  implementer.** Every quirk in the relayed transcript was a real semantic of this surface that
  nothing announced, so the rule for all of them is one rule: **a tool's contract is carried in the
  tool's declared shape, and any limit it enforces is reported in its result rather than inferred
  from a truncated or missing one.** Concretely, six dispositions this gate owes:
  - **Working directory.** Either a command's working directory is part of the call's declared
    input, or the surface states that each call is independent — the failure to avoid is a caller
    that happens to comply, which is what theirs did, by luck and unprompted.
  - **Duration.** A limit that kills a build is a real refusal and must arrive as one: a typed
    timeout disposition naming the limit, distinguishable from the command's own failure. A default
    chosen for a chat turn is the wrong default for a compile.
  - **Input.** No stdin means anything that prompts hangs until the limit kills it, and the operator
    sees a timeout rather than a prompt. Either the surface supplies input, or a prompt-shaped hang
    is typed as its own disposition.
  - **Truncation.** Keep their shape: a cap that **announces** it, so a large result is
    distinguishable from a complete one. This is already right and is adopted deliberately.
  - **Edit matching.** Keep their shape: refuse on zero or multiple matches rather than patching the
    first. Construction over validation, and already right.
  - **Context exhaustion.** A full window must be a typed refusal, not a turn that begins to fail.
    Compaction is a later capability; the disposition is owed now, because without it exhaustion
    presents as the model getting worse.
- Preserve what the codex realization learned expensively, because these are provider-independent
  and were paid for once: the three separated layers (durable goal, execution lease, controller),
  the exactly-once command identity, and the trap that an idle thread never authorizes a start.
- A harness that **reports** its own status retires the observation layer that dominates every
  existing provider module — guessing which transcript belongs to a session, inferring state from a
  foreign schema. That is where the simplification is, and it should be measured, not asserted.

### DCH-3 — Bind it at the seam and run RLM's terminal

- A second `ProviderControlInterface` variant, its receipt types, and the belt binding.
- Re-run the RLM terminal with the variant selected. That is the acceptance test.

### DCH-4 — Retire the three provider runtimes

Delete-first at the root, per the replacement-migration doctrine, once DCH-3's receipt is accepted.
The population, to be counted rather than estimated at the time: in `extdeps/llm` the anthropic,
openai, cursor and codex families; in `gunbc` the codex supervised-turn and runtime modules, the
cursor SDK modules, provider account and standing, and the Claude setup-token enrolment. The auth
surface goes with them — it exists to solve a problem that a self-hosted endpoint does not have.

## 4. What this lane must not do

- Import, vendor, or cite `ctrl`.
- Start before RLM's terminal receipt is accepted.
- Enrol the Sparks as a side effect of harness work — DCH-0 is a separate gate with a separate owner.
- Bump `fleet_intent_network_witness_test`'s endpoint-count literal to absorb enrolment. Repair the
  oracle or leave it red; a count copied from the tree it measures is not one.
- Treat "it completed one turn" as evidence for anything but that.

## 5. Adversarial review — the failure classes this lane must answer in advance

**Provenance, stated once.** On 2026-09-01 the operator relayed a transcript of the `ctrl`
mini-agent lane debugging its own harness against these same Sparks, and directed that DCH be made
to answer it before it starts. Everything in this section is **second-hand and unreproduced here**.
It is admitted on exactly the terms §2 sets for the rest of that lane's output: each row is a
**hypothesis about our design with a predicted failure**, never a fact on loan, and none of it is
citable as evidence. What their session buys us is not findings — it is a cheap enumeration of
where a harness of this shape breaks, produced by someone who paid for it.

The value is concentrated in one property: **every class below was operator-visible as something
other than itself.** A harness throw read as a clean exit. A dead session read as a working one. A
control-plane restart read as a Spark fault. An inflated rate read as fast hardware. A model that
stopped early read as a truncated message. That is one class, not five, and it is the class DESIGN
§5 exists for — so the obligation this section places on DCH is not "handle these bugs" but
**produce a disposition that cannot be mistaken for a different one.**

### 5.1 Classes, and the gate that owns each

| # | Class | Observed as | Owner |
|---|---|---|---|
| 1 | A harness fault reaches the operator as success | process exited 0 for an hour after a first-write throw | DCH-2, and Q5 below |
| 2 | One throw destroys unrelated work | any error in the loop took down the whole container | DCH-2, and Q1 below |
| 3 | Self-reported status that no observer can resolve | frozen at `idle` while the harness emitted `working` throughout | DCH-2 |
| 4 | A backend restart kills every in-flight turn | control plane and inference router shared a process | **DCH-0**, and Q3 below |
| 5 | A turn ends without doing the work and nothing types it | `end_turn`, one heading, after 50k input | DCH-2, and Q2 below |
| 6 | An instrument that fails toward a flattering number | 687 tok/s the hardware never reached, from KV reuse | DCH-2 |
| 7 | Tool-surface semantics that are real and unannounced | `cd` not persisting; a 120s SIGKILL; no stdin; no compaction | DCH-2 |
| 8 | A repair whose blast radius exceeds the bug's | a failed `chown` under `set -e` blocked every spawn on the node | this lane's own discipline |
| 9 | Evidence filtered by survivorship | one completed final turn existed; the rest died of other bugs | Q4 below |

Row 3 is the one that most directly touches a claim this plan already makes. DCH-2 says a harness
that reports its own status retires the observation layer. Their harness **did** report its own
status, correctly, the entire time — and the operator still saw a frozen session, because the
report's *transport* failed independently of the report. So self-reporting is necessary and is not
the simplification on its own; the plan's existing instruction to **measure** that claim rather
than assert it is upheld, and this is what it will be measured against.

Row 6 is the same defect this plan already corrects in DCH-0 for a different subject: a plausible
number that nobody could re-derive. DESIGN §6 governs both — name the instrument, never transcribe
its output. A harness of ours may not report a rate it cannot ground, and where a quantity is not
measurable the honest render is a refusal to render, not a zero.

Row 7 was a single line in DCH-2 ("four tools: run a command, read, write, edit"), and that line is
where every operator-visible quirk in their lane lived. **It is now specified in DCH-2 as six owed
dispositions** — working directory, duration, input, truncation, edit matching, context exhaustion —
rather than deferred by a forward reference, since a promise to expand a brief elsewhere is the same
unreachable route this plan refuses in others. Two of their choices are adopted deliberately because
they are already the right shape: an output cap that **announces** its truncation, so a large result
is distinguishable from a complete one; and an edit that **refuses on zero or multiple matches**
rather than patching the first, which is construction over validation.

Row 8 is not about the harness. It is a note to this lane's own reviewers: their repair for a
mini-agent-only failure aborted bootstrap for every session on the node. A fix that widens the
failure population is a regression regardless of the class it closes.

### 5.2 Five questions sent for adjudication, 2026-09-01

These are **open**, and this section will not be treated as settled until they return. They were
sent to the controlling reviewer rather than decided here because each one is contestable and three
of them can change a gate.

1. **Where is the stop-the-line boundary inside a coding harness?** DESIGN §5 says a failure arm
   must refuse, never widen — and their harness refuses maximally, which is what destroyed the
   sessions. The reflexive repair, "survive your own errors", has the shape of an absorbing
   fallback, so it is not adopted by reflex. Proposed discriminator: a **tool** failure is an
   observation and returning it typed to the model is the answer, not a widening; a **harness
   invariant** failure stops the line. Ruling wanted on the boundary, and on which side a tool
   timeout and an unexpected stop reason fall.
2. **Can a degenerate turn carry a typed disposition at all?** "Shorter than N is degenerate" is
   precisely the view-read-as-population defect that landed in #9946. What may be typeable without
   a threshold: the turn ended while an **announced and unperformed** intent stands — a structural
   fact about the last message, not a length.
3. **Convergence is their router restart.** DCH-0 places the serving unit under fleet convergence;
   converging it restarts the serving process; so this lane imports their worst open defect **by
   construction**, into the layer it chose on purpose. Candidates: a drain or lease so a converge
   cannot land mid-turn; or DCH-2 surviving a mid-stream backend restart, which needs real design
   because replaying a partially streamed response can duplicate work; or a declared §4b drop with
   a restoration trigger. Position taken into the ruling: the drain is the construction, and until
   it exists the drop is what honesty requires.
4. **What does n=1 under survivorship license?** The degenerate stop has one observation, and the
   sample is survivorship-filtered rather than merely small. Do these two join the
   hypotheses-with-a-predicted-failure list in §2, or does each first owe its own instrument?
5. **Is a process exit status a carrier?** Their operator read `0` for an hour; the zero was a
   watchdog's, not the harness's — a refusal typed as success at a boundary that can only carry an
   integer. Preferred answer: the exit status is **not** a carrier, the durable attempt record is
   the only disposition, and the process boundary is declared lossy. That preference imposes a real
   ordering constraint on DCH-2 — the record must be written before the process can die.

### 5.3 What is already added to §4 regardless of the rulings

- Do not report a measurement the harness cannot ground; render a refusal where a quantity is not
  measurable, never a zero and never a back-computed figure.
- Do not treat a green canary as evidence for any class in §5.1.
- Do not land a repair whose failure population is larger than the one it closes.
