use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf, SizingAlgorithm, sizing_resolution_error,
};
use crate::geometry::{FlowAxes, PhysicalAxis, PhysicalSide};
use crate::layout_math::{MaxBeforeMinScalarClampExt, OptionalSizeExt};
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollRangeSeedPolicy,
    CanonicalScrollSourceBuilderOf, ScrollOriginAxes, ScrollOriginProgression,
    SettledAutoScrollbarState,
};
use crate::sizing::resolve::{
    EdgesResultExt, SizingResolutionError, resolve_auto_or_zero_fallible,
    resolve_length_or_zero_fallible, resolve_maximum_optional,
};
use crate::{
    AvailableOf, BoxSizing, CacheAccess, Compute, ComputeInputOf, ComputeOutputOf, Edges,
    LayoutInputOf, LayoutScalar, NodeInputOf, NodeOutputOf, Point, Size, Traverse,
};

pub(crate) fn compute_hidden<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    source_index: crate::SourceIndex,
    containing_layout_context: crate::ContainingLayoutContext,
    containing_auto_scrollbar_pass: SettledAutoScrollbarState,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    ComputeOutputOf<<Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>
where
    Tree: Compute<M>
        + CacheAccess<M, Node = <Tree as Traverse>::Node, Scalar = <Tree as Traverse>::Scalar>,
{
    tree.cache_clear(node);
    tree.set_unrounded_inline_fragment_state(node, None);
    tree.set_unrounded(node, NodeOutputOf::with_source_index(source_index));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        match tree.layout_input(child) {
            LayoutInputOf::Box(_) => {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::with_source_index(crate::SourceIndex::new(index)),
                );
                let descendant_context = crate::ContainingLayoutContext::new(
                    containing_layout_context.flow_axes(),
                    crate::ParentFormattingContext::NoParent,
                );
                tree.compute_child(
                    child,
                    ComputeInputOf::hidden_in_containing_pass(
                        descendant_context,
                        containing_auto_scrollbar_pass,
                    ),
                )?;
            }
            LayoutInputOf::InlineText(_)
            | LayoutInputOf::LineBreak(_)
            | LayoutInputOf::InlineBoundary(_) => {
                tree.cache_clear(child);
                tree.set_unrounded_inline_fragment_state(child, None);
                tree.set_unrounded(
                    child,
                    NodeOutputOf::with_source_index(crate::SourceIndex::new(index)),
                );
            }
        }
    }

    Ok(ComputeOutputOf::HIDDEN)
}

pub(crate) fn compute_root<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(root).clone();
    let containing_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let containing_layout_context = crate::ContainingLayoutContext::new(
        containing_flow_axes,
        crate::ParentFormattingContext::NoParent,
    );
    let parent = available.map(AvailableOf::into_option);
    let known =
        root_known_inline::<Tree, M>(tree, root, &style, containing_flow_axes, available, parent)?;
    let output = tree.compute_child(
        root,
        ComputeInputOf::root_layout(known, parent, containing_layout_context, available),
    )?;
    let root_edges = resolve_root_edges(tree, root, &style, containing_flow_axes, parent)?;
    let scroll_geometry = root_scroll_geometry(root, &style, output, &root_edges)?;
    let location = root_start_location(containing_flow_axes, output.size, available);

    tree.set_unrounded(
        root,
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location,
            size: output.size,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    );
    Ok(())
}

pub(crate) fn compute_flex_item_root<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
    context: crate::FlexItemRootContextOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(root).clone();
    let containing_flow_axes = context.parent_flow_axes();
    let containing_layout_context = crate::ContainingLayoutContext::new(
        containing_flow_axes,
        crate::ParentFormattingContext::Flex,
    );
    let parent = context.viewport_available().map(AvailableOf::into_option);
    let known =
        root_known_inline::<Tree, M>(tree, root, &style, containing_flow_axes, available, parent)?;
    let output = tree.compute_child(
        root,
        ComputeInputOf::flex_item_root(known, parent, containing_layout_context, available),
    )?;
    let root_edges = resolve_root_edges(tree, root, &style, containing_flow_axes, parent)?;
    let scroll_geometry = root_scroll_geometry(root, &style, output, &root_edges)?;
    tree.set_unrounded(
        root,
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location: Point::ZERO,
            size: output.size,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    );
    Ok(())
}

struct RootEdges<S: LayoutScalar> {
    padding: Edges<S>,
    border: Edges<S>,
    margin: Edges<S>,
}

