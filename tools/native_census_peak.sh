#!/usr/bin/env bash
# Peak RSS and wall of one native census verb over the full corpus, with RSS sampled over time.
# usage: [NFILES=<first N sorted .dag files>] native_census_peak.sh <verb: census|census-infer> [interval_seconds=10]
# Each sample also reports modules=<per-module stderr lines so far>, so RSS is read against modules processed.
# Run from the repo root in ONE dispatch (the binary it builds must execute where it was built):
#   ctrl-build --remote -- bash tools/native_census_peak.sh census-infer
# The emitted closure binary is the one //gunbc/instruments:native-process-memory-census compiles;
# the verb is a verb of that binary, not of gunbc. Peak is ru_maxrss read once at exit; the
# sampled VmRSS series locates the phase that grows.
set -u
VERB=${1:?verb}; EVERY=${2:-10}
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
rm -rf /tmp/gunbc-emit-compile-*
setsid ./target/release/gunbc test //gunbc/instruments:native-process-memory-census > /tmp/inst.log 2>&1 &
IP=$!
for i in $(seq 1 900); do
  B=$(ls /tmp/gunbc-emit-compile-*/target/release/gunbc-emitted-closure-* 2>/dev/null | grep -v '\.d$' | head -1)
  [ -n "$B" ] && [ -x "$B" ] && break; sleep 3
done
[ -n "${B:-}" ] || { echo "NO_BINARY"; exit 3; }
cp "$B" /tmp/native_bin; sleep 2; kill -- -$IP 2>/dev/null; sleep 2
if [ -n "${NFILES:-}" ]; then
  find dag src/v2 -name '*.dag' | sort | head -"$NFILES" > /tmp/subset.txt
  rm -rf /tmp/subset; mkdir -p /tmp/subset/dag /tmp/subset/src/v2
  while read f; do mkdir -p "/tmp/subset/$(dirname "$f")"; cp "$f" "/tmp/subset/$f"; done < /tmp/subset.txt
  cd /tmp/subset
fi
echo "head=$(git rev-parse HEAD) verb=$VERB files=$(find dag src/v2 -name '*.dag' | wc -l)"
python3 - "$VERB" "$EVERY" <<'P'
import resource,subprocess,sys,time
verb,every=sys.argv[1],int(sys.argv[2])
t=time.time()
p=subprocess.Popen(['/tmp/native_bin',verb,'dag','src/v2'],stdout=open('/tmp/peak.out','w'),stderr=open('/tmp/peak.err','w'))
while p.poll() is None:
    try:
        rss=hwm=0
        for l in open('/proc/%d/status'%p.pid):
            if l.startswith('VmRSS'): rss=int(l.split()[1])
            if l.startswith('VmHWM'): hwm=int(l.split()[1])
        mods=open('/tmp/peak.err','rb').read().count(b'[native-census-infer]')
        print('t=%d rss_kb=%d hwm_kb=%d modules=%d'%(time.time()-t,rss,hwm,mods),flush=True)
    except OSError: pass
    time.sleep(every)
print('exit=%d wall_s=%.1f peak_kb=%d'%(p.returncode,time.time()-t,resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss))
P
echo files=$(find dag src/v2 -name '*.dag' | wc -l); wc -l /tmp/peak.out /tmp/peak.err; tail -3 /tmp/peak.err | cut -c1-300
