# Environmental closure by ensure: targets, blueprints and the derived host closure (D14)

> **Status: design authority for D14 of the demand-engine program (`gunbc.plans.demand_engine_program` section 7), ruled 2026-10-01 (operator, session wonderful-feynman: the direction was endorsed over four rounds and this document was written at the operator's request as the ruling's authority). No implementation lands from this document.** It SUPERSEDES the 'program runtimes as derived prerequisites' D14 draft and the requirement-obligation vocabulary considered earlier in the same session -- RequirementObligation, RequirementClosure as a public graph beside the demand graph, DependencyDemand read as an obligation, a universal ExecProposition coproduct, an authored OperandSlot, a fifteen-kind relation law product as part of this vertical, and a combined ClosedWithUnauditedCauses standing. None of those names is minted. D13 stands unchanged: this document is D13's provider binding applied to environmental state, with the lifecycle inside one demand node.
> **Operator intention carried (2026-09-30 and 2026-10-01, the operator's words).** 'This reverses the modeling direction from please remember to model everything to if you forget to model this the interface will not let you use it (during compile time).' 'Git is a member of Program -- and all programs must be installed -- so what is effectively required for this program is how are you going to install this program; if they plan to actually emit the code and run it in production, they cannot compile it unless they provide a layout of a machine that satisfies -- all the way down: git needs to both be installed and on the path, and then the user needs to also answer how it gets on the path.' 'This will basically force us into using shared libraries -- the dependencies are truly complex, but they are managed globally and uniformly.' 'I see this relationship everywhere and need a single word to identify it: that should be an upsert is much easier for developers than the kind of modeling we are discussing.' 'Model the underlying behavior, not our observations.'
> **Third-party review folded in (2026-10-01).** Five corrections to the session's earlier draft are adopted: the typed operation contract is the source of ensure demand and the failure model is an audit of it, never the sole derivation; the package lifecycle is a first-class level between the program identity and the executable; PATH is an explicit, optional level rather than a bypassed one; environmental closure and failure-audit completeness are two standings, not one combined variant; realizations induce environmental requirements, abstract semantic operations do not. Three amendments are made to that review here: a program's own child spawns induce launch-environment targets per operation, not only the caller's invocation route; the developer's machine and every CI runner are placements with blueprints, observe-only where nothing may be installed; and the package-to-path step is read back from the package's real file population, never trusted from a catalog transcription.

## 1. The law

Environmental usability is established by ensure. A selected realization may consume only qualified environmental values, never a raw environmental identity. Its typed inputs induce ensure targets -- including the launch-environment targets of programs that realization spawns by name -- and the generic implementations selected for those targets recursively induce further targets. A composition root supplies a host blueprint: base observations, provider families with their policies, launch environments, principals and the admission envelope; per-target bindings are overrides, not the ordinary route. The compiler closes the selected executable against that blueprint into a derived host closure -- the concrete package, program, path, principal, permission, artifact, service and readback plan -- which is a reviewable projection. An executable whose closure is missing, ambiguous, cyclic, incompatible or unevaluable at plan time is not admitted; a library may expose unresolved targets as part of its interface. Every host an executable runs on, the developer's own included, is a placement with a blueprint. Only an ensure implementation may mint its ensured result, and only after an effective readback; implementations expose their effect footprint and carry an admitted recovery law before automatic retry. Environmental closure and failure-audit completeness are separate standings. Pure facts, access admissions, materialization judgments, resource leases and contended runtime outcomes keep their existing authorities. Ambient machine state never discharges a target.

What this buys, in the operator's framing: more upstream contract modeling creates more derived requirements, more deployment modeling creates more satisfiable closures, more cross-layer modeling exposes more contradictions before a host is touched, and forgetting to model a required interaction makes the program uncloseable rather than fragile. The use site writes none of it.

## 2. The one word is ensure

The review phrase is: **a raw environmental identity reached a consumer -- that should be an ensure.** Precisely, an ensure is appropriate when all five hold: there is a desired state or qualified usability proposition; that state can be observed or entailed; absence, drift, conflict and unreadability are distinguishable; an admitted implementation may converge it; and a readback can ground the result. That covers program runtimes, packages, users and groups, paths and ownership, services, grants' host artifacts, acquired artifacts, cache stores, images, network configuration and host capabilities. It does not replace pure dataflow, reference binding, membership, GeneratedFrom provenance, VerifiedBy evidence, module imports or presentation containment, which stay the relations they are.

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

### 3.2 Ensure implementation

A domain function that establishes its target and returns the domain's qualified result. Its typed parameters are its prerequisites; there is no authored dependency list. The only hand-written prerequisite in the example below is the principal the readback must run as; everything else is inferred from the operations the clauses call.

```
ensure package_installed_via_apt(target: PackageInstalled) -> InstalledPackage
  retry Idempotent                                       -- admitted law: a pinned apt install
  writes [dpkg_database(target.host), tree("/usr", target.host)]
{
  observe  => dpkg_query_status(target)                  -- runs dpkg-query: requires ProgramRuntime(dpkg-query @ root), inferred
  assess   => apt_package_assess(target, observed)       -- Absent -> Apply { plan: install }; below admission -> Apply { plan: upgrade }; held -> Refuse (Conflict)
  apply    => apt_get_install(plan)                      -- privileged RunArgv: requires ProgramRuntime(apt-get @ root) + AdmittedGrant, inferred
  readback => dpkg_query_status(target)                  -- the SAME observation, independent of apt's own report
  mint     => InstalledPackage { target, files: dpkg_file_list(target) }
}

ensure program_runtime_from_package(target: ProgramRuntime,
    package: ensure InstalledPackage for package_of(target.program),   -- the catalog row names the package
    who:     ensure PresentPrincipal for principal_present(target.placement)) -> MaterializedProgram
{
  observe  => stat_exec_interp(path_in(package.files, target.program), who)
  assess   => present and executable for who -> Noop; else -> Refuse        -- this implementation never writes
  readback => stat_exec_interp(...)
  mint     => MaterializedProgram { target, executable, facts }
}
```

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

The family-level root is what makes adding a cataloged program free: `extdeps.tools` `InstallSource` already carries `SourceApt { package, bin_dir }`, so the catalog selects the concrete package and the blueprint selects only the family. Its one real cost is that a code change adding a program to a pipeline expands what srv5 carries with no blueprint edit, so the deployment consequence would be invisible in review. The derived closure is therefore a reviewable projection -- the generated-artifact pattern this repository already gates -- so 'srv5 now carries git' is still a diff a reviewer sees. Whether it is committed or reported by the footprint lens on each change is an open decision (section 15).

The blueprint need not describe everything on srv5. It must be closed over everything the selected executable demands: unknown facts outside that closure are irrelevant, unknown facts inside it are errors. It is assembled from existing authorities -- the OCI `ImageManifest`, `extdeps.dpkg` installed state, `extdeps.posix.identity` identities, the fleet account allocations, the unit files, `std.effect_grant` -- and no generic machine module re-authors them.

Two rules follow. **Library versus executable:** a library may carry unresolved ensure targets in its interface, and an emitted executable or deployment may not -- binding a program to a placement is what forces the blueprint to answer. **Every host is a placement:** the first friction a developer hits is `gunbc run` on their own machine, and the answer is an observe-only blueprint (no base, every family EnsureObserveOnly), under which a missing git is a typed eval refusal, never an install and never a fall back to PATH. CI runners already have converge recipes and get real blueprints. A blueprint that is missing is a plan-admission refusal, not an escape hatch.

### 3.4 Qualified result

Each domain keeps its own sealed result carrier -- `MaterializedProgram`, `InstalledPackage`, `PresentPrincipal`, `OwnedDirectory`, `InstalledFile`, `RunningService` -- constructible only inside an ensure implementation whose result is that type, and only after its readback. There is no generic `Ensured<T> { value, evidence }` record, for the reason `std.realization_reconcile` records about itself: the generic layer can classify decision, application and grounding, but it cannot prove the decided plan is the plan performed or that the evidence arose from that performance; the domain authority mints the carrier that binds subject, operation and readback. The protocol is generic; the earned values are domain-specific. The result stays tied to its full target through minting and consumption, so a materialized curl cannot satisfy git's parameter by type.

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

**PATH is induced by the program's own behavior, not only by the invocation route.** Git runs `ssh` by name from PATH on a fetch over ssh unless `core.sshCommand` is set, runs `gpg` for signing, and finds credential helpers and `git-lfs` by name. So `git_fetch` over an ssh remote induces `LaunchableByName(ssh @ the launch environment of git's process)` even when git itself is executed as /usr/bin/git, while `git_status_porcelain` induces nothing. That fact is per operation, it lives in `extdeps.git` as an upstream behavior, and it is the modeling-fidelity point in its sharpest form: model that git spawns children by name, not the observation 'ssh: not found'. `extdeps.git` models none of it today (no core.sshCommand, GIT_SSH or gpg.program row), which is why it is the second corpus-only step in section 12.

Every chain ends in an observation. An image-entailed implementation does not assert 'git is in this image': it requires `HostRunsImage(host, image@digest)`, which is observed, and still reads back the executable. Without that anchor, 'entailed' is ambient state with a better name.

## 6. The git vertical

Git is the demonstration because it is the operator's example and because `gunbc.git_use_authority` treats git-as-CLI as a legacy realization being replaced by native SCM -- which is exactly why the requirement must hang off the REALIZATION: when native SCM lands, the git binary drops out of `git.branch`'s closure with no caller touched. Nothing below is git-specific, and `extdeps.git` does not change for it; that is a check on the design.

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
type MaterializedProgram ensured for ProgramRuntime sole_constructor { target: ProgramRuntime, executable: AbsolutePath, facts: RuntimeFacts }
type ExecutableCommand sole_constructor { runtime: MaterializedProgram, arguments: List<String> }
fn executable_command(command: ArgvCommand, runtime: ensure MaterializedProgram for program_runtime(program: command.program, placement: here)) -> ExecutableCommand
  -- refuses when runtime.target.program is not command.program; `here` is the effect's CommandHost placement

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
    SpawnNoEntry | SpawnAccessDenied | SpawnExecFormat => StablePrecondition { targets: [command.runtime.target] }
    SpawnTryAgain                                      => Contended
    SpawnArgListTooLong | SpawnUnrecognized { .. }     => FailureCauseUnclassified   -- ARG_MAX is a host fact nobody has modeled
  }
}
-- audit: every StablePrecondition target is a member of the operation's ensured inputs; a target outside them is a compile error.
```

