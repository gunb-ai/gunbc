#!/usr/bin/env python3
# PROBE INSTRUMENT (measurement only; never imported by production code).
# dissolve-on: the E0308 population reaches zero on the self-host board, or the partition moves
# into a modeled .dag carrier that reads the diagnostic stream directly. Committed rather than
# run ad hoc because the 2026-08-21 partition had no committed classifier, so its categories
# could not be re-derived and a later lane had to re-author the discriminators from prose.
"""E0308 raw rows -> canonical sites -> mechanism clusters (candidate roots).

Reads a cargo human-format build log; emits a per-site TSV. Fail-closed:
any site that does not match a discriminator is printed as RESIDUE, never
absorbed into the nearest familiar category.
"""
import re, sys, collections

ANSI = re.compile(r"\x1b\[[0-9;]*m")
BLOCK = re.compile(r"^error\[(E\d+)\]: (.*)$")
SPAN = re.compile(r"^\s*-->\s+(\S+?):(\d+):(\d+)\s*$")
INLINE = re.compile(r"expected (?:type parameter |struct |enum |reference |unit type )?`(.+?)`, found (?:type parameter |struct |enum |reference |unit type )?`(.+?)`")
NOTE_EXP = re.compile(r"=\s*note:\s*expected (\w+(?: \w+)?) `(.+?)`\s*$")
NOTE_FOUND = re.compile(r"^\s*found (\w+(?: \w+)?) `(.+?)`\s*$")

def read_blocks(path):
    lines = [ANSI.sub("", l.rstrip("\n")) for l in open(path, encoding="utf-8", errors="replace")]
    blocks, cur = [], None
    for i, l in enumerate(lines):
        m = BLOCK.match(l)
        if m:
            if cur: blocks.append(cur)
            cur = {"code": m.group(1), "msg": m.group(2), "lines": []}
            continue
        if cur is not None:
            if l.startswith("error") or l.startswith("warning:"):
                blocks.append(cur); cur = None; continue
            cur["lines"].append(l)
    if cur: blocks.append(cur)
    return [b for b in blocks if b["code"] == "E0308"]

def block_sites(b):
    """Expand one block into canonical mismatch sites."""
    file = line = col = None
    callee_note = ""
    callee_kind = ""
    pending_note = None
    notes = []          # (expected, found) from `= note:` groups
    inlines = []        # (expected, found) from caret labels
    reorder = False
    in_callee = False
    for l in b["lines"]:
        s = SPAN.match(l)
        if s:
            if in_callee and not callee_note:
                callee_note = "%s:%s:%s" % s.groups()
                in_callee = False
            elif file is None:
                file, line, col = s.group(1), s.group(2), s.group(3)
            continue
        if l.startswith("note: ") and "defined here" in l:
            in_callee = True
            callee_kind = l[len("note: "):].strip()
            continue
        if "reorder these arguments" in l or l.startswith("help: did you mean"):
            reorder = True
        m = NOTE_EXP.search(l)
        if m:
            pending_note = (m.group(1), m.group(2)); continue
        if pending_note:
            f = NOTE_FOUND.match(l)
            if f:
                notes.append((pending_note[1], f.group(2), pending_note[0], f.group(1)))
            pending_note = None
        for m in INLINE.finditer(l):
            inlines.append((m.group(1), m.group(2)))
    pairs = []
    seen = set()
    for e, f in ((normalize(a), normalize(b)) for a, b in inlines):
        if (e, f) not in seen:
            seen.add((e, f)); pairs.append((e, f, "", ""))
    for e, f, ke, kf in ((normalize(a), normalize(b), c, d) for a, b, c, d in notes):
        if (e, f) not in seen:
            seen.add((e, f)); pairs.append((e, f, ke, kf))
    # One mismatch printed two ways: rustc's caret label elides deep generics as `...`
    # while the `= note:` prints the type in full. Keeping both would count one site twice.
    def elision_of(a, b):
        if "..." not in a[0] + a[1] or "..." in b[0] + b[1]:
            return False
        pat = lambda t: "^" + re.escape(t).replace(r"\.\.\.", ".*") + "$"
        return bool(re.match(pat(a[0]), b[0]) and re.match(pat(a[1]), b[1]))
    pairs = [a for a in pairs if not any(elision_of(a, b) for b in pairs if b is not a)]
    out = []
    if not pairs:
        out.append(dict(file=file, line=line, col=col, expected="", found="",
                        kind_e="", kind_f="", callee=callee_note, callee_kind=callee_kind,
                        reorder=reorder, msg=b["msg"], nopair=True))
    for e, f, ke, kf in pairs:
        out.append(dict(file=file, line=line, col=col, expected=e, found=f,
                        kind_e=ke, kind_f=kf, callee=callee_note, callee_kind=callee_kind,
                        reorder=reorder, msg=b["msg"], nopair=False))
    return out

