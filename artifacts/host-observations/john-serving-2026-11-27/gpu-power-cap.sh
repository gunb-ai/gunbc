#!/bin/bash
# john: hold every NVIDIA GPU at its own minimum power limit (operator 2026-11-27: smooth the work, avoid spikes).
# Reads each card's min_limit so swapped cards are capped without editing this file; reads back and fails loudly on mismatch.
set -u
nvidia-smi -pm 1 >/dev/null || { echo "persistence mode failed"; exit 1; }
rc=0
while IFS=', ' read -r idx bus minw; do
  nvidia-smi -i "$idx" -pl "$minw" >/dev/null || { echo "gpu$idx $bus: set -pl $minw failed"; rc=1; continue; }
  got=$(nvidia-smi -i "$idx" --query-gpu=power.limit --format=csv,noheader,nounits | tr -d ' ')
  if [ "${got%.*}" != "${minw%.*}" ]; then echo "gpu$idx $bus: readback $got != $minw"; rc=1; else echo "gpu$idx $bus: power limit $got W (card minimum)"; fi
done < <(nvidia-smi --query-gpu=index,pci.bus_id,power.min_limit --format=csv,noheader,nounits)
exit $rc
