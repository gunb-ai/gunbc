#![allow(clippy::disallowed_macros)]
//! Transport for `v1.tests.claim.carrier_realization_census`.
//!
//! The census itself is `.dag` and is pure: subject sources go in as `SourceFile` DATA and the
//! receipt comes out as a `String`. This binary does the two things a pure module cannot -- read
//! the subject files and write the receipt -- and decides nothing about the measurement. That
//! split is the design's `(b')` shape (docs/plans/carrier-realization-arbiter-repair-design.md):
//! the predicate under measurement is IMPORTED by the census rather than re-derived here, so this
//! file holds no copy of any compiler decision and cannot drift from one.
//!
//! It exists because the census cannot run under the interpreter: `compile_to_resolved` over an
//! authored source vector fails there with `NoSuchField Node.ident`, the nested-compile defect
//! recorded in `v1.compiler.emit_rust`. Emitted to Rust it is the ordinary seed function.
//!
//! Every failure arm refuses with a located message and a non-zero exit. There is no arm that
//! writes a partial or empty table: a receipt that could not be produced and a receipt with no
//! rows must not share a spelling.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::SourceFile;
use v1_compiler::v1_tests_claim_carrier_realization_census::typed_census_from_sources;

fn refuse(message: &str) -> ! {
    eprintln!("carrier-realization-census: REFUSED: {message}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        refuse("usage: carrier_realization_census <receipt.tsv> <subject.dag>...");
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

    let receipt = typed_census_from_sources(Rc::new(sources.into()));
    if receipt.starts_with("REFUSED") {
        refuse(&format!("census refused: {receipt}"));
    }

    // The header alone is a table with no rows. That is a real answer only if the subject genuinely
    // has no type-reference occurrences, which no subject worth censusing does, so it refuses here
    // rather than reporting a zero that an absent read would produce identically.
    let row_count = receipt.lines().count().saturating_sub(1);
    if row_count == 0 {
        refuse("census produced a header and no rows; refusing to write an empty receipt");
    }

    match std::fs::write(&receipt_path, &receipt) {
        Ok(()) => println!(
            "carrier-realization-census: rows={row_count} subjects={} receipt={receipt_path}",
            subject_paths.len()
        ),
        Err(err) => refuse(&format!("could not write receipt {receipt_path}: {err}")),
    }
}
