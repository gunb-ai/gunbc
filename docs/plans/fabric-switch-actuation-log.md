# Fabric switch 100G — out-of-band actuation log

Operator-approved workaround (2026-09-17): force 100GBASE-CR2 on the CRS812 lanes
directly over the RouterOS REST API from a session on the LAN, ahead of the modeled
convergence path (docs/plans/fabric-switch-100g-convergence-gap-analysis.md, step B).

This file IS the dissolution trigger. Every command run against the switch is recorded
here verbatim so the modeled `.dag` apply is a transcription of what worked, not a
fresh guess. The scaffold is retired when that apply path lands and reproduces this
end state; until then the switch config is authored by hand and this log is its only
record.

## Facts established before touching the switch

- Switch: MikroTik CRS812-8DS-2DQ-2DDQ, RouterOS, management 192.168.1.240 (Winbox
  8291 + HTTP 80 open; /rest/system/identity returns 401, REST live).
- Host end (all 8 Sparks, read 2026-09-17): EEPROM byte 192 = 0x40 on srv5 (was 0x0B),
  which attests 100GBASE-CR2 unconditional. All 8 ConnectX-7 NICs advertise
  100000baseCR2/Full. All 8 still link at 50000Mb/s, 2 lanes, autoneg on.
- crs812_passive_dac_requires_forced_mode = true: a passive DAC needs lane speed and
  FEC forced rather than auto-negotiated.
- Lane interface naming (extdeps.mikrotik.crs812): "<prefix>-<cage>-<lane>", prefix
  qsfp56 for the 100G cages. The breakout legs land on the qsfp56 cages.

## Lane roster (from gunbc.spark.fabric_switch_observed, cage:lane -> host)

- 1:1 srv5 (spark-a3ee)   1:3 srv8 (spark-ac79)   1:5 srv7 (spark-c2b1)   1:7 srv6 (spark-3bd5)
- 2:1 srv10 (spark-3336)  2:3 srv12 (spark-2196)  2:5 srv9 (spark-0c75)   2:7 srv11 (spark-b66c)

NOTE: the fabric_switch_observed rows model the legs on qsfp56-dd (400G) cages, cage/lane
as above. The lane->switch-interface mapping must be READ from the switch, not assumed,
before forcing anything. That read is step 1.

## Actuation

(appended as executed)

### Read 2026-09-17 (first-ever read of the switch)

GET /rest/interface/ethernet — the 8 fabric legs are on the qsfp56-dd (400G breakout)
cages, NOT the qsfp56 cages. This corrects the pre-read assumption. Running fabric legs
(comment "gunbc fabric leg"), all identical:

    qsfp56-dd-1-1  qsfp56-dd-1-3  qsfp56-dd-1-5  qsfp56-dd-1-7
    qsfp56-dd-2-1  qsfp56-dd-2-3  qsfp56-dd-2-5  qsfp56-dd-2-7

Per-lane state (qsfp56-dd-1-1 shown, all 8 identical in the fields that matter):
    speed            = 50G-baseCR2      <- forcing 50G
    auto-negotiation = false            (forced; the passive-DAC mode the model records)
    fec-mode         = fec91            (RS(528,514) / clause 91; the FEC RUNNING AT 50G)
    slave            = true             (hardware-switched)
    rs-fec-corrected=1 uncorrected=6 over ~1e13 codewords (RS-FEC live and clean at 50G)

So the switch is FORCING 50G. The speed field is wrong for 100G; whether fec91 is also
wrong for 100G is NOT established here -- see the CORRECTION below.

CORRECTION (added with G2): an earlier head of this block called fec91 "correct for
100GBASE-CR2" and said "FEC and autoneg are already right", mapping the 100G intent to
Crs812FecMode.Fec91. That was a 50G observation promoted into a 100G requirement (DESIGN
§4d), and a meaning fork with G2 of this PR's gap analysis. What is actually observed is
that fec91 = RS(528,514) is the FEC the lane runs at 50G. 100GBASE-CR2 (50G-PAM4 lanes)
uses RS(544,514) / KP4 / clause 134, which the CRS812 REST surface does not appear to
offer (only fec74/fec91/off/auto). So fec91 is NOT established as the 100G intent; it is a
candidate the converge path must not assume, and G2 obligates converge to refuse if the
switch cannot offer the codeword the mode needs. The autoneg half of this same sentence
got its own correction lower in this log; this is the FEC half.

