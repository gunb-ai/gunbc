# Fabric switch 100G — out-of-band actuation log

Operator-approved workaround (2026-09-17): the switch and host link modes were driven
directly over the RouterOS REST API and the host's `mstlink`, from a session on the LAN,
ahead of the modeled convergence path (docs/plans/fabric-switch-convergence-build-plan.md).

**This file is an INPUT to the modeled apply, and its dissolution trigger** -- it is NOT an
exact transcription of a successful configuration, because no bilateral 100G configuration
worked. It records the facts established, the NORMALIZED operation shapes attempted (with
placeholders like `<iface>`/`<pci>`/`<dev>`, and RouterOS CLI equivalents for REST
mutations), and the verdict. A future receipt may carry the exact REST method, resource
identity, request body, real interface/PCI identifiers, timestamps and pre/post
observations; that is safely deferred. The
scaffold is retired when that modeled path lands and reproduces this end state. Round-by-
round exploration (wrong turns, corrected framings) is intentionally NOT preserved here;
git history carries it, and this file states each conclusion once, at witnessed confidence.

## Facts established

- **Switch:** MikroTik CRS812-8DS-2DQ-2DDQ, RouterOS 7.20.8, management 192.168.1.240,
  identity `gunbc-fabric-switch`. REST API live (401 unauth). Credential in Secret
  Manager: `fabric-switch-factory-admin`.
- **Host recode confirmed** (read on spark-a3ee, 2026-09-17): EEPROM byte 192 = `0x40`
  (was `0x0B`), attesting 100GBASE-CR2; `mstlink` reports `Supported Cable Speed: 100G_2X`
  (was `50G_2X` only); the former "Cable speed not enabled" refusal is gone. All 8
  ConnectX-7 NICs advertise `100000baseCR2/Full`. This is carried on the modeled carrier:
  gunbc.spark.fabric_switch_observed `fabric_leg_eeprom_observations` (spark-a3ee row).
- **The fabric legs are on the switch's `qsfp56-dd` (400G breakout) cages**, comment
  "gunbc fabric leg": qsfp56-dd-{1,2}-{1,3,5,7}. Each was forced to `50G-baseCR2`,
  autoneg off, `fec-mode fec91`. `crs812_passive_dac_requires_forced_mode = true`.
- Lane roster (cage:lane -> host): 1:1 srv5, 1:3 srv8, 1:5 srv7, 1:7 srv6, 2:1 srv10,
  2:3 srv12, 2:5 srv9, 2:7 srv11.

## Normalized operation shapes attempted (an input to the modeled actuator)

Switch, per lane, over `/rest/interface/ethernet/<iface>` (RouterOS CLI form):
```
/interface/ethernet/set <iface> speed=100G-baseCR2 auto-negotiation=no fec-mode=fec91
/interface/ethernet/disable <iface> ; /interface/ethernet/enable <iface>   # bounce to retrain
```
Host (ConnectX-7, PCI 0000:01:00.1), forced 2-lane 100G via mstlink (ethtool -s cannot
pin CR2 — it ambiguously selects CR4; the flags below are mutually exclusive per call):
```
mstlink -d <pci> --link_mode_force -s 100G_2X --yes
mstlink -d <pci> -k RS --fec_speed 100G_2X --yes
```
Restore to known-good 50G: switch `speed=50G-baseCR2 auto-negotiation=no fec-mode=fec91`
+ bounce; host `mstlink -d <pci> -s 100G_2X,50G_2X,50G_1X,25G,10G,1G --yes` (re-enables
AN) then `ip link set <dev> down; ip link set <dev> up` (the host needs its own bounce to
retrain after the mstlink change).

## Verdict (witnessed confidence)

The QSFP56 host recode succeeded and is recognized: the ConnectX-7 advertises 100GBASE-CR2
and no longer refuses the cable speed. On the CRS812 breakout legs, **no bilateral 100G
link was established** under either tested production configuration — automatic (both-ends
autoneg) or forced (both-ends forced, via mstlink for the host), across every switch FEC
mode. The host sits in `Polling`; the switch reports its own side `link-ok/100Gbps`
locally while the host never trains. On the autoneg path, RouterOS reports
`auto-init-failed` and `eeprom-checksum: bad` for the still-Generic-coded QSFP-DD end.

