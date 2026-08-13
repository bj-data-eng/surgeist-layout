use super::*;
use crate::scroll::{
    CanonicalScrollGeometryErrorOf, ClipMarginSourceOf, MeasuredLeafScrollGeometrySourceOf,
    OptimalRegionInsetsOf, OptionalPhysicalContributionIntervalsOf,
    ScrollContributionAccumulatorOf, UsedOverflow, canonical_measured_leaf_scroll_geometry,
    rebuild_canonical_scroll_geometry_for_border_box,
};

#[derive(Clone, Copy, Debug)]
pub(in crate::grid) struct GridChildContribution<S: LayoutScalar = Scalar> {
    pub(in crate::grid) source_index: crate::SourceIndex,
    pub(in crate::grid) location: Point<S>,
    pub(in crate::grid) margin: Edges<S>,
    pub(in crate::grid) geometry: crate::ScrollGeometryOf<S>,
    pub(in crate::grid) descendants: OptionalPhysicalContributionIntervalsOf<S>,
    pub(in crate::grid) overflow: UsedOverflow,
    pub(in crate::grid) in_flow: bool,
}

pub(in crate::grid) fn empty_grid_contributions<S: LayoutScalar>()
-> ScrollContributionAccumulatorOf<S> {
    ScrollContributionAccumulatorOf::new(
        crate::ScrollRectOf::try_new(Point::ZERO, Size::ZERO)
            .expect("zero grid contribution seed is valid"),
    )
}

pub(in crate::grid) fn retained_grid_child_scroll_geometry<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<crate::ScrollGeometryOf<S>>,
) -> Result<crate::ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    if let Some(geometry) = child_compute_geometry {
        if geometry.border_box().origin() == Point::ZERO && geometry.border_box().size() == size {
            return Ok(geometry);
        }
        return rebuild_canonical_scroll_geometry_for_border_box(geometry, size, border, padding);
    }

    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
        flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: size,
        border,
        padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
        measured_content_size: content_size,
        scroll_snap_type: style.scroll_snap_type,
        target_scroll_margin: style.scroll_margin,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
}
pub(in crate::grid) fn grid_scroll_contributions<S: LayoutScalar>(
    mut children: Vec<GridChildContribution<S>>,
    flow_axes: FlowAxes,
    padding: Edges<S>,
) -> Result<ScrollContributionAccumulatorOf<S>, crate::scroll::ScrollContributionErrorOf<S>> {
    children.sort_by_key(|child| child.source_index);
    let mut contributions = empty_grid_contributions();
    let mut inline_end = None;
    let mut block_end = None;

    for child in children {
        if child.in_flow {
            contributions.include_in_flow_child(
                child.location,
                child.geometry.border_box(),
                child.margin,
                child.descendants,
                child.overflow,
            )?;
            let border_size = child.geometry.border_box().size();
            if border_size.width > S::ZERO && border_size.height > S::ZERO {
                include_farthest_grid_flow_end(
                    &mut inline_end,
                    flow_axes.inline_end(),
                    grid_child_flow_end(child, flow_axes.inline_end()),
                );
                include_farthest_grid_flow_end(
                    &mut block_end,
                    flow_axes.block_end(),
                    grid_child_flow_end(child, flow_axes.block_end()),
                );
            }
        } else {
            contributions.include_current_out_of_flow(
                child.location,
                child.geometry.border_box(),
                child.margin,
                child.descendants,
                child.overflow,
            )?;
        }
    }

    for (axis, coordinate) in [
        (LogicalAxis::Inline, inline_end),
        (LogicalAxis::Block, block_end),
    ] {
        if let Some(coordinate) = coordinate {
            contributions.record_final_in_flow_end(flow_axes, axis, coordinate)?;
        }
    }
    contributions.include_terminal_padding(padding)?;
    Ok(contributions)
}

fn grid_child_flow_end<S: LayoutScalar>(
    child: GridChildContribution<S>,
    side: crate::PhysicalSide,
) -> S {
    let border_box = child.geometry.border_box();
    let origin = border_box.origin();
    let size = border_box.size();
    match side {
        crate::PhysicalSide::Top => child.location.y + origin.y - child.margin.top.max(S::ZERO),
        crate::PhysicalSide::Right => {
            child.location.x + origin.x + size.width + child.margin.right.max(S::ZERO)
        }
        crate::PhysicalSide::Bottom => {
            child.location.y + origin.y + size.height + child.margin.bottom.max(S::ZERO)
        }
        crate::PhysicalSide::Left => child.location.x + origin.x - child.margin.left.max(S::ZERO),
    }
}

fn include_farthest_grid_flow_end<S: LayoutScalar>(
    end: &mut Option<S>,
    side: crate::PhysicalSide,
    candidate: S,
) {
    *end = Some(end.map_or(candidate, |current| match side {
        crate::PhysicalSide::Top | crate::PhysicalSide::Left => current.min(candidate),
        crate::PhysicalSide::Right | crate::PhysicalSide::Bottom => current.max(candidate),
    }));
}

pub(in crate::grid) fn max_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}
