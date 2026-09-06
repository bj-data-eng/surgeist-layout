# surgeist-layout

Layout primitives, algorithms, oracle tests, and fixture tooling for Surgeist document surfaces.

## Scalar Precision

`surgeist-layout` keeps its default public API at browser-style coordinate
precision: `DefaultScalar` is `f32`, and `Scalar` aliases `DefaultScalar`.
Most applications should use the default aliases such as `NodeInput`,
`ComputeInput`, and `ComputeOutput`.

For applications that need more coordinate precision, scalar-bearing APIs also
provide generic `*Of<S>` forms, and `LayoutScalar` is implemented for both
`f32` and `f64`. Pick one scalar type for a layout tree and run it end-to-end:
do not mix `f32` and `f64` values inside one tree, cache, traversal, or layout
run.

Browser parity XML and its generator remain a default-precision fixture
boundary. Use those fixtures to check the default `f32` contract; add separate
crate-local tests when a behavior specifically needs `f64` coverage.

## Public API Map

All public paths are exposed directly from the `surgeist_layout` crate root.
Default-scalar aliases use `f32`; the corresponding `*Of` forms support either
public scalar lane.

### Computation and tree

- Tree layout: [`compute_layout`](https://docs.rs/surgeist-layout/latest/surgeist_layout/fn.compute_layout.html) and [`compute_layout_invalidated`](https://docs.rs/surgeist-layout/latest/surgeist_layout/fn.compute_layout_invalidated.html).
- Direct leaf measurement: [`compute_leaf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/fn.compute_leaf.html).
- Host contracts: [`Traverse`](https://docs.rs/surgeist-layout/latest/surgeist_layout/trait.Traverse.html), [`LayoutTree`](https://docs.rs/surgeist-layout/latest/surgeist_layout/trait.LayoutTree.html), and [`LayoutBatchSink`](https://docs.rs/surgeist-layout/latest/surgeist_layout/trait.LayoutBatchSink.html).
- Cache and failure contracts: [`CacheOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.CacheOf.html), [`LayoutResultOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/type.LayoutResultOf.html), and [`LayoutErrorOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LayoutErrorOf.html).

### Node input

- Aggregate input: [`NodeInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.NodeInputOf.html) and its [`NodeInput`](https://docs.rs/surgeist-layout/latest/surgeist_layout/type.NodeInput.html) default-scalar alias.
- Tree participation: [`LayoutInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.LayoutInputOf.html), [`Display`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.Display.html), [`Position`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.Position.html), and [`ItemOrder`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ItemOrder.html).
- Flow and alignment: [`WritingMode`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.WritingMode.html), [`Direction`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.Direction.html), [`AlignItems`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.AlignItems.html), and [`AlignContent`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.AlignContent.html).
- Inline and flex input: [`InlineTextInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.InlineTextInputOf.html), [`InlineBoundaryInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.InlineBoundaryInputOf.html), [`FlexDirection`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.FlexDirection.html), [`FlexWrap`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.FlexWrap.html), and [`FlexItemCollapse`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.FlexItemCollapse.html).
- Grid input: [`GridPlacement`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.GridPlacement.html), [`GridAutoFlow`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.GridAutoFlow.html), and [`GridFlowToleranceOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.GridFlowToleranceOf.html).
- Scroll input: [`ComputedOverflow`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ComputedOverflow.html), [`OverflowClipMarginOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.OverflowClipMarginOf.html), [`ScrollPaddingOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ScrollPaddingOf.html), [`ScrollMarginOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ScrollMarginOf.html), and [`ScrollSnapType`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.ScrollSnapType.html).

### Sizing

- Property domains: [`PreferredSizeOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.PreferredSizeOf.html), [`MinSizeOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.MinSizeOf.html), [`MaxSizeOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.MaxSizeOf.html), and [`FlexBasisOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.FlexBasisOf.html).
- Validated calculations: [`SizingCalculationOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.SizingCalculationOf.html) and [`CalcSizeCalculationOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.CalcSizeCalculationOf.html).
- Finite values and resolution: [`LengthPercentageOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LengthPercentageOf.html), [`PercentageBasisOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.PercentageBasisOf.html), [`LengthOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.LengthOf.html), [`LengthAutoOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.LengthAutoOf.html), and [`AvailableOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.AvailableOf.html).

### Geometry and scroll

- Physical geometry and flow mapping: [`Point`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.Point.html), [`Size`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.Size.html), [`Edges`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.Edges.html), and [`FlowAxes`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.FlowAxes.html).
- Canonical scroll geometry: [`ScrollGeometryOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ScrollGeometryOf.html), [`ScrollRectOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ScrollRectOf.html), [`PhysicalScrollRangeOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.PhysicalScrollRangeOf.html), and [`FlowRelativeScrollRangeOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.FlowRelativeScrollRangeOf.html).
- Read-only scroll details: [`OverflowClipOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.OverflowClipOf.html), [`ScrollbarGutterRectsOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ScrollbarGutterRectsOf.html), and [`ScrollTargetGeometryOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ScrollTargetGeometryOf.html).

### Output

- Root request and transaction: [`LayoutRootRequestOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LayoutRootRequestOf.html) and [`CompletedLayoutBatchOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.CompletedLayoutBatchOf.html).
- Node and sizing results: [`NodeOutputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.NodeOutputOf.html), [`ComputeInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ComputeInputOf.html), and [`ComputeOutputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ComputeOutputOf.html).
- Inline output: [`InlineFragmentOutputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.InlineFragmentOutputOf.html) and [`InlineFragmentOutputEntryOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.InlineFragmentOutputEntryOf.html).
- Root context: [`LayoutRootContextOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.LayoutRootContextOf.html), [`FlexItemRootContextOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.FlexItemRootContextOf.html), and [`ContainingLayoutContext`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.ContainingLayoutContext.html).

### Finite grid utilities

- Lane placement: [`place_lanes`](https://docs.rs/surgeist-layout/latest/surgeist_layout/fn.place_lanes.html), [`LanePlacementInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LanePlacementInputOf.html), and [`LanePlacementReportOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LanePlacementReportOf.html).
- Intrinsic lane sizing: [`lane_intrinsic_sizing`](https://docs.rs/surgeist-layout/latest/surgeist_layout/fn.lane_intrinsic_sizing.html), [`LaneIntrinsicSizingInputOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LaneIntrinsicSizingInputOf.html), and [`LaneIntrinsicSizingReportOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.LaneIntrinsicSizingReportOf.html).
- Grid computation reports: [`GridComputationOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.GridComputationOf.html) and [`GridComputationReport`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.GridComputationReport.html).
- Finite track construction: [`TrackSizingOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.TrackSizingOf.html), [`TrackComponentOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/enum.TrackComponentOf.html), [`TrackComponentListOf`](https://docs.rs/surgeist-layout/latest/surgeist_layout/struct.TrackComponentListOf.html), and [`track_sizing_components_of`](https://docs.rs/surgeist-layout/latest/surgeist_layout/fn.track_sizing_components_of.html).

## Browser Parity Runtime

The browser-parity corpus pins its Chrome-for-Testing source, executable version,
cache root, launch profile, and report inventory in
`tests/layout/browser_parity/corpus.toml`. `generate` is the managed-pinned mode;
it is the only command that may use the configured browser fetcher.
`generate-existing` is the existing-pinned, no-fetch artifact mode: it requires
`SURGEIST_BROWSER_PATH` to be a repository-relative executable beneath the
manifest cache and verifies its exact `--version` output before any XML or
report write. Both modes use the same headless launch profile, including
`use-mock-keychain`.

`check-corpus`, `check-taffy-corpus`, and `import-taffy` are browser-free command
paths. In particular, `check-corpus` treats
`tests/layout/browser_parity/xml/generation-reports/all.json` as the sole
generated-artifact provenance authority: it verifies the shared generator,
browser, launch-profile, helper, base-style, corpus, and Taffy metadata plus each
generated entry's repository-relative source/output identity, source and linked
resource hashes, and XML hash. Generated XML is comment-free, and embedded
provenance is rejected without reading a browser selection or generation filter.

Layout owns normalized layout values, algorithm inputs, traversal contracts,
caches, reports, and box output. Retained tree identity and sibling coordination
belong to the root integration layer that provides the tree implementation.

## Modeling Contracts

`surgeist-layout` exposes layout-ready contracts rather than authored CSS syntax.
`LengthPercentageOf<S>` is a normalized finite affine value: px plus a percentage
coefficient. Resolve it only against an explicit `PercentageBasisOf<S>`;
`PercentageBasisOf::definite` rejects invalid values at construction. Resolution
reports `MissingBasis` for a required missing basis and `InvalidNumeric` for a
non-finite evaluation; no value is guessed.

Preferred size, minimum size, maximum size, and flex basis are distinct closed
property domains. Their role-valid keywords cannot cross property boundaries.
Direct `FlexBasisOf::MIN_CONTENT` and `FlexBasisOf::MAX_CONTENT` values retain
their distinct intrinsic measurement constraints through the public layout
front door; neither is normalized to the generic content basis.
`SizingCalculationOf<S>` combines finite affine leaves with nested `min`, `max`,
and `clamp` in a validated program that is evaluated iteratively. Percentages
remain symbolic until layout receives an explicit basis, and a required missing
basis remains unresolved.

`NodeInputOf::flex_item_collapse` is a normalized, layout-ready flex effect, and
`FlexItemCollapse::Normal` is its default. A collapsed in-flow flex item
participates through a finite cross-size strut replay, publishes zero committed
collapsed geometry, and hides its descendants. Root `surgeist` owns
computed-style lowering from a flex item's `visibility: collapse` to this
normalized state, while rendering owns painting. This leaf does not parse
authored CSS or provide a general visibility model.

Canonical layout-ready `calc-size()` input pairs a property-specific basis with a
validated `CalcSizeCalculationOf<S>` containing finite absolute-pixel,
percentage, and size coefficients. Track flex is separate: construct a finite,
non-negative `TrackFlexFactorOf<S>` through `try_new` and place it only in a
maximum track breadth. A valid sizing behavior owned by a later algorithm is
reported through `LayoutUnsupportedCapability::SizingBehavior` with the exact
property, behavior, algorithm, and axis instead of an automatic fallback.

`LayoutRootRequestOf<S>` validates root input for the public
`compute_layout` front door. A successful call returns a
`CompletedLayoutBatchOf<Node, S>` containing the staged layout and cache updates;
a `LayoutErrorOf<Node, S, M>` returns no partial public result. Recursive
algorithm modes remain internal.

`compute_leaf` is the direct, fallible leaf-measurement boundary. Its provider
receives non-negative content-space constraints, and invalid provider output or a
provider error becomes a typed layout error. Resolved padding and border must be
non-negative and finite, including their combined axis sums. Overflow in
layout-owned box arithmetic returns `LayoutInvalidInputOf::InvalidNumeric`;
invalid provider dimensions retain the distinct `MeasurementOutput` diagnostic.
Valid fully known sizing requests can complete without calling the provider.

Visible inline boundaries and line breaks must match their containing block's
`FlowAxes`. A mismatch returns `LayoutInvalidInputOf::InlineFlowMismatch` with
the expected and actual axes, the container and control node, and the
`ChildLayout` operation. Hidden line breaks retain their existing bypass.

`ItemOrder` is the layout-ready signed order value. `SourceIndex` is stable
source-sibling identity: outputs remain source-associated while flex, ordinary
grid, and grid-lanes consume one stable order-modified traversal sorted by item
order and then source index.

`item_is_replaced` is an independent box-generation fact. It is not inferred
from table role, measurement, aspect ratio, or stretch. Block and root sizing
use it to avoid ordinary auto-inline fill, flex uses it when selecting automatic
main-size suggestions, and grid and grid-lanes use it when resolving normal
alignment while preserving explicit stretch.

`ContainingLayoutContext` keeps the containing flow axes and
`ParentFormattingContext` role together as the complete containing context and
cache identity. Flex-item roots require explicit parent flow axes and keep the
host allocation in the root request separate from the viewport percentage
context in `FlexItemRootContext`.

Root `surgeist` owns cross-crate adapters, including canonicalizing authored CSS
sizing values, lowering computed-style values and authored CSS order into these
layout contracts, and rejecting property-invalid authored states before layout.
This crate does not parse authored CSS. Root also owns box-generation
replacedness, invalidation, consumer migration and renames, facade composition,
integration, and generated API artifacts; this crate carries no root adapters or
API artifact copies.

## Normalized Grid Boundary

The grid front door consumes normalized layout facts. Layout owns finite track
components and repetitions, explicit and implicit topology, integer placement,
named lines and validated template-area cells, auto flow, gaps, alignment,
layout-ready order and replacedness, flow axes, and canonical physical box and
scroll output. Lengths, percentages, intrinsic keywords, fit-content limits,
and flex factors enter through their existing validated property domains; this
boundary is not an authored-CSS grammar.

The implemented CSS Grid Level 2 subset is deliberately narrower than broad
conformance: ordinary grids and inherited-axis subgrids compose through shared
topology, placement-before-sizing, auto-fit occupancy, fit-content/flex sizing,
auto-maximum stretch, named occurrences, template areas, standalone-axis
measurement boundaries, inherited baseline data, and canonical overflow
contribution. Baseline grouping remains the existing inherited-axis model.
Grid-aligned positioned layout, additional baseline distribution, and
fragmentation remain separately owned work.

Inherited subgrid axes keep their inherited track extent. Definite placements
are clamped to that extent before automatic placement; settled automatic
placements are clamped afterward and may overlap when capacity is exhausted.
Only standalone axes materialize implicit tracks. Sparse placement retains
progress per locked major track, while dense placement searches earlier holes.

The experimental CSS Grid Level 3 subset is limited to `grid-lanes` single-axis
packing: hybrid item containing blocks, intrinsic candidate projection,
lanes-specific auto-fit collapse, and nested indefinite subgrid contribution
projection. It does not claim stacking-axis grid-lanes baseline alignment or
complete Level 3 behavior.

Root `surgeist` owns authored CSS parsing, cascade and computed-value
normalization, lowering into these finite inputs, cross-crate integration,
retained identity, and generated API artifacts. The browser fixture adapter is
only a constrained layout-ready test bridge and is not a second CSS front door.

## Geometry, Flow, And Scroll Contracts

The public physical geometry contract uses x/y points, width/height sizes, and
top/right/bottom/left edges. Public layout outputs, cached geometry, and scroll
geometry remain physical. Layout algorithms may use
crate-private logical algorithm geometry while working in inline/block
coordinates. Those carriers stay private until the owning `FlowAxes` projects
them to physical geometry at a contextual boundary.

`FlowAxes` is the sole production owner of writing-mode mapping for
`HorizontalTb`, `VerticalRl`, `VerticalLr`, `SidewaysRl`, and `SidewaysLr`. Its
`Direction` is the already-resolved used inline direction, not authored or
otherwise unresolved CSS. Root `surgeist` owns computed-style lowering and
supplies that used value through its cross-crate adapters.

Scroll inputs are normalized computed or otherwise layout-ready values, not
authored CSS syntax. `ComputedOverflow` atomically validates the two computed
axes; layout privately derives used overflow from that pair and replacedness.
`OverflowClipMarginOf`, `ScrollPaddingOf`, `ScrollMarginOf`, and the snap types
carry finite closed inputs. `ScrollbarWidthOf` is the explicit physical
thickness selected by the caller's overlay/classic scrollbar environment, so
layout neither probes host metrics nor guesses a missing policy.

Successful layout may publish immutable `ScrollGeometryOf`. Its read-only
helpers expose canonical border, padding, content and scrollport boxes;
independent x/y clips; physical-edge gutter rectangles; the optimal viewing
region; used overflow; and one signed physical range containing the zero
initial anchor. The canonical scroll size on each axis is `maximum - minimum`,
including zero. `NodeOutputOf::content_box_size()` and `scrollbar_size()` derive
from that same geometry instead of maintaining mutable duplicate state.

Every present geometry also contains `ScrollTargetGeometryOf`: the target's
local physical border box and scroll margin plus its flow axes, block/inline snap
alignment, and snap-stop metadata. Root consumes these values after retained
association and coordinate transformation. Root also owns authored CSS parsing,
computed-style normalization and lowering, explicit host scrollbar policy,
current offsets, focus/target scrolling, snap-container association and
selection, CSSOM, scrollbar UI, host events, and invalidation. This crate does
not claim a live scrolling runtime.

The general signed physical and flow-relative range types keep finite ordered
minimum and maximum bounds. When an axis runs in reverse, `FlowAxes` swaps and
negates endpoints so negative minima and maxima retain their meaning.

Root also owns generated API artifacts. Later initiatives and the aggregate
release gate remain outside this FRI-05 leaf contract and are not claimed here.

## Inline Metrics Contract

`InlineMetricsOf<S>` is layout-ready line box data. Layout consumes it for inline
line construction and does not derive it from authored CSS or fonts. Integration
layers should provide metrics from computed style and text/font measurement
before constructing `LineBreakInputOf<S>`.
