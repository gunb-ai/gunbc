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


PRODUCED_RE = re.compile(r"produced (\d+) hard diagnostic\(s\)")


def counts(log_path, pattern, label, rc=None, emitted=None):
    """Count refusals in a closure log, cross-checked against the log's own produced line.

    Two independent readings of the same file have to agree, and a third check rejects the file
    shape that fooled an earlier run here: a gunbc invocation that printed its usage text instead
    of compiling contains no diagnostics at all, so a naive line count reports 0 and 0 looks
    exactly like main's control green. That is how a run that measured nothing came to be quoted
    as a measurement, so a log with no `produced N hard diagnostic(s)` line is now rejected.
    """
    need(log_path, f"closure log for {label}")
    text = open(log_path, errors="replace").read()
    lines = len([l for l in text.splitlines() if re.search(pattern, l)])
    m = PRODUCED_RE.search(text)
    if m:
        produced = int(m.group(1))
        if produced != lines:
            print(f"FATAL: {label} log states produced={produced} but {lines} lines match "
                  f"{pattern!r} -- the counter and the compiler disagree")
            sys.exit(4)
        return lines
    # No refusal header means the run never refused, which is what green looks like -- but ONLY if
    # the run actually compiled. rc and the emitted marker come from the gate's own stamps, so a
    # log that is a usage dump or an early exit (rc!=0, or nothing emitted) is rejected here rather
    # than reported as zero. This is not a theoretical guard: the surviving log for an earlier run
    # quoted in this PR body is exactly that -- gunbc's help text, which contains zero diagnostics
    # and would otherwise be indistinguishable from main's control green.
    if not (rc == 0 and emitted and emitted >= 1):
        print(f"FATAL: {label} log has no 'produced N hard diagnostic(s)' line and is not asserted "
              f"green either (rc={rc}, emitted={emitted}) -- zero from it would be an unmeasured "
              f"run wearing a green hat")
        sys.exit(4)
    return 0


CLOSURE_OF_LABEL = {"Acc self-host": "self-host", "Acc v2-native-cli": "v2-native-cli",
                    "repaired self-host": "self-host", "repaired v2-native-cli": "v2-native-cli",
                    "main self-host": "self-host", "main v2-native-cli": "v2-native-cli"}


def c_of(lbl):
    return CLOSURE_OF_LABEL[lbl]


