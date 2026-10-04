# Deployment-risk conformance

Operator direction 2026-10-03; vocabulary fixed the same day by a side-chat ruling relayed through silent-lark-156. Home: `gunbc.deployment_risk`.

## Terms (approved; use these names)

| Term | Meaning | Is NOT |
|---|---|---|
| `DeploymentId` | the stable identity of one Deployment whose desired state, mutable state, and external attachments move together; an instance realizes that Deployment and carries this identity | a host, machine, process, release stage, dashboard `instance_id` |
| `DeploymentRiskClass = TestRisk \| ProdRisk` | coarse harm class, DERIVED from `ProdRoleSelection` only | a number, machine qualification, stage, branch, host |
| `DeploymentHarmMagnitudeBet` | Fermi order of magnitude of harm conditional on doing the wrong thing, typed as a §4d bet | a probability, a fact, the class, an SLA |
| `ProdRoleSelection = NoProdRole \| ProdRoleHeldBy { deployment }` | the sole switch (`prod_role_selection`) | inferred from srv1, "live", main, a host |
| `DeploymentExternalBindingSet` | the external attachments of exactly one class; absent set refuses | a fallback, a shared default, a borrow |
| `DeploymentRiskInterface` | what each deployment exposes (identity, harm bet) and consumes (class, bindings) | a `DeploymentSpec`, an actuator, a stage model |
| `DeploymentReleaseStage` | independent rollout axis | out of scope; next candidate |

`DesiredDeployment` (selection row) and `DeploymentSpec` (desired realization) stay distinct from `DeploymentId`. `MachineOperatingEnvironment` is replaced by `MachineQualificationPolicy { admits: DeploymentRiskClass }`: a machine admits a class, never holds one. A `ProdRoleHeldBy` assignment refuses unless its host admits `ProdRisk`.

## Deliverables

1. **Model** (this PR): `gunbc.deployment_risk`; `HostDashboardInstance.deployment` (required, all six constructors); `dashboard_instance_risk_class`; the failure-mode row `a_test_deployment_acts_on_the_prod_deployments_state`; RED `test.claim.deployment_risk_witness_test`.
   - Declared frontier (§3c): `resolve_deployment_bindings` and `dashboard_instance_risk_class` gain production consumers in deliverables 2 and 3.1.
   - Declared frontier (review 74937; merry-tern-58 ruling 2026-10-04): `DeploymentRiskInterface`, `DeploymentHarmMagnitudeBet` and the operator-disposition gate are **deferred until an operator policy differs by class**. Today's policy (gunbc#13124: automatic repair is "no" for every deployment) is class-independent, so a gate consulting the class would decide nothing (§3c decoration), and a class-dependent policy is the operator's call. No carrier is minted before then. Design note for when the bet lands: its magnitude is an order of magnitude over `Duration` (today `std.types` `Duration`; std has no order-of-magnitude carrier yet), e.g. a decade exponent beside its unit, typed with `extdeps.external_authority` `CitedFigureStanding`, and it never decides the class.
