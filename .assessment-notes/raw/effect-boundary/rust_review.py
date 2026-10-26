import json,pathlib,re,bisect
D=pathlib.Path('/tmp/storage_effect_audit_7daaddd1f0ce');src=(D/'scan.py').read_text();exec(src[:src.index('rows=[]')]);x=json.load(open(D/'classified_inventory.json'));out=[]
for r in x['rows']:
 if not r['path'].endswith('.rs') or '/tests/' in r['path'] or r['path'].startswith('test/'):continue
 t=pathlib.Path(r['path']).read_text();code=lex.sub(blank,t);starts=[0]+[m.end() for m in re.finditer('\n',t)];ranges=[]
 for m in re.finditer(r'#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]',code):
  begin=code.find('{',m.end());level=1;end=begin+1
  if begin<0:continue
  while end<len(code) and level:
   if code[end]=='{':level+=1
   elif code[end]=='}':level-=1
   end+=1
  ranges.append((m.start(),end))
 hs=[h for h in r['effect_sites'] if h['kind']=='rust_fs' and not any(a<=starts[h['line']-1]<b for a,b in ranges)]
 if hs:out.append({'path':r['path'],'hits':hs})
(D/'rust_production_candidates.json').write_text(json.dumps(out,indent=2)+'\n')
for r in out:
 print('\n'+r['path'])
 for h in r['hits']:print(h['line'],h['symbol'],h['source'][:210])
