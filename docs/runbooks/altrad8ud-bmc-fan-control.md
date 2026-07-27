# ALTRAD8UD BMC fan control and telemetry

This runbook records the srv3/srv4 investigation from 2026-07-27. Both machines are ASRock Rack ALTRAD8UD-1L2T boards. The fleet-wide desired state is now OpenBMC major track 3 plus the TEMP_SOC fan policy below. srv3's observed 2.07.00 is therefore drift; srv4's observed 3.22.00 is in the valid firmware band.

“3.x” is a validity rule, not an instruction to download “latest.” Firmware mutation still requires one exact, approved, board-compatible image URI and digest.

## What actually owns the fan curve

On both observed firmware versions, `phosphor-pid-control.service` runs `/usr/bin/swampd`. The selected configuration is:

```text
/usr/share/swampd/config.json -> /usr/share/swampd/config-asrr.json
```

The board configuration controls all five fan outputs from `TEMP_SOC` with this observed policy:

| TEMP_SOC at or above | Requested duty |
| ---: | ---: |
| 40 °C | 30% |
| 55 °C | 40% |
| 65 °C | 50% |
| 75 °C | 75% |
| 85 °C | 100% |

Positive and negative hysteresis are both 1 °C. Minimum duty is 30%, including below the first breakpoint. If the control temperature is unavailable, the observed fail-safe request is 75%.

The BMC 3.22 web UI's **Fan Control** resource changes `InitialDuty`; it is not the authority for these temperature/duty points.

On srv4, `/usr/share` is visible through the root overlay: a read-only SquashFS lower layer on `/dev/mtdblock4` and a writable JFFS2 upper layer on `/dev/mtdblock5`. The active board file currently comes from the lower layer; no upper-layer override exists. That makes a managed persistent override technically viable, but it is not yet a durability receipt: a BMC reboot and a firmware replacement must both be followed by rediscovery and readback.

An authenticated read-only probe projected the desired TEMP_SOC policy through the active 3.22 JSON in memory. The projection was idempotent, a 75%-to-74% perturbation made the check fail, and reapplying the projection healed it. No BMC file was written and `phosphor-pid-control.service` was not restarted. This proves the exact 3.22 transformation, not the mutation/rollback path.

## How BMC convergence fits host convergence

Ubuntu and BMC convergence share one lifecycle:

```text
desired state -> live observation -> classify -> typed plan -> apply -> independent readback -> re-drive/no-op
```

The shared state is `std.upsert_decision.UpsertClassification` and `ObservationVerdict`; the shared apply/readback result is `std.realization_reconcile.Reconciliation`. Host OS and BMC code differ only where reality differs: observer, control-plane transport, typed effect, and safety proof. A BMC shell/Redfish operation must not masquerade as a `HostOs` shell effect.

`PhaseDisposition` is now a product of `authorities` and typed `gaps`. That permits the honest middle state: comparison logic exists, but a live observer or actuator is still missing. The operator gap view is derived from the spine by `host_standup_gap_rows()`; there is no second hand-maintained gap ledger.

For the current local bootstrap, GCP access is an ensure/upsert at the point of demand, not a standup phase. A selected operation must first require a Secret Manager read/write; only that branch calls `gcloud auth print-access-token` to observe the access capability. A live session classifies `Converged/Noop`. An absent or expired session classifies `Absent/Apply(HumanGcloudLogin)`: run `gcloud auth login` with an account authorized for the fleet secrets project, then re-run the same convergent command. A path that does not need GCP never evaluates the ensure and never pauses for gcloud. The token is captured internally and must not be pasted into intent, argv, a netrc committed to the tree, or the PR. A workload-identity/OAuth handler can later realize the same ensure at the shared `AccessTokenSource` seam.

