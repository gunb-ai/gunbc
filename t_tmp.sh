mkdir -p /sys/fs/cgroup/gunbcrun; echo 51539607552 > /sys/fs/cgroup/gunbcrun/memory.max; echo $$ > /sys/fs/cgroup/gunbcrun/cgroup.procs
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -2
mkdir -p /var/lib/gunbc/materialization-store; ls -ld /var/lib/gunbc/materialization-store; id -u
( while sleep 5; do echo "MEM $(date +%T) cur=$(cat /sys/fs/cgroup/gunbcrun/memory.current) $(grep -E "^(anon|file|kernel|slab) " /sys/fs/cgroup/gunbcrun/memory.stat | tr "\n" " ") rss=$(grep VmRSS /proc/$(pgrep -nf "release/gunbc run")/status 2>/dev/null) ev=$(grep oom_kill /sys/fs/cgroup/gunbcrun/memory.events) holds=$(ls /var/lib/gunbc/materialization-store | wc -l)"; done > /tmp/mem.out 2>&1 & ) ; GUNBC_TYPECHECK_STORE=durable target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/docs_projection_gate.dag --function regen > /tmp/d.out 2>&1
echo EXIT=$?; sort -k3 -n /tmp/mem.out | tail -2; tail -14 /tmp/mem.out; grep -a 'Maximum resident\|Elapsed' /tmp/d.out; cat /sys/fs/cgroup/gunbcrun/memory.events
grep -a "typecheck-store" /tmp/d.out | cut -c1-300; grep -a "tmp-restore" /tmp/d.out | tail -6
grep -av "^advisory\|^ *[0-9]* |" /tmp/d.out | tail -25 | cut -c1-400
echo ALLDONE
