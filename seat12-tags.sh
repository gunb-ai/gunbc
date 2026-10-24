set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
python3 - <<'P'
import subprocess
o=open("all.out","w")
r=subprocess.run(["./target/release/gunbc","test","//gunbc/instruments:ambient-name-census"],stdout=o,stderr=subprocess.STDOUT)
o.close()
print("exit=%d"%r.returncode)
P
echo ---TAGS
awk -F'\t' '$1=="binding" && ($3=="Present"||$3=="Absent"){c[$3" "$5]++} END{for(k in c)print c[k],k}' all.out | sort
echo ---TOTAL; grep -ac "^binding	" all.out
