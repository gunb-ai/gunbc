#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell probe runner; until then it
# projects the cssl emit+assemble+cargo spine for per-module verdict TSV (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
# Authority: cssl_v1_compiled_probe_lib_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag
# (`curated_probe_cargo_toml_write_from_cssl_authority` — ProcessExit + Filesystem.Write;
# docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh — no parallel Cargo.toml heredoc).
#
# INVOCATION CONTRACT (2026-07-19, calm-boar-697 — durable; do not re-learn expensively):
#   CSSL_STD_SEED_LINK=1  — required for std-seed-link closure assembly via cssl_assemble.
#   PROBE_KEEP_LOG_DIR=<dir> — after each probe, copy cargo.log to <dir>/<module>.cargo.log
#                           for per-error census (e0599_census_extract.sh). Emit/assemble always
#                           use a fresh mktemp OUT per invocation — never reuse this dir as OUT
#                           (gunbc compile does not clear stale emitted .rs across runs).
#                           Clears any prior <module>.cargo.log at invocation so a missing log
#                           after a refuse path is observable (not a stale prior run).
#   shim_lib_rel (arg 2)  — ONLY the lane's own lib.rs from dag/tools/self_host_<lane>_shims/
#                           when that lane provides one (see behavioral_transport shim_lib_rel).
#   Empty = raw cssl-assembled lib.rs (correct default when no lane shim).
#   PHANTOM-GREEN hazard: wrong-lane lib.rs REPLACES cssl lib.rs entirely → cargo green with
#                           invalid shim (false green). Observed: normalize shim on 00_compile → PHANTOM.
#   FALSE-RED hazard:     missing required lane shim → out-of-scope refusals unrelated to entry.
#                           Observed: FormalNonterminal/BTreeSet without 03_normalize shims.
#   When verdicts flip between runs, diff INVOCATION first (shim path, CSSL_STD_SEED_LINK).
#   FAIL-CLOSED (2026-08-17): HARNESS_REFUSE and EMIT_REFUSE exit non-zero after printing the
#   TSV row — a recorded refusal must stop the line; exit 0 on harness down zeroed deficit frequency.
#   STALE-LOG (2026-08-17): clear_probe_keep_log rm -f's <dir>/<module>.cargo.log at invocation
#   and on harness refuse so a missing log after refuse is observable (not a prior run's file).
#   Use a fresh PROBE_KEEP_LOG_DIR per orchestrated sweep when switching cohorts.
#   PAIRED READING: publish a count beside any zero — a bare zero from this instrument is suspect.
#   ROW COLUMNS (2026-08-21, appended — existing consumers read no positions past column 8, so
#                        the five new fields are appended rather than inserted):
#     1 module  2 emit_summary  3 cargo_verdict  4 first_error  5 mapped_gate  6 verdict
#     7 error_histogram  8 raw_dup_pub_use  9 HEAD_SHA  10 CARGO_ERROR_TOTAL  11 HISTOGRAM_SUM
#     12 PRODUCER_PATH  13 EMIT_COUNT_SRC
#   WHY 9: a number published without its ref is a measurement of something nobody asked about,
#          and PROBE_EXPECT_BASE_SHA only protects callers who remember to declare a baseline.
#          The field protects every reader; the check protects a declared comparison. Both stay.
#   WHY 10 AND 11 TOGETHER, NEVER ONE: cargo's own "due to N previous errors" line and the count
#          of error lines are DIFFERENT INSTRUMENTS. Differencing one run's 10 against another's
#          11 reads exactly like a delta and is not one. Emitting both from one run makes that
#          confusion unrepresentable instead of merely warned against; a gap between them is
#          itself a reading (cargo counts errors, the histogram counts coded error lines), never
#          a discrepancy to reconcile. Both are "n/a" — deliberately not 0 — on any path that
#          never reached cargo, because a zero there reads as a clean build to anything summing
#          the column.
#   WHY 12 AND 13: SUBJECT, REF, PRODUCER. Columns 1 and 9 carry the first two; nothing carried
#          the third, and the instrument — not the ref — was the confounder every time this lane
#          differenced two numbers that were never comparable. 12 names the stage chain the run
#          ACTUALLY performed (accumulated as stages execute, so a skipped assembly or an absent
#          shim is visible rather than asserted away), and 13 names which of the two producers
#          inside column 2 supplied its count: the compiler's self-reported `compiled:` line, or
#          this script's own `find -name '*.rs'`. A row that cannot say which instrument and
#          which stage produced it cannot be safely differenced against anything, and no care at
#          the reading end recovers that — it is a field, not a habit.
#   Ground-truth discriminator for embedded refusals: rg 'UNRESOLVED_CompilerError' or the rustc
#                           error literal in the emitted crate AFTER cssl_assemble — compile_error!
#                           in source = real emit-residue (no shim can fix); string-only = note.
#   Lane shim authority: dag/tools/self_host_*_behavioral_transport.dag shim_lib_rel per module.
#   Exit codes: 0 = measurement completed (emit reached cargo — including cargo refuse rows);
#               1 = line-stop refuse (HARNESS_REFUSE or EMIT_REFUSE; HARNESS_REFUSE sets
#                   residual_histogram instrument_down:1; SAME_BASE_REFUSE below shares this code);
#               2 = usage error.
#   STALE-BINARY (2026-08-21, royal-stag-736 found it, bright-moth-92 fixed it here): the probe
#   previously rebuilt gunbc/cssl_assemble only when the file was ABSENT, so a base->head loop in a
#   single dispatch silently re-used the base tree's binary for the head pass and reported a FALSE
#   IDENTICAL — "my fix changed nothing", with no failure arm. Both binaries are now keyed on
#   `git rev-parse HEAD` via a `<binary>.tree` stamp and rebuilt on a key miss. Set GUNBC=/path to
#   pin a binary deliberately; an externally pinned binary carrying no stamp rebuilds rather than
#   being trusted. This is the binary-side twin of PROBE_EXPECT_BASE_SHA below: that pins the TREE,
#   this pins the COMPILER, and a confident number needs both.
#   SAME-BASE REFUSAL (2026-08-19, smart-ram-730): a measurement being compared against a prior
#   baseline is only meaningful if both were taken at the same tree. PROBE_EXPECT_BASE_SHA=<sha> —
#   when set, refuses BEFORE any build work if `git rev-parse HEAD` in ROOT does not match, naming
#   both SHAs. No flag or mode weakens this: absence of the var means no comparison was declared,
#   never "proceed anyway" — there is deliberately no override arm. Same shape as the stale-binary
#   check this probe's now-deleted predecessor (fast_probe.sh) named requirement (1): both failure
#   modes are a confident number computed against the wrong thing, so both refuse rather than warn.
#   RUNG (2026-08-19, smart-ram-730 review): mechanically preventable WHEN ARMED, not mechanically
#   preventable — a caller that never sets PROBE_EXPECT_BASE_SHA gets no protection and that failure
#   is silent, so the check's existence is not coverage. Next-rung trigger, named rather than
#   stalled: DERIVE the expected base (this worktree's merge-base against origin/main) instead of
#   requiring it be declared, so there is nothing to remember and nothing to forget. Not tonight's
#   work; do not bundle it into an unrelated change.
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <module.dag-path> [shim_lib_rel]" >&2
  exit 2
