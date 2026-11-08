#!/bin/bash
# fx-e2e-v10.sh — END-TO-END pre-fetch run of the v10 packet (parent msg_bf3d2085 B1):
# cgroup files shimmed (fx-harness-v10.sh), git stubbed so the FIRST fetch prints the marker and fails;
# the packet must have passed EVERY pre-fetch gate (all fixtures, runtime-path fixtures, bound checks,
# OUT exclusivity, mem-start) before the marker appears. The die2 that follows is expected (rc 2).
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
STUB="$(mktemp -d)"; OUTD="$(mktemp -d)/out.e2e"
cat > "$STUB/git" <<'GITSTUB'
#!/bin/bash
for a in "$@"; do
  if [ "$a" = fetch ]; then echo "E2E_FETCH_STUB_REACHED argv: $*"; exit 1; fi
done
exec /usr/bin/git "$@"
GITSTUB
chmod +x "$STUB/git"
PATH="$STUB:$PATH" bash "$HERE/fx-harness-v10.sh" "$OUTD" 2>&1
RC=$?
echo "E2E harness rc=$RC"
exit $RC
