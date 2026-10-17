import subprocess, sys, re
ref=sys.argv[1]; want=sys.argv[2]
pat = 'declaration_denotation_of_type(t:' if want=='reader' else 'declaration_reference_path_optional(node:'
head_re = re.compile(r'^(?:fn|type|data)\s+([A-Za-z0-9_.]+)')
# all dag files under src/v2 and dag, excluding test/claim and recurring_failure_mode
files=subprocess.run(['git','ls-tree','-r','--name-only',ref,'--','src/v2','dag'],capture_output=True,text=True).stdout.split()
files=[f for f in files if f.endswith('.dag') and '/test/' not in f and 'recurring_failure_mode' not in f]
out=[]
for f in files:
    src=subprocess.run(['git','show',f'{ref}:{f}'],capture_output=True,text=True).stdout.splitlines()
    fn=None
    for i,l in enumerate(src,1):
        st=l.rstrip()
        m=head_re.match(st)
        if m: fn=m.group(1)
        if pat in l: out.append((f,i,fn))
for f,i,fn in out: print(f'{f}:{i}|{fn}')
