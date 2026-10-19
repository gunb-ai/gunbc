#!/usr/bin/env python3
"""Probe v18 -- the BRANCH DISCRIMINATOR for how a call's result type is produced.

WHY THIS EXISTS. probe v16 named the producer (the node at dag/std/algebra.dag:73644-73647, the
declared `Map<K, V>` of `fn empty_map<K, V>()`) and I then claimed, from a static read, that
borrowed_callable_call_type handed it to the fold. I repaired that site. Two independent
instruments (the fold probe and the two-closure gate) both came back BYTE-IDENTICAL to before:
the repair was inert. The read had skipped the guard that SELECTS among three branches. The
emitted mirror shows the selection (v1_compiler_infer.rs:14770-14797):

    let type_match = type_match_for_direct_call_target(...)            // 14770
    let global_bare_callable = match type_match { Some(_) => None, None => ... }
    let borrowed_call        = match type_match { Some(_) => None, None => if params>0 {
                                 Some(borrowed_callable_call_type(...))        // BRANCH BC
                               } else { Some(BorrowedCallType{ ret: callable_return_type(gnode) }) }  // RT
                             , None => None }
    let resolved_type        = match type_match { Some(tn) => tn.clone(),      // BRANCH TM  <-- 9657
                               None => match borrowed_call { Some(b) => b.ret, None => error_type() } }

So `binding.resolved` is returned at TM and the borrowed path runs only when type_match is None.
This probe does not repair anything: it records, for every fold refusal, WHICH branch wrote
`resolved_type` for that call, so the next repair goes where the data says. It is pure
observation -- it calls nothing outside Node field reads, adds no behaviour, and its only writes
are to two probe-local atomics.

TAG PROTOCOL (stated so a stale tag cannot masquerade as evidence):
  - immediately BEFORE type_match is computed, the tag is reset to "unset" and a write counter
    advances, so a fold reached from a DIFFERENT entry point cannot inherit a previous call's tag
    without showing it: the row prints the tag and that write's sequence number alongside the
    fold's own sequence number.
  - each of the three branches then writes its own tag ("TM", "BC", "RT").
  - rows whose tag is "unset" demonstrably did not enter through this call arm; they are reported
    as NOT-COVERED rather than folded into a branch's count. There are 7 outermost fold entries
    and this arm is one of them -- claiming otherwise would be the same overreach as last time.
  - no parallelism in this path (0 rayon/par_iter in the emitted infer mirror), so an ordering
    counter is meaningful.
"""
import os
import re
import sys

MIRROR = "src/v1/stage0/src/v1_compiler_infer.rs"

# --- the three anchors, taken from the emitted mirror verbatim (line numbers are hints, not relied on)
TM_ANCHOR = "let type_match = type_match_for_direct_call_target(scope.type_env.clone(), func_name.clone(), call_target.clone());"
BC_ANCHOR = "Some(borrowed_callable_call_type(gnode.clone(), typed_arg_nodes.clone(), scope.clone()))"
RT_ANCHOR = "ret: crate::v1_compiler_infer_types::callable_return_type(gnode.clone()),"

# --- the fold-print call site (same anchor the v16 probe uses)
MARKER = "diagnostics: Rc::new(vec![type_mismatch_error("
BREAK = "break Rc::new(UnifyGenericsAcc {"

TAG_DECL = '''static REBIND18_TAG: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static REBIND18_TAGSEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static REBIND18_FOLDSEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
// 0=unset 1=TM(type_match Present => binding.resolved) 2=BC(borrowed_callable_call_type) 3=RT(callable_return_type)
fn rebind18_set(tag: usize) {
    REBIND18_TAG.store(tag, std::sync::atomic::Ordering::Relaxed);
    REBIND18_TAGSEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
fn rebind18_name() -> String {
    match REBIND18_TAG.load(std::sync::atomic::Ordering::Relaxed) {
        1 => "TM".to_string(),
        2 => "BC".to_string(),
        3 => "RT".to_string(),
        _ => "unset".to_string(),
    }
}
'''

RESET = "rebind18_set(0);"
SET_TM = 'rebind18_set(1); '
SET_BC = 'rebind18_set(2); '
SET_RT = 'rebind18_set(3); '

