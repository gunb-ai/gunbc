#!/usr/bin/env bash
# One-off: fetch the head-$SHA ref (PR head ref pushed by the head-branch convention).
. ./stage_common.sh
HEAD="$1"
test -d "head-$HEAD" && { echo "TREE_PRESENT head-$HEAD"; git -C "head-$HEAD" rev-parse HEAD; exit 0; }
git ls-remote origin "refs/heads/head-$HEAD" | grep -q "$HEAD" || { echo "NO_HEAD_REF head-$HEAD"; exit 8; }
git clone --depth 1 -q -b "head-$HEAD" git@github.com:gunb-ai/gunbc.git "head-$HEAD" || { echo CLONE_FAIL; exit 8; }
test -d "head-$HEAD" && echo "TREE_PRESENT head-$HEAD" && git -C "head-$HEAD" rev-parse HEAD || { echo "TREE_MISSING head-$HEAD"; exit 8; }
