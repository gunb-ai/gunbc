//! Case studies: bugs impossible by construction.
//!
//! Each test pair proves that a specific bug category is structurally
//! impossible in .dag programs. Negative tests show buggy code is
//! rejected with the correct diagnostic. Positive tests show correct
//! code compiles cleanly.
//!
//! See docs/bugs-impossible-by-construction.md for the full writeup.

use crate::helpers::*;
use v2_compiler::v2_compiler_artifact::RenderTarget;
use v2_compiler::v2_std_core::CompilerDiagnostic;

// ── CS-1: Impossible Typos (Generated Code) ────────────────────────────
//
// Field names exist once in the .dag declaration. The emitter generates
// struct fields, JSON keys, API parameters. No human types `user.naem`.

#[test]
fn cs1_generated_field_names_match_declaration() {
    // Declare a type and verify that emitted Rust uses exactly those field names.
    // The human never writes "name" or "email" in generated code — the emitter does.
    let source = "module cs1_typo\n\ntype User { name: String  email: String }\n\nfn greet(u: User) -> String {\n  u.name\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let content = find_file(&result, "src/cs1_typo.rs");
    assert!(content.contains("pub name: String"), "emitted struct must contain field 'name'");
    assert!(content.contains("pub email: String"), "emitted struct must contain field 'email'");
    // The emitter cannot introduce a typo — it reads the field name from the AST.
    assert!(!content.contains("naem"), "typo 'naem' must not appear in generated code");
    assert!(!content.contains("emal"), "typo 'emal' must not appear in generated code");
}

#[test]
fn cs1_misspelled_field_access_rejected() {
    // A developer writes `u.naem` instead of `u.name`. Traditional dynamic
    // languages accept this silently. gunbc rejects it at compile time.
    let source = "module cs1_typo_err\n\ntype User { name: String  email: String }\n\nfn greet(u: User) -> String {\n  u.naem\n}\n";
    let result = compile_dag(source);
    // Field access errors are InternalError (inference_error in 04_infer.dag)
    assert!(
        result.diagnostics.iter().any(|d|
            matches!(&*d.diagnostic, CompilerDiagnostic::InternalError { message, .. }
                if message.contains("naem"))
        ),
        "accessing non-existent field 'naem' should produce InternalError mentioning 'naem', got: {:?}",
        diagnostic_messages(&result)
    );
}

// ── CS-2: Exhaustive Matches ────────────────────────────────────────────
//
// Adding a variant to an enum forces every match to update. No silent
// fall-through, no default case hiding a missing arm.
//
// Existing tests:
//   match_on_coproduct_missing_variant_produces_diagnostic (pipeline.rs)
//   match_on_coproduct_all_variants_no_diagnostic (pipeline.rs)
//   optional_match_missing_none_arm_produces_diagnostic (pipeline.rs)

#[test]
fn cs2_added_variant_breaks_existing_match() {
    // Module A defines an enum with 3 variants. Module B matches on 2 of them.
    // In a traditional language, this compiles — and crashes at runtime when the
    // 3rd variant appears. gunbc catches it at compile time.
    let types = "module cs2_types\n\ntype Status = Active | Inactive | Suspended\n";
    let consumer = "module cs2_consumer\nimport cs2_types { Status }\n\nfn describe(s: Status) -> String {\n  match s {\n    Active => \"on\"\n    Inactive => \"off\"\n  }\n}\n";
    let result = compile_multi(&[("cs2_types.dag", types), ("cs2_consumer.dag", consumer)]);
    assert!(
        result.diagnostics.iter().any(|d|
            matches!(&*d.diagnostic, CompilerDiagnostic::NonExhaustiveMatch { missing, .. }
                if missing.iter().any(|v| v == "Suspended"))
        ),
        "missing Suspended arm should produce NonExhaustiveMatch, got: {:?}",
        diagnostic_messages(&result)
    );
}

