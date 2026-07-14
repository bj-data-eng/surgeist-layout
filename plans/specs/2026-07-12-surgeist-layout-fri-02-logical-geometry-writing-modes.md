# FRI-02 Logical Geometry And Writing-Mode Substrate

Status: draft

Design owner: `surgeist-layout`

Specification ID: `FRI-02`

## FRI-02.1 Authority And Outcome

This specification is the direct desired-state contract for `FRI-02` in
`plans/specs/2026-07-11-surgeist-layout-findings-resolution-index.md`. It owns
closure of `BLOCK-003`, `FLEX-001`, `GRID-004`, `OVERFLOW-004`, and `TEST-005`
from `plans/2026-07-10-surgeist-layout-full-code-review-findings.md`.

The outcome is one layout-owned logical geometry model in which:

1. `WritingMode` represents `horizontal-tb`, `vertical-rl`, `vertical-lr`,
   `sideways-rl`, and `sideways-lr`;
2. one resolved `FlowAxes` value maps a writing mode and used inline direction
   to physical axes, sides, and progression;
3. block, flex, grid, grid-lanes, subgrid, existing inline-control layout, and
   scroll-coordinate contracts consume that same mapping rather than matching
   writing modes independently;
4. public `Point`, `Size`, `Edges`, rectangles, node output, collapsible-margin
   carriers, and scroll geometry remain explicitly physical, while
   crate-private logical geometry cannot be mixed with them accidentally;
5. ordinary block flow advances along the containing block's logical block
   axis in every writing mode;
6. flex row and column directions derive from logical inline and block axes in
   every writing mode;
7. grid columns and rows remain logical inline and block tracks until one
   explicit physical projection;
8. scroll offsets and ranges name their coordinate space, preserve signed
   bounds, and have one typed physical/flow-relative conversion contract; and
9. default verification exercises real non-leaf flex containers and the owned
   block/grid writing-mode behavior instead of treating measured text leaves as
   axis coverage.

This is a breaking pre-release correction. Backward compatibility is not
required. Removed or renamed APIs are not retained through aliases, deprecated
wrappers, duplicate conversion paths, or context-free helper methods.

## FRI-02.2 Scope And Non-Goals

### Owned Scope

This specification owns:

- the five-value layout-ready `WritingMode` domain;
- the meaning of layout `Direction` as an already-resolved used inline
  direction;
- public physical-axis and physical-side semantic types;
- the public resolved `FlowAxes` mapping contract;
- the public physical collapsible-margin carrier returned by compute output;
- crate-private logical point, size, edge, and rectangle algorithm types;
- physical/logical projection used by existing block, flex, grid, grid-lanes,
  subgrid, inline-control, baseline, output, rounding, and scroll paths;
- signed physical and flow-relative scroll offset/range types and their
  conversion contract;
- ordinary block-flow axis behavior required by `BLOCK-003`;
- flex axis behavior required by `FLEX-001`;
- grid intrinsic dimension and axis-dependent baseline behavior required by
  `GRID-004`;
- browser-parity topology and fixture coverage required by `TEST-005`; and
- public reexports, layout-owned docs, tests, fixtures, XML, and generation
  reports required by those contracts.

### Explicit Non-Goals

This specification does not:

- parse authored CSS, compute `writing-mode`, compute `text-orientation`, run
  bidi, or decide the used inline direction; root style lowering supplies
  layout-ready `WritingMode` and `Direction`;
- shape or orient glyphs;
- implement the complete inline formatting, float exclusion, vertical clear,
  strut, soft-wrap, or line-alignment behavior owned by `FRI-06`;
- fix negative flex cross-axis auto margins, flex absolute auto-margin
  equations, intrinsic flex-basis keyword behavior, or collapsed-item struts
  owned by `FRI-07`;
- fix grid placement demand, the rows-only lanes containing block,
  fit-content/flexible expansion, template-area track creation, auto-fit
  collapse, auto-max stretch, duplicate named-line occurrences, or independent
  lanes traversal owned by `FRI-08`;
- add missing alignment keywords or broader baseline distribution behavior owned
  by `FRI-09`;
- redesign positioned layout beyond projecting existing in-scope static and
  relative offsets through the correct axes; `FRI-10` owns positioned layout;
- claim that current block, flex, grid, nested-overflow, gutter, clipping,
  scroll-origin, or scroll-range-bound calculations are coherent; `FRI-05`
  owns those geometry corrections;
- compare browser `scroll_width` or `scroll_height`; `TEST-002` remains owned by
  `FRI-05`;
- own live scroll state, scrolling policy, host/CSSOM adaptation, retained
  identity, painting, or rendering;
- edit root adapters, root facade exports, root API artifacts, or sibling
  repositories; or
- add compatibility aliases for `Axis`, `ScrollOffset`, or `ScrollRange`.

## FRI-02.3 Standards And Current Evidence

### Normative Geometry Evidence

CSS Writing Modes Level 4 defines five `writing-mode` values and maps abstract
block/inline dimensions and sides from the used writing mode and direction:

- <https://www.w3.org/TR/css-writing-modes-4/#propdef-writing-mode>
- <https://www.w3.org/TR/css-writing-modes-4/#abstract-box>
- <https://www.w3.org/TR/css-writing-modes-4/#logical-to-physical>

The mapping is defined even for boxes with no line boxes. `block-start` depends
only on writing mode. `inline-start` depends on writing mode and used direction.
`sideways-lr` is not flow-equivalent to `vertical-lr`: for used LTR direction,
its inline progression is bottom-to-top.

CSS Flexbox Level 1 defines `row` against the current inline axis and `column`
against the current block axis:

- <https://www.w3.org/TR/css-flexbox-1/#flex-direction-property>

CSS Grid Level 2 defines columns in the inline axis and rows in the block axis:

- <https://www.w3.org/TR/css-grid-2/>

CSSOM View defines physical x/y scroll coordinates, with x increasing rightward
and y increasing downward, and signed clamp intervals for leftward or upward
overflow directions:

- <https://www.w3.org/TR/cssom-view/#scrolling-area>
- <https://www.w3.org/TR/cssom-view/#scroll-an-element>

### Source Evidence At The Published C03 Base

This table describes commit
`584f16231bed9c3e0475a4e64056fdc9e25dc2d3`. Completed substrate remains
normative initiative state; each row names only its remaining correction.

| Evidence ID | Current source fact | Remaining correction |
| --- | --- | --- |
| `E-MODE-1` | `WritingMode` has all five values and `FlowAxes` owns the complete ten-row mapping. | Migrate the remaining block, flex, and grid consumers without adding another mapping table. |
| `E-GEOM-1` | Public `PhysicalAxis` and crate-private logical geometry are present; temporary public `Point`/`Size` main/cross helpers and `FlexDirection` physical-axis helpers remain for the live flex consumer. | Remove those temporary helpers when flex migrates; C04 extends only shared generic logical projection needed by block. |
| `E-INLINE-1` | `src/inline.rs` consumes shared `FlowAxes`, `LogicalPointOf`, and `LogicalSizeOf`; its private writing-mode table is gone. | Preserve that single-owner mapping while block projects existing inline/control reports through containing flow. |
| `E-BLOCK-1` | `layout_in_flow_children` advances physical `cursor_y`; block auto size, margins, baselines, and child positions assume top-to-bottom physical y. Compute output still carries top/bottom-only collapse state. | Perform ordinary flow in logical coordinates, add typed physical collapse output, and project once through the containing block's flow axes. |
| `E-FLEX-1` | `FlexDirection`, geometry helpers, and flex-local selectors still encode row as physical x and column as physical y. | Derive a flex-local main/cross mapping from shared `FlowAxes`. |
| `E-GRID-1` | Grid-axis comparison delegates to `FlowAxes`, but expansion bases, intrinsic totals, child areas, and baseline groups still repeatedly bind columns to width/x and rows to height/y. | Keep column/row values logical through sizing and placement, then project through `FlowAxes`. |
| `E-SCROLL-1` | Signed physical and flow-relative offset/range types, validated construction, conversion, and flow-owned range projection are implemented. | Preserve this completed C02 contract while later algorithms consume it. |
| `E-PARITY-1` | The pinned runtime and schema-two report inventory are implemented; the fixture parser/generator/helper still reject or drop sideways modes, and existing vertical-flex XML still reaches text measurement rather than non-leaf flex layout. | Add each cycle's exact sideways-capable families/reports and the topology-checked non-leaf flex matrix. |
| `E-BASELINE-1` | `BaselinesOf` remains physical and its shared selection/synthesis methods are flow-aware; block, flex, and grid callers still contain algorithm-local physical-axis assumptions. | Migrate each algorithm caller through shared flow without changing the physical output representation. |

## FRI-02.4 Resolved Design Decisions

### `D-11` FlowAxes Owns The Mapping

`FlowAxes` is a concrete public resolved-layout value constructed from one
`WritingMode` and one used `Direction`. Every pair is valid, so construction is
infallible. It stores or can return both source values and owns:

- logical-inline to physical-axis mapping;
- logical-block to physical-axis mapping;
- inline-start, inline-end, block-start, and block-end physical sides;
- line-over and line-under physical sides;
- physical progression signs for both logical axes;
- public physical/flow-relative scroll-coordinate conversion;
- crate-private physical/logical algorithm-geometry projection; and
- physical edge selection by logical side.

`FlowAxes` is public because `ScrollGeometryOf` exposes it and root must be able
to consume layout's direction convention without reproducing the table. It is
not an authored CSS type and does not expose setters.

Rejected alternative: public helper methods on `WritingMode` alone cannot map
inline progression because used direction is also required.

Rejected alternative: algorithm-local mode matches preserve the present source
of disagreement among inline, flex, grid, and scroll.

Rejected alternative: an open trait or phantom coordinate-space framework adds
an extension contract where the domain is a closed five-mode table.

### `D-12` Public Geometry Is Physical; Algorithm Geometry Is Logical

Public `Point`, `Size`, `Edges`, `ScrollRectOf`, `NodeOutputOf`, and layout entry
sizes retain physical fields and physical CSS-pixel meaning. `Axis` is renamed
to `PhysicalAxis` so public errors and selectors cannot be mistaken for logical
axes. `PhysicalSide` names top, right, bottom, and left. `LogicalAxis` names
inline and block.

The algorithm phase uses crate-private `LogicalPointOf<S>`,
`LogicalSizeOf<S>`, `LogicalEdgesOf<T>`, and `LogicalRectOf<S>` with inline/block
fields. They are distinct structs, not type aliases. Conversion is named on
`FlowAxes`; `From` is not used because point/rectangle projection needs a
containing extent whenever an axis progresses from a physical end side.

Physical authored properties remain on their physical edges. Mapping changes
which physical values participate in a logical calculation; it never relabels
the property itself. Percentage margin and padding resolution uses the
containing block's logical inline extent in every writing mode.

`ComputeOutputOf` cannot continue exposing only `top_margin`, `bottom_margin`,
and an unqualified `margins_can_collapse_through`: those names and positions
cannot represent left/right block edges, and the boolean can be misapplied
across an orthogonal parent axis. It instead exposes one
`PhysicalBlockMarginCollapseOf<S>` value. The value has private storage and is
constructed from one `FlowAxes`, logical block-start and block-end margin sets,
and collapse-through eligibility. It stores the sets on the corresponding
physical sides and binds eligibility to that physical block axis. It permits
physical-side lookup and a containing-flow-aware collapse-through query but no
raw-edge constructor or mutation. A parent selects its own physical block-start
and block-end sides from this carrier; an opposing child therefore swaps the
relevant set and remains axis-compatible, while an orthogonal child's
descendant sets and collapse-through state cannot leak onto the parent's block
axis.