1b. **Machine qualification migration**: one motion over machine_intake (re-census on main first; coordinate warm-crane-577's mtcollins1 untangle lane). eager-gull-22 handed it to this lane.
2. **Repoint the prod pins** through `prod_role_selection` (broker owner, fabric storage placement, approval client/store/trust, serve/dispatch entry points, apply's prod peer → `DashboardProtectedSibling`, `fleet_reach_endpoint`). RED: a fixture selection moves the broker owner and storage placement.
   - Landed: one resolver, `gunbc.roadmap_dashboard_instance` `prod_role_realization` (the instance roster filtered by `deployment`; no second table), refusing with `ProdRoleRealizationRefusal` (`ProdRoleUnheld` | `ProdRoleHolderUnrealized` | `ProdRoleHolderAmbiguous`) and minting the only holder token, `ProdRoleHolder` (`sole_constructor`). A spec carries it as `gunbc.live_deploy.spec` `RoleSingletonHolding`; the broker unit and front door and the fabric store's unit, route, socket, root and directories are built from it and move between targets as one subject (no instance-id ownership; the `"srv1-live"` literal and `operator_host_srv1` are gone). Also following the realization: the approval client origin, the GCP IAM converge job's runner (refused job under NoProdRole) and the approvals app's host (`gunbc.auth.approval_broker_placement`), and `DashboardProtectedSibling` (same-host siblings whose class is `ProdRisk`). Broker-side modules do NOT resolve the role (it would undo #13132's narrowed native-broker closure): the role decides where the broker is installed, the serving broker reads its own host at serve time (`approval_broker_serving_base_url`), and identity trust is derived from the broker unit's own bind row (`approval_broker_listen_host`).
   - Role-singleton convergence (`gunbc.live_deploy.role_singleton_convergence`, gating every apply): desired realization × an observation of every roster host, reusing `member_observe` legs. `RoleSingletonNoop` | `RoleSingletonInstall` (only when nothing is observed anywhere) | `RoleSingletonRefused` with `RoleSingletonMoveUnimplemented`, `RoleSingletonRemovalUnimplemented`, `RoleSingletonHostUnobservable` (unobserved is never absent), `RoleSingletonHoldersAmbiguous`, `RoleSingletonOwnerUnmapped`. **Next trigger:** Move and Remove actuations (persistent-state move, singleton removal) become arms beside the existing ones.
   - Bounded residual: the host universe is the roster's targets (`desired_deployment_roster`), so a singleton on a host outside the roster is invisible to the observation. Trigger: a roster row for that host.
   - Operational dependency: an apply observes every roster host, so the applying runner needs ssh reach to every other roster target (srv1 ↔ srv2 today); an unreachable host refuses the apply.
   - Honest non-pins (not debt): the nullary serve/belt/dispatch/launch entry points are srv1-live's *own* entry points (`serve_function`, now checked roster-wide by `test.claim.serve_function_binding_witness_test`), so they must not follow the role; `fleet_reach_endpoint` is a host-shared account fact. Not claimed: that an unbound test deployment refuses everywhere — `belt_dispatch_instance_cli`'s absent-env default stays with the belt owner's lane.
3. **Split shared bindings**, one PR each: ntfy → Codex account/home → admitted ref → publication identity/GitHub App → R2/GCP → OIDC. Each adds a field to `DeploymentExternalBindingSet` and either a TestRisk resource (operator approval first) or a standing refusal.
4. **One spec per deployment**; retire the `live_deploy.desired` rung drop by its own trigger. Renames: `srv1_production_desired_deployment → srv1_primary_desired_deployment`, `roadmap_belt_production_spawn_mode → roadmap_belt_operational_spawn_mode`.
5. **Conformance row** `deployment-risk conformance` in `gunbc.design_argument conformance_domains`, once the home is consumed.

Widened scope (side chat): classify every convergence subject as deployment-bound (Spark serving/experiments/seat grants, fabric storage, approval broker, R2 keyed by class + `BucketPurpose`, IAM targets, DNS/routes/certs, runner workloads, mutable caches), host-shared (Spark Wi-Fi/SSH/admin grants, KVM, base networking, toolchains), or fleet-control (root IAM writer, account admin). Pure computation identity and content-addressed outputs gain no deployment key. Manual convergence requests take `DeploymentConvergence { deployment } | HostResourceConvergence { host, resource } | FleetControlConvergence { operation }`.

## Shared-binding table (as of deliverable 2)

| Binding | State | Reason |
|---|---|---|
| ntfy topic | modeled in the binding set; ProdRisk only; TestRisk **refuses** at the resolver | broker path still reads `approval_ntfy_topic` directly until 3.1 |
| GitHub App (publication) | modeled; ProdRisk only; TestRisk **refuses** at the resolver | publishers still name `gunbai_*_declared` until 3.4 |
| Codex account and home | still shared | 3.2 |
| admitted ref | still shared (labs on `refs/fleet/desired`) | 3.3 |
| R2 / GCP / WIF | still shared | 3.5; coordinate royal-moth-86 (#13035) |
| approval broker unit and front door, fabric store unit/route/socket/root/directories (role singletons) | one subject on the holder's spec (`RoleSingletonHolding`); a role change refuses until Move/Remove land; NoProdRole → no target holds any | deliverable 2 |
| GCP IAM converge job runner | holder's host; NoProdRole → refused job | deliverable 2 |
| approval loopback origin / identity trust | follows the role; NoProdRole → origin refused, trust not established | deliverable 2 |
| OIDC client | still shared, unmodeled | 3.6 |
