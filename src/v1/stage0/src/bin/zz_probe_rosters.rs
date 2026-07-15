use v1_compiler::cli_run::{
    inert_carrier_names_live, non_fold_residue_live_sites, non_fold_residue_site_is_rostered,
};

fn main() {
    println!("=== INERT CARRIER live names ===");
    for n in inert_carrier_names_live() {
        println!("{}", n);
    }
    println!("=== NON_FOLD_RESIDUE unrostered sites ===");
    for s in non_fold_residue_live_sites() {
        if !non_fold_residue_site_is_rostered(s) {
            println!("{}", s);
        }
    }
}