fi

MODULE_PATH="$1"
SHIM_LIB_REL="${2:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT"

# The measured tree's sha, captured UNCONDITIONALLY and published as a row field. This is not the
# same mechanism as PROBE_EXPECT_BASE_SHA below and does not replace it: that check refuses a run
# against the wrong tree, this one makes it impossible to publish a number without its ref. The
# check protects a declared comparison; the field protects every reader, including the ones who
# never declare one. Measured cost of not having it (2026-08-21): three consecutive probe and emit
# runs in one session were attributed to a ref that was not the ref they ran at — twice because
# main advanced between checkout and dispatch, once because a `git checkout … || true` swallowed
# its own failure — and each was caught only because that session happened to echo the sha from
# inside its own wrapper. A per-caller habit is not a mechanism; the row is.
HEAD_SHA="$(git rev-parse HEAD)"

# PRODUCER, the third field every published number owes beside SUBJECT and REF (2026-08-21,
# smart-ram-730). HEAD_SHA pins the TREE. It does not pin the INSTRUMENT, and the instrument was
# the confounder in every reconciliation this lane lost time to: a raw `gunbc compile
# --output-dir` counted 176 emitted files at the exact ref where this probe's EMIT_SUMMARY said
# 177, and the two were differenced as a delta for forty minutes because neither output stated
# which stage it had measured. PRODUCER_PATH is ACCUMULATED as stages actually execute, never
# declared as a literal: CSSL_STD_SEED_LINK=0 skips assembly and a shim may or may not be
# installed, so a static string would describe a pipeline the run did not perform. EMIT_COUNT_SRC
# splits the second confounder, which lives INSIDE one column: EMIT_SUMMARY is either the
# compiler's own self-reported `compiled:` line or this script's `find -name '*.rs' | wc -l`, and
# those are two producers wearing one field name.
PRODUCER_PATH="curated_cargo_probe_one"
EMIT_COUNT_SRC="none"

