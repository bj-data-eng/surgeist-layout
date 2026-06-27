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
mod traits;
mod value;

pub type DefaultScalar = f32;
pub type Scalar = DefaultScalar;

pub use block::compute_block;
pub use cache::{Cache, CacheKeyContext, ClearState};
pub use compute::{compute_hidden, compute_leaf, compute_root, round_layout};
pub use flex::compute_flex;
pub use geometry::{Axis, Edges, Point, Size};
pub use grid::{
    DefiniteLaneIntrinsicItem, GridAxisKind, GridComputation, GridComputationReport,
    IndefiniteLaneContributionGroup, LaneContributionFacts, LaneIntrinsicItem,
    LaneIntrinsicItemKind, LaneIntrinsicSizingInput, LaneIntrinsicSizingReport, LaneItem,
    LaneItemOffset, LanePlacementError, LanePlacementInput, LanePlacementReport, LaneTrackSpan,
    LaneTrackSpanLength, NamedGridErrorReport, NamedGridReport, compute_grid,
    compute_grid_with_report, grid_axis_for_lanes, lane_axis, lane_intrinsic_sizing, place_lanes,
};
pub use node_input::{
    AlignContent, AlignItems, BoxSizing, Clear, Direction, Display, FlexDirection, FlexWrap, Float,
    GridAutoFlow, GridFlowTolerance, GridFlowToleranceOf, GridPlacement, NodeInput, NodeInputOf,
    Overflow, Position, RawGridLine, RawGridPlacement, TextAlign, VerticalAlign, WritingMode,
};
pub use output::{
    Baselines, BaselinesOf, CollapsibleMargin, CollapsibleMarginOf, ComputeInput, ComputeInputOf,
    ComputeOutput, ComputeOutputOf, NodeOutput, NodeOutputOf, RequestedAxis, RunMode, SizingMode,
};
pub use scalar::LayoutScalar;
pub use traits::{CacheAccess, Compute, Round, Traverse, compute_cached};
pub use value::{
    AspectRatio, AspectRatioOf, Available, AvailableOf, CalcExpression, CalcExpressionOf,
    CalcGeneration, CalcId, CalcResolution, CalcResolutionOf, CalcResolutionStatus, CalcResolver,
    CalcTerm, CalcTermOf, CalcUnresolvedReason, Dimension, DimensionOf, LayoutCalcStore,
    LayoutCalcStoreOf, Length, LengthAuto, LengthAutoOf, LengthOf, NoCalcResolver,
    ResolvedLengthAuto, ResolvedLengthAutoOf,
};
pub use value::{
    GridLine, GridSpan, GridTemplateAreaRow, GridTemplateAreas, MaxTrackSizing, MaxTrackSizingOf,
    MinTrackSizing, MinTrackSizingOf, SubgridLineNameComponent, SubgridLineNameRepeatCount,
    SubgridTrack, TrackComponent, TrackComponentList, TrackComponentListOf, TrackComponentOf,
    TrackRepeat, TrackRepeatCount, TrackRepetition, TrackRepetitionError, TrackRepetitionOf,
    TrackSizing, TrackSizingOf, track_sizing_components, track_sizing_components_of,
};

#[cfg(test)]
mod tests;