# ---- delta vector -----------------------------------------------------------
def strip_rc(t):
    n = 0
    while t.startswith("Rc<") and t.endswith(">"):
        t = t[3:-1]; n += 1
    return t, n

def head(t):
    """Nominal head: generic arguments dropped, module path dropped."""
    t = t.split("<")[0]
    return t.split("::")[-1].strip("&")

PATH_NOISE = re.compile(r"\b(?:std::string::|std::option::|std::vec::|std::collections::|im::|alloc::)")
def normalize(t):
    """Semantic identity of a type spelling: module-path noise removed, spacing collapsed.
    `im::Vector<..>` and `Vector<..>` are ONE type printed two ways by rustc; keeping both
    would count one mismatch twice."""
    t = PATH_NOISE.sub("", t)
    t = re.sub(r"\b[a-z0-9_]+::", "", t)
    return re.sub(r"\s+", " ", t).strip()

TEXT = {"String", "str", "&str"}
NUMERIC_NATIVE = {"i64", "u64", "i32", "usize", "{integer}", "integer", "u8", "i8", "u32"}
COLLECTIONS = {"OrdSet", "HashMap", "BTreeSet", "BTreeMap", "Vec", "Vector", "HashSet",
               "PointwisePower", "PartialFunction", "FreeMonoid"}

def delta_vector(e, f):
    eb, en = strip_rc(e); fb, fn = strip_rc(f)
    v = []
    if en != fn: v.append("rc_depth")
    if head(eb) != head(fb): v.append("nominal")
    elif eb != fb: v.append("generic_arg")
    if head(eb) in COLLECTIONS or head(fb) in COLLECTIONS: v.append("container")
    if e.startswith("&") or f.startswith("&"): v.append("borrowed")
    # ANY-DEPTH, not head-only: an Option nested inside Outcome<_> never set this flag, which is
    # how three non-wrap sites reached the old R1 arm looking like bare wrap deltas.
    if "Option<" in e or "Option<" in f or f == "None" or e == "None": v.append("optionality")
    return "+".join(v) if v else "none"


# ---- keying, declared rather than emergent ---------------------------------
# THE ARMS USE THREE KEYING SCHEMES AND THE PRECEDENCE BETWEEN THEM IS DECLARED HERE.
#   delta-keyed   : fires on the SHAPE OF THE DIFFERENCE  (R1, A-clone, BOX-WRAP, ELEM-COLL, D)
#   carrier-keyed : fires on the head type's identity     (B3, B2, T2, T3, DIAG, W, C, R2)
#   message-keyed : fires on rustc's own text             (ARG-ORDER)
# A site can satisfy a delta-keyed and a carrier-keyed definition at once -- Witness<Rc<X>> vs
# Witness<X> is both a Witness fork and a pure Rc wrap -- so SOME precedence always decides it.
# Before this block that precedence was source order: every carrier arm sat above every delta
# arm, so the carrier silently won, and the partition still summed to the site total. A sum is
# exactly the check that cannot see a keying inconsistency (found by smart-ram-730 and
# royal-dove-436 running the rule in the direction neither the author nor the first reporter ran:
# which NON-R1 rows PASS the R1 test).
#
# THE RULING: an EXACT delta test outranks a carrier test. If erasing every Rc from both sides
# makes them equal, the two sides AGREE about the carrier and disagree only about wrapping, so
# the faulty decision is the wrap decision -- naming the carrier there would name a fact both
# sides already share. Carrier arms therefore keep only sites where the carrier itself differs.
# The one exact delta test that can collide with a carrier arm is R1's, so R1 is hoisted above
# the carrier arms; the other delta-keyed arms are not exact in this sense and keep their place.
def rc_erased(t):
    """Erase every Rc<...> wrapper, at any depth. Balanced-bracket, not a regex: a regex on
    [^<>]* leaves Rc<Refined<Artifact>> untouched and would report a real R1 site as unequal."""
    out, i = [], 0
    while i < len(t):
        if t.startswith("Rc<", i):
            d, k = 0, i + 2
            while k < len(t):
                if t[k] == "<":
                    d += 1
                elif t[k] == ">":
                    d -= 1
                    if d == 0:
                        break
                k += 1
            out.append(rc_erased(t[i + 3:k]))
            i = k + 1
        else:
            out.append(t[i])
            i += 1
    return "".join(out)

