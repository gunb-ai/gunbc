#!/usr/bin/env bash
# TEMPORARY probe -- NOT shipped, NOT committed. Runs the committed generator route
# (gunbc.instruments.generated_artifact_gate :: main_wet_one) for ONE host and reports
# whether it actually reproduced the committed sudoers bytes.
#
# A run that dies before the write leaves the committed file untouched, which is NOT
# agreement: the verdict below requires rc=0 AND a nonzero Filesystem.Write count
# AND sha256 equality. Anything else is "not verified".
set -uo pipefail
cd "$(dirname "$0")/.."

REL="${1:?usage: $0 provisioning/srvN/gunbc-ghrunner.sudoers}"
[ -f "$REL" ] || { echo "@@FOLD $REL rc=2 committed=- fold=- match=NO reason=missing-file"; echo "MWO_VERIFY DONE"; exit 1; }

export PATH="$HOME/.cargo/bin:$PATH"
for p in /opt/homebrew/bin /usr/local/bin "$HOME/.local/bin"; do [ -d "$p" ] && PATH="$p:$PATH"; done

echo "@@HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
BUILD_RC=$?
echo "@@BUILD_RC=$BUILD_RC"
[ "$BUILD_RC" = 0 ] || { echo "@@FOLD $REL rc=$BUILD_RC committed=- fold=- match=NO reason=build"; echo "MWO_VERIFY DONE"; exit 1; }

before=$(sha256sum "$REL" | cut -d' ' -f1)
log="/tmp/mwo_$(basename "$(dirname "$REL")").log"

timeout 1500 ./target/release/gunbc run \
  --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry dag/gunbc/instruments/generated_artifact_gate.dag \
  --function main_wet_one --arg "path=$REL" > "$log" 2>&1
rc=$?
echo "@@MAIN_WET_RC=$rc LOG=$log"
echo "@@ERROR_TAIL $(grep -m3 -E '^error|cause:' "$log" | tr '\n' '|')"

after=$(sha256sum "$REL" | cut -d' ' -f1)
writes=$(grep -c 'Filesystem.Write' "$log" || true)
echo "@@WRITE_COUNT=$writes"
echo "@@FOLD $REL rc=$rc writes=$writes committed=$before fold=$after match=$([ "$rc" = 0 ] && [ "$writes" -gt 0 ] && [ "$before" = "$after" ] && echo YES || echo NO)"
echo "MWO_VERIFY DONE"
