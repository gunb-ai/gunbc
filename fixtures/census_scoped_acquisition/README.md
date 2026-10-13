Planted closure for v2.test.claim.census_scoped_acquisition. Outside every corpus source root on
purpose: broken.dag does not parse, and entry.dag imports a module that does not exist.
entry -> ok (acquired), broken (blocked: does not parse), missing (blocked: no such module).
behind is reachable only through broken, so it must stay unknown; outside is imported by nothing.
