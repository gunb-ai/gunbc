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
- **Concurrency is unmodeled, and it is the binding constraint.** `spark.serving_unit_render` emits
  exactly three environment axes — `OLLAMA_HOST`, `OLLAMA_MODELS`, `OLLAMA_CONTEXT_LENGTH`.
  `OLLAMA_NUM_PARALLEL` is absent, and unset means Ollama serializes. eager-pike-541 measured the
  consequence: a 350-token probe queued behind a ~61k-token prefill running at a healthy
  ~290–330 tok/s. N concurrent harness sessions against one host is a queue, not N sessions. An
  unmodeled axis has no desired value, so it cannot even be reported as drifted. The corpus also
  carries a **false cost model** for the axis it is missing — see DCH-0.
- **Desired and observed disagree on both hosts.** The deployed user unit still describes
  `gpt-oss:20b`, predating the #9859 amendment to 120b, and carried no context-length line at all.
  Two independently declared desired values, un-rendered on both hosts.
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

### Explicitly off the critical path

`ctrl`'s harness and router are **prior art, not a dependency and not a citation**. Two of their
findings are worth re-deriving deliberately rather than inheriting, because gunbc must establish
them on its own evidence: that a cold large model can exceed a default HTTP client's header/body
timeout, and that an assistant turn must be echoed back verbatim — thinking blocks included — with
all tool results for one turn batched into a single user message. Treat both as **hypotheses with a
predicted failure**, not as facts on loan.

## 3. Serial gate chain

Only one gate mutates at a time. A later gate may author inert types and fixtures but may not
activate production rows before its predecessor's receipt is accepted.

### DCH-0 — Enrol the Sparks and model the parallel axis

**Question owned:** what does the fleet know about these two machines, and who says so?

- Enrol srv5/srv6 in `fleet_intent_network.endpoints` and `fleet_intent`'s `ComputeHost` list — the
  act the address correction deliberately did not perform.
- Add `OLLAMA_NUM_PARALLEL` to the rendered unit, or declare in a row why serialization is desired.
- **Correct the cost model while adding it.** `serving_desired` states that context length and slot
  count "trade against each other inside one memory budget, because the slot count divides the
  context window." Measured by eager-pike-541 on idle spark-3bd5, same model, only the slot count
  changed: 1 slot gives `context_length` 1048576 at `size_vram` 88,865,253,620; 2 slots gives the
  same 1048576 at 90,543,761,652. **The window is not divided.** Each slot gets its own full window
  and the second cost +1,678,508,032 bytes — slot count multiplies KV, it does not divide context.
  On this hardware concurrency is close to free and serialization was the expensive thing. The
  sentence is deleted, not softened; the measured per-slot cost goes behind the desired value.
- **The per-slot number does not generalize.** 1.68 GB/slot is a DeepSeek MLA figure; a non-MLA
  model pays more. A fleet-wide slot count measured on one realization is the same overreach as a
  fleet-wide context ceiling measured on one realization. Either the desired value is
  per-realization, or a row says why one number covers the roster.

**Receipt:** an observed-vs-desired comparison of the rendered unit on both hosts, not a return
code. The rendered unit is the only member of the must-move-together set that produces **no failure
signal** when stale — ref, manifest, digest and closure each raise a checksum event, while a stale
unit is silent, which is how both hosts drifted on two axes unreported. Two hand-edits are
outstanding against that receipt: the context length raised today, and `OLLAMA_NUM_PARALLEL=4` set
on both hosts, each under operator instruction.

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

**Landed.** The authority is `extdeps.llm.anthropic_messages_api`, its own module rather than a row
in `extdeps.llm.llm` or `extdeps.llm.llm_contracts`: those two are agnostic anchors, and the
specification is a named, versioned upstream subject with a citation and a version axis of its own,
which is the `extdeps.whatwg.html_navigation` position rather than the generic-hub one. It keeps the
publisher's names because coining a neutral second name for a concept that already has one is the
nicknaming violation, and it names no implementation: `MessagesApiImplementation` is the carrier
each implementation declares its own row in.

The shape rows MOVED. `extdeps.llm.anthropic` retains only what is true of the API product — its
model roster — plus its own conformance row; the wire contracts moved with the shape into the
`_contracts` sidecar the emitter merges into the specification module, so no contract row is left
pointing at the vendor module. `extdeps.ollama.api` gains the `/v1/messages` endpoint row and
`ollama_messages_api_conformance`, and re-models none of the shape.

The version binding is the axis worth having: upstream documents that Ollama ACCEPTS the
`anthropic-version` header and does not use it, so the two implementations declare different arms of
`MessagesApiVersionBinding` against the same spec version, and a caller cannot read an accepted
request as evidence of the revision it asked for. `test.claim.extdeps_llm_type_grounding_witness`
carries the discriminating controls: it goes red if the axis collapses, if a conformance row stops
pointing at a declaration in its own module, or if a wire contract is left behind on the vendor
module. Ollama's `/v1/messages` behaviour is grounded in upstream's own
`docs/api/anthropic-compatibility.mdx`, not in a probe transcribed into prose.

### DCH-2 — The harness, in `.dag` and Rust

**Question owned:** can we drive a turn to a terminal disposition and report it?

- The loop: post a request, and while the response stops on tool use, execute the tools and post the
  results back. Four tools is the starting surface: run a command, read, write, edit.
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