`FilesystemOtherFailure` and an unrecognized errno stay Unclassified rather than being mapped to Contended: an exhaustive match over a catch-all arm is syntactically exhaustive without being semantically exhaustive, and a read-only filesystem, a noexec mount or a path-length defect hiding inside Other would be certified complete. `extdeps.transports.shell` `ShellSpawnRefused` carries its cause as text today, so the derivation has nothing to start from at the most important operation; typing it is part of the cut.

The two standings are a product, never one combined variant, because 'forgot an environmental dependency is a compiler error' and 'we have classified every operating-system failure' are different guarantees:

```
type EnsureClosureStanding = EnsureClosureClosed | EnsureClosureOpen { missing: NonEmptyList<EnsureTarget> }
type FailureAuditStanding  = FailureAuditComplete  | FailureAuditIncomplete { causes: NonEmptyList<FailureCause> }

missing ProgramRuntime(git)                 -> EnsureClosureOpen       -> not admitted
all targets closed, E2BIG unclassified      -> EnsureClosureClosed + FailureAuditIncomplete -> may run; may not claim complete failure coverage; the lens counts it
```

## 8. The lifecycle inside one demand node

Each ensure is ONE demand-engine node whose evaluation performs observe, assess, apply when decided, independent readback, mint. Edges connect ensures to each other and to consumers; the engine never sees the lifecycle, which removes the conditional-completion question an edge-per-phase encoding would have raised. Consumers depend on the readback, never on the apply report: that is the rule `std.realization_reconcile` states it cannot enforce generically -- the sealed relation decided plan, performed plan, evidence -- supplied here by the compiler-generated sequence instead of by a caller.

