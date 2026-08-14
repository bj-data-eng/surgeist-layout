use super::alignment::line_free_space;
use super::items::{FinalFlexItem, ResolvedFlexItem};
use super::lines::FlexLine;
use super::{AlignContent, Constants};
use crate::error::layout_own_geometry_error;
use crate::geometry::{LogicalAxis, PhysicalAxis, PhysicalSide};
use crate::layout_math::OptionalSizeExt;
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf,
    CanonicalScrollGeometryErrorOf, CanonicalScrollRangeSeedPolicy, CanonicalScrollSourceBuilderOf,
    ScrollBoxProjection, ScrollContributionAccumulatorOf, ScrollOriginAxes,
    ScrollOriginProgression, ScrollTargetProjection, canonical_scroll_box_from_source,
};
use crate::{
    ComputeOutputOf, Edges, LayoutResultOf, LayoutScalar, Point, RunMode, ScrollGeometryOf,
    ScrollRectOf, Size, Traverse,
};

pub(super) fn retain_flex_scroll_geometry<S: LayoutScalar>(
    output: ComputeOutputOf<S>,
    scroll_geometry: ScrollGeometryOf<S>,
) -> ComputeOutputOf<S> {
    ComputeOutputOf {
        scroll_geometry: Some(scroll_geometry),
        ..output
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FlexChildContribution<S: LayoutScalar> {
    pub(super) source_index: crate::SourceIndex,
    pub(super) location: Point<S>,
    pub(super) margin: Edges<S>,
    pub(super) geometry: ScrollGeometryOf<S>,
    pub(super) in_flow: bool,
}

pub(super) type FlexChildContributionsResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    Vec<FlexChildContribution<<Tree as Traverse>::Scalar>>,
    <Tree as Traverse>::Scalar,
    M,
>;

pub(super) fn flex_container_scroll_box<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    projection: ScrollBoxProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
) -> LayoutResultOf<Node, CanonicalScrollBoxOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf::from_projection(
        projection,
        output_size,
        constants.border,
        constants.padding,
        constants.settled_auto_scrollbars,
    ))
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

pub(super) fn flex_scroll_contributions<Node, S: LayoutScalar>(
    final_items: &[FinalFlexItem<Node, S>],
    absolute_contributions: &[FlexChildContribution<S>],
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
) -> Result<ScrollContributionAccumulatorOf<S>, crate::scroll::ScrollContributionErrorOf<S>> {
    let mut children = final_items
        .iter()
        .map(|item| FlexChildContribution {
            source_index: item.source_index,
            location: item.location,
            margin: item.margin,
            geometry: item
                .output
                .scroll_geometry
                .expect("final flex items retain canonical geometry"),
            in_flow: true,
        })
        .chain(absolute_contributions.iter().copied())
        .collect::<Vec<_>>();
    children.sort_by_key(|child| child.source_index);

    let mut contributions = ScrollContributionAccumulatorOf::new(scroll_box.padding_box());
    let mut inline_end = None;
    let mut block_end = None;
    for child in children {
        if child.in_flow {
            contributions.include_in_flow_geometry(child.location, child.margin, child.geometry)?;
            let border_size = child.geometry.border_box().size();
            if border_size.width > S::ZERO && border_size.height > S::ZERO {
                include_farthest_flow_end(
                    &mut inline_end,
                    constants.flow_axes.inline_end(),
                    child_flow_end(child, constants.flow_axes.inline_end()),
                );
                include_farthest_flow_end(
                    &mut block_end,
                    constants.flow_axes.block_end(),
                    child_flow_end(child, constants.flow_axes.block_end()),
                );
            }
        } else {
            contributions.include_current_out_of_flow_geometry(
                child.location,
                child.margin,
                child.geometry,
            )?;
        }
    }

    for (axis, coordinate) in [
        (LogicalAxis::Inline, inline_end),
        (LogicalAxis::Block, block_end),
    ] {
        if let Some(coordinate) = coordinate {
            contributions.record_final_in_flow_end(constants.flow_axes, axis, coordinate)?;
        }
    }
    include_flex_alignment_subjects(
        &mut contributions,
        final_items,
        resolved_items,
        lines,
        constants,
    )?;
    contributions.include_terminal_padding(constants.padding)?;
    Ok(contributions)
}