PRINT_FN = '''fn rebind18_print(
    bind_name: String,
    bound: Rc<Node>,
    actual: Rc<Node>,
    generic_names: Rc<Vec<String>>,
    module_name: String,
) {
    let gnames: Vec<String> = (*generic_names.clone()).clone();
    let mut bleaves: Vec<String> = Vec::new();
    rebind18_names(&bound, &mut bleaves, 3);
    let mut aleaves: Vec<String> = Vec::new();
    rebind18_names(&actual, &mut aleaves, 3);
    let mut s = String::new();
    s.push_str("REBIND18|module=");  s.push_str(&module_name);
    s.push_str("|bind=");            s.push_str(&bind_name);
    s.push_str("|bhead=");           s.push_str(&bound.name.clone());
    s.push_str("|bspan=");           s.push_str(&rebind18_span(&bound.span.clone()));
    s.push_str("|bocc=");            s.push_str(&format!("{:?}", bound.occurrence_identity.clone()));
    s.push_str("|bdecl=");           s.push_str(&format!("{:?}", bound.declaration.clone()));
    s.push_str("|bleaves=");         s.push_str(&rebind18_join(&bleaves));
    s.push_str("|aleaves=");         s.push_str(&rebind18_join(&aleaves));
    s.push_str("|gnames=");          s.push_str(&rebind18_join(&gnames));
    // the discriminator fields -- written by the call arm, read here
    s.push_str("|tag=");             s.push_str(&rebind18_name());
    s.push_str("|tagseq=");          s.push_str(&REBIND18_TAGSEQ.load(std::sync::atomic::Ordering::Relaxed).to_string());
    s.push_str("|foldseq=");         s.push_str(&REBIND18_FOLDSEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string());
    s.push_str("\\n");
    // eprint!, not println!: the fold runs inside a compile whose stdout the floor step parses.
    eprint!("{}", s);
}

fn rebind18_names(n: &Rc<Node>, out: &mut Vec<String>, depth: usize) {
    if depth == 0 { return; }
    if n.children.clone().len() as i64 == 0 {
        if n.name.clone().len() as i64 > 0 { out.push(n.name.clone()); }
        return;
    }
    for c in n.children.clone().iter() { rebind18_names(c, out, depth - 1); }
}

fn rebind18_span(sp: &SourceSpan) -> String {
    format!("{}:{}-{}", sp.file.clone(), sp.start.to_string(), sp.end.to_string())
}

fn rebind18_join(v: &Vec<String>) -> String {
    let mut out = String::new();
    for (i, x) in v.iter().enumerate() {
        if i > 0 { out.push_str(","); }
        out.push_str(x);
    }
    out
}
'''

CALL = '''rebind18_print(
                                                        bind_name.clone(),
                                                        bound.clone(),
                                                        actual.clone(),
                                                        generic_names.clone(),
                                                        module_name.clone(),
                                                    );
'''


def _inject_once(src, anchor, insert, label, allow_prefix=True):
    n = src.count(anchor)
    if n != 1:
        raise AssertionError("anchor %r matched %d times (want 1)" % (label, n))
    i = src.index(anchor)
    # insert immediately BEFORE the anchor expression
    return src[:i] + insert + src[i:]


