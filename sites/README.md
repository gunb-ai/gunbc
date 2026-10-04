# Chassis concept site

[Altra Chassis Workbench](https://altra-chassis-workbench.briansrls448156.chatgpt.site) is the private interactive design surface for iterating on the Altra carrier and rack. Site project: `appgprj_6ac187f2fe208191b1ec7ad6703d93e8`.

The separate Site checkout is `sites/chassis-workbench/` in the printing worktree. Its source is committed to the Sites-managed repository, first published commit `44ea76b0914cf99f6e80e94fee26554544b72cfe`. Reopen that same project for edits rather than creating another site.

Concept 03 source: `7f77b6884f23af034fd1632e5918c3a21e988e5a`. The page provides 29 selectable parts/reference objects in rack view (25 on a cassette) at the default three-fan setting, isolated orbit inspection, exploded assembly, a conceptual 2×2 fixed frame, cassette withdrawal, design sliders and copyable settings. No persistent user data, credentials, printer controls or external application access are included.

This is visualization geometry, not a manufacturing model. The accepted requirements remain in `docs/plans/printed-chassis-program.md`. Dynatron W1 is the target cooler and its published active envelope is depicted; installed offset/orientation remain unresolved. Concept 03 uses one fixed-frame shared 12 V supply system per four-node block, with four protected/isolatable node feeds. PSU capacity and distribution hardware remain unselected; the former per-node 400–600 W example is superseded. Actual standoff, underside, accepted PSU interfaces and load inputs remain unresolved; proposed geometry does not establish their dimensions. Before printing structural parts, selected geometry must move through the repository's modeled CAD and admission workflow.
