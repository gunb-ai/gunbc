//! first-Option family (Route-A): a function declared `-> String?` must render its
//! Rust signature as `-> Option<String>`, not a bare `-> String`. The emitter
//! previously dropped the Optional wrapper for the String text-carrier: its
//! `is_host_text_carrier_type`/faithful-String early-returns short-circuited the
//! renderer that applies the Optional template, so every `String?`-returning function
//! (and `String?` field/param) emitted a bare `String`. That is the dominant E0308
//! cluster in the faithful Route-A seed (`expected String, found Option<String>`):
//! the body already emits `Some(..)/None`, so only the signature was wrong.
//!
//! The fix wraps the carrier rendering in `Option<..>` when (and only when) the
//! declared return cardinality is `CardOptional` (`rust_carrier_optional_wrap`). It is
//! cardinality-DERIVED construction, not a blanket "always wrap String": the control
//! below pins that a non-optional `String` return stays bare.

use crate::helpers::compile_dag_target;
use v1_compiler::v1_compiler_artifact::RenderTarget;

fn emit(source: &str) -> String {
    compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

// The signature slice for `fn_name`: from `fn <name>` up to the body-opening ` {`.
// Scoped this way so the runtime's own `Option<..>` types do not pollute the assertion.
fn return_sig(emitted: &str, fn_name: &str) -> String {
    let needle = format!("fn {fn_name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fixture fn `{fn_name}` was not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let body_open = rest
        .find(" {")
        .unwrap_or_else(|| panic!("no body opener for `{fn_name}`:\n{emitted}"));
    rest[..body_open].to_string()
}

#[test]
fn optional_string_return_renders_option_signature() {
    let source = "module optsig.fixture\n\nfn maybe_label(flag: Bool) -> String? {\n  if flag { Present { value: \"x\" } } else { none }\n}\n";
    let emitted = emit(source);
    let sig = return_sig(&emitted, "maybe_label");
    assert!(
        sig.contains("Option<"),
        "a `-> String?` return must render an `Option<..>` signature, got:\n{sig}"
    );
}

#[test]
fn non_optional_string_return_stays_bare() {
    // Discriminating control: the wrap is cardinality-DERIVED, not blanket-on-String.
    let source = "module optsig.fixture\n\nfn always_label(flag: Bool) -> String {\n  \"x\"\n}\n";
    let emitted = emit(source);
    let sig = return_sig(&emitted, "always_label");
    assert!(
        !sig.contains("Option<"),
        "a non-optional `-> String` must NOT be wrapped in Option, got:\n{sig}"
    );
}

#[test]
fn optional_shared_type_return_renders_option_rc_signature() {
    // Recursive struct so `Node` is in `shared_types` and fn-sig emission applies Rc.
    let source = "module optsig.fixture\n\ntype Node = Product { child: Node? }\n\nfn maybe_node(flag: Bool) -> Node? {\n  if flag { Present { value: Node { child: none } } } else { none }\n}\n";
    let emitted = emit(source);
    let sig = return_sig(&emitted, "maybe_node");
    assert!(
        sig.contains("Option<") && sig.contains("Arc<"),
        "a `-> Node?` return on a shared type must render `Option<Arc<..>>`, got:\n{sig}"
    );
}

#[test]
fn non_optional_shared_type_return_stays_bare_rc() {
    let source = "module optsig.fixture\n\ntype Node = Product { child: Node? }\n\nfn always_node(_flag: Bool) -> Node {\n  Node { child: none }\n}\n";
    let emitted = emit(source);
    let sig = return_sig(&emitted, "always_node");
    assert!(
        sig.contains("Arc<") && !sig.contains("Option<"),
        "a non-optional `-> Node` must render bare `Arc<..>` without Option, got:\n{sig}"
    );
}
