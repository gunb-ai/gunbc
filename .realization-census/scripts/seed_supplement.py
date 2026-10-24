#!/usr/bin/env python3
"""seed_supplement.py: add the slice-01 instances the reader dropped before ruling (d) reached it.

usage: seed_supplement.py <extracted pin tree> <slice dir>
Writes <slice dir>/findings_supplement.tsv and <slice dir>/findings_dropped.tsv; merge.py consumes both.
Over the hand-written seed files (not in generated_members.txt), non-test lines only (a line at or after the
`#[cfg(test)]` or `#[test]` item, skipped by brace matching, is test code: M6; an earlier revision stopped at the first marker, which truncated cli_run.rs at line 236 of 35,760), it emits:
  - kernel ABI / upstream-constant path literal (/proc/*, /sys/*, /etc/os-release) -> access L4   (ruling d)
  - device-node literal (/dev/* except /dev/null, /dev/zero, /dev/stdin|stdout|stderr) -> placement L4   (ruling d)
  - env read of a CI-environment variable (GITHUB_*, CI, RUNNER_*) -> placement L4, environment-specific branch
symbol = the nearest enclosing `fn` above the line. Rows inside string literals that hold .dag source are dropped
by the reader's caveat (a): interp_recorded_fixture_witness.rs rows whose specimen names /tmp/x or /tmp/y.
"""
import csv, os, re, sys

TREE, SL = sys.argv[1], sys.argv[2]
gen = {l.strip() for l in open(os.path.join(SL, 'generated_members.txt')) if l.strip()}
mods = [r for r in csv.DictReader(open(os.path.join(SL, 'modules.tsv')), delimiter='\t')]
hand = [r['module'] for r in mods if r['module'] not in gen and r['class'] != 'seed-generated']

KERNEL = re.compile(r'"(/proc/[^"]*|/sys/[^"]*|/etc/os-release)"')
DEVICE = re.compile(r'"(/dev/(?!null\b|zero\b|stdin\b|stdout\b|stderr\b|fd/)[^"]+)"')
ROOTENV = re.compile(r'env::var(?:_os)?\(\s*"?([A-Z][A-Z0-9_]*(?:ROOT|DIR|PATH|TOKEN|HOME)[A-Z0-9_]*)"?\s*\)')
CIENV = re.compile(r'env::var(?:_os)?\(\s*"((?:GITHUB_|RUNNER_)[A-Z_]+|CI)"')
LITERAL = re.compile(r'r#*"[^"]*"#*|"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)\'')  # strings and char literals, blanked before brace counting
FN = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([a-z_][a-z0-9_]*)')

rows = []
for m in hand:
    p = os.path.join(TREE, m)
    if not os.path.isfile(p):
        continue
    cur = '(module scope)'
    skip, depth, opened = False, 0, False
    for line in open(p, encoding='utf-8', errors='replace'):
        s = line.strip()
        if not skip and (s.startswith('#[cfg(test)]') or s.startswith('#[test]')):
            skip, depth, opened = True, 0, False
            continue
        if skip:
            code = LITERAL.sub('', line.split('//')[0])
            depth += code.count('{') - code.count('}')
            opened = opened or '{' in code
            if (opened and depth <= 0) or (not opened and s.endswith(';')):
                skip = False
            continue
        if s.startswith('//'):
            continue
        f = FN.match(line)
        if f:
            cur = f.group(1)
        spec = s[:200]
        for k in KERNEL.findall(line):
            rows.append([m, 'access', 'L4', cur, spec, f'kernel ABI path "{k}" read directly (ruling d)', 'extdeps.linux.*', 'declare once and consume'])
        for d in DEVICE.findall(line):
            rows.append([m, 'placement', 'L4', cur, spec, f'device node "{d}" named in code (ruling d)', 'host or instance declaration naming the device', 'declare once and consume'])
        for v in ROOTENV.findall(line):
            if not v.startswith(('GITHUB_', 'RUNNER_', 'CARGO')):
                rows.append([m, 'access', 'L4', cur, spec, f'root/credential location reached through env var {v}', 'gunbc.auth.access_token_source / declared workspace root', 'declare once and consume'])
        for v in CIENV.findall(line):
            rows.append([m, 'placement', 'L4', cur, spec, f'behaviour switches on CI environment variable {v}', 'gunbc.ci.* (no CI-environment authority consumed by the seed)', 'declare once and consume'])

hdr = ['module', 'dimension', 'level', 'symbol', 'specimen', 'note', 'existing_authority', 'fix_shape']
with open(os.path.join(SL, 'findings_supplement.tsv'), 'w') as o:
    o.write('\t'.join(hdr) + '\n')
    for r in rows:
        o.write('\t'.join(x.replace('\t', ' ') for x in r) + '\n')

drop = [r for r in csv.DictReader(open(os.path.join(SL, 'findings.tsv')), delimiter='\t')
        if 'interp_recorded_fixture_witness' in r['module'] and re.search(r'/tmp/[xy]\b', r['specimen'])]
with open(os.path.join(SL, 'findings_dropped.tsv'), 'w') as o:
    o.write('\t'.join(hdr) + '\n')
    for r in drop:
        o.write('\t'.join(r[h] for h in hdr) + '\n')
print('supplement', len(rows), 'dropped', len(drop))