assert rc_erased("Rc<Refined<Artifact>>") == "Refined<Artifact>"
assert rc_erased("Rc<Vector<Rc<Token>>>") == "Vector<Token>"
assert rc_erased("Measure<(), S, Rc<i64>>") == "Measure<(), S, i64>"
assert rc_erased("String") == "String"


def split_args(t):
    """Top-level generic argument list of `Head<a, b, c>`, or None."""
    i = t.find("<")
    if i < 0 or not t.endswith(">"):
        return None
    inner, args, d, cur = t[i + 1:-1], [], 0, []
    for ch in inner:
        if ch == "<":
            d += 1
        elif ch == ">":
            d -= 1
        if ch == "," and d == 0:
            args.append("".join(cur).strip()); cur = []
        else:
            cur.append(ch)
    args.append("".join(cur).strip())
    return args

def difference_context(e, f, path=None):
    """The constructor path descended THROUGH to reach the difference, e.g.
    `Witness<Rc<Node>>` vs `Witness<Rc<RuntimeValueAcceptanceWitness>>` gives ['Witness']."""
    path = [] if path is None else path
    ea, fa = split_args(e), split_args(f)
    if ea is None or fa is None or head(e) != head(f) or len(ea) != len(fa):
        return path
    diffs = [i for i in range(len(ea)) if ea[i] != fa[i]]
    if len(diffs) != 1:
        return path
    return difference_context(ea[diffs[0]], fa[diffs[0]], path + [head(e)])

def innermost_difference(e, f):
    """Descend while the two sides agree, and return the first position at which they differ.
    A CARRIER arm must be keyed on the carrier AT THE DIFFERENCE, not at the head: the head of
    `Outcome<Option<Node>>` vs `Outcome<Node>` is `Outcome` on both sides and says nothing about
    the disagreement, which is an Option presence one level down. Head-only carrier tests are
    why three such sites fell past every carrier arm into a same-head catch-all."""
    ea, fa = split_args(e), split_args(f)
    if ea is None or fa is None or head(e) != head(f) or len(ea) != len(fa):
        return e, f
    diffs = [i for i in range(len(ea)) if ea[i] != fa[i]]
    if len(diffs) != 1:
        return e, f
    return innermost_difference(ea[diffs[0]], fa[diffs[0]])


# ---- carrier flags: evidence BESIDE the root, never a category --------------
# A cluster is keyed on what the DIFFERENCE is. Some facts that matter for costing are properties
# of the emitted DECLARATION instead, so they are not decidable from the pair at all and must not
# become arms -- an arm keyed on a cause the classifier cannot see is a guess wearing a category's
# name. They are carried as their own column, joinable across clusters, exactly as the callee note
# is: a site can then be pooled by cause without the partition being re-keyed by it.
#
# The live instance (reported by royal-dove-436, 2026-08-22): `std.measure`
# `billing_month_as_hour_count_representation_note` records that stage0 alias emission collapses
# applied-generic Measure aliases to a concrete all-unit parameter list while fn/data return sites
# still reference the un-erased alias params. That collapse shows up in the SPELLING of both sides,
# so the flag is mechanical; which producer emitted the declaration is not visible here.
def carrier_flags(e, f):
    """Mechanical properties of the SPELLINGS, keyed on generic-argument positions."""
    flags = []
    ea, fa = split_args(strip_rc(e)[0]), split_args(strip_rc(f)[0])
    if ea and fa and head(e) == head(f) and len(ea) == len(fa):
        pos = list(zip(ea, fa))
        if any((a == "()") != (b == "()") for a, b in pos):
            flags.append("generic_param_unit_on_one_side")
        elif any(a == "()" and b == "()" for a, b in pos):
            # Both sides collapsed: the erasure is real but INVISIBLE IN THE DELTA, so no arm
            # could ever key on it. This is the flag's whole reason for existing.
            flags.append("generic_params_unit_on_both_sides")
        if any((a == "_") != (b == "_") for a, b in pos):
            flags.append("generic_param_binding_differs")
    return "+".join(flags)

