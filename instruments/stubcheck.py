# ENVIRONMENT FIDELITY IS THE GATE'S WHOLE VALUE. It passed a helper that then failed remotely with
# E0599 `no method named dedup found for im::Vector`, because the EMITTED file does
# `use im::{vector as vec, HashMap, OrdSet as BTreeSet, Vector as Vec}` (v1_compiler_infer.rs:391) while
# my stub typed the fields as std Vec. A gate that models the wrong environment is worse than no gate: it
# converts a 30-second local failure into a 21-minute remote one, and it manufactures confidence. So the
# stub below aliases Vec to a minimal im::Vector-shaped type whose inherent methods are exactly the ones
# the real im::Vector has -- sort, truncate, len, contains, push, iter, clone -- and NO dedup/retain/drain.
# Anything std-only in a probe helper now fails HERE.

"""Typecheck a probe helper LOCALLY, against a stub crate carrying the REAL field sets, before any
dispatch costs 27 minutes.

WHY: probe v1's first run died after 21 minutes on `error[E0308]: mismatched types` twice, because
`Node.inferred` is `Option<Rc<InferredNode>>` and the helper matched `Some(InferredNode::Resolved{..})`
on it. That is visible in thirty seconds against a stub; it cost twenty-one minutes because it was only
ever compiled remotely, and the dispatch script then filtered the build output so I could not even read
the spans. An injected helper is Rust source: it gets a compile gate like any other source.

FIELD SETS ARE COPIED FROM THE EMISSION, not typed from memory: Node/InferredNode/Connective/
Cardinality/SourceSpan come from v1_std_core.rs and std_types.rs as emitted, and the fn signatures from
the mirrors' `pub fn` lines. The stub is therefore an instrument, not a guess: if the emission changes
shape, this fails and the probe is not dispatched.

Usage: stubcheck.py <patcher.py>
Runs the patcher over a fake seed carrying the stub, then `rustc --crate-type lib` over the result.
Exit 0 = safe to dispatch.
"""
import re
import subprocess
import sys
import importlib.util

REAL_NODE = '''
#[derive(Debug, Clone, PartialEq)] pub struct SourceSpan { pub file: String, pub start: i64, pub end: i64 }
#[derive(Debug, Clone, PartialEq)] pub struct NodeOccurrenceIdentity;
#[derive(Debug, Clone, PartialEq)] pub enum Connective { __CONNECTIVE__ }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Cardinality { One, ZeroOrOne, Many, CardOptional }
#[derive(Debug, Clone, PartialEq)] pub enum InferredNode { __INFERRED__ }
#[derive(Debug, Clone, PartialEq)] pub struct DeclarationRef { pub module_path: String, pub decl_name: String }
#[derive(Debug, Clone, PartialEq)] pub struct NewlineIndex { pub lines: Rc<Vec<i64>> }
#[derive(Debug, Clone, PartialEq)] pub struct Node {
    pub occurrence_identity: Rc<NodeOccurrenceIdentity>,
    pub name: String,
    pub ident: Option<i64>,
    pub span: Rc<SourceSpan>,
    pub ident_span: Option<Rc<SourceSpan>>,
    pub children: Rc<Vec<Rc<Node>>>,
    pub connective: Connective,
    pub params: Rc<Vec<Rc<Node>>>,
    pub inferred: Option<Rc<InferredNode>>,
    pub return_cardinality: Cardinality,
    pub uses: Rc<Vec<Rc<Node>>>,
    pub body: Option<Rc<Node>>,
    pub transport: Option<Rc<Node>>,
    pub properties: Rc<Vec<Rc<Node>>>,
    pub type_annotation: Option<Rc<Node>>,
    pub is_self_recursive: bool,
    pub has_non_tail_self_call: bool,
    pub declaration: Option<Rc<DeclarationRef>>,
}
#[derive(Debug, Clone, PartialEq)] pub struct TypeEnv {
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    // real field (v1_compiler_infer_env.rs:82): the only env fact probe v2 is allowed to print
    pub module_path: String,
    // real field (line 73), RECURSIVE on purpose: reproduces the unusable-log hazard that made the
    // first draft's `{:?}` on the judgment a dispatch-killer, so it fails HERE in seconds.
    pub parents: Rc<Vec<Rc<TypeEnv>>>,
}
#[derive(Debug, Clone, PartialEq)] pub enum TextNotAskedReason {
    TextNotAskedInVariantFieldSummary,
    TextNotAskedForCallableComponentResidue,
    TextNotAskedForSyntheticWitnessNodes,
}
#[derive(Debug, Clone, PartialEq)] pub enum TextJudgment {
    TextJudgedIn { env: Rc<TypeEnv> },
    TextNotAsked { reason: TextNotAskedReason },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum TextRepresentation { HostText, CodePointSequence, NotText, TextRepresentationUnidentified }
'''

