# Dedicated Coding Harness — scope

Project ID: DCH

Repository anchor: gunb-ai/gunbc

Baseline: main@de2f5f86346, read 2026-09-01.

Operator direction (2026-09-01): serve a large open model on the DGX Sparks and drive a minimal
coding harness against it, retiring the Claude, Codex and Cursor provider runtimes. Written from
the ground up in `.dag` and Rust. **gunbc does not know about `ctrl`** — no import, no dependency,
no citation of a `ctrl` artifact as authority. Spark parity is to be handled by fleet convergence
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
  unmodeled axis has no desired value, so it cannot even be reported as drifted.
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

### DCH-0 — Enrol the Sparks and make residency representable

**Question owned:** what does the fleet know about these two machines, and who says so?

- Enrol srv5/srv6 in `fleet_intent_network.endpoints` and `fleet_intent`'s `ComputeHost` list — the
  act the address correction deliberately did not perform.
- Model the distinction between the converged member and the resident set, so seven models are
  representable and the sixth-through-seventh are owned rather than ambient.
- Add `OLLAMA_NUM_PARALLEL` to the rendered unit, or declare in a row why serialization is desired.
- **Receipt:** an observed-vs-desired comparison of the rendered unit on both hosts, not a return
  code. The rendered unit is the only member of the must-move-together set that produces **no
  failure signal** when it is stale — ref, manifest, digest and closure each raise a checksum
  event; a stale unit is silent, which is how both hosts drifted on two axes unreported.

**Owner: eager-pike-541** (sparks lane), confirmed with them before it starts.

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
- Treat "it completed one turn" as evidence for anything but that.