# ---- discriminators (prior 15-root vocabulary) ------------------------------
def classify(s):
    e, f = s["expected"], s["found"]
    if s["nopair"]:
        if s["reorder"] or "reorder" in s["msg"]:
            return "ARG-ORDER", "call_argument_order_diverges_from_declaration"
        return "RESIDUE", "block_carries_no_expected_found_pair"
    eb, en = strip_rc(e); fb, fn = strip_rc(f)
    he, hf = head(eb), head(fb)
    if s["reorder"]:
        return "ARG-ORDER", "call_argument_order_diverges_from_declaration"
    # R1, hoisted above every carrier arm by the ruling above: the ONLY difference is Rc wrapping.
    if e != f and rc_erased(e) == rc_erased(f):
        return "R1", ("rc_wrap_only" if en != fn else "rc_wrap_at_type_argument_depth")
    # Carrier arms below key on the carrier AT THE DIFFERENCE. The outer pair is still what the
    # site reports; only the discriminator descends.
    de, df = innermost_difference(e, f)
    if (de, df) != (e, f):
        eb, en = strip_rc(de); fb, fn = strip_rc(df)
        he, hf = head(eb), head(fb)
    # A-clone: generic parameter cloned through a reference
    if s["kind_e"] == "type parameter" or (eb.isidentifier() and f == "&" + e):
        if f.startswith("&") and f.lstrip("&") == e:
            return "A-clone", "generic_param_clone_bound_absent"
    # B3 modeled Nat vs native integer
    if {he, hf} & {"Nat"} and (hf in NUMERIC_NATIVE or he in NUMERIC_NATIVE
                               or hf == "integer" or he == "integer"):
        return "B3", "modeled_numeric_vs_native"
    # B2 Bool carrier vs native/variant
    if {he, hf} & {"Bool"} and (hf in {"bool", "True", "False"} or he in {"bool", "True", "False"}):
        return "B2", "bool_carrier_vs_native_or_variant"
    # T2 text carrier vs String
    if (he in TEXT and hf in {"Vector", "FreeMonoid"}) or (hf in TEXT and he in {"Vector", "FreeMonoid"}):
        return "T2", "text_carrier_vs_string"
    # T3 collection carrier fork
    if he in COLLECTIONS and hf in COLLECTIONS and he != hf:
        return "T3", "collection_carrier_fork"
    # DIAG diagnostic carrier fork
    if {he, hf} & {"Diagnostic", "NonEmptyDiagnostics", "Diagnostics"} and he != hf:
        return "DIAG", "diagnostic_carrier_fork"
    # C carrier collapses to unit
    if eb == "()" or fb == "()" or s["kind_f"] == "unit type" or s["kind_e"] == "unit type":
        return "C", "carrier_collapses_to_unit"
    # R2 optional surface fork
    if he == "Option" or hf == "Option" or f == "None" or e == "None":
        return "R2", "optional_surface_fork"
    # BOX-WRAP: same nominal, one side Box-wrapped. Named separately from R1 because the
    # wrapper is a different one and a wrap-decision fix keyed on Rc would not reach it.
    if e == "Box<%s>" % f or f == "Box<%s>" % e:
        return "BOX-WRAP", "box_wrap_only"
    # ELEM-COLL: one side is the ELEMENT, the other its own collection. Mechanically
    # distinct from D (no alias, no arity change on one nominal) and from T3 (one carrier,
    # not two forked carriers), so it gets its own arm rather than the nearest familiar one.
    def elem_of(container_t, other_head):
        m = re.match(r"[A-Za-z_][A-Za-z0-9_]*<(.+)>$", container_t)
        if not m: return False
        args = m.group(1)
        return head(strip_rc(args.split(",")[-1].strip())[0]) == other_head
    if he in COLLECTIONS and hf not in COLLECTIONS and elem_of(eb, hf):
        return "ELEM-COLL", "element_vs_its_own_collection"
    if hf in COLLECTIONS and he not in COLLECTIONS and elem_of(fb, he):
        return "ELEM-COLL", "element_vs_its_own_collection"
    # W is CONTEXT-keyed, not carrier-keyed: the prior vocabulary's row is "Witness<_> type
    # argument", i.e. the two sides agree that the value is a Witness and disagree about what it
    # witnesses. It sits below the carrier arms by the declared precedence -- a difference that
    # is itself a carrier fork (a unit collapse, a text carrier) is named by the fork, not by the
    # constructor it happens to sit inside.
    if "Witness" in difference_context(e, f):
        return "W", "witness_type_argument"
    # D alias arity / generic argument count: one side IS the other's generic argument,
    # or the two differ only by an elided argument list
    if he != hf and (re.search(r"[<,]\s*(?:Rc<)?%s[>,]" % re.escape(hf), eb)
                     or re.search(r"[<,]\s*(?:Rc<)?%s[>,]" % re.escape(he), fb)):
        return "D", "alias_arity_generic_argument_count"
    if he != hf and ("..." in e or "..." in f):
        return "D", "alias_arity_generic_argument_count"
    # R5 duplicate type authority (same leaf spelling under two paths / near-identical nominals)
    if he != hf and (he in hf or hf in he):
        return "R5", "duplicate_type_authority"
    # The old R1 tail arms are GONE, not moved. `("Rc<" in eb or "Rc<" in fb)` admitted a site
    # whenever the substring occurred anywhere on either side, which is not evidence that the
    # DELTA is an Rc wrap: it swept in Outcome<Option<Node>> vs Outcome<Node> (an Option
    # presence), Vector<()> vs Vector<ComplexityLowering> (a unit collapse) and
    # Measure<(),S,i64> vs Measure<(),_,i64> (a type-parameter binding). The exact test above
    # replaces it; anything reaching here differs by more than wrapping.
    if he == hf and eb != fb:
        return "D", "alias_arity_generic_argument_count"
    return "RESIDUE", "unclassified_pair: %s | %s" % (e, f)

