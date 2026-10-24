#!/usr/bin/env python3
"""merge.py: combine the parent's own rows and every slice sheet into the final modules.tsv / findings.tsv.

usage: merge.py   (run from .realization-census/; reads own_modules.tsv, own_findings.tsv, slices/*/)
Writes modules.tsv, findings.tsv, merge_report.txt. Every normalisation is a logged merge decision (method.md M1-M8, ruling d).

Rules, in order:
 1. Parent rows (phases P0, P2-parent, P3) win for their module's modules row; the slice reader's INSTANCES for that module are kept
    (the findings sheet keeps every instance) and tagged, and the module's levels are re-derived from the union under rule 5.
 2. Class fixes: M2 / M8 (gunbc.fleet.fleet_bmc_observation, gunbc.systemd_directive_value_grain -> production).
 3. Seed: findings_supplement.tsv rows added and findings_dropped.tsv rows removed (M7).
 4. Ruling (d) re-dimensioning of instances written before it: a specimen naming a kernel-ABI path
    (/proc/, /sys/, /etc/os-release) scored under storage or placement moves to access; one naming a device node
    (/dev/x, not null/zero/std*) scored under storage or access moves to placement. The level is kept.
 5. Module levels are re-derived per dimension as the WORST instance in findings.tsv at L3-L5. A sheet cell at L3+
    with no instance behind it is kept but counted as UNBACKED (reported). A cell whose only L3+ instances moved away
    under rule 4 drops to the sheet's value capped at L2 (the reader's L0-L2 reading is not recoverable, so L0 is written and counted).
"""
import csv, glob, os, re, collections

DIMS = ['storage', 'caching', 'access', 'authz', 'placement']
MCOL = ['module', 'class', 'storage_level', 'caching_level', 'access_level', 'authz_level', 'placement_level',
        'ambient_calls', 'worst_finding', 'evidence_symbols', 'phase']
FCOL = ['module', 'dimension', 'level', 'symbol', 'specimen', 'note', 'existing_authority', 'fix_shape']
CLASS_FIX = {'gunbc.fleet.fleet_bmc_observation': 'production', 'gunbc.systemd_directive_value_grain': 'production'}
KERNEL = re.compile(r'/proc/|/sys/|/etc/os-release')
DEVICE = re.compile(r'/dev/(?!null\b|zero\b|stdin\b|stdout\b|stderr\b|fd/)\w')

def read(p, cols):
    if not os.path.exists(p):
        return []
    rows = []
    for r in csv.DictReader(open(p), delimiter='\t'):
        rows.append({c: (r.get(c) or '').strip() for c in cols})
    return rows

def lv(s):
    m = re.match(r'L([0-5])', s or '')
    return int(m.group(1)) if m else 0

rep = []
own_m = read('own_modules.tsv', MCOL)
own_f = read('own_findings.tsv', FCOL)
own_set = {r['module'] for r in own_m}
mods, finds = {r['module']: r for r in own_m}, list(own_f)
overlap = []
for d in sorted(glob.glob('slices/[0-9]*/')):
    s = os.path.basename(d.rstrip('/'))
    sm, sf = read(d + 'modules.tsv', MCOL), read(d + 'findings.tsv', FCOL)
    if s == '01-seed':
        sup = read(d + 'findings_supplement.tsv', FCOL)
        dropk = {(r['module'], r['symbol'], r['specimen']) for r in read(d + 'findings_dropped.tsv', FCOL)}
        before = len(sf)
        sf = [r for r in sf if (r['module'], r['symbol'], r['specimen']) not in dropk] + sup
        rep.append(f'M7 seed: +{len(sup)} supplement, -{before + len(sup) - len(sf)} dropped')
    for r in sm:
        if r['module'] in own_set:
            overlap.append((s, r['module']))
            continue
        if r['module'] in mods:
            rep.append(f'DUPLICATE READER: {r["module"]} in {s} and {mods[r["module"]].get("_slice")}')
            continue
        if not r['phase']:
            r['phase'] = 'P2'
        r['_slice'] = s
        mods[r['module']] = r
    for r in sf:
        if r['module'] in own_set:
            r['note'] = (r['note'] + f'; instance from slice {s} reader, module row is the parent\'s').lstrip('; ')
        finds.append(r)
rep.append(f'rule 1: {len(overlap)} slice module rows superseded by parent rows (their instances kept): ' + ', '.join(m for _, m in overlap))

