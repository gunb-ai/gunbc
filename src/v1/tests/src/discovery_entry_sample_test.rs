//! The entry-sample predicate's discriminating evidence.
//!
//! The `.dag` witness beside this one (`test/claim/discovery_entry_sample_witness_test.dag`)
//! can only reach the SELECTION VALUE; the predicate that decides what actually runs lives in
//! the seed, so its properties are pinned here. They are the bounds on a declared coverage
//! regression (`v2.workflow.ci_floor_plan` `corpus_discovery_sample_note`), which makes them
//! exactly the claims that must not drift quietly: a sampling predicate is the shape that
//! reads plausible while doing something else, and every failure mode here reports as a
//! successful, greener, faster run.

use v1_compiler::cli_run::DiscoveryEntrySample;

/// Degenerate and identity fractions REFUSE. Without this the two silent-coverage failures
/// are both writable: `0/N` runs nothing while reporting as a run, and `N/N` (or wider) is
/// `SelectionOff` under a second name, forking one "run everything" fact.
#[test]
fn degenerate_and_identity_fractions_refuse() {
    assert!(DiscoveryEntrySample::admit(0, 5).is_err(), "0/5 admitted");
    assert!(DiscoveryEntrySample::admit(1, 0).is_err(), "1/0 admitted");
    assert!(DiscoveryEntrySample::admit(5, 5).is_err(), "5/5 admitted");
    assert!(DiscoveryEntrySample::admit(6, 5).is_err(), "6/5 admitted");
    // The positive control: without it, a predicate that refused EVERYTHING would pass the
    // four assertions above and silently disable sampling.
    assert!(DiscoveryEntrySample::admit(1, 5).is_ok(), "1/5 refused");
}

#[test]
fn parse_refuses_every_malformed_shape_and_accepts_the_argv_form() {
    for bad in ["", "1", "1/", "/5", "a/5", "1/b", "1/5/7"] {
        assert!(
            DiscoveryEntrySample::parse(bad).is_err(),
            "parsed malformed sample spec `{bad}`"
        );
    }
    let ok = DiscoveryEntrySample::parse("1/5").expect("1/5 should parse");
    assert_eq!(ok.keep_numerator, 1);
    assert_eq!(ok.keep_denominator, 5);
}

/// ENTRY GRAIN. Retention keys on the entry alone, so every witness function in one entry
/// shares one fate. This is the property that makes the saving land on resolve (paid per
/// entry) rather than on eval (paid per row) — if retention ever varied by row, the sample
/// would touch every entry, pay every resolve, and buy almost nothing while still dropping
/// four fifths of the checking.
#[test]
fn retention_is_entry_grain_not_row_grain() {
    let sample = DiscoveryEntrySample::admit(1, 5).unwrap();
    let salt = "0123456789abcdef0123456789abcdef01234567";
    for entry in [
        "dag/test/claim/a_witness_test.dag",
        "dag/test/claim/b_witness_test.dag",
        "src/v2/test/claim/c_test.dag",
    ] {
        // The predicate takes no function argument at all, so row-grain drift is a signature
        // change rather than a silent behavioural one; this pins the consequence that every
        // call for one entry agrees.
        let first = sample.retains(entry, salt);
        for _ in 0..8 {
            assert_eq!(sample.retains(entry, salt), first, "unstable for {entry}");
        }
    }
}

/// DETERMINISM for a fixed (entry, salt): a red found under a sample must be reproducible
/// from the commit recorded in the receipt. A predicate reaching for run-varying state would
/// make the sampled population unrecoverable after the fact.
#[test]
fn retention_is_deterministic_for_one_entry_and_salt() {
    let sample = DiscoveryEntrySample::admit(1, 5).unwrap();
    let entry = "dag/test/claim/doc_reachability_witness_test.dag";
    let salt = "6a3779d442aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let expected = sample.retains(entry, salt);
    for _ in 0..32 {
        assert_eq!(sample.retains(entry, salt), expected);
    }
}

/// ROTATION. The retained subset must MOVE with the salt, because that is the entire reason
/// the unretained entries are "checked later" rather than "never checked again". A predicate
/// that ignored the salt would satisfy determinism and entry-grain above while permanently
/// freezing the same four fifths of the corpus out of CI — the failure this test exists for,
/// and one that no wall-clock or green/red signal would ever reveal.
#[test]
fn retention_rotates_with_the_salt() {
    let sample = DiscoveryEntrySample::admit(1, 5).unwrap();
    let entries: Vec<String> = (0..400)
        .map(|i| format!("dag/test/claim/entry_{i:03}_witness_test.dag"))
        .collect();
    let retained_for = |salt: &str| -> Vec<&String> {
        entries.iter().filter(|e| sample.retains(e, salt)).collect()
    };
    let a = retained_for("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let b = retained_for("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    assert_ne!(a, b, "retained set did not move with the salt");
    // ...and the movement must be a genuine reshuffle, not one entry differing.
    let shared = a.iter().filter(|e| b.contains(e)).count();
    assert!(
        shared < a.len(),
        "every entry retained under salt a was also retained under salt b"
    );
}

/// The retained fraction must be in the neighbourhood of what was declared. A predicate that
/// retained ~everything would leave CI as slow as before while the receipt claimed a sample;
/// one that retained ~nothing would report a green run over almost no coverage. The band is
/// deliberately loose — this is a hash, not a partition, so exactness would be a false claim.
#[test]
fn retained_fraction_is_near_the_declared_fraction() {
    let sample = DiscoveryEntrySample::admit(1, 5).unwrap();
    let salt = "6a3779d442aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let n = 4000;
    let retained = (0..n)
        .filter(|i| {
            sample.retains(
                &format!("dag/test/claim/entry_{i:04}_witness_test.dag"),
                salt,
            )
        })
        .count();
    let ratio = retained as f64 / n as f64;
    assert!(
        (0.15..=0.25).contains(&ratio),
        "retained {retained}/{n} = {ratio:.3}, expected near 0.20"
    );
}
