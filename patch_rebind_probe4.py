#!/usr/bin/env python3
"""PROBE v4 -- measure the EXACT predicate #14152 now carries, on real data, before trusting it.

WHY THIS EXISTS. The approved predicate is 'the binding IS the generic declaration's own form (its
leaves are its own declared parameters)'. Two readings differ and the earlier probes cannot separate
them:
  (i)  ROOT-OWN: the compared node's OWN Node.params -- what the DAG code in src/v1/04_infer.dag now
       implements;
  (ii) TREE-AGG: params collected from EVERY visited node -- what probe v3 printed as `lparams`, and
       what the 142 count silently used.
Probe v3 aggregated params into one set, so 142 is an UPPER BOUND on (i); (i) could be 0, a predicate
that admits nothing and looks like a clean no-op. This probe computes BOTH in Rust with the same shape
as the shipped DAG code, per refusal row, plus a path-tagged tree so the deciding node is identifiable
(`r` in a path is an `inferred` Resolved hop: a witness seen only under an `r` is invisible to a
children-only walk, which is why the hop had to be added).

IT ALSO CARRIES THE S1B COUNTER the parent asked for, in the SAME dispatch:
  "add a counter inside resolved_type's minting arms (Absent / else) for calls reached from
   node_type_equals_core during the fold's comparisons. If it's 0, S1b stays parked for good"
bracketed per row, so the answer is attributable to rows rather than a global total.

MEASUREMENT ONLY: builds nothing, edits no .dag, and is NOT part of #14152. The fold change is measured
from the worktree; the instrument comes from a scratch commit.
"""
import sys

ANCHOR = '''} else {
                                                    break Rc::new(UnifyGenericsAcc {
    subst: acc.clone(),
    diagnostics: Rc::new(vec![type_mismatch_error('''

CALL = '''rebind_probe_print4(
                                                        bind_name.clone(),
                                                        formal.clone(),
                                                        bound.clone(),
                                                        actual.clone(),
                                                        source_indices.clone(),
                                                        module_name.clone(),
                                                        text.clone(),
                                                    );
                                                    '''

