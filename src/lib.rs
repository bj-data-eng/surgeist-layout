//! Native layout algorithm boundary for Surgeist.
//!
//! This module owns layout input values, algorithm-facing geometry, layout
//! traversal contracts, layout caches, and final box output. It is being ported
//! into Surgeist-shaped types and public boundaries.

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

pub use block::compute_block;
pub use cache::{Cache, CacheKeyContext, CacheOf, ClearState};
pub use compute::{compute_hidden, compute_leaf, compute_root, round_layout};
pub use flex::compute_flex;
pub use geometry::{Axis, Edges, Point, Size};
pub use grid::{
    DefiniteLaneIntrinsicItem, DefiniteLaneIntrinsicItemOf, GridAxisKind, GridComputation,
    GridComputationOf, GridComputationReport, IndefiniteLaneContributionGroup,
    IndefiniteLaneContributionGroupOf, LaneContributionFacts, LaneContributionFactsOf,
    LaneIntrinsicItem, LaneIntrinsicItemKind, LaneIntrinsicItemOf, LaneIntrinsicSizingInput,
    LaneIntrinsicSizingInputOf, LaneIntrinsicSizingReport, LaneIntrinsicSizingReportOf, LaneItem,
    LaneItemOf, LaneItemOffset, LaneItemOffsetOf, LanePlacementError, LanePlacementInput,
    LanePlacementInputOf, LanePlacementReport, LanePlacementReportOf, LaneTrackSpan,
    LaneTrackSpanLength, NamedGridErrorReport, NamedGridReport, compute_grid,
    compute_grid_with_report, grid_axis_for_lanes, lane_axis, lane_intrinsic_sizing, place_lanes,
};
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
    Baselines, BaselinesOf, CollapsibleMargin, CollapsibleMarginOf, ComputeInput, ComputeInputOf,
    ComputeOutput, ComputeOutputOf, NodeOutput, NodeOutputOf, RequestedAxis, RunMode, SizingMode,
};
/// Supported scalar contract for generic layout APIs.
pub use scalar::LayoutScalar;
pub use scroll::{
    ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry, ScrollGeometryOf, ScrollOffset,
    ScrollOffsetOf, ScrollOverflowCouplingPolicy, ScrollOverflowExposure, ScrollRange,
    ScrollRangeOf, ScrollRect, ScrollRectOf, ScrollUnsupportedFeature, ScrollbarGutterRects,
    ScrollbarGutterRectsOf,
};
pub use traits::{CacheAccess, Compute, Round, Traverse, compute_cached};
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
