use super::flexible_lengths::{clamp_cross_size, flex_main_size};
use super::items::{FinalFlexItem, ResolvedFlexItem};
use super::lines::FlexLine;
use super::{
    AlignContent, AlignItems, BaselinesOf, ComputeOutputOf, Constants, Direction, Edges,
    LayoutScalar, Point, Size,
};
use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis};
use crate::layout_math::OptionalSizeExt;
use crate::output::PhysicalBaseline;

#[derive(Clone, Copy, Debug)]
pub(super) struct FlexItemBaseline<S: LayoutScalar> {
    flow_axes: FlowAxes,
    measured: Option<PhysicalBaseline<S>>,
}

impl<S: LayoutScalar> FlexItemBaseline<S> {
    pub(super) fn from_output(output: ComputeOutputOf<S>, flow_axes: FlowAxes) -> Self {
        Self {
            flow_axes,
            measured: output.baselines().first_block_baseline(flow_axes),
        }
    }

    pub(super) fn refresh(&mut self, output: ComputeOutputOf<S>) {
        self.measured = output.baselines().first_block_baseline(self.flow_axes);
    }

    fn physical(self, size: Size<S>) -> PhysicalBaseline<S> {
        self.measured.unwrap_or_else(|| {
            BaselinesOf::NONE.first_or_synthesize_block_baseline(self.flow_axes, size)
        })
    }

    fn axis(self, size: Size<S>) -> PhysicalAxis {
        self.physical(size).axis()
    }

    fn value(self, size: Size<S>, margin: Edges<S>) -> S {
        self.physical(size).coordinate() + self.flow_axes.line_over_edge(margin)
    }

    fn margin_box_size(self, size: Size<S>, margin: Edges<S>) -> S {
        self.flow_axes.block_axis_extent(size)
            + self.flow_axes.line_over_edge(margin)
            + self.flow_axes.line_under_edge(margin)
    }

    fn has_auto_line_margin(self, margin_is_auto: Edges<bool>) -> bool {
        self.flow_axes.line_over_edge(margin_is_auto)
            || self.flow_axes.line_under_edge(margin_is_auto)
    }

    fn translated(self, size: Size<S>, location: Point<S>) -> Point<Option<S>> {
        self.physical(size).translated(location)
    }
}

pub(super) fn align_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    constants: &Constants<S>,
) {
    let Some(container_cross_size) = constants.axes.cross_size(constants.node_inner_size) else {
        return;
    };
    let line_count = lines.len();
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(line_count.saturating_sub(1));
    let free_space = container_cross_size - used_cross_size;
    let align_content = alignment_fallback(free_space, line_count, constants.align_content);
    let mut cross_cursor = alignment_offset(
        free_space,
        line_count,
        cross_gap,
        align_content,
        constants.axes.cross_is_reversed(),
        true,
    );
    for (index, line) in lines.iter_mut().enumerate() {
        if index > 0 {
            cross_cursor = cross_cursor
                + alignment_offset(
                    free_space,
                    line_count,
                    cross_gap,
                    align_content,
                    constants.axes.cross_is_reversed(),
                    false,
                );
        }
        let delta = cross_cursor - line.offset_cross;
        line.offset_cross = cross_cursor;
        for item in &mut items[line.start..line.end] {
            item.offset_cross = item.offset_cross + delta;
        }
        cross_cursor = cross_cursor + line.cross_size;
    }
}

pub(super) fn stretch_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    constants: &Constants<S>,
) {
    if constants.align_content != AlignContent::Stretch {
        return;
    }

    let Some(container_cross_size) = constants.axes.cross_size(constants.node_inner_size) else {
        return;
    };
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(lines.len().saturating_sub(1));
    if used_cross_size >= container_cross_size || lines.is_empty() {
        return;
    }

    let addition = (container_cross_size - used_cross_size) / S::from_usize(lines.len());
    for line in lines {
        line.cross_size = line.cross_size + addition;
        align_items_on_cross_axis(
            &mut items[line.start..line.end],
            line.cross_size,
            line.offset_cross,
            constants,
        );
    }
}

