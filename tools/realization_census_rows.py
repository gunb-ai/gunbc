#!/usr/bin/env python3
"""Interim producer: the realization census TSVs -> dag/gunbc/realization_census_rows.dag.

Run by hand. DESIGN section 6 (intellectual sustainability): this script is a scaffold. It
dissolves when a v2 lens produces the same rows from the corpus at the pin; the types it targets
(gunbc.realization_census) and the observation shape they inhabit
(gunbc.repository_census_observation) are the durable part, and nothing here is written so that the
lens would have to undo it. It carries no judgement: it maps closed vocabularies to constructors
and refuses (exit 1, nothing written) on any value outside them, any empty or duplicate identity,
any row of the wrong width.

  python3 -I tools/realization_census_rows.py --census-dir DIR --out dag/gunbc/realization_census_rows.dag
"""
import argparse, hashlib, json, os, sys

SITES = {"None": "SiteNone", "Home": "SiteHome", "Passed": "SitePassed",
         "Rederived": "SiteRederived", "Literal": "SiteLiteral", "Forked": "SiteForked",
         "Other": "SiteOther"}
LEVELS = {f"L{i}": f"LevelL{i}" for i in range(6)}
CLASSES = {"production": "ClassProduction", "seed-generated": "ClassSeedGenerated",
           "seed": "ClassSeed", "roster": "ClassRoster", "test": "ClassTest",
           "extdeps": "ClassExtdeps"}
DIMS = {"storage": "DimensionStorage", "caching": "DimensionCaching", "access": "DimensionAccess",
        "authz": "DimensionAuthz", "placement": "DimensionPlacement", "other": "DimensionOther"}
DIM_ORDER = ["storage", "caching", "access", "authz", "placement"]
MODULE_COLUMNS = ["module", "class"] + [f"{d}_{k}" for d in DIM_ORDER for k in ("site", "level")] + [
    "ambient_calls", "worst_site", "worst_finding", "evidence_symbols", "phase"]
FINDING_COLUMNS = ["finding_id", "module", "dimension", "level", "class", "site", "symbol",
                   "specimen", "note", "existing_authority", "fix_shape"]


def die(msg):
    sys.stderr.write("realization_census_rows: " + msg + "\n")
    sys.exit(1)