def main(path, out):
    blocks = read_blocks(path)
    sites, seen = [], set()
    for b in blocks:
        for s in block_sites(b):
            key = (s["file"], s["line"], s["col"],
                   frozenset((s["expected"], s["found"])))
            if key in seen: continue
            seen.add(key)
            root, reason = classify(s)
            s["root"], s["reason"] = root, reason
            s["delta"] = "" if s["nopair"] else delta_vector(s["expected"], s["found"])
            s["carrier_flags"] = "" if s["nopair"] else carrier_flags(s["expected"], s["found"])
            sites.append(s)
    cols = ["file", "line", "col", "expected", "found", "delta", "root", "reason",
            "carrier_flags", "callee", "callee_kind", "block_msg"]
    with open(out, "w") as fh:
        fh.write("\t".join(cols) + "\n")
        for s in sites:
            fh.write("\t".join(str(s.get(c, "")) for c in
                     ["file", "line", "col", "expected", "found", "delta", "root", "reason",
                      "carrier_flags", "callee", "callee_kind", "msg"]) + "\n")
    print("raw E0308 blocks: %d" % len(blocks))
    print("mismatch projections: %d" % len(sites))
    hist = collections.Counter(s["root"] for s in sites)
    # Counts only. This classifier sees one error code in one overlapping M=1 closure at one
    # revision. A percentage would mistake this code-local projection for a closed mechanism
    # population and make it look comparable with another code, closure, or revision.
    for r, n in hist.most_common():
        print("  %-12s %4d" % (r, n))
    flagged = collections.Counter(s["carrier_flags"] for s in sites if s.get("carrier_flags"))
    if flagged:
        print("\nCarrier flags (evidence beside the root, not a category):")
        for fl, n in flagged.most_common():
            rs = collections.Counter(s["root"] for s in sites if s.get("carrier_flags") == fl)
            print("  %-46s %3d  across %s" % (fl, n, dict(rs)))
    print("\nRESIDUE detail:")
    for s in sites:
        if s["root"] == "RESIDUE":
            print("  %s:%s:%s  %s | %s  [%s] callee=%s" %
                  (s["file"], s["line"], s["col"], s["expected"], s["found"], s["delta"], s["callee"]))

main(sys.argv[1], sys.argv[2])
