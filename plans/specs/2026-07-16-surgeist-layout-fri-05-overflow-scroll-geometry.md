# FRI-05 Overflow And Scroll Geometry

Status: draft

Design owner: `surgeist-layout`

Specification ID: `FRI-05`

## FRI-05.1 Authority And Outcome

This specification is the direct desired-state contract for `FRI-05` in
`plans/specs/2026-07-11-surgeist-layout-findings-resolution-index.md`. It owns
closure of `BLOCK-001`, `BLOCK-002`, `GRID-011`, `OVERFLOW-001`,
`OVERFLOW-002`, `OVERFLOW-003`, `OVERFLOW-005`, `CORE-006`, `GRID-009`, and
`TEST-002` from
`plans/2026-07-10-surgeist-layout-full-code-review-findings.md`.

The outcome is one layout-owned overflow contract in which:

1. layout input carries the complete computed `overflow-x`/`overflow-y` pair,
   including `auto`, and cannot carry a pair that still requires CSS computed
   value coupling;
2. replaced-element used overflow, scroll-container exposure, clipping, and
   scrollbar policy are derived from that computed pair rather than encoded as
   contradictory flags;
3. clip margins, stable and both-edge gutters, scroll padding, scroll margin,
   and scroll snap input are finite layout-ready values with exact defaults;
4. one canonical geometry factory derives border, padding, content, scrollport,
   overflow-clip, optimal-viewing-region, gutter, and directional range facts
   from common primitives, including format-specific flex origins and current
   content-distribution alignment overflow;
5. no public constructor can combine an unrelated scrollport, range, clip, or
   gutter rectangle into an apparently valid `ScrollGeometryOf`;
6. one shared accumulator applies the same own-box, child-area, nested-overflow,
   clipping, margin, zero-area, out-of-flow, and terminal-padding rules across
   root, block, flex, ordinary grid, subgrid, and grid-lanes paths;
7. classic `overflow: auto` gutter decisions converge from a no-conditional-
   gutter pass, including cross-axis scrollbar induction, without corrupting
   caches or committing speculative output;
8. valid negative margins and boxes smaller than their requested scrollbar
   reservation produce saturated geometry rather than an error or panic;
9. flex and grid retain each laid-out child's geometry and emit their own
   geometry, while grid content extent retains the child's container-relative
   origin;
10. content extent and nested contribution are calculated independently on x
    and y, so zero on one axis never erases valid overflow on the other;
11. `NodeOutputOf::content_box_size()` reports the actual layout content box,
    including every effective gutter reservation;
12. browser `scroll_width` and `scroll_height` expectations are compared to the
    corresponding physical range spans and can fail parity; and
13. root receives enough immutable layout geometry to own live offsets,
    transformed snap-target association, scrolling UI, and snap selection
    without reconstructing leaf invariants.

This is a breaking pre-release correction. Backward compatibility is not
required. Removed constructors, fields, aliases, and deferred-capability
variants are not retained through wrappers or duplicate paths.

## FRI-05.2 Scope And Non-Goals

### Owned Scope

This specification owns:

- `Overflow::Auto` and a validated public computed-overflow pair;
- internal computed-to-used overflow derivation for replaced boxes;
- the computed-overflow predicate used by grid and flex automatic minimums;
- layout-ready overflow clip margin, scrollbar gutter, scroll padding, scroll
  margin, scroll snap type, snap alignment, and snap stop values;
- classic scrollbar reservation and placement in every `FlowAxes` mapping;
- bounded `overflow: auto` cross-axis gutter iteration;
- scalar-generic scroll rectangles, physical clip intervals, box rectangles,
  gutter rectangles, optimal viewing regions, target geometry, and canonical
  `ScrollGeometryOf` construction;
- directional range computation, format-specific flex scroll origins, and
  current content-distribution start-side reachability that preserve the signed
  physical range types completed by `FRI-02`;
- shared scrollable-overflow accumulation for current block, flex, ordinary
  grid, subgrid, and grid-lanes formatting paths, including their current
  absolute-child paths;
- current root and leaf output geometry and rounding;
- axis-independent `content_size` derivation and grid-area-origin correction;
- cache treatment of speculative auto-gutter passes;
- removal of arbitrary gutter/geometry construction and the obsolete
  `ScrollUnsupportedFeature` deferred list;
- the `NodeOutputOf` scrollbar and content-box helper correction;
- fixture parsing and serialization for the exact FRI-05 layout-ready values;
- eleven active Surgeist browser sources named in Section FRI-05.11;
- activation of existing parsed browser scroll expectations in the comparator;
- one final full ExistingPinned regeneration after all FRI-05 HTML, helper,
  serializer, parser, and fixture inputs settle;
- public reexports, crate documentation, focused tests, and the root handoff;
  and
- explicit later-owner boundaries where overflow consumes behavior but does not
  absorb its formatting algorithm.

### Explicit Non-Goals

This specification does not:

- parse authored CSS, run cascade, resolve logical CSS longhands, perform root
  element/body overflow propagation, or apply the specified-to-computed
  overflow coupling rules. `surgeist-css` owns the authored `auto` token;
  `surgeist-style` is the sole computed-value owner and applies axis coupling
  after cascade; root adapters only transfer the authored and computed phases
  across crate boundaries. Section FRI-05.12 records that required follow-up
  without adding it to this leaf implementation;
- own retained node identity, a current scroll offset, scroll events, user or
  programmatic scrolling commands, scrollbar painting, scroll animation,
  overscroll physics, snap candidate selection, re-snapping, focus/target
  scrolling, or CSSOM coordinate policy;
- model transforms, border radii, masks, paint containment, visibility,
  graphical clipping paths, ink overflow, or transformed 3D overflow; root and
  rendering owners combine their style/transform facts with the layout output;
- implement `scroll-initial-target`, scroll anchoring, `scroll-behavior`,
  `overscroll-behavior`, scroll timelines, or Level 2 snap behavior;
- add missing inline line construction, float exclusion, or BFC behavior owned
  by `FRI-06`, flex completeness owned by `FRI-07`, grid/subgrid placement and
  sizing completeness owned by `FRI-08`, new alignment behavior owned by
  `FRI-09`, or positioned-layout completeness owned by `FRI-10`;
- make a later formatting algorithm correct merely because its existing boxes
  now contribute coherent overflow geometry;
- run or claim the ignored aggregate browser-parity release gate owned by
  `FRI-13`;
- create a parser framework, CSS tokenizer, expression system, transform model,
  retained snap registry, geometry trait, generator subsystem, report kind,
  fixture manifest mode, command wrapper, or reusable generation abstraction;
- expand generator architecture. Generator edits are limited to serializing the
  new computed-style fields, parsing their fixture attributes, adding the named
  HTML sources, and fixing a confirmed genuine bug if evidence requires it;
- add another generator run after a valid final full regeneration over unchanged
  inputs. Scoped generation remains optional iteration-only diagnostic evidence;
- edit root adapters, root facade exports, root API artifacts, the root gitlink,
  or sibling repositories;
- acquire software, change dependencies or feature flags, add `unsafe`, or
  change the crate's MSRV.

## FRI-05.3 Standards And Current Evidence

### Normative Evidence

CSS Overflow Level 3 defines `visible`, `hidden`, `clip`, `scroll`, and `auto`,
the computed coupling of physical overflow axes, replaced-element used-value
conversion, scrollable versus non-scrollable values, programmatic hidden
scrolling, scrollable-overflow contribution, unreachable start-side overflow,
overflow clip margins, scrollbar sizing, and stable/both-edge gutters:

- <https://www.w3.org/TR/css-overflow-3/#overflow-properties>
- <https://www.w3.org/TR/css-overflow-3/#scrollable>
- <https://www.w3.org/TR/css-overflow-3/#overflow-clip-margin>
- <https://www.w3.org/TR/css-overflow-3/#scrollbar-gutter-property>

In particular, `hidden`, `scroll`, and `auto` are scrollable computed values;
`visible` and `clip` are non-scrollable computed values. A `visible` or `clip`
axis computes to `auto` or `hidden`, respectively, when paired with a scrollable
axis. A computed `hidden` value has a used value of `clip` on a replaced box.

CSS Grid Level 2 and CSS Flexbox define a content-based automatic minimum only
when the item's computed overflow in the applicable axis is non-scrollable. For
a flex item, that applicable axis is the completed flex container's main axis.
After CSS computed-value coupling, however, D-01 permits only pairs whose axes
have the same scrollability class. Layout therefore observes the same automatic-
minimum predicate for either physical axis and must not fabricate a rejected
cross-group pair to simulate main-only or cross-only scrollability:

- <https://www.w3.org/TR/css-grid-2/#min-size-auto>
- <https://www.w3.org/TR/css-flexbox-1/#min-size-auto>

CSS Box Alignment Level 3 defines the special interaction between overflowing
content distribution and scroll containers. A non-normal content-distribution
value reduces the unreachable start-side region only enough to make the final
in-flow alignment subject reachable in its start-aligned position; unrelated
out-of-flow overflow does not enlarge that adjustment:

- <https://www.w3.org/TR/css-align-3/#overflow-scroll-position>

CSS Scroll Snap Level 1 defines physical and logical snap axes, proximity and
mandatory strictness, the scrollport-derived optimal viewing region and
snapport, absolute scroll-margin outsets, block/inline snap alignment, and snap
stop. Omitted `scroll-snap-type` strictness means `proximity`. A target is
captured by the nearest snap-position-capturing ancestor, which can be a scroll
container or a non-scroll container with non-`none` snap type; the latter is a
capture barrier rather than a reason to continue to a farther scroll container.
Block/inline target alignment normally uses the snap container's writing mode,
while start/end alignment for an oversized snap area uses the target box's
writing mode. The specification intentionally leaves scroll physics and final
snap-position choice to the user agent:

- <https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-type>
- <https://www.w3.org/TR/css-scroll-snap-1/#scroll-padding>
- <https://www.w3.org/TR/css-scroll-snap-1/#scroll-margin>
- <https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-align>
- <https://www.w3.org/TR/css-scroll-snap-1/#scroll-snap-stop>

CSSOM View defines `scrollWidth` and `scrollHeight` as physical scrolling-area
dimensions. The checked-in generator stores the non-negative delta from the
corresponding client dimension, so parity compares those values to range span,
not to a signed endpoint:

- <https://www.w3.org/TR/cssom-view/#dom-element-scrollwidth>
- <https://www.w3.org/TR/cssom-view/#dom-element-scrollheight>

The standards define more rendering and host behavior than this layout crate
owns. FRI-05 implements the layout-ready inputs and geometry needed at the leaf
boundary; the explicit non-goals above remain outside that claim.

### Source Evidence At The Published Base

This table describes clean published commit
`683ffac1b7ba633410aca6b490dc2ac65bc9c8bd`.

