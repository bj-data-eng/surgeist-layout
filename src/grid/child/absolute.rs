use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{
    LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalProgression,
};
use crate::sizing::resolve::{
    resolve_maximum_optional, resolve_minimum_optional, resolve_preferred_optional,
};

#[derive(Clone, Copy)]
struct OrdinaryAbsoluteGridContext<'a, S: LayoutScalar> {
    container_style: &'a NodeInputOf<S>,
    constants: &'a Constants<S>,
    containing_size: Size<S>,
    column: super::GridPlacement,
    row: super::GridPlacement,
    column_offsets: &'a [S],
    row_offsets: &'a [S],
    columns: &'a [S],
    rows: &'a [S],
    gap: LogicalSizeOf<S>,
    column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    lines: GridLines,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridContext<'a, S: LayoutScalar>(
    OrdinaryAbsoluteGridContext<'a, S>,
);

pub(in crate::grid) struct OrdinaryAbsoluteGridContextInput<'a, S: LayoutScalar> {
    pub(in crate::grid) container_style: &'a NodeInputOf<S>,
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) containing_size: Size<S>,
    pub(in crate::grid) column: super::GridPlacement,
    pub(in crate::grid) row: super::GridPlacement,
    pub(in crate::grid) column_offsets: &'a [S],
    pub(in crate::grid) row_offsets: &'a [S],
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) lines: GridLines,
}

pub(in crate::grid) struct OrdinaryAbsoluteGridGeometryContextInput<'a, S: LayoutScalar> {
    pub(in crate::grid) container_style: &'a NodeInputOf<S>,
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) containing_size: Size<S>,
    pub(in crate::grid) column: super::GridPlacement,
    pub(in crate::grid) row: super::GridPlacement,
    pub(in crate::grid) column_offsets: &'a [S],
    pub(in crate::grid) row_offsets: &'a [S],
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) lines: GridLines,
}

impl<'a, S: LayoutScalar> AbsoluteGridContext<'a, S> {
    pub(in crate::grid) fn ordinary(input: OrdinaryAbsoluteGridContextInput<'a, S>) -> Self {
        let OrdinaryAbsoluteGridContextInput {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            lines,
        } = input;
        Self(OrdinaryAbsoluteGridContext {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            column_geometry: None,
            row_geometry: None,
            lines,
            containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
        })
    }

    pub(in crate::grid) fn ordinary_with_geometry(
        input: OrdinaryAbsoluteGridGeometryContextInput<'a, S>,
    ) -> Self {
        let OrdinaryAbsoluteGridGeometryContextInput {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            column_geometry,
            row_geometry,
            lines,
        } = input;
        Self(OrdinaryAbsoluteGridContext {
            container_style,
            constants,
            containing_size,
            column,
            row,
            column_offsets,
            row_offsets,
            columns,
            rows,
            gap,
            column_geometry: Some(column_geometry),
            row_geometry: Some(row_geometry),
            lines,
            containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
        })
    }