The public context-free `Edges::zip_inline_size` helper is removed. Its current
physical-width behavior is neither retained nor renamed. The single replacement
is a crate-private
`FlowAxes::zip_physical_edges_with_inline_extent(edges, containing_size, f)`
operation. It preserves the four physical edge positions, projects the physical
containing size through the receiver, and supplies that containing flow's one
logical inline extent as the basis for every edge. The receiver is always the
containing parent's `FlowAxes`, including when the child has a parallel,
opposing, or orthogonal flow.

Every internal `ComputeInputOf<S>` carries `containing_flow_axes: FlowAxes` and
the cache key includes it. The public `ComputeInputOf::leaf_layout` and
`leaf_content_size` constructors require that value alongside the containing
physical size, because standalone leaf measurement cannot infer its containing
flow from the measured node's own writing mode. The fields become private to the
owning module; crate code cannot create a context-free struct literal.

Tree-backed viewport-root and flex-item-root computation derive
`FlowAxes::new(root_input.writing_mode, root_input.direction)` before creating a
compute input. That value is both the root node's own flow and the initial
containing flow because the layout-ready root contract has no separate authored
initial-containing-block style. Every formatting algorithm passes its own
resolved `FlowAxes` as the containing flow for ordinary child requests.

The viewport-root path projects physical root availability into logical
inline/block availability before root auto sizing and percentage-edge
resolution. Root auto fill applies to logical inline size, not physical width.
With a definite viewport extent, the final root border box is anchored at the
mapped logical start/start corner; a reversed physical axis uses
`containing_extent - root_extent`. An intrinsic/min-content/max-content viewport
axis has no finite opposite edge and retains physical origin zero on that axis;
a required percentage basis remains missing through the FRI-01 typed resolution
contract. The flex-item-root context uses the same flow-aware sizing and basis
rules but preserves its existing physical-zero host placement because that
front door returns item sizing rather than viewport root anchoring.

### `D-13` Direction Means Used Inline Direction

Layout `Direction::{Ltr,Rtl}` represents the used inline direction supplied at
the layout-ready boundary. It is not the authored/computed CSS `direction`
token. Root style/text integration is responsible for effects such as
`text-orientation: upright` forcing used LTR before constructing layout input.

Layout neither stores `text-orientation` nor guesses used direction.

### `D-14` Sideways Modes Are Distinct States

`WritingMode` has exactly these variants:

```rust
pub enum WritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}
```

Sideways modes share a vertical inline axis and horizontal block axis, but
`SidewaysLr` has inverted line orientation and used-direction mapping relative
to `VerticalLr`. It is not normalized to a vertical variant.

### `D-15` Flex And Grid Derive Local Roles From FlowAxes

Flex owns a crate-private `FlexAxes` value derived from `FlowAxes`,
`FlexDirection`, and `FlexWrap`:

- row selects logical inline as main;
- column selects logical block as main;
- `*-reverse` reverses main progression;
- `wrap-reverse` reverses cross progression; and
- physical edge, axis, size, point, margin, inset, and baseline access always
  goes through the derived mapping.

Grid owns only the closed conversion
`GridAxisKind::{Column,Row} -> LogicalAxis::{Inline,Block}`. Track totals,
offsets, areas, and gaps remain logical until projection. Grid does not define a
second writing-mode table.

### `D-16` Baseline Points Stay Physical And Become Flow-Aware

`BaselinesOf<S>` remains a physical pair of optional x/y intersection points.
This preserves output and cross-format composition. Context-free block-baseline
helpers are replaced by methods that receive `FlowAxes` and select the physical
block axis. Synthesized first/last baselines use `line_over`/`line_under`, so
`SidewaysLr` does not inherit the vertical-rl synthesis convention.

This initiative maps existing baseline behavior. It does not add the missing
alignment values or distribution semantics owned by `FRI-09`.

### `D-17` Scroll Geometry Uses Signed Physical Ranges

Layout output and CSSOM-facing geometry are physical. Unqualified
`ScrollOffsetOf` and `ScrollRangeOf` are removed. The public replacement is:

- `PhysicalScrollOffsetOf<S>`: finite physical x/y offset;
- `PhysicalScrollAxisRangeOf<S>`: finite closed `[minimum, maximum]` interval;
- `PhysicalScrollRangeOf<S>`: physical x/y axis ranges;
- `FlowRelativeScrollOffsetOf<S>`: finite inline/block offset;
- `FlowRelativeScrollAxisRangeOf<S>`: finite logical-axis interval; and
- `FlowRelativeScrollRangeOf<S>`: inline/block ranges.

`FlowAxes` owns total named conversion between validated physical and
flow-relative offsets/ranges. Reversed projection swaps and negates interval
endpoints so ordered intervals remain ordered.

`ScrollGeometryOf<S>` stores `FlowAxes` and a `PhysicalScrollRangeOf<S>`.
Existing physical rectangles remain physical. Root owns any current retained
scroll offset, host event conversion, and CSSOM policy.

The range types permit arbitrary signed finite bounds with `minimum <= maximum`.
They do not require zero to be either endpoint because later coherent geometry
may have a nonzero origin or alignment overflow. `FRI-05` owns computation of
the actual bounds; `FRI-02` owns their unambiguous coordinate model.

Rejected alternative: physical-only ranges without flow conversion force every
consumer to reinterpret RTL and vertical modes.

Rejected alternative: logical-only ranges hide the physical boundary used by
layout output, rounding, browser comparison, and host adapters.

Rejected alternative: an anchor-bearing live scroll state object belongs to
root runtime state, not this layout calculation crate.

### `D-18` No Transitional Axis APIs Remain

The following context-free or ambiguous public contracts are removed rather
than deprecated:

- `Axis`;
- `ScrollOffset` / `ScrollOffsetOf`;
- `ScrollRange` / `ScrollRangeOf`;
- `ScrollUnsupportedFeature::InvalidScrollRange`;
- `FlexDirection::main_axis` and `cross_axis`;
- `Edges::zip_inline_size`;
- `Point::main` / `cross`;
- `Size::main` / `cross` / `from_cross`; and
- `Edges::main_sum` / `cross_sum`; and
- `ComputeOutputOf::top_margin` / `bottom_margin` /
  `margins_can_collapse_through`.

Crate-local duplicates such as `InlineAxisMapping`, physical flex edge traits,
and grid writing-mode match tables are deleted after their consumers migrate.

## FRI-02.5 Normative Flow Mapping

All coordinates are local CSS layout coordinates. Horizontal x increases to the
right; vertical y increases downward.

| Writing mode | Used direction | Inline axis | Inline start -> end | Block axis | Block start -> end | Line over -> under |
| --- | --- | --- | --- | --- | --- | --- |
| `HorizontalTb` | `Ltr` | Horizontal | Left -> Right | Vertical | Top -> Bottom | Top -> Bottom |
| `HorizontalTb` | `Rtl` | Horizontal | Right -> Left | Vertical | Top -> Bottom | Top -> Bottom |
| `VerticalRl` | `Ltr` | Vertical | Top -> Bottom | Horizontal | Right -> Left | Right -> Left |
| `VerticalRl` | `Rtl` | Vertical | Bottom -> Top | Horizontal | Right -> Left | Right -> Left |
| `VerticalLr` | `Ltr` | Vertical | Top -> Bottom | Horizontal | Left -> Right | Right -> Left |
| `VerticalLr` | `Rtl` | Vertical | Bottom -> Top | Horizontal | Left -> Right | Right -> Left |
| `SidewaysRl` | `Ltr` | Vertical | Top -> Bottom | Horizontal | Right -> Left | Right -> Left |
| `SidewaysRl` | `Rtl` | Vertical | Bottom -> Top | Horizontal | Right -> Left | Right -> Left |
| `SidewaysLr` | `Ltr` | Vertical | Bottom -> Top | Horizontal | Left -> Right | Left -> Right |
| `SidewaysLr` | `Rtl` | Vertical | Top -> Bottom | Horizontal | Left -> Right | Left -> Right |

Required invariants:

1. inline and block axes are always perpendicular;
2. each start side is on its declared axis and each end is its opposite;
3. line-over and line-under are opposites on the block axis;
4. physical-to-logical and logical-to-physical size conversions are inverse;
5. offset/range conversions are inverse for all finite validated values;
6. point and rectangle projection returns a physical box wholly described by
   the same containing physical extent;
7. direction never changes block-start or block-end; and
8. `SidewaysLr` used LTR maps inline-start to physical bottom.

## FRI-02.6 Public API And Type Contract

### Public Geometry Surface

The public front door exposes:

```rust
pub enum PhysicalAxis { Horizontal, Vertical }
pub enum LogicalAxis { Inline, Block }
pub enum PhysicalSide { Top, Right, Bottom, Left }

pub struct FlowAxes { /* private fields */ }

impl FlowAxes {
    pub const fn new(writing_mode: WritingMode, direction: Direction) -> Self;
    pub const fn writing_mode(self) -> WritingMode;
    pub const fn direction(self) -> Direction;
    pub const fn inline_axis(self) -> PhysicalAxis;
    pub const fn block_axis(self) -> PhysicalAxis;
    pub const fn inline_start(self) -> PhysicalSide;
    pub const fn inline_end(self) -> PhysicalSide;
    pub const fn block_start(self) -> PhysicalSide;
    pub const fn block_end(self) -> PhysicalSide;
    pub const fn line_over(self) -> PhysicalSide;
    pub const fn line_under(self) -> PhysicalSide;
}
```

`PhysicalAxis`, `LogicalAxis`, `PhysicalSide`, `WritingMode`, `Direction`, and
`FlowAxes` are closed copyable values with `Debug`, `Eq`, and `PartialEq`.
`WritingMode::default()` remains `HorizontalTb`, `Direction::default()` remains
`Ltr`, and `NodeInputOf::default()` therefore remains a horizontal-tb LTR
layout-ready node. `PhysicalAxis`, `LogicalAxis`, `PhysicalSide`, and `FlowAxes`
have no `Default`: each requires an explicit semantic choice or resolved pair,
not an optional post-construction step. Named default-construction tests prove
the retained `WritingMode`, `Direction`, and `NodeInputOf` values in both scalar
lanes. Compile-fail rustdoc examples and public-surface searches prove the axis,
side, and `FlowAxes` types do not implement `Default` and expose no horizontal
fallback constructor.

The accessor and public conversion names in this section are normative. All
operations above are owned by this one concrete type. A crate-private helper
that projects a point/rectangle with a reversed axis receives the containing
physical size explicitly.

The public conversion methods operate only on public coordinate-space-specific
scroll types:

```rust
impl FlowAxes {
    pub fn physical_scroll_offset<S: LayoutScalar>(
        self,
        offset: FlowRelativeScrollOffsetOf<S>,
    ) -> PhysicalScrollOffsetOf<S>;

    pub fn flow_relative_scroll_offset<S: LayoutScalar>(
        self,
        offset: PhysicalScrollOffsetOf<S>,
    ) -> FlowRelativeScrollOffsetOf<S>;

    pub fn physical_scroll_range<S: LayoutScalar>(
        self,
        range: FlowRelativeScrollRangeOf<S>,
    ) -> PhysicalScrollRangeOf<S>;

    pub fn flow_relative_scroll_range<S: LayoutScalar>(
        self,
        range: PhysicalScrollRangeOf<S>,
    ) -> FlowRelativeScrollRangeOf<S>;
}
```