| Evidence ID | Current source fact | Required correction |
| --- | --- | --- |
| `E-OVERFLOW-PAIR` | `NodeInputOf::overflow` is a mutable `Point<Overflow>` and `Overflow` omits `Auto`. The fixture parser maps `auto` to `Scroll`. | Add `Auto`, require a canonical computed pair, and preserve `auto` through fixture lowering. |
| `E-SCROLLABLE-PREDICATE` | `Overflow::is_scrollable()` is true only for `Scroll`; grid automatic-minimum predicates call it. | Treat computed `Hidden`, `Scroll`, and `Auto` as scrollable without applying replaced used-value conversion to the auto-minimum test. |
| `E-OVERFLOW-PREDICATES` | Public `Overflow::clips_contents()` and `blocks_margin_collapse()` are per-axis methods used directly by block callers, but no phase distinguishes computed BFC behavior from replaced-box used clipping. | Give each current caller one explicit computed-pair or used-axis predicate and make `Auto` participate in clipping and block formatting-context behavior. |
| `E-PHASE-UNSAFE-FACTS` | Exported `ScrollOverflowExposure`, `ScrollContainerAxis`, and `ScrollContainerFacts`, their public constructors, and the crate-visible `scroll_container_facts_from_overflow` path construct geometry facts without a canonical computed pair or replaced-box used-value conversion. | Remove those types and construction paths; derive a private used-overflow pair only from `ComputedOverflow` plus replaced status and expose the resulting axes read-only on canonical geometry. |
| `E-DEFERRED-FEATURES` | `ScrollUnsupportedFeature` lists overflow auto, clip margin, stable/both-edge gutters, scroll padding, scroll margin, snap, and mixed-axis coupling as deferred. | Replace placeholders with real input/output contracts and remove the obsolete deferred enum. |
| `E-ARBITRARY-GEOMETRY` | `ScrollbarGutterRectsOf::new` accepts arbitrary rectangles and `ScrollGeometryOf::new` accepts independently supplied scrollport, clip, overflow, range, and gutters. | Make derived geometry output-only and construct it through one canonical crate-owned factory. |
| `E-RECT-END` | `ScrollRectOf::new` validates origin and size components but not finite `origin + size` endpoints. | Validate every endpoint and return a typed construction error. |
| `E-RANGE-DIRECTION` | `physical_scroll_range_from_overflow_rects` measures only physical right/bottom overflow before projecting through `FlowAxes`. | Measure the applicable formatting-context scroll-origin end side, admit the bounded content-distribution start side, then preserve FRI-02 flow-relative projection. |
| `E-ALIGNMENT-ORIGIN` | Existing flex/grid content distribution can position an overflowing in-flow subject across the initial scrollport, while current range construction has only a zero-to-end extent. | Retain zero as the initial anchor and derive both range bounds from the final alignment subject, complete overflow, and format-specific scroll-origin progression. |
| `E-ONE-RECT-CLIP` | `ScrollGeometryOf` stores one optional clip rectangle although `visible/clip` mixed pairs require clipping only one axis. | Store independent optional physical x/y clip intervals. |
| `E-BLOCK-MARGIN` | Block overflow accumulation adds signed margin sums to size and can construct a negative-size rectangle. | Union the border area with only positive margin outsets; negative margins may position a box but never invalidate its area. |
| `E-SMALL-BOX` | Some block content-box subtraction is not saturated before scroll geometry construction. | Saturate effective edge reservations and every derived box dimension at zero. |
| `E-FLEX-GRID-OUTPUT` | Flex and grid child/container paths write `scroll_geometry: None`, while block has local accumulation logic. | Move contribution into a shared substrate and emit geometry for every laid-out box in those formats. |
| `E-ZERO-AXIS` | Block, flex, and grid helpers return `Size::ZERO` when either source dimension is non-positive. | Track and finalize physical x/y intervals independently. |
| `E-GRID-ORIGIN` | Ordinary grid and grid-lanes subtract the grid-area origin before adding a child's contribution. | Translate contribution by the final container-relative child location. |
| `E-OUTPUT-HELPER` | `NodeOutputOf::content_box_size()` subtracts padding and border but not `scrollbar_size`. | Derive the helper from canonical box geometry and remove independently mutable scrollbar size. |
| `E-UNUSED-EXPECTATION` | `Expectation::scroll_size` is parsed but the comparator checks only x, y, width, height, and children. | Compare both expected scroll deltas to physical range spans and fail when geometry is absent or mismatched. |

At this base the corpus contains 1,409 HTML sources, 5,280 generated XML
outputs, 356 unsupported cases, zero expected failures, zero quarantined cases,
and one canonical full generation report. Exactly 312 checked-in XML files carry
paired `scroll_width` and `scroll_height` expectations. These counts describe
the input baseline; the final report owns the post-fixture counts.

## FRI-05.4 Resolved Design Decisions

### `D-01` Node Input Carries Canonical Computed Overflow

`Overflow` remains one public closed semantic choice and adds `Auto`:

```rust
pub enum Overflow {
    Visible,
    Clip,
    Hidden,
    Scroll,
    Auto,
}

pub struct ComputedOverflow { /* private x/y */ }

pub enum ComputedOverflowError {
    NonCanonicalPair { x: Overflow, y: Overflow },
}

impl ComputedOverflow {
    pub const VISIBLE: Self;
    pub fn try_new(x: Overflow, y: Overflow) -> Result<Self, ComputedOverflowError>;
    pub const fn x(self) -> Overflow;
    pub const fn y(self) -> Overflow;
}
```

The constructor accepts exactly thirteen computed pairs:

- all four pairs drawn from `{ Visible, Clip }`; and
- all nine pairs drawn from `{ Hidden, Scroll, Auto }`.

It rejects the twelve cross-group pairs. Those pairs still require CSS
specified-to-computed coupling and therefore are not layout-ready. The
constructor does not silently normalize them because layout does not own
specified CSS values.

`NodeInputOf::overflow` changes from `Point<Overflow>` to `ComputedOverflow`.
Its default is `ComputedOverflow::VISIBLE`. Callers cannot mutate one axis after
construction.

The public `Overflow::is_scrollable()` predicate means the CSS computed-value
class and returns true for `Hidden`, `Scroll`, and `Auto`. Grid and flex
automatic-minimum logic uses the computed value through this predicate.

Layout derives a private used-overflow pair for geometry. On a replaced box,
each computed `Hidden` axis becomes used `Clip`; all other values remain
unchanged. This conversion does not mutate the public input and does not affect
the computed-overflow automatic-minimum predicate.

Every other current overflow decision names its phase:

| Decision | Owning value | Exact rule |
| --- | --- | --- |
| Grid content-based automatic minimum | One computed `Overflow` axis selected through the grid container's `FlowAxes` and `GridAxisKind` | `Hidden`, `Scroll`, and `Auto` are scrollable; `Visible` and `Clip` are non-scrollable. |
| Flex main-axis automatic minimum | The complete post-coupling `ComputedOverflow` pair | Construction guarantees `x.is_scrollable() == y.is_scrollable()`. Both callers use one pair classifier: every `Hidden`/`Scroll`/`Auto` pair zeros the automatic minimum, while every `Visible`/`Clip` pair retains the content-based minimum. No physical-axis projection can change that Boolean. |
| Grid intrinsic/min-content and percent-track contribution | One used-overflow axis selected through the grid container's `FlowAxes` | Only `Visible` admits nested `content_size`; `Clip`, `Hidden`, `Scroll`, and `Auto` trap it and use the item box/min-track priority. |
| Independent formatting context for a non-replaced block container | Complete `ComputedOverflow` pair | Every canonical pair from the scrollable group establishes one; every canonical `Visible`/`Clip` pair does not. |
| Block child-margin collapse | Complete `ComputedOverflow` pair | Collapse is blocked exactly when the pair establishes that independent formatting context. |
| Descendant clipping | One private used-overflow axis | `Clip`, `Hidden`, `Scroll`, and `Auto` clip; `Visible` does not. |
| Programmatic range | One private used-overflow axis | `Hidden`, `Scroll`, and `Auto` expose range; `Visible` and `Clip` do not. |
| Classic gutter/UI policy | One private used-overflow axis plus gutter state and measured overflow | `Scroll` is forced, `Auto` is conditional, `Hidden` is stable-gutter-only, and `Visible`/`Clip` never reserve. |

The public per-axis `clips_contents()` and `blocks_margin_collapse()` methods are
removed. They conflate computed and used phases. `ComputedOverflow` exposes the
pair-level independent-formatting-context predicate used by block; private
used-axis methods own clip, range, and gutter decisions.

A replaced computed `Hidden` axis is converted to used `Clip` before clip,
range, or gutter policy. It therefore uses the overflow clip edge, exposes no
range, and reserves no stable gutter. Its computed value remains scrollable for
the grid/flex automatic-minimum condition. Replaced boxes do not enter the block
child-margin-collapse decision.

Grid's intrinsic callers use this complete used-axis matrix:

| Computed axis | Ordinary used axis | Replaced used axis | Traps descendant `content_size` | Min-content span priority and percent-track branch |
| --- | --- | --- | --- | --- |
| `Visible` | `Visible` | `Visible` | No; use `max(item_size, content_size)` | Non-clipping intrinsic-track path |
| `Clip` | `Clip` | `Clip` | Yes; use `item_size` | Clipping min/max-content priority path |
| `Hidden` | `Hidden` | `Clip` | Yes; use `item_size` | Clipping min/max-content priority path |
| `Scroll` | `Scroll` | `Scroll` | Yes; use `item_size` | Clipping min/max-content priority path |
| `Auto` | `Auto` | `Auto` | Yes; use `item_size` | Clipping min/max-content priority path |

One crate-private helper derives the used axis from `ComputedOverflow`,
`item_is_replaced`, the grid container's `FlowAxes`, and `GridAxisKind`.
Columns select the container inline physical axis and rows select its block
physical axis. Ordinary-grid, intrinsic-subgrid, and grid-lanes callers use
that helper; no context-free column-to-x/row-to-y overflow match remains.

Flex has one crate-private canonical-pair classifier shared by
`automatic_min_main_size` and the intrinsic main-size contribution branch. It
receives the item's `ComputedOverflow` and returns the common computed
scrollability class guaranteed by `ComputedOverflow::try_new`; it does not need
`FlexAxes` to choose a physical axis because no valid pair can change the result.
The exhaustive thirteen-pair constructor evidence plus real flex front-door
controls for both pair groups proves the branch. Main-only and cross-only
scrollability remain rejected pre-coupling input, not a layout test state.

Rejected alternative: retaining `Point<Overflow>` permits a post-construction
mixed pair that contradicts computed CSS.

Rejected alternative: normalizing mixed specified values in layout would move
authored-style computation into the leaf crate and make fixture/root behavior
diverge.

### `D-02` Scroll Properties Are Layout-Ready Closed Values

The following public input types are owned by `surgeist-layout`:

```rust
pub enum OverflowClipBox {
    ContentBox,
    PaddingBox,
    BorderBox,
}

pub struct OverflowClipMarginOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub enum ScrollbarGutter { Auto, Stable, StableBothEdges }
pub enum ScrollPaddingValueOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub struct ScrollPaddingOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub struct ScrollMarginOf<S: LayoutScalar = DefaultScalar> { /* private */ }

pub enum ScrollSnapAxis { X, Y, Block, Inline, Both }
pub enum ScrollSnapStrictness { Proximity, Mandatory }
pub enum ScrollSnapType { None, Enabled { axis: ScrollSnapAxis, strictness: ScrollSnapStrictness } }
pub enum ScrollSnapAlignValue { None, Start, End, Center }
pub struct ScrollSnapAlign { /* private block/inline */ }
pub enum ScrollSnapStop { Normal, Always }
```

The scalar-bearing types have the standard default-scalar aliases. Their
construction and defaults are:

| Type | Valid construction | Default |
| --- | --- | --- |
| `OverflowClipMarginOf<S>` | One `OverflowClipBox` and one finite non-negative absolute length. | `PaddingBox`, zero. |
| `ScrollbarGutter` | Closed enum; `both-edges` exists only as `StableBothEdges`. | `Auto`. |
| `ScrollPaddingValueOf<S>` | `Auto` or a validated `LengthPercentageOf<S>`. Resolution clamps a negative used result to zero. | `Auto`. |
| `ScrollPaddingOf<S>` | Four physical `ScrollPaddingValueOf<S>` edges. | All `Auto`. |
| `ScrollMarginOf<S>` | Four finite signed absolute physical edge outsets. | All zero. |
| `ScrollSnapType` | `None` or one axis plus one explicit strictness. | `None`. |
| `ScrollSnapAlign` | One block and one inline alignment value. | `None`/`None`. |
| `ScrollSnapStop` | Closed enum. | `Normal`. |

The corresponding `NodeInputOf<S>` fields are:

```rust
pub overflow: ComputedOverflow,
pub overflow_clip_margin: OverflowClipMarginOf<S>,
pub scrollbar_gutter: ScrollbarGutter,
pub scrollbar_width: ScrollbarWidthOf<S>,
pub scroll_padding: ScrollPaddingOf<S>,
pub scroll_margin: ScrollMarginOf<S>,
pub scroll_snap_type: ScrollSnapType,
pub scroll_snap_align: ScrollSnapAlign,
pub scroll_snap_stop: ScrollSnapStop,
```

`ScrollbarWidthOf<S>` continues to mean the already-resolved non-negative finite
classic scrollbar thickness in layout units. Zero means overlay/no layout
reservation. Root lowers authored `auto`, `thin`, `none`, platform metrics, and
overlay policy to that scalar.

Scroll-padding percentages resolve against the corresponding physical
scrollport dimension: left/right against width and top/bottom against height.
The layout product policy resolves `auto` to zero; the standard permits a UA
heuristic but does not require one. Scroll padding changes only the optimal
viewing region and snapport. It does not change layout size, scroll origin,
range, or actual visibility.

Scroll margin remains signed because the property accepts signed absolute
lengths. It is preserved as an outset value rather than prematurely converted
to a rectangle before root applies any transform.