// ── CS-3: Termination Proofs ────────────────────────────────────────────
//
// A one-character typo — `items` vs `tail` — creates an infinite loop.
// Traditional compilers accept it. gunbc's descent proof catches it.
//
// Existing tests:
//   soundness_same_argument_stays_violation (pipeline.rs:2277)
//   cx_forever_bound_produces_violation (pipeline.rs:2219)

#[test]
fn cs3_recursive_typo_rejected() {
    // The developer meant to recurse on `tail` but wrote `items` — an infinite loop.
    // Every traditional compiler accepts this. gunbc proves non-descent → violation.
    let source = r#"module cs3_typo
type IntList = Nil | Cons { head: Int, tail: IntList }

fn sum_list(items: IntList) -> Int {
  match items {
    Nil => 0
    Cons { head: h, tail: t } => h + sum_list(items: items)
  }
}
"#;
    let complexity = compile_dag_with_complexity(source);
    let class = complexity.function_classes.get("sum_list")
        .expect("sum_list should have a complexity class");
    assert_eq!(class.as_str(), "O(?)",
        "same-argument recursion (items instead of tail) should be O(?), got {}", class);
    assert!(!complexity.violations.is_empty(),
        "same-argument recursion should produce a violation");
}

#[test]
fn cs3_correct_descent_accepted() {
    // Same function, but the recursive call correctly passes `t` (the tail).
    // The descent proof sees StrictSubValue → function terminates.
    let source = r#"module cs3_correct
type IntList = Nil | Cons { head: Int, tail: IntList }

fn sum_list(items: IntList) -> Int {
  match items {
    Nil => 0
    Cons { head: h, tail: t } => h + sum_list(items: t)
  }
}
"#;
    let complexity = compile_dag_with_complexity(source);
    let class = complexity.function_classes.get("sum_list");
    assert!(class.is_some(), "sum_list should have a complexity class");
    assert_ne!(class.unwrap().as_str(), "O(?)",
        "correct structural descent should not be O(?)");
}

// ── CS-5: Branch Type Unification ──────────────────────────────────────
//
// `if cond { 1 } else { "x" }` — both branches must unify to the same type.
// Dynamic languages accept this silently. Static languages vary.
//
// Existing test:
//   if_else_branch_type_mismatch (pipeline.rs:736)

