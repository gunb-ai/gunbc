# ---- arg check + exclusive OUT (v17 semantics, verbatim) ----
# B1: the OUTPUT dir is the one argument (the launcher maps it to <R>/out). It must be ABSENT in every form: no dir (even empty), file or symlink (dangling included).
[ "$#" = 1 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir>"
if out_reusable "$1"; then die2 "REFUSED_SETUP: output path $1 already exists (dir, file or symlink); reuse is refused"; fi
mkdir -- "$1" || die2 "mkdir $1 (exclusive, no -p: an entry that appeared since the absence check is refused)"
cd "$1" || die2 "cd $1"
R="$PWD"

# ---- ROSTERS: FULL declared lists (v2, parent ruling msg_2f4cf9de: no function-narrowing). Names generated from each file's declared order at 495da688
# ---- and re-gated per leg as an in-order single-occurrence subsequence of the declared list AT EACH PIN (identical lists at both pins; blobs gated). ----
RTP="a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand a_return_followed_by_a_statement_refuses_as_not_in_tail_position a_return_before_another_statement_in_a_fn_value_refuses an_else_less_if_that_falls_through_still_refuses_as_unlowered an_else_less_if_as_the_last_statement_still_refuses_as_unlowered a_guard_in_a_non_tail_spine_still_refuses_as_unlowered a_tail_return_of_a_fn_body_is_accepted a_tail_return_after_a_let_is_accepted a_tail_return_of_a_fn_value_block_body_is_accepted a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom a_return_in_an_arm_of_a_tail_if_is_accepted a_guard_followed_by_statements_is_accepted a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom a_return_in_an_arm_of_a_tail_match_is_accepted a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted"
CE="an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds two_files_declaring_one_module_refuse_the_emission_at_both_files_holds an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds an_importer_over_a_module_the_closure_holds_is_emitted_holds an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds an_emitted_row_carries_no_diagnostic_holds"
WC="a_match_exhaustive_through_a_wildcard_arm_resolves a_wildcard_in_a_constructor_field_resolves a_named_binder_arm_still_binds an_undeclared_name_in_a_wildcard_arm_body_is_the_sole_refusal_at_its_atom"
[ "$(echo $RTP | wc -w) $(echo $CE | wc -w) $(echo $WC | wc -w)" = "17 13 4" ] || die2 "REFUSED_SETUP: rosters are not 3/2/4"
# entry table: tag:path:pinned-blob (blobs verified identical at BOTH pins; still gated per leg after each checkout)
ENTRIES="rtp:src/v2/test/claim/namespace_xl0/return_tail_position.dag:514c1d32f10dff96aee9298bc20aea1a2c6e1583 ce:src/v2/test/claim/self_host/closure_emission.dag:ee1c679e7ef1e2923edadfe0a6f0c63cf881b019 wc:src/v2/test/claim/namespace_xl0/wildcard_arm_resolve.dag:aba6d2be15f1510629d7d8f8d47267e162bf2047"

# ---- PRE-REGISTERED EXPECTATIONS (intended semantics, written 2026-11-06 before this packet ever runs; NEVER edited after output) ----
# Key: <leg>.<tag>.<fn>. 68 rows. The B column is the v17 prediction table (all PASS) plus the wc main-green controls; the A column equals B except the
# three FIX witnesses (rtp fallthrough with a one-name brace 'if c { i }'; the ce pair on specimen 'if p { p }'), which are expected FAIL on A because
# pre-fix the record arm of the ident suffix reads the one-name brace and the named lowering refusal never fires. See the header for the per-source reasoning.
declare -A EXPECT
EXPECT["A.rtp.a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand"]=PASS
EXPECT["A.rtp.a_return_followed_by_a_statement_refuses_as_not_in_tail_position"]=PASS
EXPECT["A.rtp.a_return_before_another_statement_in_a_fn_value_refuses"]=PASS
EXPECT["A.rtp.an_else_less_if_that_falls_through_still_refuses_as_unlowered"]=FAIL
EXPECT["A.rtp.an_else_less_if_as_the_last_statement_still_refuses_as_unlowered"]=PASS
EXPECT["A.rtp.a_guard_in_a_non_tail_spine_still_refuses_as_unlowered"]=PASS
EXPECT["A.rtp.a_tail_return_of_a_fn_body_is_accepted"]=PASS
EXPECT["A.rtp.a_tail_return_after_a_let_is_accepted"]=PASS
EXPECT["A.rtp.a_tail_return_of_a_fn_value_block_body_is_accepted"]=PASS
EXPECT["A.rtp.a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom"]=PASS
EXPECT["A.rtp.a_return_in_an_arm_of_a_tail_if_is_accepted"]=PASS
EXPECT["A.rtp.a_guard_followed_by_statements_is_accepted"]=PASS
EXPECT["A.rtp.a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom"]=PASS
EXPECT["A.rtp.the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom"]=PASS
EXPECT["A.rtp.a_return_in_an_arm_of_a_tail_match_is_accepted"]=PASS
EXPECT["A.rtp.a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom"]=PASS
EXPECT["A.rtp.a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted"]=PASS
EXPECT["A.ce.an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds"]=PASS
EXPECT["A.ce.a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds"]=PASS
EXPECT["A.ce.two_files_declaring_one_module_refuse_the_emission_at_both_files_holds"]=PASS
EXPECT["A.ce.an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds"]=PASS
EXPECT["A.ce.a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds"]=FAIL
EXPECT["A.ce.the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds"]=FAIL
EXPECT["A.ce.a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds"]=PASS
EXPECT["A.ce.an_importer_over_a_module_the_closure_holds_is_emitted_holds"]=PASS
EXPECT["A.ce.an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds"]=PASS
EXPECT["A.ce.the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds"]=PASS
EXPECT["A.ce.a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds"]=PASS
EXPECT["A.ce.a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds"]=PASS
EXPECT["A.ce.an_emitted_row_carries_no_diagnostic_holds"]=PASS
EXPECT["A.wc.a_match_exhaustive_through_a_wildcard_arm_resolves"]=PASS
EXPECT["A.wc.a_wildcard_in_a_constructor_field_resolves"]=PASS
EXPECT["A.wc.a_named_binder_arm_still_binds"]=PASS
EXPECT["A.wc.an_undeclared_name_in_a_wildcard_arm_body_is_the_sole_refusal_at_its_atom"]=PASS
EXPECT["B.rtp.a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand"]=PASS
EXPECT["B.rtp.a_return_followed_by_a_statement_refuses_as_not_in_tail_position"]=PASS
EXPECT["B.rtp.a_return_before_another_statement_in_a_fn_value_refuses"]=PASS
EXPECT["B.rtp.an_else_less_if_that_falls_through_still_refuses_as_unlowered"]=PASS
EXPECT["B.rtp.an_else_less_if_as_the_last_statement_still_refuses_as_unlowered"]=PASS
EXPECT["B.rtp.a_guard_in_a_non_tail_spine_still_refuses_as_unlowered"]=PASS
EXPECT["B.rtp.a_tail_return_of_a_fn_body_is_accepted"]=PASS
EXPECT["B.rtp.a_tail_return_after_a_let_is_accepted"]=PASS
EXPECT["B.rtp.a_tail_return_of_a_fn_value_block_body_is_accepted"]=PASS
EXPECT["B.rtp.a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom"]=PASS
EXPECT["B.rtp.a_return_in_an_arm_of_a_tail_if_is_accepted"]=PASS
EXPECT["B.rtp.a_guard_followed_by_statements_is_accepted"]=PASS
EXPECT["B.rtp.a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom"]=PASS
EXPECT["B.rtp.the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom"]=PASS
EXPECT["B.rtp.a_return_in_an_arm_of_a_tail_match_is_accepted"]=PASS
EXPECT["B.rtp.a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom"]=PASS
EXPECT["B.rtp.a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted"]=PASS
EXPECT["B.ce.an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds"]=PASS
EXPECT["B.ce.a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds"]=PASS
EXPECT["B.ce.two_files_declaring_one_module_refuse_the_emission_at_both_files_holds"]=PASS
EXPECT["B.ce.an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds"]=PASS
EXPECT["B.ce.a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds"]=PASS
EXPECT["B.ce.the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds"]=PASS
EXPECT["B.ce.a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds"]=PASS
EXPECT["B.ce.an_importer_over_a_module_the_closure_holds_is_emitted_holds"]=PASS
EXPECT["B.ce.an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds"]=PASS
EXPECT["B.ce.the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds"]=PASS
EXPECT["B.ce.a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds"]=PASS
EXPECT["B.ce.a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds"]=PASS
EXPECT["B.ce.an_emitted_row_carries_no_diagnostic_holds"]=PASS
EXPECT["B.wc.a_match_exhaustive_through_a_wildcard_arm_resolves"]=PASS
EXPECT["B.wc.a_wildcard_in_a_constructor_field_resolves"]=PASS
EXPECT["B.wc.a_named_binder_arm_still_binds"]=PASS
EXPECT["B.wc.an_undeclared_name_in_a_wildcard_arm_body_is_the_sole_refusal_at_its_atom"]=PASS

declare -A OBS
MISSING=0
DEVIATIONS=0

# roster subset gate: every roster name appears EXACTLY ONCE in the file's declared 'test fn' list, and the roster order matches the declared order
roster_gate() { # $1 = file, $2... = roster names
  local f="$1"; shift
  local -a declared=(); local nm w d j found cnt
  while read -r nm; do declared+=("$nm"); done < <(grep -oE '^[[:space:]]*(pub )?test fn [a-z0-9_]+' "$f" | awk '{print $NF}')
  for w in "$@"; do
    cnt=0; for d in "${declared[@]}"; do [ "$d" = "$w" ] && cnt=$((cnt+1)); done
    [ "$cnt" = 1 ] || return 1
  done
  j=0
  for w in "$@"; do
    found=0
    while [ "$j" -lt "${#declared[@]}" ]; do
      if [ "${declared[$j]}" = "$w" ]; then found=1; j=$((j+1)); break; fi
      j=$((j+1))
    done
    [ "$found" = 1 ] || return 1
  done
  return 0
}

run_entry() { # LEG BIN TAG PATH FUNCS(csv) : one claim_batch invocation; every row contract-checked with v17's verdict classifier, then rc vs THIS entry's verdicts
  local LEG="$1" BIN="$2" T="$3" F="$4" FUNCS="$5"
  local L="$R/claims-$LEG-$T.log" RC VE VD N
  snap "run $LEG $T"
  timeout 900 "$BIN" --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$F" --functions "$FUNCS" > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[$LEG $T]=$RC"; mem "after_run_$LEG-$T"; judge_oom "run $LEG $T"
  rc_in_contract "$RC" || die2 "leg $LEG entry $T: exit status $RC is out of contract (0/1/2 only; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg $LEG entry $T: no single '[process-partition] total_ms=<digits>' completion line; batch incomplete whatever verdict lines printed"
  echo "verdict lines ($LEG $T):"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"
  VE=0
  for N in $(echo "$FUNCS" | tr ',' ' '); do
    VD="$(verdict "$L" "$N")"; echo "$LEG $T $N=$VD"
    if [ "$VD" = NONE ]; then MISSING=1; die2 "leg $LEG entry $T: '$N' has no SINGLE BARE PASS/FAIL verdict (absent, duplicate, conflicted, reasoned, UNOBSERVED or PANICKED)"; fi
    [ "$VD" = FAIL ] && VE=1
    OBS["$LEG.$T.$N"]="$VD"
    if [ "$VD" != "${EXPECT["$LEG.$T.$N"]:-NONE}" ]; then DEVIATIONS=$((DEVIATIONS+1)); echo "DEVIATION leg $LEG entry $T fn $N: observed $VD, pre-registered ${EXPECT["$LEG.$T.$N"]:-NONE}"; fi
  done
  [ "$RC" = "$VE" ] || die2 "leg $LEG entry $T: claim_batch returned $RC, its own verdict lines say $VE"
}

run_leg() { # LEG PIN TREE : fetch (600), gates, build (2400, own target dir), 3 entries (900 each), post-checks
  local LEG="$1" PIN="$2" TREE="$3"
  local HT E T F B BIN BIN_SHA RC VD N
  git init -q "wt-$LEG" && cd "wt-$LEG" || die2 "git init wt-$LEG"
  timeout 600 git fetch -q --depth 1 https://github.com/gunb-ai/gunbc "$PIN" || die2 "leg $LEG: fetch $PIN failed or timed out (600 s cap)"
  git checkout -q FETCH_HEAD || die2 "leg $LEG: checkout failed"
  [ "$(git rev-parse HEAD)" = "$PIN" ] || die2 "leg $LEG: HEAD is not the pin"
  HT="$(git rev-parse 'HEAD^{tree}')" || die2 "leg $LEG: cannot read the tree of HEAD"
  [ "$HT" = "$TREE" ] || die2 "leg $LEG: REFUSED_SETUP: tree is $HT, not the pinned $TREE"
  tree_clean . || die2 "leg $LEG: REFUSED_SETUP: checkout is not clean"
  echo "leg $LEG pin ok head=$PIN tree=$HT"
  for E in $ENTRIES; do IFS=: read -r T F B <<< "$E"
    local HB; HB=$(git hash-object "$F" 2>/dev/null) || die2 "leg $LEG: cannot hash $F"
    [ "$HB" = "$B" ] || die2 "leg $LEG: $F blob is $HB, not the pinned $B"
    local V; case "$T" in rtp) V=RTP;; ce) V=CE;; wc) V=WC;; esac
    roster_gate "$F" ${!V} || die2 "leg $LEG: roster for $T is not an in-order single-occurrence subset of the declared 'test fn' list in $F"
  done
  snap "build-$LEG"
  CARGO_TARGET_DIR="$R/target-$LEG" CARGO_BUILD_JOBS=32 timeout 2400 cargo build -j 32 --locked --release -p v1-compiler --bin claim_batch > "$R/build-$LEG.log" 2>&1
  RC=$?; judge_oom "build-$LEG"
  [ "$RC" = 0 ] || die2 "leg $LEG: build failed rc=$RC (see build-$LEG.log)"
  BIN="$R/target-$LEG/release/claim_batch"
  BIN_SHA="$(hash_of "$BIN")" || die2 "leg $LEG: cannot hash $BIN"
  echo "leg $LEG claim_batch sha256=$BIN_SHA"
  for E in $ENTRIES; do IFS=: read -r T F B <<< "$E"
    local V; case "$T" in rtp) V=RTP;; ce) V=CE;; wc) V=WC;; esac
    run_entry "$LEG" "$BIN" "$T" "$F" "$(echo ${!V} | tr ' ' ',')"
  done
  tree_clean . || die2 "leg $LEG: post-run git status --porcelain is not empty"
  local BIN_NOW; BIN_NOW="$(hash_of "$BIN")" || die2 "leg $LEG: post-run cannot hash the binary"; [ "$BIN_NOW" = "$BIN_SHA" ] || die2 "leg $LEG: claim_batch binary sha changed during the runs"
  cd "$R" || die2 "leg $LEG: cd back to OUT"
}

