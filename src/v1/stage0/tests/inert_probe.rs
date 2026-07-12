// Throwaway probe: print live inert-carrier scan vs the .dag roster delta.
use v1_compiler::cli_run;

#[test]
fn probe_inert_carrier_delta() {
    let live = cli_run::inert_carrier_names_live();
    println!("LIVE ({}): {:?}", live.len(), live);
}
