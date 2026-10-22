#!/bin/bash
HEAD_W=e86cbfeafae
for i in $(seq 1 100); do
  OUT=$(gh pr checks 14715 2>&1 | grep -vE 'skipping')
  NOW=$(gh pr view 14715 --json headRefOid -q .headRefOid | cut -c1-11)
  if [ "$NOW" != "$HEAD_W" ]; then echo "HEAD-MOVED $NOW (was $HEAD_W)"; echo "$OUT"; exit 2; fi
  if ! echo "$OUT" | grep -q 'pending\|queued'; then echo "ALL-RESOLVED"; echo "$OUT"; exit 0; fi
  sleep 30
done
echo "TIMEOUT-STILL-PENDING"; gh pr checks 14715 2>&1 | head -5
