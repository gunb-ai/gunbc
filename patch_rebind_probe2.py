"""Probe v2: attribute each fold rebind refusal to its DECIDING ARM, and print the FULL pairwise
tree that probe v1 flattened.

WHY v2 EXISTS. Probe v1 (patch_rebind_probe.py) answered Bucket B but left a self-declared gap that
I reported to stern-bat-401 rather than papering over: it prints 11 fields FLAT, prints
Resolved{node} as node.name (so the resolved node's own children are COLLAPSED), and prints no
params/body/uses/transport/expr_data. The SAME_TEXT pairs came back identical in every one of those
fields -- so "identical in every probed field" was NOT "identical in every field", and the refusal was
still unexplained. v2 closes exactly that gap.

PRE-REGISTERED OUTCOMES (stern-bat-401 msg_576466dc required registration BEFORE the run, so the
cause cannot be fitted to the data afterwards):
  A. text_crossing(ambient)=true  AND  node_type_equals_core(TextNotAsked)=true
     -> the refusal is the AMBIENT text judgment, not the two types: FOLD SCOPE. Repair in #14152 by
        passing the settled text judgment for each node's own module. (Same shape as the (c) fix:
        the fold was judging with the caller's context.)
  B. no crossing, but the pairwise tree shows a difference -- below Resolved{node}, or between raw
     children, or in declaration identity
     -> X1 PROPER: a descent/declaration defect in the relation. Its own PR.
  C. neither: no crossing, and the two trees are identical in every printed field, yet the relation
     refuses
     -> REPORT THE UNEXPLAINED PAIR BY IDENTITY. Do not pick a cause. Do not invent a predicate.

HOW THE ARMS ARE ATTRIBUTED, and why this is a measurement and not a re-implementation. I do NOT
copy node_type_equals' arm logic (that would put a second, drifting copy of the peer's relation in my
instrument and could only ever agree with itself). Instead the probe calls the REAL pub fns and takes
DIFFERENCES between them, which localises the refusal to an arm by itself:
  a_equals  = node_type_equals(lu, ru, si, text)            -- what the fold's second step asks
  a_core    = node_type_equals_core(lu, ru, si, text)       -- the same, wrapper bypassed
  a_cross   = text_crossing_in_judgment(lu, ru, text)       -- core's FIRST arm, in isolation
  a_core_na = node_type_equals_core(lu, ru, si, TextNotAsked) -- core with the judgment REMOVED
  rep_l/rep_r = text_representation_by_identity per side under the ambient env -- the crossing's own inputs
  a_textpair / a_headapp -- the fold's two remaining accepts, so a false relation is not assumed whole
Reading: a_equals=false, a_core=false, a_cross=true, a_core_na=true is outcome A and it is a
differential signature, not my reading of the code. rep_* is printed because a crossing is only
credible when one side is HostText/CodePointSequence and the other NotText.

WHAT v2 ADDS OVER v1 (the gap, closed): rebind_probe_tree walks BOTH operands in lockstep and descends
THROUGH InferredNode::Resolved{node} as well as raw children, printing per node: path, name,
connective, cardinality, decl(module::name or none), infer (full, un-collapsed), shape, nchildren,
identSpan. Descending Resolved is the whole point: v1 collapsed it, so a difference hiding one level
below a Resolved node was invisible by construction.

INSTRUMENT RULES ENFORCED HERE (each one learned from a probe that died or lied):
 - the helper is injected INTO v1_compiler_infer.rs, the fold's own file. 04_types.dag is NOT touched:
   every relation used is reached through `crate::v1_compiler_infer_types::`, whose fns are pub. So no
   second author edits the peer's file while I measure it.
 - anchors matched by count; the patch aborts unless each matches EXACTLY ONCE; the patcher EXITS
   NONZERO when it did not land, so an unpatched seed can never self-host and read as "no refusals".
 - the refusal is UNTOUCHED: the probe call is inserted immediately BEFORE the same `break
   Rc::new(UnifyGenericsAcc {`, so diagnostics, spans and the 208-row census stay comparable to config A.
 - no nested fn captures locals; all data arrives as arguments (a capture once killed four probes).
 - recursion is FUEL-BOUNDED and child-capped, and the tree is one eprintln per refusal: an unbounded
   walk over a recursive type would hang a 27-minute dispatch.
 - rustfmt must parse the result and rustc must typecheck it against a STUB CARRYING THE REAL Node
   FIELD SET, both LOCALLY, before any dispatch. v1's first run died on a 21-minute E0308 that a local
   stub compile would have caught for free.
"""
import sys