if [[ -n "${PROBE_EXPECT_BASE_SHA:-}" ]]; then
  if [[ "$HEAD_SHA" != "$PROBE_EXPECT_BASE_SHA" ]]; then
    echo "curated_cargo_probe: SAME_BASE_REFUSE — tree at $HEAD_SHA, comparison baseline expects $PROBE_EXPECT_BASE_SHA" >&2
    exit 1
  fi
fi

# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$SCRIPT_DIR/lib/render_cssl_probe_lib_cargo_toml.sh"

STD_SEED_LINK="${CSSL_STD_SEED_LINK:-0}"

GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
CSSL_ASSEMBLE="${CSSL_ASSEMBLE:-$ROOT/target/release/cssl_assemble}"

# The binary is a function of the tree it was built from, so it is KEYED on that tree.
# `-x` alone asks "does a binary exist", which is a different question and the wrong one:
# a base->head loop inside one dispatch leaves base's binary in place, and the head pass
# then emits with the PRE-FIX compiler while reporting head's SHA. That reads as "the fix
# changed nothing" — a false identical, with no failure arm anywhere.
# Rebuilding on a key miss is a cache doing its job, not a fallback: the answer computed is
# the correct one for the checked-out tree. An unreadable or absent stamp is a miss.
probe_binary_tree_key() { git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo "no-git"; }
probe_binary_is_current() {
  local bin="$1"
  [[ -x "$bin" ]] || return 1
  [[ -f "$bin.tree" ]] || return 1
  [[ "$(cat "$bin.tree" 2>/dev/null)" == "$(probe_binary_tree_key)" ]] || return 1
}
probe_stamp_binary() { probe_binary_tree_key > "$1.tree"; }

if ! probe_binary_is_current "$GUNBC"; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc >/dev/null
  GUNBC="$ROOT/target/release/gunbc"
  probe_stamp_binary "$GUNBC"
fi
if [[ "$STD_SEED_LINK" == "1" ]] && ! probe_binary_is_current "$CSSL_ASSEMBLE"; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin cssl_assemble >/dev/null
  CSSL_ASSEMBLE="$ROOT/target/release/cssl_assemble"
  probe_stamp_binary "$CSSL_ASSEMBLE"
fi
export GUNBC

PROBE_LOG_BASENAME="$(basename "$MODULE_PATH" .dag)"

probe_keep_log_path() {
  echo "${PROBE_KEEP_LOG_DIR:-}/${PROBE_LOG_BASENAME}.cargo.log"
}

clear_probe_keep_log() {
  [[ -n "${PROBE_KEEP_LOG_DIR:-}" ]] || return 0
  rm -f "$(probe_keep_log_path)"
}