Projection methods that accept or return crate-private logical algorithm
geometry are themselves crate-private. No public method mentions
`LogicalPointOf`, `LogicalSizeOf`, `LogicalEdgesOf`, or `LogicalRectOf`.

`PhysicalAxis` replaces `Axis` in public diagnostic types such as
`RootAvailabilityErrorOf` and measurement-output errors.

### Public Collapsible-Margin Output

The physical compute-output carrier is:

```rust
pub struct PhysicalBlockMarginCollapseOf<S: LayoutScalar = DefaultScalar> {
    /* private physical-edge, block-axis, and through-eligibility storage */
}

impl<S: LayoutScalar> PhysicalBlockMarginCollapseOf<S> {
    pub const NONE: Self;
    pub const fn from_block_flow(
        flow_axes: FlowAxes,
        block_start: CollapsibleMarginOf<S>,
        block_end: CollapsibleMarginOf<S>,
        can_collapse_through: bool,
    ) -> Self;
    pub const fn at(self, side: PhysicalSide) -> CollapsibleMarginOf<S>;
    pub const fn can_collapse_through(self, containing_flow: FlowAxes) -> bool;
}
```

`PhysicalBlockMarginCollapse` is the default-scalar alias. `ComputeOutputOf`
replaces its public `top_margin`, `bottom_margin`, and
`margins_can_collapse_through` fields with
`block_margin_collapse: PhysicalBlockMarginCollapseOf<S>`. Constructors that
do not report block collapse produce `NONE`. `can_collapse_through` returns true
only when the stored eligibility is true and the supplied containing flow has
the stored physical block axis; parallel and opposing flows are compatible,
orthogonal flows are not. No alias, deprecated field-equivalent accessor, raw
`Edges` constructor, unqualified through query, or compatibility conversion
survives.

The direct measured-leaf compute path constructs this value from the leaf
style's own `FlowAxes`, zero block-start and block-end sets, and a logical
empty-leaf predicate. The predicate requires both the measured content extent
and final resolved outer extent on the leaf's own logical block axis to be zero,
in addition to the existing edge/formatting-context exclusions; it never tests
an unconditional physical height. The carrier never binds eligibility to the
containing flow. This preserves valid parallel and opposing empty-leaf collapse
while making the same leaf in an orthogonal parent ineligible through the
contextual query. Non-reporting output constructors retain `NONE`.

### Public Containing-Flow Context

The two public direct-leaf input constructors become:

```rust
impl<S: LayoutScalar> ComputeInputOf<S> {
    pub fn leaf_layout(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        containing_flow_axes: FlowAxes,
        available: Size<AvailableOf<S>>,
    ) -> Result<Self, RootAvailabilityErrorOf<S>>;

    pub fn leaf_content_size(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        containing_flow_axes: FlowAxes,
        available: Size<AvailableOf<S>>,
    ) -> Result<Self, RootAvailabilityErrorOf<S>>;
}
```

`ComputeInputOf` exposes a read-only `containing_flow_axes()` accessor for direct
leaf callers and staged cache records. The recursive modes and field mutation
remain crate-private. No overload infers the context from the leaf style, and no
horizontal default constructor remains.

The owning module provides these finite private construction paths:

- `root_layout(known, parent, containing_flow_axes, available)` fixes root run
  mode, inherent sizing, and both requested axes;
- `flex_item_root(parent, containing_flow_axes, available)` fixes the same mode
  with no known size;
- `for_child(run_mode, sizing_mode, requested_axis, known, parent,
  containing_flow_axes, available)` is the one recursive algorithm constructor;
  and
- `hidden(containing_flow_axes)` replaces `ComputeInputOf::HIDDEN` and fixes all
  other fields to hidden-layout values.

`compute_hidden` receives the caller's containing `FlowAxes`; hidden descendants
propagate it unchanged because a hidden node establishes no formatting context
and resolves no edge. Hidden traversal clears output/cache state and performs no
cache lookup or store. Equality still includes the explicit flow, so test fakes
cannot accept a context-free hidden request accidentally.

### Public Scroll Surface

Each scroll scalar type has private fields and default-scalar aliases. The
constructors are fallible because external/root input can be non-finite or
inverted.

```rust
pub enum ScrollCoordinateErrorOf<S> {
    NonFinitePhysicalOffset { axis: PhysicalAxis, value: S },
    NonFiniteFlowRelativeOffset { axis: LogicalAxis, value: S },
    NonFinitePhysicalRangeMinimum { axis: PhysicalAxis, value: S },
    NonFinitePhysicalRangeMaximum { axis: PhysicalAxis, value: S },
    NonFiniteFlowRelativeRangeMinimum { axis: LogicalAxis, value: S },
    NonFiniteFlowRelativeRangeMaximum { axis: LogicalAxis, value: S },
    InvertedPhysicalRange {
        axis: PhysicalAxis,
        minimum: S,
        maximum: S,
    },
    InvertedFlowRelativeRange {
        axis: LogicalAxis,
        minimum: S,
        maximum: S,
    },
}
```

Callers can distinguish coordinate space, axis, failing endpoint/value, and
inverted bounds without parsing prose. The enum is exhaustive for this
pre-release contract; future requirements may make a breaking correction rather
than weakening the current model for compatibility.

The public constructors and accessors are:

```rust
impl<S: LayoutScalar> PhysicalScrollOffsetOf<S> {
    pub fn try_new(x: S, y: S) -> Result<Self, ScrollCoordinateErrorOf<S>>;
    pub fn x(self) -> S;
    pub fn y(self) -> S;
}

impl<S: LayoutScalar> FlowRelativeScrollOffsetOf<S> {
    pub fn try_new(inline: S, block: S) -> Result<Self, ScrollCoordinateErrorOf<S>>;
    pub fn inline(self) -> S;
    pub fn block(self) -> S;
}

impl<S: LayoutScalar> PhysicalScrollRangeOf<S> {
    pub fn try_new(
        x_minimum: S,
        x_maximum: S,
        y_minimum: S,
        y_maximum: S,
    ) -> Result<Self, ScrollCoordinateErrorOf<S>>;
    pub fn x(self) -> PhysicalScrollAxisRangeOf<S>;
    pub fn y(self) -> PhysicalScrollAxisRangeOf<S>;
    pub fn clamp(self, offset: PhysicalScrollOffsetOf<S>) -> PhysicalScrollOffsetOf<S>;
}

impl<S: LayoutScalar> FlowRelativeScrollRangeOf<S> {
    pub fn try_new(
        inline_minimum: S,
        inline_maximum: S,
        block_minimum: S,
        block_maximum: S,
    ) -> Result<Self, ScrollCoordinateErrorOf<S>>;
    pub fn inline(self) -> FlowRelativeScrollAxisRangeOf<S>;
    pub fn block(self) -> FlowRelativeScrollAxisRangeOf<S>;
    pub fn clamp(
        self,
        offset: FlowRelativeScrollOffsetOf<S>,
    ) -> FlowRelativeScrollOffsetOf<S>;
}

impl<S: LayoutScalar> PhysicalScrollAxisRangeOf<S> {
    pub fn minimum(self) -> S;
    pub fn maximum(self) -> S;
}

impl<S: LayoutScalar> FlowRelativeScrollAxisRangeOf<S> {
    pub fn minimum(self) -> S;
    pub fn maximum(self) -> S;
}
```

Axis-range values are created only by their enclosing validated range or by a
total `FlowAxes` conversion. This prevents an unlabelled one-dimensional range
from being constructed and later assigned to the wrong coordinate axis.

The six scalar-bearing scroll value types and `ScrollCoordinateErrorOf<S>` are
copyable `Debug`/`PartialEq` values and have default-scalar aliases named by
removing `Of`. They do not implement `Eq`, ordering, or `Default`. A zero offset
or zero range is requested explicitly through `try_new`, so no required
coordinate-space choice is hidden in default construction.

Constructors and conversions:

- reject NaN and infinity;
- preserve finite negative values;
- reject `minimum > maximum`;
- canonicalize signed zero consistently with other layout numeric types;
- preserve `f32`/`f64` without narrowing;
- never panic; and
- return a valid range/offset or one typed error.

Clamping is component-wise against each closed interval and is total for a
validated offset and range. For an axis interval `[minimum, maximum]`, values
below the minimum become the minimum, values above the maximum become the
maximum, and interior values are unchanged.

`ScrollGeometryOf::flow_axes()` and `physical_range()` expose the resolved
contract. Callers obtain writing mode and direction from the returned
`FlowAxes`; `ScrollGeometryOf` has no duplicate fields or forwarding accessors
for them.

`ScrollGeometryOf::new` receives one `FlowAxes` and one
`PhysicalScrollRangeOf<S>` rather than separate writing-mode/direction fields or
an unqualified range. Existing container, scrollport, overflow-clip,
scrollable-overflow, and gutter arguments remain physical. A container axis
that does not expose a scroll range accepts exactly `[0, 0]`; a scrollable axis
accepts any validated signed interval. Existing geometry-coherence validation
continues to return its current typed layout-owned error until `FRI-05` replaces
that broader contract.

### Compatibility Classification

This initiative is intentionally breaking:

- two variants are added to an exhaustive public enum;
- ambiguous public types are renamed/removed;
- invalid scroll-coordinate construction leaves `ScrollUnsupportedFeature` and
  uses `ScrollCoordinateErrorOf<S>`;
- context-free physical flex helpers are removed; and
- `ScrollGeometryOf::range()` changes to a signed physical range contract.

No alias, blanket conversion, or duplicate old constructor is retained.

## FRI-02.7 Algorithm-Phase Logical Geometry

Crate-private logical types use `inline` and `block` fields. They support only
the operations required by algorithms: map, zip, add/subtract where scalar
semantics allow it, axis selection, and explicit projection through
`FlowAxes`. They do not implement public serialization or cross-crate traits.

Logical geometry is used for:

- containing block inline/block available space;
- logical margin/padding/border/inset access;
- block cursors and collapsed block margins;
- flex main/cross sizes and offsets;
- grid column/row totals and track offsets;
- existing inline-control run positions;
- baseline-axis selection; and
- flow-relative scroll offsets/ranges.

Physical geometry is used for:

- public input/output fields;
- physical authored edges after value resolution;
- final node locations and sizes;
- physical overflow and clip rectangles;
- rounded output; and
- browser XML x/y/width/height.

Algorithms convert at named boundaries. They do not transpose a raw `Size` or
`Point` based on `is_vertical` and then continue treating it as physical.

## FRI-02.8 Block Behavior Contract

### Parent And Child Flow Ownership

The containing block's `FlowAxes` owns positioning-phase logical sides,
physical placement, auto margins, and margin collapse. A child's own
`FlowAxes` owns its sizing phase. Parallel and orthogonal child flows therefore
remain distinct rather than inheriting a physical width/height assumption.

Each block compute result constructs `PhysicalBlockMarginCollapseOf` from that
block's own flow. Its parent reads only the carrier entries at the parent's
physical block-start and block-end sides and asks through the parent's flow.
Parallel and opposing flows may collapse through; orthogonal flows may not.
Direct authored child margins remain physical and are selected by the parent
flow before collapse.

Measured leaf results follow the same ownership rule: their carrier records the
leaf's own physical block axis even though both descendant edge sets are zero.
The containing parent flow is used only when the parent queries that result.

For an orthogonal child:

- parent physical containing dimensions are projected into the child's logical
  available dimensions for child sizing;