#[test]
fn cs5_branches_unified_accepted() {
    // Both branches return Int — types unify. Clean compile.
    let source = "module cs5_ok\n\nfn pick(flag: Bool) -> Int {\n  if flag { 1 } else { 2 }\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── CS-6: Map Key Type Mismatch ────────────────────────────────────────
//
// Indexing Map<String, V> with an Int is caught. No silent wrong-key lookup.
// In JS, `obj[42]` silently coerces to `obj["42"]`. In Python, KeyError at runtime.
//
// Existing unit tests:
//   map_index_with_wrong_key_type_reports_error (infer_semantics.rs)
//   map_index_with_correct_key_type_succeeds (infer_semantics.rs)

#[test]
fn cs6_map_wrong_key_type_rejected() {
    // Full pipeline: Map<String, Int> indexed with Int key → diagnostic.
    let source = "module cs6_err\nimport std.types { Map }\n\nfn lookup(m: Map<String, Int>, id: Int) -> Int? {\n  m[id]\n}\n";
    let result = compile_dag(source);
    // Key type mismatches are InternalError (inference_error in 04_infer.dag)
    assert!(
        result.diagnostics.iter().any(|d|
            matches!(&*d.diagnostic, CompilerDiagnostic::InternalError { message, .. }
                if message.contains("key type"))
        ),
        "indexing Map<String, Int> with Int key should produce InternalError about key type, got: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn cs6_map_correct_key_type_accepted() {
    // Full pipeline: Map<String, Int> indexed with String key → clean compile.
    let source = "module cs6_ok\nimport std.types { Map }\n\nfn lookup(m: Map<String, Int>, key: String) -> Int? {\n  m[key]\n}\n";
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

// ── CS-8: Ownership / Double-Use ───────────────────────────────────────
//
// Using a binding in two consuming positions is caught. In most languages,
// this creates shared mutable state bugs. gunbc's ownership analysis
// detects multi-consumer bindings.
//
// Existing tests:
//   compile_sources_returns_ownership_proofs (pipeline.rs:1313)
//   fold_struct_accumulator_linear_ownership (pipeline.rs:4176)
//   fold_struct_accumulator_rejects_multi_move (pipeline.rs:4204)

#[test]
fn cs8_double_consumer_detected() {
    // A non-Copy binding (Map) is consumed by two different call sites.
    // In Python/JS, both get the same reference — mutations from one corrupt the other.
    // gunbc's ownership analysis marks the fold accumulator as ineligible
    // for unwrap optimization (fold_acc_unwrap.eligible = false).
    let source = r#"module cs8_double
type Accum { data: Map<String, Bool> }
fn process(items: List<String>) -> Accum {
  items |> fold(init: Accum { data: empty_map() }, f: (acc, item) =>
    let a = map_insert(acc.data, item, true)
    let b = map_insert(acc.data, item, false)
    Accum { data: b }
  )
}
"#;
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let proof = result
        .ownership
        .iter()
        .find(|p| p.func_name == "process")
        .expect("ownership proof for 'process' missing");
    // Multi-move of acc.data → fold must be ineligible for unwrap optimization
    assert!(
        !proof.fold_acc_unwrap.iter().any(|p| p.eligible),
        "double-consumer of acc.data should be detected as ineligible"
    );
}

#[test]
fn cs8_single_consumer_accepted() {
    // Each field of the accumulator is consumed exactly once per fold step.
    // Ownership analysis proves sole-owner → eligible for unwrap optimization.
    let source = r#"module cs8_single
type Accum { table: Map<String, Int>, label: String }
fn summarize(items: List<String>) -> Accum {
  items |> fold(init: Accum { table: empty_map(), label: "" }, f: (acc, item) =>
    Accum { table: map_insert(acc.table, item, 1), label: item }
  )
}
"#;
    let result = compile_dag(source);
    assert_no_diagnostics(&result);
    let proof = result
        .ownership
        .iter()
        .find(|p| p.func_name == "summarize")
        .expect("ownership proof for 'summarize' missing");
    assert!(
        proof.fold_acc_unwrap.iter().any(|p| p.eligible),
        "single-consumer of each field should be eligible for unwrap optimization"
    );
}

// =========================================================================
// Integration case studies: long-distance dependency bugs
//
// These tests demonstrate bugs that emerge from the INTERACTION between
// distant parts of a codebase — the kind that survive code review, pass
// CI with stale fixtures, and break in production weeks later.
//
// In traditional codebases, Team A owns the data model, Team B owns
// billing, Team C owns shipping. When Team A changes a type, Teams B
// and C find out at runtime. In gunbc, the compiler catches every
// downstream inconsistency at compile time — across all modules and
// all target languages simultaneously.
// =========================================================================

// ── CS-9: Schema Evolution — Field Rename Across Modules ───────────────
//
// The "Tuesday deployment": someone renames `total` to `amount` in the
// shared Order type. In Python, `order.total` returns AttributeError in
// production. In JavaScript, it returns `undefined` and the bug propagates.
// In gunbc, every module that accesses `order.total` gets a compile error.

#[test]
fn cs9_field_rename_breaks_downstream_consumer() {
    // Module A: shared data model (the "after rename" version)
    let types = r#"module cs9_types
type Order { customer: String  amount: Float  status: String }
"#;
    // Module B: billing code still uses the old field name `total`
    let billing = r#"module cs9_billing
import cs9_types { Order }

fn invoice_total(order: Order) -> Float {
  order.total
}
"#;
    let result = compile_multi(&[("cs9_types.dag", types), ("cs9_billing.dag", billing)]);
    // Field access errors are InternalError (inference_error in 04_infer.dag)
    assert!(
        result.diagnostics.iter().any(|d|
            matches!(&*d.diagnostic, CompilerDiagnostic::InternalError { message, .. }
                if message.contains("total"))
        ),
        "accessing renamed field 'total' (now 'amount') should produce InternalError mentioning 'total', got: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn cs9_field_rename_consistent_compiles() {
    // Same scenario but billing uses the new field name — clean compile.
    let types = r#"module cs9_types_ok
type Order { customer: String  amount: Float  status: String }
"#;
    let billing = r#"module cs9_billing_ok
import cs9_types_ok { Order }

fn invoice_total(order: Order) -> Float {
  order.amount
}
"#;
    let result = compile_multi(&[("cs9_types_ok.dag", types), ("cs9_billing_ok.dag", billing)]);
    assert_no_diagnostics(&result);
}

// ── CS-10: Variant Addition — Distant Match Sites Break ────────────────
//
// The "three-team problem": Team A owns PaymentStatus, Teams B and C
// match on it in separate modules. Team A adds `Refunded`. In Go,
// the switch statements silently skip it. In Python, the elif chain
// falls through to a wrong default. In gunbc, both B and C get
// NonExhaustiveMatch at compile time.

#[test]
fn cs10_variant_addition_breaks_multiple_consumers() {
    // Team A: the data model — 4 variants including new `Refunded`
    let types = r#"module cs10_types
type PaymentStatus = Pending | Approved | Declined | Refunded
"#;
    // Team B: billing — only handles 3 of 4
    let billing = r#"module cs10_billing
import cs10_types { PaymentStatus }

fn can_charge(s: PaymentStatus) -> Bool {
  match s {
    Pending  => false
    Approved => true
    Declined => false
  }
}
"#;
    // Team C: reporting — also only handles 3 of 4
    let reporting = r#"module cs10_reporting
import cs10_types { PaymentStatus }

fn status_label(s: PaymentStatus) -> String {
  match s {
    Pending  => "waiting"
    Approved => "complete"
    Declined => "failed"
  }
}
"#;
    let result = compile_multi(&[
        ("cs10_types.dag", types),
        ("cs10_billing.dag", billing),
        ("cs10_reporting.dag", reporting),
    ]);
    // Both billing and reporting should produce NonExhaustiveMatch for Refunded
    let exhaustiveness_diags: Vec<_> = result.diagnostics.iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::NonExhaustiveMatch { missing, .. }
            if missing.iter().any(|v| v == "Refunded")))
        .collect();
    assert!(
        exhaustiveness_diags.len() >= 2,
        "both billing and reporting should produce NonExhaustiveMatch for Refunded, got {}: {:?}",
        exhaustiveness_diags.len(),
        diagnostic_messages(&result)
    );
}

// ── CS-11: Record Literal Completeness — New Required Field ────────────
//
// Someone adds `priority: Int` to the shared Config type. Every module
// that constructs a Config literal is now missing a required field.
// In Python, the missing field is just absent from the dict — no error
// until someone reads it. In gunbc, every constructor site fails.

#[test]
#[ignore = "record literal completeness check not yet implemented — honest gap"]
fn cs11_new_required_field_breaks_constructor() {
    // Module A: Config with a new required field
    let types = r#"module cs11_types
type Config { retries: Int  timeout: Int  priority: Int }
"#;
    // Module B: still constructs Config with only 2 fields
    let consumer = r#"module cs11_consumer
import cs11_types { Config }

fn defaults() -> Config {
  Config { retries: 3, timeout: 30 }
}
"#;
    let result = compile_multi(&[("cs11_types.dag", types), ("cs11_consumer.dag", consumer)]);
    let msgs = diagnostic_messages(&result);
    assert!(
        !msgs.is_empty(),
        "constructing Config without 'priority' field should produce a diagnostic, got: {:?}",
        msgs
    );
}

#[test]
fn cs11_complete_constructor_compiles() {
    // Same scenario but consumer includes the new field — clean compile.
    let types = r#"module cs11_types_ok
type Config { retries: Int  timeout: Int  priority: Int }
"#;
    let consumer = r#"module cs11_consumer_ok
import cs11_types_ok { Config }

fn defaults() -> Config {
  Config { retries: 3, timeout: 30, priority: 1 }
}
"#;
    let result = compile_multi(&[("cs11_types_ok.dag", types), ("cs11_consumer_ok.dag", consumer)]);
    assert_no_diagnostics(&result);
}

// ── CS-12: Single Declaration, All Targets ──────────────────────────────
//
// The "polyglot drift" problem: a type changes in one place, but the
// Go client gets updated while the Python client doesn't. In traditional
// polyglot systems, each language has hand-written types that drift apart.
// In gunbc, one .dag declaration is the single source of truth for all
// targets. Each target is a separate `compile_sources(sources, target)`
// call, but the field identities all derive from the same .dag AST —
// per-target naming is spec-driven (snake_case for Rust/Python,
// PascalCase for Go via go_export_ident), not hand-written.

#[test]
fn cs12_type_emits_consistently_across_all_targets() {
    let source = r#"module cs12_polyglot

type Invoice {
  invoice_id: String
  line_items: List<String>
  total_cents: Int
}

fn describe(inv: Invoice) -> String {
  inv.invoice_id
}
"#;
    // Compile to all three targets — same .dag source
    let rust_result = compile_dag_target(source, RenderTarget::Rust);
    let python_result = compile_dag_target(source, RenderTarget::Python);
    let go_result = compile_dag_target(source, RenderTarget::Go);

    assert_no_diagnostics(&rust_result);
    assert_no_diagnostics(&python_result);
    assert_no_diagnostics(&go_result);

    // Rust: snake_case field names match the .dag declaration
    let rust_code = find_file(&rust_result, "src/cs12_polyglot.rs");
    assert!(rust_code.contains("invoice_id"), "Rust must contain invoice_id");
    assert!(rust_code.contains("total_cents"), "Rust must contain total_cents");

    // Python: snake_case field names match the .dag declaration
    let py_file = python_result.files.iter()
        .find(|f| f.path.ends_with(".py") && !f.path.contains("__init__"))
        .expect("Python should emit a .py file");
    assert!(py_file.content.contains("invoice_id"), "Python must contain invoice_id");
    assert!(py_file.content.contains("total_cents"), "Python must contain total_cents");

    // Go: spec-driven PascalCase export (go_export_ident) from the same .dag identifiers
    let go_file = go_result.files.iter()
        .find(|f| f.path.ends_with(".go") && !f.path.contains("go.mod"))
        .expect("Go should emit a .go file");
    assert!(go_file.content.contains("InvoiceId"), "Go must contain InvoiceId (PascalCase from invoice_id)");
    assert!(go_file.content.contains("TotalCents"), "Go must contain TotalCents (PascalCase from total_cents)");
}

// ── CS-13: Diamond Dependency — Type Identity Preserved ────────────────
//
// The "diamond import" problem: Module C imports from both A and B,
// which both import from Shared. Is `a.shared` the same type as
// `b.shared`? In many systems (especially microservices with
// duplicated protobuf), they silently diverge. In gunbc, the module
// graph deduplicates — one declaration, one identity.

#[test]
fn cs13_diamond_dependency_preserves_type_identity() {
    let shared = "module cs13_shared\ntype UserId { value: String }\n";
    let mod_a = r#"module cs13_a
import cs13_shared { UserId }
type AccountRef { owner: UserId }
"#;
    let mod_b = r#"module cs13_b
import cs13_shared { UserId }
type OrderRef { buyer: UserId }
"#;
    // Main imports from both — field access through both paths must work
    let main = r#"module cs13_main
import cs13_a { AccountRef }
import cs13_b { OrderRef }

fn same_user(a: AccountRef, o: OrderRef) -> Bool {
  a.owner.value == o.buyer.value
}
"#;
    let result = compile_multi(&[
        ("cs13_shared.dag", shared),
        ("cs13_a.dag", mod_a),
        ("cs13_b.dag", mod_b),
        ("cs13_main.dag", main),
    ]);
    assert_no_diagnostics(&result);
}
