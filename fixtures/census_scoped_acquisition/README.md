Planted closure for v2.test.claim.census_scoped_acquisition. Outside every corpus source root on
purpose: broken.dag does not parse, and entry.dag imports a module that does not exist.
entry -> ok (acquired), broken (blocked: does not parse), missing (blocked: no such module).
behind is reachable only through broken, so it must stay unknown; outside is imported by nothing.

diamond -> left, right -> broken: one malformed member reached twice must be one block with two requesters.

cross/: v2.cx.entry imports v2.cx.shared (permitted) and cx.fromdag (dag tree), which imports v2.cx.shared too -- denied, since the dag tree may not import v2. entry_denied_first is the same with the denied path first. Sharing the observation of v2.cx.shared must never share the permission.
