#!/usr/bin/env python3
"""Minimal fold-site probe: prints, per fold refusal, whether the bound side's childless leaves name the
fold's OWN generic_names. Nothing else.

Why a new patcher instead of fixing patch_rebind_probe8.py: probe v10 installed the instrumentation cleanly
(anchor:1, ok:True) and then the build died with

  error[E0308]: arguments to this function are incorrect
     --> src/v1/stage0/src/v1_compiler_infer.rs:25624:57
      = note: expected reference `&Rc<Node>`, found `&TextJudgment`

which is v8's `node_type_equals_core(bound, actual, source_indices, text)` call no longer matching the head's
emitted signature. That call is not what this measurement is for. The question probe v10 must answer is
narrow -- on the 33-38 rows whose bound side is `Map(Primitive(K), Primitive(V))`, do the bound side's
childless leaves name `generic_names`? -- and it needs only the two nodes plus the fold's `generic_names`.
So this patcher calls NOTHING outside its own helpers: no node_type_equals_core, no node_type_shape, no
s1b_* counter. Every field is computed from `Node` fields that have not moved (name, children, connective,
inferred, params). Fewer dependencies means the probe cannot silently rot against the type authority again,
which is exactly what cost a 20-minute dispatch.

The S1B minting counter and coreEq are NOT dropped information: both are already answered elsewhere
(S1B measured 0 on all 254 rows and stays parked; coreEq=1 on 4 rows was recorded in the v4 census).
"""
import sys

# The anchor is found by its UNIQUE SEMANTIC MARKER, not by indentation. An earlier patcher hard-coded the
# fold arm's leading spaces and died with 'anchor must match exactly once' (anchor:0) on a seed where the
# emitter had re-flowed the arm -- and its synthetic selftest still passed, because the selftest built the
# string from the same hard-coded constant. That is the trap: a selftest that shares its fixture with the
# thing under test proves nothing. Here the marker is the one line in the fold that constructs a diagnostic,
# and the insertion point is found by walking back to the enclosing `break`.
MARKER = "diagnostics: Rc::new(vec![type_mismatch_error("
BREAK = "break Rc::new(UnifyGenericsAcc {"


HELPERS = r'''
// ---- probe v9 helpers: Node fields only, no calls outside this block ----
fn rebind9_names(n: &Rc<Node>, declared: &Vec<String>, hops: i64) -> bool {
    // childless node: its NAME is the candidate. Mirrors the DAG predicate's leaf test.
    if n.children.clone().len() == 0 {
        return declared.iter().any(|x| x == &n.name.clone());
    }
    for c in n.children.clone().iter().cloned() {
        if rebind9_names(&c, declared, hops) {
            return true;
        }
    }
    // the `inferred` Resolved hop is load-bearing (a slot leaf under an r-hop is invisible to raw children),
    // exactly as measured in probe v4 -- patterns copied from patch_rebind_probe4.py, which produced 267 rows.
    if hops > 0 {
        match n.inferred.clone() {
            Some(rc) => match rc.as_ref() {
                InferredNode::Resolved { node } => rebind9_names(&node.clone(), declared, hops - 1),
                _ => false,
            },
            None => false,
        }
    } else {
        false
    }
}

fn rebind9_childless_leaves(n: &Rc<Node>, out: &mut Vec<String>, hops: i64) {
    if n.children.clone().len() == 0 {
        out.push(n.name.clone());
        return;
    }
    for c in n.children.clone().iter().cloned() { rebind9_childless_leaves(&c, out, hops); }
    if hops > 0 {
        match n.inferred.clone() {
            Some(rc) => match rc.as_ref() {
                InferredNode::Resolved { node } => rebind9_childless_leaves(&node.clone(), out, hops - 1),
                _ => {}
            },
            None => {}
        }
    }
}

fn rebind9_join(v: &Vec<String>) -> String {
    let mut s = String::new();
    for (i, x) in v.iter().enumerate() {
        if i > 0 { s.push_str(","); }
        s.push_str(x);
    }
    s
}

fn rebind9_params(n: &Rc<Node>) -> Vec<String> {
    // Node.params is Rc<Vec<Rc<Node>>>: the DECLARATION of each parameter is a node, and its NAME is the
    // parameter's spelling. A blank name is skipped so a blank never matches a blank leaf (v4's lesson).
    let mut v: Vec<String> = Vec::new();
    for p in n.params.clone().iter().cloned() {
        if p.name.clone().len() > 0 {
            v.push(p.name.clone());
        }
        for gp in p.params.clone().iter().cloned() {
            if gp.name.clone().len() > 0 {
                v.push(gp.name.clone());
            }
        }
    }
    v
}

fn rebind9_conn(n: &Rc<Node>) -> String {
    match n.connective.clone() {
        Connective::NoConnective => "N".to_string(),
        Connective::Disj => "D".to_string(),
        Connective::Conj => "C".to_string(),
        Connective::Neg => "L".to_string(),
        _ => "O".to_string(),
    }
}

fn rebind_probe_print9(
    bind_name: String,
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    generic_names: Rc<Vec<String>>,
    module_name: String,
) {
    let gnames: Vec<String> = (*generic_names.clone()).clone();
    let bparams = rebind9_params(&bound);
    let aparams = rebind9_params(&actual);
    let mut bleaves: Vec<String> = vec![];
    let mut aleaves: Vec<String> = vec![];
    rebind9_childless_leaves(&bound, &mut bleaves, 3);
    rebind9_childless_leaves(&actual, &mut aleaves, 3);
    // THE QUESTION THIS PROBE EXISTS TO ANSWER: if the predicate compared the bound side's childless
    // leaves against generic_names instead of gating on the bound node declaring params itself, would it
    // fire on the Map(Primitive(K), Primitive(V)) rows? ghit is exactly that widened test.
    let ghit = rebind9_names(&bound, &gnames, 3);
    let ghitact = rebind9_names(&actual, &gnames, 3);
    // and the arm as shipped, recomputed here so one row carries both the shipped and the widened answer
    let own_bound = (bparams.clone().len() as i64 > 0) && rebind9_names(&bound, &bparams, 3);
    let own_act = (aparams.clone().len() as i64 > 0) && rebind9_names(&actual, &aparams, 3);
    let mut s = String::new();
    s.push_str("REBINDV9|module="); s.push_str(&module_name);
    s.push_str("|bind="); s.push_str(&bind_name);
    s.push_str("|formal="); s.push_str(&formal.name.clone());
    s.push_str("|gnames="); s.push_str(&rebind9_join(&gnames));
    s.push_str("|ghit="); s.push_str(if ghit { "1" } else { "0" });
    s.push_str("|ghitact="); s.push_str(if ghitact { "1" } else { "0" });
    s.push_str("|ownBound="); s.push_str(if own_bound { "1" } else { "0" });
    s.push_str("|ownActual="); s.push_str(if own_act { "1" } else { "0" });
    s.push_str("|bparams="); s.push_str(&rebind9_join(&bparams));
    s.push_str("|aparams="); s.push_str(&rebind9_join(&aparams));
    s.push_str("|bleaves="); s.push_str(&rebind9_join(&bleaves));
    s.push_str("|aleaves="); s.push_str(&rebind9_join(&aleaves));
    s.push_str("|lconn="); s.push_str(&rebind9_conn(&bound));
    s.push_str("|rconn="); s.push_str(&rebind9_conn(&actual));
    s.push_str("\n");
    eprint!("{}", s);
}
'''

