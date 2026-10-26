import pathlib,re,subprocess,json,collections,bisect
ROOT=pathlib.Path.cwd(); OUT=pathlib.Path('/tmp/storage_effect_audit_7daaddd1f0ce')
paths=subprocess.check_output(['git','ls-files','-z']).decode().split('\0')[:-1]
head=subprocess.check_output(['git','rev-parse','HEAD']).decode().strip()
# Keep source positions; literals are separately inspected for generated shell/Rust effects.
lex=re.compile(r"//[^\n]*|/\*[\s\S]*?\*/|'(?:\\[\s\S]|[^'\\\n])'|"+r'(?P<raw>r(?P<hash>\#*)\"[\s\S]*?\"(?P=hash))|(?P<string>\"(?:\\[\s\S]|[^\"\\])*\")')
def blank(m):return ''.join('\n' if c=='\n' else ' ' for c in m.group())
filesystem=['Write','WriteBytes','WriteOwnerOnly','WriteCreateNew','WriteCreateNewWithMode','LinkCreateNew','Delete']
shell={'Mkdir':['NewOwnerOnly','Parents'],'Mktemp':['FileInDirectory','Dir','DirWithTemplate'],'Link':['Hard'],'Remove':['FileForce','RecursiveForce','EmptyDirectory'],'Move':['File','Replace','NoReplaceDirectory'],'Chmod':['OwnerOnlyDirectory','WorldWritableDirectory','OwnerReadWriteFile','RecursiveWritable','RecursiveReadOnly'],'Copy':['File','Recursive']}
wrappers=['filesystem_delete','filesystem_create_new','filesystem_link_create_new']
coreutils=['mkdir_parents_command','mkdir_create_only_command','mkdir_create_only_with_mode_command','cp_command','cp_preserve_if_absent_command','install_empty_file_command','rm_force_command','rm_recursive_force_command','rm_recursive_force_one_file_system_command','rm_recursive_force_one_file_system_exec_argv','ln_symbolic_force_command','tee_append_command','coreutils_install_directory_argv','coreutils_install_directories_arguments','coreutils_install_file_owned_argv','coreutils_install_directory_owned_arguments','coreutils_install_directory_owned_argv','coreutils_install_file_argv','dd_overwrite_path_command','coreutils_rm_force_argv','coreutils_rm_empty_directory_argv','coreutils_rm_force_recursive_argv','install_file_owned_command','mv_exact_destination_command']
patterns={
'dag_filesystem':r'\bFilesystem\.('+'|'.join(filesystem)+r')\s*\(',
'dag_shell':r'\bshell\.(?:'+'|'.join(k+r'\.(?:'+'|'.join(v)+r')' for k,v in shell.items())+r')\s*\(',
'dag_filesystem_wrapper':r'\b(?:'+'|'.join(wrappers)+r')\s*\(',
'dag_coreutils_builder':r'\b(?:'+'|'.join(coreutils)+r')\s*\(',
'rust_fs':r'\b(?:(?:std|tokio)::)?fs::(?:write|copy|rename|remove_file|remove_dir|remove_dir_all|create_dir|create_dir_all|hard_link|soft_link|symlink|set_permissions)\s*\(|\b(?:File|OpenOptions|DirBuilder)::(?:create|create_new|new)\s*\(|\.\s*(?:write_all|set_len|set_permissions|create_new|create|append|truncate)\s*\(',
'python_fs':r'\.(?:write_text|write_bytes|mkdir|rename|replace|unlink|rmdir)\s*\(|\b(?:shutil|os)\.(?:remove|unlink|rmdir|mkdir|makedirs|rename|replace|symlink|link|chmod|chown|rmtree|copy|copy2|copyfile|copytree)\s*\(|\bopen\s*\([^\n]*[\"\x27][wax][b+]*[\"\x27]',
}
compiled={k:re.compile(v) for k,v in patterns.items()}
# Candidate mutation programs in authored string tokens, including argv and generated scripts.
mutation_command=re.compile(r'(?m)(?:^|[\n;|&]\s*|\b(?:sudo|exec|then|do|command|env)\s+)(?:\s*)(?:/[-\w./]+/)?(?:mkdir|install|cp|mv|rm|rmdir|ln|chmod|chown|touch|truncate|tee|dd|mktemp)(?=\s|$)')
exact_program=re.compile(r'^(?:/(?:[-\w.]+/)+)?(?:mkdir|install|cp|mv|rm|rmdir|ln|chmod|chown|touch|truncate|tee|dd|mktemp)$')
# Redirection creates/appends state; exclude common shell stderr/stdout forwarding.
redirect=re.compile(r'(?:^|\s)(?:>|>>)(?![&=])\s*(?!/dev/(?:null|stderr|stdout)\b)(?:[\"\x27]?(?:/|\$|\.|[A-Za-z_]))')
rows=[]; alltext=[]; counts=collections.Counter()
for path_number,path in enumerate(paths):
 if path_number % 500 == 0: print(path_number, path, flush=True)
 p=ROOT/path
 if not p.is_file():continue
 try:text=p.read_text()
 except (UnicodeDecodeError,OSError):counts['nontext']+=1;continue
 counts['text']+=1; alltext.append(path)
 ext=p.suffix
 if ext not in {'.dag','.rs','.py','.sh','.yml','.yaml','.toml'} and 'Dockerfile' not in p.name and 'Makefile' not in p.name:continue
 counts['selected_source_files']+=1
 literals=[]
 for m in lex.finditer(text):
  if m.group('string') or m.group('raw'):literals.append(m)
 code=lex.sub(blank,text) if ext in {'.dag','.rs'} else text
 starts=[0]+[m.end() for m in re.finditer('\n',text)]
 fns=list(re.finditer(r'(?m)^\s*(?:(?:pub(?:\([^)]*\))?|async|unsafe)\s+)*(?:fn|data|service|def|class|operation)\s+([\w.]+)',code))
 fnpos=[m.start() for m in fns]
 mod=re.search(r'^module\s+(\S+)',text)
 module=mod.group(1) if mod else path
 hits=[]
 def add(kind,m,base=0):
  pos=m.start()+base; li=bisect.bisect_right(starts,pos); fi=bisect.bisect_right(fnpos,pos)-1
  name=fns[fi].group(1) if fi>=0 else '<module>'
  if kind in {'dag_filesystem_wrapper','dag_coreutils_builder'} and re.search(r'\bfn\s*$',code[max(0,pos-20):pos]):return
  hits.append({'kind':kind,'line':li,'symbol':name,'match':m.group().strip(),'source':text[starts[max(0,li-1)]:starts[li] if li<len(starts) else len(text)].strip()[:1000]})
 for kind,pat in compiled.items():
  if kind.startswith('dag_') and ext!='.dag':continue
  if kind=='rust_fs' and ext!='.rs':continue
  if kind=='python_fs' and ext!='.py':continue
  for m in pat.finditer(code):add(kind,m)
 if ext in {'.dag','.rs'}:
  for lit in literals:
   val=lit.group(); val=val[val.find('"')+1:val.rfind('"')]
   for mat in mutation_command.finditer(val):add('literal_shell_candidate',mat,lit.start()+lit.group().find('"')+1)
   if exact_program.fullmatch(val):
    fake=re.match('.+',val);add('literal_program_candidate',fake,lit.start()+lit.group().find('"')+1)
   for mat in redirect.finditer(val):add('literal_redirect_candidate',mat,lit.start()+lit.group().find('"')+1)
   if ext=='.dag':
    for mat in compiled['rust_fs'].finditer(val):add('dag_embedded_rust',mat,lit.start()+lit.group().find('"')+1)
 elif ext in {'.sh','.yml','.yaml','.toml','.py'} or 'Dockerfile' in p.name or 'Makefile' in p.name:
  for mat in mutation_command.finditer(text):add('script_shell_candidate',mat)
  for mat in redirect.finditer(text):add('script_redirect_candidate',mat)
 if hits:
  rows.append({'path':path,'module':module,'extension':ext,'hits':hits,'kind_counts':dict(collections.Counter(h['kind'] for h in hits))})
