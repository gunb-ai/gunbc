#!/usr/bin/env python3
"""Parse REBINDPROBE lines from the instrumented self-host log and answer the two
questions this probe exists for, BY IDENTITY, without re-running anything.

  Q1 (parent msg_6b7b53a9, Bucket B): for rows whose FORMAL prints Coproduct(<head>)
     and whose ACTUAL prints Container(<head>, X) / Node(<head><...>), does the
     Coproduct side carry CONCRETE children equal to X (a PRINTING difference ->
     belongs to normalization at the type authority) or unsubstituted SLOT leaves
     (INSTANTIATION -> waits on quiet-eagle-395's S1)?

  Q2 (X1, SAME_TEXT 11): two nodes that PRINT identically via node_type_shape but
     refuse in node_type_equals_core. Is the difference in `decl=` (declaration
     identity: one none, the other resolved -- or different module_path) or in
     `infer=` or in the connective/children? node_type_shape consults `declaration`
     ONLY in the children-bearing fallthrough (04_types.dag:918-971), so a pair that
     prints the same can still differ in declaration identity.

Usage: parse_rebindprobe.py <sh.log> [more logs...]
"""
import sys, re, json, collections

FIELDS = ['name','identSpan','nodeSpan','connective','cardinality','nprops','nchildren',
          'decl','infer','shape']

def split_top(text, seps):
    """Split on `seps` only at paren depth 0. Depth-aware because printed InferredNode and
    printed shapes both carry balanced parens, and source spans carry slashes."""
    parts, buf, depth = [], [], 0
    for ch in text:
        if ch == '(':
            depth += 1
        elif ch == ')':
            depth = max(0, depth - 1)
        if ch in seps and depth == 0:
            parts.append(''.join(buf)); buf = []
        else:
            buf.append(ch)
    parts.append(''.join(buf))
    return parts

def parse_side(txt):
    """Parse one side emitted by rebind_probe_side:
       name=..|identSpan=..|nodeSpan=..|connective=..|cardinality=..|nprops=N|nchildren=N|decl=..|infer=..|shape=..|children=[c;c;..]
    Split on '|' at depth 0 (a CompilerError message may itself contain '|'), then peel the
    'k=' prefixes. Anything that does not line up is returned with '_unparsed' rather than
    being silently guessed at -- a mis-parsed field is worse than an unparseable row."""
    body = txt
    kids = ''
    m = re.search(r'\|children=\[(.*)\]\s*$', body, re.S)
    if m:
        kids = m.group(1); body = body[:m.start()]
    parts = split_top(body, '|')
    d = {}
    for idx, f in enumerate(FIELDS):
        if idx >= len(parts):
            return {'_unparsed': txt[:400]}
        pre = f + '='
        if not parts[idx].startswith(pre):
            return {'_unparsed': txt[:400]}
        d[f] = parts[idx][len(pre):]
    d['nprops'] = int(d['nprops']) if d['nprops'].isdigit() else d['nprops']
    d['nchildren'] = int(d['nchildren']) if d['nchildren'].isdigit() else d['nchildren']
    d['children'] = []
    kids = kids.strip()
    if kids:
        # ';' is a child separator; rebind_probe_side emits no ';' inside a child, and children
        # are printed flat, so a depth-aware split is still the right shape of split to use.
        for k in split_top(kids, ';'):
            d['children'].append(split_child(k))
    return d

# --- child field split -------------------------------------------------------
# rebind_probe_side emits a child as `name/identSpan/infer/shape`. A plain split('/') is WRONG:
# identSpan is a real source span (dag/std/algebra.dag:74075-74078) containing slashes, and the
# CompilerError form of `infer` embeds a span too -- a naive split mis-assigns every field, and it
# silently reported a Bucket-B child as name=Char, infer='text.dag:1-4/Resolved(Char)/Char'.
# Two ambiguities must be resolved, both by ANCHORING ON THE EMITTER'S GRAMMAR rather than by luck:
#   * identSpan and infer can BOTH be the literal "none" -> take the first candidate index whose
#     ident is well formed, where the empty ident (candidate i == 1) is not.
#   * shape may itself contain a top-level slash -> the tail after `infer` is rejoined verbatim.
INFER_STARTS = ('Resolved(', 'CompilerError(', 'TypeVariable(')
SPAN_RE = re.compile(r'^[^()]*:\d+-\d+$')

def infer_is_wellformed(t):
    if t in ('none', 'Divergent'):
        return True
    if t.startswith(INFER_STARTS):
        return t.count('(') == t.count(')') and t.endswith(')')
    return False

