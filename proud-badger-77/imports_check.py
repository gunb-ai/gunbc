import re
import subprocess

REV = "021bbda1"
WT = "/home/briansrls/.worktrees/proud-badger-77"


def show(path, rev=REV):
    r = subprocess.run(["git", "-C", WT, "show", f"{rev}:{path}"],
                       capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def top_level_names(text):
    """Names an import list may legally name: top-level declarations plus
    sum-type alternatives, which are written as bare `| Variant` lines."""
    if text is None:
        return None
    names = set()
    for m in re.finditer(r'^(?:pub\s+)?(data|fn|type|op|test fn|relation|const)\s+([A-Za-z_0-9]+)', text, re.M):
        names.add(m.group(2))
    for m in re.finditer(r'^\s{2,}([A-Z][A-Za-z_0-9]*)\b', text, re.M):
        names.add(m.group(1))
    for m in re.finditer(r'^\s*\|\s*([A-Z][A-Za-z_0-9]*)', text, re.M):
        names.add(m.group(1))
    return names


def check(label, lines):
    bad, total, cache = [], 0, {}
    for line in lines:
        m = re.match(r'\s*import\s+([a-z_0-9.]+)\s*\{([^}]*)\}', line)
        if not m:
            continue
        mod, raw = m.group(1), m.group(2)
        defined = None
        for path in ("src/" + mod.replace(".", "/") + ".dag",
                     "dag/" + mod.replace(".", "/") + ".dag"):
            if path not in cache:
                cache[path] = top_level_names(show(path))
            if cache[path] is not None:
                defined = cache[path]
                break
        for name in [n.strip() for n in raw.split(",") if n.strip()]:
            total += 1
            if defined is None:
                bad.append(f"{label}: MODULE MISSING {mod}")
            elif name not in defined:
                bad.append(f"{label}: {mod} :: {name} not a top-level name")
    print(f"{label}: {total} imported names checked, {len(bad)} problems")
    for b in bad:
        print("   !!", b)
    return bad


def git_show(spec):
    return subprocess.run(["git", "-C", WT, "show", spec],
                          capture_output=True, text=True).stdout.splitlines()


allbad = (check("probe",
                git_show(f"{REV}:src/v2/test/claim/compiler/nominal_application_child_edge_shape.dag"))
          + check("contract",
                  git_show("HEAD:src/v2/test/claim/compiler/nominal_application_child_edge_contract.dag")))
print("TOTAL PROBLEMS:", len(allbad))