def lit(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("{", "\\{") + '"'


def read_tsv(path, columns, interim_ids=False):
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    header = lines[0].split("\t")
    want = [c for c in columns if not (interim_ids and c == "finding_id")]
    if sorted(header) != sorted(want):
        die(f"{path}: header {header} is not the column set {want}")
    rows = []
    for n, line in enumerate(lines[1:], 2):
        cells = line.split("\t")
        if len(cells) != len(header):
            die(f"{path}:{n}: {len(cells)} cells, expected {len(header)}")
        row = dict(zip(header, cells))
        if interim_ids:
            row["finding_id"] = f"interim-{n - 1:04d}"
        rows.append(row)
    return rows


def vocab(table, value, where):
    if value not in table:
        die(f"{where}: {value!r} is outside the closed set {sorted(table)}")
    return table[value]


def identity(rows, key, where):
    seen = set()
    for r in rows:
        k = r[key]
        if not k:
            die(f"{where}: empty {key}")
        if k in seen:
            die(f"{where}: duplicate {key} {k!r}")
        seen.add(k)


def module_record(r):
    w = f"modules.tsv {r['module']}"
    parts = []
    for d in DIM_ORDER:
        s = vocab(SITES, r[f"{d}_site"], w + f" {d}_site")
        l = vocab(LEVELS, r[f"{d}_level"], w + f" {d}_level")
        parts.append(f"{d}: DimensionReading {{ site: {s}, level: {l} }}")
    if not r["ambient_calls"].isdigit():
        die(f"{w}: ambient_calls {r['ambient_calls']!r} is not a natural number")
    return ("  ModuleCensusReading { module: %s as NonEmptyStr, module_class: %s, %s, ambient_calls: %s, "
            "worst_site: %s, worst_finding: %s, evidence_symbols: %s, phase: %s }," % (
                lit(r["module"]), vocab(CLASSES, r["class"], w + " class"), ", ".join(parts),
                r["ambient_calls"], vocab(SITES, r["worst_site"], w + " worst_site"),
                lit(r["worst_finding"]), lit(r["evidence_symbols"]), lit(r["phase"])))


def finding_record(r):
    w = f"findings.tsv {r['finding_id']}"
    return ("  FindingCensusReading { finding_id: %s as NonEmptyStr, module: %s as NonEmptyStr, "
            "dimension: %s, level: %s, module_class: %s, site: %s, symbol: %s, specimen: %s, note: %s, "
            "existing_authority: %s, fix_shape: %s }," % (
                lit(r["finding_id"]), lit(r["module"]), vocab(DIMS, r["dimension"], w + " dimension"),
                vocab(LEVELS, r["level"], w + " level"), vocab(CLASSES, r["class"], w + " class"),
                vocab(SITES, r["site"], w + " site"), lit(r["symbol"]), lit(r["specimen"]),
                lit(r["note"]), lit(r["existing_authority"]), lit(r["fix_shape"])))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--census-dir", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--pin", required=True)
    ap.add_argument("--interim", action="store_true",
                    help="census not final: no finding_id column; ids are positional and marked interim")
    a = ap.parse_args()
    d = a.census_dir
    modules = read_tsv(os.path.join(d, "modules.tsv"), MODULE_COLUMNS)
    findings = read_tsv(os.path.join(d, "findings.tsv"), FINDING_COLUMNS, interim_ids=a.interim)
    identity(modules, "module", "modules.tsv")
    identity(findings, "finding_id", "findings.tsv")
    known = {m["module"] for m in modules}
    for f in findings:
        if f["module"] not in known:
            die(f"findings.tsv {f['finding_id']}: module {f['module']!r} is not in modules.tsv")
    inputs = []
    for name in ("modules.tsv", "findings.tsv", "classes.tsv", "receipt.json", "method.md"):
        p = os.path.join(d, name)
        if os.path.exists(p):
            inputs.append((name, hashlib.sha256(open(p, "rb").read()).hexdigest()))
    me = hashlib.sha256(open(os.path.abspath(__file__), "rb").read()).hexdigest()
    out = ["module gunbc.realization_census_rows", "",
           "import std.types { String, NonEmptyStr, List }",
           "import gunbc.realization_census {"]
    out += [f"  {n}," for n in (
        ["ModuleCensusReading", "FindingCensusReading", "DimensionReading"] + sorted(SITES.values())
        + sorted(LEVELS.values()) + sorted(CLASSES.values()) + sorted(DIMS.values()))]
    out += ["}", "",
            "// GENERATED by tools/realization_census_rows.py (interim producer; dissolves when a v2 lens",
            "// produces these rows). Do not edit: regenerate. The receipts are in",
            "// docs/probes/realization_census_1dd3eda5_2026-10-24/.", "",
            f"data realization_census_pin: String = {lit(a.pin)}", "",
            "data realization_census_generator_sha256: String = " + lit(me), "",
            "data realization_census_input_sha256: List<String> = ["]
    out += [f"  {lit(n + '=' + h)}," for n, h in inputs]
    out += ["]", "", f"data realization_census_interim: String = {lit('true' if a.interim else 'false')}", "",
            "data realization_census_module_rows: List<ModuleCensusReading> = ["]
    out += [module_record(r) for r in modules]
    out += ["]", "", "data realization_census_finding_rows: List<FindingCensusReading> = ["]
    out += [finding_record(r) for r in findings]
    out += ["]", ""]
    with open(a.out, "w", encoding="utf-8") as f:
        f.write("\n".join(out))
    print(f"wrote {a.out}: {len(modules)} modules, {len(findings)} findings, generator {me[:12]}")


if __name__ == "__main__":
    main()