def _patch_text(src: str):
    rep = {}
    if "REBIND18|" in src:
        return src, {"already": True, "ok": True}
    rep["tm"] = src.count(TM_ANCHOR)
    rep["bc"] = src.count(BC_ANCHOR)
    rep["rt"] = src.count(RT_ANCHOR)
    rep["marker"] = src.count(MARKER)
    # STUB MODE, for instruments/stubcheck.py only: it patches a small synthetic seed that has the
    # fold marker but no call arm, and its job is to compile-check the injected FUNCTIONS. It is
    # opted in by env var so a real run can NEVER land a tag-less probe by accident -- without it,
    # missing anchors are a hard refusal, because a probe whose tags are all "unset" is a false green.
    if os.environ.get("PROBE18_STUB") == "1" and rep["tm"] == 0 and rep["bc"] == 0 and rep["rt"] == 0:
        assert rep["marker"] >= 1, "stub seed must contain the fold marker"
        anchor_fn = "fn unify_generics("
        j = src.index(anchor_fn)
        out = src[:j] + TAG_DECL + PRINT_FN + "\n" + src[j:]
        for a, b in (("{", "}"), ("(", ")")):
            assert out.count(a) - out.count(b) == src.count(a) - src.count(b)
        return out, {"ok": True, "stub": True}
    if rep["tm"] != 1 or rep["bc"] != 1 or rep["rt"] != 1 or rep["marker"] < 1:
        return src, rep | {"ok": False, "why": "anchor counts must be tm=1 bc=1 rt=1 marker>=1"}

    # WHERE A TAG MAY BE WRITTEN. Rust allows a statement at the head of a BLOCK and after a
    # `let`; it does NOT allow one inside a match arm (arms must begin with a pattern) or inside a
    # struct literal's field list. An earlier draft injected `rebind18_set(1);` before
    # `Some(tn) => tn.clone(),` and `rebind18_set(3);` before `ret: callable_return_type(...)`,
    # which is a syntax error in both places -- and stubcheck cannot catch it, because stubcheck
    # only typechecks the injected FUNCTIONS, not the injections at these anchors. So TM is derived
    # from the value of type_match immediately after its let (arm-equivalent: the Some(tn) arm of
    # resolved_type is taken exactly when type_match is Some), and RT is written at the head of the
    # else-block that CONTAINS the struct literal, never inside the literal.
    out = src
    after_let = TM_ANCHOR + " " + RESET + 'rebind18_set(match type_match.clone() { Some(_) => 1usize, std::option::Option::None => 0usize }); '
    n = out.count(TM_ANCHOR)
    if n != 1:
        return src, rep | {"ok": False, "why": "type_match anchor matched %d times" % n}
    out = out.replace(TM_ANCHOR, after_let, 1)
    rep["tm_via"] = "post-let match on type_match (Some arm implied)"

    out = _inject_once(out, BC_ANCHOR, SET_BC, "BC tag")   # BC sits at the head of an if-block: valid

    # RT: walk back from the struct-field anchor to the `} else {` that opens the block holding it
    i = out.index(RT_ANCHOR)
    j = out.rindex("} else {", 0, i)
    if out.count(RT_ANCHOR, j, i + len(RT_ANCHOR)) != 1:
        return src, rep | {"ok": False, "why": "RT anchor is not alone inside its else-block"}
    k = j + len("} else {")
    out = out[:k] + " " + SET_RT + out[k:]
    rep["rt_via"] = "head of the else-block containing the struct literal"

    # THE CALL SITE. The marker sits inside `break Rc::new(UnifyGenericsAcc { ... })`, so inserting
    # AT the marker puts the call inside a struct literal's field list -- "expected one of `,`, `:`,
    # or `}`, found `.`" seven times on the runner. Walk back to the enclosing statement, the same
    # way probe v16 does.
    i = out.index(MARKER)
    b = out.rfind(BREAK, 0, i)
    if b < 0:
        return src, rep | {"ok": False, "why": "no enclosing break before the diagnostic marker"}
    out = out[:b] + CALL + out[b:]
    anchor_fn = "fn unify_generics("
    j = out.index(anchor_fn)
    out = out[:j] + TAG_DECL + PRINT_FN + "\n" + out[j:]

    # guards
    assert out.count("REBIND18|") == 1
    assert out.count(SET_BC) == 1 and out.count(SET_RT) == 1
    assert out.count(RESET) == 1
    for a, b in (("{", "}"), ("(", ")")):
        assert out.count(a) - out.count(b) == src.count(a) - src.count(b), "unbalanced %s%s" % (a, b)
    code = "\n".join(l for l in (TAG_DECL + PRINT_FN).splitlines() if l.strip() and not l.strip().startswith("//"))
    for bad in ("println!", "unify_generics(", "node_type_equals", "error_type()", "bare_map_node"):
        if bad in code:
            raise AssertionError("probe injects %r" % bad)
    return out, rep | {"ok": True}


