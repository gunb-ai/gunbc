#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell log classifier; until then it
# projects per-error E0599 census from PROBE_KEEP_LOG_DIR/*.cargo.log (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
# Authority: dag/tools/e0599_probe_census.dag (e0599_message_pattern_rows + e0599_root_family_for).
# Root-family labels are derived by calling e0599_root_family_label_for_row via gunbc — not
# reimplemented in this script. Regex patterns MUST stay in sync with e0599_message_pattern_rows.
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

export E0599_CENSUS_ROOT="$ROOT"
export E0599_CENSUS_GUNBC="$GUNBC"

python3 - "$AGGREGATE" "$@" <<'PY'
# Patterns below mirror tools.e0599_probe_census e0599_message_pattern_rows (ordinal 1-5).
# Root-family labels are fetched from tools.e0599_probe_census via gunbc (single authority).
import collections
import pathlib
import re
import subprocess
import sys

aggregate = sys.argv[1] == "1"
paths = [pathlib.Path(p) for p in sys.argv[2:]]

root = pathlib.Path(__import__("os").environ["E0599_CENSUS_ROOT"])
gunbc = __import__("os").environ["E0599_CENSUS_GUNBC"]

RE_MISSING = re.compile(
    r"error\[E0599\]: no method named `([^`]+)` found for (.+?) in the current scope"
)
RE_BOUNDS = re.compile(
    r"error\[E0599\]: the method `([^`]+)` exists for (.+?), but its trait bounds were not satisfied"
)
RE_VARIANT = re.compile(
    r"error\[E0599\]: no variant(?:, associated function, or constant)? named `([^`]+)` found for (.+?)(?: in the current scope)?$"
)
RE_ASSOC = re.compile(
    r"error\[E0599\]: no function or associated item named `([^`]+)` found for (.+?)(?: in the current scope)?$"
)
RE_OTHER = re.compile(r"error\[E0599\]: (.+)")

_root_family_cache: dict[tuple[str, str, str], str] = {}


def normalize_receiver(raw: str) -> str:
    return " ".join(raw.strip().split())


def root_family_for(shape: str, method: str, receiver: str) -> str:
    key = (shape, method, receiver)
    cached = _root_family_cache.get(key)
    if cached is not None:
        return cached
    proc = subprocess.run(
        [
            gunbc,
            "run",
            "--source-root",
            str(root / "dag"),
            "--entry",
            "dag/tools/e0599_probe_census.dag",
            "--function",
            "e0599_root_family_label_for_row",
            "--arg",
            f"shape_str={shape}",
            "--arg",
            f"method={method}",
            "--arg",
            f"receiver={receiver}",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    label = None
    for line in proc.stdout.splitlines():
        if line.startswith("running "):
            continue
        if line.startswith("error:"):
            break
        if line.strip():
            label = line.strip()
            break
    if label is None:
        raise RuntimeError(
            f"e0599_root_family_label_for_row failed for {key!r}: "
            f"stdout={proc.stdout!r} stderr={proc.stderr!r}"
        )
    _root_family_cache[key] = label
    return label


def classify_line(line: str):
    for rx, shape in (
        (RE_MISSING, "missing_method"),
        (RE_BOUNDS, "bounds_unsatisfied"),
        (RE_VARIANT, "no_variant"),
        (RE_ASSOC, "no_assoc_fn"),
    ):
        m = rx.search(line)
        if m:
            return shape, m.group(1), normalize_receiver(m.group(2))
    m = RE_OTHER.search(line)
    if m:
        return "other", "?", normalize_receiver(m.group(1))
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
    print(f"# module={mod} total_E0599={len(rows)}")
    print("module\tfailure_shape\tmethod\treceiver_carrier\troot_family\tcount")
    for (shape, method, receiver), n in sorted(
        counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1], kv[0][2])
    ):
        family = root_family_for(shape, method, receiver)
        print(f"{mod}\t{shape}\t{method}\t{receiver}\t{family}\t{n}")
PY
