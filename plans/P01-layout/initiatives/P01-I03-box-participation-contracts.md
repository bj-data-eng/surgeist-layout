# P01-I03 Box Participation Contracts


Design owner: `surgeist-layout`

Specification ID: `FRI-03`

## 1 FRI-03.1 Authority And Outcome

This specification is the direct desired-state contract for `FRI-03` in
`plans/P01-layout/P01-index.md`. It owns
closure of `MODEL-001`, `CORE-005`, and `BLOCK-007` from
`plans/P01-layout/P01-initial-review-findings.md`.

The outcome is one layout-ready participation model in which:

1. a signed item-order value is distinct from a source-tree index;
2. flex, ordinary grid, and grid-lanes consume one stable order-modified
   traversal contract while block and inline flow ignore it;
3. output identity remains tied to source-tree nodes and source indexes rather
   than CSS order values or order-modified ranks;
4. replacedness remains distinct from table role, authored size, aspect ratio,
   and leaf-measurement capability, and every FRI-03-owned sizing branch
   consumes it;
5. recursive compute input carries the current box's resolved parent formatting
   context together with the containing flow axes;
6. cache identity includes every new contextual value;
7. flex and grid items cannot collapse their own margins with descendant block
   margins as ordinary block-flow children can; and
8. browser, front-door, oracle, scalar, API, and documentation evidence closes
   the three findings without expanding generator architecture.

This is a breaking pre-release correction. Backward compatibility is not
required. Removed or renamed APIs are not retained through aliases, deprecated
wrappers, duplicate fields, or permissive conversions.

## 2 FRI-03.2 Scope And Non-Goals

### 2.1 Owned Scope

This specification owns:

- a scalar-independent public `ItemOrder` value and its `NodeInputOf` field;
- a scalar-independent public `SourceIndex` value and unambiguous output naming;
- the stable `(item order, source index)` traversal key;
- flex ordering before line construction and wrapping;
- ordinary-grid ordering in every order-sensitive auto-placement phase while
  retaining source-indexed placement storage;
- production grid-lanes placement and intrinsic-contribution traversal order;
- a public resolved `ParentFormattingContext` domain;
- a public `ContainingLayoutContext` that keeps containing flow and parent
  formatting participation inseparable through recursive compute and caching;
- an explicit flex-item-root parent-flow value instead of deriving that
  containing flow from the root item's own writing mode;
- block, leaf, root, flex-root, hidden, flex, grid, and grid-lanes construction
  of that context;
- parent/first-child and parent/last-child margin-collapse barriers for flex and
  grid items without disabling their internal sibling collapse;
- replaced in-flow block and root auto-inline sizing;
- replaced ordinary-grid and grid-lanes default/normal self-alignment;
- the replaced/non-replaced branch of the existing flex automatic minimum;
- exact order capture, XML parsing, new order fixtures, and generated provenance
  required to exercise these contracts;
- exact flex-item-root parent-axis capture and XML parsing for the existing
  viewport fixture front door;
- bounded decoupling of optional diagnostic fixture filters from retained
  generation reports for the confirmed generator workflow bug; and
- public reexports, crate docs, focused tests, reports, and root integration
  requirements.

### 2.2 Explicit Non-Goals

This specification does not:

- expand or redesign the generator architecture; generator changes are limited
  to capturing and serializing one exact integer, capturing and serializing the
  existing flex-item viewport parent's computed writing mode and direction,
  parsing those three attributes, adding three order fixtures, decoupling the
  existing diagnostic filter from report persistence, and regenerating derived
  artifacts;
- teach the browser harness a natural-size or replaced-element measurement
  model, infer replacedness from a tag, or add replaced browser fixtures;
- parse authored CSS, run cascade, compute `order`, or decide which DOM elements
  are replaced; root and sibling owners supply layout-ready facts;
- change speech, navigation, accessibility, focus, or DOM order;
- implement order-dependent painting or stacking of flex/grid items or
  absolutely positioned descendants;
- globally reorder `Traverse::children`, block children, inline participants,
  floats, hidden traversal, rounded traversal, or public lane utility inputs;
- add CSS order to public `LaneItemOf` or change the caller-order contract of
  `place_lanes`;
- merge table and replaced roles or model table layout;
- implement a natural-dimension/default-object-size model, object fitting,
  replaced baselines, positioned replaced sizing, or rendering;
- exempt replaced flex items from normal or explicit cross-axis stretch;
- claim complete flex replaced sizing, which later `FRI-07` must compose with
  its completed algorithm;
- normalize outer/inner display, add missing display roles, or absorb any part
  of `FRI-12A` through `FRI-12F`;
- fix unrelated inline, overflow, alignment, positioned, fragmentation, or
  grid-completeness findings;
- edit root adapters, root facade exports, root API artifacts, or sibling
  repositories; or
- acquire software, add `unsafe`, or change the crate's MSRV.

## 3 FRI-03.3 Standards And Current Evidence

### 3.1 Normative Evidence

CSS Display Level 3 defines `order` as an integer with initial value zero,
applying to flex and grid items. Flex and grid containers lay items out from the
lowest ordinal group upward, preserving source order within an equal group:

- <https://www.w3.org/TR/css-display-3/#order-property>

