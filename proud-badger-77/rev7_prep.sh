#!/usr/bin/env bash
# kh_nal_16061_prep.sh -- PREP for the #16061 measurement. Networked, no build, no compiler run, NEVER installs a toolchain.
# Runs inside scope kh-nal-16061-prep (MemoryMax=4G, MemorySwapMax=0, CPUQuota=200%, TasksMax=512, RuntimeMaxSec=1200).
# Input: P (run root). Stage dir $P.stage holds the measurement script (scp'd by the executor). Ends with PREP_OK or exits non-zero.
set -euo pipefail
: "${P:?P unset}"
HEAD_FULL=5b02c6dd3d963cb9863b163a69cba56e900f8698
SCRIPT_SHA=636c6f739597e110a10f485742bd692ca5ce1b9577e054b028bf6cd233fb11cf
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
