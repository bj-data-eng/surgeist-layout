# Surgeist Layout Full Code Review Findings

Date: 2026-07-10

Status: repository-wide review snapshot; findings only; no implementation fixes

## Executive Assessment

`surgeist-layout` is not yet a reliable general-purpose CSS layout engine at the
current module levels commonly meant by “CSS4.” The crate has a substantial
implementation, a clean normal unit-test baseline, strong local coverage for
several flex/grid/subgrid cases, typed scalar-generic geometry, and no `unsafe`
code. Those strengths do not offset five reliability blockers, broad
writing-mode failures, several grid and flex algorithm defects, incomplete
inline/float/scroll behavior, and major unrepresented layout features.

The most serious current risks are:

- valid layout requests can panic for negative margins, small scroll containers,
  vertical line-break clear, and calc-bearing leaf/grid-lanes paths;
- a compute-size cache hit is not semantically equivalent to an uncached result
  because all output except size is discarded;
- block, flex, grid, and grid-lanes still contain confirmed axis, placement,
  track-sizing, float-exclusion, and overflow defects;
- the layout-ready public model cannot express several layout-affecting CSS
  features, including `order`, `inline-flex`, `flow-root`, table formatting,
  fragmentation, `overflow: auto`, fixed/sticky positioning, and large parts of
  modern sizing and alignment;
- the comprehensive browser-parity test is ignored by default, is not green
  when run, compares neither scroll ranges nor line-break geometry, and the
  current corpus contains no WPT source root.

This review deliberately records defects, gaps, evidence, and impact. It does
not prescribe fixes or an implementation sequence.

## Scope And Boundary

The review stayed inside `/Users/codex/Development/surgeist-layout` and covered:

- all Rust under `src/`, including production modules, separate test modules,
  and test-support code;
- public layout-ready input, output, value, cache, compute, and traversal
  contracts;
- block, inline, float, flex, grid, subgrid, grid-lanes, absolute-position, leaf,
  scroll, rounding, and hidden-layout paths;
- crate-local unit, contract, property, oracle, and browser-parity tests;
- generator and parity support code under `tests/`;
- current crate-local plans and support matrices where they identify intentional
  deferrals or stale assumptions.

The review does not assign CSS parsing, cascade, inheritance, DOM normalization,
font selection, shaping, painting, live scroll state, or event handling to this
crate. A feature is nevertheless a `surgeist-layout` gap when its resolved form
changes geometry and the public layout-ready boundary cannot represent or
calculate it.

## Standards Baseline

There is no single monolithic “CSS Level 4” specification. The W3C CSS Snapshot
defines CSS as independently versioned modules. CSS Snapshot 2026 is a Group
Note that classifies modules by stability; it is a map of the platform rather
than a single conformance specification: <https://www.w3.org/TR/css-2026/>.

This review uses two standards tiers:

- **Foundation and current algorithm references:** CSS Level 2, latest revision
  (<https://www.w3.org/TR/CSS2/>); CSS Flexible Box Layout Level 1, currently a
  Candidate Recommendation Draft (<https://www.w3.org/TR/css-flexbox-1/>); CSS
  Grid Layout Level 2 (<https://www.w3.org/TR/css-grid-2/>); CSS Writing Modes
  Level 4 (<https://www.w3.org/TR/css-writing-modes-4/>); CSS Box Alignment Level
  3 (<https://www.w3.org/TR/css-align-3/>); and CSS Overflow Level 3
  (<https://www.w3.org/TR/css-overflow/>). CSS Multi-column Layout Level 1
  (<https://www.w3.org/TR/css-multicol-1/>), CSS Containment Level 1
  (<https://www.w3.org/TR/css-contain-1/>), CSS Shapes Level 1
  (<https://www.w3.org/TR/css-shapes-1/>), and CSS Text Level 3
  (<https://www.w3.org/TR/css-text-3/>) provide additional stable/current
  geometry context. These references support the concrete expected-behavior
  findings below; the list does not imply that each document has Recommendation
  status.
- **Evolving capability references:** CSS Display Level 4
  (<https://www.w3.org/TR/css-display-4/>), CSS Sizing Level 4
  (<https://www.w3.org/TR/css-sizing-4/>), Positioned Layout Levels 3 and 4
  (<https://www.w3.org/TR/css-position-3/> and
  <https://www.w3.org/TR/css-position-4/>), Grid Layout Level 3
  (<https://www.w3.org/TR/css-grid-3/>), Inline Layout Level 3
  (<https://www.w3.org/TR/css-inline-3/>), Overflow Level 4
  (<https://www.w3.org/TR/css-overflow-4/>), Fragmentation Level 4
  (<https://www.w3.org/TR/css-break-4/>), Multi-column Layout Level 2
  (<https://www.w3.org/TR/css-multicol-2/>), and Tables Level 3
  (<https://www.w3.org/TR/css-tables-3/>). Findings based only on these evolving
  drafts are capability gaps, not claims of uniform stable conformance.

“Reliable” in this review means that representable, valid layout-ready inputs do
not panic; cached and uncached execution are equivalent; geometry agrees with
the applicable CSS algorithm across supported writing modes and directions;
unsupported geometry-affecting features are explicit; and conformance evidence
is broad enough to justify the claimed surface.

## Method

The review used four independent passes: block/inline/float, flex/core/cache,
grid/subgrid/grid-lanes, and final cross-cutting model/test analysis. Evidence is
classified as:

- **Executed:** reproduced by actually running a focused crate test, a checked-in
  browser fixture, a disposable Rust repro, or a Chrome-for-Testing comparison.
- **Source-proven:** follows directly from an exhaustive branch or unused public
  input and does not depend on browser interpretation.
- **Capability gap:** required geometry cannot be represented by the public
  layout-ready contract.
- **Assurance gap:** tests or oracles cannot detect a relevant class of defect.

Repository inventory at review time:

Reviewed source revision: `a598e44089273be301ae18fb13e2e3c90c82259c`
(`Clamp child overflow in parent scroll geometry`). The working tree was clean
before the review began.

| Surface | Count |
| --- | ---: |
| Rust files under `src/` (production, test modules, and test support) | 49 |
| Rust lines under `src/` (same combined surface) | 74,200 |
| Rust files under `tests/` | 6 |
| Rust lines under `tests/` | 11,040 |
| Constrained browser HTML files | 1,351 |
| Checked-in browser XML files | 5,048 |
| Explicit manifest `[[cases]]` entries | 248 |

Baseline commands before focused failure execution:

```sh
cargo fmt --check
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo test -p surgeist-layout -q
cargo test -p surgeist-layout --features layout-golden-generate -q
```

The normal suite passed: 954 library tests passed with one ignored, and 85
integration tests passed with one ignored. The generator-feature test target
also passed. Focused ignored parity runs were built into an isolated
`CARGO_TARGET_DIR` after a disposable diagnostic checkout contaminated shared
build artifacts; only isolated/current-source results are used below.

## Severity

| Severity | Meaning in this review |
| --- | --- |
| Blocker | Valid input can abort, or core infrastructure can return a semantically corrupted layout result. |
| High | Broad deterministic mislayout, loss of required geometry, or an entire required layout capability is absent. |
| Medium | Narrower but real correctness defect, incomplete semantic branch, contract ambiguity, or material oracle blind spot. |
| Low | Public API inconsistency or validation weakness with less immediate layout reach. |

Known/deferred means an existing plan acknowledges all or part of the gap. It
does not reduce the observed impact.

| Recorded severity | Findings |
| --- | ---: |
| Blocker | 5 |
| High | 21 |
| Medium | 30 |
| Low | 3 |
| **Total** | **59** |

## Reliability Blockers

### CORE-001 — Compute-size cache hits erase required output state

**Severity:** Blocker
**Evidence:** Executed and source-proven
**Status:** Newly verified

`CacheOf::store_with_context` stores only `output.size` for `ComputeSize` at
`src/cache.rs:118-124`. A hit reconstructs
`ComputeOutputOf::from_outer_size` at `src/cache.rs:91-95`. This discards
`content_size`, `scroll_geometry`, first and last baselines, collapsible top and
bottom margins, and `margins_can_collapse_through`, even though those are part
of `ComputeOutputOf` at `src/output.rs:149-159` and parent block layout consumes
them.

A direct cold/hit repro returned the same outer size but changed content size
from `30x40` to zero, baseline `7` to none, margins `12/-4` to zero, and
`margins_can_collapse_through` from true to false. A cache hit is therefore not
semantically equivalent to the computation it replaces. Existing cache tests
construct size-only outputs, so they cannot observe the loss.

### CORE-002 — Public calc-bearing paths lack usable resolver composition

**Severity:** Blocker
**Evidence:** Executed and source-proven
**Status:** Known restriction, still active

The public `compute_leaf` entry point hard-codes `NoCalcResolver` at
`src/compute.rs:232-241`; the resolver-aware path at `src/compute.rs:243-248`
is crate-private. Missing resolver state panics at `src/compute.rs:430-450`.
The parity tree implements `CalcResolver` at
`tests/layout/browser_parity/support.rs:888-904` but must call public
`compute_leaf` at `tests/layout/browser_parity/support.rs:551-573`.

All three active calc families encounter this path:

- `block_calc_width_margin`;
- `flex_calc_basis_margin_gap`;
- `grid_calc_track_and_item_margin`.

Each aborts at `src/compute.rs:448` with `calc resolution requires an explicit
resolver`. A unit test at `src/compute_tests.rs:35-54` intentionally locks the
panic, but the public contract remains unusable for a calc-bearing measured
leaf and prevents the checked-in browser fixtures from reaching comparison.

The same resolver-composition failure exists in public grid-lanes APIs.
`resolve_tolerance` calls resolver-free `LengthOf::resolve` at
`src/grid/lanes.rs:1714-1719`; `LengthOf::Calc` panics at
`src/value.rs:467-473`. The tree-backed lane path calls that function at
`src/grid/lanes.rs:803-809`, and a direct public `place_lanes` repro with a calc
tolerance aborted there. Public `lane_intrinsic_sizing` also installs
`NoCalcResolver` at `src/grid/lanes.rs:400-404`, causing calc track minima to
degrade to zero rather than use the represented expression.

### BLOCK-001 — Valid negative margins panic during overflow accumulation

**Severity:** Blocker
**Evidence:** Executed
**Status:** Newly verified

`ScrollableOverflowAccumulator::include_child` forms a margin rectangle with
`size + margin.sum_axes()` and asserts that the resulting size is non-negative
at `src/block.rs:2074-2086`. Valid negative CSS margins can make that synthetic
rectangle negative even when the layout itself is well-defined.

`block_margin_y_simple_negative__border_box_ltr.xml` expects a valid
zero-height root and a child at `y=-10`; the engine instead panics with
`InvalidScrollRect`. The same failure occurs in negative sibling-collapse,
collapse-through, complex collapse, total-collapse, and intrinsic-width
families. The failure is in overflow accumulation, not in the margin-collapse
calculation.

### BLOCK-002 — A box smaller than its scrollbar reservation panics

**Severity:** Blocker
**Evidence:** Executed
**Status:** Newly verified

Block constants subtract the full scrollbar-adjusted content-box inset without
clamping at `src/block.rs:2588-2594`. The negative inner size then reaches
`ScrollableOverflowAccumulator::new`, whose `ScrollRect` construction is
asserted at `src/block.rs:2053-2057`.

`block_overflow_scrollbars_overridden_by_size__border_box_ltr.xml` uses a
`2x4` scroll box with a `15px` reservation. Browser geometry clamps the content
box to `0x0`; the engine panics with `InvalidScrollRect`. The corresponding
available-space family also panics.

### BLOCK-014 — Vertical line-break clear is a public panic path

**Severity:** Blocker
**Evidence:** Source-proven
**Status:** Known unimplemented branch

`LineBreakInputOf` publicly permits vertical writing modes and every `Clear`
value at `src/node_input.rs:270-346`. When such a visible control has any clear
value other than `None`, block layout unconditionally panics at
`src/block.rs:441-443` with `vertical line-break clear layout is not
implemented`.

This is a valid, representable layout-ready combination rather than an invalid
constructor state. It therefore meets this review's blocker definition even
though current constrained parity excludes the broader vertical-clear surface.

## High-Severity Correctness Findings

### BLOCK-003 — Ordinary block flow ignores vertical writing-mode block axes

**Severity:** High
**Evidence:** Executed and source-proven
**Status:** Not covered as a complete block-flow capability

`layout_in_flow_children` always advances a physical `cursor_y` beginning at
`src/block.rs:531-545`; child placement and advancement remain physical-y at
`src/block.rs:881-934`. `writing_mode` is retained in constants but does not
select the block axis.

A `100x100` `vertical-rl` block with two `20x10` children should place them at
`(80,0)` and `(60,0)`. The engine places them at `(0,0)` and `(0,10)`. This is a
formatting-context-level failure, not a narrow line-break issue.

### BLOCK-004 — Ordinary inline lines overlap active floats

**Severity:** High
**Evidence:** Executed and test-codified
**Status:** Partly known/deferred

`layout_inline_run_with_clear` bypasses the float-exclusion model whenever the
run contains no clearing break at `src/block.rs:1206-1220`. Ordinary line boxes
therefore do not obtain the remaining inline band beside an active float.

The existing unit scenario at `src/block_tests.rs:2508` places a `20px` inline
box at `x=0` over an active `80x50` left float. The usable line band begins at
`x=80`. Current coverage characterizes the wrong overlap rather than browser
behavior.

### FLEX-001 — Flex row/column axes are physical and ignore writing mode

**Severity:** High
**Evidence:** Executed and source-proven
**Status:** Partly known/deferred

Flex constants retain `flex_direction` and text direction but not writing mode
at `src/flex.rs:81-102,182-205`. `FlexDirection` maps row to horizontal and
column to vertical at `src/node_input.rs:580-594`, and flex layout repeats those
physical assumptions throughout placement and margin access.

For `writing-mode: vertical-lr; flex-direction: row` in a `100x100` container,
two `10x20` items are placed by the engine at `(0,0)` and `(10,0)`. Chrome 149
places them at `(0,0)` and `(0,20)` because row follows the vertical inline
axis. Existing vertical-writing flex XML does not expose this: those flex nodes
are represented as measured text leaves rather than non-leaf flex containers.

### GRID-001 — Implicit-track demand is guessed before placement

**Severity:** High
**Evidence:** Executed and source-proven
**Status:** Newly verified

Grid pre-sizing reduces items to a total cell count at `src/grid/mod.rs:843-850`
and derives row demand with `div_ceil` at `src/grid/mod.rs:941-955`. When actual
packing needs additional tracks, `src/grid/placement.rs:525-535` returns a
zero-size out-of-range area rather than reflecting that placement demand.

Two focused repros demonstrate both directions of error:

- with three columns, one explicit occupied middle cell, and a later auto item
  spanning two columns, the browser needs a second row (`120x40`, item at
  `(0,20)` sized `80x20`); the engine stays `120x20` and sends the item through
  the zero-area path;
- two explicitly overlapping children in one explicit `10x10` cell cause an
  unnecessary implicit `20px` row, producing `10x30` instead of `10x10`.

### GRID-002 — Rows-only grid-lanes loses the definite inline containing block

**Severity:** High
**Evidence:** Executed
**Status:** Newly verified

When the grid axis is `Row`, lane measurement leaves `parent.width` indefinite
at `src/grid/lanes.rs:1569-1624`. A percentage child width consequently
measures as zero; that value becomes its lane-axis margin box and final area at
`src/grid/lanes.rs:1306-1340`.

The active `grid_lanes_item_containing_block_content_width` family has exactly
four failures across the 56 grid-lanes XML files. LTR variants expect width
`100` and receive `0`; RTL variants additionally move from expected `x=0` to
`x=100` because the zero-width item is end-anchored.

### GRID-003 — One column fit-content track disables all flexible expansion

**Severity:** High
**Evidence:** Executed and source-proven
**Status:** Newly verified

The column resolver returns early if any max track sizing function is
`FitContent` at `src/grid/tracks.rs:1983-1993`. The alternate pass at
`src/grid/tracks.rs:2269-2294` resolves non-fit tracks only to their base size
and does not perform flexible-track expansion.

In a definite `200px` inline axis with tracks `[fit-content(50px), 1fr]` and
intrinsic bases `[20,0]`, the expected result is `[20,180]`; the engine returns
`[20,0]`. The row resolver does not use this early-return path.

### GRID-004 — Intrinsic vertical-writing grid dimensions remain unswapped

**Severity:** High
**Evidence:** Source-proven current behavior; standards-derived expectation
**Status:** Partly known/deferred

Column and row expansion bases are fixed to physical width and height at
`src/grid/mod.rs:781-814`, and intrinsic track totals retain that ordering at
`src/grid/mod.rs:302-357`.

`src/grid_tests.rs:3026-3067` codifies `70x110` for a `vertical-rl` grid with
logical columns totaling `70` and rows totaling `110`. Physical output should
be `110x70`: the inline-axis columns map to physical height and block-axis rows
to physical width. Related vertical/column-axis baseline application remains
explicitly horizontal-only at `src/grid/child.rs:88-90`.

### GRID-011 — `overflow: hidden` does not zero grid automatic minimums

**Severity:** High
**Evidence:** Source-proven current behavior; standards-derived expectation
**Status:** Newly verified

`Overflow::is_scrollable` returns true only for `Scroll` at
`src/node_input.rs:105-107`. Grid's automatic-minimum predicates rely directly
on that method at `src/grid/tracks.rs:1333-1342`, with uses throughout intrinsic
track contribution and sizing; grid-lanes reuses the same predicates at
`src/grid/lanes.rs:1104-1105`.

For the CSS Grid automatic-minimum condition, `hidden` is a scrollable overflow
value and the automatic minimum in the relevant axis is zero. The current
branch retains a min-content automatic minimum instead, which can broadly
oversize hidden-overflow grid and grid-lanes tracks. Flex already treats both
`Hidden` and `Scroll` as zero-auto-minimum cases at `src/flex.rs:716-719`, and
the scroll model groups both as scrollable clips at `src/scroll.rs:158-165`.

### OVERFLOW-001 — Flex/grid layout discards nested scroll geometry

**Severity:** High
**Evidence:** Executed and source-proven
**Status:** Known/deferred to scroll Phase 4

Flex writes `scroll_geometry: None` for in-flow and absolute children at
`src/flex.rs:2485-2497,2718-2730`, and its container output defaults to none.
Grid follows the same pattern in `src/grid/mod.rs`, `src/grid/child.rs:519-532,
2020-2033`, and `src/grid/lanes.rs:1514-1527`. Block layout, by contrast,
preserves or synthesizes child geometry at `src/block.rs:910-930`.

A direct repro returning `Some(scroll_geometry)` from a child produced
`NodeOutput.scroll_geometry=None` under both a flex and a grid parent. The same
node can expose a scrollport/range as a root or block child and lose it solely
because of its parent formatting context. Nested runtime/render traversal
cannot recover those facts from the output tree.

### OVERFLOW-002 — Required overflow and scrolling semantics are unrepresentable

**Severity:** High
**Evidence:** Capability gap and source-proven
**Status:** Known/deferred

`Overflow` contains only `Visible`, `Clip`, `Hidden`, and `Scroll` at
`src/node_input.rs:89-112`; `overflow: auto` is absent. `src/scroll.rs:3-31`
also explicitly records unsupported clip margin, stable and both-edge gutters,
scroll padding, scroll margin, snap, and layout-owned mixed-axis coupling.
Ancestor/nested clipping and complete out-of-flow descendant contribution are
not emitted.

The parity adapter aliases `"auto"` to `Overflow::Scroll` at
`tests/layout/browser_parity/support.rs:1515-1521`, changing conditional
scrollbar behavior into always-reserved scroll gutters. This surface cannot
reliably calculate current CSS Overflow behavior even when upstream has fully
resolved authored syntax.

### MODEL-001 — CSS order-modified document order cannot be represented

**Severity:** High
**Evidence:** Capability gap and source-proven
**Status:** Unmodeled

`NodeInputOf` has no `order` field at `src/node_input.rs:817-863`. Flex assigns
order from child enumeration at `src/flex.rs:326-345`; grid placement iterates
the stored child vector at `src/grid/placement.rs:651-675`; grid-lanes similarly
uses enumeration. `NodeOutput.order` only reports that source index.

Flex layout and grid auto-placement therefore cannot operate in
order-modified document order. This is not a parser omission: the resolved
integer has no layout-ready input slot.

### FLOW-001 — Inline layout has no measured-text participant contract in use

**Severity:** High
**Evidence:** Capability gap and source-proven
**Status:** Known/deferred cross-crate work

The only inline participants are atomic boxes, forced line breaks, and inline
boundaries at `src/inline.rs:308-314`. The crate relies on synthetic measured
text leaves in the parity adapter and cannot consume shaped runs, break
opportunities, bidi fragments, whitespace effects, or per-run metrics as
participants in mixed inline formatting.

The generation report excludes 100 mixed text/element cases, while the current
contract plan explicitly leaves measured text integration open. Because line
construction, wrapping, baselines, float interaction, and fragmentation depend
on those layout-ready facts, this is a core geometry capability gap even though
font selection and shaping remain outside the crate.

### FLOW-002 — Vertical inline layout is only a forced-column subset

**Severity:** High
**Evidence:** Source-proven and capability gap
**Status:** Partly known/deferred

`layout_vertical_inline_run` appends boxes without applying available inline
extent and starts a new column only at a forced break at
`src/inline.rs:669-703`. Baseline and atomic-box metrics are largely modeled on
horizontal dimensions; the separate representable vertical-clear panic is
recorded as BLOCK-014.

Vertical writing modes therefore lack ordinary soft wrapping, complete clear
interaction, and robust baseline mapping. The generation report still excludes
144 vertical `<br>` cases despite a small constrained vertical-break subset.
The public `WritingMode` enum at `src/node_input.rs:162-175` also lacks the
Level 4 sideways writing modes.

### FLOW-004 — Line clamping and block ellipsis are unrepresentable

**Severity:** High
**Evidence:** Evolving-draft capability gap
**Status:** Unmodeled

Neither `NodeInputOf` at `src/node_input.rs:817-863` nor the participant and
output contracts represent the line-count, continuation, or ellipsis facts
needed for CSS Overflow Level 4 `max-lines`, `block-ellipsis`, and the
`line-clamp` shorthand. These states change line construction, fragmentation,
and used block size, so they fall inside layout geometry even though authored
shorthand parsing and glyph shaping remain outside this crate.

### FLOW-003 — Float and block-formatting-context behavior is incomplete

**Severity:** High
**Evidence:** Executed, source-proven, and capability gap
**Status:** Partly known/deferred

Beyond the confirmed inline-overlap defect in BLOCK-004, current float behavior
does not fully cover float-only BFC auto height, the complete set of
BFC-establishing displays, auto-width BFC sizing inside the available float
band, mixed inline exclusion, logical float/clear directions in vertical
writing, or non-rectangular float shapes. The generated float XML suite contains
only four files.

The richer `xfloat_*` HTML sources are excluded from active parity. Basic
`float_simple` passes, but that is not evidence for CSS float/BFC reliability.

### MODEL-002 — Major display and formatting-context roles are absent

**Severity:** High
**Evidence:** Capability gap
**Status:** Unmodeled or partial

`Display` at `src/node_input.rs:7-17` contains block, flex, grid, grid-lanes,
three atomic inline variants, and none. It cannot express `inline-flex`,
`flow-root`, list-item/marker geometry, table wrapper/internal boxes, or ruby
layout roles. `item_is_table` is only a special-case flag; there is no table
formatting algorithm. The parity parser exposes the same narrow set at
`tests/layout/browser_parity/support.rs:1458-1468`.

These layout-owned roles establish different formatting contexts, intrinsic
contributions, baselines, and fragmentation behavior. The existing
`InlineBoundaryInputOf` represents non-atomic inline boundaries, while the
participant/fragment limitations are recorded separately in FLOW-001. DOM box
tree normalization and `display: contents` remain upstream responsibilities.
Layout- and size-containment states that change formatting-context and
intrinsic-size behavior are also absent.

### MODEL-003 — Modern positioning modes are absent or conflated

**Severity:** High
**Evidence:** Capability gap
**Status:** Unmodeled

`Position` has only `Relative` and `Absolute`, with `Relative` as the default,
at `src/node_input.rs:115-120`. Static and relative positioning cannot be
distinguished, and fixed and sticky geometry cannot be represented. Positioned
Layout Level 4 features such as anchor positioning likewise have no
layout-ready inputs.

This leaves containing-block selection, viewport attachment, scroll-dependent
sticky constraints, static-position rules, and anchor-derived inset/size
geometry outside the public calculation surface.

### MODEL-004 — Fragmentation and multi-column layout are absent

**Severity:** High
**Evidence:** Capability gap
**Status:** Unmodeled

`LayoutInputOf` has only `Box`, `LineBreak`, and `InlineBoundary` at
`src/node_input.rs:973-978`; `NodeOutputOf` contains one location and one size at
`src/output.rs:223-233`. There is no fragmentainer input, column/page geometry,
break-before/after/inside, widows/orphans, box-decoration-break state, or
multi-fragment output.

This prevents CSS Fragmentation, multi-column layout, and fragmented block,
flex, grid, table, and inline behavior from being calculated within the crate.

### MODEL-005 — The sizing/value surface cannot express current CSS sizing

**Severity:** High
**Evidence:** Capability gap and source-proven
**Status:** Partial

`DimensionOf` contains px, percentage, calc ID, fr, auto, min-content, and
max-content at `src/value.rs:661-670`. Size, min-size, max-size, and flex-basis
therefore cannot represent the full property-appropriate surface, including
`fit-content()` sizing, `stretch`, `contain`, `flex-basis: content`, or other
modern intrinsic/containment sizing states.

`CalcExpressionOf` is only a flat sum of px and percentage terms at
`src/value.rs:332-388`; unresolved layout-dependent `min()`, `max()`, `clamp()`,
and newer sizing functions cannot be preserved through the layout-owned basis
resolution path. This is a layout-ready typed-value gap, not a request for this
crate to parse CSS strings.

### GRID-010 — Grid Level 2/3 support retains explicit unsupported branches

**Severity:** High
**Evidence:** Source-proven and capability gap
**Status:** Known/deferred

The grid subsystem explicitly rejects standalone subgrid intrinsic traversal at
`src/grid/subgrid.rs:267-269` and nested indefinite grid-lanes subgrid sizing at
`src/grid/mod.rs:1474-1477`. Baseline application in vertical/column axes is
documented as horizontal-only at `src/grid/child.rs:88-90`.

The 508 checked-in subgrid XML files pass, but they cover the accepted subset.
These branches prevent a broad Grid Level 2 claim; the 56 grid-lanes fixtures
are also too narrow to qualify the evolving Grid Level 3 surface.

### TEST-001 — There is no release-grade conformance gate

**Severity:** High
**Evidence:** Assurance gap and executed
**Status:** Current test architecture

The comprehensive checked-in browser-parity test is `#[ignore]` at
`tests/layout/browser_parity.rs:461-503`. When run, the current corpus is not
green: active calc fixtures panic, active block fixtures expose additional
panics/mismatches, and four grid-lanes variants mismatch. Normal `cargo test`
therefore reports green without executing the main browser comparison.

The current `corpus.toml` declares only Taffy and Surgeist source roots; it has
no WPT root. The most recent retained WPT snapshot at
`plans/2026-06-20-surgeist-wpt-parity-bucket-findings.md`, dated 2026-06-20,
recorded 2,176 passes and 9,753 failures out of 11,929 fixtures (about 18.24%),
including large grid alignment, flex multiline alignment, abspos alignment,
and grid minimum-size buckets. Those historical numbers are not a current pass
rate, but removing the WPT corpus means the known failure classes are no longer
an executable regression gate.

## Medium-Severity Correctness And Contract Findings

### CORE-003 — Calc IDs and cache generations have no store identity

**Severity:** Medium
**Evidence:** Source-proven contract gap
**Status:** Residual from earlier calc-model work

`CalcId` is only a `u32` store index at `src/value.rs:106-123`. Two stores can
therefore produce the same ID for different expressions, and an ID created from
one store resolves silently against the expression at the same index in another
store. `LayoutCalcStore::calc_generation` is only the store length at
`src/value.rs:311-329`; `CacheKeyContext` carries only that value at
`src/cache.rs:8-27`.

Replacing a resolver/store with a different store of the same length leaves the
cache context unchanged and can reuse output calculated from different
expressions. The public types do not encode resolver/store provenance or
expression identity.

### CORE-004 — Leaf measurement receives negative definite availability

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Newly verified

Leaf layout subtracts padding, border, and gutter insets from definite
availability without flooring at zero at `src/compute.rs:336-360`. A
border-box width of `10` with `10px` left and right padding passes
`Available::Definite(-10)` to the measurement callback even though final outer
width is floored to `20`.

Negative content-box availability violates the layout-ready measurement
contract and can make text/intrinsic measurement diverge before the final box
size is clamped. Flex constants already avoid the equivalent negative inner
size at `src/flex.rs:168-177`.

### CORE-005 — Replaced-element sizing input is dead

**Severity:** Medium
**Evidence:** Source-proven and executed in grid repro
**Status:** Unimplemented public input

`item_is_replaced` is public at `src/node_input.rs:817-822` but is never read by
production code. A replaced in-flow block with a `50px` intrinsic width in a
`200px` containing block follows ordinary block auto-width behavior and is
stretched to `200px`. Grid normal alignment similarly selects stretch without
consulting replaced status at `src/grid/child.rs:1597-1639`; a natural `10x10`
replaced item in a `100x20` grid area receives a perform-layout known size of
`100x20`.

The input advertises semantics that the algorithms do not implement.

### BLOCK-006 — The line after a forced break loses the containing strut

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Known pending inline-boundary integration

A forced break commits metrics to the preceding line and resets to an empty
line at `src/inline.rs:553-580`. The parity adapter does not create containing
inline boundary/strut participants for the next empty line.

All four `block_br_inline_block_metrics` variants expect two `30px` lines and a
`60px` total height; the engine returns `40px`. This is the visible consequence
of the open root-boundary metadata contract.

### BLOCK-007 — Blocks cannot know they are flex/grid items during margin collapse

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Unmodeled formatting-context role

`ComputeInputOf` at `src/output.rs:37-45` carries no parent formatting-context
or item role. Block collapse eligibility examines only the child's own style at
`src/block.rs:2574-2629`, so a block that is a flex or grid item can collapse
margins as if it participated in an ordinary block formatting context.

All four `block_align_baseline_child_margin_percent` parity variants expect the
nested child at `y=1`; the engine reports `y=0` after the illegal collapse.

### BLOCK-008 — Multiline alignment uses the widest line for every line

**Severity:** Medium
**Evidence:** Source-proven
**Status:** Uncovered

Inline layout computes one `report_inline_extent` from the maximum line width
and passes it to every physical placement at `src/inline.rs:587-642`. Block
layout then computes a single run offset at `src/block.rs:1387`.

Shorter wrapped lines therefore cannot receive their own right/center offset;
the same shared extent incorrectly shifts shorter RTL legacy-left lines. The
current fixture surface does not compare unequal multiline alignment.

### BLOCK-009 — `vertical-align: top` is treated as a zero baseline

**Severity:** Medium
**Evidence:** Source-proven
**Status:** Narrow vertical-align model

Block layout maps a top-aligned atomic box to `first_baseline: 0` at
`src/block.rs:1363-1367`. Inline baseline placement then moves it down to the
line baseline at `src/inline.rs:598-607`. A `5px` top-aligned box beside a
`10px` baseline box lands at `y=10` instead of the line top.

`InlineControlAlignment` is carried but not used to provide independent
top/bottom line-box alignment semantics.

### BLOCK-010 — Inline-sequence abspos boxes lose their hypothetical position

**Severity:** Medium
**Evidence:** Source-proven and test-codified
**Status:** Uncovered by browser parity

An absolutely positioned child encountered in an inline run is recorded at the
run's physical start at `src/block.rs:1317-1319`, ignoring preceding inline
advance and wrapping. `src/block_tests.rs:2903` locks an abspos child following
a `10px` inline box at `(0,0)` while the next in-flow child begins at `(10,0)`.

The broad block abspos parity families pass; this finding is specifically the
hypothetical static position of an out-of-flow box embedded in inline content.

### BLOCK-011 — Inline-block baseline fallback and overflow rules are incomplete

**Severity:** Medium
**Evidence:** Source-proven
**Status:** Partially covered

Atomic inline baseline synthesis at `src/inline.rs:323-336` falls back to the
border-box bottom rather than the bottom margin edge. Inner baselines are also
used without distinguishing the inline-block overflow condition that selects a
fallback baseline.

Current tests cover a marginless fallback and visible-overflow inner baseline,
but not a nonzero bottom margin or non-visible overflow.

### BLOCK-012 — Percentage height on atomic inline children lacks a definite basis

**Severity:** Medium
**Evidence:** Source-proven
**Status:** Uncovered

Inline child computation always passes `parent.height = None` at
`src/block.rs:1331-1345`, even when the containing block's inner height is
definite. A percentage height on an atomic inline child therefore cannot
resolve against a known containing-block height.

### BLOCK-013 — Fixed-size compute can discard control-established baselines

**Severity:** Medium
**Evidence:** Source-proven
**Status:** Uncovered

The fixed-size `ComputeSize` early return at `src/block.rs:53-60` depends on
`normal_flow_children_can_establish_baseline`. That predicate returns false for
all `LineBreak` and `InlineBoundary` inputs at `src/block.rs:162-175` even
though controls can carry line metrics.

A fixed-size block containing only metric-bearing controls can consequently
return no baseline without constructing its lines.

### BLOCK-005 — Browser adapter constructs a line break with invalid flow metadata

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Adapter/contract diagnostic gap

Visible controls are intentionally required to match their containing inline
flow at `src/block.rs:444-445`. The crate-owned browser adapter instead copies
the `<br>` node's own resolved direction into `LineBreakInputOf` at
`tests/layout/browser_parity/support.rs:1019-1036`, without containing-flow
context.

`block_direction_rtl_with_br__border_box_ltr.xml` therefore aborts with
`line-break flow must match containing inline flow` before it can compare the
browser's `100x20` result. This proves a crate-local adapter and contract-
diagnostic failure, not that correctly normalized same-flow layout input is
miscomputed; the public model can represent the matching flow.

### FLEX-002 — Cross-axis auto margins become negative under overflow

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Uncovered

`resolve_cross_axis_auto_margins` at `src/flex.rs:1397-1432` divides negative
free space between two auto margins and assigns negative free space directly to
a sole start-side auto margin.

For a `100x40` row flex container with a `20x60` item and top/bottom auto
margins, the engine places the item at `y=-10` with margins `-10/-10`. Chrome
uses `0/0` and lets overflow proceed toward cross-end. Existing unit tests cover
only positive free space.

### FLEX-003 — Abspos flex auto margins use the wrong available equation

**Severity:** Medium
**Evidence:** Executed and test-codified
**Status:** Current tests expect the wrong result

`resolve_absolute_margins` at `src/flex.rs:2764-2798` ignores insets and bases
free space on `node_inner_size`, while abspos sizing uses padding-box-relative
`inset_relative_size` at `src/flex.rs:2591-2597,2662-2688`.

With a `100x40` flex container, a `20px` absolute child, `left:0`, `right:auto`,
and both horizontal margins auto, the engine centers at `x=40` with `40/40`
margins. Chrome places it at `x=0` with used margins `0/0`. The expectation at
`src/flex_tests.rs:2511-2603` characterizes the engine result.

### FLEX-004 — Intrinsic flex-basis keywords collapse to max-content behavior

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Uncovered

Both `MinContent` and `MaxContent` resolve as non-numeric at
`src/value.rs:749-763`. Flex sends the unresolved fallback through a
max-content measurement at `src/flex.rs:397-412,525-548`.

A direct measurement where min-content is `20` and max-content is `100`
produced `100` for both bases. A Chrome text comparison produced distinct
intrinsic widths. No parity fixture exercises these intrinsic keywords; the
separate public-value gap is recorded in MODEL-005.

### FLEX-005 — Collapsed flex-item struts are unrepresentable

**Severity:** Medium
**Evidence:** Capability gap
**Status:** Unmodeled

There is no visibility/collapse input in `NodeInputOf`. Flexbox's
`visibility: collapse` layout behavior requires a cross-size strut even though
painting/visibility ownership may remain outside layout. The crate cannot
represent that resolved geometry state.

### GRID-005 — Grid-template-areas does not create the corresponding explicit tracks

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Missed planned requirement

Explicit track counts are fixed from expanded track lists at
`src/grid/mod.rs:816-817`. Template-area facts expand named-line maps at
`src/grid/named.rs:973-1038` but do not expand the actual explicit grid.

With one area row `"foo foo foo"`, no explicit track list,
`grid-auto-columns:40px`, `grid-auto-rows:20px`, and a fixed `10x10` item in
`foo`, browser semantics treat the area-created tracks as explicit auto tracks
and yield intrinsic `10x10`. The engine treats them as implicit auto-track
sizes and returns `120x20`.

### GRID-006 — Auto-fit collapses tracks from child count before placement

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Newly verified

Grid passes `visible_child_count` before placement at
`src/grid/mod.rs:786-807`; `src/grid/tracks.rs:2406-2409` truncates auto-fit
repetition to that number. Empty repeated tracks are therefore inferred from
item count rather than occupancy after placement.

In a `120px` container with `repeat(auto-fit,40px)` and two children explicitly
overlapping the first track, only one occupied track remains in browser layout,
centered at `x=40`. The engine retains two repetitions and centers them at
`x=20`.

### GRID-007 — Content-distribution stretch excludes minmax(..., auto)

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Newly verified

Both track resolvers count and stretch only tracks whose minimum and maximum
are both exactly `Auto` at `src/grid/tracks.rs:1777-1825,2104-2155`.
Tracks with an auto max sizing function but another minimum are excluded.

A single `minmax(0px,auto)` track in `100px` definite space under stretch
alignment remains `0`; its expected used size is `100`.

### GRID-008 — Duplicate name tokens on one line create duplicate occurrences

**Severity:** Medium
**Evidence:** Executed, source-proven, and test-codified
**Status:** Current test encodes the mismatch

`src/grid/named.rs:732-743` emits an occurrence for every matching token, even
when the same line repeats a name. `src/grid_tests.rs:13653-13672` locks
`[a b a]` as two occurrences on line 1.

For `[a a] 40px [a] 40px` with start `a 2`, the engine resolves the duplicated
token on line 1 and places at `x=0`; a line's names form a set for occurrence
counting, so the second occurrence is line 2 and the expected `x` is `40`.

### GRID-009 — Legacy grid content-size accumulation loses grid-area origin

**Severity:** Medium
**Evidence:** Source-proven and test-codified
**Status:** Partly known/deferred with scroll work

Ordinary grid passes `location - area_origin` to content contribution at
`src/grid/child.rs:497-517`; lanes repeat the pattern at
`src/grid/lanes.rs:1430-1441`. The item's position in the container is therefore
discarded when computing content extent.

`src/grid_tests.rs:10308-10397` places an `80px` content extent at container
`x=50` but expects root content width `100` instead of `130`, codifying the
area-relative result. Typed grid scroll geometry remains explicitly deferred,
but `content_size` is already public and consumed.

### OVERFLOW-003 — Zero in either dimension erases contribution on both axes

**Severity:** Medium
**Evidence:** Executed and source-proven
**Status:** Cross-algorithm defect

The contribution helpers return `Size::ZERO` whenever either dimension is
non-positive at `src/block.rs:1849-1875`, `src/flex.rs:2394-2420`, and
`src/grid/child.rs:2189-2215`.

A visible `0x10` item with descendant content extending to height `100` should
contribute `100` vertically. The helpers return zero on both axes; Chrome
reports the equivalent `scrollHeight=100`. Block's newer origin-bearing
overflow accumulator may retain some cases independently, but flex/grid public
content extent does not.

### OVERFLOW-004 — Scroll range direction conventions remain unresolved

**Severity:** Medium
**Evidence:** Source-proven contract gap
**Status:** Explicit open decision

`ScrollRangeOf` stores only a non-negative maximum size and clamps every offset
to physical `0..maximum` at `src/scroll.rs:88-120`. `ScrollGeometryOf` separately
stores writing mode and direction at `src/scroll.rs:471-524`, but the range type
does not state whether offsets are physical or logical and cannot encode a
nonzero minimum.

The scroll support matrix already identifies physical-versus-logical ranges as
open. Until that contract is resolved, runtime interpretation across RTL and
vertical writing modes is ambiguous even where geometry is emitted.

### MODEL-006 — Alignment and inline alignment values are materially incomplete

**Severity:** Medium
**Evidence:** Capability gap
**Status:** Partial

`AlignContent` at `src/node_input.rs:488-501` cannot express baseline content
distribution. `TextAlign` only has auto and legacy physical values at
`src/node_input.rs:147-153`, with no layout state for justification, and
`VerticalAlign` only has baseline/top at `src/node_input.rs:155-160`.

Several authored Box Alignment keywords can legitimately be normalized by an
upstream style/tree layer, so their spelling alone is not counted as a layout
gap. The remaining geometry gaps are context-sensitive baseline distribution,
text justification, and the full inline vertical-alignment set. Block layout
also never reads the public `align_content` field, so represented block-
container cross-axis content alignment is inert; this finding does not claim
that every other content/self-alignment property applies to ordinary block
containers.

### MODEL-007 — Property-agnostic Dimension admits invalid `fr` states

**Severity:** Medium
**Evidence:** Source-proven contract gap
**Status:** Residual modeling issue

`DimensionOf` includes `Fr` at `src/value.rs:661-670`, and the same type is used
for box size, min/max size, and flex basis at
`src/node_input.rs:832-851`. Fractional units are track sizing values, not valid
for those box properties. Outside a track context, `Fr` resolves as non-numeric
at `src/value.rs:754-762`, which can silently make an invalid typed state behave
like an unresolved/auto-like value.

### DIAG-001 — Unsupported and invalid layout states have no unified compute result

**Severity:** Medium
**Evidence:** Source-proven contract gap
**Status:** Partial diagnostics only

`Compute::compute_child` returns a plain `ComputeOutputOf` at
`src/traits.rs:18-30`; top-level compute functions are likewise infallible in
their signatures. Unsupported states are variously handled by panic, zero
geometry, silent fallback, crate-private reports, or separately fallible scroll
constructors. `ScrollUnsupportedFeature` is not integrated into the compute
contract.

Callers cannot consistently distinguish valid layout, unresolved basis,
unsupported capability, and invalid layout-ready input end to end.

## Test And Oracle Findings

### TEST-002 — Parsed browser scroll expectations are never compared

**Severity:** Medium
**Evidence:** Source-proven assurance gap
**Status:** Known while scroll output is partial

`Expectation.scroll_size` is defined and parsed at
`tests/layout/browser_parity/support.rs:200-207,274-294`. The comparator checks
only x, y, width, height, and children at
`tests/layout/browser_parity/support.rs:951-994`; `scroll_size` has no comparison
use.

There are 312 checked-in XML files containing `scroll_width`/`scroll_height`
expectations. None can fail parity because of an incorrect scroll range or
overflow extent.

### TEST-003 — Browser parity skips every line-break node's geometry

**Severity:** Medium
**Evidence:** Source-proven assurance gap
**Status:** Intentional shortcut

`compare_expectation` returns immediately for a `LayoutInput::LineBreak` at
`tests/layout/browser_parity/support.rs:951-964`. It compares neither the
control's expected position nor any other node-level expectation for that
entry. Line-break sequencing can therefore be wrong while surrounding box
sizes happen to agree.

### TEST-004 — Corpus generation excludes important inline/BR surfaces

**Severity:** Medium
**Evidence:** Assurance gap
**Status:** Current generation report

The checked-in report at
`tests/layout/browser_parity/xml/generation-reports/all.json` records 5,048
generated and 356 unsupported cases:

| Unsupported reason | Count |
| --- | ---: |
| Vertical `<br>` line-break semantics | 144 |
| Mixed text/element content | 100 |
| `<br>` outside block inline-run semantics | 96 |
| Missing `#test-root` | 16 |

These exclusions align with real production gaps rather than harmless fixture
syntax. In addition, the active float XML suite contains only four files while
richer float sources remain outside active comparison.

### TEST-005 — Current vertical-flex fixtures do not exercise flex layout axes

**Severity:** Medium
**Evidence:** Assurance gap and source inspection
**Status:** Misleading apparent coverage

The 12 vertical-writing flex XML cases represent the relevant flex element as a
text leaf. They therefore go through measurement rather than non-leaf
`compute_flex`. Their success does not cover the writing-mode axis failure in
FLEX-001.

## Low-Severity Public Contract Findings

### CORE-006 — `NodeOutput::content_box_size` omits scrollbar reservation

**Severity:** Low
**Evidence:** Source-proven
**Status:** API inconsistency

`NodeOutputOf::content_box_size` subtracts padding and border only at
`src/output.rs:259-272`. Internal content-box construction includes scrollbar
reservation at `src/scroll.rs:338-344`, while `NodeOutputOf` exposes
`scrollbar_size` separately.

A `100px` border box with a `15px` classic vertical gutter can therefore report
`100px` from `content_box_size()` while the layout algorithm used `85px` as its
content width.

### CORE-007 — Raw public numeric inputs admit non-finite and negative states

**Severity:** Low
**Evidence:** Source-proven
**Status:** Public invariant gap

`NodeInputOf` exposes raw `scrollbar_width`, `flex_grow`, and `flex_shrink`
scalars at `src/node_input.rs:827-851`. Negative or non-finite scrollbar widths
flow through `src/scroll.rs:274-316` into negative sizes/insets; flex factors
likewise have no layout-ready validation boundary.

This contradicts the README's claim that public layout-ready contracts preserve
their invariants and allows invalid numeric states to reach geometry code.

### OVERFLOW-005 — Scroll constructors validate fields but not full coherence

**Severity:** Low
**Evidence:** Source-proven
**Status:** Partial validation

`ScrollOffsetOf::new` accepts non-finite positions at `src/scroll.rs:69-86`.
`ScrollbarGutterRectsOf::new` accepts arbitrary rectangles at
`src/scroll.rs:232-260`. `ScrollGeometryOf::new` validates only whether the
container accepts a range and whether a clip is present at
`src/scroll.rs:485-513`; it does not establish coherence among scrollport,
overflow rectangle, range magnitude, visible-axis clips, and gutter rectangles.

## Focused Browser-Parity Evidence

The current XML distribution is:

| Suite | XML files | Focused result at review time |
| --- | ---: | --- |
| block | 888 | Multiple active mismatch and panic families listed below |
| blockflex | 28 | Green in focused run |
| blockgrid | 56 | Green in focused run |
| flex | 2,280 | Calc fixture panics; remaining focused corpus green |
| float | 4 | Green but extremely narrow |
| grid | 1,148 | Calc fixture panics; remaining 1,144 files green in diagnostic run |
| grid-lanes | 56 | 52 pass; four containing-block variants fail |
| gridflex | 24 | Green in focused run |
| leaf | 56 | Green in focused run |
| subgrid | 508 | Green in focused run |

The block families confirmed by filtered current-source runs are:

| Family | Observed result |
| --- | --- |
| `block_align_baseline_child_margin_percent` | Four y mismatches |
| `block_br_inline_block_metrics` | Four height mismatches |
| `block_calc_width_margin` | Calc resolver panic |
| `block_direction_rtl_with_br` | Flow-mismatch panic |
| `block_margin_x_intrinsic_size_negative` | Invalid scroll-rect panic |
| `block_margin_y_collapse_complex` | Invalid scroll-rect panic |
| `block_margin_y_collapse_through_negative` | Invalid scroll-rect panic |
| `block_margin_y_sibling_collapse_negative` | Invalid scroll-rect panic |
| `block_margin_y_sibling_collapse_positive_and_negative` | Invalid scroll-rect panic |
| `block_margin_y_simple_negative` | Invalid scroll-rect panic |
| `block_margin_y_total_collapse_complex` | Invalid scroll-rect panic |
| `block_overflow_scrollbars_overridden_by_available_space` | Negative content-box panic |
| `block_overflow_scrollbars_overridden_by_size` | Negative content-box panic |

The ignored all-corpus runner cannot produce one clean aggregate result because
the first uncaught panic aborts the test process. The per-family results above
were used so one abort would not conceal later independent failures.

## Confirmed Strengths And Rejected Hypotheses

The review did not treat every incomplete-looking path as a defect. Confirmed
strengths and stale hypotheses rejected during review include:

- the crate contains no `unsafe` code;
- normal formatting, clippy, unit, integration, and generator-feature checks
  pass;
- the cache key itself includes run mode, sizing mode, requested axis, parent,
  known size, available size, and calc generation; CORE-001 concerns the cached
  payload, not those key dimensions;
- zero grid lines/spans and invalid empty track repetitions now have typed
  validation;
- named-grid validation now produces a typed report rather than universally
  falling back silently;
- absolute flex/grid children are excluded from normal flex-line/grid occupancy
  collection;
- matching-flow horizontal RTL atomic-inline placement works;
- hidden line controls remain zero-sized and do not create visible lines;
- focused horizontal `<br clear>` cases pass; the float defect is ordinary
  non-clearing line exclusion;
- constrained vertical `<br>` cases pass; broader vertical inline behavior is
  still incomplete;
- `overflow: clip` not establishing a BFC/margin-collapse barrier matches the
  reviewed CSS behavior and is not listed as a defect;
- percentage margins use the containing inline-size basis;
- the broad absolute-position parity families pass outside the inline-static
  and flex-auto-margin cases recorded above;
- sub-one flex grow/shrink sums have focused passing coverage;
- all 508 checked-in subgrid fixtures pass, and no additional grid-lanes parity
  mismatch was found beyond the four width/RTL variants;
- basic generated `float_simple` fixtures pass.

## Review Limitations

- Browser comparisons used the repository's pinned Chrome-for-Testing 149 and
  constrained fixtures; a current WPT corpus is absent.
- Several focused algorithm repros were placed only in disposable `/tmp`
  checkouts. They are execution evidence for this review but not permanent
  regression tests.
- Evolving drafts such as Grid Level 3 and Positioned Layout Level 4 can change.
  Their absent features are recorded as capability gaps, not claims that every
  draft detail is already a stable conformance requirement.
- This is a static and deterministic layout review. Performance under very
  large trees, denial-of-service resistance, platform font variability, paint,
  accessibility tree order, and runtime invalidation are outside the crate-local
  scope.

## Final Conclusion

The crate has a credible base for selected horizontal block/flex/grid and
subgrid cases, but its current success criteria are too narrow for the stated
goal. Reliability is blocked first by panics and cache semantic loss. Beyond
that, vertical writing, implicit grid growth, fit-content/flexible tracks,
inline float exclusion, nested scrolling, and several ordinary inline/flex/grid
branches are demonstrably wrong. The public model and output shape also leave
large parts of current modular CSS layout unrepresentable.

No implementation source, fixture, generated artifact, or sibling crate was
changed as part of this review.