def ident_is_wellformed(t):
    return t == 'none' or bool(SPAN_RE.match(t))

def split_child(k):
    parts = split_top(k, '/')
    if len(parts) < 4:
        return {'_raw': k}
    name = parts[0]
    for i in range(2, len(parts)):            # i >= 2: ident is at least one part, never empty
        ident = '/'.join(parts[1:i])
        if not ident_is_wellformed(ident):
            continue
        # consume balanced parens so an infer whose message contains '/' is not cut
        depth, j = 0, i
        while j < len(parts):
            depth += parts[j].count('(') - parts[j].count(')')
            j += 1
            if depth <= 0:
                break
        infer = '/'.join(parts[i:j])
        if not infer_is_wellformed(infer) or j >= len(parts):
            continue
        return {'name': name, 'identSpan': ident, 'infer': infer,
                'shape': '/'.join(parts[j:])}
    return {'_raw': k}

def head_of(side):
    """The TYPE head, for Bucket-B pairing.

    NOT the first word of the printed shape: node_type_shape prints a Disj as
    Coproduct(<name>) (04_types.dag:918-971), so the first word of a Coproduct
    spelling's shape is the CONNECTIVE ('Coproduct'), while the applied side prints
    Container(FreeMonoid, Char) whose first word is 'Container'. Comparing shape heads
    pairs nothing and reports Bucket B EMPTY -- a false negative in the very measurement
    this instrument exists to take. The census keyed on the node `name`
    (F.name=FreeMonoid vs A.name=FreeMonoid), which is what the two spellings share.
    """
    if isinstance(side, dict):
        n = (side.get('name') or '').strip()
        if n:
            return n
        sh = side.get('shape') or ''
    else:
        sh = side or ''
    m = re.match(r'^([A-Za-z_][A-Za-z0-9_]*)', sh)
    return m.group(1) if m else sh

def is_slot(child):
    """An unsubstituted generic slot that still carries its TypeVariable marker."""
    return child.get('infer', '').startswith('TypeVariable')

def is_unresolved_bare_leaf(child):
    """A bare, unapplied leaf whose type is not settled: infer is none or a CompilerError.

    These must NOT be counted as concrete children. A generic slot reaches the fold either still
    marked TypeVariable, or (after quiet-eagle-395's resolved_type fallback, the population that
    S1 exists to fix) as name='' / error_type with NO marker at all. Both are unsubstituted slots
    that wait on S1 -- and a slot whose marker was already destroyed by error_type prints exactly
    like a concrete leaf, so trusting the absence of TypeVariable here would route S1-gated rows to
    'printing difference -> the type authority' and hand the owner the wrong list. Anything in this
    bucket is UNDECIDED by this probe, not evidence of a printing difference.
    """
    sh = (child.get('shape') or '').strip()
    if not sh or '(' in sh:
        return False                      # applied / structured: not a bare leaf
    inf = child.get('infer') or 'none'
    return inf == 'none' or inf.startswith('CompilerError')

def is_error_leaf(child):
    return child.get('infer', '').startswith('CompilerError') or child.get('name', '') == ''

