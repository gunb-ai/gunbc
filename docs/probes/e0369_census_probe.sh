#!/usr/bin/env bash
# E0369 census probe — same protocol as Phase C / curated_cargo_probe_one.sh
# (CSSL_STD_SEED_LINK=1, empty shim). Retains cargo.log and extracts E0369
# message shapes + operand-type spans. READ-ONLY census; no emitter edits.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$ROOT/docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh"

STAMP_ARG="${1:-$ROOT/docs/probes/e0369_census_$(date -u +%Y-%m-%d)}"
mkdir -p "$STAMP_ARG"
STAMP_DIR="$(cd "$STAMP_ARG" && pwd)"
mkdir -p "$STAMP_DIR/logs" "$STAMP_DIR/emitted" "$STAMP_DIR/shapes"

export CSSL_STD_SEED_LINK=1
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
CSSL_ASSEMBLE="${CSSL_ASSEMBLE:-$ROOT/target/release/cssl_assemble}"
[[ -x "$GUNBC" ]] || { echo "missing gunbc at $GUNBC" >&2; exit 1; }
[[ -x "$CSSL_ASSEMBLE" ]] || { echo "missing cssl_assemble at $CSSL_ASSEMBLE" >&2; exit 1; }

MODULES=(
  src/v2/compiler/06_translate.dag
  src/v2/compiler/04_infer.dag
  src/v2/compiler/05_eval.dag
  src/v2/compiler/05_emit.dag
  src/v2/compiler/emit_host.dag
  src/v2/compiler/emit_module.dag
  src/v2/compiler/materialization_carriers.dag
)

{
  echo -e "label\tE0369_census"
  echo -e "classifier_stamp\trule1-first-error-plus-residual-histogram-v3-uncoded-split"
  echo -e "protocol\tCSSL_STD_SEED_LINK=1; shim_lib_rel=empty; gunbc compile -> cssl_assemble -> cargo build --release --lib"
  echo -e "gunbc_sha\t$(sha256sum "$GUNBC" | awk '{print $1}')"
  echo -e "git_sha\t$(git rev-parse HEAD)"
  echo -e "gunbc_mtime\t$(stat -c '%y' "$GUNBC")"
  echo -e "module\temit\tcargo\tfirst_error\tmapped_gate\tverdict\tresidual_histogram\te0369_count"
} > "$STAMP_DIR/canonical_seven.tsv"

uncoded_histogram_suffix() {
  python3 - "$1" <<'PY'
import re, sys, collections
counts = collections.Counter()
with open(sys.argv[1], encoding="utf-8", errors="replace") as fh:
    for line in fh:
        if not line.startswith("error: "):
            continue
        if "could not compile" in line or "aborting due to" in line:
            continue
        msg = " ".join(line[len("error: "):].split())
        counts[msg] += 1
parts = []
for msg, n in counts.most_common():
    slug = re.sub(r"[^a-zA-Z0-9_]+", "_", msg).strip("_")[:56]
    parts.append(f"uncoded_{slug}:{n}")
print(" ".join(parts))
PY
}

