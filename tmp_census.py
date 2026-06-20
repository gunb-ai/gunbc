#!/usr/bin/env python3
import hashlib
from pathlib import Path
from collections import defaultdict

ws = Path('.')
ITEM_KW = ("data ", "fn ", "func ", "type ", "service ", "const ", "pattern ", "resource ")


def normalize(body: str) -> str:
    lines = []
    for line in body.splitlines():
        line = line.split("//")[0].strip()
        if line:
            lines.append(line)
    return "\n".join(lines)


def hash_body(body: str) -> str:
    return hashlib.sha256(normalize(body).encode()).hexdigest()[:16]


def extract_decls(content: str):
    lines = content.splitlines()
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("test "):
            i += 1
            continue
        kw = next((k for k in ITEM_KW if line.startswith(k)), None)
        if not kw:
            i += 1
            continue
        rest = line[len(kw) :]
        name = ""
        for c in rest:
            if c.isalnum() or c == "_":
                name += c
            else:
                break
        if not name:
            i += 1
            continue
        body = [line]
        i += 1
        depth = line.count("{") - line.count("}")
        while i < len(lines):
            nxt = lines[i]
            if (
                depth <= 0
                and any(nxt.startswith(k) for k in ITEM_KW)
                and not nxt.startswith("test ")
            ):
                break
            body.append(nxt)
            depth += nxt.count("{") - nxt.count("}")
            i += 1
        out.append((name, hash_body("\n".join(body) + "\n")))
    return out


records = []
for tree, root in [("dsl", ws / "dsl"), ("v2", ws / "src/v2")]:
    for path in root.rglob("*.dag"):
        rel = str(path.relative_to(root)).replace("\\", "/")
        content = path.read_text()
        for name, h in extract_decls(content):
            records.append((f"{rel}:{name}", tree, h))

by_key: dict[str, dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
for key, tree, h in records:
    by_key[key][h].add(tree)

coexist = sorted(
    k
    for k, v in by_key.items()
    if "dsl" in {t for ts in v.values() for t in ts}
    and "v2" in {t for ts in v.values() for t in ts}
)
diverged = []
for k, v in by_key.items():
    trees = {t for ts in v.values() for t in ts}
    if "dsl" in trees and "v2" in trees and len(v) > 1:
        diverged.append(k)
diverged.sort()

print(f"coexistence count: {len(coexist)}")
print(f"diverged count: {len(diverged)}")
print("\n=== DIVERGED ===")
for k in diverged:
    print(k)
print("\n=== ALL COEXISTENCE ===")
for k in coexist:
    print(k)