CSS Flexbox Level 1 requires order-modified document order for flex layout,
defines flex items as independent formatting contexts, states that adjacent
flex-item margins do not collapse, and distinguishes replaced from non-replaced
automatic minimum sizing:

- <https://www.w3.org/TR/css-flexbox-1/#order-property>
- <https://www.w3.org/TR/css-flexbox-1/#flex-items>
- <https://www.w3.org/TR/css-flexbox-1/#item-margins>
- <https://www.w3.org/TR/css-flexbox-1/#min-size-auto>

CSS Grid Level 2 applies `order` to grid items, uses order-modified document
order in the definite-major and remaining auto-placement phases, and states
that adjacent grid-item margins do not collapse:

- <https://www.w3.org/TR/css-grid-2/#order-property>
- <https://www.w3.org/TR/css-grid-2/#auto-placement-algo>
- <https://www.w3.org/TR/css-grid-2/#item-margins>

CSS Box Alignment Level 3 defines grid `normal` self-alignment as stretch for a
typical non-replaced box and start for a typical replaced box. Explicit
`stretch` remains distinct and can stretch a replaced box:

- <https://www.w3.org/TR/css-align-3/#justify-self-property>
- <https://www.w3.org/TR/css-align-3/#align-self-property>

CSS 2 defines a block-level replaced box's auto width through the replaced
width rules rather than the ordinary non-replaced block fill equation:

- <https://www.w3.org/TR/CSS2/visudet.html#block-replaced-width>

CSS Values Level 4 permits an implementation-defined finite numeric range.
`ItemOrder` therefore uses an exact signed 32-bit layout-ready domain; authored
values outside that domain are an upstream lowering concern and are never
rounded through a layout scalar:

- <https://www.w3.org/TR/css-values-4/#integers>

### 3.2 Source Evidence At The Published Base

This table describes clean published commit
`14f887823c22b69c083e522a73826e6a30b180e0`.

| Evidence ID | Current source fact | Required correction |
| --- | --- | --- |
| `E-ORDER-INPUT` | `NodeInputOf` in `src/node_input.rs` has no item-order value. | Add exact signed `ItemOrder`, default zero, without using `S`, `f32`, or `f64`. |
| `E-SOURCE-ID` | `NodeOutputOf.order: u32`, `with_order`, and numerous private `order: u32` fields actually carry source sibling ordinals. | Replace ambiguous naming with typed `SourceIndex`; CSS order never overwrites identity. |
| `E-FLEX-ORDER` | `src/flex.rs` enumerates source children and constructs lines in that order. | Sort in-flow collected items before any wrapping, line sizing, placement, or baseline selection. |
| `E-GRID-ORDER` | `src/grid/placement.rs` traverses a source-aligned `children`/`items` vector during placement. | Traverse the order-modified permutation in order-sensitive phases and write areas back by source index. |
| `E-LANES-ORDER` | `src/grid/lanes.rs` uses source order for production placement and sequential intrinsic contributions. | Feed both paths the canonical in-flow permutation; keep the public lane utility caller-ordered. |
| `E-REPLACED` | `item_is_replaced` is declared and defaulted but has no production read. | Consume it in every FRI-03-owned branch without conflating it with measurement or table role. |
| `E-BLOCK-REPLACED` | `in_flow_child_known_size` fills every ordinary auto-inline non-table child; `root_known_inline` similarly fills roots. | Do not inject ordinary fill for replaced boxes; allow measurement/authored sizing to determine the result. |
| `E-GRID-REPLACED` | Grid and the separate lanes pre-placement path default auto-sized items to stretch without replacedness. | Resolve only default/normal replaced alignment to start; preserve explicit stretch. |
| `E-FLEX-REPLACED` | Flex automatic minimum always takes the larger content/transferred suggestion. | Use the smaller suggestion for replaced items and the larger for non-replaced items. |
| `E-PARENT-CONTEXT` | `ComputeInputOf` carries containing flow axes but no parent formatting context; `CacheKeyOf` mirrors that omission. `FlexItemRootContextOf` carries only viewport availability, so `compute_flex_item_root` substitutes the item's own axes for its flex parent's axes. | Carry one closed context value through every constructor, recursive call, and cache key, and require the flex-item-root front door to supply its parent axes explicitly. |
| `E-COLLAPSE` | Block constants and leaf collapse output decide eligibility only from the current node's style and run mode. | Flex/grid participation blocks collapse across the item boundary while preserving internal sibling collapse. |
| `E-PARITY` | The four `block_align_baseline_child_margin_percent` variants expect nested-child `y=1`; the engine produces `y=0`. No checked-in XML can encode CSS order, existing `root-context="flex-item"` XML carries no parent axes, and fixture support reuses parent viewport availability as the root item's host allocation. | Make all four variants pass, add narrow exact-integer order capture/parser/fixtures, and make the existing flex-item viewport schema carry its actual parent axes and browser-observed host inline allocation without a fallback. |

The current browser corpus has 5,256 generated cases, 356 unsupported cases,
zero expected failures, and six reports (one full plus five retained FRI-02
scopes). Root `surgeist` is clean at
`19590f6d9fa01c0df197c5ef07fb626c5cf18ced`; its committed layout gitlink is
`c0c6852610b835b60e46c680fbd1a4fb127d1d13`.

## 4 FRI-03.4 Resolved Design Decisions