publish_probe_keep_log() {
  local build_log="$1"
  [[ -n "${PROBE_KEEP_LOG_DIR:-}" ]] || return 0
  [[ -f "$build_log" ]] || return 0
  cp "$build_log" "$(probe_keep_log_path)"
}

if [[ -n "${PROBE_KEEP_LOG_DIR:-}" ]]; then
  mkdir -p "$PROBE_KEEP_LOG_DIR"
  clear_probe_keep_log
fi
OUT="$(mktemp -d "${TMPDIR:-/tmp}/cssl-probe.XXXXXX")"
cleanup() { rm -rf "$OUT"; }
trap cleanup EXIT

EMIT_LOG="$OUT/emit.log"
EMIT_OK=0
if "$GUNBC" compile \
  --source-root dag \
  --source-root src/v2 \
  --entry "$MODULE_PATH" \
  --output-dir "$OUT" \
  --target rust \
  --dependency-pool-index primary-precedence \
  >"$EMIT_LOG" 2>&1; then
  EMIT_OK=1
fi

EMIT_SUMMARY="emit_fail"
if [[ "$EMIT_OK" -eq 1 ]]; then
  if grep -q 'compiled:' "$EMIT_LOG"; then
    EMIT_SUMMARY="$(grep -m1 'compiled:' "$EMIT_LOG" | sed 's/.*compiled: //')"
    EMIT_COUNT_SRC="gunbc_compiled_line"
  else
    FILE_COUNT="$(find "$OUT" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
    EMIT_SUMMARY="${FILE_COUNT}files,unknown_diag"
    EMIT_COUNT_SRC="probe_find_rs_files"
  fi
fi
if [[ "$EMIT_OK" -eq 1 ]]; then
  PRODUCER_PATH="$PRODUCER_PATH+emit"
fi

CARGO_VERDICT="skip"
# Defaults for the paths that never reach cargo. "n/a" is deliberately not "0": a run that did not
# build has no error total, and a zero there would read as a clean build on any consumer that sums
# the column.
CARGO_ERROR_TOTAL="n/a"
HISTOGRAM_SUM="n/a"
FIRST_ERROR=""
MAPPED_GATE=""
ERROR_HISTOGRAM=""
RAW_DUP_PUB_USE="unmeasured"

measure_raw_dup_pub_use() {
  local src_dir="$1"
  python3 - "$src_dir" <<'PY'
import re, sys, pathlib
src = pathlib.Path(sys.argv[1])
findings = []
for f in sorted(src.glob("*.rs")):
    seen = {}
    for line in f.read_text().splitlines():
        t = line.strip()
        if not t.startswith("pub use "):
            continue
        rest = t[len("pub use "):]
        m = re.match(r"^(.*)\{(.*)\}\s*;", rest)
        syms = [s.strip() for s in m.group(2).split(",")] if m else [rest.rstrip(";").split("::")[-1].strip()]
        for s in syms:
            if not s:
                continue
            base = s.split(" as ")[0].strip()
            if base in seen:
                findings.append(f"{f.name}:{base}")
            else:
                seen[base] = True
print(f"{len(findings)}" + (":" + ",".join(findings[:8]) if findings else ""))
PY
}

uncoded_histogram_suffix() {
  local build_log="$1"
  python3 - "$build_log" <<'PY'
import re, sys, collections
path = sys.argv[1]
counts = collections.Counter()
with open(path, encoding="utf-8", errors="replace") as fh:
    for line in fh:
        if not line.startswith("error: "):
            continue
        if "could not compile" in line or "aborting due to" in line:
            continue
        msg = " ".join(line[len("error: "):].split())
        counts[msg] += 1
parts = []
for msg, n in counts.most_common():
    slug = re.sub(r"[^a-zA-Z0-9_]+", "_", msg).strip("_")[:56]
    parts.append(f"uncoded_{slug}:{n}")
print(" ".join(parts))
PY
}

emit_harness_refuse_row_and_exit() {
  ERROR_HISTOGRAM="instrument_down:1"
  clear_probe_keep_log
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT" "$ERROR_HISTOGRAM" "$RAW_DUP_PUB_USE" \
    "$HEAD_SHA" "$CARGO_ERROR_TOTAL" "$HISTOGRAM_SUM" "$PRODUCER_PATH" "$EMIT_COUNT_SRC"
  exit 1
}

