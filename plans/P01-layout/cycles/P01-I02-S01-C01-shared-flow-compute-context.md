# P01-I02-S01-C01 Shared Flow And Compute Context

Status: complete

Cycle ID: `P01/I02/S01/C01`

Owning repository: `surgeist-layout`

Cycle base: `2ddc5b56664833eb068c8300e87d04075e112b3c`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I02-logical-geometry-writing-modes.md`
at `3b04314e2d5afb3da4e10b321bea032536370f5218c430338528c2a43683e751`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`,
sections `FRI-02.1` through `FRI-02.5`; the public geometry and
containing-flow portions of `FRI-02.6` excluding its four scroll-conversion
methods and complete public scroll surface; `FRI-02.7`; non-scroll errors in
`FRI-02.12`; shared/core rows of `FRI-02.14`; and construction and mapping
evidence in `FRI-02.17`.

Reviewed sequence:
`plans/P01-layout/sequences/P01-I02-S01-logical-geometry-writing-modes.md`
at `c806dac4c55a1f83fc93fad4d5d234ceb37543337d27891b7901b87ff736e15b`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `C01`.

Bounded outcome: one canonical, scalar-generic `FlowAxes` model maps all five
writing modes and both used directions; public physical and crate-private
logical geometry are distinct; every direct, root, recursive, and hidden
compute input carries its containing flow; root/core edge and baseline behavior
uses that context without implementing the later scroll or formatting-algorithm
cycles.

## 1 Boundary

This cycle owns shared geometry and writing-mode types, used-direction semantics,
existing inline-control projection, physical diagnostic axes, compute-input
construction, cache identity, containing-inline edge bases, viewport/flex-root
setup, hidden propagation, root logical auto fill/start placement, and
flow-aware selection and synthesis of existing physical baseline points.

It may touch `src/geometry.rs`, `src/node_input.rs`, `src/output.rs`,
`src/cache.rs`, `src/compute.rs`, `src/inline.rs`, `src/grid/axis.rs`, direct
writing-mode mapping helpers, every `ComputeInputOf` construction call site,
`src/lib.rs`, and focused tests needed to compile and prove those contracts.

It does not add any scroll offset/range type or `FlowAxes` scroll conversion;
retune or restructure browser generation; add HTML/XML/report artifacts; finish
ordinary block, flex, grid, lanes, or subgrid logical algorithms; resolve
vertical clear; edit root or siblings; refresh root API artifacts; or revise the
README. Existing context-free flex main/cross helpers may remain only for the
named live `C05` consumers, and remaining algorithm-coordinate assumptions stay
owned by `C04` through `C07`. No new compatibility alias or fallback mapping is
allowed.

Current evidence: `Axis` is public; `WritingMode` has three variants; inline and
grid contain independent mapping matches; `ComputeInputOf` has crate-visible
fields and a context-free `HIDDEN` constant; root and algorithm call sites use
struct literals; cache identity omits flow; `Edges::zip_inline_size` always uses
physical width; root auto fill and location are horizontal; and baseline helpers
always select physical y.

## 2 Impacts

Public API: breaking pre-release addition of `PhysicalAxis`, `LogicalAxis`,
`PhysicalSide`, `FlowAxes`, `SidewaysRl`, and `SidewaysLr`; `Axis` is replaced in
diagnostic surfaces; direct-leaf constructors require containing `FlowAxes`.
No compatibility alias or inferred-horizontal overload remains.

Dependencies/features/artifacts: no dependency or feature change and no
generated fixture or root API artifact in this cycle. Both default and existing
generator-feature compilation remain required.

Docs/examples: rustdoc and compile-fail public-surface examples for C01-owned
types and used-direction meaning are updated; README closure remains `C08`.

MSRV/root: Rust 1.97 remains unchanged. Root later supplies used direction and
adapts the breaking geometry/input surface after the published FRI-02 candidate;
no root edit occurs now.

Unsafe: no Surgeist-owned unsafe may be added or retained.

## 3 Tasks

### 3.1 `P01/I02/S01/C01/T01` - Add The Canonical Flow-Axes Model

