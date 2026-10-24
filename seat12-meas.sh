set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
python3 - <<'P'
import subprocess,resource,time
t=time.time()
o=open("all.out","w")
r=subprocess.run(["./target/release/gunbc","test","//gunbc/instruments:ambient-name-census"],stdout=o,stderr=subprocess.STDOUT)
o.close()
print("exit=%d elapsed=%.1fs maxrss_kb=%d"%(r.returncode,time.time()-t,resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss))
P
echo ---BYTES; wc -c all.out; grep -ac "^binding	" all.out; tail -c 200 all.out
