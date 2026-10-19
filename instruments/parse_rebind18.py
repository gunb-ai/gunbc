#!/usr/bin/env python3
"""Probe v18 analyzer: WHICH call arm handed the fold the bound node.

Pre-registered BEFORE the run (stated here so the outcome cannot be chosen afterwards). The tag is
written by the call arm immediately before the fold entry; BC/RT are written only when type_match is
None, and TM is derived from type_match's own value, so the three are mutually exclusive per call.

  D1  TM dominates the Map family AND the 39 producer-1 rows carry the algebra.dag span
      => my static read's successor is confirmed: binding.resolved at 9657 is the producer.
         The repair belongs there, gated on the callee declaring type parameters.
  D2  BC dominates AND producer-1 rows carry the algebra.dag span
      => my FIRST read was right after all, and the inert repair failed for a different reason
         (the seeded variables were consumed before substitute_generics) -- re-open the patch,
         do not move the repair.
  D3  RT dominates
      => callable_return_type (04_infer.dag:9652) is the producer; that arm requires params==0,
         which would itself be a finding about how empty_map's gnode is shaped.
  D4  'unset' dominates the family
      => the family is NOT produced by this call arm at all; the probe did not cover the path.
         Report the unexplained rows; do NOT pick a branch to patch.
  D5  the tag field is missing/empty on any family row, or |tagseq-foldseq| is large on many rows
      => the probe did not measure (stale or uninitialised tags); fix the probe, report nothing.

Also reports the family split by bound-node span (producer 1 = dag/std/algebra.dag:73644-73647,
producer 2 = <synthetic>:0-0), because that split was measured before the repair existed.
"""
import re, sys, collections, pathlib

path = pathlib.Path(sys.argv[1])
rows = []
caps = 0
for l in path.read_text(errors="replace").splitlines():
    if l.startswith("REBINDV4|CAP"):
        caps += 1
        continue
    if not l.startswith("REBIND18|"):
        continue
    d = {}
    for part in l[len("REBIND18|"):].split("|"):
        if "=" in part:
            k, v = part.split("=", 1)
            d[k] = v
    rows.append(d)

assert rows, "no REBIND18 rows -- the probe did not run; report nothing"

def leaves(s):
    return [x for x in re.split(r",(?![^()]*\))", s or "") if x]

P1 = "dag/std/algebra.dag:73644-73647"
P2 = "<synthetic>:0-0"

fam = [d for d in rows if d.get("bhead") == "Map"
       and leaves(d.get("bleaves")) and set(leaves(d.get("bleaves"))) <= {"K", "V"}]

# GUARD (the rule this lane learned when a literal-paren regex returned '' for every field and read
# as a measured zero): every field the classification keys on must be PRESENT and non-empty.
missing = [d for d in fam if not str(d.get("tag", "")).strip() or not str(d.get("bspan", "")).strip()]
assert not missing, "%d family rows lack tag or bspan -- the probe did NOT measure; do not conclude" % len(missing)
unknown_tag = [d for d in fam if d["tag"] not in ("TM", "BC", "RT", "unset")]
assert not unknown_tag, "unexpected tag values: %r" % {d["tag"] for d in unknown_tag}

print("rows=%d  caps=%d  Map-family=%d" % (len(rows), caps, len(fam)))
print("\nALL rows by tag:            %s" % collections.Counter(d["tag"] for d in rows).most_common())
print("FAMILY rows by tag:         %s" % collections.Counter(d["tag"] for d in fam).most_common())
print("FAMILY by (tag, bound span):")
for (t, sp), n in collections.Counter((d["tag"], d["bspan"]) for d in fam).most_common(10):
    print("   %-6s %-40s %3d" % (t, sp, n))
p1 = [d for d in fam if d["bspan"] == P1]
p2 = [d for d in fam if d["bspan"] == P2]
other = [d for d in fam if d["bspan"] not in (P1, P2)]
print("\nproducer 1 (algebra.dag span) = %d   producer 2 (<synthetic>) = %d   other = %d"
      % (len(p1), len(p2), len(other)))
if other:
    print("   other spans (NAME them, do not drop them): %s"
          % collections.Counter(d["bspan"] for d in other).most_common(6))
print("producer 1 by tag: %s" % collections.Counter(d["tag"] for d in p1).most_common())
print("producer 2 by tag: %s" % collections.Counter(d["tag"] for d in p2).most_common())

# staleness of the tag: how far the tag write was from the fold print that read it
def delta(d):
    try:
        return abs(int(d.get("tagseq", "")) - int(d.get("foldseq", "")))
    except ValueError:
        return -1
ds = sorted(delta(d) for d in fam)
if ds:
    print("\n|tagseq-foldseq| over the family: min=%s median=%s max=%s"
          % (ds[0], ds[len(ds) // 2], ds[-1]))
    print("   rows with delta>400 (stale-tag suspects): %d" % sum(1 for x in ds if x > 400))

tags = collections.Counter(d["tag"] for d in fam)
top = (tags.most_common(1) or [("none", 0)])[0]
print("\n== OUTCOME (pre-registered) ==")
out = []
if not fam:
    out.append("D5: no family rows at all -- the probe did not measure the family.")
elif top[0] == "unset" and top[1] >= len(fam) // 2:
    out.append("D4: 'unset' dominates -- the family does not come through this call arm. "
               "Report the rows; do NOT pick a branch to patch.")
elif top[0] == "TM":
    out.append("D1: TM dominates -- binding.resolved (the type_match Present arm) is the producer. "
               "Repair there, gated on the callee declaring type parameters.")
elif top[0] == "BC":
    out.append("D2: BC dominates -- borrowed_callable_call_type after all; the inert repair failed for "
               "another reason. Re-open the patch, do not move the repair.")
elif top[0] == "RT":
    out.append("D3: RT dominates -- callable_return_type is the producer; note that arm requires "
               "params==0, itself a finding about the gnode's shape.")
if ds and sum(1 for x in ds if x > 400) > len(fam) // 3:
    out.append("D5: most family rows have a stale tag -- the probe did not measure.")
for line in out:
    print("  " + line)
assert len(out) == 1 or out[0].startswith("D5"), "expected exactly one outcome, got %d" % len(out)
print("\nSELECTED: " + out[0])