### 4.1 `D-01` Item Order And Source Index Are Different Types

`ItemOrder` is a public scalar-independent newtype over `i32`. Every signed
32-bit value is valid. `ItemOrder::ZERO` and `Default` represent the CSS initial
value. Construction and access are explicit and infallible:

```rust
pub struct ItemOrder(i32);

impl ItemOrder {
    pub const ZERO: Self;
    pub const fn new(value: i32) -> Self;
    pub const fn get(self) -> i32;
}
```

`NodeInputOf<S>` exposes `pub item_order: ItemOrder`. It is not generic over
`S`, because order is an exact integer and never participates in coordinate
arithmetic.

`SourceIndex` is a distinct public newtype over `usize` with `ZERO`, `new`, and
`get`. For a non-root node it is the zero-based index among its source siblings;
root and standalone outputs use `ZERO` by convention. It is not a CSS value,
ordinal group, sorted rank, painting index, retained identity, or tree node
handle.

`NodeOutputOf<S>` replaces `order: u32` with
`source_index: SourceIndex`. `with_order` is replaced by
`with_source_index(SourceIndex)`. Private source-ordinal carriers in block,
inline, flex, grid, grid-lanes, and subgrid use `source_index` naming and either
`SourceIndex` or `usize`; no production `order: u32` source carrier remains.

Rejected alternative: two primitive fields named `order` preserve the exact
semantic collision that caused `MODEL-001`.

Rejected alternative: storing item order in the layout scalar loses exactness
and makes a non-geometric value vary between scalar lanes.

Rejected alternative: replacing source identity with order-modified rank breaks
source-aligned grid/subgrid reports and equal-order identity.

### 4.2 `D-02` One Stable Order-Modified Permutation Serves Layout Algorithms

One crate-private helper accepts a finite sequence of in-flow source indexes and
their `ItemOrder` values and returns a permutation sorted lexicographically by:

1. ascending signed `ItemOrder`; then
2. ascending `SourceIndex`.

Equal item-order values therefore retain source order. All-zero defaults return
the original sequence. Algorithms select their own eligible in-flow items
before invoking the helper; the helper never traverses the tree, decides
display/position, or mutates output.

Flex consumes the permutation before line construction, so ordering affects
wrapping, flexing, placement, baselines, and visual progression. Reverse flex
directions reverse physical main-axis progression; they do not reverse the
order-modified sequence a second time.

Ordinary grid preserves its source-indexed `children`, `items`, `areas`, and
subgrid-report arrays. Fully definite cells are marked independently of order.
The definite-major and remaining auto-placement phases traverse the canonical
permutation and write each result to its source-indexed slot. Dense versus
sparse cursor behavior otherwise remains unchanged.

Production grid-lanes placement receives the same in-flow permutation before
running-offset assignment. Its sequential intrinsic-contribution path also
uses that permutation, so measurement and final placement cannot disagree.
`place_lanes` and `LaneItemOf` remain caller-ordered public utilities; the
production adapter supplies already ordered inputs instead of adding another
public CSS field.

Block, inline, float, hidden, absolute-position layout, rounding traversal, and
tree traversal ignore `ItemOrder`. Order-dependent painting is outside this
initiative. All outputs continue to publish their source indexes. Hidden
children receive their actual enumerated source indexes even though they occupy
no layout slot; a hidden root uses `SourceIndex::ZERO`.

### 4.3 `D-03` Containing Flow And Parent Formatting Context Travel Together

`ParentFormattingContext` is a public closed enum:

```rust
pub enum ParentFormattingContext {
    NoParent,
    BlockFlow,
    Flex,
    Grid,
}
```

`Grid` covers both ordinary grid and grid-lanes scheduling. The enum describes
the generated parent's scheduling/containing algorithm for the current box; it
does not by itself assert that the box is in flow. An absolutely positioned
child scheduled by flex or grid therefore retains `Flex` or `Grid`, while its
existing `Position::Absolute` state excludes item-only ordering and collapse.
The enum does not replace `Display`, describe the current box's inner algorithm,
or normalize CSS outer display.

`ContainingLayoutContext` is a public immutable value with private fields:

```rust
pub struct ContainingLayoutContext {
    flow_axes: FlowAxes,
    formatting_context: ParentFormattingContext,
}
```

It has an infallible `new`, `flow_axes`, and `formatting_context` API and no
`Default`. Every combination is meaningful: the flow mapping remains required
for percentage and logical-axis resolution even when no generated parent
exists.

`ComputeInputOf` replaces its standalone `containing_flow_axes` field with one
`ContainingLayoutContext`. Existing `containing_flow_axes()` remains only as a
named projection of that context, while new `containing_layout_context()` and
`parent_formatting_context()` accessors expose the complete value and its role.
Public `leaf_layout` and `leaf_content_size` take a
`ContainingLayoutContext`, not two independently ordered context arguments.
There is no old-signature overload.

All private constructors and recursive calls accept the same context value.
Block parents construct `BlockFlow`, flex parents construct `Flex`, and ordinary
grid/grid-lanes parents construct `Grid`, always with the parent's resolved
`FlowAxes`; this applies consistently to their in-flow, intrinsic, and absolute
child requests. A viewport root constructs `NoParent` with its own resolved
axes because the layout-ready viewport contract has no separate
initial-containing-block style.

