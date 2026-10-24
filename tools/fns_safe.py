import sys, re
# top-level test fns only: skip line comments, block comments, and string literals (escapes handled)
src = open(sys.argv[1]).read()
out=[]; i=0; n=len(src)
state=None  # None | 'line' | 'block' | 'str'
while i < n:
    c=src[i]
    if state=='line':
        if c=="\n": state=None
        i+=1; continue
    if state=='block':
        if src.startswith("*/",i): state=None; i+=2; continue
        i+=1; continue
    if state=='str':
        if c=="\\": i+=2; continue
        if c=='"': state=None
        i+=1; continue
    if src.startswith("//",i): state='line'; i+=2; continue
    if src.startswith("/*",i): state='block'; i+=2; continue
    if c=='"': state='str'; i+=1; continue
    if (i==0 or src[i-1]=="\n") and src.startswith("test fn ", i):
        m=re.match(r"test fn\s+([A-Za-z0-9_]+)\s*\(", src[i:])
        if m: out.append(m.group(1))
    i+=1
for x in sorted(set(out)): print(x)
