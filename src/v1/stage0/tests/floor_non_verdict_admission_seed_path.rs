//! THE DISCRIMINATING RED ON THE SEED PATH, not on the model it mirrors.
//!
//! `v2.workflow.floor_non_verdict_admission` carries the modeled admission and its own witnesses
//! establish that the RULE discriminates. Those witnesses say nothing about the Rust that
//! actually stops the line: the seed recomputes the same two sets, and a mirror can be wrong in
//! ways its carrier cannot see. This file exercises the seed's own decision function directly,
//! with synthetic identity sets, so the wall has a red that goes through the code that gates.
//!
//! WHAT THIS DOES NOT ESTABLISH, stated so it is not read as more than it is: it does not prove
//! the wiring from a real thrown witness into `observed`. That path remains unexercised, and it
//! is the honest residual gap — narrower than "the seed wall is untested", and not zero.

use std::collections::HashSet;
use v1_compiler::cli_run::{non_verdict_admission, non_verdict_admits};

fn ids(xs: &[&str]) -> HashSet<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

fn roster() -> HashSet<String> {
    ids(&["m.a", "m.b", "m.c"])
}

#[test]
fn unchanged_population_is_admitted() {
    let a = non_verdict_admission(&ids(&["m.a", "m.b", "m.c"]), &roster());
    assert!(a.added.is_empty(), "added: {:?}", a.added);
    assert!(a.repaid.is_empty(), "repaid: {:?}", a.repaid);
    assert!(non_verdict_admits(&a));
}

/// A REPAID IDENTITY WHOSE ROW STILL STANDS IS A STALE ROW, AND STALE ROWS REFUSE. An earlier
/// revision of this test asserted the opposite, reasoning that refusing repayment would red the
/// merge that repairs the population (gunbc#9020 repays 99 in one landing). That reasoning was
/// right about the goal and wrong about the subject: what refuses is not the repayment, it is the
/// roster row left behind asserting a debt that no longer exists. The executing gate already
/// refuses it — `stale_non_verdict` is a conjunct of `required_floor_outcome_is_clean` — so
/// admitting it here made this mirror disagree with its own consumer (review 55577).
#[test]
fn repayment_that_leaves_the_row_behind_refuses() {
    let a = non_verdict_admission(&ids(&["m.b"]), &roster());
    assert!(a.added.is_empty(), "added: {:?}", a.added);
    assert_eq!(a.repaid, vec!["m.a".to_string(), "m.c".to_string()]);
    assert!(!non_verdict_admits(&a));
}

/// THE CONTROL THAT KEEPS THE ROW ABOVE FROM READING AS A PENALTY ON REPAYMENT. Repaying and
/// deleting the row are one act: delete the two repaid identities from the roster in the same
/// change and both collections are empty, so the same function that refuses the stale form admits
/// this one. Without this pair the suite would not discriminate "stale rows refuse" from
/// "repayment refuses", which are opposite rules.
#[test]
fn repayment_with_the_row_deleted_is_admitted() {
    let a = non_verdict_admission(&ids(&["m.b"]), &ids(&["m.b"]));
    assert!(a.added.is_empty(), "added: {:?}", a.added);
    assert!(a.repaid.is_empty(), "repaid: {:?}", a.repaid);
    assert!(non_verdict_admits(&a));
}

#[test]
fn growth_stops_the_line() {
    let a = non_verdict_admission(&ids(&["m.a", "m.b", "m.c", "m.d"]), &roster());
    assert_eq!(a.added, vec!["m.d".to_string()]);
    assert!(!non_verdict_admits(&a));
}

/// THE CASE THE WHOLE MECHANISM EXISTS FOR, and the one a count-based implementation passes in
/// silence: `m.c` was repaired and `m.d` began producing no verdict in the same change. The
/// population is still three identities, so every count in the run is identical to the unchanged
/// case above — and a repaired witness has bought permission for an unrelated witness to lose its
/// verdict. The length equality is asserted here so the refusal cannot be read as an artifact of
/// the two sets being different sizes.
#[test]
fn a_swap_that_leaves_the_count_unmoved_still_stops_the_line() {
    let observed = ids(&["m.a", "m.b", "m.d"]);
    let roster = roster();
    assert_eq!(
        observed.len(),
        roster.len(),
        "the discriminating property of this case is that the counts agree"
    );
    let a = non_verdict_admission(&observed, &roster);
    assert_eq!(a.added, vec!["m.d".to_string()]);
    assert_eq!(a.repaid, vec!["m.c".to_string()]);
    assert!(!non_verdict_admits(&a));
}

/// AN EMPTY ROSTER IS THE STRICTEST STATE, NOT THE MOST PERMISSIVE. This is the polarity that
/// makes a roster read failure unable to flatter a run: with nothing enrolled, every observed
/// non-verdict identity is growth and the line stops. The opposite polarity would rebuild the
/// absorbing-fallback shape inside the mechanism written to close one.
#[test]
fn an_empty_roster_refuses_every_observed_non_verdict() {
    let a = non_verdict_admission(&ids(&["m.a"]), &HashSet::new());
    assert_eq!(a.added, vec!["m.a".to_string()]);
    assert!(!non_verdict_admits(&a));

    // ...and an empty observation over an empty roster is clean, so the strictness above is a
    // property of what was observed rather than a function stuck at refuse.
    let b = non_verdict_admission(&HashSet::new(), &HashSet::new());
    assert!(non_verdict_admits(&b));
}