`FlexItemRootContextOf<S>` adds a scalar-independent `parent_flow_axes` field of
type `FlowAxes` alongside `viewport_available`. Its only constructor is
`under_viewport(viewport_available, parent_flow_axes)`, and
`parent_flow_axes()` exposes the exact value. The flex-item-root path constructs
`ContainingLayoutContext::new(context.parent_flow_axes(), Flex)` and uses that
parent mapping for percentage bases, logical edges, auto-inline sizing, and
cache identity. It does not derive containing flow from the root item's own
writing mode. The root item's axes remain authoritative only when that box
schedules its descendants. There is no one-argument compatibility constructor
or implicit horizontal fallback.

The public root request's `available` value remains distinct from
`FlexItemRootContextOf::viewport_available()`. For a browser fixture whose root
is already allocated as a flex item, `available` carries that host allocation:
the parent inline physical axis is definite at the browser-observed item border
box size and the other physical axis is max-content. The context retains the
parent viewport dimensions for percentage bases. Reusing the viewport size as
the host allocation incorrectly turns 400/60-unit parents into 400/60-unit
items instead of the browser's max/min-content-clamped 160/80-unit items.

A hidden node preserves the context supplied by the algorithm that encountered
it, but hidden computation does not consult the role and gives that
non-generated node's descendants `NoParent`.

`CacheKeyOf` stores the entire `ContainingLayoutContext`, and the manual
`matches_output` predicate compares the complete value rather than projecting
only `FlowAxes`. Two requests that differ only in parent formatting context must
miss; cached and uncached outputs must agree for size, content size, baselines,
and physical margin-collapse state.

Rejected alternative: adding a boolean such as `is_flex_or_grid_item` encodes a
single current symptom, cannot distinguish the resolved producer, and invites
another flag when a later context differs.

Rejected alternative: inferring the role from the current node's `Display`
confuses inner layout with outer participation and repeats the present bug.

Rejected alternative: expanding `Display` is owned by `FRI-12A`, not this
initiative.

### 4.4 `D-04` Parent Context Gates Only Boundary Collapse

A block box in `BlockFlow` may collapse its block-start/block-end margins with
its first/last in-flow block child when all existing style, edge, size,
position, run-mode, and axis conditions permit. `Flex`, `Grid`, and `NoParent`
do not permit collapse across that box boundary.

This gate is applied in both block constants and measured-leaf collapse output.
It disables the current box's `collapse_top_margin`,
`collapse_bottom_margin`, and `can_collapse_through` boundary states where
appropriate. It does not disable margin collapse between adjacent in-flow block
children inside a flex/grid item's independent block formatting context.

The gate is logical-axis neutral. Existing `FlowAxes` and
`PhysicalBlockMarginCollapseOf` remain the only owners of mapping and physical
collapse output; this initiative adds no physical top/bottom special case.

### 4.5 `D-05` Replacedness Remains An Independent Proposition

`pub item_is_replaced: bool` remains the representation and defaults to false.
A boolean is appropriate because replacedness is one independent proposition,
not a mutually exclusive mode shared with table role, measurement capability,
or authored sizing. The following facts remain orthogonal:

- `item_is_table` controls existing table-wrapper/shrink behavior;
- `LayoutTree::has_leaf_measurement` says whether intrinsic measurement is
  available;
- the measurement callback supplies natural/intrinsic dimensions;
- `aspect_ratio` is a layout-ready preferred ratio; and
- explicit self-alignment remains distinct from default/normal alignment.

FRI-03 consumes replacedness as follows:

| Context | Required behavior |
| --- | --- |
| In-flow block child | Ordinary auto-inline fill is injected only when both `item_is_table` and `item_is_replaced` are false. A measured 50-unit replaced child in a 200-unit containing block remains 50 units; the paired ordinary block fills 200. |
| Viewport or flex-item root | `root_known_inline` does not inject ordinary auto-inline fill for a replaced measured root. Authored/min/max constraints and measurement remain authoritative. |
| Ordinary grid | When both item and container alignment are absent (the layout-ready `normal` state), an auto-sized replaced item resolves to `Start` on each axis. An explicit item/container `Stretch` still stretches it. |
| Grid-lanes | Final grid-item sizing and the separate pre-placement grid-axis measurement use the same replaced-aware normal-alignment helper. |
| Flex automatic minimum | When a transferred suggestion exists, a replaced item selects the smaller content/transferred suggestion; a non-replaced item selects the larger. Existing caps, clamps, overflow-zero behavior, and padding/border floor remain. |

If `item_is_table` and `item_is_replaced` are both true, each role applies to
its own consumers. The shared block auto-fill exclusion does not make them one
box kind or reject the combination.

This initiative does not globally exempt replaced flex items from stretch and
does not invent natural dimensions when no measurement provider or authored
size exists.

### 4.6 `D-06` Batch Entry Order Is Not CSS Order

`CompletedLayoutBatchOf::final_entries()` retains source-tree rounding
traversal. `unrounded_entries()` remains computation staging order and is not a
semantic source, CSS, painting, or accessibility order. Both methods are
documented accordingly. Consumers identify an entry through `entry.node()` and
read `output.source_index`; they do not infer identity from slice position.

