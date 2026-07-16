//! Native layout algorithm boundary for Surgeist.
//!
//! The public physical geometry contract uses x/y points, width/height sizes,
//! and top/right/bottom/left edges. Public layout outputs, cached geometry, and
//! scroll geometry remain physical. Layout algorithms may use
//! crate-private logical algorithm geometry while working in inline/block
//! coordinates. Those carriers stay private until the owning [`FlowAxes`]
//! projects them to physical geometry at a contextual boundary.
//!
//! [`FlowAxes`] is the sole production owner of writing-mode mapping for
//! [`WritingMode::HorizontalTb`], [`WritingMode::VerticalRl`],
//! [`WritingMode::VerticalLr`], [`WritingMode::SidewaysRl`], and
//! [`WritingMode::SidewaysLr`]. Its [`Direction`] is the already-resolved
//! used inline direction, not authored or otherwise unresolved CSS. Root
//! `surgeist` owns computed-style lowering and supplies that used value through
//! its cross-crate adapters.
//!
//! The signed physical scroll ranges and signed flow-relative scroll ranges keep
//! finite ordered minimum and maximum bounds. When an axis runs in reverse,
//! [`FlowAxes`] swaps and negates the endpoints so negative minima and maxima
//! retain their meaning. Layout owns scroll-container geometry, not a current
//! offset; root integration owns live scroll state and host/CSSOM policy.
//!
//! `LengthPercentageOf<S>` is a normalized finite affine value (px plus a
//! percentage coefficient) resolved explicitly with `PercentageBasisOf<S>`.
//! `LayoutRootRequestOf<S>` validates root input for `compute_layout`, which
//! returns either a complete `CompletedLayoutBatchOf<Node, S>` or a typed
//! `LayoutErrorOf<Node, S, M>` with no partial public result. Recursive compute
//! modes are internal.
//!
//! `compute_leaf` is the direct fallible measurement boundary: providers receive
//! non-negative content-space constraints and provider failures or invalid output
//! become typed layout errors. `DefaultScalar` and `Scalar` use `f32`; generic
//! `*Of<S>` contracts support end-to-end `f32` and `f64` scalar lanes.
//!
//! [`ItemOrder`] is the layout-ready signed order value. [`SourceIndex`] is
//! stable source-sibling identity: outputs remain source-associated while flex,
//! ordinary grid, and grid-lanes consume one stable order-modified traversal
//! sorted by item order and then source index.
//!
//! [`NodeInputOf::item_is_replaced`] is an independent box-generation fact, not
//! inferred from table role, measurement, aspect ratio, or stretch. Block and
//! root sizing use it to avoid ordinary auto-inline fill; flex uses it for
//! automatic main-size suggestion selection; and grid and grid-lanes use it for
//! normal alignment while preserving explicit stretch.
//!
//! [`ContainingLayoutContext`] and [`ParentFormattingContext`] form the complete
//! containing context and cache identity, including explicit containing flow and
//! parent role. Flex-item roots require explicit parent flow axes and keep host
//! allocation in the root request separate from the viewport percentage context
//! in [`FlexItemRootContext`].
//!
//! Browser-parity generation is crate-local tooling. `generate` is the
//! managed-pinned mode and may use the configured fetcher for the exact manifest
//! pin. `generate-existing` is the existing-pinned, no-fetch mode and accepts
//! only a repository-relative executable under that manifest cache whose exact
//! `--version` matches the pin. Both use the shared headless launch profile,
//! including the mock-keychain argument; corpus freshness checks remain
//! browser-free.
//!
//! Root `surgeist` owns authored CSS order lowering, box-generation replacedness,
//! invalidation, consumer migration and renames, facade composition, cross-crate
//! integration, retained identity, live scroll state, and generated API artifacts.
//! This crate does not parse authored CSS or own those integration concerns.
//! The later inline, overflow, flex, grid, alignment, and positioned initiatives
//! remain outside this geometry closure and are not claimed here.
//!
//! ```compile_fail
//! use surgeist_layout::{LogicalEdgesOf, LogicalPointOf, LogicalRectOf, LogicalSizeOf};
//! ```