- the parent flow still selects which physical child margins participate in
  parent block collapse and inline auto-margin resolution;
- the child returns physical output;
- parent placement projects the child's physical size into the parent's logical
  axes; and
- missing definite context follows existing typed availability/result behavior,
  never a guessed width or panic.

This initiative does not implement at-risk automatic multicol behavior from
Writing Modes Level 4. Browser-observable block sizing and positioning remain
the conformance target.

### Ordinary In-Flow Behavior Matrix

| Behavior | Required logical rule | Physical output requirement |
| --- | --- | --- |
| Child stacking | Advance one block cursor from block-start to block-end. | Horizontal modes stack y; `*-rl` modes stack from right to left; `*-lr` modes stack left to right. |
| Inline stretch | Resolve auto inline size within containing inline extent. | Width in horizontal mode; height in vertical/sideways modes. |
| Auto block size | Use in-flow block extent plus block edges. | Height in horizontal mode; width in vertical/sideways modes. |
| Percentage edges | Resolve every margin/padding percentage against containing inline extent. | Basis is physical width only in horizontal mode. |
| Margin collapse | Collapse only parent block-start/block-end physical margins. | Top/bottom in horizontal; right/left in `*-rl`; left/right in `*-lr`. |
| Inline alignment | Apply direction to logical inline start/end. | RTL affects x in horizontal and y in vertical/sideways modes. |
| Content extent | Accumulate logical extents, then project. | Public content size and overflow rectangles remain physical. |
| Baselines | Select/synthesize on mapped block axis and line-over side. | Baseline points remain physical x/y. |

The finding's named regression is normative: a `100x100` `vertical-rl`
container with two `20x10` ordinary block children places them at `(80,0)` and
`(60,0)`. The corresponding `vertical-lr` positions are `(0,0)` and `(20,0)`.
Sideways block progression matches its `-rl`/`-lr` block direction while using
the sideways inline direction table.

### Existing Adjacent Paths

Existing atomic-inline/control runs consume shared `FlowAxes` for projection so
they support both sideways values without another table. Full soft wrapping,
float exclusion, and vertical clear remain `FRI-06`.

Existing relative offsets and absolute static-position fallback use the mapped
logical sides where they are touched by ordinary block flow. This is axis
migration only; `FRI-10` still owns positioned-layout correctness.

## FRI-02.9 Flex Behavior Contract

### FlexAxes

`FlexAxes` is crate-private and complete for one container. It records:

- main and cross `LogicalAxis`;
- physical main/cross axes;
- physical main-start/end and cross-start/end sides;
- main reversal from `row-reverse`/`column-reverse`;
- cross reversal from `wrap-reverse`; and
- flow direction used by alignment and placement.

It is the only flex edge/axis selector. Existing flex-local `EdgeAxisExt`,
`BoolEdgeAxisExt`, `OptionEdgeAxisExt`, and context-free geometry main/cross
methods are removed.

### Required Flex Consumers

The mapped axes apply to:

- container inner, min/max, available, and known main/cross sizes;
- flex-basis and intrinsic contribution measurement;
- percentage margin/padding resolution against containing inline extent;
- gap selection;
- line collection, wrapping, reverse order, and wrap-reverse;
- flex grow/shrink target dimensions without changing FRI-07 equations;
- auto-margin edge selection without changing the FRI-07 negative-free-space
  rule;
- justify/align/content/self placement;
- baseline coordinate selection and synthesis;
- relative offsets;
- existing absolute/static-position projection without changing the FRI-07
  auto-margin equation;
- content extent and physical node output; and
- scrollbar/output projection where the current algorithm already emits it.

The finding's named regression is normative: a `100x100` `vertical-lr` row
container with two `10x20` items places them at `(0,0)` and `(0,20)`, not
`(0,0)` and `(10,0)`.

### Flex Direction Matrix

| Flex direction | Logical main axis | Main progression |
| --- | --- | --- |
| `Row` | Inline | Inline start -> inline end |
| `RowReverse` | Inline | Inline end -> inline start |
| `Column` | Block | Block start -> block end |
| `ColumnReverse` | Block | Block end -> block start |

Cross progression is the other logical axis and is reversed only by
`wrap-reverse`. `Direction` affects row progression through `FlowAxes`; it does
not change block progression for column.

## FRI-02.10 Grid, Subgrid, And Lanes Behavior Contract

### Logical Grid Axes

Grid columns are always logical inline tracks. Grid rows are always logical
block tracks. `GridAxisKind` maps to `LogicalAxis`; it never maps directly to
width/height or x/y.

The logical contract applies to:

- explicit and implicit track expansion bases;
- column/row gaps;
- intrinsic min/max and available-space inputs;
- percentage and flexible track reruns;
- intrinsic track totals;
- final container size;
- grid-area size and origin;
- item alignment and physical area projection;
- absolute grid area projection;
- baseline grouping/application on both mapped physical axes;
- subgrid parent/child axis inheritance, including parallel, opposing, and
  orthogonal flows;
- grid-lanes measurement, track totals, item areas, and final placement; and
- final physical output/content extents where currently owned.

Grid-area logical extents use a distinct crate-private representation or
`LogicalSizeOf`; raw `Size { width, height }` cannot silently carry
column/row totals.

### Required Grid Results

For unequal logical totals `inline=70`, `block=110`:

- `horizontal-tb` physical size is `70x110`;
- every vertical and sideways writing mode physical size is `110x70`.

The existing `vertical-rl` test that expects `70x110` changes to `110x70` while
retaining correct child area projection.

Existing baseline groups are applied along the mapped block axis. The initiative
does not add missing alignment keywords or solve other FRI-08 track/placement
findings.

### Grid Finding Boundary

The following remain outside `FRI-02`: `GRID-001`, `GRID-002`, `GRID-003`,
`GRID-005`, `GRID-006`, `GRID-007`, `GRID-008`, and `GRID-010`. Axis migration
must not lock their current incorrect result into a new abstraction. Focused
tests isolate writing-mode projection from those known defects.

## FRI-02.11 Scroll Coordinate Contract

### Coordinate Meaning

`PhysicalScrollOffsetOf` and `PhysicalScrollRangeOf` use physical local x/y CSS
layout coordinates:

- positive x is rightward;
- positive y is downward;
- rightward/downward ranges commonly use `[0, extent]`;
- leftward/upward ranges commonly use `[-extent, 0]`; and
- arbitrary signed valid bounds remain representable.

`FlowRelativeScrollOffsetOf` and `FlowRelativeScrollRangeOf` use inline/block
coordinates whose positive progression is logical start to logical end.

For a logical interval `[0, extent]`, conversion follows:

| Physical progression | Physical interval |
| --- | --- |
| Left -> Right | `[0, extent]` on x |
| Right -> Left | `[-extent, 0]` on x |
| Top -> Bottom | `[0, extent]` on y |
| Bottom -> Top | `[-extent, 0]` on y |

`FlowAxes` performs the axis swap and sign reversal. Conversion round trips
exactly for finite `f32` and `f64` values except the established signed-zero
canonicalization.

### Layout-Produced Range Pipeline

Production layout never constructs a `PhysicalScrollRangeOf<S>` directly from
physical overflow extents. The current extent calculation first computes the
same non-negative physical x/y overflow magnitudes it owns today, honoring
`ScrollContainerFacts` exposure but not claiming FRI-05 origin/coherence work.
`FlowAxes` maps that physical magnitude size to logical inline/block magnitudes.
Layout then constructs one `FlowRelativeScrollRangeOf<S>` with `[0, extent]` on
each exposed logical axis and `[0, 0]` on each unexposed axis, and obtains the
stored `PhysicalScrollRangeOf<S>` only through
`FlowAxes::physical_scroll_range`.

The one crate-private
`physical_scroll_range_from_overflow_rects(flow_axes, container, scrollport,
scrollable_overflow)` helper owns that pipeline. Both ordinary
`scroll_geometry_from_layout` and `round_scroll_geometry` call it. Rounding
recomputes the flow-relative magnitude from the rounded physical rectangles and
projects it through the geometry's stored `FlowAxes`; it never rebuilds a bare
physical `[0, extent]` range.

Consequently every reversed logical progression produces a signed physical
interval even before FRI-05 improves the magnitude: horizontal RTL produces a
negative x interval, `vertical-rl`/`sideways-rl` produce a negative block-axis x
interval, vertical RTL produces a negative inline-axis y interval, and
`sideways-lr` LTR produces a negative inline-axis y interval. Named normal and
rounded geometry tests cover every reversed axis across all 10 flow mappings in
both scalar lanes. They assert the signed endpoint and the non-reversed opposite
axis independently.

### Geometry Boundary

`FRI-02` replaces the ambiguous model and migrates current constructors,
rounding, cache storage, output, and tests to it. It does not assert that the
current algorithm computes every bound correctly. In particular:

- negative-origin contribution;
- nested scroll contribution;
- flex/grid scroll geometry;
- gutter/scrollport/range coherence;
- mixed-axis overflow coupling;
- initial scroll position under alignment overflow; and
- browser scroll extent comparison

remain `FRI-05` work.

No current offset is stored in `ScrollGeometryOf`. Layout outputs geometry;
root runtime owns live state.

## FRI-02.12 Errors And Failure Semantics

`FlowAxes` construction cannot fail. Every `WritingMode`/`Direction` pair is a
valid layout-ready state.

Scroll coordinate construction returns typed errors for:

- non-finite physical offset components;
- non-finite flow-relative offset components;
- non-finite range bounds; and
- inverted range bounds.

Algorithm projection that receives valid logical/physical geometry is total.
Any later algorithm-specific numeric overflow is returned through the FRI-01
`LayoutErrorOf` envelope with site and operation; it is not saturated, replaced
with zero, or panicked.

Within FRI-02-owned ordinary block, flex, grid, lanes, subgrid, scroll, and
non-clearing existing inline-control axis paths, no represented writing mode may
reach a wildcard fallback, zero-size placeholder, `unreachable!`, `todo!`, or
panic. A vertical `LineBreakInputOf` with `Clear` other than `None` remains the
explicit `BLOCK-014`/FRI-06 capability finding; FRI-02 neither exercises it as
owned acceptance nor weakens its existing evidence.

## FRI-02.13 Browser Fixture And Oracle Contract

### Parser And Generator

The layout-owned XML parser accepts all five exact writing-mode strings. The
generator records the browser's computed writing mode without normalizing
sideways modes to vertical modes. Unknown values remain an explicit fixture
error.

Generated XML continues to store physical x/y/width/height and source
provenance. Generator output is never hand-edited.

### Ordinary Block Families

Add five HTML families:

- `block_axes_horizontal_tb`;
- `block_axes_vertical_rl`;
- `block_axes_vertical_lr`;
- `block_axes_sideways_rl`; and
- `block_axes_sideways_lr`.

Each contains at least two ordinary block children with unequal physical sizes
and an inline-start-sensitive margin/alignment. The existing generator variants
cover LTR/RTL and content-box/border-box. A named non-ignored integration test
requires all four variants of all five families and compares them through
`compute_layout`.

### Non-Leaf Flex Families

Add the 20 family cross product:

```text
flex_axes_<horizontal_tb|vertical_rl|vertical_lr|sideways_rl|sideways_lr>_
          <row|row_reverse|column|column_reverse>
```

Every flex container has at least two element children and no direct text-only
shape that can lower the container as a measured leaf. Fixture support includes
a topology assertion that the target node is a non-leaf flex container before
comparison. The generator's four body variants produce 80 XML cases.

