#!/usr/bin/env bash
# CHAIN CONTROL (v7): the complete change from the field-proven v17 frame bytes to THIS frame is the THREE
# published diffs applied in order. It is posted as a chain rather than as one diff because a single v17-to-v7
# diff is far too large for one comment; a chain is stronger evidence, because every intermediate step is checked
# against a published sha, and each step is shown to REFUSE a base that is not its own.
# Nothing here builds or runs gunbc: it is bytes, patches and hashes.
set -u
cd "$(dirname "$0")/.."
V17=v17-frame-published-AUTHORITATIVE.sh
V17_SHA=f8412fdf742a6a4f3a47348f341f4fdd3852560a7cc0017e636ec0d8bb61a86e
V5=$(sha256sum v5-kept/packet-evidence-freeze2.sh | cut -d' ' -f1)
V6=$(sha256sum v6-kept/packet-evidence-freeze2.sh | cut -d' ' -f1)
HEAD=$(sha256sum packet-evidence-freeze2.sh | cut -d' ' -f1)
HEAD_BYTES=$(wc -c < packet-evidence-freeze2.sh)
HEAD_LINES=$(wc -l < packet-evidence-freeze2.sh)
OK=0; BAD=0
ok(){ echo "  OK   $1"; OK=$((OK+1)); }
bad(){ echo "  BAD  $1"; BAD=$((BAD+1)); }
[ "$(sha256sum "$V17" | cut -d' ' -f1)" = "$V17_SHA" ] && ok "the v17 bytes on disk are the published v17 frame ($V17_SHA)" || bad "v17 bytes do not match their published sha"
W=$(mktemp -d /var/tmp/chain-v7.XXXXXX); chmod 700 "$W"; trap 'rm -rf "$W"' EXIT
cp "$V17" "$W/frame.sh"
if patch -s -T --no-backup-if-mismatch -p0 "$W/frame.sh" < packet-evidence-v17-vs-v5.diff; then
  got=$(sha256sum "$W/frame.sh" | cut -d' ' -f1)
  [ "$got" = "$V5" ] && ok "v17 bytes + packet-evidence-v17-vs-v5.diff == the v5 frame ($V5)" || bad "step 1 gave $got, want the v5 frame sha"
else
  bad "step 1: the published v17-to-v5 diff did not apply cleanly"
fi
if patch -s -T --no-backup-if-mismatch -p0 "$W/frame.sh" < packet-evidence-v5-vs-v6.diff; then
  got=$(sha256sum "$W/frame.sh" | cut -d' ' -f1)
  [ "$got" = "$V6" ] && ok "… + packet-evidence-v5-vs-v6.diff == the v6 frame ($V6)" || bad "step 2 gave $got, want the v6 frame sha"
else
  bad "step 2: the v5-to-v6 diff did not apply cleanly"
fi
if patch -s -T --no-backup-if-mismatch -p0 "$W/frame.sh" < packet-evidence-v6-vs-v7.diff; then
  got=$(sha256sum "$W/frame.sh" | cut -d' ' -f1)
  [ "$got" = "$HEAD" ] && ok "… + packet-evidence-v6-vs-v7.diff == THE FRAME UNDER TEST ($HEAD, $HEAD_BYTES B, $HEAD_LINES lines)" || bad "step 3 gave $got, want $HEAD"
  bash -n "$W/frame.sh" && ok "the reconstructed frame passes bash -n" || bad "the reconstructed frame does not parse"
  cmp -s "$W/frame.sh" packet-evidence-freeze2.sh && ok "the reconstructed frame is byte-identical to the frame under test (cmp)" || bad "the reconstructed frame is not byte-identical"
else
  bad "step 3: the v6-to-v7 diff did not apply cleanly"
fi
# INVERSIONS: each step must REFUSE a base that is not its own, and a step must not be appliable twice.
cp v5-kept/packet-evidence-freeze2.sh "$W/w1.sh"
patch -s -T -p0 --dry-run "$W/w1.sh" < packet-evidence-v17-vs-v5.diff 2>/dev/null && bad "INVERSION: the v17-to-v5 diff applied to v5 bytes" || ok "INVERSION: the v17-to-v5 diff refuses the v5 bytes"
cp v6-kept/packet-evidence-freeze2.sh "$W/w2.sh"
patch -s -T -p0 --dry-run "$W/w2.sh" < packet-evidence-v5-vs-v6.diff 2>/dev/null && bad "INVERSION: the v5-to-v6 diff applied to v6 bytes (a step twice)" || ok "INVERSION: the v5-to-v6 diff refuses the v6 bytes"
cp v6-kept/packet-evidence-freeze2.sh "$W/w3.sh"
patch -s -T -p0 --dry-run "$W/w3.sh" < packet-evidence-v6-vs-v7.diff >/dev/null 2>&1; rc=$?
[ "$rc" = 0 ] && ok "the v6-to-v7 diff applies to the v6 frame (its own base)" || bad "the v6-to-v7 diff does not apply to its own base (rc=$rc)"
cp v5-kept/packet-evidence-freeze2.sh "$W/w4.sh"
patch -s -T -p0 --dry-run "$W/w4.sh" < packet-evidence-v6-vs-v7.diff 2>/dev/null && bad "INVERSION: the v6-to-v7 diff applied to v5 bytes (out of order)" || ok "INVERSION: the v6-to-v7 diff refuses the v5 bytes (order is enforced by the shas)"
# ORDER, not identity, is what a chain has to protect: a later step applied to the v17 bytes must be refused.
cp "$V17" "$W/w5.sh"
patch -s -T -p0 --dry-run "$W/w5.sh" < packet-evidence-v5-vs-v6.diff 2>/dev/null && bad "INVERSION: the v5-to-v6 diff applied to the v17 bytes (a step out of order)" || ok "INVERSION: the v5-to-v6 diff refuses the v17 bytes (order is enforced)"
cp "$V17" "$W/w6.sh"
patch -s -T -p0 --dry-run "$W/w6.sh" < packet-evidence-v6-vs-v7.diff 2>/dev/null && bad "INVERSION: the v6-to-v7 diff applied to the v17 bytes (a step out of order)" || ok "INVERSION: the v6-to-v7 diff refuses the v17 bytes (order is enforced)"
echo "CHAIN OK=$OK BAD=$BAD  (chain: v17 $V17_SHA -> v5 -> v6 -> head $HEAD)"
[ "$BAD" = 0 ] || exit 1
exit 0