    pub(in crate::grid) fn with_containing_auto_scrollbar_pass(
        mut self,
        containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    ) -> Self {
        self.0.containing_auto_scrollbar_pass = containing_auto_scrollbar_pass;
        self
    }
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridAreaInput<'a, S: LayoutScalar> {
    pub(in crate::grid) column: super::GridPlacement,
    pub(in crate::grid) row: super::GridPlacement,
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(in crate::grid) row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(in crate::grid) column_offsets: &'a [S],
    pub(in crate::grid) row_offsets: &'a [S],
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) lines: GridLines,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridAxisInput<'a, S: LayoutScalar = Scalar> {
    pub(in crate::grid) placement: super::GridPlacement,
    pub(in crate::grid) tracks: &'a [S],
    pub(in crate::grid) offsets: &'a [S],
    pub(in crate::grid) geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) padding_box_location: S,
    pub(in crate::grid) padding_box_size: S,
    pub(in crate::grid) is_reverse: bool,
    pub(in crate::grid) explicit_start: usize,
    pub(in crate::grid) explicit_count: usize,
}
pub(in crate::grid) fn layout_absolute_grid_child<Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    source_index: usize,
    child_style: &NodeInputOf<Tree::Scalar>,
    context: AbsoluteGridContext<'_, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridChildContribution<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let context = context.0;
    let container_style = context.container_style;
    let constants = context.constants;
    let containing_size = context.containing_size;
    let area = absolute_grid_area(AbsoluteGridAreaInput {
        column: context.column,
        row: context.row,
        columns: context.columns,
        rows: context.rows,
        gap: context.gap,
        column_geometry: context.column_geometry,
        row_geometry: context.row_geometry,
        column_offsets: context.column_offsets,
        row_offsets: context.row_offsets,
        constants,
        lines: context.lines,
    });
    let containing_flow_axes = constants.flow_axes;
    let physical_area_size = containing_flow_axes.physical_size(area.size);
    let area_parent = physical_area_size.map(Some);
    let unresolved_margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.margin, area_parent, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let available_size = Size::new(
        (physical_area_size.width - non_auto_margin.horizontal_sum()).max(Tree::Scalar::ZERO),
        (physical_area_size.height - non_auto_margin.vertical_sum()).max(Tree::Scalar::ZERO),
    );
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.padding, area_parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.border, area_parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let style_size = Size::new(
        resolve_preferred_optional(
            &child_style.size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_preferred_optional(
            &child_style.size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let padding_border_size = (padding + border).sum_axes();
    let min_size = Size::new(
        resolve_minimum_optional(
            &child_style.min_size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_minimum_optional(
            &child_style.min_size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .add_optional(box_sizing_adjustment)
    .or(padding_border_size.map(Some))
    .max_optional(padding_border_size.map(Some))
    .apply_aspect_ratio(child_style.aspect_ratio);
    let max_size = Size::new(
        resolve_maximum_optional(
            &child_style.max_size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_maximum_optional(
            &child_style.max_size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let inset = child_style
        .inset
        .zip_size(area_parent, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let mut known = Size::new(
        style_size.width.or_else(|| {
            inset.left.zip(inset.right).map(|(left, right)| {
                (physical_area_size.width - non_auto_margin.horizontal_sum() - left - right)
                    .max(Tree::Scalar::ZERO)
            })
        }),
        style_size.height.or_else(|| {
            inset.top.zip(inset.bottom).map(|(top, bottom)| {
                (physical_area_size.height - non_auto_margin.vertical_sum() - top - bottom)
                    .max(Tree::Scalar::ZERO)
            })
        }),
    );
    if let (Some(ratio), Some(width)) = (child_style.aspect_ratio, known.width)
        && child_style.size.height.is_auto()
    {
        known.height = Some(width / ratio.get());
    } else if let (Some(ratio), Some(height)) = (child_style.aspect_ratio, known.height)
        && child_style.size.width.is_auto()
    {
        known.width = Some(height * ratio.get());
    }
    let known = known
        .apply_aspect_ratio(child_style.aspect_ratio)
        .clamp_optional(min_size, max_size);
    let output = tree.compute_child(
        child,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            known,
            area_parent,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    container_style.writing_mode,
                    container_style.direction,
                ),
                crate::ParentFormattingContext::Grid,
            ),
            Size::new(
                AvailableOf::definite(available_size.width),
                AvailableOf::definite(available_size.height),
            ),
        )
        .with_containing_auto_scrollbar_pass(context.containing_auto_scrollbar_pass),
    )?;
    let final_size = known
        .unwrap_or(output.size)
        .clamp_optional(min_size, max_size);
    let justify = child_style
        .justify_self
        .unwrap_or(container_style.justify_items.unwrap_or(AlignItems::Start));
    let align = child_style
        .align_self
        .unwrap_or(container_style.align_items.unwrap_or(AlignItems::Start));
    let (location, margin) = {
        let logical_size = containing_flow_axes.logical_size(final_size);
        let logical_margin = containing_flow_axes.logical_edges(unresolved_margin);
        let logical_inset = containing_flow_axes.logical_edges(inset);
        let inline_axis = absolute_grid_axis(AbsoluteGridAxis {
            area_location: area.location.inline,
            static_area_location: area.static_location.inline,
            area_size: area.size.inline,
            static_area_size: area.static_size.inline,
            size: logical_size.inline,
            margin_start: logical_margin.inline_start,
            margin_end: logical_margin.inline_end,
            inset_start: logical_inset.inline_start,
            inset_end: logical_inset.inline_end,
            alignment: justify,
            progression: PhysicalProgression::Increasing,
        });
        let block_axis = absolute_grid_axis(AbsoluteGridAxis {
            area_location: area.location.block,
            static_area_location: area.static_location.block,
            area_size: area.size.block,
            static_area_size: area.static_size.block,
            size: logical_size.block,
            margin_start: logical_margin.block_start,
            margin_end: logical_margin.block_end,
            inset_start: logical_inset.block_start,
            inset_end: logical_inset.block_end,
            alignment: align,
            progression: PhysicalProgression::Increasing,
        });
        (
            containing_flow_axes.physical_point(
                LogicalPointOf::new(inline_axis.location, block_axis.location),
                logical_size,
                containing_size,
            ),
            containing_flow_axes.physical_edges(LogicalEdgesOf::new(
                inline_axis.margin_start,
                inline_axis.margin_end,
                block_axis.margin_start,
                block_axis.margin_end,
            )),
        )
    };

    let scroll_geometry = retained_grid_child_scroll_geometry(
        child_style,
        final_size,
        output.content_size,
        padding,
        border,
        output.scroll_geometry,
    )
    .map_err(|error| layout_child_geometry_error(child, child, error))?;
    tree.set_unrounded(
        child,
        NodeOutputOf {
            source_index: crate::SourceIndex::new(source_index),
            location,
            size: final_size,
            content_size: output.content_size,
            scroll_geometry: Some(scroll_geometry),
            border,
            padding,
            margin,
        },
    );

    Ok(GridChildContribution {
        source_index: crate::SourceIndex::new(source_index),
        location,
        margin,
        geometry: scroll_geometry,
        descendants: scroll_geometry.propagatable_descendant_intervals(),
        overflow: UsedOverflow::from_computed(child_style.overflow, child_style.item_is_replaced),
        in_flow: false,
    })
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct LogicalAbsoluteGridArea<S: LayoutScalar = Scalar> {
    pub(in crate::grid) location: LogicalPointOf<S>,
    pub(in crate::grid) static_location: LogicalPointOf<S>,
    pub(in crate::grid) size: LogicalSizeOf<S>,
    pub(in crate::grid) static_size: LogicalSizeOf<S>,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridAxisArea<S: LayoutScalar = Scalar> {
    pub(in crate::grid) location: S,
    pub(in crate::grid) size: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct AbsoluteGridAxis<S: LayoutScalar = Scalar> {
    pub(in crate::grid) area_location: S,
    pub(in crate::grid) static_area_location: S,
    pub(in crate::grid) area_size: S,
    pub(in crate::grid) static_area_size: S,
    pub(in crate::grid) size: S,
    pub(in crate::grid) margin_start: Option<S>,
    pub(in crate::grid) margin_end: Option<S>,
    pub(in crate::grid) inset_start: Option<S>,
    pub(in crate::grid) inset_end: Option<S>,
    pub(in crate::grid) alignment: AlignItems,
    pub(in crate::grid) progression: PhysicalProgression,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct ResolvedAbsoluteGridAxis<S: LayoutScalar = Scalar> {
    pub(in crate::grid) location: S,
    pub(in crate::grid) margin_start: S,
    pub(in crate::grid) margin_end: S,
}

pub(in crate::grid) fn absolute_grid_axis<S: LayoutScalar>(
    axis: AbsoluteGridAxis<S>,
) -> ResolvedAbsoluteGridAxis<S> {
    let AbsoluteGridAxis {
        area_location,
        static_area_location,
        area_size,
        static_area_size,
        size,
        margin_start,
        margin_end,
        inset_start,
        inset_end,
        alignment,
        progression,
    } = axis;
    let non_auto_start = margin_start.unwrap_or(S::ZERO);
    let non_auto_end = margin_end.unwrap_or(S::ZERO);
    let raw_free_space = area_size - size - non_auto_start - non_auto_end;
    let free_space = raw_free_space.max(S::ZERO);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let auto_margin = if auto_margin_count > 0 {
        free_space / S::from_usize(auto_margin_count)
    } else {
        S::ZERO
    };
    let resolved_start = margin_start.unwrap_or(auto_margin);
    let resolved_end = margin_end.unwrap_or(auto_margin);
    let uses_static_area = inset_start.is_none() && inset_end.is_none();
    let offset = match (inset_start, inset_end) {
        (Some(_), Some(end)) if progression.is_decreasing() => {
            area_size - end - size - non_auto_end
        }
        (Some(start), _) => start + non_auto_start,
        (None, Some(end)) => area_size - end - size - non_auto_end,
        (None, None) => match alignment.safe_fallback(raw_free_space) {
            AlignItems::Start
            | AlignItems::FlexStart
            | AlignItems::Baseline
            | AlignItems::Stretch
                if progression.is_decreasing() =>
            {
                static_area_size - size - resolved_end
            }
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline
                if progression.is_decreasing() =>
            {
                resolved_start
            }
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
                static_area_size - size - resolved_end
            }
            AlignItems::Center => {
                (static_area_size - size + resolved_start - resolved_end) / S::from_f64(2.0)
            }
            AlignItems::Start
            | AlignItems::FlexStart
            | AlignItems::Baseline
            | AlignItems::Stretch => resolved_start,
            AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
                unreachable!("safe_fallback returns unsafe item alignment")
            }
        },
    };
    let base_location = if uses_static_area {
        static_area_location
    } else {
        area_location
    };
    ResolvedAbsoluteGridAxis {
        location: base_location + offset,
        margin_start: resolved_start,
        margin_end: resolved_end,
    }
}
