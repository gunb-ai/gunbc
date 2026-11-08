#!/bin/bash
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
STUB="$(mktemp -d)"; OUTD="$(mktemp -d)/out.e2e"
cat > "$STUB/git" <<'GITSTUB'
for a in "$@"; do
  if [ "$a" = fetch ]; then echo "E2E_FETCH_STUB_REACHED argv: $*"; exit 1; fi
done
exec /usr/bin/git "$@"
GITSTUB
chmod +x "$STUB/git"
PATH="$STUB:$PATH" bash "$HERE/fx-harness-v14.sh" "$OUTD" 2>&1
RC=$?
echo "E2E harness rc=$RC"
exit $RC
