use super::{
    box_geometry::{
        AutoScrollbarOverflowObservation, CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf,
        ClipMarginSourceOf, OptimalRegionInsetsOf, ScrollBoxClipGutterErrorOf,
        ScrollBoxClipGutterSourceOf, SettledAutoScrollbarState, UsedOverflow,
        derive_scroll_box_clip_gutter,
    },
    contribution::{
        ScrollContributionAccumulatorOf, ScrollContributionErrorOf, ScrollOriginAxes,
        ScrollOriginProgression, derive_origin_aware_scroll_range,
    },
    model::{
        ScrollCoordinateErrorOf, ScrollGeometryOf, ScrollRectErrorOf, ScrollRectOf,
        ScrollTargetGeometryOf,
    },
};
use crate::{
    ComputedOverflow, DefaultScalar, Edges, FlowAxes, LayoutScalar, LogicalAxis, NodeInputOf,
    PhysicalAxis, PhysicalSide, Point, ScrollMarginOf, ScrollSnapAlign, ScrollSnapStop,
    ScrollSnapType, ScrollbarGutter, ScrollbarWidthOf, Size,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalScrollGeometrySourceOf<S: LayoutScalar> {
    pub(crate) flow_axes: FlowAxes,
    pub(crate) computed_overflow: ComputedOverflow,
    pub(crate) item_is_replaced: bool,
    pub(crate) border_box_size: Size<S>,
    pub(crate) border: Edges<S>,
    pub(crate) padding: Edges<S>,
    pub(crate) scrollbar_gutter: ScrollbarGutter,
    pub(crate) scrollbar_width: ScrollbarWidthOf<S>,
    pub(crate) settled_auto_scrollbars: SettledAutoScrollbarState,
    pub(crate) clip_margin: ClipMarginSourceOf<S>,
    pub(crate) scroll_padding: OptimalRegionInsetsOf<S>,
    pub(crate) contributions: ScrollContributionAccumulatorOf<S>,
    pub(crate) origin_axes: ScrollOriginAxes,
    pub(crate) scroll_snap_type: ScrollSnapType,
    pub(crate) target_border_box: ScrollRectOf<S>,
    pub(crate) target_scroll_margin: ScrollMarginOf<S>,
    pub(crate) target_flow_axes: FlowAxes,
    pub(crate) target_snap_align: ScrollSnapAlign,
    pub(crate) target_snap_stop: ScrollSnapStop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalScrollRangeSeedPolicy {
    IncludeReservedGutter,
    ExcludeReservedGutter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CanonicalRetainedScrollSourceOf<'a, S: LayoutScalar> {
    Existing(&'a ScrollGeometryOf<S>),
    Reconstruct { content_size: Size<S> },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalScrollSourceBuilderOf<S: LayoutScalar> {
    flow_axes: FlowAxes,
    computed_overflow: ComputedOverflow,
    item_is_replaced: bool,
    border_box_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    scrollbar_gutter: ScrollbarGutter,
    scrollbar_width: ScrollbarWidthOf<S>,
    settled_auto_scrollbars: SettledAutoScrollbarState,
    clip_margin: ClipMarginSourceOf<S>,
    scroll_padding: OptimalRegionInsetsOf<S>,
    origin_axes: ScrollOriginAxes,
    scroll_snap_type: ScrollSnapType,
    target_scroll_margin: ScrollMarginOf<S>,
    target_snap_align: ScrollSnapAlign,
    target_snap_stop: ScrollSnapStop,
}

impl<S: LayoutScalar> CanonicalScrollSourceBuilderOf<S> {
    #[must_use]
    pub(crate) fn for_node(
        style: &NodeInputOf<S>,
        flow_axes: FlowAxes,
        border_box_size: Size<S>,
        border: Edges<S>,
        padding: Edges<S>,
        settled_auto_scrollbars: SettledAutoScrollbarState,
        origin_axes: ScrollOriginAxes,
    ) -> Self {
        Self {
            flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size,
            border,
            padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars,
            clip_margin: ClipMarginSourceOf::new(
                style.overflow_clip_margin.clip_box(),
                style.overflow_clip_margin.margin(),
            ),
            scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
            origin_axes,
            scroll_snap_type: style.scroll_snap_type,
            target_scroll_margin: style.scroll_margin,
            target_snap_align: style.scroll_snap_align,
            target_snap_stop: style.scroll_snap_stop,
        }
    }

    pub(crate) fn geometry_from_contributions(
        self,
        contributions: ScrollContributionAccumulatorOf<S>,
        target_border_box: ScrollRectOf<S>,
    ) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
        canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
            flow_axes: self.flow_axes,
            computed_overflow: self.computed_overflow,
            item_is_replaced: self.item_is_replaced,
            border_box_size: self.border_box_size,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: self.scrollbar_gutter,
            scrollbar_width: self.scrollbar_width,
            settled_auto_scrollbars: self.settled_auto_scrollbars,
            clip_margin: self.clip_margin,
            scroll_padding: self.scroll_padding,
            contributions,
            origin_axes: self.origin_axes,
            scroll_snap_type: self.scroll_snap_type,
            target_border_box,
            target_scroll_margin: self.target_scroll_margin,
            target_flow_axes: self.flow_axes,
            target_snap_align: self.target_snap_align,
            target_snap_stop: self.target_snap_stop,
        })
    }

    pub(crate) fn geometry_from_retained_source(
        self,
        source: CanonicalRetainedScrollSourceOf<'_, S>,
        range_seed_policy: CanonicalScrollRangeSeedPolicy,
    ) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
        let content_size = match source {
            CanonicalRetainedScrollSourceOf::Existing(geometry) => {
                if geometry.border_box().origin() == Point::ZERO
                    && geometry.border_box().size() == self.border_box_size
                {
                    return Ok(*geometry);
                }
                return rebuild_canonical_scroll_geometry_for_border_box(
                    *geometry,
                    self.border_box_size,
                    self.border,
                    self.padding,
                );
            }
            CanonicalRetainedScrollSourceOf::Reconstruct { content_size } => content_size,
        };

        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: self.flow_axes,
            computed_overflow: self.computed_overflow,
            item_is_replaced: self.item_is_replaced,
            border_box_size: self.border_box_size,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: self.scrollbar_gutter,
            scrollbar_width: self.scrollbar_width,
            settled_auto_scrollbars: self.settled_auto_scrollbars,
        })?;
        let content_box = scroll_box.content_box();
        let direct_content = ScrollRectOf::try_new(
            content_box.origin(),
            Size::new(
                content_box.size().width.max(content_size.width),
                content_box.size().height.max(content_size.height),
            ),
        )
        .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
        let mut contributions = ScrollContributionAccumulatorOf::new(scroll_box.padding_box());
        match range_seed_policy {
            CanonicalScrollRangeSeedPolicy::IncludeReservedGutter => {}
            CanonicalScrollRangeSeedPolicy::ExcludeReservedGutter => {
                contributions.exclude_reserved_gutter_from_range();
            }
        }
        contributions.include_direct_line(direct_content);
        self.geometry_from_contributions(contributions, scroll_box.border_box())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredLeafScrollGeometrySourceOf<S: LayoutScalar> {
    pub(crate) flow_axes: FlowAxes,
    pub(crate) computed_overflow: ComputedOverflow,
    pub(crate) item_is_replaced: bool,
    pub(crate) border_box_size: Size<S>,
    pub(crate) border: Edges<S>,
    pub(crate) padding: Edges<S>,
    pub(crate) scrollbar_gutter: ScrollbarGutter,
    pub(crate) scrollbar_width: ScrollbarWidthOf<S>,
    pub(crate) settled_auto_scrollbars: SettledAutoScrollbarState,
    pub(crate) clip_margin: ClipMarginSourceOf<S>,
    pub(crate) scroll_padding: OptimalRegionInsetsOf<S>,
    pub(crate) measured_content_size: Size<S>,
    pub(crate) scroll_snap_type: ScrollSnapType,
    pub(crate) target_scroll_margin: ScrollMarginOf<S>,
    pub(crate) target_snap_align: ScrollSnapAlign,
    pub(crate) target_snap_stop: ScrollSnapStop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalScrollRectFact {
    BorderBox,
    TargetBorderBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CanonicalScrollGeometryErrorOf<S: LayoutScalar> {
    BoxClipGutter(ScrollBoxClipGutterErrorOf<S>),
    Contribution(ScrollContributionErrorOf<S>),
    ScrollableOverflow(ScrollRectErrorOf<S>),
    Range(ScrollCoordinateErrorOf<S>),
    RoundedRect {
        fact: CanonicalScrollRectFact,
        source: ScrollRectErrorOf<S>,
    },
    RoundedContribution(ScrollCoordinateErrorOf<S>),
    RoundedFinalInFlowEnd {
        side: PhysicalSide,
        value: S,
    },
    RoundedScrollbarWidth {
        value: S,
    },
    RoundedOptimalRegionInset {
        side: PhysicalSide,
        value: S,
    },
}

pub(crate) fn canonical_scroll_box_from_source<S: LayoutScalar>(
    source: CanonicalScrollBoxSourceOf<S>,
) -> Result<CanonicalScrollBoxOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let boxes = derive_scroll_box_clip_gutter(ScrollBoxClipGutterSourceOf {
        flow_axes: source.flow_axes,
        used_overflow,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::default(),
        optimal_region_insets: OptimalRegionInsetsOf::default(),
    })
    .map_err(CanonicalScrollGeometryErrorOf::BoxClipGutter)?;

    Ok(CanonicalScrollBoxOf {
        border_box: boxes.border_box,
        padding_box: boxes.padding_box,
        effective_border: boxes.effective_border,
        effective_padding: boxes.effective_padding,
        effective_gutter: boxes.effective_reservation,
        scrollport: boxes.scrollport,
        content_box: boxes.content_box,
    })
}

pub(crate) fn settled_auto_scrollbars_change_available_geometry<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    next_state: SettledAutoScrollbarState,
) -> Result<bool, CanonicalScrollGeometryErrorOf<S>> {
    let source = geometry.source;
    let box_source = |settled_auto_scrollbars| CanonicalScrollBoxSourceOf {
        flow_axes: source.flow_axes,
        computed_overflow: source.computed_overflow,
        item_is_replaced: source.item_is_replaced,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars,
    };
    let current = canonical_scroll_box_from_source(box_source(source.settled_auto_scrollbars))?;
    let prospective = canonical_scroll_box_from_source(box_source(next_state))?;

    Ok(current.effective_gutter() != prospective.effective_gutter()
        || current.content_box() != prospective.content_box())
}

