#!/usr/bin/env python3
"""Per-module hard-diagnostic counts over an emitted closure, compared with main BY IDENTITY.

Why this exists (stern-bat-401, msg_79410475): main's unify_generics returns a bare subst map and takes
first-wins on a CONFLICTING rebind with no diagnostic at all, so main can report 0 hard diagnostics for this
class not because the class is absent but because nothing can report it. This PR changes the fold to
UnifyGenericsAcc { subst, diagnostics }, which is the first thing that makes those conflicts observable --
and the CI floor treats a hard diagnostic as structural red. So the number that gates the push is NOT the
census row count (which says nothing about a module the census never emitted, e.g. source_root_eval); it is
the per-module hard-diagnostic count, diffed against main by identity, so every module whose refusals the
Acc surfaces is LISTED and DISPOSITIONED.

Identity, deliberately: (file, bind-name, bound-side head constructor, first leaf). Not the whole printed
tree -- those run to 700+ characters, and an identity that includes the whole tree changes when an unrelated
type anywhere inside it changes, which would report the same refusal as new on every commit.

Both closures matter: CI emits //gunbc/instruments:self-host and //gunbc/instruments:v2-native-cli. A job
that dies in the first step never runs the second, so the second closure has to be measured locally --
reading only the job log would report a green-looking empty set for it.
"""
import re, sys, collections, json

LINE = re.compile(r"type mismatch: expected '(?P<exp>.*)', got '(?P<got>.*)' \((?P<loc>[^)]+)\)")
BIND = re.compile(r"^type variable (\w+) bound as (\w+)\(([^,(]*)")


KERNEL = re.compile(r"<[^>]+>")


def loc_of(loc):
    """Cite a diagnostic site without cutting a kernel citation in half.

    A `.dag` path is `path.dag:LINE-COL`, so cutting at the first colon is right. A kernel citation
    is `<kernel:Map>:0-4`, and cutting at the first colon leaves `<kernel` -- every kernel site would
    collapse onto one file name, silently merging refusals from different kernel types. Measured on
    the current head: no identity changes today (there is only one kernel site), so this is not fixing
    a wrong number -- it is removing a way for a future number to be wrong.
    """
    loc = loc.strip()
    if loc.startswith("<"):
        m = KERNEL.match(loc)
        if m:
            return m.group(0)
    return loc.split(":")[0].strip()


def parse(path):
    counts = collections.Counter()
    shapes = {}
    total = 0
    unparsed = 0
    for line in open(path, errors="replace"):
        if "type mismatch" not in line:
            continue
        m = LINE.search(line)
        if not m:
            unparsed += 1
            continue
        total += 1
        loc = loc_of(m.group("loc"))
        exp = m.group("exp")
        b = BIND.match(exp)
        if b:
            key = (loc, b.group(1), b.group(2), b.group(3)[:24])
        else:
            key = (loc, "?", "?", exp[:24])
        counts[key] += 1
        shapes.setdefault(key, (exp[:110], m.group("got")[:110]))
    return counts, total, unparsed, shapes


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        print("usage: hard_diag_by_identity.py HEAD.log MAIN.log [out.json]")
        return 2
    h, th, hu, hs = parse(sys.argv[1])
    m, tm, mu, _ = parse(sys.argv[2])
    print("HEAD closure : %3d hard diagnostics, %2d distinct identities, %d unparsed" % (th, len(h), hu))
    print("MAIN closure : %3d hard diagnostics, %2d distinct identities, %d unparsed" % (tm, len(m), mu))
    if hu or mu:
        # A parse failure here would silently under-count, and under-counting is exactly how a red becomes a
        # green-looking zero. Refuse to be trusted while lines are unexplained.
        print("WARNING: unparsed lines exist -- the counts above are not complete. Fix LINE before using this.")
    print("\n=== PER MODULE (head), with main's count for the same file ===")
    per_h, per_m = collections.Counter(), collections.Counter()
    for k, v in h.items():
        per_h[k[0]] += v
    for k, v in m.items():
        per_m[k[0]] += v
    for f in sorted(set(per_h) | set(per_m), key=lambda x: -per_h.get(x, 0)):
        print("   %-56s head=%3d  main=%3d  %s" % (
            f[:56], per_h.get(f, 0), per_m.get(f, 0),
            "NEW" if per_m.get(f, 0) == 0 and per_h.get(f, 0) else ("REMOVED" if per_h.get(f, 0) == 0 else "both")))
    new = set(h) - set(m)
    gone = set(m) - set(h)
    print("\n=== %d identities at HEAD that MAIN does not have (what the Acc surfaces) ===" % len(new))
    for k in sorted(new, key=lambda k: -h[k]):
        print("   %3d  %-34s bind=%-3s %s(%s)" % (h[k], k[0][:34], k[1], k[2], k[3]))
    print("\n=== %d identities MAIN has that HEAD lost (a diagnostic this PR would be deleting) ===" % len(gone))
    for k in sorted(gone, key=lambda k: -m[k])[:10]:
        print("   %3d  %-34s bind=%-3s %s(%s)" % (m[k], k[0][:34], k[1], k[2], k[3]))
    if len(sys.argv) > 3:
        json.dump({"head": {repr(k): v for k, v in h.items()},
                   "main": {repr(k): v for k, v in m.items()}}, open(sys.argv[3], "w"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
