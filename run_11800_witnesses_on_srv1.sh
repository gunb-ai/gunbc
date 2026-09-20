set -uo pipefail
REPO=${REPO:-}
HEADSHA=eb9a0698ced3ca3121fc9460453a0ba1da1be920
FILE=dag/test/claim/runner/runner_host_file_converge_witness_test.dag
if [ -z "$REPO" ]; then REPO=$(mktemp -d)/gunbc; git clone https://github.com/gunb-ai/gunbc.git "$REPO"; fi
cd "$REPO"
git fetch origin pull/11800/head
git checkout -q --detach "$HEADSHA"
echo "HEAD: $(git rev-parse HEAD)"
D=$(mktemp -d)
python3 - "$D" "$FILE" <<'PY'
import re, sys
d, p = sys.argv[1], sys.argv[2]
names = re.findall(r'^test fn (\w+)', open(p).read(), re.M)
imp = ",\n  ".join(names)
rows = ",\n    ".join('join(["%s", if %s() { "=PASS" } else { "=FAIL" }], "")' % (n, n) for n in names)
open(d + "/runner_host_file_report.dag", "w").write(
"""module scratch.runner_host_file_report

import std.types { Bool, String }
import test.claim.runner_host_file_converge_witness_test {
  %s
}

fn report() -> String {
  join([
    %s
  ], "\\n")
}
""" % (imp, rows))
print("claims in driver:", len(names))
PY
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/tmp/gunbc-11800-target}
cargo build --release -p v1-compiler --bin gunbc || exit 1
G=$CARGO_TARGET_DIR/release/gunbc
echo "=== RUN 1: unmutated, all claims at $HEADSHA ==="
systemd-run --user --scope -p MemoryMax=24G "$G" run --source-root dag --source-root src/v2 --source-root "$D" --entry "$D/runner_host_file_report.dag" --function report
echo "rc=$?"
echo "=== MUTATION: the identity join in the_real_srv1_retiree_roster_reaches_one_admitted_read loses its grounding (the authority side becomes the in-file three-retiree fixture) ==="
sed -i 's/retired == required_retirees_for_host(host: operator_host_srv1)/retired == three_retirees()/' "$FILE"
git --no-pager diff --stat -- "$FILE"
echo "=== RUN 2: mutated; the_real_srv1_retiree_roster_reaches_one_admitted_read MUST read FAIL and every other claim MUST be unchanged ==="
systemd-run --user --scope -p MemoryMax=24G "$G" run --source-root dag --source-root src/v2 --source-root "$D" --entry "$D/runner_host_file_report.dag" --function report
echo "rc=$?"
git checkout -- "$FILE"
echo "=== restored: $(git status --porcelain -- "$FILE" | wc -l) modified paths remain ==="
