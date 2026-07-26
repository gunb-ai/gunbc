#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell log classifier; until then it
# projects per-error E0599 census from PROBE_KEEP_LOG_DIR/*.cargo.log (probe-only).
# dissolve-on alt: modeled cssl_probe transport in .dag (same spine as curated_cargo_probe_one.sh).
# Authority: dag/tools/e0599_probe_census.dag (e0599_message_pattern_rows + e0599_root_family_for).
# Realization MUST keep regex patterns in sync with e0599_message_pattern_rows — witness:
# dag/test/claim/e0599_probe_census_witness_test.dag. Frozen output receipt (not authority):
# docs/probes/e0599_canonical_seven_census_2026-07-26.tsv. Input logs from
# docs/probes/curated_cargo_probe_one.sh PROBE_KEEP_LOG_DIR hook.
# Inline python avoids a committed .py file (gitignore_authority models *.py as local-dev-only).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 [--aggregate] <cargo.log> [...]" >&2
  exit 2
fi

AGGREGATE=0
if [[ "${1:-}" == "--aggregate" ]]; then
  AGGREGATE=1
  shift
fi

python3 - "$AGGREGATE" "$@" <<'PY'
# Patterns below mirror tools.e0599_probe_census e0599_message_pattern_rows (ordinal 1-5).
import collections, pathlib, re, sys

aggregate = sys.argv[1] == "1"
paths = [pathlib.Path(p) for p in sys.argv[2:]]

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


def normalize_receiver(raw: str) -> str:
    return " ".join(raw.strip().split())


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
    print("failure_shape\tmethod\treceiver_carrier\ttotal_count\tmodules_hit")
    mod_sets = collections.defaultdict(set)
    for mod, rows in per_module.items():
        seen = {(s, m, r) for s, m, r in rows}
        for key in seen:
            mod_sets[key].add(mod)
    for key, n in sorted(global_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        shape, method, receiver = key
        print(f"{shape}\t{method}\t{receiver}\t{n}\t{len(mod_sets[key])}")
else:
    path = paths[0]
    mod = module_from_log(path)
    rows = parse_log(path)
    counts = collections.Counter()
    for shape, method, receiver in rows:
        counts[(shape, method, receiver)] += 1
    print(f"# module={mod} total_E0599={len(rows)}")
    print("module\tfailure_shape\tmethod\treceiver_carrier\tcount")
    for (shape, method, receiver), n in sorted(
        counts.items(), key=lambda kv: (-kv[1], kv[0][0], kv[0][1], kv[0][2])
    ):
        print(f"{mod}\t{shape}\t{method}\t{receiver}\t{n}")
PY