`gunbc.ensure` is today a pure classifier: `ensure_decide` and `ensure_reconcile` take the observation, an observation_satisfies Bool, the Application arm and the after-observation from the caller, and their own commentary says they do not establish that the lifecycle executed. The language feature therefore does not merely wrap them. Six things become structural: only an `ensure fn` mints its ensured result; the body's actual read supplies the observation; the branch actually taken supplies Applied or NotApplied; a distinct readback dominates the mint; the returned value is tied to the exact target; and the implementation's effect footprint (reads, writes, effect namespace, grant needs, placement) is visible to the enclosing plan, so two mutating ensures on one host are not scheduled concurrently -- a conservative first rung serializes every mutating ensure per host, and the terminal design uses their declared write sets.

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
| a consumer passes a curl runtime for a git command | compile | cannot happen: the target expression derives git from command.program; executable_command refuses as a backstop |
| spawn returns an unrecognized errno | runtime | typed refusal; FailureAuditIncomplete counts it; never reported as a closed environment |
| git fetch over ssh, openssh-client not in the blueprint | plan admission | LaunchableByName(ssh @ gunbc-deploy.service): PATH is the systemd default and nothing in srv5's blueprint installs ssh into it; candidates: SourceApt openssh-client |

## 10. Census: what this replaces on main