Rejected alternative: keeping these values in `ScrollUnsupportedFeature`
would leave well-formed required input unrepresentable and would not close
`OVERFLOW-002`.

Rejected alternative: precomputing snap positions in layout requires retained
identity, ancestor capture, transforms, and host policy that this crate does not
own.

### `D-03` Output Separates Container Geometry From Target Geometry

`ScrollGeometryOf<S>` is immutable container/output geometry. It exposes:

- the retained `FlowAxes`;
- used x/y overflow through `used_overflow_x()` and `used_overflow_y()`;
- canonical border, padding, content, and scrollport rectangles;
- independent optional physical x and y clip intervals;
- the complete local scrollable-overflow rectangle;
- the signed `PhysicalScrollRangeOf<S>`;
- canonical physical edge gutter rectangles and their aggregate reservation;
- resolved physical scroll-padding edges;
- the optimal viewing region; and
- the container's `ScrollSnapType`.

The exact read-only accessor surface is:

```rust
pub const fn flow_axes(self) -> FlowAxes
pub const fn used_overflow_x(self) -> Overflow
pub const fn used_overflow_y(self) -> Overflow
pub const fn border_box(self) -> ScrollRectOf<S>
pub const fn padding_box(self) -> ScrollRectOf<S>
pub const fn content_box(self) -> ScrollRectOf<S>
pub const fn scrollport(self) -> ScrollRectOf<S>
pub const fn overflow_clip(self) -> OverflowClipOf<S>
pub const fn scrollable_overflow(self) -> ScrollRectOf<S>
pub const fn physical_range(self) -> PhysicalScrollRangeOf<S>
pub const fn gutters(self) -> ScrollbarGutterRectsOf<S>
pub const fn scrollbar_size(self) -> Size<S>
pub const fn resolved_scroll_padding(self) -> Edges<S>
pub const fn optimal_viewing_region(self) -> ScrollRectOf<S>
pub const fn scroll_snap_type(self) -> ScrollSnapType
pub const fn target(self) -> ScrollTargetGeometryOf<S>
```

`ScrollbarGutterRectsOf<S>` is the immutable physical-edge gutter output. It
has private `top`, `right`, `bottom`, and `left` optional rectangles and exposes
only matching `top()`, `right()`, `bottom()`, and `left()` accessors. It has no
public constructor and no `Default`. `ScrollGeometryOf::scrollbar_size()` is the
aggregate physical reservation derived from those effective edges, not the
requested classic scrollbar thickness.

The value privately retains one canonical source record containing the final
border size, resolved edges, effective gutter state, used overflow, clip margin,
resolved scroll padding, finite target scroll margin, accumulated overflow,
propagatable descendant intervals, `ScrollOriginAxes`, active alignment subject,
container snap type, and target snap alignment/stop. Rounding rebuilds from that
record. These private provenance fields have no independent public setters or
duplicate constructor path; public accessors expose only the derived contract
above.

The output clip is not one optional rectangle. It is a public read-only value
with one optional finite ordered interval per physical axis:

```rust
pub struct PhysicalClipAxisOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub struct OverflowClipOf<S: LayoutScalar = DefaultScalar> { /* private x/y */ }
```

`None` on an axis means overflow is visible on that axis. `Some(interval)` means
descendant overflow is intersected with that interval on that axis. This
represents `visible/clip` and `clip/visible` without falsely clipping both axes.

Each laid-out box also returns immutable `ScrollTargetGeometryOf<S>` containing:

- its local physical border box;
- its finite signed physical scroll-margin outsets;
- its own `FlowAxes`;
- block/inline snap alignment; and
- snap stop.

This target value is emitted even when snap alignment is `none`, because scroll
margin also affects root-owned focus, target, and scroll-into-view operations.
Root maps the border box through transforms, expands the resulting axis-aligned
bounds by scroll margin, and chooses live scroll or snap positions. Layout does
not retain an ancestor association.

Root applies two distinct ancestor rules. Focus, fragment targeting, and
`scrollIntoView` use root's nearest-scroll-container policy and use the expanded
target area even when snapping is disabled. Snap association instead walks the
containing-block chain to the nearest snap-position-capturing ancestor. A box
captures when it is a scroll container or its computed `scroll-snap-type` is not
`None`. The target has a snap container only when that nearest capturer is both
a scroll container and has non-`None` snap type. A nearer non-scroll capturer,
or a nearer scroll container with `None`, is a capture barrier that yields no
snap container; root must not skip it to reach a farther enabled scroller.

When a snap container exists, its `ScrollSnapType::Enabled` `Block` and `Inline`
axes are interpreted through that container's retained `FlowAxes`. The two
`ScrollSnapAlign` values likewise mean block and inline in the snap container's
writing mode for an ordinary target. Root first transforms the target border
box into container coordinates and then applies the target's physical scroll
margin to form the axis-aligned snap area. If that final snap area is larger
than the snapport in a relevant physical axis, `Start` and `End` on that axis
are interpreted through the target's own retained `FlowAxes`; `Center` remains
physical center alignment. Oversize is decided independently per physical
axis, so an orthogonal or transformed target can use container interpretation
on one axis and target interpretation on the other. Root still owns the set of
valid positions across an oversized area and final live snap selection.

`ScrollGeometryOf<S>` owns exactly one target value and exposes it through:

```rust
pub const fn target(self) -> ScrollTargetGeometryOf<S>
```

There is no parallel optional target field. The concrete carriers are the
existing `ComputeOutputOf::scroll_geometry` and `NodeOutputOf::scroll_geometry`
options: `Some(geometry)` always includes `geometry.target()`, while `None`
means neither container nor target geometry exists. Every performed-box path
that converts a `ComputeOutputOf` into a `NodeOutputOf` copies the complete
option. A path that must rebuild geometry after final border-box resolution
does so through the same canonical factory and rebuilds the nested target in
the same operation; it may not preserve container geometry while dropping or
defaulting the target.

`NodeOutputOf<S>` retains `scroll_geometry: Option<ScrollGeometryOf<S>>` because
non-box control outputs and unlaid/default outputs have no box geometry. Every
successfully performed box layout, including a visible-overflow box, returns
`Some`. `display:none`, line-break controls, inline-boundary controls, and
measurement-only results that do not produce a box return `None`.

Compute and node caches retain the complete output value, including the nested
target, with no target-specific side cache. Rounding rebuilds the target inside
the rounded `ScrollGeometryOf`; it cannot create a target for a `None` output or
turn a present target into absence.

`NodeOutputOf::scrollbar_size()` replaces the independently mutable public
`scrollbar_size` field and derives aggregate physical reservation from
`scroll_geometry`. `NodeOutputOf::content_box_size()` returns the canonical
content-box size when geometry exists. Its no-geometry fallback subtracts
padding and border and saturates each axis at zero.

Rejected alternative: placing a registry or list of descendant targets inside
each container geometry would duplicate tree identity and create retained snap
state in the leaf.

### `D-04` Derived Geometry Has One Canonical Factory

`ScrollRectOf::new` is replaced by `ScrollRectOf::try_new` with a scalar-generic
`ScrollRectErrorOf<S>`. Construction rejects:

- non-finite origin components;
- non-finite size components;
- negative size components; and
- a non-finite physical end produced by `origin + size`.

Signed zero is canonicalized. Zero width, zero height, and a zero-area rectangle
are valid.

The public constructors for `ScrollbarGutterRectsOf` and
`ScrollGeometryOf` are removed. Their fields remain private. One crate-owned
factory receives only source facts:

- flow axes, `ComputedOverflow`, and replaced-box status, from which the factory
  derives its private used-overflow pair;
- final border-box size;
- resolved padding and border edges;
- effective scrollbar state and thickness;
- overflow clip margin;
- resolved scroll padding;
- finite target scroll margin, snap alignment, and snap stop;
- final accumulated scrollable overflow;
- format-specific scroll-origin progression and any active final in-flow
  content-distribution subject; and
- scroll snap type.

It derives every output rectangle, clip interval, range, and gutter. Callers do
not pass derived parts back into it. The factory either returns one coherent
geometry value or a private geometry error mapped to the existing contextual
`LayoutInternalInvariant` site for root, block, flex, or grid. A well-formed
public FRI-05 input never returns `UnsupportedCapability`.

The factory enforces these invariants:

1. every rectangle and interval is finite and ordered;
2. the padding box is the border box inset by effective border edges;
3. the content box is the border box inset by border, padding, and effective
   gutter edges;
4. the scrollport is the padding box inset by effective gutter edges;
5. each gutter is edge-aligned to the padding box and has exactly the effective
   edge thickness;
6. opposing effective gutter edges never sum beyond the padding-box dimension;
7. visible axes have no clip and exactly a zero range;
8. clip axes use the overflow clip edge and exactly a zero range;
9. hidden, scroll, and auto axes clip to the scrollport and expose only the
   canonically derived range;
10. replaced computed hidden axes obey their used clip behavior;
11. the optimal viewing region is an inset of the scrollport and never has a
    negative size; and
12. the physical range is exactly the `FlowAxes` projection of the derived
    ordered flow-relative bounds from non-negative origin-start and origin-end
    extents.

`ScrollUnsupportedFeature` and `ScrollOverflowCouplingPolicy` are removed. The
first no longer describes unsupported behavior, and the second is replaced by
the validated computed pair plus real auto-gutter iteration.

Rejected alternative: broadening validation in the existing arbitrary
`ScrollGeometryOf::new` still allows two independently calculated but mutually
inconsistent sources of truth.

### `D-05` Small Boxes Use Proportionally Saturated Edge Reservations

All box derivation is per axis and saturating. Border, padding, and requested
gutter insets are non-negative, but their sum may exceed a final small border or
padding box.

For each pair of opposing requested gutter edges:

1. retain the requested values when their sum is no greater than the available
   padding-box dimension;
2. when the sum exceeds that dimension, multiply both by
   `dimension / requested_sum`; and
3. when the available dimension or requested sum is zero, use zero effective
   insets.

This preserves one-sided placement, preserves symmetry for `both-edges`, and
ensures effective insets sum to exactly the available dimension. Derived
content and scrollport sizes then saturate at zero without overlapping or
negative gutter geometry. A `2px` box with a `15px` one-sided classic request
therefore has a `2px` effective gutter and a zero content-box width rather than
an invalid `-13px` width.

Border and padding inset derivation uses the same non-negative saturation rule
for rectangles, but authored border and padding output values are not rewritten.

### `D-06` Scrollbar Placement And Auto Coupling Are Flow-Aware

Scrollbar reservation is expressed in logical roles and then placed through
`FlowAxes`:

- a scrollbar for block-axis overflow occupies the inline-end padding edge;
- a scrollbar for inline-axis overflow occupies the block-end padding edge;
- `StableBothEdges` mirrors an existing inline-end gutter on the inline-start
  edge; and
- it does not mirror the block-end gutter.

The corresponding physical x/y overflow value is selected through the
container's `FlowAxes`; formatting code does not repeat physical direction
matches.

The block-axis scrollbar reservation matrix is:

| Used block-axis overflow | `Auto` gutter | `Stable` gutter | `StableBothEdges` gutter |
| --- | --- | --- | --- |
| `Visible` or `Clip` | None | None | None |
| `Hidden` | None | Inline end | Inline end plus inline start |
| `Scroll` | Inline end | Inline end | Inline end plus inline start |
| `Auto`, zero derived range span | None | Inline end | Inline end plus inline start |
| `Auto`, non-zero derived range span | Inline end | Inline end | Inline end plus inline start |

This matrix receives the private used-overflow value. A replaced computed
`Hidden` axis has already become used `Clip`, so it follows the no-gutter row.

The inline-axis scrollbar, which occupies block-end, is reserved for `Scroll`
and for `Auto` with a non-zero derived range span. That span includes the
bounded content-distribution start extent and ordinary origin-end extent.
`Hidden`, `Visible`, and `Clip` do not reserve it. `scrollbar-gutter: stable`
does not force or mirror a block-edge gutter.

Zero `ScrollbarWidthOf` produces no layout reservation and no gutter rectangle
even when a scrollbar is logically present. Overflow clipping and range remain
correct.

Each block, flex, and grid formatting algorithm resolves `Auto` with the same
private monotone state machine:

1. start with forced `Scroll` reservations and stable block-axis reservations,
   but no conditional auto reservation;
2. perform layout into temporary pass output;
3. derive the provisional start/end range span against that pass's scrollport;
4. add each newly required `Auto` reservation;
5. rerun only when a newly added reservation has non-zero thickness and changes
   available geometry; and
