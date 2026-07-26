#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires hand-shell probe runners; until then this is a
# POST-PASS only over cargo logs kept by curated_cargo_probe_one.sh (PROBE_KEEP_LOG_DIR).
# Authority: cssl_v1_compiled_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag —
# the emit→assemble→cargo spine is NOT reimplemented here (E0277 census precedent, PR #7280).
#
# usage:
#   PROBE_KEEP_LOG_DIR=docs/probes/e0369_census_2026-07-26/logs \
#     docs/probes/e0369_extract_shapes.sh docs/probes/e0369_census_2026-07-26/shapes
#
# Produce the kept logs first, e.g.:
#   export CSSL_STD_SEED_LINK=1
#   export PROBE_KEEP_LOG_DIR=docs/probes/e0369_census_YYYY-MM-DD/logs
#   for m in ...canonical seven...; do docs/probes/curated_cargo_probe_one.sh "$m"; done
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <shapes-out-dir> [log-dir]" >&2
  echo "  log-dir defaults to \$PROBE_KEEP_LOG_DIR" >&2
  exit 2
fi

OUT_DIR="$1"
LOG_DIR="${2:-${PROBE_KEEP_LOG_DIR:-}}"
if [[ -z "$LOG_DIR" ]]; then
  echo "error: pass log-dir or set PROBE_KEEP_LOG_DIR" >&2
  exit 2
fi
mkdir -p "$OUT_DIR"

python3 - "$LOG_DIR" "$OUT_DIR" <<'PY'
import re, sys, collections, pathlib
log_dir, out_dir = map(pathlib.Path, sys.argv[1:3])
logs = sorted(log_dir.glob("*.cargo.log"))
if not logs:
    sys.exit(f"no *.cargo.log under {log_dir}")

for blog in logs:
    stem = blog.name.removesuffix(".cargo.log")
    log = blog.read_text(errors="replace")
    blocks = re.split(r"(?=^error\[E0369\])", log, flags=re.M)
    shapes = collections.Counter()
    details = []
    for b in blocks:
        if not b.startswith("error[E0369]"):
            continue
        first = b.splitlines()[0]
        msg = first.split("error[E0369]:", 1)[1].strip()
        shapes[msg] += 1
        m = re.search(r"-->\s+([^:\s]+):(\d+):(\d+)", b)
        path = line = col = None
        if m:
            path, line, col = m.group(1), int(m.group(2)), int(m.group(3))
        left = right = None
        for lm in re.finditer(r"left-hand side has type `([^`]+)`", b):
            left = lm.group(1)
        for rm in re.finditer(r"right-hand side has type `([^`]+)`", b):
            right = rm.group(1)
        if left is None or right is None:
            types = re.findall(r"has type `([^`]+)`", b)
            if len(types) >= 2:
                left, right = types[0], types[1]
            elif len(types) == 1 and left is None:
                left = types[0]
        single = None
        sm = re.search(r"cannot be applied to type `([^`]+)`", msg)
        if sm:
            single = sm.group(1)
        code = ""
        for ln in b.splitlines():
            if re.match(r"\s*\d+\s*\|\s+\S", ln):
                code = ln.split("|", 1)[1].strip()
                break
        details.append({
            "msg": msg, "path": path, "line": line, "col": col,
            "left": left, "right": right, "single": single, "code": code,
        })

    with (out_dir / f"{stem}.shapes.tsv").open("w") as fh:
        fh.write("count\tmessage\n")
        for msg, n in shapes.most_common():
            fh.write(f"{n}\t{msg}\n")
    with (out_dir / f"{stem}.instances.tsv").open("w") as fh:
        fh.write("message\tpath\tline\tcol\tleft\tright\tsingle\tcode\n")
        for d in details:
            fh.write("\t".join([
                d["msg"].replace("\t", " "),
                d["path"] or "",
                str(d["line"] or ""),
                str(d["col"] or ""),
                (d["left"] or "").replace("\t", " "),
                (d["right"] or "").replace("\t", " "),
                (d["single"] or "").replace("\t", " "),
                (d["code"] or "").replace("\t", " "),
            ]) + "\n")
    print(f"{stem}: {sum(shapes.values())} E0369 across {len(shapes)} shapes", file=sys.stderr)
PY
