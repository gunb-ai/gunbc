import re,sys,os
src,tag=sys.argv[1],sys.argv[2]
lines=open(src).read().split('\n')
out=f'wrprobe/chunks/{tag}'; os.makedirs(out,exist_ok=True)
starts=[i for i,l in enumerate(lines) if re.match(r'(pub )?(test fn|fn|type|data|service|workflow|trait|impl|const) ',l)]
paths=[]
for k,s in enumerate(starts):
    e=starts[k+1] if k+1<len(starts) else len(lines)
    body=[l for l in lines[s:e] if not l.startswith('//')]
    p=f'{out}/c{s+1:05d}.dag'
    open(p,'w').write('module chunk\n\n'+'\n'.join(body)+'\n')
    paths.append(p)
print(len(paths))
open(f'{out}/paths.txt','w').write('\n'.join(paths))