The implementation need not buffer flex/grid staging writes merely to make
unrounded entries follow CSS order or source order.

## 5 FRI-03.5 Public Contract

The completed public front door includes and reexports:

- `ItemOrder`;
- `SourceIndex`;
- `ParentFormattingContext`;
- `ContainingLayoutContext`;
- the two-argument `FlexItemRootContextOf::under_viewport` constructor and its
  `parent_flow_axes` accessor;
- `NodeInputOf::item_order`;
- `NodeOutputOf::source_index`; and
- the complete-context `ComputeInputOf` constructors and accessors.

The following old public surface is absent:

- `NodeOutputOf::order`;
- `NodeOutputOf::with_order`; and
- the `ComputeInputOf::leaf_layout` / `leaf_content_size` signatures that take a
  bare `FlowAxes` without parent participation.

No compatibility aliases, deprecated fields, convenience overloads, implicit
primitive conversions, or duplicate context paths remain. `ItemOrder` and
`SourceIndex` use their named constructors and accessors.

All new types are scalar-independent and derive the ordinary copy/debug/equality
and ordering/hash traits required by their semantic use. They introduce no
generic scalar bound and no runtime allocation.

## 6 FRI-03.6 Algorithm Contracts

### 6.1 Order Matrix

| Algorithm/path | Eligible order consumers | Ordering point | Storage/output invariant |
| --- | --- | --- | --- |
| Block and inline | None | Never | Source traversal and source indexes remain unchanged. |
| Flex | Visible in-flow flex items | Immediately after collection, before line construction | Geometry follows order-modified sequence; output source indexes do not. |
| Ordinary grid | In-flow grid items requiring order-sensitive auto-placement | Before definite-major and remaining placement traversal | `areas` and reports remain source-indexed. |
| Grid-lanes production placement | Visible in-flow lane items | Before running-offset placement | Node/source mappings remain authoritative. |
| Grid-lanes intrinsic contributions | Same production in-flow item set | Before sequential contribution application | Intrinsic sizing and final placement use the same sequence. |
| Fully definite grid placement | No traversal dependency | Occupancy is marked independently | Overlap remains allowed; painting is out of scope. |
| Absolute/display-none children | None for in-flow layout | Never | They retain their enumerated source indexes and occupy no in-flow slot. |

Required cases include negative, zero, positive, and equal values; flex row and
reverse progression; grid row/column flow; sparse/dense placement; mixed fully
definite, definite-major, and auto items; and grid-lanes running offsets.

### 6.2 Parent-Context Matrix

| Current box context | Boundary margin behavior | Constructor owner |
| --- | --- | --- |
| `NoParent` | No collapse with a nonexistent generated parent; root run mode remains an independent barrier. | Viewport root and descendants of a hidden/non-generated box. |
| `BlockFlow` | Existing CSS block parent/child collapse rules may apply. | Block formatting algorithm. |
| `Flex` | An in-flow item's margins do not collapse with its contents; internal block siblings may collapse. Absolute children retain this scheduling context but remain excluded by position. | Flex algorithm and flex-item-root front door. |
| `Grid` | An in-flow item's margins do not collapse with its contents; internal block siblings may collapse. Absolute children retain this scheduling context but remain excluded by position. | Ordinary grid and grid-lanes algorithms. |

Every sizing and perform-layout request for the same child uses the same
parent-formatting role. Intrinsic passes do not silently fall back to
`BlockFlow`. The role is observable in cache identity even when the current
request's geometry happens to match.

## 7 FRI-03.7 Failure And Numeric Semantics

- Every `i32` item order and every `usize` source index is valid; their
  constructors are infallible.
- Sorting is total and deterministic. No numeric subtraction comparator is used,
  so extreme signed values cannot overflow.
- Browser XML parsing accepts only a canonical signed base-10 integer that fits
  `i32`; fractions, exponent notation, non-numeric strings, and out-of-range
  values return fixture parse errors.
- Root CSS/style lowering later owns authored/computed values outside layout's
  `i32` domain. It must clamp or reject according to its explicit CSS contract;
  it must not convert through `f32` or `f64`.
- Every `ContainingLayoutContext` combination is valid and construction is
  infallible.
- No new `LayoutErrorKind`, panic path, silent fallback, or unsupported bucket is
  introduced by this initiative.
- Existing fallible length, measurement, scroll, and tree operations retain
  their current typed errors.

## 8 FRI-03.8 Browser, Fixture, And Oracle Contract

### 8.1 Narrow Generator/Parser Update

The existing constrained-HTML pipeline changes only as follows:

1. `test_helper.js` captures the computed `order` token as an exact base-10
   integer string with initial value zero;
2. for an existing `.viewport` flex-item root, `parseViewportConstraint`
   captures `getComputedStyle(e.parentElement).writingMode` and `.direction` as
   exact computed tokens plus `hostInlineSize`, the root border-box width or
   height selected by those parent axes, alongside `rootContext: "flex-item"`;
3. the Rust serializer emits non-default `order="..."` attributes and emits
   `parent-writing-mode="..."`, `parent-direction="..."`, and a finite
   non-negative `host-inline-size="...px"` on every `root-context="flex-item"`
   viewport;
