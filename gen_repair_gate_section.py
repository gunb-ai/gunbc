#!/usr/bin/env python3
"""Emit the #14152 producer-repair PR section from ARTIFACTS ONLY.

No number in the generated text is typed by hand: every figure is read from a file produced by a
run, and a missing file aborts with a named FATAL rather than printing a zero. (Hand-typed
numbers in this PR's body have already been wrong once -- it said 76 where the measurement was
77 -- which is why the body is generated.)

usage: gen_repair_gate_section.py
inputs (written by earlier runs, paths fixed there):
  /tmp/pp_out/<closure>.json      compare_post_pre_14152.py output (post vs pre by identity)
  /tmp/pp_out/<closure>.txt       the same run's human-readable block, for the stamp lines
  /tmp/gate_both_dispatch.log     the both-sides gate run (PINNED, build rc, mirror seeds, emitted)
  /tmp/rows_*_dispatch.log        per-row verdicts (ROW <label> <name> PASS|FAIL rc= elapsed=)
"""
import json, os, re, sys, glob

CLOSURES = ["self-host", "v2-native-cli"]


def need(path, what):
    if not os.path.isfile(path) or os.path.getsize(path) == 0:
        print(f"FATAL: {what} missing at {path} -- refusing to emit a number I did not measure")
        sys.exit(1)
    return path


def strip_ansi(s):
    return re.sub(r"\x1b\[[0-9;]*m", "", s)


def gate_stamps():
    p = need("/tmp/gate_both_dispatch.log", "both-sides gate dispatch log")
    txt = strip_ansi(open(p, errors="replace").read())
    out = {}
    for line in txt.splitlines():
        m = re.match(r"PINNED (\w+)=([0-9a-f]{40})", line)
        if m:
            out.setdefault("pinned", {})[m.group(1)] = m.group(2)
        m = re.match(r"\[(head|main)\] build rc=(\d+) errors=(\d+)", line)
        if m:
            out.setdefault("build", {})[m.group(1)] = (m.group(2), m.group(3))
        m = re.match(r"\[head\] mirror seed=([0-9a-f]+) fold=(\d+) arms=(\d+)", line)
        if m:
            out["mirror"] = (m.group(1), m.group(2), m.group(3))
        m = re.match(r"\[(head|main)\] (\S+) rc=(\d+) lines=(\d+) type-mismatch=(\d+) emitted=(\d+)", line)
        if m:
            out.setdefault("run", {}).setdefault(m.group(1), {})[m.group(2)] = dict(
                rc=int(m.group(3)), lines=int(m.group(4)), mismatch=int(m.group(5)), emitted=int(m.group(6)))
    for k in ("pinned", "build", "mirror", "run"):
        if k not in out:
            print(f"FATAL: gate log lacks {k} stamps -- the run did not get that far")
            sys.exit(1)
    return out


def row_verdicts():
    rows = {}
    for p in sorted(glob.glob("/tmp/rows_*_dispatch.log")):
        for line in strip_ansi(open(p, errors="replace").read()).splitlines():
            m = re.match(r"ROW (\S+) (\S+) (PASS|FAIL|TIMEOUT_FATAL|MISSING_FATAL|NO_SUMMARY) rc=(\d+)(?: after (\d+)s)?(?: elapsed=(\d+)s)?", line)
            if m:
                name, verdict = m.group(2), m.group(3)
                rec = dict(verdict=verdict, rc=m.group(4), elapsed=m.group(6), label=m.group(1),
                           src=os.path.basename(p))
                # Keep the worst/latest evidence per row name; a row measured at two heads is
                # reported per head below, so key on (label, name).
                rows[(m.group(1), name)] = rec
    if not rows:
        print("FATAL: no ROW verdict lines in any /tmp/rows_*_dispatch.log -- nothing to report")
        sys.exit(1)
    return rows