6. publish and cache only the first stable pass.

The two conditional axis bits only transition from absent to present. Therefore
at most three geometry-changing layout evaluations are possible: initial, one
axis induced, and the other axis induced. Both axes may settle earlier. No
retry limit, tolerance, or non-convergence error is needed.

Speculative passes do not populate shared final-output caches under the original
request. Any inner cache used during a pass either includes the scrollbar-state
bits in its key or is pass-local. The stable result alone is committed under the
ordinary request, preserving cached/uncached equivalence.

Rejected alternative: reserving `Auto` unconditionally repeats the current
fixture parser's incorrect alias to `Scroll`.

Rejected alternative: running the entire layout twice unconditionally makes a
diagnostic technique part of ordinary verification and still misses cross-axis
induction that needs a third state.

### `D-07` Clip Edges Are Per Axis

For used `Hidden`, `Scroll`, or `Auto`, the clip interval on that physical axis
is the corresponding scrollport interval. Overflow clip margin has no effect.

For used `Clip`, the interval is derived from the selected content, padding, or
border reference box and expanded by the finite non-negative clip-margin
length. The expansion is applied only on the clipped physical axis. For used
`Visible`, the interval is absent.

Clip intervals describe the child's own rendering/scrollport geometry for root;
they are not parent-propagation bounds. Parent propagation has a separate
eligibility decision. On each physical axis, only used `Visible` permits the
child's propagatable descendant interval to enlarge the parent. Used `Clip`,
`Hidden`, `Scroll`, and `Auto` trap descendant scrollable overflow on that axis,
so the parent receives no nested interval there rather than an intersected one.

The canonical child geometry retains crate-private propagatable descendant
intervals separately from its full scrollable-overflow rectangle. They contain
direct line/descendant contributions and transitive used-visible overflow, but
exclude the child's own padding, border, and margin areas. A parent translates
only those intervals to parent-local coordinates. This distinction lets a
zero-area box propagate real visible descendant overflow without treating its
own zero-area padding interval as descendant evidence.

The child's own positive-area border or applicable margin area is included
separately regardless of whether it traps descendants. A margin area is eligible
only when the child's border box itself has positive area; margins cannot revive
a zero-area own-box contribution. This per-axis operation applies to
`visible/clip`, `clip/visible`, and the fully scrollable computed-pair group. A
trapped child cannot enlarge the parent through nested overflow or activate the
parent's auto gutter merely because a formatting helper preserved its local clip
rectangle.

### `D-08` One Shared Accumulator Owns Contribution Semantics

`src/scroll.rs` owns one crate-private accumulator used by root, block, flex,
ordinary grid, subgrid, and grid-lanes. It tracks minimum and maximum physical
coordinates independently on x and y, plus the final in-flow end edges needed
for terminal padding. For each currently implemented content-distribution axis,
it also accepts the final bounding interval of the in-flow alignment subject,
separate from full scrollable overflow. It does not use `Size::ZERO` as an
all-axis early return.

The contribution matrix is:

| Source | Inclusion rule |
| --- | --- |
| Container's own padding box | Always seeds the accumulator, including a zero-size box. |
| Direct line box available from the current block/inline bridge | Include its area in container-local coordinates. FRI-06 still owns missing line construction. |
| Child border box | Include only when width and height are both positive, matching the zero-area exclusion. |
| Flex/grid item margin area | Only when the border box itself has positive area, union it with positive physical margin outsets. Negative margins never shrink the border contribution or create an inverted rectangle. |
| Other in-flow block margin | Only when the border box itself has positive area, keep the existing browser-backed block inclusion policy, implemented through border area plus positive outsets so a valid negative margin cannot fail. |
| Current absolute/out-of-flow child laid out against this containing block | When its border box has positive area, include its final margin area exactly once. Later positioned-layout completeness remains FRI-10-owned. |
| Child propagatable descendant overflow | On each used-`Visible` axis, include the child's separate descendant interval even when its border box has zero area; translate by final child location. On every trapped axis, include no nested interval. |
| Terminal own padding | Extend the final in-flow/floated logical end edge by the corresponding own padding, unless another included source already reaches farther. |
| Active content-distribution subject | Record the final in-flow subject bounds for range-origin adjustment; exclude out-of-flow boxes and nested overflow beyond that subject. |

The margin-area operation is equivalent to unioning the border box with a
well-ordered margin box: only positive outsets can enlarge the union. Signed
margins continue to affect the child's final location through the formatting
algorithm; they simply cannot turn the synthetic contribution rectangle
negative.

Propagatable geometry carries transitive descendants only through used-visible
axes. Each formatting algorithm adds each direct child and each out-of-flow box
it owns exactly once; it does not walk a descendant a second time.

The accumulator retains physical start-side coordinates, including negative
origins, because a parent with a reversed progression can observe them as its
logical end side. Reachability is decided only when a particular container's
range is derived.

Rejected alternative: taking only `child.location + child.content_size` loses
negative origins, child clips, and transitive overflow.

Rejected alternative: ignoring every zero-area child also ignores the standard
case where a zero-area box has non-zero descendant scrollable overflow.

### `D-09` Ranges Own Format Origins And Alignment Overflow

The initial layout scroll offset is physical zero. Every range contains zero.
Root can later choose another current or snap-initial offset inside that range,
but layout does not retain it.

One private `ScrollOriginAxes` describes, for each flow-relative inline/block
axis, whether the formatting context's scroll-origin progression is flow-endward
or flow-startward:

| Formatting context | Origin progression |
| --- | --- |
| Root, leaf, block, ordinary grid, subgrid, and grid-lanes | Inline-start to inline-end and block-start to block-end from `FlowAxes`. |
| Flex main axis | Main-start to main-end from the completed `FlexAxes`; row/column reverse can make this flow-startward. |
| Flex cross axis | Cross-start to cross-end from `FlexAxes`; wrap-reverse can make this flow-startward. |

Scrollbar placement still follows writing-mode `FlowAxes` as specified in D-06.
Only range reachability uses the formatting context's scroll origin.

For each exposed origin-relative axis, where positive means origin-start toward
origin-end, the factory derives:

```text
end_extent = overflow beyond the scrollport's origin-end edge
start_extent = active alignment subject beyond the scrollport's origin-start edge
origin_relative_range = [-start_extent, end_extent]
```

For an increasing physical origin progression, the exact magnitudes are
`max(overflow.end - scrollport.end, 0)` and, when active,
`max(scrollport.start - subject.start, 0)`. For a decreasing physical origin
progression they are `max(scrollport.start - overflow.start, 0)` and, when
active, `max(subject.end - scrollport.end, 0)`, respectively.

`end_extent` uses the complete accumulated scrollable-overflow rectangle.
`start_extent` is non-zero only when the formatting algorithm actually applied a
non-normal content-distribution value on that axis and its final in-flow
alignment subject overflows origin-start. It is exactly the travel needed to
place that subject's origin-start edge against the scrollport's origin-start
edge.

The current content-distribution mapping is finite:

- flex `justify_content: Some` supplies the final main-axis subject;
- flex `align_content: Some` supplies the final cross-axis line subject only
  when that property applies to the formed lines;
- grid `justify_content: Some` supplies the final inline track subject;
- grid `align_content: Some` supplies the final block track subject; and
- `None`, an inapplicable property, and a safe fallback that actually placed the
  subject at origin-start supply zero start extent.

The published block source does not consume `align_content`, so block supplies
no alignment adjustment in FRI-05. FRI-05 does not add that missing block
alignment algorithm. Item self-alignment, absolute alignment, and new alignment
values also do not activate this adjustment; their final boxes remain ordinary
contribution and FRI-09/FRI-10 keep their behavior ownership.

An out-of-flow box or descendant overflow farther into origin-start than the
in-flow alignment subject remains unreachable. It can enlarge `end_extent` only
when it lies toward origin-end. This preserves the CSS distinction between the
alignment subject and the complete scrollable-overflow area.

The origin-relative interval is converted to a flow-relative interval before
public output:

- when origin progression is flow-endward, retain `[-start_extent, end_extent]`;
- when origin progression is flow-startward, negate and swap it to
  `[-end_extent, start_extent]`; and
- when the axis does not expose range, use `[0, 0]`.

Layout constructs one validated `FlowRelativeScrollRangeOf<S>` from those
inline/block bounds and obtains the stored physical range only through
`FlowAxes::physical_scroll_range`. FRI-02's signed types, ordering,
canonical-zero behavior, conversion, clamp, and physical projection therefore
remain authoritative.

Consequences are exact:

- ordinary flow-endward overflow produces `[0, end_extent]` in that logical
  axis;
- a reversed flex-origin axis can produce `[-end_extent, 0]` before physical
  projection;
- unsafe end or center content distribution can produce a negative start bound
  while retaining zero as the initial anchor;
- center distribution can produce non-zero bounds on both sides of zero;
- hidden, scroll, and auto expose a programmatic range;
- visible and clip have `[0, 0]` even when overflow exists;
- start-side overflow outside the bounded alignment subject remains
  unreachable; and
- the browser delta remains `maximum - minimum` on the corresponding physical
  axis.

The accumulator's complete rectangle can still propagate to a visible ancestor;
local reachability never destroys source geometry. Rounding repeats the same
origin/subject derivation from rounded primitives before FRI-02 projection.

FRI-09 may add alignment states or correct how a formatting algorithm positions
its subject. It must feed the resulting subject into this FRI-05 origin contract;
it does not own a second range convention or defer the existing alignment-origin
behavior defined here.

### `D-10` Box And Content Extents Are Axis-Independent

The canonical output content box is derived from final border size and effective
edges. Every subtraction saturates x and y independently.

`NodeOutputOf::content_size` remains a physical layout-content extent, distinct
from range. It is derived as the size of the minimal axis-aligned union of the
content-box origin anchor and the accumulated scrollable-overflow rectangle.
Each axis is calculated independently. A `0x10` child whose nested overflow
reaches `100px` vertically can therefore contribute `0x100`; zero x never
forces y to zero.

Grid, grid-lanes, and subgrid translate child contribution by the child's final
container-local location. They do not subtract the grid-area origin. An item at
container x `50px` with an `80px` contribution reaches x `130px`, regardless of
where its grid area began.

Range remains the authoritative scroll delta because it also accounts for
scrollport, direction, clipping mode, and unreachable overflow. Browser parity
never substitutes `content_size` for range.

### `D-11` Rounding Rebuilds Derived Geometry

Rounding operates on the source border size, resolved edges, effective
scrollbar state, accumulated overflow, format-origin progression, active
alignment subject, scroll padding, and target border box in the existing
cumulative-origin coordinate system. It then invokes the same canonical
factories.

It does not independently round a scrollport, gutter, clip, range, and optimal
viewing region and combine them through a public constructor. Rounded geometry
therefore obeys the same coherence invariants as unrounded geometry.

Range is recomputed from rounded scrollport, overflow, and alignment-subject
bounds in the retained format origin, then projected through the retained
`FlowAxes`. Target geometry rounds its local border box; finite scroll-margin
input and semantic snap values are retained.

### `D-12` Browser Scroll Expectations Compare Range Span

The generator's existing expectation meaning remains:

```text
scroll_width  = max(element.scrollWidth  - element.clientWidth,  0)
scroll_height = max(element.scrollHeight - element.clientHeight, 0)
```

The parity comparator performs the following whenever
`Expectation::scroll_size` is `Some`:

1. require `NodeOutputOf::scroll_geometry` to be `Some`;
2. compute physical x span as `range.x().maximum() - range.x().minimum()`;
3. compute physical y span as `range.y().maximum() - range.y().minimum()`;
4. compare expected width to x span and expected height to y span using the
   established scalar tolerance and mismatch reporting; and
5. continue ordinary child comparison only after both pass.

Expected zero is observed, not treated as absence. Missing output geometry is a
named scroll-geometry mismatch. The comparator does not use a signed endpoint,
absolute endpoint, `content_size`, or `scrollable_overflow.size()` as a proxy.

The helper continues to emit the paired attributes only for browser computed
overflow that can be a scroll container (`hidden`, `scroll`, or `auto`). The XML
parser continues to require both attributes together.

### `D-13` Fixture Lowering Is Narrow And Computed-Style Based

The browser helper records these additional computed-style fields:

- `overflowClipMargin`;
- `scrollbarGutter`;
- physical `scrollPaddingTop/Right/Bottom/Left`;
- physical `scrollMarginTop/Right/Bottom/Left`;
- `scrollSnapType`;
- `scrollSnapAlign`; and
- `scrollSnapStop`.

The existing Rust generator serializer emits corresponding kebab-case fixture
attributes only when they differ from the FRI-05 defaults. If either overflow
axis is non-default, it emits both computed axes so the parser constructs one
pair atomically.

The fixture parser accepts only the finite computed forms required by the named
sources:

- overflow keywords `visible`, `clip`, `hidden`, `scroll`, and `auto`;
- clip boxes `content-box`, `padding-box`, and `border-box` with one finite
  non-negative px length;
- gutter values `auto`, `stable`, and canonical `stable both-edges`;
- scroll-padding `auto`, finite px, finite percentage, or existing affine
  length-percentage syntax;
- finite signed px scroll margins;
- snap type `none` or one supported axis plus explicit computed strictness;
- two computed snap-align keywords in block/inline order; and
- snap stop `normal` or `always`.

The parser does not accept authored shorthand ambiguity, CSS-wide keywords,
relative units, `var()`, a transform, an arbitrary CSS token stream, or a value
outside the production type's invariant. Such authored lowering remains root
owned.

Rejected alternative: placing parser-only variants in production enums would
mix fixture and normalized phases.

## FRI-05.5 Public Contract

The public front door reexports the types named in Sections D-01 through D-04,
their default-scalar aliases, `ScrollCoordinateErrorOf`, and the signed scroll
coordinate types completed by FRI-02.

The breaking API delta is:

| Current public surface | FRI-05 surface |
| --- | --- |
| `NodeInputOf::overflow: Point<Overflow>` | `NodeInputOf::overflow: ComputedOverflow` |
| No clip-margin/gutter/padding/margin/snap fields | The exact normalized fields in D-02 |
| `Overflow` without `Auto` | `Overflow` with `Auto` and corrected predicates |
| Per-axis public `clips_contents()` and `blocks_margin_collapse()` | Pair-level computed IFC/BFC predicate plus private used-axis clip/range/gutter predicates |
| Public `ScrollRectOf::new` returning `ScrollUnsupportedFeature` | `ScrollRectOf::try_new` returning `ScrollRectErrorOf<S>` |
| Public `ScrollbarGutterRectsOf::new` and horizontal/vertical-only output | No public constructor; private physical-edge output with `top()`/`right()`/`bottom()`/`left()` accessors |
| Public `ScrollGeometryOf::new` | No public constructor; layout factory output with the exact D-03 accessors only |
| Exported `ScrollOverflowExposure`, `ScrollContainerAxis`, `ScrollContainerFacts`, their public constructors, crate-visible `scroll_container_facts_from_overflow`, and `ScrollGeometryOf::container()` | Removed; private used-overflow derivation plus read-only `ScrollGeometryOf::used_overflow_x()` and `used_overflow_y()` |
| One `Option<ScrollRectOf>` overflow clip | `OverflowClipOf<S>` with independent x/y intervals |
| Public mutable `NodeOutputOf::scrollbar_size` field | Derived `NodeOutputOf::scrollbar_size()` method |
| `compute_leaf` accepts a one-shot `FnOnce` measurement callback | `compute_leaf` accepts `FnMut(LeafMeasureInputOf<S>) -> Result<Size<S>, M>` so one logical call can perform the bounded auto-gutter passes |
| No target output | Required `ScrollTargetGeometryOf<S>` nested in every present `ScrollGeometryOf<S>` and exposed by `target()` |
| `ScrollUnsupportedFeature` and `ScrollOverflowCouplingPolicy` | Removed; real capabilities and typed input/rect errors |

All new scalar-bearing public values are generic over `LayoutScalar`, have
default-scalar aliases, validate finite-state rules at construction, and have
`f32`/`f64` contract tests. No type exposes a mutable field whose edit can break
its invariant.

`ScrollGeometryOf`, gutter output, clip output, and nested target values expose read-only
accessors. They do not implement `Default`: an all-zero placeholder is not a
performed layout. `ComputedOverflow`, input property values, and snap values do
implement their real CSS initial defaults.

The legacy scroll exposure/axis/facts types and conversion function have no
compatibility alias or public replacement constructor. This is an intentional
breaking pre-release leaf change: a caller can inspect the canonical used axes
on output geometry but cannot manufacture phase-ambiguous facts. No
compatibility alias for the other removed constructors, raw overflow point,
scrollbar field, or unsupported-feature enum remains.

## FRI-05.6 Behavior Matrices

### Computed And Used Overflow Matrix

| Computed value | Computed scrollable | Computed block-container IFC/BFC | Ordinary used value | Replaced used value | Used clip/range | Used classic gutter/UI |
| --- | --- | --- | --- | --- | --- | --- |
| `Visible` | No | No | `Visible` | `Visible` | No clip; no range | None |
| `Clip` | No | No | `Clip` | `Clip` | Clip edge; no range | None |
| `Hidden` | Yes | Yes | `Hidden` | `Clip` | Ordinary: scrollport clip and range. Replaced: clip edge and no range. | Ordinary: no UI; stable inline gutter may reserve. Replaced: none. |
| `Scroll` | Yes | Yes | `Scroll` | `Scroll` | Scrollport clip and range | Forced classic UI/reservation when thickness is non-zero |
| `Auto` | Yes | Yes | `Auto` | `Auto` | Scrollport clip and range | Conditional on non-zero derived range span |

The IFC/BFC column applies to a non-replaced block container and is a pair-level
decision. Canonical construction guarantees both axes are in the same computed
scrollability group. This table does not add the missing general BFC/float
algorithm owned by FRI-06; it requires current margin-collapse and existing
formatting-context branches to classify `Auto` correctly.

### Geometry Presence Matrix

| Output state | Container geometry | Target geometry |
| --- | --- | --- |
| Successfully laid-out root/leaf/block/flex/grid/subgrid/grid-lanes box | `Some`, including visible overflow | Present through `ScrollGeometryOf::target()` |
| Existing atomic inline box that receives a `NodeOutputOf` | `Some` using its inner formatting result | Present through `ScrollGeometryOf::target()` |
| `display:none` or omitted box | `None` | Absent |
| Line-break or inline-boundary control | `None` | Absent |
| Measurement-only result with no box output | `None` | Absent |

### Child Propagation Matrix

| Child used overflow on axis | Child nested overflow contribution to parent on that axis |
| --- | --- |
| `Visible` | Translated crate-private propagatable descendant interval, when present |
| `Clip`, `Hidden`, `Scroll`, or `Auto` | None; the child traps nested overflow |

The child's own positive-area border/margin contribution is independent of this
matrix. A zero-area child's own box remains excluded; only a used-visible
propagatable descendant interval can exercise the zero-area exception.

### Invalid And Failure Matrix

| Input or state | Result |
| --- | --- |
| Cross-group computed overflow pair | `ComputedOverflowError::NonCanonicalPair` |
| Non-finite or negative clip margin | Existing finite/non-negative scalar construction error |
| Non-finite signed scroll margin edge | `FiniteScalarErrorOf<S>` with the edge identified by the aggregate constructor |
| Non-finite scroll-padding calculation component | Existing `LengthPercentageOf` construction error |
| Negative resolved scroll padding | Clamp that used edge to zero |
| Non-finite rect origin, size, or end | `ScrollRectErrorOf<S>` |
| Negative rect size | `ScrollRectErrorOf<S>` |
| Opposing gutters wider than the padding box | Proportional effective saturation; valid zero-size scrollport/content box |
| Negative margins larger than the child | Valid final position; no negative synthetic contribution rectangle |
| Impossible geometry after validated inputs | Contextual `LayoutInternalInvariant`, never panic and never later-FRI unsupported |
| Missing output geometry for a present browser scroll expectation | Parity mismatch |

## FRI-05.7 Algorithm Integration Contract

### Root And Leaf

Root and leaf use the same canonical box/reservation/range factory as formatting
containers. Leaf measurement receives the effective content box for the current
auto-gutter pass. A stable pass emits geometry with its required nested target;
rounding rebuilds both together. Their scroll-origin axes are the ordinary
`FlowAxes` progression and they have no content-distribution start adjustment.
The public direct `compute_leaf` callback is `FnMut`, not `FnOnce`: it is invoked
once for each geometry-changing auto-gutter pass that requires measurement and
no more than three times. When measurement is required and no non-zero
conditional reservation changes available geometry, it is invoked exactly once.
The existing fully known `ComputeSize` fast path remains measurement-free and
invokes it zero times. Tree-backed measurement follows the same applicable input
sequence.

### Block

Block moves its local accumulator behavior to the shared scroll module. It
retains current margin-collapse and line/float calculations, but contributes
their final geometry through the shared operations. Content-box subtraction is
saturated before any child layout or accumulator seed. In-flow children,
current floats, current inline fragments, and current absolute children retain
their nested geometry and are included exactly once. Block's current
margin-collapse decision uses the complete computed-overflow IFC/BFC predicate,
so `Auto` follows hidden/scroll and `Visible`/`Clip` do not. Its range origin is
flow inline/block start and it supplies no content-distribution start adjustment
until FRI-09 adds block content alignment.

### Flex

Flex performs its sizing and placement with the effective scrollbar reservation
for the current pass. After final placement it accumulates every in-flow item and
current absolute item in source/output identity order, retains child geometry,
and emits its own canonical geometry. Both automatic-minimum callers use one
crate-private classifier for the canonical computed pair. The D-01 construction
invariant makes the pair's x/y scrollability classes equal, so no physical-axis
selection or rejected mixed pair is part of layout behavior. Flex derives
main/cross scroll-origin progression from `FlexAxes`, including row/column
reverse and wrap-reverse, and supplies the final applicable justify/align-
content subjects independently from out-of-flow overflow. FRI-07 remains
responsible for missing flex sizing and positioning semantics, not for a second
overflow path.

### Grid, Subgrid, And Grid-Lanes

Ordinary grid and grid-lanes use the effective reservation in available content
space and track the final container-relative item location. Their final
accumulation includes in-flow and current absolute children, propagates or
traps nested geometry per used physical axis through the shared helper, and
emits canonical container geometry.
Subgrid paths use the same parent-local translation and do not synthesize a
second range convention.

Grid and lanes automatic-minimum predicates use computed `Hidden`, `Scroll`, and
`Auto` as scrollable. The ordinary grid and lanes proof covers both physical axes
and every `FlowAxes` projection applicable to their logical sizing axes. Their
scroll origins follow flow inline/block start, and their final justified/aligned
track rectangles are the content-distribution subjects.

Every grid intrinsic, min-content, and percentage-track caller derives one
private used-overflow axis through the container's `FlowAxes` and
`GridAxisKind`. Only used `Visible` admits the item's propagatable descendant
`content_size`; `Clip`, `Hidden`, `Scroll`, and `Auto` trap it and retain the
item-box/min-track priority. Replaced computed `Hidden` becomes used `Clip`
before this decision. Ordinary grid, intrinsic subgrid, and grid-lanes share
that helper; no caller performs a context-free `Column => x` or `Row => y`
match.

### Cache And Diagnostics

Every final `NodeOutputOf` field, including container geometry and target
geometry, participates in existing cache equality and cached/uncached tests.
Speculative auto-gutter state cannot escape into cached final output.

Errors preserve the existing contextual subject/site mapping. Temporary
diagnostic instrumentation is removed before task completion and is never used
as closure evidence.

## FRI-05.8 Focused Evidence

The initiative requires these named evidence families:

| Evidence family | Required proof |
| --- | --- |
| Computed overflow construction | Exhaustive 25-pair table proves 13 accepted and 12 rejected pairs, defaults, accessors, and replaced used-value derivation. |
| Overflow predicate phase | Auto/hidden/scroll versus visible/clip prove computed auto minimum, current block margin/IFC behavior, used clipping/range, and ordinary versus replaced-hidden gutter behavior through real front doors. |
| Property model | Defaults, finite/negative validation, padding resolution, snap closed states, no-default output geometry, and public construction in `f32` and `f64`. |
| Canonical geometry | Box nesting, finite ends, partial-axis clips, clip-margin reference boxes, gutter placement, proportional small-box saturation, and constructor inaccessibility. |
| Flow direction and origin | Range and gutter placement across all ten `WritingMode`/`Direction` pairs before and after rounding, including every reversed physical axis plus flex row/column reverse and wrap-reverse origin mappings. |
| Alignment-origin range | Existing flex/grid start, end, center, safe fallback, reverse, and distributed content cases prove zero initial anchor, both-sided bounds, start-alignment reach, terminal padding, and exclusion of farther start-side out-of-flow overflow in all flow mappings. |
| Auto coupling | No-overflow, x-only, y-only, x-induces-y, y-induces-x, forced scroll, hidden stable, stable both-edges, and zero-thickness overlay cases; direct and tree-backed leaf evidence proves the exact effective measurement inputs, the one-to-three measurement-required call bound, and the fully known `ComputeSize` zero-call fast path. |
| Block blockers | The named negative-margin and smaller-than-scrollbar browser families complete without panic and match geometry. |
| Nested contribution | Under block, flex, grid, and lanes, used `Visible` propagates only its physical-axis descendant interval; `Clip`, `Hidden`, `Scroll`, and `Auto` trap nested overflow even when their local clip or full scrollable-overflow rectangle is non-empty. Partial-axis cases prove the two decisions are independent, and current absolute children are included once. |
| Zero-area contribution | `0xN` and `Nx0` child boxes prove that a real used-visible propagatable descendant interval survives on the non-zero axis in block, flex, grid, and grid-lanes, while the same nested geometry under every trapped value contributes zero to the parent. |
| Flex automatic minimum | The exhaustive thirteen accepted computed pairs prove equal x/y scrollability classes; rejected cross-group pairs prove main-only or cross-only scrollability cannot reach layout. Real flex front-door controls prove both callers retain content-based minimums for the `Visible`/`Clip` group and zero them for the `Hidden`/`Scroll`/`Auto` group without depending on flex direction or flow mapping. |
| Grid automatic minimum | Hidden, scroll, auto, visible, and clip cases through ordinary grid and lanes front doors. |
| Grid intrinsic used overflow | All five values through ordinary grid, intrinsic subgrid, and lanes callers prove the physical axis selected through all `FlowAxes` mappings, the visible-only `content_size` branch, trapped alternatives, and replaced-hidden conversion. |
| Grid origin | An item at non-zero container origin contributes through its final end; old area-relative expectation no longer passes. |
| Output helpers | `content_box_size()` and `scrollbar_size()` agree exactly with canonical geometry for no gutter, one edge, both edges, and saturated boxes. |
| Cache and rounding | Cached/uncached equality and normal/rounded geometry include the private used axes and required nested target with its margin/alignment/stop metadata; absent geometry remains absent. |
| Comparator activation | Correct non-zero and zero scroll deltas pass; wrong x, wrong y, and missing geometry each produce the named mismatch. |
| Fixture lowering | The exact eleven HTML sources serialize and parse every FRI-05-owned token without accepting broader CSS; matching active manifest records produce four variants each. |
| Corpus freeze | The pre-run manifest has full buckets 5,324/356/0/0/0, no scoped report, and a recorded hash; after the one full run, `check-corpus` proves the same hash and exact report buckets without a manifest edit. |
| Public surface | `lib.rs` reexports the new types; compile-fail/static searches prove removed raw fields, constructors, legacy scroll exposure/axis/facts types and conversion function, policies, and deferred variants are absent; present geometry always exposes `target()` and used-axis accessors. |

Behavior changes use reconstructed RED evidence at the exact task base. Focused
tests exercise the real `compute_layout` or formatting front door rather than a
parallel geometry simulator. Pure constructor tests may call the owning public
constructor directly.

The absolute unsafe scan covers all tracked and non-ignored Surgeist-owned Rust
files. No FRI-05 code, test, fixture helper, or generator change may add or retain
executable `unsafe`.

## FRI-05.9 Module And Implementation Outline

| Module or artifact | Desired responsibility |
| --- | --- |
| `src/node_input.rs` | Computed overflow pair, phase-correct pair/axis predicates, layout-ready scroll property and snap input types, defaults, and `NodeInputOf` fields. |
| `src/scroll.rs` | Rect errors, private used-overflow derivation and legacy-facts removal, reservation state, box/clip/gutter/target geometry, format-origin/alignment range, canonical factory, shared accumulator, rounding support. |
| `src/output.rs` | Existing optional geometry carriers, complete compute-to-node propagation, removal of independent scrollbar field, canonical helper methods. |
| `src/compute.rs` | Root/leaf pass integration, nested target construction, contextual geometry errors, final rounding, cache-safe publication. |
| `src/block.rs` | Shared accumulation calls, saturated constants, auto-gutter pass, retained child geometry. |
| `src/flex.rs` | One canonical computed-pair classifier shared by both automatic-minimum callers, flow-aware reservation, auto-gutter pass, retained child/absolute geometry, shared accumulation, `FlexAxes` scroll origin, and final content-distribution subjects. |
| `src/grid/mod.rs` | Grid container reservation/pass integration, final content-distribution subjects, and final geometry. |
| `src/grid/child.rs` | Final container-local item contribution, retained child/absolute geometry, zero-axis behavior. |
| `src/grid/lanes.rs` and `src/grid/subgrid.rs` | Shared origin, contribution, clipping, and geometry behavior for lanes/subgrid paths. |
| `src/grid/tracks.rs` | Correct computed-overflow auto-minimum predicate plus flow-aware private used-overflow trapping for intrinsic/min-content/percentage-track callers; no unrelated track algorithm expansion. |
| `src/lib.rs` | Intentional FRI-05 public reexports and removed legacy exports. |
| Focused Rust tests | Model, property, scalar, flow, block, flex, grid, lanes, root, cache, rounding, comparator, and public contract evidence. |
| `tests/layout/browser_parity/support.rs` | Atomic computed-overflow parsing, exact new property parsers, output range-span comparison, mismatch diagnostics. |
| `tests/layout/browser_parity/scripts/gentest/test_helper.js` | Read only the named computed-style fields. |
| `tests/bin/surgeist-layout-generate/generator.rs` | Serialize only the named fixture attributes and preserve existing expectation semantics. |
| `tests/layout/browser_parity/corpus.toml` | Eleven matching active case records and exact frozen full-report buckets 5,324/356/0/0/0 before the sole final full run. |
| Named HTML and generated XML | Bounded browser evidence with canonical provenance. |
| `README.md` and parity README | Public ownership, normalized input, output geometry, and finite fixture adapter contract. |

The implementation may choose private helper names and internal struct
decomposition. It may not weaken the public phases, construction invariants,
matrices, or ownership boundaries above.

## FRI-05.10 Errors, Capabilities, And State

FRI-05 leaves no layout-owned deferred capability for its input surface.
`Overflow::Auto`, clip margin, stable/both-edge gutters, scroll padding, scroll
margin, snap metadata, and mixed-axis coupling are accepted and represented.

This does not claim root runtime capabilities. Layout always returns immutable
geometry. Root may use or ignore snap metadata according to its own available
runtime integration, but it does not ask layout to select a live offset.

Public caller input errors are atomic: a failed constructor returns no partially
valid value. Formatting pass errors commit no partial final node output. Auto
gutter iteration is monotone and has no cancellation or timeout state. Cache
publication occurs only after stability.

The private canonical factory is the sole owner of derived-geometry invariants.
An internal geometry failure identifies the formatting operation and subject
through the existing `LayoutErrorOf` context. It is not erased to a string and
does not panic.

## FRI-05.11 Browser, Fixture, And Artifact Contract

### Owned Browser Sources

FRI-05 adds exactly these eleven active Surgeist HTML sources under the existing
suite directories:

1. `block/fri05_overflow_auto_cross_axis.html`;
2. `flex/fri05_overflow_auto_cross_axis.html`;
3. `grid/fri05_overflow_auto_cross_axis.html`;
4. `grid/fri05_hidden_auto_minimum.html`;
5. `grid-lanes/fri05_hidden_auto_minimum.html`;
6. `block/fri05_mixed_axis_clip_margin.html`;
7. `block/fri05_scrollbar_gutter_stable_both_edges.html`;
8. `flex/fri05_nested_zero_axis_overflow.html`;
9. `grid/fri05_nested_zero_axis_overflow.html`;
10. `grid/fri05_scroll_extent_area_origin.html`; and
11. `block/fri05_scroll_target_geometry.html`.

The sources use only the constrained existing fixture vocabulary plus the exact
FRI-05 attributes in D-13. Direction and box-sizing variants remain generator
owned. A source is not duplicated merely to obtain a generated variant.

Before full regeneration, `tests/layout/browser_parity/corpus.toml` adds one
matching active `[[cases]]` record for each source above, with the same
extensionless path as `id`, `source_root = "surgeist"`, the listed `.html` path
as `source`, and `generator = "constrained-html"`. Each active source produces
the existing four direction/box-sizing variants. Starting from the recorded
5,280 generated and 356 unsupported baseline, the frozen full-report
expectations are therefore exactly:

| Manifest bucket | Frozen pre-run value |
| --- | ---: |
| `generated` | 5,324 |
| `unsupported` | 356 |
| `expected_fail` | 0 |
| `quarantined` | 0 |
| `failed_to_generate` | 0 |

`generation_reports.scoped` remains empty and the full report remains
`all.json`. No new source may be assigned a non-active status to force these
counts.

Existing browser families for negative block margins, tiny scroll boxes,
scrollbar reservations, RTL scrollbars, flex nested overflow, grid overflow,
subgrid overflow, and grid-lanes scrollers remain applicable evidence. Their
HTML is changed only if a confirmed source-input bug is demonstrated.

Existing `flex/overflow_scroll_main_axis_justify_content_end.html`, the flex
safe justify/align-content overflow families, and the grid safe/unsafe
justify/align-content overflow families are the browser-backed
alignment-origin evidence. FRI-05 changes neither their authored alignment
values nor the alignment algorithm merely to obtain a preferred range; it
compares the range implied by their final browser-backed positions.

### One Final Full Regeneration

During implementation, a scoped ExistingPinned generation may be used to
diagnose a changed source. It is report-free, may touch only matching derived
XML, and is not mandated verification evidence.

After every FRI-05 helper, serializer, parser, HTML, and manifest input is
settled, require the eleven active records and exact bucket values above to be
present, record the byte-exact `corpus.toml` SHA-256, and freeze the manifest.
Then run exactly one unfiltered full ExistingPinned regeneration with the
already-present pinned browser and an empty generation filter. That run owns all
derived XML pruning and writes the canonical `all.json`, including the frozen
manifest hash. It does not write `corpus.toml`; the manifest owns the expected
counts rather than receiving them from the report.

After a successful final run, generator inputs remain frozen. Verification is
read-only: `check-corpus` must pass with `corpus.toml` byte-identical to the
recorded pre-run hash and all five report buckets equal to the frozen values.
Focused FRI-05 parity, Rust gates, diff review, and provenance review neither
edit the manifest nor regenerate.

If later evidence confirms a genuine input bug in helper, serializer, parser,
HTML, or manifest data, the prior run is invalidated. Correct all such inputs,
including any corrected manifest record or expected bucket, let them settle,
record the replacement manifest hash, and perform one replacement full run. A
bucket mismatch proves the prior manifest expectation was wrong but does not
permit a post-run manifest-only edit: it follows this input-bug replacement
rule. A test failure, review request, uncertainty, or desire to refresh evidence
without an input change is not permission for another run.

No generated XML or report is hand-edited. The final inventory contains one
full canonical report, no scoped report, no stale XML, no expected failure or
quarantine introduced for an owned FRI-05 behavior, and exact source provenance.

### Verification Boundary

FRI-05 verification uses the repository's existing `just` recipes for normal,
generator, and corpus gates. Focused parity filters cover the eleven owned
sources and named pre-existing regression families. The ignored aggregate
`just parity-all` release gate remains untouched and unclaimed for `FRI-13`.

## FRI-05.12 Root Integration Handoff

At inspected root revision
`19590f6d9fa01c0df197c5ef07fb626c5cf18ced`, pinned
`surgeist-css@040bc1b4f7cca5e4978f732b3b778c00d7cdef40` and
`surgeist-style@fcc42de2c32a318e073233dd51508dd4cc28041a` each expose
only four overflow keywords. CSS rejects `auto`, style has no computed-pair
type or coupling finalization, and `src/adapters/style_layout.rs` lowers raw x
and y independently. The pinned sources also lack typed models for clip margin,
gutter, scroll padding/margin, and snap metadata; `scrollbar-width` is currently
an unconstrained number. The leaf candidate handoff resolves those missing
upstream phases without performing the work here.