A named non-ignored integration test requires the exact path set and runs all
80 through `compute_layout`. This closes `TEST-005`; the older 12 text-leaf
vertical cases remain valid text-measurement coverage but are not counted as
flex-axis evidence.

### Grid Families

Add these exact nine ordinary-grid intrinsic-axis families, each with unequal
column and row totals and at least one in-flow item whose position exposes the
mapping:

- `grid/grid_axes_horizontal_tb_parallel`;
- `grid/grid_axes_vertical_rl_parallel`;
- `grid/grid_axes_vertical_lr_parallel`;
- `grid/grid_axes_sideways_rl_parallel`;
- `grid/grid_axes_sideways_lr_parallel`;
- `grid/grid_axes_vertical_opposing`;
- `grid/grid_axes_sideways_opposing`;
- `grid/grid_axes_horizontal_parent_orthogonal_child`; and
- `grid/grid_axes_vertical_parent_orthogonal_child`.

The five parallel families use the named mode on container and item. Opposing
families pair `vertical-rl` with `vertical-lr` and `sideways-rl` with
`sideways-lr`. Orthogonal families cover both
horizontal-parent/vertical-child and vertical-parent/horizontal-child sizing
and placement. They use explicit tracks and definite non-overlapping placement
so they assert axis projection without depending on FRI-08 demand or placement
corrections.

Add these exact nine grid-lanes families:

- `grid-lanes/grid_lanes_axes_horizontal_tb_parallel`;
- `grid-lanes/grid_lanes_axes_vertical_rl_parallel`;
- `grid-lanes/grid_lanes_axes_vertical_lr_parallel`;
- `grid-lanes/grid_lanes_axes_sideways_rl_parallel`;
- `grid-lanes/grid_lanes_axes_sideways_lr_parallel`;
- `grid-lanes/grid_lanes_axes_vertical_opposing`;
- `grid-lanes/grid_lanes_axes_sideways_opposing`;
- `grid-lanes/grid_lanes_axes_horizontal_parent_orthogonal_child`; and
- `grid-lanes/grid_lanes_axes_vertical_parent_orthogonal_child`.

Each grid-lanes family contains one columns-lanes and one rows-lanes container
with unequal logical track totals. Parallel families use the named mode on the
container and items. The opposing family uses `vertical-rl` on the containing
flow and `vertical-lr` on the relevant item flow; the sideways opposing family
pairs `sideways-rl` and `sideways-lr`. The two orthogonal families exercise both
horizontal-parent/vertical-child and vertical-parent/horizontal-child sizing
and placement.

Add these exact nine subgrid families:

- `subgrid/subgrid_axes_horizontal_tb_parallel`;
- `subgrid/subgrid_axes_vertical_rl_parallel`;
- `subgrid/subgrid_axes_vertical_lr_parallel`;
- `subgrid/subgrid_axes_sideways_rl_parallel`;
- `subgrid/subgrid_axes_sideways_lr_parallel`;
- `subgrid/subgrid_axes_vertical_opposing`;
- `subgrid/subgrid_axes_sideways_opposing`;
- `subgrid/subgrid_axes_horizontal_parent_orthogonal_child`; and
- `subgrid/subgrid_axes_vertical_parent_orthogonal_child`.

Each subgrid family contains one columns-subgrid and one rows-subgrid case with
unequal inherited tracks and an item whose physical position and size expose an
axis swap or progression reversal. Opposing families pair `vertical-rl` with
`vertical-lr` and `sideways-rl` with `sideways-lr`. Orthogonal families cover
both parent/child orientations. These cases assert only axis mapping,
inheritance, and projection; they do not encode the FRI-08 placement, demand,
track-sizing, auto-fit, named-line, or traversal defects as expected behavior.

LTR/RTL and both box-sizing variants are generated for every family. The exact
inventories therefore contain 36 ordinary-grid, 36 grid-lanes, and 36 subgrid
XML files. Every HTML family is an active `surgeist` case in `corpus.toml`, and
the report contract below accounts for all FRI-02 files without hiding unrelated
corpus classifications.

### FRI-02 Generation Reports

`corpus.toml` retains the schema version 2 implemented by C03 and remains the
sole fixture-phase owner of the browser pin, stable launch profile, and report
inventory. C04-C07 change only scoped inventory/counts, generated artifacts,
and the C07 correction that makes browser ownership, job/cleanup deadlines, and
sealed artifact publication total; C08 performs the final inventory cleanup.
The retained schema is:

```toml
schema_version = 2

[browser]
source = "chrome-for-testing"
version = "149.0.7827.115"
version_output = "Google Chrome for Testing 149.0.7827.115"
cache_root = "target/surgeist-browser"
provenance_format = "chrome-for-testing/{version} ({repository_relative_executable})"

[browser.launch]
batch_size = 50
job_timeout_ms = 10000
dom_poll_interval_ms = 25
retry_count = 1
job_order = "sorted-sequential"
retry_error_class = "browser-job-fault"
profile_scope = "per-batch-and-retry"
page_scope = "per-job"
disable_default_args = true
disable_cache = true
arguments = [
  "headless=new",
  "mute-audio",
  "disable-background-networking",
  "disable-background-timer-throttling",
  "disable-backgrounding-occluded-windows",
  "disable-breakpad",
  "disable-client-side-phishing-detection",
  "disable-component-extensions-with-background-pages",
  "disable-component-update",
  "disable-default-apps",
  "disable-dev-shm-usage",
  "disable-domain-reliability",
  "disable-features=TranslateUI,MediaRouter,OptimizationHints,AutofillServerCommunication",
  "disable-hang-monitor",
  "disable-ipc-flooding-protection",
  "disable-popup-blocking",
  "disable-prompt-on-repost",
  "disable-renderer-backgrounding",
  "disable-sync",
  "enable-automation",
  "enable-blink-features=IdleDetection,CSSGridLanesLayout",
  "enable-features=NetworkService,NetworkServiceInProcess",
  "force-color-profile=srgb",
  "lang=en_US",
  "metrics-recording-only",
  "no-default-browser-check",
  "no-first-run",
  "use-mock-keychain",
]

[generation_reports.full]
file = "all.json"
generated = 5256
unsupported = 356
expected_fail = 0
quarantined = 0
failed_to_generate = 0

[[generation_reports.scoped]]
filter = "block/block_axes"
file = "block_block_axes.json"
generated = 20

[[generation_reports.scoped]]
filter = "flex/flex_axes"
file = "flex_flex_axes.json"
generated = 80

[[generation_reports.scoped]]
filter = "grid/grid_axes"
file = "grid_grid_axes.json"
generated = 36

[[generation_reports.scoped]]
filter = "grid-lanes/grid_lanes_axes"
file = "grid-lanes_grid_lanes_axes.json"
generated = 36

[[generation_reports.scoped]]
filter = "subgrid/subgrid_axes"
file = "subgrid_subgrid_axes.json"
generated = 36
```

Each scoped entry implicitly requires zero `unsupported`, `expected_fail`,
`quarantined`, and `failed_to_generate` entries. Browser, launch, full-report,
and scoped-report structs reject unknown or duplicate fields. Existing source
roots, imports, and case tables retain their current schema under version 2.

The launch profile digest is SHA-256 over compact UTF-8 JSON serialization of
this ordered tuple: `(1, batch_size, job_timeout_ms,
dom_poll_interval_ms, retry_count, job_order, retry_error_class, profile_scope,
page_scope, disable_default_args, disable_cache, arguments)`. XML provenance and
every report metadata object add the resulting
`launch-profile-sha256`; `check-corpus` recomputes it from the manifest.

Artifacts also carry `generator-sha256`. It is SHA-256 over compact UTF-8 JSON
serialization of this exact ordered array of `[repository_path, file_sha256]`
pairs: `Cargo.toml`, `Cargo.lock`,
`tests/bin/surgeist-layout-generate.rs`, and
`tests/bin/surgeist-layout-generate/generator.rs`. Each inner digest covers the
file's exact compiled-in bytes through `include_bytes!`, binding provenance to
the executable's generator inputs rather than mutable runtime files. XML uses
`generator-sha256`; report metadata uses `generator_sha256`. A newly built
`check-corpus` recomputes the compiled identity and rejects every differing or
missing XML/report value. No binary self-hash or Git-state dependency is used.

Every browser artifact also carries one deterministic
`artifact-snapshot-sha256`; XML uses that spelling and report metadata uses
`artifact_snapshot_sha256`. This value is the SHA-256 of compact UTF-8 JSON for
the following exact ordered tuple:

```text
(1,
 report_metadata_without_snapshot,
 sorted_xml_entries,
 sorted_full_report_entries)
```

`report_metadata_without_snapshot` is the exact ordered tuple
`(schema_version, generator, browser_source, browser_version,
launch_profile_sha256, helper_sha256, base_style_sha256,
corpus_manifest_sha256, taffy_commit, generator_sha256)`. Each
`sorted_xml_entries` member is `(output_path, provenance_without_snapshot,
xml_body_sha256)`, where the provenance tuple is `(schema_version, source,
source_sha256, linked_resources, linked_resources_recorded, helper_sha256,
base_style_sha256_or_null, browser_provenance, launch_profile_sha256,
generator_sha256)`. `xml_body_sha256` covers the exact bytes after the one
generated-provenance line, so the commitment is not recursive.
Each `sorted_full_report_entries` member is a tagged tuple containing the exact
serialized fields of one full-report `generated`, `unsupported`,
`expected_fail`, `quarantined`, or `failed_to_generate` entry. Paths and report
entries sort by their complete tuple using bytewise UTF-8 order. The manifest
digest commits the report inventory and expected counts. No timestamp, process
ID, staging path, or random run identity enters the commitment.

The existing single-prefix selector syntax is sufficient; no new selector
syntax is introduced. Artifact-writing generation accepts either no filter for
the full run or one exact `generation_reports.scoped.filter` value from the
selected manifest. Prefixes that are not exact manifest entries are rejected
before browser resolution, staging, canonical artifact mutation, or stale-file
pruning. The complete accepted report-file set is therefore exactly `all.json`
plus the five manifest entries above.

Browser jobs never write the canonical XML or report tree. They write one
generator-owned candidate under `target/`; a job, browser-lifecycle, or candidate
validation failure changes no canonical artifact. A full run constructs the
complete candidate from its own
outcomes and may begin when the current snapshot is absent or inadmissible. A
scoped run first requires `check-corpus` to accept the current sealed snapshot,
then replaces only its selected outcomes in that validated baseline. A stale
compiled generator, manifest, source, helper, report, XML, or snapshot identity
therefore forces a full run rather than permitting a scoped repair.

Both successful modes derive the full report and every scoped report from the
same candidate full-report partition. A scoped report is an exact filter
projection, not an independently trusted run record. Before publication the
generator validates the candidate with the same counts, path relations,
provenance, projection, and commitment rules as `check-corpus`. It then
atomically replaces individual XML files and scoped reports through sibling
temporary files, performs authorized stale deletions, and atomically replaces
`all.json` last. The prepared seal is already in its sibling temporary file and
all generator staging cleanup completes before that final rename; no fallible
publication or cleanup step follows it. That last replacement is the publication
seal. Any error or
process interruption before the seal leaves the prior `all.json`; its snapshot
identity or content commitment cannot validate a mixed tree. An error after all
jobs but before sealing publishes no admissible snapshot. If a cleanup failure occurs only
after all jobs have staged, no duplicate job failure is invented; the command
returns an infrastructure error and still withholds the seal.

