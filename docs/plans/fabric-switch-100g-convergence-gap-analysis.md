# Fabric switch 100G convergence — gap analysis

Subject: bringing the eight DGX Spark fabric legs on the MikroTik CRS812-8DS-2DQ-2DDQ-RM from
50GBASE-CR2 to 100GBASE-CR2, end to end, by execution rather than by hand.

Authorities read: `extdeps.mikrotik.crs812`, `extdeps.ethernet.link_mode`,
`gunbc.spark.fabric_switch_desired`, `gunbc.spark.fabric_switch_observed`,
`gunbc.spark.fabric_switch_assessment`, `gunbc.spark.managed_access_bootstrap`,
`gunbc.spark.grant_privileged_operation`, `gunbc.fleet_ssh_locus`,
`product.breakout_leg_programming`.

## 0. The standing observation, and what changed under it

`gunbc.spark.fabric_switch_observed` records the legs as QSFP28-coded — extended compliance byte 192
= `0x0B`, byte 116 = `00h` — so nothing attests 100GBASE-CR2, and the ConnectX-7 derives
`50G_2X` and refuses `100G_2X` (mstlink opcode 16).

**The operator re-flashed the legs with an FS BOX V4 on 2026-09-17.** That invalidates the EEPROM
half of the observation above; it does not replace it, because nothing has re-read the bytes.

Read live on 2026-09-17 against `spark-a3ee` (192.168.1.226) as `gunbc-automation`: both fabric
netdevs (`enp1s0f1np1`, `enP2p1s0f1np1`) are **up, autoneg on, 50000Mb/s**.

That reading is **not** evidence the re-flash failed, and must not be recorded as such. Two causes
are live and unseparated:

1. the coding did not change, or
2. the coding is correct and **nothing has forced the mode on either end** —
   `crs812_passive_dac_requires_forced_mode` is `true`, and the host is currently on autoneg.

The discriminator is the EEPROM bytes plus the NIC's supported-speed set. Neither is readable today
(§1). Establishing which cause holds is the **first** work item; everything downstream is premised
on it.

> **SUPERSEDED 2026-09-17 (see G12).** This §0 was written before the read grant landed and the
> bytes were read. They have since been read: byte 192 = `0x40` (attests 100GBASE-CR2) and the NIC
> reports `Supported Cable Speed: 100G_2X`. So cause (1) is ruled out — **the re-flash took** — and
> "the first work item" is done. The current standing fact lives in **G12** and the actuation log;
> the two paragraphs above are retained as the original framing, not a current open question. The
> live blocker is now the switch-side QSFP-DD end, not the host coding.

## 1. Gap: the host end cannot be read or set

`sudo -n -l` on spark-a3ee, 2026-09-17, shows `gunbc-automation` holds exactly:

```
(root) NOPASSWD: /usr/bin/loginctl enable-linger gunbc-automation
(root) NOPASSWD: /usr/bin/systemctl reboot
```

So `ethtool -m` (EEPROM read), `ethtool -s` (forced speed) and `mstlink` (supported-speed set,
`100G_2X` enable) are all unreachable by the principal the fleet runs as.

Missing, in dependency order:

| # | Gap | Home |
|---|---|---|
| 1.1 | No `extdeps` authority for the `ethtool` binary — name, path, verbs | new `extdeps/ethtool/` |
| 1.2 | No `extdeps` authority for `mstlink` / the MFT tools | new `extdeps/mellanox/` |
| 1.3 | `SparkManagedGrant` has no arm for a module read or a link-mode set | `gunbc.spark.managed_access_bootstrap` |
| 1.4 | No disclosure standing for what `ethtool -m` returns | same, `spark_managed_grant_disclosure_standing` |

On 1.4: the existing arms are `GrantReturnsNoHostData` (reboot) or `GrantDisclosureUnestablished`
(container inspect, which is why that grant is **desired but not installable**). `ethtool -m` returns
module vendor / part / serial and the EEPROM page — host data, but with no analogue of `Config.Env`.
It needs its own classification; it should not borrow `GrantReturnsNoHostData`, and it is not the
hard case `InspectServingContainer` is.

**A read grant and a set grant are two grants, not one.** Reading the EEPROM is what settles §0;
forcing the mode is an actuation on a live fabric. They should be installable independently so the
discriminating read does not require authorizing the mutation.

## 2. Gap: the switch has never been read

Better than expected — the REST surface is already modeled in `extdeps.mikrotik.crs812`:
`routeros_rest_authority`, and paths for `/rest/interface/ethernet`, `/rest/interface/ethernet/monitor`,
`/rest/ip/neighbor`, `/rest/system/identity`, plus `crs812_factory_address` (192.168.88.1/24) and
`crs812_factory_credential_location`.

And the assessment vertical is genuinely complete: `gunbc.spark.fabric_switch_assessment` carries
intent, observed, per-lane divergence with causes, typed refusals, expectation-driven unknowns, and a
`GoalInspection`. `gunbc.spark.fabric_switch_desired` correctly derives the desired lane speed from
the cable plant rather than declaring it, and refuses rather than degrades when the plant does not
converge.