ANCHOR = '''} else {
                                                    break Rc::new(UnifyGenericsAcc {
    subst: acc.clone(),
    diagnostics: Rc::new(vec![type_mismatch_error('''

HELPER = r'''fn rebind_probe_infer2(inferred: Option<Rc<InferredNode>>) -> String {
    match inferred.clone() {
        Some(rc) => match rc.as_ref() {
            // `{}` placeholders, not `%s`: this file is EMITTED Rust and %s is the DAG surface's
            // interpolation syntax. As Rust it is an invalid format string, which the LOCAL stub gate
            // catches in under a second -- the same gate that exists because v1's E0308 cost 21 minutes.
            InferredNode::Resolved { node } => format!(
                "Resolved{{name={},shape={},nc={}}}",
                node.name.clone(),
                crate::v1_compiler_infer_types::node_type_shape(node.clone(), source_indices_empty()),
                node.children.clone().len() as i64
            ),
            InferredNode::TypeVariable { id } => format!("TypeVariable({})", id),
            InferredNode::CompilerError { message, span } => format!(
                "CompilerError({}@{}:{}-{})",
                message, span.file.clone(), span.start, span.end
            ),
            InferredNode::Divergent => "Divergent".to_string(),
        },
        None => "none".to_string(),
    }
}

fn source_indices_empty() -> Rc<HashMap<String, Rc<NewlineIndex>>> {
    Rc::new(HashMap::new())
}

fn rebind_probe_decl(n: &Rc<Node>) -> String {
    match n.declaration.clone() {
        Some(d) => format!("{}::{}", d.module_path.clone(), d.decl_name.clone()),
        None => "none".to_string(),
    }
}

fn rebind_probe_node(n: &Rc<Node>, path: &str, si: Rc<HashMap<String, Rc<NewlineIndex>>>) -> String {
    let mut s = String::new();
    s.push_str(path);
    s.push_str(" n=");
    s.push_str(&n.name.clone());
    s.push_str("|conn=");
    s.push_str(&format!("{:?}", n.connective.clone()));
    s.push_str("|card=");
    s.push_str(&format!("{:?}", n.return_cardinality.clone()));
    s.push_str("|decl=");
    s.push_str(&rebind_probe_decl(n));
    s.push_str("|infer=");
    s.push_str(&rebind_probe_infer2(n.inferred.clone()));
    s.push_str("|shape=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(n.clone(), si.clone()));
    s.push_str("|nc=");
    s.push_str(&format!("{}", n.children.clone().len() as i64));
    s.push_str("|idsp=");
    s.push_str(&match n.ident_span.clone() {
        Some(sp) => format!("{}:{}-{}", sp.file.clone(), sp.start, sp.end),
        None => "none".to_string(),
    });
    s
}

/// Lockstep walk of both operands. Descends raw children AND, for each side, the node behind
/// InferredNode::Resolved{node} -- the descent v1 collapsed and therefore could not see.
fn rebind_probe_tree(
    l: &Rc<Node>,
    r: &Rc<Node>,
    path: &str,
    depth: i64,
    si: Rc<HashMap<String, Rc<NewlineIndex>>>,
    out: &mut String,
) {
    if depth > 3 {
        out.push_str(" ||DEPTH-CAP");
        return;
    }
    out.push_str(" ;; ");
    out.push_str(&rebind_probe_node(l, &format!("L{}", path), si.clone()));
    out.push_str(" >< ");
    out.push_str(&rebind_probe_node(r, &format!("R{}", path), si.clone()));
    let lc = l.children.clone();
    let rc = r.children.clone();
    let n = if lc.len() > rc.len() { lc.len() } else { rc.len() } as i64;
    if n > 8 {
        out.push_str(" ||CHILD-CAP");
    }
    let lim = if n > 8 { 8 } else { n };
    let mut i: i64 = 0;
    while i < lim {
        let li = if (i as usize) < lc.len() {
            lc[i as usize].clone()
        } else {
            l.clone()
        };
        let ri = if (i as usize) < rc.len() {
            rc[i as usize].clone()
        } else {
            r.clone()
        };
        rebind_probe_tree(
            &li,
            &ri,
            &format!("{}{}", path, i),
            depth + 1,
            si.clone(),
            out,
        );
        i += 1;
    }
    // the Resolved descent v1 omitted
    let lr = match l.inferred.clone() {
        Some(rc2) => match rc2.as_ref() {
            InferredNode::Resolved { node } => Some(node.clone()),
            _ => None,
        },
        None => None,
    };
    let rr = match r.inferred.clone() {
        Some(rc2) => match rc2.as_ref() {
            InferredNode::Resolved { node } => Some(node.clone()),
            _ => None,
        },
        None => None,
    };
    if lr.is_some() || rr.is_some() {
        rebind_probe_tree(
            &lr.clone().unwrap_or_else(|| l.clone()),
            &rr.clone().unwrap_or_else(|| r.clone()),
            &format!("{}r", path),
            depth + 1,
            si.clone(),
            out,
        );
    }
}

fn rebind_probe_print2(
    bind_name: String,
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    module_name: String,
    text: Rc<TextJudgment>,
) {
    let lu = generic_type_unify_unfold(
        bound.clone(),
        text.clone(),
        0,
        source_indices.clone(),
    );
    let ru = generic_type_unify_unfold(
        actual.clone(),
        text.clone(),
        0,
        source_indices.clone(),
    );
    let not_asked = Rc::new(TextJudgment::TextNotAsked {
        reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
    });
    // The REAL relations, called, not copied. A refusal localises to an arm by the DIFFERENCE
    // between these, which is why instrumenting the peer's file is unnecessary and undesirable.
    let a_equals = crate::v1_compiler_infer_types::node_type_equals(
        lu.clone(),
        ru.clone(),
        source_indices.clone(),
        text.clone(),
    );
    let a_core = crate::v1_compiler_infer_types::node_type_equals_core(
        lu.clone(),
        ru.clone(),
        source_indices.clone(),
        text.clone(),
    );
    let a_cross = crate::v1_compiler_infer_types::text_crossing_in_judgment(
        lu.clone(),
        ru.clone(),
        text.clone(),
    );
    let a_core_na = crate::v1_compiler_infer_types::node_type_equals_core(
        lu.clone(),
        ru.clone(),
        source_indices.clone(),
        not_asked.clone(),
    );
    let a_equals_na = crate::v1_compiler_infer_types::node_type_equals(
        lu.clone(),
        ru.clone(),
        source_indices.clone(),
        not_asked.clone(),
    );
    let a_textpair = generic_types_are_the_text_pair(lu.clone(), ru.clone(), source_indices.clone());
    let a_headapp = generic_head_matches_application(
        lu.clone(),
        ru.clone(),
        source_indices.clone(),
    ) || generic_head_matches_application(
        ru.clone(),
        lu.clone(),
        source_indices.clone(),
    );
    let (rep_l, rep_r) = match (*text.clone()).clone() {
        TextJudgment::TextJudgedIn { env: e } => (
            format!(
                "{:?}",
                crate::v1_compiler_infer_env::text_representation_by_identity(lu.clone(), e.clone())
            ),
            format!(
                "{:?}",
                crate::v1_compiler_infer_env::text_representation_by_identity(ru.clone(), e.clone())
            ),
        ),
        TextJudgment::TextNotAsked { reason: _ } => ("NotAsked".to_string(), "NotAsked".to_string()),
    };
    let mut tree = String::new();
    rebind_probe_tree(
        &lu,
        &ru,
        "0",
        0,
        source_indices.clone(),
        &mut tree,
    );
    let mut s = String::new();
    s.push_str("REBINDARM|bind=");
    s.push_str(&bind_name.clone());
    s.push_str("|module=");
    s.push_str(&module_name.clone());
    // NEVER {:?} THE JUDGMENT. TextJudgment::TextJudgedIn{env} holds Rc<TypeEnv>, which carries
    // bindings + str_bindings + ancestry_str_bindings + parents: Vec<Rc<TypeEnv>> (recursively) +
    // source_indices: Map<String, NewlineIndex> for every file + intern_table + symbol_index
    // (v1_compiler_infer_env.rs:69-84). Debug-printing it once per refusal -- 208 times -- produces a
    // log nobody can read and would have burned a 27-minute dispatch to obtain. The ONLY env fact this
    // measurement needs is WHICH module the ambient judgment is bound to, because the pre-registered
    // hypothesis is that the fold judges with the CALLER's module. So: the name, and nothing else.
    s.push_str("|text=");
    s.push_str(&match (*text.clone()).clone() {
        TextJudgment::TextJudgedIn { env: e } => {
            format!("JudgedIn(module={})", e.module_path.clone())
        }
        TextJudgment::TextNotAsked { reason: r } => format!("NotAsked({:?})", r),
    });
    s.push_str("|equals=");
    s.push_str(&format!("{}", a_equals));
    s.push_str("|core=");
    s.push_str(&format!("{}", a_core));
    s.push_str("|cross=");
    s.push_str(&format!("{}", a_cross));
    s.push_str("|core_na=");
    s.push_str(&format!("{}", a_core_na));
    s.push_str("|equals_na=");
    s.push_str(&format!("{}", a_equals_na));
    s.push_str("|textpair=");
    s.push_str(&format!("{}", a_textpair));
    s.push_str("|headapp=");
    s.push_str(&format!("{}", a_headapp));
    s.push_str("|repL=");
    s.push_str(&rep_l);
    s.push_str("|repR=");
    s.push_str(&rep_r);
    s.push_str("|unfoldedL=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(lu.clone(), source_indices.clone()));
    s.push_str("|unfoldedR=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(ru.clone(), source_indices.clone()));
    s.push_str("|formal=");
    s.push_str(&rebind_probe_node(&formal, "F", source_indices.clone()));
    s.push_str(";;TREE");
    s.push_str(&tree);
    eprintln!("{}", s);
}

'''

CALL = '''rebind_probe_print2(
                                                        bind_name.clone(),
                                                        formal.clone(),
                                                        bound.clone(),
                                                        actual.clone(),
                                                        source_indices.clone(),
                                                        module_name.clone(),
                                                        text.clone(),
                                                    );
                                                    '''


def patch(path, out=None):
    src = open(path).read()
    report = {"anchor": src.count(ANCHOR), "already": src.count("rebind_probe_print2")}
    if report["already"]:
        return report | {"ok": True, "note": "already patched"}
    if report["anchor"] != 1:
        return report | {"ok": False, "why": "anchor must match exactly once"}
    head, tail = src.split(ANCHOR, 1)
    new = head + ANCHOR.replace("break Rc::new(UnifyGenericsAcc {", CALL + "break Rc::new(UnifyGenericsAcc {") + tail
    i = new.index("\npub fn ")
    new = new[:i] + "\n" + HELPER + new[i + 1:]
    open(out or path, "w").write(new)
    return report | {"ok": True, "bytes": len(new)}


if __name__ == "__main__":
    r = patch(sys.argv[1], sys.argv[2] if len(sys.argv) > 1 else None)
    print(r)
    sys.exit(0 if r.get("ok") else 4)
