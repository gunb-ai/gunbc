# Fabric switch convergence — build plan (the normal path)

Goal: when the correctly-coded QSFP-DD end arrives, converge the 8 CRS812 fabric lanes to
100GBASE-CR2 through a MODELED, repeatable path — no ad-hoc curl. This defines that path
and sequences it into reviewable PRs. It dissolves the actuation scaffold in
fabric-switch-actuation-log.md and closes gaps G1-G16 in
fabric-switch-100g-convergence-gap-analysis.md.

## Layering (DESIGN §3: interface / realization / policy are three facts)

- INTERFACE — `extdeps.mikrotik.crs812` already owns the REST paths, lane speed and FEC
  wire values, and the forced-mode fact. Extended here with the autoneg axis and a live
  link-outcome shape (G1, G3). This is what RouterOS's API IS, not what we do with it.
- REALIZATION — RouterOS REST operations declared as `transport rest` operation rows
  (method × path × basic-auth), realized by the shared HTTP-client machinery and
  fixture-tested via `RestExchangeFixture`. Precedent: `extdeps.bmc.http` Redfish ops.
- POLICY — `gunbc.spark.fabric_switch_*` (workflow layer): which lanes, forced 100G-CR2 +
  RS-FEC, the credential, the converge order. This is our business decision, not a fact
  about MikroTik.

## Credential (G7)

Mirror `gunbc.spark.bootstrap_credential` / `grant_privileged_operation`:
- `fabric-switch-factory-admin` (exists in Secret Manager) as one-time bootstrap material.
- `fabric-switch-admin` (rotated) as the operating credential.
- A `FabricSwitchCredentialStanding = CredentialMaterialized{ref} | CredentialMaterializationPending`.
- Fetched runner-side over Secret Manager REST (as spark_grants does), never in argv.
- Bootstrap = rotate factory -> admin; the factory credential is not a serving credential.

## PR sequence

- **PR-1 (model only, safe, this session's target): domain + credential.**
  - Add `auto_negotiation` to the lane intent/reading (G1).
  - Add `Crs812LinkOutcome` (negotiated speed, link state:
    LinkUp|Polling|AutoInitFailed|Down, fec-locked) as a two-ended live fact (G3, G4).
  - Record the FEC-codeword fact honestly (G2): fec91 = RS(528,514); 100GBASE-CR2's
    50G-PAM4 lanes use RS(544,514); the CRS812 REST surface exposes only fec74/fec91/off/
    auto. Model this as a stated capability fact so the converge intent can refuse if the
    switch cannot offer the FEC the mode needs, rather than trying and hanging in Polling.
  - `FabricSwitchCredentialStanding` + the two SecretRef rows.
  - Update `gunbc.spark.fabric_switch_observed` (G12): the 2026-09-17 recode is an EVENT
    (byte 192 0x0B->0x40, NIC Supported Cable Speed 50G_2X->100G_2X) plus the live 50G
    link reading; do not silently overwrite the prior observation.
  - Witnesses for each new type/derivation; fixtures from the real REST/mstlink output
    already captured in the actuation log.

- **PR-2: the read path.** RouterOS REST read operations (GetEthernetInterfaces,
  MonitorLane) as `transport rest` rows, a reader producing `List<Crs812LaneReading>` from
  the JSON, fixture-tested against captured responses. Wire `fabric_switch_subject()` to a
  live read so `assess_fabric_switch` stops being vacuous (readings: []). Read-only.

- **PR-3: the converge path.** The forced-100G apply (PATCH speed/autoneg/FEC + port
  bounce), mirroring `gunbc.spark.managed_access_apply`: idempotent, self-checking, each
  step's outcome typed, exit folds over the report. Run via `fleet-converge.yml` with the
  switch credential materialized/removed in-step. Refuses (never hangs) when the intent's
  FEC exceeds the switch's capability. Two-ended convergence: success only when BOTH the
  switch monitor AND the host mstlink report the negotiated speed, not one end's local
  "link-ok".

- **PR-4 (host side): mstlink authority + set grant.** `extdeps.mellanox` for mstlink
  (G9), the 2-lane-force fact (G10: ethtool cannot pin CR2), and the host-side set grant
  (G11) so the host end is forced through the modeled grant path, not sudo-over-SSH.

## What none of this can do until the hardware is right

The model + fixtures do not need a live 100G link to be correct and tested. But end-to-end
CONVERGENCE to 100G cannot SUCCEED until the QSFP-DD end links at 100G (pending FS /
substitution). PR-3's converge will correctly REFUSE or report Polling until then — which
is the honest behavior, not a failure of the model.
