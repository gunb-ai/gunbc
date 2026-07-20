//! Discriminating witness for deref-side boxing consolidation (#6243 / §7b item 4).
//!
//! `field_access_field_is_boxed` used to OR in `is_recursive_type_by_name`, which fired for
//! every recursive leaf — including types already rendered as `Arc<T>` via `shared_types`.
//! That made field access emit `(*x.field).clone()` on an `Rc` field, which is ill-typed
//! (`Option<Arc<…>> cannot be dereferenced` class). `needs_box_wrapping` already returns
//! `false` when `shared_types` dominates; the redundant disjunct was the bug.
//!
//! `Link` is recursive (so it lands in `recursive_type_set`) and Rc-shared under HostNative.
//! Accessing `l.next` must NOT deref — only `.clone()`. Re-adding the old disjunct would
//! reintroduce `(*` in the emitted body.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module linkderef.fixture\n\n",
    "type Link {\n",
    "  next: Link\n",
    "  n: Int\n",
    "}\n\n",
    "fn follow(l: Link) -> Link {\n",
    "  l.next\n",
    "}\n"
);

fn emit_host() -> String {
    compile_dag_named(
        "src/v1/shared_recursive_field_access_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    )
    .files
    .iter()
    .map(|f| f.content.clone())
    .collect::<Vec<_>>()
    .join("\n")
}

fn fn_body(emitted: &str, name: &str) -> String {
    let needle = format!("fn {name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fn `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest[needle.len()..]
        .find("\npub fn ")
        .map(|i| i + needle.len())
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn shared_recursive_rc_field_access_not_dereferenced() {
    let body = fn_body(&emit_host(), "follow");
    assert!(
        !body.contains("(*"),
        "Rc-shared recursive field access must not deref (shared_types dominate needs_box_wrapping), got:\n{body}"
    );
    assert!(
        body.contains(".next"),
        "expected plain field access on shared recursive type, got:\n{body}"
    );
}