mem start
run_leg A "$PIN_A" "$TREE_A"
run_leg B "$PIN_B" "$TREE_B"

# ---- observed matrix, deviation count, conclusions (every rule that FIRES is printed; rules written WITH the expectations, before the run) ----
echo "OBSERVED:"
for k in "${!OBS[@]}"; do echo "  $k = ${OBS[$k]} (expected ${EXPECT[$k]})"; done | sort
g() { echo "${OBS[$1]:-NONE}"; }
WA_OK=1; WB_OK=1; for f in $WC; do [ "$(g "A.wc.$f")" = PASS ] || WA_OK=0; [ "$(g "B.wc.$f")" = PASS ] || WB_OK=0; done
PA="$(g A.rtp.a_tail_return_of_a_fn_body_is_accepted)"; PB="$(g B.rtp.a_tail_return_of_a_fn_body_is_accepted)"
UA="$(g A.rtp.a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom)"; UB="$(g B.rtp.a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom)"
FA="$(g A.rtp.an_else_less_if_that_falls_through_still_refuses_as_unlowered)"; FB="$(g B.rtp.an_else_less_if_that_falls_through_still_refuses_as_unlowered)"
C1A="$(g A.ce.a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds)"; C1B="$(g B.ce.a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds)"
C2A="$(g A.ce.the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds)"; C2B="$(g B.ce.the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds)"
CONC=()
[ "$PB" = PASS ] && CONC+=("R11 V17_RTP_DID_NOT_REPRODUCE")
[ "$C1B" = PASS ] && [ "$C2B" = PASS ] && CONC+=("R12 V17_CE_DID_NOT_REPRODUCE")
[ "$WA_OK" = 0 ] && CONC+=("R1 BASE_ROUTE_BROKEN")
[ "$WA_OK" = 1 ] && [ "$WB_OK" = 0 ] && CONC+=("R2 PR_BREAKS_ORDINARY_RESOLVE")
if [ "$WA_OK" = 1 ] && [ "$WB_OK" = 1 ]; then
  [ "$PA" = PASS ] && [ "$PB" = FAIL ] && CONC+=("R3 PR_BREAKS_TAIL_ACCEPTANCE")
  [ "$PA" = FAIL ] && [ "$PB" = FAIL ] && CONC+=("R4 BASE_BREAKS_TAIL_ACCEPTANCE")
  [ "$PA" = FAIL ] && [ "$PB" = PASS ] && CONC+=("R5 BASE_BROKE_ACCEPTANCE_PR_FIXES")
  if [ "$PA" = PASS ] && [ "$PB" = PASS ]; then
    [ "$UA" = FAIL ] && CONC+=("R6 UNBOUND_REPORTING_BROKEN:A")
    [ "$UB" = FAIL ] && CONC+=("R6 UNBOUND_REPORTING_BROKEN:B")
  fi
  [ "$FA" = PASS ] && CONC+=("R7 FALLTHROUGH_NOT_FIX_SPECIFIC")
  [ "$FA" = FAIL ] && [ "$FB" = FAIL ] && CONC+=("R8 PR_INCOMPLETE_ELSE_LESS_IF_REFUSAL")
fi
{ [ "$C1A" = PASS ] || [ "$C2A" = PASS ]; } && CONC+=("R9 CE_CONTRACT_NOTE_CONTRADICTED")
[ "$WA_OK" = 1 ] && [ "$WB_OK" = 1 ] && [ "$C1A" = FAIL ] && [ "$C2A" = FAIL ] && { [ "$C1B" = FAIL ] || [ "$C2B" = FAIL ]; } && CONC+=("R10 CE_FIX_INCOMPLETE")
echo "CONCLUSIONS:"; if [ "${#CONC[@]}" = 0 ]; then echo "  (none fired)"; else for c in "${CONC[@]}"; do echo "  $c"; done; fi
[ "${#OBS[@]}" = 68 ] || die2 "not every expected witness has an observed verdict (${#OBS[@]} of 68)"
[ "$MISSING" -gt 0 ] && die2 "an expected witness has no observed verdict"
echo "JOIN_MISSING=0 FAILS=$DEVIATIONS"
echo DONE
[ "$DEVIATIONS" -gt 0 ] && exit 1
exit 0