pub(super) fn resolve_main_axis_auto_margins<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    for item in &mut *items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, S::ZERO);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, S::ZERO);
        }
    }

    let free_space = line_free_space(items, constants);
    if free_space <= S::ZERO {
        return;
    }

    let auto_margin_count = items
        .iter()
        .map(|item| {
            usize::from(item.margin_main_start_is_auto(constants))
                + usize::from(item.margin_main_end_is_auto(constants))
        })
        .sum::<usize>();
    if auto_margin_count == 0 {
        return;
    }

    let margin = free_space / S::from_usize(auto_margin_count);
    for item in items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, margin);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, margin);
        }
    }
}

pub(super) fn align_items_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    line_cross_size: S,
    line_cross_offset: S,
    constants: &Constants<S>,
) {
    let cross_axis = constants.axes.cross_physical_axis();
    let max_baseline = max_line_baseline(items, cross_axis);
    for item in items {
        resolve_cross_axis_auto_margins(item, line_cross_size, constants);
        let outer_cross_size = constants.axes.cross_size(item.target_size)
            + constants.axes.cross_edge_sum(item.margin);
        let free_space = line_cross_size - outer_cross_size;
        if item.align_self == AlignItems::Stretch && item.cross_size_is_auto {
            let stretched_cross_size = S::max(
                S::ZERO,
                line_cross_size - constants.axes.cross_edge_sum(item.margin),
            );
            let stretched_cross_size = clamp_cross_size(item, stretched_cross_size);
            item.target_size = constants
                .axes
                .with_cross_size(item.target_size, stretched_cross_size);
        }
        let alignment_offset = match item.align_self.safe_fallback(free_space) {
            AlignItems::Start => {
                if constants.axes.cross_is_reversed() {
                    free_space
                } else {
                    S::ZERO
                }
            }
            AlignItems::End | AlignItems::LastBaseline => {
                if constants.axes.cross_is_reversed() {
                    S::ZERO
                } else {
                    free_space
                }
            }
            AlignItems::FlexStart | AlignItems::Stretch => S::ZERO,
            AlignItems::Center => free_space / S::from_f64(2.0),
            AlignItems::FlexEnd => free_space,
            AlignItems::Baseline if item.baseline.axis(item.target_size) == cross_axis => {
                max_baseline - item.baseline.value(item.target_size, item.margin)
            }
            AlignItems::Baseline
                if constants.wraps && constants.axes.flow_direction() == Direction::Rtl =>
            {
                free_space
            }
            AlignItems::Baseline => S::ZERO,
            AlignItems::SafeEnd | AlignItems::SafeFlexEnd | AlignItems::SafeCenter => {
                unreachable!("safe_fallback returns unsafe item alignment")
            }
        };
        let line_over_margin = if item.align_self == AlignItems::Baseline
            && item.baseline.axis(item.target_size) == cross_axis
        {
            item.baseline.flow_axes.line_over_edge(item.margin)
        } else {
            constants.axes.cross_start_edge(item.margin)
        };
        item.offset_cross = line_cross_offset + line_over_margin + alignment_offset;
    }
}

pub(super) fn line_cross_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let cross_axis = constants.axes.cross_physical_axis();
    let max_baseline = max_line_baseline(items, cross_axis);
    items
        .iter()
        .map(|item| line_item_cross_size(item, max_baseline, constants))
        .fold(S::ZERO, S::max)
}

fn line_item_cross_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    max_baseline: S,
    constants: &Constants<S>,
) -> S {
    let outer_cross_size =
        constants.axes.cross_size(item.target_size) + constants.axes.cross_edge_sum(item.margin);
    if item.align_self == AlignItems::Baseline
        && item.baseline.axis(item.target_size) == constants.axes.cross_physical_axis()
        && !item.baseline.has_auto_line_margin(item.margin_is_auto)
    {
        return max_baseline - item.baseline.value(item.target_size, item.margin)
            + item.baseline.margin_box_size(item.target_size, item.margin);
    }

    outer_cross_size
}

fn max_line_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    cross_axis: PhysicalAxis,
) -> S {
    items
        .iter()
        .filter(|item| {
            item.align_self == AlignItems::Baseline
                && item.baseline.axis(item.target_size) == cross_axis
        })
        .map(|item| item.baseline.value(item.target_size, item.margin))
        .fold(S::ZERO, S::max)
}

