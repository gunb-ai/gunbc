//! REAL-ENTRY RESOLVE WITNESSES FOR THE ORG-ROSTER REST ROUTE (fleet_machine_authority P8d
//! follow-up, node adhoc-cf8a613a-843; parent vivid-koi-387).
//!
//! THE GAP THIS CLOSES. The #16156 demanded-grain regression (5e3151317af, 2026-11-10 08:45Z)
//! made `gunbc run` typecheck only the declarations the demanded wave realizes, and a rest
//! service-qualified call reads rows no authored lexeme spells (the operation surface keyed by the
//! SERVICE name; the carrier `RestResult` synthesized at call time). The refusal surfaced as
//! `MethodExistenceUndecided` on the
//! `github.ActionsSelfHostedRunners.ListOrganizationRunnersPage` call inside
//! `gunbc.fleet.organization_runner_roster_read::rest_roster_page` --
//! `method 'ListOrganizationRunnersPage', receiver 'Primitive(github.ActionsSelfHostedRunners)'`
//! -- before any network read, on every entry that reaches the REST realization
//! (`organization_runner_roster_report_via_rest`): the P8d dry-run entry
//! `gunbc.runner.runner_jit_dead_sweep_run::jit_dead_sweep_dry_run_wet`, both entries of
//! `gunbc.runner.runner_lost_boot_sweep_run` (the route's original consumer, #14303), and
//! `gunbc.fleet.runner_connectivity_observe_wet::runner_connectivity_observe_wet`. The route's
//! OTHER realization -- the gh shell route taken by the pre-existing fleet-converge mode
//! `org_runner_roster_observe` (`organization_runner_roster_report` -> `gh_roster_transport` ->
//! `github.CliOrgSelfHostedRunners.ListOrgRunnersPage`, transport shell) -- resolves GREEN: the
//! asymmetry localizes the defect to the rest realization's synthesized carrier read. P8d's
//! witnesses passed because every claim in `test.claim.runner.runner_jit_dead_sweep_witness`
//! supplies its own roster: none of them ever resolved a real entry, so no witness executed the
//! route the production dry run executes (`gunbc.design_argument::witness_one_interface`:
//! supplying inputs may never remove the last execution of the real path). These witnesses ARE
//! that last execution: they drive the SAME call production `gunbc run --source-root dag
//! --source-root src/v2 --entry <file> --function
//! <fn>` makes (`cli_run::resolve_entry_graph_demand_acquired_scoped` with the fleet-converge
//! steps' exact source-root argv, main.rs's run path) and
//! stop at resolve -- no eval, no network, no fleet effect.
//!
//! SIBLING INSTANCES (root_generalization, gunbc.design_argument::root_generalization). The class
//! is `github.<Service>.<Op>(` service-qualified calls whose declaring module is imported for
//! wire rows only; the corpus carries 80 such occurrences, 52 of them production lines across 28
//! modules -- the census is recorded in the SIBLING INSTANCES paragraph of the ledger row
//! gunbc.recurring_failure_mode::rest_service_carrier_row_unrealized_by_the_run_demand_grain_refuses_a_misattributed_no_method_surface.
//! Already recorded instances of the same #16156 fallout: wire-contract rows (fixed on main by
//! 4aab999ddc3 #16241), rest carrier `RestResult` (seed LANDED: 356cec0a347, gunbc#16279),
//! `main_wet_one` `NoSuchVariable github.Pulls` (equivalence-harness main-wet arm), and
//! `PullRequestPage/Observe` `MethodExistenceUndecided` (harness note at fd021fee54f, attributed
//! to a service-reachability rule that was unowned as of this writing). This witness is route
//! coverage for the rest-route entry family (P8d dry run, lost-boot sweep dry/wet, connectivity
//! observe) plus the green gh-route canary; it refuses to fix the resolver, so its rest-route
//! arms were RED on the unseeded demanded-grain main and are GREEN on seeded main by EXECUTION:
//! srv1 arm C at 12480021961 (cargo test -p v1-compiler --lib, both source roots) reports 7
//! passed / 1 failed -- the one failure, `the_runner_connectivity_observe_entry_...`, refuses for
//! a DIFFERENT main defect (constructor-call admission: `stage_controller_app_key` admits no
//! connectivity staged-key caller), tracked apart as
//! `gunbc.integration_concern::runner_connectivity_observe_admitted_by_no_controller_key_stager`,
//! not by this class. Receipts recorded in the ledger row above.
//!
//! HAND-WRITTEN-RUST RECEIPT: this file is hand-written witness Rust beside the generated stage0
//! mirror, deferred to the landing lane named by the tracking row
//! gunbc.integration_concern::grain_witness_tests_are_hand_written_rust_in_the_stage0_mirror
//! (follow-up node adhoc-0eedbb6e-a35), same as demanded_typecheck_equivalence.rs.
fn repo_root() -> std::path::PathBuf {
    let root = std::env::current_dir()
        .unwrap()
        .ancestors()
        .find(|d| d.join("dag").exists() && d.join("Cargo.toml").exists())
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&root).unwrap();
    root
}