**The gap is the transport, and it is total.** `fabric_switch_subject()` passes `readings: []`.
`observe_fabric_switch_attempt` takes readings as a **supplied** argument. Nothing in the corpus has
ever performed an HTTP request against the switch, so today every lane resolves to
`Crs812LaneUnknown` and the assessment is structurally incapable of returning anything else.

| # | Gap | Home |
|---|---|---|
| 2.1 | No RouterOS REST **transport** — nothing turns a modeled path into a request | new realization handler |
| 2.2 | No producer of `List<Crs812LaneReading>` from a live response | `gunbc.spark.fabric_switch_observed` or a new observer |
| 2.3 | No management address for the switch (only `crs812_factory_address`) | `gunbc.fleet.fleet_intent_network` |
| 2.4 | No credential standing for the switch | new, mirroring `gunbc.spark.bootstrap_credential` |
| 2.5 | No apply/actuation path — assessment can diverge but nothing converges | new, after 2.1–2.4 |

Note the ordering constraint from DESIGN §3d: selection precedes convergence, and the assessment
fold must not grow a realization. 2.1 is a **bound handler**, peripheral to the interface — not an
edit to `assess_fabric_switch`.

## 3. Gap: the switch credential

`crs812_factory_credential_location` already records that the credential is on the underside label,
user `admin`. What is missing is a **carried** `SecretRef`.

Per the operator ruling of 2026-08-07 recorded in `gunbc.spark.grant_privileged_operation`, secret
names are **carried, never derived** — a computed name makes the secret namespace a function of how a
host is spelled today. And a bare `SecretRef` asserts a secret exists, which is why
`SparkCredentialStanding` models `CredentialMaterialized` against `CredentialMaterializationPending`.

Minted now so the operator can upload against an exact name, in project `gunbai-secrets`
(`fleet_secrets_gcp_project`), following the established kebab convention
(`bmc-srv3-admin`, `spark-administrator-password`, `fleet-automation-ssh-key`):

- **`fabric-switch-factory-admin`** — the factory label credential, treated as **one-time
  authorization material** exactly as `OperatorBootstrapCredentialRequired` treats the Spark
  bootstrap admin. Not a serving credential, not retained.
- **`fabric-switch-admin`** — the rotated credential the fleet actually runs under, generated during
  bootstrap.

Bootstrap is therefore **not** "log in with the sticker password". It is: authorize once with
`fabric-switch-factory-admin`, mint and store `fabric-switch-admin`, and leave the factory credential
dead. Leaving the device on its factory password would be a standing exposure the model would be
silent about.

These rows land **with their first consumer**, not before — a `SecretRef` nothing reads is the
dangling declaration DESIGN §3c makes red.

## 4. What is NOT a gap

Worth stating so the work does not re-invent it:

- the desired-speed derivation (cable-plant-driven, refuses rather than degrades) — done;
- the divergence / unknown / refusal algebra over lanes — done;
- lane interface naming, lane-speed and FEC wire encodings, the forced-mode fact — done;
- the SSH credential locus and known-hosts anchor for the host end — done, and verified working;
- the human cost of the re-flash, and the part that removes it (`extdeps.transceiver.fs_145681`) — done.

## 5. Sequence

**A. Settle §0 before building anything in §2.** Land 1.1 + 1.3 + 1.4 (read grant only), install it,
read bytes 116 / 192 / 222 and the mstlink supported set off a re-flashed leg. One read decides
whether the fabric work proceeds or returns to the cable plant.

**B. If the coding attests the mode:** the switch address and `fabric-switch-factory-admin` upload,
then 2.1–2.4 — read the switch for the first time, which turns eight `Crs812LaneUnknown`s into real
readings and makes the existing assessment say something.

**C. Then converge,** both ends together: 1.2 + the set grant on the host, 2.5 on the switch, forced
100GBASE-CR2 with matched FEC.

**D. Re-read and confirm** `100G_2X` actually enabled and the link at 100000Mb/s, then update
`gunbc.spark.fabric_switch_observed`. Per that module's own standing, this is an **experiment**: only
the post-change read establishes anything about the copper. If the legs are correctly recoded, both
ends are forced, and the link still will not come up at 100G, `extdeps.ethernet.link_mode` is explicit
that operating at 50GBASE-CR2 never certified 100GBASE-CR2 — that outcome is a real possible answer,
not a failure of the procedure.

---

## 6. Modeling gaps & incompatibilities found during actuation (2026-09-17)

Discovered by actually driving the switch and hosts (see fabric-switch-actuation-log.md).
Each is a place the `.dag` model does not yet express something the real path required.

### Switch model (extdeps.mikrotik.crs812 / gunbc.spark.fabric_switch_*)

- **G1 — no autonegotiation dimension.** The lane model carries speed and FEC but NOT
  auto-negotiation on/off. Copper (BASE-CR) bring-up is entirely governed by autoneg
  state; the whole experiment turned on it. `Crs812LaneIntent` needs an autoneg field.