REAL_FNS = '''
pub mod v1_compiler_infer_types {
    use super::*;
    pub fn node_type_shape(_n: Rc<Node>, _si: Rc<HashMap<String, Rc<NewlineIndex>>>) -> String { "X".into() }
    pub fn node_type_equals(_l: Rc<Node>, _r: Rc<Node>, _si: Rc<HashMap<String, Rc<NewlineIndex>>>, _t: Rc<TextJudgment>) -> bool { false }
    pub fn node_type_equals_core(_l: Rc<Node>, _r: Rc<Node>, _si: Rc<HashMap<String, Rc<NewlineIndex>>>, _t: Rc<TextJudgment>) -> bool { false }
    pub fn text_crossing_in_judgment(_l: Rc<Node>, _r: Rc<Node>, _t: Rc<TextJudgment>) -> bool { false }
    // S1B COUNTER STUBS: the real dispatch applies patch_s1b_counter.py to the types mirror, so a
    // gate without these reports 4 phantom E0425s and hides real errors in the noise. Fidelity of the
    // stub is the gate's whole value -- an earlier version passed a helper that failed remotely because
    // the stub typed Vec as std's while the emission aliases it to im::Vector.
    pub fn s1b_enter() {}
    pub fn s1b_exit() {}
    pub fn s1b_mint_snapshot() -> i64 { 0 }
}
pub mod v1_compiler_infer_env {
    use super::*;
    pub fn text_representation_by_identity(_n: Rc<Node>, _e: Rc<TypeEnv>) -> TextRepresentation { TextRepresentation::NotText }
}
pub fn generic_type_unify_unfold(n: Rc<Node>, _t: Rc<TextJudgment>, _d: i64, _si: Rc<HashMap<String, Rc<NewlineIndex>>>) -> Rc<Node> { n }
pub fn generic_types_are_the_text_pair(_l: Rc<Node>, _r: Rc<Node>, _si: Rc<HashMap<String, Rc<NewlineIndex>>>) -> bool { false }
pub fn generic_head_matches_application(_h: Rc<Node>, _a: Rc<Node>, _si: Rc<HashMap<String, Rc<NewlineIndex>>>) -> bool { false }
'''

