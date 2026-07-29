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

# The module roster is NOT restated here. It loads from its single authority
# (tools.e0599_probe_census e0599_canonical_seven_modules) through gunbc, the same way
# docs/probes/e0599_census_extract.sh loads it.
MODULES = None

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


def load_module_roster():
    """The canonical-seven roster, from its single authority — never restated here."""
    blob = gunbc_blob_noarg("dag/tools/e0599_probe_census.dag",
                            "e0599_write_canonical_seven_module_log_labels_blob")
    roster = [line.strip() for line in blob.splitlines() if line.strip()]
    if len(roster) != 7:
        raise SystemExit(f"REFUSED: roster authority returned {len(roster)} modules, expected 7")
    return roster


def gunbc_blob_noarg(entry, function):
    with tempfile.NamedTemporaryFile(prefix="e0599_b0_", delete=False) as tmp:
        out_path = tmp.name
    try:
        cmd = [GUNBC, "run", "--source-root", str(ROOT / "dag"), "--source-root", str(ROOT / "src/v2"),
               "--entry", entry, "--function", function, "--arg", f"out_path={out_path}"]
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


MODULE_INDEX = {}


def build_module_index():
    """emitted-file stem -> .dag path, keyed on each module's OWN `module` declaration.

    A .dag file's path is a storage fact; its module identity is the declaration. The v1
    emitter names the emitted file after the module path, so joining on the declaration is
    the faithful inverse — a filename heuristic mis-resolves every module whose file stem
    differs from its module name (v2.compiler.translate lives in 06_translate.dag).
    """
    if MODULE_INDEX:
        return
    for base in ("src/v2", "dag", "src/v1"):
        root = ROOT / base
        if not root.is_dir():
            continue
        for cand in root.rglob("*.dag"):
            try:
                head = cand.open(encoding="utf-8", errors="replace").readline()
            except OSError:
                continue
            m = re.match(r"\s*module\s+([A-Za-z0-9_.]+)", head)
            if not m:
                continue
            stem = m.group(1).replace(".", "_")
            MODULE_INDEX.setdefault(stem, str(cand.relative_to(ROOT)))


def dag_source_for(emitted_file):
    build_module_index()
    return MODULE_INDEX.get(pathlib.Path(emitted_file).stem, "")


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


MODULES = load_module_roster()
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

# The R1/R2/R3 scope set is NOT restated here. It loads from the Phase A authority
# (tools.e0599_probe_census e0599_mechanistic_root_families), whose labels derive from the
# same e0599_root_family_label the family join above uses — so a rename in .dag cannot
# silently narrow this filter.
R123 = set(gunbc_blob_noarg("dag/tools/e0599_probe_census.dag",
                            "e0599_write_mechanistic_root_family_labels_blob").split("\n"))
R123 = {s for s in (x.strip() for x in R123) if s}
if not R123:
    raise SystemExit("REFUSED: mechanistic root-family scope set is empty — the authority "
                     "returned no labels, and an empty filter would silently scope the "
                     "census to zero sites (DESIGN §5: refuse, never widen or narrow)")
unknown = {r["root_family"] for r in rows} - R123
scoped = [r for r in rows if r["root_family"] in R123]
if not scoped:
    raise SystemExit(f"REFUSED: no diagnostic matched the scope set {sorted(R123)}; "
                     f"observed families were {sorted(unknown)} — a scope set that matches "
                     "nothing is a located refusal, never an empty census")

# --- classification: via the B0 authority (no second cause table) ----------------------
E0599_CLASSIFICATION_FIELDS = 7
cls_keys = sorted({(r["method"], r["receiver_expr"]) for r in scoped})
cls_blob = "\n".join(f"{m}\t{e}" for m, e in cls_keys)
cls_out = gunbc_blob("dag/tools/e0599_emitter_decision_census.dag",
                     "e0599_write_classify_from_blob", cls_blob).splitlines()
if len(cls_out) != len(cls_keys):
    raise SystemExit(f"REFUSED: cause authority returned {len(cls_out)} rows for {len(cls_keys)} keys")
CLASS = {}
for key, line in zip(cls_keys, cls_out):
    f = line.split("\t")
    if len(f) != E0599_CLASSIFICATION_FIELDS:
        raise SystemExit(
            f"REFUSED: cause authority returned {len(f)} fields (expected "
            f"{E0599_CLASSIFICATION_FIELDS}) for key {key!r}: {line!r} — a malformed "
            f"authority row is a located refusal, never a padded census row (DESIGN §5)")
    CLASS[key] = dict(operation=f[0], source_construct=f[1], ownership_alternative=f[2],
                      cause=f[3], required_trait=f[4], emitter_authority=f[5],
                      external_authority=f[6])

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
        "ownership_verdict": c["ownership_alternative"],
        "required_trait": (f"{needed}: {c['required_trait']}" if needed and c["required_trait"] else c["required_trait"]),
        "requirement_cause": c["cause"],
        "external_authority": c["external_authority"],
        "emitter_authority": c["emitter_authority"],
        "legacy_helper": legacy,
        "root_family": f0["root_family"],
        "occurrences": len(occs),
        "modules": ",".join(sorted({o["module"] for o in occs})),
        # Diagnostic-only: not in `cols`, so the TSV shape is unchanged. These are what the
        # classifier actually failed on, so an Unresolved refusal can name it.
        "probe_method": f0["method"],
        "probe_receiver_expr": f0["receiver_expr"],
    })

