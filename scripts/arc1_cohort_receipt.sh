#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: the Arc store path is decided for good. Delete this
# harness together with docs/plans/receipts/arc-1-shareability-frontier/ when
# either (a) the width>1 crossover lands and Arc returns as one combined change,
# or (b) the Arc direction is retired outright. It exists only to reproduce the
# +5.84% serial cost banked in docs/plans/arc-store-path-migration-decision.md.
#
# Arc-1 width-1 50-entry cohort receipt (operator bar, smart-badger-549).
# Both arms from one host, interleaved, one variable: Arc wrapper (after) vs Rc (base/main).
#
# BOTH ARMS ARE PINNED. The `after` arm is NOT the current checkout: #7875 closed
# unmerged, so main carries `std::rc::Rc` only and building `after` from $ROOT would
# compare Rc against Rc and report ~0% — silently contradicting the very measurement
# this harness exists to reproduce (DESIGN §5, fabricated plausible output). The pin
# below is asserted, not assumed: the run refuses if the arms are not what they claim.
#
# THE PROBE IS ONE FILE, SHARED BY BOTH ARMS. Only the library may differ. main's
# `p1_cohort_probe.rs` hardcodes `DiscoveryWidthPolicy::Adaptive` and reports no
# typecheck_compute_count, while the pinned probe defaults to Serial and reports the
# counter — so building each arm from its own tree would vary width policy and
# instrumentation alongside the representation, and a three-variable run cannot
# attribute its delta to Arc. subject.tsv already declares the correct method
# ("p1_cohort_probe.rs copied to base worktree so only compiler differs"); this
# harness now performs that copy instead of only documenting it.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RECEIPT_DIR="$ROOT/docs/plans/receipts/arc-1-shareability-frontier"
WT="$ROOT/.receipt-worktrees/arc1-base"
AFTER_WT="$ROOT/.receipt-worktrees/arc1-after"
# The measured commit. Authority: receipts/arc-1-shareability-frontier/subject.tsv `after_commit`.
AFTER_COMMIT="dfc1cb46c0233e468c5a947494b154824476a013"
ARC_ALIAS="std::sync::Arc as Rc"
PROBE_REL="src/v1/stage0/src/bin/p1_cohort_probe.rs"
mkdir -p "$RECEIPT_DIR"

# Fail closed on the one confusion that makes this harness lie: an arm that is not
# the representation it is labelled with. Typed, located, and loud — never a silent
# ~0% delta.
assert_arm_representation() {
  local label="$1" tree="$2" want="$3"  # want = present | absent
  local hits status n
  # grep exit 1 means "no matches" — the EXPECTED result for the Rc baseline, not an
  # error. Under `set -euo pipefail` an inline pipeline would take that 1 as the
  # substitution's status and kill the run with no diagnostic at all, so status is
  # captured and classified: 0 = matched, 1 = no match, >=2 = grep itself failed and
  # the arm is unverifiable, which refuses rather than reading as "absent".
  set +e
  hits="$(grep -rl -- "$ARC_ALIAS" "$tree/src/v1/stage0/src" 2>/dev/null)"
  status=$?
  set -e
  if [ "$status" -ge 2 ]; then
    echo "arc1_cohort_receipt: REFUSED — could not scan arm '$label' ($tree); grep exited $status." >&2
    echo "  The arm's representation is unverifiable. Refusing rather than treating an" >&2
    echo "  unreadable tree as the Rc baseline." >&2
    exit 2
  fi
  n="$([ -z "$hits" ] && echo 0 || printf '%s\n' "$hits" | wc -l | tr -d ' ')"
  if [ "$want" = present ] && [ "$n" -eq 0 ]; then
    echo "arc1_cohort_receipt: REFUSED — arm '$label' ($tree) contains no '$ARC_ALIAS'." >&2
    echo "  This arm must be the Arc representation; it is not. Refusing rather than" >&2
    echo "  reporting an Rc-vs-Rc delta of ~0% against a banked +5.84%." >&2
    exit 2
  fi
  if [ "$want" = absent ] && [ "$n" -ne 0 ]; then
    echo "arc1_cohort_receipt: REFUSED — arm '$label' ($tree) contains '$ARC_ALIAS' in $n file(s)." >&2
    echo "  This arm must be the Rc baseline; it is not. Refusing rather than reporting" >&2
    echo "  an Arc-vs-Arc delta." >&2
    exit 2
  fi
  echo "arc1_cohort_receipt: arm '$label' representation OK ($want, $n file(s) matched)"
}