This makes **switch-side QSFP-DD coding/compatibility the leading hypothesis, NOT a proven
cause**: the current tests do not isolate it from signal integrity, lane grouping,
firmware, or endpoint interoperability. The tested srv5 leg was the mutated specimen and
was restored and verified at 50G; all eight legs were verified at their known-good 50G state
after the experiment.

## FS support escalation (2026-09-18)

FS support asked us to follow the CRS812 breakout documentation: force the port speed with
auto-negotiation off (`auto-negotiation=no speed=100G-baseCR2` on the odd sub-ports). That
is the configuration already tested above. They also asked whether the ConnectX-7 was
actually up when the switch reported link-ok. To answer, one operator-approved retest ran on
the srv5 leg only (`qsfp56-dd-1-1`, 12:55:07-12:56:00 UTC). Both credentials were loaded
before the change and the restore was armed to run on exit, which closes the expired-token
failure above.

- **FS's exact configuration was applied** on the switch with `fec91` and a bounce. The
  host was forced to `100G_2X` with RS-FEC and bounced.
- **Both ends were read at +10s and +30s, with the same result each time:**
  - CRS812: `link-ok`, `rate=100Gbps`.
  - ConnectX-7: `mstlink State: Polling`, speed N/A.
  - Linux: `NO-CARRIER`, carrier 0. `ethtool`: `Link detected: no (Autoneg, No partner
    detected)`, active FEC None.
  - Ping across the fabric link: 100% loss.
- **Answer to FS: no, the ConnectX-7 was not operationally up.** The switch's 100G reading
  describes only its own side.
- **RouterOS reports `eeprom-checksum: bad` for the QSFP-DD end at the healthy 50G state
  too.** On its own, the flag therefore does not explain the 100G failure.
- **The restore was verified from both ends:**
  - Switch: 50G-baseCR2, fec91, link-ok.
  - Host: Active, 50G, 2 lanes, RS(528,514), carrier 1.
  - Ping to .14 and .12: 0% loss.
- **Operator sent the evidence to FS the same day** (after redacting it): the CRS812 system
  information and hide-sensitive export, the port and module information, the ConnectX-7
  mstlink/ethtool/module EEPROM baselines, and simultaneous captures from both ends for the
  forced test and the restore. Awaiting FS engineering's review. Cable: FS
  QDD-400G-4QPC015, host-end serial S2630771509-2. Switch: RouterOS 7.20.8. NIC firmware:
  28.45.4028.

The verdict above stands unchanged. The leading hypothesis is still unproven, and the next
step is still the substitution control below, unless FS identifies a configuration error.

## FS FEC=Auto retest (2026-09-21)

FS replied on 2026-09-21 naming no cause: no configuration error in the 09-18 packet, no
recode ordered, no replacement cable. They asked for one further test -- their screenshot
switch configuration again, but with `fec-mode=auto`, and the ConnectX-7 on Auto for both
speed and FEC -- plus all FEC configuration and status, and whether any other 400G QSFP-DD
port is available for a cross-test (none is today; the operator expects one in a few
months, too far out to serve this thread).

One operator-approved retest ran on the srv5 leg only (`qsfp56-dd-1-1` <-> spark-a3ee
`enp1s0f1np1`, 15:48-15:50 UTC). Both credentials were loaded and verified BEFORE any
change and the restore was armed on exit.

- **The link stayed up at 50G and never reached 100G.** Switch: `link-ok`, `rate=100Gbps`,
  resolved `fec: fec91`. Host: `Active`, `50G`, `2x`, `RS(528,514)`, autoneg ON, carrier 1,
  0% ping loss. Identical at +10s and +30s.
