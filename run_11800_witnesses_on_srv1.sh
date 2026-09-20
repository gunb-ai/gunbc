set -uo pipefail
REPO=${REPO:-}
HEADSHA=bfa6197c4e3
FILE=dag/test/claim/runner/runner_host_file_converge_witness_test.dag
if [ -z "$REPO" ]; then REPO=$(mktemp -d)/gunbc; git clone https://github.com/gunb-ai/gunbc.git "$REPO"; fi
cd "$REPO"
git fetch origin pull/11800/head
git checkout -q --detach "$HEADSHA"
echo "HEAD: $(git rev-parse HEAD)"
# The driver must live INSIDE the repo: gunbc panics on a source root outside the process
# workspace root (repo_relative_path_normalized: path /tmp/tmp.XXXX is not under process workspace root).
D="$REPO/.scratch-driver"
rm -rf "$D"; mkdir -p "$D"
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
# A non-interactive ssh to srv1 does not have cargo on PATH.
export PATH="$HOME/.cargo/bin:$PATH"
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
echo "=== MUTATION 2: the disjointness conjunct loses its subject (desired becomes the retiree roster itself, so the two overlap) ==="
sed -i 's/let desired = srv1_desired()/let desired = srv1_retired()/' "$FILE"
git --no-pager diff --stat -- "$FILE"
echo "=== RUN 3: mutated; the_real_srv1_retiree_roster_reaches_one_admitted_read MUST read FAIL and every other claim MUST be unchanged ==="
systemd-run --user --scope -p MemoryMax=24G "$G" run --source-root dag --source-root src/v2 --source-root "$D" --entry "$D/runner_host_file_report.dag" --function report
echo "rc=$?"
git checkout -- "$FILE"
rm -rf "$D"
# NOTE: rc=2 on both runs is EXPECTED and is not a failure. gunbc run refuses to map a String
# return to an exit code, and prints the report inside the refusal text. Read the lines, not the rc.
echo "=== restored: $(git status --porcelain -- "$FILE" | wc -l) modified paths remain ==="
