#!/usr/bin/env python3
"""Read REBINDV3 lines and return the PRE-REGISTERED routing table.

Registered BEFORE the run (stern-bat-401 msg_43674e26, option (ii)), so no cause is fitted to the data:
  H  the parent's hypothesis for the conn-127 class: the BOUND side IS the callee generic's own
     DECLARATION, i.e. every leaf name on the bound side appears among that side's own declared
     parameter names (lleafinown=true). Prediction: most first-difference==conn rows show true.
     ACCEPT CONTROL: two DISTINCT instances of one generic (FreeMonoid<Int> vs FreeMonoid<Bool>) keep
     refusing with lleafinown=false. If the control row shows true, the predicate is too loose and the
     extension must not be built.
  R  refusal withholds when the judgment is removed (core_na=true): the judgment is consumed by
     peel_alias_rhs_once, core's first step -- the lead quiet-eagle-395 handed over with the identity
     v2.std.integer / bind=T / Coproduct(StandardIntegerType).
  S  S1 relevance: CompilerError (error_type) nodes at UNBOUNDED depth. v2 could only speak within
     fuel 3. If lce/rce stay 0 at fuel 40 with pointer dedup, S1 does not touch this lane.
  C  rows identical in every aggregate: the unexplained residue, reported by identity, cause unpicked.

Instrument honesty rules enforced in the parser itself:
 - caps are REPORTED. Any row with lcaps/rcaps>0 is excluded from the "whole tree" claims and counted
   separately; a bounded census must never be summarised as an unbounded one (that is how I got the
   52-producers and 7-fabrications claims wrong).
 - ',..TRUNC' in a leaf/param field makes lleafinown UNTRUSTWORTHY for that row: the truncated tail could
   have contained a leaf that is not a parameter. Truncation is reported, never silently averaged.
 - leaves_are_own_params is false for an empty leaf set in the EMITTER; the parser re-checks and labels
   the empty-leaf rows separately so no row vacuously poses as a control.
"""
import sys, collections, json

# V3's LEAF FIELD IS DEAD, and this is why the join in h_hypothesis_join.py exists.
# rebind_probe_walk called a leaf "nc == 0 AND the printed shape contains no '('". But node_type_shape
# prints a leaf AS Primitive(Symbol) -- with parens -- so the predicate matched nothing and EVERY row
# reported lleafn=rleafn=0 (254/254). The parser therefore excludes truncated/capped rows and labels
# empty-leaf rows, and the HYPOTHESIS WAS ANSWERED FROM v2 INSTEAD: v2 already printed, per node, the
# child count and the name, so joining v2's per-node tree to v3's params by
# (module, bind, lshape, rshape) -- 254/254 rows joined, 0 unmatched -- recovers the leaf multisets at
# zero dispatch cost. Lesson kept in the open: when a field reads 0 on EVERY row, suspect the instrument
# before believing the data; a uniformly-empty field is the signature of a broken predicate, not of a
# clean corpus.

def parse(line):
    b = line[len('REBINDV3|'):].rstrip('\n')
    d = {}
    for kv in b.split('|'):
        if '=' in kv:
            k, v = kv.split('=', 1)
            d[k] = v
    return d

def num(d, k):
    try: return int(d.get(k, '0'))
    except ValueError: return -1