pub(crate) fn rebuild_canonical_scroll_geometry_for_border_box<S: LayoutScalar>(
    geometry: ScrollGeometryOf<S>,
    border_box_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let mut source = geometry.source;
    let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
        flow_axes: source.flow_axes,
        computed_overflow: source.computed_overflow,
        item_is_replaced: source.item_is_replaced,
        border_box_size,
        border,
        padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
    })?;
    source.border_box_size = border_box_size;
    source.border = border;
    source.padding = padding;
    source
        .contributions
        .replace_container_seed(scroll_box.padding_box());
    source
        .contributions
        .include_terminal_padding(padding)
        .map_err(CanonicalScrollGeometryErrorOf::Contribution)?;
    source.target_border_box = scroll_box.border_box();
    canonical_scroll_geometry_from_source(source)
}

pub(crate) fn canonical_scroll_geometry_from_source<S: LayoutScalar>(
    source: CanonicalScrollGeometrySourceOf<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let boxes = derive_scroll_box_clip_gutter(ScrollBoxClipGutterSourceOf {
        flow_axes: source.flow_axes,
        used_overflow,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: source.clip_margin,
        optimal_region_insets: source.scroll_padding,
    })
    .map_err(CanonicalScrollGeometryErrorOf::BoxClipGutter)?;

    let complete = source.contributions.complete_overflow();
    let scrollable_overflow = ScrollRectOf::try_new(
        Point::new(complete.x().minimum(), complete.y().minimum()),
        Size::new(
            complete.x().maximum() - complete.x().minimum(),
            complete.y().maximum() - complete.y().minimum(),
        ),
    )
    .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
    let physical_range = derive_origin_aware_scroll_range(
        source.flow_axes,
        source.origin_axes,
        used_overflow,
        boxes.scrollport,
        &source.contributions,
    )
    .map_err(CanonicalScrollGeometryErrorOf::Range)?;
    let mut auto_contributions = source.contributions;
    auto_contributions.replace_container_seed(boxes.scrollport);
    let auto_range = derive_origin_aware_scroll_range(
        source.flow_axes,
        source.origin_axes,
        used_overflow,
        boxes.scrollport,
        &auto_contributions,
    )
    .map_err(CanonicalScrollGeometryErrorOf::Range)?;
    let auto_scrollbar_observation = AutoScrollbarOverflowObservation::from_range(auto_range);
    let target = ScrollTargetGeometryOf {
        border_box: source.target_border_box,
        scroll_margin: source.target_scroll_margin,
        flow_axes: source.target_flow_axes,
        snap_align: source.target_snap_align,
        snap_stop: source.target_snap_stop,
    };

    Ok(ScrollGeometryOf {
        source,
        flow_axes: source.flow_axes,
        used_overflow,
        border_box: boxes.border_box,
        padding_box: boxes.padding_box,
        content_box: boxes.content_box,
        scrollport: boxes.scrollport,
        overflow_clip: boxes.overflow_clip,
        scrollable_overflow,
        physical_range,
        auto_scrollbar_observation,
        gutters: boxes.gutters,
        aggregate_reservation: boxes.aggregate_reservation,
        resolved_scroll_padding: boxes.resolved_scroll_padding,
        optimal_viewing_region: boxes.optimal_viewing_region,
        scroll_snap_type: source.scroll_snap_type,
        target,
    })
}

