#!/bin/bash
# S0 EVIDENCE DISPATCH (run remotely: `ctrl-build --remote -- bash .s0evidence.sh`).
#
# One fresh build, then: the file digests, the NATIVE v2 parse census, the base claim-run, and ten
# one-line mutations. The base must be green or the run aborts -- a mutation row measured against a
# red base means nothing.
#
# WHY PYTHON THROUGH A QUOTED HEREDOC: the first version of this harness was pure bash and lost
# three of ten rows to quoting -- two needles never applied (they span lines), and one produced a
# file that failed to CONSTRUCT, which the harness reported as exit=1 PASS=0 FAIL=0. A mutation that
# does not build is NOT a red test. The Python below classifies every row RED / GREEN / BROKEN /
# NOT_APPLIED, so an invalid row can never be counted as a caught defect.
#
# WHY THE CENSUS: `gunbc run` and --claim-run both stay green on a file the v2 bootstrap gate
# refuses whole-file (deep-ram-338 hit this on S4, and I hit the same with a closure parameter named
# `module`). So the claim-run is not a parse proof, and CI's v2-bootstrap is the only thing that
# would have caught it. This prints it directly.
set -u
MODULE=dag/gunbc/firmware_network_boot_standing.dag
WITNESS=dag/test/claim/firmware_network_boot_standing_witness.dag
ENTRY=dag/test/claim/firmware_network_boot_standing_witness.dag

echo "### HEAD: $(git rev-parse HEAD)"
echo "### tracked-file digests (compare against git show <sha>:<path>)"
sha256sum "$MODULE" "$WITNESS"
git show "$(git rev-parse HEAD):$MODULE" | sha256sum
git show "$(git rev-parse HEAD):$WITNESS" | sha256sum

cargo build --release -p v1-compiler --bin gunbc >/tmp/build.log 2>&1 || {
  echo BUILD_FAIL; tail -30 /tmp/build.log; exit 1; }

echo "### NATIVE v2 PARSE CENSUS"
timeout 600 ./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/compiler/parse_acceptance_census.dag \
  --function parse_acceptance_census_for_paths \
  --arg paths_csv="$MODULE,$WITNESS" 2>&1 | grep -E "accepted|refused|unreadable" | head -5

cat > /tmp/s0mutate.py <<'PYEOF'
import subprocess

MODULE = "dag/gunbc/firmware_network_boot_standing.dag"
TYPES = "dag/extdeps/bmc/types.dag"
ENTRY = "dag/test/claim/firmware_network_boot_standing_witness.dag"
GUNBC = "./target/release/gunbc"

