#!/usr/bin/env python3
"""readme_tables.py: the generated tables README.md quotes (run after merge.py and classes.py).

Writes readme_tables.md. Every number is computed from modules.tsv / findings.tsv / classes.tsv, so each count has a
member list behind it (filter the sheet by the named columns).
Worst-module ranking: a module's score is the sum over the five dimensions of its decision-site level when that level
is a problem (Rederived 3, Literal 4, Forked 5), then its instance count as tie-break. This is an ordering for reading,
not a severity model: the README says so.
"""
import csv, collections, re

DIMS = ['storage', 'caching', 'access', 'authz', 'placement']
SITES = ['None', 'Home', 'Passed', 'Rederived', 'Literal', 'Forked']
M = list(csv.DictReader(open('modules.tsv'), delimiter='\t'))
F = list(csv.DictReader(open('findings.tsv'), delimiter='\t'))
C = list(csv.DictReader(open('classes.tsv'), delimiter='\t'))
HAND = ['production', 'extdeps', 'seed']
out = []
w = out.append

w('## Counts: modules per decision site, per dimension, per module class\n')
for k in HAND + ['seed-generated', 'roster']:
    rows = [r for r in M if r['class'] == k]
    if not rows:
        continue
    w(f'### {k} ({len(rows)} modules)\n')
    w('| dimension | ' + ' | '.join(SITES) + ' |')
    w('|---|' + '---:|' * len(SITES))
    for d in DIMS:
        c = collections.Counter(r[d + '_site'] for r in rows)
        w(f'| {d} | ' + ' | '.join(str(c[s]) for s in SITES) + ' |')
    worst = collections.Counter(r['worst_site'] for r in rows)
    w('| **worst of five** | ' + ' | '.join(str(worst[s]) for s in SITES) + ' |\n')

w('## Instances per dimension and decision site (hand-read classes)\n')
w('| dimension | Rederived | Literal | Forked | modules with any |')
w('|---|---:|---:|---:|---:|')
for d in DIMS:
    rs = [r for r in F if r['dimension'] == d and r['class'] in HAND and r['site'] in ('Rederived', 'Literal', 'Forked')]
    c = collections.Counter(r['site'] for r in rs)
    w(f"| {d} | {c['Rederived']} | {c['Literal']} | {c['Forked']} | {len({r['module'] for r in rs})} |")
w('')

mrow = {r['module']: r for r in M}
def ranked(k):
    score = {}
    for r in M:
        if r['class'] != k:
            continue
        sc = sum(int(r[d + '_level'][1]) for d in DIMS if int(r[d + '_level'][1]) >= 3)
        if sc:
            score[r['module']] = (sc, sum(1 for f in F if f['module'] == r['module'] and f['site'] in ('Rederived', 'Literal', 'Forked')))
    return score, sorted(score, key=lambda m: (-score[m][0], -score[m][1], m))
for k, n in (('production', 25), ('extdeps', 10), ('seed', 10)):
    score, order = ranked(k)
    if not order:
        continue
    w(f'## The {min(n, len(order))} worst {k} modules (of {len(order)} with any problem)\n')
    if k == 'production':
        w('Ordered by the sum of their problem levels across the five dimensions (Rederived 3, Literal 4, Forked 5), then by instance count. This is a reading order, not a severity model.\n')
    w('| # | module | problem dimensions | instances | worst finding |')
    w('|---:|---|---|---:|---|')
    for i, m in enumerate(order[:n], 1):
        r = mrow[m]
        dims = ', '.join(f"{d} {r[d + '_site']}" for d in DIMS if int(r[d + '_level'][1]) >= 3)
        fs = sorted([f for f in F if f['module'] == m and f['dimension'] in DIMS], key=lambda f: -int(f['level'][1]))
        wf = f"{fs[0]['dimension']} {fs[0]['site']}: `{fs[0]['symbol'][:60]}` {fs[0]['specimen'][:90]}" if fs else r['worst_finding'][:150]
        w(f"| {i} | `{m}` | {dims} | {score[m][1]} | {wf.replace('|', '/')} |")
    w('')

w('## Most duplicated literal or mechanism, per dimension\n')
w('A literal is a path-shaped token (first two segments under a system root, or a repo-relative target/ or artifacts/ name), a host name or an env-var name, taken from the specimen and note; a mechanism is an effect call. Counted over distinct modules.\n')
w('| dimension | most duplicated literal | modules | most duplicated mechanism | modules |')
w('|---|---|---:|---|---:|')
# a literal is a path-shaped token (absolute under a system root, or repo-relative target/ artifacts/), a host name, or an env-var name
TOK = re.compile(r'(/(?:var|etc|tmp|home|opt|run|usr|proc|sys|dev|srv|Users)/[A-Za-z0-9_.@-]+(?:/[A-Za-z0-9_.@-]+)?/?|\b(?:target|artifacts)/[A-Za-z0-9_.-]+|\bsrv[0-9]+\b|\b[A-Z][A-Z0-9]*_[A-Z0-9_]{2,}\b)')
CALL = re.compile(r'\b((?:Filesystem|shell|std::fs|fs)(?:\.|::)[A-Za-z_.]+|sudo_elevate|OnceLock|thread_local|srv1_live_dashboard_instance|mkdir_[a-z_]+command|rm_recursive[a-z_]*|env::var)')
for d in DIMS:
    lit, mech = collections.defaultdict(set), collections.defaultdict(set)
    for f in F:
        if f['dimension'] != d or f['class'] not in HAND or f['site'] not in ('Rederived', 'Literal', 'Forked'):
            continue
        for x in set(TOK.findall(f['specimen'] + ' ' + f['note'])):
            lit[x].add(f['module'])
        for x in set(CALL.findall(f['specimen'] + ' ' + f['symbol'])):
            mech[x].add(f['module'])
    bl = max(lit.items(), key=lambda kv: (len(kv[1]), kv[0]), default=('', set()))
    bm = max(mech.items(), key=lambda kv: (len(kv[1]), kv[0]), default=('', set()))
    w(f"| {d} | `{bl[0]}` | {len(bl[1])} | `{bm[0]}` | {len(bm[1])} |")
w('')

w('## Error classes (classes.tsv), by modules affected\n')
w('| class_id | dimension | modules | instances | current rung | owning authority |')
w('|---|---|---:|---:|---:|---|')
for r in C:
    w(f"| `{r['class_id']}` | {r['dimension']} | {r['modules']} | {r['instances']} | {r['current_rung']} | {r['owning_authority'][:110].replace('|', '/')} |")
w('')
open('readme_tables.md', 'w').write('\n'.join(out) + '\n')
print('\n'.join(out[:3]))
# README.md = the hand-written reading (README_head.md) followed by these generated tables, verbatim.
open('README.md', 'w').write(open('README_head.md').read() + '\n---\n\n# Generated tables (scripts/readme_tables.py)\n\n' + '\n'.join(out) + '\n')
