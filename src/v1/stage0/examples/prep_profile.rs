// Scratch measurement harness (NOT for commit): runs the floor's strict
// preparation alone so reconcile's interior can be stack-sampled.
use v1_compiler::cli_run;

fn main() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let t = std::time::Instant::now();
    let r = cli_run::prepare_repository_once(&roots, &cli_run::floor_prepared_subject_exclusions());
    match r {
        Ok((prepared, _inv)) => eprintln!(
            "prep ok in {:?}: modules_resolved={} excluded={}",
            t.elapsed(),
            prepared.modules_resolved,
            prepared.modules_excluded
        ),
        Err(e) => eprintln!("prep refused in {:?}: {}", t.elapsed(), &e[..e.len().min(2000)]),
    }
}