def main(paths):
    rows = []
    for p in paths:
        for line in open(p, errors='replace'):
            if 'REBINDPROBE|bind=' not in line:
                continue
            body = line.split('REBINDPROBE|bind=', 1)[1]
            m = re.match(r'^(.*?)\|module=(.*?)\|F=\[(.*)\]\|A=\[(.*)\]\|FM=\[(.*)\]\s*$', body, re.S)
            if not m:
                continue
            bind, module, F, A, FM = m.groups()
            rows.append({'bind': bind, 'module': module,
                         'bound': parse_side(F), 'actual': parse_side(A),
                         'formal': parse_side(FM)})
    print("probe lines parsed: %d" % len(rows))
    if not rows:
        print("NOTHING PARSED -- the seed did not carry the probe, or the run died before it.")
        return 1

    def ident(r):
        return (r['module'], r['bound'].get('nodeSpan',''), r['actual'].get('nodeSpan',''), r['bind'])

    uniq = {}
    for r in rows:
        uniq.setdefault(ident(r), r)
    print("distinct (module,boundSpan,actualSpan,bind): %d" % len(uniq))

    print("\n=== Q1: Bucket B -- Coproduct(<h>) vs an applied spelling of the same head ===")
    bucketB, printing, instantiation, other, undecided = [], 0, 0, 0, 0
    for k, r in uniq.items():
        f, a = r['bound'], r['actual']
        if f.get('connective','').startswith('Disj') and head_of(f) == head_of(a) \
           and not a.get('connective','').startswith('Disj'):
            bucketB.append((k, r))
            kids = f.get('children', [])
            # An UNPARSEABLE child must never be counted as 'concrete': the only difference between
            # "printing difference -> normalization at the type authority" and "instantiation -> waits
            # on S1" is whether the children are slots, and a mis-parsed child reads as non-slot,
            # which would silently route the row to the wrong owner. Unknown is its own bucket.
            if any('_raw' in c or '_unparsed' in c for c in kids) or f.get('_unparsed') or a.get('_unparsed'):
                other += 1
            elif any(is_slot(c) for c in kids):
                instantiation += 1
            elif any(is_unresolved_bare_leaf(c) for c in kids):
                undecided += 1
            elif kids:
                printing += 1
            else:
                other += 1
    print("  candidates=%d  concreteChildren(PRINTING)=%d  slotLeaves(INSTANTIATION)=%d  bareLeafUnsettled(UNDECIDED->S1)=%d  unparseable-or-childless=%d"
          % (len(bucketB), printing, instantiation, undecided, other))
    if printing == 0 and instantiation == 0 and undecided == 0 and len(bucketB) > 0:
        print('  !! NOTHING CLASSIFIED -- suspect the child parser, not the data.')
    for k, r in bucketB[:40]:
        f, a = r['bound'], r['actual']
        print("  %s %s\n     F shape=%s conn=%s nchildren=%d decl=%s infer=%s\n     A shape=%s conn=%s nchildren=%d decl=%s infer=%s\n     F children=%s"
              % (k[0], k[3], f.get('shape'), f.get('connective'), f.get('nchildren'), f.get('decl'), f.get('infer'),
                 a.get('shape'), a.get('connective'), a.get('nchildren'), a.get('decl'), a.get('infer'),
                 [ (c.get('name'), c.get('infer'), c.get('shape')) for c in f.get('children', []) ][:6]))

    print("\n=== Q2: SAME_TEXT -- identical printed shape, still refused ===")
    same = [(k, r) for k, r in uniq.items() if r['bound'].get('shape') == r['actual'].get('shape')]
    print("  identical-print pairs=%d" % len(same))
    tally = collections.Counter()
    for k, r in same:
        f, a = r['bound'], r['actual']
        why = []
        if f.get('decl') != a.get('decl'): why.append('decl')
        if f.get('infer') != a.get('infer'): why.append('infer')
        if f.get('connective') != a.get('connective'): why.append('connective')
        if f.get('nchildren') != a.get('nchildren'): why.append('nchildren')
        if f.get('return_cardinality' if 'return_cardinality' in f else 'cardinality') != a.get('cardinality'): why.append('card')
        tally['+'.join(why) or 'IDENTICAL IN EVERY PROBED FIELD'] += 1
        print("  %-52s %-28s why=%s" % (k[0] + ':' + str(k[1]), (f.get('shape') or '')[:28], '+'.join(why) or 'NONE-OF-THE-PROBED-FIELDS'))
        print("      F decl=%s infer=%s | A decl=%s infer=%s" % (f.get('decl'), f.get('infer'), a.get('decl'), a.get('infer')))
    print("  tally:", dict(tally))

    print("\n=== declaration state over ALL rows (the X1 producer question) ===")
    st = collections.Counter()
    for k, r in uniq.items():
        for side in ('bound','actual','formal'):
            d = r[side].get('decl')
            st['%s.decl=%s' % (side, 'none' if d in (None,'none') else 'present')] += 1
    print(" ", dict(st))
    print("  rows where BOTH sides carry no declaration: %d" % sum(
        1 for k,r in uniq.items() if r['bound'].get('decl') in (None,'none') and r['actual'].get('decl') in (None,'none')))
    print("\n=== per-class counts (rebind refusals, by the census class the row would take) ===")
    cc = collections.Counter()
    for k, r in uniq.items():
        f, a = r['bound'], r['actual']
        if f.get('shape') == a.get('shape'): cc['SAME_TEXT'] += 1
        elif f.get('connective','').startswith('Disj') and head_of(f) == head_of(a): cc['BUCKET_B'] += 1
        elif head_of(f) == head_of(a): cc['SAME_HEAD'] += 1
        elif 'FormalTypeAnnotation' in (f.get('infer') or '') or f.get('infer')=='none': cc['FORMAL_VS_CONCRETE'] += 1
        else: cc['OTHER'] += 1
    print(" ", dict(cc))
    json.dump({str(k): v for k, v in uniq.items()}, open('/tmp/rebindprobe_rows.json','w'), indent=1)
    print("\nrows written: /tmp/rebindprobe_rows.json")
    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
