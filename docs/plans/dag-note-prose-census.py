#!/usr/bin/env python3
"""Census of prose-bearing String rows in the .dag corpus.

Re-derives every number in docs/plans/dag-note-prose-census.md. Run from the
repo root:  python3 docs/plans/dag-note-prose-census.py

SCAFFOLD. Dissolution trigger: the typed carriers (ruling register, event log,
citation edge) land and the annotation-budget lens counts ROWS. At that point a
lexical census over prose is superseded by a fold over typed rows, and this
file is deleted. It is a measurement instrument, never an enforcement gate --
nothing in CI consumes it.

HONESTY BOUND (DESIGN §5): the input is unstructured English, so detection is
lexical. Hand-verification on a 35-sentence stratified sample put primary-class
precision at ~70%, with a KNOWN DIRECTIONAL BIAS: SPEC_NORM is understated (the
UNCLASSIFIED residue is almost entirely missed SPEC_NORM), EVENT and RULING are
overstated (incidental "operator's" / "wave N" keywords fire them). So the
history share is an UPPER bound and the spec/norm share a LOWER bound. Every
number below is reported as a measurement with that bias named, never as a
proof.
"""
import json
import os
import re
import sys
from collections import Counter, defaultdict

MIN_PROSE = 200  # bytes; below this a String row is a tag/path/name, not prose

# ---------------------------------------------------------------- extraction

DECL = re.compile(r'^(\s*)data\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*String\s*=\s*"')
FIELD = re.compile(r'^(\s*)([a-z_][A-Za-z0-9_]*)\s*:\s*"')
LIST_OPEN = re.compile(r'^\s*data\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*List<String>\s*=\s*\[')
LIST_ELEM = re.compile(r'^(\s+)"')
ESCAPES = {"n": "\n", "t": "\t", "r": "\r", '"': '"', "\\": "\\"}


def unescape(raw):
    out, i = [], 0
    while i < len(raw):
        if raw[i] == "\\" and i + 1 < len(raw):
            out.append(ESCAPES.get(raw[i + 1], raw[i + 1]))
            i += 2
        else:
            out.append(raw[i])
            i += 1
    return "".join(out)


def scan_string(line, start):
    """start = index of the opening quote; returns raw inner text or None."""
    i, buf = start + 1, []
    while i < len(line):
        if line[i] == "\\":
            buf.append(line[i:i + 2])
            i += 2
            continue
        if line[i] == '"':
            return "".join(buf)
        buf.append(line[i])
        i += 1
    return None


def extract(root="."):
    rows = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d != ".git"]
        for fn in sorted(filenames):
            if not fn.endswith(".dag"):
                continue
            path = os.path.join(dirpath, fn)
            rel = path[2:] if path.startswith("./") else path
            cur_list = None
            with open(path, encoding="utf-8") as fh:
                for lineno, line in enumerate(fh, 1):
                    mo = LIST_OPEN.match(line)
                    if mo:
                        cur_list = mo.group(1)
                    elif cur_list and line.lstrip().startswith("]"):
                        cur_list = None
                    if cur_list:
                        me = LIST_ELEM.match(line)
                        if me:
                            raw = scan_string(line, me.end() - 1)
                            if raw is not None:
                                add(rows, "list_elem", rel, lineno, cur_list, raw)
                            continue
                    m = DECL.match(line)
                    if m:
                        raw = scan_string(line, m.end() - 1)
                        if raw is not None:
                            add(rows, "data", rel, lineno, m.group(2), raw)
                        continue
                    m = FIELD.match(line)
                    if m and m.group(1):  # indented => inside a record literal
                        raw = scan_string(line, m.end() - 1)
                        if raw is not None:
                            add(rows, "field", rel, lineno, m.group(2), raw)
    return rows


def add(rows, kind, rel, lineno, name, raw):
    text = unescape(raw)
    rows.append({"kind": kind, "file": rel, "line": lineno, "name": name,
                 "bytes": len(text.encode("utf-8")), "text": text})