The read-only managed-credential checks `srv3_managed_credential_probe_check` and `srv4_managed_credential_probe_check` both completed live on 2026-07-27. On each host, the factory credential was rejected with HTTP 401, the active gcloud session satisfied the access ensure without human intervention, the Secret Manager `payload.data` value was decoded from standard base64, and the stored credential authenticated to the BMC. The expected factory-auth failure rendered only `<redacted-secret>` in argv diagnostics. The first execution found two defects before this receipt could go green: treating the Secret Manager payload object as credential bytes and rendering `Secret`-typed shell inputs inside failed argv traces. Both are now covered by discriminating tests. These checks prove credential selection and liveness; they do not materialize a scoped netrc/SSH binding or authorize any BMC mutation.

The current spine has 18 named gaps, 14 of them in the BMC prefix. They cover managed credential binding; live firmware observation; exact approved artifact; update actuator; post-update return/readback and recovery; subsumption-entrypoint wiring; live Redfish surface rediscovery; live fan-policy observation; persistent overlay actuation and rollback; post-apply thermal/tach/service proof; reboot persistence; and a workload-qualified thermal envelope. The remaining four are pre-existing OS/assimilation gaps.

## What subsumption does today

| Host | Snapshot classification | What a current subsumption run does |
| --- | --- | --- |
| srv3 | Firmware `Drifted`; fan realization `Conflict` until firmware converges and surfaces are rediscovered | Does **not** update the BMC. No exact approved image/digest or BMC update actuator is wired. |
| srv4 | Firmware `Converged`; declared curve `Converged`, therefore `Noop` | Does **not** rewrite the curve. The snapshot agrees, but the managed live observer and BMC driver are not wired. |

`gunbc converge` currently drives the `HostOs` policy only. Its receipt is not a BMC-convergence receipt. The explicit `bmc-subsumption-entrypoint-wiring` gap prevents the host path from being described as if it also reconciled the controller.

Read-only Redfish checks found `/redfish/v1/UpdateService/update` on both machines and marked each active BMC image `Updateable=true`. Each inventory exposed exactly one BMC image and no backup BMC image. The update transport shape therefore exists, but rollback cannot be inferred from the generic manual's “backup if supported” language; the recovery gap remains real.

Once the missing effect lands, a fan-policy apply is permitted only when all of these agree:

1. The host is on the declared firmware track.
2. A live observer normalizes the current controller config.
3. An exact board + firmware + config-path + service projection capability exists.
4. Writable persistent storage is observed on that same exact firmware.
5. The semantic curve differs from intent.

The realizer must stage on the same filesystem, validate JSON, atomically replace the upper-layer file, restart the controller, independently read back config/service/temperature/tachs, and automatically restore the prior file on any failed check. A BMC-reboot persistence witness remains a separate acceptance condition.

## Why srv3 and srv4 sound different

The curve points are the same. The operating points and controller/resource shapes are not.

| Observation | srv3 | srv4 |
| --- | --- | --- |
| BMC firmware | 2.07.00 | 3.22.00 |
| BIOS observed | 2.06 | 3.10 |
| TEMP_SOC range observed during investigation | 61–63 °C | 69–77 °C |
| Corresponding curve region | 40% | 50%, then a large step to 75% |
| Fan PID configuration | ten independent/alias entries | one aggregate entry over FAN1–FAN5 |
| Populated tach headers observed | FAN1, FAN3–FAN5 | FAN1–FAN4 |
| Complete Redfish thermal surface | `ThermalSubsystem` plus `Sensors` | legacy `Thermal` |
| `/Chassis/.../Sensors` members | 51, including thermals/fans | 9, omitting thermals/fans |
| `/Managers/bmc/FanControl` | absent | `InitialDuty` resource present |

The supplied srv4 snapshot showed `TEMP_SOC` at 75 °C and FAN1 near 9,482 RPM, which is consistent with the normal 75% curve step. It is not the 100% curve point; that begins at 85 °C. A missing temperature reading can request the same 75% duty, so an observation must retain whether `TEMP_SOC` was readable rather than collapsing both causes into “fan spike.”

