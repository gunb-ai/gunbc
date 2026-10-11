# Convergence coverage: the ensure chain to the execution boundary

**Governed by** [v2-compiler-architecture](v2-compiler-architecture.md) as a compiler-stage implementation program of the stage: completion, residual and closure -- covering production convergence paths to the execution boundary. Derived from gunbc.plan_governance; the body restates none of it.

> **Status: implementation program, not a design authority.** It continues `gunbc.plans.v2_compiler_architecture` (completion, residual and closure) and `gunbc.plans.ensure_closure_design` (D14 environmental instance). Ruled onto the public roadmap 2026-10-11 (operator, parent bright-moth-475: document this as its own scope, relevant to the roadmap and to debt payoff; go through every convergence path and see what must be covered). Compiler-floor value: a live compiler that accepts an operation without its complete prerequisite chain makes incomplete convergence cheaper to ship (DESIGN section 4b ordinary floor first). Debt-payoff value: the same gaps are named recurring-failure-mode and rung-drop classes paid on every run until retired; covering the ensure chain to the execution boundary retires that debt rather than adding a parallel lane. Placement of the project node is held for the operator (v2-compiler-live, debt-payoff, or a fifth top-level). No implementation lands from this document. It does not restate, amend or replace D14 or the architecture.
> **The missing guarantee.** D14's operator intention: if you forget to model this, the interface will not let you use it, at compile time. Inspected production paths have useful convergence procedures, typed decisions and some constructor restrictions. They do not establish that an operation cannot be admitted without its complete prerequisite chain, and cannot consume success until that chain has actually established the required facts. The decisive missing edge is at execution: `extdeps.shell.exec` `RunArgv` / `RunArgvStdin` / `RunArgvOutcome` take a nonempty program spelling and argv expansion, not an executable runtime for that program on that placement usable by that principal. Removing a prerequisite-producing ensure does not deprive the operation of a required operand. D14 already names the replacement (qualified operands, restricted construction, completion from root bindings) and withdrew `ensure fn` / `ensured` syntax. This program covers production paths against that replacement.
> **Evidence standing.** The cause list is the operator's 2026-10-11 review of the srv1 workspace bring-up findings ledger (59 findings, 2026-10-10/11), source pinned to main@acc218340ed6e24a9ad06c550a5494307d0320bb. That review did not rerun srv1. Its earliest-boundary corrections are authoritative over the ledger's hypotheses. Recurring failure mode rows already on main stay the class authorities; this program does not fork them.

## 1. What exists vs what it establishes

`gunbc.ensure` `ensure_decide` chooses Noop/Apply/Refuse from supplied observations, satisfaction, policy and plan availability. `std.realization_reconcile` `ensure_reconcile` and `ConvergedEffect` classify the consistency of supplied decision, application and readback reports. `ensure_reconcile` does not execute the plan or establish that the later observation came from that execution. `ConvergedEffect`'s restricted constructor proves the classifier processed the report, not that the report came from the selected effect plus an independent observation -- `std.realization_reconcile` reserves that for the domain's effectful path.

Real effectful convergence worth preserving: `gunbc.host.host_effect_realize` `srv3_ensure_apt` (observe, branch, install, re-probe); `run_microvm_network_apply` gating privileged work on `ensure_host_tool` results; `converge_typed_remote_file` staging, chmod, rename, compare, and constructing success through the effectful route. Preserve those routes. Do not treat a supplied-report classifier as their substitute.

## 2. What this program is not

- Not a redesign of D14, of omitted-operand elaboration, of `HostBlueprint` / `HostClosure`, or of mint provenance. Those stay in `gunbc.plans.ensure_closure_design` and `gunbc.plans.v2_compiler_architecture`.
- Not a debt-payoff milestone. Known gaps this program retires remain named gaps; they sit here because the missing guarantee is one program, not six unrelated patches. Dual membership is refused (`MembershipAmbiguous`).
- Not a new ensure syntax, a second decision algebra, or a home for the effectful admission-actuation-observation cycle. DESIGN section 3d already names that cycle as having no consumed home yet; the fleet admission spine is its first consumer. Leaf E covers production paths against that fact. It does not mint the home.
- Not a transfer of seed findings onto v2. A seed defect needs its own executed negative control on the native route (leaf A).

