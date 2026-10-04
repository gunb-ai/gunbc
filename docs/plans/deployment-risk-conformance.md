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
   - Landed: one resolver, `gunbc.roadmap_dashboard_instance` `prod_role_realization` (the instance roster filtered by `deployment`; no second table), refusing with `ProdRoleRealizationRefusal` (`ProdRoleUnheld` | `ProdRoleHolderUnrealized` | `ProdRoleHolderAmbiguous` | `ProdRoleHolderUnactuatable`) and minting the only holder token, `ProdRoleHolder` (`sole_constructor`). **Actuatability join:** `gunbc.live_deploy.desired` `prod_role_actuated_realization` joins the holder to the one `desired_deployment_roster`; a holder with no row (MacBook, srv2-preview, srv2's lab) refuses as `ProdRoleHolderUnactuatable`. Every role follower reads the joined realization. A spec carries the token as `gunbc.live_deploy.spec` `RoleSingletonHolding`, and the singleton members -- broker unit and front door, plus fabric store unit, route, socket, root, directories and **ownership marker** -- are built from it and move between targets as one subject. The marker is the dashboard ownership-marker rule (`dashboard_ownership_marker_in`) applied to the store root, so an old store with no unit stays observable. The broker is GIVEN its address (`GUNBC_APPROVAL_BROKER_BASE_URL`, from `gunbc.auth.approval_broker_placement`); broker-side modules never resolve the role (#13132's closure). Also following the realization: approval client origin, the GCP IAM job's runner, the approvals app host, `DashboardProtectedSibling`.
   - Role-singleton convergence (`gunbc.live_deploy.role_singleton_convergence`):
     - **Observation.** It sits behind `RoleSingletonReadingSource`: an ssh probe today, host self-reports later, and which one production uses is still owed by the operator. Sightings are kept at (host, deployment, member) until multiplicity is adjudicated, and every roster deployment's broker, fabric unit and marker are probed on every roster host.
     - **Decision.** Possible outcomes:
       - `RoleSingletonNoop`: exactly one complete realization of the desired holder on its target.
       - `RoleSingletonInstall`: nothing observed anywhere.
       - `RoleSingletonRefused`, with one of these causes: move, removal, `RoleSingletonDesiredRealizationRefused`, unmarked, unobservable, two-hosts, misplaced, two-on-one, ambiguous, broker-without-store, partial, stale-spec.

       Only `ProdRoleUnheld` means no prod.
     - **Admission.** `RoleSingletonAdmission` (`sole_constructor`) binds the exact spec. Its mint is sealed (`admit_callers`) to the canonical assessor `assess_and_admit(spec, own)`, which reads the actuated desired realization and the ssh observation itself. The assessor, the token's readers (`admitted_spec`, `admitted_decision`) and every admitted apply step are in turn sealed to the two canonical assess-and-act compositions (`live_deploy_apply_via_transport`, `live_deploy_transaction_decided`). So a token can be neither forged, obtained elsewhere nor replayed later. Fixtures use the pure `classify_role_singleton_admission`. The apply receipt records `role_singleton_disposition` (Noop, Installed or RefusedBeforeApply).
     - **Unmarked legacy store (no automated adoption).** A realization with its broker and fabric unit but no store ownership marker refuses as `RoleSingletonUnmarked`. Today's srv1 holder predates the marker, so it is the expected case. Under the manual-resume ruling (#13124) it waits for the operator, and the refusal text names exactly what to write. **Operator's one-time disposition:**
       - on `srv1`, write `/opt/gunbc/fabric-storage/.gunbc-dashboard-instance-srv1-live`;
       - mode 0644, owner briansrls;
       - content exactly `deployment=srv1-daily-workspace` + newline + `host=srv1` + newline.

       This is the same path and content the holder spec's own marker ensure writes. After it, the next apply observes a complete realization and converges as Noop.
     - **Next trigger.** Move and Remove actuations become arms beside these.
   - Bounded residual: the host universe is the roster's targets (`desired_deployment_roster`), so a singleton on a host outside the roster is invisible to the observation. Trigger: a roster row for that host.
   - Operational dependency: an apply observes every roster host, so the applying runner needs ssh reach to every other roster target (srv1 ↔ srv2 today); an unreachable host refuses the apply.
   - Honest non-pins (not debt): the nullary serve/belt/dispatch/launch entry points are srv1-live's *own* entry points (`serve_function`, now checked roster-wide by `test.claim.serve_function_binding_witness_test`), so they must not follow the role; `fleet_reach_endpoint` is a host-shared account fact. Not claimed: that an unbound test deployment refuses everywhere — `belt_dispatch_instance_cli`'s absent-env default stays with the belt owner's lane.
3. **Split shared bindings**, one PR each: ntfy → Codex account/home → admitted ref → publication identity/GitHub App → R2/GCP → OIDC. Each adds a field to `DeploymentExternalBindingSet` and either a TestRisk resource (operator approval first) or a standing refusal.
4. **One spec per deployment**; retire the `live_deploy.desired` rung drop by its own trigger. Renames: `srv1_production_desired_deployment → srv1_primary_desired_deployment`, `roadmap_belt_production_spawn_mode → roadmap_belt_operational_spawn_mode`.
   - Naming residue: `gunbc.host_layout` `srv1_gunbc_approval_broker_root` ("/opt/gunbc/approval-broker") is host-agnostic in value but srv1 in name; rename it with the per-deployment names (not a prod pin; D2 review, 2026-10-04).
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
