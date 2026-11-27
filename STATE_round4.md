
## Round-8 monitor/driver ops (2026-11-27/28)
- CI at 8dad46d4e2b: v2-bootstrap pass 1h20m, heal pass 1h18m, shadow-gate config pass = GREEN.
- Claims sweep: 20/48 rc=0 at relaunch. OOM killed first driver pair once; relaunched 2x (setsid, staggered 120s). Monitor relaunched -> /tmp/monitor2.log (6h bash cap; relaunch if it dies before CLAIMS-COMPLETE).
- cursor_runtime_root_wet_witness entry split: 2 activation_* fns removed from sweep line (need /etc/gunbc-cursor-harness/api-key custody key, absent on this host) -> documented-not-run in table. 5 remaining wet fns run under sweep.
- abc trio (workspace_network_producer, fleet/fleet_converge_plan_binary_presence, compute/remote_build_serving_converge) ARE inside the 48 (earlier separate reruns were redundant); their lines carry the fixed labels //test/claim/....
- Line format of final_entries.txt: MODULE|FN1,FN2,...|SLUG (3 fields).