## 3. Leaves (deepest causes)

The census is first. Each later leaf is one cause class from the operator review. A class already rostered under `gunbc.recurring_failure_mode` is cited, not restated.

### cc-path-census

Enumerate production observe / decide / apply / readback / mint paths. Classify each against the missing execution-boundary guarantee: does removing a prerequisite-producing ensure deprive the consumer of a required operand? Output is a typed roster with one row per path (route, subject, what is supplied vs established, covering leaf or named exclusion). The roster is the denominator; later leaves do not invent their own.

### cc-compiler-floor (A)

An ordinary type obligation can stay unresolved while execution proceeds (ledger F19/F36). Confirmed: `dispatch_worker_unit_properties_for_plan` returns a list of SystemdRunProperty and `belt_session_container_create` passes it where `session_placement_record_write` (with its intermediate constructor and persisted record) declares a list of String. Not established as root cause: list element types aren't checked -- `direct_call_generic_type_argument_inhabitance_diags` exists (`v1.compiler.infer`). The weakness: a record at a scalar formal can produce `InhabitanceUndecidable`, and `DeclaredTypeInhabitanceUndecided` is `GateAdvisoryTypecheck` (`v1.compiler.core`), so it does not block. Existing class: `gunbc.recurring_failure_mode` `product_value_at_a_scalar_formal_is_accepted`. The SystemdRunProperty-in-list path still needs a discriminating execution. Zero blocking errors does not mean every declared type obligation was established. DESIGN section 4b puts the ordinary floor first.

### cc-structural-carriers (B)

Consumers assume a shape they never established (F37). `bash_build_subshell` accepts any `Node` body. The serializer's shared `bash_fold_flatten_stmt_list` reads positional children 0 and 1 only, so an `or_else` node passed as the body keeps left and drops right, with no refusal arm (`v2.extdeps.languages.bash_build`; `v2.workflow.bash_command_fold_serialize`). Two insufficient boundaries: the body carrier does not establish grammar membership, and the serializer turns an unestablished shape into plausible partial output (DESIGN section 5). The incident's caller fix (`bash_build_subshell_from_stmts`) leaves the shared reader able to drop behavior through other consumers. Durable repair: establish shape at construction, preserve every admitted child, refuse malformed input. No new Bash representation.

### cc-qualified-operands (C)

Environmental facts are optional descriptions, not required operands (F02, F07, F14, F16, F35, F43-F44). A directory path does not establish the principal can create its socket; a secret location does not establish readers can read it; a repo path does not establish checkout capability. The reversal belongs once at the operation's owning boundary: the operation asks for the established capability, its selected provider induces further requirements, the compiler carries them outward, the root answers policy and placement. Concrete §3 defect: `srv3_ensure_apt(subject: AptPresenceSubject, package: NonEmptyStr, ...)` -- install uses `package`, readback checks `subject`, and they can disagree. D14 section 10 already carries this specimen. Good owning-boundary repairs exist (event-carrier directory, tailnet-door directory as deployment demands). Remaining obligation: demands follow from the consumer systematically.

### cc-observation-distinctions (D)

Observation loses distinctions that decide whether repair is possible (F11; related F01, F46, F52). `ProcessRunningRelease` carrying a revision, or `ProcessNotRunning` carrying an active flag: an active service whose /healthz fails becomes `MemberUnobservable`, which `decide_member` refuses (`gunbc.live_deploy.member_observe`, `member_identity`). There is no representation of an observable, owned, active service with a broken health/release endpoint, so repair requires the damaged service to cooperate. Class already recorded: `gunbc.recurring_failure_mode` `a_deploy_that_observes_its_own_service_cannot_repair_it`. Needed distinctions: desired holds, no mutation; demonstrably absent, create if policy allows; owned and repairable drift, derive the repair; conflicts with ownership/identity, refuse or an explicitly admitted transition; unobservable, refuse dependent actions. An arbitrary 403 must not become absent. An incomplete census cannot always be best-effort. D14: domain assessment owns the decision; a generic satisfaction Boolean is insufficient.

