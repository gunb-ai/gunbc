# Appends a chunks_<tag>() entry per split module to wrprobe/probe/wr_probe.dag.
# Usage: python3 wrprobe/split.py <module.dag> <tag>; python3 wrprobe/gen_chunk_entries.py <tag>...
# Run: gunbc run --source-root dag --source-root src/v2 --source-root wrprobe/probe \
#        --entry wrprobe/probe/wr_probe.dag --function chunks_<tag>
# (returns a String, so gunbc prints it inside a "not ProcessExit" refusal; the text is the report)
import sys
add=''
for t in sys.argv[1:]:
    ps=open(f'wrprobe/chunks/{t}/paths.txt').read().split('\n')
    lst=',\n    '.join(f'"{p}"' for p in ps)
    add+=f'\nfn chunks_{t}() -> String {{\n  fold_list(xs: [\n    {lst}\n  ], empty: "", cons: fn(acc, p) {{ concat(acc, verdict(path: p)) }})\n}}\n'
open('wrprobe/probe/wr_probe.dag','a').write(add)
