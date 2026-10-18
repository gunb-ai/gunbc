#!/usr/bin/env python3
"""Pre-flight for the defect class that cost a 35-minute dispatch: main DELETES a name, my branch still
USES it, and git merges it silently because the two edits are in different regions.

The regen is the only thing that normally notices, ~30 minutes and one runner slot later. This asks the
same question locally in under a second: which names did main delete since the merge base, and does my tree
still reference any of them?

It is deliberately over-broad (a name 'deleted' by main may be re-added under a different definition, and a
line-based scan sees identifiers inside strings). Every hit is therefore reported with its use site and must
be read, not filtered away -- a false positive costs a second of reading, a miss costs a dispatch.
"""
import re, subprocess, sys, collections

BASE = sys.argv[1]           # merge base
MAIN = sys.argv[2]          # usually origin/main
FILES = sys.argv[3:] or ['src/v1/04_infer.dag', 'src/v1/04_types.dag']

def sh(*a):
    return subprocess.run(a, capture_output=True, text=True).stdout

# 1. names whose DEFINITION line main removed, since the base.
diff = sh('git', 'diff', BASE, MAIN, '--')  # whole-tree: main may delete a name in a file I never touched
removed = set()
for line in diff.splitlines():
    if line.startswith('-') and not line.startswith('---'):
        m = re.match(r'^-\s*(?:pub\s+)?(fn|type|struct|enum|const)\s+([A-Za-z0-9_]+)', line)
        if m: removed.add(m.group(2))

# 2. does my working tree still USE any of them, in a file this PR carries?
hits = collections.Counter(); shown = {}
for f in FILES:
    try:
        body = open(f, encoding='utf-8', errors='replace').read()
    except OSError:
        continue
    # strip comments so a mention inside an explanatory comment is not a use
    body = re.sub(r'//[^\n]*', '', body)
    for name in removed:
        n = len(re.findall(r'\b%s\b' % re.escape(name), body))
        if n:
            hits[(f, name)] = n
            shown[(f, name)] = [i + 1 for i, l in enumerate(body.splitlines()) if re.search(r'\b%s\b' % re.escape(name), l)][:4]

print("main deleted %d definition name(s) since %s" % (len(removed), BASE[:11]))
# KNOWN NOISE, stated so a reader does not trust the hit count. A whole-tree line diff sees a definition
# MOVED to another file as a deletion, and it cannot see enum VARIANTS (written `| Name`, not `fn Name`), so
# both moved functions and restored variants show up. The run that found the real defect printed 19 hits and
# exactly 1 was genuine; the discriminator is cheap and is the check to run on any hit:
#   grep -rEc "^\\s*(pub\\s+)?(fn|type|struct|enum)\\s+NAME\\b|^  \\| NAME" --include=*.dag src/
# 0 hits everywhere = the name really is undefined; >=1 = the checker was guessing about a move.
for (f, name), n in list(hits.items()):
    import subprocess as _sp
    r = _sp.run(['grep', '-rEc', r'^\s*(pub\s+)?(fn|type|struct|enum)\s+%s\b|^  \| %s' % (name, name),
                 '--include=*.dag', 'src/'], capture_output=True, text=True)
    if sum(int(l.split(':')[-1] or 0) for l in r.stdout.splitlines()) > 0:
        del hits[(f, name)]   # defined somewhere in the tree: a move, not a deletion
if not hits:
    print("CLEAN after the definition-exists filter (all hits were moves/variant blindness)")
    sys.exit(0)
if not hits:
    print("CLEAN: none of them is still used by %s" % ", ".join(FILES))
    sys.exit(0)
for (f, name), n in hits.most_common():
    print("  USED-BUT-DELETED  %-22s %-46s %d use(s) at lines %s" % (f.split('/')[-1], name, n, shown[(f, name)]))
sys.exit(1)
