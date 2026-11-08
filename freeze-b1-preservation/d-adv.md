**Diff: v3a -> LEG A (v4a).** The complete change set: job-name rename (header ledger, `JOB=`, scope fixtures), the restored per-leg completeness gates block replacing the unconditional `JOIN_MISSING=0` literal (OBS count == 34, this-leg key prefix check, MISSING == 0, JOIN_MISSING printed from the check), the header byte-count update + v4 change note, and job-name mentions in the ORDER/PAIRING and offline-evaluation sentences. No rule, expectation, roster, fixture or gate otherwise changes. v3a canonical: sha `490831c32c631f09a470523cecdab5a43a1adf788bc18e3ebf0f175019d017e7`, 38,616 B, 368 lines (never launched).

```diff
--- packet-v3a.sh	2026-11-06 07:32:48.141420344 +0000
+++ packet-v4a.sh	2026-11-06 07:36:16.155867188 +0000
@@ -68,26 +68,27 @@
 #   CPUQuota=3200% (cpu.max "3200000 100000") PAIRED with cargo -j 32 / CARGO_BUILD_JOBS=32 (nproc=128; the ~2100 s uncapped cold build would exceed a smaller
 #     cap; quota and -j move together). MemoryHigh=28G (30064771072): throttle 4G below the max. MemoryMax=32G (34359738368): a CEILING, UNMEASURED at -j 32;
 #     a breach is an OOM, exit 2, never a verdict. MemorySwapMax=0. TasksMax=4096 (pids.max).
-#   Ledger rows under the NEW job: JOB=sleek-seal-226-rtp-ce-diag-v3a; JOB-res (cap MEMMAX), JOB-drv (cap 1), JOB (cap 0); units JOB.scope and calmcrab-drv-JOB.service.
+#   Ledger rows under the NEW job: JOB=sleek-seal-226-rtp-ce-diag-v4a; JOB-res (cap MEMMAX), JOB-drv (cap 1), JOB (cap 0); units JOB.scope and calmcrab-drv-JOB.service.
 # TIMEOUTS AND THE OUTER WALL, stated as capped sums. v17 MEASURED at B in this frame: build 224 s; cpr entry 340 s (15 claims, index_build 154 s); rtp 666 s
 #   (17 claims); ce 551 s (13 claims); cgroup peak 13.69 GiB, oom 0. Entry cost is dominated by the ~155 s shared index build plus ~20-40 s per claim, so a
 #   2-4 name roster is an ESTIMATE of 250-350 s; 900 s per entry run is generous. CAPPED SUM per leg: fetch 600 + build 2400 + 3 entries x 900 = 5700;
 #   this leg: 5700 s. MARGIN 300 s for the uncapped parts (git init, fixtures, gates, post-run re-hash): OUTER WALL RuntimeMaxSec=6000;
 #   driver-service RuntimeMaxSec=6300 (wall + 300). The installed launcher calmcrab-udrv v4 (855920ac) bounds RUNTIME_S to 1..7200 (the paired wall 11700 was
 #   refused pre-scope; the v2 attempt ran NO work and produced NO observation).
-# - ORDER + PAIRING (keen-otter ruling msg_50b75473, via sleek-owl-397): leg A (v3a) runs first; leg B (v3b) launches ONLY after A is
+# - ORDER + PAIRING (keen-otter ruling msg_50b75473, via sleek-owl-397): leg A (v4a) runs first; leg B (v4b) launches ONLY after A is
 #   terminal and reconciled. Pairing the results needs BOTH valid observations, under the same limits on causal conclusions
 #   (R1..R12 are diagnostic hypotheses, not established ownership; a FAIL on both pins does not prove the same mechanism and does not
 #   exonerate the PR; one positive control does not establish the whole failure vector) and on the B-vs-v17 comparison.
 # - This packet's launcher argument RUNTIME_S is 6000, which FITS the installed launcher calmcrab-udrv v4 bound RUNTIME_S 1..7200.
-# - Byte counts, stated exactly: this packet is 38616 bytes, 368 lines (wc -l); bash -n clean. The frozen decision table R1..R12 is UNCHANGED by the split; its
-#   evaluation MOVES to the offline reporting step joining the two preserved matrices (v3a = leg A, v3b = leg B). No rule, expectation or roster changes. CAPPED SUMS ARE NOT THE WORST-CASE WHOLE-ROW DURATION; RuntimeMaxSec IS A HARD CUTOFF; a row the wall
+# - Byte counts, stated exactly: this packet is 39380 bytes, 374 lines (wc -l); bash -n clean.
+# - v4 change (tidy-ant-251 byte review): the v2 terminal completeness gates are RESTORED per leg (see the gates block before JOIN/DONE): OBS count == 34, every OBS key carries this leg prefix, MISSING == 0, and JOIN_MISSING prints from the actual check. Nothing else changes vs v3. The frozen decision table R1..R12 is UNCHANGED by the split; its
+#   evaluation MOVES to the offline reporting step joining the two preserved matrices (v4a = leg A, v4b = leg B). No rule, expectation or roster changes. CAPPED SUMS ARE NOT THE WORST-CASE WHOLE-ROW DURATION; RuntimeMaxSec IS A HARD CUTOFF; a row the wall
 #   cuts is incomplete and is NO OBSERVATION whatever it printed. An entry that times out is NO OBSERVATION (rc 124), never a FAIL. One attempt.
 # RAW BOUNDARY RETENTION: every memory.events text behind an OOM decision (before and after each build and each entry run) is appended, timestamped, to
 #   OUT/oom-raw.log before it is parsed; a failed write refuses (exit 2). OUT keeps build-A.log, build-B.log, claims-<leg>-<tag>.log, oom-raw.log.
 # The counter re-reads (memory.peak, memory.events) are separate reads, not an atomic snapshot; none is claimed.
 set -uo pipefail
-JOB=sleek-seal-226-rtp-ce-diag-v3a
+JOB=sleek-seal-226-rtp-ce-diag-v4a
 UNIT="$JOB.scope"
 PIN_A=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1; TREE_A=2549a8e6545468a55fd8de3d0a082049bae19b32
 PIN_B=495da688d0ce8490c805c32f193ee99bbe626b83; TREE_B=29ca30b0dc27874a36256782c6214cbfd87b7934
@@ -167,7 +168,7 @@
 # (3) exit contract
 [ "$(r rc_in_contract 0) $(r rc_in_contract 1) $(r rc_in_contract 2) $(r rc_in_contract 124) $(r rc_in_contract 137) $(r rc_in_contract 3)" = "accepted accepted accepted refused refused refused" ] || die2 "REFUSED_SETUP: exit-contract fixture misfire"
 # (4) scope glob literal suffix, restore cleanliness
-[ "$(r scope_ok '0::/a/sleek-seal-226-rtp-ce-diag-v3a.scope') $(r scope_ok '0::/a/sleek-seal-226-rtp-ce-diag-v3aXscope') $(r scope_ok '0::/a/xsleek-seal-226-rtp-ce-diag-v3a.scope') $(r scope_ok '0::/a/sleek-seal-226-rtp-ce-diag-v3a.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
+[ "$(r scope_ok '0::/a/sleek-seal-226-rtp-ce-diag-v4a.scope') $(r scope_ok '0::/a/sleek-seal-226-rtp-ce-diag-v4aXscope') $(r scope_ok '0::/a/xsleek-seal-226-rtp-ce-diag-v4a.scope') $(r scope_ok '0::/a/sleek-seal-226-rtp-ce-diag-v4a.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
 git init -q "$FX/g" && : > "$FX/g/f" || die2 "fixture git init"
 git -C "$FX/g" add f && git -C "$FX/g" -c user.name=fx -c user.email=fx@fx commit -qm fx || die2 "fixture commit"
 A="$(r tree_clean $FX/g)"; : > "$FX/g/u"; B="$(r tree_clean $FX/g)"
@@ -362,7 +363,12 @@
 FA="$(g A.rtp.an_else_less_if_that_falls_through_still_refuses_as_unlowered)"
 C1="$(g A.ce.a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds)"; C2="$(g A.ce.the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds)"
 echo "LEG A HEALTH: wc_all_pass=$WOK positive=$PA unbound_twin=$UA fallthrough=$FA ce_pair=$C1,$C2"
-echo "JOIN_MISSING=0 FAILS=$DEVIATIONS"
+# completeness gates (per leg, restored from v2): every expected witness of THIS leg observed,
+# exactly 34 rows, every OBS key carries this leg prefix (no foreign-leg rows), and MISSING == 0.
+[ "${#OBS[@]}" = 34 ] || die2 "not every expected witness has an observed verdict (${#OBS[@]} of 34)"
+for k in "${!OBS[@]}"; do case "$k" in A.*) ;; *) die2 "OBS carries a foreign-leg key: $k";; esac; done
+[ "$MISSING" -gt 0 ] && die2 "an expected witness has no observed verdict"
+echo "JOIN_MISSING=$MISSING FAILS=$DEVIATIONS"
 echo DONE
 [ "$DEVIATIONS" -gt 0 ] && exit 1
 exit 0
```
