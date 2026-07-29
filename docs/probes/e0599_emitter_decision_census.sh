#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell probe joiner; until then it
# joins structured rustc diagnostics from the real compilation path against the single
# authority dag/tools/e0599_emitter_decision_census.dag (probe-only, P-fn Phase B0).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
#
# Authority: dag/tools/e0599_emitter_decision_census.dag — the typed causes, the lowering
# rows and the operation->cause decision load through gunbc via
# e0599_write_classify_from_blob. THERE IS NO SECOND CLASSIFICATION TABLE HERE; this file
# extracts (method, receiver_expr) pairs off the emitted artifact and asks the authority.
# Root families for the R1/R2/R3 scope filter load from the Phase A authority
# dag/tools/e0599_probe_census.dag the same way, via e0599_write_root_family_labels_from_blob.
#
# Input: a work dir populated by the emit+assemble+cargo spine with --message-format=json
# retained per module (see the receipt doc for the exact invocation).
# Inline python avoids a committed .py file (gitignore_authority models *.py as local-dev-only).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <work-dir>" >&2
  exit 2
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
if [[ ! -x "$GUNBC" ]]; then
  echo "error: gunbc not found at $GUNBC (build v1-compiler --bin gunbc)" >&2
  exit 2
fi

export E0599_B0_ROOT="$ROOT"
export E0599_B0_GUNBC="$GUNBC"
export E0599_B0_WORK="$1"

python3 - <<'PY'
import collections
import json
import os
import pathlib
import re
import subprocess
import tempfile

ROOT = pathlib.Path(os.environ["E0599_B0_ROOT"])
GUNBC = os.environ["E0599_B0_GUNBC"]
WORK = pathlib.Path(os.environ["E0599_B0_WORK"])

MODULES = ["04_infer", "05_emit", "05_eval", "06_translate", "emit_host", "emit_module",
           "materialization_carriers"]

FN_RE = re.compile(r"^\s*pub fn ([A-Za-z0-9_]+)\s*(<[^(]*>)?\s*\(")
MSG_MISSING = re.compile(r"no method named `([^`]+)` found for (.+?) in the current scope")
MSG_BOUNDS = re.compile(r"the method `([^`]+)` exists for (.+?), but its trait bounds were not satisfied")

CLOSERS = {")": "(", "]": "[", "}": "{"}
OPENERS = {"(": ")", "[": "]", "{": "}"}


def gunbc_blob(entry, function, blob):
    """Ask a .dag authority to classify a blob. Refusals propagate — never defaulted."""
    with tempfile.NamedTemporaryFile(prefix="e0599_b0_", delete=False) as tmp:
        out_path = tmp.name
    try:
        cmd = [GUNBC, "run", "--source-root", str(ROOT / "dag"), "--source-root", str(ROOT / "src/v2"),
               "--entry", entry, "--function", function,
               "--arg", f"blob={blob}", "--arg", f"out_path={out_path}"]
        proc = subprocess.run(cmd, capture_output=True, text=True, check=False, cwd=ROOT)
        if proc.returncode != 0:
            raise RuntimeError(f"{function} refused (exit {proc.returncode}): {proc.stderr[-2000:]}")
        return pathlib.Path(out_path).read_text(encoding="utf-8")
    finally:
        pathlib.Path(out_path).unlink(missing_ok=True)


def receiver_expr(text, dot_index):
    """One balanced postfix expression ending at the '.' before the failing method."""
    i = dot_index - 1
    depth = []
    while i >= 0:
        c = text[i]
        if c in CLOSERS:
            depth.append(CLOSERS[c])
        elif c in OPENERS:
            if not depth:
                break
            depth.pop()
        elif not depth:
            if c.isspace():
                break
            if not (c.isalnum() or c in "_.:*&<>!?"):
                break
        i -= 1
    return text[i + 1:dot_index].strip()