pub(crate) fn canonical_measured_leaf_scroll_geometry<S: LayoutScalar>(
    source: MeasuredLeafScrollGeometrySourceOf<S>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let used_overflow =
        UsedOverflow::from_computed(source.computed_overflow, source.item_is_replaced);
    let boxes = derive_scroll_box_clip_gutter(ScrollBoxClipGutterSourceOf {
        flow_axes: source.flow_axes,
        used_overflow,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: source.clip_margin,
        optimal_region_insets: source.scroll_padding,
    })
    .map_err(CanonicalScrollGeometryErrorOf::BoxClipGutter)?;
    let measured_content = measured_leaf_content_rect(
        source.flow_axes,
        boxes.content_box,
        source.measured_content_size,
    )
    .map_err(CanonicalScrollGeometryErrorOf::ScrollableOverflow)?;
    let mut contributions = ScrollContributionAccumulatorOf::new(boxes.padding_box);
    contributions.exclude_reserved_gutter_from_scroll_container_axes(used_overflow);
    contributions.include_direct_line(measured_content);
    for axis in [LogicalAxis::Inline, LogicalAxis::Block] {
        let side = match axis {
            LogicalAxis::Inline => source.flow_axes.inline_end(),
            LogicalAxis::Block => source.flow_axes.block_end(),
        };
        let coordinate = match side {
            PhysicalSide::Top => measured_content.origin().y,
            PhysicalSide::Right => measured_content.origin().x + measured_content.size().width,
            PhysicalSide::Bottom => measured_content.origin().y + measured_content.size().height,
            PhysicalSide::Left => measured_content.origin().x,
        };
        contributions
            .record_final_in_flow_end(source.flow_axes, axis, coordinate)
            .map_err(CanonicalScrollGeometryErrorOf::Contribution)?;
    }
    contributions
        .include_terminal_padding(source.padding)
        .map_err(CanonicalScrollGeometryErrorOf::Contribution)?;

    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: source.flow_axes,
        computed_overflow: source.computed_overflow,
        item_is_replaced: source.item_is_replaced,
        border_box_size: source.border_box_size,
        border: source.border,
        padding: source.padding,
        scrollbar_gutter: source.scrollbar_gutter,
        scrollbar_width: source.scrollbar_width,
        settled_auto_scrollbars: source.settled_auto_scrollbars,
        clip_margin: source.clip_margin,
        scroll_padding: source.scroll_padding,
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: source.scroll_snap_type,
        target_border_box: boxes.border_box,
        target_scroll_margin: source.target_scroll_margin,
        target_flow_axes: source.flow_axes,
        target_snap_align: source.target_snap_align,
        target_snap_stop: source.target_snap_stop,
    })
}