# ------------------------------------------------------------ classification

RULING = [
    r"operator[- ](?:ruling|signed|acceptance|review|direct|align|rul|call|decision|sequenc|pin)",
    r"OPERATOR[- ]?SIGNED", r"\bper operator\b", r"\(operator", r"operator's\b",
    r"\bsigned[- ]off\b", r"\bratified\b", r"review-pinned", r"\bruling\b",
    r"\bconfession grade\b", r"\bmandate", r"\bsign-?off\b", r"\bparent 20\d\d",
]
EVENT = [
    r"#\d{3,5}\b", r"\bLANDED\b", r"\blanded\b", r"\bfired\b",
    r"\b(?:was|were|has been|had been|have been) (?:removed|deleted|dissolved|renamed|"
    r"replaced|flipped|retired|added|fixed|corrected|repaired|restored|shipped|caught|"
    r"exposed|promoted|dropped|dead|green|red)\b",
    r"\bnow dissolved\b", r"\bmerge(?:d)? [0-9a-f]{7,40}\b", r"\b[0-9a-f]{10,40}\b",
    r"\b20\d\d-\d\d-\d\d\b", r"\braised\b[^.]{0,40}(?:to|->|→)", r"\bregression\b",
    r"\bformerly\b", r"\bused to\b", r"\bpreviously\b", r"\bhistoric",
    r"\bsession [a-z]+-[a-z]+-\d+", r"\bpre-fix\b", r"\bretired\b",
    r"\bgreen-by-execution\b", r"\bat flip time\b", r"\bthe first landing\b",
    r"\bsame-day\b", r"\bPR [A-Z0-9]\b", r"\bwave \d\b", r"\bturned on\b",
    r"\bflip(?:ped)? (?:on|to)\b", r"\bonce (?:was|were)\b",
    r"\bthe (?:20\d\d|prior|original|initial) ", r"\bshipped\b", r"\bincident\b",
]
RECEIPT = [
    r"\b\d+(?:\.\d+)?\s*(?:ms|sec|min|hrs?)\b",
    r"\b\d+(?:\.\d+)?\s*(?:Gi?B|Mi?B|Ki?B|bytes)\b",
    r"\b\d+(?:\.\d+)?\s*(?:%|percent)\b", r"\bmeasured\b",
    r"\b[\d,]+\s*(?:->|→)\s*[\d,]+\b", r"\b\d+(?:\.\d+)?\s*[x×]\b", r"\bn\s*=\s*\d+",
    r"\bcount(?:er)?s? (?:is|was|are|were)\s+\d+",
    r"\b[\d,]{2,}\s+(?:rows|entries|errors|sites|files|violations|occurrences|calls|lines)\b",
    r"\bbudget\b[^.]{0,30}\d", r"\btimeout\b[^.]{0,30}\d", r"\bplateau\b",
    r"\bwall\b[^.]{0,30}\d", r"\bA/B\b", r"\bbyte-identical\b",
]
XREF = [
    r"\bsee\b\s+\S", r"docs/plans/", r"DESIGN\s*(?:§|section)", r"\b§\s*\d",
    r"\b(?:src|dag)/[\w/]+\.(?:dag|rs|md|ya?ml)\b",
    r"\b(?:std|extdeps|gunbc|v1|v2|tools|compiler|workflow|lens)\.[a-z_][\w.]*\b",
    r"\b[a-z_]{3,}\.[a-z_]{3,}(?:\.[a-z_]+)+\b",
    r"\bsame (?:shape|move|pattern|posture|axis|case) as\b", r"\bcf\.\b",
    r"\bparallel to\b", r"\bmirrors?\b",
    r"\bper (?:the )?[\w ]{0,20}(?:design|plan|doc|thread|lane)\b",
    r"\banalogue of\b", r"\bprecedent", r"\bthe (?:same|one) authority\b",
]
RATIONALE = [
    r"\bbecause\b", r"\btherefore\b", r"\bso that\b", r"\bsince\b", r"\bthe reason\b",
    r"\bwhy\b", r"\bwould (?:be|have|let|make|leak|force|need|keep|pin|race|fail|break)\b",
    r"\bnot\b[^.]{0,60}\bbut\b", r"\botherwise\b", r"\bhence\b", r"\brather than\b",
    r"\binstead of\b", r"\bthe tell\b", r"\bso\s+(?:the|it|a|that|they)\b",
    r"\bwhich (?:is|means|makes|would)\b", r"\bif\b[^.]{0,60}\bthen\b", r"\bconflat",
    r"\bthe point is\b", r"\bby construction\b", r"\bthe whole difference\b",
]
SPEC_NORM = [
    r"\b(?:is|are)\s+(?:the|a|an|one|not|only)\b", r"\bcarries\b", r"\bholds?\b",
    r"\broutes?\b", r"\breads?\b", r"\bwrites?\b", r"\bderives?\b", r"\byields?\b",
    r"\breturns?\b", r"\bencodes?\b", r"\bmodels?\b", r"\bprojects?\b", r"\bbinds?\b",
    r"\bselects?\b", r"\bresolves?\b", r"\bdescend", r"\bskipped\b", r"\bdispatch",
    r"\bnever\b", r"\bmust\b", r"\balways\b", r"\bonly\b", r"\bprefer\b",
    r"\bdo not\b", r"\bfail[- ]closed\b", r"\brefuses?\b", r"\bthe single authority\b",
]
# present normative force or an UN-fired trigger => the row still binds
LIVE = [
    r"\b(?:must|never|always|shall)\b", r"\bis the (?:single )?authority\b",
    r"\brefuses?\b", r"\bfail[- ]closed\b", r"\bremains?\b", r"\bstays?\b",
    r"\bdissolution trigger\b", r"\bdissolves? (?:when|on|to|into)\b",
    r"\bdissolve[- ]on\b", r"\buntil\b", r"\bTODO\b", r"\bpending\b", r"\bremaining\b",
    r"\bnot yet\b", r"\bstill\b", r"\bslated\b", r"\bprefer\b", r"\bdo not\b",
    r"\bdon't\b", r"\brequires?\b", r"\bexpects?\b", r"\bgates?\b", r"\bblocks?\b",
    r"\bscaffold", r"\bforward-looking\b", r"\bfuture\b", r"\bwill\b", r"\bshould\b",
    r"\bcannot\b", r"\bmay not\b", r"\bis not\b", r"\bkeeps?\b", r"\bintended\b",
    r"\bnext rung\b", r"\bopen\b", r"\bonly\b", r"\bunbounded\b", r"\bdeferred\b",
]

