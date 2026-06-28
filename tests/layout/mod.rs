use std::collections::HashMap;

use crate::support;

use surgeist_layout::{
    AlignContent, AlignItems, AspectRatio, Available, AvailableOf, Axis, BoxSizing, Cache,
    CacheAccess, CacheOf, ClearState, CollapsibleMargin, Compute, ComputeInput, ComputeInputOf,
    ComputeOutput, ComputeOutputOf, Dimension, DimensionOf, Direction, Display, Edges,
    FlexDirection, FlexWrap, Float, GridAutoFlow, GridFlowTolerance, GridPlacement, Length,
    LengthAuto, LengthAutoOf, LengthOf, MaxTrackSizing, MinTrackSizing, NodeInput, NodeInputOf,
    NodeOutput, NodeOutputOf, Overflow, Point, Position, RequestedAxis, Round, RunMode, Scalar,
    Size, SizingMode, TextAlign, TrackComponent, Traverse, WritingMode, compute_cached,
    compute_flex, compute_hidden, compute_leaf, compute_root, round_layout,
};

use support::oracle::grid::{
    AlignmentSafety, AutoPlacer, ContributionSize, DefiniteTracks, Flow, GridArea, GridTrack,
    ItemContributionFacts, LinePlacement, Track, TrackAlignment, TrackSizingSlice,
    align_tracks_report,
};

fn output_from_known_or(input: ComputeInput, fallback: Size) -> ComputeOutput {
    let size = Size::new(
        input.known.width.unwrap_or(fallback.width),
        input.known.height.unwrap_or(fallback.height),
    );
    ComputeOutput::from_sizes(size, size)
}

#[path = "unit/block.rs"]
mod block;
mod browser_parity;
#[path = "unit/cache.rs"]
mod cache;
#[path = "unit/contract.rs"]
mod contract;
#[path = "unit/flex.rs"]
mod flex;
#[path = "unit/grid.rs"]
mod grid;
#[path = "unit/leaf.rs"]
mod leaf;
#[path = "unit/root.rs"]
mod root;