fn root_scroll_geometry<Node, S, M>(
    node: Node,
    style: &NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    edges: &RootEdges<S>,
) -> LayoutResultOf<Node, Option<crate::ScrollGeometryOf<S>>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    if style.display == crate::Display::None {
        return Ok(None);
    }

    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let settled_auto_scrollbars = SettledAutoScrollbarState::INITIAL;
    let source = match output.scroll_geometry {
        Some(ref geometry) => CanonicalRetainedScrollSourceOf::Existing(geometry),
        None => CanonicalRetainedScrollSourceOf::Reconstruct {
            content_size: output.content_size,
        },
    };
    CanonicalScrollSourceBuilderOf::for_node(
        style,
        flow_axes,
        output.size,
        edges.border,
        edges.padding,
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
    .map(Some)
    .map_err(|error| root_scroll_error(node, error))
}

fn root_scroll_error<Node, S, M, E>(node: Node, error: E) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    let kind =
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry);

    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        kind,
    )
}

type RootKnownInlineResult<Node, S, M> = LayoutResultOf<Node, Size<Option<S>>, S, M>;

fn resolve_root_edges<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    parent: Size<Option<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, RootEdges<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.margin, parent, |length, basis| {
            resolve_auto_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;

    Ok(RootEdges {
        padding,
        border,
        margin,
    })
}

fn root_known_inline<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    fill_available: Size<AvailableOf<Tree::Scalar>>,
    percentage_parent: Size<Option<Tree::Scalar>>,
) -> RootKnownInlineResult<<Tree as Traverse>::Node, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let inline_axis = containing_flow_axes.inline_axis();
    if style.display.is_inline_level()
        || style.item_is_replaced
        || !root_physical_axis_value(style.size.clone(), inline_axis).is_auto()
        || !root_physical_axis_value(style.min_size.clone(), inline_axis).is_auto()
    {
        return Ok(Size::NONE);
    }

    let Some(available_inline) =
        root_physical_axis_value(fill_available, inline_axis).into_option()
    else {
        return Ok(Size::NONE);
    };
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = Size::new(
        resolve_maximum_optional(
            &style.max_size.width,
            SizingAlgorithm::Block,
            PhysicalAxis::Horizontal,
            percentage_parent.width,
            false,
        ),
        resolve_maximum_optional(
            &style.max_size.height,
            SizingAlgorithm::Block,
            PhysicalAxis::Vertical,
            percentage_parent.height,
            false,
        ),
    );
    if matches!(max_size.width, Err(SizingResolutionError::Unsupported(_)))
        || matches!(max_size.height, Err(SizingResolutionError::Unsupported(_)))
    {
        // The root optimization cannot know whether a root with block display is
        // measured as a leaf. Defer contextual rejection to the actual consumer.
        return Ok(Size::NONE);
    }
    let max_size = Size::new(
        max_size
            .width
            .map_err(|error| sizing_resolution_error(node, error))?,
        max_size
            .height
            .map_err(|error| sizing_resolution_error(node, error))?,
    )
    .add_optional(box_sizing_adjustment);

    Ok(root_known_on_axis(
        inline_axis,
        available_inline
            .clamp_max_before_min_optional(None, root_physical_axis_value(max_size, inline_axis)),
    ))
}

fn root_physical_axis_value<T>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn root_known_on_axis<S: LayoutScalar>(axis: PhysicalAxis, value: S) -> Size<Option<S>> {
    match axis {
        PhysicalAxis::Horizontal => Size::new(Some(value), None),
        PhysicalAxis::Vertical => Size::new(None, Some(value)),
    }
}

fn root_start_location<S: LayoutScalar>(
    containing_flow_axes: FlowAxes,
    root_size: Size<S>,
    available: Size<AvailableOf<S>>,
) -> Point<S> {
    Point::new(
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Horizontal,
        ),
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Vertical,
        ),
    )
}

fn root_start_coordinate<S: LayoutScalar>(
    inline_start: PhysicalSide,
    block_start: PhysicalSide,
    root_size: Size<S>,
    available: Size<AvailableOf<S>>,
    axis: PhysicalAxis,
) -> S {
    let start_side = if inline_start.axis() == axis {
        inline_start
    } else {
        block_start
    };
    match start_side {
        PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
        PhysicalSide::Right | PhysicalSide::Bottom => root_physical_axis_value(available, axis)
            .into_option()
            .map_or(S::ZERO, |extent| {
                extent - root_physical_axis_value(root_size, axis)
            }),
    }
}
