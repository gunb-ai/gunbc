S1 falsifier instrument (plan gunbc.plans.fabric_serving_router). Dissolves with extdeps.proxy.openresty.
1. Emit the config the proxy runs:
   gunbc run --source-root dag --entry dag/gunbc/serving/router_proxy_config.dag --function router_proxy_config_text \
     --arg listen_port=18080 --arg authz_port=18082 --arg stream_end_port=18082 > proxy.conf
2. python3 run.py <pinned openresty image> proxy.conf openresty -g 'daemon off;' -c /etc/p.conf
   (fakes.py 18081 18082 must be running: fake streaming upstream and authz/stream-end callouts.)
Controls: nginx -t accepts the config; release fires on normal end, client disconnect and upstream reset;
authz down refuses with no upstream request; proxy kill fires nothing (recovered by lease expiry).
