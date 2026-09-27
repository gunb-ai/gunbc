"""Test-only import closure staging shared by offline Namecheap controls."""
import pathlib
import re
import shutil


def stage_import_closure(entry, destination):
    root = pathlib.Path.cwd()
    index = {}
    for tree in ('dag', 'src/v2'):
        for source in (root / tree).rglob('*.dag'):
            content = source.read_text()
            module = re.search(r'^module\s+([\w.]+)', content, re.M)
            if module:
                index[module[1]] = (source, content)
    pending, seen = [entry], set()
    while pending:
        source = pending.pop()
        if source in seen:
            continue
        seen.add(source)
        for module in re.findall(r'^import\s+([\w.]+)', source.read_text(), re.M):
            if module not in index:
                raise RuntimeError('Unresolved fixture import: ' + module)
            pending.append(index[module][0])
    for source in seen:
        copied = destination / source.relative_to(root)
        copied.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, copied)
    return destination / entry.relative_to(root)