fn include_flex_alignment_subjects<Node, S: LayoutScalar>(
    contributions: &mut ScrollContributionAccumulatorOf<S>,
    final_items: &[FinalFlexItem<Node, S>],
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Result<(), crate::scroll::ScrollContributionErrorOf<S>> {
    if let Some(authored) = constants.authored_justify_content
        && lines.iter().any(|line| {
            let free_space = line_free_space(&resolved_items[line.start..line.end], constants);
            !safe_alignment_lands_at_origin_start(authored, free_space)
        })
        && let Some((minimum, maximum)) = final_item_subject_interval(final_items, constants)
    {
        set_alignment_subject_interval(
            contributions,
            constants.axes.main_physical_axis(),
            minimum,
            maximum,
        )?;
    }

    if let Some(authored) = constants.authored_align_content
        && lines.len() > 1
        && let Some(free_space) = line_cross_free_space(lines, constants)
        && !safe_alignment_lands_at_origin_start(authored, free_space)
        && let Some((minimum, maximum)) = line_subject_interval(lines, constants)
    {
        set_alignment_subject_interval(
            contributions,
            constants.axes.cross_physical_axis(),
            minimum,
            maximum,
        )?;
    }
    Ok(())
}

fn safe_alignment_lands_at_origin_start<S: LayoutScalar>(
    authored: AlignContent,
    free_space: S,
) -> bool {
    free_space < S::ZERO
        && matches!(
            authored,
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter
        )
}

fn final_item_subject_interval<Node, S: LayoutScalar>(
    final_items: &[FinalFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Option<(S, S)> {
    final_items.iter().fold(None, |bounds, item| {
        let border_box = item
            .output
            .scroll_geometry
            .expect("final flex items retain canonical geometry")
            .border_box();
        let (origin, end) = match constants.axes.main_physical_axis() {
            PhysicalAxis::Horizontal => {
                let origin = item.location.x + border_box.origin().x;
                (
                    origin - item.margin.left.max(S::ZERO),
                    origin + border_box.size().width + item.margin.right.max(S::ZERO),
                )
            }
            PhysicalAxis::Vertical => {
                let origin = item.location.y + border_box.origin().y;
                (
                    origin - item.margin.top.max(S::ZERO),
                    origin + border_box.size().height + item.margin.bottom.max(S::ZERO),
                )
            }
        };
        Some(bounds.map_or((origin, end), |(minimum, maximum): (S, S)| {
            (minimum.min(origin), maximum.max(end))
        }))
    })
}

fn line_cross_free_space<S: LayoutScalar>(
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<S> {
    let container_cross_size = constants.axes.cross_size(constants.node_inner_size)?;
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(lines.len().saturating_sub(1));
    Some(container_cross_size - used_cross_size)
}

fn line_subject_interval<S: LayoutScalar>(
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<(S, S)> {
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    lines.iter().fold(None, |bounds, line| {
        let origin = constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            line.offset_cross,
            line.cross_size,
            S::ZERO,
        );
        let end = origin + line.cross_size;
        Some(bounds.map_or((origin, end), |(minimum, maximum): (S, S)| {
            (minimum.min(origin), maximum.max(end))
        }))
    })
}

fn set_alignment_subject_interval<S: LayoutScalar>(
    contributions: &mut ScrollContributionAccumulatorOf<S>,
    axis: PhysicalAxis,
    minimum: S,
    maximum: S,
) -> Result<(), crate::scroll::ScrollContributionErrorOf<S>> {
    let subject = match axis {
        PhysicalAxis::Horizontal => ScrollRectOf::try_new(
            Point::new(minimum, S::ZERO),
            Size::new(maximum - minimum, S::ZERO),
        ),
        PhysicalAxis::Vertical => ScrollRectOf::try_new(
            Point::new(S::ZERO, minimum),
            Size::new(S::ZERO, maximum - minimum),
        ),
    }?;
    contributions.set_active_alignment_subject(axis, subject);
    Ok(())
}

fn child_flow_end<S: LayoutScalar>(child: FlexChildContribution<S>, side: PhysicalSide) -> S {
    let border_box = child.geometry.border_box();
    let origin = border_box.origin();
    let size = border_box.size();
    match side {
        PhysicalSide::Top => child.location.y + origin.y - child.margin.top.max(S::ZERO),
        PhysicalSide::Right => {
            child.location.x + origin.x + size.width + child.margin.right.max(S::ZERO)
        }
        PhysicalSide::Bottom => {
            child.location.y + origin.y + size.height + child.margin.bottom.max(S::ZERO)
        }
        PhysicalSide::Left => child.location.x + origin.x - child.margin.left.max(S::ZERO),
    }
}

fn include_farthest_flow_end<S: LayoutScalar>(
    end: &mut Option<S>,
    side: PhysicalSide,
    candidate: S,
) {
    *end = Some(end.map_or(candidate, |current| match side {
        PhysicalSide::Top | PhysicalSide::Left => current.min(candidate),
        PhysicalSide::Right | PhysicalSide::Bottom => current.max(candidate),
    }));
}

pub(super) fn flex_container_scroll_geometry<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    constants: &Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<Node, ScrollGeometryOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    CanonicalScrollSourceBuilderOf::for_node(
        box_projection,
        target_projection,
        scroll_box.border_box().size(),
        constants.border,
        constants.padding,
        constants.settled_auto_scrollbars,
        constants.axes.scroll_origin_axes(),
    )
    .geometry_from_contributions(contributions, scroll_box.border_box())
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

pub(super) fn retained_flex_child_scroll_geometry<S: LayoutScalar>(
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<ScrollGeometryOf<S>>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let settled_auto_scrollbars = crate::scroll::SettledAutoScrollbarState::INITIAL;
    let source = match child_compute_geometry {
        Some(ref geometry) => CanonicalRetainedScrollSourceOf::Existing(geometry),
        None => CanonicalRetainedScrollSourceOf::Reconstruct { content_size },
    };
    CanonicalScrollSourceBuilderOf::for_node(
        box_projection,
        target_projection,
        size,
        border,
        padding,
        settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_retained_source(
        source,
        CanonicalScrollRangeSeedPolicy::IncludeReservedGutter,
    )
}