CALL = '''rebind_probe_print9(
                                                        bind_name.clone(),
                                                        formal.clone(),
                                                        bound.clone(),
                                                        actual.clone(),
                                                        generic_names.clone(),
                                                        module_name.clone(),
                                                    );
'''


def patch(a, b=None):
    """Dual entry: patch(src_text) -> (out_text, report) for tests, and
    patch(in_path, out_path) -> report for instruments/stubcheck.py and the dispatch scripts.
    Both styles exist across my patchers; matching the harness here is what lets the local rustc gate
    cover this file at all."""
    if isinstance(a, str) and b is not None:
        src = open(a, encoding="utf-8").read()
        out, rep = _patch_text(src)
        if rep.get("ok"):
            open(b, "w", encoding="utf-8").write(out)
        return rep
    return _patch_text(a)


def _patch_text(src: str):
    n_marks = src.count(MARKER)
    report = {"marker": n_marks, "already": src.count("rebind_probe_print9")}
    if report["already"]:
        return src, report | {"ok": True, "note": "already patched"}
    if n_marks != 1:
        return src, report | {"ok": False, "why": "the diagnostic marker must appear exactly once, found %d" % n_marks}
    m = src.index(MARKER)
    b = src.rfind(BREAK, 0, m)
    if b < 0:
        return src, report | {"ok": False, "why": "no enclosing break before the marker"}
    # the fold must still be recognisable, or we are patching a file that is not the fold
    if src.count("pub fn unify_generics(") != 1:
        return src, report | {"ok": False, "why": "expected exactly 1 pub fn unify_generics"}
    fn = src.index("pub fn unify_generics(")
    if not src[fn:fn + 900].count("generic_names"):
        return src, report | {"ok": False, "why": "the fold declares no generic_names -- nothing to measure"}
    out = src[:fn] + HELPERS + "\n" + src[fn:]
    # recompute the insertion offset after the helpers shifted the text
    m = out.index(MARKER)
    b = out.rfind(BREAK, 0, m)
    out = out[:b] + CALL + out[b:]
    for banned in ("node_type_equals_core", "node_type_shape", "s1b_", "resolved_type("):
        if banned in HELPERS + CALL:
            return out, report | {"ok": False, "why": "banned external dependency in injected code: " + banned}
    return out, report | {"ok": True}


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        # The fixture is written OUT LONG by hand, with deliberately odd indentation, so a patcher that
        # accidentally depends on spacing fails here instead of in a 20-minute remote dispatch.
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
        assert "rebind_probe_print9" in out and "REBINDV9|" in out
        # the call must sit immediately BEFORE the arm's break, not inside the diagnostic
        k = out.index("rebind_probe_print9(")
        d = out.index("diagnostics: Rc::new(vec![type_mismatch_error(")
        assert k < d, "injection landed after the diagnostic"
        print("selftest OK")
        sys.exit(0)
    out, rep = _patch_text(open(sys.argv[1], encoding="utf-8").read())
    print("patch report:", rep)
    if not rep["ok"]:
        sys.exit(43)
    open(sys.argv[2], "w", encoding="utf-8").write(out)