fn measured_leaf_content_rect<S: LayoutScalar>(
    flow_axes: FlowAxes,
    content_box: ScrollRectOf<S>,
    measured_content_size: Size<S>,
) -> Result<ScrollRectOf<S>, ScrollRectErrorOf<S>> {
    let origin = content_box.origin();
    let size = content_box.size();
    let x_start = if flow_axes.inline_axis() == PhysicalAxis::Horizontal {
        flow_axes.inline_start()
    } else {
        flow_axes.block_start()
    };
    let y_start = if flow_axes.inline_axis() == PhysicalAxis::Vertical {
        flow_axes.inline_start()
    } else {
        flow_axes.block_start()
    };
    ScrollRectOf::try_new(
        Point::new(
            match x_start {
                PhysicalSide::Left => origin.x,
                PhysicalSide::Right => origin.x + size.width - measured_content_size.width,
                PhysicalSide::Top | PhysicalSide::Bottom => {
                    unreachable!("physical x start side must be left or right")
                }
            },
            match y_start {
                PhysicalSide::Top => origin.y,
                PhysicalSide::Bottom => origin.y + size.height - measured_content_size.height,
                PhysicalSide::Right | PhysicalSide::Left => {
                    unreachable!("physical y start side must be top or bottom")
                }
            },
        ),
        measured_content_size,
    )
}
type DefaultCanonicalScrollGeometryFactory =
    fn(
        CanonicalScrollGeometrySourceOf<DefaultScalar>,
    )
        -> Result<ScrollGeometryOf<DefaultScalar>, CanonicalScrollGeometryErrorOf<DefaultScalar>>;
const _: DefaultCanonicalScrollGeometryFactory =
    canonical_scroll_geometry_from_source::<DefaultScalar>;

#[cfg(test)]
pub(super) mod fri05_c02_factory_tests {
    use super::*;
    use crate::scroll::{
        OptimalRegionInsetOf, PhysicalContributionIntervalOf,
        box_geometry::ScrollBoxClipGutterErrorOf, contribution::derive_origin_aware_scroll_range,
    };
    use crate::{
        Direction, LengthPercentageOf, Overflow, OverflowClipBox, ScrollMarginOf, ScrollSnapAlign,
        ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType,
        WritingMode,
    };

