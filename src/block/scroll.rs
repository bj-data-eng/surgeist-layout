use super::Constants;
use super::in_flow::block_final_in_flow_end;
use crate::error::{layout_child_geometry_error, layout_own_geometry_error};
use crate::geometry::LogicalSizeOf;
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollBoxSourceOf, CanonicalScrollGeometryErrorOf,
    CanonicalScrollRangeSeedPolicy, CanonicalScrollSourceBuilderOf, ScrollBoxProjection,
    ScrollContributionAccumulatorOf, ScrollOriginAxes, ScrollOriginProgression,
    ScrollTargetProjection, canonical_scroll_box_from_source, scrollbar_size_from_overflow,
};
use crate::{
    Compute, Edges, LayoutErrorOf, LayoutResultOf, LayoutScalar, LogicalAxis, NodeInputOf, Point,
    RunMode, ScrollGeometryOf, ScrollRectOf, Size, Traverse,
};

pub(super) fn prepare_scroll_contributions<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    projection: ScrollBoxProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    scroll_content_size: LogicalSizeOf<S>,
    mut contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<Node, ScrollContributionAccumulatorOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let final_scroll_box =
        canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf::from_projection(
            projection,
            output_size,
            constants.border,
            constants.padding,
            constants.settled_auto_scrollbars,
        ))
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    contributions.replace_container_seed(final_scroll_box.padding_box());
    contributions.exclude_reserved_gutter_from_range();
    for (axis, extent) in [
        (LogicalAxis::Inline, scroll_content_size.inline),
        (LogicalAxis::Block, scroll_content_size.block),
    ] {
        contributions
            .record_final_in_flow_end(
                constants.flow_axes,
                axis,
                block_final_in_flow_end(
                    final_scroll_box.content_box(),
                    constants.flow_axes,
                    axis,
                    extent,
                ),
            )
            .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    }
    Ok(contributions)
}

pub(super) struct PublishedScrollOf<S: LayoutScalar> {
    pub(super) geometry: ScrollGeometryOf<S>,
    pub(super) content_size: Size<S>,
}

pub(super) fn finish_scroll_geometry<Tree, S, M>(
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    mut contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, PublishedScrollOf<S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    contributions
        .include_terminal_padding(constants.padding)
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    let scroll_geometry = block_scroll_geometry::<Tree, S, M>(
        node,
        run_mode,
        box_projection,
        target_projection,
        constants,
        output_size,
        contributions,
    )?;
    let content_size = contributions
        .content_size_from_anchor(scroll_geometry.content_box().origin())
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    Ok(PublishedScrollOf {
        geometry: scroll_geometry,
        content_size,
    })
}

fn block_scroll_geometry<Tree, S, M>(
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ScrollGeometryOf<S>, S, M>
where
    Tree: Compute<M, Scalar = S>,
    S: LayoutScalar,
{
    let target_border_box = ScrollRectOf::try_new(Point::ZERO, output_size)
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    CanonicalScrollSourceBuilderOf::for_node(
        box_projection,
        target_projection,
        output_size,
        constants.border,
        constants.padding,
        constants.settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_contributions(contributions, target_border_box)
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

pub(super) fn block_inline_geometry_error<Node, S, M, E>(
    container: Node,
    subject: Option<Node>,
    run_mode: RunMode,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    match subject {
        Some(subject) => layout_child_geometry_error(container, subject, error),
        None => layout_own_geometry_error(container, run_mode, error),
    }
}

pub(super) fn retained_child_scroll_geometry<S: LayoutScalar>(
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
        CanonicalScrollRangeSeedPolicy::ExcludeReservedGutter,
    )
}

pub(super) fn child_scrollbar_size<S: LayoutScalar>(style: &NodeInputOf<S>) -> Size<S> {
    scrollbar_size_from_overflow(
        style.overflow,
        style.item_is_replaced,
        style.scrollbar_width.get(),
    )
}
