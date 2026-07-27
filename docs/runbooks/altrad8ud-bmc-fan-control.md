# ALTRAD8UD BMC fan control and telemetry

This runbook records the srv3/srv4 investigation from 2026-07-27. Both machines are ASRock Rack ALTRAD8UD-1L2T boards. The fleet-wide desired state is now OpenBMC major track 3 plus the TEMP_SOC fan policy below. srv3's observed 2.07.00 is therefore drift; srv4's observed 3.22.00 is in the valid firmware band.

“3.x” is a validity rule, not an instruction to download “latest.” Firmware mutation still requires one exact, approved, board-compatible image URI and digest.

## What actually owns the fan curve

On both observed firmware versions, `phosphor-pid-control.service` runs `/usr/bin/swampd`. The selected configuration is:

```text
/usr/share/swampd/config.json -> /usr/share/swampd/config-asrr.json
```

The factory board configuration controls all five fan outputs from `TEMP_SOC` with this observed policy:

| Factory TEMP_SOC at or above | Requested duty |
| ---: | ---: |
| 40 °C | 30% |
| 55 °C | 40% |
| 65 °C | 50% |
| 75 °C | 75% |
| 85 °C | 100% |

The fleet-owned desired policy is deliberately flat at the factory-established minimum through 90 °C, then becomes an emergency ramp:

| Desired TEMP_SOC at or above | Requested duty |
| ---: | ---: |
| 40 °C | 30% |
| 75 °C | 30% |
| 90 °C | 30% |
| 93 °C | 55% |
| 95 °C | 70% |
| 97 °C | 85% |
| 98 °C | 100% |

Rising hysteresis is 1 °C and falling hysteresis is 2 °C. The cited Ampere Altra Max M128-30 limits are 100 °C maximum continuous junction temperature, 105 °C throttle, and 120 °C shutdown. Because phosphor-pid-control only recomputes after the input moves farther than rising hysteresis, the 98 °C full-duty step ensures effective full cooling below 100 °C. A missing `TEMP_SOC` or a real zone failure requests 75%.

The BMC 3.22 web UI's **Fan Control** resource changes `InitialDuty`; it is not the authority for these temperature/duty points.

On srv4, `/usr/share` is visible through the root overlay: a read-only SquashFS lower layer on `/dev/mtdblock4` and a writable JFFS2 upper layer on `/dev/mtdblock5`. The live converge creates an upper-layer override through an atomic same-filesystem replacement. Persistence across one graceful BMC restart is now witnessed below. A firmware replacement remains a distinct event and must still be followed by rediscovery and readback.

The controller topology is part of desired state, not incidental JSON. srv4 has four installed fans (`FAN1`–`FAN4`) while the factory aggregate PID still referenced an unpopulated `FAN5`. `FAN5=nan` held `zone0` in failsafe. The factory zone fallback was 30%, so normal calculated outputs could mask that state. Raising the fallback to 75% without removing stale FAN5 made the latent failsafe immediately audible. The v2 projection therefore owns both the TEMP_SOC curve and the exact four-input aggregate PID topology, and no-op additionally requires `zone0 FailSafe=false`.

## How BMC convergence fits host convergence

Ubuntu and BMC convergence share one lifecycle:

```text
desired state -> live observation -> classify -> typed plan -> apply -> independent readback -> re-drive/no-op
```

The shared state is `std.upsert_decision.UpsertClassification` and `ObservationVerdict`; the shared apply/readback result is `std.realization_reconcile.Reconciliation`. Host OS and BMC code differ only where reality differs: observer, control-plane transport, typed effect, and safety proof. A BMC shell/Redfish operation must not masquerade as a `HostOs` shell effect.

`PhaseDisposition` is now a product of `authorities` and typed `gaps`. That permits the honest middle state: comparison logic exists, but a live observer or actuator is still missing. The operator gap view is derived from the spine by `host_standup_gap_rows()`; there is no second hand-maintained gap ledger.

For the current local bootstrap, GCP access is an ensure/upsert at the point of demand, not a standup phase. A selected operation must first require a Secret Manager read/write; only that branch calls `gcloud auth print-access-token` to observe the access capability. A live session classifies `Converged/Noop`. An absent or expired session classifies `Absent/Apply(HumanGcloudLogin)`: run `gcloud auth login` with an account authorized for the fleet secrets project, then re-run the same convergent command. A path that does not need GCP never evaluates the ensure and never pauses for gcloud. The token is captured internally and must not be pasted into intent, argv, a netrc committed to the tree, or the PR. A workload-identity/OAuth handler can later realize the same ensure at the shared `AccessTokenSource` seam.

