#!/usr/bin/env bash
# CHAIN CONTROL: the complete change from the field-proven v17 frame bytes to THIS frame is the two published diffs
# applied in order.  It is posted as a chain rather than as one diff because a single v17-to-v6 diff is 185 KB and
# cannot fit one comment; a chain is stronger evidence, because each step is checked against a published sha.
# Nothing here builds or runs gunbc: it is bytes, patches and hashes.
set -u
cd "$(dirname "$0")/.."
V17=v17-frame-published-AUTHORITATIVE.sh
V17_SHA=f8412fdf742a6a4f3a47348f341f4fdd3852560a7cc0017e636ec0d8bb61a86e
V5_SHA=$(sha256sum v5-kept/packet-evidence-freeze2.sh | cut -d' ' -f1)
V6=$(sha256sum packet-evidence-freeze2.sh | cut -d' ' -f1)
V6_BYTES=$(wc -c < packet-evidence-freeze2.sh)
V6_LINES=$(wc -l < packet-evidence-freeze2.sh)
OK=0; BAD=0
ok(){ echo "  OK   $1"; OK=$((OK+1)); }
bad(){ echo "  BAD  $1"; BAD=$((BAD+1)); }
[ "$(sha256sum "$V17" | cut -d' ' -f1)" = "$V17_SHA" ] && ok "the v17 bytes on disk are the published v17 frame ($V17_SHA)" || bad "v17 bytes do not match their published sha"
W=$(mktemp -d /var/tmp/chain-v6.XXXXXX); chmod 700 "$W"; trap 'rm -rf "$W"' EXIT
cp "$V17" "$W/frame.sh"
if patch -s -T --no-backup-if-mismatch -p0 "$W/frame.sh" < packet-evidence-v17-vs-v5.diff; then
  got=$(sha256sum "$W/frame.sh" | cut -d' ' -f1)
  [ "$got" = "$V5_SHA" ] && ok "v17 bytes + packet-evidence-v17-vs-v5.diff == the v5 frame ($V5_SHA)" || bad "step 1 gave $got, want the v5 frame sha"
else
  bad "step 1: the published v17-to-v5 diff did not apply cleanly"
fi
if patch -s -T --no-backup-if-mismatch -p0 "$W/frame.sh" < packet-evidence-v5-vs-v6.diff; then
  got=$(sha256sum "$W/frame.sh" | cut -d' ' -f1)
  [ "$got" = "$V6" ] && ok "… + packet-evidence-v5-vs-v6.diff == THE FRAME UNDER TEST ($V6, $V6_BYTES B, $V6_LINES lines)" || bad "step 2 gave $got, want $V6"
  bash -n "$W/frame.sh" && ok "the reconstructed frame passes bash -n" || bad "the reconstructed frame does not parse"
else
  bad "step 2: the v5-to-v6 diff did not apply cleanly"
fi
# the inversion: applying the v5-to-v6 diff to the WRONG base must fail, not silently produce something else
cp v5-kept/packet-evidence-freeze2.sh "$W/wrong.sh"
if patch -s -T -p0 --dry-run "$W/wrong.sh" < packet-evidence-v17-vs-v5.diff 2>/dev/null; then bad "INVERSION: the v17-to-v5 diff applied to v5 bytes"; else ok "INVERSION: the v17-to-v5 diff refuses the v5 bytes"; fi
echo "CHAIN OK=$OK BAD=$BAD"
[ "$BAD" = 0 ] || exit 1
exit 0
