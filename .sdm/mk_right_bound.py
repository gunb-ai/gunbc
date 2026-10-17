# Right-bound nested series: the recheck is GENUINELY REQUIRED here.
# In `gar_app(f: fn(v){ ... }, x: <expr>)` the lambda sits LEFT of the argument that
# binds its parameter type T, so on the discovery pass T is missing from the lambda's
# expected context; only the settled substitution binds it. Depth d nests d such calls
# inside the lambda body, so each level's recheck re-derives the level below it.
TPL = """module test.gar_right_{d}
import std.types {{ Int }}
fn gar_id<T>(x: T) -> T {{ x }}
fn gar_app<T, R>(f: fn(T) -> R, x: T) -> R {{ f(x) }}
fn probe() -> Int {{
{body}
}}
"""
def build(d):
    inner = "gar_id(x: v1)"
    lines = []
    pad = "  "
    for i in range(d):
        lines.append(pad + "gar_app(f: fn(v%s) { " % (i+1))
        pad = "  " + pad
    lines.append(pad + inner)
    for i in range(d):
        pad = pad[:-2]
        lines.append(pad + "}, x: 1)")
    return TPL.format(d=d, body="\n".join(lines))
for d in range(1,7):
    open(".sdm/fixtures/garr_%d.dag" % d, "w").write(build(d))
    print("wrote garr_%d.dag" % d)