The pinned style model exposes only `HorizontalTb`, `VerticalLr`, and
`VerticalRl`; root maps those three and currently defaults any malformed
writing-mode value to horizontal. Pinned CSS lists authored `writing-mode` as
known unsupported. Those facts bound the upstream logical-edge proof below to
six style mappings and two default-horizontal CSS mappings; they do not narrow
the leaf's existing five-mode, ten-mapping contract.

Computed overflow follows this phase chain:

| Phase | Owner | Required representation and transition |
| --- | --- | --- |
| Authored CSS | `surgeist-css` | `CssOverflow` adds `Auto`; strict longhand and one/two-value shorthand parsing preserve it as authored syntax and retain typed rejection for unknown tokens. |
| Specified style | `surgeist-style` | `Overflow` adds `Auto`; `OverflowAxes` remains the unconstrained two-axis specified value used by Rust-authored declarations and the root CSS adapter. |
| Computed style | `surgeist-style` | New private-field `ComputedOverflowAxes` owns exactly the same thirteen valid pairs as layout. One resolver-private conversion couples the final longhands simultaneously after rule, local, animated, and global-keyword resolution, stores the coupled x/y values before cache insertion, and exposes the pair only through `Resolved::computed_overflow()`. No public API can couple an arbitrary pre-cascade `OverflowAxes`; no root adapter computes or repairs this pair. |
| Layout lowering | Root `surgeist` | `src/adapters/style_layout.rs` reads `Resolved::computed_overflow()` once, maps all five keywords losslessly, and calls `layout::ComputedOverflow::try_new(x, y)` once. It never assigns independent raw axes. |

The style-owned computed carrier has this minimum public contract:

```rust
pub struct ComputedOverflowAxes { /* private x/y */ }

impl ComputedOverflowAxes {
    pub const fn x(self) -> Overflow;
    pub const fn y(self) -> Overflow;
}

impl Resolved {
    pub fn computed_overflow(&self) -> ComputedOverflowAxes;
}
```

There is no public raw computed-pair constructor, `Default`, `From`/`TryFrom`,
specified-to-computed conversion, or mutable axis. Only resolver-private code
constructs the carrier, and `Resolved::computed_overflow()` is the sole public
route to one. A compile consumer and API audit prove that `OverflowAxes` cannot
be converted directly.

The CSS and style enum additions are breaking for exhaustive downstream
matches; their candidates update every match explicitly and provide no
`Auto => Scroll` compatibility shim. Neither leaf adds a dependency on the
other or on layout; root owns both phase adapters.

For original specified pair `(x, y)`, style computes both axes from that original
pair, not from a partially converted intermediate:

- specified `Visible` computes to `Auto` when the other specified axis is not
  `Visible` or `Clip`, and otherwise remains `Visible`;
- specified `Clip` computes to `Hidden` when the other specified axis is not
  `Visible` or `Clip`, and otherwise remains `Clip`; and
- specified `Hidden`, `Scroll`, and `Auto` remain unchanged.

This total 25-pair conversion yields only the thirteen layout-valid pairs.
`Auto` is never aliased to `Scroll`. CSS syntax failures retain the existing
typed property/location diagnostic. Style coupling is total and has no fallback
or error state. If the style-to-layout adapter nevertheless observes a rejected
leaf pair, it returns the existing
`AdapterErrorKind::StyleLayoutValue { property: "overflow", .. }`; it never
defaults either axis or retries with a repaired pair.

Every other D-02 field follows this property-by-property upstream contract.
The CSS column is the bounded grammar required by this handoff, not a claim that
the root parser now accepts every CSS unit or the Level 4 per-edge expansion of
`overflow-clip-margin`. Existing strict `0`, `px`, percentage where allowed,
and well-typed `calc()` support is retained; any other unit remains an existing
typed unsupported-syntax result. All rows also accept CSS-wide keywords through
the existing cascade machinery.

| Layout field | `surgeist-css` authored grammar and initial value | `surgeist-style` specified/computed ownership | Root normalized lowering |
| --- | --- | --- | --- |
| `overflow_clip_margin` | Level 3 box-model subset: `overflow-clip-margin: [ content-box | padding-box | border-box ] || <length [0,infinity]>`; either component can be omitted, defaulting to `padding-box` and `0px`. Percentages, negative values, Level 4 per-edge properties, and SVG-only visual-box keywords are typed unsupported syntax. | One typed specified value and one private-field computed value hold the box plus finite non-negative absolute length. No logical mapping applies. `Resolved::overflow_clip_margin()` returns the computed value. | Map the three box keywords exhaustively and call `layout::OverflowClipMargin::try_new` once; no box or length default is supplied by the adapter. |
| `scrollbar_gutter` | `scrollbar-gutter: auto | stable && both-edges?`; initial `auto`. `both-edges` without `stable`, duplicates, or extra tokens are invalid. | Closed `ScrollbarGutter` normalizes only to `Auto`, `Stable`, or `StableBothEdges`; computed value is as specified. `Resolved::scrollbar_gutter()` is the normalized accessor. | Exhaustively map the three values. Overlay versus classic behavior comes only from the explicit scrollbar environment below; the adapter does not reinterpret gutter syntax. |
| `scrollbar_width` | Replace the numeric grammar with `scrollbar-width: auto | thin | none`; initial `auto`. Numbers are no longer accepted. | Closed `ScrollbarWidth::{Auto, Thin, None}` is computed as specified and returned by `Resolved::scrollbar_width()`. It is never represented as `Value::Number`. | Resolve the keyword through the required host scrollbar environment below, validate the resulting scalar once, and pass one `layout::ScrollbarWidth`. |
| `scroll_padding` | `scroll-padding: [ auto | <length-percentage [0,infinity]> ]{1,4}`, four physical longhands, four flow-relative longhands, and the one/two-value `scroll-padding-block`/`scroll-padding-inline` shorthands; initial `auto` per side. Negative values are invalid. | Physical and logical declarations remain distinct through cascade. After final `writing-mode` and `direction` resolution, the resolver maps the winning declarations to one physical top/right/bottom/left aggregate, preserving `Auto` and computed length-percentage/calc values. `Resolved::scroll_padding()` returns only that physical aggregate. | Map each physical edge to `layout::ScrollPaddingValue`; calc values use the existing lowering session and calc store. The no-store convenience path reports `scroll-padding` when calc is present, as it does for other calc-owned properties. Root performs no logical mapping and supplies no heuristic for `Auto`; layout's documented used zero policy remains authoritative. |
| `scroll_margin` | `scroll-margin: <length>{1,4}`, four physical longhands, four flow-relative longhands, and the one/two-value `scroll-margin-block`/`scroll-margin-inline` shorthands; initial `0px` per side. Signed lengths are valid and percentages are invalid. | The same post-cascade physical/logical conflict resolution uses the target box's computed `writing-mode` and `direction`. Computed edges are finite signed absolute lengths and `Resolved::scroll_margin()` returns the physical aggregate. | Call `layout::ScrollMargin::try_new` once with all four edges. An unresolved or non-finite absolute length is an error, never zero. Transform application remains a later root runtime operation. |
| `scroll_snap_type` | `scroll-snap-type: none | [ x | y | block | inline | both ] [ mandatory | proximity ]?`; initial `none`. Omitted strictness is accepted. | A closed type normalizes omitted strictness to explicit `Proximity` during declaration construction/resolution; `Resolved::scroll_snap_type()` never exposes an enabled value without strictness. Logical `Block`/`Inline` remains semantic rather than being prematurely physicalized. | Exhaustively map `None` or axis plus explicit strictness. Runtime axis interpretation uses the eventual snap container's retained `FlowAxes`, not the styled target or the adapter's current node axes. |
| `scroll_snap_align` | `scroll-snap-align: [ none | start | end | center ]{1,2}`; initial `none`. One token duplicates to both block and inline computed values. | Private-field `ScrollSnapAlign` always stores explicit block and inline values. `Resolved::scroll_snap_align()` preserves those semantic roles; it does not map them to the target's physical axes. | Map both values losslessly into layout target metadata. Root interprets them with the snap-container/oversized-target rules below after association and transform. |
| `scroll_snap_stop` | `scroll-snap-stop: normal | always`; initial `normal`. | Closed computed-as-specified enum returned by `Resolved::scroll_snap_stop()`. | Exhaustive one-to-one mapping; live pass-over behavior remains root runtime policy. |

At the pinned style revision, `Metadata::new` defaults to no impact,
non-animatable, and discrete, while `Invalidation::between` derives its flags
only from changed canonical properties and their metadata. The upstream style
candidate therefore gives computed overflow and every D-02 property the
explicit lifecycle below. All are non-inherited. Their style `Change` scope is
the changed node; the root-owned scroll-tree refresh in the final column is an
additional runtime consequence, not CSS inheritance.

| Canonical style value | Required impact metadata | Animation/interpolation contract | Root update after changed resolved style |
| --- | --- | --- | --- |
| `OverflowX`/`OverflowY` and coupled `ComputedOverflowAxes` | Layout and paint | Animatable with discrete interpolation. Both final axes are recoupled simultaneously for every sampled value. | Re-lower and lay out the node; if scroll-container or capture status changes, rebuild descendant snap association from this containing-block-chain point, clamp live offsets to the new range, and re-snap affected containers. |
| `OverflowClipMargin` | Layout and paint | Animatable by computed absolute length while the visual-box keyword matches; otherwise discrete. | Rebuild clip geometry and paint state. Clip margin does not change range, snapport, or target area, so this change alone does not trigger re-snapping. |
| `ScrollbarGutter` | Layout and paint | Animatable with discrete interpolation. | Re-run reservation/layout with the same explicit scrollbar environment, update scrollport/range, clamp offsets, and re-snap affected containers. |
| `ScrollbarWidth` | Layout and paint | Animatable with discrete interpolation among `Auto`, `Thin`, and `None`; host metrics are applied after sampling. | Same geometry, offset-clamp, and re-snap path as gutter; a policy change without a style change also invalidates every lowered node that consumed that environment. |
| `ScrollPadding` | Layout | Per physical edge, interpolate compatible computed length-percentage/calc values; `Auto` versus a numeric value is discrete. | Rebuild optimal viewing region/snapport and re-snap the container when its current snap position is no longer valid. |
| `ScrollMargin` | Layout | Interpolate all four computed signed absolute lengths. | Rebuild transformed target area, focus/target scrolling data, and snap positions; re-snap an associated container when the active target or valid position moved. |
| `ScrollSnapType` | Layout | Animatable with discrete interpolation. | Rebuild capture association for containing-block descendants until their nearer capture barriers, recalculate candidate positions, and re-snap or unsnap the changed container as required. |
| `ScrollSnapAlign` | Layout | Animatable with discrete interpolation of the two computed keywords. | Recalculate this target's positions and re-snap its associated container when its active position changed. |
| `ScrollSnapStop` | Layout | Animatable with discrete interpolation. | Refresh this target's live pass-over policy; preserve the current offset unless ordinary re-snap rules independently require movement. |

The style metadata marks every row animatable and records the interpolation
kind above; a generic fallback to `Interpolation::Discrete` is insufficient for
clip margin, scroll padding, or scroll margin. Conditional tuple interpolation
is implemented in the owning typed value rather than by flattening it to a
generic number or `Edges`. Animated declarations pass through the same final
normalization and computed-overflow coupling as rule, local, and global-keyword
values before cache insertion.

Each normalized computed D-02 value is stored under its canonical `Property`
entry in `Resolved`; the typed accessor reads that same value. Consequently
`Resolved` equality and `Invalidation::between` observe a changed normalized
physical edge even when it originated from a logical declaration. Every row
must make `Invalidation::between(before, after).layout` true; the four
layout-and-paint rows also make `paint` true. `Change::from_properties` includes
the node and does not include descendants merely through inheritance. Resolver
cache keys include rule/local/animated inputs, and cached and uncached resolution
must return identical normalized carriers.

For scroll padding and scroll margin, each shorthand expands at its declaration's
cascade position. Physical and logical longhands are not collapsed before
`writing-mode` and `direction` are computed. The resolver then applies the
winning physical/logical declarations in cascade order through the six mappings
currently representable by style: `HorizontalTb`, `VerticalLr`, and
`VerticalRl`, each with `Ltr` and `Rtl`. It caches only the normalized physical
aggregate.

The CSS-to-style adapter transfers authored semantic values and source locations
but never chooses physical edges. Snap type and snap alignment are deliberately
excluded from that physical-edge normalization because their logical axes are
defined by the eventual snap container, with the oversized target exception.

