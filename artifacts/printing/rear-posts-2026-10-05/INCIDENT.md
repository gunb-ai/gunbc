# Printer-02 failure, 2026-10-05

The durable evidence registry is [`product.printing.failure_incidents`](../../../dag/gunbc/product/printing/failure_incidents.dag), row `rear_fan_upright_20261005`. It records all three original Google Drive image URLs, HEIC SHA-256 hashes, the project digest, observations, uncertain cause and follow-up work. Local converted images are convenience copies, not the durable reference.

The fan grille (G140) and cradle (M140) were printed upright and failed; the exact onset mechanism is unconfirmed. The operator reports deterioration around 60% of the cage height. Do not treat printer completion as physical acceptance.

Verified archive corrections to the externally supplied assessment: brim width is **4 mm**, and M140 reaches **168 mm** high. A flat orientation reduces the upright thin-member risk; it does not eliminate every failure mode. The later [terminal report](terminal-report.json), read through `gunbc.fleet.printer_report.observe_both` before recovery, confirms the exact failed project in FINISH at 100% with error zero. This directly demonstrates that telemetry completion did not establish physical success.