if [[ "$EMIT_OK" -eq 1 ]]; then
  RAW_DUP_PUB_USE="$(measure_raw_dup_pub_use "$OUT/src")"
  if [[ "$STD_SEED_LINK" == "1" ]]; then
    if ! "$CSSL_ASSEMBLE" --out-dir "$OUT" --entry-dag "$MODULE_PATH" --root "$ROOT" >"$OUT/assemble.log" 2>&1; then
      CARGO_VERDICT="harness_refuse"
      FIRST_ERROR="$(grep -m1 'CSSL_ASSEMBLE: REFUSED' "$OUT/assemble.log" || head -1 "$OUT/assemble.log")"
      MAPPED_GATE="HARNESS_SEED_LINK"
      VERDICT="HARNESS_REFUSE"
      emit_harness_refuse_row_and_exit
    fi
    PRODUCER_PATH="$PRODUCER_PATH+seedlink"
  fi

  if [[ -n "$SHIM_LIB_REL" && -f "$ROOT/$SHIM_LIB_REL" ]]; then
    cp "$ROOT/$SHIM_LIB_REL" "$OUT/src/lib.rs"
    shim_dir="$(dirname "$ROOT/$SHIM_LIB_REL")"
    for f in "$shim_dir"/*.rs; do
      [[ -f "$f" ]] || continue
      base="$(basename "$f")"
      [[ "$base" == "lib.rs" ]] && continue
      cp "$f" "$OUT/src/$base"
    done
    PRODUCER_PATH="$PRODUCER_PATH+shim"
  fi

  if ! render_cssl_probe_lib_cargo_toml "$ROOT" "$OUT/Cargo.toml"; then
    CARGO_VERDICT="harness_refuse"
    FIRST_ERROR="cssl harness authority unavailable"
    MAPPED_GATE="HARNESS_MISSING"
    VERDICT="HARNESS_REFUSE"
    emit_harness_refuse_row_and_exit
  fi

  BUILD_LOG="$OUT/cargo.log"
  PRODUCER_PATH="$PRODUCER_PATH+cargo"
  if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$BUILD_LOG"); then
    CARGO_VERDICT="green"
    ERROR_HISTOGRAM="clean"
  else
    CARGO_VERDICT="refuse"
    # BOTH TOTALS, ALWAYS, AND THIS IS THE POINT OF THE PAIR. cargo's own
    # "due to N previous errors" line and the sum of this histogram are DIFFERENT INSTRUMENTS
    # answering the same-sounding question, and differencing one against the other is a silent
    # category error — it reads as a delta and is not one. Publishing exactly one of them is what
    # made that mistake possible: a reader who receives "655" cannot tell which field produced it,
    # and a later run that publishes the other field looks comparable. Emitting both, from one
    # run, makes the ambiguity unrepresentable rather than merely warned about, and a gap between
    # them is itself informative (cargo counts errors, the histogram counts coded error LINES).
    CARGO_ERROR_TOTAL="$(grep -oE 'due to [0-9]+ previous error' "$BUILD_LOG" | grep -oE '[0-9]+' | head -1)"
    CARGO_ERROR_TOTAL="${CARGO_ERROR_TOTAL:-unreported}"
    ERROR_HISTOGRAM="$(grep -oE '^error\[E[0-9]+\]' "$BUILD_LOG" | sort | uniq -c | sort -rn | awk '{printf "%s%s:%s", sep, $2, $1; sep=" "}' || true)"
    HISTOGRAM_SUM="$(grep -cE '^error(\[E[0-9]+\])?:' "$BUILD_LOG" || true)"
    UNCODED_SUFFIX="$(uncoded_histogram_suffix "$BUILD_LOG")"
    if [[ -z "$ERROR_HISTOGRAM" ]]; then
      ERROR_HISTOGRAM="${UNCODED_SUFFIX:-uncoded_only:0}"
    elif [[ -n "$UNCODED_SUFFIX" ]]; then
      ERROR_HISTOGRAM="$ERROR_HISTOGRAM $UNCODED_SUFFIX"
    fi
    FIRST_ERROR="$(grep -m1 -E '^error(\[E[0-9]+\])?:' "$BUILD_LOG" || grep -m1 -E '^error:' "$BUILD_LOG" || head -1 "$BUILD_LOG" || true)"
    if echo "$FIRST_ERROR" | grep -qE 'is defined multiple times|defined multiple times'; then
      MAPPED_GATE="HARNESS_ARTIFACT_std_dup"
    elif echo "$FIRST_ERROR" | grep -qE 'UNRESOLVED_CompilerError'; then
      MAPPED_GATE="UNKNOWN_unresolved"
    elif echo "$FIRST_ERROR" | grep -qE 'expected item after attributes|expected one of|unexpected token'; then
      MAPPED_GATE="UNKNOWN_emit_shape"
    elif echo "$FIRST_ERROR" | grep -qE 'error\[E0432\]: unresolved import|cannot find .+ in this scope|not found in this scope|unbound'; then
      MAPPED_GATE="namespace_resolution"
    elif echo "$FIRST_ERROR" | grep -qE 'error\[E0382\]|error\[E0507\]|error\[E0597\]|cannot move out of|cannot borrow|mismatched types.*Rc<|expected .+ found .+Rc<'; then
      MAPPED_GATE="Gate_A_emitter_Rc_Optional"
    elif echo "$FIRST_ERROR" | grep -qE 'wrapper\.retained|body_producer|Arrow|Behavior'; then
      MAPPED_GATE="Gate_B_body_producer"
    else
      MAPPED_GATE="UNKNOWN"
    fi
  fi
  if [[ -n "${BUILD_LOG:-}" && -f "$BUILD_LOG" ]]; then
    publish_probe_keep_log "$BUILD_LOG"
  fi
fi

VERDICT="PENDING"
if [[ "$CARGO_VERDICT" == "green" ]]; then
  if [[ -n "$SHIM_LIB_REL" ]]; then
    VERDICT="PHANTOM"
  else
    VERDICT="CARGO_GREEN"
  fi
elif [[ "$CARGO_VERDICT" == "refuse" ]]; then
  if [[ "$MAPPED_GATE" == "HARNESS_ARTIFACT_std_dup" ]]; then
    VERDICT="HARNESS_ARTIFACT"
  elif [[ "$MAPPED_GATE" == "Gate_A_emitter_Rc_Optional" ]]; then
    VERDICT="CONFIRMED-Gate_A"
  elif [[ "$MAPPED_GATE" == "Gate_B_body_producer" ]]; then
    VERDICT="CONFIRMED-Gate_B"
  elif [[ "$MAPPED_GATE" == "namespace_resolution" ]]; then
    VERDICT="CONFIRMED-namespace"
  elif [[ "$MAPPED_GATE" == UNKNOWN_* ]]; then
    VERDICT="UNKNOWN-$(echo "$FIRST_ERROR" | tr ' ' '_' | cut -c1-80)"
  else
    VERDICT="UNKNOWN-$(echo "$FIRST_ERROR" | tr ' ' '_' | cut -c1-80)"
  fi
elif [[ "$EMIT_OK" -eq 0 ]]; then
  CARGO_VERDICT="emit_fail"
  FIRST_ERROR="$(head -3 "$EMIT_LOG" | tr '\n' ' ')"
  VERDICT="EMIT_REFUSE"
  clear_probe_keep_log
fi

printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
  "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT" "$ERROR_HISTOGRAM" "$RAW_DUP_PUB_USE" \
  "$HEAD_SHA" "$CARGO_ERROR_TOTAL" "$HISTOGRAM_SUM" "$PRODUCER_PATH" "$EMIT_COUNT_SRC"

case "$VERDICT" in
  HARNESS_REFUSE | EMIT_REFUSE) exit 1 ;;
esac
