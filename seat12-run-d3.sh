set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
python3 - <<'P'
import subprocess,resource,time
t=time.time()
o=open("all.out","w")
r=subprocess.run(["./target/release/gunbc","test","//gunbc/instruments:ambient-operation-disposition-census"],stdout=o,stderr=subprocess.STDOUT)
o.close()
print("exit=%d elapsed=%.1fs maxrss_kb=%d"%(r.returncode,time.time()-t,resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss))
P
echo ---BYTES; wc -c all.out
echo ---DECLS; grep -ac "^binding_declaration" all.out
echo ---HEAD; grep -a "^subjects" all.out | cut -c1-600
echo ---SUMMARY; grep -a "^disposition_summary" all.out
echo ---TAIL; tail -c 700 all.out
echo ---FOLD; grep -aP "^disposition\t(fold|sort_by|map|filter|flat_map|split|join|any|all|count)\t" all.out | cut -c1-500
echo ---ALL; grep -a "^disposition	" all.out | cut -c1-330 | head -400
echo ---REFUSED; grep -aE "^(refused_rewritten|unobserved_rewritten)" all.out | head -20 | cut -c1-400; grep -aE "refused_rewritten" all.out | head -3 | cut -c1-300
