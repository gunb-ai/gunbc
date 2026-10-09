# Fan-print process observations

Produced by `product.printed_chassis.print_preparation.observe_process_batch`,
using the same archive reader and process observer that preparation now uses.
Each `.process.json` filename is its input archive's SHA-256. Original start and
preparation receipts remain unchanged. This retrospective run did not start,
stop, reslice or modify any print.

| Archive digest prefix | Job | Slicer-marked layers | Retraction episodes | Nonextruding XY commands |
| --- | --- | ---: | ---: | ---: |
| `6c7fbb7afd5e` | Failed upright M140/G140 plus posts/pins | 1,389 | 23,379 | 124,316 |
| `c4bebe773bbc` | Flat M140, printer-01 recovery | 214 | 1,163 | 8,390 |
| `5c0d360672cd` | Flat G140, printer-02 recovery | 79 | 836 | 10,183 |

These are command observations, not geometric island counts or successful-part
counts. XY commands include wipes, and several commands can belong to one travel.
Whole-job totals are not normalized for layer count, object count or material.
The replacement archives are the exact files named in the `started-r6` receipts,
not the earlier preparation-only archives.

## What the evidence changes

All three archives carry the same raw settings for `retraction_length` (0.8),
`retraction_speed` (30), `deretraction_speed` (30), `wipe` (1), `wipe_distance` (2),
`z_hop` (0.4), and `retraction_minimum_travel` (1). Their corresponding filament
overrides are `nil`. `slow_down_for_layer_cooling` is 1,
`slow_down_layer_time` is 8, and `slow_down_min_speed` is 20. The reports retain
the original values/types and all other project settings, including override
sentinels; they do not independently resolve vendor inheritance. Retraction was
commanded repeatedly in the failed job, not simply disabled.

The failed archive's per-object bounds put all CP pins' final deposited height at
28 mm and all CU posts' at 102 mm. M140 continues to 168 mm and G140 to 152 mm.
Among layers within 0.5 mm of 100 mm, median commanded-motion seconds is about
27.86; around 110 mm it is 10.64. Median retraction episodes in those same windows
are 28 and 20. Above the posts, the remaining geometry therefore has substantially
less commanded work per layer, while retaining frequent retraction/travel.

This is consistent with investigating a change in cooling/workload when the posts
finish. It does **not** establish overheating, inadequate retraction, column sway,
or the first failure event. The photographed fuzz band's height has not been
registered to the toolpath. Pins and posts finish at different heights. The claim
that constant settings cannot produce a localized band is not supported here.

## Timing and interpretation limits

`commanded_motion_seconds` sums path length / commanded feed. It omits
acceleration, firmware limits, synchronization, dwell and temperature waits. The
failed, flat-cradle and flat-grille reports contain respectively 2,462, 429 and
135 unestimated motion commands (for example nondepositing arcs). These are
explicitly counted rather than assigned zero duration silently. Neither measured
nor slicer-provided per-layer times are available; the fields are null. The proxy
cannot establish whether the 8-second cooling target was met.

The flat parts retain travel and retraction; orientation does not eliminate them.
These observations do not classify a plate as safe/unsafe or prescribe new
retraction values. Physical inspection of the recovery parts is still needed.

## Reproduction

Use `gunbc run` with both source roots (`dag`, `src/v2`), entry
`dag/gunbc/product/printed_chassis/print_preparation.dag`, function
`observe_process_batch`, `projects_json` containing a JSON array of archive paths,
and `output` naming a fresh absolute directory. The generated reader and reports
are retained there; existing report files refuse overwrite.

Input archives for this run were the failed
`/home/briansrls/print-prep-2026-10-05/cassette-posts/rear-and-upper-posts.3mf`
and the two digest-bound files copied from srv1's `fan-recovery/prepared-r6` into
`/home/briansrls/print-prep-2026-10-05/fan-recovery-r6`.