The nine pre-FRI-02 scoped reports are obsolete after their completed focused
cycles and are removed by the successful full generator publication, not by
hand. A full publication removes every XML or report outside the manifest-owned
candidate before sealing. A scoped publication may replace or delete only its
selected XML outputs, but it refreshes the metadata and exact projection of all
manifest-owned reports before sealing and never removes an unrelated output.

The snapshot protocol is inventory-generic. During C07 it covers the exact
transitional 15-report manifest without pruning the 13 reports that entered that
cycle; after C08 it covers the final six-report manifest described here. No
snapshot or publication code hard-codes either count.

Each scoped report records its exact filter, contains exactly the named count in
`generated`, and has zero `unsupported`, `expected_fail`, `quarantined`, and
`failed_to_generate` entries. The union of its 208 generated output paths is
exactly the FRI-02 XML inventory, with no duplicate or path outside the five
prefixes.

The regenerated `xml/generation-reports/all.json` contains those same 208 paths
in `generated` and none in another bucket. Its final summary is exactly 5,256
generated, 356 unsupported, zero expected-fail, zero quarantined, and zero
failed-to-generate. The existing 356 unrelated unsupported entry
`(name, source, variant, reason)` tuples remain unchanged and visible; FRI-02
does not reclassify them. The HTML inventory test changes only to the resulting
exact totals: 1,159 Taffy-plus-local ordinary fixtures, 25 grid-lanes fixtures,
and 219 subgrid fixtures.

`check-corpus` derives the six-file set from `corpus.toml` and rejects any
missing report, extra JSON or non-JSON report artifact, mismatched filter/file
name, stale metadata or provenance, incorrect bucket/count/path union,
incorrect scoped projection, or incorrect full-report relation. It reconstructs
the complete artifact commitment from the XML payloads and full report, requires
every XML and every manifest-owned report to carry that same value, and requires
the value
to equal the `all.json` publication seal. It does not accept a report merely
because its filename agrees with its self-declared filter.

Browser XML is admissible as semantic oracle evidence only after the exact
pinned executable has passed runtime version validation, generation completed
without a failed/expected-fail/quarantined entry for the owned scope, and
`check-corpus` has validated source, helper, generator, launch-profile, provenance,
report-inventory, output-path, scoped-projection, and content-commitment relations
for the same sealed artifact snapshot.
Parity execution and review use that unchanged snapshot; changing source,
manifest, helper, generator behavior, XML, or reports invalidates admissibility
until generation and `check-corpus` pass again.

A pin-valid Chrome result that conflicts with the applicable CSS algorithm is
not layout truth merely because it was generated. Confirm a Chrome shortcoming
by reproducing the exact pinned result, matching it to normative CSS steps and
WPT expected output plus Chrome failure metadata when present, and inspecting
the corresponding exact-version Blink path when causality remains uncertain;
independent WebKit evidence may break a remaining tie. Once confirmed, retain
the browser output only as observation and stop the affected acceptance gate.
Do not hand-edit XML, silently exclude the case, weaken tolerance, or add an
expected-fail/quarantine entry. A reviewed specification/sequence/cycle-plan
amendment must instead assign semantic ownership to a CSS/WPT-derived unit or
layout-oracle test and explicitly authorize any browser-report classification.
Until then the zero-bucket C07 contract remains blocking.

Named non-ignored tests
`runs_fri_02_grid_axis_families_against_surgeist_layout`,
`runs_fri_02_grid_lanes_axis_families_against_surgeist_layout`, and
`runs_fri_02_subgrid_axis_families_against_surgeist_layout` require those exact
path sets, reject missing/misplaced/duplicate variants, and compare every case
through `compute_layout`. A child-only writing mode under an otherwise
horizontal container cannot satisfy a container-axis family.

### Oracle And Property Evidence

The test matrix includes:

| Evidence | Required coverage |
| --- | --- |
| Flow mapping table | All 10 writing-mode/direction pairs, every axis/side/line-over result. |
| Geometry round trip | Logical/physical size, edges, point, and rectangle for `f32` and `f64`, including reversed axes. |
| Scroll round trip and clamp | Signed offsets and intervals across all 10 pairs, negative minima, nonzero bounds, invalid/non-finite construction, lower/interior/upper clamping, and conversion-clamp equivalence. |
| Block | Ordinary stacking, auto inline/block size, percentage edge basis, collapse edges, baseline projection, parallel and orthogonal children. |
| Flex | Five modes x two directions x four flex directions, plus wrap-reverse, margins/insets, alignment, baseline, absolute/static projection, and scalar lanes. |
| Grid | Unequal intrinsic totals, areas, baseline projection, lanes, subgrid inheritance, parallel/opposing/orthogonal flows, and scalar lanes. |
| Public surface | Removed aliases/helpers absent; new semantic types and five modes reexported. |

Property tests generate finite extents and positions. Named concrete tests remain
the primary evidence for each behavior and failure class.

## FRI-02.14 Module And Code Outline

| Area | Required outcome |
| --- | --- |
| `src/geometry.rs` | C01 implemented `PhysicalAxis`, `LogicalAxis`, `PhysicalSide`, `FlowAxes`, crate-private logical geometry, and containing-flow edge basis. C04 adds only generic logical-size operations required by block; C05 removes the temporary live flex main/cross helpers. |
| `src/node_input.rs` | C01 added sideways modes and documented used `Direction`; C05 removes the temporary physical `FlexDirection` axis helpers. |
| `src/inline.rs` | C01 removed the private mapping/logical duplicate. Later algorithm cycles consume this shared participant/control behavior without another writing-mode table. |
| `src/output.rs`, `src/cache.rs`, `src/compute.rs` | C01 made diagnostics, baseline primitives, compute construction, cache identity, and root flow-aware. C04 replaces the top/bottom/unqualified-through compute fields with `PhysicalBlockMarginCollapseOf`, binds block and measured-leaf producers to their own flow, and preserves root/flex-root/hidden evidence. |
| `src/scroll.rs` | C02 implemented typed signed physical/flow-relative offsets and ranges, conversion, errors, and flow-owned layout range projection. Later cycles preserve that contract. |
| `src/block.rs` | Convert ordinary in-flow constants, sizing, cursor, margins, baselines, inline report placement, and physical projection to shared logical geometry. |
| `src/flex.rs` | Derive `FlexAxes`; replace physical main/cross and edge helper logic throughout current flex behavior. |
| `src/grid/axis.rs` | C01 delegated writing-mode comparison to `FlowAxes`; C06/C07 retain only grid-local logical-axis roles while migrating sizing and placement consumers. |
| `src/grid/mod.rs`, `tracks.rs` | Keep column/row bases, totals, gaps, intrinsic sizing, and reruns logical until projection. |
| `src/grid/child.rs`, `subgrid.rs`, `lanes.rs` | Project areas/offsets/baselines/inherited axes through shared mapping without absorbing other grid findings. |
| tests and browser parity | Add the required mapping, property, algorithm, topology, fixture, XML, and default regression evidence. |
| `Cargo.toml`, generator, `corpus.toml` | C03 implemented both validated pinned-browser modes, one launch-settings owner, schema-two reports, and browser-free corpus checks. C07 adds an owned-child browser session, a typed total job/cleanup lifecycle, staged sealed-snapshot publication, and generator/snapshot provenance without retuning launch or batching. C04-C07 add five exact scoped entries cumulatively; C08 removes the nine temporary pre-FRI-02 entries and leaves the final six-file inventory. |
| `src/lib.rs`, README, rustdoc | C01-C03 document and reexport the implemented substrate. Each later cycle adds only its intentional public type/docs; C08 verifies the final front door and root boundary. |

No module may define another exhaustive `WritingMode` mapping table. Direct
matches are limited to the one owning geometry implementation, enum parsing,
and tests that verify the canonical table.

## FRI-02.15 Root Integration Handoff

After a published layout candidate exists, the archival root handoff states
that root later must:

1. lower all five computed writing-mode values to layout `WritingMode`;
2. supply the used inline `Direction`, not an unresolved authored token;
3. replace `Axis` references with `PhysicalAxis`;
4. consume `FlowAxes` instead of duplicating physical-side tables;
5. replace direct top/bottom/through compute-margin access with
   `PhysicalBlockMarginCollapseOf` and its containing-flow-aware query;
6. consume signed `PhysicalScrollRangeOf` and any flow-relative conversion
   through layout's public contract;
7. keep live retained scroll offsets and host/CSSOM policy in root-owned state;
8. update root adapters/facade exports without compatibility aliases; and
9. refresh root-owned API artifacts after the candidate is visible.

Layout does not edit root, style, retained, text, or generated root artifacts.

## FRI-02.16 Dependency, Feature, Artifact, Documentation, And MSRV Impact

No new third-party dependency or feature is required. The existing optional
`chromiumoxide` `fetcher`, `rustls`, and `zip8` features remain because managed
browser acquisition is an intentional generator capability. The default layout
engine and existing `layout-golden-generate` crate feature remain the two
relevant feature states.

Changing the retry-class tuple member refreshes the launch-profile digest in
all generator provenance and reports. The successful full run owns that metadata
refresh and adds the compiled generator digest and artifact-snapshot commitment
to every XML/report. All 5,184 pre-C07 XML provenance lines therefore refresh;
their XML bodies and the 356 unsupported classification tuples remain exact.

Rust 1.97 remains the crate MSRV. All source, tests, and generator-feature code
compile with the already-installed Rust 1.97 toolchain.

New HTML sources, generated XML, and generation-report changes are expected and
must come from the existing layout-owned generator. Source and output provenance
remain current. No root API artifact is present or generated in this leaf.

Browser resolution has exactly two methods represented by a crate-private
closed `BrowserResolutionMode`:

1. `ManagedPinned` is selected by the existing `generate` command. It asks
   `BrowserFetcher` for `browser.version` in the repository-root-relative
   `browser.cache_root` from the selected corpus manifest. The fetcher reuses
   the expected platform binary when present and downloads/unpacks that exact
   pinned build when absent.
2. `ExistingPinned` is selected by a new `generate-existing` command. It
   requires `SURGEIST_BROWSER_PATH` to name an existing executable under the
   manifest-owned pinned cache, never calls `BrowserFetcher`, and fails with a
   missing-tooling error when the path is absent or invalid. Combined with
   `SURGEIST_LAYOUT_GENERATE_FILTER`, it is the supported AI one-shot fixture
   invocation; agents do not launch the Chrome binary manually.

Command and environment precedence is exact:

| Command | Browser method | Browser and filter contract |
| --- | --- | --- |
| `generate` | `ManagedPinned` | `SURGEIST_BROWSER_PATH` must be absent. Pin and cache come only from `corpus.toml`. `SURGEIST_LAYOUT_GENERATE_FILTER` is absent for the full run or exactly equals one manifest scoped filter. |
| `generate-existing` | `ExistingPinned` | `SURGEIST_BROWSER_PATH` is required. Pin and cache still come only from `corpus.toml`. `SURGEIST_LAYOUT_GENERATE_FILTER` has the same closed domain as `generate`. |
| `check-corpus` | None | No browser-selection variable or generation filter is read and no browser resolver or executable is entered. |
| `check-taffy-corpus` | None | Preserve the current pinned-source verification behavior. No browser-selection variable or generation filter is read. |
| `import-taffy` | None | Preserve the current pinned Taffy import behavior and manifest contract. No browser-selection variable or generation filter is read. Agents do not invoke this acquisition-capable command without exact user permission. |
| `__remove-generator-temp <path>` | None | Private child-process mode entered only by the running generator. Validate one direct-child path under a fixed generator-temp root, remove it, and perform no browser, source, manifest, XML, or report access. |
| no command or any other command | None | Return a usage error without browser, source-import, or artifact access. |