This cross-repository boundary does not add `SidewaysLr` or `SidewaysRl` to
`surgeist-style` and does not make `writing-mode` authored CSS parseable. CSS
integration proves the default `HorizontalTb` mappings in both directions;
Rust-authored style integration proves all six currently representable mappings.
The four Sideways `WritingMode`/`Direction` pairs remain valid and fully tested
for direct `surgeist-layout` callers under FRI-02/FRI-05, but CSS/style/root
Sideways expansion is a separate upstream initiative and is not required for
this candidate handoff. Root maps the three current style variants exhaustively;
a malformed value or future unmatched variant returns a typed
`StyleLayoutValue { property: "writing-mode", .. }` instead of defaulting to
`HorizontalTb`.

The normalized style accessors above are the only style-to-layout inputs for
these fields. Their computed carriers have private fields and read-only
accessors. Root does not read generic `Resolved::get` entries for these
properties, reconstruct CSS defaults, inspect paired logical longhands, or
repair invalid combinations.

Scrollbar thickness requires explicit root-owned host policy. Root adds a
validated lowering environment with exactly these semantic states:

| Environment | Authored `auto` | Authored `thin` | Authored `none` |
| --- | ---: | ---: | ---: |
| Overlay | `0` | `0` | `0` |
| Classic `{ auto_width, thin_width }` | `auto_width` | `thin_width` | `0` |

Classic widths must be finite and non-negative, and `thin_width` must not exceed
`auto_width`; invalid metrics fail construction through a dedicated typed root
configuration error before any node is lowered. The environment has no
`Default`. Every public lowering entry point and `LayoutLoweringSession`
construction receives it explicitly, so callers must choose `Overlay` rather
than receiving an implicit zero fallback. A missing capability is therefore
unrepresentable, not guessed. Zero thickness still preserves overflow clipping,
range, and semantic gutter state while producing no layout reservation.

Failure ownership is phase-specific. Invalid grammar produces the existing CSS
property/location diagnostic. A CSS-to-style representation mismatch produces
the existing source-located CSS/style adapter diagnostic. Non-finite computed
lengths fail typed style resolution. A layout constructor rejection produces
`AdapterErrorKind::StyleLayoutValue` naming the exact CSS property; root neither
substitutes the initial value nor retries with a simpler value. The only defaults
are the initial values recorded in the table and installed by style metadata.

Cross-repository evidence for the matrix is mandatory: CSS parser tables cover
every grammar, omission/default normalization, shorthand expansion, and invalid
token class; style tests cover initial values, CSS-wide keywords, cache hits,
physical/logical precedence in all six style-representable flow mappings,
omitted snap strictness, and one-value snap alignment. Metadata and interpolation
tests cover every lifecycle row, including compatible midpoints and each
required discrete boundary. `Invalidation::between` and
`Change::from_properties` tests prove the exact layout/paint flags and node
scope, and a before/after resolver-cache test proves that changed style reaches
a changed typed accessor and changed lowered `NodeInput` field. Root tests carry
each authored property through CSS, style, and the public layout-lowering entry
point, using CSS-origin evidence for both default-horizontal directions and
Rust-authored style evidence for all six mappings. Root tests also cover
overlay, classic auto, classic thin, none, invalid metrics, environment-policy
invalidation, calc/no-store diagnostics, every layout constructor failure
mapping, capture-subtree refresh, offset clamping and each required re-snap or
no-re-snap transition, plus an API/static proof that no raw number,
generic-value read, adapter-side logical mapping, or silent default path remains.

The cross-repository handoff requires:

1. a published `surgeist-css` candidate that adds authored `Auto` plus every
   bounded D-02 grammar in the property matrix, with parser, shorthand,
   omission, default-component, and typed-invalid evidence;
2. a published `surgeist-style` candidate that adds `Auto`, the resolver-only
   computed-overflow carrier, every typed D-02 model and normalized accessor,
   post-cascade simultaneous coupling, six-map physical/logical edge resolution,
   exact impact/interpolation metadata and invalidation behavior, and exhaustive
   default/cached-resolution evidence;
3. root `src/adapters/css_style.rs` evidence that transfers all authored
   overflow and D-02 values losslessly, preserves declaration locations and
   logical roles, and leaves coupling and physical-edge resolution to style;
4. root `src/adapters/style_layout.rs` evidence that all 25 specified pairs
   reach the exact expected leaf `ComputedOverflow`, including mixed
   `Visible`/`Clip` conversion and distinct `Auto`, plus the named typed drift
   diagnostic;
5. root `src/adapters/style_layout.rs` evidence for every D-02 row through the
   normalized style accessors, including explicit overlay/classic scrollbar
   environment injection, calc-store behavior, all six supported logical edge
   mappings, typed rejection instead of writing-mode fallback,
   omitted snap strictness as `Proximity`, and the exact typed failures named
   above;
6. migration of removed scroll constructors,
   `ScrollOverflowExposure`/`ScrollContainerAxis`/`ScrollContainerFacts`,
   `ScrollGeometryOf::container()`, and the removed
   `NodeOutputOf::scrollbar_size` field to read-only canonical
   geometry/accessors, without recreating the leaf's removed phase-unsafe
   construction path;
7. consumption of per-axis clips, optimal viewing region, the zero-anchored
   signed physical range including content-distribution origin adjustment, and
   nested target geometry through `ScrollGeometryOf::target()` without
   recomputing leaf box invariants;
8. separate root integration evidence for nearest-scroll-container focus,
   fragment, and `scrollIntoView` behavior versus nearest-capturer snap
   association, including non-scroll and snap-type-`None` capture barriers and
   the lifecycle table's capture-subtree invalidation;
9. snap-axis evidence for orthogonal container/target writing modes, ordinary
   container-relative block/inline alignment, and the per-axis target-writing-
   mode exception after transform plus scroll margin makes a snap area larger
   than its snapport;
10. preservation of root ownership for transformed coordinate mapping, current
   offsets, host events, scroll UI, CSSOM adaptation, target/focus scrolling,
   oversized-area valid-position ranges, snap selection, and the lifecycle
   table's offset-clamp/re-snap decisions;
11. promotion of the exact published CSS, style, and layout candidate gitlinks
   before the root adapter commit; and
12. refresh and check of root-owned `api/crates/surgeist-css.txt`,
    `api/crates/surgeist-style.txt`, `api/crates/surgeist-layout.txt`, and
    `api/public-api.txt` from those promoted sources, with all three exact leaf
    SHAs and the breaking API inventory retained in the root promotion evidence.

The leaf adds no adapter or facade compatibility layer. Root may use its own
temporary migration sequence, but the final integrated surface contains one
canonical lowering path.

## FRI-05.13 Durable Sequence Boundaries

An implementation sequence can derive these durable dependency boundaries
without redesign:

1. canonical computed-overflow and scroll-property input/output model;
2. canonical rect, clip, gutter, format-origin/alignment range, target, factory,
   accumulator, and rounding substrate;
3. root/leaf/block integration, negative-margin closure, small-box saturation,
   and auto-gutter fixed point;
4. flex retained/nested geometry and axis-independent contribution;
5. ordinary grid, subgrid, and grid-lanes geometry, auto minimum, zero-axis, and
   container-origin closure; and
6. bounded fixture lowering, pre-run manifest freeze, one full regeneration,
   comparator activation, public/docs evidence, finding trace, and candidate
   closure.

Each boundary can be completed and reviewed as a coherent implementation cycle.
Future cycle plans remain just-in-time and may split a boundary only when source
evidence shows that one reviewable coding range would otherwise be oversized.
They may not merge generator architecture, later formatting behavior, or root
integration into FRI-05.

The leaf sequence ends at boundary 6. Section FRI-05.12 defines the subsequent
cross-repository dependency boundary rather than a seventh leaf cycle:
`surgeist-css` authored-overflow/D-02 syntax and `surgeist-style`
computed-overflow/D-02 normalization candidates remain separate owner work and
may proceed independently, while the root adapter/gitlink/API-artifact
integration waits for those two published candidates and the published FRI-05
layout candidate.

## FRI-05.14 Finding Traceability

| Finding | Required closure evidence |
| --- | --- |
| `BLOCK-001` | Shared positive-outset margin accumulation plus named negative-margin front-door/browser evidence with no geometry error. |
| `BLOCK-002` | Proportional effective gutter saturation and tiny-box block/root evidence with zero content geometry. |
| `GRID-011` | Computed hidden/scroll/auto automatic-minimum tests through ordinary grid and grid-lanes. |
| `OVERFLOW-001` | Flex/grid in-flow and current absolute children retain geometry; nested parent outputs propagate transitive descendant intervals only through used-visible axes and trap them for clip/hidden/scroll/auto. |
| `OVERFLOW-002` | All D-01/D-02 values represented, every computed/used predicate assigned, auto coupling and current alignment-origin range executed, per-axis clipping and current out-of-flow contribution emitted, target geometry carried concretely, and deferred plus phase-unsafe legacy variants removed. |
| `OVERFLOW-003` | Independent x/y accumulator and `0xN`/`Nx0` front-door tests across block, flex, grid, and lanes prove used-visible nested propagation and trapped-value exclusion. |
| `OVERFLOW-005` | Finite-end rect validation, no public gutter/geometry constructors, canonical coherence property tests, and retained FRI-02 coordinate validation. |
| `CORE-006` | `content_box_size()` and derived scrollbar accessor agree with canonical geometry in ordinary, both-edge, and saturated cases. |
| `GRID-009` | Container-relative ordinary-grid and lanes extent is `origin + contribution`, with the old area-relative expectation replaced by browser-backed evidence. |
| `TEST-002` | Comparator consumes `scroll_size`; wrong x/y and missing geometry fail; focused generated fixtures pass with range spans. |

The initiative status does not advance to complete while any row lacks its named
implementation and evidence.

## FRI-05.15 Initiative Acceptance

FRI-05 is complete only when:

1. all ten owned findings satisfy the traceability table and remain assigned
   only to FRI-05 in the findings-resolution index;
2. every public FRI-05 input state is intrinsically valid, has a real default,
   and is consumed without a later-FRI unsupported placeholder;
3. every successfully laid-out block, flex, grid, subgrid, and grid-lanes box
   emits coherent geometry through the shared contract;
4. computed versus used overflow, all thirteen valid pairs, twelve invalid
   pairs, five axis values, current IFC/margin predicates, flex canonical-pair
   and grid automatic minimum, grid flow-aware intrinsic mapping, partial clips,
   stable/both-edge gutters, auto cross-axis coupling, and replaced hidden
   behavior have focused proof;
5. all ten flow mappings produce correct physical gutter placement and signed
   range spans before and after rounding in both scalar lanes, including flex
   reverse origins and the bounded start-side adjustment for current flex/grid
   content distribution;
6. negative margins, tiny boxes, zero-axis used-visible propagation, trapped
   nested descendants, current absolute descendants, and non-zero grid origins
   return correct geometry without panic, silent zeroing, or guessed context;
7. cached and uncached final outputs agree for every new geometry value,
   including its required nested target, and speculative auto passes cannot be
   observed;
8. `NodeOutputOf` helpers derive the actual canonical content box and gutter
   reservation;
9. parsed browser scroll expectations are compared and the named focused parity
   families can fail on a wrong range;
10. the eleven HTML sources and matching active manifest records settle first;
    `corpus.toml` is frozen at full buckets 5,324/356/0/0/0, then one valid full
    regeneration produces the derived XML and `all.json`, and subsequent
    read-only corpus checks pass against the unchanged recorded manifest hash;
11. normal and generator verification, corpus validation, focused parity,
    formatting, Clippy with `-F unsafe-code -D warnings`, diff checks, and the
    tracked/non-ignored Rust unsafe scan are clean;
12. public exports and both READMEs describe the implemented ownership and
    normalized fixture boundary without claiming root runtime behavior;
13. no dependency, feature, MSRV, generator architecture, root, sibling,
    aggregate FRI-13 gate, or unrelated formatting behavior changes; and
14. Section FRI-05.12 records the complete breaking leaf-to-root integration
    contract, including resolver-only computed-overflow construction, every
    D-02 CSS/style/root phase and diagnostic, explicit scrollbar environment,
    six-map upstream and ten-map leaf flow boundaries, style lifecycle and
    invalidation, snap capture barriers and axis rules,
    the canonical geometry and target carriers, removed-surface migrations, and
    the three-candidate pointer/API-artifact dependency required of the eventual
    root promotion.
