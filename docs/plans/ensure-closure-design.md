# Completion, residual and closure: environmental dependencies derived from structure (D14)

**Governed by** [v2-compiler-architecture](v2-compiler-architecture.md) as a compiler-stage specialization of the stage: completion, residual and closure -- environmental completion at a concrete placement. Derived from gunbc.plan_governance; the body restates none of it.

> **Status: design authority for D14 of the demand-engine program (`gunbc.plans.demand_engine_program` section 7), ruled 2026-10-01 (operator, session wonderful-feynman: the direction was endorsed over four rounds and this document was written at the operator's request as the ruling's authority). No implementation lands from this document.** It SUPERSEDES the 'program runtimes as derived prerequisites' D14 draft and the requirement-obligation vocabulary considered earlier in the same session -- RequirementObligation, RequirementClosure as a public graph beside the demand graph, DependencyDemand read as an obligation, a universal ExecProposition coproduct, an authored OperandSlot, a fifteen-kind relation law product as part of this vertical, and a combined ClosedWithUnauditedCauses standing. None of those names is minted. D13 stands unchanged: this document is D13's provider binding applied to environmental state, with the lifecycle inside one demand node.
> **Operator intention carried (2026-09-30 and 2026-10-01, the operator's words).** 'This reverses the modeling direction from please remember to model everything to if you forget to model this the interface will not let you use it (during compile time).' 'Git is a member of Program -- and all programs must be installed -- so what is effectively required for this program is how are you going to install this program; if they plan to actually emit the code and run it in production, they cannot compile it unless they provide a layout of a machine that satisfies -- all the way down: git needs to both be installed and on the path, and then the user needs to also answer how it gets on the path.' 'This will basically force us into using shared libraries -- the dependencies are truly complex, but they are managed globally and uniformly.' 'I see this relationship everywhere and need a single word to identify it: that should be an upsert is much easier for developers than the kind of modeling we are discussing.' 'Model the underlying behavior, not our observations.'
> **Third-party review folded in (2026-10-01).** Five corrections to the session's earlier draft are adopted: the typed operation contract is the source of ensure demand and the failure model is an audit of it, never the sole derivation; the package lifecycle is a first-class level between the program identity and the executable; PATH is an explicit, optional level rather than a bypassed one; environmental closure and failure-audit completeness are two standings, not one combined variant; realizations induce environmental requirements, abstract semantic operations do not. Three amendments are made to that review here: a program's own child spawns induce launch-environment targets per operation, not only the caller's invocation route; the developer's machine and every CI runner are placements with blueprints, observe-only where nothing may be installed; and the package-to-path step is read back from the package's real file population, never trusted from a catalog transcription.
> **Amended 2026-10-01, later the same session (operator ruling: no keyword for what the structure already states).** The organizing law is COMPLETION, RESIDUAL and CLOSURE (section 1), of which ensure is one completion mechanism and the library name of the convergence lifecycle, never syntax. The `ensure fn` declaration, the `ensure` parameter keyword, the `ensured` type marker and the `for` target expression that this document's first version proposed are WITHDRAWN as dual representations: each restated a fact the types, sole constructors and root bindings already carry, and a second spelling of one fact drifts (DESIGN section 3). The derived / policy / irreducible parameter classification proposed between the two versions is withdrawn too: classification is an OUTPUT of completion for transparent code and is authored only at an opaque boundary. The one genuine semantic addition is named in section 1.5. The operator's framing: the developer tells the compiler what it does NOT know, instead of describing the world repeatedly; for materialization the compiler tells the developer why it cannot cache, for CLI construction it emits every derivable argument and asks only for the unresolved semantic inputs, for a machine layout it generates the demanded closure and asks the deployment author only to close its holes.
> **Review of f38567c9ae folded in (2026-10-01, request changes; every finding accepted).** (1) The mint example constructed the qualified carrier before entering the mint; construction now happens inside the domain-owned mint from the grounded readback, and the mint requires the existing sealed `std.realization_reconcile` `ConvergedEffect`, which is the effectful construction path section 8's provenance claims rest on, stated at its honest rung. (2) 'git status --porcelain induces nothing' was FALSE: with `core.fsmonitor` set to an executable hook, the exact argv `extdeps.git` `git_status_porcelain_args` produces spawns the hook (reproduced on git 2.47.3; `-c core.fsmonitor=false` prevents it), so a child-spawn footprint is a function of effective configuration and the first corpus step is an effective-configuration contract, not a list of operations. (3) Three dual representations remained -- a provider-level write set, registration rows restating the target type, and a package-roster refusal for openssh-client -- and are removed. (4) The developer blueprint's observe-only providers regressed through dpkg-query's own runtime; the leaf is now named: native filesystem observation through the root-bound Filesystem provider of D13, never PATH and never self-attestation. Q4 of the review brief is answered NO as written: completion adds two admission obligations beyond D13, now stated in section 1.5 and in the D14 row. The corpus findings beside the design (srv3_ensure_apt's separately supplied package, apt_package_of's first-source fold, RunArgvOutcome's sibling spawn input, and that ensure_host_tool's verdict DOES gate its workflow through host_tool_prerequisite_holds) are carried in section 10.
> **Re-rooted 2026-10-01 (second review of #12872: the law was architecturally unrooted).** The universal law -- partial programs, maximal lawful completion, typed residuals with addressees, closure at runnable roots, lenses as the reflective half, the relation to coercion -- is now owned by `gunbc.plans.v2_compiler_architecture`, and this document is its environmental specialization. The relation is ONE authored row in `gunbc.plan_governance`; the banner above, the children list on the parent, and reachability from the compiler root are derived from it, never written into a body, and the lens claims (every governed plan reaches the root; a cycle or an orphan does not) are enrolled in `test.claim.plan_governance_witness`. The demand-engine program is a consumer of the completed, closed program, not this design's parent. The paragraphs of section 1 that stated the law in full are kept below, marked retained for the record, so the diff that moved them is reviewable; the architecture owns them.
> **Review of 14f36a479f folded in (2026-10-01; every finding accepted).** The mint-provenance rung claimed mechanically preventable on the strength of a convenient honest path, which is not DESIGN section 4b's definition; section 3.2 now splits the mint into three guarantees with three standings -- construction confinement (structural once implemented), grounding of supplied evidence (structural consistency), execution provenance (a declared trust boundary, unestablished beyond trust) -- and names the identity-join instrument, ProviderMintProvenance and its mandatory gate, that lifts the third to mechanically preventable, with the engine-minted carrier as the structural rung after it; section 8 calls its six relationships the target substrate. The fsmonitor row conflated a missing model with an unresolved policy and an unread configuration; section 9 now carries the three residuals with their three addressees. The governance uniqueness defect and the pipeline ordering are corrected in the architecture and in `gunbc.plan_governance`.

## 1. The law this document specializes

The law is owned by `gunbc.plans.v2_compiler_architecture` section 1 -- a program is a partial specification; the compiler computes its maximal unique lawful completion; what it cannot derive is a typed residual with provenance and an addressee; libraries may stay open and runnable roots must close; lenses are the reflective half -- and the governance banner at the top of this document is derived from the one authored row in `gunbc.plan_governance`. Nothing of that law is restated here. What follows is its ENVIRONMENTAL INSTANCE: completion at a concrete placement, where the routes are providers, catalogs, blueprints and observers, and the review question the law leaves for this document is: why is the developer supplying this -- is it actually residual?

RETAINED FOR THE RECORD, superseded by the architecture: the first versions of this document stated the law here in full. A program is a partial specification. The compiler COMPLETES it from what the repository already knows -- types and their sole constructors, the selected realization's upstream facts, the catalogs, the deployment root's policies, registered providers and observers -- deriving every operand and dependency the structure determines. What completion cannot derive is a typed RESIDUAL that carries its provenance and its addressee: the call site for an irreducible input, the root for a policy choice, an upstream module for a fact never modeled, an observer for a fact only the world holds. A library may carry residuals in its interface; an executable or deployment must CLOSE them, and an open executable is not admitted. A developer supplies only residuals; supplying a fact the compiler could derive is a second authority and refuses unless stated as a divergence with its reason (DESIGN section 3b). Environmental usability is one completion mechanism among several -- ensure for convergable environmental state, materialization for storage and reuse, observation for world readings, grants for admission, leases for capacity, catalog lookup for upstream facts -- and ensure is its library name (`gunbc.ensure`), never a keyword. Ambient machine state never completes anything. The review question is: why is the developer supplying this -- is it actually residual?

### 1.1 The environmental instance

A selected realization may consume only qualified environmental values, never a raw environmental identity. Its typed inputs induce ensure targets -- including the launch-environment targets of programs that realization spawns by name -- and the generic implementations selected for those targets recursively induce further targets. A composition root supplies a host blueprint: base observations, provider families with their policies, launch environments, principals and the admission envelope; per-target bindings are overrides, not the ordinary route. The compiler closes the selected executable against that blueprint into a derived host closure -- the concrete package, program, path, principal, permission, artifact, service and readback plan -- which is a reviewable projection. An executable whose closure is missing, ambiguous, cyclic, incompatible or unevaluable at plan time is not admitted; a library may expose unresolved targets as part of its interface. Every host an executable runs on, the developer's own included, is a placement with a blueprint. Only a provider registered for a target may mint its result, and only after an effective readback; providers expose their effect footprint and carry an admitted recovery law before automatic retry. Environmental closure and failure-audit completeness are separate standings. Pure facts, access admissions, materialization judgments, resource leases and contended runtime outcomes keep their existing authorities. Ambient machine state never discharges a target.

What this buys, in the operator's framing: more upstream contract modeling creates more derived requirements, more deployment modeling creates more satisfiable closures, more cross-layer modeling exposes more contradictions before a host is touched, and forgetting to model a required interaction makes the program uncloseable rather than fragile. The use site writes none of it.

### 1.2 Sibling routes, pointed to and not absorbed

The common law is completion and residual; the answer routes stay distinct, and this document owns only the environmental one. COERCION (`v2.std.coercion`) completes a semantics-preserving representation crossing by homomorphism evidence; no provider can solve a coercion residual, and no homomorphism can supply a runtime. MATERIALIZATION (`std.materialization_ladder`, `v2.std.materialize`) decides how an admitted judgment is reused or stored; no homomorphism can supply a cache provider, and an ensure may establish only a provider's readiness. GRANTS (`std.effect_grant`) admit authority; environmental completion cannot grant itself permission. LEASES (`v2.std.demand_engine` BlockedWaitingForCapacity) supply volatile capacity; an ensure cannot promise current capacity. OBSERVATION supplies world facts; the compiler never invents them, and an observer's own runtime is itself a hole this document's leaf (section 3.3) must close. A design that routes one of these through another is the absorbing word the architecture's section 5 refuses.

### 1.3 What this document keeps, and what the demand engine consumes

Kept here: environmental targets (3.1), provider implementations and the mint (3.2), the host blueprint and derived host closure (3.3), qualified results (3.4), the boundary of what is and is not an ensure (4), the package -> runtime -> launch levels (5), the git vertical (6), the environmental failure audit (7), the lifecycle and its honest rung (8), behavior (9), the census (10), the landing order, controls, frontier and open decisions (12-15). The demand engine (`gunbc.plans.demand_engine_program`) is a CONSUMER of the completed and closed program and its derived DependencyRelations: environmental completion produces qualified operands and relations before the engine seals, and the number D14 is historical rather than a statement that this design lives inside the engine. The two admission obligations that exceed D13 -- the derivable-operand rule over complete applications, and lifecycle execution provenance -- are stated in the architecture's section 1.5 and carried here only where they bind: the mint (3.2) and the lifecycle (8).

RETAINED FOR THE RECORD, superseded by the architecture's sections 1.2 to 1.5, which now own this text, as amended there: an omitted operand is a typed hole, and the hole's TYPE admits exactly one completion route: a `Placement` is projected from the effect's host binding; a `MaterializedProgram`, `PresentPrincipal` or `OwnedDirectory` is completed by the provider the root registers for its target; a `PackageCoordinate` is a catalog lookup; a materialized artifact is a `std.materialization_ladder` judgment; an `AdmittedGrant` is an admission; a `ResourceLease` is a lease; an observed fact is its observer; a plain `Int` is never completed. The routes are a partition, not a preference order: completion never tries one route and falls back to another, because a first-route-that-works search is the absorbing fallback of DESIGN section 5 moved into elaboration. A hole with zero routes or two routes is a residual, not a guess. When a route's input is not yet in scope -- the placement is unknown inside a library function -- the hole propagates outward as an inferred requirement of the enclosing function, which is how a library stays open and an executable closes.

Completion runs before the demand engine seals, and everything it resolves is sealed with the graph: resolution is subject-indexed (git at srv5 for deploy-runner, not 'a program'), static, provenance-carrying, and refuses zero or multiple answers. That is the distinction from dependency injection, which resolves by type at runtime and silently picks.

A residual is not an error. It is a typed statement of what the compiler does not know and WHO can answer it, and the addressee is part of the type: the call site (an irreducible input such as a repository path or a requested model), the root (a policy choice such as apt versus immutable image, observe-only versus may-apply, a tolerated staleness), an upstream module (a fact never modeled, such as which git operations spawn children by name), or an observer (which probe reads the running image digest). Each residual carries the provenance path through which the hole was reached, and that same path is the diagnostic when closure fails.

Two defects are measured against it. OVER-ASKING: a residual addressed to a call site that structure or a root could have answered is a counted modeling defect with a location, and the measurement is two, not one: holding the contract, route universe, root policy and observer surface fixed, avoidable call-site over-asking may only shrink, while the total residual population is not monotone because newly modeled behavior exposes new residuals, which are fidelity discoveries rather than regressions (the architecture section 1 owns the wording). UNDER-ASKING: a completion that fills a hole from ambient state -- PATH, a framework default device, 'probably apt' -- is fail-open and refused by construction, because no route reads ambient state. Silence means fully derived; a residual means genuinely unknown; nothing exists between them.

A library function's interface is its irreducible parameters plus its inferred requirements. Binding a program to a placement is what forces every requirement to a route, and the executable is admitted only when the residual set addressed to anyone other than its own call sites is empty. A hole whose subject is only known at runtime -- a program identity computed from data -- cannot be completed at plan time and is a residual addressed to the call site: make the subject static or supply an observer. It is never completed by a runtime fallback.

Almost everything above is already supported: sole constructors and admit_callers confine minting, D13 derives dependency demand from resolved references, the demand engine deduplicates and seals, the materialization ladder refuses uncovered recurrence. What is NOT supported today is the hole itself: the compiler floor (DESIGN section 4b) requires applications to bind in exact bijection, so an omitted operand is a refusal, not a variable. The one semantic addition is therefore ELABORATION OF OMITTED OPERANDS: the authored program may be partial, and the bijection holds on the completed program. This is not a feature beside D13; it is D13 generalized from resource interfaces to every operand, and stating it that way keeps it one mechanism. No syntax is added for it. Two admission obligations DO exceed D13 and are stated here so they are not smuggled in as 'widening': first, a derivable-operand rule over COMPLETE applications -- an operand whose type has a completion route must be omitted, or supplied with a stated divergence, so the compiler examines applications where nothing is omitted, which D13 never had to; second, lifecycle execution provenance -- that a provider's result was minted from its own readback after the branch it actually took -- is not established by demand derivation at all but by the library's sealed carriers and the mint's admission (section 3.2, section 8), and its rung is stated there rather than claimed from D13.

## 2. The one word stays ensure -- as the name of a completion mechanism, not a keyword

The review phrase is: **a raw environmental identity reached a consumer -- that should be an ensure**, with section 1's question beside it: why is the developer supplying this at all? Precisely, an ensure is appropriate when all five hold: there is a desired state or qualified usability proposition; that state can be observed or entailed; absence, drift, conflict and unreadability are distinguishable; an admitted implementation may converge it; and a readback can ground the result. That covers program runtimes, packages, users and groups, paths and ownership, services, grants' host artifacts, acquired artifacts, cache stores, images, network configuration and host capabilities. It does not replace pure dataflow, reference binding, membership, GeneratedFrom provenance, VerifiedBy evidence, module imports or presentation containment, which stay the relations they are.

The modality already exists: `gunbc.ensure` is imported across the corpus and its own header rules that ensure is a convergence modality and never a roster of packages or tools, with one requirement, observation and receipt per semantic capability rather than one function per executable. It sits on `std.upsert_decision` (Noop, Apply, Refuse over an ObservationVerdict) and `std.realization_reconcile` (grounded readback, ReconciledEffect, the sealed ConvergedEffect). DESIGN section 3d names `ensure` as the convergence primitive. Introducing 'upsert' as the developer word beside it would give one concept two names -- the nicknaming DESIGN section 3 forbids -- so ensure is the word and `UpsertDecision` is what an ensure decides. The second reason is semantic: `EnsurePolicy` already distinguishes EnsureObserveOnly from EnsureMayApply, and 'ensure git is present, refuse if it is not' reads as what it is, where 'upsert' promises a write a read-only binding never makes.

The universal mechanism is NOT that everything is an ensure. It is that every implementation's typed parameters are dependencies, and the parameter's type decides which authority discharges it: an ensured environmental value, a pure authority fact, an access admission, an observation, a lease, or an ordinary semantic value. That is one deep dependency graph without one vocabulary forced onto every relation.

## 3. Four public concepts

### 3.1 Ensure target

A desired environmental fact at a concrete placement. Placement is host, principal (the existing `gunbc.fleet_posix_accounts` `PosixAccountOccurrence`) and launch context; version admission, architecture and image identity are part of the target's identity where they matter. The full target is the deduplication key: two uses that demand the same target become one node, and two targets that differ remain distinct, carry a modeled compatibility relation, or refuse as conflict -- no guessed subsumption of one version, mode or ownership request by another.

```
ProgramRuntime      { program: ProgramIdentity, admission: VersionAdmission, placement }
PackageInstalled    { package: PackageCoordinate, admission, host }
LaunchableProgram   { runtime: ProgramRuntime, launch: LaunchContext }
LaunchEnvironment   { launch: LaunchContext }           -- PATH and the rest of the launch-time environment
PrincipalPresent    { login, uid, host }
PrincipalMemberOf   { login, group, host }
OwnedDirectory      { path, owner, group, mode, host }
InstalledFile       { path, exact bytes, host }
RunningService      { unit, host }
```

Targets are fine-grained capabilities, not giant account states: `ExecutorPrivilegedOperation` already separates EnsureSystemAccount from EnsureSupplementaryGroupMembership, and that is the right grain. One consequence it exposes: changing a group membership does not update a process that has already called initgroups, so `PrincipalMemberOf` must precede the launch whose credential set consumes it, or the service needs a restart ensure. The hierarchy derives that ordering; nobody remembers it.

### 3.2 Provider implementation

An ordinary function that establishes its target and returns the domain's qualified result. Its typed parameters are its prerequisites; there is no authored dependency list and no declaration marks it as an ensure. What makes it a provider is three facts the structure already carries: its result type is a sole constructor it is admitted to call; the root registers it for a target family; and its body runs the `gunbc.ensure` lifecycle -- observe, assess, apply when decided, independent readback, mint. Completion fills its parameters exactly as it fills any other call's: `package` is completed by the provider registered for `PackageInstalled`, `who` by the provider registered for the `PrincipalPresent` target (the sealed result it yields is `PresentPrincipal`; roots register providers for TARGET families, never for result types, and keeping the two names distinct is what mint confinement rests on), and the operations its body calls induce their own holes (dpkg-query, apt-get, the install grant).

```
fn package_installed_via_apt(target: PackageInstalled) -> InstalledPackage
  -- retry law: Idempotent (a pinned apt install). No write set is declared here: the footprint is DERIVED
  -- from the operations the body calls (dpkg-query reads the dpkg database; apt-get writes it and /usr),
  -- and those facts are authored once at the opaque upstream boundary, never restated per provider.
{
  let before   = dpkg_query_status(target)               -- RunArgv over dpkg-query: its MaterializedProgram is a hole completion fills
  let decision = ensure_cap(policy_of(target), apt_package_assess(target, before))
                                                         -- Absent -> Apply { plan: install }; below admission -> Apply { plan: upgrade }; held -> Refuse (Conflict)
  let effect   = ensure_apply(decision, apt_get_install) -- RUNS the branch; the Application arm is its return, not a literal
  let after    = dpkg_query_status(target)               -- the SAME observation, independent of apt's own report
  mint_installed_package(target, reconcile_effect(decision, effect, after))
}

-- THE DOMAIN-OWNED MINT. The sole constructor of InstalledPackage is admitted to this function and to nothing
-- else; it builds the carrier FROM THE GROUNDED READBACK, and it cannot be called without the sealed
-- std.realization_reconcile ConvergedEffect that classify_reconciled mints only when the read grounds the target.
fn mint_installed_package(target: PackageInstalled, converged: ConvergedEffect<AptPlan, DpkgStatus>) -> InstalledPackage
  admit_callers: [registered providers of PackageInstalled]
{
  InstalledPackage { target, files: converged.evidence.files }
}

fn program_runtime_from_package(target: ProgramRuntime, package: InstalledPackage, who: PresentPrincipal) -> MaterializedProgram
{
  let facts = stat_exec_interp(path_in(package.files, target.program), who)   -- this provider never writes
  mint_materialized_program(target, reconcile_effect(Noop, NotApplied, facts))
}

-- root registration (the blueprint's family rows, section 3.3): names the IMPLEMENTATION and the POLICY only.
-- The target type it provides is read off the function's result; spelling it again here was a dual representation.
--   apt_family      { by: package_installed_via_apt,    policy: EnsureMayApply }
--   package_runtime { by: program_runtime_from_package, policy: EnsureObserveOnly }
```

What the mint establishes is THREE guarantees with three different standings, never one indivisible rung. (1) CONSTRUCTION CONFINEMENT -- `InstalledPackage` is constructed only inside the domain mint: `sole_constructor` plus the mint's admit roster; structural once implemented. (2) GROUNDING OF SUPPLIED EVIDENCE -- the supplied readback is classified as grounding the supplied target: the mint requires a `ConvergedEffect`, sole_constructor on main and minted only by `classify_reconciled`; structural consistency of the supplied values, exactly as `std.realization_reconcile` states its own strength. (3) EXECUTION PROVENANCE -- that the decision, the application and the readback came from the nodes and the branch actually executed for THIS attempt: no mechanism today; a registered provider can hand `reconcile_effect` fabricated values, and `ensure_apply` returning the `Application` arm makes the honest path convenient without blocking the dishonest one. Standing: a DECLARED TRUST BOUNDARY on registered providers, unestablished beyond trust (rung: mitigatable, DESIGN section 4b). Stating it as mechanically preventable was rung inflation and is withdrawn.

The instrument that lifts (3) to mechanically preventable is an identity join, named here so landing step 4 builds that and not a convenience: `ProviderMintProvenance { target, provider_demand, decision_demand, application_demand, readback_demand, attempt }`, and a mandatory gate establishing that each mint joins uniquely to the selected provider demand, the observation that actually ran, the decision produced from that observation, either the effect that actually ran or the explicit NotApplied branch, an independent readback that ran afterwards, and the same target and attempt throughout -- a missing, duplicated, cross-attempt or caller-invented link refuses. Its own control: weaken any one join and the gate goes red. The structural rung after that is the demand engine sole-constructing that carrier from its terminal rows, with the domain mint requiring it, so a provider can no longer author the causal chain at all.

Every parameter is a dependency; not every parameter is an ensure. `grant: AdmittedGrant` is an admission proof from `std.effect_grant`, `allocation: UidAllocation` is a pure fleet fact, `capacity: ResourceLease` is a volatile reservation. The compiler lowers all of them to DependencyRelations with the operand slot derived from the parameter, which is the operand model the earlier draft wanted authored: the parameter already names the prerequisite value, the consumer input slot and the expected type.

### 3.3 Host blueprint (authored) and host closure (derived)

The word blueprint was being used for two artifacts and they are split here. The BLUEPRINT is what the author provides: small, stable, per host. The CLOSURE is what the compiler derives from an executable against that blueprint: concrete, per executable and host, nobody writes it. The operator's 'provide a layout of a machine that satisfies' is the blueprint; the review's 'the concrete output of requirement closure' is the closure.

```
-- AUTHORED. Families and policies, not per-program rows.
data srv5: HostBlueprint = host_blueprint(
  base:        image(srv5_img),                              -- closed by observing the running digest
  packages:    apt_family(policy: EnsureMayApply),
  principals:  posix_family(policy: EnsureMayApply),
  launch:      [service(gunbc_deploy, user: deploy_runner, path: systemd_default_path)],
  grants:      srv5_effect_envelope,
  prefer:      [image_member, package, artifact_store],      -- the std.decision policy among families
  overrides:   [],                                           -- per-target rows only as exceptions
)

-- DERIVED. The closure of deploy_pipeline against srv5.
HostClosure(deploy_pipeline @ srv5):
  PackageInstalled(git)                       via apt_family, from the CliTool row's SourceApt
  MaterializedProgram(git @ deploy-runner)    /usr/bin/git, read back: file, interpreter, mode for this uid
  PrincipalPresent(deploy-runner)             via posix_family, uid from the fleet allocation
  SafeDirectoryAdmitted(checkout)             via the owned-checkout ensure
  AdmittedGrant(apt-get install git), AdmittedGrant(useradd ...)   from srv5_effect_envelope
```

The family-level root is what makes adding a cataloged program free: `extdeps.tools` `InstallSource` already carries `SourceApt { package, bin_dir }`, so the catalog selects the concrete package and the blueprint selects only the family. Its one real cost is that a code change adding a program to a pipeline expands what srv5 carries with no blueprint edit, so the deployment consequence would be invisible in review. The derived closure is therefore a reviewable projection -- the generated-artifact pattern this repository already gates -- so 'srv5 now carries git' is still a diff a reviewer sees. For the first landings it is a MANDATORY, HEAD-pinned before/after closure diff whose publication is gated on the change; a count or an optional report is insufficient, and the committed projection follows with landing step 7 (section 15).

The blueprint need not describe everything on srv5. It must be closed over everything the selected executable demands: unknown facts outside that closure are irrelevant, unknown facts inside it are errors. It is assembled from existing authorities -- the OCI `ImageManifest`, `extdeps.dpkg` installed state, `extdeps.posix.identity` identities, the fleet account allocations, the unit files, `std.effect_grant` -- and no generic machine module re-authors them.

Two rules follow. **Library versus executable:** a library may carry unresolved ensure targets in its interface, and an emitted executable or deployment may not -- binding a program to a placement is what forces the blueprint to answer. **Every host is a placement:** the first friction a developer hits is `gunbc run` on their own machine, and the answer is an observe-only blueprint (no base, every family EnsureObserveOnly), under which a missing git is a typed eval refusal, never an install and never a fall back to PATH. CI runners already have converge recipes and get real blueprints. A blueprint that is missing is a plan-admission refusal, not an escape hatch. THE LEAF MUST BE NAMED, because observe-only does not remove an observer's own runtime dependency: a dpkg-query observation needs dpkg-query's runtime, whose observation needs dpkg-query's runtime, and with no base image there is no entailment to stop the regress. The developer blueprint's program family is therefore `program_runtime_observed_native`: it reads the executable's presence, mode, format and interpreter through the root-bound Filesystem provider of D13 (the process's own `extdeps.filesystem.filesystem_io` capability, not a spawned program), at the path the blueprint's one policy row supplies (`program_locations: catalog bin_dir`). It consults no PATH and attests nothing about itself; it skips the package level entirely, which is honest on a host nobody converges. The fleet blueprints bottom out through the observed image digest instead.

### 3.4 Qualified result

Each domain keeps its own sealed result carrier -- `MaterializedProgram`, `InstalledPackage`, `PresentPrincipal`, `OwnedDirectory`, `InstalledFile`, `RunningService` -- constructible only through `ensure_mint` by a provider registered for its target, and only after its readback. There is no generic `Ensured<T> { value, evidence }` record, for the reason `std.realization_reconcile` records about itself: the generic layer can classify decision, application and grounding, but it cannot prove the decided plan is the plan performed or that the evidence arose from that performance; the domain authority mints the carrier that binds subject, operation and readback. The protocol is generic; the earned values are domain-specific. The result stays tied to its full target through minting and consumption, so a materialized curl cannot satisfy git's parameter by type.

## 4. What is an ensure, and what is not

| Concept | Fits the ensure model? | Result |
| --- | --- | --- |
| Program and package runtime | Yes | the demonstration vertical (section 6) |
| POSIX account, group, owned directory | Yes | at capability grain, never one giant account state |
| Systemd service deployment | Yes | the deepest real test: `gunbc.auth.approval_ntfy_deployment` records the exact missing subtree (binary, unit bytes, config bytes, readback) that closure would refuse on |
| GCP API enablement | Yes | `gunbc.gcp_service_enablement` proves repair semantics are domain-owned: it enables and refuses to disable |
| Artifact acquisition | Yes | `gunbc.artifact_acquisition` already stages, verifies and publishes; NoSourceDeclared is a plan error, EverySourceUnreachable stays a runtime refusal |
| Secret version provisioning | Yes, with a recovery law | `gunbc.secret_provision_actuator` names AddVersion a non-idempotent write; the ensure carries an attempt identity and exact-version readback or it may not be retried |
| Effect grant | Partly | admission stays `std.effect_grant`; only the host artifact that implements a grant (a sudoers row) is ensured |
| Cache or materialization provider | Partly | the obligation comes from `std.materialization_ladder`; only the provider's readiness (store directory, store program) is ensured |
| Memory, process and runner capacity | No | observed supply, selected offer, leased capacity: `v2.std.demand_engine` BlockedWaitingForCapacity |
| Current network reachability | No | configuration can be ensured; reachability now is contended |
| Encoder and decoder compatibility | No | a type or protocol contract |
| Roadmap membership and provenance | No | relations and admission, not environmental convergence |

Two negative controls worth naming. The V4.1 snapshot's missing config.json is NOT fixed by ensure closure: ensure establishes that the declared artifact population is present, and cannot invent a file the artifact authority never declared, so that defect remains a result-contract completeness defect -- a good sign that the model does not absorb every correctness problem. And 'ensures are idempotent by construction' is false: the property belongs to each implementation, which is why the retry law is declared per implementation and admission refuses automatic retry without one.

## 5. The levels

The levels exist exactly where some selected implementation needs them; nobody models all of Linux up front. For a program they are:

1. **PackageInstalled** owns version, pinning, hold and conflict, architecture and package-database state. It is the deduplication node: one package provides many programs (`extdeps.tools` carries separate program modules for mkdir, chmod, chown and stat, each minting its own uncataloged identity, although one coreutils package provides them all), so without it every program re-runs the package lifecycle. `extdeps.dpkg` already owns installed state; the package-to-file projection (the package's real file list) is the one new upstream fact this level needs, and the readback checks the executable at the derived path, so a package that lands its binary elsewhere refuses rather than being trusted from the catalog's `bin_dir` transcription.
2. **MaterializedProgram** owns the exact executable path, format, interpreter and the execute and search permissions evaluated for the target principal's uid and groups. It is minted only after that readback, so a package that installed but whose PT_INTERP target is absent, or whose mode is wrong for this uid, never mints.
3. **LaunchableProgram** owns how a launch context invokes that executable, with two implementation families: `program_launch_via_absolute_path`, which executes the materialized path and induces no PATH requirement; and `program_launch_via_path_search`, which requires `LaunchEnvironment(launch)` -- the PATH value of THAT context (a systemd unit's Environment or systemd's compiled default, never the user's profile), ordered first-match resolution, and a readback that the resolved path IS the admitted MaterializedProgram, which is how a wrapper earlier in PATH is detected rather than silently used.

**PATH is induced by the program's own behavior, not only by the invocation route.** Git runs `ssh` by name from PATH on a fetch over ssh unless `core.sshCommand` is set, runs `gpg` for signing, and finds credential helpers and `git-lfs` by name. So `git_fetch` over an ssh remote induces `LaunchableByName(ssh @ the launch environment of git's process)` even when git itself is executed as /usr/bin/git. And the child-spawn set is a function of EFFECTIVE CONFIGURATION, not of the operation alone: this document's first version said `git_status_porcelain` induces nothing, and that was false -- with `core.fsmonitor` set to an executable hook, the exact argv `extdeps.git` `git_status_porcelain_args` produces spawns the hook (reproduced in review on git 2.47.3: status exits zero with empty stdout and the hook ran; `-c core.fsmonitor=false` prevents it). So an operation-only footprint is insufficient, and the key asks three sequential questions with three addressees (section 9): how the key affects the footprint (the upstream git model); which admitted realization this deployment uses -- disable the hook, admit a modeled hook, or refuse (root or realization policy); and what effective configuration and hook path are present at the placement (a bound observer). A no-hook realization carries the policy and DERIVES `-c core.fsmonitor=false` into its argv, after which no observation of that key is needed; the upstream authority never chooses that for every deployment. The fact is per operation and per configuration key, it lives in `extdeps.git` as an effective-configuration contract, and it is the modeling-fidelity point in its sharpest form: model that git spawns children by name under these keys, not the observation 'ssh: not found'. `extdeps.git` models none of it today (no core.sshCommand, core.fsmonitor, GIT_SSH or gpg.program row), which is why it is the second corpus-only step in section 12.

Every chain ends in an observation. An image-entailed implementation does not assert 'git is in this image': it requires `HostRunsImage(host, image@digest)`, which is observed, and still reads back the executable. Without that anchor, 'entailed' is ambient state with a better name.

## 6. The git vertical

Git is the demonstration because it is the operator's example and because `gunbc.git_use_authority` treats git-as-CLI as a legacy realization being replaced by native SCM -- which is exactly why the requirement must hang off the REALIZATION: when native SCM lands, the git binary drops out of `git.branch`'s closure with no caller touched. Nothing below is git-specific. The USE SITE does not change for it; `extdeps.git` changes exactly as landing step 1 says -- `git_program()` becomes cataloged and the effective-configuration contract (which operations spawn which children under which keys) is added as upstream facts -- and never learns about providers, placements or blueprints. That the upstream module gains only upstream facts is the check on the design.

The use site, unchanged by everything below:

```
fn checkout_is_clean(checkout: CheckoutPath) -> Result<CleanCheckout, DirtyCheckout> =
  parse_porcelain(run(executable_command(git_status_porcelain_command(repository_dir: checkout))))
```

What the compiler derives from that one call against srv5's blueprint, each line a demand node and each indentation a DependencyRelation:

```
checkout_is_clean(checkout) @ srv5/deploy-runner/gunbc-deploy.service
+- selected realization: git_cli.status
   +- LaunchableProgram(git @ gunbc-deploy.service)              via absolute path: no PATH requirement
   |  +- MaterializedProgram(git @ srv5/deploy-runner)
   |     +- PackageInstalled(debian:git @ srv5, admitted version)  via apt_family
   |     |  +- ProgramRuntime(apt-get @ srv5/root)                  via image_member <- HostRunsImage(srv5_img) [observed]
   |     |  +- ProgramRuntime(dpkg-query @ srv5/root)               via image_member <- (same node)
   |     |  +- AdmittedGrant(apt-get install git)                    from srv5_effect_envelope
   |     +- readback: executable file, PT_INTERP target, execute and search permission for deploy-runner
   +- PresentPrincipal(deploy-runner @ srv5)                        via posix_family
   |  +- ProgramRuntime(useradd @ srv5/root)                        via image_member
   |  +- PresentGroup(deploy @ srv5)
   |  +- UidAllocation(deploy-runner)                                pure fleet fact, not an ensure
   |  +- AdmittedGrant(useradd ...)
   +- SafeDirectoryAdmitted(checkout, deploy-runner)                 git >= 2.35.2 refuses dubious ownership; shares the PresentPrincipal node
```

Rebinding `packages` to image-only removes the apt subtree; rebinding `principals` to image-entailed removes the useradd subtree; neither touches `checkout_is_clean`. Four consumers of `PresentPrincipal(deploy-runner)` -- the service's User=, the execute-permission readback, safe.directory's owner comparison, and the login-shell PATH when the launch context is a shell -- reach one node, because identity deduplicates it.

The exec boundary cut that makes the whole thing load-bearing, landable in the corpus before any compiler change because `sole_constructor` and `admit_callers` already exist:

```
type MaterializedProgram sole_constructor { target: ProgramRuntime, executable: AbsolutePath, facts: RuntimeFacts }   -- minted by ensure_mint only
type ExecutableCommand sole_constructor { runtime: MaterializedProgram, arguments: List<String> }
fn executable_command(command: ArgvCommand, runtime: MaterializedProgram) -> ExecutableCommand
  -- refuses when runtime.target.program is not command.program
  -- at a call site `runtime` is OMITTED: completion resolves the hole by the provider registered for
  -- ProgramRuntime { program: command.program, placement: the effect's CommandHost placement }; a caller that
  -- passes one explicitly is supplying a derivable fact and refuses unless it states the divergence

operation RunArgv { input { command: ExecutableCommand } ... }      -- was: program: NonEmptyStr, resolved by execvp over ambient PATH
```

## 7. Two standings, and the failure audit

The typed operation contract is the source of ensure demand -- `RunArgv` demands a `MaterializedProgram` because its input type says so -- and the failure model AUDITS that contract for completeness. The equality 'requirements = preventable failure arms' is rejected: a bare git may resolve through PATH, spawn and exit zero while being the wrong build, a wrapper or from another image layer, and there is no failure arm from which to infer that the exact admitted runtime must be used; a privileged effect may succeed because the ambient process is root, and its missing grant is not thereby acceptable; and a cache provider is mandatory because repeated demand crosses an isolation boundary, which `std.materialization_ladder` derives from the demand graph, not from a cache failure.

```
type FailureClass
  = Outcome                                              -- a legitimate answer: a dirty tree, AlreadyExists on an exclusive create
  | StablePrecondition { targets: NonEmptyList<EnsureTarget> }   -- a time-stable world fact a caller could have established
  | Contended                                            -- world-dependent but not stable: ENOSPC, EAGAIN, a reset
  | FailureCauseUnclassified                             -- the realization's cause is too coarse to classify

fn spawn_failure_class(kind: SpawnFailureKind, command: ExecutableCommand) -> FailureClass {
  match kind {
    SpawnNoEntry | SpawnExecFormat                     => StablePrecondition { targets: [command.runtime.target] }
    SpawnAccessDenied                                  => FailureCauseUnclassified   -- EACCES covers mode and ancestor search, which the runtime
                                                                                   -- readback establishes, AND a noexec mount or an LSM decision,
                                                                                   -- which no modeled fact represents: until an execution-policy fact
                                                                                   -- (ExecutionPermittedAt { mount, policy }) joins the target and its
                                                                                   -- readback, mapping EACCES to the already-ensured target would
                                                                                   -- certify a cause the audit never models
    SpawnTryAgain                                      => Contended
    SpawnArgListTooLong | SpawnUnrecognized { .. }     => FailureCauseUnclassified   -- ARG_MAX is a host fact nobody has modeled
  }
}
-- audit: every StablePrecondition target is a member of the operation's ensured inputs; a target outside them is a compile error.
```

`FilesystemOtherFailure` and an unrecognized errno stay Unclassified rather than being mapped to Contended: an exhaustive match over a catch-all arm is syntactically exhaustive without being semantically exhaustive, and a read-only filesystem or a path-length defect hiding inside Other would be certified complete -- and the same discipline is why EACCES above stays Unclassified rather than being mapped to the runtime target: a noexec mount arrives as EACCES, not through Other, and the runtime readback does not model execution policy. `extdeps.transports.shell` `ShellSpawnRefused` carries its cause as text today, so the derivation has nothing to start from at the most important operation; typing it is part of the cut.

The two standings are a product, never one combined variant, because 'forgot an environmental dependency is a compiler error' and 'we have classified every operating-system failure' are different guarantees:

```
type EnsureClosureStanding = EnsureClosureClosed | EnsureClosureOpen { missing: NonEmptyList<EnsureTarget> }
type FailureAuditStanding  = FailureAuditComplete  | FailureAuditIncomplete { causes: NonEmptyList<FailureCause> }

missing ProgramRuntime(git)                 -> EnsureClosureOpen       -> not admitted
all targets closed, E2BIG unclassified      -> EnsureClosureClosed + FailureAuditIncomplete -> may run; may not claim complete failure coverage; the lens counts it
```

## 8. The lifecycle inside one demand node

Each ensure is ONE demand-engine node whose evaluation performs observe, assess, apply when decided, independent readback, mint. Edges connect ensures to each other and to consumers; the engine never sees the lifecycle, which removes the conditional-completion question an edge-per-phase encoding would have raised. Consumers depend on the readback, never on the apply report: that is the rule `std.realization_reconcile` states it cannot enforce generically -- the sealed relation decided plan, performed plan, evidence -- supplied here by the compiler-generated sequence instead of by a caller.

`gunbc.ensure` is today a pure classifier: `ensure_decide` and `ensure_reconcile` take the observation, an observation_satisfies Bool, the Application arm and the after-observation from the caller, and their own commentary says they do not establish that the lifecycle executed. Completion therefore does not merely wrap them. Six relationships are the TARGET SUBSTRATE, and they are not structural merely because this design prescribes a generated sequence; each is marked with what establishes it: (construction confinement, today) only a provider registered for the target mints its result, through the domain mint; (grounding of supplied evidence, today) a distinct readback dominates the mint and the returned value is tied to the exact target, by the required `ConvergedEffect`; (execution provenance, a declared trust boundary until the ProviderMintProvenance gate of section 3.2 is enrolled, then mechanically preventable; structural only when the engine mints that carrier) the body's actual read supplies the observation and the branch actually taken supplies Applied or NotApplied; and (landing step 6) the implementation's effect footprint (reads, writes, effect namespace, grant needs, placement) is visible to the enclosing plan, so two mutating ensures on one host are not scheduled concurrently -- a conservative first rung serializes every mutating ensure per host, and the terminal design uses their declared write sets.

The domain assesses; policy only caps. `ensure_decide`'s observation_satisfies Bool collapses every mismatch into one state, and real domains need at least Converged, Absent, RepairableDrift, Conflict, Inaccessible and Unknown: an account absent may be created, an account with the wrong supplementary group may be updated, an account name present with a different uid is a conflict that must not be renumbered, and `gunbc.gcp_service_enablement` refuses to disable because the upstream warns the API's resources may be deleted. So `assess(target, observation) -> UpsertDecision<Plan>` is the domain's, and `EnsurePolicy` is an outer cap: EnsureObserveOnly turns Apply into Refuse and leaves Noop and Refuse alone; EnsureMayApply preserves the domain decision. Unknown and Inaccessible never become Absent.

## 9. Behavior

| Situation | When it fails | What happens |
| --- | --- | --- |
| First run on srv5, git absent | -- | observe -> Absent -> Apply -> apt-get install git -> readback -> mint -> git runs as /usr/bin/git |
| Second run | -- | observe -> Converged -> Noop -> readback -> mint; apt never runs |
| git on srv5's PATH, no package family and no override | plan admission | cannot ensure ProgramRuntime(git @ srv5/deploy-runner/service): no binding; required by RunArgv <- executable_command <- checkout_is_clean. PATH is never consulted |
| Same program, deployed to srv6 with no blueprint | plan admission | the same refusal for srv6; srv5 unaffected |
| git satisfiable by image and apt, no family preference | plan admission | ambiguous: image_member, package |
| packages family EnsureObserveOnly, git absent | eval, before spawn | Refuse: observe-only binding, target does not hold; nothing is installed |
| srv5 is not running srv5_img | eval | HostRunsImage refuses; every image-entailed runtime is BlockedByRefusedPrerequisite; git never runs |
| apt succeeds, readback finds no PT_INTERP or a mode wrong for this uid | eval | nothing is minted; apt's own report never counts as success |
| installed version below admission | eval | assess -> Apply { plan: upgrade }; held or pinned elsewhere -> Refuse (Conflict) |
| spawn ENOENT after the mint (file removed in between) | runtime | classified StablePrecondition, reported as invalidated after ensure, never as git missing -- the invalidation gap, a declared drop (section 14) |
| transient apt failure | eval | retry Idempotent admits an engine retry; a secret-version ensure with no recovery law is refused retry at admission |
| a consumer passes a curl runtime for a git command | compile | cannot happen: the hole's target is derived from command.program, and an explicitly passed runtime is a refused second authority; executable_command refuses as a backstop |
| spawn returns an unrecognized errno | runtime | typed refusal; the environment may still be EnsureClosureClosed -- what may never be claimed is complete failure coverage, so FailureAuditIncomplete counts it |
| git fetch over ssh; openssh-client cataloged with a SourceApt row and apt_family admits on srv5 | -- | closure ADDS PackageInstalled(openssh-client) and LaunchableByName(ssh @ gunbc-deploy.service) to srv5's derived closure; no blueprint row per package exists to be missing, and the closure diff shows the addition |
| git fetch over ssh; no CliTool row for openssh-client | plan admission | residual addressed to the upstream module: the catalog carries no install source for ssh; the root cannot answer it |
| git status; extdeps.git does not yet model how core.fsmonitor induces a spawn | plan admission | residual addressed to the UPSTREAM MODULE: the effective-configuration contract has no row for the key, so the footprint of status is unknown (a missing model) |
| git status; the contract models core.fsmonitor, the realization neither pins it nor admits it | plan admission | residual addressed to the ROOT or realization POLICY: disable the hook (-c core.fsmonitor=false) or admit the spawn and close its launch-environment target; the upstream module cannot answer a deployment's choice |
| git status; the realization admits the hook and the effective configuration on the host is unread | plan admission | residual addressed to an OBSERVER: which effective git configuration is present at the placement; once read, the admitted spawn's target closes or refuses like any other |

## 10. Census: what this replaces on main

Main already carries four ensure vocabularies and two hand-kept substitutes for closure. Under the replacement-migration doctrine each is cut at its root, one vertical at a time, with its consumers enumerated by name in the change that deletes it:

| On main | Becomes |
| --- | --- |
| `gunbc.ensure` (pure classifier: ensure_decide, ensure_reconcile with caller Bools) | the lifecycle helper under the generated sequence; the Bools are replaced by the domain's assess and the taken branch |
| `gunbc.executor_privileged_operation` EnsureOwnedDirectory, EnsureSystemAccount, EnsureSupplementaryGroupMembership, InstallFile, SystemdEnableUnit, SystemdStartUnit | the APPLY plans of the POSIX and systemd ensure implementations; they keep their exact argv and sudoers derivation |
| `gunbc.live_deploy.spec` EnsuredDependencyKind (TailscalePackage, TmuxPackage, DevbootArtifactStoreDirectory, ...) with EnsuredSubject | DELETED: a closed list of nouns is exactly what `gunbc.ensure`'s own header forbids; replaced by PackageInstalled and OwnedDirectory targets in the deploy's closure |
| `gunbc.runner_microvm_network_apply` ensure_host_tool and `gunbc.host_effect_realize` srv3_ensure_apt_tool (one apt ensure, named after one host). Its verdict is NOT discarded -- it gates the enclosing workflow through host_tool_prerequisite_holds; the structural gap is that the exec consumer does not require the qualified runtime as an operand, so deleting the ensure leaves the exec well-typed | package_installed_via_apt and program_runtime_from_package; ensure_host_tool returns the MaterializedProgram the exec then requires, which is the first real consumer |
| `gunbc.host_effect_realize` srv3_ensure_apt_tool / srv3_ensure_apt take the subject AND a separately supplied package name: the install consumes the package while the readback checks the subject, which admits disagreement and an unrelated package mutation; require_version_probe is a Bool beside the declared version requirement | the package is derived once from the selected install source and the version admission from the CliTool row; neither is a second parameter |
| `extdeps.tools` apt_package_of and apt_binary_path_of fold installable_via and take the FIRST apt source, so a second source becomes list-order policy | two admissible sources are a residual addressed to the root (a preference) or a catalog defect, never resolved by position; D14 does not inherit these folds as its unique-answer resolver |
| `shell.Exec.RunArgvOutcome` keeps the same raw-string spawn input as RunArgv; its caller is the declared seed-route exception | migrated or explicitly fenced in the same cut as RunArgv -- changing RunArgv alone leaves a public escape route |
| `gunbc.host_cli_dependency` hand-kept per-workflow tool rosters checked at run start | the derived closure; the rosters are the manual, runtime form of what closure derives at plan time |
| `gunbc.tool_readiness` ToolReadiness and `extdeps.tools` CliTool.installable_via | the catalog row stays and becomes consumed: installable_via selects the package; the readiness verdict is subsumed by the ensure's readback |
| `extdeps.transports.shell` ShellSpawnRefused with a text cause; `shell.Exec.RunArgv` taking program: NonEmptyStr | a closed SpawnFailureKind with its classification; RunArgv over ExecutableCommand |
| `extdeps.git` git_program() as uncataloged_program (UncatalogedDebt); most ProgramIdentity mints uncataloged | a CliTool row for git with its apt source; the uncataloged population becomes the footprint lens's first census |
| `gunbc.command_runner` spawning by name on the seed route | unchanged under v1's frozen semantics, visible as a declared drop whose trigger is the host_effect_apply ExecutableCommand handler the module's own dissolution row already names |

## 11. Where this sits on the ladder

Today the class -- a program, account or path reaching an effect with nothing establishing it -- sits at mitigation: a runtime error naming the symptom (command not found, status=217/USER, dubious ownership), caught by hand rosters where someone remembered. The exec cut lifts program use to structurally guaranteed within the corpus: an Accepted program cannot hand a bare spelling to spawn. Plan admission lifts the closure itself to structurally guaranteed at the executable boundary: an open closure is not an admitted program. The invalidation gap (section 14) and the failure audit's unclassified causes remain honestly below that, each declared.

## 12. Landing order

1. **Corpus only, three steps, no compiler change.** (a) A `CliTool` row for git with its `SourceApt` package, and `git_program()` cataloged through it -- git is the operator's example and has no install source today. (b) The effective-configuration contract in `extdeps.git`: per operation, which configuration keys can induce a child spawn by name (`core.fsmonitor` for status, `core.sshCommand` / `GIT_SSH` for ssh remotes, `gpg.program` for signing, credential helpers, git-lfs) and what each realization PINS, so a fetch over ssh induces the launch-environment target and an unpinned key is a residual rather than a zero footprint. (c) The developer-host blueprint: observe-only, no base, the shape every `gunbc run` outside the fleet closes against.
2. **The exec cut, still corpus only.** `MaterializedProgram` and `ExecutableCommand` with sole_constructor; `ensure_host_tool` returns the former and the microvm-network exec consumes it; the ambient-PATH red (binary on PATH, no binding, refusal); `SpawnFailureKind` replacing the text cause.
3. **v2 footprint lens.** For each function and placement: the ensure targets it induces, the provenance path of each, and the EnsureClosureStanding and FailureAuditStanding -- reported, not enforced. Its first run is the census of open footprints on main, including every uncataloged program mint.
4. **v2 language substrate, synthetic only.** Elaboration of omitted operands as typed holes (section 1.5, D13 generalized) with the route partition of section 1.2; residuals with provenance and addressee; `ensure_mint` confinement to registered providers; recursive completion with deduplication; the lifecycle helpers that replace `gunbc.ensure`'s caller Bools; the controls: missing route, two routes, cycle, type mismatch, unevaluable subject, no retry law, and a call site supplying a derivable fact. The typed policy-departure form lands in this step and is tested before the derivable-operand rule rejects anything: rejection without a tested legitimate green is a wall with no door.
5. **Verticals.** ntfy `RunningService` first, because `gunbc.auth.approval_ntfy_deployment` already documents the exact missing hierarchy and the false daemon-reload-as-installed precedent; then artifact acquisition (parent directory and transfer runtime inherited, no-source static, unreachable contended); then the secret version (retry refused without the attempt-identity path, admitted with it). `EnsuredDependencyKind` is deleted in the cut that moves live_deploy over.
6. **Effect-footprint ordering.** Per-host serialization of mutating ensures first; declared write sets after.
7. **Plan admission enforces.** An open EnsureClosureStanding at the executable boundary refuses; the derived closure projection lands with it.

## 13. Discriminating controls

- Git really installed and on PATH on the test host; no package family bound and no override: plan admission refuses, with the provenance path through executable_command to the use site. The most important control: it proves ambient state is not a provider.
- Bind the apt family: the closure derives install, readback and execute in that order and real git runs from the absolute path the readback returned.
- Rebind to image-only: the install node disappears; the caller is byte-identical.
- Provider bound for srv6, step placed on srv5: placement refusal.
- Executable present, PT_INTERP target absent: refusal before spawn.
- Principal lacks execute or search permission: permission refusal naming the uid.
- dpkg observer inaccessible: Refuse (Inaccessible), never Absent and never Apply { plan: install }.
- Git reached through helper A and helper B: one target node, two provenance paths.
- Remove the registered provider of MaterializedProgram, leaving the consumer intact: closure refuses and the exec consumer does not construct. Then attempt to construct the carrier directly, outside the domain mint: refused, which is the mint-confinement control.
- Weaken one join of ProviderMintProvenance (readback from another attempt, decision not produced from the observation that ran): the provenance gate goes red; restoring it greens. Until the gate exists this control is unauthorable, which is exactly why execution provenance stands at a trust boundary and not above.
- Pass the CORRECT git runtime explicitly at a call site where completion would have derived it: refused as a second authority. Then supply it through the declared policy-departure form: admitted, with the divergence visible. Both halves are required; the first alone is a wall with no door.
- Omit the apt-get binding under an apt-backed git: the refusal reports the transitive chain, and the image binding greens it.
- Two ensures wanting different end states for one subject (two uids): ambiguity at closure, not a race at runtime.
- A secret-version ensure without an attempt identity: retry admission refuses; with the exact-version receipt path: admitted.
- Negative controls: a memory lease, current endpoint reachability, an effect-grant decision, a materialization obligation, protocol compatibility and roadmap membership do not appear in any ensure footprint.

## 14. Declared frontier and drops

- **Invalidation** -- a mount over /usr after the install, a chmod after the readback, a group change after the service called initgroups. A fact established at one node and consumed at another is invalidated by an effect on the same placement whose write set overlaps the fact's footprint and is not ordered outside the window. Declared as a DESIGN section 4b(3) drop at the first landing; its trigger is the write-set comparison of landing step 6, and until then a post-mint spawn failure is reported as invalidated-after-ensure rather than as the missing-dependency class.
- **The seed route.** `gunbc.command_runner` keeps spawning by name under v1's frozen semantics; the drop's trigger is the `host_effect_apply` ExecutableCommand handler. The native route's `v1.compiler.emit_rust` `emit_shell_call` emits a PATH-resolved spawn today and is the same drop's second member.
- **Dynamic targets.** A program identity computed at runtime (a caller-supplied binary path) has no plan-time target and refuses; a resolver concept is not added until a real consumer needs one.
- **Selection among families** is one `std.decision` policy declared on the blueprint; automating a per-target choice beyond that preference order is later work under section 3d, and two admissible families with no preference refuse as ambiguous.

## 15. Open decisions for the operator

1. The two names: `HostBlueprint` for the authored input and `HostClosure` for the derived output, as used here, or other spellings.
2. RULED in review: the derived closure is reported first as a mandatory, HEAD-pinned before/after diff gated on the change, and committed as a generated projection with landing step 7. Remaining open: the diff's publication surface (a check annotation, a PR comment, or a projection under docs/).
3. FailureAuditIncomplete as a counted standing that still admits execution (this document), versus the review's stronger 'an unclassified cause refuses the operation's use' -- which today would make every filesystem operation unusable through FilesystemOtherFailure.
4. The override form: how a call site states a DESIGN section 3b divergence when it must supply a fact completion would otherwise derive (a pinned binary for a reproduction, an explicit principal for a test), so that the first legitimate exception is a declared departure rather than a workaround. RULED in review: deferral through landing step 1 is acceptable because the catalog, configuration-contract and observe-only-blueprint definitions need no syntax; enforcement of the derivable-operand rule in step 4 requires the typed, visible departure form to exist and be tested first. Remaining open: its spelling.
5. Materialization's one new residual carrier: tolerated staleness for a world read, which the ladder's identity, scope and retention judgments cannot derive and which today is the time bound `EvidenceFreshness` lacks.
6. Confirming 'ensure' over 'upsert' as the one word, given `gunbc.ensure` already exists; if 'upsert' is preferred, `gunbc.ensure` is renamed in the same change and no second word is introduced.

## Dissolution trigger (DESIGN §6)

Delete this document when landing steps 1 through 7 of its section 12 are landed and green on the required floor: omitted operands are completed by the route partition and registered providers mint their results under ensure_mint confinement, shell.Exec.RunArgv takes an ExecutableCommand and the ambient-PATH control is enrolled red-then-green, the host blueprint and derived closure carriers exist with the git vertical closed against srv5 and the developer host, the footprint lens reports both standings, and plan admission refuses an open closure at the executable boundary. At that point the carriers, their witnesses and the rung-drop rows for invalidation and the seed route are the authority for what environmental closure means, and this document would be a second statement of it.
