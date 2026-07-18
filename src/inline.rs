use super::{
    AvailableOf, Clear, DefaultScalar, Direction, Edges, InlineBoundaryKind, InlineMetricsOf,
    LayoutScalar, Point, Size, VerticalAlign, WritingMode,
};
use crate::geometry::{FlowAxes, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct InlineRunInput<S: LayoutScalar = DefaultScalar> {
    pub available_width: AvailableOf<S>,
    pub writing_mode: WritingMode,
    pub direction: Direction,
    pub items: Vec<InlineParticipant<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineBoxParticipant<S: LayoutScalar = DefaultScalar> {
    pub source_index: usize,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
    pub first_baseline: Option<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineFlowOf<S: LayoutScalar = DefaultScalar> {
    flow_axes: FlowAxes,
    available_inline_extent: AvailableOf<S>,
}

impl<S: LayoutScalar> InlineFlowOf<S> {
    #[must_use]
    pub(super) const fn new(
        writing_mode: WritingMode,
        direction: Direction,
        available_inline_extent: AvailableOf<S>,
    ) -> Self {
        Self {
            flow_axes: FlowAxes::new(writing_mode, direction),
            available_inline_extent,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn writing_mode(self) -> WritingMode {
        self.flow_axes.writing_mode()
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn direction(self) -> Direction {
        self.flow_axes.direction()
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn available_inline_extent(self) -> AvailableOf<S> {
        self.available_inline_extent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InlineControlAlignment {
    Baseline,
    Top,
    Bottom,
}

impl From<VerticalAlign> for InlineControlAlignment {
    fn from(value: VerticalAlign) -> Self {
        match value {
            VerticalAlign::Baseline => Self::Baseline,
            VerticalAlign::Top => Self::Top,
            VerticalAlign::Bottom => Self::Bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ForcedLineBreakControlOf<S: LayoutScalar = DefaultScalar> {
    source_index: usize,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
    clear: Clear,
}

impl<S: LayoutScalar> ForcedLineBreakControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        source_index: usize,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment,
        clear: Clear,
    ) -> Self {
        Self {
            source_index,
            flow,
            metrics,
            alignment,
            clear,
        }
    }

    #[must_use]
    pub(super) const fn source_index(self) -> usize {
        self.source_index
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn clear(self) -> Clear {
        self.clear
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineBoundaryControlOf<S: LayoutScalar = DefaultScalar> {
    source_index: usize,
    kind: InlineBoundaryKind,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
}

#[allow(dead_code)]
impl<S: LayoutScalar> InlineBoundaryControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        source_index: usize,
        kind: InlineBoundaryKind,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment,
    ) -> Self {
        Self {
            source_index,
            kind,
            flow,
            metrics,
            alignment,
        }
    }

    #[must_use]
    pub(super) const fn source_index(self) -> usize {
        self.source_index
    }

    #[must_use]
    pub(super) const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineControlItemOf<S: LayoutScalar = DefaultScalar> {
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

impl<S: LayoutScalar> InlineParticipant<S> {
    #[cfg(test)]
    pub(super) const fn new(
        source_index: usize,
        size: Size<S>,
        margin: Edges<S>,
        first_baseline: Option<S>,
    ) -> Self {
        Self::Box(AtomicInlineBoxParticipant {
            source_index,
            size,
            content_size: size,
            margin,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline,
        })
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn forced_line_break(control: ForcedLineBreakControlOf<S>) -> Self {
        Self::ForcedLineBreak(control)
    }

    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn inline_boundary(control: InlineBoundaryControlOf<S>) -> Self {
        Self::Boundary(control)
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineParticipant<S: LayoutScalar = DefaultScalar> {
    Box(AtomicInlineBoxParticipant<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

impl<S: LayoutScalar> AtomicInlineBoxParticipant<S> {
    #[must_use]
    fn advance(self) -> S {
        self.margin.left + self.size.width + self.margin.right
    }

    #[must_use]
    fn baseline(self) -> S {
        self.first_baseline
            .unwrap_or(self.size.height)
            .min(self.size.height)
    }

    #[must_use]
    fn line_baseline(self) -> S {
        self.margin.top + self.baseline()
    }

    #[must_use]
    fn line_descent(self) -> S {
        self.size.height - self.baseline() + self.margin.bottom
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InlineParticipantLayoutKind {
    Box,
    ForcedLineBreak,
    InlineBoundaryStart,
    InlineBoundaryEnd,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineParticipantLayoutItem<S: LayoutScalar = DefaultScalar> {
    pub kind: InlineParticipantLayoutKind,
    pub source_index: usize,
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct InlineRunReport<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub items: Vec<InlineParticipantLayoutItem<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PendingInlineItem<S: LayoutScalar = DefaultScalar> {
    Box {
        item: AtomicInlineBoxParticipant<S>,
        x: S,
    },
    ForcedLineBreak {
        source_index: usize,
        x: S,
    },
    Boundary {
        control: InlineBoundaryControlOf<S>,
        x: S,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
struct InlineLine<S: LayoutScalar = DefaultScalar> {
    items: Vec<PendingInlineItem<S>>,
    width: S,
    baseline: S,
    descent: S,
    has_breakable_inline_content: bool,
}

impl<S: LayoutScalar> InlineLine<S> {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[must_use]
    fn has_breakable_inline_content(&self) -> bool {
        self.has_breakable_inline_content
    }

    fn push_box(&mut self, item: AtomicInlineBoxParticipant<S>) {
        let baseline = item.line_baseline();
        self.baseline = self.baseline.max(baseline);
        self.descent = self.descent.max(item.line_descent());
        self.items.push(PendingInlineItem::Box {
            item,
            x: self.width + item.margin.left,
        });
        self.width = self.width + item.advance();
        self.has_breakable_inline_content = true;
    }

    fn push_forced_line_break(&mut self, control: ForcedLineBreakControlOf<S>) {
        let metrics = control.metrics();
        self.baseline = self.baseline.max(metrics.baseline());
        self.descent = self.descent.max(metrics.after_baseline());
        self.items.push(PendingInlineItem::ForcedLineBreak {
            source_index: control.source_index(),
            x: self.width,
        });
    }

    fn push_boundary(&mut self, control: InlineBoundaryControlOf<S>) {
        let metrics = control.metrics();
        self.baseline = self.baseline.max(metrics.baseline());
        self.descent = self.descent.max(metrics.after_baseline());
        self.items.push(PendingInlineItem::Boundary {
            control,
            x: self.width,
        });
    }

    #[must_use]
    fn height(&self) -> S {
        self.baseline + self.descent
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PendingVerticalInlineItem<S: LayoutScalar = DefaultScalar> {
    Box {
        item: AtomicInlineBoxParticipant<S>,
        logical_inline_start: S,
    },
    ForcedLineBreak {
        source_index: usize,
        logical_inline_start: S,
        baseline: S,
    },
    Boundary {
        control: InlineBoundaryControlOf<S>,
        logical_inline_start: S,
        baseline: S,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
struct VerticalInlineLine<S: LayoutScalar = DefaultScalar> {
    items: Vec<PendingVerticalInlineItem<S>>,
    inline_extent: S,
    block_extent: S,
    first_report_baseline: Option<S>,
    last_report_baseline: Option<S>,
}

impl<S: LayoutScalar> VerticalInlineLine<S> {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn push_box(&mut self, item: AtomicInlineBoxParticipant<S>) {
        self.inline_extent = self.inline_extent + item.margin.top;
        let logical_inline_start = self.inline_extent;
        self.inline_extent = self.inline_extent + item.size.height + item.margin.bottom;
        let baseline = logical_inline_start + item.baseline();
        self.first_report_baseline.get_or_insert(baseline);
        self.last_report_baseline = Some(baseline);
        self.block_extent = self
            .block_extent
            .max(item.margin.left + item.size.width + item.margin.right);
        self.items.push(PendingVerticalInlineItem::Box {
            item,
            logical_inline_start,
        });
    }

    fn push_forced_line_break(&mut self, control: ForcedLineBreakControlOf<S>) {
        let metrics = control.metrics();
        self.first_report_baseline.get_or_insert(self.inline_extent);
        self.last_report_baseline = Some(self.inline_extent);
        self.block_extent = self.block_extent.max(metrics.line_extent());
        self.items.push(PendingVerticalInlineItem::ForcedLineBreak {
            source_index: control.source_index(),
            logical_inline_start: self.inline_extent,
            baseline: metrics.baseline(),
        });
    }

    fn push_boundary(&mut self, control: InlineBoundaryControlOf<S>) {
        let metrics = control.metrics();
        self.first_report_baseline
            .get_or_insert(self.inline_extent + metrics.baseline());
        self.last_report_baseline = Some(self.inline_extent + metrics.baseline());
        self.block_extent = self.block_extent.max(metrics.line_extent());
        self.items.push(PendingVerticalInlineItem::Boundary {
            control,
            logical_inline_start: self.inline_extent,
            baseline: metrics.baseline(),
        });
    }
}

fn inline_boundary_layout_kind(kind: InlineBoundaryKind) -> InlineParticipantLayoutKind {
    match kind {
        InlineBoundaryKind::Start => InlineParticipantLayoutKind::InlineBoundaryStart,
        InlineBoundaryKind::End => InlineParticipantLayoutKind::InlineBoundaryEnd,
    }
}

#[must_use]
pub(super) fn layout_inline_run<S: LayoutScalar>(input: InlineRunInput<S>) -> InlineRunReport<S> {
    let flow_axes = FlowAxes::new(input.writing_mode, input.direction);
    if flow_axes.inline_axis() == PhysicalAxis::Vertical {
        return layout_vertical_inline_run(input);
    }

    let available_width = match input.available_width {
        AvailableOf::Definite(width) => Some(width),
        AvailableOf::MinContent => Some(inline_run_min_content_width(&input.items)),
        AvailableOf::MaxContent => None,
    };
    let mut lines = Vec::new();
    let mut line = InlineLine::<S>::default();

    for item in input.items {
        match item {
            InlineParticipant::Box(item) => {
                let advance = item.advance();
                if let Some(available_width) = available_width
                    && line.has_breakable_inline_content()
                    && line.width + advance > available_width
                {
                    lines.push(line);
                    line = InlineLine::<S>::default();
                }

                line.push_box(item);
            }
            InlineParticipant::ForcedLineBreak(control) => {
                line.push_forced_line_break(control);
                lines.push(line);
                line = InlineLine::<S>::default();
            }
            InlineParticipant::Boundary(control) => {
                line.push_boundary(control);
            }
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    let mut y = S::ZERO;
    let mut width = S::ZERO;
    let mut items = Vec::new();
    let mut first_baseline = None;
    let mut last_baseline = None;
    let report_inline_extent = lines.iter().map(|line| line.width).fold(S::ZERO, S::max);

    for line in lines {
        width = width.max(line.width);
        let line_baseline = y + line.baseline;
        let line_height = line.height();
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);

        for pending in line.items {
            match pending {
                PendingInlineItem::Box { item, x } => {
                    items.push(InlineParticipantLayoutItem {
                        kind: InlineParticipantLayoutKind::Box,
                        source_index: item.source_index,
                        location: flow_axes.physical_point(
                            LogicalPointOf::new(x, y + line.baseline - item.baseline()),
                            LogicalSizeOf::new(item.size.width, item.size.height),
                            flow_axes.physical_size(LogicalSizeOf::new(
                                report_inline_extent,
                                line_height,
                            )),
                        ),
                        size: item.size,
                        content_size: item.content_size,
                        margin: item.margin,
                        padding: item.padding,
                        border: item.border,
                        scrollbar_size: item.scrollbar_size,
                    });
                }
                PendingInlineItem::ForcedLineBreak { source_index, x } => {
                    items.push(InlineParticipantLayoutItem {
                        kind: InlineParticipantLayoutKind::ForcedLineBreak,
                        source_index,
                        location: flow_axes.physical_point(
                            LogicalPointOf::new(x, line_baseline),
                            LogicalSizeOf::new(S::ZERO, S::ZERO),
                            flow_axes.physical_size(LogicalSizeOf::new(
                                report_inline_extent,
                                line_height,
                            )),
                        ),
                        size: Size::ZERO,
                        content_size: Size::ZERO,
                        margin: Edges::ZERO,
                        padding: Edges::ZERO,
                        border: Edges::ZERO,
                        scrollbar_size: Size::ZERO,
                    });
                }
                PendingInlineItem::Boundary { control, x } => {
                    items.push(InlineParticipantLayoutItem {
                        kind: inline_boundary_layout_kind(control.kind()),
                        source_index: control.source_index(),
                        location: flow_axes.physical_point(
                            LogicalPointOf::new(x, line_baseline),
                            LogicalSizeOf::new(S::ZERO, S::ZERO),
                            flow_axes.physical_size(LogicalSizeOf::new(
                                report_inline_extent,
                                line_height,
                            )),
                        ),
                        size: Size::ZERO,
                        content_size: Size::ZERO,
                        margin: Edges::ZERO,
                        padding: Edges::ZERO,
                        border: Edges::ZERO,
                        scrollbar_size: Size::ZERO,
                    });
                }
            }
        }

        y = y + line_height;
    }

    let content_size = Size::new(width, y);

    InlineRunReport {
        size: content_size,
        content_size,
        first_baseline,
        last_baseline,
        items,
    }
}

fn layout_vertical_inline_run<S: LayoutScalar>(input: InlineRunInput<S>) -> InlineRunReport<S> {
    let flow_axes = FlowAxes::new(input.writing_mode, input.direction);
    debug_assert_eq!(flow_axes.inline_axis(), PhysicalAxis::Vertical);

    let mut lines = Vec::new();
    let mut line = VerticalInlineLine::<S>::default();

    for item in input.items {
        match item {
            InlineParticipant::Box(item) => {
                line.push_box(item);
            }
            InlineParticipant::ForcedLineBreak(control) => {
                line.push_forced_line_break(control);
                lines.push(line);
                line = VerticalInlineLine::<S>::default();
            }
            InlineParticipant::Boundary(control) => {
                line.push_boundary(control);
            }
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    layout_vertical_inline_lines(flow_axes, input.available_width, lines)
}

fn layout_vertical_inline_lines<S: LayoutScalar>(
    flow_axes: FlowAxes,
    available_width: AvailableOf<S>,
    lines: Vec<VerticalInlineLine<S>>,
) -> InlineRunReport<S> {
    let line_inline_extent = lines
        .iter()
        .map(|line| line.inline_extent)
        .fold(S::ZERO, S::max);
    let logical_block_extent = lines
        .iter()
        .map(|line| line.block_extent)
        .fold(S::ZERO, |sum, extent| sum + extent);
    let container_block_extent = if flow_axes.block_start() == PhysicalSide::Right {
        match available_width {
            AvailableOf::Definite(width) => width.max(logical_block_extent),
            AvailableOf::MinContent | AvailableOf::MaxContent => logical_block_extent,
        }
    } else {
        logical_block_extent
    };
    let mut logical_block_start = S::ZERO;
    let mut items = Vec::new();
    let mut first_baseline = None;
    let mut last_baseline = None;

    for line in lines {
        let line_block_extent = line.block_extent;
        if let Some(baseline) = line.first_report_baseline {
            first_baseline.get_or_insert(baseline);
        }
        if let Some(baseline) = line.last_report_baseline {
            last_baseline = Some(baseline);
        }

        for pending in line.items {
            match pending {
                PendingVerticalInlineItem::Box {
                    item,
                    logical_inline_start,
                } => {
                    let logical_block_start_for_item = if item.size.height == S::ZERO {
                        logical_block_start + item.margin.right - item.size.width / S::from_f64(2.0)
                    } else {
                        logical_block_start + item.margin.right
                    };
                    items.push(InlineParticipantLayoutItem {
                        kind: InlineParticipantLayoutKind::Box,
                        source_index: item.source_index,
                        location: flow_axes.physical_point(
                            LogicalPointOf::new(logical_inline_start, logical_block_start_for_item),
                            LogicalSizeOf::new(item.size.height, item.size.width),
                            flow_axes.physical_size(LogicalSizeOf::new(
                                line_inline_extent,
                                container_block_extent,
                            )),
                        ),
                        size: item.size,
                        content_size: item.content_size,
                        margin: item.margin,
                        padding: item.padding,
                        border: item.border,
                        scrollbar_size: item.scrollbar_size,
                    });
                }
                PendingVerticalInlineItem::ForcedLineBreak {
                    source_index,
                    logical_inline_start,
                    baseline,
                } => {
                    items.push(InlineParticipantLayoutItem {
                        kind: InlineParticipantLayoutKind::ForcedLineBreak,
                        source_index,
                        location: flow_axes.physical_point(
                            LogicalPointOf::new(
                                logical_inline_start,
                                logical_block_start + baseline,
                            ),
                            LogicalSizeOf::new(S::ZERO, S::ZERO),
                            flow_axes.physical_size(LogicalSizeOf::new(
                                line_inline_extent,
                                container_block_extent,
                            )),
                        ),
                        size: Size::ZERO,
                        content_size: Size::ZERO,
                        margin: Edges::ZERO,
                        padding: Edges::ZERO,
                        border: Edges::ZERO,
                        scrollbar_size: Size::ZERO,
                    });
                }
                PendingVerticalInlineItem::Boundary {
                    control,
                    logical_inline_start,
                    baseline,
                } => {
                    items.push(InlineParticipantLayoutItem {
                        kind: inline_boundary_layout_kind(control.kind()),
                        source_index: control.source_index(),
                        location: flow_axes.physical_point(
                            LogicalPointOf::new(
                                logical_inline_start,
                                logical_block_start + baseline,
                            ),
                            LogicalSizeOf::new(S::ZERO, S::ZERO),
                            flow_axes.physical_size(LogicalSizeOf::new(
                                line_inline_extent,
                                container_block_extent,
                            )),
                        ),
                        size: Size::ZERO,
                        content_size: Size::ZERO,
                        margin: Edges::ZERO,
                        padding: Edges::ZERO,
                        border: Edges::ZERO,
                        scrollbar_size: Size::ZERO,
                    });
                }
            }
        }

        logical_block_start = logical_block_start + line_block_extent;
    }

    let content_size = flow_axes.physical_size(LogicalSizeOf::new(
        line_inline_extent,
        container_block_extent,
    ));

    InlineRunReport {
        size: content_size,
        content_size,
        first_baseline,
        last_baseline,
        items,
    }
}

#[must_use]
pub(super) fn inline_run_min_content_width<S: LayoutScalar>(items: &[InlineParticipant<S>]) -> S {
    items
        .iter()
        .filter_map(|item| match item {
            InlineParticipant::Box(item) => Some(item.advance()),
            InlineParticipant::ForcedLineBreak(_) | InlineParticipant::Boundary(_) => None,
        })
        .fold(S::ZERO, S::max)
}

#[must_use]
#[cfg(test)]
pub(super) fn inline_run_max_content_width<S: LayoutScalar>(items: &[InlineParticipant<S>]) -> S {
    let mut max_width = S::ZERO;
    let mut segment_width = S::ZERO;
    for item in items {
        match item {
            InlineParticipant::Box(item) => {
                segment_width = segment_width + item.advance();
            }
            InlineParticipant::ForcedLineBreak(_) => {
                max_width = max_width.max(segment_width);
                segment_width = S::ZERO;
            }
            InlineParticipant::Boundary(_) => {}
        }
    }
    max_width.max(segment_width)
}
