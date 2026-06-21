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

#[allow(unused_imports)]
pub use alignment::{
    AlignmentReport, AlignmentSafety, TrackAlignment, align_tracks, align_tracks_report,
};
#[allow(unused_imports)]
pub use axis::{
    AxisMappingError, AxisMappingInput, AxisMappingReport, OracleDirection, OracleWritingMode,
    map_axis, opposite_axis,
};
#[allow(unused_imports)]
pub use baseline::{
    BaselineAlignment, BaselineFallback, BaselineGeometry, BaselineGroupInput, BaselineGroupKind,
    BaselineGroupReport, BaselineItemFacts, BaselineParticipationReport, BaselineShim,
    ContainerBaselineFallbackItem, ContainerBaselineInput, ContainerBaselineReport,
    SubgridBaselineInheritanceInput, SubgridBaselineInheritanceReport,
    SubgridBaselinePublicationInput, SubgridBaselinePublicationReport, baseline_groups,
    baseline_intrinsic_shim, baseline_offset, baseline_participation, container_baselines,
    inherit_subgrid_baselines, publish_subgrid_baseline,
};
#[allow(unused_imports)]
pub use contributions::{ContributionSize, ItemContributionFacts, ItemContributions};
#[allow(unused_imports)]
pub use lanes::{
    DefiniteLaneIntrinsicItem, GridLanesBaselineInput, GridLanesBaselinePolicyReport,
    GridLanesBaselineReason, IndefiniteLaneContributionGroup, LaneAutoFlow, LaneFlowTolerance,
    LaneIntrinsicItem, LaneIntrinsicSizingInput, LaneIntrinsicSizingReport, LaneItemInput,
    LaneItemOffset, LanePlacementInput, LanePlacementReport, grid_axis_for_lanes,
    grid_lanes_baseline_policy, grid_lanes_container_baselines, lane_axis, lane_intrinsic_sizing,
    place_lanes,
};
#[allow(unused_imports)]
pub use named::{
    AreaGeneratedFacts, AreaRectangle, ClippedAreaSource, LineNameEntry, LineNameOrigin,
    NamedAxisPlacement, NamedGridAreaPlacement, NamedGridAreaResolutionReport, NamedGridError,
    NamedGridLine, NamedGridLines, NamedLineOccurrence, NamedLookupReport,
    NamedPlacementConflictResolution, NamedPlacementReport, NamedTrackComponent, PlacementSide,
    SubgridAxisPlacementReport, SubgridLineNameInheritanceReport, SubgridNameComponent,
    SubgridNameExpansionReport, SubgridNameRepeatCount, TemplateAreas, area_generated_facts,
    area_generated_lines, expand_axis_shorthand, expand_grid_area_shorthand,
    expand_named_fixed_repeat, expand_subgrid_name_list, inherit_named_subgrid_lines,
    resolve_anonymous_span_from_end, resolve_anonymous_span_from_start, resolve_named_area,
    resolve_named_axis_placement, resolve_named_grid_area, resolve_named_grid_area_report,
    resolve_named_line, resolve_named_span_from_end, resolve_named_span_from_start,
    resolve_named_subgrid_axis_placement, resolve_numeric_line,
};
#[allow(unused_imports)]
pub use placement::{
    AutoPlacer, AxisPlacement, Flow, GridArea, GridAxis, ItemPlacement, LinePlacement,
    PlacementCursor, PlacementError, PlacementReport, ResolvedItemPlacement,
};
#[allow(unused_imports)]
pub use scenario::{
    BaselineAlignedItemRectInput, GridItemRect, GridScenarioReport, LaneItemRectInput,
    SubgridItemRectInput, SubgridItemRectReport, compose_baseline_aligned_item_rect,
    compose_grid_scenario, compose_lane_item_rect, compose_subgrid_item_rect,
};
#[allow(unused_imports)]
pub use subgrid::{
    AxisEdges, OracleGap, OracleGapReport, OracleGridError, SubgridAxisKind, SubgridChild,
    SubgridEligibilityInput, SubgridEligibilityReport, SubgridIneligibleReason, SubgridLeaf,
    SubgridLeafContribution, SubgridNode, SubgridTrackInheritanceInput,
    SubgridTrackInheritanceReport, SubgridTraversalInput, SubgridTraversalReport, TrackSpan,
    inherit_subgrid_tracks, subgrid_eligibility, traverse_subgrid_intrinsic,
};
#[allow(unused_imports)]
pub use tracks::{
    DefiniteTracks, EqualShareIntrinsicTracks, GridTrack, GrowthLimit, SolvedTrack, SolvedTracks,
    Track, TrackArea, TrackMax, TrackMin, TrackSize, TrackSizingError, TrackSizingReport,
    TrackSizingSlice, TrackState,
};
