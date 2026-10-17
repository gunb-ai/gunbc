#!/usr/bin/env python3
"""Out-of-tree instrument injector for the v1 infer second-argument-round A/B.

Injected INSIDE the remote dispatch into each variant's generated mirror and
v1_compiler_compile.rs. NEVER committed on any variant. The accounting block
(gap_block.rs) is inserted byte-identically everywhere; only the hook sites differ,
because the variants shape the second argument round differently:

  control    main's algorithm: the second round is a gated SECOND
             infer_call_arguments_generic_pass (a whole replay). The region guard is
             set at that call site; the per-argument event fires in the fold body and
             arms only for a pass frame launched as a second round.
  treatment  the sparse fold: the per-argument event fires in the recheck branch
             (argument_needs_generic_recheck true) -- nothing else is re-derived.

WHY ANCHORS ARE WHITESPACE-TOLERANT REGEXES: main regenerated this mirror with a
different indent since the merge-base (the gated call moved 8 columns), so literal
anchors from one revision break on another. The regexes match any run of whitespace.

Fail-closed (DESIGN.md sec.5): every anchor must match EXACTLY once, every injected
marker must be found exactly once after re-reading from disk, or this aborts rc=7
without writing. A silently missed hook would produce a plausible-but-wrong count,
which is worse than no measurement.

usage: hooks.py control|treatment|treatmentB <gap_block.rs> [infer.rs] [compile.rs]
"""
import re
import sys

BLOCK_MARKER = "pub mod gap_instrument {"
DUMP_MARK = "crate::v1_compiler_infer::gap_instrument::dump();"

RE_BLOCK_DEF = re.compile(r"^[ \t]*(?:pub )?fn call_needs_generic_rebinding_pass\(", re.M)
RE_BLOCK_DEF_IFDELETED = re.compile(r"^[ \t]*pub fn call_needs_generic_rebinding_pass\(", re.M)
RE_PASS_DEF = re.compile(
    r"^[ \t]*pub fn infer_call_arguments_generic_pass\([ \t]*\n"
    r"[ \t]*call_args: [^,\n]*,", re.M)
RE_DUMP = re.compile(
    r"let[ \t]+_[ \t]*=[ \t]*v1_rt::trace_mark\([ \t]*\"compile\.reconcile\.done\""
    r"[ \t]*\.[ \t]*to_string\([ \t]*\)[ \t]*\)[ \t]*;")
# the gated second-round call site: the `if <gate>( ... ) {` immediately followed by the
# second pass call. Matched as one unit so the guard can be inserted as the first
# statement of the then-block.
RE_REGION = re.compile(
    r"if[ \t]+call_needs_generic_rebinding_pass\([^\n]*\n"
    r"(?:[ \t]*[^\n{]*\n)*?"
    r"[ \t]*\)[ \t]*\{[ \t]*\n"
    r"([ \t]*)(infer_call_arguments_generic_pass\()")
RE_RECHECK_BRANCH = re.compile(
    r"if[ \t]+argument_needs_generic_recheck\([^\n]*\n"
    r"(?:[ \t]*[^\n{]*\n)*?"
    r"[ \t]*\)[ \t]*\{[ \t]*\n"
    r"[ \t]*\{[ \t]*\n")
RE_FOLD_BODY = re.compile(
    r"^[ \t]*\|[ \t]*st:[ \t]*Rc<ArgGenericFoldState>[ \t]*,[ \t]*pair:[ \t]*\(i64,[ \t]*Rc<Node>\)[ \t]*\|[ \t]*\{",
    re.M)
# treatmentB: the gate is deleted, so the fold is unconditional. Locate the fold by the
# same shape as treatment (the recheck branch) -- it must still be there.


def die(msg):
    sys.stderr.write("GAP-HOOK FAIL: %s\n" % msg)
    sys.exit(7)


def one(matches, label):
    if len(matches) != 1:
        die("anchor %s matched %d times (need exactly 1)" % (label, len(matches)))
    return matches[0]


