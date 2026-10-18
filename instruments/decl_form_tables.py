#!/usr/bin/env python3
"""THE PR-BODY TABLES for the 'binding IS the generic declaration's own form' arm (#14152).

Joins the v2 DECIDING-ARM probe's per-node tree walk to the v3 whole-tree params/leaf capture on
(module, bind, lshape, rshape) = the ROW IDENTITY. That join matched 254/254 with 0 unmatched at
scratch head 2dbfa1ec866, so no row is silently dropped.

WHY THE LOG AND NOT THE JSON: rebindarm_rows.json holds only the key=value prefix of each REBINDARM
line; the per-node tree is the raw payload after ';;TREE'. Reading a 'nodes' field from that JSON yields
None for every row, which makes the predicate FALSE EVERYWHERE -- a wrong answer that looks clean. The
parse below is the one instruments/h_hypothesis_join.py was validated with.

PREDICATE UNDER TEST (the parent's definition, semantic, not class-scoped):
    P(n) := n.params is non-empty AND at least one leaf name reachable from n is one of n.params
Reported by identity, as asked: (1) the 20 out-of-class firing rows; (2) the rows whose params sit on
the ACTUAL side; (3) the refusal twins (same head, both applied, different shapes), where P must be
false. 'conn class' = bound is a Disj naming the actual's head with the actual unconnected: that class
was only the EVIDENCE, so its rows and the out-of-class rows are reported separately and P is applied to
both.
"""
import json
import collections
import re

ARMLOG = '/tmp/probeB8_unpack/sh.log'
V3 = '/tmp/rebindv3_rows.json'
OUT = '/tmp/decl_form_tables.json'

CAPS = re.compile(r'\s*\|\|CHILD-CAP|\s*\|\|DEPTH-CAP')
NODE = re.compile(
    r'^(?P<path>[LR][0-9A-Za-z]*) n=(?P<name>[^|]*)\|conn=(?P<conn>[^|]*)\|card=(?P<card>[^|]*)'
    r'\|decl=(?P<decl>[^|]*)\|infer=(?P<infer>.*?)\|shape=(?P<shape>.*?)\|nc=(?P<nc>\d+)\|idsp=(?P<idsp>.*)$'
)


def load_arm():
    v2 = collections.defaultdict(list)
    n_arm = 0
    for line in open(ARMLOG, errors='replace'):
        if not line.startswith('REBINDARM|'):
            continue
        n_arm += 1
        b = line[len('REBINDARM|'):].rstrip('\n')
        i = b.find(';;TREE')
        d = dict(kv.split('=', 1) for kv in b[:i].rstrip('|').split('|') if '=' in kv)
        nodes = {}
        for seg in b[i + 6:].split(' ;; '):
            for side in seg.split(' >< '):
                m = NODE.match(side.strip())
                if m:
                    g = m.groupdict()
                    g['idsp'] = CAPS.sub('', g['idsp'])
                    nodes[g['path']] = g
        key = (d.get('module'), d.get('bind'), d.get('unfoldedL'), d.get('unfoldedR'))
        v2[key].append(nodes)
    return v2, n_arm


def P(s):
    return set(x for x in (s or '').split(',') if x)


def leaves(nodes, side):
    return sorted(g['name'] for g in nodes.values()
                  if g['path'].startswith(side) and g['nc'] == '0')


def decl_form(nodes, side, params):
    lv = leaves(nodes, side)
    return (len(params) > 0 and any(x in params for x in lv)), lv


v2, n_arm = load_arm()
rows = json.load(open(V3))
print("v2 REBINDARM lines parsed: %d   v3 rows: %d   (want 254 / 254)" % (n_arm, len(rows)))

out = {'predicate': 'n.params non-empty AND some reachable leaf name of n is one of n.params',
       'join_total': 0, 'join_unmatched': 0,
       'join_key_missing': 0, 'buckets': {}}
by = collections.Counter()
fire, conn_fire, out_class, actual_side, twins = [], [], [], [], []