**Files/area:** `src/geometry.rs`, `src/node_input.rs`, `src/inline.rs`, `src/grid/axis.rs`, `src/grid/child.rs`, directly exhaustive writing-mode helpers, focused geometry/inline/grid-axis tests

**Intended behavior/outcome:** Add the private-field `FlowAxes` owner, public semantic axes/sides, crate-private logical geometry, all five `WritingMode` states, used `Direction` semantics, and canonical physical/logical projection. Existing inline-control and every grid-axis/direction mapping, including `grid_physical_axis_direction`, delegates to this owner so enum expansion creates no wildcard or second table. Scroll conversions remain absent.

**RED evidence:** Focused tests fail because sideways states, the ten-row mapping, semantic axis/side types, reversed point/rect projection, logical round trips, and sideways inline-control projection do not exist; the grid-child fallback accepts unmapped variants.

**Acceptance criteria:** Named tests cover every constructor/accessor and all ten mapping rows; size/edge/point/rect round trips and containing extents pass for `f32` and `f64`; `SidewaysLr` line-over and used-direction inversion are exact; new semantic types have no `Default`; private logical types do not enter the public surface; grid-child physical-axis direction is derived from `FlowAxes`; the scoped non-owner mapping/fallback search is empty; existing non-clearing inline controls support sideways projection without weakening `BLOCK-014`.

**Commands:** `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flow_axes -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout inline -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_axis -- --nocapture`; `rg -n --pcre2 "match\\s*(?:\\([^\\n]*(?:\\bwriting_mode\\b(?!\\.is_vertical)|\\.writing_mode(?!\\.is_vertical))[^\\n]*,|(?:[A-Za-z_][A-Za-z0-9_]*\\.)?writing_mode\\s*\\{)|_\\s*=>\\s*Direction::Ltr" src --glob '!geometry.rs' --glob '!*_tests.rs'`; `cargo fmt --check`

**Depends on:** Reviewed spec and sequence

**Intended commit:** `geometry: add canonical flow axes model`

### 3.2 `P01/I02/S01/C01/T02` - Require Containing Flow Context

**Files/area:** `src/geometry.rs`, `src/output.rs`, `src/cache.rs`, `src/compute.rs`, `src/block.rs`, `src/flex.rs`, `src/grid/**`, layout test trees and focused compute/cache tests

**Intended behavior/outcome:** Make `ComputeInputOf` fields private and replace every direct, root, flex-root, child, and hidden literal/constant with the finite reviewed constructors. Carry containing flow through every path, preserve it unchanged through hidden descendants, key caches by it, remove `Edges::zip_inline_size` from its owning geometry module, and migrate all callers to the one containing-flow edge-basis operation.

**RED evidence:** Constructor tests fail because direct inputs accept no flow and hidden is context-free; a cache characterization test hits across distinct containing flows; source searches find generic/default-alias struct literals, both aliases of `HIDDEN`, and the physical-width edge helper.

**Acceptance criteria:** Direct leaf constructors require and expose containing flow; private root/flex-root/child/hidden constructors fix all other state; all source/test call sites use them; ordinary children receive their container's resolved flow; hidden descendants retain the caller flow without cache lookup/store; cache equality separates every flow pair; every edge percentage basis uses the containing flow's logical inline extent; every qualified `ComputeInputOf`/`ComputeInput` `HIDDEN`, every owner-external literal regardless of field order, and `Edges::zip_inline_size` are absent.

**Commands:** `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout compute_input -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout cache -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout hidden -- --nocapture`; `rg -n "zip_inline_size" src tests`; `rg -n --pcre2 "\\b(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{};\\n]+>)?::HIDDEN\\b" src tests`; `rg -n -U --pcre2 "(?s)->\\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{}\\n]+>)?\\s*\\{(*SKIP)(*F)|\\b(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{}\\n]+>)?\\s*\\{" src tests --glob '!output.rs'`; `cargo fmt --check`

**Depends on:** `P01/I02/S01/C01/T01`

**Intended commit:** `layout: require containing flow context`

### 3.3 `P01/I02/S01/C01/T03` - Make Core Layout Flow-Context Aware

**Files/area:** `src/compute.rs`, `src/output.rs`, direct baseline consumers in block/flex/grid/lanes, `src/root_tests.rs`, focused leaf/baseline tests

