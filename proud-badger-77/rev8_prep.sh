#!/usr/bin/env bash
# kh_nal_16061_prep.sh -- PREP for the #16061 measurement. Networked, no build, no compiler run, NEVER installs a toolchain.
# Runs inside scope kh-nal-16061-prep (MemoryMax=4G, MemorySwapMax=0, CPUQuota=200%, TasksMax=512, RuntimeMaxSec=1200).
# Input: P (run root). Stage dir $P.stage holds the measurement script (scp'd by the executor). Ends with PREP_OK or exits non-zero.
set -euo pipefail
: "${P:?P unset}"
HEAD_FULL=021bbda11cfb0a15dcb200151a0389293eb9dc9d
SCRIPT_SHA=295822d96bade4a3f7da2cc6f4004cdcc0c384abf2697ccd7e43ce34e8d3cbe1
[ ! -e "$P" ] || { echo "PREP REFUSED: $P exists"; exit 2; }
[ -f "$P.stage/nominal_child_edge_table.sh" ] || { echo "PREP REFUSED: staged script missing"; exit 2; }
avail=$(df -B1G --output=avail "$(dirname "$P")" | tail -1 | tr -d ' ')
[ "$avail" -ge 60 ] || { echo "PREP REFUSED: ${avail}G free < 60G"; exit 2; }
mkdir -p "$P"
timeout -k 10 600 git clone --no-checkout https://github.com/gunb-ai/gunbc.git "$P/src"
timeout -k 10 300 git -C "$P/src" fetch origin "$HEAD_FULL"
timeout -k 10 300 git -C "$P/src" checkout --detach "$HEAD_FULL"
[ "$(git -C "$P/src" rev-parse HEAD)" = "$HEAD_FULL" ] || { echo "PREP REFUSED: HEAD mismatch"; exit 2; }
timeout -k 10 300 env CARGO_HOME="$P/cargo-home" cargo fetch --locked --manifest-path "$P/src/Cargo.toml"
ch=$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$P/src/rust-toolchain.toml")
[ -n "$ch" ] || { echo "PREP REFUSED: no pinned channel"; exit 2; }
want="$ch-$(uname -m)-unknown-linux-gnu"
list=$(RUSTUP_AUTO_INSTALL=0 timeout -k 5 30 rustup toolchain list)
printf '%s\n' "$list" | awk '{print $1}' | grep -xF "$want" >/dev/null || { echo "PREP REFUSED: toolchain $want not installed (never installed here)"; printf '%s\n' "$list"; exit 2; }
mv "$P.stage/nominal_child_edge_table.sh" "$P/nominal_child_edge_table.sh"
echo "$SCRIPT_SHA  $P/nominal_child_edge_table.sh" | sha256sum -c
du -sh "$P"
echo PREP_OK
