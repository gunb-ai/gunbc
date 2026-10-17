"""Patch the stage0 seed's v1_compiler_infer.rs so the fold's own rebind refusal prints ONE structural
line per refusal. Used for two questions at once, and every field below answers one of them:

  Q1 (stern-bat-401 msg_6b7b53a9): route Bucket B's 20 `Coproduct(FreeMonoid)` vs `Container(FreeMonoid,X)`
     rows. node_type_shape (04_types.dag:918-971) prints the Disj arm as `Coproduct(name)` and HIDES its
     children, so a bare generic (instantiation -> waits on S1, msg_42284010) and a concrete child equal to
     X (printing difference -> the container normalization at the type authority, msg_d7ee8f8a item (2))
     are INDISTINGUISHABLE in the census. `connective` + per-child name/ident_span/inferred is the
     discriminator; nothing else in the emitted artifact sees through the Disj arm.
  Q2 (X1, msg_e5215d82 / msg_9c641978): why do 11 rows with IDENTICAL printings still refuse? Reading
     says the struct arm of node_type_equals_core descends into RAW children while its four sibling arms
     and the printer unwrap with child_type_node -- so field-bearing nodes are compared as nodes, not as
     types. The probe decides that: if children's NAMES are field names that differ while their printed
     TYPES agree, the raw descent is confirmed. Per-child node_type_shape is printed for exactly that
     comparison, alongside declaration (Absent vs Present vs both) which is X1's other hypothesis.

WHY THE PROBE PRINTS `bound` AND `actual` AND NOT `formal`. Read off the emitted refusal (v1_compiler_infer.rs
line 25304): the diagnostic is built from node_type_shape(bound) for the "bound as" side and
node_type_shape(actual) for the "got" side. So the census column I have been calling 'formal' is the
STORED BINDING, not the callee's formal parameter -- which matters for (c) and for reading the classes, and
is why the probe prints all three (formal too) rather than trusting the census's two.

INSTRUMENT RULES ENFORCED HERE (each learned from a probe that died or lied):
 - the patch text is verified against the EMITTED file, which I read locally first; anchors are matched by
   count and the patch aborts unless each matches EXACTLY ONCE;
 - rustfmt must parse the result -- and it is run LOCALLY before any dispatch (four probes died on bad
   injections), so a malformed patch costs nothing;
 - no nested fn captures locals (two top-level fns, all data passed as arguments);
 - one eprintln per record, built with push_str;
 - the refusal itself is UNTOUCHED: the probe call is inserted before `break Rc::new(UnifyGenericsAcc {`
   inside the same arm, so diagnostics and their spans stay byte-identical and the census stays comparable
   to config A's 208 rows;
 - `module_name` is in scope at the refusal (it is passed to type_mismatch_error there), so the probe gets
   it as an argument instead of reaching for a global.
"""
import sys

ANCHOR = '''} else {
                                                    break Rc::new(UnifyGenericsAcc {
    subst: acc.clone(),
    diagnostics: Rc::new(vec![type_mismatch_error('''