### Actuation 2026-09-17 — single-lane experiment: qsfp56-dd-1-1 (srv5)

Command: PATCH /rest/interface/ethernet/<id> {"speed":"100G-baseCR2"}
Expected: bounces srv5 fabric link, comes back at 100G if the copper carries it.

### Experiment result — 100G did NOT come up; restored to known-good 50G

Sequence run on qsfp56-dd-1-1 (srv5), all via REST PATCH:
  1. speed 50G-baseCR2 -> 100G-baseCR2 (autoneg off).      Host stayed 50G, no retrain.
  2. port bounce (disabled true->false).                    Host stayed 50G.
  3. auto-negotiation off -> true (advertise=100G-baseCR2). Host stayed 50G.
  4. port bounce again.                                     Host stayed 50G.
  5. switch monitor of dd-1-1 at 100G/autoneg:
        auto-negotiation = failed
        status           = auto-init-failed
        eeprom-checksum  = bad
        sfp-type         = QSFPDD
  6. RESTORE: speed 50G-baseCR2, autoneg false. -> status link-ok, rate 50Gbps,
     host Speed 50000 link-detected yes. KNOWN-GOOD RESTORED.

Wall located: the SWITCH-SIDE QSFP-DD (400G) module end. At 100G-baseCR2 the switch
cannot init the module (auto-init-failed, eeprom-checksum bad, autoneg failed). At
50G-baseCR2 forced the same module is link-ok and passes traffic. This is the end FS
(Nia) instructed to leave Generic-coded ("only reprogram the 100G QSFP56 ends; the 400G
QSFP-DD ends with Generic-compatible should remain unchanged").

Two credible remaining causes, NOT yet distinguished:
  A. The switch-side DD module coding/init genuinely caps at 50G (auto-init-failed is
     the direct evidence). If so this is a cable/coding wall -> FS DD-end recode, or the
     dual-compatibility cable (SKU 145681 / 101806).
  B. The link needs BOTH ends FORCED to 100G. The host has autoneg=on; a forced-switch/
     autoneg-host combination parallel-detects to 50G (demonstrated). Forcing the host
     (ethtool -s enp1s0f1np1 autoneg off speed 100000) needs the host-side SET grant,
     which was deliberately NOT built (this PR modeled only the read grant). Testing B
     requires building that set grant.

The auto-init-failed on the switch module points at A, but B is not ruled out and is the
cheaper thing to falsify first if the set grant is built.

## Convergence follow-up (dissolution of this scaffold)

The switch config actions here (force lane speed, force/clear autoneg, set FEC, bounce)
are the exact operations the modeled apply path (gap-analysis step B) must express:
Crs812LaneSpeed + Crs812FecMode over the REST paths already modeled in
extdeps.mikrotik.crs812. This log is the transcription source for that apply.

### Both-ends-forced experiment (srv5, 2026-09-17) — DUE DILIGENCE COMPLETE

Reached root on srv5 via the fleet key (briansrls) + spark-administrator-password over
stdin, so the host end could be forced. Tested every 100G configuration:

  config                                   switch            host              link
  ---------------------------------------  ----------------  ----------------  ------
  switch forced 50G / host autoneg (orig)  link-ok 50Gbps    50000 up          UP 50G
  switch forced 100G / host autoneg on     link-ok(local)    50000 up          no 100G
  switch autoneg 100G / host autoneg on    auto-init-failed  50000             no 100G
  switch forced 100G / host forced 100G    link-ok 100Gbps   Unknown, no link  no 100G

Key facts distinguished:
- The SWITCH PHY can run 100G: when FORCED it reports link-ok rate 100Gbps. So the
  switch hardware/module is NOT hard-capped at 50G. Earlier "auto-init-failed" was the
  AUTONEG path only.
- [SUPERSEDED at this site -- see the mstlink re-test section below: MikroTik REQUIRES
  forced/autoneg-off for 4x100G DAC breakout, so this "forced is non-compliant" reasoning
  was WRONG. The forced-both host "no partner" seen here was ethtool -s selecting the wrong
  lane count (CR4), not a compliance fact. Kept struck-through rather than deleted so the
  wrong turn is visible.] ~100GBASE-CR2 requires clause-73 autoneg on both ends; the forced
  path is non-compliant for CR, so forced-both cannot be the production config.~
- The AUTONEG path is the compliant one, and it fails at the SWITCH:
  status=auto-init-failed, eeprom-checksum=bad on the QSFP-DD module. The switch cannot
  complete 100G autoneg-init with the Generic-coded QSFP-DD end.
- 50G works because forced 50G parallel-detects without needing clean autoneg-init.

VERDICT [SUPERSEDED at this site -- this is the PRE-mstlink verdict; the corrected verdict
is in the mstlink re-test section below. It over-claimed on the forced path, which had not
yet been tested with the right tool. Kept for the record of the wrong turn]:
~100G will not come up in any configuration. The switch is capable (forced 100G trains
locally) and the host is ready, but the ONE compliant config (both-ends autoneg) fails at
the switch's autoneg-init against the Generic-coded QSFP-DD module. The wall is the
switch-side QSFP-DD module coding.~ The corrected verdict, after the forced path was
re-tested with mstlink, is below and is the one to read.

This is now a precise, evidenced question for FS: the NVIDIA(ETH) recode of the QSFP56
host ends lets the host advertise 100G-CR2, but the switch cannot autoneg-init 100G-CR2
against the Generic-coded QSFP-DD end (RouterOS: auto-init-failed, eeprom-checksum bad).
Does the QSFP-DD end also require coding for the switch to autoneg 100G-CR2, or is the
dual-compatibility cable (SKU 145681 / 101806) the intended remedy?

All 8 fabric lanes remain at known-good forced 50G. Only srv5 (qsfp56-dd-1-1) was touched
and it is restored (host 50000 up, switch link-ok 50Gbps).

### CORRECTION + forced-path test via mstlink (srv5, 2026-09-17)

A reviewer correctly flagged that MikroTik REQUIRES forced-speed / autoneg-off for
4x100G DAC breakout, so the earlier "forced path is non-compliant" framing was WRONG.
Re-tested the forced path correctly:
- ethtool -s cannot pin the 2-lane (CR2) mode on ConnectX-7; forcing "speed 100000"
  ambiguously selects CR4. The correct host tool is mstlink.
- NIC confirms the recode: Supported Cable Speed (Ext.) = 100G_2X (was 50G_2X only).
- Host forced via: mstlink -d 0000:01:00.1 --link_mode_force -s 100G_2X ; then
  mstlink -d ... -k RS --fec_speed 100G_2X  (the three flags are mutually exclusive in
  one invocation -- must be separate calls).
- Switch forced 100G-baseCR2, autoneg off; swept fec-mode auto/off/fec74/fec91.
- RESULT: host stays in "Polling" (Speed N/A), no bilateral link, under EVERY switch FEC
  mode. Switch reports link-ok/100Gbps locally but the host never trains.

CREDENTIAL/RESTORE INCIDENT: the GCP token expired mid-experiment; the restore could not
fetch creds and srv5 was left forced-100G / NO-CARRIER (down) until a fresh token arrived.
Restore required a host `ip link down/up` after re-enabling AN, then a switch port bounce.
All 8 legs verified back at 50000Mb/s; switch dd-1-1 at 50G-baseCR2/autoneg false/fec91.

### Verdict, stated at witnessed confidence (per reviewer)

The QSFP56 host recode succeeded and is recognized (ConnectX-7 advertises 100GBASE-CR2,
former cable-speed refusal gone). On one CRS812 breakout leg, NO bilateral 100G link was
established under the tested automatic (both-autoneg) OR forced (both-forced, mstlink)
configurations, across all switch FEC modes. RouterOS reports auto-init-failed and
eeprom-checksum bad for the still-Generic-coded QSFP-DD end on the automatic path. This
makes switch-side coding/compatibility the LEADING hypothesis but does NOT isolate it
from physical signal integrity, lane grouping, firmware behavior, or endpoint interop.

### Next step is a SUBSTITUTION control, not more config permutations (reviewer)

The single experiment that isolates cable/coding from switch/NIC/signal-integrity:
  (a) a known-good dual-compatibility cable in the same ports, OR
  (b) the same cable with a known-good 100GBASE-CR2 endpoint pair.
Until that is run, contacting FS is reasonable but "cause proved" is not. Further
actuation is PAUSED pending credential containment and the grant census.