/// The EXACT `--source-root` argv the production fleet-converge dry-run steps pass to
/// `gunbc run` for the witnessed entries (.github/workflows/fleet-converge.yml:
/// `gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry
/// dag/gunbc/runner/runner_jit_dead_sweep_run.dag --function jit_dead_sweep_dry_run_wet`, and the
/// lost-boot step likewise). The tests run at the repo root, so the relative spellings name the
/// same directories the workflow passes absolutely. One definition, used by BOTH the resolve
/// witness and the inhabitance controls: a witness may not see a wider closure than production.
fn run_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

/// The production run interface, up to the point of the observed failure: the exact scoped
/// demand-acquired resolve `gunbc run --entry <entry_file> --function <function>` performs
/// (main.rs's run arm). Ok(()) means resolve settled with no blocking diagnostic; Err carries the
/// joined refusal text, which is what srv1 receipts quote.
fn production_run_resolve(entry_file: &str, function: &str) -> Result<(), String> {
    repo_root();
    let requested = vec![function.to_string()];
    crate::cli_run::resolve_entry_graph_demand_acquired_scoped(
        &run_source_roots(),
        entry_file,
        &requested,
    )
    .map(|_| ())
}

fn assert_run_resolves(entry_file: &str, function: &str) {
    match production_run_resolve(entry_file, function) {
        Ok(()) => {}
        Err(cause) => {
            panic!("{function} (entry {entry_file}) refused on the run demand grain: {cause}")
        }
    }
}

/// THE PAIRING CONTROLS (`witness_one_interface` inhabitance claim). The rest-route witnesses
/// green by ABSENCE OF REFUSAL -- a route deleted from the entry would green them too. These
/// controls assert the demanded grain genuinely reaches the named refusal-site route out of each
/// real entry (and the shell route out of the canary): deleting an integration (or letting the
/// demanded set shrink below it) makes THIS fail.
/// REUSES `demanded_typecheck_equivalence`'s shared grain helpers -- `graph_from` (the one
/// closure/parse/resolve pipeline) and `grain_demand` (the one census + realization rule) --
/// per root_generalization: this file adds no second grain implementation.
fn assert_route_realized_from(entry_file: &str, seeds: &[&str], expected: &[&str]) {
    let (_sources, _intern, si, modules, _graph) =
        super::demanded_typecheck_equivalence::graph_from_scoped(entry_file, &run_source_roots());
    let demand = super::demanded_typecheck_equivalence::grain_demand(&modules, seeds, &si);
    let realization = match &*demand {
        crate::v1_compiler_infer::DeclarationDemand::OnlyDeclarations { realization, .. } => {
            realization.clone()
        }
        other => panic!("grain_demand must yield OnlyDeclarations, got {other:?}"),
    };
    for name in expected {
        assert!(
            crate::v1_compiler_infer::realization_includes(realization.clone(), name.to_string()),
            "the demanded grain seeded from {entry_file} {seeds:?} must realize the REST \
             roster route declaration `{name}` -- if this fails, the witnesses above are green by \
             absence, not by resolution"
        );
    }
}