HELPER = r'''
fn rebind_probe_print(
    bind_name: String,
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    module_name: String,
) {
    let mut s = String::new();
    s.push_str("REBINDPROBE|bind=");
    s.push_str(&bind_name);
    s.push_str("|module=");
    s.push_str(&module_name);
    s.push_str("|F=[");
    rebind_probe_side(&mut s, bound.clone(), source_indices.clone());
    s.push_str("]|A=[");
    rebind_probe_side(&mut s, actual.clone(), source_indices.clone());
    s.push_str("]|FM=[");
    rebind_probe_side(&mut s, formal.clone(), source_indices.clone());
    s.push_str("]");
    eprintln!("{}", s);
}

fn rebind_probe_side(
    s: &mut String,
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) {
    s.push_str("name=");
    s.push_str(&n.name.clone());
    s.push_str("|identSpan=");
    s.push_str(&match n.ident_span.clone() {
        Some(sp) => format!("{}:{}-{}", sp.file.clone(), sp.start, sp.end),
        None => "none".to_string(),
    });
    s.push_str("|nodeSpan=");
    s.push_str(&format!("{}:{}-{}", n.span.clone().file, n.span.clone().start, n.span.clone().end));
    s.push_str("|connective=");
    s.push_str(&format!("{:?}", n.connective.clone()));
    s.push_str("|cardinality=");
    s.push_str(&format!("{:?}", n.return_cardinality.clone()));
    s.push_str("|nprops=");
    s.push_str(&format!("{}", n.properties.clone().len()));
    s.push_str("|nchildren=");
    s.push_str(&format!("{}", n.children.clone().len()));
    s.push_str("|decl=");
    s.push_str(&match n.declaration.clone() {
        Some(d) => format!("{}::{}", d.module_path.clone(), d.decl_name.clone()),
        None => "none".to_string(),
    });
    s.push_str("|infer=");
    s.push_str(&match n.inferred.clone() {
        Some(InferredNode::Resolved { node }) => format!("Resolved({})", node.name.clone()),
        Some(InferredNode::TypeVariable { id }) => format!("TypeVariable({})", id),
        Some(InferredNode::CompilerError { message, span }) => {
            format!("CompilerError({}@{}:{}-{})", message, span.file.clone(), span.start, span.end)
        }
        None => "none".to_string(),
    });
    s.push_str("|shape=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(
        n.clone(),
        source_indices.clone(),
    ));
    s.push_str("|children=[");
    let mut first = true;
    for c in n.children.clone().iter().cloned() {
        if !first {
            s.push_str(";");
        }
        first = false;
        s.push_str(&c.name.clone());
        s.push_str("/");
        s.push_str(&match c.ident_span.clone() {
            Some(sp) => format!("{}:{}-{}", sp.file.clone(), sp.start, sp.end),
            None => "none".to_string(),
        });
        s.push_str("/");
        s.push_str(&match c.inferred.clone() {
            Some(InferredNode::Resolved { node }) => format!("Resolved({})", node.name.clone()),
            Some(InferredNode::TypeVariable { id }) => format!("TypeVariable({})", id),
            Some(InferredNode::CompilerError { message, span }) => {
                format!("CompilerError({}@{}:{}-{})", message, span.file.clone(), span.start, span.end)
            }
            None => "none".to_string(),
        });
        s.push_str("/");
        s.push_str(&crate::v1_compiler_infer_types::node_type_shape(
            c.clone(),
            source_indices.clone(),
        ));
    }
    s.push_str("]");
}

'''

CALL = '''rebind_probe_print(
                                                        bind_name.clone(),
                                                        formal.clone(),
                                                        bound.clone(),
                                                        actual.clone(),
                                                        source_indices.clone(),
                                                        module_name.clone(),
                                                    );
                                                    '''


def patch(path, out=None):
    src = open(path).read()
    report = {"anchor": src.count(ANCHOR), "already": src.count("rebind_probe_print")}
    if report["already"]:
        return report | {"ok": True, "note": "already patched"}
    if report["anchor"] != 1:
        return report | {"ok": False, "why": "anchor must match exactly once"}
    head, tail = src.split(ANCHOR, 1)
    new = head + ANCHOR.replace("break Rc::new(UnifyGenericsAcc {", CALL + "break Rc::new(UnifyGenericsAcc {") + tail
    # The helpers go at top level, before the first `pub fn`, so nothing captures locals.
    i = new.index("\npub fn ")
    new = new[:i] + "\n" + HELPER + new[i + 1:]
    open(out or path, "w").write(new)
    return report | {"ok": True, "bytes": len(new)}


if __name__ == "__main__":
    r = patch(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
    print(r)
    # Exit NONZERO when the patch did not land. Without this the caller's `|| exit 5` can never fire:
    # the function returned a dict with ok=False and the process still exited 0, so an UNPATCHED seed
    # would have been built and self-hosted, and 0 probe lines would have looked like "no refusals"
    # rather than "the probe never went in". Same class of lie as an empty suite printed as a pass.
    sys.exit(0 if r.get("ok") else 4)
