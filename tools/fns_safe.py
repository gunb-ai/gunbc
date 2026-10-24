import sys, re
# test fns at top level only: skip regions inside double-quoted string literals (handles \" escapes)
src = open(sys.argv[1]).read()
out=[]; i=0; n=len(src); inq=False
while i < n:
    c=src[i]
    if inq:
        if c=="\\": i+=2; continue
        if c=='"': inq=False
        i+=1; continue
    if c=='"': inq=True; i+=1; continue
    if (i==0 or src[i-1]=="\n") and src.startswith("test fn ", i):
        m=re.match(r"test fn\s+([A-Za-z0-9_]+)\s*\(", src[i:])
        if m: out.append(m.group(1))
    i+=1
for x in sorted(set(out)): print(x)