def main(paths):
    rows = []
    for p in paths:
        for line in open(p, errors='replace'):
            if line.startswith('REBINDV3|'):
                d = parse(line); d['_src'] = p
                rows.append(d)
    print("REBINDV3 rows: %d" % len(rows))
    if not rows:
        print("NOTHING PARSED -> the probe never reached the site. 0 rows != 0 refusals.")
        return 1

    capped = [r for r in rows if num(r,'lcaps') > 0 or num(r,'rcaps') > 0]
    trunc  = [r for r in rows if ',..TRUNC' in (r.get('lleaf','') + r.get('rleaf','') + r.get('lparams','') + r.get('rparams',''))]
    empty  = [r for r in rows if num(r,'lleafn') == 0 or num(r,'rleafn') == 0]
    whole  = [r for r in rows if r not in capped and r not in trunc]

    print("\n=== validity / honesty (read this before any tally) ===")
    print("  rows where the depth fuel STILL fired (excluded from whole-tree claims): %d" % len(capped))
    print("  rows with a truncated leaf/param field (lleafinown untrustworthy):       %d" % len(trunc))
    print("  rows with an EMPTY leaf set on a side (predicate is false-by-construction): %d" % len(empty))
    print("  rows admitted to the whole-tree claims: %d of %d" % (len(whole), len(rows)))
    tot_caps = sum(max(num(r,'lcaps'),0) + max(num(r,'rcaps'),0) for r in rows)
    print("  total cap events: %d  (v2's were 3598 DEPTH + 18 CHILD at fuel 3)" % tot_caps)

    print("\n=== H: the parent's hypothesis (bound side == the generic's own declaration form) ===")
    conn = [r for r in whole if r.get('lconn','').startswith('Disj') and not r.get('rconn','').startswith('Disj')
            and r.get('lname') == r.get('rname')]
    print("  conn-class rows (L is Disj, R is an applied spelling, same head name): %d" % len(conn))
    t = sum(1 for r in conn if r.get('lleafinown') == 'true')
    f = sum(1 for r in conn if r.get('lleafinown') == 'false')
    print("    lleafinown=true  (hypothesis CONFIRMED): %d" % t)
    print("    lleafinown=false (hypothesis REFUTED for these): %d" % f)
    print("  ALL rows by lleafinown: %s" % dict(collections.Counter(r.get('lleafinown') for r in rows)))
    print("  ACCEPT CONTROL: distinct-instance pairs (same head, both applied, DIFFERENT rleaf) that refuse:")
    ctrl = [r for r in whole if r.get('lconn')==r.get('rconn') and r.get('lname')==r.get('rname')
            and r.get('lleaf') != r.get('rleaf')]
    for r in ctrl[:6]:
        print("    module=%s bind=%s lleaf=%s rleaf=%s lleafinown=%s core=%s" % (
            r['module'], r['bind'], r.get('lleaf','')[:40], r.get('rleaf','')[:40], r.get('lleafinown'), r.get('core')))
    print("    any control that ACCEPTS (core=true) would falsify the refusal twin: %d" % sum(
        1 for r in ctrl if r.get('core') == 'true'))

    print("\n=== R: the judgment-consuming row (peel_alias_rhs_once lead) ===")
    flips = [r for r in rows if r.get('core_na') == 'true']
    print("  rows that accept once the judgment is withheld: %d" % len(flips))
    for r in flips[:10]:
        print("    module=%s bind=%s lshape=%s rshape=%s cross=%s text=%s" % (
            r['module'], r['bind'], (r.get('lshape') or '')[:60], (r.get('rshape') or '')[:60],
            r.get('cross'), (r.get('text') or '')[:40]))

    print("\n=== S: error_type at unbounded depth (the S1 critical-path question) ===")
    ce = [r for r in whole if num(r,'lce') > 0 or num(r,'rce') > 0]
    tv = [r for r in whole if num(r,'ltv') > 0 or num(r,'rtv') > 0]
    print("  rows with >=1 CompilerError node: %d of %d whole-tree rows" % (len(ce), len(whole)))
    print("  rows with >=1 still-marked TypeVariable: %d" % len(tv))
    print("  total CompilerError nodes: L=%d R=%d ; TypeVariable nodes: L=%d R=%d" % (
        sum(max(num(r,'lce'),0) for r in rows), sum(max(num(r,'rce'),0) for r in rows),
        sum(max(num(r,'ltv'),0) for r in rows), sum(max(num(r,'rtv'),0) for r in rows)))
    print("  CAVEAT the probe cannot close: the RELATION converts a Present{TypeVariable} child to")
    print("  error_type itself, via child_type_node -> resolved_type (04_types.dag), whose Absent/else")
    print("  arms MINT CompilerError. So lce=0 speaks about the NODE DATA, not about what the relation sees.")

    print("\n=== C: identical in every aggregate, still refused (report by identity, cause unpicked) ===")
    def agg_sig(r):
        return tuple(r.get(k) for k in ('lname','rname','lconn','rconn','lshape','rshape','ldecl','rdecl',
                                        'linfer','rinfer','lnodes','rnodes','lleaf','rleaf','lparams','rparams',
                                        'lleafn','rleafn','ltv','rtv','lce','rce'))
    ident = [r for r in whole if (r.get('lleaf')==r.get('rleaf') and r.get('lshape')==r.get('rshape')
             and r.get('ldecl')==r.get('rdecl') and r.get('linfer')==r.get('rinfer')
             and num(r,'lnodes')==num(r,'rnodes') and num(r,'lce')==num(r,'rce') and num(r,'ltv')==num(r,'rtv'))]
    print("  rows: %d" % len(ident))
    seen=set()
    for r in ident[:20]:
        k=(r['module'],r['bind'],r.get('lshape'))
        if k in seen: continue
        seen.add(k)
        print("    %-40s bind=%-4s %s" % (r['module'], r['bind'], (r.get('lshape') or '')[:100]))

    print("\n=== arm differential (all rows) ===")
    for k,v in collections.Counter((r.get('equals'),r.get('core'),r.get('cross'),r.get('core_na'))
                                   for r in rows).most_common(8):
        print("  equals=%-5s core=%-5s cross=%-5s core_na=%-5s x%d" % (k+(v,)))
    print("\n=== Unit routing: which rows carry Product(Unit<>) ===")
    for side,key in (('BOUND','lshape'),('ACTUAL','rshape')):
        hit=[r for r in rows if 'Product(Unit<>)' in (r.get(key) or '')]
        print("  %s side: %d rows" % (side, len(hit)))
    json.dump(rows, open('/tmp/rebindv3_rows.json','w'), indent=1)
    print("\nrows written: /tmp/rebindv3_rows.json")
    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
