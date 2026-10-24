#!/usr/bin/env python3
"""Phase 1: mechanical default levels per module, per dimension, plus the AMBIENT count.

usage: phase1.py <extracted pin tree> <out.tsv>
Each heuristic below is a LEAD for the hand read, not a verdict; method.md states FP/FN risk.
"""
import os, re, sys

ROOT, OUT = sys.argv[1], sys.argv[2]

def klass(rel):
    if '/test/' in rel or rel.startswith('dag/test/') or re.search(r'(^|/)tests?/', rel) or rel.endswith('_test.rs') or rel.endswith('/tests.rs'):
        return 'test'
    if rel.startswith('src/v1/'):
        return 'seed'
    if rel.startswith('src/v2/'):
        return 'compiler'
    if '/instruments/' in rel:
        return 'instrument'
    if re.search(r'recurring_failure_mode/|rung_drop/|/plans/|design_ruling', rel):
        return 'ledger'
    base = os.path.basename(rel)[:-4]
    if rel.startswith('dag/gunbc/guarantee_stall/') or (base not in ROSTER_EXEMPT and re.search(r'(_scaffold|_seed_growth|census|_frontier|_freeze)$|^census_', base)):
        return 'roster'
    if rel.startswith('dag/extdeps/'):
        return 'extdeps'
    return 'production'

def modname(rel):
    if rel.endswith('.rs'):
        return rel
    p = rel[:-4]
    if p.startswith('dag/'):
        return p[4:].replace('/', '.')
    if p.startswith('src/v2/'):
        return 'v2.' + p[7:].replace('/', '.')
    return p

ROSTER_EXEMPT = {'privileged_effect_census'}  # authorities named by the brief keep their class
STR = re.compile(r'"(?:[^"\\\n]|\\.)*"')
def strip_comments(text, rs):
    out = []
    for line in text.split('\n'):
        s = line.lstrip()
        if s.startswith('//'):
            continue
        out.append(line)
    return '\n'.join(out)

def code_literals(code):
    """string literals that look like values, not prose: no spaces, <=160 chars."""
    lits = []
    for m in STR.finditer(code):
        v = m.group(0)[1:-1]
        if ' ' in v or len(v) > 160:
            continue
        lits.append(v)
    return lits

def prose_free(code):
    return STR.sub('""', code)

AMBIENT_DAG = re.compile(r'\b(Filesystem|shell|Http|Git|R2|Systemd|Process|Ssh)\.[A-Z][A-Za-z]*(?:\.[A-Z][A-Za-z]*)?')
AMBIENT_RS = re.compile(r'\bstd::fs::[a-z_]+|\bfs::(?:read|write|create_dir_all|remove_dir_all|remove_file|read_dir|rename|copy|read_to_string)\b|\bCommand::new\b|\b(?:std::)?env::var(?:_os)?\b|\bTcpStream::connect\b|\bFile::(?:open|create)\b|\bOpenOptions::new\b')
ABS_PATH = re.compile(r'^/(var|etc|home|tmp|srv|opt|run|usr|root|nix|mnt|Users|private|proc|sys|dev)(/|$)')
HOST = re.compile(r'\bsrv[0-9]+\b|\bfci[0-9]+\b|\.gunb\.ai\b|^localhost$|127\.0\.0\.1|0\.0\.0\.0|r2\.cloudflarestorage|amazonaws\.com|tailscale|\.ts\.net')
ENV_READ_DAG = re.compile(r'shell\.Env\.Get')
SUDO = re.compile(r'\bsudo_elevate\b|"sudo"|/usr/bin/sudo|\bvisudo\b|sudoers')
LIVE_PIN = re.compile(r'\b[a-z0-9_]*_live_dashboard_instance\(\)|\bsrv[0-9]_[a-z_]*dashboard_instance\(\)')
UNIT = re.compile(r'\.(service|socket|timer|path|slice)$')
PORT = re.compile(r':[0-9]{2,5}(/|$)')
MECH_STORE = re.compile(r'\bflock\b|O_EXCL|\bcreate_new\b|WriteCreateNew|Mkdir\.|mkdir_[a-z_]*command|\bcreate_dir_all\b|\bfs::rename\b|Filesystem\.Rename|rm_recursive|Remove\.RecursiveForce')
MECH_CACHE_RS = re.compile(r'\bstatic\s+[A-Z_]+\s*:\s*(?:std::sync::)?(?:OnceLock|LazyLock|Mutex<(?:Hash|BTree)Map)|thread_local!|lazy_static!')
CACHE_IDENT = re.compile(r'\b[a-z_]*(?:memo(?!ry)|cache)[a-z_]*\b|\b[A-Za-z]*(?:Memo(?!ry)|Cache)[A-Za-z]*\b')

