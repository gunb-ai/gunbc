#!/bin/bash
# v14 seed-gate simulation: real depth-2 fetch of PIN_P from origin; run the packet's post-fetch gate lines.
set -u
PIN_A=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1
PIN_P=70d1c6ebb2e760def318bf254d0a7a1fde9b8dc0
TREE_P=c363a9495ff86273563d5d2c0e3d7450346fdb24
BLOB_PROBE=861944d809115363dbbd6a71c8020f1780df919c
BLOB_WC=aba6d2be15f1510629d7d8f8d47267e162bf2047
PROBE_PATH=src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag
WC_PATH=src/v2/test/claim/namespace_xl0/wildcard_arm_resolve.dag
BOUND=218
SCR=$(mktemp -d); trap 'rm -rf "$SCR"' EXIT
git init -q "$SCR/wt-A" 2>/dev/null; cd "$SCR/wt-A"
git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 2 --no-tags origin "$PIN_P" || { echo "GATE0 FAIL fetch"; exit 1; }
git checkout -q FETCH_HEAD || { echo "GATE0 FAIL checkout"; exit 1; }
H=$(git rev-parse HEAD); T=$(git rev-parse 'HEAD^{tree}'); P1=$(git rev-parse 'HEAD^1' 2>/dev/null || echo MISSING)
D=$(git diff --name-only "$PIN_A" HEAD 2>/dev/null | tr '\n' ',')
[ "$H" = "$PIN_P" ] && echo "GATE1 head ok $H" || { echo "GATE1 FAIL $H"; exit 1; }
[ "$P1" = "$PIN_A" ] && echo "GATE2 parent ok $P1" || { echo "GATE2 FAIL $P1"; exit 1; }
[ "$T" = "$TREE_P" ] && echo "GATE3 tree ok $T" || { echo "GATE3 FAIL $T"; exit 1; }
[ "$D" = "$PROBE_PATH," ] && echo "GATE4 delta ok (one path)" || { echo "GATE4 FAIL delta=$D"; exit 1; }
BP=$(git rev-parse "HEAD:$PROBE_PATH" 2>/dev/null || echo MISSING)
[ "$BP" = "$BLOB_PROBE" ] && echo "GATE5 probe blob ok $BP" || { echo "GATE5 FAIL $BP"; exit 1; }
BW=$(git rev-parse "HEAD:$WC_PATH" 2>/dev/null || echo MISSING)
[ "$BW" = "$BLOB_WC" ] && echo "GATE6 wc blob ok $BW" || { echo "GATE6 FAIL $BW"; exit 1; }
RB=$(sed -n 's/.*data reason_bound: Int = \([0-9]\+\).*/\1/p' "$PROBE_PATH" | head -1)
[ "$RB" = "$BOUND" ] && echo "GATE9 reason_bound ok $RB" || { echo "GATE9 FAIL $RB"; exit 1; }
[ -z "$(git status --porcelain)" ] && echo "GATE7 porcelain ok" || { echo "GATE7 FAIL dirty"; exit 1; }
echo "leg A probe pin ok head=$H tree=$T parent=$P1 delta=1 blob=$BP wc=$BW bound=$RB"