for r in rows:
    k = (r['module'], r['bind'], r['lshape'], r['rshape'])
    if k not in v2:
        out['join_key_missing'] += 1
        continue
    nodes = v2[k][0]
    out['join_total'] += 1
    Lp, Rp = P(r.get('lparams')), P(r.get('rparams'))
    lf, Llv = decl_form(nodes, 'L', Lp)
    rf, Rlv = decl_form(nodes, 'R', Rp)
    conn = (r.get('lconn', '').startswith('Disj')
            and not r.get('rconn', '').startswith('Disj')
            and r.get('lname') == r.get('rname'))
    same_applied = (r.get('lconn') == r.get('rconn') and r.get('lname') == r.get('rname')
                    and r.get('lshape') != r.get('rshape'))
    ident = {'module': r['module'], 'bind': r['bind'], 'bound': r['lshape'], 'actual': r['rshape'],
             'bound_params': sorted(Lp), 'actual_params': sorted(Rp),
             'bound_leaves_sample': Llv[:8], 'actual_leaves_sample': Rlv[:8],
             'formal': r.get('formal'), 'core': r.get('core'), 'equals': r.get('equals'),
             'bound_decl_form': lf, 'actual_decl_form': rf}
    by[('conn' if conn else 'out-of-class',
        'bound-decl-form' if lf else 'bound-not-own',
        'actual-has-params' if Rp else 'actual-no-params')] += 1
    if lf:
        (conn_fire if conn else out_class).append(ident)
    if Rp:
        actual_side.append(ident)
    if same_applied:
        twins.append(ident)
        if lf:
            out.setdefault('LOOSE_TWINS', []).append(ident)

out['buckets'] = {' | '.join(k2): v for k2, v in sorted(by.items(), key=lambda kv: -kv[1])}
out['conn_class_firing'] = conn_fire
out['out_of_class_firing'] = out_class
out['actual_side_has_params'] = actual_side
out['refusal_twins'] = twins
json.dump(out, open(OUT, 'w'), indent=1)

print("join: %d matched of %d, %d unmatched (want 254 / 0)" % (out['join_total'], len(rows), out['join_key_missing']))
for k, v in out['buckets'].items():
    print("  %4d  %s" % (v, k))
print("\nBOUND-SIDE FIRING: %d conn-class + %d OUT-OF-CLASS = %d"
      % (len(conn_fire), len(out_class), len(conn_fire) + len(out_class)))
print("ACTUAL-SIDE params: %d  (of which the actual is itself a decl form: %d)"
      % (len(actual_side), sum(1 for a in actual_side if a['actual_decl_form'])))
print("REFUSAL TWINS (same head, both applied, different shapes): %d" % len(twins))
print("LOOSE TWINS (predicate wrongly fires; want 0): %d" % len(out.get('LOOSE_TWINS', [])))
for l in out.get('LOOSE_TWINS', []):
    print("   LOOSE %s bind=%s params=%s leaves=%s"
          % (l['module'], l['bind'], l['bound_params'][:5], l['bound_leaves_sample'][:5]))
print("\nOUT-OF-CLASS FIRING ROWS, by identity:")
for x in out_class:
    print("   %-34s bind=%-6s params=%s core=%s\n      bound  %s\n      actual %s"
          % (x['module'][:34], x['bind'], x['bound_params'][:6], x['core'],
             x['bound'][:74], x['actual'][:74]))
print("\nACTUAL-SIDE-params ROWS, by identity:")
for a in actual_side:
    print("   %-34s bind=%-6s actual_params=%s actual_decl_form=%s bound_decl_form=%s\n      bound  %s\n      actual %s"
          % (a['module'][:34], a['bind'], a['actual_params'][:6], a['actual_decl_form'],
             a['bound_decl_form'], a['bound'][:70], a['actual'][:70]))
print("\nFIRST 5 CONN-CLASS FIRING (evidence for the arm):")
for x in conn_fire[:5]:
    print("   %-34s bind=%-6s params=%s\n      leaves %s"
          % (x['module'][:34], x['bind'], x['bound_params'][:6], x['bound_leaves_sample'][:7]))
print("\nWANT: %d conn-class, %d out-of-class, %d actual-side, %d twins, 0 loose"
      % (120, 20, 4, 8, ))