During the comparison, srv4 was also doing more CPU work and showed roughly 49 W core-rail power versus roughly 42 W on srv3, with materially hotter SOC/core-VRD readings. The card-side reading was not hotter on srv4, so the evidence points first toward workload and CPU-local heat transfer/airflow rather than room temperature alone. If srv4 reaches 75 °C at comparable sustained work while srv3 remains near 62 °C, inspect the air baffle, CPU-fan orientation, heatsink mount pressure, and thermal-interface material.

Different fan models and headers produce different RPM at the same PWM duty. Compare requested PWM/curve region first; raw RPM is not a portable duty measurement.

The fan-related event storm on srv4 consisted of tach-zero assert/deassert transitions during the physical fan work and stopped after the replacement. `swampd` also had one startup `SIGSEGV` followed by a successful systemd restart; it remained active afterward. Treat a new controller restart or sensor-read error at the time of a spike as a separate fail-safe hypothesis, but neither was the continuing pattern in the post-repair log.

## Safe Redfish observation

Use a permission-restricted netrc file; do not place a password in shell history:

```sh
export BMC_HOST=192.168.1.195
export BMC_NETRC_FILE=/path/to/restricted.netrc

curl --fail-with-body -sS -k --netrc-file "$BMC_NETRC_FILE" \
  "https://$BMC_HOST/redfish/v1/Managers/bmc" |
  jq '{FirmwareVersion, DateTime, Status}'

curl --fail-with-body -sS -k --netrc-file "$BMC_NETRC_FILE" \
  "https://$BMC_HOST/redfish/v1/Chassis/ALTRAD8UD_1L2T" |
  jq '{Sensors, Thermal, ThermalSubsystem}'
```

Do not select the thermal query solely from intended firmware. Probe the chassis links or both read-only resource paths: intent can drift, and the observed resource surface is what determines query completeness.

For the 2.07 shape, temperatures and fans are members of:

```text
/redfish/v1/Chassis/ALTRAD8UD_1L2T/Sensors
```

For the 3.22 shape, use:

```text
/redfish/v1/Chassis/ALTRAD8UD_1L2T/Thermal
```

`gunbc.tools.bmc_read_telemetry` now queries the manager, probes `ThermalSubsystem`, `Thermal`, and `Sensors`, and refuses when it cannot establish a complete thermal surface. An HTTP-successful `Sensors` request by itself is deliberately insufficient.

Prefer Redfish or single D-Bus reads for diagnosis. Rapid direct polling of BMC hwmon sysfs produced sensor-read contention and a fail-safe event during this investigation; such a probe can perturb the controller it is trying to observe.

## Code authorities and witnesses

- Global valid BMC state: `gunbc.bmc_intent.fleet_bmc_valid_state`
- Per-host attachment of that state: `gunbc.fleet_intent`
- Exact, time-stamped firmware and Redfish observations: `gunbc.fleet_bmc_state`
- Board/firmware curve and controller observations: `extdeps.boards.asrock_rack`
- Exact board/firmware projection rule and observed srv4 capability: `gunbc.bmc_fan_projection` and `gunbc.fleet_bmc_state`
- Query completeness rule and fan-duty cause model: `extdeps.bmc.types`
- Shared-classification BMC plans: `gunbc.bmc_converge`
- Canonical phases and derived typed gaps: `gunbc.host_standup`
- Executable discriminating witness: `test.claim.bmc_firmware_thermal_witness`

The board's official product page and CPU-cooler airflow guidance are maintained by ASRock Rack:

- <https://www.asrockrack.com/general/productdetail.asp?Model=ALTRAD8UD-1L2T>
- <https://www.asrockrack.com/support/CPUCooler/ALTRAD8UD-1L2T.pdf>
- <https://www.asrockrack.com/support/faq.asp?id=65>