AUTH = {
  'storage': r'std\.(artifact_store|cache_interface|durable_compare_and_set|realization)|gunbc\.(durable_cas_file_store|filesystem_interface|owned_directory|managed_directory|roadmap\.dashboard_instance_directories)\b',
  'caching': r'std\.(materialization_ladder|materialization_provider|materialization_store|cache_interface)|extdeps\.realization\.cache_purity',
  'access': r'gunbc\.auth\.(access_token_source|gcp_secret_access|github_gcp_federation|secret_access_grant)|std\.resources\b',
  'authz': r'std\.(scoped_authorization|effect_grant)|gunbc\.auth\.(authorization_pattern_selection|privileged_effect_census|approval_[a-z_]+)',
  'placement': r'gunbc\.roadmap\.roadmap_dashboard_instance\b|product\.fabric\.selection|gunbc\.compute\.work_provider_local|gunbc\.live_deploy\.|gunbc\.fleet\.',
}
IMPORT = re.compile(r'^\s*import\s+([A-Za-z0-9_.]+)', re.M)

def levels(rel, text):
    rs = rel.endswith('.rs')
    code = strip_comments(text, rs)
    nostr = prose_free(code)
    lits = code_literals(code)
    imports = IMPORT.findall(code)
    lv = {d: 0 for d in AUTH}
    why = {d: [] for d in AUTH}
    def up(d, l, w):
        if l > lv[d]: lv[d] = l
        why[d].append(f'L{l}:{w}')
    for d, pat in AUTH.items():
        hit = [i for i in imports if re.match(pat, i)]
        if hit: up(d, 2, 'imports ' + ','.join(sorted(set(hit))[:3]))
    amb = AMBIENT_RS.findall(nostr) if rs else [m.group(0) for m in AMBIENT_DAG.finditer(nostr)]
    # storage
    BIN = re.compile(r'^/(usr/(local/)?s?bin|s?bin|opt/homebrew/bin)/[^/]+$')
    bins = sorted({l for l in lits if BIN.match(l)})
    if bins: up('placement', 4, 'upstream binary ' + '|'.join(bins[:4]))
    absp = sorted({l for l in lits if ABS_PATH.match(l) and not BIN.match(l)})
    if absp: up('storage', 4, 'abs ' + '|'.join(absp[:4]))
    writes = len(re.findall(r'Filesystem\.Write|std::fs::write|fs::write|File::create', nostr))
    if writes and lv['storage'] >= 2: up('storage', 5, f'{writes} raw writes beside store authority')
    elif writes: up('storage', 4, f'{writes} raw writes, no store authority')
    mech = sorted(set(MECH_STORE.findall(nostr)))
    if mech: up('storage', 5, 'mech ' + '|'.join(mech[:4]))
    # caching
    cm = MECH_CACHE_RS.findall(code) if rs else []
    if cm: up('caching', 5, f'{len(cm)} process-wide memo statics')
    ci = sorted({c for c in CACHE_IDENT.findall(nostr)})
    if ci and lv['caching'] < 2: up('caching', 3, 'cache vocab ' + '|'.join(ci[:4]))
    # access
    envs = len(ENV_READ_DAG.findall(nostr)) + len(re.findall(r'env::var', nostr))
    if envs: up('access', 4, f'{envs} env reads')
    cred = [l for l in lits if re.search(r'(token|secret|credential|\.pem$|\.key$|mac-key|oauth)', l, re.I) and ABS_PATH.match(l)]
    if cred: up('access', 4, 'credential path ' + '|'.join(cred[:3]))
    # authz
    sd = len(SUDO.findall(code))
    if sd: up('authz', 4, f'{sd} sudo sites')
    # placement
    pins = len(LIVE_PIN.findall(nostr))
    if pins: up('placement', 3, f'{pins} live-instance pins')
    hosts = sorted({l for l in lits if HOST.search(l)})
    if hosts: up('placement', 4, 'host ' + '|'.join(hosts[:4]))
    units = sorted({l for l in lits if UNIT.search(l) and '/' not in l[:1]})
    if units: up('placement', 4, 'unit ' + '|'.join(units[:3]))
    if any(l.startswith('/etc/systemd/') for l in lits): up('placement', 4, '/etc/systemd literal')
    return lv, why, len(amb), sorted(set(amb))[:6]

rows = []
for base in ('dag', 'src/v1', 'src/v2'):
    for dp, _, fs in os.walk(os.path.join(ROOT, base)):
        for f in fs:
            if not (f.endswith('.dag') or f.endswith('.rs')):
                continue
            p = os.path.join(dp, f); rel = os.path.relpath(p, ROOT)
            if rel.endswith('.rs') and not rel.startswith('src/v1/'):
                continue
            text = open(p, encoding='utf-8', errors='replace').read()
            lv, why, na, amb = levels(rel, text)
            signal = int(na > 0 or any(v >= 3 for v in lv.values()))
            rows.append([modname(rel), klass(rel), rel] + [f'L{lv[d]}' for d in AUTH] + [str(na), str(signal), ' ; '.join(w for d in AUTH for w in why[d] if not w.startswith('L2')), ','.join(amb)])
rows.sort()
with open(OUT, 'w') as o:
    o.write('\t'.join(['module', 'class', 'path', 'storage', 'caching', 'access', 'authz', 'placement', 'ambient', 'signal', 'leads', 'ambient_ops']) + '\n')
    for r in rows:
        o.write('\t'.join(x.replace('\t', ' ') for x in r) + '\n')