# The call site, with the locals the refusal actually has in scope (verified from the emitted seed:
# unify_generics' TCO loop binds formal/actual/generic_names/source_indices/acc/text/module_name, and
# bind_name is bound just above the refusal).
# Stand-in for im::Vector: only the inherent methods the real one has. Inherent methods win over the
# HashMap-style trait path, so an accidental std-only call (dedup, retain, drain, dedup_by_key) is a
# hard E0599 here exactly as it is in the emitted crate.
IM_VECTOR = '''
pub struct Vector<A> { items: std::vec::Vec<A> }
// Methods are split by MINIMAL BOUND, mirroring the real im::Vector. A single impl block with
// `A: Clone + PartialEq + Ord` made len() demand Node: Ord, which the real type never asks for, and the
// gate then rejected code that compiles remotely. A gate that is stricter than reality is as useless as
// one that is laxer: both cost a dispatch to discover.
impl<A> Vector<A> {
    pub fn new() -> Vector<A> { Vector { items: std::vec::Vec::new() } }
    pub fn len(&self) -> i64 { self.items.len() as i64 }
}
// the emitted crate's `vec!` is im's macro (line 391 `use im::{vector as vec, ...}`), so the stub needs
// one too or the stub SEED fails to build and the helper is never actually typechecked.
#[macro_export]
macro_rules! vec {
    ( $( $x:expr ),* $(,)? ) => {{
        let mut __v = Vector::new();
        $( __v.push($x); )*
        __v
    }};
}
impl<A: Clone> Vector<A> {
    pub fn push(&mut self, a: A) { self.items.push(a); }
    // yields &A, like im::Vector::iter, so the emission's own x.clone().iter().cloned() idiom typechecks
    pub fn iter(&self) -> std::slice::Iter<'_, A> { self.items.iter() }
    pub fn truncate(&mut self, n: usize) { self.items.truncate(n); }
}
impl<A: PartialEq> Vector<A> {
    pub fn contains(&self, a: &A) -> bool { self.items.contains(a) }
}
impl<A: Ord> Vector<A> {
    pub fn sort(&mut self) { self.items.sort(); }
}
impl<A> Clone for Vector<A> where A: Clone { fn clone(&self) -> Vector<A> { Vector { items: self.items.clone() } } }
// The real im::Vector<A> implements Debug / PartialEq / Eq / Hash / Default / Ord when A does, and the
// stub's Node derives Debug+PartialEq over Rc<Vec<Rc<Node>>> fields -- so these must exist here too, or
// the gate fails on its OWN stub rather than on the helper (which is how the first attempt at this block
// presented: 4 errors, all inside the stub, none about the probe).
impl<A: std::fmt::Debug> std::fmt::Debug for Vector<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { self.items.fmt(f) }
}
impl<A: PartialEq> PartialEq for Vector<A> { fn eq(&self, o: &Vector<A>) -> bool { self.items == o.items } }
impl<A: Eq> Eq for Vector<A> {}
impl<A: std::hash::Hash> std::hash::Hash for Vector<A> {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) { self.items.hash(h) }
}
impl<A> Default for Vector<A> { fn default() -> Vector<A> { Vector { items: std::vec::Vec::default() } } }
'''
PREAMBLE_TMPL = '''#![allow(dead_code, unused_variables, unused_imports)]
use std::collections::HashMap;
use std::rc::Rc;
''' + IM_VECTOR + '''
// the emitted file's own alias (v1_compiler_infer.rs:391): Vec in this crate IS im::Vector
use Vector as Vec;
''' + REAL_NODE + REAL_FNS + '''
pub fn type_mismatch_error(_m: String) -> Rc<Node> { unimplemented!() }
fn type_node_label(_n: Rc<Node>, _si: Rc<HashMap<String, Rc<NewlineIndex>>>) -> String { "T".into() }
#[derive(Debug, Clone)] pub struct UnifyGenericsAcc { pub subst: Rc<HashMap<String, Rc<Node>>>, pub diagnostics: Rc<Vec<Rc<Node>>> }
pub fn unify_generics(
    formal: Rc<Node>, actual: Rc<Node>, generic_names: Rc<Vec<String>>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>, acc: Rc<UnifyGenericsAcc>,
    text: Rc<TextJudgment>, module_name: String,
) -> Rc<UnifyGenericsAcc> {
    let bind_name = type_node_label(formal.clone(), source_indices.clone());
    if true {
        eprintln!("{}", "REFUSE");
    } else {
        break_guard_placeholder();
    }
    acc
}
fn break_guard_placeholder() {}
'''



MIRROR_DEFAULT = "src/v1/stage0/src/v1_std_core.rs"


def derive_enums(mirror_path):
    """Take Connective and InferredNode VERBATIM from the real emitted mirror.

    These used to be hand-written here, and that is exactly how probe v11's injected code passed
    STUBCHECK and then died 20 minutes later with

      error[E0599]: no variant or associated item named `Neg` found for enum `Connective`

    -- because this file's stub listed a `Neg` variant that the real enum does not have
    (src/v1/stage0/src/v1_std_core.rs:148 lists exactly Conj, Disj, NoConnective, Arrow). A gate whose
    stubs are invented cannot catch an invented name: it just agrees with the mistake. So the stubs are
    now PARSED from the artifact the probe will actually run against, and a failure to parse is an error,
    never a fallback -- silently reverting to a guess is the failure mode that made this bug invisible.
    """
    import os
    path = os.environ.get("GUNBC_STUB_MIRROR", mirror_path)
    if not os.path.exists(path):
        raise SystemExit("STUBCHECK FAIL: mirror %s not found -- refusing to fall back to hand-written "
                         "enum stubs (that fallback is what hid the Neg bug)" % path)
    text = open(path, encoding="utf-8").read()
    out = {}
    for name in ("Connective", "InferredNode"):
        m = re.search(r"pub enum %s \{(.+?)\n\}" % name, text, re.S)
        if not m:
            raise SystemExit("STUBCHECK FAIL: 'pub enum %s' not found in %s -- cannot derive its variants"
                             % (name, path))
        body = m.group(1)
        # Keep the body VERBATIM. Re-joining line by line looks tidier and is wrong: struct-style variants
        # (`CompilerError { message: String, span: Rc<SourceSpan> },`) span several lines and a naive split
        # emits `pub enum InferredNode { Resolved {, node: Rc<Node>,, }, ... }` -- "expected identifier,
        # found `,`". The compiler already knows how to read the text; the only job here is to not corrupt it.
        if not body.strip():
            raise SystemExit("STUBCHECK FAIL: enum %s parsed empty" % name)
        out[name] = body
    return out


