#!/usr/bin/env bash
# STAGE 0: fetch the cutover main tree (4036f188602) so the FLOOR module and the
# floor-side legs exist on srv1. Plain git — no slot; needs the floor module present.
. ./stage_common.sh
MB=4036f188602
test -d "main-$MB" && { echo "MAIN_TREE_PRESENT main-$MB"; git -C "main-$MB" rev-parse --short HEAD; ls "main-$MB/src/v2/test/claim/filesystem_effect_reach_wall.dag" && echo FLOOR_MODULE_PRESENT && exit 0; }
git ls-remote origin refs/heads/main | grep -q . || { echo REMOTE_FAIL; exit 8; }
git clone --depth 1 -q -b main git@github.com:gunb-ai/gunbc.git "main-$MB" || { echo CLONE_FAIL; exit 8; }
git -C "main-$MB" fetch --depth 1 origin "$MB" -q && git -C "main-$MB" checkout -q FETCH_HEAD || echo "CHECKOUT_PIN_FAILED staying at tip"
git -C "main-$MB" rev-parse --short HEAD
test -f "main-$MB/src/v2/test/claim/filesystem_effect_reach_wall.dag" && echo FLOOR_MODULE_PRESENT || { echo FLOOR_MODULE_MISSING; exit 8; }
