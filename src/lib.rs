//! Native layout algorithm boundary for Surgeist.
//!
//! This crate owns normalized layout-ready values, algorithm-facing geometry,
//! traversal contracts, layout caches, and final box output. It does not parse
//! authored CSS or own retained tree identity, cross-crate adapters, or generated
//! API artifacts; root `surgeist` owns those integration concerns.
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
pub use geometry::{Axis, Edges, Point, Size};
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
    InlineBoundaryKind, InlineMetrics, InlineMetricsError, InlineMetricsOf, LayoutInput,
    LayoutInputOf, LineBreakDisplay, LineBreakInput, LineBreakInputOf, NodeInput, NodeInputOf,
    Overflow, Position, RawGridLine, RawGridPlacement, ScrollbarWidth, ScrollbarWidthOf, TextAlign,
    VerticalAlign, WritingMode,
};
pub use output::{
    Baselines, BaselinesOf, CollapsibleMargin, CollapsibleMarginOf, CompletedLayoutBatch,
    CompletedLayoutBatchOf, ComputeInput, ComputeInputOf, ComputeOutput, ComputeOutputOf,
    FlexItemRootContext, FlexItemRootContextOf, LayoutCacheClearEntry, LayoutCacheStoreEntry,
    LayoutCacheStoreEntryOf, LayoutOutputEntry, LayoutOutputEntryOf, LayoutRootContext,
    LayoutRootContextOf, LayoutRootRequest, LayoutRootRequestOf, LayoutRoundingMode, NodeOutput,
    NodeOutputOf, RootAvailabilityError, RootAvailabilityErrorOf,
};
pub(crate) use output::{RequestedAxis, RunMode, SizingMode};
/// Supported scalar contract for generic layout APIs.
pub use scalar::LayoutScalar;
pub use scroll::{
    ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry, ScrollGeometryOf, ScrollOffset,
    ScrollOffsetOf, ScrollOverflowCouplingPolicy, ScrollOverflowExposure, ScrollRange,
    ScrollRangeOf, ScrollRect, ScrollRectOf, ScrollUnsupportedFeature, ScrollbarGutterRects,
    ScrollbarGutterRectsOf,
};
#[cfg(test)]
pub(crate) use traits::compute_cached;
pub(crate) use traits::{CacheAccess, Compute, Round};
pub use traits::{LayoutTree, Traverse};
pub use value::{
    AspectRatio, AspectRatioOf, Available, AvailableOf, Dimension, DimensionOf,
    FiniteScalarErrorOf, Length, LengthAuto, LengthAutoOf, LengthOf, LengthPercentageErrorOf,
    LengthPercentageOf, LengthResolution, LengthResolutionOf, LengthResolutionStatus,
    NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf, NumericResolutionOf, PercentageBasisOf,
    ResolvedLengthAuto, ResolvedLengthAutoOf, UnresolvedLengthReason,
};
pub use value::{
    GridLine, GridSpan, GridTemplateAreaRow, GridTemplateAreas, MaxTrackSizing, MaxTrackSizingOf,
    MinTrackSizing, MinTrackSizingOf, SubgridLineNameComponent, SubgridLineNameRepeatCount,
    SubgridTrack, TrackComponent, TrackComponentList, TrackComponentListOf, TrackComponentOf,
    TrackRepeat, TrackRepeatCount, TrackRepetition, TrackRepetitionError, TrackRepetitionOf,
    TrackSizing, TrackSizingOf, track_sizing_components, track_sizing_components_of,
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
