butch (srv14) serving bring-up, 2026-11-10. Operator's hand deployment and its readbacks, relayed verbatim by the operator in smart-crane-730's session.

DEPLOYMENT (hand, operator)
- systemd template strata-pool@.service; instances strata-pool@a and strata-pool@b, enabled; each Requires a2-power-cap.service (all 4 A2 at 35 W).
- Same build and server as srv3 production, LAN-bound, 24 pool workers per server. The old 1x4 deployment under /opt/strata is untouched and disabled.
- butch-a 192.168.1.205:18081 GPUs 0,1 CPUs 0-39; butch-b :18082 GPUs 2,3 CPUs 40-79; model qwen3.8-flash-next-iq3_s; max_model_len 229376.
- One API key for both servers: ~/strata-pool/api_key on butch. Not yet in Secret Manager (proposed name gunbc-srv14-strata-api-key, after gunbc-srv4-strata-api-key).

ROUTER (hand edit on srv1, ~17:53Z): two entries in /home/briansrls/router/upstreams.json, inserted after srv4-strata.
Backup taken first: upstreams.json.bak-before-butch-add-20261110T175300Z. Checked with openresty -t, then gunbc-router reloaded gracefully.
{"name":"butch-a","scheme":"http","host":"192.168.1.205","port":18081,"key":"<butch strata api_key>","model":"qwen3.8-flash-next-iq3_s","cap":2,"max_model_len":229376,"overflow":false,"path_prefix":"","retiring_names":[],"retain":4}
{"name":"butch-b","scheme":"http","host":"192.168.1.205","port":18082,"key":"<butch strata api_key>","model":"qwen3.8-flash-next-iq3_s","cap":2,"max_model_len":229376,"overflow":false,"path_prefix":"","retiring_names":[],"retain":4}
The key field holds the literal key on srv1; it is redacted here.
Router test before the reboot: 8 requests with fresh session ids; butch took 4 (2 per server), all OK; 3 got 429 because every other strata server's sticky slots were full.

CONTROLLED REBOOT: butch had no active requests, sticky sessions or open connections. Issued 17:57:27Z, OS up 18:00:10Z, both engines answering /v1/models at 18:01:32Z.
$ systemctl is-active strata-pool@a strata-pool@b a2-power-cap
active
active
active
$ nvidia-smi --query-gpu=uuid,power.limit --format=csv
uuid, power.limit [W]
GPU-c7080fc0-73f7-5c2d-0412-c5041edd95f4, 35.00 W
GPU-d6551356-b835-59e5-8ea9-542098216e6f, 35.00 W
GPU-63e42447-95fe-67cb-afde-3e2e80e4e389, 35.00 W
GPU-b44c2926-091a-b3ac-3d11-bda9613c67fb, 35.00 W
direct completions: port 18081 -> "ready" (33 tokens); port 18082 -> "ready" (30 tokens)
through the srv1 router (https://192.168.1.203:18443, fresh X-Session-Id per request): 4/4 requests returned 200 in 4.5-6.2 s; router /router/stats upstream:butch-a:200 went 2 -> 4, upstream:butch-b:200 went 2 -> 4.

ALSO OBSERVED: ctrl-vast-pool was removed from the live router at 17:52:43Z by another session (backup upstreams.json.bak-before-ctrl-retire-20261110T175243Z). srv1-overflow-cap-sync.timer has failed every minute since.
