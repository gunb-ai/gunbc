#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell log classifier; until then it
# projects per-error E0599 census from PROBE_KEEP_LOG_DIR/*.cargo.log (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
# Authority: dag/tools/e0599_probe_census.dag — patterns via e0599_write_message_pattern_rows_blob,
# root-family labels via e0599_write_root_family_labels_from_blob (ProcessExit gunbc exports).
# Witness: dag/test/claim/e0599_probe_census_witness_test.dag. Frozen output receipt (not authority):
# docs/probes/e0599_canonical_seven_census_2026-07-26.tsv. Input logs from
# docs/probes/curated_cargo_probe_one.sh PROBE_KEEP_LOG_DIR hook.
# Inline python avoids a committed .py file (gitignore_authority models *.py as local-dev-only).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 [--aggregate] <cargo.log> [...]" >&2
  exit 2
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
if [[ ! -x "$GUNBC" ]]; then
  echo "error: gunbc not found at $GUNBC (build v1-compiler --bin gunbc)" >&2
  exit 2
fi

AGGREGATE=0
if [[ "${1:-}" == "--aggregate" ]]; then
  AGGREGATE=1
  shift
fi

if [[ $# -lt 1 ]]; then
  echo "error: no cargo.log paths provided" >&2
  exit 2
fi

export E0599_CENSUS_ROOT="$ROOT"
export E0599_CENSUS_GUNBC="$GUNBC"

python3 - "$AGGREGATE" "$@" <<'PY'
import collections
import pathlib
import re
import subprocess
import sys
import tempfile

aggregate = sys.argv[1] == "1"
paths = [pathlib.Path(p) for p in sys.argv[2:]]
if not paths:
    print("error: no cargo.log paths provided", file=sys.stderr)
    sys.exit(2)
for path in paths:
    if not path.is_file():
        print(f"error: cargo.log not found: {path}", file=sys.stderr)
        sys.exit(2)

root = pathlib.Path(__import__("os").environ["E0599_CENSUS_ROOT"])
gunbc = __import__("os").environ["E0599_CENSUS_GUNBC"]

_root_family_cache: dict[tuple[str, str, str], str] = {}


def gunbc_write(function: str, out_path: str, **kwargs: str) -> str:
    cmd = [
        gunbc,
        "run",
        "--source-root",
        str(root / "dag"),
        "--entry",
        "dag/tools/e0599_probe_census.dag",
        "--function",
        function,
        "--arg",
        f"out_path={out_path}",
    ]
    for key, value in kwargs.items():
        cmd.extend(["--arg", f"{key}={value}"])
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(
            f"gunbc {function} failed (exit {proc.returncode}): "
            f"stdout={proc.stdout!r} stderr={proc.stderr!r}"
        )
    return pathlib.Path(out_path).read_text(encoding="utf-8")


def load_message_patterns() -> list[tuple[str, re.Pattern[str]]]:
    with tempfile.NamedTemporaryFile(prefix="e0599_patterns_", delete=False) as tmp:
        out_path = tmp.name
    try:
        blob = gunbc_write("e0599_write_message_pattern_rows_blob", out_path)
    finally:
        pathlib.Path(out_path).unlink(missing_ok=True)
    patterns: list[tuple[str, re.Pattern[str]]] = []
    for line in blob.splitlines():
        if not line.strip():
            continue
        shape, pattern = line.split("\t", 1)
        patterns.append((shape, re.compile(pattern)))
    if not patterns:
        raise RuntimeError("e0599_message_pattern_rows_blob returned no pattern rows")
    return patterns


MESSAGE_PATTERNS = load_message_patterns()


def normalize_receiver(raw: str) -> str:
    return " ".join(raw.strip().split())


def gunbc_root_family_labels(keys: list[tuple[str, str, str]]) -> dict[tuple[str, str, str], str]:
    if not keys:
        return {}
    blob = "\n".join(f"{shape}\t{method}\t{receiver}" for shape, method, receiver in keys)
    with tempfile.NamedTemporaryFile(prefix="e0599_labels_", delete=False) as tmp:
        out_path = tmp.name
    try:
        out = gunbc_write("e0599_write_root_family_labels_from_blob", out_path, blob=blob)
    finally:
        pathlib.Path(out_path).unlink(missing_ok=True)
    labels = out.splitlines() if out else []
    if len(labels) != len(keys):
        raise RuntimeError(
            f"e0599_root_family_labels_from_blob returned {len(labels)} labels for {len(keys)} keys: "
            f"stdout={out!r}"
        )
    return dict(zip(keys, labels))


def root_family_for(shape: str, method: str, receiver: str) -> str:
    key = (shape, method, receiver)
    cached = _root_family_cache.get(key)
    if cached is not None:
        return cached
    labels = gunbc_root_family_labels([key])
    label = labels[key]
    _root_family_cache[key] = label
    return label


def prefetch_root_families(keys: list[tuple[str, str, str]]) -> None:
    missing = [key for key in keys if key not in _root_family_cache]
    if not missing:
        return
    labels = gunbc_root_family_labels(missing)
    _root_family_cache.update(labels)


def classify_line(line: str):
    for shape, rx in MESSAGE_PATTERNS:
        m = rx.search(line)
        if not m:
            continue
        if shape == "other":
            return shape, "?", normalize_receiver(m.group(1))
        return shape, m.group(1), normalize_receiver(m.group(2))
    return None


def parse_log(path: pathlib.Path):
    rows = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        hit = classify_line(line)
        if hit:
            rows.append(hit)
    return rows


def module_from_log(path: pathlib.Path) -> str:
    return path.name.removesuffix(".cargo.log")


if aggregate or len(paths) > 1:
    per_module = {}
    global_counts = collections.Counter()
    module_totals = {}
    for path in sorted(paths):
        mod = module_from_log(path)
        rows = parse_log(path)
        per_module[mod] = rows
        module_totals[mod] = len(rows)
        for shape, method, receiver in rows:
            global_counts[(shape, method, receiver)] += 1
    prefetch_root_families(list(global_counts.keys()))
    print("# e0599_canonical_seven_census aggregate")
    print("module\ttotal_E0599")
    for mod in sorted(module_totals):
        print(f"{mod}\t{module_totals[mod]}")
    print(f"TOTAL\t{sum(module_totals.values())}")
    print()
    print("failure_shape\tmethod\treceiver_carrier\troot_family\ttotal_count\tmodules_hit")
    mod_sets = collections.defaultdict(set)
    for mod, rows in per_module.items():
        seen = {(s, m, r) for s, m, r in rows}
        for key in seen:
            mod_sets[key].add(mod)
    for key, n in sorted(global_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        shape, method, receiver = key
        family = root_family_for(shape, method, receiver)
        print(f"{shape}\t{method}\t{receiver}\t{family}\t{n}\t{len(mod_sets[key])}")
    print()
    print("root_family\ttotal_count")
    family_counts = collections.Counter()
    for (shape, method, receiver), n in global_counts.items():
        family_counts[root_family_for(shape, method, receiver)] += n
    for family, n in sorted(family_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        print(f"{family}\t{n}")
else:
    path = paths[0]
    mod = module_from_log(path)
    rows = parse_log(path)
    counts = collections.Counter()
    for shape, method, receiver in rows:
        counts[(shape, method, receiver)] += 1
    prefetch_root_families(list(counts.keys()))
    print(f"# module={mod} total_E0599={len(rows)}")
    print("module\tfailure_shape\tmethod\treceiver_carrier\troot_family\tcount")
    for (shape, method, receiver), n in sorted(
        counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1], kv[0][2])
    ):
        family = root_family_for(shape, method, receiver)
        print(f"{mod}\t{shape}\t{method}\t{receiver}\t{family}\t{n}")
PY
