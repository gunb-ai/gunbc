//! `int_pow_bounded`: negative exponent must not silently yield a plausible `Int`
//! (PR #726 review — align with `ceil_log` / `cost_poly` fail-closed posture).

use v2_compiler::std_induction::int_pow_bounded;

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
