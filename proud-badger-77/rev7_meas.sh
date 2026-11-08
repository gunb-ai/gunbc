#!/usr/bin/env bash
# nominal_child_edge_table.sh -- FROZEN, REVIEW-ONLY. Not granted; nothing runs it until keen-otter names a grant AND a backend.
# Question (adhoc-afb0f800-78c, PR #16061): per side (declared vs produced) and per subject (concrete / generic / alias), which
#   connective and which EdgeLabel arm do nominal type-application ARGUMENT edges carry, and does the judge accept/refuse?
# THIS IS A MEASUREMENT TABLE, NOT A HYPOTHESIS TEST: several cells are DESIGNED to be FAIL today. A FAIL is data, not a mismatch.
# EXIT CONTRACT (project): 0 = OBSERVED (every one of the 42 cells has exactly one typed PASS or FAIL line, table complete);
#   1 = not used by this measurement (no cell's FAIL is a refutation); 2 = NO_OBSERVATION (infra refusal, build failure,
#   closure failure, incomplete roster, a cell not evaluated, abort, signal).
# COLLECTOR RULE: a receipt is valid ONLY with exactly one `OUTCOME=<OBSERVED|NO_OBSERVATION> rc=<n> reason=...` line
#   whose rc equals the process exit status. A receipt with none, two, or a mismatched rc is NO_OBSERVATION. No retry.
# LAUNCHER MUST PROVIDE: a private network namespace (e.g. PrivateNetwork=yes; else preflight refuses), a private cgroup with
#   memory.max <= BOOKED_MEM_BYTES, MemorySwapMax 0, finite pids.max; inputs SRC (clean pinned checkout), OUT (scratch dir),
#   PRIVATE_ROOT (dir that owns CARGO_HOME), CARGO_HOME (pre-populated, under PRIVATE_ROOT, not $HOME/.cargo), BOOKED_MEM_BYTES.
# PROPOSED, NEVER-GRANTED limits: memory.max <= 16G, MemorySwapMax 0, wall 45 min. Phase budgets incl. kill grace (-k 20s):
#   P0 preflight 3m enforced by $SECONDS check; ONE aggregate 2700s deadline gates every later phase (see DEADLINE_S) (each git/rustup/rustc call 10s) + P1 build 12m+20s + P3 claim run 28m20s+20s + collect 1m = 45m (rev7: separate P2 closure gate removed; claim_batch's own index/closure/resolve failure is NO_OBSERVATION in P3).
# UNVERIFIED (grep limit): whether claim_batch/compile_dag_rust_emit_check needs fleet-converge's --features
#   (text_lookup_work_counter,interp_test_witness,test_hooks); I found no requirement by grep. If it does, every cell is
#   UNOBSERVED = NO_OBSERVATION.
# HERMETIC SEMANTICS (v1_interpreter ExecutionMode::Hermetic, pinned blob): env reads return None; service effects are refused or
#   served from fixtures; Filesystem.Read/List of a path proven under the checkout root stays a real read (checkout-input carve-out);
#   host effects refuse with HostEffectRefused. The cells build their sources from String data and resolve in memory.
# PROVENANCE: head 5b02c6dd (PR #16061, one added file vs base), merge-base with main a34217368e9f; pins.txt records the tree of
#   dag, src/v2, src/v1 (the seed), rust-toolchain.toml and Cargo.lock, plus the toolchain's rustc -vV.
# PROCESS CLEANUP: each phase runs in its own session (setsid); finish() SIGKILLs every recorded process group, so no
#   grandchild outlives the script. The launcher's cgroup is the outer bound.
# SIGNALS: phases run in the background and the script waits, so TERM/HUP/INT interrupt the wait at once; finish() then SIGKILLs
#   every phase process group and writes NO_OBSERVATION. A scope SIGKILL leaves no OUTCOME line = NO_OBSERVATION (collector rule).
# All 42 cells return Bool (checked at the pinned blob). Mode: claim_batch --claim-run --hermetic (explicit). Classification is from typed outcomes only: no line relabels another claim.
set -uo pipefail
OUT="${OUT:-}"; R=""
FINISHED=""
PGS=""
reap() { local g; for g in $PGS; do kill -KILL -- "-$g" 2>/dev/null; done; return 0; }
# ONE AGGREGATE DEADLINE: DEADLINE_S=2700 (45m) over the whole script, measured by bash $SECONDS from script start.
# Every phase refuses to START unless remaining >= its cap + 20s kill grace + COLLECT_S reserved for collection/cleanup;
# collection is re-checked before the final OUTCOME. Sum of caps: 180 + (720+20) + (1700+20) + 60 = 2700 <= 2700.
DEADLINE_S=2700; COLLECT_S=60
deadline_gate() { local need=$(( $1 + 20 + COLLECT_S )) rem=$(( DEADLINE_S - SECONDS ))
  [ "$rem" -ge "$need" ] || refuse "aggregate_deadline: ${rem}s remain, phase needs ${need}s (at ${SECONDS}s of ${DEADLINE_S}s)"; }