SETS = [("RULING", RULING), ("RECEIPT", RECEIPT), ("EVENT", EVENT),
        ("RATIONALE", RATIONALE), ("XREF", XREF), ("SPEC_NORM", SPEC_NORM)]
COMPILED = {k: [re.compile(p, re.I) for p in v] for k, v in SETS}
LIVE_C = [re.compile(p, re.I) for p in LIVE]
ORDER = [k for k, _ in SETS]
SENT = re.compile(r"(?<=[.!?;])\s+(?=[A-Z(\[])")


def sentences(text):
    return [s.strip() for s in SENT.split(text) if s.strip()] or [text.strip()]


def label(s):
    sc = {k: sum(1 for p in ps if p.search(s)) for k, ps in COMPILED.items()}
    live = sum(1 for p in LIVE_C if p.search(s))
    primary = (max(ORDER, key=lambda k: (sc[k], -ORDER.index(k)))
               if any(sc.values()) else "UNCLASSIFIED")
    return sc, primary, live


# ------------------------------------------------- crisp deletable genres

FIRED = [re.compile(p, re.I) for p in [
    r"\bDISSOLVED\b\s+20\d\d-\d\d-\d\d", r"dissolution trigger fired",
    r"\btrigger (?:has )?fired\b", r"^\s*DISSOLVED\b",
    r"\bthe recorded .{0,30}trigger fired\b",
    r"\b(?:now )?(?:fully )?dissolved\b.{0,40}\b20\d\d-\d\d-\d\d",
]]
DATED = re.compile(r"_(20\d\d)_(\d\d)_(\d\d)_|_(20\d\d)(\d\d)(\d\d)_")
SNAPSHOT = re.compile(r"refresh|sweep|snapshot|census|probe|audit", re.I)


