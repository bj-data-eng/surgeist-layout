//! Small grid oracle helpers for tests.
//!
//! These helpers produce expected numbers for focused grid algorithm phases:
//! positive numeric placement, track sizing, contribution arithmetic, alignment,
//! and thin scenario composition. They are intentionally not a second production
//! layout engine: no tree traversal, no style resolution, no parser, and no child
//! measurement.

pub mod alignment;
pub mod axis;
pub mod baseline;
pub mod contributions;
pub mod lanes;
pub mod named;
pub mod placement;
pub mod scenario;
pub mod subgrid;
pub mod tracks;

pub use alignment::{AlignmentSafety, TrackAlignment, align_tracks, align_tracks_report};
pub use axis::{AxisMappingInput, OracleDirection, OracleWritingMode, map_axis};
pub use baseline::{
    BaselineAlignment, BaselineFallback, BaselineGeometry, BaselineGroupInput, BaselineGroupKind,
    BaselineGroupReport, BaselineItemFacts, BaselineShim, ContainerBaselineFallbackItem,
    ContainerBaselineInput, SubgridBaselineInheritanceInput, SubgridBaselinePublicationInput,
    baseline_groups, baseline_intrinsic_shim, baseline_offset, baseline_participation,
    container_baselines, inherit_subgrid_baselines, publish_subgrid_baseline,
};
pub use contributions::{ContributionSize, ItemContributionFacts, ItemContributions};
pub use lanes::{
    GridLanesBaselineInput, GridLanesBaselineReason, LaneAutoFlow, LaneFlowTolerance,
    LaneIntrinsicItem, LaneIntrinsicSizingInput, LaneIntrinsicSizingReport, LaneItemInput,
    LanePlacementInput, LanePlacementReport, LaneTrackSpanLength, grid_axis_for_lanes,
    grid_lanes_baseline_policy, grid_lanes_container_baselines, lane_axis, lane_intrinsic_sizing,
    place_lanes,
};
pub use named::{
    LineNameOrigin, NamedAxisPlacement, NamedGridError, NamedGridLine, NamedGridLines,
    NamedLineOccurrence, NamedPlacementConflictResolution, NamedTrackComponent,
    SubgridNameComponent, SubgridNameRepeatCount, TemplateAreas, area_generated_facts,
    area_generated_lines, expand_axis_shorthand, expand_grid_area_shorthand,
    expand_named_fixed_repeat, expand_subgrid_name_list, inherit_named_subgrid_lines,
    resolve_anonymous_span_from_end, resolve_anonymous_span_from_start, resolve_named_area,
    resolve_named_axis_placement, resolve_named_grid_area_report, resolve_named_line,
    resolve_named_span_from_end, resolve_named_span_from_start,
    resolve_named_subgrid_axis_placement, resolve_numeric_line,
};
pub use placement::{
    AutoPlacer, Flow, GridArea, GridAxis, ItemPlacement, LinePlacement, PlacementCursor,
    PlacementError, PlacementReport,
};
pub use scenario::{
    BaselineAlignedItemRectInput, GridItemRect, GridScenarioReport, LaneItemRectInput,
    SubgridItemRectInput, compose_baseline_aligned_item_rect, compose_grid_scenario,
    compose_lane_item_rect, compose_subgrid_item_rect,
};
pub use subgrid::{
    AxisEdges, OracleGapReport, OracleGridError, SubgridAxisKind, SubgridChild,
    SubgridEligibilityInput, SubgridIneligibleReason, SubgridLeaf, SubgridNode,
    SubgridTrackInheritanceInput, SubgridTraversalInput, TrackSpan, inherit_subgrid_tracks,
    subgrid_eligibility, traverse_subgrid_intrinsic,
};
pub use tracks::{
    DefiniteTracks, EqualShareIntrinsicTracks, GridTrack, GrowthLimit, Track, TrackMax, TrackMin,
    TrackSize, TrackSizingError, TrackSizingReport, TrackSizingSlice,
};
