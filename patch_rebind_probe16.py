#!/usr/bin/env python3
"""probe v16: name the PRODUCER of each Map(K,V) binding, per row -- no repair, no build.

stern-bat-401's ruling (msg_840e0c82): do NOT add a head-declared-parameters accessor and do NOT widen the
predicate. The question is not how to admit `A bound as Map(K,V)` but WHICH PRODUCER hands the fold a Map
whose arguments are builtin Map's own formal parameters instead of instantiated types or fresh variables
(an empty-map constructor? a declared type read without instantiation?). That producer is the earliest
candidate boundary under DESIGN.md SS6b, and I report to him before building anything.

So this probe prints, for every fold refusal, the IDENTITY of the node the fold is binding and of each of
its children:
  - span file:start-end, for the bound node AND for each child leaf. A child K,V whose span points at
    <kernel:Map> is a different producer from one pointing at a user .dag line.
  - ident_span, which distinguishes "this name was written in source" from "this name was minted".
  - declaration (Option<DeclarationRef>) and expr_data, printed via {:?} ONLY.
  - occurrence_identity, which is the compiler's own answer to "was this node synthesized?" --
    OccurrenceSynthetic / OccurrenceMinted / OccurrenceProjected / OccurrencePending.

WHY {:?} AND NOT FIELD ACCESS: DeclarationRef's fields are NonEmptyStr and ExprData is a ~10-variant enum
holding LiteralValue, CallSemantics, VarBindingKind and more. Naming ANY of that from the probe would be a
second signature-drift failure (probe v11 died on Connective::Neg, an invented variant, 20 minutes into a
dispatch). Debug is derived on all of them (verified in src/v1/stage0/src/v1_std_core.rs, std_decl_ref.rs,
std_occurrence_identity.rs), so {:?} gives the variant NAME -- which is precisely the producer identity --
without my code depending on any of it. The trade is disclosed rather than hidden: I read these fields as
text, and if the emitter's Debug output changes, my parsing of those two columns changes with it.

Helper style is copied VERBATIM from patch_rebind_probe4.py -- the patcher that produced 267 real rows --
including explicit types on every binding, .clone() on every field read, and match arms ending in
`_ => false`. probe v15 died of E0282 (type annotations needed) from constructs I invented; v4's style has
never done that.
"""
import sys

# Located by its unique semantic marker, then walked back to the enclosing break -- indentation-independent,
# because verbatim-indentation anchors died when the emitter re-flowed the fold arm.
MARKER = "diagnostics: Rc::new(vec![type_mismatch_error("
BREAK = "break Rc::new(UnifyGenericsAcc {"