/// THE P8d WITNESS. The dry-run entry fleet-converge mode `jit_dead_registration_sweep_dry_run`
/// runs (`gunbc.runner.runner_jit_dead_sweep_run::jit_dead_sweep_dry_run_wet`) reads the org
/// roster through the typed REST route (its call of
/// `gunbc.fleet.organization_runner_roster_read::organization_runner_roster_report_via_rest`,
/// which calls `rest_roster_page`, whose call of
/// `github.ActionsSelfHostedRunners.ListOrganizationRunnersPage` is the refusal site). Resolve
/// must settle with no blocking diagnostic.
/// RED on main at 7e57510 (the refusal this node names); GREEN only with the demanded-grain seed
/// fix landed.
#[test]
fn the_jit_dead_sweep_dry_run_entry_resolves_under_the_run_demand_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_run_resolves(
                "dag/gunbc/runner/runner_jit_dead_sweep_run.dag",
                "jit_dead_sweep_dry_run_wet",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// THE OLDER GREEN CANARY. fleet-converge mode `org_runner_roster_observe` predates P8d and reads
/// the roster through the gh REALIZATION (`gunbc.fleet.organization_runner_roster_read::
/// organization_runner_roster_observe_wet` down through `organization_runner_roster_report` ->
/// `roster_walk_pump` -> `gh_roster_transport`, whose
/// `github.CliOrgSelfHostedRunners.ListOrgRunnersPage` is a SHELL-transport operation declared in
/// extdeps.github.ci_runner). It never reaches the rest realization's carrier read, so it resolves
/// GREEN on every head -- and the asymmetry is itself the localizer: the module resolves, the
/// shell route resolves, only the REST realization's synthesized carrier read refuses. Kept as the
/// entry-level witness for the observe mode's interface and as a regression canary for the
/// shell-transport service surface on the demanded grain.
#[test]
fn the_org_runner_roster_observe_entry_resolves_under_the_run_demand_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_run_resolves(
                "dag/gunbc/fleet/organization_runner_roster_read.dag",
                "organization_runner_roster_observe_wet",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// A REST-ROUTE SIBLING OLDER THAN P8d. fleet-converge mode `lost_boot_registration_sweep`
/// (dry-run arm; the route's original consumer, #14303) reads the roster through
/// `organization_runner_roster_report_via_rest` in `plan_without_deleting`, so the same
/// refusal-site declarations hang on this entry too. RED on a main that carries #16156 without
/// the carrier seed (same `ListOrganizationRunnersPage` refusal), GREEN at the seeded head —
/// with the P8d arm and the connectivity arm this witnesses that the refusal belongs to the
/// REST route and not to any one entry, which is what makes a line-874 patch the wrong fix.
#[test]
fn the_lost_boot_sweep_dry_run_entry_resolves_under_the_run_demand_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_run_resolves(
                "dag/gunbc/runner/runner_lost_boot_sweep_run.dag",
                "lost_boot_sweep_dry_run_wet",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// THE REST-ROUTE THIRD SIBLING, older than P8d: the runner-connectivity observe entry
/// (`gunbc.fleet.runner_connectivity_observe_wet`, #14303 route family) reads the roster through
/// the same REST realization from its own module. DECLARED RED ON MAIN -- but by a DIFFERENT
/// defect than this file's class: srv1 (arm C, 12480021961) resolves this entry with NO
/// ListOrganizationRunnersPage refusal (the carrier repair reaches it) and refuses instead
/// 'constructor call admission refused: gunbc.github_effect_perform.stage_controller_app_key
/// refuses call from ...observe_under_staged_key' -- an admit_callers enrollment gap the entry has
/// always had, tracked as gunbc.integration_concern::runner_connectivity_observe_admitted_by_no_
/// controller_key_stager (the allowlist names only lost-boot/qualification/service-slot/jit
/// staged-key callers). Do NOT fix the allowlist here; this arm greens when that enrollment lands.
#[test]
fn the_runner_connectivity_observe_entry_resolves_under_the_run_demand_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_run_resolves(
                "dag/gunbc/fleet/runner_connectivity_observe_wet.dag",
                "runner_connectivity_observe_wet",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// THE DISCRIMINATING FIXTURE PAIR (shape attribution, offered to #16279's fixture family; mgr
/// rule froze that PR, so the pair lives here). Both consumers call the SAME rest service
/// (`fleetroster.RosterCarrierRest`, namespace root not module root, like the victim) from a
/// function BODY with a wire-rows-only import of the declaring module. The only difference is
/// whether the carrier `RestResult` is NAMED by an authored lexeme. This arm names it and so
/// must be GREEN on every head: if it ever goes red, the fixture shape itself broke and no
/// resolver conclusion may be drawn from the sibling witness below.
#[test]
fn the_named_carrier_rest_consumer_fixture_resolves_under_the_run_demand_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_run_resolves(
                "dag/test/fixture/demanded_scope/rest_carrier_wire_rows_grain_consumer_named.dag",
                "roster_carrier_page_count_named",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The un-named arm: wire rows + the two carrier ARMS imported, the carrier identity itself
/// spelled nowhere -- the victim's import shape in
/// `gunbc.fleet.organization_runner_roster_read` (wire rows from
/// `extdeps.github.actions_self_hosted_runners`, `RestAnswered`/`RestRefused` from
/// `extdeps.transports.rest`, carrier unmentioned). Refuses
/// `MethodExistenceUndecided` naming `Primitive(fleetroster.RosterCarrierRest)` on a main that
/// carries #16156 without the carrier seed; must be GREEN at the head that lands it. Together
/// with the control above this isolates the failure to the synthesized carrier read and nothing
/// else about the call shape.
#[test]
fn the_unnamed_carrier_rest_consumer_fixture_resolves_under_the_run_demand_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_run_resolves(
                "dag/test/fixture/demanded_scope/rest_carrier_wire_rows_grain_consumer.dag",
                "roster_carrier_page_count",
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The inhabitance controls: no witness may be greened by its route going unreachable. Each
/// refused (rest-route) witness gets a positive control that its refusal-site declarations are
/// realized from that entry's demanded grain, and the green canary gets the same for its shell
/// route — so the canary is green because the shell surface resolves, not because the test
/// watched nothing.
#[test]
fn the_rest_roster_route_is_demanded_from_the_rest_route_entries() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_route_realized_from(
                "dag/gunbc/runner/runner_jit_dead_sweep_run.dag",
                &["jit_dead_sweep_dry_run_wet"],
                &[
                    "organization_runner_roster_report_via_rest",
                    "rest_roster_page",
                ],
            );
            assert_route_realized_from(
                "dag/gunbc/runner/runner_lost_boot_sweep_run.dag",
                &["lost_boot_sweep_dry_run_wet"],
                &[
                    "organization_runner_roster_report_via_rest",
                    "rest_roster_page",
                ],
            );
            assert_route_realized_from(
                "dag/gunbc/fleet/runner_connectivity_observe_wet.dag",
                &["runner_connectivity_observe_wet"],
                &[
                    "organization_runner_roster_report_via_rest",
                    "rest_roster_page",
                ],
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The canary's inhabitance control: the observe entry demands its gh shell route, so its GREEN
/// verdict means the shell service surface resolved under the demanded grain, not that the
/// witness watched an empty grain.
#[test]
fn the_gh_shell_route_is_demanded_from_the_observe_entry() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            assert_route_realized_from(
                "dag/gunbc/fleet/organization_runner_roster_read.dag",
                &["organization_runner_roster_observe_wet"],
                &["organization_runner_roster_report", "roster_walk_pump"],
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