- **G2 — the FEC enum names FEC by RouterOS label, not by codeword, and the correct 100G
  codeword is UNWITNESSED.** `Crs812FecMode = FecAuto|FecOff|Fec74|Fec91` carries RouterOS's
  strings, not the RS codeword (528,514 vs 544,514) each implies — a modeling nicety. Two
  §4d errors to NOT repeat: (a) an earlier head asserted 100GBASE-CR2 needs RS(544,514) from
  the IEEE 50G-PAM4 lane FEC and concluded the CRS812 might not expose it — an inference
  stated as a fact about this switch, retracted; (b) a later head asserted "MikroTik
  documents fec91 for 100G-baseCR2" (from review, no cited vendor document) and on that
  strength deleted the converge's FEC-capability refusal arm — the same move in the other
  direction, also retracted. NEITHER codeword claim is grounded. What IS grounded: the eight
  fabric legs are OBSERVED running fec-mode fec91 today (at 50G). So `fec91` is the desired-FEC
  CANDIDATE (the FEC the plant already runs), the converge KEEPS its refusal arm rather than
  assuming the candidate trains 100G, and grounding the codeword needs a cited MikroTik doc,
  not an annotation.
- **G3 — the live link OUTCOME is unmodeled.** The assessment models intent-vs-observed
  lane divergence, but the observed reading needs: negotiated speed, link state
  (up / polling / down / auto-init-failed), and FEC-locked, read from BOTH ends. RouterOS
  `status` (link-ok/auto-init-failed) and `eeprom-checksum` are unmodeled.
- **G4 — a link is two-ended; the model treats the switch lane as the subject.** The
  switch reporting `link-ok/100Gbps` is a LOCAL claim; the host was in `Polling`. Link-up
  is a bilateral fact requiring both ends to agree. The observed model joins both ends for
  the CABLE receipt but not for the live LINK state.
- **G5 — QSFP-DD (switch-side) module is unmodeled.** The switch reads the QSFP-DD end,
  which is CMIS/SFF-8024, NOT the SFF-8636 the host QSFP56 end uses. `sff8636_*` decoders
  don't cover it. The `eeprom-checksum: bad` on the Generic-coded DD end has no home.
- **G6 — no RouterOS REST transport / reader / apply.** Still the whole of step B: nothing
  turns crs812_rest_* paths into requests; `fabric_switch_subject()` passes `readings: []`.
- **G7 — no switch credential standing landed.** `fabric-switch-factory-admin` exists in
  Secret Manager but no SecretRef rows; factory->rotated bootstrap unmodeled.
- **G8 — the fabric legs are on qsfp56-dd cages, not qsfp56.** Confirm desired/assessment
  target the qsfp56-dd interfaces (the 50G breakout legs), per the observed roster.

### Host model (gunbc.spark.*)

- **G9 — host link control is via mstlink (MFT), which has NO extdeps authority.** The
  NVIDIA firmware tool surface (mstlink, mstconfig, mstflint) is entirely unmodeled. Host
  link force/FEC/lane-count lives there, not in ethtool.
- **G10 — ethtool cannot express 2-lane (CR2) forcing on ConnectX-7.** `ethtool -s speed
  100000` is lane-count-ambiguous and selects CR4; only mstlink can pin 100G_2X. A modeled
  host-force MUST use mstlink, and mstlink's flags `-s`, `--link_mode_force`, `-k` are
  mutually exclusive per invocation (separate calls required).
- **G11 — no host-side SET/force grant.** This PR modeled only the EEPROM READ grant.
  Forcing the host link needs a mstlink grant (root), unmodeled and uninstalled.
- **G12 — fabric_switch_observed is now STALE on main.** It states legs are 0x0B-coded and
  the NIC refuses 100G_2X. Post-recode: byte 192 = 0x40, NIC Supported Cable Speed =
  100G_2X. The observed authority must be updated with the new reading (and should model
  the recode as an event, not overwrite silently).

### Operational / workflow

- **G13 — spark_grants dispatch cannot fan out.** Dispatching 8 fleet-converge runs 3s
  apart CANCELLED 4 via the workflow concurrency group. Multi-target grant install must be
  serialized (one run completes before the next) or the workflow needs a fan-out mode.
- **G14 — spark_grants installs ALL missing grants, not a scoped one.** No dispatch-level
  way to install only the EEPROM grant vs also InspectServingContainer.
- **G15 — dropin filename drift.** Host holds `/etc/sudoers.d/gunbc-gunbc-automation` for
  the linger grant while the model derives `gunbc-enable-linger`. The installed filename
  and the modeled `spark_managed_grant_dropin_name` disagree; re-install would write a
  second file. (Noticed earlier, unrelated to this PR, still real.)

### Credential containment (process, not model)

- **G16 — actuation ran on ad-hoc curl with pasted GCP tokens.** The token expired
  mid-experiment and left srv5 down with no recovery path in-session. The modeled path
  (WIF on a runner, credential materialized and removed in-step) exists for grant install
  and must be the ONLY actuation path; ad-hoc curl from a session is the scaffold to kill.
