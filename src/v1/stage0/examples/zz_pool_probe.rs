//! Scratch probe: for a failing bare name, is its declaring module IN THE POOL
//! (so the defect is resolution) or ABSENT (so the defect is closure)?
//! Deleted before merge; not a witness.
fn main() {
    let roots: Vec<String> = vec!["dag".into(), "src/v2".into()];
    let entry = "dag/std/materialization_provider.dag";
    let sources = match v1_compiler::cli_run::load_sources_for_entry(&roots, entry) {
        Ok(s) => s,
        Err(e) => {
            println!("LOAD FAILED: {}", &e[..e.len().min(400)]);
            return;
        }
    };
    println!("closure size: {} files", sources.len());
    // The modules that declare the names the corpus reports as not-found.
    for want in [
        "dag/std/nat.dag",
        "dag/std/algebra.dag",
        "dag/std/stack.dag",
        "dag/std/types.dag",
        "dag/std/citation.dag",
        "dag/std/realization_measurement.dag",
        "dag/std/observation.dag",
        // The magnitude hypothesis: ObservationCounts fields are the FULLY
        // QUALIFIED std.nat.Nat, which aliases CommutativeSemiring<Magnitude>.
        // If magnitude/algebra are absent the alias cannot ground to an Int
        // checkpoint, and an integer literal stops inhabiting it -- which is
        // exactly the 51-diagnostic class, and is a pool question rather than a
        // binding one.
        "dag/std/magnitude.dag",
        "dag/std/integer.dag",
    ] {
        let present = sources.iter().any(|s| s.path.ends_with(want));
        println!(
            "{:>8}  {}",
            if present { "IN POOL" } else { "ABSENT" },
            want
        );
    }
}