The managed-credential checks `srv3_managed_credential_probe_check` and `srv4_managed_credential_probe_check` both completed live on 2026-07-27. On each host, the factory credential was rejected with HTTP 401, the active gcloud session satisfied the access ensure without human intervention, the Secret Manager `payload.data` value was decoded from standard base64, and the stored credential authenticated to the BMC. The expected factory-auth failure rendered only `<redacted-secret>` in argv diagnostics. Password SSH now consumes that same `Secret` through `sshpass -d 0`: the password is stdin/file-descriptor data, never argv or remote-script text. The remaining credential gap is the generic lifecycle for netrc/Redfish/IPMI consumers, not this fan driver's SSH binding.

The current spine has 13 named gaps, 9 of them in the BMC prefix. Fan live observation, persistent overlay actuation, automatic rollback/readback, immediate thermal/tach/controller proof, and graceful-reboot persistence are now authorities rather than gaps. Remaining BMC gaps cover generic credential binding; live firmware observation; exact approved artifact; update actuator; post-update return/readback and recovery; full subsumption-entrypoint wiring; live Redfish surface rediscovery; and a workload-qualified thermal envelope. The remaining four are pre-existing OS/assimilation gaps.

## What subsumption does today

| Host | Snapshot classification | What a current subsumption run does |
| --- | --- | --- |
| srv3 | Firmware `Drifted`; fan realization `Conflict` until firmware converges and surfaces are rediscovered | `srv3_bmc_fan_converge` observes and refuses before mutation. It does **not** update 2.07 to 3.x because no exact approved image/digest or firmware actuator is wired. |
| srv4 | Firmware `Converged`; curve/topology/controller state are live-observed | `srv4_bmc_fan_converge` applies drift or proves no-op through independent readback. The 2026-07-27 live run is now `Noop`. |

The executable observer first reads only `/etc/os-release`, using no version-specific utility or JSON layout. It compares that live identity to the global firmware track before querying controller configuration. This matters on srv3: OpenBMC 2.07 has no `jq`, but the current read-only result is now the intended typed refusal—`observed OpenBMC 2.07.00; fleet valid state requires OpenBMC 3.x`—rather than an accidental missing-tool error. A valid-track version still requires an exact board/version projection before any detailed observation or mutation.

The fan prefix now has per-host subsumption entrypoints:

```sh
target/release/gunbc run \
  --source-root dag --source-root src/v2 \
  --entry dag/gunbc/bmc_fan_converge.dag \
  --function srv4_bmc_fan_observe

target/release/gunbc run \
  --source-root dag --source-root src/v2 \
  --entry dag/gunbc/bmc_fan_converge.dag \
  --function srv4_bmc_fan_converge
```

The read-only function exits nonzero with `safe drift: apply required` when mutation is admissible; it exits zero only for an independently safe no-op. `srv4_bmc_fan_rollback_previous` restores the exact-version transaction snapshot. `gunbc converge` still drives the `HostOs` policy only, so its receipt is not yet a whole-host BMC receipt. The retained `bmc-subsumption-entrypoint-wiring` gap is for composing firmware, surface rediscovery, and fan phases into that top-level driver—not for the now-live fan prefix.

Read-only Redfish checks found `/redfish/v1/UpdateService/update` on both machines and marked each active BMC image `Updateable=true`. Each inventory exposed exactly one BMC image and no backup BMC image. The update transport shape therefore exists, but rollback cannot be inferred from the generic manual's “backup if supported” language; the recovery gap remains real.

A fan-policy apply is permitted only when all of these agree:

1. The host is on the declared firmware track.
2. A live observer normalizes the current controller config.
3. An exact board + firmware + config-path + service + zone + aggregate-PID projection capability exists.
4. Writable persistent storage is observed on that same exact firmware.
5. The semantic curve or installed-tach topology differs from intent.
6. `TEMP_SOC`, every required component temperature, and all four installed tachs are readable and within their pre-apply bounds.

The realizer stages on the same filesystem, validates JSON, atomically replaces the upper-layer file, restarts the controller, requires the zone to leave failsafe, independently reads back config/topology/service/temperature/tachs, and automatically restores the prior file on any failed check. A workload-qualified envelope remains a separate acceptance condition.

## Live srv4 execution receipt

The 2026-07-27 execution exercised both rollback and successful convergence:

