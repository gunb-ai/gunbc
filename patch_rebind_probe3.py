"""Probe v3: ROUTING aggregates at unbounded depth, replacing v2's fuel-bounded tree dump.

WHY v3 EXISTS. stern-bat-401 (msg_43674e26) chose option (ii): re-measure on whole trees, with fuel high
enough that ZERO caps fire, or report how many still do. v2 walked at depth 3 / 8 children and its caps
fired 3598 DEPTH + 18 CHILD times, so "0 CompilerError nodes" and the first-differing-field table were
statements about WHAT WAS PRINTED, not about the trees -- and I have twice been wrong by generalising
from a partial census (the "52 producers all none", the "7 Unit fabrications").

WHY AGGREGATES AND NOT A BIGGER TREE DUMP. The types here are RECURSIVE: the FreeMonoid declaration's own
variant list contains Node(FreeMonoid<Primitive(T)>), so the Resolved descent re-enters the same shape.
Multiplying v2's per-node print by an unbounded depth over 254 rows does not produce a log, it produces
a multi-hundred-megabyte base64 tar that dies in transit -- and a log that cannot be shipped answers
nothing. So v3 walks with (a) a VISITED SET keyed on pointer identity, which stops the recursion where
it is genuinely cyclic rather than merely deep, and (b) a depth fuel set high enough to be no longer the
binding constraint -- and reports the cap counts instead of printing the tree. Every question the routing
needs is answerable from aggregates:

  Q1 (stern-bat-401's hypothesis for the conn-127 class): is the BOUND side the callee's generic
     DECLARATION'S OWN FORM, i.e. are its leaves exactly that declaration's own parameter names?
     Recorded per row as:
       lleaf / rleaf   multiset of LEAF names on each side (a leaf = nc==0, no shape parens)
       lparams / rparams  the `params` names on each side's nodes -- Node.params (v1_std_core.rs:1570)
                          is a field I never printed in v1 or v2, and it is what "its own declared
                          parameters" means mechanically
       lleaf_in_params / rleaf_in_params  EVERY leaf name appears among that side's own params names
       lleaf_eq_rleaf  the two leaf multisets are equal
     PREDICTION, registered BEFORE the run: if the parent's hypothesis holds, the ~127 rows whose first
     difference was `conn` satisfy lleaf_in_params=true on the BOUND side, and the accept control is that
     two DISTINCT INSTANCES of the same generic (FreeMonoid<Int> vs FreeMonoid<Bool>) must keep refusing
     with lleaf_in_params=false. If instead many conn rows show lleaf_in_params=false, the hypothesis is
     refuted and the fix is NOT an 'unsettled' extension.
  Q2 (S1 relevance, the critical-path question): at unbounded depth, do any CompilerError nodes appear on
     the compared path after all? v2 could only say "none within fuel".
  Q3 (the 18 outcome-C rows): still identical when whole trees are compared? If a difference appears at
     depth >3 it reclassifies B; if not, they are genuinely identical and get reported, not explained.
  Q4 (Unit): Product(Unit<>) on the bound side is upstream typing, unattributed; v3 records which rows so
     the 35 can be routed by identity rather than by my inference.

WHAT v3 DELIBERATELY DOES NOT DO. It does not touch src/v1/04_types.dag (quiet-eagle-395's authority; the
peel_alias_rhs_once/TextJudgment row is reported, not fixed), it does not build the 'unsettled' extension
(parent: "Don't build it until the data says so"), and it does not print per-node detail: an aggregate I
cannot verify against the emitter is how v2 produced two fabricated readings.
"""
import sys

ANCHOR = '''} else {
                                                    break Rc::new(UnifyGenericsAcc {
    subst: acc.clone(),
    diagnostics: Rc::new(vec![type_mismatch_error('''