- **Why this differs from 09-18 rather than contradicting it:** the host kept
  auto-negotiation ON. Forced-both-ends put the host in Polling with no carrier;
  host-Auto against an autoneg-disabled switch port leaves it where it already was.
  The switch's 100Gbps again describes only its own side.
- **Restored and verified from both ends:** switch 50G-baseCR2 / fec91 / link-ok, host
  Active 50G RS(528,514) carrier 1, 0% loss.

### WITHDRAWN: the RS-528 versus RS-544 disjointness claim

An earlier revision of this section concluded that the CRS812 presents RS-FEC(528,514) at
100GBASE-CR2 while the ConnectX-7 accepts only RS-FEC(544,514), that their FEC sets are
therefore disjoint, and that neither a recode of the QSFP-DD end nor a substitution cable
could produce a 100G link. **That conclusion is withdrawn. It was not established, and the
step that produced it is the one this repository already has a name for: reading an
implementation off a NAME.**

What was measured stands and is worth keeping:

- ConnectX-7 `mstlink --show_fec`: `FEC Capability 100G_2X : 0x80 (RS-FEC (544,514))` and
  `FEC Capability 50G_2X : 0x7 (No-FEC, Firecode_FEC, RS-FEC (528,514))`. This is a
  per-speed CAPABILITY table and is NOT the active FEC of the present 50G link, which
  reads `RS(528,514)` separately.
- CRS812 `fec-mode` accepts exactly `fec74`, `fec91`, `auto`, `off`; `fec119`, `fec134`,
  `rs-fec544` and `rs544` are refused. With `fec-mode=auto` the monitor reported `fec91`.

What does NOT follow: that `fec91` denotes RS(528,514) on this port at this speed. `fec91`
is RouterOS's user-facing FEC selector, and MikroTik's own compatibility documentation
lists `fec91` as the setting REQUIRED for `100G-baseCR2` -- which is a two-lane PAM4 mode
whose IEEE 802.3cd FEC is RS(544,514). The most economical reading of those two facts
together is that RouterOS uses the historical `fec91` token as its generic RS-FEC selector
and picks the speed-appropriate codeword internally. The refusal of `fec134` and `rs544`
proves those strings are absent from the CONFIGURATION GRAMMAR; a parser's vocabulary is
not a PHY's capability, and treating it as one is the same move as reading a type name as
a safety guarantee.

The open question is therefore narrower and belongs to MikroTik rather than to FS: **on a
`qsfp56-dd` breakout sub-port at `speed=100G-baseCR2`, does RouterOS's `fec91` realize
RS-FEC(544,514)?** RouterOS's output does not expose the codeword, so it is not decidable
from either endpoint.

### The producer disagreement, which is what this test actually established

The three readings taken during the test cannot all describe one link:

```
CRS812:      link-ok, rate=100Gbps
ConnectX-7:  Active, 50G, 2 lanes
traffic:     passing, 0% loss
```

A single Ethernet link does not run as 100GBASE-CR2 at one end and 50GBASE-CR2 at the
other while carrying valid frames. That contradiction was recorded and then reasoned PAST
into a FEC diagnosis; it should have stopped the derivation, and it is the finding.

**Two of the candidate explanations were closed by measurement afterwards (2026-09-21, at
the restored 50G state).** Subject mapping and traffic path are now proven rather than
assumed:

- `ip route get 192.168.110.14` resolves via `enp1s0f1np1`, which holds `192.168.110.11/24`.
  The host's other UP ConnectX port (`enP2p1s0f1np1`, the cross-cabled twin) carries no
  address on that subnet and cannot serve the ping.
- Counter discrimination on the switch port: background `rx-packet` over ~10s = **196**;
  the same window carrying 5000 pings = **+5257**. The traffic is on `qsfp56-dd-1-1`.
  (First attempt read `driver-rx-packet`, which is the CPU path and barely moves for
  hardware-switched frames -- the wrong record, corrected to `rx-packet`/`tx-packet`.)