def patch(a, b=None):
    """Dual entry (the contract instruments/stubcheck.py calls):
         patch(src_text)            -> (out, report)
         patch(in_path, out_path)   -> report
    An earlier version returned an int, and stubcheck crashed on `r.get("ok")` instead of
    reporting a verdict -- a gate that errors is not a gate that passes."""
    if isinstance(a, str) and b is None and ("\n" in a):
        return _patch_text(a)
    src = open(a, encoding="utf-8").read()
    out, rep = _patch_text(src)
    if rep.get("ok") and not rep.get("already"):
        open(b if b else a, "w", encoding="utf-8").write(out)
    return rep


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        # The fixture is written OUT LONG, with the three call-arm anchors spelled by hand and at
        # odd indentation -- it shares no constant with TM_ANCHOR/BC_ANCHOR/RT_ANCHOR, so a patcher
        # that only matches its own literal (or that depends on spacing) fails HERE rather than 20
        # minutes into a remote build. This is the rule the lane learned when a selftest derived its
        # fixture from the constant under test and passed while the real seed failed.
        seed = (
            "pub fn unify_generics(formal: Node, generic_names: Vec<String>) -> Acc {\n"
            "  loop { match v {\n"
            "        Some(bound) => {\n"
            "            let type_match = type_match_for_direct_call_target(scope.type_env.clone(), func_name.clone(), call_target.clone());\n"
            "let borrowed_call = match type_match.clone() {\n"
            "    Some(_) => std::option::Option::None,\n"
            "    std::option::Option::None => match global_bare_callable.clone() {\n"
            "    Some(gnode) => if ((gnode.params.clone().len() as i64) > 0) {\n"
            "                                        Some(borrowed_callable_call_type(gnode.clone(), typed_arg_nodes.clone(), scope.clone()))\n"
            "                                    } else {\n"
            "                                        Some(Rc::new(BorrowedCallType {\n"
            "    ret: crate::v1_compiler_infer_types::callable_return_type(gnode.clone()),\n"
            "    diagnostics: Rc::new(vec![]),\n"
            "})),\n"
            "                    },\n"
            "};\n"
            "let resolved_type = match type_match.clone() {\n"
            "    Some(tn) => tn.clone(),\n"
            "    std::option::Option::None => match borrowed_call.clone() {\n"
            "    Some(b) => b.ret.clone(),\n"
            "    std::option::Option::None => error_type(),\n"
            "},\n"
            "};\n"
            "            if x {\n"
            "                break Rc::new(UnifyGenericsAcc { subst: acc, diagnostics: zero });\n"
            "            } else {\n"
            "                break Rc::new(UnifyGenericsAcc { subst: acc, "
            "diagnostics: Rc::new(vec![type_mismatch_error(a, b, c)]));\n"
            "            }\n"
            "        }\n"
            "    }\n  }\n}\n"
        )
        out, rep = _patch_text(seed)
        print("SELFTEST", rep)
        assert rep.get("ok"), rep
        assert "REBIND18|" in out
        # every tag must be written, and the tag writes must sit BEFORE the fold print that reads them
        # TM is written by a match ON type_match's value (arm-equivalent, and legal after a let),
        # so it has no literal `rebind18_set(1)` in the source; BC and RT are literal.
        assert out.count("rebind18_set(2)") == 1, out.count("rebind18_set(2)")
        assert out.count("rebind18_set(3)") == 1, out.count("rebind18_set(3)")
        assert out.count("rebind18_set(match type_match.clone()") == 1
        assert "Some(_) => 1usize" in out and "None => 0usize" in out, "TM arms missing"
        assert "rebind18_set(0);" in out
        # the LAST occurrence is the CALL SITE: the helper DEFINITION is injected before
        # unify_generics, so index() would find the definition and every tag would look like it
        # landed too late. Asserting on the definition would have failed a correct patcher.
        assert out.count("rebind18_print(") == 2, \
            "expected 1 definition + 1 call, got %d" % out.count("rebind18_print(")
        k = out.rindex("rebind18_print(")
        assert k > out.rindex("fn unify_generics("), "call site is not inside the fold"
        for tag in ("rebind18_set(match type_match.clone()", "rebind18_set(2)", "rebind18_set(3)"):
            assert out.index(tag) < k, "%s landed after the fold print -- the tag would be stale" % tag
        assert out.index("rebind18_set(0);") < out.index("rebind18_set(2)"), "reset must precede the arm tags"
        # SYNTAX POSITION GUARDS -- the class of error that would otherwise cost a 25-minute remote
        # build: no tag statement may sit inside a match arm or a struct literal field list.
        for tag in ("rebind18_set(2);", "rebind18_set(3);"):
            pos = out.index(tag)
            tail = out[pos + len(tag):pos + len(tag) + 80]
            assert not re.match(r"\s*[A-Za-z_][A-Za-z0-9_]*\s*=>", tail), \
                "%s is immediately before a match arm pattern -- invalid Rust" % tag
            assert not re.match(r"\s*[A-Za-z_][A-Za-z0-9_]*\s*:", tail), \
                "%s is immediately before a struct field initializer -- invalid Rust" % tag
        print("SELFTEST ok")
        sys.exit(0)
    a = sys.argv[1]
    b = sys.argv[2] if len(sys.argv) > 2 else None
    r = patch(a, b)
    print("report:", r)
    sys.exit(0 if r.get("ok") else 1)