def enclosing_fns(rs_path):
    out = []
    for i, line in enumerate(rs_path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        m = FN_RE.match(line)
        if m:
            out.append((i, m.group(1), (m.group(2) or "").strip()))
    return out


def resolve_fn(fns, line):
    best = None
    for ln, name, tps in fns:
        if ln > line:
            break
        best = (ln, name, tps)
    return best


def dag_source_for(emitted_file):
    """v2_std_algebra.rs -> src/v2/std/algebra.dag ; std_types.rs -> dag/std/types.dag."""
    stem = pathlib.Path(emitted_file).stem
    parts = stem.split("_")
    for split in range(len(parts), 0, -1):
        rel = "/".join(parts[:split - 1] + ["_".join(parts[split - 1:])]) + ".dag"
        for base in ("src", "dag"):
            cand = ROOT / base / rel
            if cand.is_file():
                return str(cand.relative_to(ROOT))
    # exhaustive fallback: search the two source roots for a module whose emitted name matches
    for base in ("src/v2", "dag", "src/v1"):
        for cand in (ROOT / base).rglob("*.dag"):
            flat = str(cand.relative_to(ROOT)).replace("src/", "").replace("/", "_")[:-4]
            if flat == stem:
                return str(cand.relative_to(ROOT))
    return ""


SRC_FN_CACHE = {}


def dag_fn_signature(dag_rel, fn_name):
    """The exact source type-parameter declaration, read from the .dag authority."""
    if not dag_rel:
        return ("", "")
    key = dag_rel
    if key not in SRC_FN_CACHE:
        text = (ROOT / dag_rel).read_text(encoding="utf-8", errors="replace")
        sigs = {}
        for m in re.finditer(r"^fn ([A-Za-z0-9_]+)(<[^>(]*>)?\s*\(", text, re.M):
            sigs.setdefault(m.group(1), (m.group(2) or "", text[:m.start()].count("\n") + 1))
        SRC_FN_CACHE[key] = sigs
    sig = SRC_FN_CACHE[key].get(fn_name)
    return (sig[0], str(sig[1])) if sig else ("", "")


def collect():
    rows = []
    missing = []
    for module in MODULES:
        cj = WORK / module / "cargo.json"
        if not cj.is_file():
            missing.append(module)
            continue
        fn_cache = {}
        for raw in cj.open(encoding="utf-8", errors="replace"):
            try:
                msg = json.loads(raw)
            except json.JSONDecodeError:
                continue
            if msg.get("reason") != "compiler-message":
                continue
            d = msg["message"]
            if (d.get("code") or {}).get("code") != "E0599":
                continue
            spans = [s for s in d["spans"] if s.get("is_primary")]
            if not spans:
                rows.append({"module": module, "file": "", "line": 0, "method": "",
                             "receiver_type": "", "receiver_expr": "", "shape": "no_primary_span",
                             "emitted_fn": "", "emitted_type_params": ""})
                continue
            s = spans[0]
            text = d["message"]
            mm, mb = MSG_MISSING.search(text), MSG_BOUNDS.search(text)
            if mm:
                shape, method, recv_type = "missing_method", mm.group(1), " ".join(mm.group(2).split())
            elif mb:
                shape, method, recv_type = "bounds_unsatisfied", mb.group(1), " ".join(mb.group(2).split())
            else:
                shape, method, recv_type = "other", "", " ".join(text.split())

            t = (s.get("text") or [{}])[0]
            line_text = t.get("text", "")
            hs = t.get("highlight_start", 1) - 1
            dot = hs - 1
            rexpr = receiver_expr(line_text, dot) if dot >= 0 and line_text[dot:dot + 1] == "." else ""

            fname = s["file_name"]
            if fname not in fn_cache:
                rs = WORK / module / fname
                fn_cache[fname] = enclosing_fns(rs) if rs.is_file() else []
            enc = resolve_fn(fn_cache[fname], s["line_start"])
            rows.append({
                "module": module, "file": fname, "line": s["line_start"],
                "method": method, "receiver_type": recv_type, "receiver_expr": rexpr,
                "shape": shape,
                "emitted_fn": enc[1] if enc else "",
                "emitted_type_params": enc[2] if enc else "",
            })
    return rows, missing


rows, missing = collect()
if missing:
    raise SystemExit(f"REFUSED: no build for module(s) {missing} — the census is not "
                     f"reported over a partial roster (DESIGN §5: a failure arm refuses, "
                     f"never widens)")

# --- scope filter: R1/R2/R3, via the Phase A authority (no second family table) --------
fam_keys = sorted({(r["shape"], r["method"], r["receiver_type"]) for r in rows})
fam_blob = "\n".join(f"{s}\t{m}\t{rt}" for s, m, rt in fam_keys)
fam_out = gunbc_blob("dag/tools/e0599_probe_census.dag",
                     "e0599_write_root_family_labels_from_blob", fam_blob).splitlines()
if len(fam_out) != len(fam_keys):
    raise SystemExit(f"REFUSED: family authority returned {len(fam_out)} labels for {len(fam_keys)} keys")
FAMILY = dict(zip(fam_keys, fam_out))
for r in rows:
    r["root_family"] = FAMILY[(r["shape"], r["method"], r["receiver_type"])]

R123 = {"R1CloneBoundOnTypeParam", "R2VectorMethodBounds", "R3ContainerCloneBounds"}
scoped = [r for r in rows if r["root_family"] in R123]

# --- classification: via the B0 authority (no second cause table) ----------------------
cls_keys = sorted({(r["method"], r["receiver_expr"]) for r in scoped})
cls_blob = "\n".join(f"{m}\t{e}" for m, e in cls_keys)
cls_out = gunbc_blob("dag/tools/e0599_emitter_decision_census.dag",
                     "e0599_write_classify_from_blob", cls_blob).splitlines()
if len(cls_out) != len(cls_keys):
    raise SystemExit(f"REFUSED: cause authority returned {len(cls_out)} rows for {len(cls_keys)} keys")
CLASS = {}
for key, line in zip(cls_keys, cls_out):
    f = line.split("\t")
    while len(f) < 6:
        f.append("")
    CLASS[key] = dict(operation=f[0], source_construct=f[1], cause=f[2],
                      required_trait=f[3], emitter_authority=f[4], external_authority=f[5])

# --- join ------------------------------------------------------------------------------
SITES = collections.defaultdict(list)
for r in scoped:
    SITES[(r["file"], r["line"], r["method"], r["receiver_type"])].append(r)

out_rows = []
for key, occs in sorted(SITES.items()):
    f0 = occs[0]
    c = CLASS[(f0["method"], f0["receiver_expr"])]
    dag_rel = dag_source_for(f0["file"])
    src_tp, src_line = dag_fn_signature(dag_rel, f0["emitted_fn"])
    emitted_tp = f0["emitted_type_params"]
    # which type param does this site need a bound on?
    needed = ""
    m = re.search(r"type parameter `([A-Za-z0-9_]+)`", f0["receiver_type"])
    if m:
        needed = m.group(1)
    else:
        m = re.search(r"<([A-Za-z0-9_]+)>", f0["receiver_type"])
        if m:
            needed = m.group(1)
    legacy = "unknown"
    if needed:
        legacy = "covered" if re.search(rf"\b{re.escape(needed)}\s*:\s*Clone\b", emitted_tp) else "not_covered"
    out_rows.append({
        "emitted_file": f0["file"], "emitted_line": key[1],
        "emitted_fn": f0["emitted_fn"], "emitted_type_params": emitted_tp,
        "dag_source": dag_rel, "dag_source_line": src_line, "source_type_params": src_tp,
        "source_construct": c["source_construct"],
        "target_representation": f0["receiver_type"],
        "lowering_operation": c["operation"],
        "ownership_verdict": ("emitter-ownership-defork candidate"
                              if c["cause"] == "CloneSharedRequirement" else "n/a"),
        "required_trait": (f"{needed}: {c['required_trait']}" if needed and c["required_trait"] else c["required_trait"]),
        "requirement_cause": c["cause"],
        "external_authority": c["external_authority"],
        "emitter_authority": c["emitter_authority"],
        "legacy_helper": legacy,
        "root_family": f0["root_family"],
        "occurrences": len(occs),
        "modules": ",".join(sorted({o["module"] for o in occs})),
    })

unresolved = [r for r in out_rows if r["requirement_cause"] == "Unresolved"]

print("# section=b0_scope")
print(f"modules_measured\t{','.join(MODULES)}")
print(f"E0599_all_families_diagnostics\t{len(rows)}")
print(f"R1R2R3_diagnostics\t{len(scoped)}")
print(f"R1R2R3_unique_sites\t{len(out_rows)}")
print(f"unresolved_sites\t{len(unresolved)}")
print()
print("# section=cause_rollup")
cause_sites = collections.Counter(r["requirement_cause"] for r in out_rows)
cause_occ = collections.Counter()
for r in out_rows:
    cause_occ[r["requirement_cause"]] += r["occurrences"]
print("requirement_cause\tunique_sites\toccurrences")
for cause in ("TargetApiRequirement", "OwnedDeconstructionRequirement", "CloneSharedRequirement",
              "NoRequirement", "Unresolved"):
    print(f"{cause}\t{cause_sites.get(cause, 0)}\t{cause_occ.get(cause, 0)}")
print(f"TOTAL\t{len(out_rows)}\t{sum(cause_occ.values())}")
print()
print("# section=lowering_operation_rollup")
op_sites = collections.Counter(r["lowering_operation"] for r in out_rows)
op_occ = collections.Counter()
for r in out_rows:
    op_occ[r["lowering_operation"]] += r["occurrences"]
print("lowering_operation\tunique_sites\toccurrences")
for op, n in op_sites.most_common():
    print(f"{op}\t{n}\t{op_occ[op]}")
print()
print("# section=legacy_helper_cross_reference")
lg = collections.Counter(r["legacy_helper"] for r in out_rows)
print("legacy_helper_covers_required_param\tunique_sites")
for k in ("covered", "not_covered", "unknown"):
    print(f"{k}\t{lg.get(k, 0)}")
print()
print("# section=per_site")
cols = ["emitted_file", "emitted_line", "emitted_fn", "emitted_type_params", "dag_source",
        "dag_source_line", "source_type_params", "source_construct", "target_representation",
        "lowering_operation", "ownership_verdict", "required_trait", "requirement_cause",
        "external_authority", "emitter_authority", "legacy_helper", "root_family",
        "occurrences", "modules"]
print("\t".join(cols))
for r in out_rows:
    print("\t".join(str(r[c]) for c in cols))

with (WORK / "b0_census.json").open("w") as fh:
    json.dump(out_rows, fh, indent=1)
PY
