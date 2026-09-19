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

## Next step: a substitution control, not more configuration

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
