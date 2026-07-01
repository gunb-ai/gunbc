use v1_compiler::std_induction::{ceil_log, int_pow_bounded};

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
    assert_eq!(int_pow_bounded(2, 257), None);
}

#[test]
fn int_pow_bounded_degenerate_bases_do_not_deep_recurse_at_cap_exponent() {
    assert_eq!(int_pow_bounded(1, 256), Some(1));
    assert_eq!(int_pow_bounded(0, 256), Some(0));
    assert_eq!(int_pow_bounded(-1, 256), Some(1));
    assert_eq!(int_pow_bounded(-1, 255), Some(-1));
}

#[test]
fn ceil_log_overflow_or_invalid_is_none() {
    assert_eq!(ceil_log(1, 10), None);
    assert_eq!(ceil_log(2, 0), None);
    assert_eq!(ceil_log(2, i64::MAX), None);
}
