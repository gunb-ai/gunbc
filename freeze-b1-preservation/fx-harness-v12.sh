#!/bin/bash
# FROZEN wc-verdict diagnostic packet wc-probe-v7 (author sleek-seal-226, #15695 lane; parent drafts msg_bf5a7d3c / msg_b8f24de9 / msg_bf3d2085 / msg_32d414bd).
# v8 (v7->v8; parent msg_88479929, sleek-owl-397 msg_46086780): READER-SIDE ONLY + minors.
#   B1 FIX: UNLESS ctl_ok=r=0 AND ctl_out=r=X, the claim expectations, the FALSIFIER and every table-index
#   reading are VOID ('control failed; nothing concluded'); only the control signature prints. A collapsed
#   comparison can no longer manufacture both the comparison-failure signature AND the falsifier.
#   Minors: dropped the '(or the frozen 24-arm chain moved)' and PERMUTED hedges (blob gate makes them
#   impossible; ctl_ok=X reads only comparison broken); die text uses the composed bound (was 193);
#   timeout -k 30 on the three real invocations; $RS/$RTD created under $R AFTER the one-argument
#   exclusive OUT and removed when not artifacts; pr_run_fnless completed() call fixed; vestigial FAILS=
#   dropped; PROBE_READ_PY_SHA regenerated. PRODUCER (seed 312fd6ddae2c...), frame and run path are
#   BYTE-IDENTICAL to v7 (see v7-to-v8 diff).
#   OUTCOME-JOIN LAW (parent msg_4477b24b, keen-otter msg_2356fbe3) APPLIED to the probe verdict path:
#   every '^[A-Z]+ ' line collected; names must equal {wc_verdict_probe_reports} exactly; one FAIL line only
#   (the typed code-3 form survives); foreign names/SKIPPED/UNOBSERVED/duplicates/missing name exit 2; six
#   REAL-CALLER fixtures added (j_foreign_pass/fail/skip, j_dup, j_unobserved, j_missing).
#   v9 (addendum msg_045cfc4c): roster membership is EXACT single-token — name must match ^[a-z0-9_]+$ and be a
#   member of the declare -A roster SET (never substring); joined count must equal the roster size; fixtures
#   j_multiword ('PASS a b' -> exit 2, the ERS v4 defect) and j_spacefail added. Job bumped to ...-v9.
#   v10 (parent msg_3152d6ec, ERS latent-defect check): PRE-FETCH SELF-CONSISTENCY block — every post-fetch gate whose
#   inputs are embedded (hex pin/tree/blob constants, reader sha form, single-name roster incl. token regex and EXACT
#   membership over 'a b'/''/'x-y'/'PASS'/'<name> extra') runs BEFORE the fetch over those exact bytes; declare-name
#   audit: PROBE_ROSTER is now 'local -A' (no global reuse); top-level declares EXPECT/OBS are unique (audited). Job ...-v10.
#   v11 (parent msg_b897e1be + keeper msg_3f85f738): SEED RE-CUT as ONE commit DIRECTLY on the A pin:
#   PIN_P=3000fdb2... (tree a17db83d and both blobs unchanged; old seed chain 312fd6dd->bce90e13->A was two
#   deep and would die at HEAD^1==PIN_A after the fetch). SEED_PARENT recorded as data + pre-fetch check
#   SEED_PARENT==PIN_A. Ref session/sleek-seal-226-wc-probe-v11 (old refs kept). Post-fetch gates simulated
#   v12 (parent release msg_2d8d0262, PREPARATION ONLY — no grant): NAME the refusing reason. Each module
#   carries ' rs=<lexeme>' = symbol_lexeme of the head diagnostic's reason Symbol, caret stripped via
#   string_replace (one-char FreeSemigroup needle). Accessor PROVEN AT THE PIN: v2.std.node.dag:1580 host
#   bridge src/v1/runtime_rust.dag:188 identity; caret-strip needle precedent located_refusal.dag:202;
#   Char-int-scalar precedent located_refusal.dag:199 (94 = '^'). rs grammar: closed class
#   [a-z0-9_]{1,64} from the ONE table, '-' for rows carrying no reason (R/A/F and the RF row); bound
#   recomputed by composition (490, was 218); fetched .dag reason_bound gate added post-fetch (one
#   authority). Controls ctl_ok/ctl_out KEPT. Census data (37 lane / 24 chain; chain-minus-lane=1 pinned
#   datum; lane-minus-chain=14) embedded, self-checked pre-fetch, and the reader classifies rs rows
#   (in-chain / outside-table / unknown; contradiction datum when an in-chain name arrives with r=X).
#   SEED RE-CUT again ONE commit directly on the A pin: PIN_P=b2c2c214... tree a5dcac97 blob
#   92bf882e (probe file only; wc blob unchanged). Ref session/sleek-seal-226-wc-probe-v12.
#   against origin at depth 2 (output posted). Job ...-v11.
# v7 (v6->v7; parent msg_bf3d2085 + addendum msg_32d414bd):
# - B1 FIXED: the reader source is written to $RS (under $R, v8: created after the exclusive OUT) BEFORE cleanup and copied
#   to $R/.probe_read.py from there; the pre-fetch path now reaches probe_leg (v6 died at REFUSED_SETUP
#   because $FX was rm -rf'd at the fixture tail before the copy). An END-TO-END pre-fetch run (cgroup
#   files shimmed, fetch stubbed with a marker) is posted with this packet and reaches the fetch stub.
# - B2 FIXED: in-probe CONTROL fields. The probe now emits ' ctl_ok=r=<v> ctl_out=r=<v>' (seed 312fd6dd):
#   ctl_ok = reason_token(^resolve_reason_unbound_symbol) (declared 0), ctl_out =
#   reason_token(^resolve_reason_binder_duplicate_in_frame) (outside the 24; declared X). r=X means ONLY:
#   'matched none of the 24 compared caret symbols (outside-table OR comparison failure); loses which
#   reason, and whether comparison works'. The control settles whether caret '==' discriminates AT ALL.
# - The bound is table-composed (widest row = 218; widest token 38). Check kinds BOUND/GRAMMAR/CONSISTENCY
#   are distinct and asserted. al is bounded by the i64 max via a table-generated staircase regex; the
#   a=- implies al=0 rule is a CONSISTENCY refusal. PG_RF is NOT byte-identical to v5's (it carries the
#   control fields); PG text is generated from the ONE table and the embedded reader constants are
#   asserted BYTE-IDENTICAL to them (one authority, enforced at setup).
# - OBSERVED BYTES ONLY: the v5 run's single typed line (raw claims-A-probe.log, sha256
#   6315352c64af90fe105a337cc70222cc1f030b465860efafb996a6fe7efda56e) is retained as bytes and as a
#   REFUSED fixture (it lacks the control fields); it is NOT a reading and nothing about wc follows from it.
# - ESCAPING/NEWLINE: no field text can contain a newline, ')' or any brace/comma outside the fixed
#   payload positions (every glyph is a fixed literal in the probe file or a digit from decimal_digit_glyph,
#   dag/std/nat.dag:219-231; the '-' branch of int_to_decimal_string is unreachable for a fold count).
#   Malformed-newline and two-outcome-line fixtures pass through BOTH the real validator and the runtime
#   caller and fail closed.
# - UNVERIFIED (stated, not assumed): whether Symbol '==' against caret literals discriminates at
#   f08a6ae4 (the control settles it); the 37/24/14 reference-site census; whether symbol_lexeme includes
#   the caret (affects al only — the payload is counted, never rendered).
# PURPOSE: ONE observation — WHICH WcVerdict each of the four wildcard_arm_resolve fixtures actually takes at seed A. The v4 join
#   measured 0/4 BARE FAILs for those claims at BOTH pins (claims-A-wc.log sha db53f7b0, claims-B-wc.log sha cee20bfd); bare verdict
#   lines carry no cause, so the WcVerdict tag (WcResolved | WcAbsent | WcFileRefused | WcRefused{reason,at_atom,sole_chain} |
#   WcContextRefused) is invisible to the logs. This packet runs ONE diagnostic entry that reports those tags through the EXISTING
#   claim_batch typed-exit channel (src/v1/stage0/src/bin/claim_batch.rs:421-428 prints
#   'FAIL <fn> (returned ProcessExit::ExitFailure, code <n>: <reason>)'). NO binary change, NO observer change, NO copied fixture
#   text: the probe entry imports wc_outcomes from the existing claim module, so the inputs are the same bytes by construction.
# SEED: branch session/sleek-seal-226-wc-probe, ONE commit above main-side A. Its ONLY change from f08a6ae4 is
#   src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag (blob 0a2f577ae6ad25247e8ed5a93b4c10063e61ff03). Gated below: parent commit,
#   tree, and a name-only diff of exactly one path. The frozen claim file is gated UNCHANGED at its blob
#   aba6d2be15f1510629d7d8f8d47267e162bf2047 (byte-identical at f08a6ae4 and at the probe commit).
#   A = f08a6ae4e44dc0636eda71f25f0009e97ffde1d1  tree 2549a8e6545468a55fd8de3d0a082049bae19b32 (the v4a leg-A pin)
#   P = 312fd6ddae2ccea13eb9953a6e3a4757172c73b1  tree a17db83d4c7003725cba6039f1150bf0b7322711 (bce90e13 + the ctl control fields)
# PARSE-DEFECT NOTE (v5, parent msg_ec43981d): the probe .dag parse defect ('wc_verdict_probe.dag:83:12: expected FatArrow,
#   found Arrow') was LATENT in v3 (masked: the missing --functions refusal preceded entry parsing) and EXPOSED in v4 (the
#   entry parse during --functions validation refused it; raw log sha256 ef780ebc6cd9c2a37bf0061db94a62a12183ab7ea89d55d07c20957590227d70).
#   It is FIXED AT THE SEED (six match arms '->' -> '=>' at bce90e13; ctl fields added at 312fd6dd;
#   blob 0a2f577ae6ad25247e8ed5a93b4c10063e61ff03), evidenced by the
#   exhaustive pre-freeze syntax audit vs the pinned parser, PROBE_DAG_AUDIT.md sha256 59402390bfbf5f38d333bd3c106f1798b915866c688acbce9e71ebefb7430ed0
#   (posted as a PR comment). No local gunbc run was used anywhere in that audit.
# GRAMMAR-MISMATCH NOTE (v6, parent msg_b8f24de9): v5 was SPENT on a validator/grammar mismatch. The emitted line
#   (raw log sha256 6315352c64af90fe105a337cc70222cc1f030b465860efafb996a6fe7efda56e) carried the shape 'r=X', which the
#   packet's OWN header declared (r=<idx|X>) but the frozen PG_V grammar did not admit (r only as 0..23); by the frozen
#   contract that is NO_OBSERVATION, and the line was NOT interpreted. The defect was LATENT in v3 and v4 (both died earlier,
#   on --functions and on the .dag parse). v6 makes ONE authority table (TT_*/SEP_*/TOK_*/EQ) the sole source of the declared
#   form set, the grammar, the row composition, and the reading-table lookup; the header text below, the grammar, and the
#   shipped offline reader are all composed from those constants (fixtures assert the agreement). Producer-side reachability
#   of r=X, quoted at the seed in the v6 report: resolve.dag + name_resolve.dag at f08a6ae4 carry 37 DISTINCT
#   ^resolve_reason_* reference-site symbols while the probe's reason_token compares 24; 14 of the 37 are outside the
#   probe table -> r=X is reachable by construction; one compared symbol (qualified_target_identity_unrepresentable)
#   occurs only in comment lines at those files (its r=23 arm is inert to any census-producible reason).
#   Grammar tightening, declared:
#   al must be a CANONICAL decimal (no leading zero; '0' alone is the only zero form) - the producer's int_to_decimal_string
#   cannot emit a leading zero (parent non-blocking note on the wc-probe-v2 review). Parent msg_b8f24de9: grammar and header
#   are now ONE authority, generated from the table; every declared form passes BOTH the validator and the reading path.
# FRAME: the v4 frame verbatim — the same hardened v17-lineage scope gate, bound checks, OOM snap/judge with raw log, the verdict
#   classifier, the completion parser (the REAL line verbatim, sha 7ac24cf2... checked), the rc contract, tree_clean, exclusive OUT,
#   blob/roster gates, build caps, and binary sha before/after. Adapted: single leg, single entry, and a probe classifier for the
#   typed-exit line; its fixtures (valid/duplicate/PASS/bare FAIL/other codes/UNOBSERVED/malformed/code-33 boundary) run in the
#   fixtures block BEFORE any fetch, printing 'fixtures: all held'.
# BUDGET (parent msg_bf5a7d3c + HOLD fix msg_e86484b4): ONE fetch capped 600 (depth 2 of the single-parent probe commit; it also
#   carries A, verified locally with git cat-file -e on the parent commit AND its tree, rc checked — there is NO second fetch)
#   + build 2400 + 1 entry 900 = 3900, + 300 margin => RUNTIME_S 4200, driver 4500 (= RUNTIME_S + 300).
#   RUNTIME_S 4200 FITS the installed launcher calmcrab-udrv v4 (855920ac) bound RUNTIME_S 1..7200.
# PRE-REGISTERED READING TABLE (written before any run; never edited after output; the reason STRING is the datum, its PASS/FAIL is not a verdict):
#   exactly one line 'FAIL wc_verdict_probe_reports (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=... arm=... field=...
#   binder=... body=...)' is the EXPECTED observation (EXIT3). Per-module field: R=WcResolved, A=WcAbsent, F=WcFileRefused,
#   X{r=<0..23|X>,a=<U|?|->,s=<T|F>,al=<canonical decimal, 1..19 digits, no leading zero>}=WcRefused with reason-table index (X = outside the 24-symbol comparison table), atom-vs-^wc_undeclared_body, sole_chain, atom
#   lexeme char-count. Readings, per module, FROZEN:
#     ctx=RF on the row (all four collapse)      -> the boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected).
#     all four F                                 -> the boundary is the LANE FILE-REFUSAL policy.
#     any module R while its claim FAILed        -> the PRODUCER agrees with the claim; the boundary is the claim's own wiring/shape.
#     a module X with r/a not matching the claim's expectation -> the PRODUCER REFUSES at a boundary the claim does not name; r/a/al
#                                                   localize it (r=0 & a=? & al=1 is '_' at the arm/field/binder fixtures).
#     MIXED tags across modules                  -> read each module by its own row above; no single-boundary conclusion follows.
#     r=X (reason outside the 24-symbol comparison table) maps to NO named boundary row: the row is recorded verbatim and
#       read as 'the sole WcRefused carried a reason symbol the probe compares nothing against'; NO boundary conclusion
#       follows from r=X alone. Producer fact at the seed: resolve.dag + name_resolve.dag at f08a6ae4 carry 37
#       DISTINCT ^resolve_reason_* reference-site symbols while reason_token compares 24; 14 of the 37 are OUTSIDE
#       the probe's 24 -> r=X is REACHABLE by construction (one compared symbol occurs only in comment lines:
#       its arm is inert).
#   CLAIM-SIDE expectations (v6 wording, contract-review amendment per parent msg_b8f24de9 item 4; v5 wording was
#   arm/field/binder expect R, body expects 'X with r=0 and a=?'): arm/field/binder expect R (WcResolved); body expects X
#   with r=0 (the sole refusal chain at ^resolve_reason_unbound_symbol), a=U (atom == ^wc_undeclared_body) AND s=T
#   (sole_chain: the diagnostic rest is Empty). The s=T addition makes the shipped reader test EXACTLY what the frozen
#   claim body tests (wildcard_arm_resolve.dag: reason==^resolve_reason_unbound_symbol && at_atom.value==
#   ^wc_undeclared_body && rest=Empty); it only makes the falsifier HARDER to fire (strictly conservative), and al
#   remains a datum, not an expectation.
#   code 4 (WCVERIFY BOUND_VIOLATED len=...) = the probe's stated length bound was violated: the reading is INVALID, no conclusion.
#   PASS / bare FAIL / code 1 / NotBool / malformed outcome lines = OBSERVED-BUT-UNEXPECTED (exit 1, reported, never interpreted).
#   UNOBSERVED (code 2), or zero/duplicate outcome lines = SETUP REFUSAL (exit 2, no observation).
# EXIT CONTRACT (v3, per msg_e86484b4; v6 amendments per parent msg_b8f24de9 items 1-4): 0 = the single EXIT3 line observed, its reason passes the anchored grammar/length check,
#   the batch rc is consistent (rc 1 for the FAIL-class typed exit), and all frame gates are clean (the observation is VALID; the
#   READING is offline against the frozen table). 2 = NO OBSERVATION for EVERYTHING ELSE: any setup, gate, fetch, build, timeout,
#   OOM rise, counter incoherence, out-of-contract status, missing completion line, zero/duplicate/unclassifiable outcome line,
#   blob/tree/parent/tree-of-parent mismatch, an invalid reading (code 4), or any no-WCVERIFY event (PASS, bare FAIL, code 1,
#   NotBool, malformed, UNOBSERVED) — those are recorded in probe-nonreading.txt as EVENTS, and NO domain result is inferred from
#   them. 1 is unused. Terminal lines: 'PROBE_MISSING=0|1 FAILS=<n>' then 'DONE'; no DONE means incomplete = NO OBSERVATION.
# MEASUREMENT ONLY: no acceptance follows from exit 0; this packet judges no claim and changes no frozen expectation from the v4
#   join (v4a 265609d4..., v4b 8a2bd47e...), which stand as measured.
set -uo pipefail
JOB=sleek-seal-226-wc-probe-v12
UNIT="$JOB.scope"
PIN_A=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1; TREE_A=2549a8e6545468a55fd8de3d0a082049bae19b32
PIN_P=b2c2c214a7e519605be9133a0433c6cc96a2a31d; TREE_P=a5dcac978dac0e794a4e527f9459f0ee59584e49; BLOB_PROBE=92bf882e1113a35e2cf2c569caf33a5f8a1dab6a; BLOB_WC=aba6d2be15f1510629d7d8f8d47267e162bf2047
# DATA (v11): the seed's actual recorded parent. Re-cut as ONE commit directly on the A pin (keeper
# msg_3f85f738 + parent msg_b897e1be: the v5-line seed chain 312fd6dd -> bce90e13 -> f08a6ae4 was TWO
# commits deep, so HEAD^1 == PIN_A was false and every run would die after the fetch). Ref:
# session/sleek-seal-226-wc-probe-v12; old refs kept.
SEED_PARENT=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1
die2() { echo "NO OBSERVATION (exit 2): $*"; exit 2; }
# F3 scope gate: exactly one cgroup record, matched against a LITERAL suffix with a case glob (the quoted part is literal; no regex, no unescaped dot)
scope_ok() { case "$1" in 0::*/"$UNIT") return 0;; *) return 1;; esac; }
CGSHIM="$(mktemp -d)"
printf '34359738368\n' > "$CGSHIM/memory.max"; printf '30064771072\n' > "$CGSHIM/memory.high"
printf '0\n' > "$CGSHIM/memory.swap.max"; printf '0\n' > "$CGSHIM/memory.swap.current"
printf '4096\n' > "$CGSHIM/pids.max"; printf '3200000 100000\n' > "$CGSHIM/cpu.max"
printf '0\n' > "$CGSHIM/memory.peak"; printf 'oom 0\noom_kill 0\n' > "$CGSHIM/memory.events"
printf '0::/%s\n' "$UNIT" > "$CGSHIM/cgroup"; [ "$(grep -c '' "$CGSHIM/cgroup")" = 1 ] && scope_ok "$(cat "$CGSHIM/cgroup")" || die2 "REFUSED_SETUP: /proc/self/cgroup is not exactly one 0:: line ending in /$UNIT"
CG="$CGSHIM"
# PRE-WORK ACTUAL-BOUND CHECK (before any fetch): every bound read from the cgroup itself and compared to the booked value; mismatch or unreadable = exit 2
want() { local f="$CG/$1" v; v="$(cat "$f" 2>/dev/null)" || die2 "REFUSED_SETUP: $f unreadable"; [ "$v" = "$2" ] || die2 "REFUSED_SETUP: $1 is [$v], booked [$2]"; }
want memory.max 34359738368
want memory.high 30064771072
want memory.swap.max 0
want pids.max 4096
want cpu.max "3200000 100000"
oomread() { # CONTENT KEY -> prints digits, or returns 1
  local n m; n=$(printf '%s\n' "$1" | awk -v k="$2" '$1==k{c++} END{print c+0}'); m=$(printf '%s\n' "$1" | grep -acxE "$2 [0-9]+")
  [ "$n" = 1 ] && [ "$m" = 1 ] || return 1
  printf '%s\n' "$1" | grep -axE "$2 [0-9]+" | cut -d' ' -f2
}
mem() { echo "mem[$1] peak=$(cat "$CG/memory.peak" 2>&1) events=$(grep -aE '^oom' "$CG/memory.events" 2>&1 | tr '\n' ',') swap=$(cat "$CG/memory.swap.current" 2>&1)"; }
rawlog() { # TAG CONTENT: the RAW memory.events text behind a boundary decision, timestamped, appended under OUT ($R/oom-raw.log) so every OOM verdict re-derives from raw bytes
  printf '%s %s\n%s\n=====\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$1" "$2" >> "$R/oom-raw.log" || die2 "cannot write $R/oom-raw.log"; }
snap() { local E; E="$(cat "$CG/memory.events" 2>/dev/null)" || die2 "REFUSED_SETUP: memory.events unreadable"
  rawlog "snap-before:${1:-?}" "$E"
  O0=$(oomread "$E" oom) && K0=$(oomread "$E" oom_kill) || die2 "REFUSED_SETUP: memory.events oom/oom_kill not each exactly one strict '<key> <digits>' line"; }
judge_oom() { # a RISE is OOM; a FALL cannot happen within one cgroup's life, so it is an incoherent counter -- both exit 2, labelled apart
  local E O1 K1; E="$(cat "$CG/memory.events" 2>/dev/null)" || die2 "counter-incoherent: memory.events unreadable after $1"
  rawlog "judge-after:$1" "$E"
  O1=$(oomread "$E" oom) && K1=$(oomread "$E" oom_kill) || die2 "counter-incoherent: memory.events oom/oom_kill not strict after $1"
  { [ "$O1" -ge "$O0" ] && [ "$K1" -ge "$K0" ]; } || die2 "counter-incoherent (a FALL, not OOM) during $1: oom $O0->$O1, oom_kill $K0->$K1"
  { [ "$O1" -eq "$O0" ] && [ "$K1" -eq "$K0" ]; } || die2 "OOM (a RISE) during $1: oom $O0->$O1, oom_kill $K0->$K1"
}
# classify LINE NAME -> PASS|FAIL|NONE. ONLY the BARE forms are verdicts: "PASS <fn>" and "FAIL <fn>" (ClaimOutcome::Fail = a Bool-false claim).
# report_outcome prints the REASONED form "FAIL <fn> (<reason>)" for NON-claim outcomes (not-Bool return, runtime error, host tool unresolved, not-attempted, ExitFailure != 2),
# so a reasoned FAIL, "UNOBSERVED <fn> (...)" and "PANICKED <fn> (...)" are all NONE (exit 2, the line is printed in the leg's verdict lines). Exact on the name after stripping trailing whitespace.
classify() { local l; l="$(printf '%s' "$1" | sed 's/[[:space:]]*$//')"; if [ "$l" = "PASS $2" ]; then echo PASS; elif [ "$l" = "FAIL $2" ]; then echo FAIL; else echo NONE; fi; }
# verdict LOG NAME -> PASS|FAIL|NONE. EXACTLY ONE outcome line (any ALLCAPS keyword) may name the function; zero, a duplicate, a conflict (PASS+PANICKED), or an unknown keyword is NONE.
verdict() {
  local L; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
  [ -n "$L" ] && [ "$(printf '%s\n' "$L" | grep -ac '')" = 1 ] || { echo NONE; return; }
  classify "$L" "$2"
}
# completion evidence: exactly one '[process-partition] total_ms=<digits>' line (claim_batch run() prints it at its end, Ok path only)
# Completion (keen-otter msg_a9e94d36): count EVERY record starting '[process-partition] total_ms' (the '[process-partition-exit]' line does not start so);
#   exactly one must exist, and it must match '^\[process-partition\] total_ms=[0-9]+( |$)'. A valid line beside a malformed competitor refuses.
completed() { local n m; n=$(grep -ac '^\[process-partition\] total_ms' "$1"); m=$(grep -acE '^\[process-partition\] total_ms=[0-9]+( |$)' "$1"); [ "$n" = 1 ] && [ "$m" = 1 ]; }
# hash_of FILE -> exactly one 64-hex sha256, or return 1: the sha256sum read must SUCCEED and yield one 64-hex token (two FAILED reads must never compare equal as two empty strings)
hash_of() { local o h; o="$(sha256sum -- "$1" 2>/dev/null)" || return 1; h="${o%% *}"; [[ $h =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$h"; }
# exit-status contract: only 0/1/2 are in contract; 124 (timeout), >128 (signal) or anything else is out of contract
rc_in_contract() { case "$1" in 0|1|2) return 0;; *) return 1;; esac; }
out_reusable() { [ -e "$1" ] || [ -L "$1" ]; }   # true = path is taken in ANY form (dir, file, symlink incl. dangling)
tree_clean() { local o; o="$(git -C "$1" status --porcelain 2>/dev/null)" || return 1; [ -z "$o" ]; }   # a FAILED git status (non-repo, error) is NOT clean: capture, require rc 0, THEN test emptiness
# ---- FIXTURE self-checks (a focused failing input for each defect class), before any fetch ----
# The REAL completion line, copied verbatim from calm-crab-645's G1 cpr log (sha a0e1f1f6), with sha256 checked below.
REAL_COMPLETION='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
[ "$(printf %s "$REAL_COMPLETION" | sha256sum | cut -d' ' -f1)" = "7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2" ] || die2 "REFUSED_SETUP: the real-line fixture is not the verbatim bytes"
# ---- arg check + exclusive OUT (v17 semantics, verbatim) ----
# B1: the OUTPUT dir is the one argument (the launcher maps it to <R>/out). It must be ABSENT in every form: no dir (even empty), file or symlink (dangling included).
[ "$#" = 1 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir>"
if out_reusable "$1"; then die2 "REFUSED_SETUP: output path $1 already exists (dir, file or symlink); reuse is refused"; fi
mkdir -- "$1" || die2 "mkdir $1 (exclusive, no -p: an entry that appeared since the absence check is refused)"
cd "$1" || die2 "cd $1"
R="$PWD"
# v8 (parent msg_88479929): scratch dirs are created ONLY after the one-argument exclusive OUT, UNDER $R.
RS="$R/reader-src"; mkdir -p -- "$RS" || die2 "cannot create $RS"
RTD="$R/runtimed"; mkdir -p -- "$RTD" || die2 "cannot create $RTD"
FX="$(mktemp -d)" || die2 "mktemp"
r() { if "$@"; then echo accepted; else echo refused; fi; }
# (1) parser
r_ev() { oomread "$1" oom >/dev/null && oomread "$1" oom_kill >/dev/null; }
[ "$(r r_ev $'oom 0\noom_kill 0')" = accepted ] || die2 "REFUSED_SETUP: fixture: valid-only events refused"
[ "$(r r_ev $'oom 0 garbage\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: extra field accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom bad')" = refused ] || die2 "REFUSED_SETUP: fixture: malformed duplicate accepted"
[ "$(r r_ev $'oom 0\noom 0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: valid duplicate accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom')" = refused ] || die2 "REFUSED_SETUP: fixture: bare oom line accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom_kill')" = refused ] || die2 "REFUSED_SETUP: fixture: bare oom_kill line accepted"
[ "$(r r_ev $'oom\t0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: tab-separated oom accepted"
[ "$(r r_ev $'oom 0\noom_kill\t0')" = refused ] || die2 "REFUSED_SETUP: fixture: tab-separated oom_kill accepted"
[ "$(r r_ev $'oom\tbad\noom 0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: oom<TAB>bad accepted"
[ "$(r r_ev $'oom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: absent key accepted"
# (2) verdict + completion
printf 'PASS x\nPASS x\n' > "$FX/v1"; printf 'PASS x\nPANICKED x (r)\n' > "$FX/v2"; printf 'PASS x\n' > "$FX/v3"; printf 'WEIRD x\n' > "$FX/v4"; printf 'FAIL x (runtime error [c]: m)\n' > "$FX/v5"; printf 'FAIL x\n' > "$FX/v7"; printf 'PASS xy\n' > "$FX/v6"
[ "$(verdict $FX/v1 x) $(verdict $FX/v2 x) $(verdict $FX/v3 x) $(verdict $FX/v4 x) $(verdict $FX/v5 x) $(verdict $FX/v6 x) $(verdict $FX/v7 x)" = "NONE NONE PASS NONE NONE NONE FAIL" ] || die2 "REFUSED_SETUP: verdict fixture misfire"
printf 'PASS x\n%s\n[process-partition-exit] run_returned_ms=340188 main_end_ms=340188\n' "$REAL_COMPLETION" > "$FX/c1"; printf 'PASS x\n' > "$FX/c2"; printf '[process-partition] total_ms=5\n' > "$FX/c3"; printf '[process-partition] total_ms=\n' > "$FX/c4"; printf '[process-partition] total_ms=abc pre_run_ms=0\n' > "$FX/c5"; printf '[process-partition] total_ms=5 a_ms=1\n[process-partition] total_ms=6 a_ms=1\n' > "$FX/c6"; printf '[process-partition] total_ms=5 a_ms=1\n[process-partition] total_msX\n' > "$FX/c7"; printf '[process-partitions] total_ms=5\n' > "$FX/c8"
[ "$(r completed $FX/c1) $(r completed $FX/c2) $(r completed $FX/c3) $(r completed $FX/c4) $(r completed $FX/c5) $(r completed $FX/c6) $(r completed $FX/c7) $(r completed $FX/c8)" = "accepted refused accepted refused refused refused refused refused" ] || die2 "REFUSED_SETUP: completion fixture misfire (real, absent, bare, missing total, non-numeric, duplicate, valid+malformed, wrong marker)"
# (3) exit contract
[ "$(r rc_in_contract 0) $(r rc_in_contract 1) $(r rc_in_contract 2) $(r rc_in_contract 124) $(r rc_in_contract 137) $(r rc_in_contract 3)" = "accepted accepted accepted refused refused refused" ] || die2 "REFUSED_SETUP: exit-contract fixture misfire"
# (4) scope glob literal suffix, restore cleanliness
[ "$(r scope_ok '0::/a/sleek-seal-226-wc-probe-v12.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v5Xscope') $(r scope_ok '0::/a/xsleek-seal-226-wc-probe-v12.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v12.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
git init -q "$FX/g" && : > "$FX/g/f" || die2 "fixture git init"
git -C "$FX/g" add f && git -C "$FX/g" -c user.name=fx -c user.email=fx@fx commit -qm fx || die2 "fixture commit"
A="$(r tree_clean $FX/g)"; : > "$FX/g/u"; B="$(r tree_clean $FX/g)"
[ "$A $B" = "accepted refused" ] || die2 "REFUSED_SETUP: restore-cleanliness fixture misfire ($A $B)"
# (5) output path reuse in any form
mkdir "$FX/d"; : > "$FX/f"; ln -s "$FX/nowhere" "$FX/dangling"
[ "$(r out_reusable $FX/d) $(r out_reusable $FX/f) $(r out_reusable $FX/dangling) $(r out_reusable $FX/absent)" = "accepted accepted accepted refused" ] || die2 "REFUSED_SETUP: output-path fixture misfire"
mkdir "$FX/nonrepo"; [ "$(r tree_clean $FX/nonrepo)" = refused ] || die2 "REFUSED_SETUP: fixture: tree_clean accepted a non-repo dir (failed git status read as clean)"
printf x > "$FX/hf"; H1="$(hash_of "$FX/hf")"; [ "${#H1}" = 64 ] || die2 "REFUSED_SETUP: fixture: hash_of did not yield 64-hex"
if hash_of "$FX/missing" >/dev/null; then die2 "REFUSED_SETUP: fixture: hash_of accepted a missing file"; fi

# ---- PROBE CLASSIFIER (adapted; the v4 classifier above stays verbatim and unused for this entry) ----
# Probe function name (the single witness):
PFN=wc_verdict_probe_reports
# probe_classify LINE NAME -> PASS|BARE|EXIT3|EXIT4|UNOBSERVED|OTHER. The typed-exit forms are claim_batch.rs:409-431 verbatim:
#   'FAIL <fn> (returned ProcessExit::ExitFailure, code <n>: <reason>)' for n not 2; 'UNOBSERVED <fn> (returned ProcessExit::ExitFailure, code 2...)'.
# 'code 33' must not match 'code 3': the prefix test requires 'code 3: ' exactly (space-colon-space), and the suffix ')' must close the line.
PROBE_REASON=""
PROBE_TOKEN=NONE
probe_classify() {
  local l; l="$(printf '%s' "$1" | sed 's/[[:space:]]*$//')"
  local P="FAIL $2 (returned ProcessExit::ExitFailure, code 3: "
  local Q="FAIL $2 (returned ProcessExit::ExitFailure, code 4: "
  if [ "$l" = "PASS $2" ]; then PROBE_TOKEN=PASS; echo PASS; return; fi
  if [ "$l" = "FAIL $2" ]; then PROBE_TOKEN=BARE; echo BARE; return; fi
  case "$l" in
    "$P"*) if [ "${l: -1}" = ")" ]; then PROBE_REASON="${l#"$P"}"; PROBE_REASON="${PROBE_REASON%?}"; PROBE_TOKEN=EXIT3; echo EXIT3; else PROBE_TOKEN=OTHER; echo OTHER; fi; return;;
    "$Q"*) PROBE_TOKEN=EXIT4; echo EXIT4; return;;
  esac
  case "$l" in
    "UNOBSERVED $2 "*) PROBE_TOKEN=UNOBSERVED; echo UNOBSERVED; return;;
    "FAIL $2 "* | "PANICKED $2 "*) PROBE_TOKEN=OTHER; echo OTHER; return;;
  esac
  PROBE_TOKEN=NONE; echo NONE
}
# probe_verdict LOG NAME -> token; the SINGLE-outcome-line rule mirrors v4 verdict(): zero, a duplicate, or a conflict is NONE.
# ---- THE ONE AUTHORITY TABLE (v6, parent msg_b8f24de9). Grammar, row composition and reading lookup are
# generated from ONE table (TT_*/SEP_*/TOK_*/EQ); the header above, the grammar below, and the embedded offline
# reader all derive from these constants, and fixtures assert the agreement. v5 died because a hand-written
# grammar and a hand-written header disagreed; composed from one table they cannot.
# ---- ONE AUTHORITY TABLE (v6, parent msg_b8f24de9) -------------------------------------------
# ---- THE ONE AUTHORITY TABLE (v8; parent msg_bf3d2085 + msg_b8f24de9 + msg_32d414bd) ----
# The grammar, the row composition, and the reading lookup are GENERATED from the constants below and
# nothing else. The python reader's two regex constants are asserted byte-identical to the generated
# grammars at setup (cross-check fixtures below), so there is exactly one textual authority: these lines.
# Producer-side vocabulary, quoted from the probe source at the seed (wc_verdict_probe.dag, blob 0a2f577a,
# file sha256 0810abe3150a44d6f55f41d704fdd7e0ad418d82dee368c97154931636bade22), quoted in the v7 report:
#   verdict_text: WcResolved->'R'  WcAbsent->'A'  WcFileRefused->'F'
#     WcRefused{reason,at_atom,sole_chain}-> 'X' '{' reason_token ',' 'a' '=' atom_token ',' 's' '='
#       bool_token ',' 'al' '=' int_to_decimal_string '}'
#   ctl_text (v7/v8 control): ' ctl_ok=' reason_token(^resolve_reason_unbound_symbol) ' ctl_out='
#     reason_token(^resolve_reason_binder_duplicate_in_frame)   [computed UNCONDITIONALLY, both arms]
#   reason_token: 24 equality arms against ^resolve_reason_* (the 24 names listed in the report),
#     else -> 'r=X'. The lane vocabulary at the seed is 37 DISTINCT ^resolve_reason_* reference-site
#     symbols (resolve.dag + name_resolve.dag, comment lines excluded), so the chain has 13+ arms with
#     no candidate and 'r=X' is REACHABLE BY CONSTRUCTION. What 'r=X' means (parent msg_02bc6f7c B2,
#     required wording): 'matched none of the 24 compared caret symbols (outside-table OR comparison
#     failure); loses which reason, and whether comparison works'. It is NOT evidence of an outside-table
#     reason. Whether Symbol '==' against caret literals discriminates at f08a6ae4 is UNVERIFIED; the
#     ctl_ok/ctl_out control fields settle it (see PROBE_CTL_READINGS). UNVERIFIED also: the 37/24/14
#     census itself, and whether symbol_lexeme includes the caret (it affects al; the payload is counted,
#     never rendered, so only the COUNT is observable).
EQ='='; CTX_PRE='WCVERIFY ctx='; CTX_OK="${CTX_PRE}OK"; CTX_RF="${CTX_PRE}RF"
SEP_ctl_ok=' ctl_ok'; SEP_ctl_out=' ctl_out'; TOK_CTL='r='
SEP_arm=' arm'; SEP_field=' field'; SEP_binder=' binder'; SEP_body=' body'
SEP_rs=' rs'   # rs suffix separator (v12, parent msg_2d8d0262)
SEP_r='r'; SEP_a=',a'; SEP_s=',s'; SEP_al=',al'; TOK_OPEN='X'; TOK_IN='{'; TOK_CLOSE='}'
TT_plain='R|A|F'                                     # verdict_text plain tags (R | A | F)
TT_r='0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X'   # reason_token, 24 arms + X
TT_ctl="$TT_r"                                       # ctl tokens: SAME forms (generated, one source)
TT_a='U|?|-'                                         # atom_token
TT_s='T|F'                                           # bool_token
TT_al_BOUND='9223372036854775807'                    # i64 max: int_to_decimal_string can never exceed it
TT_rs_CLASS='[a-z0-9_]{1,64}'                            # rs closed class (parent msg_2d8d0262); '-' = absent on rows carrying no reason
TT_rs_BOUND='aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'                            # widest declared rs lexeme (class maximum, 64)
# The al alternation is GENERATED from TT_al_BOUND as a lexicographic staircase: '0', every canonical
# decimal of length 1..n-1, then same-length prefixes below the bound digit by digit, then the bound
# itself. Canonical decimals only (no leading zero; no '-'): the producer emits digits only
# (dag/std/nat.dag:219-231 decimal_digit_glyph; the '-' branch of integer.dag:316-335 is unreachable
# because atom_len is a fold_list count, never negative).
regex_le() {
  local B="$1" n="${#1}" i d pre cls out=('0')
  for ((i=1;i<n;i++)); do
    if [ "$i" = 1 ]; then out+=('[1-9]'); else out+=("[1-9][0-9]{$((i-1))}"); fi
  done
  for ((i=0;i<n;i++)); do
    d="${B:i:1}"
    [ "$d" = 0 ] && continue
    pre="${B:0:i}"
    if [ "$i" = 0 ]; then cls="[1-$((d-1))]"; else cls="[0-$((d-1))]"; fi
    if [ "$i" = $((n-1)) ]; then out+=("$pre$cls"); else out+=("$pre$cls[0-9]{$((n-i-1))}"); fi
  done
  out+=("$B")
  local IFS='|'; printf '%s' "${out[*]}"
}
TT_al_FORM="$(regex_le "$TT_al_BOUND")"
RD_R='WcResolved'; RD_A='WcAbsent'; RD_F='WcFileRefused'
RD_atom_U='atom == ^wc_undeclared_body (the frozen candidate)'; RD_atom_Q='atom outside the candidate set'; RD_atom_D='atom ABSENT (no locus)'
# Frozen reading table (parent msg_bf5a7d3c); 'observation only' lines are DATA, never acceptance.
RDX_NOCTX='READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
RDX_ALLF='FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
RDX_ALLA='FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
RDX_FALS='FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
RDX_X='reason matched none of the 24 compared caret symbols (outside-table OR comparison failure); loses which reason, and whether comparison works'
CTL_VOID='CONTROL VERDICT: CONTROL FAILED — CLAIM EXPECTATIONS, FALSIFIER AND ALL TABLE-INDEX READINGS ARE VOID (control failed; nothing concluded)'
# CONTROL readings (generated semantics; data, never acceptance). ctl_ok on ^resolve_reason_unbound_symbol
# (arm 0 of the chain), ctl_out on ^resolve_reason_binder_duplicate_in_frame (outside the 24).
ctl_ok_reading() {
  case "$1" in
    0) printf '%s\n' 'CONTROL HOLDS: the known symbol matched its own arm (0): caret "==" discriminates';;
    X) printf '%s\n' 'CONTROL VIOLATION: the known symbol matched NOTHING: caret "==" comparison is BROKEN; every r=X in this line is then a comparison failure, not an outside-table reason';;
    *) printf '%s\n' "CONTROL FAILED: the known symbol matched arm $1 instead of 0; VOID";;
  esac
}
ctl_out_reading() {
  case "$1" in
    X) printf '%s\n' 'CONTROL HOLDS: the outside-table symbol matched nothing (X), as declared: the chain does not compare symbols outside its 24 names';;
    *) printf '%s\n' "CONTROL VIOLATION: the outside-table symbol matched arm $1: equality COLLAPSED or the chain compares a symbol outside its 24 names";;
  esac
}
# probe_token_r R -> 'r=R', from the table. Values outside the declared set refuse.
probe_token_r() { case "|$TT_r|" in *'|'"$1"'|'**) printf '%s%s%s' "$SEP_r" "$EQ" "$1";; *) return 1;; esac; }
# probe_ctl FIELD VALUE -> ' FIELD=r=VALUE' (ctl tokens are reason_token outputs: 'r=' + value).
probe_ctl() { case "|$TT_ctl|" in *'|'"$2"'|'**) printf '%s%s%s%s' "$1" "$EQ" "$TOK_CTL" "$2";; *) return 1;; esac; }
# probe_token R A S AL [RS] -> the composed braced token, from the table only. Every token
# carries ' rs=' (v12): braced tokens carry the closed-class lexeme (default: a declared lane
# name); plain R/A/F carry '-' via probe_row_ok normalization.
probe_token() {
  local r="$1" a="$2" s="$3" al="$4" rs="${5:-resolve_reason_observation_incomplete}" t
  case "|$TT_r|" in *'|'"$r"'|'**) :;; *) return 1;; esac
  case "|$TT_a|" in *'|'"$a"'|'**) :;; *) return 1;; esac
  case "|$TT_s|" in *'|'"$s"'|'**) :;; *) return 1;; esac
  [[ "$al" =~ ^(${TT_al_FORM})$ ]] || return 1
  [ "${#al}" -le "${#TT_al_BOUND}" ] || return 1
  [[ "$rs" =~ ^(${TT_rs_CLASS})$ ]] || return 1
  t="$SEP_r$EQ$r$SEP_a$EQ$a$SEP_s$EQ$s$SEP_al$EQ$al"
  printf '%s' "$TOK_OPEN$TOK_IN$t$TOK_CLOSE$SEP_rs$EQ$rs"
}
# probe_row_ok COK COUT T1 T2 T3 T4 / probe_row_rf COK COUT -> complete reason strings, table only.
probe_row_ok() {
  local c1 c2; c1="$(probe_ctl "$SEP_ctl_ok" "$1")" || return 1; c2="$(probe_ctl "$SEP_ctl_out" "$2")" || return 1
  local toks=("$3" "$4" "$5" "$6") out=() t
  for t in "${toks[@]}"; do case "$t" in R|A|F) out+=("$t$SEP_rs$EQ-");; *) out+=("$t");; esac; done
  printf '%s%s%s%s%s%s%s%s%s%s%s%s%s' "$CTX_OK" "$c1" "$c2" "$SEP_arm" "$EQ${out[0]}" "$SEP_field" "$EQ${out[1]}" "$SEP_binder" "$EQ${out[2]}" "$SEP_body" "$EQ${out[3]}"
}
probe_row_rf() {
  local c1 c2; c1="$(probe_ctl "$SEP_ctl_ok" "$1")" || return 1; c2="$(probe_ctl "$SEP_ctl_out" "$2")" || return 1
  printf '%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s' "$CTX_RF" "$c1" "$c2" "$SEP_arm" "$EQ" "-" "$SEP_rs" "$EQ" "-" "$SEP_field" "$EQ" "-" "$SEP_rs" "$EQ" "-" "$SEP_binder" "$EQ" "-" "$SEP_rs" "$EQ" "-" "$SEP_body" "$EQ" "-" "$SEP_rs" "$EQ" "-"
}
# THE GRAMMARS, derived from the same constants. Every field alternative is the verbatim declared set,
# so any row that validates is byte-composable from the table alone (the split/reconstruct gate below
# proves it per row). Escapes ('{' '}' '?') are the ones ERE and python re accept IDENTICALLY; the
# byte-equality fixture against the reader constants enforces it.
PG_PLAIN="((${TT_plain})${SEP_rs}${EQ}-)"
ALT_r="(${TT_r})"; ALT_a_RAW="$(printf '%s' "$TT_a" | sed 's/?/\\?/g')"; ALT_a="($ALT_a_RAW)"
ALT_s="(${TT_s})"; ALT_ctl="(${TT_ctl})"; ALT_al="(${TT_al_FORM})"
PG_X="${TOK_OPEN}\\{${SEP_r}${EQ}${ALT_r}${SEP_a}${EQ}${ALT_a}${SEP_s}${EQ}${ALT_s}${SEP_al}${EQ}${ALT_al}\\}${SEP_rs}${EQ}(${TT_rs_CLASS})"
PG_V="(${PG_PLAIN}|${PG_X})"
PG_OK="^${CTX_OK}${SEP_ctl_ok}${EQ}${TOK_CTL}${ALT_ctl}${SEP_ctl_out}${EQ}${TOK_CTL}${ALT_ctl}${SEP_arm}${EQ}${PG_V}${SEP_field}${EQ}${PG_V}${SEP_binder}${EQ}${PG_V}${SEP_body}${EQ}${PG_V}\$"
PG_RF="^${CTX_RF}${SEP_ctl_ok}${EQ}${TOK_CTL}${ALT_ctl}${SEP_ctl_out}${EQ}${TOK_CTL}${ALT_ctl}${SEP_arm}${EQ}-${SEP_rs}${EQ}-${SEP_field}${EQ}-${SEP_rs}${EQ}-${SEP_binder}${EQ}-${SEP_rs}${EQ}-${SEP_body}${EQ}-${SEP_rs}${EQ}-\$"
# The BOUND is not a hand-written number: it is the LENGTH OF THE TABLE-COMPOSED WIDEST ROW.
WIDEST_TOKEN="$(probe_token 23 '?' F "$TT_al_BOUND" "$TT_rs_BOUND")" || die2 "REFUSED_SETUP: the widest declared token refused its own table"
WIDEST_ROW="$(probe_row_ok 23 23 "$WIDEST_TOKEN" "$WIDEST_TOKEN" "$WIDEST_TOKEN" "$WIDEST_TOKEN")" || die2 "REFUSED_SETUP: the widest declared row refused its own table"
WIDEST_LEN="${#WIDEST_ROW}"; BOUND="$WIDEST_LEN"
# probe_validate REASON -> 0 accepted / 1 refused; PROBE_VALIDATE_KIND records WHICH check refused
# (BOUND | GRAMMAR | CONSISTENCY). Check order is fixed: bound, grammar, consistency. The bound check
# may be bypassed ONLY via PROBE_BOUND_BYPASS=1, used by one fixture to prove the grammar itself
# refuses an over-long row, so the two checks are demonstrably distinct.
probe_validate() {
  local r="$1"
  PROBE_VALIDATE_KIND=OK
  if [ "${PROBE_BOUND_BYPASS:-0}" != 1 ]; then
    if [ "${#r}" -gt "$BOUND" ]; then PROBE_VALIDATE_KIND=BOUND; return 1; fi
  fi
  if [[ "$r" =~ $PG_RF ]]; then :
  elif [[ "$r" =~ $PG_OK ]]; then :
  else PROBE_VALIDATE_KIND=GRAMMAR; return 1; fi
  # CONSISTENCY (table rule TT_rule_a_minus_al_zero=1): a braced token with a='-' must carry al='0'.
  # RF rows have no braced tokens; OK rows are split once and each token parsed with the SAME parser.
  local c1 c2 t_arm t_field t_binder t_body tok t1 t2 t3 t4 a al
  if [[ "$r" =~ $PG_OK ]]; then
    probe_split_ok "$r" c1 c2 t_arm t_field t_binder t_body || { PROBE_VALIDATE_KIND=GRAMMAR; return 1; }
    for tok in "$t_arm" "$t_field" "$t_binder" "$t_body"; do
      case "$tok" in "$TOK_OPEN$TOK_IN"*)
        probe_parse_token "$tok" t1 t2 t3 t4 || { PROBE_VALIDATE_KIND=GRAMMAR; return 1; }
        a="$t2"; al="$t4"
        if [ "$a" = '-' ] && [ "$al" != 0 ]; then PROBE_VALIDATE_KIND=CONSISTENCY; return 1; fi
      esac
    done
  fi
  return 0
}
# probe_parse_token TOKEN RVAR AVAR SVAR ALVAR -> 0 and sets the four variables; refuses (1) on any
# token that is not composed of declared alternatives with the declared separators, in order, once each.
# The closing-brace checks are REACHABLE via direct-call fixtures (v8-p1..p4): on grammar-validated
# rows the anchored grammar already excludes those shapes, so this defense is exercised directly.
probe_parse_token() {
  local v="$1" __r __a __s __al rest __rs __core
  local -n __R="$2" __A="$3" __S="$4" __AL="$5"
  # rs suffix (v12): exactly one trailing '<SEP_rs><EQ><value>'; plain cores require '-', braced
  # cores require the closed class. A second occurrence anywhere is illegal.
  case "$v" in *"$SEP_rs$EQ"*) :;; *) return 1;; esac
  __core="${v%"$SEP_rs$EQ"*}"; __rs="${v##*"$SEP_rs$EQ"}"
  case "$__core" in *"$SEP_rs$EQ"*) return 1;; esac
  case "$__rs" in *"$SEP_rs$EQ"*) return 1;; esac
  case "$__core" in
    R|A|F) [ "$__rs" = '-' ] || return 1;;
    "$TOK_OPEN$TOK_IN"*) [[ "$__rs" =~ ^(${TT_rs_CLASS})$ ]] || return 1;;
  esac
  v="$__core"
  if [ "$#" -ge 6 ]; then local -n __RS6="$6"; __RS6="$__rs"; fi
  case "$v" in
    R) __r='__plain'; __a='R'; __s='__tag'; __al='R';;
    A) __r='__plain'; __a='A'; __s='__tag'; __al='A';;
    F) __r='__plain'; __a='F'; __s='__tag'; __al='F';;
    "$TOK_OPEN$TOK_IN"*)
      rest="${v#"$TOK_OPEN$TOK_IN"}"
      case "$v" in *"$TOK_CLOSE") :;; *) return 1;; esac           # must end with the single TOK_CLOSE
      rest="${rest%"$TOK_CLOSE"}"
      case "$rest" in *"$TOK_CLOSE"*) return 1;; esac           # a second '}' anywhere is illegal
      case "$rest" in "$SEP_r$EQ"*) :;; *) return 1;; esac
      rest="${rest#"$SEP_r$EQ"}"
      case "$rest" in *"$SEP_a$EQ"*) :;; *) return 1;; esac
      __r="${rest%%"$SEP_a$EQ"*}"; rest="${rest#*"$SEP_a$EQ"}"
      case "$rest" in *"$SEP_s$EQ"*) :;; *) return 1;; esac
      __a="${rest%%"$SEP_s$EQ"*}"; rest="${rest#*"$SEP_s$EQ"}"
      case "$rest" in *"$SEP_al$EQ"*) :;; *) return 1;; esac
      __s="${rest%%"$SEP_al$EQ"*}"; __al="${rest#*"$SEP_al$EQ"}"
      case "|$TT_r|" in *'|'"$__r"'|'**) :;; *) return 1;; esac
      case "|$TT_a|" in *'|'"$__a"'|'**) :;; *) return 1;; esac
      case "|$TT_s|" in *'|'"$__s"'|'**) :;; *) return 1;; esac
      [[ "$__al" =~ ^(${TT_al_FORM})$ ]] || return 1
      [ "${#__al}" -le "${#TT_al_BOUND}" ] || return 1;;
    *) return 1;;
  esac
  if [ "$__r" = __plain ]; then __R="$__al"; __A=''; __S=''; __AL=''; else __R="$__r"; __A="$__a"; __S="$__s"; __AL="$__al"; fi
  return 0
}
# CENSUS DATA (data, pinned by reference-site scan at the seed parent f08a6ae4; command recorded:
#   for f in src/v2/compiler/{source_root_eval,resolve,name_resolve}.dag; do
#     git show f08a6ae4:$f | grep -v '^\s*//' | grep -o '\^resolve_reason_[a-z0-9_]*'; done | sort -u
# => 37 distinct lane names; the chain (probe .dag, file order) has 24; chain-minus-lane = 1
# (resolve_reason_qualified_target_identity_unrepresentable); lane-minus-chain = 14. This block is
# EMBEDDED DATA; SELF_CONSISTENCY checks counts, distinctness, the class shape, and the arithmetic.
CENSUS_CHAIN24=(resolve_reason_unbound_symbol resolve_reason_ambiguous_symbol resolve_reason_declarer_refused resolve_reason_pattern_atom_names_visible_value resolve_reason_test_code_referenced resolve_reason_generated_reference_declaration_absent resolve_reason_malformed_imported_root resolve_reason_type_parameter_shadows_visible_name resolve_reason_reference_category_mismatch resolve_reason_realized_declaration_mismatch resolve_reason_where_predicate_unbound resolve_reason_type_parameter_in_value_position resolve_reason_realized_declaration_shadowed resolve_reason_malformed_tree resolve_reason_primitive_row_name_disagrees resolve_reason_lexical_reference_unkeyed resolve_reason_construct_tag_not_a_constructor resolve_reason_catch_all_binder_unread resolve_reason_binder_hides_visible_value resolve_reason_anonymous_parameter_referenced resolve_reason_type_application_arity_mismatch resolve_reason_type_argument_kind_not_inhabited resolve_reason_requirement_not_a_resource resolve_reason_qualified_target_identity_unrepresentable)
CENSUS_LANE37=(resolve_reason_ambiguous_symbol resolve_reason_anonymous_parameter_referenced resolve_reason_binder_duplicate_in_frame resolve_reason_binder_hides_visible_value resolve_reason_catch_all_binder_unread resolve_reason_construct_tag_not_a_constructor resolve_reason_declarer_refused resolve_reason_forwarded_parameter_kind_differs resolve_reason_forwarded_parameter_kind_unresolved resolve_reason_forwarded_parameter_unkinded resolve_reason_generated_reference_declaration_absent resolve_reason_kernel_type_spelling_names_a_foreign_declaration resolve_reason_lexical_reference_unkeyed resolve_reason_malformed_imported_root resolve_reason_malformed_tree resolve_reason_member_kind_duplicate_declaration resolve_reason_namespace_provider_admission_conflict resolve_reason_observation_incomplete resolve_reason_optional_declaration_incompatible resolve_reason_optional_declaration_unavailable resolve_reason_parameter_path_not_indexed resolve_reason_pattern_atom_names_visible_value resolve_reason_primitive_row_name_disagrees resolve_reason_realized_declaration_mismatch resolve_reason_realized_declaration_shadowed resolve_reason_reference_category_mismatch resolve_reason_requirement_not_a_resource resolve_reason_requirement_repeated resolve_reason_test_code_referenced resolve_reason_type_application_arity_mismatch resolve_reason_type_argument_kind_not_inhabited resolve_reason_type_parameter_in_value_position resolve_reason_type_parameter_kind_unresolved resolve_reason_type_parameter_order_malformed resolve_reason_type_parameter_shadows_visible_name resolve_reason_unbound_symbol resolve_reason_where_predicate_unbound)
rs_class() {   # NAME -> classification string (data rows; interpretation stays with the frozen table)
  local n="$1" i j hit_i='' hit_j=''
  for i in "${!CENSUS_CHAIN24[@]}"; do [ "${CENSUS_CHAIN24[$i]}" = "$n" ] && hit_i="$i"; done
  for j in "${!CENSUS_LANE37[@]}"; do [ "${CENSUS_LANE37[$j]}" = "$n" ] && hit_j="$j"; done
  if [ -n "$hit_i" ]; then printf 'IN the 24-name chain (chain arm %s)' "$hit_i"
  elif [ -n "$hit_j" ]; then printf 'OUTSIDE the 24; IN lane vocabulary (lane census #%s of 37)' "$((hit_j + 1))"
  else printf 'NOT IN EITHER VOCABULARY (unknown lexeme; fail-closed datum)'; fi
}
# probe_tok_reading TOKEN -> the reading of one verdict token; refuses (1) on an illegal token.
probe_tok_reading() {
  local v="$1" r a s al rsx
  case "$v" in R"$SEP_rs$EQ-"|A"$SEP_rs$EQ-"|F"$SEP_rs$EQ-")
    printf '%s; rs=- (absent: no reason symbol on a non-refused row; the expected row)\n' \
      "$(case "${v%%$SEP_rs*}" in R) echo "$RD_R";; A) echo "$RD_A";; *) echo "$RD_F";; esac)"; return 0;; esac
  probe_parse_token "$v" r a s al rsx || return 1
  local rs; if [ "$r" = X ]; then rs="$RDX_X"; else rs="reason = table index $r (the 24-name resolve_reason list)"; fi
  local at; case "$a" in U) at="$RD_atom_U";; \?) at="$RD_atom_Q";; -) at="$RD_atom_D";; *) return 1;; esac
  local sc; if [ "$s" = T ]; then sc='sole_chain=True (rest of the diagnostic chain Empty)'; else sc='sole_chain=False'; fi
  local rsd="rs=$rsx ($(rs_class "$rsx"))"
  if [ "$r" = X ] && rs_class "$rsx" | grep -q '^IN the 24-name chain'; then
    rsd="$rsd; CONTRADICTION DATA: the named symbol is in the 24 compared, yet the chain answered X with controls held — record as data; interpretation stays with the controls"
  fi
  printf 'WcRefused{%s; %s; %s; atom_lexeme_len=%s; %s}\n' "$rs" "$at" "$sc" "$al" "$rsd"
}
# probe_split_ok ROW C1 C2 T_ARM T_FIELD T_BINDER T_BODY — delimiter-based split on the SIX keys, in
# order, each exactly once; the reconstruction via probe_row_ok must reproduce the row BYTE-FOR-BYTE
# (layout proof; one authority). Values are bare (ctl values stripped of the 'r=' prefix here only).
probe_split_ok() {
  local r="$1" line
  local -n _C1="$2" _C2="$3" _A1="$4" _A2="$5" _A3="$6" _A4="$7"
  case "$r" in "$CTX_OK"*) :;; *) return 1;; esac
  local p
  for p in "$SEP_ctl_ok" "$SEP_ctl_out" "$SEP_arm" "$SEP_field" "$SEP_binder"; do
    case "$r" in *"$p$EQ"*) :;; *) return 1;; esac
  done
  line="$(printf '%s\n' "$r" | sed -e "s/^${CTX_OK}//" -e "s/$SEP_ctl_ok$EQ/\n/" -e "s/$SEP_ctl_out$EQ/\n/" -e "s/$SEP_arm$EQ/\n/" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | sed -e 's/^ *//' -e 's/ *$//' | sed '/^$/d')"
  [ "$(printf '%s\n' "$line" | wc -l)" = 6 ] || return 1
  _C1="$(printf '%s\n' "$line" | sed -n 1p)"; _C2="$(printf '%s\n' "$line" | sed -n 2p)"
  _C1="${_C1#"$TOK_CTL"}"; _C2="${_C2#"$TOK_CTL"}"
  _A1="$(printf '%s\n' "$line" | sed -n 3p)"; _A2="$(printf '%s\n' "$line" | sed -n 4p)"
  _A3="$(printf '%s\n' "$line" | sed -n 5p)"; _A4="$(printf '%s\n' "$line" | sed -n 6p)"
  local rebuilt; rebuilt="$(probe_row_ok "$_C1" "$_C2" "$_A1" "$_A2" "$_A3" "$_A4")"
  [ "$rebuilt" = "$r" ] || return 1
  return 0
}
# probe_read_line REASON: the reading-path lookup, mirroring probe_read.py (shipped in the packet and
# sha-gate-checked below; its grammar constants are byte-checked against PG_OK/PG_RF at setup).
# Observation only; it never implies acceptance.
probe_read_line() {
  local r="$1"
  if [ "${#r}" -gt "$BOUND" ]; then echo "REFUSAL: over the stated ${BOUND}-char bound (widest table-composed row); NO OBSERVATION"; return 2; fi
  local c1 c2 arm field binder body f v rd allf=1 alla=1
  if [[ "$r" =~ $PG_RF ]]; then
    # RF rows still carry the control; extract it with the same byte-exact discipline.
    local line; line="$(printf '%s\n' "$r" | sed -e "s/^${CTX_RF}//" -e "s/$SEP_ctl_ok$EQ/\n/" -e "s/$SEP_ctl_out$EQ/\n/" -e "s/$SEP_arm$EQ/\n/" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | sed -e 's/^ *//' -e 's/ *$//' | sed '/^$/d')"
    [ "$(printf '%s\n' "$line" | wc -l)" = 6 ] || { echo 'REFUSAL: RF control split failed'; return 2; }
    c1="$(printf '%s\n' "$line" | sed -n 1p)"; c2="$(printf '%s\n' "$line" | sed -n 2p)"
    c1="${c1#"$TOK_CTL"}"; c2="${c2#"$TOK_CTL"}"
    case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
    case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
RFDASH="-$SEP_rs$EQ-"
    [ "$(printf '%s\n' "$line" | sed -n 3p)" = "$RFDASH" ] && [ "$(printf '%s\n' "$line" | sed -n 4p)" = "$RFDASH" ] &&
    [ "$(printf '%s\n' "$line" | sed -n 5p)" = "$RFDASH" ] && [ "$(printf '%s\n' "$line" | sed -n 6p)" = "$RFDASH" ] ||
      { echo 'REFUSAL: RF row carries non-dash payload'; return 2; }
    echo "CONTROL (ctx=RF row): ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
    echo "CONTROL (ctx=RF row): ctl_out=$c2 -> $(ctl_out_reading "$c2")"
    if [ "$c1" = 0 ] && [ "$c2" = X ]; then echo "$RDX_NOCTX"; else echo "$CTL_VOID"; fi; return 0
  fi
  if ! [[ "$r" =~ $PG_OK ]]; then echo 'REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'; return 2; fi
  probe_split_ok "$r" c1 c2 arm field binder body || { echo 'REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'; return 2; }
  case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
  case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
  echo "CONTROL: ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
  echo "CONTROL: ctl_out=$c2 -> $(ctl_out_reading "$c2")"
  if [ "$c1" != 0 ] || [ "$c2" != X ]; then
    for f in arm field binder body; do probe_tok_reading "${!f}" >/dev/null || { echo "REFUSAL: token ${!f} is not a declared form"; return 2; }; done
    echo "$CTL_VOID"
    echo 'Observation only; acceptance never follows from this reading.'
    return 0
  fi
  echo 'CONTROL VERDICT (observation only): caret "==" comparison DISCRIMINATES for the two control symbols.'
  echo 'CLAIM EXPECTATIONS (the frozen wc claims; the body expectation names r=0, a=U and s=T — al is a datum, not an expectation):'
  echo 'PER-MODULE READINGS (MIXED tags => read each row on its own; no single-boundary conclusion follows):'
  for f in arm field binder body; do
    v="${!f}"
    rd="$(probe_tok_reading "$v")" || { echo "REFUSAL: token $v is not a declared form"; return 2; }
    printf '  %-7s %-52s %s\n' "$f" "$v" "$rd"
    [ "${v%%$SEP_rs*}" = F ] || allf=0
    [ "${v%%$SEP_rs*}" = A ] || alla=0
  done
  if [ "$allf" = 1 ]; then echo "$RDX_ALLF"; fi
  if [ "$alla" = 1 ]; then echo "$RDX_ALLA"; fi
  local okarm=0 okfield=0 okbinder=0 okbody=0
  [ "${arm%%$SEP_rs*}" = R ] && okarm=1
  [ "${field%%$SEP_rs*}" = R ] && okfield=1
  [ "${binder%%$SEP_rs*}" = R ] && okbinder=1
  case "$body" in "${TOK_OPEN}${TOK_IN}${SEP_r}${EQ}0${SEP_a}${EQ}U${SEP_s}${EQ}T${SEP_al}${EQ}"*) okbody=1;; esac
  :
  printf '  %-7s %-47s observed %-44s %s\n' arm 'expect R' "$arm" "$([ "$okarm" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' field 'expect R' "$field" "$([ "$okfield" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' binder 'expect R' "$binder" "$([ "$okbinder" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' body 'expect X with r=0, a=U and s=T (sole unbound refusal at ^wc_undeclared_body)' "$body" "$([ "$okbody" = 1 ] && echo MATCH || echo MISMATCH)"
  if [ "$okarm" = 1 ] && [ "$okfield" = 1 ] && [ "$okbinder" = 1 ] && [ "$okbody" = 1 ]; then echo "$RDX_FALS"; fi
  echo 'Observation only; acceptance never follows from this reading.'
  return 0
}

PROBE_TOKEN=NONE
probe_verdict() {
  local L n; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
  if [ -z "$L" ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
  n="$(printf '%s\n' "$L" | grep -ac '')"
  if [ "$n" != 1 ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
  probe_classify "$L" "$2" >/dev/null; echo "$PROBE_TOKEN"
}

# (6b) THE SHIPPED OFFLINE READER, embedded verbatim (quoted heredoc delimiter: zero expansion at write),
#   and sha-gated at run time into $R. The offline reading path IS the embedded copy; the fixtures exercise the
#   shipped shell reader, and a cross-check fixture (when python3 exists in the build env) asserts BOTH agree.
PROBE_READ_PY_SHA=52be5833e2b2c440ed17ef8f3fea71e5da74292deb4e2e19da0399bd043079ea

# ---- PRE-FETCH SELF-CONSISTENCY (parent msg_3152d6ec, ERS latent defect: an embedded roster with a trailing
# newline sha'd wrong and every real run would die AFTER the fetch; a pre-fetch E2E cannot see post-fetch gates).
# Every post-fetch gate whose inputs are EMBEDDED in this packet is exercised HERE, over those exact bytes:
#   (a) declared hex constants are well-formed (the post-fetch pin/tree/blob gates consume them verbatim);
#   (b) the roster name passes the outcome-join token regex and the single-name set behaves under EXACT
#       membership (multi-token, empty, hyphenated, keyword and '<name> extra' non-members all refuse;
#       count equals roster size) — the ERS v4 class, refused before the fetch;
#   (c) PROBE_READ_PY_SHA is well-formed (the embedded source itself is sha-gated before any fetch below).
for _c in "$PIN_P" "$TREE_P" "$BLOB_PROBE" "$BLOB_WC" "$SEED_PARENT"; do
  [[ "$_c" =~ ^[0-9a-f]{40}$ ]] || die2 "REFUSED_SETUP: embedded 40-hex constant '$_c' is malformed (post-fetch pin/blob gates would die after the fetch)"
done
[[ "$PROBE_READ_PY_SHA" =~ ^[0-9a-f]{64}$ ]] || die2 "REFUSED_SETUP: PROBE_READ_PY_SHA is malformed"
# v11 (parent msg_b897e1be): the post-fetch parent gate needs SEED_PARENT == PIN_A; checked pre-fetch over
# the embedded constants, so a wrong-parent seed is refused BEFORE the fetch instead of after it.
[ "$SEED_PARENT" = "$PIN_A" ] || die2 "REFUSED_SETUP: embedded SEED_PARENT $SEED_PARENT != PIN_A $PIN_A (the depth-2 fetch could not satisfy HEAD^1 == PIN_A)"
[[ "$PFN" =~ ^[a-z0-9_]+$ ]] || die2 "REFUSED_SETUP: the roster name PFN does not pass the outcome-join token regex"
{ declare -A SELF_ROSTER=([$PFN]=1)
  [ "${#SELF_ROSTER[@]}" = 1 ] || die2 "REFUSED_SETUP: embedded roster size != 1"
  [ -n "${SELF_ROSTER[$PFN]+x}" ] || die2 "REFUSED_SETUP: roster name not an exact member of its own set"
  for _bad in 'a b' '' 'x-y' 'PASS' "${PFN} extra"; do
    [ -z "${SELF_ROSTER[$_bad]+x}" ] || die2 "REFUSED_SETUP: roster set accepted non-member '$_bad' (substring/loose matching defect)"
  done
}
# CENSUS DATA SELF-CHECK (embedded inputs, per msg_3152d6ec): counts, distinctness, class shapes, and the
# pinned arithmetic (chain-minus-lane = 1 named datum; lane-minus-chain = 14). Data, not interpretation.
[ "${#CENSUS_CHAIN24[@]}" = 24 ] || die2 "REFUSED_SETUP: census chain roster is not 24"
[ "${#CENSUS_LANE37[@]}" = 37 ] || die2 "REFUSED_SETUP: census lane roster is not 37"
[ "$(printf '%s\n' "${CENSUS_CHAIN24[@]}" | sort -u | wc -l)" = 24 ] || die2 "REFUSED_SETUP: census chain roster has duplicates"
[ "$(printf '%s\n' "${CENSUS_LANE37[@]}" | sort -u | wc -l)" = 37 ] || die2 "REFUSED_SETUP: census lane roster has duplicates"
for _n in "${CENSUS_CHAIN24[@]}"; do [[ "$_n" =~ ^resolve_reason_[a-z0-9_]{1,64}$ ]] || die2 "REFUSED_SETUP: census chain name outside the class: $_n"; done
for _n in "${CENSUS_LANE37[@]}"; do [[ "$_n" =~ ^resolve_reason_[a-z0-9_]{1,64}$ ]] || die2 "REFUSED_SETUP: census lane name outside the class: $_n"; done
_chainonly="$(comm -23 <(printf '%s\n' "${CENSUS_CHAIN24[@]}" | sort) <(printf '%s\n' "${CENSUS_LANE37[@]}" | sort))"
_laneonly_n="$(comm -13 <(printf '%s\n' "${CENSUS_CHAIN24[@]}" | sort) <(printf '%s\n' "${CENSUS_LANE37[@]}" | sort) | wc -l)"
[ "$_chainonly" = "resolve_reason_qualified_target_identity_unrepresentable" ] || die2 "REFUSED_SETUP: census chain-minus-lane is not the pinned single datum"
[ "$_laneonly_n" = 14 ] || die2 "REFUSED_SETUP: census lane-minus-chain is not 14"
echo "SELF_CONSISTENCY=held (hex constants incl SEED_PARENT, SEED_PARENT==PIN_A, PROBE_READ_PY_SHA form, roster EXACT membership, CENSUS DATA incl arithmetic)"
cat > "$RS/.probe_read-src.py" <<'PROBE_READ_EOF'
#!/usr/bin/env python3
# probe_read.py v8 — the offline reader for the wc verdict probe (sleek-seal-226, PR #15695).
# Usage: python3 probe_read.py <file-containing-exactly-one-WCVERIFY-reason-line>
# Contract: exactly one line; if it does not validate, print REFUSAL and exit 2. Output is OBSERVATION
# ONLY; acceptance never follows. The two regex constants below are asserted BYTE-IDENTICAL to the
# grammars generated from the ONE authority table in the packet (fixtures at setup), and BOUND equals
# the length of the table-composed widest row; any drift is REFUSED_SETUP in the packet before any run.
import re, sys

BOUND = 490

RE_OK = re.compile(r"^WCVERIFY ctx=OK ctl_ok=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X) ctl_out=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X) arm=(((R|A|F) rs=-)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\} rs=([a-z0-9_]{1,64})) field=(((R|A|F) rs=-)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\} rs=([a-z0-9_]{1,64})) binder=(((R|A|F) rs=-)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\} rs=([a-z0-9_]{1,64})) body=(((R|A|F) rs=-)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\} rs=([a-z0-9_]{1,64}))$")

RE_RF = re.compile(r"^WCVERIFY ctx=RF ctl_ok=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X) ctl_out=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X) arm=- rs=- field=- rs=- binder=- rs=- body=- rs=-$")

RDX_NOCTX = 'READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
RDX_ALLF = 'FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
RDX_ALLA = 'FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
RDX_FALS = 'FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
RDX_X = 'reason matched none of the 24 compared caret symbols (outside-table OR comparison failure); loses which reason, and whether comparison works'
CTL_VOID = 'CONTROL VERDICT: CONTROL FAILED — CLAIM EXPECTATIONS, FALSIFIER AND ALL TABLE-INDEX READINGS ARE VOID (control failed; nothing concluded)'

def ctl_ok_reading(v):
    if v == '0':
        return 'CONTROL HOLDS: the known symbol matched its own arm (0): caret "==" discriminates'
    if v == 'X':
        return 'CONTROL VIOLATION: the known symbol matched NOTHING: caret "==" comparison is BROKEN; every r=X in this line is then a comparison failure, not an outside-table reason'
    return 'CONTROL FAILED: the known symbol matched arm %s instead of 0; VOID' % v

def ctl_out_reading(v):
    if v == 'X':
        return 'CONTROL HOLDS: the outside-table symbol matched nothing (X), as declared: the chain does not compare symbols outside its 24 names'
    return 'CONTROL VIOLATION: the outside-table symbol matched arm %s: equality COLLAPSED or the chain compares a symbol outside its 24 names' % v

CENSUS_CHAIN24 = ['resolve_reason_unbound_symbol', 'resolve_reason_ambiguous_symbol', 'resolve_reason_declarer_refused', 'resolve_reason_pattern_atom_names_visible_value', 'resolve_reason_test_code_referenced', 'resolve_reason_generated_reference_declaration_absent', 'resolve_reason_malformed_imported_root', 'resolve_reason_type_parameter_shadows_visible_name', 'resolve_reason_reference_category_mismatch', 'resolve_reason_realized_declaration_mismatch', 'resolve_reason_where_predicate_unbound', 'resolve_reason_type_parameter_in_value_position', 'resolve_reason_realized_declaration_shadowed', 'resolve_reason_malformed_tree', 'resolve_reason_primitive_row_name_disagrees', 'resolve_reason_lexical_reference_unkeyed', 'resolve_reason_construct_tag_not_a_constructor', 'resolve_reason_catch_all_binder_unread', 'resolve_reason_binder_hides_visible_value', 'resolve_reason_anonymous_parameter_referenced', 'resolve_reason_type_application_arity_mismatch', 'resolve_reason_type_argument_kind_not_inhabited', 'resolve_reason_requirement_not_a_resource', 'resolve_reason_qualified_target_identity_unrepresentable']
CENSUS_LANE37 = ['resolve_reason_ambiguous_symbol', 'resolve_reason_anonymous_parameter_referenced', 'resolve_reason_binder_duplicate_in_frame', 'resolve_reason_binder_hides_visible_value', 'resolve_reason_catch_all_binder_unread', 'resolve_reason_construct_tag_not_a_constructor', 'resolve_reason_declarer_refused', 'resolve_reason_forwarded_parameter_kind_differs', 'resolve_reason_forwarded_parameter_kind_unresolved', 'resolve_reason_forwarded_parameter_unkinded', 'resolve_reason_generated_reference_declaration_absent', 'resolve_reason_kernel_type_spelling_names_a_foreign_declaration', 'resolve_reason_lexical_reference_unkeyed', 'resolve_reason_malformed_imported_root', 'resolve_reason_malformed_tree', 'resolve_reason_member_kind_duplicate_declaration', 'resolve_reason_namespace_provider_admission_conflict', 'resolve_reason_observation_incomplete', 'resolve_reason_optional_declaration_incompatible', 'resolve_reason_optional_declaration_unavailable', 'resolve_reason_parameter_path_not_indexed', 'resolve_reason_pattern_atom_names_visible_value', 'resolve_reason_primitive_row_name_disagrees', 'resolve_reason_realized_declaration_mismatch', 'resolve_reason_realized_declaration_shadowed', 'resolve_reason_reference_category_mismatch', 'resolve_reason_requirement_not_a_resource', 'resolve_reason_requirement_repeated', 'resolve_reason_test_code_referenced', 'resolve_reason_type_application_arity_mismatch', 'resolve_reason_type_argument_kind_not_inhabited', 'resolve_reason_type_parameter_in_value_position', 'resolve_reason_type_parameter_kind_unresolved', 'resolve_reason_type_parameter_order_malformed', 'resolve_reason_type_parameter_shadows_visible_name', 'resolve_reason_unbound_symbol', 'resolve_reason_where_predicate_unbound']

def rs_class(n):
    if n in CENSUS_CHAIN24:
        return 'IN the 24-name chain (chain arm %d)' % CENSUS_CHAIN24.index(n)
    if n in CENSUS_LANE37:
        return 'OUTSIDE the 24; IN lane vocabulary (lane census #%d of 37)' % (CENSUS_LANE37.index(n) + 1)
    return 'NOT IN EITHER VOCABULARY (unknown lexeme; fail-closed datum)'

RD_R = 'WcResolved'; RD_A = 'WcAbsent'; RD_F = 'WcFileRefused'
RD_atom_U = 'atom == ^wc_undeclared_body (the frozen candidate)'
RD_atom_Q = 'atom outside the candidate set'
RD_atom_D = 'atom ABSENT (no locus)'

def split_rs(v):
    # -> (core, rs) or None; exactly one trailing ' rs=' occurrence.
    if v.count(' rs=') != 1:
        return None
    i = v.rfind(' rs=')
    core, rs = v[:i], v[i + 4:]
    if ' rs=' in core or not rs:
        return None
    return core, rs

def tok_reading(v):
    sp = split_rs(v)
    if sp is None:
        return None
    core, rsx = sp
    if core in ('R', 'A', 'F'):
        if rsx != '-':
            return None
        return {'R': RD_R, 'A': RD_A, 'F': RD_F}[core] + '; rs=- (absent: no reason symbol on a non-refused row; the expected row)'
    m = re.match(r'^X\{r=([^,]+),a=([^,]+),s=([^,]+),al=([0-9]+)\}$', core)
    if not m:
        return None
    r, a, s, al = m.groups()
    if not re.fullmatch(r'[a-z0-9_]{1,64}', rsx):
        return None
    rs = RDX_X if r == 'X' else 'reason = table index %s (the 24-name resolve_reason list)' % r
    rsd = 'rs=%s (%s)' % (rsx, rs_class(rsx))
    if r == 'X' and rsx in CENSUS_CHAIN24:
        rsd += '; CONTRADICTION DATA: the named symbol is in the 24 compared, yet the chain answered X with controls held — record as data; interpretation stays with the controls'
    at = {'U': RD_atom_U, '?': RD_atom_Q, '-': RD_atom_D}[a]
    sc = 'sole_chain=True (rest of the diagnostic chain Empty)' if s == 'T' else 'sole_chain=False'
    return 'WcRefused{%s; %s; %s; atom_lexeme_len=%s; %s}' % (rs, at, sc, al, rsd)

def split_row(r):
    # Six fields in order; the reconstruction via the declared layout must reproduce the row EXACTLY.
    m = re.match(r'^WCVERIFY ctx=OK ctl_ok=r=([^ ]+) ctl_out=r=([^ ]+) arm=(\S+ rs=\S+) field=(\S+ rs=\S+) binder=(\S+ rs=\S+) body=(\S+ rs=\S+)$', r)
    if not m:
        return None
    c1, c2, arm, field, binder, body = m.groups()
    rebuilt = ('WCVERIFY ctx=OK ctl_ok=r=%s ctl_out=r=%s arm=%s field=%s binder=%s body=%s'
               % (c1, c2, arm, field, binder, body))
    if rebuilt != r:
        return None
    return c1, c2, arm, field, binder, body

def main():
    if len(sys.argv) != 2:
        print('usage: probe_read.py <file with exactly one WCVERIFY reason line>'); return 2
    try:
        raw = open(sys.argv[1], 'rb').read().decode('utf-8')
    except Exception as e:
        print('REFUSAL: cannot read file: %s' % e); return 2
    if raw.endswith('\n'):
        raw = raw[:-1]
    if '\n' in raw:
        print('REFUSAL: more than one line in the file (the reason is a SINGLE line, and no reason field may contain a newline)'); return 2
    if len(raw) > BOUND:
        print('REFUSAL: over the stated %d-char bound (widest table-composed row); NO OBSERVATION' % BOUND); return 2
    if RE_RF.match(raw):
        m = re.match(r'^WCVERIFY ctx=RF ctl_ok=r=([^ ]+) ctl_out=r=([^ ]+) arm=- rs=- field=- rs=- binder=- rs=- body=- rs=-$', raw)
        c1, c2 = m.group(1), m.group(2)
        print('CONTROL (ctx=RF row): ctl_ok=%s -> %s' % (c1, ctl_ok_reading(c1)))
        print('CONTROL (ctx=RF row): ctl_out=%s -> %s' % (c2, ctl_out_reading(c2)))
        print(RDX_NOCTX if (c1 == '0' and c2 == 'X') else CTL_VOID)
        return 0
    if not RE_OK.match(raw):
        print('REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'); return 2
    sp = split_row(raw)
    if sp is None:
        print('REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'); return 2
    c1, c2, arm, field, binder, body = sp
    print('CONTROL: ctl_ok=%s -> %s' % (c1, ctl_ok_reading(c1)))
    print('CONTROL: ctl_out=%s -> %s' % (c2, ctl_out_reading(c2)))
    if not (c1 == '0' and c2 == 'X'):
        for v in (arm, field, binder, body):
            if tok_reading(v) is None:
                print('REFUSAL: token %s is not a declared form' % v); return 2
        print(CTL_VOID)
        print('Observation only; acceptance never follows from this reading.')
        return 0
    print('CONTROL VERDICT (observation only): caret "==" comparison DISCRIMINATES for the two control symbols.')
    print('CLAIM EXPECTATIONS (the frozen wc claims; the body expectation names r=0, a=U and s=T — al is a datum, not an expectation):')
    print('PER-MODULE READINGS (MIXED tags => read each row on its own; no single-boundary conclusion follows):')
    allf = alla = True
    for name, v in (('arm', arm), ('field', field), ('binder', binder), ('body', body)):
        rd = tok_reading(v)
        if rd is None:
            print('REFUSAL: token %s is not a declared form' % v); return 2
        print('  %-7s %-52s %s' % (name, v, rd))
        if not v.startswith('F'): allf = False
        if not v.startswith('A'): alla = False
    if allf: print(RDX_ALLF)
    if alla: print(RDX_ALLA)
    ok = {'arm': arm == 'R rs=-', 'field': field == 'R rs=-', 'binder': binder == 'R rs=-',
          'body': bool(re.match(r'^X\{r=0,a=U,s=T,al=[0-9]+\} rs=[a-z0-9_]{1,64}$', body))}
    body_expect = 'expect X with r=0, a=U and s=T (sole unbound refusal at ^wc_undeclared_body)'
    for name, v, e in (('arm', arm, 'expect R'), ('field', field, 'expect R'), ('binder', binder, 'expect R'), ('body', body, body_expect)):
        print('  %-7s %-47s observed %-44s %s' % (name, e, v, 'MATCH' if ok[name] else 'MISMATCH'))
    if all(ok.values()): print(RDX_FALS)
    print('Observation only; acceptance never follows from this reading.')
    return 0

sys.exit(main())
PROBE_READ_EOF
probe_read_offline() { if command -v python3 >/dev/null 2>&1; then python3 "$RS/.probe_read-src.py" "$1"; else echo "READER_PY=absent; use $R/.probe_read.py offline"; return 3; fi; }

# (7) probe classifier fixtures (a focused failing input for each misread class), before any fetch.
# All WCVERIFY payloads carry the v7 control fields (ctl_ok/ctl_out); the classifier is shape-agnostic
# and its contract is unchanged — only the embedded reason lines take the new declared layout.
W_OK='ctl_ok=r=0 ctl_out=r=X'
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$W_OK" > "$FX/p1"
printf 'PASS %s\nFAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$PFN" "$W_OK" > "$FX/p2"
printf 'PASS %s\n' "$PFN" > "$FX/p3"
printf 'FAIL %s\n' "$PFN" > "$FX/p4"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 1: boom)\n' "$PFN" > "$FX/p5"
printf 'UNOBSERVED %s (returned ProcessExit::ExitFailure, code 2: nothing observed)\n' "$PFN" > "$FX/p6"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=RF %s arm=- field=- binder=- body=-\n' "$PFN" "$W_OK" > "$FX/p7"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 33: WCVERIFY ctx=OK %s arm=R field=R binder=R body=R)\n' "$PFN" "$W_OK" > "$FX/p8"
printf 'FAIL %s (returned `Int`, not Bool; --claim-run entries must return Bool)\n' "$PFN" > "$FX/p9"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 4: WCVERIFY BOUND_VIOLATED len=140)\n' "$PFN" > "$FX/p10"
[ "$(probe_verdict $FX/p1 $PFN) $(probe_verdict $FX/p2 $PFN) $(probe_verdict $FX/p3 $PFN) $(probe_verdict $FX/p4 $PFN) $(probe_verdict $FX/p5 $PFN) $(probe_verdict $FX/p6 $PFN) $(probe_verdict $FX/p7 $PFN) $(probe_verdict $FX/p8 $PFN) $(probe_verdict $FX/p9 $PFN) $(probe_verdict $FX/p10 $PFN)" = "EXIT3 NONE PASS BARE OTHER UNOBSERVED OTHER OTHER OTHER EXIT4" ] || die2 "REFUSED_SETUP: probe-classifier fixture misfire (valid, duplicate, PASS, bare, code1, UNOBSERVED, unclosed, code-33 boundary, NotBool, code-4)"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$W_OK" > "$FX/pv"
probe_verdict "$FX/pv" "$PFN" >/dev/null
# the extraction check is VERBATIM on the raw line (v11 bytes: no rs suffix) — extraction is rs-agnostic
[ "$PROBE_REASON" = "WCVERIFY ctx=OK $W_OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}" ] || die2 "REFUSED_SETUP: probe reason extraction is not verbatim"

# (8) bound self-check: the BOUND is the LENGTH OF THE TABLE-COMPOSED WIDEST ROW (no hand-written
#   number survives anywhere). Printed for the run record.
echo "PROBE_BOUND: widest-declared=$WIDEST_LEN bound=$BOUND widest-token=${#WIDEST_TOKEN} ctl=($((${#SEP_ctl_ok} + 1 + ${#TOK_CTL} + 2)),$((${#SEP_ctl_out} + 1 + ${#TOK_CTL} + 2))) al-bound=${TT_al_BOUND}"

# (9) grammar fixtures through the ACTUAL path: a full typed line -> probe_verdict -> probe_validate.
PGD="$FX"
pv_line() { printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1" > "$PGD/gline"; probe_verdict "$PGD/gline" "$PFN" >/dev/null; probe_validate "$PROBE_REASON"; echo $?; }
pv_kind() { printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1" > "$PGD/gline"; probe_verdict "$PGD/gline" "$PFN" >/dev/null; probe_validate "$PROBE_REASON" >/dev/null; echo "$PROBE_VALIDATE_KIND"; }
# valid OK row and valid RF row (table-composed)
[ "$(pv_line "$(probe_row_ok 0 X R R R "$(probe_token 0 ? T 1)")")" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid OK) refused"
[ "$(pv_line "$(probe_row_rf 0 X)")" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid RF) refused"
# garbage reason; extra trailing field; unknown tag
[ "$(pv_line 'hello world')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (garbage) accepted"
[ "$(pv_line "$(probe_row_ok 0 X R R R R) extra=R")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (extra field) accepted"
[ "$(pv_line "$(probe_row_ok 0 X Q R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (unknown tag) accepted"
# malformed tokens, built OUTSIDE the table functions on purpose (the table cannot produce them):
BADTOK="X{r=24,a=U,s=T,al=1} rs=resolve_reason_observation_incomplete"
[ "$(pv_line "$(probe_row_ok 0 X "$BADTOK" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (reason index 24) accepted"
[ "$(pv_line "$(probe_row_ok 0 X "X{r=07,a=U,s=T,al=1} rs=resolve_reason_observation_incomplete" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (r=07 leading zero) accepted"
[ "$(pv_line "$(probe_row_ok 0 X "X{r=0,a=U,s=T,al=01} rs=resolve_reason_observation_incomplete" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (al=01 leading zero) accepted"
[ "$(pv_line "$(probe_row_ok 0 X X R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare tag X) accepted"
[ "$(pv_line 'WCVERIFY ctx=RF')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare RF) accepted"
# CONTROL-field fixtures: missing, out of order, illegal values, bare form
[ "$(pv_line "WCVERIFY ctx=OK ctl_out=r=X arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: a row without ctl_ok was accepted"
[ "$(pv_line "WCVERIFY ctx=RF ctl_out=r=X arm=- field=- binder=- body=-")" = 1 ] || die2 "REFUSED_SETUP: an RF row without ctl_ok was accepted"
[ "$(pv_line "WCVERIFY ctx=OK ctl_out=r=X ctl_ok=r=0 arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: ctl fields out of order were accepted"
[ "$(pv_line "$(probe_row_ok 24 X R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_ok=r=24 was accepted"
[ "$(pv_line "$(probe_row_ok 0 24 R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_out=r=24 was accepted"
[ "$(pv_line "WCVERIFY ctx=OK ctl_ok=X ctl_out=r=X arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: a bare ctl token (no 'r=') was accepted"
# i64 staircase through the real path: max accepted; max+1 and 19x9 refused by GRAMMAR
[ "$(pv_line "$(probe_row_ok 0 X R R R "$(probe_token 23 U T 9223372036854775807)")")" = 0 ] || die2 "REFUSED_SETUP: i64-max al refused"
[ "$(pv_kind "$(probe_row_ok 0 X R R R "X{r=23,a=U,s=T,al=9223372036854775808} rs=resolve_reason_observation_incomplete")")" = GRAMMAR ] || die2 "REFUSED_SETUP: i64max+1 must be a GRAMMAR refusal"
[ "$(pv_kind "$(probe_row_ok 0 X R R R "X{r=23,a=U,s=T,al=9999999999999999999} rs=resolve_reason_observation_incomplete")")" = GRAMMAR ] || die2 "REFUSED_SETUP: 19x9 must be a GRAMMAR refusal"
# CONSISTENCY rule: a='-' with al!=0 refused as CONSISTENCY; with al=0 accepted
[ "$(pv_kind "$(probe_row_ok 0 X R R R "X{r=5,a=-,s=F,al=5} rs=resolve_reason_observation_incomplete")")" = CONSISTENCY ] || die2 "REFUSED_SETUP: a=- with al!=0 must be a CONSISTENCY refusal"
[ "$(pv_line "$(probe_row_ok 0 X R R R "$(probe_token 5 - F 0)")")" = 0 ] || die2 "REFUSED_SETUP: a=- with al=0 refused"
# BOUND vs GRAMMAR are demonstrably distinct: widest accepted; widest+1 refused as BOUND; with the
# bound check bypassed (fixture-only toggle) the SAME row is refused as GRAMMAR.
LONG="${WIDEST_ROW}X"
[ "${#LONG}" = $((WIDEST_LEN + 1)) ] || die2 "REFUSED_SETUP: the over-long fixture is not widest+1"
[ "$(pv_kind "$LONG")" = BOUND ] || die2 "REFUSED_SETUP: widest+1 must be a BOUND refusal"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$LONG" > "$PGD/gline"
probe_verdict "$PGD/gline" "$PFN" >/dev/null
PROBE_BOUND_BYPASS=1 probe_validate "$PROBE_REASON" >/dev/null; [ "$PROBE_VALIDATE_KIND" = GRAMMAR ] || die2 "REFUSED_SETUP: with the bound bypassed the same row must be a GRAMMAR refusal"
# the v5 OBSERVED-BYTES line (raw claims-A-probe.log, sha256 6315352c...): kept ONLY as bytes; under
# the v8 grammar it is refused (no control fields), which is exactly why the v5 run was NO_OBSERVATION.
V5LINE='WCVERIFY ctx=OK arm=X{r=X,a=-,s=T,al=0} field=X{r=X,a=-,s=T,al=0} binder=X{r=X,a=-,s=T,al=0} body=X{r=X,a=-,s=T,al=0}'
[ "$(pv_line "$V5LINE")" = 1 ] || die2 "REFUSED_SETUP: the v5 observed-bytes row (no control fields) must be refused under the v8 grammar"
printf '%s\n' "$V5LINE" > "$PGD/observed-v5.txt"

# (10) PER-FORM SWEEP through the REAL validator AND the reading path: every declared alternative of
#   every field, one line each, each asserted ACCEPTED and mapped to its declared reading.
sw() {  # sw REASON NEEDLE...  -> assert accepted + reading-path contains each needle
  local r="$1"; shift
  [ "$(pv_line "$r")" = 0 ] || die2 "REFUSED_SETUP: sweep row refused: $r"
  local out; out="$(probe_read_line "$r")" || die2 "REFUSED_SETUP: sweep reading refused: $r"
  local needle
  for needle in "$@"; do printf '%s\n' "$out" | grep -qF "$needle" || die2 "REFUSED_SETUP: sweep reading missing '$needle' for: $r"; done
}
for k in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23; do
  TOK="$(probe_token "$k" U T 0)"; ROW="$(probe_row_ok 0 X "$TOK" R R R)"
  if [ "$k" = 0 ]; then sw "$ROW" "table index 0"; else sw "$ROW" "table index $k"; fi
done
TOK="$(probe_token X U T 0)"; sw "$(probe_row_ok 0 X "$TOK" R R R)" "outside-table OR comparison failure"
for a in U ? -; do
  TOK="$(probe_token 0 "$a" T 0)"; ROW="$(probe_row_ok 0 X R R R "$TOK")"
  case "$a" in U) sw "$ROW" 'atom == ^wc_undeclared_body';; \?) sw "$ROW" 'atom outside the candidate set';; -) sw "$ROW" 'atom ABSENT (no locus)';; esac
done
for s in T F; do
  TOK="$(probe_token 0 U "$s" 0)"; ROW="$(probe_row_ok 0 X R R R "$TOK")"
  if [ "$s" = T ]; then sw "$ROW" 'sole_chain=True'; else sw "$ROW" 'sole_chain=False'; fi
done
for al in 0 1 7 99 999999999 9223372036854775806 9223372036854775807; do
  TOK="$(probe_token 0 U T "$al")"; sw "$(probe_row_ok 0 X R R R "$TOK")" "atom_lexeme_len=$al"
done
for k in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23; do
  ROW="$(probe_row_ok "$k" X R R R R)"; OUT="$(probe_read_line "$ROW")"
  printf '%s\n' "$OUT" | grep -qF "CONTROL: ctl_ok=$k" || die2 "REFUSED_SETUP: ctl_ok sweep row missing: $k"
  if [ "$k" = 0 ]; then printf '%s\n' "$OUT" | grep -qF 'CONTROL HOLDS: the known symbol matched its own arm' || die2 "REFUSED_SETUP: ctl_ok=0 must read HOLDS"
  else printf '%s\n' "$OUT" | grep -qF 'VOID' || die2 "REFUSED_SETUP: ctl_ok=$k must read CONTROL FAILED (VOID)"; fi
  ROW="$(probe_row_ok 0 "$k" R R R R)"; OUT="$(probe_read_line "$ROW")"
  printf '%s\n' "$OUT" | grep -qF "CONTROL: ctl_out=$k" || die2 "REFUSED_SETUP: ctl_out sweep row missing: $k"
  printf '%s\n' "$OUT" | grep -qF 'equality COLLAPSED' || die2 "REFUSED_SETUP: ctl_out=$k (a matched arm) must read COLLAPSED"
done
ROW="$(probe_row_ok 0 X R R R R)"; printf '%s\n' "$(probe_read_line "$ROW")" | grep -qF 'CONTROL HOLDS: the outside-table symbol matched nothing' || die2 "REFUSED_SETUP: ctl_out=X must read HOLDS"
ROW="$(probe_row_ok X X R R R R)"; printf '%s\n' "$(probe_read_line "$ROW")" | grep -qF 'CONTROL VERDICT: CONTROL FAILED' || die2 "REFUSED_SETUP: ctl_ok=X must yield the VOID control-failure verdict"
# (10b) v8 CONTROL GATE fixtures (parent msg_88479929 B1): a FAILED control VOIDs the claim expectations,
#   the FALSIFIER and every table-index reading — even when the row is otherwise falsifier-shaped.
for spec in "0 0 falsifier-shaped" "5 X permuted" "X X broken"; do
  set -- $spec; K1="$1"; K2="$2"
  ROW="$(probe_row_ok "$K1" "$K2" R R R "$(probe_token 0 U T 0)")"
  OUT="$(probe_read_line "$ROW")"
  printf '%s\n' "$OUT" | grep -qxF "$CTL_VOID" || die2 "REFUSED_SETUP: ctl $K1/$K2 must print the VOID verdict"
  # the VOID sentence itself contains those words; exclude it, then nothing conclusion-shaped may remain
  OUTX="$(printf '%s\n' "$OUT" | grep -vxF "$CTL_VOID")"
  if printf '%s\n' "$OUTX" | grep -qF 'CLAIM EXPECTATIONS'; then die2 "REFUSED_SETUP: failed ctl $K1/$K2 still printed claim expectations"; fi
  if printf '%s\n' "$OUTX" | grep -qF 'FALSIFIER'; then die2 "REFUSED_SETUP: failed ctl $K1/$K2 manufactured the falsifier"; fi
  if printf '%s\n' "$OUTX" | grep -qF 'PER-MODULE READINGS'; then die2 "REFUSED_SETUP: failed ctl $K1/$K2 still printed table-index readings"; fi
  if printf '%s\n' "$OUTX" | grep -qE 'expect (R|A) at index|r=[0-9]+ -> \^resolve_reason'; then die2 "REFUSED_SETUP: failed ctl $K1/$K2 still printed a table-index reading"; fi
done
ROWRF="$(probe_row_rf X 0)"; OUT="$(probe_read_line "$ROWRF")"
printf '%s\n' "$OUT" | grep -qF 'CONTROL VERDICT: CONTROL FAILED' || die2 "REFUSED_SETUP: RF row with failed controls must print the VOID verdict"
if printf '%s\n' "$OUT" | grep -qF "$RDX_NOCTX"; then die2 "REFUSED_SETUP: RF row with failed controls still printed the NOCTX reading"; fi
ROWRF2="$(probe_row_rf 5 X)"; printf '%s\n' "$(probe_read_line "$ROWRF2")" | grep -qF 'CONTROL VERDICT: CONTROL FAILED' || die2 "REFUSED_SETUP: RF ctl 5/X must print the VOID verdict"

# (11) aggregate readings through the shipped reader, plus direct parse_token reachability fixtures
#   (the closing-brace checks are unreachable on grammar-validated rows, so they are exercised here).
probe_parse_token 'X{r=0,a=U,s=T,al=0' t1 t2 t3 t4 && die2 "REFUSED_SETUP: token missing the closing brace was accepted"
probe_parse_token 'X{r=0,a=U,s=T,al=0}}' t1 t2 t3 t4 && die2 "REFUSED_SETUP: token with a doubled closing brace was accepted"
probe_parse_token 'X{a=U,s=T,al=0}' t1 t2 t3 t4 && die2 "REFUSED_SETUP: token without the r key was accepted"
probe_parse_token 'X{r=0,s=T,a=U,al=0}' t1 t2 t3 t4 && die2 "REFUSED_SETUP: out-of-order token was accepted"
probe_parse_token 'X{r=0,a=U,s=T,al=0} rs=resolve_reason_observation_incomplete' t1 t2 t3 t4 t5 || die2 "REFUSED_SETUP: the well-formed token was refused"
[ "$t1" = 0 ] && [ "$t2" = U ] && [ "$t3" = T ] && [ "$t4" = 0 ] || die2 "REFUSED_SETUP: parse_token returned wrong fields for the well-formed token"
printf '%s\n' "$(probe_read_line "$(probe_row_ok 0 X F F F F)")" | grep -qF "LANE FILE-REFUSAL" || die2 "REFUSED_SETUP: all-F frozen reading missing"
printf '%s\n' "$(probe_read_line "$(probe_row_ok 0 X A A A A)")" | grep -qF "census-name/path" || die2 "REFUSED_SETUP: all-A frozen reading missing"
printf '%s\n' "$(probe_read_line "$(probe_row_rf 0 X)")" | grep -qF "INGEST ADMISSION" || die2 "REFUSED_SETUP: ctx=RF frozen reading missing"
printf '%s\n' "$(probe_read_line "$(probe_row_ok 0 X R R R "$(probe_token 0 U T 16)")")" | grep -qF "FALSIFIER FIRES" || die2 "REFUSED_SETUP: the falsifier row must fire"

# (12) ESCAPING/NEWLINE and CONFLICT fixtures through the real validator AND the runtime caller
#   (addendum msg_32d414bd item 2). The producer cannot emit these bytes (header charset audit); the
#   instrument must still fail closed if they ever appear.
# (a) malformed: a NEWLINE inside the reason. Validator: refused. Runtime caller: the FAIL line no
#     longer closes with ')' at its end, so probe_verdict must classify it NONE (=> NO_OBSERVATION).
BADNL="WCVERIFY ctx=OK $W_OK arm=R field=R binder=R body=X{r=0,a=U,s=T,
al=0}"
probe_validate "$BADNL" >/dev/null; [ "$PROBE_VALIDATE_KIND" != OK ] || die2 "REFUSED_SETUP: a reason containing a newline was accepted by the validator"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=U,s=T,\nal=0})\n' "$PFN" "$W_OK" > "$FX/nl"
NL_TOK="$(probe_verdict "$FX/nl" "$PFN")"
case "$NL_TOK" in NONE|OTHER) :;; *) die2 "REFUSED_SETUP: the newline-injected log line was not failed closed by the runtime caller (got $NL_TOK)";; esac
[ "$NL_TOK" != EXIT3 ] || die2 "REFUSED_SETUP: a reason spanning lines must never be classified EXIT3"
# (b) conflicting: TWO typed outcome lines, both individually valid, different reasons. The exactly-
#     one-outcome gate must refuse (NONE) — a reading requires a unique outcome.
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=R)\nFAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=A field=A binder=A body=A)\n' "$PFN" "$W_OK" "$PFN" "$W_OK" > "$FX/conflict"
[ "$(probe_verdict "$FX/conflict" "$PFN")" = NONE ] || die2 "REFUSED_SETUP: two conflicting outcome lines were not refused"

# (13) reader cross-checks: constants BYTE-IDENTICAL to the generated grammars; behavior agrees.
RD_SRC="$RS/.probe_read-src.py"
PY_RE_OK="$(sed -n 's/^RE_OK = re\.compile(r"\(.*\)")$/\1/p' "$RD_SRC")"
PY_RE_RF="$(sed -n 's/^RE_RF = re\.compile(r"\(.*\)")$/\1/p' "$RD_SRC")"
PY_BOUND="$(sed -n 's/^BOUND = \([0-9]*\)$/\1/p' "$RD_SRC")"
[ "$PY_RE_OK" = "$PG_OK" ] || die2 "REFUSED_SETUP: the embedded reader RE_OK is not byte-identical to the generated grammar"
[ "$PY_RE_RF" = "$PG_RF" ] || die2 "REFUSED_SETUP: the embedded reader RE_RF is not byte-identical to the generated grammar"
[ "$PY_BOUND" = "$BOUND" ] || die2 "REFUSED_SETUP: the embedded reader BOUND is not the table-composed widest length"
printf '%s\n' "$(probe_row_ok 0 X R R R "$(probe_token 0 U T 0)")" > "$PGD/xrow1.txt"
printf '%s\n' "$(probe_row_ok 3 X F A R "$(probe_token 12 ? T 42)")" > "$PGD/xrow2.txt"
printf '%s\n' "$(probe_row_rf 0 X)" > "$PGD/xrow3.txt"
printf '%s\n' "$(probe_row_ok X X R R R R)" > "$PGD/xrow4.txt"
printf '%s\n' "$(probe_row_ok 0 0 R R R "$(probe_token 0 U T 0)")" > "$PGD/xrow5.txt"
printf '%s\n' "$(probe_row_rf X 0)" > "$PGD/xrow6.txt"
printf '%s\n' "$(probe_row_ok 5 X R R R R)" > "$PGD/xrow7.txt"
if command -v python3 >/dev/null 2>&1; then
  for xf in xrow1 xrow2 xrow3 xrow4 xrow5 xrow6 xrow7; do
    diff <(probe_read_line "$(cat "$PGD/$xf.txt")") <(python3 "$RD_SRC" "$PGD/$xf.txt") >/dev/null || die2 "REFUSED_SETUP: the embedded python reader disagrees with the shell reader on $xf"
  done
  echo "READER_PY=cross-checked (constants byte-identical; 7 rows behaviour-identical)"
else
  echo "READER_PY=absent-in-build-env (use the embedded copy at result/.probe_read.py offline)"
fi
rm -rf "$FX"
# B1 closure AT v6's failure point: the reader source must survive the fixture cleanup and still hash
# to the frozen constant (v6 died precisely here, at the probe_leg cp from the deleted $FX copy).
[ -f "$RS/.probe_read-src.py" ] || die2 "REFUSED_SETUP: the reader source did not survive the fixture cleanup (the v6 defect)"
[ "$(sha256sum "$RS/.probe_read-src.py" | cut -d' ' -f1)" = "$PROBE_READ_PY_SHA" ] || die2 "REFUSED_SETUP: the surviving reader source is not the frozen bytes"
# ---- v12 rs fixtures (parent msg_2d8d0262), through the REAL probe_validate ----
mkdir -p "$FX"   # $FX was cleaned after the READER_PY block; recreate for this fixture block (gline target)
W_RS64="$(probe_row_ok 0 X R R R "$(probe_token 23 '?' T "$TT_al_BOUND" "$TT_rs_BOUND")")"
pv_line 0 "$W_RS64"                                                   # widest row incl 64-char rs: ACCEPTED
pv_line 1 "${W_RS64}x"                                                # rs 65 chars: GRAMMAR refusal
pv_line 1 "${W_RS64/ rs=$TT_rs_BOUND/ rs=resolve_reason_Abc}"         # uppercase: GRAMMAR refusal
pv_line 1 "${W_RS64/ rs=$TT_rs_BOUND/ rs=}"                            # empty rs: GRAMMAR refusal
pv_line 1 "${W_RS64/ rs=$TT_rs_BOUND/ rs=-}"                           # absent-mark on the refused row: GRAMMAR refusal
pv_line 1 "WCVERIFY ctx=OK $W_OK arm=R rs=resolve_reason_a field=R rs=- binder=R rs=- body=R rs=-"   # named rs on a plain row: GRAMMAR refusal
pv_line 1 "WCVERIFY ctx=OK $W_OK arm=R field=R binder=R body=R"       # rs missing everywhere: GRAMMAR refusal
pv_line 0 "WCVERIFY ctx=RF$W_OK arm=- rs=- field=- rs=- binder=- rs=- body=- rs=-"                   # RF row: absent marks are its own row
# every declared lexeme of both vocabularies is a grammar-valid rs (real validator sweep)
for _n in "${CENSUS_LANE37[@]}" "${CENSUS_CHAIN24[@]}"; do
  pv_line 0 "$(probe_row_ok 0 X R R R "$(probe_token X '-' T 0 "$_n")")"
done
echo "fixtures: all held (incl rs class, refusals, RF absent row, and the 61-name declared-form sweep)"

# ---- ENTRY: the single diagnostic file on the probe commit (blob gated after checkout) ----
PROBE_PATH="src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag"
PFN=wc_verdict_probe_reports

# ---- PRE-REGISTERED EXPECTATION (intended observation, written 2026-11-06 BEFORE this packet ever ran; never edited after output) ----
# The witness is a MEASUREMENT INSTRUMENT: it always returns ProcessExit code 3, so exactly one EXIT3 line is pre-registered.
# Its content is read OFFLINE against the frozen reading table in the header; PASS/bare-FAIL/other-code outcomes are setup refusals.
declare -A EXPECT
EXPECT["A.probe.$PFN"]=EXIT3

declare -A OBS
MISSING=0
DEVIATIONS=0

probe_run() { # BIN : one claim_batch invocation of the probe entry; frame contracts identical to v4 run_entry
  local BIN="$1"
  local L="$R/claims-A-probe.log" RC VE VD
  snap "run A probe"
  timeout -k 30 900 "$BIN" --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$PROBE_PATH" --functions "$PFN" > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[A probe]=$RC"; mem "after_run_A-probe"; judge_oom "run A probe"
  rc_in_contract "$RC" || die2 "leg A probe: exit status $RC is out of contract (0/1/2 only; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg A probe: no single '[process-partition] total_ms=<digits>' completion line; batch incomplete whatever verdict lines printed"
  echo "verdict lines (A probe):"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"
  # OUTCOME-JOIN LAW (parent msg_4477b24b, keen-otter msg_2356fbe3; packet-wide law, applied here to the
  # probe's single-fn roster): collect EVERY line matching '^[A-Z]+ ' in the batch log. The SET of names on
  # those lines must EQUAL the pre-registered roster {wc_verdict_probe_reports}: no foreign name, no missing
  # name; EXACTLY ONE outcome line for the roster name; its keyword must be FAIL (the instrument's only
  # surviving form is the typed code-3 FAIL — bare PASS/FAIL, reasoned forms of any other code, and every
  # other keyword (SKIPPED, UNOBSERVED, PANICKED, ...) are refused here, exit 2); duplicates are refused by
  # the count and by probe_verdict's single-line rule. The batch rc is then cross-checked against THAT
  # validated set only: rc 1 (a FAIL is present in the roster; no PASS case exists for this instrument).
  local AJ_N AJ_KW AJ_NAME AJ_ERR
  local -A PROBE_ROSTER=([$PFN]=1)   # the roster, as a set; function-local (outcome-join law addendum, parent msg_045cfc4c; declare-name audit msg_3152d6ec)
  AJ_N="$(grep -acE '^[A-Z]+ ' "$L")"; AJ_ERR=""
  if [ "$AJ_N" != "${#PROBE_ROSTER[@]}" ]; then
    AJ_ERR="expected exactly one '^[A-Z]+ ' outcome line (roster = {$PFN}), found $AJ_N; foreign names, duplicates and stray ALLCAPS lines are refused"
  else
    AJ_KW="$(grep -aE '^[A-Z]+ ' "$L" | awk 'NR==1{print $1}')"
    AJ_NAME="$(grep -aE '^[A-Z]+ ' "$L" | awk 'NR==1{print $2}')"
    if ! [[ "$AJ_NAME" =~ ^[a-z0-9_]+$ ]]; then AJ_ERR="outcome name '$AJ_NAME' is not a single [a-z0-9_] token (multi-token outcome lines are refused)"
    elif [ -z "${PROBE_ROSTER[$AJ_NAME]+x}" ]; then AJ_ERR="foreign name '$AJ_NAME' on the only outcome line; the roster is exactly {$PFN} (exact set membership, not substring)"
    elif [ "$AJ_KW" != FAIL ]; then AJ_ERR="outcome keyword '$AJ_KW' is not the roster's single surviving FAIL form (no other keyword is admitted)"
    fi
  fi
  if [ -n "$AJ_ERR" ]; then
    { echo "outcome-join-law: $AJ_ERR"; grep -aE '^[A-Z]+ ' "$L"; } > "$R/probe-nonreading.txt" 2>&1
    die2 "leg A probe: OUTCOME-JOIN LAW: $AJ_ERR; raw lines preserved in claims-A-probe.log and probe-nonreading.txt"
  fi
  probe_verdict "$L" "$PFN" > "$R/.probe.vd" || true; VD="$(cat "$R/.probe.vd")"; echo "A probe $PFN=$VD"
  [ "$PROBE_TOKEN" = "$VD" ] || die2 "leg A probe: classifier token disagreement ($VD vs $PROBE_TOKEN)"
  if [ "$VD" = NONE ]; then MISSING=1; die2 "leg A probe: '$PFN' has no SINGLE outcome line (absent, duplicate, or unclassifiable)"; fi
  if [ "$VD" = EXIT3 ]; then
    RAW3="$(grep -aE "^FAIL $PFN \(returned ProcessExit::ExitFailure, code 3: " "$L")"
    echo "RAW: $RAW3"
    probe_validate "$PROBE_REASON" || die2 "leg A probe: NO_OBSERVATION: the reason fails the anchored grammar or the ${BOUND}-char bound; raw line preserved in claims-A-probe.log and here: $RAW3"
    printf '%s\n' "$PROBE_REASON" > "$R/probe-reason.txt" || die2 "cannot write probe-reason.txt"
    echo "PROBE_REASON: $PROBE_REASON"
    echo "PROBE_REASON_BYTES=${#PROBE_REASON}"
    VE=1
  else
    # ANY non-EXIT3 token (EXIT4 invalid reading, UNOBSERVED, PASS, BARE, OTHER incl. code 1/NotBool/malformed):
    # there is NO WCVERIFY observation. Recorded as an event, then NO OBSERVATION (exit 2). No domain result is inferred.
    { echo "token=$VD claim_batch_rc=$RC"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"; } > "$R/probe-nonreading.txt" 2>&1
    die2 "leg A probe: NO WCVERIFY READING (token $VD, claim_batch rc $RC); raw lines preserved in claims-A-probe.log and probe-nonreading.txt; no domain result follows"
  fi
  # the only survivors are validated EXIT3 rows: the batch rc MUST be consistent with a FAIL-class typed exit (claim_batch.rs sets any_failed)
  [ "$RC" = 1 ] || die2 "leg A probe: claim_batch returned $RC, but a typed code-3 FAIL class requires rc 1"
  OBS["A.probe.$PFN"]="EXIT3"
}

probe_leg() { # fetch (600), gates (parent/delta/blobs), build (2400, own target dir), one entry (900), post-checks
  local HT BIN BIN_SHA HB
  git init -q "wt-A" && cd "wt-A" || die2 "git init wt-A"
  timeout -k 30 600 git fetch -q --depth 2 https://github.com/gunb-ai/gunbc "$PIN_P" || die2 "leg A: fetch $PIN_P failed or timed out (600 s cap)"
  git checkout -q FETCH_HEAD || die2 "leg A: checkout failed"
  [ "$(git rev-parse HEAD)" = "$PIN_P" ] || die2 "leg A: HEAD is not the probe pin"
  HT="$(git rev-parse 'HEAD^{tree}')" || die2 "leg A: cannot read the tree of HEAD"
  [ "$HT" = "$TREE_P" ] || die2 "leg A: REFUSED_SETUP: tree is $HT, not the pinned $TREE_P"
  tree_clean . || die2 "leg A: REFUSED_SETUP: checkout is not clean"
  # the one-file-delta gates: parent is exactly A, and the name-only diff to A is exactly the probe path.
  # NO second fetch: the depth-2 fetch of the single-parent probe commit already carries A. The parent commit and
  # ITS TREE must be present locally; fail closed (exit 2) if not.
  [ "$(git rev-parse 'HEAD^1')" = "$PIN_A" ] || die2 "leg A: REFUSED_SETUP: the probe commit's parent is not the A pin $PIN_A"
  git cat-file -e "$PIN_A^{commit}" 2>/dev/null || die2 "leg A: REFUSED_SETUP: parent commit $PIN_A is absent locally (the single depth-2 fetch must provide it)"
  git cat-file -e "$PIN_A^{tree}"   2>/dev/null || die2 "leg A: REFUSED_SETUP: the tree of parent $PIN_A is absent locally"
  local DN; DN="$(git diff --name-only "$PIN_A" HEAD 2>/dev/null | tr '\n' '|')"
  [ "$DN" = "$PROBE_PATH|" ] || die2 "leg A: REFUSED_SETUP: the delta from A is '$DN', not exactly the one probe path"
  HB="$(git hash-object "$PROBE_PATH" 2>/dev/null)" || die2 "leg A: cannot hash $PROBE_PATH"
  [ "$HB" = "$BLOB_PROBE" ] || die2 "leg A: $PROBE_PATH blob is $HB, not the pinned $BLOB_PROBE"
  HB="$(git hash-object src/v2/test/claim/namespace_xl0/wildcard_arm_resolve.dag 2>/dev/null)" || die2 "leg A: cannot hash the frozen wc claim file"
  [ "$HB" = "$BLOB_WC" ] || die2 "leg A: the wc claim file blob is $HB, not the frozen $BLOB_WC"
  grep -q "data reason_bound: Int = $BOUND" "$PROBE_PATH" 2>/dev/null || die2 "leg A: REFUSED_SETUP: the fetched probe's reason_bound constant is not the packet-composed BOUND=$BOUND (one authority)"
  echo "leg A probe pin ok head=$PIN_P tree=$HT parent=$PIN_A delta=1 blob=$BLOB_PROBE wc=$BLOB_WC bound=$BOUND"
  snap "build-A"
  CARGO_TARGET_DIR="$R/target-A" CARGO_BUILD_JOBS=32 timeout -k 30 2400 cargo build -j 32 --locked --release -p v1-compiler --bin claim_batch > "$R/build-A.log" 2>&1
  local RC=$?; judge_oom "build-A"
  [ "$RC" = 0 ] || die2 "leg A: build failed rc=$RC (see build-A.log)"
  BIN="$R/target-A/release/claim_batch"
  BIN_SHA="$(hash_of "$BIN")" || die2 "leg A: cannot hash the claim_batch binary"
  echo "leg A claim_batch sha256=$BIN_SHA"
  probe_run "$BIN"
  tree_clean . || die2 "leg A: post-run git status --porcelain is not empty"
  local BIN_NOW; BIN_NOW="$(hash_of "$BIN")" || die2 "leg A: post-run cannot hash the binary"; [ "$BIN_NOW" = "$BIN_SHA" ] || die2 "leg A: claim_batch binary sha changed during the run"
  cd "$R" || die2 "leg A: cd back to OUT"
}

# ---- (10) RUNTIME-PATH fixtures, driven through probe_run itself with a STUB claim_batch (canned log + chosen rc).
# EXPECTED, stated before any run: valid3&rc1 -> 0; valid3&rc0 (batch-rc mismatch) -> 2; exit4 -> 2; malformed reason -> 2;
# NotBool -> 2; code 1 -> 2; PASS(rc0) -> 2 (no WCVERIFY reading); bare FAIL(rc1) -> 2. All non-validated paths are exit 2.
# v4 ARGV CONTRACT (msg_b02cda13): every stub asserts argv == "--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry <probe path> --functions wc_verdict_probe_reports";
# any dropped or altered argument -> stub prints STUB_ARGV_REFUSAL and exits 99 -> claim_batch rc 3 with no outcome/completion lines -> exit 2 (NO OBSERVATION) -> the fixture gate FAILS (a dropped or altered flag can never pass fixtures).
# argvok  : exact-argv valid case -> 0;  argvdropfn: stub expects argv WITHOUT --functions while probe_run passes it -> mismatch -> 3 -> exit 2;
# argvbadsrc: --source-root dag altered -> 3 -> exit 2;  argvwrongfn: --functions value WRONG -> 3 -> exit 2;
# fnless: probe_run variant invoked WITHOUT --functions against the full-contract stub -> stub refusal -> fixture gate fires (parent-mandated case).
CL='[process-partition] total_ms=1000 pre_run_ms=0 args_ms=0 index_build_ms=0 edge_index_warm_ms=0 output_policy_ms=0 roster_ms=0 published_mock_precompute_ms=0 entries_ms=0 reporting_ms=0 entries_resolve_ms=0 entries_witness_ms=0'
RT_CONTRACT="--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH --functions $PFN"
# RT_WANT (env, default RT_CONTRACT) sets the argv string this stub must see; any positional arg is a log line.
mkstub() { local kind="$1" erc="$2" want="${RT_WANT-$RT_CONTRACT}"; unset RT_WANT 2>/dev/null || true; shift; shift; : > "$RTD/rt-log.$kind"; for ln in "$@"; do printf '%s\n' "$ln" >> "$RTD/rt-log.$kind"; done
  printf '#!/bin/bash\nif [ "$*" != "%s" ]; then echo "STUB_ARGV_REFUSAL expected=[%s] got=[$*]"; exit 3; fi\ncat "%s"\nexit %s\n' "$want" "$want" "$RTD/rt-log.$kind" "$erc" > "$RTD/rt-cb-$kind"; chmod +x "$RTD/rt-cb-$kind"; }
pr_run() { ( R="$RTD/rt-$1"; mkdir -p "$R"; RAW="$R/oom-raw.log"; OOM_SEEN=""; declare -A OBS; MISSING=0; DEVIATIONS=0
  probe_run "$RTD/rt-cb-$1" >/dev/null 2>&1 ); echo "$?"; }
V3L="FAIL $PFN (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=X{r=0,a=?,s=T,al=1} rs=resolve_reason_observation_incomplete)"
mkstub valid3 1 "$V3L" "$CL"
mkstub rc0   0 "$V3L" "$CL"
mkstub e4    1 "FAIL $PFN (returned ProcessExit::ExitFailure, code 4: WCVERIFY BOUND_VIOLATED len=140)" "$CL"
mkstub mal   1 "FAIL $PFN (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=Q field=R binder=R body=R)" "$CL"
mkstub nb    1 "FAIL $PFN (returned \`Int\`, not Bool; --claim-run entries must return Bool)" "$CL"
mkstub c1    1 "FAIL $PFN (returned ProcessExit::ExitFailure, code 1: boom)" "$CL"
mkstub pass  0 "PASS $PFN" "$CL"
mkstub bare  1 "FAIL $PFN" "$CL"
# rxpath (v8): an r=X row — the shape the v5 run printed (modulo the new control fields) — driven
#   through the REAL probe_run with batch rc 1. Expected 0 (a validated EXIT3 under the table
#   grammar IS a reading). The stub still asserts the full argv.
V5RXL="FAIL $PFN (returned ProcessExit::ExitFailure, code 3: $(probe_row_ok 0 X "$(probe_token X - T 0)" "$(probe_token X - T 0)" "$(probe_token X - T 0)" "$(probe_token X - T 0)"))"
mkstub rxpath 1 "$V5RXL" "$CL"
[ "$(pr_run valid3)" = 0 ] || die2 "REFUSED_SETUP: runtime fixture valid3&rc1 must exit 0"
[ "$(pr_run rc0)"   = 2 ] || die2 "REFUSED_SETUP: runtime fixture batch-rc mismatch must exit 2"
[ "$(pr_run e4)"    = 2 ] || die2 "REFUSED_SETUP: runtime fixture EXIT4 (invalid reading) must exit 2"
[ "$(pr_run mal)"   = 2 ] || die2 "REFUSED_SETUP: runtime fixture malformed reason must exit 2"
[ "$(pr_run nb)"    = 2 ] || die2 "REFUSED_SETUP: runtime fixture NotBool must exit 2"
[ "$(pr_run c1)"    = 2 ] || die2 "REFUSED_SETUP: runtime fixture code 1 must exit 2"
[ "$(pr_run pass)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture PASS must exit 2 (no WCVERIFY reading)"
[ "$(pr_run bare)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture bare FAIL must exit 2 (no WCVERIFY reading)"
# rxpath (v8): the r=X shape must be a VALIDATED reading end-to-end, and the reason must be written.
[ "$(pr_run rxpath)" = 0 ] || die2 "REFUSED_SETUP: runtime fixture r=X row must be a validated reading (exit 0)"
[ -f "$RTD/rt-rxpath/probe-reason.txt" ] || die2 "REFUSED_SETUP: runtime fixture r=X did not write probe-reason.txt"
mkstub argvok 1 "$V3L" "$CL"
RT_WANT="--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH" mkstub argvdropfn 1 "$V3L" "$CL"
RT_WANT="--claim-run --source-root WRONG --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH --functions $PFN" mkstub argvbadsrc 1 "$V3L" "$CL"
RT_WANT="--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH --functions NOT_THE_PROBE_FN" mkstub argvwrongfn 1 "$V3L" "$CL"
# fnless: the parent-mandated case — probe_run invoked WITHOUT --functions must fail the fixtures.
pr_run_fnless() { ( R="$RTD/rt-fnless"; mkdir -p "$R"; RAW="$R/oom-raw.log"; OOM_SEEN=""; declare -A OBS; MISSING=0; DEVIATIONS=0
  timeout 900 "$RTD/rt-cb-argvok" --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$PROBE_PATH" > "$R/claims-A-probe.log" 2>&1
  RC=$?; echo "claim_batch_exit[A probe]=$RC" >/dev/null
  rc_in_contract "$RC" "A probe" || true; completed "$R/claims-A-probe.log" || true
  probe_verdict "$R/claims-A-probe.log" "$PFN" > "$R/.probe.vd" || true
  [ "$RC" = 1 ] || exit 2 ); echo "$?"; }
# OUTCOME-JOIN LAW fixtures (parent msg_4477b24b), through the REAL probe_run with stub log and rc.
# Expected exits, pre-registered BEFORE this block was ever run: every foreign-name line, every second
# outcome line, SKIPPED/UNOBSERVED keywords, a missing roster name and an rc that disagrees with the
# validated set give exit 2; the valid typed code-3 line alone at rc 1 gives exit 0 (already valid3).
mkstub j_foreign_pass 1 "$V3L" "PASS other_fn" "$CL"
mkstub j_foreign_fail 1 "$V3L" "FAIL other_fn" "$CL"
mkstub j_foreign_skip 1 "$V3L" "SKIPPED foo" "$CL"
mkstub j_dup          1 "$V3L" "$V3L" "$CL"
mkstub j_unobserved   1 "UNOBSERVED $PFN" "$CL"
mkstub j_missing      1 "$CL"
mkstub j_multiword    1 "PASS a b" "$CL"
mkstub j_spacefail    1 "FAIL $PFN extra" "$CL"
[ "$(pr_run j_foreign_pass)" = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture foreign PASS must exit 2"
[ "$(pr_run j_foreign_fail)" = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture foreign FAIL must exit 2"
[ "$(pr_run j_foreign_skip)" = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture foreign SKIPPED must exit 2"
[ "$(pr_run j_dup)"          = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture duplicate roster line must exit 2"
[ "$(pr_run j_unobserved)"   = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture UNOBSERVED keyword must exit 2"
[ "$(pr_run j_missing)"      = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture missing roster name must exit 2"
# addendum (parent msg_045cfc4c): 'PASS a b' must exit 2 via exact-token membership (the ERS v4 substring defect);
# 'FAIL <roster-name> extra' exits 2 downstream (not the typed code-3 form) even though the name is a member.
[ "$(pr_run j_multiword)"    = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture 'PASS a b' (multi-token, foreign) must exit 2"
[ "$(pr_run j_spacefail)"    = 2 ] || die2 "REFUSED_SETUP: outcome-join fixture 'FAIL name extra' must exit 2 (not the typed code-3 form)"
# reasoned FAIL (code 1), bare PASS, bare FAIL and rc-disagreement were already real-caller fixtures above
# (c1, pass, bare, rc0): each exits 2; PASS at rc 0 with the roster name present is a NO_OBSERVATION under
# the frozen expectation (the instrument always returns code 3), never an acceptance.
[ "$(pr_run argvok)"      = 0 ] || die2 "REFUSED_SETUP: runtime fixture argvok (exact argv) must exit 0"
[ "$(pr_run argvdropfn)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture argvdropfn (stub expects argv WITHOUT --functions) must exit 2 via stub argv refusal (exit 3)"
[ "$(pr_run argvbadsrc)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture argvbadsrc (altered --source-root) must exit 2 via stub argv refusal"
[ "$(pr_run argvwrongfn)" = 2 ] || die2 "REFUSED_SETUP: runtime fixture argvwrongfn (WRONG --functions value) must exit 2 via stub argv refusal"
[ "$(pr_run_fnless)"      = 2 ] || die2 "REFUSED_SETUP: runtime fixture fnless (invoked WITHOUT --functions) must fire the fixture gate (stub argv refusal -> exit 2)"
grep -aq STUB_ARGV_REFUSAL "$RTD/rt-argvdropfn/claims-A-probe.log" 2>/dev/null || die2 "REFUSED_SETUP: argvdropfn did not exercise the stub argv refusal"
grep -aq STUB_ARGV_REFUSAL "$RTD/rt-argvwrongfn/claims-A-probe.log" 2>/dev/null || die2 "REFUSED_SETUP: argvwrongfn did not exercise the stub argv refusal"
grep -aq STUB_ARGV_REFUSAL "$RTD/rt-fnless/claims-A-probe.log" 2>/dev/null || die2 "REFUSED_SETUP: fnless did not exercise the stub argv refusal"
[ -s "$RTD/rt-pass/probe-nonreading.txt" ] || die2 "REFUSED_SETUP: the PASS event was not preserved in probe-nonreading.txt"
# ---- v12 reader classification fixtures through the REAL reader source (msg_6d50975a discipline) ----
_mkrow() { printf 'WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=R rs=- field=R rs=- binder=R rs=- body=X{r=%s,a=-,s=T,al=0} rs=%s\n' "$1" "$2" > "$RTD/rt-rs-row.txt"; }
RTPY="$RS/.probe_read-src.py"   # the frozen reader source (sha-checked against PROBE_READ_PY_SHA above)
_mkrow X resolve_reason_forwarded_parameter_kind_differs
[ "$(python3 "$RTPY" "$RTD/rt-rs-row.txt" | grep -c 'OUTSIDE the 24; IN lane vocabulary')" = 1 ] || die2 "REFUSED_SETUP: rs outside-table classification failed"
_mkrow X resolve_reason_unbound_symbol
[ "$(python3 "$RTPY" "$RTD/rt-rs-row.txt" | grep -c 'CONTRADICTION DATA')" = 1 ] || die2 "REFUSED_SETUP: rs in-chain-with-X contradiction datum not recorded"
_mkrow X resolve_reason_not_a_real_symbol_at_all
[ "$(python3 "$RTPY" "$RTD/rt-rs-row.txt" | grep -c 'NOT IN EITHER VOCABULARY')" = 1 ] || die2 "REFUSED_SETUP: unknown-lexeme classification failed"
_mkrow 13 resolve_reason_qualified_target_identity_unrepresentable
[ "$(python3 "$RTPY" "$RTD/rt-rs-row.txt" | grep -c 'CONTRADICTION DATA')" = 0 ] || die2 "REFUSED_SETUP: consistent chain row must not print contradiction"
[ "$(python3 "$RTPY" "$RTD/rt-rs-row.txt" | grep -c 'chain arm')" = 1 ] || die2 "REFUSED_SETUP: in-chain reading line missing"
python3 "$RTPY" "$RTD/rt-rs-row.txt" | grep -q 'rs=- (absent' || die2 "REFUSED_SETUP: plain-row absent rs reading missing"
rm -f "$RTD/rt-rs-row.txt"
echo "runtime-path fixtures: all held (incl rs classification through the real reader)"
rm -rf "$RTD"

# the embedded offline reader becomes a run-time artifact in $R, re-hashed against the frozen constant (v6).
mkdir -p "$R" || die2 "cannot create $R before the reader copy"
cp "$RS/.probe_read-src.py" "$R/.probe_read.py" 2>/dev/null || die2 "REFUSED_SETUP: the embedded reader source is missing"
[ "$(sha256sum "$R/.probe_read.py" | cut -d' ' -f1)" = "$PROBE_READ_PY_SHA" ] || die2 "REFUSED_SETUP: the embedded offline reader is not the frozen bytes"
# v8: $RTD is removed after its fixtures; $RS is removed once its only artifact ($R/.probe_read.py) is copied and gated. Neither is an artifact.
rm -rf -- "$RS" || die2 "cannot remove the non-artifact reader scratch dir"

mem start
probe_leg

# ---- the observation, preserved verbatim; the READING is offline against the frozen table ----
echo "OBSERVED leg A probe:"
for k in "${!OBS[@]}"; do echo "  $k = ${OBS[$k]} (pre-registered ${EXPECT[$k]})"; done | sort
# completeness gates (v4 semantics, single row): exactly 1 observation, this leg's prefix only, MISSING == 0.
[ "${#OBS[@]}" = 1 ] || die2 "not every expected witness has an observed verdict (${#OBS[@]} of 1)"
for k in "${!OBS[@]}"; do case "$k" in A.*) ;; *) die2 "OBS carries a foreign key: $k";; esac; done
[ "$MISSING" -gt 0 ] && die2 "an expected witness has no observed verdict"
[ -f "$R/probe-reason.txt" ] || [ "$DEVIATIONS" -gt 0 ] || die2 "the EXIT3 observation is required for a valid reading"
echo "PROBE_MISSING=$MISSING"
echo DONE
# exit 1 is unused by v3: every non-validated path died with exit 2 (NO OBSERVATION) above.
exit 0