meta={'head':head,'tracked_paths':len(paths),'counts':dict(counts),'rows':len(rows),'effects':sum(len(r['hits']) for r in rows),'patterns':patterns,'filesystem_operations':filesystem,'shell_operations':shell,'wrapper_symbols':wrappers,'coreutils_builder_symbols':coreutils,'shell_command_selector':mutation_command.pattern,'shell_redirect_selector':redirect.pattern,'selected_extensions':['.dag','.rs','.py','.sh','.yml','.yaml','.toml','*Dockerfile*','*Makefile*'],'limits':['Lexical selector candidate census, not execution coverage','Comments and quoted strings stripped for direct DAG/Rust calls; quoted strings independently searched for shell/generated Rust','Caller/body review required for string candidates and Rust method targets; arbitrary computed commands and unmodeled native FFI need authority review','No external callers, deployed filesystem reads, builds or runtime probes']}
(OUT/'raw_inventory.json').write_text(json.dumps({'metadata':meta,'rows':rows},indent=2)+'\n')
(OUT/'selectors.json').write_text(json.dumps(meta,indent=2)+'\n')
(OUT/'roster.tsv').write_text('path\tmodule\tkinds\thits\tsymbols\n'+'\n'.join('\t'.join([r['path'],r['module'],','.join(r['kind_counts']),str(len(r['hits'])),','.join(sorted({h['symbol'] for h in r['hits']}))]) for r in rows)+'\n')
print(json.dumps({k:v for k,v in meta.items() if k in ['head','tracked_paths','counts','rows','effects']},indent=2))
print(dict(collections.Counter(r['extension'] for r in rows)))
