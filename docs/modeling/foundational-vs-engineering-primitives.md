### Foundational vs engineering primitives

The foundation says: everything is logic. String is a composition of
code points, which is a composition of bits.

The compiler says: String is a named semantic unit I can reason about.

Both are true. The compiler uses **engineering primitives** — named
types that it treats as units for reasoning (typechecking, inference,
emission). These are the semantic kernel. The foundation **justifies**
the kernel (String's definition is derivable from logic) but the
compiler doesn't expand String to bits at every use.

The practical rule: the compiler works at the semantic kernel level.
The foundation is the denotational story — it tells you what a type
MEANS, not how the compiler represents it. If someone asks "what IS
a String?", the answer is in the foundation (a composition of code
points). If the compiler needs to typecheck a String, it uses the
engineering primitive.

This is why `Primitive { name: "String" }` in the compiler IR is
acceptable as a scaffold — it's the engineering primitive. But it
should be traceable: there should be a .dag definition that shows
String's compositional structure, and the compiler should be able
to verify that the engineering primitive is consistent with the
definition. The name is a shorthand for the composition, not a
replacement for it.