    pub(in crate::scroll) const FLOW_MAPPINGS: [(WritingMode, Direction); 10] = [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::HorizontalTb, Direction::Rtl),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Rtl),
        (WritingMode::VerticalLr, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
        (WritingMode::SidewaysLr, Direction::Rtl),
    ];

    pub(in crate::scroll) fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    pub(in crate::scroll) fn rect<S: LayoutScalar>(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> ScrollRectOf<S> {
        ScrollRectOf::try_new(
            Point::new(scalar(x), scalar(y)),
            Size::new(scalar(width), scalar(height)),
        )
        .unwrap()
    }

    pub(in crate::scroll) fn px<S: LayoutScalar>(value: f64) -> OptimalRegionInsetOf<S> {
        OptimalRegionInsetOf::Value(LengthPercentageOf::px(scalar(value)).unwrap())
    }

    pub(in crate::scroll) fn percent<S: LayoutScalar>(value: f64) -> OptimalRegionInsetOf<S> {
        OptimalRegionInsetOf::Value(
            LengthPercentageOf::from_percent_fraction(scalar(value)).unwrap(),
        )
    }

    pub(in crate::scroll) fn factory_source<S: LayoutScalar>(
        flow_axes: FlowAxes,
    ) -> CanonicalScrollGeometrySourceOf<S> {
        let padding_box = rect(4.0, 1.0, 34.0, 26.0);
        let mut contributions = ScrollContributionAccumulatorOf::new(padding_box);
        contributions.include_direct_line(rect(-5.0, -7.0, 60.0, 50.0));
        contributions
            .record_final_in_flow_end(flow_axes, LogicalAxis::Inline, scalar(31.0))
            .unwrap();
        contributions
            .record_final_in_flow_end(flow_axes, LogicalAxis::Block, scalar(19.0))
            .unwrap();
        contributions
            .include_terminal_padding(Edges::new(
                scalar(2.0),
                scalar(3.0),
                scalar(4.0),
                scalar(5.0),
            ))
            .unwrap();
        contributions
            .set_active_alignment_subject(PhysicalAxis::Horizontal, rect(-2.0, 0.0, 10.0, 10.0));
        contributions
            .set_active_alignment_subject(PhysicalAxis::Vertical, rect(0.0, -4.0, 10.0, 10.0));

        CanonicalScrollGeometrySourceOf {
            flow_axes,
            computed_overflow: ComputedOverflow::try_new(Overflow::Scroll, Overflow::Auto).unwrap(),
            item_is_replaced: false,
            border_box_size: Size::new(scalar(40.0), scalar(30.0)),
            border: Edges::new(scalar(1.0), scalar(2.0), scalar(3.0), scalar(4.0)),
            padding: Edges::new(scalar(2.0), scalar(3.0), scalar(4.0), scalar(5.0)),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidthOf::try_new(scalar(6.0)).unwrap(),
            settled_auto_scrollbars: SettledAutoScrollbarState { x: true, y: true },
            clip_margin: ClipMarginSourceOf::new(OverflowClipBox::BorderBox, scalar(2.0)),
            scroll_padding: OptimalRegionInsetsOf::new(px(2.0), percent(0.25), px(30.0), px(3.0)),
            contributions,
            origin_axes: ScrollOriginAxes::new(
                ScrollOriginProgression::FlowEndward,
                ScrollOriginProgression::FlowEndward,
            ),
            scroll_snap_type: ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            },
            target_border_box: rect(-1.25, 2.5, 8.5, 7.25),
            target_scroll_margin: ScrollMarginOf::try_new(
                scalar(-1.0),
                scalar(2.0),
                scalar(3.0),
                scalar(-4.0),
            )
            .unwrap(),
            target_flow_axes: FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
            target_snap_align: ScrollSnapAlign::new(
                ScrollSnapAlignValue::End,
                ScrollSnapAlignValue::Center,
            ),
            target_snap_stop: ScrollSnapStop::Always,
        }
    }

    fn assert_finite_rect<S: LayoutScalar>(rect: ScrollRectOf<S>) {
        let origin = rect.origin();
        let size = rect.size();
        assert!(origin.x.is_finite());
        assert!(origin.y.is_finite());
        assert!(size.width.is_finite() && size.width >= S::ZERO);
        assert!(size.height.is_finite() && size.height >= S::ZERO);
        assert!((origin.x + size.width).is_finite());
        assert!((origin.y + size.height).is_finite());
    }

    fn assert_rect_contains<S: LayoutScalar>(outer: ScrollRectOf<S>, inner: ScrollRectOf<S>) {
        let outer_origin = outer.origin();
        let outer_size = outer.size();
        let inner_origin = inner.origin();
        let inner_size = inner.size();
        assert!(inner_origin.x >= outer_origin.x);
        assert!(inner_origin.y >= outer_origin.y);
        assert!(inner_origin.x + inner_size.width <= outer_origin.x + outer_size.width);
        assert!(inner_origin.y + inner_size.height <= outer_origin.y + outer_size.height);
    }

    pub(in crate::scroll) fn assert_canonical_coherence<S: LayoutScalar>(
        geometry: ScrollGeometryOf<S>,
    ) {
        for rect in [
            geometry.border_box,
            geometry.padding_box,
            geometry.content_box,
            geometry.scrollport,
            geometry.scrollable_overflow,
            geometry.optimal_viewing_region,
            geometry.target.border_box(),
        ] {
            assert_finite_rect(rect);
        }
        assert_rect_contains(geometry.border_box, geometry.padding_box);
        assert_rect_contains(geometry.padding_box, geometry.scrollport);
        assert_rect_contains(geometry.scrollport, geometry.content_box);
        assert_rect_contains(geometry.scrollport, geometry.optimal_viewing_region);

        let padding_origin = geometry.padding_box.origin();
        let padding_size = geometry.padding_box.size();
        let scrollport_origin = geometry.scrollport.origin();
        let scrollport_size = geometry.scrollport.size();
        if let Some(top) = geometry.gutters.top {
            assert_eq!(top.origin().y, padding_origin.y);
            assert_eq!(top.origin().x, scrollport_origin.x);
            assert_eq!(top.size().width, scrollport_size.width);
            assert_eq!(top.origin().y + top.size().height, scrollport_origin.y);
        }
        if let Some(right) = geometry.gutters.right {
            assert_eq!(
                right.origin().x,
                scrollport_origin.x + scrollport_size.width
            );
            assert_eq!(right.origin().y, scrollport_origin.y);
            assert_eq!(right.size().height, scrollport_size.height);
            assert_eq!(
                right.origin().x + right.size().width,
                padding_origin.x + padding_size.width
            );
        }
        if let Some(bottom) = geometry.gutters.bottom {
            assert_eq!(bottom.origin().x, scrollport_origin.x);
            assert_eq!(
                bottom.origin().y,
                scrollport_origin.y + scrollport_size.height
            );
            assert_eq!(bottom.size().width, scrollport_size.width);
            assert_eq!(
                bottom.origin().y + bottom.size().height,
                padding_origin.y + padding_size.height
            );
        }
        if let Some(left) = geometry.gutters.left {
            assert_eq!(left.origin().x, padding_origin.x);
            assert_eq!(left.origin().y, scrollport_origin.y);
            assert_eq!(left.size().height, scrollport_size.height);
            assert_eq!(left.origin().x + left.size().width, scrollport_origin.x);
        }
        assert!(geometry.aggregate_reservation.width <= padding_size.width);
        assert!(geometry.aggregate_reservation.height <= padding_size.height);

        for clip in [geometry.overflow_clip.x(), geometry.overflow_clip.y()]
            .into_iter()
            .flatten()
        {
            assert!(clip.minimum().is_finite());
            assert!(clip.maximum().is_finite());
            assert!(clip.minimum() <= clip.maximum());
        }
        for (used, clip, range) in [
            (
                geometry.used_overflow.x(),
                geometry.overflow_clip.x(),
                geometry.physical_range.x(),
            ),
            (
                geometry.used_overflow.y(),
                geometry.overflow_clip.y(),
                geometry.physical_range.y(),
            ),
        ] {
            assert!(range.minimum().is_finite());
            assert!(range.maximum().is_finite());
            assert!(range.minimum() <= S::ZERO && range.maximum() >= S::ZERO);
            match used.value() {
                Overflow::Visible => {
                    assert_eq!(clip, None);
                    assert_eq!((range.minimum(), range.maximum()), (S::ZERO, S::ZERO));
                }
                Overflow::Clip => {
                    assert!(clip.is_some());
                    assert_eq!((range.minimum(), range.maximum()), (S::ZERO, S::ZERO));
                }
                Overflow::Hidden | Overflow::Scroll | Overflow::Auto => {
                    assert!(clip.is_some());
                }
            }
        }

        assert_eq!(geometry.flow_axes, geometry.source.flow_axes);
        assert_eq!(
            geometry.used_overflow,
            UsedOverflow::from_computed(
                geometry.source.computed_overflow,
                geometry.source.item_is_replaced,
            )
        );
        assert_eq!(
            geometry.physical_range,
            derive_origin_aware_scroll_range(
                geometry.flow_axes,
                geometry.source.origin_axes,
                geometry.used_overflow,
                geometry.scrollport,
                &geometry.source.contributions,
            )
            .unwrap()
        );
        let complete = geometry.source.contributions.complete_overflow();
        assert_eq!(
            geometry.scrollable_overflow.origin().x,
            complete.x().minimum()
        );
        assert_eq!(
            geometry.scrollable_overflow.origin().y,
            complete.y().minimum()
        );
        assert_eq!(
            geometry.scrollable_overflow.origin().x + geometry.scrollable_overflow.size().width,
            complete.x().maximum()
        );
        assert_eq!(
            geometry.scrollable_overflow.origin().y + geometry.scrollable_overflow.size().height,
            complete.y().maximum()
        );
        assert_eq!(geometry.scroll_snap_type, geometry.source.scroll_snap_type);
        assert_eq!(
            geometry.target.scroll_margin(),
            geometry.source.target_scroll_margin
        );
        assert_eq!(
            geometry.target.flow_axes(),
            geometry.source.target_flow_axes
        );
        assert_eq!(
            geometry.target.snap_align(),
            geometry.source.target_snap_align
        );
        assert_eq!(
            geometry.target.snap_stop(),
            geometry.source.target_snap_stop
        );
    }

    fn assert_factory_contract<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let source = factory_source(flow_axes);
        let geometry = canonical_scroll_geometry_from_source(source).unwrap();
        assert_canonical_coherence(geometry);

        assert_eq!(geometry.border_box, rect(0.0, 0.0, 40.0, 30.0));
        assert_eq!(geometry.padding_box, rect(4.0, 1.0, 34.0, 26.0));
        assert_eq!(geometry.scrollport, rect(10.0, 1.0, 22.0, 20.0));
        assert_eq!(geometry.content_box, rect(15.0, 3.0, 14.0, 14.0));
        assert_eq!(geometry.gutters.top, None);
        assert_eq!(geometry.gutters.left, Some(rect(4.0, 1.0, 6.0, 20.0)));
        assert_eq!(geometry.gutters.right, Some(rect(32.0, 1.0, 6.0, 20.0)));
        assert_eq!(geometry.gutters.bottom, Some(rect(10.0, 21.0, 22.0, 6.0)));
        assert_eq!(
            geometry.aggregate_reservation,
            Size::new(scalar(12.0), scalar(6.0))
        );
        assert_eq!(geometry.overflow_clip.x().unwrap().minimum(), scalar(10.0));
        assert_eq!(geometry.overflow_clip.x().unwrap().maximum(), scalar(32.0));
        assert_eq!(geometry.overflow_clip.y().unwrap().minimum(), scalar(1.0));
        assert_eq!(geometry.overflow_clip.y().unwrap().maximum(), scalar(21.0));
        assert_eq!(
            geometry.resolved_scroll_padding,
            Edges::new(scalar(2.0), scalar(5.5), scalar(30.0), scalar(3.0))
        );
        assert_eq!(geometry.scrollable_overflow, rect(-5.0, -7.0, 60.0, 50.0));
        assert_eq!(
            (
                geometry.physical_range.x().minimum(),
                geometry.physical_range.x().maximum(),
                geometry.physical_range.y().minimum(),
                geometry.physical_range.y().maximum(),
            ),
            (scalar(-12.0), scalar(23.0), scalar(-5.0), scalar(22.0))
        );
        assert_eq!(geometry.target.border_box(), source.target_border_box);

        let mut partial_source = source;
        partial_source.computed_overflow =
            ComputedOverflow::try_new(Overflow::Visible, Overflow::Clip).unwrap();
        partial_source.scrollbar_gutter = ScrollbarGutter::Auto;
        partial_source.settled_auto_scrollbars = SettledAutoScrollbarState { x: false, y: false };
        let partial = canonical_scroll_geometry_from_source(partial_source).unwrap();
        assert_canonical_coherence(partial);
        assert_eq!(partial.used_overflow.x().value(), Overflow::Visible);
        assert_eq!(partial.used_overflow.y().value(), Overflow::Clip);
        assert_eq!(partial.overflow_clip.x(), None);
        assert_eq!(partial.overflow_clip.y().unwrap().minimum(), scalar(-2.0));
        assert_eq!(partial.overflow_clip.y().unwrap().maximum(), scalar(32.0));

        let mut replaced_source = source;
        replaced_source.computed_overflow =
            ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap();
        replaced_source.item_is_replaced = true;
        let replaced = canonical_scroll_geometry_from_source(replaced_source).unwrap();
        assert_canonical_coherence(replaced);
        assert_eq!(replaced.used_overflow.x().value(), Overflow::Clip);
        assert_eq!(replaced.used_overflow.y().value(), Overflow::Clip);
        assert_eq!(replaced.aggregate_reservation, Size::ZERO);
        assert_eq!(
            (
                replaced.physical_range.x().minimum(),
                replaced.physical_range.x().maximum(),
                replaced.physical_range.y().minimum(),
                replaced.physical_range.y().maximum(),
            ),
            (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
        );

        let mut saturated_source = source;
        saturated_source.computed_overflow =
            ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll).unwrap();
        saturated_source.border_box_size = Size::new(scalar(2.0), scalar(2.0));
        saturated_source.border = Edges::ZERO;
        saturated_source.padding = Edges::ZERO;
        saturated_source.scrollbar_width = ScrollbarWidthOf::try_new(scalar(15.0)).unwrap();
        saturated_source.scroll_padding = OptimalRegionInsetsOf::default();
        saturated_source.contributions =
            ScrollContributionAccumulatorOf::new(rect(0.0, 0.0, 2.0, 2.0));
        saturated_source.target_border_box = rect(0.0, 0.0, 2.0, 2.0);
        let saturated = canonical_scroll_geometry_from_source(saturated_source).unwrap();
        assert_canonical_coherence(saturated);
        assert_eq!(
            saturated.aggregate_reservation,
            Size::new(scalar(2.0), scalar(2.0))
        );
        assert_eq!(saturated.scrollport, rect(1.0, 0.0, 0.0, 0.0));
        assert_eq!(saturated.gutters.left, Some(rect(0.0, 0.0, 1.0, 0.0)));
        assert_eq!(saturated.gutters.right, Some(rect(1.0, 0.0, 1.0, 0.0)));
        assert_eq!(saturated.gutters.bottom, Some(rect(1.0, 0.0, 0.0, 2.0)));
    }

    #[test]
    fn fri05_c02_factory_composes_source_only_geometry_and_all_coherence_invariants() {
        assert_factory_contract::<f32>();
        assert_factory_contract::<f64>();
    }

    fn assert_factory_failure_contract<S>(largest: S)
    where
        S: LayoutScalar + std::panic::UnwindSafe,
    {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let mut non_finite = factory_source(flow_axes);
        non_finite.border_box_size.width = S::INFINITY;
        let outcome =
            std::panic::catch_unwind(move || canonical_scroll_geometry_from_source(non_finite));
        assert!(outcome.is_ok());
        assert!(matches!(
            outcome.unwrap(),
            Err(CanonicalScrollGeometryErrorOf::BoxClipGutter(
                ScrollBoxClipGutterErrorOf::Rect(ScrollRectErrorOf::NonFiniteSize {
                    axis: PhysicalAxis::Horizontal,
                    ..
                })
            ))
        ));

        let mut impossible = factory_source(flow_axes);
        impossible.contributions.container_seed.x = PhysicalContributionIntervalOf {
            minimum: -largest,
            maximum: largest,
        };
        let outcome =
            std::panic::catch_unwind(move || canonical_scroll_geometry_from_source(impossible));
        assert!(outcome.is_ok());
        assert!(matches!(
            outcome.unwrap(),
            Err(CanonicalScrollGeometryErrorOf::ScrollableOverflow(
                ScrollRectErrorOf::NonFiniteSize {
                    axis: PhysicalAxis::Horizontal,
                    ..
                }
            ))
        ));
    }

    #[test]
    fn fri05_c02_factory_returns_typed_finite_failures_without_panic_or_unsupported() {
        assert_factory_failure_contract::<f32>(f32::MAX);
        assert_factory_failure_contract::<f64>(f64::MAX);
    }

    #[test]
    fn fri05_c03_root_block_legacy_absence_factory_has_no_migration_or_rounding_adapter() {
        let construction = include_str!("construction.rs");
        let construction_production = construction
            .split("#[cfg(test)]\npub(super) mod fri05_c02_factory_tests")
            .next()
            .unwrap();
        let facade = include_str!("../scroll.rs");
        let facade_production = facade
            .split("#[cfg(test)]\nmod fri05_c02_factory_rounding_tests")
            .next()
            .unwrap();
        assert_eq!(
            construction_production
                .matches("fn canonical_scroll_geometry_from_source<")
                .count(),
            1
        );
        assert_eq!(
            construction_production
                .matches("canonical_scroll_geometry_from_source(")
                .count(),
            3,
            "the canonical source builder, measured leaf, and retained-source rebuild are the construction-owner callers"
        );
        assert_eq!(
            facade_production
                .matches("canonical_scroll_geometry_from_source(")
                .count(),
            1,
            "canonical rounding reconstructs only through the construction owner"
        );
        assert_eq!(
            facade_production
                .matches("fn rebuild_rounded_canonical_scroll_geometry<")
                .count(),
            1
        );
        assert_eq!(
            facade_production
                .matches("rebuild_rounded_canonical_scroll_geometry(")
                .count(),
            0,
            "callers invoke canonical rounding directly without a compatibility wrapper"
        );
        for forbidden in [
            "pub struct CanonicalScrollGeometrySourceOf",
            "pub fn canonical_scroll_geometry_from_source",
            "impl<S: LayoutScalar> CanonicalScrollGeometrySourceOf",
            "impl<S: LayoutScalar> Default for CanonicalScrollGeometrySourceOf",
        ] {
            assert!(
                !construction_production.contains(forbidden),
                "unexpected surface: {forbidden}"
            );
        }
        let production = format!("{construction_production}\n{facade_production}");
        for removed in [
            "ScrollUnsupportedFeature",
            "scroll_geometry_from_layout",
            "round_scroll_geometry",
            "ScrollGeometryOf::new",
            "MeasuredLeafProvenance",
        ] {
            assert!(!production.contains(removed), "retained adapter: {removed}");
        }
        let public_front_door = include_str!("../lib.rs");
        assert!(!public_front_door.contains("CanonicalScrollGeometry"));
    }
}
