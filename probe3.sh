set -e
cd "$(git rev-parse --show-toplevel)"
S=src/v1/stage0/src
python3 - <<'P'
def sub(p, a, b):
    s = open(p).read()
    assert s.count(a) == 1, (p, a, s.count(a))
    open(p, 'w').write(s.replace(a, b))
core = 'src/v1/stage0/src/v1_std_core.rs'
sub(core, "pub fn error_type() -> Rc<Node> {",
'''#[track_caller]
pub fn error_type() -> Rc<Node> {
    if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
        let l = std::panic::Location::caller();
        eprintln!("QE_ET {}:{}", l.file(), l.line());
    }
    error_type_inner()
}

pub fn error_type_inner() -> Rc<Node> {''')
sub(core, "pub fn field_node_type_expr(n: Rc<Node>) -> Rc<Node> {\n    match n.children.clone().first().cloned() {\n        Some(v) => v.clone(),\n        std::option::Option::None => make_expr_error_node(",
'''#[track_caller]
pub fn field_node_type_expr(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
        Some(v) => v.clone(),
        std::option::Option::None => make_expr_error_node_traced(''')
sub(core, "pub fn field_node_cardinality(n: Rc<Node>) -> Cardinality {",
'''#[track_caller]
pub fn make_expr_error_node_traced(a: Rc<NodeOccurrenceIdentity>, b: ExprErrorKind, c: String, d: Rc<SourceSpan>) -> Rc<Node> {
    if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
        let l = std::panic::Location::caller();
        eprintln!("QE_FN {}:{} {}", l.file(), l.line(), d.file.chars().take(40).collect::<String>().replace(" ", "_"));
    }
    make_expr_error_node(a, b, c, d)
}

pub fn field_node_cardinality(n: Rc<Node>) -> Cardinality {''')
ty = 'src/v1/stage0/src/v1_compiler_infer_types.rs'
sub(ty, "pub fn resolved_type(n: Rc<Node>) -> Rc<Node> {\n    match n.inferred.clone().as_deref().cloned() {\n        Some(InferredNode::Resolved { node: rt, .. }) => rt.clone(),\n        _ => error_type(),",
'''#[track_caller]
pub fn resolved_type(n: Rc<Node>) -> Rc<Node> {
    match n.inferred.clone().as_deref().cloned() {
        Some(InferredNode::Resolved { node: rt, .. }) => rt.clone(),
        other => {
            if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
                let l = std::panic::Location::caller();
                let v: String = match other {
                    None => "None".to_string(),
                    Some(InferredNode::TypeVariable { .. }) => "TypeVariable".to_string(),
                    Some(InferredNode::CompilerError { message: m, .. }) => format!("CompilerError:{}", m),
                    Some(o) => format!("{:?}", o).chars().take(24).collect::<String>(),
                };
                eprintln!("QE_RT {}:{} {}", l.file(), l.line(), v);
            }
            crate::v1_std_core::error_type_inner()
        }''')
sub(ty, "pub fn child_type_node(ch: Rc<Node>) -> Rc<Node> {", "#[track_caller]\npub fn child_type_node(ch: Rc<Node>) -> Rc<Node> {")
P
grep -n 'make_expr_error_node' $S/v1_std_core.rs | head -3
cargo build --release -p v1-compiler --bin gunbc
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/qe3.log 2>&1 || echo "rc=$?"
echo ==SIZES; grep -c '^QE_ET' /tmp/qe3.log; grep -c '^QE_RT' /tmp/qe3.log; grep -c '^QE_FN' /tmp/qe3.log
python3 - <<'P'
import re,collections,glob,os
files={os.path.basename(p):p for p in glob.glob('src/v1/stage0/src/*.rs')}
cache={}
def lines(f):
    if f not in cache: cache[f]=open(files[os.path.basename(f)]).read().split('\n')
    return cache[f]
def enc(f,ln):
    L=lines(f)
    for i in range(ln-1,-1,-1):
        m=re.match(r'\s*(?:pub(?:\([a-z]+\))? )?fn (\w+)',L[i])
        if m: return os.path.basename(f)+'::'+m.group(1)
    return '?'
hits=collections.Counter()
for line in open('/tmp/qe3.log',errors='replace'):
    if line.startswith('QE_'):
        p=line.split()
        k=p[0]; f,ln=p[1].rsplit(':',1)
        v=' '.join(p[2:]) if len(p)>2 else '-'
        hits[(k,enc(f,int(ln)),v)]+=1
print('==TABLE')
for (k,fn,v),c in sorted(hits.items(),key=lambda x:-x[1]):
    print(c,k,fn,v[:70])
# static site census (Rust mirrors): fns containing resolved_type( or error_type() or field_node_type_expr(
print('==STATIC')
sites=set()
for b,p in files.items():
    L=open(p).read().split('\n')
    cur=None
    for i,l in enumerate(L):
        m=re.match(r'\s*(?:pub(?:\([a-z]+\))? )?fn (\w+)',l)
        if m: cur=m.group(1)
        for tok in ('resolved_type(','error_type()','field_node_type_expr(','child_type_node('):
            if tok in l and 'fn '+tok[:-1] not in l and cur: sites.add((b+'::'+cur,tok))
hit_fns={fn for (_,fn,_) in hits}
for s,t in sorted(sites):
    print('HIT' if s in hit_fns else 'ZERO',s,t)

print('==ETCTX')
ctx=collections.Counter()
for line in open('/tmp/qe3.log',errors='replace'):
    if line.startswith('QE_ET'):
        f,ln=line.split()[1].rsplit(':',1)
        if enc(f,int(ln)).endswith('::infer_expr_body'): ctx[(f,int(ln))]+=1
for (f,ln),c in sorted(ctx.items(),key=lambda x:-x[1])[:25]:
    L=lines(f)
    print('---',c,os.path.basename(f),ln)
    for j in range(max(0,ln-9),ln): print('   ',L[j][:150])
P