4. `support.rs` parses order directly into `ItemOrder` without a layout scalar,
   requires all three flex-item attributes, constructs `FlowAxes`, and builds
   request availability with the parent inline physical axis definite at the
   host inline size and the other axis max-content;
5. a root viewport omits all three flex-item attributes, while a flex-item
   viewport with any attribute missing, invalid, non-pixel, non-finite,
   negative, or silently defaulted is a fixture error; and
6. exactly three new active Surgeist HTML sources exercise flex, ordinary-grid,
   and grid-lanes order behavior; and
7. a filtered ExistingPinned run accepts one normalized fixture path/prefix
   that matches at least one source, writes only matching XML, writes or prunes
   no generation report, and never substitutes for the final full run.

This additional viewport metadata is a bounded parser/schema correction for two
confirmed front-door bugs: the 16 existing flex-item-root outputs cannot supply
the public parent-context contract without parent axes, and cannot distinguish
the root request's host allocation from viewport percentage context without the
browser-observed host inline size. It adds no source or output, changes no
browser launch/import mechanism, does not infer parent facts from the item root,
and does not feed oracle expectations back into fixture input.

The existing filter/report coupling is a confirmed generator workflow bug:
diagnostic filtering currently requires a manifest report and therefore turns
iteration into retained verification state. The bounded fix uses the existing
filter and `generate-existing` command, rejects absolute, escaping, whitespace-
padded, or unmatched filters before artifact writes, emits no diagnostic report,
and leaves full generation as the sole report writer and stale-report pruner.
It adds no command, module, script, dependency, schema version, report kind,
browser behavior, or acquisition path.

The new source IDs are:

- `flex/fri03_order_modified_flex`;
- `grid/fri03_order_modified_grid`; and
- `grid-lanes/fri03_order_modified_lanes`.

Each source generates the standard four box-sizing/direction variants, for 12
new generated outputs. The existing
`block/block_align_baseline_child_margin_percent` source supplies four
participation outputs. The bounded viewport-schema regression set adds the 16
existing outputs from these four source IDs:

- `grid/grid_available_space_greater_than_max_content`;
- `grid/grid_available_space_smaller_than_max_content`;
- `grid/grid_available_space_smaller_than_min_content`; and
- `grid/chrome_issue_325928327`.

The exact FRI-03 browser union is therefore 32 outputs: 12 order, four margin
participation, and 16 flex-item-root parent-axis schema outputs.