1. Read-only classification observed exact 3.22 support, writable overlay storage, safe temperatures, and four healthy installed tachs.
2. The first transaction exposed the latent `FAN5=nan` failsafe when zone fallback was raised to 75%. The exact prior factory file was restored and the controller restarted successfully.
3. The projection was corrected to own the installed topology (`FAN1`–`FAN4`) and typed zone failsafe observation.
4. The second transaction atomically applied curve plus topology. Independent readback proved semantic equality, four nonzero tachs, required component temperatures, active service, and `FailSafe=false`.
5. Twelve live samples over six minutes stayed safe. `TEMP_SOC` ranged 72–77 °C and ended at 73 °C. Guarded DIMMs peaked at 69 °C, X550 ended at 68.625 °C, FAN1 ended at 3,616 RPM, and FAN2–FAN4 ended at 889–943 RPM.
6. Re-driving `srv4_bmc_fan_converge` must now be a safe `Noop`; that idempotency check is part of the handoff test sequence.
7. A later pre-reboot check found BMC SSH password auth temporarily refusing while the same managed credential remained valid through Redfish. The advertised `/Managers/bmc/Actions/Manager.Reset` `GracefulRestart` action was therefore used instead of treating SSH as the only control plane.
8. Redfish accepted the BMC-only reset with HTTP 200. The BMC was observed down at 23:09:49Z and back at 23:12:26Z, a 157-second observed outage. srv4's host SSH remained reachable throughout and its Redfish `PowerState` remained `On`.
9. Before reboot, `TEMP_SOC` was 85 °C and FAN1–FAN4 were 3,676/960/948/896 RPM. After return they were 83 °C and 3,659/961/944/906 RPM.
10. Post-reboot SSH recovered. The active path was still `/usr/share/swampd/config-asrr.json`, its SHA-256 remained `4812a134939f6e6eba9c90fe15fa93cc05936d0d2ef1a4365cc4ee0a4d262481`, the controller was active, `FailSafe=false`, the semantic observer returned `ExitSuccess`, and a converge re-drive returned `ExitSuccess` through the no-op path.

The reboot receipt proves this exact JFFS2 upper override across one graceful BMC restart. It does not prove persistence across a firmware replacement. The thermal samples remain short equilibrium receipts under the workloads present during the runs, not the still-open workload-qualified operating envelope.

## Why srv3 and srv4 sound different

The observed factory curve points were the same. The operating points and controller/resource shapes were not.

| Observation | srv3 | srv4 |
| --- | --- | --- |
| BMC firmware | 2.07.00 | 3.22.00 |
| BIOS observed | 2.06 | 3.10 |
| TEMP_SOC range observed during investigation | 61–63 °C | 69–77 °C |
| Corresponding factory curve region | 40% | 50%, then a large step to 75% |
| Fan PID configuration | ten independent/alias entries | one aggregate entry over FAN1–FAN5 |
| Populated tach headers observed | FAN1, FAN3–FAN5 | FAN1–FAN4 |
| Complete Redfish thermal surface | `ThermalSubsystem` plus `Sensors` | legacy `Thermal` |
| `/Chassis/.../Sensors` members | 51, including thermals/fans | 9, omitting thermals/fans |
| `/Managers/bmc/FanControl` | absent | `InitialDuty` resource present |

The supplied srv4 snapshot showed `TEMP_SOC` at 75 °C and FAN1 near 9,482 RPM, which was consistent with the factory 75% curve step. It was not the 100% curve point; that began at 85 °C. A missing temperature reading could request the same 75% duty, so an observation must retain whether `TEMP_SOC` was readable rather than collapsing both causes into “fan spike.”

The live apply also demonstrated why curve equality alone is insufficient. The first transaction changed the zone fallback from 30% to 75% but retained uninstalled FAN5 as an input. The controller correctly reported `FailSafe=true` and held FAN1 near 9,500 RPM. Automatic transaction rollback was exercised, then the model was corrected to own topology and failsafe state. The second transaction removed FAN5 from the aggregate PID, produced `FailSafe=false`, and at 74 °C reduced FAN1 to about 3,600 RPM and FAN2–FAN4 to about 890–940 RPM.

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
- Password-on-fd-0 SSH transport: `extdeps.ssh.password_session`
- Live observe/apply/rollback/readback driver: `gunbc.bmc_fan_converge`
- Canonical phases and derived typed gaps: `gunbc.host_standup`
- Executable discriminating witnesses: `test.claim.bmc_firmware_thermal_witness` and `test.claim.bmc_fan_converge_witness`

The board's official product page and CPU-cooler airflow guidance are maintained by ASRock Rack:

- <https://www.asrockrack.com/general/productdetail.asp?Model=ALTRAD8UD-1L2T>
- <https://www.asrockrack.com/support/CPUCooler/ALTRAD8UD-1L2T.pdf>
- <https://www.asrockrack.com/support/faq.asp?id=65>

Thermal/controller semantics are grounded in the Ampere Altra Max datasheet and upstream phosphor-pid-control documentation:

- <https://amperecomputing.com/assets/Altra_Max_Rev_A1_DS_v1_15_20230809_b7cdce449e_424d129849.pdf>
- <https://github.com/openbmc/phosphor-pid-control/blob/master/configure.md>