HELPERS = r'''
// ---- probe v16: producer identity. Node fields only; no call outside this block. ----
// Field set verified against the REAL struct at src/v1/stage0/src/v1_std_core.rs:1568 (Node), so the only
// way any of these reads can be wrong is if the struct changes under me -- which the build will tell me.
fn rebind16_span(s: &Rc<SourceSpan>) -> String {
    let mut o = s.file.clone();
    o.push_str(":");
    o.push_str(&s.start.to_string());
    o.push_str("-");
    o.push_str(&s.end.to_string());
    o
}

fn rebind16_opt_span(s: &Option<Rc<SourceSpan>>) -> String {
    match s.clone() {
        Some(rc) => rebind16_span(&rc),
        None => "-".to_string(),
    }
}

fn rebind16_names(n: &Rc<Node>, out: &mut Vec<String>, hops: i64) {
    if (n.children.clone().len() as i64) == 0 {
        out.push(n.name.clone());
    }
    for c in n.children.clone().iter().cloned() {
        rebind16_names(&c, out, hops);
    }
    if hops > 0 {
        match n.inferred.clone() {
            Some(rc) => match rc.as_ref() {
                InferredNode::Resolved { node } => rebind16_names(&node.clone(), out, hops - 1),
                _ => {}
            },
            None => {}
        }
    }
}

fn rebind16_any_name(n: &Rc<Node>, declared: &Vec<String>, hops: i64) -> bool {
    // same shape as the shipped node_mentions_own_parameter (04_infer.dag): the name test does NOT
    // short-circuit the hop, because a childless node can still carry a Resolved subtree.
    let mut hit = false;
    if (n.children.clone().len() as i64) == 0 && declared.contains(&n.name.clone()) {
        hit = true;
    }
    if !hit {
        for c in n.children.clone().iter().cloned() {
            if rebind16_any_name(&c, declared, hops) {
                hit = true;
                break;
            }
        }
    }
    if !hit && hops > 0 {
        match n.inferred.clone() {
            Some(rc) => match rc.as_ref() {
                InferredNode::Resolved { node } => {
                    if rebind16_any_name(&node.clone(), declared, hops - 1) {
                        hit = true;
                    }
                }
                _ => {}
            },
            None => {}
        }
    }
    hit
}

fn rebind16_join(v: &Vec<String>) -> String {
    let mut s = String::new();
    let mut i: i64 = 0;
    for x in v.iter() {
        if i > 0 {
            s.push_str(",");
        }
        s.push_str(x);
        i += 1;
    }
    s
}

fn rebind16_params(n: &Rc<Node>) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for p in n.params.clone().iter().cloned() {
        if p.name.clone().len() > 0 {
            v.push(p.name.clone());
        }
    }
    v
}

fn rebind16_child_idents(n: &Rc<Node>) -> Vec<String> {
    // THE producer evidence: one entry per direct child, carrying where that child came from.
    let mut v: Vec<String> = Vec::new();
    for c in n.children.clone().iter().cloned() {
        let mut e = c.name.clone();
        e.push_str("@");
        e.push_str(&rebind16_span(&c.span.clone()));
        e.push_str("/idspan=");
        e.push_str(&rebind16_opt_span(&c.ident_span.clone()));
        e.push_str("/occ=");
        e.push_str(&format!("{:?}", c.occurrence_identity.clone()));
        e.push_str("/decl=");
        e.push_str(&format!("{:?}", c.declaration.clone()));
        v.push(e);
    }
    v
}

fn rebind16_print(
    bind_name: String,
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    generic_names: Rc<Vec<String>>,
    module_name: String,
) {
    let gnames: Vec<String> = (*generic_names.clone()).clone();
    let bparams = rebind16_params(&bound);
    let mut bleaves: Vec<String> = Vec::new();
    rebind16_names(&bound, &mut bleaves, 3);
    let mut aleaves: Vec<String> = Vec::new();
    rebind16_names(&actual, &mut aleaves, 3);
    let ghit = rebind16_any_name(&bound, &gnames, 3);
    let own_bound = (bparams.clone().len() as i64 > 0)
        && rebind16_any_name(&bound, &bparams, 3);
    // is the bound node's OWN occurrence synthetic/minted, or parsed from source?
    let bocc = format!("{:?}", bound.occurrence_identity.clone());
    let bdecl = format!("{:?}", bound.declaration.clone());
    let bexpr = format!("{:?}", bound.expr_data.clone());
    // does any CHILD carry an expr_data that is not the empty marker? printed whole; parsed as text.
    let mut kid_expr: Vec<String> = Vec::new();
    for c in bound.children.clone().iter().cloned() {
        kid_expr.push(format!("{:?}", c.expr_data.clone()));
    }
    let mut s = String::new();
    s.push_str("REBIND16|module=");            s.push_str(&module_name);
    s.push_str("|bind=");                      s.push_str(&bind_name);
    s.push_str("|formal=");                    s.push_str(&formal.name.clone());
    s.push_str("|bhead=");                     s.push_str(&bound.name.clone());
    s.push_str("|bspan=");                     s.push_str(&rebind16_span(&bound.span.clone()));
    s.push_str("|bident=");                    s.push_str(&rebind16_opt_span(&bound.ident_span.clone()));
    s.push_str("|bocc=");                      s.push_str(&bocc);
    s.push_str("|bdecl=");                     s.push_str(&bdecl);
    s.push_str("|bexpr=");                     s.push_str(&bexpr);
    s.push_str("|bparams=");                   s.push_str(&rebind16_join(&bparams));
    s.push_str("|bleaves=");                   s.push_str(&rebind16_join(&bleaves));
    s.push_str("|aleaves=");                    s.push_str(&rebind16_join(&aleaves));
    s.push_str("|gnames=");                    s.push_str(&rebind16_join(&gnames));
    s.push_str("|ghit=");                      s.push_str(if ghit { "1" } else { "0" });
    s.push_str("|ownBound=");                  s.push_str(if own_bound { "1" } else { "0" });
    s.push_str("|achildren=");                 s.push_str(&rebind16_join(&rebind16_child_idents(&bound)));
    s.push_str("|aexprs=");                    s.push_str(&rebind16_join(&kid_expr));
    s.push_str("\n");
    // eprint!, not println!: the fold runs inside a compile whose stdout the floor step parses; a probe
    // line on stdout would be read as compiler output.
    eprint!("{}", s);
}
'''