HELPER = r'''fn rebind_probe_infer3(inferred: Option<Rc<InferredNode>>) -> String {
    match inferred.clone() {
        Some(rc) => match rc.as_ref() {
            InferredNode::Resolved { node } => format!(
                "Resolved{{name={},nc={}}}",
                node.name.clone(),
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

/// One side's aggregate. `seen` is keyed on POINTER IDENTITY: these types are recursive (the FreeMonoid
/// declaration's variant list contains Node(FreeMonoid<Primitive(T)>)), so a plain recursive walk re-enters
/// the same node forever. Deduplicating by address stops genuine cycles while still visiting every DISTINCT
/// node, which is what "whole tree" means here -- depth fuel then only guards pathological non-cyclic depth,
/// and its count is REPORTED rather than silently truncating (stern-bat-401: fuel high enough that zero caps
/// fire, or report how many still do).
fn rebind_probe_walk(
    n: &Rc<Node>,
    depth: i64,
    fuel: i64,
    seen: &mut std::collections::HashSet<usize>,
    agg: &mut RebindAgg,
) {
    // The fuel guard is a MEASUREMENT, not a safety net: pointer dedup already stops genuine cycles, so
    // a cap that still fires is a fact about the data's depth and must be counted, never silent.
    if depth > fuel {
        agg.caps_depth += 1;
        return;
    }
    let key = Rc::as_ptr(n) as usize;
    if seen.contains(&key) {
        agg.revisit += 1;
        return;
    }
    seen.insert(key);
    agg.nodes += 1;
    let shape = crate::v1_compiler_infer_types::node_type_shape(
        n.clone(),
        Rc::new(HashMap::new()),
    );
    let kids = n.children.clone();
    let is_leaf = kids.len() == 0 && !shape.contains('(');
    if is_leaf {
        agg.leaves.push(n.name.clone());
    }
    // Node.params (v1_std_core.rs:1570) is "this node's own declared parameters" -- the field v1 and v2
    // never printed, and the only mechanical reading of the parent's "its leaves are its own declared
    // parameters". Read directly; no peer fn called.
    for p in n.params.clone().iter() {
        if p.name.clone().len() > 0 {
            agg.params.push(p.name.clone());
        }
        for gp in p.params.clone().iter() {
            if gp.name.clone().len() > 0 {
                agg.params.push(gp.name.clone());
            }
        }
    }
    match n.inferred.clone() {
        Some(rc) => match rc.as_ref() {
            InferredNode::CompilerError { .. } => agg.compiler_error += 1,
            InferredNode::TypeVariable { .. } => agg.type_variable += 1,
            InferredNode::Resolved { node } => {
                agg.resolved += 1;
                rebind_probe_walk(&node.clone(), depth + 1, fuel, seen, agg);
            }
            InferredNode::Divergent => agg.divergent += 1,
        },
        None => agg.infer_none += 1,
    }
    for c in kids.iter() {
        rebind_probe_walk(&c.clone(), depth + 1, fuel, seen, agg);
    }
}

struct RebindAgg {
    nodes: i64,
    revisit: i64,
    leaves: Vec<String>,
    params: Vec<String>,
    compiler_error: i64,
    type_variable: i64,
    resolved: i64,
    divergent: i64,
    infer_none: i64,
    caps_depth: i64,
}

impl RebindAgg {
    fn new() -> RebindAgg {
        RebindAgg {
            nodes: 0,
            revisit: 0,
            leaves: Vec::new(),
            params: Vec::new(),
            compiler_error: 0,
            type_variable: 0,
            resolved: 0,
            divergent: 0,
            infer_none: 0,
            caps_depth: 0,
        }
    }
    /// Sorted so two multisets can be compared for equality, and truncated for the log with the
    /// truncation REPORTED. A silently truncated field is how an instrument starts lying.
    fn leaf_field(&self, limit: i64) -> String {
        let mut v = self.leaves.clone();
        v.sort();
        let trunc = v.len() as i64 > limit;
        if trunc {
            v.truncate(limit as usize);
        }
        format!("{}{}", v.join(","), if trunc { ",..TRUNC" } else { "" })
    }
    fn param_field(&self, limit: i64) -> String {
        let mut v = self.params.clone();
        v.sort();
        v.dedup();
        let trunc = v.len() as i64 > limit;
        if trunc {
            v.truncate(limit as usize);
        }
        format!("{}{}", v.join(","), if trunc { ",..TRUNC" } else { "" })
    }
    /// EVERY leaf name appears among that side's own declared parameter names. Empty leaves => false,
    /// never vacuously true: a vacuous control once let a row pose as a control (parent, msg_70dfb7e1).
    fn leaves_are_own_params(&self) -> bool {
        if self.leaves.len() == 0 || self.params.len() == 0 {
            return false;
        }
        let ps = self.params.clone();
        self.leaves.iter().all(|l| ps.contains(l))
    }
}

fn rebind_probe_print3(
    bind_name: String,
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    module_name: String,
    text: Rc<TextJudgment>,
) {
    let lu = generic_type_unify_unfold(bound.clone(), text.clone(), 0, source_indices.clone());
    let ru = generic_type_unify_unfold(actual.clone(), text.clone(), 0, source_indices.clone());
    let not_asked = Rc::new(TextJudgment::TextNotAsked {
        reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
    });
    // The REAL relations, called not copied; the refusal localises to an arm by their DIFFERENCE.
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
    let a_textpair = generic_types_are_the_text_pair(lu.clone(), ru.clone(), source_indices.clone());
    let a_headapp = generic_head_matches_application(lu.clone(), ru.clone(), source_indices.clone())
        || generic_head_matches_application(ru.clone(), lu.clone(), source_indices.clone());
    // FUEL: depth 40, with pointer deduplication doing the real work. v2's 3 was the binding constraint
    // (3598 DEPTH caps); 40 is chosen so that if a cap STILL fires it is a fact about the data, not about
    // the instrument, and it is reported as one.
    let mut la = RebindAgg::new();
    let mut ra = RebindAgg::new();
    let mut lseen = std::collections::HashSet::new();
    let mut rseen = std::collections::HashSet::new();
    rebind_probe_walk(&lu, 0, 40, &mut lseen, &mut la);
    rebind_probe_walk(&ru, 0, 40, &mut rseen, &mut ra);
    let mut s = String::new();
    s.push_str("REBINDV3|bind=");
    s.push_str(&bind_name.clone());
    s.push_str("|module=");
    s.push_str(&module_name.clone());
    s.push_str("|text=");
    // NEVER {:?} the judgment: TextJudgedIn{env} derefs Rc<TypeEnv>, which carries all bindings, the
    // intern table, per-file line offsets and parents: Vec<Rc<TypeEnv>> RECURSIVELY. 208 such prints is
    // an unreadable log; the module name is the only env fact this measurement needs.
    s.push_str(&match (*text.clone()).clone() {
        TextJudgment::TextJudgedIn { env: e } => format!("JudgedIn(module={})", e.module_path.clone()),
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
    s.push_str("|textpair=");
    s.push_str(&format!("{}", a_textpair));
    s.push_str("|headapp=");
    s.push_str(&format!("{}", a_headapp));
    s.push_str("|fconn=");
    s.push_str(&format!("{:?}", formal.connective.clone()));
    s.push_str("|lconn=");
    s.push_str(&format!("{:?}", lu.connective.clone()));
    s.push_str("|rconn=");
    s.push_str(&format!("{:?}", ru.connective.clone()));
    s.push_str("|lname=");
    s.push_str(&lu.name.clone());
    s.push_str("|rname=");
    s.push_str(&ru.name.clone());
    s.push_str("|lshape=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(lu.clone(), source_indices.clone()));
    s.push_str("|rshape=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(ru.clone(), source_indices.clone()));
    s.push_str("|formal=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(formal.clone(), source_indices.clone()));
    s.push_str("|ldecl=");
    s.push_str(&match lu.declaration.clone() {
        Some(d) => format!("{}::{}", d.module_path.clone(), d.decl_name.clone()),
        None => "none".to_string(),
    });
    s.push_str("|rdecl=");
    s.push_str(&match ru.declaration.clone() {
        Some(d) => format!("{}::{}", d.module_path.clone(), d.decl_name.clone()),
        None => "none".to_string(),
    });
    s.push_str("|linfer=");
    s.push_str(&rebind_probe_infer3(lu.inferred.clone()));
    s.push_str("|rinfer=");
    s.push_str(&rebind_probe_infer3(ru.inferred.clone()));
    // aggregates, named l* / r* so neither side is privileged
    s.push_str("|lnodes=");
    s.push_str(&format!("{}", la.nodes));
    s.push_str("|rnodes=");
    s.push_str(&format!("{}", ra.nodes));
    s.push_str("|lrevisit=");
    s.push_str(&format!("{}", la.revisit));
    s.push_str("|rrevisit=");
    s.push_str(&format!("{}", ra.revisit));
    s.push_str("|lce=");
    s.push_str(&format!("{}", la.compiler_error));
    s.push_str("|rce=");
    s.push_str(&format!("{}", ra.compiler_error));
    s.push_str("|ltv=");
    s.push_str(&format!("{}", la.type_variable));
    s.push_str("|rtv=");
    s.push_str(&format!("{}", ra.type_variable));
    s.push_str("|lcaps=");
    s.push_str(&format!("{}", la.caps_depth));
    s.push_str("|rcaps=");
    s.push_str(&format!("{}", ra.caps_depth));
    s.push_str("|lleafinown=");
    s.push_str(&format!("{}", la.leaves_are_own_params()));
    s.push_str("|rleafinown=");
    s.push_str(&format!("{}", ra.leaves_are_own_params()));
    s.push_str("|lleaf=");
    s.push_str(&la.leaf_field(24));
    s.push_str("|rleaf=");
    s.push_str(&ra.leaf_field(24));
    s.push_str("|lparams=");
    s.push_str(&la.param_field(16));
    s.push_str("|rparams=");
    s.push_str(&ra.param_field(16));
    s.push_str("|lleafn=");
    s.push_str(&format!("{}", la.leaves.len() as i64));
    s.push_str("|rleafn=");
    s.push_str(&format!("{}", ra.leaves.len() as i64));
    eprintln!("{}", s);
}

'''

