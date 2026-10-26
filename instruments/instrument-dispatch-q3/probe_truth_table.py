import json, subprocess, urllib.error, os, sys
REPO='gunb-ai/gunbc'; WORKFLOW_FILE='instrument-dispatch.yml'
def api(path):
    out = subprocess.run(['gh','api',path], capture_output=True, text=True)
    if out.returncode != 0: raise urllib.error.HTTPError(path, 404, out.stderr.strip()[:80], None, None)
    return json.loads(out.stdout)
# OLD (buggy) predicate
def old(rid):
    try:
        w = api('/repos/%s/actions/runs/%s' % (REPO, rid))
        return (w.get('workflow') or {}).get('path') == 'workflows/' + WORKFLOW_FILE
    except urllib.error.HTTPError: return False
# NEW (fixed) predicate: top-level path, both endpoints agree on shape
def new(rid):
    try:
        w = api('/repos/%s/actions/runs/%s' % (REPO, rid))
        return w.get('path') == '.github/workflows/' + WORKFLOW_FILE
    except urllib.error.HTTPError: return False
cases = [
 ('40268668972', 'wm707 kit-e2e base (must ACCEPT)', True),
 ('40219194854', 'my base leg @9e4d16a5bc3 (must ACCEPT)', True),
 ('40205818229', 'wm707 tip-shard base (must ACCEPT)', True),
]
# find a foreign-workflow run for the negative case
runs = api('/repos/%s/actions/runs?per_page=50' % REPO)['workflow_runs']
for r in runs:
    if r.get('path') and not r['path'].endswith('instrument-dispatch.yml'):
        cases.append((str(r['id']), 'foreign workflow %s (must REJECT)' % r['path'], False)); break
# nonexistent run: HTTPError -> False
cases.append(('999999999999', 'nonexistent run (HTTPError->REJECT)', False))
ok=True
for rid, desc, want in cases:
    o,n = old(rid), new(rid)
    mark = 'OK ' if n==want else 'FAIL'
    if n!=want: ok=False
    print('%s run %s %-52s old=%s new=%s want=%s' % (mark, rid, desc, o, n, want))
sys.exit(0 if ok else 1)