def main():
    if len(sys.argv) < 3:
        die("usage: hooks.py control|treatment|treatmentB gap_block.rs [infer.rs] [compile.rs]")
    side, block_path = sys.argv[1], sys.argv[2]
    if side not in ("control", "treatment", "treatmentB"):
        die("side must be control|treatment|treatmentB")
    infer_path = sys.argv[3] if len(sys.argv) > 3 else "src/v1/stage0/src/v1_compiler_infer.rs"
    compile_path = sys.argv[4] if len(sys.argv) > 4 else "src/v1/stage0/src/v1_compiler_compile.rs"
    block = open(block_path).read()
    if BLOCK_MARKER not in block:
        die("block missing its marker")

    infer = open(infer_path).read()
    comp = open(compile_path).read()
    if BLOCK_MARKER in infer:
        die("instrument already applied to %s" % infer_path)

    # ---- plan every edit before writing anything (§5: all-or-nothing) -------------
    # block anchor: the gate definition, or (treatmentB) any single stable anchor
    # The gate's presence/absence tells us which algorithm the tree actually carries --
    # a control built from the wrong mirror would otherwise measure the wrong thing.
    defs = list(RE_BLOCK_DEF.finditer(infer))
    if side == "treatmentB":
        if len(defs) != 0:
            die("treatmentB expects the coarse gate deleted, found %d definition(s)" % len(defs))
    else:
        if len(defs) != 1:
            die("%s expects exactly the coarse gate definition, found %d" % (side, len(defs)))
    m_pass = one(list(RE_PASS_DEF.finditer(infer)), "pass guard anchor")
    m_dump = one(list(RE_DUMP.finditer(comp)), "dump anchor")

    region = r2 = fold = None
    if side == "control":
        region = one(list(RE_REGION.finditer(infer)), "control second-round call site")
        fold = one(list(RE_FOLD_BODY.finditer(infer)), "control fold body")
    else:
        r2 = one(list(RE_RECHECK_BRANCH.finditer(infer)), "%s recheck branch" % side)

    # ---- apply ---------------------------------------------------------------------
    # 1) the accounting block, before the pass definition (stable in every variant)
    i = m_pass.start()
    infer = infer[:i] + block + "\n" + infer[i:]
    # offsets shifted by the block; re-locate everything else
    def rel(m):
        return m.start() + len(block) + 1

    # 2) the r1 pass guard, right after the pass signature
    m_pass = one(list(RE_PASS_DEF.finditer(infer)), "pass guard anchor (post-block)")
    sig_end = infer.index("{", m_pass.end()) + 1
    infer = infer[:sig_end] + "\n    let _gap_pass = gap_instrument::pass_guard();" + infer[sig_end:]

    # 3) the r2 hooks, shape-specific
    if side == "control":
        m_region = one(list(RE_REGION.finditer(infer)), "control second-round call site (post)")
        indent, call = m_region.group(1), "infer_call_arguments_generic_pass("
        ins = m_region.start(2)
        infer = (infer[:ins]
                 + "let _gap_region = gap_instrument::r2_region();\n" + indent + call
                 + infer[ins + len(call):])
        m_fold = one(list(RE_FOLD_BODY.finditer(infer)), "control fold body (post)")
        fe = m_fold.end()
        infer = (infer[:fe]
                 + "\n            let _gap_ev = gap_instrument::r2_event_if_region();"
                 + infer[fe:])
        want_event = "let _gap_ev = gap_instrument::r2_event_if_region();"
        want_region = 1
    else:
        m_r2 = one(list(RE_RECHECK_BRANCH.finditer(infer)), "%s recheck branch (post)" % side)
        fe = m_r2.end()
        infer = (infer[:fe]
                 + "\n                            let _gap_ev = gap_instrument::r2_event();"
                 + infer[fe:])
        want_event = "let _gap_ev = gap_instrument::r2_event();"
        want_region = 0

    open(infer_path, "w").write(infer)

    # 4) the dump at reconcile completion -- identical on every variant
    m_dump = one(list(RE_DUMP.finditer(comp)), "dump anchor (post)")
    de = m_dump.end()
    comp = comp[:de] + "\n                " + DUMP_MARK + comp[de:]
    open(compile_path, "w").write(comp)

    # ---- verify from disk, not from memory -----------------------------------------
    infer2 = open(infer_path).read()
    comp2 = open(compile_path).read()
    checks = [
        (infer2.count(BLOCK_MARKER), 1, "accounting block"),
        (infer2.count("let _gap_pass = gap_instrument::pass_guard();"), 1, "pass guard"),
        (infer2.count(want_event), 1, "r2 event hook"),
        (infer2.count("let _gap_region = gap_instrument::r2_region();"), want_region, "region hook"),
        (comp2.count(DUMP_MARK), 1, "dump hook"),
    ]
    for got, want, what in checks:
        if got != want:
            die("post-check %s: %d != %d" % (what, got, want))
    print("GAP-HOOK OK side=%s" % side)


main()
