#!/usr/bin/env python3
"""Post-vs-pre hard-diagnostic comparison for #14152, per closure.

WHY THIS EXISTS WHEN THE GATE ALREADY COMPARES. gate_two_closures_14152.sh compares the head
against MAIN. That is the wrong baseline for the producer repair's own gate: main carries no Acc
at all, so main reports ZERO type-mismatch diagnostics in these closures and every head refusal
shows up as "NEW". The hard test the repair must pass is different -- measured against the
immediately preceding head (the Acc, before the producer repair):

    set(post) - set(pre) must be EMPTY       no new refusal may appear anywhere
    set(pre)  - set(post)  named by identity what the repair actually removes

Both directions are by IDENTITY (file, bound variable, head shape), the same key the shipped
comparator uses, imported rather than reimplemented so the two measurements cannot drift apart.
A count going down is not enough: 77 -> 43 is only meaningful if the 43 are a subset of the 77.

usage: compare_post_pre_14152.py POST_DIR PRE_DIR CLOSURE [out.json]
       where each directory holds <closure>.log produced by the same gunbc invocation form.
"""
import sys, os, json, collections

_HERE = os.path.dirname(os.path.abspath(__file__))
for _cand in (_HERE, os.path.join(_HERE, "instruments")):
    if os.path.isfile(os.path.join(_cand, "hard_diag_by_identity.py")):
        sys.path.insert(0, _cand)
        break
from hard_diag_by_identity import parse  # same LINE/BIND regexes as the gate's comparator


def main():
    if len(sys.argv) < 4:
        print(__doc__)
        return 2
    post_dir, pre_dir, target = sys.argv[1], sys.argv[2], sys.argv[3]
    post_path = os.path.join(post_dir, f"{target}.log")
    pre_path = os.path.join(pre_dir, f"{target}.log")
    for p in (post_path, pre_path):
        if not os.path.isfile(p) or os.path.getsize(p) == 0:
            # A missing log would parse to zero diagnostics, which reads as "the repair removed
            # everything" -- the most convincing possible false pass. Refuse instead.
            print(f"FATAL: {p} missing or empty -- zero would be a lie, not a measurement")
            return 3
    post, tp, up, sp = parse(post_path)
    pre, tq, uq, sq = parse(pre_path)

    print(f"closure: {target}")
    print(f"POST  {post_path}")
    print(f"  hard={tp} identities={len(post)} unparsed={up}")
    print(f"PRE   {pre_path}")
    print(f"  hard={tq} identities={len(pre)} unparsed={uq}")
    if up or uq:
        print(f"WARN: unparsed diagnostic lines (post={up} pre={uq}) -- those lines carry no "
              f"identity and are invisible to both directions below")

    added = sorted(k for k in post if k not in pre)
    removed = sorted(k for k in pre if k not in post)
    changed = sorted(k for k in post if k in pre and post[k] != pre[k])

    print(f"\n=== HARD TEST: identities present POST but not PRE (must be 0) : {len(added)}")
    for k in added:
        print(f"    NEW  {post[k]:>3}x  {k}")
    print(f"\n=== identities PRE had that POST lost (what the repair removes): {len(removed)}")
    for k in removed:
        print(f"    GONE {pre[k]:>3}x  {k}")
    print(f"\n=== same identity, different occurrence count: {len(changed)}")
    for k in changed:
        print(f"    DIFF pre={pre[k]:>3} post={post[k]:>3}  {k}")

    per = collections.defaultdict(lambda: [0, 0])
    for k, v in post.items():
        per[k[0]][0] += v
    for k, v in pre.items():
        per[k[0]][1] += v
    print("\n=== PER FILE (post | pre) ===")
    for f in sorted(per, key=lambda f: (-per[f][1], f)):
        a, b = per[f]
        print(f"   {f:<52} post={a:>3} pre={b:>3}")

    if len(sys.argv) > 4:
        json.dump({"closure": target,
                   "post_total": tp, "pre_total": tq,
                   "post_unparsed": up, "pre_unparsed": uq,
                   "added": [[list(k), post[k]] for k in added],
                   "removed": [[list(k), pre[k]] for k in removed],
                   "count_changed": [[list(k), pre[k], post[k]] for k in changed],
                   "post_identities": {repr(k): v for k, v in sorted(post.items())},
                   "pre_identities": {repr(k): v for k, v in sorted(pre.items())}},
                  open(sys.argv[4], "w"), indent=1)
        print(f"\nwrote {sys.argv[4]}")

    # Exit non-zero when the hard test fails, so a driver cannot ignore it.
    return 1 if added else 0


if __name__ == "__main__":
    sys.exit(main())