for m, c in CLASS_FIX.items():
    if m in mods and mods[m]['class'] != c:
        rep.append(f'class fix {m}: {mods[m]["class"]} -> {c}')
        mods[m]['class'] = c

moved = collections.Counter()
moved_from = collections.defaultdict(set)
for r in finds:
    sp, dim = r['specimen'] + ' ' + r['symbol'], r['dimension']
    if dim in ('storage', 'placement') and KERNEL.search(sp) and not DEVICE.search(sp):
        moved_from[r['module']].add(dim); r['dimension'] = 'access'; r['note'] = (r['note'] + '; re-dimensioned to access by ruling (d)').lstrip('; ')
        moved[(dim, 'access')] += 1
    elif dim in ('storage', 'access') and DEVICE.search(sp) and not KERNEL.search(sp):
        moved_from[r['module']].add(dim); r['dimension'] = 'placement'; r['note'] = (r['note'] + '; re-dimensioned to placement by ruling (d)').lstrip('; ')
        moved[(dim, 'placement')] += 1
rep.append('ruling (d) moves: ' + ', '.join(f'{a}->{b}: {n}' for (a, b), n in sorted(moved.items())))

orphan_f = sorted({r['module'] for r in finds} - set(mods))
if orphan_f:
    rep.append(f'findings for modules with no modules row ({len(orphan_f)}): ' + ', '.join(orphan_f[:20]))

worst = collections.defaultdict(int)
for r in finds:
    if r['dimension'] in DIMS:
        worst[(r['module'], r['dimension'])] = max(worst[(r['module'], r['dimension'])], lv(r['level']))
unbacked, dropped_cells, raised = [], [], []
for m, r in mods.items():
    if r['class'] in ('seed-generated', 'roster'):
        continue
    for d in DIMS:
        sheet, fw = lv(r[d + '_level']), worst[(m, d)]
        if fw >= 3:
            if fw > sheet:
                raised.append((m, d, sheet, fw))
            new = fw
        elif sheet >= 3:
            if d in moved_from[m]:
                new = 0; dropped_cells.append((m, d, sheet))
            else:
                new = sheet; unbacked.append((m, d, sheet))
        else:
            new = sheet
        r[d + '_level'] = f'L{new}'
rep.append(f'cells raised to their worst instance: {len(raised)}')
rep.append(f'cells dropped because their instances moved under ruling (d): {len(dropped_cells)} ' + ', '.join(f'{m}:{d}' for m, d, _ in dropped_cells[:30]))
rep.append(f'UNBACKED L3+ cells (sheet level with no instance row): {len(unbacked)}')
rep += [f'  {m} {d} L{l}' for m, d, l in unbacked]

# Schema addendum (parent, 2026-10-24): each L0-L5 cell is also written as a decision-site class name; the L number stays beside it for sorting.
SITE = ['None', 'Home', 'Passed', 'Rederived', 'Literal', 'Forked']
OUT_M = ['module', 'class'] + [c for d in DIMS for c in (d + '_site', d + '_level')] + ['ambient_calls', 'worst_site', 'worst_finding', 'evidence_symbols', 'phase']
OUT_F = ['module', 'class', 'dimension', 'site', 'level', 'symbol', 'specimen', 'note', 'existing_authority', 'fix_shape']
def clean(x):
    return (x or '').replace('\t', ' ').replace('\n', ' ')
with open('modules.tsv', 'w') as o:
    o.write('\t'.join(OUT_M) + '\n')
    for m in sorted(mods):
        r = dict(mods[m])
        if r['class'] in ('seed-generated', 'roster'):
            for d in DIMS: r[d + '_level'] = 'L0'
        for d in DIMS: r[d + '_site'] = SITE[lv(r[d + '_level'])]
        r['worst_site'] = SITE[max(lv(r[d + '_level']) for d in DIMS)]
        o.write('\t'.join(clean(r.get(c)) for c in OUT_M) + '\n')
with open('findings.tsv', 'w') as o:
    o.write('\t'.join(OUT_F) + '\n')
    for r in sorted(finds, key=lambda r: (r['module'], r['dimension'], -lv(r['level']))):
        r = dict(r); r['class'] = mods.get(r['module'], {}).get('class', '')
        l = lv(r['level']); r['level'] = f'L{l}'; r['site'] = SITE[l] if r['dimension'] in DIMS else 'Other'
        o.write('\t'.join(clean(r.get(c)) for c in OUT_F) + '\n')
open('merge_report.txt', 'w').write('\n'.join(rep) + '\n')
print('\n'.join(rep[:12]))
print('modules', len(mods), 'findings', len(finds))