CALL = '''rebind_probe_print3(
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
    report = {"anchor": src.count(ANCHOR), "already": src.count("rebind_probe_print3")}
    if report["already"]:
        return report | {"ok": True, "note": "already patched"}
    if report["anchor"] != 1:
        return report | {"ok": False, "why": "anchor must match exactly once"}
    head, tail = src.split(ANCHOR, 1)
    new = head + ANCHOR.replace(
        "break Rc::new(UnifyGenericsAcc {", CALL + "break Rc::new(UnifyGenericsAcc {"
    ) + tail
    i = new.index("\npub fn ")
    new = new[:i] + "\n" + HELPER + new[i + 1:]
    open(out or path, "w").write(new)
    return report | {"ok": True, "bytes": len(new)}


def selftest():
    """Run patch() over a synthetic seed carrying the anchor in its real context, in ~30ms.

    The patcher is only ever invoked AFTER the 20-minute regen that produces the seed, so every defect in
    it is priced at a full dispatch: the v2 run proved that by dying on `sys.argv[2] if len(sys.argv) > 1`
    (guard must be > 2) after 20 converging minutes. Cheap gate, expensive failure. It cannot prove the
    anchor exists in the REAL emission -- only a rehearsal mirror can -- so the anchor count is still
    reported and still gates.
    """
    synth = ("pub fn f() {\n    loop {\n        if false { eprintln!(\"x\"); }\n        "
             + ANCHOR + '"m".to_string())]),\n});\n    }\n}\n\npub fn other() {}\n')
    open("/tmp/p3_selftest_seed.rs", "w").write(synth)
    r = patch("/tmp/p3_selftest_seed.rs", "/tmp/p3_selftest_out.rs")
    if not r.get("ok"):
        print("SELFTEST FAIL:", r)
        return 1
    out = open("/tmp/p3_selftest_out.rs").read()
    for need in ("rebind_probe_print3(", "fn rebind_probe_walk", "struct RebindAgg",
                 "fn leaves_are_own_params"):
        if need not in out:
            print("SELFTEST FAIL: injected output lacks", need)
            return 1
    print("SELFTEST OK", r)
    return 0


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--selftest":
        sys.exit(selftest())
    # > 2, not > 1: with argv=[prog, seed] a `> 1` guard passes and sys.argv[2] raises IndexError
    # -- after the regen, i.e. priced at a whole dispatch.
    r = patch(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
    print(r)
    sys.exit(0 if r.get("ok") else 4)
