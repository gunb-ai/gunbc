set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
python3 - <<'P'
import subprocess,resource,time,sys
t=time.time()
out=open("census.out","w")
r=subprocess.run(["./target/release/gunbc","test","//gunbc/instruments:ambient-name-census"],stdout=out,stderr=open("census.err","w"))
out.close()
print("exit=%d elapsed=%.1fs maxrss_kb=%d"%(r.returncode,time.time()-t,resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss))
P
wc -c census.out; grep -c "^binding_declaration" census.out; grep -E "^disposition_summary|^subjects" census.out; tail -c 600 census.err
echo ---SORT_BY_ROWS; grep -P "\tsort_by\t" census.out | head -150; echo ---SORT_BY_DECLS; grep -P "^binding_declaration\t[^\t]*\t(sort_by|list_sort_by_key)\t" census.out | head
echo ---ERRS; grep -nE "^(error|warning)|-->" census.err | head -40 | cut -c1-300