HELPER = r'''
// ---- PROBE v4 (instrumentation; NEVER lands; see the module docstring) --------------------
// THIS FILE'S `Vec` IS NOT std's: line 391 reads `use im::{vector as vec, HashMap, OrdSet as
// BTreeSet, Vector as Vec};`, so a bare Vec<String> is im::Vector. It has push/sort/truncate/len/
// contains/iter but NO join() and NO dedup() -- an earlier probe lost a 21-minute dispatch to that.
// Strings are therefore built by iteration and membership uses contains().

fn rebind_probe_join4(v: &Vec<String>, sep: &str) -> String {
    let mut out = String::new();
    let mut first = true;
    for it in v.clone().iter().cloned() {
        if !first {
            out.push_str(sep);
        }
        out.push_str(&it);
        first = false;
    }
    out
}

fn rebind_probe_names_of(n: &Rc<Node>) -> Vec<String> {
    // A node's OWN declared parameters, plus each parameter's own parameters -- the nested level the
    // measured capture used. Empty names are skipped so a blank never matches a blank leaf.
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

/// PREDICATE (i) -- ROOT-OWN: mirrors the DAG code in src/v1/04_infer.dag EXACTLY, so the probe
/// measures what lands rather than what was intended. Childless node's NAME (never its printed shape:
/// a slot leaf prints `Primitive(T)` with name T), raw children, then the bounded `inferred` Resolved
/// hop. `fired` counts visited nodes and bails, because `inferred` can point back up.
fn rebind_probe_pred_own(
    n: &Rc<Node>,
    declared: &Vec<String>,
    hops: i64,
    fired: &mut i64,
) -> bool {
    *fired += 1;
    if *fired > 400000 {
        return false;
    }
    if (n.children.clone().len() as i64) == 0 && declared.contains(&n.name.clone()) {
        return true;
    }
    for c in n.children.clone().iter().cloned() {
        if rebind_probe_pred_own(&c, declared, hops, fired) {
            return true;
        }
    }
    if hops > 0 {
        match n.inferred.clone() {
            Some(rc) => match rc.as_ref() {
                InferredNode::Resolved { node } => {
                    rebind_probe_pred_own(&node.clone(), declared, hops - 1, fired)
                }
                _ => false,
            },
            None => false,
        }
    } else {
        false
    }
}

/// PREDICATE (ii) -- TREE-AGGREGATED: probe v3's reading, kept for contrast so a disagreement between
/// the two readings is VISIBLE rather than assumed. Params from any visited node count as declared.
fn rebind_probe_tree(
    n: &Rc<Node>,
    depth: i64,
    fuel: i64,
    seen: &mut std::collections::HashSet<usize>,
    names: &mut Vec<String>,
    params: &mut Vec<String>,
    path: String,
    tree: &mut Vec<String>,
    caps: &mut i64,
) {
    if depth > fuel {
        *caps += 1;
        return;
    }
    let key = Rc::as_ptr(n) as usize;
    if seen.contains(&key) {
        return;
    }
    seen.insert(key);
    for p in rebind_probe_names_of(n).iter().cloned() {
        params.push(p.clone());
    }
    let shape = crate::v1_compiler_infer_types::node_type_shape(
        n.clone(),
        Rc::new(HashMap::new()),
    );
    let kids = n.children.clone();
    if kids.len() as i64 == 0 {
        names.push(n.name.clone());
    }
    if (tree.len() as i64) < 80 {
        tree.push(format!(
            "{}#{} n={} nc={} s={}",
            if tree.len() as i64 == 0 { "" } else { " ~ " },
            path,
            n.name.clone(),
            kids.len() as i64,
            shape.clone()
        ));
    }
    let mut idx = 0;
    for c in kids.clone().iter().cloned() {
        rebind_probe_tree(
            &c,
            depth + 1,
            fuel,
            seen,
            names,
            params,
            format!("{}{}", path.clone(), idx),
            tree,
            caps,
        );
        idx += 1;
    }
    match n.inferred.clone() {
        Some(rc) => match rc.as_ref() {
            InferredNode::Resolved { node } => rebind_probe_tree(
                &node.clone(),
                depth + 1,
                fuel,
                seen,
                names,
                params,
                format!("{}r", path.clone()),
                tree,
                caps,
            ),
            _ => {}
        },
        None => {}
    }
}

fn rebind_probe_bool(b: bool) -> String {
    if b { "1".to_string() } else { "0".to_string() }
}

/// The connective as a word, for the PR-body classification. Matched with a `_` arm on purpose: naming
/// every Connective variant would make this helper fail to COMPILE if the enum grows, and a helper that
/// does not compile costs a whole dispatch, while `_ => other` only loses a label. Never {:?} here --
/// an earlier probe lost a dispatch to a Debug impl that recursed through parents.
fn rebind_probe_conn(n: &Rc<Node>) -> String {
    match n.connective.clone() {
        Connective::Disj => "Disj".to_string(),
        Connective::NoConnective => "None".to_string(),
        _ => "other".to_string(),
    }
}

fn rebind_probe_print4(
    bind_name: String,
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    module_name: String,
    text: Rc<TextJudgment>,
) {
    let lp = rebind_probe_names_of(&bound);
    let rp = rebind_probe_names_of(&actual);
    let mut f1: i64 = 0;
    let mut own_bound = false;
    for c in bound.children.clone().iter().cloned() {
        if rebind_probe_pred_own(&c, &lp, 3, &mut f1) {
            own_bound = true;
        }
    }
    let mut f2: i64 = 0;
    let mut own_actual = false;
    for c in actual.children.clone().iter().cloned() {
        if rebind_probe_pred_own(&c, &rp, 3, &mut f2) {
            own_actual = true;
        }
    }
    let mut seen_l: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut ln: Vec<String> = Vec::new();
    let mut lpar: Vec<String> = Vec::new();
    let mut ltree: Vec<String> = Vec::new();
    let mut lcap: i64 = 0;
    rebind_probe_tree(
        &bound, 0, 40, &mut seen_l, &mut ln, &mut lpar, String::new(), &mut ltree, &mut lcap,
    );
    let tree_bound_all = ln.len() as i64 > 0
        && lpar.len() as i64 > 0
        && ln.iter().all(|x| lpar.contains(x));
    let tree_bound_any = ln.len() as i64 > 0
        && lpar.len() as i64 > 0
        && ln.iter().any(|x| lpar.contains(x));
    let mut seen_r: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut rn: Vec<String> = Vec::new();
    let mut rpar: Vec<String> = Vec::new();
    let mut rtree: Vec<String> = Vec::new();
    let mut rcap: i64 = 0;
    rebind_probe_tree(
        &actual, 0, 40, &mut seen_r, &mut rn, &mut rpar, String::new(), &mut rtree, &mut rcap,
    );
    let tree_actual_any = rn.len() as i64 > 0
        && rpar.len() as i64 > 0
        && rn.iter().any(|x| rpar.contains(x));
    // THE S1B COUNTER: minting inside the type authority, during THIS fold comparison only. Bracketing
    // it here rather than globally is what makes "reached from node_type_equals_core" mean something.
    let m0 = crate::v1_compiler_infer_types::s1b_mint_snapshot();
    crate::v1_compiler_infer_types::s1b_enter();
    let eq_core = crate::v1_compiler_infer_types::node_type_equals_core(
        bound.clone(),
        actual.clone(),
        source_indices.clone(),
        text.clone(),
    );
    crate::v1_compiler_infer_types::s1b_exit();
    let m1 = crate::v1_compiler_infer_types::s1b_mint_snapshot();
    let mut s = String::new();
    s.push_str("REBINDV4|module=");
    s.push_str(&module_name);
    s.push_str("|bind=");
    s.push_str(&bind_name);
    s.push_str("|formal=");
    s.push_str(&formal.name.clone());
    s.push_str("|lshape=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(
        bound.clone(), Rc::new(HashMap::new()),
    ));
    s.push_str("|rshape=");
    s.push_str(&crate::v1_compiler_infer_types::node_type_shape(
        actual.clone(), Rc::new(HashMap::new()),
    ));
    s.push_str("|lconn=");
    s.push_str(&rebind_probe_conn(&bound));
    s.push_str("|rconn=");
    s.push_str(&rebind_probe_conn(&actual));
    s.push_str("|lname=");
    s.push_str(&bound.name.clone());
    s.push_str("|rname=");
    s.push_str(&actual.name.clone());
    s.push_str("|ownBound=");
    s.push_str(&rebind_probe_bool(own_bound));
    s.push_str("|ownActual=");
    s.push_str(&rebind_probe_bool(own_actual));
    s.push_str("|treeBoundAny=");
    s.push_str(&rebind_probe_bool(tree_bound_any));
    s.push_str("|treeBoundAll=");
    s.push_str(&rebind_probe_bool(tree_bound_all));
    s.push_str("|treeActualAny=");
    s.push_str(&rebind_probe_bool(tree_actual_any));
    s.push_str("|coreEq=");
    s.push_str(&rebind_probe_bool(eq_core));
    s.push_str("|s1bMint=");
    s.push_str(&format!("{}", (m1 - m0) as i64));
    s.push_str("|rootOwnParams=");
    s.push_str(&rebind_probe_join4(&lp, ","));
    s.push_str("|actOwnParams=");
    s.push_str(&rebind_probe_join4(&rp, ","));
    s.push_str("|treeParams=");
    s.push_str(&rebind_probe_join4(&lpar, ","));
    s.push_str("|lleafn=");
    s.push_str(&format!("{}", ln.len() as i64));
    s.push_str("|lcaps=");
    s.push_str(&format!("{}", lcap as i64));
    s.push_str("|tcaps=");
    s.push_str(&format!("{}", rcap as i64));
    s.push_str("|leaves=");
    s.push_str(&rebind_probe_join4(&ln, ","));
    s.push_str("|tleaves=");
    s.push_str(&rebind_probe_join4(&rn, ","));
    s.push_str("|treepath=");
    s.push_str(&rebind_probe_join4(&ltree, ""));
    eprintln!("{}", s);
}
// ---- END PROBE v4 -------------------------------------------------------------------------

'''