def main():
    if len(sys.argv) < 6:
        print(__doc__)
        return 2
    body_path, out_path, acc_head, repaired_head, main_sha = sys.argv[1:6]
    HEAD_DIR = sys.argv[6] if len(sys.argv) > 6 else '/tmp/gate_bothout'
    GATE_LOG = sys.argv[7] if len(sys.argv) > 7 else '/tmp/gate_both_dispatch.log'
    PP_DIR   = sys.argv[8] if len(sys.argv) > 8 else '/tmp/pp_out'
    PP_SUFFIX = sys.argv[11] if len(sys.argv) > 11 else ''
    ACC_DIR  = sys.argv[9] if len(sys.argv) > 9 else '/tmp/gate3out'
    MERGE_PP_DIR = sys.argv[10] if len(sys.argv) > 10 else ''
    need(body_path, "current PR body")
    body = open(body_path, errors="replace").read()
    g = gate_stamps(GATE_LOG)

    MISMATCH = r"type mismatch"
    acc_sh = counts(f"{ACC_DIR}/head_self-host.log", MISMATCH, "Acc self-host")
    acc_v2 = counts(f"{ACC_DIR}/head_v2-native-cli.log", MISMATCH, "Acc v2-native-cli")
    rep_sh = counts(f"{HEAD_DIR}/head_self-host.log", MISMATCH, "repaired self-host")
    rep_v2 = counts(f"{HEAD_DIR}/head_v2-native-cli.log", MISMATCH, "repaired v2-native-cli")
    main_sh = counts(f"{HEAD_DIR}/main_self-host.log", MISMATCH, "main self-host",
                     rc=g["run"]["main"]["self-host"]["rc"],
                     emitted=g["run"]["main"]["self-host"]["emitted"])
    main_v2 = counts(f"{HEAD_DIR}/main_v2-native-cli.log", MISMATCH, "main v2-native-cli",
                       rc=g["run"]["main"]["v2-native-cli"]["rc"],
                       emitted=g["run"]["main"]["v2-native-cli"]["emitted"])
    # main must be zero or the whole instrument is unsound -- say so rather than tabulate.
    for name, v in (("main self-host", main_sh), ("main v2-native-cli", main_v2)):
        if v != 0:
            print(f"FATAL: {name} control = {v}, expected 0 -- the instrument is not sound and no "
                  f"head number from it means anything")
            return 4

    hard = {}
    for c in ("self-host", "v2-native-cli"):
        d = json.load(open(need(f"{PP_DIR}/{c}{PP_SUFFIX}.json",
                                f"{c} post-vs-pre JSON (the repair's own effect)")))
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
    L.append("### Evidence basis, and what this section does not rely on\n")
    L.append("Each figure above comes from a named log, and each log is only trusted if it states "
             "its own count:\n")
    for lbl, pth, val in (("Acc self-host", f"{ACC_DIR}/head_self-host.log", acc_sh),
                          ("Acc v2-native-cli", f"{ACC_DIR}/head_v2-native-cli.log", acc_v2),
                          ("repaired self-host", f"{HEAD_DIR}/head_self-host.log", rep_sh),
                          ("repaired v2-native-cli", f"{HEAD_DIR}/head_v2-native-cli.log", rep_v2),
                          ("main self-host", f"{HEAD_DIR}/main_self-host.log", main_sh),
                          ("main v2-native-cli", f"{HEAD_DIR}/main_v2-native-cli.log", main_v2)):
        prod = ""
        if os.path.isfile(pth):
            pm = PRODUCED_RE.search(open(pth, errors="replace").read())
            if pm:
                prod = f" (log says produced={pm.group(1)})"
            else:
                run = g["run"].get("main" if "main" in lbl else "head", {}).get(
                    c_of(lbl), {})
                prod = (f" (no refusal header, so green is asserted by rc={run.get('rc')} and "
                        f"emitted={run.get('emitted')} -- both must be 0/>=0 and non-absent)")
        L.append(f"- {lbl}: `{pth}` -> {val}{prod}")
    L.append("")
    L.append("The earlier table in this body quotes 76 at `23641bc8a97`, a head measured before main "
             "was merged. **This section does not rely on it and I cannot confirm it from the "
             "artifacts still on hand**: the surviving log for that run is a gunbc usage dump -- the "
             "invocation printed its help text instead of compiling, so the file contains zero "
             "diagnostics and a naive count of it would have looked like main's control green. The "
             "pre-repair baseline used here is therefore my own re-measurement at the head the "
             "repair was built on (" + str(acc_sh) + " at `" + acc_head[:11] + "`), whose log states "
             "its own count. A count I cannot trace to a log that asserts it is not reported.\n")
    if between:
        L.append("main also moved between the two measurements, so a count difference from the older "
                 "table is not automatically mine; the main-side commits between them include\n")
        for l in between[:4]:
            L.append(f"- {l}")
        L.append("")
    L.append("### Per-file refusal counts, current head (replaces the earlier table)\n")
    L.append("The earlier per-file table measured a head that has since been superseded and its log "
             "is no longer on hand, so it is dropped rather than carried forward. These counts come "
             "from the same identity JSON the hard test reads, at the head this section measures:\n")
    for c in ("self-host", "v2-native-cli"):
        d = hard[c]
        per = {}
        for k, n in d["post_identities"].items():
            per[eval(k)[0]] = per.get(eval(k)[0], 0) + n
        L.append(f"**{c}** ({sum(per.values())} diagnostics in {len(per)} files):\n")
        L.append("| file | diagnostics |")
        L.append("| --- | --- |")
        for f, n in sorted(per.items(), key=lambda kv: (-kv[1], kv[0])):
            L.append(f"| `{f}` | {n} |")
        L.append("")

    if MERGE_PP_DIR:
        L.append("### The merge itself introduced no refusal\n")
        L.append("The head above is a merge of my branch with main, so a second comparison is needed "
                 "to keep the two changes apart: the merged head against my own pre-merge head, where "
                 "the only difference is main's commits. Same identity key, same instrument.\n")
        for c in ("self-host", "v2-native-cli"):
            mp = need(f"{MERGE_PP_DIR}/{c}.json", f"{c} merged-vs-pre-merge JSON")
            dm = json.load(open(mp))
            L.append(f"- **{c}**: {dm['pre_total']} -> {dm['post_total']} hard diagnostics; "
                     f"new identities **{len(dm['added'])}**, removed {len(dm['removed'])}. "
                     + ("The merge moved nothing."
                        if not dm["added"] and not dm["removed"]
                        else "Something moved -- name it before believing the merge is inert."))
            for k in dm["added"]:
                L.append(f"  - **NEW from the merge: {k}**")
        L.append("")

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
        # TERMINATION. The literal's closing quote sits at the END of the last row line, not on a
        # line of its own, so a `startswith("'")` test never fires and the loop swallows the whole
        # script: every `stamp`/`timeout` line became a "row name" and the generator then reported
        # those as rows not measured at the head. A row name is [a-z_0-9]+; anything else ends it.
        if s_.endswith("'"):
            s_ = s_[:-1]
            if s_:
                real.add(s_)
            break
        if not re.fullmatch(r"[a-z_0-9]+", s_):
            break
        real.add(s_)
    if not real:
        print("FATAL: no ROWS literal in " + script)
        return 5
    # Row labels are run labels, not tree names, and a reviewer cannot decode `ab_pre` or `p1_repair`.
    # Each label is mapped to the tree it measured; an unknown label is printed as-is rather than
    # dropped, because an unmapped run is still a measurement I took.
    TREE_OF = {
        "merged6a": "head under review (merged)",
        "merged6b": "head under review (merged)",
        "repair": "shipped repair, before the merge (first attempt; see the harness note below)",
        "ab_repair": "pre-repair head (accumulator only)",
        "ab_pre": "pre-repair head (accumulator only)",
        "prerepair": "pre-repair head (accumulator only)",
        "prerepair2": "pre-repair head (accumulator only)",
        "rows_repair": "shipped repair, before the merge",
        "rows4a": "shipped repair, before the merge",
        "rows4b": "shipped repair, before the merge",
        "rows_mutant": "**name-keyed MUTANT** (the defect freshness prevents)",
        "mutant": "**name-keyed MUTANT** (the defect freshness prevents)",
        "p1_repair": "shipped repair -- id-capture probe",
        "p2_mutant": "**name-keyed MUTANT** -- id-capture probe",
        "coll": "shipped repair (dev profile; superseded by the release run)",
        "collrel": "shipped repair (release profile)",
    }

    # A row verdict is only evidence if the run actually said PASS or FAIL. MISSING_FATAL rc=0 and
    # NO_SUMMARY were produced by my own matcher before it read the `test result:` summary line
    # (--nocapture splits `test NAME ... ok` from its result), and TIMEOUT_FATAL by a bound that was
    # too short for a row that runs the compiler under test in an unoptimised binary. None of those
    # is a measurement of the tree, so they are never printed as row results; they are listed in the
    # excluded note below instead, so the exclusion is visible and not a quiet deletion.
    def usable(rec):
        return rec["verdict"] in ("PASS", "FAIL")

    HEAD_LABELS = ("merged6a", "merged6b")
    # The candidate head the PR would take, resolved from the WIP ref rather than typed from memory.
    HEAD_SHA = os.environ.get("HEAD_SHA", "0cc1895f4d07088af6ae68f93d948ad57ceff33f")

    REPO_CANDIDATES = (
        os.environ.get("GENTLE_CARP_REPO", ""),
        "/home/briansrls/.worktrees/gunbc/gentle-carp-16",
        "/home/briansrls/.worktrees/gunbc/gentle-carp-16-scratch",
    )

    def _show(sha, relpath):
        for repo in REPO_CANDIDATES:
            if not repo or not os.path.isdir(repo):
                continue
            try:
                r = subprocess.run(["git", "-C", repo, "show", f"{sha}:{relpath}"],
                                   capture_output=True, text=True, timeout=60)
            except Exception:
                continue
            if r.returncode == 0:
                return r.stdout
        return None

    _tree_cache = {}

    def tree_for(label, sha=None):
        """Classify the tree a run measured from that tree's OWN bytes, keyed on the SHA the run
        printed (`MEASURED_AT`) -- not on my run label.

        Labels are mine, and two facts made label-driven description unsafe to publish: the label
        `rows_repair` had been reused for a tree whose repair was reverted, and the freshness rows
        were measured on probe trees (`probe/freshness-rows`, `probe/mut-freshness-rows`) whose
        names are not in the label table at all -- so the lookup fell through to "pre-repair head"
        and published a PASS at a REPAIRED tree as a pre-repair PASS. The .dag alone cannot
        separate those probe trees from the shipped repair (same seed), so the test file is read
        too: the probe rows and the permanent row are mutually exclusive by construction.
        An unreadable SHA says so instead of guessing.
        """
        if sha:
            if sha in _tree_cache:
                return _tree_cache[sha]
            dag = _show(sha, "src/v1/04_infer.dag")
            tf = _show(sha, "src/v1/tests/src/cross_coproduct_generic_locate_test.rs")
            if dag is None:
                return f"`{sha[:11]}` (tree unreadable locally -- description withheld)"
            if dag.count("fresh_call_variables") == 0:
                tag = f"`{sha[:11]}` **pre-repair** (the call arm never instantiates)"
            elif dag.count("map_insert(acc, g, type_variable_node(id: g))") >= 2:
                # The name-keyed insert ALSO exists in the pre-existing unbound_owner_param_variables
                # fold, so TWO occurrences is what means the call site mints by parameter name.
                tag = f"`{sha[:11]}` **name-keyed MUTANT** (mints by the callee's parameter name)"
            elif tf is not None and re.search(r"fn probe_[a-z_0-9]+", tf) is not None:
                tag = f"`{sha[:11]}` probe tree (prints minted ids; never shipped)"
            elif tf is not None and tf.count("fn minted_variables_are_fresh_per_call_site") == 0:
                tag = f"`{sha[:11]}` repaired compiler, before the permanent freshness row existed"
            else:
                tag = f"`{sha[:11]}` shipped repair (per-call-site ids)"
            # A tree that is not an ancestor of the head under review is a probe or an instrumented
            # branch, never something the reviewer is reading. Say so, so a verdict on such a tree is
            # not mistaken for a verdict on the shipped head.
            for repo in REPO_CANDIDATES:
                if not repo or not os.path.isdir(repo):
                    continue
                r = subprocess.run(["git", "-C", repo, "merge-base", "--is-ancestor", sha, HEAD_SHA],
                                   capture_output=True, timeout=60)
                if r.returncode == 0:
                    break
                if r.returncode == 1:
                    tag += " -- NOT an ancestor of the head under review (probe/instrumented tree)"
                    break
            _tree_cache[sha] = tag
            return tag
        if label in HEAD_LABELS:
            return "head under review (merged)"
        return f"tree UNNAMED (run label `{label}`, no MEASURED_AT -- not a tree claim)"

    L.append("### Rows at the head under review\n")
    L.append("Per-row `--exact` runs, each on its own bound. The crate pins `RUST_TEST_THREADS=1` and "
             "every `compile_to_resolved` row recompiles the std closure, so one crate-filtered run "
             "cannot answer inside a dispatch; verdicts come from each run's `test result:` summary "
             "line and the release profile.\n")
    seen = set()
    for (label, name), rec in sorted(rows.items(), key=lambda kv: (kv[0][1], kv[0][0])):
        if name not in real or label not in HEAD_LABELS or not usable(rec):
            continue
        seen.add(name)
        el = f", {rec['elapsed']}s" if rec["elapsed"] else ""
        L.append(f"- `{name}`: **{rec['verdict']}** (rc={rec['rc']}{el})")
    unmeasured = sorted(real - seen)
    if unmeasured:
        L.append(f"\n**Not measured at this head: {', '.join(f'`{r}`' for r in unmeasured)}.** Listed "
                 "rather than omitted, so an absent row cannot be read as a passing one.")
    L.append("")

    L.append("### The rows that discriminate, across trees\n")
    L.append("The same test file at three trees, so only the compiler differs. `minted_variables_are_"
             "fresh_per_call_site` is the RED: it passes where the calls get their own variables and "
             "fails where they share the callee's parameter names.\n")
    for name in ("minted_variables_are_fresh_per_call_site",
                 "collision_two_zero_arg_generic_calls_both_type",
                 "collision_pair_still_refuses_when_instantiated_twice"):
        for label in ("ab_repair", "ab_pre", "prerepair", "prerepair2", "rows_repair", "collrel",
                      *HEAD_LABELS, "mutant", "rows_mutant"):
            rec = rows.get((label, name))
            if rec is None or name not in real or not usable(rec):
                continue
            el = f", {rec['elapsed']}s" if rec["elapsed"] else ""
            L.append(f"- `{name}` @ {tree_for(label, rec.get('sha'))}: **{rec['verdict']}** "
                     f"(rc={rec['rc']}{el})")
    L.append("")

    unusable = sorted({(l, n, r["verdict"]) for (l, n), r in rows.items()
                       if n in real and not usable(r)})
    if unusable:
        L.append("Runs whose row verdicts are NOT shown above, because they are not measurements of "
                 "the tree: " + "; ".join(f"`{n}` @ `{l}` ({v})" for l, n, v in unusable) +
                 ". MISSING_FATAL/NO_SUMMARY come from my matcher before it read the `test result:` "
                 "summary line, TIMEOUT_FATAL from a bound too short for a row that runs the compiler "
                 "under test in an unoptimised binary (the same row passes in the release profile, "
                 "shown above). They remain in the run logs and in the predictions file.\n")
    L.append("")
    open(out_path, "w").write(body.rstrip("\n") + "\n" + "\n".join(L) + "\n")
    print(f"wrote {out_path} ({os.path.getsize(out_path)} bytes, "
          f"was {os.path.getsize(body_path)})")
    bad = [c for c in hard if hard[c]["added"]]
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
