use super::{
    AtomicInlineParticipationOf, AvailableOf, Clear, DefaultScalar, Direction, Edges,
    InlineBoundaryKind, InlineBreakKind, InlineBreakOpportunityOf, InlineMetricsOf,
    InlineSegmentId, InlineWhitespaceEdge, LayoutScalar, ShapedInlineSegmentOf, Size, TextAlign,
    VerticalAlign, WritingMode,
};
#[cfg(test)]
use crate::Point;
use crate::geometry::FlowAxes;
#[cfg(test)]
use crate::geometry::{LogicalPointOf, LogicalSizeOf};

#[cfg(test)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MixedInlineParticipantOf<S: LayoutScalar = DefaultScalar> {
    ShapedText(ShapedTextParticipantOf<S>),
    Atomic {
        item: AtomicInlineBoxParticipant<S>,
        participation: AtomicInlineParticipationOf<S>,
    },
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MixedInlineRunInputOf<S: LayoutScalar = DefaultScalar> {
    pub available_inline_extent: AvailableOf<S>,
    pub flow_axes: FlowAxes,
    pub text_align: TextAlign,
    pub participants: Vec<MixedInlineParticipantOf<S>>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineSourceOf<S: LayoutScalar = DefaultScalar> {
    pub item: AtomicInlineBoxParticipant<S>,
    pub inline_start: S,
    pub block_start: S,
    pub line_index: usize,
    pub visual_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PostLineClearIntent {
    None,
    LineStart,
    LineEnd,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineControlSourceOf<S: LayoutScalar = DefaultScalar> {
    pub kind: InlineParticipantLayoutKind,
    pub source_index: usize,
    pub inline_start: S,
    pub block_start: S,
    pub line_index: usize,
    pub visual_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MixedInlineRunReportOf<S: LayoutScalar = DefaultScalar> {
    pub inline_extent: S,
    pub block_extent: S,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub fragments: Vec<ShapedTextFragmentSourceOf<S>>,
    pub anchors: Vec<ShapedTextAnchorOf<S>>,
    pub atomics: Vec<AtomicInlineSourceOf<S>>,
    pub controls: Vec<InlineControlSourceOf<S>>,
    pub post_line_clear_intents: Vec<PostLineClearIntent>,
}

#[derive(Clone, Copy)]
struct SelectedInlineUnitOf<S: LayoutScalar> {
    participant: MixedInlineParticipantOf<S>,
    discarded: bool,
    replacement_inline_extent: Option<S>,
}

#[derive(Clone)]
struct SelectedInlineLineOf<S: LayoutScalar> {
    units: Vec<SelectedInlineUnitOf<S>>,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    post_line_clear_intent: PostLineClearIntent,
    baseline: S,
    after_baseline: S,
    used_inline_extent: S,
}

#[derive(Clone, Copy)]
struct InlineMetricContributionOf<S: LayoutScalar> {
    baseline: S,
    after_baseline: S,
}

#[derive(Clone, Copy)]
struct InlineLineMetricGroupsOf<S: LayoutScalar> {
    baseline: S,
    after_baseline: S,
    line_over_extent: S,
    line_under_extent: S,
}

impl<S: LayoutScalar> InlineLineMetricGroupsOf<S> {
    fn from_strut(strut: Option<InlineMetricContributionOf<S>>) -> Self {
        Self {
            baseline: strut.map_or(S::ZERO, |metrics| metrics.baseline),
            after_baseline: strut.map_or(S::ZERO, |metrics| metrics.after_baseline),
            line_over_extent: S::ZERO,
            line_under_extent: S::ZERO,
        }
    }

    fn include(
        &mut self,
        alignment: InlineControlAlignment,
        metrics: InlineMetricContributionOf<S>,
    ) {
        match alignment {
            InlineControlAlignment::Baseline => {
                self.baseline = self.baseline.max(metrics.baseline);
                self.after_baseline = self.after_baseline.max(metrics.after_baseline);
            }
            InlineControlAlignment::Top => {
                self.line_over_extent = self
                    .line_over_extent
                    .max(metrics.baseline + metrics.after_baseline);
            }
            InlineControlAlignment::Bottom => {
                self.line_under_extent = self
                    .line_under_extent
                    .max(metrics.baseline + metrics.after_baseline);
            }
        }
    }

    fn resolve(self) -> InlineMetricContributionOf<S> {
        let line_under = (self.baseline + self.after_baseline).max(self.line_over_extent);
        let line_over = (line_under - self.line_under_extent).min(S::ZERO);
        InlineMetricContributionOf {
            baseline: self.baseline - line_over,
            after_baseline: line_under - self.baseline,
        }
    }
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

impl<S: LayoutScalar> MixedInlineParticipantOf<S> {
    fn following_break(self) -> Option<InlineBreakOpportunityOf<S>> {
        match self {
            Self::ShapedText(participant) => Some(participant.segment.following_break()),
            Self::Atomic { participation, .. } => Some(participation.following_break()),
            Self::ForcedLineBreak(_) | Self::Boundary(_) => None,
        }
    }

    fn bidi_level(self) -> u8 {
        match self {
            Self::ShapedText(participant) => participant.segment.bidi_level().get(),
            Self::Atomic { participation, .. } => participation.bidi_level().get(),
            Self::Boundary(control) => match control.flow().direction() {
                Direction::Ltr => 0,
                Direction::Rtl => 1,
            },
            Self::ForcedLineBreak(_) => 0,
        }
    }

    fn whitespace_edge(self) -> Option<InlineWhitespaceEdge> {
        match self {
            Self::ShapedText(participant) => Some(participant.segment.whitespace_edge()),
            Self::Atomic { .. } | Self::ForcedLineBreak(_) | Self::Boundary(_) => None,
        }
    }

    fn inline_advance(self, flow_axes: FlowAxes) -> S {
        match self {
            Self::ShapedText(participant) => participant.segment.inline_extent(),
            Self::Atomic { item, .. } => {
                let margin = flow_axes.logical_edges(item.margin);
                flow_axes.logical_size(item.size).inline + margin.inline_sum()
            }
            Self::ForcedLineBreak(_) | Self::Boundary(_) => S::ZERO,
        }
    }

    fn metrics(self, flow_axes: FlowAxes) -> InlineMetricContributionOf<S> {
        match self {
            Self::ShapedText(participant) => {
                let metrics = participant.segment.metrics();
                InlineMetricContributionOf {
                    baseline: metrics.baseline(),
                    after_baseline: metrics.after_baseline(),
                }
            }
            Self::Atomic { item, .. } => {
                let logical_size = flow_axes.logical_size(item.size);
                let logical_margin = flow_axes.logical_edges(item.margin);
                let baseline = item.first_baseline.unwrap_or(logical_size.block);
                InlineMetricContributionOf {
                    baseline: logical_margin.block_start + baseline,
                    after_baseline: logical_size.block - baseline + logical_margin.block_end,
                }
            }
            Self::ForcedLineBreak(control) => {
                let metrics = control.metrics();
                InlineMetricContributionOf {
                    baseline: metrics.baseline(),
                    after_baseline: metrics.after_baseline(),
                }
            }
            Self::Boundary(control) => {
                let metrics = control.metrics();
                InlineMetricContributionOf {
                    baseline: metrics.baseline(),
                    after_baseline: metrics.after_baseline(),
                }
            }
        }
    }

    fn alignment(self) -> InlineControlAlignment {
        match self {
            Self::ShapedText(_) => InlineControlAlignment::Baseline,
            Self::Atomic { item, .. } => item.alignment,
            Self::ForcedLineBreak(control) => control.alignment(),
            Self::Boundary(control) => control.alignment(),
        }
    }
}

#[must_use]
pub(super) const fn mapped_post_line_clear_intent(
    flow_axes: FlowAxes,
    clear: Clear,
) -> PostLineClearIntent {
    let _ = flow_axes.inline_start();
    match clear {
        Clear::None => PostLineClearIntent::None,
        Clear::Left => PostLineClearIntent::LineStart,
        Clear::Right => PostLineClearIntent::LineEnd,
        Clear::Both => PostLineClearIntent::Both,
    }
}

fn select_inline_line<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    line_break: Option<ForcedLineBreakControlOf<S>>,
    selected_break: bool,
    strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> SelectedInlineLineOf<S> {
    let mut discarded = vec![false; participants.len()];
    for (index, participant) in participants.iter().enumerate() {
        if participant.whitespace_edge().is_some_and(discards_at_start) {
            discarded[index] = true;
        } else {
            break;
        }
    }
    for (index, participant) in participants.iter().enumerate().rev() {
        if participant.whitespace_edge().is_some_and(discards_at_end) {
            discarded[index] = true;
        } else {
            break;
        }
    }

    let mut metric_groups = InlineLineMetricGroupsOf::from_strut(strut);
    if let Some(control) = line_break {
        metric_groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    let mut used_inline_extent = S::ZERO;
    let selected_replacement = selected_break
        .then(|| {
            participants
                .last()
                .and_then(|participant| participant.following_break())
                .and_then(InlineBreakOpportunityOf::replacement_inline_extent)
        })
        .flatten();
    let units = participants
        .iter()
        .copied()
        .enumerate()
        .map(|(index, participant)| {
            if !discarded[index] {
                metric_groups.include(participant.alignment(), participant.metrics(flow_axes));
                used_inline_extent = used_inline_extent + participant.inline_advance(flow_axes);
            }
            let replacement_inline_extent = (index + 1 == participants.len())
                .then_some(selected_replacement)
                .flatten();
            if let Some(replacement) = replacement_inline_extent {
                used_inline_extent = used_inline_extent + replacement;
            }
            SelectedInlineUnitOf {
                participant,
                discarded: discarded[index],
                replacement_inline_extent,
            }
        })
        .collect();
    let metrics = metric_groups.resolve();

    SelectedInlineLineOf {
        units,
        line_break,
        post_line_clear_intent: line_break.map_or(PostLineClearIntent::None, |control| {
            mapped_post_line_clear_intent(flow_axes, control.clear())
        }),
        baseline: metrics.baseline,
        after_baseline: metrics.after_baseline,
        used_inline_extent,
    }
}

fn inline_min_content<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    flow_axes: FlowAxes,
) -> S {
    let mut maximum = S::ZERO;
    let mut group_start = 0;
    for (index, participant) in participants.iter().enumerate() {
        if matches!(participant, MixedInlineParticipantOf::ForcedLineBreak(_)) {
            maximum = maximum.max(
                select_inline_line(
                    &participants[group_start..index],
                    None,
                    false,
                    None,
                    flow_axes,
                )
                .used_inline_extent,
            );
            group_start = index + 1;
            continue;
        }
        let Some(following_break) = participant.following_break() else {
            continue;
        };
        if following_break.kind() == InlineBreakKind::Prohibited {
            continue;
        }
        let line = select_inline_line(
            &participants[group_start..=index],
            None,
            true,
            None,
            flow_axes,
        );
        maximum = maximum.max(line.used_inline_extent);
        group_start = index + 1;
    }
    if group_start < participants.len() {
        maximum = maximum.max(
            select_inline_line(&participants[group_start..], None, false, None, flow_axes)
                .used_inline_extent,
        );
    }
    maximum
}

fn inline_max_content<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    flow_axes: FlowAxes,
) -> S {
    let mut maximum = S::ZERO;
    let mut group_start = 0;
    for (index, participant) in participants.iter().enumerate() {
        if matches!(participant, MixedInlineParticipantOf::ForcedLineBreak(_)) {
            maximum = maximum.max(
                select_inline_line(
                    &participants[group_start..index],
                    None,
                    false,
                    None,
                    flow_axes,
                )
                .used_inline_extent,
            );
            group_start = index + 1;
            continue;
        }
        if participant
            .following_break()
            .map(InlineBreakOpportunityOf::kind)
            != Some(InlineBreakKind::Mandatory)
        {
            continue;
        }
        maximum = maximum.max(
            select_inline_line(
                &participants[group_start..=index],
                None,
                false,
                None,
                flow_axes,
            )
            .used_inline_extent,
        );
        group_start = index + 1;
    }
    if group_start < participants.len() {
        maximum = maximum.max(
            select_inline_line(&participants[group_start..], None, false, None, flow_axes)
                .used_inline_extent,
        );
    }
    maximum
}

fn pending_inline_extent<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    flow_axes: FlowAxes,
) -> S {
    let mut at_line_start = true;
    participants.iter().fold(S::ZERO, |extent, participant| {
        if at_line_start && participant.whitespace_edge().is_some_and(discards_at_start) {
            return extent;
        }
        at_line_start = false;
        extent + participant.inline_advance(flow_axes)
    })
}

fn reordered_inline_unit_indices<S: LayoutScalar>(units: &[SelectedInlineUnitOf<S>]) -> Vec<usize> {
    let mut indices = (0..units.len()).collect::<Vec<_>>();
    let Some(minimum_odd_level) = units
        .iter()
        .map(|selected| selected.participant.bidi_level())
        .filter(|level| level % 2 == 1)
        .min()
    else {
        return indices;
    };
    let maximum_level = units
        .iter()
        .map(|selected| selected.participant.bidi_level())
        .max()
        .unwrap_or(minimum_odd_level);

    for level in (minimum_odd_level..=maximum_level).rev() {
        let mut start = 0;
        while start < indices.len() {
            while start < indices.len() && units[indices[start]].participant.bidi_level() < level {
                start += 1;
            }
            let mut end = start;
            while end < indices.len() && units[indices[end]].participant.bidi_level() >= level {
                end += 1;
            }
            indices[start..end].reverse();
            start = end;
        }
    }

    indices
}

fn text_line_offset<S: LayoutScalar>(
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
pub(super) fn layout_mixed_inline_run<S: LayoutScalar>(
    input: MixedInlineRunInputOf<S>,
) -> MixedInlineRunReportOf<S> {
    let available = match input.available_inline_extent {
        AvailableOf::Definite(value) => value,
        AvailableOf::MinContent => inline_min_content(&input.participants, input.flow_axes),
        AvailableOf::MaxContent => inline_max_content(&input.participants, input.flow_axes),
    };
    let wraps = !matches!(input.available_inline_extent, AvailableOf::MaxContent);
    let mut selected_lines = Vec::new();
    let mut line_start = 0;
    let mut scan = 0;
    let mut latest_allowed = None;
    let mut pending_strut = None;

    while scan < input.participants.len() {
        let participant = input.participants[scan];
        if let MixedInlineParticipantOf::ForcedLineBreak(control) = participant {
            selected_lines.push(select_inline_line(
                &input.participants[line_start..scan],
                Some(control),
                false,
                pending_strut.take(),
                input.flow_axes,
            ));
            pending_strut = Some(participant.metrics(input.flow_axes));
            scan += 1;
            line_start = scan;
            latest_allowed = None;
            continue;
        }
        let candidate_inline_extent =
            pending_inline_extent(&input.participants[line_start..=scan], input.flow_axes);
        if wraps
            && candidate_inline_extent > available
            && let Some(break_end) = latest_allowed
        {
            selected_lines.push(select_inline_line(
                &input.participants[line_start..break_end],
                None,
                true,
                pending_strut.take(),
                input.flow_axes,
            ));
            line_start = break_end;
            scan = line_start;
            latest_allowed = None;
            continue;
        }

        scan += 1;
        let Some(following_break) = participant.following_break() else {
            continue;
        };
        match following_break.kind() {
            InlineBreakKind::Allowed | InlineBreakKind::AllowedWithReplacement => {
                latest_allowed = Some(scan);
            }
            InlineBreakKind::Mandatory => {
                selected_lines.push(select_inline_line(
                    &input.participants[line_start..scan],
                    None,
                    true,
                    pending_strut.take(),
                    input.flow_axes,
                ));
                pending_strut = Some(participant.metrics(input.flow_axes));
                line_start = scan;
                latest_allowed = None;
            }
            InlineBreakKind::Prohibited => {}
        }
    }

    if line_start < input.participants.len() {
        selected_lines.push(select_inline_line(
            &input.participants[line_start..],
            None,
            false,
            pending_strut.take(),
            input.flow_axes,
        ));
    } else if let Some(strut) = pending_strut {
        selected_lines.push(select_inline_line(
            &[],
            None,
            false,
            Some(strut),
            input.flow_axes,
        ));
    }

    let mut block_start = S::ZERO;
    let mut inline_extent = S::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut fragments = Vec::new();
    let mut anchors = Vec::new();
    let mut atomics = Vec::new();
    let mut controls = Vec::new();
    let mut post_line_clear_intents = Vec::new();
    for (line_index, line) in selected_lines.into_iter().enumerate() {
        let line_baseline = block_start + line.baseline;
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);
        let line_inline_start = text_line_offset(
            line.used_inline_extent,
            available,
            input.flow_axes,
            input.text_align,
        );
        inline_extent = inline_extent.max(line_inline_start + line.used_inline_extent);
        post_line_clear_intents.push(line.post_line_clear_intent);
        let visual_order = reordered_inline_unit_indices(&line.units);
        let mut visual_indices = vec![0; line.units.len()];
        let mut inline_starts = vec![S::ZERO; line.units.len()];
        let mut inline_start = line_inline_start;
        for (visual_index, source_index) in visual_order.into_iter().enumerate() {
            let selected = line.units[source_index];
            visual_indices[source_index] = visual_index;
            inline_starts[source_index] = inline_start;
            if !selected.discarded {
                inline_start = inline_start
                    + selected.participant.inline_advance(input.flow_axes)
                    + selected.replacement_inline_extent.unwrap_or(S::ZERO);
            }
        }
        for (source_index, selected) in line.units.into_iter().enumerate() {
            let inline_start = inline_starts[source_index];
            match selected.participant {
                MixedInlineParticipantOf::ShapedText(participant) => {
                    anchors.push(ShapedTextAnchorOf {
                        source_index: participant.source_index,
                        inline_start,
                        block_start,
                        baseline: line_baseline,
                    });
                    if selected.discarded {
                        continue;
                    }
                    let metrics = participant.segment.metrics();
                    fragments.push(ShapedTextFragmentSourceOf {
                        source_index: participant.source_index,
                        segment_id: participant.segment.segment_id(),
                        inline_start,
                        block_start: line_baseline - metrics.baseline(),
                        inline_extent: participant.segment.inline_extent(),
                        block_extent: metrics.line_extent(),
                        baseline: line_baseline,
                        line_index,
                        visual_index: visual_indices[source_index],
                        replacement_inline_extent: selected.replacement_inline_extent,
                    });
                }
                MixedInlineParticipantOf::Atomic { item, .. } => {
                    let logical_margin = input.flow_axes.logical_edges(item.margin);
                    let logical_size = input.flow_axes.logical_size(item.size);
                    let line_block_extent = line.baseline + line.after_baseline;
                    let baseline = item.first_baseline.unwrap_or(logical_size.block);
                    let item_block_start = match item.alignment {
                        InlineControlAlignment::Baseline => line_baseline - baseline,
                        InlineControlAlignment::Top => block_start - logical_margin.block_start,
                        InlineControlAlignment::Bottom => {
                            block_start + line_block_extent
                                - logical_margin.block_end
                                - logical_size.block
                        }
                    };
                    atomics.push(AtomicInlineSourceOf {
                        item,
                        inline_start: inline_start + logical_margin.inline_start,
                        block_start: item_block_start,
                        line_index,
                        visual_index: visual_indices[source_index],
                    });
                }
                MixedInlineParticipantOf::Boundary(control) => {
                    let control_block = control_block_position(
                        control.alignment(),
                        control.metrics(),
                        block_start,
                        line_baseline,
                        line.baseline + line.after_baseline,
                    );
                    controls.push(InlineControlSourceOf {
                        kind: inline_boundary_layout_kind(control.kind()),
                        source_index: control.source_index(),
                        inline_start,
                        block_start: control_block,
                        line_index,
                        visual_index: Some(visual_indices[source_index]),
                    });
                }
                MixedInlineParticipantOf::ForcedLineBreak(_) => {
                    unreachable!("visible line breaks commit before line reordering")
                }
            }
        }
        if let Some(control) = line.line_break {
            let control_block = control_block_position(
                control.alignment(),
                control.metrics(),
                block_start,
                line_baseline,
                line.baseline + line.after_baseline,
            );
            controls.push(InlineControlSourceOf {
                kind: InlineParticipantLayoutKind::ForcedLineBreak,
                source_index: control.source_index(),
                inline_start: line_inline_start + line.used_inline_extent,
                block_start: control_block,
                line_index,
                visual_index: None,
            });
        }
        block_start = block_start + line.baseline + line.after_baseline;
    }

    MixedInlineRunReportOf {
        inline_extent,
        block_extent: block_start,
        first_baseline,
        last_baseline,
        fragments,
        anchors,
        atomics,
        controls,
        post_line_clear_intents,
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
    pub alignment: InlineControlAlignment,
}

fn control_block_position<S: LayoutScalar>(
    alignment: InlineControlAlignment,
    metrics: InlineMetricsOf<S>,
    line_block_start: S,
    line_baseline: S,
    line_block_extent: S,
) -> S {
    match alignment {
        InlineControlAlignment::Baseline => line_baseline,
        InlineControlAlignment::Top => line_block_start + metrics.baseline(),
        InlineControlAlignment::Bottom => {
            line_block_start + line_block_extent - metrics.after_baseline()
        }
    }
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

    #[cfg(test)]
    #[must_use]
    pub(super) const fn writing_mode(self) -> WritingMode {
        self.flow_axes.writing_mode()
    }

    #[must_use]
    pub(super) const fn direction(self) -> Direction {
        self.flow_axes.direction()
    }

    #[cfg(test)]
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

    #[cfg(test)]
    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }

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

    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }
}

#[cfg(test)]
impl<S: LayoutScalar> InlineParticipant<S> {
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
            alignment: InlineControlAlignment::Baseline,
        })
    }

    #[must_use]
    pub(super) const fn forced_line_break(control: ForcedLineBreakControlOf<S>) -> Self {
        Self::ForcedLineBreak(control)
    }

    #[must_use]
    pub(super) const fn inline_boundary(control: InlineBoundaryControlOf<S>) -> Self {
        Self::Boundary(control)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineParticipant<S: LayoutScalar = DefaultScalar> {
    Box(AtomicInlineBoxParticipant<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InlineParticipantLayoutKind {
    #[cfg(test)]
    Box,
    ForcedLineBreak,
    InlineBoundaryStart,
    InlineBoundaryEnd,
}

#[cfg(test)]
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

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct InlineRunReport<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub items: Vec<InlineParticipantLayoutItem<S>>,
}

fn inline_boundary_layout_kind(kind: InlineBoundaryKind) -> InlineParticipantLayoutKind {
    match kind {
        InlineBoundaryKind::Start => InlineParticipantLayoutKind::InlineBoundaryStart,
        InlineBoundaryKind::End => InlineParticipantLayoutKind::InlineBoundaryEnd,
    }
}

#[cfg(test)]
#[must_use]
pub(super) fn layout_inline_run<S: LayoutScalar>(input: InlineRunInput<S>) -> InlineRunReport<S> {
    let flow_axes = FlowAxes::new(input.writing_mode, input.direction);
    let participants = input
        .items
        .into_iter()
        .map(|participant| match participant {
            InlineParticipant::Box(item) => MixedInlineParticipantOf::Atomic {
                item,
                participation: AtomicInlineParticipationOf::try_new(
                    super::BidiLevel::try_new(0).expect("base bidi level is valid"),
                    InlineBreakOpportunityOf::allowed(),
                )
                .expect("allowed atomic participation is valid"),
            },
            InlineParticipant::ForcedLineBreak(control) => {
                MixedInlineParticipantOf::ForcedLineBreak(control)
            }
            InlineParticipant::Boundary(control) => MixedInlineParticipantOf::Boundary(control),
        })
        .collect();
    let report = layout_mixed_inline_run(MixedInlineRunInputOf {
        available_inline_extent: input.available_width,
        flow_axes,
        text_align: TextAlign::Auto,
        participants,
    });
    let logical_report_size = LogicalSizeOf::new(report.inline_extent, report.block_extent);
    let report_size = flow_axes.physical_size(logical_report_size);
    let mut items = report
        .atomics
        .into_iter()
        .map(|source| {
            let logical_size = flow_axes.logical_size(source.item.size);
            InlineParticipantLayoutItem {
                kind: InlineParticipantLayoutKind::Box,
                source_index: source.item.source_index,
                location: flow_axes.physical_point(
                    LogicalPointOf::new(source.inline_start, source.block_start),
                    logical_size,
                    report_size,
                ),
                size: source.item.size,
                content_size: source.item.content_size,
                margin: source.item.margin,
                padding: source.item.padding,
                border: source.item.border,
                scrollbar_size: source.item.scrollbar_size,
            }
        })
        .chain(
            report
                .controls
                .into_iter()
                .map(|source| InlineParticipantLayoutItem {
                    kind: source.kind,
                    source_index: source.source_index,
                    location: flow_axes.physical_point(
                        LogicalPointOf::new(source.inline_start, source.block_start),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                        report_size,
                    ),
                    size: Size::ZERO,
                    content_size: Size::ZERO,
                    margin: Edges::ZERO,
                    padding: Edges::ZERO,
                    border: Edges::ZERO,
                    scrollbar_size: Size::ZERO,
                }),
        )
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.source_index);

    InlineRunReport {
        size: report_size,
        content_size: report_size,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
        items,
    }
}

#[cfg(test)]
#[must_use]
pub(super) fn inline_run_min_content_width<S: LayoutScalar>(items: &[InlineParticipant<S>]) -> S {
    layout_inline_run(InlineRunInput {
        available_width: AvailableOf::MIN_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: items.to_vec(),
    })
    .size
    .width
}

#[must_use]
#[cfg(test)]
pub(super) fn inline_run_max_content_width<S: LayoutScalar>(items: &[InlineParticipant<S>]) -> S {
    layout_inline_run(InlineRunInput {
        available_width: AvailableOf::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: items.to_vec(),
    })
    .size
    .width
}
