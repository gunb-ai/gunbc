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
echo ---BYTES; wc -c < all.out
echo ---SHA; git rev-parse HEAD
for k in binding binding_unanswered rewritten module_subjects; do echo "---ROWS $k $(grep -aP "^$k\t" all.out | LC_ALL=C sort -u | wc -l) $(grep -aP "^$k\t" all.out | LC_ALL=C sort -u | sha256sum | cut -c1-16)"; done
echo ---DECLROWS; grep -acP "^binding_declaration\t" all.out
echo ---TAGS
awk -F'\t' '$1=="binding" && ($3=="Present"||$3=="Absent"){c[$3" "$5]++} END{for(k in c)print c[k],k}' all.out | sort
grep -aP "^(binding|binding_unanswered|rewritten|module_subjects)\t" all.out | LC_ALL=C sort -u > rows.sorted
echo ---ROWSFILE; wc -l < rows.sorted
echo ---REWRITTEN; grep -aP "^rewritten\t" all.out | cut -c1-300
echo ---REFUSED; grep -aP "^(unreached|refused_rewritten|refused)\t" all.out | cut -c1-300 | head -40
echo ---TAIL; grep -a "rewritten module\|refused at resolve\|census:" all.out | cut -c1-400 | head -8
echo ---ROWSB64; gzip -9 < rows.sorted | base64 -w0; echo; echo ---END
