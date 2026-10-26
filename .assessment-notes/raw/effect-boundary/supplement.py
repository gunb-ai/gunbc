import pathlib,re,json,subprocess,bisect
D=pathlib.Path('/tmp/storage_effect_audit_7daaddd1f0ce'); src=(D/'scan.py').read_text();exec(src[:src.index('rows=[]')])
names=['mkdir_parents_args','mkdir_parents_argv','mkdir_parents_command_at','mkdir_exact_args','mkdir_exact_argv','mkdir_exact_command','mkdir_exact_command_at','chmod_add_executable_args','chmod_add_executable_argv','chmod_recursive_set_mode_command','chmod_set_mode_command','chown_argv','mktemp_directory_command','ln_hard_command','ln_symbolic_force_into_directory_command','rmdir_command','sort_in_place_command','cp_program','mkdir_program','install_program','ln_program','mktemp_program','tee_program','dd_program','rm_program','rmdir_program','mv_program']
ops=['openbmc.PasswordSshTransport.'+n for n in ['MakeDirectoryParents','MakeDirectory','RemoveDirectory','RemoveFile','CopyFilePreservingMetadata','RenameReplace','SynchronizeFilesystems']]+['coreutils.Truncate.ToSize']
pat=re.compile(r'\b('+'|'.join(re.escape(n) for n in names+ops)+r')\s*\(')
rows=[]
for path in subprocess.check_output(['git','ls-files','*.dag']).decode().splitlines():
 t=pathlib.Path(path).read_text();code=lex.sub(blank,t); mm=list(pat.finditer(code))
 if not mm:continue
 lines=[0]+[m.end() for m in re.finditer('\n',t)]
 defs=list(re.finditer(r'(?m)^\s*(?:fn|data|service|operation)\s+(\w+)',code));dp=[m.start() for m in defs]
 mod=re.search(r'^module\s+(\S+)',t);hits=[]
 for m in mm:
  if re.search(r'\bfn\s*$',code[max(0,m.start()-20):m.start()]):continue
  li=bisect.bisect_right(lines,m.start());fi=bisect.bisect_right(dp,m.start())-1
  hits.append({'kind':'additional_typed_fs_constructor_or_operation','line':li,'symbol':defs[fi].group(1) if fi>=0 else '<module>','match':m.group().strip(),'source':t[lines[li-1]:lines[li] if li<len(lines) else len(t)].strip()[:1000]})
 if hits:rows.append({'path':path,'module':mod.group(1) if mod else path,'extension':'.dag','hits':hits})
(D/'supplemental_inventory.json').write_text(json.dumps({'symbols':names,'operations':ops,'rows':rows},indent=2)+'\n');print(len(rows),sum(len(r['hits']) for r in rows))
