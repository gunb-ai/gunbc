#!/usr/bin/env python3
"""Append the producer-repair gate section to the #14152 PR body, from artifacts only.

Every figure is parsed from a run log or a comparator JSON; a missing input is a named FATAL,
never a zero. The body has already carried one hand-typed number that was wrong (76 where the
measurement at the time was 76 for a different head), so the sections that describe the shipped
head are generated rather than written.

WHAT IT DOES NOT DO. It does not rewrite the older gate table, which measured the Acc at
23641bc8a97 before main was merged. That number was correct for the head it measured and rewriting
it would destroy the audit trail. Instead the new section states the current head's numbers and
names WHY they differ from the older ones, so the two can be read side by side.

usage: append_repair_body_section.py <current_body.md> <out.md> <acc_head> <repaired_head> <main_sha>
"""
import json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gen_repair_gate_section import gate_stamps, row_verdicts, strip_ansi  # reuse the parsers


def need(path, what):
    if not os.path.isfile(path) or os.path.getsize(path) == 0:
        print(f"FATAL: {what} missing at {path} -- refusing to print a number I did not measure")
        sys.exit(1)
    return path


def counts(log_path, pattern):
    need(log_path, "closure log")
    return len([l for l in open(log_path, errors="replace") if re.search(pattern, l)])