The corpus retains exactly one generation report,
`generation-reports/all.json`. Existing scoped report entries and files are
retired from final evidence, and the final manifest has an empty scoped-report
inventory. Scoped runs remain optional diagnostic tools while
iterating, but are neither mandatory gates nor retained verification evidence.
After the final schema implementation and fixtures settle, exactly one
successful full ExistingPinned run must produce all 5,268 XML outputs, write
`all.json`, and prune every non-manifest scoped report. Manual report deletion
and a repeated full regeneration of those same settled inputs are forbidden.
The exact nonignored inventory test covers the 32 owned outputs without
requiring scoped evidence.
The full report contains exactly 5,268 generated and 356 unsupported cases,
with every failure-class count zero. The unsupported tuple set remains
byte-semantically identical to the published base, with normalized tuple SHA-256
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`.

The generated inventory contains exactly 1,406 HTML sources and 5,268 XML
outputs. Its HTML split is 26 grid-lanes, 219 subgrid, and 1,161 other sources.

Changing the embedded helper changes its provenance hash. The crate-owned
derived state therefore contains current helper provenance on all 5,268
outputs, including the explicit parent-axis and host-inline-size attributes on
the 16 existing flex-item-root outputs. The full report carries the current
helper and manifest hashes, and corpus validation checks its output set against
the complete XML inventory. One full derivation followed by read-only checks
proves the bounded parser/schema update; it is not authority to refactor the
generator, change launch/runtime policy, import a corpus, or hand-edit XML.

Browser-derived evidence identifies the already-present pinned Chrome
`149.0.7827.115`, the repository-relative cached executable, and the unchanged
manifest-owned launch profile. It requires no managed acquisition.

### 8.2 Replaced Evidence Boundary

The current browser harness converts authored/replaced natural sizing into used
style dimensions and only provides text/zero-size leaf measurement. Inferring
tags, preserving natural dimensions, or introducing an object-size model would
expand generator architecture and produce a false parity claim. Replaced
behavior therefore closes through focused real `LayoutTree` front-door tests,
paired non-replaced controls, both scalar lanes, and the normative standards
links in this specification. No XML is hand-authored and no replaced case is
hidden in an unsupported bucket.

### 8.3 Focused Evidence

The implementation supplies at least:

- `ItemOrder`/`SourceIndex` construction, default, extreme, comparison, and
  non-confusion contract tests;
- generator/helper tests proving computed-style capture retains the exact
  string token including initial `"0"`, serialization omits zero and emits both
  signed `i32` bounds exactly, and parsing accepts min/zero/max while rejecting
  out-of-range values, fractions, exponents, text, `+1`, leading zeros, `-0`,
  and surrounding whitespace;
- helper/serializer/parser tests proving a flex-item viewport captures and
  requires its actual parent computed writing-mode/direction tokens and
  browser-observed host inline size, a root viewport omits all three,
  missing/invalid/stray combinations fail closed, and an inline non-square
  orthogonal case selects the host height while preserving parent/root axis
  disagreement;
- request-lowering tests proving parent viewport availability remains the
  percentage context while the definite host inline allocation is separate,
  the other host axis is max-content, and the 400/80/60-unit fixtures produce
  their browser-observed 160/80/80-unit item widths without reading expected
  geometry as input;
- generator tests proving a valid matched diagnostic filter writes matching XML
  without changing report files, invalid or unmatched filters fail before
  writes, and only a full run writes `all.json` and prunes scoped reports;
- stale-helper provenance coverage and direct evidence that the one successful
  full run writes the exact one-report inventory, prunes every scoped report,
  and leaves full-report outputs exactly matching the XML inventory;
- flex negative/equal/positive order tests, including reverse direction,
  wrapping, and unchanged source indexes in both scalar lanes;
- ordinary-grid row/column and sparse/dense auto-placement tests with mixed
  definite-major and auto items, source-indexed areas, and source-indexed
  subgrid reports;
- grid-lanes placement and intrinsic-contribution tests proving the same
  permutation drives both paths;
- a block test proving item order is ignored;
- batch-entry tests proving node identity and source indexes survive ordering
  and rounding;
- cache hit/miss tests differing only in parent formatting context;
- block-flow, flex-item, grid-item, root, flex-item-root, hidden, measured-leaf,
  parallel-flow, and orthogonal-flow context tests in `f32` and `f64`;
- a non-square orthogonal flex-item-root test in both scalar lanes proving that
  percentage/logical resolution and cache identity use the supplied flex-parent
  axes rather than the root item's axes;
- the four existing `block_align_baseline_child_margin_percent` browser variants
  passing with nested-child `y=1`;
- one nonignored exact-inventory test named with the `runs_fri_03_` prefix that
  executes only the 32 FRI-03 browser outputs, plus a companion matrix test that
  rejects a missing, duplicate, misplaced, or extra output;
- paired replaced/non-replaced block tests with natural width 50 in width 200;
- paired viewport-root and flex-item-root replaced auto-inline tests;
- ordinary-grid replaced default/start, non-replaced default/stretch, and
  explicit replaced stretch tests on both axes;
- grid-lanes pre-placement measurement proving replaced default alignment does
  not inject the span while the paired ordinary item still does; and
- flex automatic-minimum tests in which content and transferred suggestions
  differ, plus preservation of replaced cross-axis stretch.

## 9 FRI-03.9 Source And Module Outline

| Path | Required responsibility |
| --- | --- |
| `src/node_input.rs` | Own `ItemOrder`, default it on `NodeInputOf`, and document independent replaced/table roles. |
| `src/output.rs` | Own `SourceIndex`, `ParentFormattingContext`, `ContainingLayoutContext`, the explicit flex-item-root parent axes, complete compute construction, and unambiguous output/batch docs. |
| `src/cache.rs` | Key and manually compare the complete containing context. |
| `src/compute.rs` | Construct root/flex-root/hidden contexts, preserve hidden source indexes, honor replaced root sizing, and apply leaf participation collapse gates. |
| `src/block.rs` | Construct block child contexts, gate boundary collapse, and suppress replaced auto-inline fill. |
| `src/flex.rs` | Consume stable item order before lines, construct flex contexts, preserve source indexes, and apply replaced automatic-minimum selection. |
| `src/grid/placement.rs` | Traverse the stable permutation in ordinary-grid order-sensitive placement phases. |
| `src/grid/child.rs` | Preserve source-indexed storage, construct grid contexts, and resolve replaced-aware normal alignment. |
| `src/grid/lanes.rs` | Use the permutation for production placement and intrinsic contributions and reuse replaced-aware default alignment. |
| `src/grid/subgrid.rs` and `src/inline.rs` | Rename source-index carriers without adopting CSS ordering. |
| `src/lib.rs` and `README.md` | Reexport and explain the layout-ready order, source identity, replaced, and containing-context contracts. |
| `tests/layout/browser_parity/scripts/gentest/test_helper.js` | Capture exact computed order plus the actual computed axes and browser-observed allocated inline size of an existing flex-item viewport parent. |
| `tests/bin/surgeist-layout-generate/generator.rs` | Serialize order, flex-parent axes, and host inline size; decouple diagnostic filters from report persistence; and preserve full-report provenance/pruning invariants. |
| `tests/layout/browser_parity/support.rs` | Parse exact item order, require complete flex-item viewport metadata, and keep host request availability separate from viewport percentage context. |
| `tests/layout/browser_parity.rs` | Own the exact 32-output inventory/topology gate and nonignored FRI-03 comparison. |
| `tests/layout/browser_parity/README.md` | Document optional diagnostic scoped runs and one final full ExistingPinned regeneration followed by read-only verification, with no repeated full run. |
| `tests/layout/browser_parity/html/` and `xml/` | Own the three generated order sources, the existing participation fixture, the 16 existing flex-item-root schema cases, and all derived XML. |
| `tests/layout/browser_parity/corpus.toml` | Record the three active sources and the single full generation report. |

No new module, feature, dependency, build script, proc macro, code generator,
or external crate is introduced.

## 10 FRI-03.10 Root Integration Contract

Root integration is deliberately read-only in this leaf initiative. A
compatible root composition has these observable requirements:

- exact signed-integer CSS parsing for `order`, without a floating conversion;
- a non-inherited computed `order` property with initial value zero lowered to
  `ItemOrder`;
- replacedness obtained from DOM/box-generation semantics rather than
  `surgeist-style` and passed through a root-owned layout-ready box-facts input;
- independent table and replaced roles;
- the resolved flex-parent `FlowAxes` supplied to every flex-item-root context,
  never substituted with the item's axes;
- retained invalidation that observes item order and replaced role changes;
- consumers migrated from `NodeOutput.order` to `NodeOutput.source_index`, using
  entry node identity where appropriate;
- integration evidence that order, replaced facts, and flex-parent axes reach
  the leaf API; and
- root facade and root-owned API artifacts that describe the completed leaf
  surface.

The current root style API has no `Order` property, and
`src/adapters/style_layout.rs` lowers only `style::Resolved` while defaulting
non-style box facts. This leaf does not guess either missing fact or edit root.

## 11 FRI-03.11 Compatibility, Feature, And Documentation Impact

- Adding `NodeInputOf::item_order` breaks exhaustive public struct literals;
  default/FRU construction receives zero.
- `NodeOutputOf::order` and `with_order` are removed in favor of typed source
  index names with no alias.
- Direct `ComputeInputOf` construction uses `ContainingLayoutContext` and has no
  bare-flow overload.
- `FlexItemRootContextOf::under_viewport` requires explicit parent flow axes;
  its old one-argument signature is absent.
- `item_is_replaced` retains its public field shape but gains specified
  behavior; callers already setting it observe corrected geometry.
- Browser fixture flex-item roots require explicit host inline allocation in
  addition to parent axes; public root request types and signatures do not
  change.
- Default item order zero and ordinary non-replaced/block-flow contexts preserve
  existing geometry except where `BLOCK-007` identifies illegal collapse.
- `place_lanes`, `LaneItemOf`, traversal traits, root request shape, scalar
  aliases, and layout error types do not change.
- Filtered ExistingPinned runs become report-free diagnostics; the unfiltered
  run remains the only persisted generation-report producer.
- No Cargo feature, dependency, lockfile entry, task-runner recipe, browser pin,
  launch profile, import provenance, or MSRV changes.
- Source remains authoritative; root owns generated API artifacts.

## 12 FRI-03.12 Initiative-Wide Evidence

Geometry evidence covers both scalar lanes. The generator's focused tests,
corpus validation, the full report, exact owned-output inventory, and
generated-tree cleanliness together prove that no order-sensitive algorithm,
context constructor, cache component, replaced branch, old source-order name,
hand-authored XML, or generator expansion escaped the bounded contracts above.

The ignored aggregate `runs_all_checked_in_browser_parity_xml` remains visible
but is not an FRI-03 green gate: it contains failures owned by later initiatives.
FRI-03 neither weakens nor quarantines those failures and claims only its exact
nonignored 32-output union.

## 13 FRI-03.13 Finding Closure Matrix

| Finding | Closure evidence |
| --- | --- |
| `MODEL-001` | `ItemOrder` is exact and distinct from `SourceIndex`; flex/grid/grid-lanes consume the stable permutation at the specified phases; block ignores it; output/source/subgrid identity remains source-indexed; parser, browser, oracle, scalar, and root-integration evidence pass. |
| `CORE-005` | `item_is_replaced` has production consumers in block/root auto-inline sizing, grid/grid-lanes normal alignment, and flex automatic minimum selection; paired controls prove it remains distinct from table, measurement, aspect ratio, and explicit stretch. |
| `BLOCK-007` | Complete parent context, including explicit flex-item-root parent axes, is constructed and cached on every path; flex/grid item boundaries suppress only illegal parent/child collapse; the exact four browser variants and logical/scalar/cache tests pass. |

No later initiative may claim closure for these IDs. `FRI-07`, `FRI-08`, and
`FRI-12A` consume the completed order, replaced, and participation contracts
without reopening their FRI-03-owned behavior.

## 14 FRI-03.14 Acceptance

FRI-03 is complete only when:

1. all public and private order/source concepts have the specified distinct
   types and names, with no legacy alias or scalar conversion;
2. default/equal/extreme item order is deterministic and every named algorithm
   consumes or ignores it exactly as specified;
3. ordinary-grid and grid-lanes storage/output remain source-indexed while their
   order-sensitive traversal is order-modified;
4. containing flow and parent formatting context travel as one value through
   every entry path, recursive pass, hidden path, and cache key;
5. flex/grid item boundary collapse is blocked without disabling internal
   sibling block-margin collapse or duplicating logical-axis mapping;
6. every FRI-03-owned replaced branch and paired non-replaced/explicit control
   passes in `f32` and `f64`;
7. the exact nonignored 32-output FRI-03 browser union, single full
   5,268/356 generation report, unchanged unsupported tuples, complete generated
   provenance, and corpus checks are green without claiming the aggregate parity
   test;
8. no replaced fixture, unsupported bucket, XML hand edit, generator change
   beyond the bounded metadata/schema and diagnostic-report bug fixes, software
   acquisition, root edit, dependency, feature, MSRV change, or `unsafe` is
   present;
9. public docs, README, source, tests, manifest, reports, and root integration
   requirements agree.