# The probe must be byte-identical across arms, or the delta is not attributable to
# the representation. Refuse loudly rather than measure a three-variable run.
assert_probe_identical() {
  if ! cmp -s "$AFTER_WT/$PROBE_REL" "$WT/$PROBE_REL"; then
    echo "arc1_cohort_receipt: REFUSED — $PROBE_REL differs between arms after the copy." >&2
    echo "  Both arms must run one probe so the only variable is the library" >&2
    echo "  representation. Refusing rather than attributing a width-policy or" >&2
    echo "  instrumentation difference to Arc." >&2
    exit 2
  fi
  echo "arc1_cohort_receipt: probe identical across arms ($PROBE_REL)"
}

export CTRL_BUILD_WRAP_CARGO=0
export CTRL_BUILD_BYPASS_SHIMS=1
export RUSTC_WRAPPER=

echo "arc1_cohort_receipt: building after (Arc) binary at pinned $AFTER_COMMIT"
if ! git cat-file -e "${AFTER_COMMIT}^{commit}" 2>/dev/null; then
  echo "arc1_cohort_receipt: REFUSED — the measured commit $AFTER_COMMIT is not present." >&2
  echo "  Fetch it (it lives on session/loyal-ferret-892) before reproducing. Refusing" >&2
  echo "  rather than substituting the current checkout, which carries Rc only." >&2
  exit 2
fi
rm -rf "$AFTER_WT"
git worktree add -f --detach "$AFTER_WT" "$AFTER_COMMIT" >/dev/null
assert_arm_representation after "$AFTER_WT" present
cd "$AFTER_WT"
cargo build --release --bin p1_cohort_probe
AFTER_BIN="$AFTER_WT/target/release/p1_cohort_probe"
AFTER_SHA="$(sha256sum "$AFTER_BIN" | awk '{print $1}')"

echo "arc1_cohort_receipt: building base (main/Rc) binary in worktree"
rm -rf "$WT"
git worktree add -f "$WT" origin/main >/dev/null
# Representation is asserted on main's own tree, BEFORE the probe copy, so the check
# reads the library the base arm actually measures. The pinned probe carries no
# `$ARC_ALIAS` (it uses plain `std::sync::Arc`), so the copy cannot flip this verdict.
assert_arm_representation base "$WT" absent
cp "$AFTER_WT/$PROBE_REL" "$WT/$PROBE_REL"
assert_probe_identical
cd "$WT"
cargo build --release --bin p1_cohort_probe
BASE_BIN="$WT/target/release/p1_cohort_probe"
BASE_SHA="$(sha256sum "$BASE_BIN" | awk '{print $1}')"

run_arm() {
  local label="$1"
  local bin="$2"
  local sha="$3"
  local stderr="$RECEIPT_DIR/${label}.stderr.txt"
  local stdout="$RECEIPT_DIR/${label}.stdout.txt"
  local tsv="$RECEIPT_DIR/${label}.tsv"
  echo "arc1_cohort_receipt: starting $label"
  local start_ms
  start_ms="$(date +%s%3N)"
  set +e
  /usr/bin/time -f '__TIME__ wall_sec %e peak_rss_kb %M exit %x' \
    env GUNBC_P1_COHORT_RECEIPT=1 "$bin" >"$stdout" 2>"$stderr"
  local exit_code=$?
  set -e
  local end_ms
  end_ms="$(date +%s%3N)"
  local wall_ms=$((end_ms - start_ms))
  local time_line
  time_line="$(rg '^__TIME__' "$stderr" | tail -1 || true)"
  local peak_rss_kb="unreadable"
  if [[ -n "$time_line" ]]; then
    peak_rss_kb="$(echo "$time_line" | awk '{print $4}')"
  fi
  local pass fail typecheck_count
  pass="$(rg -c '^p1_cohort_probe: PASS' "$stderr" || echo 0)"
  fail="$(rg 'p1_cohort_probe: FAIL' "$stderr" | wc -l | tr -d ' ')"
  typecheck_count="$(rg -o 'typecheck_compute_count=[0-9]+' "$stderr" | tail -1 | cut -d= -f2 || echo unreadable)"
  {
    echo -e "label\t$label"
    echo -e "binary_sha256\t$sha"
    echo -e "exit\t$exit_code"
    echo -e "wall_ms\t$wall_ms"
    echo -e "peak_rss_kb\t$peak_rss_kb"
    echo -e "typecheck_compute_count\t$typecheck_count"
    echo -e "pass_probe_line\t$pass"
    echo -e "fail_lines\t$fail"
  } >"$tsv"
  echo "arc1_cohort_receipt: finished $label exit=$exit_code wall_ms=$wall_ms peak_rss_kb=$peak_rss_kb typecheck=$typecheck_count"
}

