//! REAL-ENTRY RESOLVE WITNESSES FOR THE ORG-ROSTER REST ROUTE (fleet_machine_authority P8d
//! follow-up, node adhoc-cf8a613a-843; parent vivid-koi-387).
//!
//! THE GAP THIS CLOSES. The #16156 demanded-grain regression (5e3151317af, 2026-11-10 08:45Z)
//! made `gunbc run` typecheck only the declarations the demanded wave realizes, and a rest
//! service-qualified call reads rows no authored lexeme spells (the operation surface keyed by the
//! SERVICE name; the carrier `RestResult` synthesized at call time). The refusal surfaced as
//! `MethodExistenceUndecided` at dag/gunbc/fleet/organization_runner_roster_read.dag:874 --
//! `method 'ListOrganizationRunnersPage', receiver 'Primitive(github.ActionsSelfHostedRunners)'`
//! -- before any network read, on the P8d dry-run entry `jit_dead_sweep_dry_run_wet` AND on the
//! pre-existing fleet-converge mode `org_runner_roster_observe` (same route). P8d's witnesses
//! passed because every claim in `test.claim.runner.runner_jit_dead_sweep_witness` supplies its
//! own roster: none of them ever resolved a real entry, so no witness executed the route the
//! production dry run executes (`gunbc.design_argument::witness_one_interface`: supplying inputs
//! may never remove the last execution of the real path). These witnesses ARE that last
//! execution: they drive the SAME call production `gunbc run --entry <file> --function <fn>`
//! makes (`cli_run::resolve_entry_graph_demand_acquired_scoped`, main.rs's run path) and stop at
//! resolve -- no eval, no network, no fleet effect.
//!
//! SIBLING INSTANCES (root_generalization, gunbc.design_argument::root_generalization). The class
//! is `github.<Service>.<Op>(` service-qualified calls whose declaring module is imported for
//! wire rows only; the corpus carries ~70 such call sites (the roster travels with the PR). Already
//! recorded instances of the same #16156 fallout: wire-contract rows (fixed on main by 4aab999ddc3
//! #16241), rest carrier `RestResult` (fix in review, #16279 `pr1/service-root-scope`),
//! `main_wet_one` `NoSuchVariable github.Pulls` (equivalence-harness main-wet arm), and
//! `PullRequestPage/Observe` `MethodExistenceUndecided` (harness note at fd021fee54f, attributed
//! to a service-reachability rule that was unowned as of this writing). This witness is route
//! coverage for the roster pair; it refuses to fix the resolver, so it is RED on a main that
//! carries #16156 without the seed fix, and must be GREEN at the head that lands the seed.
use std::rc::Rc;

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

/// The production run interface, up to the point of the observed failure: the exact scoped
/// demand-acquired resolve `gunbc run --entry <entry_file> --function <function>` performs
/// (main.rs's run arm). Ok(()) means resolve settled with no blocking diagnostic; Err carries the
/// joined refusal text, which is what srv1 receipts quote.
fn production_run_resolve(entry_file: &str, function: &str) -> Result<(), String> {
    repo_root();
    let requested = vec![function.to_string()];
    crate::cli_run::resolve_entry_graph_demand_acquired_scoped(
        &["dag".to_string()],
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

/// THE PAIRING CONTROL (`witness_one_interface` inhabitance claim). The two witnesses above green
/// by ABSENCE OF REFUSAL -- a route deleted from the entry would green them too. This control
/// asserts the demanded grain genuinely reaches the REST roster route out of each real entry:
/// deleting the integration (or letting the demanded set shrink below it) makes THIS fail.
/// Mirrors `demanded_typecheck_equivalence`'s grain helpers: the same census + realization the
/// demanded typecheck threads, computed over the entry's real closure.
fn assert_route_realized_from(entry_file: &str, seeds: &[&str], expected: &[&str]) {
    let root = repo_root();
    let seed_file = Rc::new(crate::v1_compiler_compile::SourceFile {
        path: entry_file.to_string(),
        content: std::fs::read_to_string(root.join(entry_file)).unwrap(),
    });
    let mut sources =
        super::entry_resolve::resolve_seeded_compile_closure(vec![seed_file], &["dag".to_string()])
            .unwrap();
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    let mut intern = crate::v1_std_core::empty_intern_table();
    let mut parsed = Vec::new();
    let mut si = crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>();
    for s in &sources {
        let idx = crate::v1_std_core::build_newline_index(s.path.clone(), s.content.clone());
        si = crate::v1_rt::rc_map_insert(si, s.path.clone(), idx.clone());
        let toks = crate::v1_compiler_tokenize::tokenize(
            s.content.clone(),
            s.path.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        );
        let p = crate::v1_compiler_parse::parse_with_table(
            toks,
            crate::v1_rt::rc_map_insert(
                crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>(),
                idx.file.clone(),
                idx.clone(),
            ),
            intern.clone(),
        );
        intern = p.intern_table.clone();
        parsed.push(
            p.result
                .module
                .clone()
                .unwrap_or_else(|| panic!("parse {}", s.path)),
        );
    }
    let graph = crate::v1_compiler_resolve::resolve_modules(Rc::new(parsed.into()), si.clone());
    let census =
        crate::v1_compiler_infer::build_symbol_index_census(graph.modules.clone(), si.clone());
    let seed_labels: Vec<String> = seeds.iter().map(|s| s.to_string()).collect();
    let seeds_vec: Rc<im::Vector<String>> =
        Rc::new(seed_labels.iter().cloned().collect::<im::Vector<String>>());
    let realization = crate::v1_compiler_infer::realization_of(
        graph.modules.clone(),
        seeds_vec,
        census,
        si.clone(),
    );
    for name in expected {
        assert!(
            crate::v1_compiler_infer::realization_includes(realization.clone(), name.to_string()),
            "the demanded grain seeded from {entry_file} {seed_labels:?} must realize the REST \
             roster route declaration `{name}` -- if this fails, the witnesses above are green by \
             absence, not by resolution"
        );
    }
}

/// THE P8d WITNESS. The dry-run entry fleet-converge mode `jit_dead_registration_sweep_dry_run`
/// runs (`runner_jit_dead_sweep_run.dag:410`) reads the org roster through the typed REST route
/// (line 224 -> `organization_runner_roster_report_via_rest` -> `rest_roster_page` -> the
/// `github.ActionsSelfHostedRunners.ListOrganizationRunnersPage` call at
/// organization_runner_roster_read.dag:874). Resolve must settle with no blocking diagnostic.
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

/// THE OLDER SIBLING. fleet-converge mode `org_runner_roster_observe` predates P8d and reads the
/// same route (organization_runner_roster_read.dag:1150 -> ... -> :874). Its entry resolve is the
/// evidence that the defect is older than P8d, and the regression canary that P8d's witnesses
/// were not.
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
/// spelled nowhere -- the victim's import shape at
/// gunbc/fleet/organization_runner_roster_read.dag:20 and :27. Refuses
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

/// The inhabitance control for both witnesses: the REST roster route is realized from each real
/// entry's demanded grain, so the two witnesses cannot be greened by the route going unreachable.
#[test]
fn the_rest_roster_route_is_demanded_from_both_real_entries() {
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
                "dag/gunbc/fleet/organization_runner_roster_read.dag",
                &["organization_runner_roster_observe_wet"],
                &["organization_runner_roster_report", "rest_roster_page"],
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