def patch(path, out=None):
    src = open(path).read()
    report = {"anchor": src.count(ANCHOR), "already": src.count("rebind_probe_print4")}
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
    synth = ("pub fn f() {\n    loop {\n        if false { eprintln!(\"x\"); }\n        "
             + ANCHOR + '"m".to_string())]),\n});\n    }\n}\n\npub fn other() {}\n')
    open("/tmp/p4_selftest_seed.rs", "w").write(synth)
    r = patch("/tmp/p4_selftest_seed.rs", "/tmp/p4_selftest_out.rs")
    if not r.get("ok"):
        print("SELFTEST FAIL:", r)
        return 1
    out = open("/tmp/p4_selftest_out.rs").read()
    for need in ("rebind_probe_print4(", "fn rebind_probe_pred_own", "fn rebind_probe_tree",
                 "fn rebind_probe_join4", "rebind_probe_conn", "|lconn=", "|lname=",
                 "s1b_mint_snapshot", "node_type_equals_core"):
        if need not in out:
            print("SELFTEST FAIL: injected output lacks", need)
            return 1
    # THE TRAP THAT COST A DISPATCH: `.join(`/`.dedup(` compile against a std-Vec stub and fail
    # remotely on im::Vector. Refuse them here, at zero cost, instead of at 69 minutes.
    for bad in (".join(", ".dedup(", "rebind_probe_pred_own(&kids["):
        if bad in out:
            print("SELFTEST FAIL: forbidden im::Vector method in injected helper:", bad)
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