Main already carries four ensure vocabularies and two hand-kept substitutes for closure. Under the replacement-migration doctrine each is cut at its root, one vertical at a time, with its consumers enumerated by name in the change that deletes it:

| On main | Becomes |
| --- | --- |
| `gunbc.ensure` (pure classifier: ensure_decide, ensure_reconcile with caller Bools) | the lifecycle helper under the generated sequence; the Bools are replaced by the domain's assess and the taken branch |
| `gunbc.executor_privileged_operation` EnsureOwnedDirectory, EnsureSystemAccount, EnsureSupplementaryGroupMembership, InstallFile, SystemdEnableUnit, SystemdStartUnit | the APPLY plans of the POSIX and systemd ensure implementations; they keep their exact argv and sudoers derivation |
| `gunbc.live_deploy.spec` EnsuredDependencyKind (TailscalePackage, TmuxPackage, DevbootArtifactStoreDirectory, ...) with EnsuredSubject | DELETED: a closed list of nouns is exactly what `gunbc.ensure`'s own header forbids; replaced by PackageInstalled and OwnedDirectory targets in the deploy's closure |
| `gunbc.runner_microvm_network_apply` ensure_host_tool and `gunbc.host_effect_realize` srv3_ensure_apt_tool (one apt ensure, named after one host, whose verdict is read back but never consumed by the exec) | package_installed_via_apt and program_runtime_from_package; ensure_host_tool returns the MaterializedProgram the exec then requires, which is the first real consumer |
| `gunbc.host_cli_dependency` hand-kept per-workflow tool rosters checked at run start | the derived closure; the rosters are the manual, runtime form of what closure derives at plan time |
| `gunbc.tool_readiness` ToolReadiness and `extdeps.tools` CliTool.installable_via | the catalog row stays and becomes consumed: installable_via selects the package; the readiness verdict is subsumed by the ensure's readback |
| `extdeps.transports.shell` ShellSpawnRefused with a text cause; `shell.Exec.RunArgv` taking program: NonEmptyStr | a closed SpawnFailureKind with its classification; RunArgv over ExecutableCommand |
| `extdeps.git` git_program() as uncataloged_program (UncatalogedDebt); most ProgramIdentity mints uncataloged | a CliTool row for git with its apt source; the uncataloged population becomes the footprint lens's first census |
| `gunbc.command_runner` spawning by name on the seed route | unchanged under v1's frozen semantics, visible as a declared drop whose trigger is the host_effect_apply ExecutableCommand handler the module's own dissolution row already names |