pub(super) fn first_vertical_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.first()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
        .or_else(|| line_items.first())?;
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let location = constants.axes.point_from_main_cross(
        constants.axes.main_position_from_start(
            container,
            constants.axes.main_start_edge(constants.content_box_inset),
            item.offset_main,
            constants.axes.main_size(item.target_size),
            S::ZERO,
        ),
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            item.offset_cross,
            constants.axes.cross_size(item.target_size),
            S::ZERO,
        ),
    );
    Some(item_physical_baseline(
        location,
        item.target_size,
        item.baseline,
    ))
}

pub(super) fn last_vertical_baseline<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.last()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .rev()
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
        .or_else(|| line_items.last())?;
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let location = constants.axes.point_from_main_cross(
        constants.axes.main_position_from_start(
            container,
            constants.axes.main_start_edge(constants.content_box_inset),
            item.offset_main,
            constants.axes.main_size(item.target_size),
            S::ZERO,
        ),
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            item.offset_cross,
            constants.axes.cross_size(item.target_size),
            S::ZERO,
        ),
    );
    Some(item_physical_baseline(
        location,
        item.target_size,
        item.baseline,
    ))
}

pub(super) fn first_final_vertical_baseline<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.first()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
        .or_else(|| line_items.first())?;
    Some(item_physical_baseline(
        item.location,
        item.output.size,
        item.baseline,
    ))
}

pub(super) fn last_final_vertical_baseline<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let line = lines.last()?;
    let line_items = &items[line.start..line.end];
    let item = line_items
        .iter()
        .rev()
        .find(|item| {
            constants.axes.main_logical_axis() == LogicalAxis::Block
                || item.align_self == AlignItems::Baseline
        })
        .or_else(|| line_items.last())?;
    Some(item_physical_baseline(
        item.location,
        item.output.size,
        item.baseline,
    ))
}

fn item_physical_baseline<S: LayoutScalar>(
    location: Point<S>,
    size: Size<S>,
    baseline: FlexItemBaseline<S>,
) -> Point<Option<S>> {
    baseline.translated(size, location)
}

fn resolve_cross_axis_auto_margins<Node, S: LayoutScalar>(
    item: &mut ResolvedFlexItem<Node, S>,
    line_cross_size: S,
    constants: &Constants<S>,
) {
    let auto_start = constants.axes.normal_cross_start_edge(item.margin_is_auto);
    let auto_end = constants.axes.normal_cross_end_edge(item.margin_is_auto);
    if !auto_start && !auto_end {
        return;
    }
    if auto_start {
        constants
            .axes
            .set_normal_cross_start_edge(&mut item.margin, S::ZERO);
    }
    if auto_end {
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, S::ZERO);
    }

    let free_space = line_cross_size
        - constants.axes.cross_size(item.target_size)
        - constants.axes.cross_edge_sum(item.margin);
    if free_space <= S::ZERO {
        let overflow_end = line_cross_size
            - constants.axes.cross_size(item.target_size)
            - constants.axes.normal_cross_start_edge(item.margin);
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, overflow_end);
        return;
    }

    if auto_start && auto_end {
        let margin = free_space / S::from_f64(2.0);
        constants
            .axes
            .set_normal_cross_start_edge(&mut item.margin, margin);
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, margin);
    } else if auto_start {
        constants
            .axes
            .set_normal_cross_start_edge(&mut item.margin, free_space);
    } else if auto_end {
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, free_space);
    }
}

pub(super) fn line_free_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let Some(container_main_size) = flex_main_size(constants) else {
        return S::ZERO;
    };
    let used_space = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.axes.main_size(constants.gap)
            };
            gap + constants.axes.main_size(item.target_size)
                + constants.axes.main_edge_sum(item.margin)
        })
        .fold(S::ZERO, |sum, value| sum + value);
    container_main_size - used_space
}