So the readings are of the same physical leg, and the frames really cross it. What remains
is that **the CRS812's `rate` under a forced speed does not evidence a bilateral wire
rate.** Two independent observations support that and neither depends on the FEC story: on
2026-09-18 the switch read `link-ok / 100Gbps` while the host had NO CARRIER AT ALL, a
state in which a bilateral 100G link is certainly false; on 2026-09-21 it read the same
while the host passed traffic at 50G. Whether that is local-state reporting by design or a
defect in forced mode is a question for MikroTik, and it is the reason the switch's 100G
reading may not be used as evidence in either direction.

**What FS is owed is therefore the raw packet and the question, not a diagnosis.** The
strongest result here is a cleanly reproducible producer disagreement that gives FS and
MikroTik each something concrete to answer.

**A capture gap, stated rather than papered over.** This run preserved SUMMARIZED readings
(the fields in run.log), not the full raw artifacts a vendor packet should carry --
`/interface ethernet print detail`, `monitor once`, `print stats-detail`, `/export
hide-sensitive`, and the host-side `mstlink` operational block, module info and full
`ethtool`. The host-side FEC setting IS evidenced as having taken: `ethtool --show-fec`
moved from `Supported/Configured FEC encodings: RS` at baseline to `Auto` after
`mstlink -k AU`. The SPEED half of FS's request was already satisfied before the change --
autoneg was ON and the enabled set was already the full
`100G_2X,50G_1X,50G_2X,25G,10G,1G` -- so that half was a no-op, which the packet must say
rather than imply a setting was applied.

### Run 2, 2026-09-21 16:27-16:30 UTC: the same result with the capture gap closed

The first run preserved summarized readings, which is not a vendor packet. Run 2 executed
the identical configuration on the same leg with full raw capture at four stages --
baseline, +10s, +30s, restore-verify -- each stage one file per end under `packet2/`,
plus the exact commands with their exit statuses in `applied-commands.txt`. Restore armed
on exit, as before.

**The run reproduced the endpoint-reporting mismatch.** (Not "reproduced exactly": run 1 preserved no equivalent raw evidence, so no claim is made that every transition was identical.) Switch at both samples: `speed=100G-baseCR2`,
`fec-mode=auto`, monitor `link-ok / 100Gbps`, resolved `fec: fec91`. Host at both samples:
`Active`, `50G`, `2x`, `RS(528,514)`, `Auto Negotiation: ON`, physical state
`ETH_AN_FSM_ENABLE`. Restored to 50G/fec91 and verified from both ends.

**The host-side settings are now evidenced as having taken, separately from the outcome.**
`ethtool --show-fec` reads `Configured FEC encodings: RS` at baseline, `Auto` at +10s and
+30s, and `RS` again after restore, with `mstlink -k AU` returning exit 0. The SPEED half
of FS's request was a no-op re-assert and the packet says so rather than implying a change
was applied: autoneg was already ON at baseline with the enabled set already
`100G_2X,50G_1X,50G_2X,25G,10G,1G`.

**The decisive addition is the traffic proof taken AT THE TEST STATE**, which run 1 never
had -- it measured only after restore, so it could not speak to the contradiction it was
supposed to resolve. Switch-port `rx-packet`, 10s background versus the same window
carrying 5000 offered pings:

| stage | background | with 5000 pings | ping result |
|---|---|---|---|
| baseline (50G) | 199 | +5256 | 5000/5000, 0% loss |
| **t+30s (switch forced 100G)** | **197** | **+5264** | **5000/5000, 0% loss** |
| restore-verify (50G) | 182 | +5248 | 5000/5000, 0% loss |

So while the switch port is configured at `100G-baseCR2` and reports `rate=100Gbps`, five
thousand offered frames cross `qsfp56-dd-1-1` and the host receives all of them at a link
it reports as 50G. The producer disagreement is not a stale or mismatched reading: it is
live, reproducible, and holds under load.

