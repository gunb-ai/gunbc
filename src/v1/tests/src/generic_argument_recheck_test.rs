//! Structural controls for sparse generic-argument recheck
//! (`v1.compiler.infer` `argument_needs_generic_recheck`).
//!
//! Nested left-bound compile is `test.claim.generic_argument_recheck_witness`
//! `a_nested_left_bound_bind_chain_is_accepted` (debug `compile_to_resolved` of
//! that chain does not finish under the remote 45m envelope). The right-bound
//! missing-set case is the discriminating red if recheck demand is omitted.

use std::rc::Rc;

use v1_compiler::v1_compiler_infer::argument_needs_generic_recheck;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core;

fn empty_subst() -> Rc<im::HashMap<String, Rc<v1_std_core::Node>>> {
    v1_rt::rc_empty_map()
}

fn subst_t() -> Rc<im::HashMap<String, Rc<v1_std_core::Node>>> {
    v1_rt::rc_map_insert(empty_subst(), "T".to_string(), v1_std_core::error_type())
}

#[test]
fn left_bound_empty_missing_set_does_not_demand_recheck() {
    assert!(!argument_needs_generic_recheck(
        Rc::new(im::vector![]),
        subst_t()
    ));
}

#[test]
fn right_bound_missing_generic_demands_recheck_once_bound() {
    assert!(argument_needs_generic_recheck(
        Rc::new(im::vector!["T".to_string()]),
        subst_t()
    ));
}

#[test]
fn unbound_missing_generic_does_not_demand_recheck() {
    assert!(!argument_needs_generic_recheck(
        Rc::new(im::vector!["T".to_string()]),
        empty_subst()
    ));
}