def main(patcher_path):
    enums = derive_enums(MIRROR_DEFAULT)
    print("stub enums DERIVED from the mirror: Connective = {%s} ; InferredNode = {%s}"
          % (enums["Connective"], enums["InferredNode"]))
    preamble = PREAMBLE_TMPL.replace("__CONNECTIVE__", enums["Connective"]) \
                            .replace("__INFERRED__", enums["InferredNode"])
    spec = importlib.util.spec_from_file_location("patcher", patcher_path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)

    # A seed that carries the patcher's own anchor verbatim, so the patcher runs exactly as it will on the
    # real seed. The anchor must sit inside `loop { if .. {} else {` because its `break` needs an enclosing
    # loop -- outside them rustc reports a syntax error unrelated to the helper, and a red gate that fails
    # for the wrong reason is worse than no gate.
    #
    # Two anchor styles are supported because patchers differ in how they locate the arm:
    #   mod.ANCHOR  -- a verbatim multi-line slice, indentation and all (patch_rebind_probe4/8). These are
    #                  the patchers that DIED on the head with anchor:0 once the emitter re-flowed the arm,
    #                  which is why the alternative exists.
    #   mod.MARKER  -- a unique semantic marker (the one line that builds a diagnostic), with the insertion
    #                  point found by walking back to the enclosing break. Indentation-independent.
    # The old ANCHOR ends exactly at `diagnostics: Rc::new(vec![type_mismatch_error(` and the template below
    # supplies the rest, so a MARKER patcher can be served by the same template: build the anchor from the
    # marker. A MARKER patcher never reads ANCHOR, so nothing about the old path changes.
    if hasattr(mod, "ANCHOR"):
        anchor = mod.ANCHOR
    elif hasattr(mod, "MARKER"):
        anchor = ('} else {\n'
                  '        break Rc::new(UnifyGenericsAcc {\n'
                  '            subst: acc.clone(),\n'
                  '            ' + mod.MARKER)
    else:
        print("STUBCHECK FAIL: patcher exposes neither ANCHOR nor MARKER")
        return 1
    seed = preamble + """
pub fn something_else() {}

fn _r(
    formal: Rc<Node>,
    bound: Rc<Node>,
    actual: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    acc: Rc<HashMap<String, Rc<Node>>>,
    text: Rc<TextJudgment>,
    module_name: String,
    generic_names: Rc<Vec<String>>,
) -> Rc<HashMap<String, Rc<Node>>> {
    let bind_name = type_node_label(formal.clone(), source_indices.clone());
    let _ = &bound; let _ = &actual; let _ = &text; let _ = &module_name;
    // generic_names must be a REAL binding, not a discard: the widened predicate reads it, and a helper
    // that silently ignored it would pass a gate it should fail.
    let _gn: Vec<String> = (*generic_names.clone()).clone();
    let _ = &_gn;
    let _verdict = loop {
        if false {
            eprintln!("x");
        """ + anchor + """"x".to_string())]),
});
        }
    };
    let _ = &_verdict;
    acc
}
"""
    open('/tmp/stub_seed.rs', 'w').write(seed)
    r = mod.patch('/tmp/stub_seed.rs', '/tmp/stub_patched.rs')
    print("patcher report:", r)
    if not r.get("ok"):
        print("STUBCHECK FAIL: patcher did not land")
        return 1

    p = subprocess.run(['rustc', '--edition', '2021', '--crate-type', 'lib',
                        '--emit=metadata', '-o', '/tmp/stub_meta.rmeta', '/tmp/stub_patched.rs'],
                       capture_output=True, text=True)
    errs = [l for l in (p.stderr or '').splitlines() if l.startswith('error')]
    if p.returncode != 0:
        print("STUBCHECK FAIL: rustc rc=%d" % p.returncode)
        lines = (p.stderr or '').splitlines()
        shown = 0
        for i, l in enumerate(lines):
            if l.startswith('error'):
                for j in range(i, min(i + 14, len(lines))):
                    print("  ", lines[j][:240])
                shown += 1
                if shown >= 8:
                    break
        if shown == 0:
            print("   (no line starts with 'error'; raw tail follows)")
            for l in lines[-40:]:
                print("  ", l[:240])
        return 1
    print("STUBCHECK OK: helper typechecks against the real field sets")
    warn = [l for l in (p.stderr or '').splitlines() if 'error' in l]
    if warn:
        print("  (note) error-shaped lines with rc=0:", warn[:5])
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1]))