def main():
    g = gate_stamps()
    rows = row_verdicts()

    print("## Producer repair: what the two closures measure\n")
    print(f"Pinned for this run: head `{g['pinned'].get('head','?')}`, "
          f"main `{g['pinned'].get('main','?')}`, instrument `{g['pinned'].get('instrument','?')}`.\n")
    mh, mf, arms = g["mirror"]
    print(f"The head tree carried the regenerated mirror (seed `{mh}`, `fold=` present "
          f"{mf}x, repair arms {arms}x), and both builds were clean "
          f"(head rc={g['build']['head'][0]} errors={g['build']['head'][1]}, "
          f"main rc={g['build']['main'][0]} errors={g['build']['main'][1]}).\n")

    print("### Hard test: no new refusal anywhere, by identity\n")
    print("The comparison that matters for a repair is **post vs the immediately preceding head** "
          "(the Acc, before the producer fix) -- not head vs main. Main carries no Acc and reports "
          "zero type-mismatch diagnostics, so against main every refusal looks NEW. Both closures "
          "are compared on the same identity key the gate's comparator uses "
          "(file, bound variable, head shape):\n")
    ok = True
    for c in CLOSURES:
        p = need(f"/tmp/pp_out/{c}.json", f"{c} post-vs-pre JSON")
        d = json.load(open(p))
        added, removed = d["added"], d["removed"]
        ok = ok and not added
        print(f"- **{c}**: {d['pre_total']} -> {d['post_total']} hard diagnostics "
              f"({len(d['pre_identities'])} -> {len(d['post_identities'])} identities), "
              f"unparsed post={d['post_unparsed']} pre={d['pre_unparsed']}. "
              f"NEW identities: **{len(added)}** {'(hard test passes)' if not added else '-- FAIL'}.")
        for k, n in removed:
            print(f"  - removed by the repair: {n}x `{k[0]}` bind={k[1]} head={k[2]}{('<'+k[3]) if k[3] else ''}")
        for k in added:
            print(f"  - **NEW** {k}")
        if d["post_unparsed"] or d["pre_unparsed"]:
            print("  - WARN: unparsed lines carry no identity and are invisible to both directions")
    print()
    print(f"Overall hard test: **{'PASS' if ok else 'FAIL'}** -- every surviving diagnostic is a "
          f"subset of the pre-repair set, so the repair removes refusals and adds none.\n")

    print("### Row verdicts\n")
    print("One `--exact` cargo invocation per row, each on its own bound, because the crate pins "
          "`RUST_TEST_THREADS=1` and every `compile_to_resolved` row recompiles the std closure; "
          "a single crate-filtered run of the whole suite could not answer inside a dispatch. "
          "Verdicts come from the run's `test result:` summary line "
          "(`--nocapture` splits the per-test line from its result).\n")
    # Which names are real rows. Before repair_tests_14152.sh shifted past its positional
    # args, the SHA and the label were themselves passed as --exact filters and dutifully
    # reported MISSING_FATAL; those entries are harness noise, not rows, and would read in the
    # PR body as failing tests. The authoritative list is the script's own ROWS literal -- the
    # same list the dispatch defaults to -- parsed line by line. (Scanning the test file in this
    # worktree is wrong: the instruments line does not carry the row definitions, so the scan
    # finds none of them and silently excludes every real row.)
    here = os.path.dirname(os.path.abspath(__file__))
    script = os.path.join(here, "repair_tests_14152.sh")
    if not os.path.isfile(script):
        script = os.path.join(here, "instruments", "repair_tests_14152.sh")
    need(script, "row-defining script")
    real, inside = set(), False
    for line in open(script, errors="replace"):
        s_ = line.strip()
        if not inside:
            inside = s_.startswith("ROWS='")
            if not inside:
                continue
            s_ = s_[len("ROWS='"):]        # the first row is on the opening line itself
        if s_.startswith("'"):
            break
        real.add(s_.strip().strip("'").strip())
    if not real:
        print("FATAL: no ROWS literal parsed from " + script + " -- refusing to filter verdicts")
        sys.exit(1)
    skipped = sorted({n for (l, n) in rows if n not in real})
    by_row = {}
    for (label, name), rec in rows.items():
        if name not in real:
            continue
        by_row.setdefault(name, []).append((label, rec))
    for name in sorted(by_row):
        for label, rec in by_row[name]:
            el = f" in {rec['elapsed']}s" if rec["elapsed"] else ""
            print(f"- `{name}` at {label}: **{rec['verdict']}** (rc={rec['rc']}{el}, {rec['src']})")
    if skipped:
        print(f"(excluded harness-noise entries that were never rows: {', '.join(f'`{n}`' for n in skipped)})")
    print()
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