CALL = '''rebind16_print(
                                                        bind_name.clone(),
                                                        formal.clone(),
                                                        bound.clone(),
                                                        actual.clone(),
                                                        generic_names.clone(),
                                                        module_name.clone(),
                                                    );
'''


def _patch_text(src: str):
    n_marks = src.count(MARKER)
    report = {"marker": n_marks, "already": src.count("rebind16_print")}
    if report["already"]:
        return src, report | {"ok": True, "note": "already patched"}
    if n_marks != 1:
        return src, report | {"ok": False,
                              "why": "the diagnostic marker must appear exactly once, found %d" % n_marks}
    if src.count("pub fn unify_generics(") != 1:
        return src, report | {"ok": False, "why": "expected exactly 1 pub fn unify_generics"}
    fn = src.index("pub fn unify_generics(")
    if "generic_names" not in src[fn:fn + 900]:
        return src, report | {"ok": False, "why": "the fold declares no generic_names -- nothing to measure"}
    out = src[:fn] + HELPERS + "\n" + src[fn:]
    m = out.index(MARKER)
    b = out.rfind(BREAK, 0, m)
    if b < 0:
        return src, report | {"ok": False, "why": "no enclosing break before the marker"}
    out = out[:b] + CALL + out[b:]
    # The probe's whole point is that it depends on nothing outside Node's own fields. These are the names
    # that rotted on me before (probe v10 died on node_type_equals_core's signature, v11 on Connective::Neg).
    for banned in ("node_type_equals_core", "node_type_shape", "s1b_", "resolved_type(",
                   "Connective::Neg", "OccurrenceMinted", "OccurrenceProjected", "OccurrencePending",
                   "OccurrenceSynthetic", "NoExprData", "ExprLiteral", "ExprVar", "ExprCall"):
        if banned in HELPERS + CALL:
            return out, report | {"ok": False,
                                  "why": "banned external/variant dependency in injected code: " + banned}
    return out, report | {"ok": True}


def patch(a, b=None):
    """Dual entry: patch(src_text) -> (out, report); patch(in_path[, out_path]) -> report."""
    if isinstance(a, str) and b is not None:
        src = open(a, encoding="utf-8").read()
        out, rep = _patch_text(src)
        if rep.get("ok"):
            open(b, "w", encoding="utf-8").write(out)
        return rep
    return _patch_text(a)


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        # fixture written OUT LONG with odd indentation, so a patcher that accidentally depends on spacing
        # fails HERE. The old selftest derived its fixture from the same constant the patcher matched and
        # passed while the real seed failed -- a selftest sharing its fixture with the thing under test
        # proves nothing.
        seed = ("pub fn unify_generics(formal: Node, generic_names: Vec<String>) -> Acc {\n"
                "  loop { match v {\n"
                "        Some(bound) => {\n"
                "            if x {\n"
                "                break Rc::new(UnifyGenericsAcc { subst: acc, diagnostics: zero });\n"
                "            } else {\n"
                "                break Rc::new(UnifyGenericsAcc { subst: acc, "
                "diagnostics: Rc::new(vec![type_mismatch_error(a, b, c)]);\n"
                "            }\n"
                "        }\n"
                "    }\n  }\n}\n")
        out, rep = _patch_text(seed)
        print("SELFTEST", rep)
        assert rep["ok"], rep
        assert "rebind16_print" in out and "REBIND16|" in out
        k = out.index("rebind16_print(")
        d = out.index("diagnostics: Rc::new(vec![type_mismatch_error(")
        assert k < d, "injection landed after the diagnostic"
        print("selftest OK")
        sys.exit(0)
    if len(sys.argv) not in (2, 3):
        print("usage: patch_rebind_probe16.py FILE [OUT]  (no OUT patches in place)")
        sys.exit(2)
    inp = sys.argv[1]
    outp = sys.argv[2] if len(sys.argv) == 3 else sys.argv[1]
    out, rep = _patch_text(open(inp, encoding="utf-8").read())
    print("patch report:", rep)
    if not rep["ok"]:
        sys.exit(43)
    open(outp, "w", encoding="utf-8").write(out)
