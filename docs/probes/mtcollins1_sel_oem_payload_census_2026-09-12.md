# OEM SEL payload census — Mt. Collins unit 1, whole log

Scored against the whole SEL rather than the tail, because the tail is what every prior round
read and the tail cannot show co-emission. Artifact:
`artifacts/bmc/mtcollins1-sel-full-2026-09-12.txt`, sha256
`8d93cb24349549ef4fefb884a56dbe9f72cc03b0530fa559afdca089b5250f89`, 1291 records,
`ipmitool sel list` verbatim. The BMC cannot decode these itself
(`advanced_event_description: "unknown"`), so this is the only available structure.

## Payload families present, with counts

| Payload | Count | Family |
|---|---|---|
| `12c805004000` / `12c805000000` | 50 / 50 | `12c8` |
| `12c87f010000` | 9 | `12c8` |
| `12c800000000` | 1 | `12c8` |
| `0fde70330102` | 19 | `0fde` |
| `0fde70331102` | 10 | `0fde` |
| `0fde70330502` | 2 | `0fde` |
| `0fde703b1102` | 1 | `0fde` |
| `0fde7033ff10` / `0fde703bff10` | 3 / 3 | `0fde` |
| `caee7107ffff` / `caeef107ffff` | 8 / 8 | `caee` |
| `caeff104ffff` / `caef7104ffff` | 5 / 5 | `caee` |
| `caeef104ffff` / `caee7104ffff` | 4 / 4 | `caee` |
| `caeff107ffff` / `caef7107ffff` | 3 / 3 | `caee` |
| `caeef114ffff` / `caee7114ffff` | 3 / 3 | `caee` |
| `caeef102ffff` / `caee7102ffff` | 1 / 1 | `caee` |

## The new finding: co-emission, and its absence

`0fde7033ff10` and `0fde703bff10` occur at the SAME SECOND with ADJACENT record IDs, three
separate times (`ad`/`ae` 23:33:08, `b4`/`b5` 23:38:18, `bb`/`bc` 23:43:29, all 2026-08-19).

The differing bit is exactly bit 3 of byte 3 — the bit two controlled crossovers had already
shown tracking the intervened socket. A record emitted once per socket, both in the same
second, is a SECOND AND INDEPENDENT line of support for the socket reading: it does not go
through the crossover at all, so the companion-population confound that bounded the crossover
inference does not apply to it.

Every FAILURE payload is UNPAIRED. `0fde70330102` (19), `0fde70331102` (10),
`0fde70330502` (2) each appear alone at their timestamps; `0fde703b1102` appears once, in the
round where the quartet was in socket 1, with no `33` companion. So on a training failure
exactly ONE socket record is emitted, and it is the socket carrying the intervened modules.

## What this earns, and what it does not

EARNED, at inference with two independent supports: byte 3 bit 3 selects the socket.
The supports are a declared-in-advance crossover (twice) and same-second co-emission (three
times, a different mechanism).

STILL NOT EARNED: that the emitting socket is the FAILING socket rather than the reporting
one; the stage, status and tail readings; any mapping from payload bytes to a physical
connector. No vendor documentation for any of this exists. **This may still not be used to
name a DIMM.**

The `caee` family pairs at equal counts too, but its pairing axis is INCONSISTENT — the 2022
records pair on byte 2 bit 7 (`caee71xx` then `caeef1xx`, one second apart) while 2026 records
pair on byte 1 bit 0 (`caef7104` then `caee7104`). So `caee` pairing is an observation with no
reading attached, and the `0fde` result must not be generalized onto it.

## Retry outcome, round 3 of the 2DRx4 quartet

Power-up 21:57:57Z, restart 22:03:26Z (5m29s), payload `0fde70331102` — the same payload as
the previous socket-0 round. Followed 23s later by `caee7107ffff` + `caeef107ffff`.

The armed recorders captured NOTHING: `logs/video` is `[ ]` and
`settings/sol-recorded-video` is `[ ]`, with `settings/video/triggers` and
`settings/video/sol-triggers` both confirmed armed on power-on/reset/critical/non-recoverable/
watchdog and `pre-event` enabled. So arming the recorders did not open the pre-OS window, and
the hypothesis that they would capture DRAM training output is FALSIFIED for this controller.

## BMC web-stack fault, recorded because it will recur across 20 units

`POST /api/session` over plain HTTP returns a token and a `QSESSIONID`, and every subsequent
authenticated `/api/*` call with that session returns `{"cc": 7, "error": "Invalid
Authentication"}`. The identical sequence over HTTPS works. `GET /` over HTTP returns 200, so
the web server is serving; it is the HTTP session that is not honoured.

Separately, the web UI reported the host "powered off" while the BMC's OWN API answered
`{"power_status": 1}`, IPMI answered `Chassis Power is on`, and CPU0 rails read 0.62–2.50 V
`ok`. The UI's power widget disagreed with the controller behind it — a UI fault, not a
power state. Any convergence flow that reads power state must read it from IPMI or
`/api/chassis-status`, never from the rendered page.
