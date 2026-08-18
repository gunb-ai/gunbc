"""Expand the (file, name) binding census into OCCURRENCE rows.

Usage:  occurrence_expander.py <pair-receipt.json> <out-receipt.json>

WHAT THIS IS: an INDEX. An upper bound on the sites a namespace rewrite would
have to consider, at occurrence grain rather than the pair grain of
binding_identity_oracle.py (which used re.search and therefore recorded PRESENCE
-- one row per (file, imported name) however many times the name occurred).

WHAT THIS IS NOT, stated because the pair census was misread as an edit manifest
by its own author: this is NOT an edit manifest and must not drive a rewrite.

Three reasons, each demonstrated rather than asserted:

 1. NO REFERENCE ROLE. A spelling can be a module reference, a generic
    parameter, a lambda parameter, a pattern binder, a local declaration, or a
    field label, in the SAME file. Receipt: replaying a qualification map derived
    this way onto src/v1/05_emit_rust.dag qualified `emit_info`, which is both a
    parameter name and a module path. The control that caught it was replaying
    the transform on the merge base and requiring byte equality with the already
    -qualified file.

 2. REGEX, NOT PARSER SPANS. String-literal exclusion here is a crude balanced
    -quote scan. Real exclusion comes from parser-provided reference spans, at
    which point prose disappears by construction instead of by heuristic.

 3. CODE-AS-DATA IS NOT PROSE. Some String rows carry .dag or generated source
    for another consumer. Those are NOT excludable as prose and are NOT
    rewritable as code; they are reported separately and left undecided, because
    source text and prose are both undifferentiated `String` in this corpus.

The oracle for an actual rewrite is unchanged and lives in
docs/plans/namespace-cut-unqualified-reference-population.md:

    pre-cut resolved declaration identity(o)
      == proposed qualified target
      == post-cut resolved declaration identity(o)
"""
import re,json,sys,collections

if len(sys.argv)!=3: sys.exit(__doc__)
PAIRS, OUT = sys.argv[1], sys.argv[2]
pairs=json.load(open(PAIRS))

STRING=re.compile(r'"(?:\\.|[^"\\])*"', re.S)
# a String row whose payload looks like .dag source rather than prose
CODEISH=re.compile(r'\b(module|fn|type|data|match|import)\s')

def spans(s): return [(m.start(),m.end()) for m in STRING.finditer(s)]

cache={}
def src(f):
    if f not in cache:
        try: cache[f]=open(f,encoding='utf-8',errors='replace').read()
        except OSError: cache[f]=None
    return cache[f]

rows=[]; tally=collections.Counter()
for p in pairs.get('rows',[]):
    s=src(p['file'])
    if s is None: tally['file_absent']+=1; continue
    sp=spans(s); name=p['name']
    lit={}
    for a,b in sp: lit[(a,b)]=s[a:b]
    for m in re.finditer(r'(?<![\w.])'+re.escape(name)+r'\b', s):
        i=m.start()
        holder=next(((a,b) for a,b in sp if a<=i<b), None)
        if holder is None:
            cls='code'
        elif CODEISH.search(lit[holder]):
            cls='code_as_data_undecided'
        else:
            cls='prose'
        tally[cls]+=1
        line=s.count('\n',0,i)+1
        col=i-(s.rfind('\n',0,i)+1)
        rows.append({'file':p['file'],'name':name,'line':line,'col':col,
                     'class':cls,'import_said':p.get('import_said'),
                     'declarers':p.get('declarers')})

out={'instrument':'occurrence_expander',
     'grain':'one row per bare occurrence (regex-derived)',
     'is_edit_manifest':False,
     'authority':'docs/plans/namespace-cut-unqualified-reference-population.md',
     'source_pair_receipt':{'totals':pairs.get('totals'),
                            'provenance':pairs.get('provenance')},
     'totals':dict(tally),
     'rows':rows}
json.dump(out,open(OUT,'w'),indent=1)
print(json.dumps(out['totals'],indent=1))
print('occurrence rows:',len(rows))
