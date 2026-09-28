# probe (not for merge): run the emit probe in a pruned source tree so a 7 GB runner fits.
B=${1:?claim_batch}
E="--entry src/v2/test/manual/emitprobe/emit_probe.dag"
for f in $(cat src/v2/test/manual/emitprobe/host_entries.txt); do E="$E --entry $f"; done
$B --print-entry-closure --source-root dag --source-root src/v2 $E > clo.txt 2>&1
grep -o 'file="[^"]*"' clo.txt | sed 's/file="//;s/"$//' | sort -u > files.txt
echo FILES $(wc -l < files.txt); grep entry-closure-summary clo.txt
rm -rf /tmp/mini; mkdir -p /tmp/mini; tar cf - -T files.txt | tar xf - -C /tmp/mini
cd /tmp/mini; git init -q .; touch Cargo.toml
CG=/sys/fs/cgroup/emitprobe; mkdir -p $CG; echo 6000000000 > $CG/memory.max
bash -c "echo \$\$ > $CG/cgroup.procs; GUNBC_FLATTEN_SITE_DUMP_SECS=100000 exec $B --claim-run --source-root dag --source-root src/v2 --entry src/v2/test/manual/emitprobe/emit_probe.dag --functions probe_ingest_only_holds,probe_ingest_emit_holds" > out.txt 2>&1
echo EXIT $? peak=$(cat $CG/memory.peak 2>/dev/null)
grep -E "fn-self|fn-calls|witness\]|PASS|FAIL" out.txt | cut -c1-200 | head -210
grep -v "^  \|fn-self\|fn-calls\|eval-profile" out.txt | tail -8 | cut -c1-300
