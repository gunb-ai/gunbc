#!/usr/bin/env python3
"""Brace-depth-aware import stripper for .dag trees.

Removes whole `import a.b` / `import a.b { X, Y }` declarations, including the
multi-line brace form. Reports per-file and total counts so the strip itself is
denominated (a silently-partial strip reads as a corpus failure downstream).
"""
import sys, pathlib

def strip(text):
    out, removed, i = [], 0, 0
    lines = text.split("\n")
    while i < len(lines):
        line = lines[i]
        if line.lstrip().startswith("import "):
            depth = line.count("{") - line.count("}")
            i += 1
            while depth > 0 and i < len(lines):
                depth += lines[i].count("{") - lines[i].count("}")
                i += 1
            removed += 1
            continue
        out.append(line)
        i += 1
    return "\n".join(out), removed

def main(argv):
    manifest = None
    if "--manifest" in argv:
        i = argv.index("--manifest")
        manifest = argv[i + 1]
        argv = argv[:i] + argv[i + 2:]
    files = total = 0
    lines = []
    for root in argv:
        base = pathlib.Path(root)
        for p in sorted(base.rglob("*.dag")):
            src = p.read_text()
            new, n = strip(src)
            if n:
                p.write_text(new)
                files += 1
                total += n
                lines.append(f"{n}\t{p.relative_to(base)}")
    if manifest:
        # The manifest is the strip's own denominator: a silently-partial strip
        # reads downstream as a corpus failure, so what was removed is recorded
        # per file rather than asserted in aggregate.
        with open(manifest, "w") as f:
            f.write("imports_removed\tfile\n")
            f.write("\n".join(sorted(lines, key=lambda s: s.split("\t")[1])) + "\n")
    print(f"stripped {total} import declarations across {files} files")

if __name__ == "__main__":
    main(sys.argv[1:])