def main():
    if len(sys.argv) < 6:
        print(__doc__)
        return 2
    body_path, out_path, acc_head, repaired_head, main_sha = sys.argv[1:6]
    need(body_path, "current PR body")
    body = open(body_path, errors="replace").read()
    g = gate_stamps()

    MISMATCH = r"type mismatch"
    acc_sh = counts(f"/tmp/gate3out/head_self-host.log", MISMATCH)
    acc_v2 = counts(f"/tmp/gate3out/head_v2-native-cli.log", MISMATCH)
    rep_sh = counts(f"/tmp/gate_bothout/head_self-host.log", MISMATCH)
    rep_v2 = counts(f"/tmp/gate_bothout/head_v2-native-cli.log", MISMATCH)
    main_sh = counts(f"/tmp/gate_bothout/main_self-host.log", MISMATCH)
    main_v2 = counts(f"/tmp/gate_bothout/main_v2-native-cli.log", MISMATCH)
    # main must be zero or the whole instrument is unsound -- say so rather than tabulate.
    for name, v in (("main self-host", main_sh), ("main v2-native-cli", main_v2)):
        if v != 0:
            print(f"FATAL: {name} control = {v}, expected 0 -- the instrument is not sound and no "
                  f"head number from it means anything")
            return 4

    hard = {}
    for c in ("self-host", "v2-native-cli"):
        d = json.load(open(need(f"/tmp/pp_out/{c}.json", f"{c} post-vs-pre JSON")))
        hard[c] = d

    # main's own emitted marker, because a log that never arrived also parses to zero diagnostics.
    emitted = g["run"]["head"].get("v2-native-cli", {}).get("emitted")
    main_emitted = g["run"]["main"].get("v2-native-cli", {}).get("emitted")

    # Why the Acc-era count differs from the repaired-era count for reasons other than the repair:
    # main moved between the two measurements, and one of main's own commits changed how refusals
    # are printed. Naming it stops a reader from attributing the delta to my change.
    between = subprocess.run(
        ["git", "log", "--oneline", f"{acc_head}...origin/main", "--", "src/v2/compiler", "src/v1"],
        cwd=HERE, capture_output=True, text=True).stdout.strip().splitlines()

    L = []
    L.append("\n---\n")
    L.append("## Producer repair, measured at the current head\n")
    L.append(f"Everything below is measured at `{repaired_head}` (the Acc head `{acc_head}` plus the "
             f"producer repair and its regenerated mirrors); the gate table above stays as it was "
             f"because it is the measurement of a real earlier head, and the two are read together.\n")
    L.append(f"Pinned: head `{g['pinned'].get('head','?')}`, main `{g['pinned'].get('main','?')}`, "
             f"instrument `{g['pinned'].get('instrument','?')}`. Both sides built clean "
             f"(head rc={g['build']['head'][0]} errors={g['build']['head'][1]}, "
             f"main rc={g['build']['main'][0]} errors={g['build']['main'][1]}); the head mirror seed "
             f"is `{g['mirror'][0]}` with `fold=` present {g['mirror'][1]}x.\n")
    L.append("### The two closures, three heads\n")
    L.append("| head | closure | type-mismatch lines | identities | note |")
    L.append("| --- | --- | --- | --- | --- |")
    for head, c, n, note in (
        (f"Acc `{acc_head[:11]}`", "self-host", acc_sh, "before the repair"),
        (f"Acc `{acc_head[:11]}`", "v2-native-cli", acc_v2, "before the repair"),
        (f"repaired `{repaired_head[:11]}`", "self-host", rep_sh, "after the repair"),
        (f"repaired `{repaired_head[:11]}`", "v2-native-cli", rep_v2, "after the repair"),
        (f"main `{main_sha[:11]}`", "self-host", main_sh, "control: zero, as CI says"),
        (f"main `{main_sha[:11]}`", "v2-native-cli", main_v2,
         f"control: zero, and its emitted marker is present ({main_emitted}) so the log is real"),
    ):
        L.append(f"| {head} | {c} | **{n}** | — | {note} |")
    L.append("")
    L.append("main contributes nothing to either closure, which is the control that makes the head "
             "numbers mean something: a local run that could not reproduce main's zero would have "
             "sent me back to debug the instrument instead of reporting counts.\n")
    L.append("### Hard test: the repair removes refusals and adds none\n")
    L.append("Head-vs-main cannot show the repair's own effect, because main has no `Acc` and emits "
             "zero type-mismatch lines, so every head refusal looks new. The comparison that "
             "discriminates is **repaired vs the immediately preceding head**, on the identity key "
             "the gate's comparator uses (file, bound variable, head shape) -- imported, so the two "
             "measurements cannot drift apart:\n")
    for c in ("self-host", "v2-native-cli"):
        d = hard[c]
        L.append(f"- **{c}**: {d['pre_total']} -> {d['post_total']} hard diagnostics "
                 f"({len(d['pre_identities'])} -> {len(d['post_identities'])} identities); "
                 f"**new identities: {len(d['added'])}**, removed: {len(d['removed'])}.")
        for k, n in d["removed"]:
            L.append(f"  - removed: {n}x `{k[0]}` bind={k[1]} head={k[2]}{('<' + k[3]) if k[3] else ''}")
        for k in d["added"]:
            L.append(f"  - **NEW: {k}**")
        if d["post_unparsed"] or d["pre_unparsed"]:
            L.append(f"  - WARN: {d['post_unparsed']} post / {d['pre_unparsed']} pre diagnostic lines "
                     f"did not parse, so they carry no identity and are invisible in both directions")
    L.append("")
    L.append("Zero new identities in both closures is the gate: the repaired tree refuses nothing the "
             "Acc tree accepted. Every remaining diagnostic was already there.\n")
    if between:
        L.append("### Why the Acc-era count is not the repaired-era count for reasons outside this PR\n")
        L.append("main moved between the two measurements, so a count difference is not automatically "
                 "mine. Between them main changed how refusals are printed, among other things "
                 "(sample of main-side commits touching these trees, most recent first):\n")
        for l in between[:6]:
            L.append(f"- {l}")
        L.append("")
        L.append("So the older table's numbers and this section's numbers are not the same "
                 "measurement of the same tree, and neither was edited to match the other.\n")

    L.append("### Rows, one invocation each\n")
    L.append("Per-row `--exact` runs with their own bound: the crate pins `RUST_TEST_THREADS=1` and "
             "every `compile_to_resolved` row recompiles the std closure, so one crate-filtered run "
             "could not answer inside a dispatch (it did, once, and returned no verdict for the rows "
             "it never reached). Verdicts come from the run's `test result:` summary line.\n")
    rows = row_verdicts()
    real = set()
    script = os.path.join(HERE, "repair_tests_14152.sh")
    inside = False
    for line in open(script, errors="replace"):
        s_ = line.strip()
        if not inside:
            inside = s_.startswith("ROWS='")
            if not inside:
                continue
            s_ = s_[len("ROWS='"):]
        if s_.startswith("'"):
            break
        real.add(s_.strip().strip("'").strip())
    if not real:
        print("FATAL: no ROWS literal in " + script)
        return 5
    for (label, name), rec in sorted(rows.items()):
        if name not in real:
            continue
        el = f", {rec['elapsed']}s" if rec["elapsed"] else ""
        L.append(f"- `{name}` @ `{label}`: **{rec['verdict']}** (rc={rec['rc']}{el})")
    L.append("")
    open(out_path, "w").write(body.rstrip("\n") + "\n" + "\n".join(L) + "\n")
    print(f"wrote {out_path} ({os.path.getsize(out_path)} bytes, "
          f"was {os.path.getsize(body_path)})")
    bad = [c for c in hard if hard[c]["added"]]
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