def crisp_deletable(rows):
    """Fired-dissolution / superseded-snapshot rows that bind NOTHING live.

    The live==0 gate is load-bearing, not a refinement: hand-reading the
    ungated output showed the dominant false-positive mode is a row recording
    that a SUB-PART dissolved while the row itself still binds (an un-fired
    "Dissolution trigger:" further down, or a "pending" clause). Requiring zero
    live-force markers drops exactly those. A candidate is still a proposal for
    hand-review, never an automatic deletion.
    """
    fired = [r for r in rows
             if any(p.search(r["text"]) for p in FIRED) and not r.get("live")]
    stems = defaultdict(list)
    for r in rows:
        m = DATED.search(r["name"])
        if not m or not (SNAPSHOT.search(r["name"]) or SNAPSHOT.search(r["text"][:200])):
            continue
        g = [x for x in m.groups() if x]
        stems[(r["file"], DATED.sub("_@_", r["name"]))].append(("".join(g[:3]), r))
    superseded = []
    for lst in stems.values():
        if len(lst) < 2:
            continue
        lst.sort(key=lambda t: t[0])
        for _, r in lst[:-1]:
            r["superseded_by"] = lst[-1][1]["name"]
            superseded.append(r)
    return fired, superseded


# ------------------------------------------------------------------ report