## 11. Where this sits on the ladder

Today the class -- a program, account or path reaching an effect with nothing establishing it -- sits at mitigation: a runtime error naming the symptom (command not found, status=217/USER, dubious ownership), caught by hand rosters where someone remembered. The exec cut lifts program use to structurally guaranteed within the corpus: an Accepted program cannot hand a bare spelling to spawn. Plan admission lifts the closure itself to structurally guaranteed at the executable boundary: an open closure is not an admitted program. The invalidation gap (section 14) and the failure audit's unclassified causes remain honestly below that, each declared.

## 12. Landing order

1. **Corpus only, three steps, no compiler change.** (a) A `CliTool` row for git with its `SourceApt` package, and `git_program()` cataloged through it -- git is the operator's example and has no install source today. (b) The child-spawn facts in `extdeps.git`: which operations run ssh, gpg, credential helpers or git-lfs by name, keyed by the config that overrides each (`core.sshCommand`, `gpg.program`), so a fetch over ssh induces the launch-environment target. (c) The developer-host blueprint: observe-only, no base, the shape every `gunbc run` outside the fleet closes against.
2. **The exec cut, still corpus only.** `MaterializedProgram` and `ExecutableCommand` with sole_constructor; `ensure_host_tool` returns the former and the microvm-network exec consumes it; the ambient-PATH red (binary on PATH, no binding, refusal); `SpawnFailureKind` replacing the text cause.
3. **v2 footprint lens.** For each function and placement: the ensure targets it induces, the provenance path of each, and the EnsureClosureStanding and FailureAuditStanding -- reported, not enforced. Its first run is the census of open footprints on main, including every uncataloged program mint.
4. **v2 language substrate, synthetic only.** `ensure fn`; ensured-result mint confinement; the target-qualified input; recursive parameter resolution with deduplication; the generated lifecycle (the Bools leave `gunbc.ensure`); the missing, ambiguous, cycle, type-mismatch, unevaluable-target and no-retry-law controls.
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
- Delete the ensure that mints MaterializedProgram: the exec consumer does not construct.
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
2. Whether the derived closure is committed as a generated projection with the drift gate, or reported by the footprint lens as a per-change diff. This document assumes the lens report first and the committed projection with landing step 7.
3. FailureAuditIncomplete as a counted standing that still admits execution (this document), versus the review's stronger 'an unclassified cause refuses the operation's use' -- which today would make every filesystem operation unusable through FilesystemOtherFailure.
4. Surface syntax: an `ensure` parameter keyword with a `for` target expression (this document), versus inferring the demand from a sole_constructor parameter of a closure-only type with no new syntax. The second needs the rule that such a parameter is a demand, which D13 needs regardless.
5. Confirming 'ensure' over 'upsert' as the one word, given `gunbc.ensure` already exists; if 'upsert' is preferred, `gunbc.ensure` is renamed in the same change and no second word is introduced.

## Dissolution trigger (DESIGN §6)

Delete this document when landing steps 1 through 7 of its section 12 are landed and green on the required floor: the ensure modality mints its results under compiler confinement, shell.Exec.RunArgv takes an ExecutableCommand and the ambient-PATH control is enrolled red-then-green, the host blueprint and derived closure carriers exist with the git vertical closed against srv5 and the developer host, the footprint lens reports both standings, and plan admission refuses an open closure at the executable boundary. At that point the carriers, their witnesses and the rung-drop rows for invalidation and the seed route are the authority for what environmental closure means, and this document would be a second statement of it.