unresolved = [r for r in out_rows if r["requirement_cause"] == "Unresolved"]

print("# section=b0_scope")
print("measured_producer\tV1SeedEmitter")
print("measured_producer_caveat\tDIAGNOSTIC COMPARISON ONLY - not CompilerFixedPoint progress and not a v2 sizing authority; CompilerFixedPoint recenters on the V2 emitter, a different producer (operator ruling 2026-07-29)")
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
# The cause roster is NOT restated here. It loads from the B0 authority
# (tools.e0599_emitter_decision_census e0599_rollup_cause_order), whose labels derive from
# the same e0599_requirement_cause_label the classification uses — so a new variant cannot
# be silently omitted from the rollup.
ROLLUP_CAUSES = [s for s in (x.strip() for x in gunbc_blob_noarg(
    "dag/tools/e0599_emitter_decision_census.dag",
    "e0599_write_rollup_cause_labels_blob").split("\n")) if s]
if not ROLLUP_CAUSES:
    raise SystemExit("REFUSED: cause authority returned no rollup labels")
missing = set(cause_sites) - set(ROLLUP_CAUSES)
if missing:
    raise SystemExit(f"REFUSED: measured cause(s) {sorted(missing)} are absent from the "
                     "authority's rollup roster — the rollup would silently drop them "
                     "(DESIGN §5: a failure arm must refuse, never widen)")
for cause in ROLLUP_CAUSES:
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
print("# section=legacy_helper_bounds_produced")
print("note\tv1_rt.rs is the hand-written seed runtime, not emitter output, and is excluded")
print("module_closure\thelper_bounded_fns")
helper = {}
per_mod = {}
for m in MODULES:
    seen = set()
    for rs in sorted((WORK / m).glob("src/*.rs")):
        if rs.name == "v1_rt.rs":
            continue
        for line in rs.read_text(encoding="utf-8", errors="replace").splitlines():
            mm = FN_RE.match(line)
            if mm and mm.group(2) and ": Clone" in mm.group(2):
                helper[(rs.name, mm.group(1))] = mm.group(2).strip()
                seen.add((rs.name, mm.group(1)))
    per_mod[m] = len(seen)
    print(f"{m}\t{len(seen)}")
print(f"UNION\t{len(helper)}")
print()
print("# section=legacy_helper_overlap_with_defect_sites")
site_fns = {(r["emitted_file"].split("/")[-1], r["emitted_fn"]): r for r in out_rows}
overlap = sorted(k for k in helper if k in site_fns)
print(f"helper_bounded_fns_hosting_a_defect_site\t{len(overlap)}\tof\t{len(helper)}")
print("emitted_file\temitted_fn\thelper_bound_param\tsite_required_param\tdisjoint")
for k in overlap:
    req = sorted({r["required_trait"] for r in out_rows
                  if (r["emitted_file"].split("/")[-1], r["emitted_fn"]) == k})
    bound_params = [seg.split(":")[0].strip() for seg in helper[k].strip("<>").split(",") if ":" in seg]
    req_params = [x.split(":")[0].strip() for x in req]
    disjoint = "yes" if not (set(bound_params) & set(req_params)) else "no"
    print(f"{k[0]}\t{k[1]}\t{helper[k]}\t{','.join(req)}\t{disjoint}")
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

# DESIGN §5, the factory model: a deficit stops the line. The census above is emitted first
# so every deficit is inspectable and located (the sanctioned stopped-line audit "reports,
# it does not green"), and THEN the line stops. An unrecognized emitter shape must never
# leave this probe with a zero exit, because a green incomplete census is indistinguishable
# from a complete one to any consumer.
if unresolved:
    shown = unresolved[:20]
    detail = "; ".join(
        "{}:{} {} (method={!r} receiver={!r})".format(
            r["emitted_file"], r["emitted_line"], r["emitted_fn"],
            r["probe_method"], r["probe_receiver_expr"])
        for r in shown)
    more = "" if len(unresolved) <= len(shown) else " ... and {} more".format(len(unresolved) - len(shown))
    raise SystemExit(
        "REFUSED: {} site(s) reached the Unresolved arm — the census above is INCOMPLETE "
        "and must not be consumed as a measurement. No lowering row in "
        "tools.e0599_emitter_decision_census names these shapes, so their requirement cause "
        "is unknown, not absent. Located: {}{}".format(len(unresolved), detail, more))
PY