# Interleaved order per operator method reminder.
run_arm base-r1 "$BASE_BIN" "$BASE_SHA"
run_arm after-r1 "$AFTER_BIN" "$AFTER_SHA"
run_arm base-r2 "$BASE_BIN" "$BASE_SHA"
run_arm after-r2 "$AFTER_BIN" "$AFTER_SHA"

python3 - "$RECEIPT_DIR" "$AFTER_COMMIT" "$BASE_SHA" "$AFTER_SHA" "$PROBE_REL" <<'PY'
import json, pathlib, statistics, sys

receipt_dir = pathlib.Path(sys.argv[1])
# The measured commit is the PIN, not `git rev-parse HEAD` of whatever checkout
# invoked the script — a re-run from any other tree would otherwise overwrite the
# receipt's subject with a commit that was never measured (DESIGN §3).
after_commit = sys.argv[2]
base_sha, after_sha = sys.argv[3], sys.argv[4]
probe_rel = sys.argv[5]

def load_tsv(name):
    data = {}
    for line in (receipt_dir / f"{name}.tsv").read_text().splitlines():
        k, v = line.split("\t", 1)
        data[k] = v
    return data

arms = {name: load_tsv(name) for name in ("base-r1", "base-r2", "after-r1", "after-r2")}

def f(vals):
    return [float(v) for v in vals]

base_wall = f([arms["base-r1"]["wall_ms"], arms["base-r2"]["wall_ms"]])
after_wall = f([arms["after-r1"]["wall_ms"], arms["after-r2"]["wall_ms"]])
base_median = statistics.median(base_wall)
after_median = statistics.median(after_wall)
delta_pct = ((after_median - base_median) / base_median * 100) if base_median else 0

base_tc = [arms["base-r1"]["typecheck_compute_count"], arms["base-r2"]["typecheck_compute_count"]]
after_tc = [arms["after-r1"]["typecheck_compute_count"], arms["after-r2"]["typecheck_compute_count"]]

summary = {
    "subject": {
        "after_commit": after_commit,
        "base_ref": "origin/main",
        "cohort": "docs/plans/receipts/fill-composition-overlay-direction/cohort.tsv (first 50)",
        "width_policy": "Serial (width-1 inline drain)",
        "variable": "emitter wrapper Arc-as-Rc vs main Rc",
        "probe_source": f"{probe_rel} copied from {after_commit} into the base worktree "
                        "so both arms run one probe and only the library differs",
    },
    "binary_sha256": {"base": base_sha, "after": after_sha},
    "wall_ms": {"base": base_wall, "after": after_wall, "base_median": base_median, "after_median": after_median, "delta_pct": delta_pct},
    "peak_rss_kb": {
        "base": [arms["base-r1"]["peak_rss_kb"], arms["base-r2"]["peak_rss_kb"]],
        "after": [arms["after-r1"]["peak_rss_kb"], arms["after-r2"]["peak_rss_kb"]],
    },
    "typecheck_compute_count": {"base": base_tc, "after": after_tc, "identical": base_tc == after_tc},
    "bar": {"wall_regression_lte_5pct": delta_pct <= 5.0, "typecheck_identical": base_tc == after_tc},
}

(receipt_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary["bar"], indent=2))
PY

echo "arc1_cohort_receipt: wrote $RECEIPT_DIR/summary.json"