`SURGEIST_BROWSER_VERSION` and `SURGEIST_BROWSER_CACHE` are removed as override
authorities; setting either on a generation command is an ambiguity error.
`SURGEIST_LAYOUT_BROWSER_PARITY_ROOT` selects the self-contained corpus root and
its manifest for every recognized command. On either generation command,
`SURGEIST_LAYOUT_GENERATE_FILTER` is normalized and then must equal one complete
manifest scoped filter; empty means the full run. Invalid filter or browser-env
combinations fail before resolving a browser or touching source, XML, or report
artifacts. The two Taffy commands retain their current source-acquisition and
verification behavior; FRI-02 changes only command dispatch so they do not
construct browser configuration.

The managed acquisition capability is product behavior, not standing workflow
permission for an agent to download software. Agent invocations use
`ExistingPinned`; a cache miss remains the canonical missing-tooling blocker
unless the user explicitly authorizes the managed acquisition.

Both resolution methods return one validated crate-private `PinnedBrowser`
value. Before creating or changing XML/report artifacts, the generator invokes
the resolved executable with `--version`, normalizes whitespace, and requires
exactly `browser.version_output`. Managed resolution likewise rejects a fetched
or cached executable whose reported version differs from the requested pin.

Existing-path validation is ordered and fail-closed: the environment value must
be non-empty UTF-8, relative to `CARGO_MANIFEST_DIR`, and contain no root,
prefix, `.` or `..` component; joining it to the repository root and
canonicalizing it must produce an executable regular file beneath the
canonicalized manifest cache root. A symlink escape, absolute path,
non-executable file, missing file, or version-command failure is rejected.
Managed output passes through the same canonical containment, executable, and
version validation after the fetcher returns.

`PinnedBrowser` records the canonical executable plus its repository-relative
forward-slash provenance path, manifest source, and exact version. The
repository-relative path is obtained by stripping the canonical repository root
after containment succeeds; a machine-absolute path cannot enter artifacts.
Both methods therefore emit identical stable XML and report provenance:

```text
chrome-for-testing/149.0.7827.115 (target/surgeist-browser/...)
```

One generator-owned `browser_launch_contract(pinned_browser, profile)` function
is the only production builder for the primary batch browser, recovery browser,
retry browser, and existing-pinned one-shot invocation. It returns the matching
Chromiumoxide `BrowserConfig` and `HandlerConfig`, applying the same headless
mode, unique user-data profile, disabled default arguments and cache, and exact
ordered `browser.launch.arguments` list from `corpus.toml`; no duplicate
production argument constant remains. That list retains `use-mock-keychain`,
`no-first-run`, `no-default-browser-check`, pinned locale/color behavior,
background-network suppression, and the required layout feature flags. The
handler request timeout and every outer launch/job deadline use the same
manifest-owned `job_timeout_ms`; library defaults cannot diverge by path.

The private launch phases are separate valid types. `OwnedBrowserProcess` owns
the `chromiumoxide::async_process::Child` and validated profile path immediately
after spawn. A successful DevTools attach consumes it into
`OwnedBrowserSession`, which additionally owns the connected `Browser` and
handler task; no struct stores a partially initialized set of optional owners.
Launch calls public `BrowserConfig::launch()` to obtain the process, discovers
the exact `DevTools listening on ws://.../devtools/browser/...` URL from its
stderr under the launch deadline, and calls `Browser::connect_with_config` with
the shared handler config. It does not call `Browser::launch`, whose
error/timeout path cannot return the child handle, and it never calls
`Browser::wait` or `Browser::kill`. Thus every post-spawn launch fault retains
an owned process that can enter bounded teardown. No documented or internal
single-use path executes Chrome outside this owner.

### Generator Stability Invariants

The current fixture-generation operating profile is preserved as tested
behavior. FRI-02 changes browser resolution/version validation and centralizes
launch construction; it does not retune generation. The required values and
lifecycles are:

| Setting | Required behavior |
| --- | --- |
| Batch size | At most 50 fixture jobs per primary browser process. Recovery may shorten a process lifetime but never enlarges or combines logical batches. Larger batches previously caused intermittent omitted fixtures and are not an implementation option. |
| Job order | Deterministic sorted fixture order, processed sequentially within each batch. |
| Job timeout | 10 seconds for every initial or retry browser-job attempt. |
| DOM poll interval | 25 milliseconds. |
| Retry | One retry only for typed `BrowserJobFault`, in a fresh browser process with the same launch config. Content failures are terminal for that job without invalidating a healthy browser. |
| Profile | One unique temporary user-data directory per primary, recovery-primary, and retry browser, removed after bounded shutdown. |
| Page lifecycle | One page per job, closed after measurement; browser and handler are closed/joined after the batch. |
| Launch config | Default Chromiumoxide arguments disabled, browser cache disabled, explicit `headless=new`, and the current exact 28 custom arguments including `use-mock-keychain`. |
| Failure accounting | Every failed or unattempted job is represented exactly once in the in-memory `failed_to_generate` partition; no generic skip, duplicate, or silent omission is permitted. A failed run does not publish that partition as a canonical corpus report. |

Each primary attempt's budget begins before page creation and ends after helper
measurement is decoded and its page has closed. A retry budget begins before
retry-browser launch. Primary and recovery launch handshakes each have the same
10-second bound. A successful attempt stages that fixture's outputs/report
entries only in the generator-owned candidate; no failed attempt stages output
and no attempt writes the canonical corpus.

The crate-private attempt model is closed and typed:

| Failure kind | Source and browser state | Retry/accounting |
| --- | --- | --- |
| `LaunchFault` | Executable spawn failure, missing/invalid DevTools stderr URL, launch-handshake deadline, or connect `CdpError`. No child exists after a spawn failure; every later launch fault retains the owned child. | After successful containment/profile cleanup, account the launch's assigned jobs as specified below. It never recursively relaunches itself or consumes a job retry. |
| `ContentFailure` | Fixture/helper/schema/value failure or `CdpError::JavascriptException`; browser remains reusable only after bounded page close succeeds. | Record the job once; no retry; continue in that primary. |
| `BrowserJobFault` | Initial/retry outer attempt deadline, page/protocol/transport/process `CdpError` other than `JavascriptException`, or a page that cannot close inside the attempt budget; browser is invalid. | An initial attempt consumes the job's sole fresh-browser retry after successful session teardown; any retry failure is terminal. |
| `CleanupFailure` | Handler, owned child, or generator-temp cleanup cannot reach its specified terminal state inside the bounded teardown. | Run-fatal: no retry, browser reuse, recovery process, or later batch; account every unstaged job remaining in the run exactly once. |

Mapping matches `CdpError` variants, never formatted messages. Local decode and
validation after a successful helper value are `ContentFailure`; every CDP error
except `JavascriptException` is conservatively browser-invalidating. A content
failure whose page cannot close becomes `BrowserJobFault` because the whole
owned session must be discarded; it becomes `CleanupFailure` only if that
discard cannot terminate cleanly.

Launch and recovery use this finite matrix:

| Transition | Required result |
| --- | --- |
| Initial primary launch succeeds | Process the logical batch sequentially. |
| Initial primary `LaunchFault` | Contain any owned child and remove the profile; record every job in that logical batch as terminal failure without consuming per-job retry; continue with the next logical batch. |
| Initial attempt has `ContentFailure` | Close the page, record the job, and continue; cleanup failure follows its row below. |
| Initial attempt has `BrowserJobFault` | Tear down the invalid primary; if clean, run that job's one retry, then launch one recovery primary for only the untouched remainder. |
| Retry `LaunchFault`, `ContentFailure`, or `BrowserJobFault` | Consume the retry and record that job once; contain any retry child/session, then launch a recovery primary only after clean teardown. |
| Recovery-primary `LaunchFault` | Contain any owned child and record every untouched job in that logical batch once without consuming their retry budgets; continue with the next logical batch. |
| Any `CleanupFailure` | Mark the current unstaged job plus every untouched job in all remaining logical batches once, stop launching immediately, suppress snapshot publication, and return a run-fatal error. |

Any terminal job failure prevents publication even when later jobs run to
complete ordinary accounting. `CleanupFailure` does not run later batches merely
to account them; it deterministically walks the already-known sorted job list in
memory. Before returning nonzero, the command renders every in-memory failure in
sorted job order with its source and typed diagnostic plus the final failed-job
count; it never directs the user to an unchanged canonical report. When cleanup
fails after the final job has staged, there is no current or
remaining job to record, so the generator returns only the infrastructure error
and does not fabricate a duplicate entry. In every failure case the prior sealed
canonical snapshot remains untouched unless publication itself was interrupted;
an interrupted publication is rejected by its unchanged `all.json` seal.

Each browser-fault segment permits one recovery-primary launch; a later job may
start a new segment, so at most the finite job count bounds recovery. Launch
failure never recurses.

Owned-session teardown ordering is exact. First attempt `Browser::close` for at
most 10 seconds; timeout or error is diagnostic and forces the process path but
does not by itself mean ownership was lost. Abort the handler task and require
its join to resolve as normal completion or expected cancellation within 10
seconds. Then call `Child::try_wait`. If the child was not reaped, call its
Tokio inner child's non-awaiting `start_kill` once and bound `Child::wait` by 10
seconds. If `start_kill` reports an error, one immediate `try_wait` may prove the
child already exited; otherwise cleanup fails. A wait error or second deadline
is `CleanupFailure`; the run launches nothing else, drops the still-owned child
with `kill_on_drop`, and terminates. `Browser::wait`, `Browser::kill`, and an
unbounded browser future are never used.

No teardown error short-circuits a later ownership step. Browser-close and
handler-join defects are retained while child kill/reap still runs. Profile
cleanup runs after a reaped child even when an earlier defect exists; it is
skipped when the child cannot be reaped so an active profile is never removed.
The returned `CleanupFailure` aggregates those typed defects in transition
order. A pre-attach `OwnedBrowserProcess` performs only child kill/reap followed
by profile cleanup.

Generator temporary-directory deletion never uses `spawn_blocking` or detached
filesystem work. The
generator has one private `__remove-generator-temp` child mode in its current
executable. It accepts exactly one lexically validated direct-child path under
the fixed browser-profile or snapshot-staging root, performs `remove_dir_all`,
and exits success only when the path is absent. Ordinary command dispatch cannot
select it through environment state. The parent starts that helper with
`kill_on_drop`, bounds it by 10 seconds, and on timeout calls `start_kill` and
waits at most 10 more seconds for reaping. Exactly three helper attempts are
allowed, separated by 25 milliseconds, and a later attempt starts only after the
previous child was reaped. An unreaped helper, rejected path, or final present
profile is `CleanupFailure`. This subprocess boundary makes blocking filesystem
deletion cancellable from the generator without adding a dependency or allowing
overlapping removal work.

The helper/base-style injection, document-write loading path, DOM-readiness
predicate/poll interval, and four fixture variants remain unchanged. The old
`navigation_timeout_ms`, `open-load-reset-timeout`, and `browser-job-timeout`
are rejected, not aliased. Future lifecycle changes require separate corpus
evidence.

The managed human-facing command is:

```sh
cargo run --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate
```

With `SURGEIST_BROWSER_PATH` already set to the existing pinned cached
executable, the no-fetch AI/artifact commands are exactly:

