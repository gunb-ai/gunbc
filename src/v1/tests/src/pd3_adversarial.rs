//! A3 ADVERSARIAL PD-3 VERIFIER (cool-swift-143) — independent skeptic suite.
//!
//! Authored against the bounded direct-call arg check (node_type_compatible,
//! direct-call ExprCall arm) landed by sleek-wolf-108 / PR #4524. Goal: try to
//! BREAK the expanded PD-3 gate, not confirm it. Each test states the attack and
//! the must-hold verdict. A red here is a real finding for Mgr-A.
//!
//! Mandated attacks:
//!   (1) false-accept brand-twin UserId-for-AccountId (brand erased at call site)
//!   (2) over-reject List-for-FreeMonoid (structural alias must stay transparent)
//!   (3) over-reject UserId-for-UserId (same brand must pass)
//!   (4) suite regression (covered by running the full crate)

use v1_compiler::v1_std_core::CompilerDiagnostic;

fn has_type_mismatch(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| matches!(&*d.diagnostic, CompilerDiagnostic::TypeMismatch { .. }))
}

// ── (1) FALSE-ACCEPT brand-twin: harder variants than the basic positive ──

// Twin in 2nd arg position — guards the enumerate/skip index pairing.
#[test]
fn adv_brand_twin_in_second_arg_must_reject() {
    let source = r#"
module pd3adv.twin_arg2

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>

fn take_two(n: Int, acct: AccountId) -> String {
  ""
}

fn caller(uid: UserId) -> String {
  take_two(1, uid)
}
"#;
    let result = crate::helpers::compile_dag(source);
    assert!(
        has_type_mismatch(&result),
        "PD-3 ADV: brand twin in 2nd arg slot must be rejected, got: {:?}",
        crate::helpers::diagnostic_messages(&result)
    );
}

// Twin nested inside a container element — brand must survive recursion.
#[test]
fn adv_brand_twin_in_list_element_must_reject() {
    let source = r#"
module pd3adv.twin_in_list

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>

fn take_accts(xs: List<AccountId>) -> Int {
  0
}

fn caller(us: List<UserId>) -> Int {
  take_accts(us)
}
"#;
    let result = crate::helpers::compile_dag(source);
    assert!(
        has_type_mismatch(&result),
        "PD-3 ADV: brand twin nested in List element must be rejected, got: {:?}",
        crate::helpers::diagnostic_messages(&result)
    );
}

// Twin where the value flows through a let-binding before the call.
#[test]
fn adv_brand_twin_via_let_must_reject() {
    let source = r#"
module pd3adv.twin_let

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>

fn take_account(id: AccountId) -> String {
  ""
}

fn caller(uid: UserId) -> String {
  let x = uid
  take_account(x)
}
"#;
    let result = crate::helpers::compile_dag(source);
    assert!(
        has_type_mismatch(&result),
        "PD-3 ADV: brand twin via let-binding must be rejected, got: {:?}",
        crate::helpers::diagnostic_messages(&result)
    );
}

// ── (2) OVER-REJECT structural alias: List = FreeMonoid transparency ──

// Reverse direction of the implementer's accept test.
#[test]
fn adv_alias_freemonoid_for_list_must_accept() {
    let source = r#"
module pd3adv.alias_reverse

fn take_list(xs: List<Int>) -> Int {
  0
}

fn caller(xs: FreeMonoid<Int>) -> Int {
  take_list(xs)
}
"#;
    let result = crate::helpers::compile_dag(source);
    crate::helpers::assert_no_diagnostics(&result);
}

// Nested alias: FreeMonoid<List<Int>> for List<List<Int>>.
#[test]
fn adv_nested_alias_must_accept() {
    let source = r#"
module pd3adv.alias_nested

fn take_nested(xs: List<List<Int>>) -> Int {
  0
}

fn caller(xs: FreeMonoid<List<Int>>) -> Int {
  take_nested(xs)
}
"#;
    let result = crate::helpers::compile_dag(source);
    crate::helpers::assert_no_diagnostics(&result);
}

// ── (3) OVER-REJECT same brand: UserId-for-UserId variants ──

// Same brand in 2nd arg slot.
#[test]
fn adv_same_brand_second_arg_must_accept() {
    let source = r#"
module pd3adv.same_brand_arg2

type Refined<T> {
  base: T
}
type UserId = Refined<String>

fn take_two(n: Int, id: UserId) -> String {
  ""
}

fn caller(uid: UserId) -> String {
  take_two(1, uid)
}
"#;
    let result = crate::helpers::compile_dag(source);
    crate::helpers::assert_no_diagnostics(&result);
}

// Same brand nested in container.
#[test]
fn adv_same_brand_in_list_must_accept() {
    let source = r#"
module pd3adv.same_brand_list

type Refined<T> {
  base: T
}
type UserId = Refined<String>

fn take_users(xs: List<UserId>) -> Int {
  0
}

fn caller(us: List<UserId>) -> Int {
  take_users(us)
}
"#;
    let result = crate::helpers::compile_dag(source);
    crate::helpers::assert_no_diagnostics(&result);
}

// Plain (unbranded) matching types must not be perturbed by the new gate.
#[test]
fn adv_plain_matching_args_must_accept() {
    let source = r#"
module pd3adv.plain_ok

fn take_str(s: String) -> Int {
  0
}

fn caller(s: String) -> Int {
  take_str(s)
}
"#;
    let result = crate::helpers::compile_dag(source);
    crate::helpers::assert_no_diagnostics(&result);
}
