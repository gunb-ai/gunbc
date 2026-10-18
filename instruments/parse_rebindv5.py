#!/usr/bin/env python3
"""Parse probe v5's REBINDV4 rows into the PR-BODY identity tables for the 'own form' arm.

The parent asked (msg_0b50238d) for the 20 out-of-class firing rows and the 4 rows with params on the
ACTUAL side, BY IDENTITY, 'saying what the predicate does to each'. Those numbers came from probe v3,
which aggregated params over the whole visited tree. Probe v4/v5 established that the SHIPPED predicate
is the root-own reading (141 of 254 fire, vs 155 under aggregation), so the identity tables must be
rebuilt from the root-own fields -- otherwise the PR body would cite rows the shipped arm does not
actually admit.

Fields consumed (all computed in Rust, in the same shape as the DAG code):
  ownBound / ownActual   the shipped predicate's answer for each side, root-own params
  treeBoundAny           probe v3's aggregated reading, kept so the two are comparable per row
  rootOwnParams          the compared node's OWN declared parameter names (identity evidence)
  leaves / lshape / rshape, formal, coreEq, s1bMint
Validity: one line per refusal row and zero cap events, else the whole-tree claims are void.
"""
import collections
import json
import os
import re
import sys

LOG = sys.argv[1] if len(sys.argv) > 1 else '/tmp/v5_unpack/sh.log'
OUT = sys.argv[2] if len(sys.argv) > 2 else '/tmp/own_form_tables.json'

rows = []
for line in open(LOG, errors='replace'):
    if not line.startswith('REBINDV4|'):
        continue
    kv = {}
    for part in line[len('REBINDV4|'):].rstrip('\n').split('|'):
        if '=' in part:
            k, v = part.split('=', 1)
            kv[k] = v
    rows.append(kv)

print("REBINDV4 rows parsed: %d" % len(rows))
if not rows:
    print("NO ROWS -- nothing is reportable from this log")
    sys.exit(4)

caps = sum(1 for r in rows if r.get('lcaps', '0') != '0' or r.get('tcaps', '0') != '0')
print("rows with a cap event: %d  (want 0; a capped tree cannot support a whole-tree claim)" % caps)

def P(s):
    return [x for x in (s or '').split(',') if x]


def bucket(r):
    lc, rc = r.get('lconn', ''), r.get('rconn', '')
    return r


def cls(r):
    conn = (r.get('lconn', '').startswith('Disj')
            and not r.get('rconn', '').startswith('Disj')
            and r.get('lname') == r.get('rname'))
    twin = (r.get('lconn') == r.get('rconn') and r.get('lname') == r.get('rname')
            and r.get('lshape') != r.get('rshape'))
    return conn, twin


fire = [r for r in rows if r.get('ownBound') == '1']
conn_fire = [r for r in fire if cls(r)[0]]
out_fire = [r for r in fire if not cls(r)[0]]
twins = [r for r in rows if cls(r)[1]]
loose_twins = [r for r in twins if r.get('ownBound') == '1']
actual_side = [r for r in rows if P(r.get('actOwnParams'))]
agg_only = [r for r in rows if r.get('treeBoundAny') == '1' and r.get('ownBound') != '1']
s1b = [r for r in rows if r.get('s1bMint', '0') != '0']


def ident(r):
    return {'module': r.get('module'), 'bind': r.get('bind'), 'formal': r.get('formal'),
            'bound': r.get('lshape'), 'actual': r.get('rshape'),
            'bound_own_params': P(r.get('rootOwnParams')),
            'actual_own_params': P(r.get('actOwnParams')),
            'bound_leaves': P(r.get('leaves'))[:10],
            'matched_params': sorted(set(P(r.get('rootOwnParams'))) & set(P(r.get('leaves')))),
            'coreEq': r.get('coreEq'), 's1bMint': r.get('s1bMint'),
            'ownBound': r.get('ownBound'), 'ownActual': r.get('ownActual'),
            'treeBoundAny': r.get('treeBoundAny')}


out = {
    'predicate': 'ROOT-OWN: n.params non-empty AND some childless node reachable from n (raw children '
                 'plus a bounded inferred-Resolved hop) has NAME equal to one of those params',
    'rows': len(rows), 'cap_events': caps,
    'firing_total': len(fire), 'firing_conn_class': len(conn_fire),
    'firing_out_of_class': len(out_fire),
    'aggregated_only_rows': len(agg_only),
    'twins_same_head_both_applied': len(twins),
    'twins_that_fire_MUST_BE_ZERO': len(loose_twins),
    'actual_side_has_own_params': len(actual_side),
    'rows_with_s1b_minting_MUST_BE_ZERO': len(s1b),
    'out_of_class_firing': [ident(r) for r in out_fire],
    'conn_class_firing': [ident(r) for r in conn_fire],
    'actual_side_rows': [ident(r) for r in actual_side],
    'aggregated_only': [ident(r) for r in agg_only],
    'loose_twins': [ident(r) for r in loose_twins],
    'by_module': collections.Counter(r.get('module') for r in fire).most_common(12),
}
json.dump(out, open(OUT, 'w'), indent=1)

print("\nfiring (shipped root-own predicate): %d = %d conn-class + %d OUT-OF-CLASS"
      % (len(fire), len(conn_fire), len(out_fire)))
print("rows the AGGREGATED reading would add but the shipped one does not: %d" % len(agg_only))
print("same-head/both-applied twins: %d   of which firing (must be 0): %d" % (len(twins), len(loose_twins)))
print("rows with params on the ACTUAL side: %d" % len(actual_side))
print("rows with S1B minting (must be 0): %d" % len(s1b))
print("\nOUT-OF-CLASS FIRING ROWS, by identity (the parent asked for these 20, and for what the predicate does to each):")
for r in out_fire:
    print("   %-36s bind=%-6s ownParams=%s matched=%s coreEq=%s\n      bound  %s\n      actual %s"
          % ((r.get('module') or '')[:36], r.get('bind'), P(r.get('rootOwnParams'))[:5],
             sorted(set(P(r.get('rootOwnParams'))) & set(P(r.get('leaves'))))[:5], r.get('coreEq'),
             (r.get('lshape') or '')[:74], (r.get('rshape') or '')[:74]))
print("\nACTUAL-SIDE-params ROWS (what my symmetric keep-arm does to each):")
for r in actual_side:
    print("   %-36s bind=%-6s actualOwnParams=%s ownActual=%s\n      bound  %s\n      actual %s"
          % ((r.get('module') or '')[:36], r.get('bind'), P(r.get('actOwnParams'))[:5],
             r.get('ownActual'), (r.get('lshape') or '')[:70], (r.get('rshape') or '')[:70]))
print("\nLOOSE TWINS (predicate wrongly admits a distinct settled instance): %d" % len(loose_twins))
for r in loose_twins[:10]:
    print("   %-36s bind=%-6s ownParams=%s\n      bound  %s\n      actual %s"
          % ((r.get('module') or '')[:36], r.get('bind'), P(r.get('rootOwnParams'))[:5],
             (r.get('lshape') or '')[:70], (r.get('rshape') or '')[:70]))
print("\nwrote %s" % OUT)