```sh
test -x "$SURGEIST_BROWSER_PATH"
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER= cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER=block/block_axes cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER=flex/flex_axes cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER=grid/grid_axes cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER=grid-lanes/grid_lanes_axes cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="$SURGEIST_BROWSER_PATH" SURGEIST_LAYOUT_GENERATE_FILTER=subgrid/subgrid_axes cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true SURGEIST_LAYOUT_GENERATE_FILTER= cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```

`check-corpus` is browser-provenance validation over existing artifacts; it does
not require, resolve, execute, or download a browser. It derives the required
browser source/version from the committed pinned corpus contract, validates the
compiled generator digest, artifact-snapshot commitment and seal, five scoped
report projections and their XML provenance, the full-report 208-path subset
relation and exact summary, and the three HTML inventory totals.
The generated artifact delta separately contains no change to any of the 356
pre-existing unsupported `(name, source, variant, reason)` tuples. It never
invokes the Taffy import path or either browser resolver.

README and rustdoc document:

- public physical geometry;
- `FlowAxes` and all five modes;
- used-direction ownership;
- crate-private logical algorithm geometry;
- signed physical scroll range meaning;
- managed pinned fetch/cache generation, existing-pinned AI one-shot generation,
  exact executable-version validation, shared mock-keychain launch settings,
  and browser-free corpus freshness checks;
- root ownership of adapters, used-style lowering, live scroll state, and API
  artifacts; and
- the boundaries with later inline, overflow, flex, grid, alignment, and
  positioned initiatives.

Surgeist-owned Rust remains free of `unsafe`.

## FRI-02.17 Required Test Evidence

### Construction And Mapping

Tests cover every public constructor and accessor, all 10 mapping rows, opposing
sides, axis inversion, sideways-lr direction inversion, physical/logical round
trips, signed zero, non-finite values, inverted intervals, and both scalar
lanes.

Compute-input tests cover direct leaf, viewport root, flex-item root, recursive
child, and hidden construction. Vertical and sideways root cases in both scalar
lanes prove logical-inline auto fill, percentage padding/margin basis, mapped
start/start physical location, cache-key separation by containing flow, and
unchanged propagation through hidden descendants. A source/API search proves no
`ComputeInputOf::HIDDEN`, context-free struct literal, or missing-flow
constructor remains.

Named clamp tests cover below-minimum, exact-minimum, interior, exact-maximum,
and above-maximum values on both components of both coordinate spaces in `f32`
and `f64`. Property tests over all 10 flow mappings and signed nonzero intervals
prove both directions of:

```text
convert(range).clamp(convert(offset)) == convert(range.clamp(offset))
```

They also prove that each clamped result lies inside its range and that a second
clamp is idempotent.

### Algorithm Behavior

Focused tests use the real public `compute_layout` front door and prove:

- ordinary block children follow logical block flow;
- compute output constructs collapse carriers from the child's flow, parent
  selection uses the parent's physical block sides, parallel/opposing through
  queries remain eligible, and orthogonal through queries are rejected;
- flex containers are non-leaf and follow logical main/cross flow;
- grid intrinsic totals and areas project correctly;
- baseline points use the mapped physical block axis and line-over side;
- output/cache/rounding retain physical geometry; and
- no FRI-02-owned sideways or vertical request panics or silently falls back.

Named collapse regressions include a parallel child, an opposing child, and an
orthogonal child whose own empty block can collapse through but whose parent-axis
margins and through state must remain isolated. Both scalar lanes exercise these
cases through the real public layout front door. A second named matrix uses
measured empty leaves in the same three relationships and proves their carrier is
bound to the leaf's own flow rather than the containing flow. That matrix covers
zero logical block/nonzero logical inline extents as eligible and nonzero logical
block/zero logical inline extents as ineligible for vertical and sideways leaves
in both scalar lanes.

Normal and rounded scroll-geometry tests prove layout-produced magnitudes first
form flow-relative ranges and then project to the expected signed physical
interval on every reversed axis for all 10 mappings and both scalar lanes. A
production-source search rejects direct physical range construction from
overflow extents outside the one `FlowAxes` projection pipeline.

### Fixture Evidence

The block, flex, and grid family inventories are exact. Missing variants,
duplicates, misplaced paths, leaf-lowered flex topology, unsupported
writing-mode parse results, or comparison bypasses fail default tests.

Generator tests prove both resolution modes reject a runtime version mismatch
before creating or changing XML/report artifacts. Existing-pinned tests also
cover absent, empty, absolute, outside-cache, and non-executable paths and prove
the fetcher is never entered. Managed-resolution tests prove the exact requested
pin is passed to the cache/fetch boundary before the returned binary is
validated; tests do not perform a network acquisition.

Command-dispatch tests prove both generation modes reject a filter outside the
manifest's exact scoped set before browser resolution or artifact access, an
empty filter selects the full report, and each exact scoped filter selects only
its manifest-named measurement scope. They prove the private temp-removal mode
rejects every path outside its two fixed roots and cannot enter browser or
artifact code. They also prove `check-taffy-corpus` and
`import-taffy` remain recognized non-browser commands and do not construct or
validate browser state; tests exercise dispatch boundaries without performing a
source acquisition.

One launch-contract test asserts every production launch site consumes the
shared config/handler owner and its exact 28 arguments, including
`use-mock-keychain`. A source assertion rejects `Browser::launch`,
`Browser::wait`, and `Browser::kill` in generator production code. Stability
tests assert the 50-job maximum, sorted sequential jobs, 10-second launch and job
deadlines, 25-millisecond polling, typed launch/content/browser-fault/cleanup
classification, one fresh-browser retry, bounded close/handler/start-kill/wait
and helper-process cleanup, sequential primary recovery, and exact in-memory
failure accounting. Deterministic private harnesses prove spawn failure,
post-spawn URL/connect timeout with retained-child teardown, initial fault then
success, retry fault, content failure with healthy reuse,
initial/recovery/retry launch faults, cleanup failure, child-wait timeout,
handler abort/join, helper timeout/reap, profile-cleanup exhaustion, no overlapping
removal attempts, exact whole-run remainder accounting/order, no duplicate
staging, and no launch after a run-fatal outcome without wall-clock waits.

Snapshot tests prove full generation can construct a new candidate without a
baseline; scoped generation rejects an inadmissible baseline before browser
resolution; a scoped candidate replaces only its projection; and both modes
derive every manifest-owned report from one full partition. Commitment tests
mutate each
global metadata field, XML path, parsed provenance field, XML payload, and
full-report bucket entry and require a different digest. Publication tests
interrupt before XML replacement, during XML/scoped-report replacement, during
stale deletion, and immediately before the final `all.json` replacement; every
partial state is rejected while the prior seal can never accept mixed content.
A terminal browser or cleanup failure leaves canonical artifacts byte-identical.
Idempotent full and scoped publications reproduce the same commitment and bytes.
Digest tests prove each of the four compiled generator inputs contributes in
exact order, and `check-corpus` rejects stale, missing, mixed, or unsealed XML
and report identities plus a scoped report that is not the exact full-report
projection. A single-filter existing-pinned test exercises the AI one-shot path
through the same measurement, owned-child launch, staging, and sealed-publication
configuration.

`check-corpus` passes without a browser environment, rejects every missing or
extra report filename, and verifies the exact scoped/full report relationships.

### Regression Boundary

Existing focused evidence for later findings remains visible. A FRI-02 test does
not change an unrelated expected result merely to make axis migration green. If
axis-correct execution exposes a later finding, the test isolates and records
the exact later-owned behavior without weakening the FRI-02 assertion.
The vertical line-break clear panic remains visible as `BLOCK-014` evidence and
is not reclassified as an FRI-02 regression.

## FRI-02.18 Verification Surface

The product verification surface includes these repository commands. Their
execution, review, landing, and publication procedure remains owned solely by
`$surgeist-agent`:

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --features layout-golden-generate
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Additional product evidence consists of:

- named default block/flex/grid writing-mode regression tests;
- focused browser-parity filters for every new family;
- mapping/scroll property tests in both scalar lanes;
- source/API searches proving old aliases/helpers and duplicate mapping tables
  are absent;
- the configured generator/report freshness checks for changed fixtures; and
- a repository-wide source scan proving Surgeist-owned Rust contains no unsafe
  construct.

The ignored full browser corpus is not claimed as FRI-02 closure; `FRI-13` owns
the aggregate normal-verification gate. Every FRI-02-owned family itself is
non-ignored and green.

## FRI-02.19 Finding Closure Matrix

| Finding | Closure condition | Evidence |
| --- | --- | --- |
| `BLOCK-003` | Ordinary block sizing, placement, margins, and baselines use the containing block's logical axes for all five modes. | Named block regressions, all-mode browser families, orthogonal-flow tests, and physical output assertions. |
| `FLEX-001` | Every current flex main/cross consumer derives from `FlowAxes`; row/column/reverse/wrap behavior is correct for all five modes and both directions. | Flex mapping/property tests plus 80 non-leaf browser XML comparisons. |
| `GRID-004` | Columns/rows remain inline/block through intrinsic sizing, areas, baselines, lanes, and subgrid projection; vertical/sideways unequal totals swap physical dimensions. | `70x110 -> 110x70` regression, grid property/oracle tests, and owned browser families. |
| `OVERFLOW-004` | Public offset/range types name physical or flow-relative coordinates, encode signed minima/maxima, and convert through the canonical mapping. | Constructor/error/round-trip tests, public API search, and `ScrollGeometry` contract review. |
| `TEST-005` | Default parity evidence proves target flex nodes are non-leaf and executes all five writing modes through flex layout. | Exact topology-checked family inventory and non-ignored comparison test. |

## FRI-02.20 Initiative Acceptance

`FRI-02` is complete only when all of the following are true:

1. all five writing modes and both used directions obey the normative mapping
   table;
2. public physical and crate-private logical geometry are distinct and every
   contextual conversion is named;
3. `FlowAxes` is the only production owner of writing-mode mapping;
4. `Axis`, unqualified scroll offset/range types, context-free flex helpers,
   `Edges::zip_inline_size`, top/bottom-only compute margin fields, and an
   unqualified collapse-through output field are absent without aliases, and
   algorithm-local mapping tables are absent;
5. ordinary block flow produces correct physical geometry across horizontal,
   vertical, sideways, parallel, and orthogonal cases owned here, including
   viewport-root, flex-item-root, and hidden-flow construction;
6. current flex sizing/placement/alignment/baseline/absolute/output paths use
   logical main/cross axes and the non-leaf browser matrix is green;
7. grid, grid-lanes, and subgrid map columns/rows, intrinsic totals, areas, and
   current baseline behavior through logical axes without claiming later grid
   findings;
8. physical and flow-relative signed scroll ranges are valid, scalar-generic,
   round-trippable, and unambiguous, and normal/rounded layout geometry reaches
   physical ranges only through flow-relative projection while live state and
   later geometry remain outside layout;
9. new HTML/XML/report artifacts are generator-produced and provenance-current,
   the report directory has exactly its manifest-owned six files, both validated
   pinned-browser resolution methods use one launch profile, every browser job,
   retry, child/session teardown, and profile cleanup is finite, and the
   established batch maximum, numeric timeout, retry count, launch arguments,
   and keychain bypass remain intact; all browser comparisons satisfy the
   generator-bound sealed artifact-snapshot admissibility predicate;
10. README, rustdoc, public reexports, Rust 1.97 MSRV, feature behavior, and the
    archival root handoff match the implemented contract;
11. every owned browser family runs in normal non-ignored verification; and
12. every product command and observable predicate in `FRI-02.18` passes.
