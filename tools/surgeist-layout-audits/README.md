# Surgeist Layout Audit Catalog

This package-excluded catalog holds opt-in, planning-path-named Dylint audits.
It is not a product dependency, product test, standing command, CI gate,
publication gate, or permanent architecture policy. Its lints remain `Allow` by
default and become diagnostic only when a coordinator or reviewer explicitly
selects one.

## Node-projection boundary audit

`p01_i08_s02_r06_t02_node_projection_boundary` preserves the audit question
originating in
`plans/cycles/P01-I08-S02-R06-node-projections-and-compatible-api-map.md` at
the published R06 revision
`20ad8202e536c4c63f0bd211f0872653462116bf`. That R06 plan was governed by
specification revision
`f12c1aed35aabcb35231cca372eca6381daff57bfe2b6a053679bcf2f4d2d94f`
and sequence revision
`fa6b9ff466b2d61053ddb2961602671f64d19cdf5bb5efa2fc2a19f1a448b284`.

The historical question is: after an algorithm role is settled, does any
production descendant of `block`, `inline`, `flex`, `grid`, or `scroll` again
borrow a complete `NodeInput`/`NodeInputOf` or
`LayoutInput`/`LayoutInputOf`, or call `LayoutTree::node_input`, outside the
six projection-construction owners `crate::node_projection`, `block::input`,
`inline::input`, `flex::input`, `grid::input`, and `scroll::input`? Type aliases
and visibility reexports are escapes rather than new owners.

R06 originally applied a staged lexical audit to fixed source selections while
it moved scroll, block/inline, flex, grid-container, and then the complete grid
tree behind role-specific projections. It masked strings, comments, and exact
`cfg(test)` items and rejected aggregate or `node_input` tokens outside fixed
owner files. The semantic lint supersedes
`scripts/audit-node-projection-boundaries.sh`: it follows resolved compiler
definitions, type aliases, reexports, trait methods, UFCS, and extracted method
items across every compiled descendant instead of depending on source spelling
or a fixed file list. The script remains present only until R06A/T03 performs
its separately reviewed deletion.

After later architecture changes, a diagnostic means only that the current
source no longer answers the original R06 question the same way. Reviewers must
interpret that evidence against the architecture and plan revision under
review; an intentional new boundary may update or retire the audit. Selecting
the lint never establishes new product policy, and extending this pilot into
shared skill guidance requires separate authorization.

The UI fixtures are compiler-behavior evidence for the lint itself. They cover
the six allowed owners, both aggregate families, aliases and visibility
reexports, direct and nested descendants, direct/UFCS/extracted method uses,
ordinary strings and comments, excluded test-only code, exact diagnostics, and
the default-`Allow` behavior.