# (name, file, from, to) -- every needle must occur EXACTLY once in its file.
CASES = [
    # Ready names the boot-order HEAD. Two fixtures differ only in order; if the head is a constant
    # instead of what the pattern bound, the ordering test cannot tell them apart.
    ("M-HEAD present-arm head no longer comes from the list", MODULE,
     "Cons { head: first_target, tail: rest } =>\n      NetworkEntriesPresent { head: first_target, additional_count: length(xs: rest) }",
     "Cons { head: first_target, tail: rest } =>\n      NetworkEntriesPresent { head: BootTargetNone, additional_count: length(xs: rest) }"),

    # examined_count is the number of entries the classifier looked at; the fold guard below reads it
    # to tell 'four options, none network' (a conflict) from 'no options' (an unconfigured board).
    ("M-COUNT empty-arm examined_count is fabricated, not measured", MODULE,
     "Empty => NoNetworkEntry { examined_count: length(xs: entries) }",
     "Empty => NoNetworkEntry { examined_count: 0 }"),

    # additional_count: a field a reader credits but nothing pins is a fabricated output (DESIGN 5).
    ("M-EXTRA additional_count is a constant", MODULE,
     "NetworkEntriesPresent { head: first_target, additional_count: length(xs: rest) }",
     "NetworkEntriesPresent { head: first_target, additional_count: 0 }"),

    # Three NIC shapes, not two. Collapsing 'several' onto 'one' lets a Ready name one interface of
    # several -- which is the guess this module exists to prevent.
    ("M-SHAPE several-NIC collapses onto the one-NIC arm", MODULE,
     "      if length(xs: rest) > 0 {",
     "      if false {"),

    # The boot-target classifier's arm is the finding. Treating every target as network makes the
    # whole seam report Ready on a disk-only BootOrder.
    ("M-DECLASS every boot target is classified as network", TYPES,
     '    BootTargetNone => NotNetworkBootTarget { why: "None selects no device at all" }',
     '    BootTargetNone => NetworkBootTarget { via: "declass" }'),

    # The count/list disagreement this fold should not be able to reach (review 81893). Dropping the
    # guard must still refuse, not silently pick the non-conflict branch.
    ("M-FOLD count/list disagreement is absorbed instead of refused", MODULE,
     "          if listed_count > 0 {",
     "          if false {"),

    # A redirect's procedure stops naming the existing-option demand, so the admissibility filter
    # (std.algebra.contains over route_requires) can no longer drop a redirect credited to 'no
    # option exists' -- the pairing review 81760 forbids becomes reachable again.
    ("M-REQUIRES redirect no longer demands an existing option", MODULE,
     "      RequiresReset,\n      RequiresExistingBootOption,\n    ]\n    RedfishOneShotOverride { service_root: _, authority: _ } => [",
     "      RequiresReset,\n    ]\n    RedfishOneShotOverride { service_root: _, authority: _ } => ["),

    # 'exists but is not selected' and 'does not exist' are different remediations (review 81893);
    # collapsing them re-creates the one-name-covers-two-meanings defect.
    ("M-BLOCKER not-selected collapses onto absent", MODULE,
     "PointUefiBootNext { service_root: _, boot_option: _, authority: _ } =>\n      NetworkBootOptionNotSelected",
     "PointUefiBootNext { service_root: _, boot_option: _, authority: _ } =>\n      NetworkBootOptionAbsent"),

    # Legacy BIOS: the unit's firmware is fine, OUR edge serves UEFI only. If the arm becomes
    # serviceable, a PXE-BIOS unit reports Ready and someone drives over to netboot a board that
    # cannot. Whole-arm replacement: a partial needle here left a dangling `why:` and the row was
    # BROKEN (0 tests ran) -- which a naive harness would have counted as RED.
    ("M-EDGE legacy BIOS mode no longer refused by the edge", MODULE,
     '    LegacyBiosPxe { tftp_root: _ } =>\n      EdgeCannotServe {\n        why: "the site PXE edge of cut 6 serves UEFI only; a PXE BIOS unit has no boot target here",\n      }',
     "    LegacyBiosPxe { tftp_root: _ } => EdgeCanServe"),

    # Secure Boot ON means an unsigned snponly.efi will not boot, so it must be preparation, not
    # Ready. Re-pointing it at another blocker keeps the shape and loses the reason.
    ("M-SB Secure Boot ON no longer forces preparation", MODULE,
     "SecureBootEnabled => prepare(blocker: SecureBootBlocksUnsignedChainloader)",
     "SecureBootEnabled => prepare(blocker: NoInterfaceCarriesPxe)"),
]


def run_claim():
    p = subprocess.run([GUNBC, "run", "--claim-run", "--source-root", "dag",
                        "--source-root", "src/v2", "--entry", ENTRY],
                       capture_output=True, text=True, timeout=1200)
    out = p.stdout + p.stderr
    return (p.returncode,
            [l for l in out.splitlines() if l.startswith("PASS ")],
            [l for l in out.splitlines() if l.startswith("FAIL ")],
            out)


def report(rc, passed, failed, out):
    if not passed and not failed:
        print("  BROKEN -- no tests ran (exit %d); NOT counted as a caught defect" % rc)
        for line in out.splitlines():
            low = line.lower()
            if "error" in low or "resolve" in low:
                print("    >> %s" % line.strip()[:220])
        return "BROKEN"
    for line in failed:
        print("    RED: %s" % line.strip()[:170])
    print("  exit=%d PASS=%d FAIL=%d" % (rc, len(passed), len(failed)))
    return "RED" if failed else "GREEN"


rc, passed, failed, out = run_claim()
print("=== BASE (committed tree) ===")
for line in failed:
    print("    BASE-FAIL: %s" % line.strip()[:170])
testfn = sum(1 for l in open(ENTRY) if l.startswith("test fn "))
print("  exit=%d PASS=%d FAIL=%d TESTFN=%d (PASS+FAIL must equal TESTFN)"
      % (rc, len(passed), len(failed), testfn))
if failed:
    print("ABORT: base is not green; a mutation row against a red base means nothing.")
    raise SystemExit(1)

results = {}
for name, path, frm, to in CASES:
    original = open(path).read()
    n = original.count(frm)
    if n != 1:
        print("=== %s\n  NOT_APPLIED (needle occurs %d times)" % (name, n))
        results[name] = "NOT_APPLIED"
        continue
    print("=== %s" % name)
    try:
        open(path, "w").write(original.replace(frm, to))
        results[name] = report(*run_claim())
    finally:
        open(path, "w").write(original)

print("\n=== TALLY (every row must be RED to count as caught) ===")
for k, v in results.items():
    print("  %-11s %s" % (v, k))
notred = [k for k, v in results.items() if v != "RED"]
print("rows not RED: %d of %d" % (len(notred), len(results)))
for path in {c[1] for c in CASES}:
    if subprocess.run(["git", "diff", "--quiet", "--", path]).returncode != 0:
        print("WARNING: %s left MODIFIED" % path)
PYEOF

echo "### BASE + MUTATIONS"
python3 /tmp/s0mutate.py
