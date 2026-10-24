#!/usr/bin/env python3
"""Storage-effect site reader over .dag sources.

Strips // comments (outside strings) and string-literal contents, then records every
storage effect call site with its module, enclosing top-level fn, line, and the raw
argument text (from the ORIGINAL source, so literal paths stay visible).

Effects (Filesystem.*, shell.*) are effect primitives: they are not import-bound names
and cannot be aliased, so a lexical read with comments and strings removed is the
symbol grain for them. CAS library calls are imports and are also recorded here, with
the import list, so the binding receipt can be cross-checked.
"""
import os, re, sys, json

ROOT = sys.argv[1]
OUT = sys.argv[2]

EFFECTS = re.compile(r'\b(Filesystem\.(?:Write|WriteCreateNew|WriteOwnerOnly|WriteBytes|WriteCreateNewWithMode|LinkCreateNew|Delete)'
                     r'|shell\.(?:Mkdir\.[A-Za-z]+|Remove\.[A-Za-z]+|Move\.[A-Za-z]+|Copy\.[A-Za-z]+|Chmod\.[A-Za-z]+|Mktemp\.[A-Za-z]+|Link\.[A-Za-z]+))\s*\(')
CAS_FNS = None  # filled from the library module

def strip(src):
    """Return src with comments blanked and string contents replaced by spaces (same length)."""
    out = list(src)
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == '/' and i + 1 < n and src[i+1] == '/':
            j = src.find('\n', i)
            j = n if j < 0 else j
            for k in range(i, j): out[k] = ' '
            i = j
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                if src[j] == '\\': j += 1
                j += 1
            for k in range(i+1, min(j, n)):
                if out[k] != '\n': out[k] = ' '
            i = j + 1
        else:
            i += 1
    return ''.join(out)

def args_at(src, start):
    """Raw text of the balanced (...) beginning at src[start]=='('."""
    depth, i = 0, start
    instr = False
    while i < len(src):
        c = src[i]
        if instr:
            if c == '\\': i += 1
            elif c == '"': instr = False
        elif c == '"': instr = True
        elif c == '(': depth += 1
        elif c == ')':
            depth -= 1
            if depth == 0: return src[start+1:i]
        i += 1
    return src[start+1:start+200]

FN_RE = re.compile(r'^(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)', re.M)
DATA_RE = re.compile(r'^(?:pub\s+)?data\s+([A-Za-z_][A-Za-z0-9_]*)', re.M)
MOD_RE = re.compile(r'^module\s+([A-Za-z0-9_.]+)', re.M)
IMPORT_RE = re.compile(r'^import\s+([A-Za-z0-9_.]+)(\s*\{([^}]*)\})?', re.M)

lib = open(os.path.join(ROOT, 'dag/gunbc/durable_cas_file_store.dag')).read()
CAS_FNS = sorted(set(FN_RE.findall(strip(lib))))

rows = []
for base in ('dag', 'src/v2'):
    for d, _, fs in os.walk(os.path.join(ROOT, base)):
        for f in fs:
            if not f.endswith('.dag'): continue
            p = os.path.join(d, f)
            rel = os.path.relpath(p, ROOT)
            src = open(p, encoding='utf-8', errors='replace').read()
            s = strip(src)
            m = MOD_RE.search(s)
            mod = m.group(1) if m else rel
            heads = sorted([(mm.start(), 'fn ' + mm.group(1)) for mm in FN_RE.finditer(s)] +
                           [(mm.start(), 'data ' + mm.group(1)) for mm in DATA_RE.finditer(s)])
            def owner(pos):
                o = '-'
                for st, name in heads:
                    if st <= pos: o = name
                    else: break
                return o
            def line(pos): return s.count('\n', 0, pos) + 1
            for mm in EFFECTS.finditer(s):
                a = args_at(src, mm.end() - 1)
                rows.append(dict(module=mod, path=rel, kind='effect', symbol=mm.group(1), owner=owner(mm.start()),
                                 line=line(mm.start()), args=' '.join(a.split())[:240]))
            # CAS library: selective imports and qualified uses
            imported = set()
            for im in IMPORT_RE.finditer(s):
                if im.group(1) == 'gunbc.durable_cas_file_store':
                    if im.group(3):
                        imported |= {x.strip() for x in im.group(3).split(',') if x.strip()}
                    else:
                        imported |= set(CAS_FNS)  # whole-module import: any name may be used
            if mod == 'gunbc.durable_cas_file_store':
                continue
            for fn in CAS_FNS:
                pat = re.compile(r'(?<![A-Za-z0-9_.])(?:gunbc\.durable_cas_file_store\.)?' + re.escape(fn) + r'\s*\(')
                for mm in pat.finditer(s):
                    q = s[max(0, mm.start()-30):mm.start()].endswith('gunbc.durable_cas_file_store.')
                    if fn in imported or 'gunbc.durable_cas_file_store.' in s[max(0,mm.start()-31):mm.end()]:
                        rows.append(dict(module=mod, path=rel, kind='cas', symbol=fn, owner=owner(mm.start()),
                                         line=line(mm.start()), args=' '.join(args_at(src, mm.end()-1).split())[:240]))
with open(OUT, 'w') as o:
    for r in rows:
        o.write('\t'.join(str(r[k]) for k in ('module','path','kind','symbol','owner','line','args')) + '\n')
print(len(rows), 'rows;', len({r['module'] for r in rows}), 'modules;', len(CAS_FNS), 'cas fns')
