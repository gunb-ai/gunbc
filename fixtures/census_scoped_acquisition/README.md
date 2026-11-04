# Scoped-acquisition fixtures

Exercise acquisition without widening past a blocked dependency. `entry` reaches `ok`, malformed `broken`, and nonexistent `missing`; `behind` stays unknown behind `broken`, and `outside` is unreachable. `diamond` reaches the same malformed member through two requesters, producing one block with both identities.

`cross/` exercises shared observations reached through different import edges. Admission follows `v2.std.cross_tree.import_model.cross_tree_import_admitted`: roots must be supplied and crossing namespaces declared. Tree direction alone is not a prohibition; sharing an observation must not share an edge's admission verdict. Controls: `v2.test.claim.census_scoped_acquisition`.