**Intended behavior/outcome:** Use the root node's `FlowAxes` for viewport and flex-item-root inputs, logical-inline auto fill, percentage bases, and viewport start/start projection. Keep flex-item-root host location at physical zero. Make existing physical baseline selection and synthesis receive flow and use mapped block axis and line-over/line-under sides.

**RED evidence:** Vertical/sideways root tests fail because auto fill targets width and placement handles only horizontal RTL; percentage edges use width; baseline tests always choose/synthesize y.

**Acceptance criteria:** Both scalar lanes prove vertical and sideways viewport-root logical-inline fill, missing intrinsic opposite-edge behavior, percentage basis, and mapped start/start location; flex-item-root uses the same sizing/basis rules at physical-zero placement; leaf resolution consumes containing flow; baseline points remain physical while first/last selection and synthesis are correct for all ten mappings; no C01-owned valid flow panics or silently becomes horizontal.

**Commands:** `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout root_flow -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout leaf -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout baseline -- --nocapture`; `cargo fmt --check`

**Depends on:** `P01/I02/S01/C01/T02`

**Intended commit:** `layout: make core flow context aware`

### 3.4 `P01/I02/S01/C01/T04` - Expose Physical And Flow Geometry

**Files/area:** `src/geometry.rs`, `src/output.rs`, `src/compute.rs`, `src/node_input.rs`, `src/lib.rs`, rustdoc, contract/public-surface tests, C01-wide source checks

**Intended behavior/outcome:** Finish the C01 public front door: replace the old public/diagnostic `Axis` with `PhysicalAxis`, reexport only reviewed C01 types, document used direction and construction invariants, and prove no mapping/context or C02 scroll escape remains. Preserve only sequence-named later-cycle algorithm helpers and unrelated private domain-local enums.

**RED evidence:** Public contract tests fail because the old geometry `Axis` remains exported and used by diagnostics/direct tests, new types are not all reexported/documented, compile-fail/default/source assertions are absent, and no predicate guards the C02 scroll boundary.

**Acceptance criteria:** The old geometry/public-diagnostic `Axis` is absent from its owning, reexport, diagnostic, and direct contract-test surfaces without renaming unrelated private oracle concepts; public diagnostics expose `PhysicalAxis`; all C01 types, constructors, accessors, derives, defaults/non-defaults, and scalar aliases match the spec; no public API mentions crate-private logical geometry; no old input/edge helper, hidden alias, owner-external input literal, or duplicate exhaustive mapping remains; an exact-name predicate proves the four scroll-conversion methods, six new physical/flow-relative scroll value types, and `ScrollCoordinateErrorOf` are absent while existing scroll APIs remain untouched; doctests execute the compile-fail examples; `Direction` rustdoc states used inline direction; default `NodeInputOf` remains horizontal-tb LTR in both scalar lanes.

**Commands:** `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout contract -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout lib_tests -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc`; `rg -n "\\bAxis\\b" src/geometry.rs src/node_input.rs src/output.rs src/compute.rs src/lib.rs src/leaf_tests.rs src/root_tests.rs src/flex_tests.rs src/compute_tests.rs src/block_tests.rs src/contract_tests.rs src/lib_tests.rs`; `rg -n "zip_inline_size" src tests`; `rg -n --pcre2 "\\b(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{};\\n]+>)?::HIDDEN\\b" src tests`; `rg -n -U --pcre2 "(?s)->\\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{}\\n]+>)?\\s*\\{(*SKIP)(*F)|\\b(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{}\\n]+>)?\\s*\\{" src tests --glob '!output.rs'`; `rg -n --pcre2 "match\\s*(?:\\([^\\n]*(?:\\bwriting_mode\\b(?!\\.is_vertical)|\\.writing_mode(?!\\.is_vertical))[^\\n]*,|(?:[A-Za-z_][A-Za-z0-9_]*\\.)?writing_mode\\s*\\{)|_\\s*=>\\s*Direction::Ltr" src --glob '!geometry.rs' --glob '!*_tests.rs'`; `rg -n "\\b(?:physical_scroll_offset|flow_relative_scroll_offset|physical_scroll_range|flow_relative_scroll_range|PhysicalScrollOffsetOf|PhysicalScrollAxisRangeOf|PhysicalScrollRangeOf|FlowRelativeScrollOffsetOf|FlowRelativeScrollAxisRangeOf|FlowRelativeScrollRangeOf|ScrollCoordinateErrorOf)\\b" src tests`; `RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps`; `cargo fmt --check`

