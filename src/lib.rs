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
mod traits;
mod value;

pub type Scalar = f32;

pub use block::compute_block;
pub use cache::{Cache, CacheKeyContext, ClearState};
pub use compute::{compute_hidden, compute_leaf, compute_root, round_layout};
pub use flex::compute_flex;
pub use geometry::{Axis, Edges, Point, Size};
pub use grid::{
    DefiniteLaneIntrinsicItem, GridAxisKind, GridComputation, GridComputationReport,
    IndefiniteLaneContributionGroup, LaneContributionFacts, LaneIntrinsicItem,
    LaneIntrinsicSizingInput, LaneIntrinsicSizingReport, LaneItem, LaneItemOffset,
    LanePlacementError, LanePlacementInput, LanePlacementReport, LaneTrackSpan,
    NamedGridErrorReport, NamedGridReport, compute_grid, compute_grid_with_report,
    grid_axis_for_lanes, lane_axis, lane_intrinsic_sizing, place_lanes,
};
pub use node_input::{
    AlignContent, AlignItems, BoxSizing, Clear, Direction, Display, FlexDirection, FlexWrap, Float,
    GridAutoFlow, GridFlowTolerance, GridPlacement, NodeInput, Overflow, Position, RawGridLine,
    RawGridPlacement, TextAlign, VerticalAlign, WritingMode,
};
pub use output::{
    Baselines, CollapsibleMargin, ComputeInput, ComputeOutput, NodeOutput, RequestedAxis, RunMode,
    SizingMode,
};
pub use traits::{CacheAccess, Compute, Round, Traverse, compute_cached};
pub use value::{
    AspectRatio, Available, CalcExpression, CalcGeneration, CalcId, CalcResolution,
    CalcResolutionStatus, CalcResolver, CalcTerm, Dimension, LayoutCalcStore, Length, LengthAuto,
    NoCalcResolver,
};
pub use value::{
    GridLine, GridSpan, GridTemplateAreaRow, GridTemplateAreas, MaxTrackSizing, MinTrackSizing,
    SubgridLineNameComponent, SubgridLineNameRepeatCount, SubgridTrack, TrackComponent,
    TrackComponentList, TrackRepeat, TrackRepeatCount, TrackRepetition, TrackRepetitionError,
    TrackSizing, track_sizing_components,
};

#[cfg(test)]
mod tests;
