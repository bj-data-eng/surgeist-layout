# FRI-03-C04 Block And Root Participation
Status: complete
Cycle ID: `FRI-03-C04`
Owning repository: `surgeist-layout`
Cycle base: `c44e06fc0d5fb00ea744fb2ae3ac230d240f2e9b`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `6ca195b4ba560ae49bc6963176234f8494cfb50a91674f6dcec358d19fa9769c`,
commit `52d87a75751f9987251ec2fdf8200e75eba3e17b`, sections `FRI-03.2`,
`E-BLOCK-REPLACED`, `E-COLLAPSE`, `D-04`, the block/root rows of `D-05`,
the parent-context matrix in `FRI-03.6`, relevant `FRI-03.8` and
`FRI-03.9`, and acceptance items 2, 5, and 6.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `d59317e1b80337ff4041a034c062867dc7e744048eb7047d2b2e7b412aea130a`,
commit `03e7582565fa2d4f3aa7f71973f6dfebe273c4fb`, entry `C04`.

C03 handoff: candidate `c44e06fc0d5fb00ea744fb2ae3ac230d240f2e9b`
was pushed to and read back from `origin/main`; local, tracking, and observed
remote `main` were equal and clean. Complete parent context now reaches block,
measured-leaf, viewport-root, and flex-item-root paths and cache identity.

Bounded outcome: the existing parent role gates only a block box's boundary
margin collapse; replaced block children and viewport/flex-item roots retain
measurement-driven auto-inline size; and block geometry remains independent of
item order.

## Boundary
This cycle owns role consumption in block constants and measured-leaf collapse,
replacedness consumption in ordinary block-child and root auto-inline fill,
focused real-tree tests, and read-only comparison of the four settled
`block_align_baseline_child_margin_percent` XML variants.

`BlockFlow` permits existing parent/child boundary collapse when every existing
style, edge, size, position, run-mode, and flow condition permits it. `Flex`,
`Grid`, and `NoParent` block only that current box boundary. Internal adjacent
block siblings retain their existing collapse behavior. Logical `FlowAxes` and
`PhysicalBlockMarginCollapseOf` remain the sole mapping owners.

Replacedness remains independent from table role, measurement capability,
aspect ratio, and authored/min/max sizing. Only ordinary auto-inline fill is
suppressed for replaced block children and roots. C05-C07 retain flex/grid/order
consumption. No replaced browser fixture is added because the reviewed harness
cannot preserve natural replaced dimensions.

No generator command is permitted. Parser grammar, HTML, XML, reports, corpus
metadata, provenance, and generator source remain byte-identical to the base.
`verify-generator` runs generator-feature check, test, and Clippy verification
without invoking fixture generation or browser capture; `corpus-check` invokes
only read-only `check-corpus`. The ignored aggregate parity test remains visible
and unclaimed.

Base evidence: `Constants::new` in `src/block.rs` derives all three boundary
collapse flags without `parent_formatting_context`; measured-leaf output in
`src/compute.rs` does the same; `in_flow_child_known_size` excludes table but
not replaced boxes from fill; and `root_known_inline` fills every eligible
ordinary root. The four browser variants currently expose nested-child `y=0`
where their generated expectation is `y=1`.

Impacts: API - unchanged; dependencies/features and MSRV - unchanged;
artifacts - unchanged and read-only; docs/examples - unchanged; root follow-up
- deferred to C08; unsafe - none.

## Task
### C04-T1 - Enforce Block And Root Participation
Files: `src/block.rs`, `src/compute.rs`, `src/block_tests.rs`,
`src/leaf_tests.rs`, `src/root_tests.rs`, `tests/layout/browser_parity.rs`, and
directly affected existing tests in those modules.
Dependencies: published/read-back C03 base only.

Outcome: include `input.parent_formatting_context() == BlockFlow` in block
boundary and measured-leaf collapse eligibility without changing internal
sibling collapse. Exclude `item_is_replaced` beside `item_is_table` from block
child auto-inline fill and from root auto-inline fill. Do not read item order in
production block code.