def main():
    rows = extract(sys.argv[1] if len(sys.argv) > 1 else ".")
    prose = [r for r in rows if r["bytes"] >= MIN_PROSE]
    short = [r for r in rows if r["bytes"] < MIN_PROSE]

    print("=" * 68)
    print("POPULATION")
    print("=" * 68)
    for kind in ("data", "field", "list_elem"):
        k = [r for r in rows if r["kind"] == kind]
        kp = [r for r in k if r["bytes"] >= MIN_PROSE]
        print(f"  {kind:10s} total n={len(k):5d} {sum(r['bytes'] for r in k)/1024:8.1f} KiB"
              f"   | prose(>={MIN_PROSE}B) n={len(kp):5d} {sum(r['bytes'] for r in kp)/1024:8.1f} KiB")
    note = [r for r in prose if r["kind"] == "data" and "note" in r["name"]]
    print(f"  {'':10s} of prose-data rows, *note*-named: n={len(note)} "
          f"{sum(r['bytes'] for r in note)/1024:.1f} KiB")
    tot = sum(r["bytes"] for r in prose)
    print(f"\n  PROSE TOTAL n={len(prose)}  {tot/1024:.1f} KiB "
          f"(short tag/path strings excluded: n={len(short)}, "
          f"{sum(r['bytes'] for r in short)/1024:.1f} KiB)")

    all_s = []
    for r in prose:
        r["sents"] = []
        for s in sentences(r["text"]):
            sc, primary, live = label(s)
            r["sents"].append({"bytes": len(s.encode()), "cls": sc,
                               "primary": primary, "live": live})
        r["live"] = sum(x["live"] for x in r["sents"])
        all_s.extend(r["sents"])
    ns = len(all_s)
    nsb = sum(s["bytes"] for s in all_s)

    print("\n" + "=" * 68)
    print(f"SENTENCE PRIMARY CLASS  (n={ns}, {nsb/1024:.1f} KiB)")
    print("=" * 68)
    c = Counter(s["primary"] for s in all_s)
    for k in ORDER + ["UNCLASSIFIED"]:
        b = sum(s["bytes"] for s in all_s if s["primary"] == k)
        print(f"  {k:13s} n={c[k]:5d} ({100*c[k]/ns:4.1f}%)   {b/1024:7.1f} KiB ({100*b/nsb:4.1f}%)")

    print("\n-- multi-label incidence (a sentence can carry several) --")
    for k, _ in SETS:
        n = sum(1 for s in all_s if s["cls"][k])
        print(f"  {k:13s} n={n:5d} ({100*n/ns:4.1f}%)")

    print("\n" + "=" * 68)
    print("MIXING: distinct primary classes per note")
    print("=" * 68)
    cc = Counter(len({s["primary"] for s in r["sents"]} - {"UNCLASSIFIED"}) for r in prose)
    mixed = mixedb = 0
    for n in sorted(cc):
        b = sum(r["bytes"] for r in prose
                if len({s["primary"] for s in r["sents"]} - {"UNCLASSIFIED"}) == n)
        print(f"  {n} classes: n={cc[n]:5d}  {b/1024:7.1f} KiB")
        if n >= 2:
            mixed += cc[n]
            mixedb += b
    print(f"  >=2 classes (the anemic-serialization mass): n={mixed} "
          f"{mixedb/1024:.1f} KiB ({100*mixedb/tot:.0f}% of prose)")

    print("\n" + "=" * 68)
    print("HISTORY MASS (upper bound -- EVENT/RULING are overstated)")
    print("=" * 68)
    hs = [s for s in all_s if s["primary"] in ("EVENT", "RECEIPT") and s["live"] == 0]
    print(f"  history-class sentences (no live force): n={len(hs)}  "
          f"{sum(s['bytes'] for s in hs)/1024:.1f} KiB ({100*sum(s['bytes'] for s in hs)/nsb:.1f}%)")
    for thr in (0.4, 0.6):
        sel = [r for r in prose
               if sum(s["bytes"] for s in r["sents"] if s["primary"] in ("EVENT", "RECEIPT"))
               >= thr * max(1, sum(s["bytes"] for s in r["sents"]))]
        print(f"  notes >={int(thr*100)}% history sentences:   n={len(sel):4d}  "
              f"{sum(r['bytes'] for r in sel)/1024:7.1f} KiB "
              f"({100*sum(r['bytes'] for r in sel)/tot:.1f}%)")

    fired, superseded = crisp_deletable(prose)
    union = {(r["file"], r["line"]): r for r in fired + superseded}
    ub = sum(r["bytes"] for r in union.values())
    print(f"\n  CRISP DELETABLE (hand-checkable one by one): n={len(union)}  "
          f"{ub/1024:.1f} KiB ({100*ub/tot:.1f}% of prose)")
    for r in sorted(union.values(), key=lambda r: -r["bytes"]):
        tag = f"  <- superseded by {r['superseded_by']}" if "superseded_by" in r else ""
        print(f"    {r['bytes']:5d}B  {r['file']}:{r['line']}  {r['name']}{tag}")

    print("\n" + "=" * 68)
    print("CONCENTRATION")
    print("=" * 68)
    byf = Counter()
    for r in prose:
        byf[r["file"]] += r["bytes"]
    cum = 0
    for i, (f, b) in enumerate(byf.most_common(), 1):
        cum += b
        if cum >= tot * 0.5:
            print(f"  50% of prose bytes live in {i} of {len(byf)} prose-bearing files")
            break
    for f, b in byf.most_common(10):
        print(f"    {b/1024:6.1f} KiB  {f}")

    print("\n  the already-invented target model (closure|rationale|trigger):")
    proto = [r for r in rows if re.search(r"closure:[^\"]*\|trigger:", r["text"])]
    print(f"    n={len(proto)}  {sum(r['bytes'] for r in proto)/1024:.1f} KiB")


if __name__ == "__main__":
    main()
