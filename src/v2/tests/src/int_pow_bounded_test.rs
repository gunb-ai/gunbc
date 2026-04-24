//! `int_pow_bounded` / `ceil_log`: invalid or overflowing `Int` bridges must not
//! wrap or panic (PR #726 — P3 fail-closed vs raw i64 `*` / `+`).

use v2_compiler::std_induction::{ceil_log, int_pow_bounded};

#[test]
fn int_pow_bounded_negative_exp_is_none() {
    assert_eq!(int_pow_bounded(2, -1), None);
    assert_eq!(int_pow_bounded(10, -5), None);
}

#[test]
fn int_pow_bounded_non_negative_matches_pow() {
    assert_eq!(int_pow_bounded(3, 0), Some(1));
    assert_eq!(int_pow_bounded(2, 10), Some(1024));
    assert_eq!(int_pow_bounded(5, 3), Some(125));
}

#[test]
fn int_pow_bounded_overflow_is_none() {
    assert_eq!(int_pow_bounded(2, 63), None);
    assert_eq!(int_pow_bounded(10, 20), None);
}

#[test]
fn int_pow_bounded_exponent_above_peano_materialization_cap_is_none() {
    // P4: `int_pow_bounded` rejects exp > 256 (M9 / `std.termination` literal-bridge ceiling)
    // so huge `work_exponent` cannot force O(exp) stack recursion before fail-closed `none`.
    assert_eq!(int_pow_bounded(2, 257), None);
}

#[test]
fn ceil_log_overflow_or_invalid_is_none() {
    assert_eq!(ceil_log(1, 10), None);
    assert_eq!(ceil_log(2, 0), None);
    // Would require ~2^62 iterations without intermediate overflow; `power * base` hits none first.
    assert_eq!(ceil_log(2, i64::MAX), None);
}