### cc-lifecycle-ownership (E)

Convergence covers selected successful transitions, not failure and retirement (F12/F39, F13, F17, F20/F21, F24). `identity_member_of_step` leaves belt, publication, fabric-storage, approval-front-door and tailnet-door artifacts outside the identity-aware route; `deployment_apply_plan` reconciles them against `observed: []` (`gunbc.live_deploy.emit`; unfinished per-member-kind migration). Dropping a unit from desired does not retire one an earlier deploy owned. A candidate's serving closure is checked only after switching. Attempt creation and container activation have incomplete recovery; dispatch and cleanup observe different container populations. Removing the clock exposed missing mutation-to-projection causal dependencies. A foreign listener is discovered only by failed activation or revision mismatch. Moving the record write after container creation swaps the crash gap; the shared operation needs an activation identity and reconciliation over both sides. Deployment establishes two facts at different times: before promotion, the exact candidate's serving entry and closure are accepted and materialized; after actuation, the instance is independently observed running that candidate. An invalid annotation is correctly refused (DESIGN section 4c); consulting that refusal too late is F39.

### cc-selection-to-use (F)

Scope and policy are not carried from selection to use (F03/F04, F18, F22, F33, F45; admission F25, F26, F31, F47). Which environment, candidate, principal, state store and source tree belong to this attempt is not answered by the host alone. Correction to F04: the operator allowed branch development operations; the declared restoration is binding revision and branch to an admitted identity (`gunbc.rung_drop` `fleet_converge_named_revision_and_branch`). Restoring universal main-only does not satisfy it. The repair is explicit target and candidate selection consumed consistently; production policy constrains that selection without breaking the dev route. Security context, writable scope, concurrency limits and grants exist as descriptions without controlling the effect. They keep their own authorities (D14: an ensure cannot grant itself permission or promise volatile capacity). The compiler's job is to make those distinct prerequisites unavoidable in the same dependency graph.

### cc-evidence-to-consumers (G)

Evidence and reusable results stop short of their consumers (F05, F06, F09/F57, F15, F32, F34, F42, F51, F53, F59). The required job covers two emitted closures and does not evaluate witnesses or compare generated artifacts (`gunbc.rung_drop` `v1_required_lanes_withdrawn`), so green CI does not establish the workspace's serving/deployment closure or the sudoers projection. Witnesses found a regression whose result did not constrain the merge; evidence was for another revision; a projection lost its consuming gate; an unobserved claim supplied no execution evidence; a real type error sat outside the required population. This is DESIGN sections 3c and the pairing obligation: a result must qualify the exact candidate and behavior the next step relies on. Supplying inputs may never remove the last execution of the real path.

## 4. Landing order

1. Census the paths (cc-path-census). No repair lands against an unenumerated path.
2. Compiler floor negative control on the native route (A), in parallel with structural-carrier refusal (B): both are walls the later leaves assume.
3. Qualified operands at owning boundaries (C), using D14's existing target and mint shapes, not a parallel vocabulary.
4. Observation distinctions (D) and lifecycle ownership (E), after the census names which member kinds and probes are in scope.
5. Selection-to-use (F) and evidence-to-consumers (G), after the operations they constrain are the ones the census listed.

## 5. Controls

- RED for the program: a production path in the census whose consumer still type-checks after its prerequisite-producing ensure is deleted.
- Positive control: a path whose qualified operand is constructed only on the effectful mint route, and whose consumer cannot bind a raw identity.
- Each leaf keeps the discriminating RED of any recurring-failure-mode class it climbs; dissolution deletes production machinery, never the evidence (DESIGN section 4b(4)).

## Dissolution trigger (DESIGN §6)

Delete this document when every production path in the census either consumes a qualified operand minted on the D14 effectful route or carries a named exclusion with a DESIGN section 3b reason, each covering leaf's handback is met, and the roadmap project convergence-coverage is accepted. At that point the carriers, witnesses and rung-drop rows are the authority, and this program document would be a second statement of them.
