# Fan-print process observations

**Subject:** failed upright parts and the two `started-r6` recovery archives. Each `.process.json` is named by its input archive's SHA-256; original preparation/start receipts are unchanged. This retrospective reading caused no print or slicing operation.

**Method:** `product.printed_chassis.print_preparation.observe_process_batch` reads the archives and records command, bound and setting observations. Invoke `dag/product/printed_chassis/print_preparation.dag` with source roots `dag` and `src/v2`, `projects_json` as an archive-path array, and `output` as a fresh absolute directory; existing reports refuse overwrite. Inputs came from the cassette-posts archive and the digest-bound `fan-recovery/prepared-r6` copies identified by the receipts.

**Interpretation:** use the report fields rather than copied totals. They show repeated retraction and less commanded work after the posts finish; this motivates investigating cooling/workload but establishes neither overheating, sway, inadequate retraction nor a first failure. The photographed band is not registered to the path; pins and posts end at different heights.

Commanded-motion time omits acceleration, firmware limits, waits and explicitly unestimated motion. It cannot establish the cooling target was met; measured and slicer per-layer times remain absent. Raw settings preserve override sentinels without resolving vendor inheritance. Counts are commands, not islands or successful parts, and are not normalized between jobs. Physical recovery-part inspection is still required.
