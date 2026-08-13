use super::{
    ComputedOverflow, DefaultScalar, Direction, Edges, FlowAxes, LayoutScalar, LengthPercentageOf,
    LogicalAxis, NumericResolutionOf, Overflow, OverflowClipBox, PercentageBasisOf, PhysicalAxis,
    PhysicalSide, Point, ScrollbarGutter, ScrollbarWidthOf, Size,
    scalar::{canonical_zero, round_layout_coordinate},
};
use crate::geometry::LogicalEdgesOf;

mod box_geometry;
mod construction;
mod contribution;
mod model;
pub(crate) mod rounding;

#[cfg(test)]
pub(crate) use box_geometry::UsedOverflowGutter;
use box_geometry::{AutoScrollbarOverflowObservation, scroll_rect_axis_interval, used_overflow_at};
pub(crate) use box_geometry::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, ClipMarginSourceOf,
    MeasuredLeafContentBoxInsetSourceOf, OptimalRegionInsetOf, OptimalRegionInsetsOf,
    ScrollbarReservationOf, SettledAutoScrollbarState, UsedOverflow, UsedOverflowAxis,
    content_box_inset_with_scrollbar, measured_leaf_content_box_inset,
    scrollbar_size_from_overflow,
};

use contribution::{FinalInFlowEndOf, PhysicalContributionBoundsOf, PhysicalFinalInFlowEndsOf};
pub(crate) use contribution::{
    OptionalPhysicalContributionIntervalsOf, PhysicalContributionIntervalOf,
    ScrollContributionAccumulatorOf, ScrollContributionErrorOf, ScrollOriginAxes,
    ScrollOriginProgression,
};

use construction::CanonicalScrollRectFact;
pub(crate) use construction::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollGeometryErrorOf,
    CanonicalScrollGeometrySourceOf, CanonicalScrollRangeSeedPolicy,
    CanonicalScrollSourceBuilderOf, MeasuredLeafScrollGeometrySourceOf,
    canonical_measured_leaf_scroll_geometry, canonical_scroll_box_from_source,
    canonical_scroll_geometry_from_source, rebuild_canonical_scroll_geometry_for_border_box,
    settled_auto_scrollbars_change_available_geometry,
};

use model::validate_physical_scroll_range;
pub use model::{
    FlowRelativeScrollAxisRange, FlowRelativeScrollAxisRangeOf, FlowRelativeScrollOffset,
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRange, FlowRelativeScrollRangeOf, OverflowClip,
    OverflowClipOf, PhysicalClipAxis, PhysicalClipAxisOf, PhysicalScrollAxisRange,
    PhysicalScrollAxisRangeOf, PhysicalScrollOffset, PhysicalScrollOffsetOf, PhysicalScrollRange,
    PhysicalScrollRangeOf, ScrollCoordinateError, ScrollCoordinateErrorOf, ScrollGeometry,
    ScrollGeometryOf, ScrollRect, ScrollRectError, ScrollRectErrorOf, ScrollRectOf,
    ScrollTargetGeometry, ScrollTargetGeometryOf, ScrollbarGutterRects, ScrollbarGutterRectsOf,
};