**THE RETRAIN IS EVIDENCED, AND A HYPOTHESIS RAISED HERE IS WITHDRAWN BY IT.** An earlier
revision of this paragraph noted that the host's operational block is byte-identical across
baseline, +10s and +30s, and offered as a hypothesis that the link might never have
retrained at all -- which would mean the old 50G session simply remained resident and the
new switch configuration never reached the wire, making the whole run uninformative. The
discriminator it named was a per-netdev link-down record. That record exists and was read,
and it answers the other way. Host kernel log around the apply at 16:28:10 UTC:

```
16:28:13 mlx5_core 0000:01:00.1 enp1s0f1np1: Link up
16:28:14 mlx5_core 0000:01:00.1 rocep1s0f1: Port: 1 Link DOWN
16:28:17 mlx5_core 0000:01:00.1 enp1s0f1np1: Link up
```

Both ends were explicitly bounced (`/interface ethernet disable`/`enable` on the switch,
`ip link set down`/`up` on the host, both recorded with exit statuses in
`applied-commands.txt`), the link went down, and **it retrained -- to 50G -- while the
switch port was configured for `100G-baseCR2` with auto-negotiation disabled.** The
identical operational blocks are therefore the post-retrain state, not a stale session.
This is the strongest form of the result: it is not that 100G was never attempted, it is
that the retrain under FS's exact configuration settled at 50G.

**One narrowing that the evidence requires.** It is correct to say the host's 50G state and
the traffic path were current and verified on the intended leg. It is NOT correct to
conclude from that that the RouterOS `rate` field is "stale" -- it may be configured,
local, cached or something else, and which of those it is remains unknown. What follows is
only that the simultaneous `rate=100Gbps` cannot be read as the bilateral wire rate without
an explanation from MikroTik or FS.

## Next step: the raw packet to FS, and the codeword question to MikroTik

SUPERSEDED 2026-09-21 by the withdrawal above. The substitution control below is neither
promoted nor ruled out: the argument that ranked it below an RS-544 question depended on
the disjointness claim, which is withdrawn, so the control returns to standing on its own
merits as the experiment that isolates cable and coding. It is still not the immediate
next step, for a different and weaker reason -- two vendor questions are outstanding and
both are free. The two questions, each to the party that can answer it:

1. **FS** -- is this cable, with its Generic-compatible QSFP-DD end, validated on the
   CRS812 in `4x100G-baseCR2` breakout mode; is `fec91` expected to interoperate with the
   ConnectX-7's `100G_2X` RS-FEC; is anything in the raw output set wrong; and does their
   validated configuration assume a RouterOS newer than 7.20.8?
2. **MikroTik** -- on a `qsfp56-dd` breakout sub-port with `speed=100G-baseCR2` and FEC
   resolving to `fec91`, does RouterOS select RS-FEC(544,514) for the two-lane PAM4 mode
   despite the `fec91` name? Asked as a question about what the token realizes, NOT as an
   assertion that the switch lacks RS-544.

**The RouterOS version is an uncontrolled variable and is recorded as one.** The switch is
on 7.20.8. No claim of a firmware limitation may be made without either confirmation that
7.20.8 is validated for CRS812 4x100G breakout, or a backed-up, controlled update and a
repeat of the one-leg test. Upgrading a live fabric switch mid-evidence-collection is not
done casually and is not proposed here.

The single experiment that isolates cable/coding from switch/NIC/signal-integrity is a
hardware substitution: a known-good dual-compatibility cable (FS SKU 145681 / 101806) in
the same ports, OR the same cable with a known-good 100GBASE-CR2 endpoint pair. Until that
runs, contacting FS is reasonable but "cause proved" is not. Further out-of-band actuation
is paused pending credential containment.

## Cross-references

- Credential/restore incident (token expired mid-run, srv5 left down): filed as
  gunbc.recurring_failure_mode
  `a_rollback_depends_on_a_credential_that_can_expire_mid_actuation`.
- Modeling gaps found while driving the hardware: gap-analysis §6 (G1–G16).
- The modeled path this scaffold dissolves into: fabric-switch-convergence-build-plan.md.