for m in "${MODULES[@]}"; do
  echo "=== probing $m ===" >&2
  stem="$(basename "$m" .dag)"
  OUT="$(mktemp -d "${TMPDIR:-/tmp}/e0369-census.XXXXXX")"
  EMIT_LOG="$STAMP_DIR/logs/${stem}.emit.log"
  BUILD_LOG="$STAMP_DIR/logs/${stem}.cargo.log"

  EMIT_OK=0
  if "$GUNBC" compile \
    --source-root dag --source-root src/v2 \
    --entry "$m" --output-dir "$OUT" --target rust \
    --dependency-pool-index primary-precedence \
    >"$EMIT_LOG" 2>&1; then
    EMIT_OK=1
  fi

  EMIT_SUMMARY="emit_fail"
  if [[ "$EMIT_OK" -eq 1 ]]; then
    if grep -q 'compiled:' "$EMIT_LOG"; then
      EMIT_SUMMARY="$(grep -m1 'compiled:' "$EMIT_LOG" | sed 's/.*compiled: //')"
    else
      FILE_COUNT="$(find "$OUT" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
      EMIT_SUMMARY="${FILE_COUNT}files,unknown_diag"
    fi
  fi

  CARGO_VERDICT="skip"
  FIRST_ERROR=""
  MAPPED_GATE=""
  ERROR_HISTOGRAM=""
  e0369=0

  if [[ "$EMIT_OK" -eq 1 ]]; then
    if ! "$CSSL_ASSEMBLE" --out-dir "$OUT" --entry-dag "$m" --root "$ROOT" \
      >"$STAMP_DIR/logs/${stem}.assemble.log" 2>&1; then
      CARGO_VERDICT="harness_refuse"
      FIRST_ERROR="$(grep -m1 'CSSL_ASSEMBLE: REFUSED' "$STAMP_DIR/logs/${stem}.assemble.log" || head -1 "$STAMP_DIR/logs/${stem}.assemble.log")"
      MAPPED_GATE="HARNESS_SEED_LINK"
      VERDICT="HARNESS_REFUSE"
    elif ! render_cssl_probe_lib_cargo_toml "$ROOT" "$OUT/Cargo.toml"; then
      CARGO_VERDICT="harness_refuse"
      FIRST_ERROR="cssl harness authority unavailable"
      MAPPED_GATE="HARNESS_MISSING"
      VERDICT="HARNESS_REFUSE"
    else
      if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib \
        >"$STAMP_DIR/logs/${stem}.cargo.stdout" 2>"$BUILD_LOG"); then
        CARGO_VERDICT="green"
        ERROR_HISTOGRAM="clean"
        VERDICT="CARGO_GREEN"
      else
        CARGO_VERDICT="refuse"
        ERROR_HISTOGRAM="$(grep -oE '^error\[E[0-9]+\]' "$BUILD_LOG" | sort | uniq -c | sort -rn | awk '{printf "%s%s:%s", sep, $2, $1; sep=" "}' || true)"
        UNCODED_SUFFIX="$(uncoded_histogram_suffix "$BUILD_LOG")"
        if [[ -z "$ERROR_HISTOGRAM" ]]; then
          ERROR_HISTOGRAM="${UNCODED_SUFFIX:-uncoded_only:0}"
        elif [[ -n "$UNCODED_SUFFIX" ]]; then
          ERROR_HISTOGRAM="$ERROR_HISTOGRAM $UNCODED_SUFFIX"
        fi
        e0369="$(echo "$ERROR_HISTOGRAM" | grep -oE 'error\[E0369\]:[0-9]+' | head -1 | cut -d: -f2 || true)"
        e0369="${e0369:-0}"
        FIRST_ERROR="$(grep -m1 -E '^error(\[E[0-9]+\])?:' "$BUILD_LOG" || true)"
        if echo "$FIRST_ERROR" | grep -qE 'UNRESOLVED_CompilerError'; then
          MAPPED_GATE="UNKNOWN_unresolved"
        else
          MAPPED_GATE="UNKNOWN"
        fi
        VERDICT="UNKNOWN-$(echo "$FIRST_ERROR" | tr ' ' '_' | cut -c1-80)"
      fi
    fi
  else
    CARGO_VERDICT="emit_fail"
    FIRST_ERROR="$(head -3 "$EMIT_LOG" | tr '\n' ' ')"
    VERDICT="EMIT_REFUSE"
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$m" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT" "$ERROR_HISTOGRAM" "$e0369" \
    >> "$STAMP_DIR/canonical_seven.tsv"

  if [[ -f "$BUILD_LOG" ]]; then
    python3 - "$STAMP_DIR" "$stem" "$OUT" "$BUILD_LOG" <<'PY'
import re, sys, pathlib, shutil, collections
stamp, stem, out, blog = map(pathlib.Path, sys.argv[1:5])
log = blog.read_text(errors="replace")
blocks = re.split(r"(?=^error\[E0369\])", log, flags=re.M)
shapes = collections.Counter()
details = []
src_needed = set()
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
        src_needed.add(path)
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
    # code snippet line if present
    code = ""
    for ln in b.splitlines():
        if re.match(r"\s*\d+\s*\|\s+\S", ln):
            code = ln.split("|", 1)[1].strip()
            break
    details.append({
        "msg": msg, "path": path, "line": line, "col": col,
        "left": left, "right": right, "single": single, "code": code,
    })

with (stamp / "shapes" / f"{stem}.shapes.tsv").open("w") as fh:
    fh.write("count\tmessage\n")
    for msg, n in shapes.most_common():
        fh.write(f"{n}\t{msg}\n")
with (stamp / "shapes" / f"{stem}.instances.tsv").open("w") as fh:
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

emitted = stamp / "emitted" / stem
emitted.mkdir(parents=True, exist_ok=True)
for rel in sorted(src_needed):
    p = out / rel
    if not p.is_file():
        p = out / "src" / pathlib.Path(rel).name
    if p.is_file():
        shutil.copy2(p, emitted / p.name)
print(f"{stem}: {sum(shapes.values())} E0369 across {len(shapes)} shapes", file=sys.stderr)
PY
  fi
  rm -rf "$OUT"
done

echo "DONE -> $STAMP_DIR" >&2
