// Throwaway probe: print live reflection-scan vs roster deltas for the lens-hygiene gates.
use v1_compiler::cli_run;

#[test]
fn probe_inert_carrier_delta() {
    let live = cli_run::inert_carrier_names_live();
    println!("INERT LIVE ({}): {:?}", live.len(), live);
}

#[test]
fn probe_non_fold_residue_delta() {
    let sites = cli_run::non_fold_residue_live_sites();
    let unrostered: Vec<&String> = sites
        .iter()
        .filter(|s| !cli_run::non_fold_residue_site_is_rostered(s.as_str()))
        .collect();
    println!(
        "NFR sites={} roster={} unrostered={} stale={}",
        sites.len(),
        cli_run::non_fold_residue_roster_size(),
        cli_run::non_fold_residue_unrostered_count(),
        cli_run::non_fold_residue_stale_roster_count()
    );
    println!("NFR UNROSTERED: {unrostered:?}");
}
