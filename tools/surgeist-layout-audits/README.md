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
or a fixed file list. R06A/T03 deleted the superseded script after the corrected,
independently reviewed lint revision audited the published R06 source with zero
diagnostics.

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
the default-`Allow` behavior. Macro-provenance coverage accepts expressions
defined by a macro in an allowed owner while retaining consumer diagnostics for
caller-supplied aggregate expressions and macro-generated type and item escapes.

## Pilot lessons

- Setup is part of the audit's semantics, not incidental bootstrap detail. The
  working stack is exactly `cargo-dylint` 6.0.3, `dylint-link` 6.0.3,
  `dylint_linting = "=6.0.3"`, `dylint_testing = "=6.0.3"`, and
  `nightly-2026-05-28` with `rustc-dev` and `llvm-tools-preview`; the installed
  Dylint driver also reports 6.0.3. Dylint library discovery requires the
  target-specific `dylint-link` configuration to produce the toolchain-suffixed
  dynamic library. Every catalog build and test shares the repository-root
  `target/dylint-audits`; letting nested Cargo use
  `tools/surgeist-layout-audits/target` breaks that single cleanup and discovery
  boundary.
- HIR and `DefId` resolution replaced spelling-based guesses. The lint resolves
  qpaths, type-dependent method identities, aliases, reexports, UFCS paths, and
  extracted method items, then checks definition-module ancestry so unrelated
  types or methods with the same names are not findings. Macro expansion identity
  needs separate treatment: an expression defined by an allowed-owner macro
  retains that definition's ownership, while caller-supplied expressions and
  macro-generated consumer items retain consumer provenance and remain
  diagnosable.
- UI fixtures should isolate one compiler behavior at a time. `allowed_owners.rs`
  proves the six owners, `semantic_escapes.rs` fixes the positive diagnostic
  inventory, `default_allow.rs` proves unselected findings compile, and the
  `ui-test-only/test_only.rs` crate is compiled with the test harness rather than
  merely parsed. `macro_provenance.rs` and its exact stderr distinguish
  owner-defined expressions from caller expressions and generated type, alias,
  and visibility escapes.
- Exact diagnostics exposed a real lint false positive before script retirement:
  the first semantic product audit reported six owner-defined macro-expression
  findings. The correction used macro-definition identity and added focused UI
  coverage while preserving caller-expression and generated-item findings. A
  reviewed lint revision is selected at most once; a lint defect returns to a
  focused correction and fresh review, while a genuine product finding returns
  to the owning product cycle.
- Maintenance remains question-scoped. The lint name, historical plan and
  revision, owners, diagnostic meanings, fixtures, and false-positive history
  travel together in review. A diagnostic is evidence that the old R06 question
  now has a different answer, not automatic proof that later architecture is
  wrong. Changes to the audit require matching UI evidence and independent review
  before another one-time selection.
- Default `Allow` means opt-in interpretation, not silent standing enforcement.
  Ordinary product Cargo commands, catalog checks, task runners, CI, and
  publication do not select the lint. A coordinator or reviewer explicitly
  selects the named question for one reviewed revision and interprets any result
  against the current architecture.
- These are leaf-local pilot lessons only. Turning them into reusable shared
  skill-reference guidance is a separate artifact and requires later explicit
  authorization; this catalog does not create or update that shared policy.
