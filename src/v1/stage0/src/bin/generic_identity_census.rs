#![allow(clippy::disallowed_macros)]
//! Transport for `v1.tests.claim.generic_identity_census`.
//!
//! The census is `.dag` and pure: subject sources go in as `SourceFile` DATA and the receipt comes
//! out as a `String`. This binary reads the subject files and writes the receipt, and decides
//! nothing about the measurement. It is the same split as `carrier_realization_census`, for the
//! same reason: `compile_to_resolved` over an authored source vector does not run under the
//! interpreter.
//!
//! Every failure arm refuses with a located message and a non-zero exit. A receipt with no
//! `UNOBSERVED` row is refused as well: the census states the populations it cannot reach in
//! every receipt, so their absence means the receipt is not the census's.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::SourceFile;
use v1_compiler::v1_tests_claim_generic_identity_census::generic_identity_census_from_sources;

fn refuse(message: &str) -> ! {
    eprintln!("generic-identity-census: REFUSED: {message}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        refuse("usage: generic_identity_census <receipt.tsv> <subject.dag>...");
    }
    let receipt_path = args[0].clone();
    let subject_paths = &args[1..];

    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    for path in subject_paths {
        match std::fs::read_to_string(path) {
            Ok(content) => sources.push(Rc::new(SourceFile {
                path: path.clone(),
                content,
            })),
            Err(err) => refuse(&format!("could not read subject source {path}: {err}")),
        }
    }

    let receipt = generic_identity_census_from_sources(Rc::new(sources.into()));
    if receipt.starts_with("REFUSED") {
        refuse(&format!("census refused: {receipt}"));
    }

    let observed = receipt
        .lines()
        .filter(|line| line.starts_with("observed\t"))
        .count();
    let unobserved = receipt
        .lines()
        .filter(|line| line.starts_with("UNOBSERVED\t"))
        .count();
    if unobserved == 0 {
        refuse("receipt carries no UNOBSERVED row; refusing a receipt that hides its blind spot");
    }
    // No observed row is a real answer only for a subject with no generic and no kernel container,
    // which no subject worth censusing is, so it refuses rather than reporting a zero that a
    // failed walk would produce identically.
    if observed == 0 {
        refuse("census observed no row; refusing to write an empty receipt");
    }

    match std::fs::write(&receipt_path, &receipt) {
        Ok(()) => println!(
            "generic-identity-census: observed={observed} unobserved_populations={unobserved} subjects={} receipt={receipt_path}",
            subject_paths.len()
        ),
        Err(err) => refuse(&format!("could not write receipt {receipt_path}: {err}")),
    }
}
