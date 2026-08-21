# INDEPENDENT reference for FNV1A64-STRUCTURAL-NODE-PROTOCOL-0, written from the
# declared rule in v2.std.node, NOT by transcribing the authority's output.
OFF, PRIME, MASK = 0xcbf29ce484222325, 0x100000001b3, (1 << 64) - 1
def fnv(bs):
    h = OFF
    for b in bs: h = ((h ^ b) * PRIME) & MASK
    return h
def atom(s):    return "%016x" % fnv(s.encode())
def comb(a, b): return "%016x" % fnv(a.encode() + b"\x00" + b.encode())

# ---- nodes as (kind, children); kind = ('T', conn, sym?) | ('C', behavior)
def N(kind, children=()): return (kind, list(children))
def E(label, target):     return (label, target)   # label = ('N', sym) | ('P',)

def conn_hash(k):
    _, c, sym = k
    return comb(atom("canonical_tag_atom"), atom(sym)) if c == "Atom" else atom("canonical_tag_" + c.lower())
def kind_hash(k):
    if k[0] == 'T': return comb(atom("canonical_tag_type_node"), conn_hash(k))
    return comb(atom("canonical_tag_computation_node"), atom("canonical_tag_" + k[1].lower()))
def label_hash(l):
    return comb(atom("canonical_tag_named_edge"), atom(l[1])) if l[0] == 'N' else atom("canonical_tag_positional_edge")
def sort_key(e):
    return e[0][1] if e[0][0] == 'N' else "canonical_tag_positional_edge"

def discipline(kind, children):
    if kind[0] == 'T':
        c = kind[1]
        if c == "Atom":         return "NoEdges" if len(children) == 0 else "LabeledEdges"
        if c in ("Conj","Disj"):return "LabeledEdges"
        if c == "Arrow":        return "ArrowBodyEdges"
        return "PositionalEdges"                      # Cardinality, Instantiation
    b = kind[1]
    if b == "Value": return "NoEdges"
    if b == "Loop":  return "LoopBoundEdges"
    return "PositionalEdges"                          # Transform, Branch, Bind, Match

def split_special(children, special):
    pos   = [e for e in children if e[0][0] == 'P']
    other = sorted([e for e in children if e[0][0] == 'N' and e[0][1] != special], key=sort_key)
    spec  = [e for e in children if e[0][0] == 'N' and e[0][1] == special]
    return pos + other + spec

def canonical_children(kind, children):
    d = discipline(kind, children)
    if d == "LabeledEdges":   return sorted(children, key=sort_key)
    if d == "ArrowBodyEdges": return split_special(children, "arrow_body_edge")
    if d == "LoopBoundEdges": return split_special(children, "loop_bound_edge")
    return list(children)                             # NoEdges, PositionalEdges

def content_hash(node):
    kind, children = node
    acc = kind_hash(kind)
    for e in canonical_children(kind, children):
        acc = comb(comb(acc, label_hash(e[0])), content_hash(e[1]))
    return acc

# ---- fixtures
A  = lambda s: N(('T','Atom',s))
TY = lambda c: N(('T',c,None))
CO = lambda b: N(('C',b,None)) if False else N(('C',b))
def CN(b, ch=()): return N(('C',b), ch)
nf, ng = ('N','f'), ('N','g')
P = ('P',)

# ---- the receipt: re-derive every frozen digest and JOIN it against the literals the
# witness actually carries. This file is the answer to "independent according to whom?" --
# it recomputes the protocol from its stated rule, in a language that shares nothing with
# the substrate, and then diffs. Running it is the evidence; the prose beside it is not.
#
# WHAT THIS CLOSES, precisely. The corpus's mutation receipt proves the witness DETECTS a
# broken rule. It cannot prove where the constants came from: a literal captured from
# content_hash and then frozen is static too, so it goes red under exactly the same
# mutations. Sensitivity and origin are different properties. This file reaches the second.
import re, sys, pathlib

EXPECTED = {
    "conj":         content_hash(TY("Conj")),
    "disj":         content_hash(TY("Disj")),
    "arrow":        content_hash(TY("Arrow")),
    "cardinality":  content_hash(TY("Cardinality")),
    "instantiation":content_hash(TY("Instantiation")),
    "atom":         content_hash(A("alpha")),
    "value":        content_hash(CN("Value")),
    "transform":    content_hash(CN("Transform")),
    "branch":       content_hash(CN("Branch")),
    "loop":         content_hash(CN("Loop")),
    "bind":         content_hash(CN("Bind")),
    "match":        content_hash(CN("Match")),
    "labeled_conj_fg":  content_hash(N(("T","Conj",None), [E(nf,A("alpha")), E(ng,A("beta"))])),
    "positional_card_ab": content_hash(N(("T","Cardinality",None), [E(P,A("alpha")), E(P,A("beta"))])),
    "arrow_body":   content_hash(N(("T","Arrow",None), [E(("N","arrow_body_edge"),A("body")), E(ng,A("beta")), E(P,A("sig")), E(nf,A("alpha"))])),
    "loop_bound":   content_hash(CN("Loop", [E(("N","loop_bound_edge"),A("bound")), E(ng,A("beta")), E(P,A("body")), E(nf,A("alpha"))])),
    "nested":       content_hash(N(("T","Conj",None), [E(nf, N(("T","Conj",None), [E(ng, A("deep"))]))])),
}

WITNESS = pathlib.Path(__file__).resolve().parents[3] / "dag/test/claim/node_hash_protocol_witness_test.dag"

def main():
    if not WITNESS.exists():
        print("REFERENCE: witness not found at %s" % WITNESS); return 2
    literals = set(re.findall(r'"([0-9a-f]{16})"', WITNESS.read_text()))
    if not literals:
        # An empty extraction must never read as agreement -- it is the absence of an
        # observation, not a passing one.
        print("REFERENCE: FAIL extracted 0 digest literals from the witness"); return 2
    derived = set(EXPECTED.values())
    missing = derived - literals
    for name, d in sorted(EXPECTED.items()):
        print("  %-20s %s %s" % (name, d, "OK" if d in literals else "NOT-IN-WITNESS"))
    print("derived=%d witness_literals=%d derived_absent_from_witness=%d"
          % (len(derived), len(literals), len(missing)))
    if missing:
        print("REFERENCE: FAIL -- the witness does not carry these independently derived digests:")
        for d in sorted(missing): print("   %s" % d)
        return 1
    print("REFERENCE: OK -- every digest this file derives from the spec appears in the witness")
    return 0

if __name__ == "__main__":
    sys.exit(main())
