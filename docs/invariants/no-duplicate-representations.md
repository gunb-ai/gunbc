### No duplicate representations

Every fact should be encoded in exactly one place. When two structures
represent the same information, one gets updated and the other doesn't.
The stale copy produces silently wrong behavior instead of failing.

**The test:** if changing a fact requires editing two files, one of them
is a derived copy that should be deleted or computed.

**The fix:** delete the derived representation and read from the source.
If the source isn't accessible, make it accessible — don't cache a copy
that can go stale.

**Structural prevention:** Facts are edges to definitions, not copied
strings. `kernel_types` is not `List<String> = ["Int", "Bool", ...]`
— it is `List<Node>` pointing to the actual type definition nodes. You
can't have a stale name because you don't have a name — you have a
reference. If the definition changes, the edge follows. The escape
hatch is `String`-typed fact storage; the fix is `Node`-typed
(edge-based) fact storage.