# phase <secs> <stdout> <stderr> cmd...: deadline-gated, run in a new session, bounded, group-killed after; sets PRC.
phase() { local s="$1" o="$2" e="$3"; shift 3
  deadline_gate "$s"
  setsid timeout -k 20 "$s" "$@" >"$o" 2>"$e" & local pg=$!; PGS="$PGS $pg"
  wait "$pg"; PRC=$?; kill -KILL -- "-$pg" 2>/dev/null; return 0; }
finish() { # outcome rc reason
  [ -z "$FINISHED" ] || return 0
  FINISHED=1
  reap
  local line="OUTCOME=$1 rc=$2 reason=$3"
  echo "$line"; [ -n "$R" ] && echo "$line" >>"$R"
  trap - EXIT; exit "$2"
}
trap 'finish NO_OBSERVATION 2 "aborted_before_outcome (set -u or unexpected exit)"' EXIT
trap 'finish NO_OBSERVATION 2 "signal_TERM"' TERM
trap 'finish NO_OBSERVATION 2 "signal_HUP"' HUP
trap 'finish NO_OBSERVATION 2 "signal_INT"' INT
refuse() { finish NO_OBSERVATION 2 "$1"; }
say() { echo "$*"; [ -n "$R" ] && echo "$*" >>"$R"; return 0; }

HEAD_FULL=5b02c6dd3d963cb9863b163a69cba56e900f8698
TREE_FULL=a5d5e58f257d507445127048aa855a8062257ab4
WITNESS=src/v2/test/claim/compiler/nominal_application_child_edge_shape.dag
WITNESS_BLOB=eef2fa5d1ab2a1fe5552cec76e6bb971b13c97a4
BATCH_SRC_BLOB=12a71786bd92f683dbd0aec2ae5d6f426e651a44
LOCK_BLOB=35ad794ac7701b0e7d77687c2954db37a5473203
FUNCTIONS=nal_arm_concrete_declared_argument_authored_holds,nal_arm_concrete_declared_argument_positional_holds,nal_arm_concrete_declared_argument_structural_holds,nal_arm_concrete_produced_argument_authored_holds,nal_arm_concrete_produced_argument_positional_holds,nal_arm_concrete_produced_argument_structural_holds,nal_arm_generic_declared_argument_authored_holds,nal_arm_generic_declared_argument_positional_holds,nal_arm_generic_declared_argument_structural_holds,nal_arm_generic_declared_normalized_authored_holds,nal_arm_generic_declared_normalized_positional_holds,nal_arm_generic_produced_argument_authored_holds,nal_arm_generic_produced_argument_positional_holds,nal_arm_generic_produced_argument_structural_holds,nal_arm_generic_produced_normalized_authored_holds,nal_arm_generic_produced_normalized_positional_holds,nal_differing_alias_application_refuses_holds,nal_differing_concrete_nominal_application_refuses_holds,nal_differing_generic_nominal_application_refuses_holds,nal_identical_alias_application_is_accepted_holds,nal_identical_concrete_nominal_application_is_accepted_holds,nal_identical_generic_nominal_application_is_accepted_holds,nal_nested_second_argument_disagreement_refuses_holds,nal_node_concrete_declared_arguments_positional_holds,nal_node_concrete_declared_denotation_reads_holds,nal_node_concrete_declared_is_instantiation_holds,nal_node_concrete_declared_normalized_is_instantiation_holds,nal_node_concrete_declared_side_is_reachable_holds,nal_node_concrete_produced_arguments_positional_holds,nal_node_concrete_produced_denotation_reads_holds,nal_node_concrete_produced_is_instantiation_holds,nal_node_concrete_produced_normalized_is_instantiation_holds,nal_node_concrete_produced_side_is_reachable_holds,nal_node_concrete_sides_are_structurally_equal_holds,nal_node_generic_declared_arguments_positional_holds,nal_node_generic_declared_is_instantiation_holds,nal_node_generic_declared_normalized_arguments_positional_holds,nal_node_generic_declared_normalized_is_instantiation_holds,nal_node_generic_produced_arguments_positional_holds,nal_node_generic_produced_is_instantiation_holds,nal_node_generic_produced_normalized_arguments_positional_holds,nal_node_generic_produced_normalized_is_instantiation_holds
ROSTER="${FUNCTIONS//,/ }"

