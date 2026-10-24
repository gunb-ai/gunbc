#!/usr/bin/env python3
"""Symbol-level storage-surface reader over the .dag corpus.

For each module: lex (strip // comments, separate string literals), parse import
bindings, and record every bound reference to a storage surface:
  - CAS library symbols imported from gunbc.durable_cas_file_store (root/fs-taking)
  - attempt_lifecycle consumers (SlotStore takers)
  - local_filesystem / FilesystemInterface fields used for writes
  - ambient Filesystem.Write* / Delete / LinkCreateNew
  - shell.Mkdir.*, shell.Copy/Move/Chmod/Remove.*, shell.Mktemp.* (scratch hint)
plus absolute-path string literals (durable root hints).
Output: TSV per module.
"""
import os, re, sys, json

ROOTS = sys.argv[1:] or ["dag", "src/v2"]

def lex(src):
    """Return (code_without_comments_and_strings, [string literals])."""
    out, strs, i, n = [], [], 0, len(src)
    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            i = n if j < 0 else j
            continue
        if src.startswith('r#"', i) or src.startswith('r"', i):
            hashes = 1 if src.startswith('r#"', i) else 0
            start = i + 2 + hashes
            end = src.find('"' + '#' * hashes, start)
            end = n if end < 0 else end
            strs.append(src[start:end]); out.append('""')
            i = end + 1 + hashes
            continue
        if c == '"':
            j, buf = i + 1, []
            while j < n and src[j] != '"':
                if src[j] == '\\' and j + 1 < n:
                    buf.append(src[j:j+2]); j += 2; continue
                buf.append(src[j]); j += 1
            strs.append("".join(buf)); out.append('""')
            i = j + 1
            continue
        out.append(c); i += 1
    return "".join(out), strs

IMPORT_RE = re.compile(r'\bimport\s+([A-Za-z_][\w.]*)\s*(\{([^}]*)\})?', re.S)
IDENT = re.compile(r'[A-Za-z_]\w*')

CAS = "gunbc.durable_cas_file_store"
ROOT_TAKING_CAS = None  # filled from the CAS module's own signatures

def module_name(code):
    m = re.search(r'^\s*module\s+([\w.]+)', code, re.M)
    return m.group(1) if m else None

def fn_sigs(code):
    """name -> param text for top-level fns."""
    sigs = {}
    for m in re.finditer(r'\bfn\s+([a-z_]\w*)\s*(<[^>]*>)?\s*\(', code):
        depth, j = 1, m.end()
        while j < len(code) and depth:
            if code[j] == '(':
                depth += 1
            elif code[j] == ')':
                depth -= 1
            j += 1
        sigs[m.group(1)] = code[m.end():j-1]
    return sigs

ABS = re.compile(r'^(/(var|srv|etc|opt|home|run|mnt|data|tmp|usr/local)\b[^\s"]*|~/[^\s"]*|\$HOME[^\s"]*|\$\{HOME\}[^\s"]*)')

files = []
for r in ROOTS:
    for d, _, fs in os.walk(r):
        for f in fs:
            if f.endswith(".dag"):
                files.append(os.path.join(d, f))

parsed = {}
for p in files:
    src = open(p, encoding="utf-8", errors="replace").read()
    code, strs = lex(src)
    parsed[p] = (code, strs)

# root/fs-taking CAS functions, read from the CAS module's own signatures
cas_path = "dag/gunbc/durable_cas_file_store.dag"
cas_code = parsed[cas_path][0]
ROOT_TAKING_CAS = sorted(n for n, ps in fn_sigs(cas_code).items()
                         if re.search(r'\broot\s*:', ps) or re.search(r'\bfs\s*:\s*FilesystemInterface', ps))
CAS_STORE_BUILDERS = {"file_slot_store_over", "file_slot_store", "slot_store_of", "slot_root_observe"}

rows = []
for p, (code, strs) in sorted(parsed.items()):
    mod = module_name(code) or "?"
    binds = {}
    for m in IMPORT_RE.finditer(code):
        src_mod, body = m.group(1), m.group(3)
        if body is None:
            binds[src_mod.split(".")[-1]] = (src_mod, "*")
            continue
        for item in body.split(","):
            item = item.strip()
            if not item:
                continue
            parts = item.split(" as ")
            name = parts[0].strip(); local = parts[-1].strip()
            binds[local] = (src_mod, name)
    # body = code minus import statements
    body = IMPORT_RE.sub(" ", code)
    idents = IDENT.findall(body)
    counts = {}
    for t in idents:
        counts[t] = counts.get(t, 0) + 1
    cas_refs = sorted({f"{binds[t][1]}" for t in counts if t in binds and binds[t][0] == CAS and binds[t][1] in ROOT_TAKING_CAS})
    cas_builders = sorted({binds[t][1] for t in counts if t in binds and binds[t][0] == CAS and binds[t][1] in CAS_STORE_BUILDERS})
    if mod == CAS:
        cas_refs, cas_builders = ["(self)"], ["(self)"]
    lifecycle = sorted({binds[t][1] for t in counts if t in binds and binds[t][0] == "gunbc.compute.attempt_lifecycle"})
    localfs = "local_filesystem" in binds and counts.get("local_filesystem", 0) > 0
    fs_writes = len(re.findall(r'\.(write_create_new_with_mode|write_create_new|write_owner_only|delete)\b', body))
    amb = {}
    for m in re.finditer(r'\bFilesystem\.(Write\w*|Delete|LinkCreateNew)\b', body):
        amb[m.group(1)] = amb.get(m.group(1), 0) + 1
    sh = {}
    for m in re.finditer(r'\bshell\.(Mkdir|Copy|Move|Chmod|Remove|Mktemp|Chown|Link|Install)(\.\w+)?', body):
        k = m.group(1) + (m.group(2) or "")
        sh[k] = sh.get(k, 0) + 1
    argv_mkdir = sum(1 for s in strs if s in ("install", "mkdir") ) + sum(1 for s in strs if re.match(r'^(install -d|mkdir -p)\b', s))
    abs_paths = sorted({s for s in strs if ABS.match(s)})
    rows.append(dict(path=p, module=mod, cas_refs=cas_refs, cas_builders=cas_builders, lifecycle=lifecycle,
                     local_filesystem=localfs, fs_field_writes=fs_writes, ambient=amb, shell=sh,
                     argv_mkdir=argv_mkdir, abs_paths=abs_paths))

json.dump(dict(root_taking_cas=ROOT_TAKING_CAS, rows=rows), sys.stdout, indent=0)
