"""Symbol-level reader over a .dag tree at a git ref.

For every module: its imports (name -> source module), its top-level declarations, and every call
site attributed to (module, enclosing declaration, callee) with the callee resolved through the
module's imports. Comments and string literal contents are stripped before reading, so a name in
prose or a fixture string is not a call. String literals are kept separately per declaration so
a durable-root literal can be attributed to the declaration that holds it.

usage: reader.py <git-ref> <out.json>
"""
import json, re, subprocess, sys

ref, out = sys.argv[1], sys.argv[2]

def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True, check=True).stdout

paths = [p for p in git("ls-tree", "-r", "--name-only", ref, "--", "dag", "src/v2").splitlines() if p.endswith(".dag")]
blob = {}
# batch read
proc = subprocess.run(["git", "cat-file", "--batch"], input="".join(f"{ref}:{p}\n" for p in paths).encode(),
                      capture_output=True, check=True)
data = proc.stdout
i = 0
for p in paths:
    hdr_end = data.index(b"\n", i)
    hdr = data[i:hdr_end].split()
    size = int(hdr[2])
    blob[p] = data[hdr_end + 1: hdr_end + 1 + size].decode("utf-8", "replace")
    i = hdr_end + 1 + size + 1

TOK = re.compile(r'//[^\n]*|"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*|[{}()\[\]]|\n|\S')

def klass(p):
    if p.startswith("dag/test/"): return "test"
    if p.startswith("src/v2/"): return "compiler"
    if "/instruments/" in p: return "instrument"
    if any(s in p for s in ("recurring_failure_mode/", "rung_drop/", "/plans/", "design_ruling")): return "ledger"
    if p.startswith("dag/extdeps/"): return "extdeps"
    return "production"

mods = {}
blob_names_cache = {}
for p, src in blob.items():
    toks = [t for t in TOK.findall(src) if not t.startswith("//")]
    module = None
    imports = {}
    decls = {}  # name -> {kind, calls:[...], strings:[...], line}
    depth = 0
    cur = None
    j = 0
    n = len(toks)
    line = 1
    while j < n:
        t = toks[j]
        if t == "\n":
            line += 1; j += 1; continue
        if depth == 0:
            if t == "module" and j + 1 < n:
                module = toks[j + 1]; j += 2; continue
            if t == "import" and j + 1 < n:
                src_mod = toks[j + 1]; k = j + 2
                while k < n and toks[k] == "\n": k += 1
                if k < n and toks[k] == "{":
                    k += 1
                    while k < n and toks[k] != "}":
                        tk = toks[k]
                        if re.match(r"[A-Za-z_]", tk) and tk not in ("as",):
                            imports[tk.split(".")[0]] = src_mod
                        k += 1
                    j = k + 1; continue
                j += 2; continue
            if t in ("fn", "data", "type") and j + 1 < n:
                name = toks[j + 1]
                cur = name
                decls.setdefault(name, {"kind": t, "calls": [], "strings": [], "line": line})
                j += 2; continue
        if t in "{([":
            depth += 1
        elif t in "})]":
            depth = max(0, depth - 1)
        elif t.startswith('"'):
            if cur: decls[cur]["strings"].append(t[1:-1])
        elif cur and re.match(r"[A-Za-z_]", t):
            k = j + 1
            if k < n and toks[k] == "(":
                head = t.split(".")[0]
                if "." in t and head[0].isupper():
                    res = ("effect", t)
                elif "." in t:
                    res = ("qualified", t)
                elif t in imports:
                    res = (imports[t], t)
                elif t in decls or t in blob_names_cache.get(p, ()):
                    res = (module, t)
                else:
                    res = (module or "?", t)  # local or builtin; resolved below
                decls[cur]["calls"].append({"to_mod": res[0], "name": res[1], "line": line})
            elif t not in decls:
                # a bare reference (e.g. a data row used as a value): record as ref
                decls[cur].setdefault("refs", [])
                if t in imports:
                    decls[cur]["refs"].append({"to_mod": imports[t], "name": t})
        j += 1
    mods[module or p] = {"path": p, "class": klass(p), "imports": imports, "decls": decls}

blob_names_cache = {}
json.dump(mods, open(out, "w"))
print(len(mods), "modules")