RED: the role matrix initially reports collapse for `Flex`, `Grid`, and
`NoParent`; measured leaves collapse through those roles; the four browser
variants compare nested-child `y=0` against expected `y=1`; and measured
replaced block/root boxes fill 200 instead of retaining natural width 50.

Acceptance:
- both scalar lanes prove `BlockFlow` preserves eligible start/end/through
  collapse while `Flex`, `Grid`, and `NoParent` block only the current boundary;
- root run mode remains an independent barrier; parallel and orthogonal flow
  mapping stays logical; adjacent internal block siblings still collapse;
- measured-leaf collapse uses the same role gate as block constants;
- all four settled `block_align_baseline_child_margin_percent` variants run
  nonignored, retain exact topology, and match nested-child `y=1` without XML
  changes;
- paired measured replaced/non-replaced block children in width 200 produce 50
  and 200 respectively in `f32` and `f64`; table exclusion and authored/min/max
  constraints remain independent;
- paired measured replaced/non-replaced viewport and flex-item roots produce 50
  and the ordinary fill width in both scalar lanes, with explicit flex-parent
  axes preserved; and
- the existing block item-order test remains green and no later flex/grid/order
  branch is implemented.

Exact library tests:
- `block_tests::parent_context_gates_only_block_boundary_collapse_in_both_scalar_lanes`
- `leaf_tests::parent_context_gates_measured_leaf_boundary_collapse_in_both_scalar_lanes`
- `block_tests::replaced_block_child_keeps_measured_auto_inline_size_in_both_scalar_lanes`
- `root_tests::replaced_viewport_and_flex_item_roots_keep_measured_auto_inline_size_in_both_scalar_lanes`
- `block_tests::block_layout_ignores_item_order_for_geometry`

Exact integration test:
`layout::browser_parity::block_item_boundary_margin_variants_match_browser`.

For each library name, run both commands separately:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib -- --list | rg -x 'TEST_NAME: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --lib TEST_NAME -- --exact
```
For the integration name:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout -- --list | rg -x 'layout::browser_parity::block_item_boundary_margin_variants_match_browser: test'
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::block_item_boundary_margin_variants_match_browser -- --exact
```

Task gates: `CARGO_NET_OFFLINE=true just fmt-check`;
`CARGO_NET_OFFLINE=true just verify`; `CARGO_NET_OFFLINE=true just verify-generator`;
`CARGO_NET_OFFLINE=true just corpus-check`; strict locked Clippy; rustdoc with
warnings denied; canonical unsafe scan; `git diff --check`; protected-artifact
identity against the cycle base; clean status. No generation or browser capture.

Task unsafe gate:
```sh
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
```

Commit: `layout: enforce block and root participation`.

## Completion
```sh
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
rg -n 'parent_formatting_context\(\)|item_is_replaced' src/block.rs src/compute.rs
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
git diff --check
git diff --exit-code c44e06fc0d5fb00ea744fb2ae3ac230d240f2e9b -- Cargo.toml Cargo.lock Justfile README.md scripts tests/bin/surgeist-layout-generate tests/layout/browser_parity/support.rs tests/layout/browser_parity/html tests/layout/browser_parity/xml tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/README.md tests/layout/browser_parity/scripts tests/layout/browser_parity/generation-reports
test -z "$(git status --porcelain)"
```

Cycle acceptance: the task range is independently `CLEAN`; the complete cycle
range is holistic `CLEAN`; the final commands pass on local `main`; the
immutable candidate is pushed to authority `origin/main`; a fresh fetch and
remote query prove local `HEAD`, local `main`, `origin/main`, and observed remote
`main` agree; and C05 receives the published SHA plus explicit evidence that
only C04-owned block/root behavior changed.

Genuine blockers are limited to unavailable required tooling without authorized
acquisition, unowned dirty state, contradictory reviewed requirements, or a
required unsafe/ownership violation. A failing test or review finding returns
this plan to `in_progress` and is corrected inside C04.