pub(super) fn alignment_fallback<S: LayoutScalar>(
    free_space: S,
    item_count: usize,
    alignment_mode: AlignContent,
) -> AlignContent {
    let alignment_mode = alignment_mode.safe_fallback(free_space);
    if item_count > 1 && free_space > S::ZERO {
        return alignment_mode;
    }

    match alignment_mode {
        AlignContent::Stretch
        | AlignContent::SpaceBetween
        | AlignContent::SpaceAround
        | AlignContent::SpaceEvenly
            if free_space <= S::ZERO =>
        {
            AlignContent::FlexStart
        }
        AlignContent::Stretch | AlignContent::SpaceBetween => AlignContent::FlexStart,
        AlignContent::SpaceAround | AlignContent::SpaceEvenly => AlignContent::Center,
        mode => mode,
    }
}

pub(super) fn alignment_offset<S: LayoutScalar>(
    free_space: S,
    item_count: usize,
    gap: S,
    alignment_mode: AlignContent,
    layout_is_flex_reversed: bool,
    is_first: bool,
) -> S {
    if is_first {
        match alignment_mode {
            AlignContent::Start => {
                if layout_is_flex_reversed {
                    free_space
                } else {
                    S::ZERO
                }
            }
            AlignContent::FlexStart => S::ZERO,
            AlignContent::End => {
                if layout_is_flex_reversed {
                    S::ZERO
                } else {
                    free_space
                }
            }
            AlignContent::FlexEnd => free_space,
            AlignContent::Center => free_space / S::from_f64(2.0),
            AlignContent::Stretch | AlignContent::SpaceBetween => S::ZERO,
            AlignContent::SpaceAround => {
                if free_space >= S::ZERO {
                    (free_space / S::from_usize(item_count)) / S::from_f64(2.0)
                } else {
                    free_space / S::from_f64(2.0)
                }
            }
            AlignContent::SpaceEvenly => {
                if free_space >= S::ZERO {
                    free_space / S::from_usize(item_count + 1)
                } else {
                    free_space / S::from_f64(2.0)
                }
            }
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    } else {
        let free_space = free_space.max(S::ZERO);
        gap + match alignment_mode {
            AlignContent::SpaceBetween => free_space / S::from_usize(item_count - 1),
            AlignContent::SpaceAround => free_space / S::from_usize(item_count),
            AlignContent::SpaceEvenly => free_space / S::from_usize(item_count + 1),
            AlignContent::Start
            | AlignContent::FlexStart
            | AlignContent::End
            | AlignContent::FlexEnd
            | AlignContent::Center
            | AlignContent::Stretch => S::ZERO,
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    }
}

impl<Node, S: LayoutScalar> ResolvedFlexItem<Node, S> {
    pub(super) fn margin_main_start(&self, constants: &Constants<S>) -> S {
        constants.axes.main_start_edge(self.margin)
    }

    fn margin_main_start_is_auto(&self, constants: &Constants<S>) -> bool {
        constants.axes.main_start_edge(self.margin_is_auto)
    }

    fn margin_main_end_is_auto(&self, constants: &Constants<S>) -> bool {
        constants.axes.main_end_edge(self.margin_is_auto)
    }

    fn set_margin_main_start(&mut self, constants: &Constants<S>, value: S) {
        constants.axes.set_main_start_edge(&mut self.margin, value);
    }

    fn set_margin_main_end(&mut self, constants: &Constants<S>, value: S) {
        constants.axes.set_main_end_edge(&mut self.margin, value);
    }

    pub(super) fn final_main_location(&self, constants: &Constants<S>, output_size: Size<S>) -> S {
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        constants.axes.main_position_from_start(
            container,
            constants.axes.main_start_edge(constants.content_box_inset),
            self.offset_main,
            constants.axes.main_size(output_size),
            self.relative_main_inset(constants),
        )
    }

    fn relative_main_inset(&self, constants: &Constants<S>) -> S {
        let normal_offset = constants
            .axes
            .normal_main_start_edge(self.inset)
            .or_else(|| {
                constants
                    .axes
                    .normal_main_end_edge(self.inset)
                    .map(|inset| -inset)
            })
            .unwrap_or(S::ZERO);
        constants.axes.main_offset_from_normal_flow(normal_offset)
    }

    pub(super) fn final_cross_location(&self, constants: &Constants<S>, output_size: Size<S>) -> S {
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            self.offset_cross,
            constants.axes.cross_size(output_size),
            self.relative_cross_inset(constants),
        )
    }

    fn relative_cross_inset(&self, constants: &Constants<S>) -> S {
        let normal_offset = constants
            .axes
            .normal_cross_start_edge(self.inset)
            .or_else(|| {
                constants
                    .axes
                    .normal_cross_end_edge(self.inset)
                    .map(|inset| -inset)
            })
            .unwrap_or(S::ZERO);
        constants.axes.cross_offset_from_normal_flow(normal_offset)
    }
}

#[cfg(test)]
mod final_baseline_selection_tests {
    use super::*;
    use crate::{AvailableOf, FlexDirection, FlexWrap, NodeInputOf};

    fn default_flow_axes<S: LayoutScalar>() -> FlowAxes {
        let style = NodeInputOf::<S>::default();
        FlowAxes::new(style.writing_mode, style.direction)
    }

    fn final_item<S: LayoutScalar>(
        align_self: AlignItems,
        location_y: f64,
        baseline_y: f64,
    ) -> FinalFlexItem<(), S> {
        let size = Size::new(S::from_f64(10.0), S::from_f64(10.0));
        let output = ComputeOutputOf::from_sizes_and_baselines(
            size,
            size,
            BaselinesOf::first(Point::new(None, Some(S::from_f64(baseline_y)))),
        );
        FinalFlexItem {
            _node: core::marker::PhantomData,
            source_index: crate::SourceIndex::ZERO,
            output,
            margin: Edges::ZERO,
            align_self,
            baseline: FlexItemBaseline::from_output(output, default_flow_axes::<S>()),
            location: Point::new(S::ZERO, S::from_f64(location_y)),
        }
    }

    fn constants<S: LayoutScalar>(flex_direction: FlexDirection) -> Constants<S> {
        let flow_axes = default_flow_axes::<S>();
        Constants {
            flow_axes,
            axes: super::super::FlexAxes::new(flow_axes, flex_direction, FlexWrap::NoWrap),
            node_outer_size: Size::NONE,
            node_inner_size: Size::NONE,
            min_outer_size: Size::NONE,
            max_outer_size: Size::NONE,
            max_inner_size: Size::NONE,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            padding_border_size: Size::ZERO,
            scrollport_inset: Edges::ZERO,
            non_gutter_box_inset: Edges::ZERO,
            content_box_inset: Edges::ZERO,
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            gap: Size::ZERO,
            align_items: AlignItems::Stretch,
            authored_align_content: None,
            align_content: AlignContent::Stretch,
            authored_justify_content: None,
            justify_content: AlignContent::FlexStart,
            wraps: false,
            available: Size::splat(AvailableOf::MAX_CONTENT),
            available_main: AvailableOf::MAX_CONTENT,
        }
    }

    fn assert_final_baselines_follow_main_logical_axis<S: LayoutScalar>() {
        let items = [
            final_item::<S>(AlignItems::Stretch, 10.0, 1.0),
            final_item::<S>(AlignItems::Baseline, 20.0, 2.0),
            final_item::<S>(AlignItems::Stretch, 30.0, 3.0),
        ];
        let lines = [FlexLine {
            start: 0,
            end: items.len(),
            strut_floor: S::ZERO,
            contains_collapsed_slot: false,
            main_size: S::ZERO,
            cross_size: S::ZERO,
            offset_cross: S::ZERO,
        }];
        for (flex_direction, first, last) in [
            (FlexDirection::Row, 22.0, 22.0),
            (FlexDirection::Column, 11.0, 33.0),
        ] {
            let constants = constants::<S>(flex_direction);
            assert_eq!(
                first_final_vertical_baseline(&items, &lines, &constants),
                Some(Point::new(None, Some(S::from_f64(first)))),
                "the first final baseline follows the resolved main logical axis"
            );
            assert_eq!(
                last_final_vertical_baseline(&items, &lines, &constants),
                Some(Point::new(None, Some(S::from_f64(last)))),
                "the last final baseline follows the resolved main logical axis"
            );
        }
    }

    #[test]
    fn logical_flex_placement_final_baselines_follow_main_logical_axis_for_f32() {
        assert_final_baselines_follow_main_logical_axis::<f32>();
    }

    #[test]
    fn logical_flex_placement_final_baselines_follow_main_logical_axis_for_f64() {
        assert_final_baselines_follow_main_logical_axis::<f64>();
    }
}