mod block;
mod cache;
mod compute;
mod flex;
mod geometry;
mod grid;
mod inline;
mod node_input;
mod output;
mod scalar;
mod scroll;
mod sizing;
mod traits;
mod value;

/// Default scalar precision used by the non-generic public layout aliases.
///
/// This is intentionally `f32`, matching the crate's browser-parity fixture
/// boundary and the default Surgeist layout coordinate contract.
pub type DefaultScalar = f32;

/// Convenience alias for the default scalar precision.
///
/// Use explicit `*Of<S>` types with `S: LayoutScalar` when one layout tree
/// needs to run end-to-end with a different supported precision such as `f64`.
pub type Scalar = DefaultScalar;

#[cfg(test)]
pub(crate) use block::compute_block;
pub use cache::{Cache, CacheKeyContext, CacheOf, ClearState};
pub use compute::{
    InvalidMeasurementOutput, InvalidMeasurementOutputOf, LayoutError, LayoutErrorKind,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSite, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutInvalidInput, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation, LayoutResult,
    LayoutResultOf, LayoutUnsupportedCapability, LeafMeasureError, LeafMeasureErrorOf,
    LeafMeasureInput, LeafMeasureInputOf, MeasurementAvailable, MeasurementAvailableOf,
    compute_layout, compute_leaf,
};
#[cfg(test)]
pub(crate) use compute::{compute_hidden, compute_root, round_layout};
#[cfg(test)]
pub(crate) use flex::compute_flex;
pub use geometry::{Edges, FlowAxes, LogicalAxis, PhysicalAxis, PhysicalSide, Point, Size};
pub use grid::{
    DefiniteLaneIntrinsicItem, DefiniteLaneIntrinsicItemOf, GridAxisKind, GridComputation,
    GridComputationOf, GridComputationReport, IndefiniteLaneContributionGroup,
    IndefiniteLaneContributionGroupOf, LaneContributionFacts, LaneContributionFactsOf,
    LaneIntrinsicItem, LaneIntrinsicItemKind, LaneIntrinsicItemOf, LaneIntrinsicSizingInput,
    LaneIntrinsicSizingInputOf, LaneIntrinsicSizingReport, LaneIntrinsicSizingReportOf, LaneItem,
    LaneItemOf, LaneItemOffset, LaneItemOffsetOf, LanePlacementError, LanePlacementInput,
    LanePlacementInputOf, LanePlacementReport, LanePlacementReportOf, LaneTrackSpan,
    LaneTrackSpanLength, NamedGridErrorReport, NamedGridReport, grid_axis_for_lanes, lane_axis,
    lane_intrinsic_sizing, place_lanes,
};
#[cfg(test)]
pub(crate) use grid::{compute_grid, compute_grid_with_report};
pub use node_input::{
    AlignContent, AlignItems, BoxSizing, Clear, Direction, Display, FlexDirection, FlexGrow,
    FlexGrowOf, FlexShrink, FlexShrinkOf, FlexWrap, Float, GridAutoFlow, GridFlowTolerance,
    GridFlowToleranceOf, GridPlacement, InlineBoundaryInput, InlineBoundaryInputOf,
    InlineBoundaryKind, InlineMetrics, InlineMetricsError, InlineMetricsOf, ItemOrder, LayoutInput,
    LayoutInputOf, LineBreakDisplay, LineBreakInput, LineBreakInputOf, NodeInput, NodeInputOf,
    Overflow, Position, RawGridLine, RawGridPlacement, ScrollbarWidth, ScrollbarWidthOf, TextAlign,
    VerticalAlign, WritingMode,
};
pub use output::{
    Baselines, BaselinesOf, CollapsibleMargin, CollapsibleMarginOf, CompletedLayoutBatch,
    CompletedLayoutBatchOf, ComputeInput, ComputeInputOf, ComputeOutput, ComputeOutputOf,
    ContainingLayoutContext, FlexItemRootContext, FlexItemRootContextOf, LayoutCacheClearEntry,
    LayoutCacheStoreEntry, LayoutCacheStoreEntryOf, LayoutOutputEntry, LayoutOutputEntryOf,
    LayoutRootContext, LayoutRootContextOf, LayoutRootRequest, LayoutRootRequestOf,
    LayoutRoundingMode, NodeOutput, NodeOutputOf, ParentFormattingContext,
    PhysicalBlockMarginCollapse, PhysicalBlockMarginCollapseOf, RootAvailabilityError,
    RootAvailabilityErrorOf, SourceIndex,
};
pub(crate) use output::{RequestedAxis, RunMode, SizingMode};
/// Supported scalar contract for generic layout APIs.
pub use scalar::LayoutScalar;
pub use scroll::{
    FlowRelativeScrollAxisRange, FlowRelativeScrollAxisRangeOf, FlowRelativeScrollOffset,
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRange, FlowRelativeScrollRangeOf,
    PhysicalScrollAxisRange, PhysicalScrollAxisRangeOf, PhysicalScrollOffset,
    PhysicalScrollOffsetOf, PhysicalScrollRange, PhysicalScrollRangeOf, ScrollContainerAxis,
    ScrollContainerFacts, ScrollCoordinateError, ScrollCoordinateErrorOf, ScrollGeometry,
    ScrollGeometryOf, ScrollOverflowCouplingPolicy, ScrollOverflowExposure, ScrollRect,
    ScrollRectOf, ScrollUnsupportedFeature, ScrollbarGutterRects, ScrollbarGutterRectsOf,
};
pub use sizing::{
    CalcSizeCalculation, CalcSizeCalculationErrorOf, CalcSizeCalculationOf,
    CalcSizeConstructionError, FlexBasis, FlexBasisCalcBasis, FlexBasisOf, MaxSize,
    MaxSizeCalcBasis, MaxSizeOf, MinSize, MinSizeCalcBasis, MinSizeOf, PreferredSize,
    PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation, SizingCalculationError,
    SizingCalculationOf,
};
#[cfg(test)]
pub(crate) use traits::compute_cached;
pub(crate) use traits::{CacheAccess, Compute, Round};
pub use traits::{LayoutTree, Traverse};
pub use value::{
    AspectRatio, AspectRatioOf, Available, AvailableOf, FiniteScalarErrorOf, Length, LengthAuto,
    LengthAutoOf, LengthOf, LengthPercentageErrorOf, LengthPercentageOf, LengthResolution,
    LengthResolutionOf, LengthResolutionStatus, NonNegativeFiniteOf,
    NonNegativeFiniteScalarErrorOf, NumericResolutionOf, PercentageBasisOf, ResolvedLengthAuto,
    ResolvedLengthAutoOf, UnresolvedLengthReason,
};
pub use value::{
    GridLine, GridSpan, GridTemplateAreaRow, GridTemplateAreas, MaxTrackSizing, MaxTrackSizingOf,
    MinTrackSizing, MinTrackSizingOf, SubgridLineNameComponent, SubgridLineNameRepeatCount,
    SubgridTrack, TrackComponent, TrackComponentList, TrackComponentListOf, TrackComponentOf,
    TrackFlexFactor, TrackFlexFactorOf, TrackRepeat, TrackRepeatCount, TrackRepetition,
    TrackRepetitionError, TrackRepetitionOf, TrackSizing, TrackSizingOf, track_sizing_components,
    track_sizing_components_of,
};

#[cfg(test)]
mod block_tests;
#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod compute_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod flex_tests;
#[cfg(test)]
mod inline_tests;
#[cfg(test)]
mod leaf_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod root_tests;
#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod test_support;