**Depends on:** `P01/I02/S01/C01/T03`

**Intended commit:** `api: expose physical and flow geometry`

`P01/I02/S01/C01/T01` and `P01/I02/S01/C01/T04` additionally run this authoritative multiline negative
predicate. It catches field, tuple, and method-call mappings while leaving only
later-cycle orientation queries through `writing_mode[()].is_vertical()`:

```sh
rg -n -U --pcre2 "(?s)\\bmatch\\b\\s*(?:(?!\\{).)*?(?:\\bwriting_mode\\b|\\.writing_mode)(?!\\s*(?:\\(\\))?\\.is_vertical)(?:(?!\\{).)*?\\{|_\\s*=>\\s*Direction::Ltr" src --glob '!geometry.rs' --glob '!*_tests.rs'
```

## 4 Completion

Cycle acceptance:

1. one `FlowAxes` mapping covers all ten writing-mode/direction pairs and every
   C01-owned projection in both scalar lanes;
2. physical public and logical algorithm geometry are distinct, and public
   diagnostics use `PhysicalAxis` with no `Axis` alias;
3. every compute path carries containing flow, hidden propagation is explicit,
   cache identity includes it, and context-free input/edge helpers are absent;
4. viewport/flex-root setup, direct leaf edge resolution, and existing physical
   baseline selection/synthesis obey the reviewed flow context;
5. existing non-clearing inline-control and grid-axis mapping delegate to
   `FlowAxes` without claiming later algorithm completeness; and
6. no scroll API, browser artifact, root/sibling edit, compatibility alias,
   fallback mapping, or Surgeist-owned unsafe enters the range.

Final command list:

```sh
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
rg -n "\\bAxis\\b" src/geometry.rs src/node_input.rs src/output.rs src/compute.rs src/lib.rs src/leaf_tests.rs src/root_tests.rs src/flex_tests.rs src/compute_tests.rs src/block_tests.rs src/contract_tests.rs src/lib_tests.rs
rg -n "zip_inline_size" src tests
rg -n --pcre2 "\\b(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{};\\n]+>)?::HIDDEN\\b" src tests
rg -n -U --pcre2 "(?s)->\\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{}\\n]+>)?\\s*\\{(*SKIP)(*F)|\\b(?:[A-Za-z_][A-Za-z0-9_]*::)*(?:ComputeInputOf|ComputeInput)(?:(?:::)?<[^<>{}\\n]+>)?\\s*\\{" src tests --glob '!output.rs'
rg -n -U --pcre2 "(?s)\\bmatch\\b\\s*(?:(?!\\{).)*?(?:\\bwriting_mode\\b|\\.writing_mode)(?!\\s*(?:\\(\\))?\\.is_vertical)(?:(?!\\{).)*?\\{|_\\s*=>\\s*Direction::Ltr" src --glob '!geometry.rs' --glob '!*_tests.rs'
rg -n "\\b(?:physical_scroll_offset|flow_relative_scroll_offset|physical_scroll_range|flow_relative_scroll_range|PhysicalScrollOffsetOf|PhysicalScrollAxisRangeOf|PhysicalScrollRangeOf|FlowRelativeScrollOffsetOf|FlowRelativeScrollAxisRangeOf|FlowRelativeScrollRangeOf|ScrollCoordinateErrorOf)\\b" src tests
bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files[${#files[@]}]="$file"; done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
```

The six old-surface/boundary searches must return no matches; they are listed as
observable negative predicates rather than successful matching commands.

Required handoff: `C02` receives the reviewed `FlowAxes` owner and adds all four
scroll conversions and the complete signed scroll type surface. No root handoff
is emitted before the complete FRI-02 candidate is published.

Genuine blockers: a requirement for any C02 scroll type/method, C04-C07 complete
algorithm migration, browser acquisition, root/sibling edit, or unresolved
product choice outside the reviewed C01 sections stops this cycle and returns to
planning rather than adding a workaround.
