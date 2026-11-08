#!/usr/bin/env bash
set -euo pipefail
SHA="__GZB64__"
printf '%s' "$SHA" | base64 -d | gunzip > /tmp/packet/v3_staged.sh
echo "staged sha256: $(sha256sum /tmp/packet/v3_staged.sh | cut -d' ' -f1)"
wc -c -l < /tmp/packet/v3_staged.sh
bash -n /tmp/packet/v3_staged.sh && echo "bash -n OK"