# ---- P0 preflight (3m): refuse on anything unproven.
[ -n "$OUT" ] || refuse "OUT unset"; mkdir -p "$OUT" 2>/dev/null || refuse "cannot create OUT"; R="$OUT/END_receipt.txt"; : >"$R"
SRC="${SRC:-}"; [ -n "$SRC" ] || refuse "SRC unset"
BOOKED="${BOOKED_MEM_BYTES:-}"; [[ "$BOOKED" =~ ^[0-9]+$ ]] || refuse "BOOKED_MEM_BYTES unset or not numeric"
PRIV="${PRIVATE_ROOT:-}"; [ -n "$PRIV" ] || refuse "PRIVATE_ROOT unset"
CH="${CARGO_HOME:-}"; [ -n "$CH" ] || refuse "CARGO_HOME unset"
CHR="$(realpath -m "$CH")"; PRR="$(realpath -m "$PRIV")"
[ "$CHR" != "$(realpath -m "${HOME:-/nonexistent}/.cargo")" ] || refuse "CARGO_HOME is \$HOME/.cargo"
case "$CHR/" in "$PRR"/*) ;; *) refuse "CARGO_HOME not under PRIVATE_ROOT" ;; esac
CG="/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup)"
MEMMAX="$(cat "$CG/memory.max" 2>/dev/null || echo missing)"
[[ "$MEMMAX" =~ ^[0-9]+$ ]] || refuse "no numeric cgroup memory.max ($MEMMAX)"
[ "$MEMMAX" -le "$BOOKED" ] || refuse "memory.max $MEMMAX exceeds BOOKED_MEM_BYTES $BOOKED"
[ "$(cat "$CG/memory.swap.max" 2>/dev/null || echo missing)" = 0 ] || refuse "MemorySwapMax is not 0"
PIDSMAX="$(cat "$CG/pids.max" 2>/dev/null || echo missing)"; { [ "$PIDSMAX" != max ] && [ "$PIDSMAX" != missing ]; } || refuse "pids.max is $PIDSMAX"
OOM0="$(awk '/^oom_kill /{print $2}' "$CG/memory.events" 2>/dev/null)"; [ -n "$OOM0" ] || refuse "memory.events unreadable"
NETIFS="$(awk -F: 'NR>2{gsub(/ /,"",$1); print $1}' /proc/net/dev 2>/dev/null)" || refuse "/proc/net/dev unreadable"
[ -n "$NETIFS" ] || refuse "/proc/net/dev lists no interfaces"
for i in $NETIFS; do [ "$i" = lo ] || refuse "network interface $i present (private network namespace not established)"; done
cd "$SRC" || refuse "cannot cd SRC"
g() { timeout -k 2 10 git "$@"; }
[ "$(g rev-parse HEAD)" = "$HEAD_FULL" ] || refuse "HEAD != $HEAD_FULL"
[ "$(g rev-parse 'HEAD^{tree}')" = "$TREE_FULL" ] || refuse "tree != $TREE_FULL"
[ -z "$(g status --porcelain --ignored=no)" ] || refuse "SRC is dirty"
[ "$(g rev-parse HEAD:$WITNESS)" = "$WITNESS_BLOB" ] || refuse "witness blob differs"
[ "$(g rev-parse HEAD:src/v1/stage0/src/bin/claim_batch.rs)" = "$BATCH_SRC_BLOB" ] || refuse "claim_batch.rs blob differs"
[ "$(g rev-parse HEAD:Cargo.lock)" = "$LOCK_BLOB" ] || refuse "Cargo.lock blob differs"
MERGE_BASE=a34217368e9f2241f4ee76e9936f9f7bb4299e0e
[ "$(g rev-parse HEAD^1)" = "$MERGE_BASE" ] || [ "$(g merge-base HEAD "$MERGE_BASE")" = "$MERGE_BASE" ] || refuse "merge-base $MERGE_BASE not an ancestor"
{ echo "head=$HEAD_FULL tree=$TREE_FULL merge_base=$MERGE_BASE"; for p in dag src/v2 src/v1 rust-toolchain.toml Cargo.lock; do echo "$p $(g rev-parse HEAD:$p)"; done; } >"$OUT/pins.txt"
export RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true CARGO_HOME="$CHR" CARGO_TARGET_DIR="$OUT/target"
CHAN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)"; [ -n "$CHAN" ] || refuse "no pinned channel"
timeout -k 2 10 rustup toolchain list >"$OUT/toolchains.txt" 2>/dev/null; grep "^$CHAN" "$OUT/toolchains.txt" >/dev/null || refuse "pinned toolchain $CHAN not installed (never installed here)"
timeout -k 2 10 rustc -vV >>"$OUT/pins.txt" 2>&1 || refuse "rustc -vV failed"; cat "$OUT/pins.txt" >>"$R"

[ "$SECONDS" -le 180 ] || refuse "preflight exceeded 180s (took ${SECONDS}s)"
# ---- P1 build (12m+20s; offline, locked). Any nonzero rc, incl. a dependency/toolchain cache miss or timeout, is NO_OBSERVATION.
phase 720 "$OUT/build.log" "$OUT/build.err" cargo build --release --offline --locked -p v1-compiler --bin claim_batch; BRC=$PRC
[ $BRC -eq 0 ] || refuse "build_failed rc=$BRC (build.log kept; cache miss, compile error and timeout are not distinguished and none is a claim verdict)"
BIN="$CARGO_TARGET_DIR/release/claim_batch"
timeout -k 2 60 sha256sum "$BIN" | tee "$OUT/bin.sha256.before" >>"$R"

# ---- (no separate closure gate, rev7) claim_batch --claim-run runs the same process_shared_index_or_refuse index, closure and resolve
#   first; a failure there exits non-zero with no per-function verdict line, which the roster check below reads as NO_OBSERVATION.
#   closure.txt was never read by any check; the run's stderr (claim.stderr, [process-partition] timings) replaces closure.err.

# ---- P3 claim run (28m20s+20s, hermetic). Observed claim lines are classified per claim from claim_batch's own typed lines.
phase 1700 "$OUT/claim.stdout" "$OUT/claim.stderr" "$BIN" --claim-run --hermetic --source-root dag --source-root src/v2 --entry "$WITNESS" --functions "$FUNCTIONS" --outcomes-out "$OUT/outcomes.json"; CLAIM_RC=$PRC
timeout -k 2 60 sha256sum "$BIN" | tee "$OUT/bin.sha256.after" >>"$R"
say "memory.peak_whole_script_incl_build=$(cat "$CG/memory.peak" 2>/dev/null || echo unreadable) claim_rc=$CLAIM_RC"
OOM1="$(awk '/^oom_kill /{print $2}' "$CG/memory.events")"
[ "$OOM1" = "$OOM0" ] || refuse "oom_kill recorded ($OOM0 -> $OOM1)"
[ $CLAIM_RC -lt 124 ] || refuse "claim_run_timeout_or_killed rc=$CLAIM_RC"
[ "$(awk '{print $1}' "$OUT/bin.sha256.before")" = "$(awk '{print $1}' "$OUT/bin.sha256.after")" ] || refuse "binary_changed_during_run"
grep -hvE '^(PASS|FAIL|UNOBSERVED) ' "$OUT/claim.stderr" "$OUT/claim.stdout" >"$OUT/diagnostics.txt"
grep -hE '^(PASS|FAIL|UNOBSERVED) [A-Za-z0-9_]+' "$OUT/claim.stdout" "$OUT/claim.stderr" | sort >"$OUT/verdicts.txt"
say "-- VERDICTS (verbatim)"; cat "$OUT/verdicts.txt" >>"$R"; cat "$OUT/verdicts.txt"
say "-- DIAGNOSTICS (verbatim, first 40)"; head -40 "$OUT/diagnostics.txt" | tee -a "$R"

# --outcomes-out writes the TSV terminal ledger "gunbc-terminal-ledger/2" (v2.workflow.floor_terminal_ledger_wire render_row:
#   <module>.<function> TAB expect-hold|expect-red TAB <terminal tag> TAB <detail>); Pass -> returned-true, Fail -> returned-false
#   (derive_outcome_terminal). File name kept as given to --outcomes-out; content is that ledger, not JSON.
LEDGER="$OUT/outcomes.json"
[ -s "$LEDGER" ] || refuse "outcomes ledger missing or empty (claim_rc=$CLAIM_RC)"
say "outcomes ledger sha256=$(timeout -k 2 60 sha256sum "$LEDGER" | awk '{print $1}')"
# ---- classification, typed only. A cell's FAIL is DATA. Completeness is the only verdict.
STRAY="$(awk '{print $2}' "$OUT/verdicts.txt" | grep -vxF -f <(tr ' ' '\n' <<<"$ROSTER" | grep .) | tr '\n' ' ')"
[ -z "$STRAY" ] || finish NO_OBSERVATION 2 "verdict lines for names outside the roster: $STRAY"
PASSED=0; FAILED=0; NOOBS=""
for c in $ROSTER; do
  n=$(grep -cE "^(PASS|FAIL|UNOBSERVED) $c( |\$)" "$OUT/verdicts.txt")
  if [ "$n" != 1 ]; then NOOBS="$NOOBS $c(roster:$n lines)"; continue; fi
  # DATA is exactly `PASS <name>` or `FAIL <name>` (claim_batch report_outcome: ClaimOutcome::Pass / ::Fail print the bare
  # name). ANY suffix (runtime error, budget, ExitFailure code, not-Bool, not-attempted, closure diagnostic) or UNOBSERVED
  # is NO_OBSERVATION for that cell.
  if grep -qxF "PASS $c" "$OUT/verdicts.txt"; then PASSED=$((PASSED+1))
  elif grep -qxF "FAIL $c" "$OUT/verdicts.txt"; then FAILED=$((FAILED+1))
  else NOOBS="$NOOBS $c(not_a_bare_verdict: $(grep -E "^(PASS|FAIL|UNOBSERVED) $c( |\$)" "$OUT/verdicts.txt" | head -c 200))"; fi
done
# ---- ledger cross-check: every roster name exactly once, value agreeing with its bare verdict line; no extra rows.
awk -F'\t' 'NF>=3 && ($2=="expect-hold" || $2=="expect-red") {print $1 "\t" $3}' "$LEDGER" >"$OUT/ledger_rows.tsv"
LDIS=""
for c in $ROSTER; do
  m=$(awk -F'\t' -v c="$c" '{n=split($1,a,"."); if (a[n]==c) print $2}' "$OUT/ledger_rows.tsv")
  k=$(printf '%s' "$m" | grep -c .)
  if [ "$k" != 1 ]; then LDIS="$LDIS $c(ledger_rows:$k)"; continue; fi
  if grep -qxF "PASS $c" "$OUT/verdicts.txt"; then want=returned-true; elif grep -qxF "FAIL $c" "$OUT/verdicts.txt"; then want=returned-false; else want=none; fi
  [ "$m" = "$want" ] || LDIS="$LDIS $c(ledger:$m verdict:$want)"
done
LEXTRA="$(awk -F'\t' '{n=split($1,a,"."); print a[n]}' "$OUT/ledger_rows.tsv" | grep -vxF -f <(tr ' ' '\n' <<<"$ROSTER" | grep .) | tr '\n' ' ')"
[ -z "$LEXTRA" ] || LDIS="$LDIS extra_ledger_rows:$LEXTRA"
[ -z "$LDIS" ] || finish NO_OBSERVATION 2 "ledger_disagrees_with_verdicts:$LDIS"
[ "$SECONDS" -le "$DEADLINE_S" ] || finish NO_OBSERVATION 2 "aggregate_deadline exceeded during collection (${SECONDS}s)"
[ -z "$NOOBS" ] || finish NO_OBSERVATION 2 "table_incomplete no_observation:$NOOBS; pass=$PASSED fail=$FAILED; claim_rc=$CLAIM_RC"
[ "$CLAIM_RC" -le 1 ] || finish NO_OBSERVATION 2 "claim_rc=$CLAIM_RC is neither 0 nor 1 with a complete table; pass=$PASSED fail=$FAILED"
finish OBSERVED 0 "table_complete 42/42 pass=$PASSED fail=$FAILED claim_rc=$CLAIM_RC (FAIL cells are data; table in VERDICTS)"
