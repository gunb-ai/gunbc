#!/bin/bash
# S0 measurement: base claim-run + one-line mutations, ONE build, ONE dispatch.
# Every mutation is a one-line change to the module (or witness); needles are verified to occur
# exactly once, or the row reports NOT_APPLIED rather than quietly passing.
set -u
BASE=dag/gunbc/firmware_network_boot_standing.dag
ENTRY=dag/test/claim/firmware_network_boot_standing_witness.dag
cp "$BASE" /tmp/S0.orig

cargo build --release -p v1-compiler --bin gunbc >/tmp/build.log 2>&1 || { echo BUILD_FAIL; tail -30 /tmp/build.log; exit 1; }

run () {
  timeout 900 ./target/release/gunbc run --claim-run --source-root dag --source-root src/v2 \
    --entry "$ENTRY" > /tmp/run.txt 2>&1
  local code=$?
  local p f
  p=$(grep -c '^PASS ' /tmp/run.txt); f=$(grep -c '^FAIL ' /tmp/run.txt)
  echo "exit=$code PASS=$p FAIL=$f"
  grep '^FAIL ' /tmp/run.txt | sed 's/^/    RED: /'
}

echo "=== BASE (committed tree) ==="; run
echo "TESTFN=$(grep -c '^test fn ' $ENTRY)"

mutate () {
  local name="$1" file="$2" from="$3" to="$4"
  cp "$BASE" /tmp/S0.orig
  cp "$file" /tmp/TARGET.orig
  python3 - "$file" "$from" "$to" <<'PY'
import sys
p, a, b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
n = s.count(a)
if n != 1:
    print("NEEDLE_COUNT=%d" % n); sys.exit(3)
open(p, "w").write(s.replace(a, b))
PY
  if [ $? -ne 0 ]; then echo "$name: NEEDLE NOT UNIQUE -- SKIPPED"; cp /tmp/TARGET.orig "$file"; cp /tmp/S0.orig "$BASE"; return; fi
  echo "=== $name ==="; run
  cp /tmp/TARGET.orig "$file"
  cp /tmp/S0.orig "$BASE"
}

# M-HEAD: the head bound by the pattern is replaced by a fixed target. The Ready must name the
# boot-order head (the_ready_names_the_boot_order_head test pins both orders).
mutate "M-HEAD present-arm head no longer comes from the list" "$BASE" \
  'Cons { head: first_target, tail: rest } =>
      NetworkEntriesPresent { head: first_target, additional_count: length(xs: rest) }' \
  'Cons { head: first_target, tail: rest } =>
      NetworkEntriesPresent { head: BootTargetNone, additional_count: length(xs: rest) }'

# M-COUNT: the examined count stops being the number of entries looked at.
mutate "M-COUNT empty-arm examined_count is fabricated, not measured" "$BASE" \
  'Empty => NoNetworkEntry { examined_count: length(xs: entries) }' \
  'Empty => NoNetworkEntry { examined_count: 0 }'

# M-EXTRA: the tail stops being counted, so 'exactly one' and 'several' collapse.
mutate "M-EXTRA additional_count is a constant" "$BASE" \
  'NetworkEntriesPresent { head: first_target, additional_count: length(xs: rest) }' \
  'NetworkEntriesPresent { head: first_target, additional_count: 0 }'

# M-SHAPE: the NIC list is never asked which shape it has; several collapses onto one.
mutate "M-SHAPE several-NIC collapses onto the one-NIC arm" "$BASE" \
  '      if length(xs: rest) > 0 {' \
  '      if false {'

# M-DECLASS: the target classifier stops distinguishing kinds by arm.
mutate "M-DECLASS every boot target is classified as network" dag/extdeps/bmc/types.dag \
  '    BootTargetNone => NotNetworkBootTarget { why: "None selects no device at all" }' \
  '    BootTargetNone => NetworkBootTarget { via: "declass" }'

# M-FOLD: the enumeration-contradiction refusal falls back to a guess.
mutate "M-FOLD count/list disagreement is absorbed instead of refused" "$BASE" \
  'Absent => Refused {
              cause: BootOrderNamesUnenumeratedNetworkOption {' \
  'Absent => needs(blocker: NetworkBootOptionAbsent)'

# M-REQUIRES: a redirect's procedure stops demanding an existing option, so the admissibility
# filter can no longer drop a redirect credited to 'no option exists'.
mutate "M-REQUIRES redirect no longer demands an existing option" "$BASE" \
  'PointUefiBootNext { service_root: _, boot_option: _, authority: _ } =>
      [PointAtOption' \
  'PointUefiBootNext { service_root: _, boot_option: _, authority: _ } =>
      ['

# M-BLOCKER: 'exists but not selected' collapses back onto 'absent' -- the two-name split undone.
mutate "M-BLOCKER not-selected collapses onto absent" "$BASE" \
  'PointUefiBootNext { service_root: _, boot_option: _, authority: _ } =>
      NetworkBootOptionNotSelected' \
  'PointUefiBootNext { service_root: _, boot_option: _, authority: _ } =>
      NetworkBootOptionAbsent'

# M-EDGE: legacy BIOS mode becomes serviceable at the edge.
mutate "M-EDGE legacy BIOS mode no longer refused" "$BASE" \
  '    LegacyBiosPxe { tftp_root: _ } =>
      EdgeCannotServe {' \
  '    LegacyBiosPxe { tftp_root: _ } => EdgeCanServe'

# M-SB: Secure Boot ON stops being preparation (the unsigned chainloader would read Ready).
mutate "M-SB Secure Boot ON no longer forces preparation" "$BASE" \
  'SecureBootEnabled => prepare(blocker: SecureBootBlocksUnsignedChainloader)' \
  'SecureBootEnabled => prepare(blocker: NoInterfaceCarriesPxe)'

cp /tmp/S0.orig "$BASE"
echo "=== TREE RESTORED ==="
git diff --stat -- "$BASE" | tail -2
