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
import hashlib
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

# Message shapes are NOT restated here. They load from their single authority
# (tools.e0599_probe_census e0599_message_pattern_rows), the same rows
# docs/probes/e0599_census_extract.sh consumes. This joiner previously hardcoded two
# regexes covering only MissingMethod and BoundsUnsatisfied, so the authority's NoVariant /
# NoAssocFn shapes fell through to an untyped "other" and were then discarded by the
# R1/R2/R3 filter — a third duplicated roster, and the reason a real NoVariant diagnostic
# was silently leaving the denominator. Patterns are anchored with the authority's own
# "error[E0599]: " prefix, so the raw JSON message is prefixed before matching.
MESSAGE_PATTERNS = None
E0599_PREFIX = "error[E0599]: "

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


def load_message_patterns():
    """The E0599 message shapes, from their single authority — never restated here."""
    blob = gunbc_blob_noarg("dag/tools/e0599_probe_census.dag",
                            "e0599_write_message_pattern_rows_blob")
    pats = []
    for line in blob.splitlines():
        if not line.strip():
            continue
        shape, pattern = line.split("\t", 1)
        pats.append((shape, re.compile(pattern)))
    if not pats:
        raise SystemExit("REFUSED: message-pattern authority returned no rows")
    return pats


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
    """emitted-file stem -> [(declared module identity, .dag path)], a MULTIMAP.

    A .dag file's path is a storage fact; its module identity is the declaration. The v1
    emitter names the emitted file after the module path, so joining on the declaration is
    the faithful inverse — a filename heuristic mis-resolves every module whose file stem
    differs from its module name (v2.compiler.translate lives in 06_translate.dag).

    The stem is a LOSSY projection of that identity: `a.b_c` and `a_b.c` both flatten to
    `a_b_c`. The previous `setdefault` silently kept whichever file the walk reached first
    — the exact first-match ambiguity the namespace program exists to remove. Collisions
    are now retained and resolved by REFUSAL at lookup, never by arrival order.
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
            module_id = m.group(1)
            MODULE_INDEX.setdefault(module_id.replace(".", "_"), []).append(
                (module_id, str(cand.relative_to(ROOT))))


def dag_source_for(emitted_file):
    """The .dag source for an emitted artifact, or a located refusal. Never a guess."""
    build_module_index()
    stem = pathlib.Path(emitted_file).stem
    hits = MODULE_INDEX.get(stem, [])
    if len(hits) == 1:
        return hits[0][1]
    if not hits:
        raise SystemExit(
            "REFUSED: no .dag module declares an identity flattening to {!r} (emitted file "
            "{!r}). The census cannot report a source-side type-parameter declaration it "
            "did not locate, and an empty dag_source column would read as 'no source' "
            "rather than 'not found' (DESIGN §5).".format(stem, emitted_file))
    raise SystemExit(
        "REFUSED: emitted file {!r} flattens to stem {!r}, which {} distinct declared "
        "modules share: {}. Underscore-flattening is not injective, so the join cannot "
        "identify the exact declaration; picking one by walk order is the first-match "
        "ambiguity this probe must not reintroduce.".format(
            emitted_file, stem, len(hits), ", ".join(sorted(m for m, _ in hits))))


SRC_FN_CACHE = {}


def dag_fn_signature(dag_rel, fn_name, emitted_file):
    """The exact source type-parameter declaration, or a located refusal.

    Declarations are collected into a MULTIMAP. A duplicate `fn` name means the join cannot
    identify which declaration produced the emitted function, and the previous `setdefault`
    answered with whichever came first in file order — a silent wrong answer in a column
    the census reports as the exact source declaration. Both the ambiguous and the absent
    case now refuse, because "" would read as "declares no type parameters" rather than
    "could not be located".
    """
    if not dag_rel:
        raise SystemExit("REFUSED: dag_fn_signature called with no dag_source for {!r}"
                         .format(emitted_file))
    if not fn_name:
        raise SystemExit(
            "REFUSED: no enclosing `pub fn` was resolved for a diagnostic in {!r}, so the "
            "source function is unknown. The census reports one row per emitted function; "
            "an unnamed row cannot be joined to its .dag declaration.".format(emitted_file))
    if dag_rel not in SRC_FN_CACHE:
        text = (ROOT / dag_rel).read_text(encoding="utf-8", errors="replace")
        sigs = {}
        for m in re.finditer(r"^fn ([A-Za-z0-9_]+)(<[^>(]*>)?\s*\(", text, re.M):
            sigs.setdefault(m.group(1), []).append(
                (m.group(2) or "", text[:m.start()].count("\n") + 1))
        SRC_FN_CACHE[dag_rel] = sigs
    hits = SRC_FN_CACHE[dag_rel].get(fn_name, [])
    if len(hits) == 1:
        return (hits[0][0], str(hits[0][1]))
    if not hits:
        raise SystemExit(
            "REFUSED: emitted fn {!r} (from {!r}) has no `fn {}` declaration in its source "
            "{!r}. The exact source type-parameter declaration is a required census field "
            "and must not be reported as empty.".format(fn_name, emitted_file, fn_name, dag_rel))
    raise SystemExit(
        "REFUSED: {!r} declares `fn {}` {} times (lines {}), so the join cannot identify "
        "which declaration produced emitted fn {!r} in {!r}. Taking the first is the "
        "first-match ambiguity this probe must not reintroduce.".format(
            dag_rel, fn_name, len(hits), ", ".join(str(l) for _, l in hits),
            fn_name, emitted_file))


DIGEST_CACHE = {}


def emitted_file_digest(module, fname):
    """sha256 of the emitted artifact this diagnostic was produced from.

    The same dependency is emitted into all seven module closures. Collapsing those into
    one census row is only sound if the emitted bytes are identical; the digest makes that
    a fact in the key rather than an assumption. Differing content becomes distinct sites.
    """
    key = (module, fname)
    if key not in DIGEST_CACHE:
        p = WORK / module / fname
        if not p.is_file():
            raise SystemExit(
                "REFUSED: emitted artifact {!r} for module {!r} is absent from the work "
                "dir, so its content digest cannot be computed and cross-module collapse "
                "would be unverified.".format(fname, module))
        DIGEST_CACHE[key] = hashlib.sha256(p.read_bytes()).hexdigest()[:16]
    return DIGEST_CACHE[key]


def collect():
    rows = []
    missing = []
    for module in MODULES:
        cj = WORK / module / "cargo.json"
        if not cj.is_file():
            missing.append(module)
            continue
        fn_cache = {}
        for lineno, raw in enumerate(cj.open(encoding="utf-8", errors="replace"), 1):
            if not raw.strip():
                continue
            try:
                msg = json.loads(raw)
            except json.JSONDecodeError as exc:
                # A truncated or malformed compiler message must not vanish from the
                # denominator. Silently skipping it under-counts the population and the
                # census would report the shortfall as a smaller, cleaner corpus.
                raise SystemExit(
                    "REFUSED: {}/cargo.json line {} is not valid JSON ({}). A dropped "
                    "compiler message silently shrinks the measured population, so the "
                    "census refuses rather than reporting an under-count. First 200 "
                    "chars: {!r}".format(module, lineno, exc, raw[:200]))
            if msg.get("reason") != "compiler-message":
                continue
            d = msg["message"]
            if (d.get("code") or {}).get("code") != "E0599":
                continue
            if MESSAGE_PATTERNS is None:
                raise SystemExit("REFUSED: message-pattern authority not loaded")
            spans = [s for s in d["spans"] if s.get("is_primary")]
            if not spans:
                rows.append({"module": module, "file": "", "line": 0, "method": "",
                             "receiver_type": "", "receiver_expr": "", "shape": "no_primary_span",
                             "emitted_fn": "", "emitted_type_params": ""})
                continue
            s = spans[0]
            text = d["message"]
            probe_text = E0599_PREFIX + " ".join(text.split())
            shape, method, recv_type = None, "", ""
            for shape_label, rx in MESSAGE_PATTERNS:
                m = rx.search(probe_text)
                if not m:
                    continue
                shape = shape_label
                if shape_label == "other":
                    break
                method = m.group(1)
                recv_type = " ".join(m.group(2).split()) if m.lastindex and m.lastindex >= 2 else ""
                break
            if shape is None or shape == "other":
                # An E0599 whose message shape neither pattern understands must refuse HERE,
                # before the R1/R2/R3 filter can quietly discard it as out-of-scope. An
                # unparsed diagnostic is not evidence of a tail family; it is evidence the
                # probe does not understand the compiler's output.
                raise SystemExit(
                    "REFUSED: {}/cargo.json carries an E0599 whose message shape matches "
                    "neither the missing-method nor the trait-bounds pattern, so its method "
                    "and receiver type are unknown. It must not reach the scope filter, "
                    "where it would be discarded as a tail family and silently leave the "
                    "denominator. Message: {!r}".format(module, " ".join(text.split())[:300]))

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
                # Exact primary-span identity. `line` alone collapses two distinct clone
                # expressions on one generated line; the span start/end separate them.
                "col_start": s.get("column_start", 0), "col_end": s.get("column_end", 0),
                "line_end": s.get("line_end", s["line_start"]),
                "byte_start": s.get("byte_start", -1), "byte_end": s.get("byte_end", -1),
                "emitted_digest": emitted_file_digest(module, fname),
                "method": method, "receiver_type": recv_type, "receiver_expr": rexpr,
                "shape": shape,
                "emitted_fn": enc[1] if enc else "",
                "emitted_type_params": enc[2] if enc else "",
            })
    return rows, missing


MESSAGE_PATTERNS = load_message_patterns()
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
# SITE IDENTITY. Previously (file, line, method, receiver_type) — which excluded the
# primary span column, the receiver expression and the emitted artifact's content, so two
# distinct clone expressions on one generated line with the same receiver type collapsed
# into a single row and inherited occs[0]'s cause. The key now carries the emitted digest
# and the exact primary span, and — decisively — the receiver expression itself, which
# makes a heterogeneous group UNWRITABLE rather than merely detected (DESIGN §5:
# construction over validation). The assertion below is a backstop against a future edit
# that removes receiver_expr from the key, not the primary mechanism.
SITE_KEY_FIELDS = ("emitted_digest", "file", "line", "col_start", "line_end", "col_end",
                   "method", "receiver_expr")
SITES = collections.defaultdict(list)
for r in scoped:
    SITES[tuple(r[k] for k in SITE_KEY_FIELDS)].append(r)

out_rows = []
for key, occs in sorted(SITES.items(), key=lambda kv: tuple(str(x) for x in kv[0])):
    f0 = occs[0]
    distinct_exprs = {o["receiver_expr"] for o in occs}
    if len(distinct_exprs) != 1:
        raise SystemExit(
            "REFUSED: site {!r} groups {} distinct receiver expressions {} — classifying "
            "the group by its first member would attribute one expression's lowering cause "
            "to another. Site identity must separate them.".format(
                key, len(distinct_exprs), sorted(distinct_exprs)))
    c = CLASS[(f0["method"], f0["receiver_expr"])]
    dag_rel = dag_source_for(f0["file"])
    src_tp, src_line = dag_fn_signature(dag_rel, f0["emitted_fn"], f0["file"])
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
        "emitted_file": f0["file"], "emitted_line": f0["line"],
        "emitted_span": "{}:{}-{}:{}".format(f0["line"], f0["col_start"], f0["line_end"], f0["col_end"]),
        "emitted_digest": f0["emitted_digest"],
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

# The producer label and its caveat are NOT restated here. They load from their single
# authority (tools.e0599_emitter_decision_census e0599_measured_producer /
# e0599_measured_producer_caveat) — a hardcoded label is not provenance evidence.
_prod = [s for s in (x.strip() for x in gunbc_blob_noarg(
    "dag/tools/e0599_emitter_decision_census.dag",
    "e0599_write_producer_blob").split("\n")) if s]
if len(_prod) != 2:
    raise SystemExit("REFUSED: producer authority returned {} lines, expected "
                     "producer + caveat".format(len(_prod)))
print("# section=b0_scope")
print("measured_producer\t{}".format(_prod[0]))
print("measured_producer_caveat\t{}".format(_prod[1]))
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
cols = ["emitted_file", "emitted_line", "emitted_span", "emitted_digest",
        "emitted_fn", "emitted_type_params", "dag_source",
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
