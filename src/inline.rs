use super::{
    AvailableOf, Clear, DefaultScalar, Direction, Edges, InlineBoundaryKind, InlineBreakKind,
    InlineMetricsOf, InlineSegmentId, InlineWhitespaceEdge, LayoutScalar, Point,
    ShapedInlineSegmentOf, Size, TextAlign, VerticalAlign, WritingMode,
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
pub(super) struct ShapedTextParticipantOf<S: LayoutScalar = DefaultScalar> {
    pub source_index: usize,
    pub segment: ShapedInlineSegmentOf<S>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShapedTextRunInputOf<S: LayoutScalar = DefaultScalar> {
    pub available_inline_extent: AvailableOf<S>,
    pub flow_axes: FlowAxes,
    pub text_align: TextAlign,
    pub participants: Vec<ShapedTextParticipantOf<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ShapedTextFragmentSourceOf<S: LayoutScalar = DefaultScalar> {
    pub source_index: usize,
    pub segment_id: InlineSegmentId,
    pub inline_start: S,
    pub block_start: S,
    pub inline_extent: S,
    pub block_extent: S,
    pub baseline: S,
    pub line_index: usize,
    pub visual_index: usize,
    pub replacement_inline_extent: Option<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ShapedTextAnchorOf<S: LayoutScalar = DefaultScalar> {
    pub source_index: usize,
    pub inline_start: S,
    pub block_start: S,
    pub baseline: S,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShapedTextRunReportOf<S: LayoutScalar = DefaultScalar> {
    pub inline_extent: S,
    pub block_extent: S,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub fragments: Vec<ShapedTextFragmentSourceOf<S>>,
    pub anchors: Vec<ShapedTextAnchorOf<S>>,
}

#[derive(Clone, Copy)]
struct SelectedTextSegmentOf<S: LayoutScalar> {
    participant: ShapedTextParticipantOf<S>,
    discarded: bool,
    replacement_inline_extent: Option<S>,
}

#[derive(Clone)]
struct SelectedTextLineOf<S: LayoutScalar> {
    segments: Vec<SelectedTextSegmentOf<S>>,
    baseline: S,
    after_baseline: S,
    used_inline_extent: S,
}

fn discards_at_start(edge: InlineWhitespaceEdge) -> bool {
    matches!(
        edge,
        InlineWhitespaceEdge::DiscardAtLineStart | InlineWhitespaceEdge::DiscardAtBoth
    )
}

fn discards_at_end(edge: InlineWhitespaceEdge) -> bool {
    matches!(
        edge,
        InlineWhitespaceEdge::DiscardAtLineEnd | InlineWhitespaceEdge::DiscardAtBoth
    )
}

fn select_text_line<S: LayoutScalar>(
    participants: &[ShapedTextParticipantOf<S>],
    selected_break: bool,
    strut: Option<InlineMetricsOf<S>>,
) -> SelectedTextLineOf<S> {
    let mut discarded = vec![false; participants.len()];
    for (index, participant) in participants.iter().enumerate() {
        if discards_at_start(participant.segment.whitespace_edge()) {
            discarded[index] = true;
        } else {
            break;
        }
    }
    for (index, participant) in participants.iter().enumerate().rev() {
        if discards_at_end(participant.segment.whitespace_edge()) {
            discarded[index] = true;
        } else {
            break;
        }
    }

    let mut baseline = strut.map_or(S::ZERO, InlineMetricsOf::baseline);
    let mut after_baseline = strut.map_or(S::ZERO, InlineMetricsOf::after_baseline);
    let mut used_inline_extent = S::ZERO;
    let selected_replacement = selected_break
        .then(|| {
            participants.last().and_then(|participant| {
                participant
                    .segment
                    .following_break()
                    .replacement_inline_extent()
            })
        })
        .flatten();
    let segments = participants
        .iter()
        .copied()
        .enumerate()
        .map(|(index, participant)| {
            if !discarded[index] {
                let metrics = participant.segment.metrics();
                baseline = baseline.max(metrics.baseline());
                after_baseline = after_baseline.max(metrics.after_baseline());
                used_inline_extent = used_inline_extent + participant.segment.inline_extent();
            }
            let replacement_inline_extent = (index + 1 == participants.len())
                .then_some(selected_replacement)
                .flatten();
            if let Some(replacement) = replacement_inline_extent {
                used_inline_extent = used_inline_extent + replacement;
            }
            SelectedTextSegmentOf {
                participant,
                discarded: discarded[index],
                replacement_inline_extent,
            }
        })
        .collect();

    SelectedTextLineOf {
        segments,
        baseline,
        after_baseline,
        used_inline_extent,
    }
}

fn shaped_text_min_content<S: LayoutScalar>(participants: &[ShapedTextParticipantOf<S>]) -> S {
    let mut maximum = S::ZERO;
    let mut group_start = 0;
    for (index, participant) in participants.iter().enumerate() {
        if participant.segment.following_break().kind() == InlineBreakKind::Prohibited {
            continue;
        }
        let line = select_text_line(&participants[group_start..=index], true, None);
        maximum = maximum.max(line.used_inline_extent);
        group_start = index + 1;
    }
    if group_start < participants.len() {
        maximum = maximum
            .max(select_text_line(&participants[group_start..], false, None).used_inline_extent);
    }
    maximum
}

fn shaped_text_max_content<S: LayoutScalar>(participants: &[ShapedTextParticipantOf<S>]) -> S {
    let mut maximum = S::ZERO;
    let mut group_start = 0;
    for (index, participant) in participants.iter().enumerate() {
        if participant.segment.following_break().kind() != InlineBreakKind::Mandatory {
            continue;
        }
        maximum = maximum.max(
            select_text_line(&participants[group_start..=index], false, None).used_inline_extent,
        );
        group_start = index + 1;
    }
    if group_start < participants.len() {
        maximum = maximum
            .max(select_text_line(&participants[group_start..], false, None).used_inline_extent);
    }
    maximum
}

fn pending_text_inline_extent<S: LayoutScalar>(participants: &[ShapedTextParticipantOf<S>]) -> S {
    let mut at_line_start = true;
    participants.iter().fold(S::ZERO, |extent, participant| {
        if at_line_start && discards_at_start(participant.segment.whitespace_edge()) {
            return extent;
        }
        at_line_start = false;
        extent + participant.segment.inline_extent()
    })
}

fn reordered_text_segment_indices<S: LayoutScalar>(
    segments: &[SelectedTextSegmentOf<S>],
) -> Vec<usize> {
    let mut indices = (0..segments.len()).collect::<Vec<_>>();
    let Some(minimum_odd_level) = segments
        .iter()
        .map(|selected| selected.participant.segment.bidi_level().get())
        .filter(|level| level % 2 == 1)
        .min()
    else {
        return indices;
    };
    let maximum_level = segments
        .iter()
        .map(|selected| selected.participant.segment.bidi_level().get())
        .max()
        .unwrap_or(minimum_odd_level);

    for level in (minimum_odd_level..=maximum_level).rev() {
        let mut start = 0;
        while start < indices.len() {
            while start < indices.len()
                && segments[indices[start]]
                    .participant
                    .segment
                    .bidi_level()
                    .get()
                    < level
            {
                start += 1;
            }
            let mut end = start;
            while end < indices.len()
                && segments[indices[end]]
                    .participant
                    .segment
                    .bidi_level()
                    .get()
                    >= level
            {
                end += 1;
            }
            indices[start..end].reverse();
            start = end;
        }
    }

    indices
}

fn shaped_text_line_offset<S: LayoutScalar>(
    used_inline_extent: S,
    available_inline_extent: S,
    flow_axes: FlowAxes,
    text_align: TextAlign,
) -> S {
    let free_space = (available_inline_extent - used_inline_extent).max(S::ZERO);
    let inline_decreases = flow_axes
        .logical_axis_progression(crate::LogicalAxis::Inline)
        .is_decreasing();
    match text_align {
        TextAlign::Auto => S::ZERO,
        TextAlign::LegacyLeft if inline_decreases => free_space,
        TextAlign::LegacyRight if !inline_decreases => free_space,
        TextAlign::LegacyCenter => free_space / S::from_f64(2.0),
        TextAlign::LegacyLeft | TextAlign::LegacyRight => S::ZERO,
    }
}

#[must_use]
pub(super) fn layout_shaped_text_run<S: LayoutScalar>(
    input: ShapedTextRunInputOf<S>,
) -> ShapedTextRunReportOf<S> {
    let available = match input.available_inline_extent {
        AvailableOf::Definite(value) => value,
        AvailableOf::MinContent => shaped_text_min_content(&input.participants),
        AvailableOf::MaxContent => shaped_text_max_content(&input.participants),
    };
    let wraps = !matches!(input.available_inline_extent, AvailableOf::MaxContent);
    let mut selected_lines = Vec::new();
    let mut line_start = 0;
    let mut scan = 0;
    let mut latest_allowed = None;
    let mut pending_strut = None;

    while scan < input.participants.len() {
        let participant = input.participants[scan];
        let candidate_inline_extent =
            pending_text_inline_extent(&input.participants[line_start..=scan]);
        if wraps && candidate_inline_extent > available {
            if let Some(break_end) = latest_allowed {
                selected_lines.push(select_text_line(
                    &input.participants[line_start..break_end],
                    true,
                    pending_strut.take(),
                ));
                line_start = break_end;
                scan = line_start;
                latest_allowed = None;
                continue;
            }
        }

        scan += 1;
        match participant.segment.following_break().kind() {
            InlineBreakKind::Allowed | InlineBreakKind::AllowedWithReplacement => {
                latest_allowed = Some(scan);
            }
            InlineBreakKind::Mandatory => {
                selected_lines.push(select_text_line(
                    &input.participants[line_start..scan],
                    true,
                    pending_strut.take(),
                ));
                pending_strut = Some(participant.segment.metrics());
                line_start = scan;
                latest_allowed = None;
            }
            InlineBreakKind::Prohibited => {}
        }
    }

    if line_start < input.participants.len() {
        selected_lines.push(select_text_line(
            &input.participants[line_start..],
            false,
            pending_strut.take(),
        ));
    } else if let Some(strut) = pending_strut {
        selected_lines.push(select_text_line(&[], false, Some(strut)));
    }

    let mut block_start = S::ZERO;
    let mut inline_extent = S::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut fragments = Vec::new();
    let mut anchors = Vec::new();
    for (line_index, line) in selected_lines.into_iter().enumerate() {
        let line_baseline = block_start + line.baseline;
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);
        let line_inline_start = shaped_text_line_offset(
            line.used_inline_extent,
            available,
            input.flow_axes,
            input.text_align,
        );
        inline_extent = inline_extent.max(line_inline_start + line.used_inline_extent);
        let visual_order = reordered_text_segment_indices(&line.segments);
        let mut visual_indices = vec![0; line.segments.len()];
        let mut inline_starts = vec![S::ZERO; line.segments.len()];
        let mut inline_start = line_inline_start;
        for (visual_index, source_index) in visual_order.into_iter().enumerate() {
            let selected = line.segments[source_index];
            visual_indices[source_index] = visual_index;
            inline_starts[source_index] = inline_start;
            if !selected.discarded {
                inline_start = inline_start
                    + selected.participant.segment.inline_extent()
                    + selected.replacement_inline_extent.unwrap_or(S::ZERO);
            }
        }
        for (source_index, selected) in line.segments.into_iter().enumerate() {
            let inline_start = inline_starts[source_index];
            anchors.push(ShapedTextAnchorOf {
                source_index: selected.participant.source_index,
                inline_start,
                block_start,
                baseline: line_baseline,
            });
            if selected.discarded {
                continue;
            }
            let metrics = selected.participant.segment.metrics();
            fragments.push(ShapedTextFragmentSourceOf {
                source_index: selected.participant.source_index,
                segment_id: selected.participant.segment.segment_id(),
                inline_start,
                block_start: line_baseline - metrics.baseline(),
                inline_extent: selected.participant.segment.inline_extent(),
                block_extent: metrics.line_extent(),
                baseline: line_baseline,
                line_index,
                visual_index: visual_indices[source_index],
                replacement_inline_extent: selected.replacement_inline_extent,
            });
        }
        block_start = block_start + line.baseline + line.after_baseline;
    }

    ShapedTextRunReportOf {
        inline_extent,
        block_extent: block_start,
        first_baseline,
        last_baseline,
        fragments,
        anchors,
    }
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
