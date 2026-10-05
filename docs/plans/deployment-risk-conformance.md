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
   - Deferred to deliverable 2 (review 74937): `DeploymentRiskInterface` and `DeploymentHarmMagnitudeBet` land with their first executing consumer, the operator-disposition gate (which actions on a deployment need an operator), and the bet's magnitude is grounded as an order of magnitude over `std.measure` `Duration`, not a bare `Int`.
1b. **Machine qualification migration**: one motion over machine_intake (re-census on main first; coordinate warm-crane-577's mtcollins1 untangle lane). eager-gull-22 handed it to this lane.
2. **Repoint the prod pins** through `prod_role_selection` (broker owner, fabric storage placement, approval client/store/trust, serve/dispatch entry points, apply's prod peer → `DashboardProtectedSibling`, `fleet_reach_endpoint`). RED: a fixture selection moves the broker owner and storage placement.
3. **Split shared bindings**, one PR each: ntfy → Codex account/home → admitted ref → publication identity/GitHub App → R2/GCP → OIDC. Each adds a field to `DeploymentExternalBindingSet` and either a TestRisk resource (operator approval first) or a standing refusal.
4. **One spec per deployment**; retire the `live_deploy.desired` rung drop by its own trigger. Renames: `srv1_production_desired_deployment → srv1_primary_desired_deployment`, `roadmap_belt_production_spawn_mode → roadmap_belt_operational_spawn_mode`.
5. **Conformance row** `deployment-risk conformance` in `gunbc.design_argument conformance_domains`, once the home is consumed.

Widened scope (side chat): classify every convergence subject as deployment-bound (Spark serving/experiments/seat grants, fabric storage, approval broker, R2 keyed by class + `BucketPurpose`, IAM targets, DNS/routes/certs, runner workloads, mutable caches), host-shared (Spark Wi-Fi/SSH/admin grants, KVM, base networking, toolchains), or fleet-control (root IAM writer, account admin). Pure computation identity and content-addressed outputs gain no deployment key. Manual convergence requests take `DeploymentConvergence { deployment } | HostResourceConvergence { host, resource } | FleetControlConvergence { operation }`.

## Shared-binding table (as of deliverable 1)

| Binding | State | Reason |
|---|---|---|
| ntfy topic | modeled in the binding set; ProdRisk only; TestRisk **refuses** at the resolver | broker path still reads `approval_ntfy_topic` directly until 3.1 |
| GitHub App (publication) | modeled; ProdRisk only; TestRisk **refuses** at the resolver | publishers still name `gunbai_*_declared` until 3.4 |
| Codex account and home | still shared | 3.2 |
| admitted ref | still shared (labs on `refs/fleet/desired`) | 3.3 |
| R2 / GCP / WIF | still shared | 3.5; coordinate royal-moth-86 (#13035) |
| OIDC client | still shared, unmodeled | 3.6 |
