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
use std::cell::Cell;

mod input;

pub(crate) use input::{InlineParticipantKindOf, InlineParticipantProjection};

#[cfg(test)]
thread_local! {
    static INLINE_CANDIDATE_SCAN_VISITS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn observe_inline_candidate_scan_visit() {
    INLINE_CANDIDATE_SCAN_VISITS.with(|visits| visits.set(visits.get() + 1));
}

#[cfg(test)]
pub(super) fn reset_inline_candidate_scan_visits() {
    INLINE_CANDIDATE_SCAN_VISITS.with(|visits| visits.set(0));
}

#[cfg(test)]
pub(super) fn inline_candidate_scan_visits() -> usize {
    INLINE_CANDIDATE_SCAN_VISITS.with(Cell::get)
}

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
pub(super) struct LogicalLineBandQueryResultOf<S: LayoutScalar = DefaultScalar> {
    pub inline_start: S,
    pub inline_end: S,
    pub next_transition: Option<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LogicalLineBandOf<S: LayoutScalar = DefaultScalar> {
    pub inline_start: S,
    pub inline_end: S,
    pub block_start: S,
    pub block_end: S,
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
    pub float_edge_phase: Option<S>,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub fragments: Vec<ShapedTextFragmentSourceOf<S>>,
    pub anchors: Vec<ShapedTextAnchorOf<S>>,
    pub atomics: Vec<AtomicInlineSourceOf<S>>,
    pub controls: Vec<InlineControlSourceOf<S>>,
    pub post_line_clear_intents: Vec<PostLineClearIntent>,
    pub line_bands: Vec<LogicalLineBandOf<S>>,
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
    fallback_line_band: Option<InlineMetricContributionOf<S>>,
    used_inline_extent: S,
    split_baseline_edge_phase: Option<S>,
    inline_start_override: Option<S>,
    uses_float_strut_phase: bool,
    band: LogicalLineBandOf<S>,
}

#[derive(Clone, Copy)]
struct InlineLineSummaryOf<S: LayoutScalar> {
    discarded_start_end: usize,
    discarded_end_start: usize,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    post_line_clear_intent: PostLineClearIntent,
    baseline: S,
    after_baseline: S,
    fallback_line_band: Option<InlineMetricContributionOf<S>>,
    used_inline_extent: S,
    split_baseline_edge_phase: Option<S>,
    selected_terminal_replacement: Option<S>,
}

impl<S: LayoutScalar> InlineLineSummaryOf<S> {
    fn is_discarded(self, index: usize) -> bool {
        index < self.discarded_start_end || index >= self.discarded_end_start
    }
}

#[derive(Clone)]
struct SelectedInlineLineCandidateOf<S: LayoutScalar> {
    line: SelectedInlineLineOf<S>,
    next_source_cursor: usize,
    pending_strut: Option<InlineMetricContributionOf<S>>,
}

#[derive(Clone, Copy)]
struct InlineMetricContributionOf<S: LayoutScalar> {
    baseline: S,
    after_baseline: S,
}

impl<S: LayoutScalar> InlineMetricContributionOf<S> {
    fn extent(self) -> S {
        self.baseline + self.after_baseline
    }
}

#[derive(Clone, Copy)]
struct InlineLineMetricGroupsOf<S: LayoutScalar> {
    baseline: S,
    after_baseline: S,
    zero_after_baseline_peak: S,
    has_positive_after_baseline: bool,
    line_over_extent: S,
    line_under_extent: S,
}

impl<S: LayoutScalar> InlineLineMetricGroupsOf<S> {
    fn from_strut(strut: Option<InlineMetricContributionOf<S>>) -> Self {
        Self {
            baseline: strut.map_or(S::ZERO, |metrics| metrics.baseline),
            after_baseline: strut.map_or(S::ZERO, |metrics| metrics.after_baseline),
            zero_after_baseline_peak: strut
                .filter(|metrics| metrics.after_baseline == S::ZERO)
                .map_or(S::ZERO, |metrics| metrics.baseline),
            has_positive_after_baseline: strut
                .is_some_and(|metrics| metrics.after_baseline > S::ZERO),
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
                if metrics.after_baseline == S::ZERO {
                    self.zero_after_baseline_peak =
                        self.zero_after_baseline_peak.max(metrics.baseline);
                } else {
                    self.has_positive_after_baseline = true;
                }
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

    fn split_baseline_edge_phase(self) -> Option<S> {
        let phase_span = self.baseline + self.zero_after_baseline_peak;
        (self.has_positive_after_baseline
            && self.zero_after_baseline_peak > S::ZERO
            && phase_span > S::ZERO)
            .then(|| self.zero_after_baseline_peak / phase_span)
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
            Self::Atomic { item, .. } => item.metrics(flow_axes),
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

fn resolve_completed_fallback_envelope<S: LayoutScalar>(
    line: &mut SelectedInlineLineOf<S>,
    flow_axes: FlowAxes,
    uses_float_band: bool,
) {
    let Some(fallback_band) = line.fallback_line_band else {
        return;
    };
    let has_fallback_atomic = line.units.iter().copied().any(|unit| {
        !unit.discarded
            && matches!(
                unit.participant,
                MixedInlineParticipantOf::Atomic { item, .. }
                    if item
                        .fallback_block_start_in_band(flow_axes, fallback_band)
                        .is_some()
            )
    });
    if !has_fallback_atomic {
        return;
    }

    if uses_float_band {
        line.baseline = fallback_band.baseline;
        line.after_baseline = fallback_band.after_baseline;
        return;
    }

    line.fallback_line_band = None;
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

fn summarize_inline_line<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    line_break: Option<ForcedLineBreakControlOf<S>>,
    selected_break: bool,
    strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> InlineLineSummaryOf<S> {
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
    let fallback_line_band =
        (strut.is_some() || line_break.is_some()).then(|| metric_groups.resolve());
    let mut used_inline_extent = S::ZERO;
    let selected_replacement = selected_break
        .then(|| {
            participants
                .last()
                .and_then(|participant| participant.following_break())
                .and_then(InlineBreakOpportunityOf::replacement_inline_extent)
        })
        .flatten();
    let mut discarded_start_end = 0;
    let mut discarded_end_start = participants.len();
    let mut at_line_start = true;
    let mut before_trailing_metric_groups = metric_groups;
    let mut before_trailing_inline_extent = used_inline_extent;
    for (index, participant) in participants.iter().copied().enumerate() {
        if participant.whitespace_edge().is_some_and(discards_at_end) {
            if discarded_end_start == participants.len() {
                discarded_end_start = index;
                before_trailing_metric_groups = metric_groups;
                before_trailing_inline_extent = used_inline_extent;
            }
        } else {
            discarded_end_start = participants.len();
        }

        if at_line_start && participant.whitespace_edge().is_some_and(discards_at_start) {
            discarded_start_end = index + 1;
        } else {
            at_line_start = false;
            metric_groups.include(participant.alignment(), participant.metrics(flow_axes));
            used_inline_extent = used_inline_extent + participant.inline_advance(flow_axes);
        }
        if index + 1 == participants.len()
            && let Some(replacement) = selected_replacement
        {
            used_inline_extent = used_inline_extent + replacement;
        }
    }
    if discarded_end_start < participants.len() {
        metric_groups = before_trailing_metric_groups;
        used_inline_extent = before_trailing_inline_extent;
        if let Some(replacement) = selected_replacement {
            used_inline_extent = used_inline_extent + replacement;
        }
    }
    let metrics = metric_groups.resolve();

    InlineLineSummaryOf {
        discarded_start_end,
        discarded_end_start,
        line_break,
        post_line_clear_intent: line_break.map_or(PostLineClearIntent::None, |control| {
            mapped_post_line_clear_intent(flow_axes, control.clear())
        }),
        baseline: metrics.baseline,
        after_baseline: metrics.after_baseline,
        fallback_line_band,
        used_inline_extent,
        split_baseline_edge_phase: metric_groups.split_baseline_edge_phase(),
        selected_terminal_replacement: selected_replacement,
    }
}

fn select_inline_line<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    line_break: Option<ForcedLineBreakControlOf<S>>,
    selected_break: bool,
    strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> SelectedInlineLineOf<S> {
    let summary = summarize_inline_line(participants, line_break, selected_break, strut, flow_axes);
    let units = participants
        .iter()
        .copied()
        .enumerate()
        .map(|(index, participant)| SelectedInlineUnitOf {
            participant,
            discarded: summary.is_discarded(index),
            replacement_inline_extent: (index + 1 == participants.len())
                .then_some(summary.selected_terminal_replacement)
                .flatten(),
        })
        .collect();

    SelectedInlineLineOf {
        units,
        line_break: summary.line_break,
        post_line_clear_intent: summary.post_line_clear_intent,
        baseline: summary.baseline,
        after_baseline: summary.after_baseline,
        fallback_line_band: summary.fallback_line_band,
        used_inline_extent: summary.used_inline_extent,
        split_baseline_edge_phase: summary.split_baseline_edge_phase,
        inline_start_override: None,
        uses_float_strut_phase: false,
        band: LogicalLineBandOf {
            inline_start: S::ZERO,
            inline_end: S::ZERO,
            block_start: S::ZERO,
            block_end: S::ZERO,
        },
    }
}

fn resolve_float_strut_phase<S: LayoutScalar>(
    line: &mut SelectedInlineLineOf<S>,
    carried_strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> Option<InlineMetricContributionOf<S>> {
    let mut metric_groups = InlineLineMetricGroupsOf::from_strut(None);
    let mut active_strut = carried_strut;
    let mut uses_phase = carried_strut.is_some();
    if let Some(control) = line.line_break {
        metric_groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    for selected in line.units.iter().copied().filter(|unit| !unit.discarded) {
        match selected.participant {
            MixedInlineParticipantOf::Boundary(control) => {
                uses_phase = true;
                active_strut = match control.kind() {
                    InlineBoundaryKind::Start => Some(InlineMetricContributionOf {
                        baseline: control.metrics().baseline(),
                        after_baseline: control.metrics().after_baseline(),
                    }),
                    InlineBoundaryKind::End => None,
                };
            }
            participant => {
                metric_groups.include(participant.alignment(), participant.metrics(flow_axes));
            }
        }
    }
    let metrics = metric_groups.resolve();
    let line_extent = active_strut.map_or(metrics.extent(), |strut| {
        metrics.extent().max(strut.extent())
    });
    line.baseline = metrics.baseline;
    line.after_baseline = line_extent - metrics.baseline;
    line.fallback_line_band = None;
    line.uses_float_strut_phase = uses_phase;
    active_strut
}

fn starts_inline_strut_phase<S: LayoutScalar>(line: &SelectedInlineLineOf<S>) -> bool {
    line.units.iter().any(|selected| {
        matches!(
            selected.participant,
            MixedInlineParticipantOf::Boundary(control)
                if control.kind() == InlineBoundaryKind::Start
        )
    })
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
                summarize_inline_line(
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
        let line = summarize_inline_line(
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
            summarize_inline_line(&participants[group_start..], None, false, None, flow_axes)
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
                summarize_inline_line(
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
            summarize_inline_line(
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
            summarize_inline_line(&participants[group_start..], None, false, None, flow_axes)
                .used_inline_extent,
        );
    }
    maximum
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

fn resolved_inline_available<S: LayoutScalar>(input: &MixedInlineRunInputOf<S>) -> S {
    match input.available_inline_extent {
        AvailableOf::Definite(value) => value,
        AvailableOf::MinContent => inline_min_content(&input.participants, input.flow_axes),
        AvailableOf::MaxContent => inline_max_content(&input.participants, input.flow_axes),
    }
}

fn select_next_inline_line<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    source_cursor: usize,
    available_inline_extent: S,
    wraps: bool,
    pending_strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> Option<SelectedInlineLineCandidateOf<S>> {
    if source_cursor == participants.len() {
        return pending_strut.map(|strut| SelectedInlineLineCandidateOf {
            line: select_inline_line(&[], None, false, Some(strut), flow_axes),
            next_source_cursor: source_cursor,
            pending_strut: None,
        });
    }

    let mut scan = source_cursor;
    let mut latest_allowed = None;
    let mut candidate_inline_extent = S::ZERO;
    let mut at_line_start = true;
    while scan < participants.len() {
        let participant = participants[scan];
        if let MixedInlineParticipantOf::ForcedLineBreak(control) = participant {
            return Some(SelectedInlineLineCandidateOf {
                line: select_inline_line(
                    &participants[source_cursor..scan],
                    Some(control),
                    false,
                    pending_strut,
                    flow_axes,
                ),
                next_source_cursor: scan + 1,
                pending_strut: Some(participant.metrics(flow_axes)),
            });
        }

        #[cfg(test)]
        observe_inline_candidate_scan_visit();
        if !(at_line_start && participant.whitespace_edge().is_some_and(discards_at_start)) {
            at_line_start = false;
            candidate_inline_extent =
                candidate_inline_extent + participant.inline_advance(flow_axes);
        }
        if wraps
            && candidate_inline_extent > available_inline_extent
            && let Some(break_end) = latest_allowed
        {
            return Some(SelectedInlineLineCandidateOf {
                line: select_inline_line(
                    &participants[source_cursor..break_end],
                    None,
                    true,
                    pending_strut,
                    flow_axes,
                ),
                next_source_cursor: break_end,
                pending_strut: None,
            });
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
                return Some(SelectedInlineLineCandidateOf {
                    line: select_inline_line(
                        &participants[source_cursor..scan],
                        None,
                        true,
                        pending_strut,
                        flow_axes,
                    ),
                    next_source_cursor: scan,
                    pending_strut: Some(participant.metrics(flow_axes)),
                });
            }
            InlineBreakKind::Prohibited => {}
        }
    }

    Some(SelectedInlineLineCandidateOf {
        line: select_inline_line(
            &participants[source_cursor..],
            None,
            false,
            pending_strut,
            flow_axes,
        ),
        next_source_cursor: participants.len(),
        pending_strut: None,
    })
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

fn selected_line_inline_start<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    flow_axes: FlowAxes,
    text_align: TextAlign,
) -> S {
    line.inline_start_override.unwrap_or_else(|| {
        line.band.inline_start
            + text_line_offset(
                line.used_inline_extent,
                line.band.inline_end - line.band.inline_start,
                flow_axes,
                text_align,
            )
    })
}

fn resolved_float_terminal_block_extent<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    carried_strut: Option<InlineMetricContributionOf<S>>,
    float_transition: Option<S>,
    flow_axes: FlowAxes,
    line_inline_start: S,
    line_inline_end: S,
) -> S {
    let line_block_end = line.band.block_end;
    let Some(strut) = carried_strut.filter(|strut| strut.after_baseline > S::ZERO) else {
        return line_block_end;
    };
    let touches_float_edge = if flow_axes
        .logical_axis_progression(crate::LogicalAxis::Inline)
        .is_decreasing()
    {
        line_inline_start == line.band.inline_start
    } else {
        line_inline_end == line.band.inline_end
    };
    if float_transition != Some(line_block_end) || !touches_float_edge {
        return line_block_end;
    }
    let baseline_phase = (line.baseline - strut.baseline).max(S::ZERO) / strut.after_baseline;
    line_block_end + baseline_phase
}

fn resolved_inline_unit_slots<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    flow_axes: FlowAxes,
    text_align: TextAlign,
) -> (S, Vec<usize>, Vec<S>) {
    let line_inline_start = selected_line_inline_start(line, flow_axes, text_align);
    let visual_order = reordered_inline_unit_indices(&line.units);
    let mut visual_indices = vec![0; line.units.len()];
    let mut inline_starts = vec![S::ZERO; line.units.len()];
    for (visual_index, source_index) in visual_order.iter().copied().enumerate() {
        visual_indices[source_index] = visual_index;
    }
    let mut inline_start = line_inline_start;
    for source_index in visual_order {
        let selected = line.units[source_index];
        let selected_inline_extent = if selected.discarded {
            S::ZERO
        } else {
            selected.participant.inline_advance(flow_axes)
                + selected.replacement_inline_extent.unwrap_or(S::ZERO)
        };
        inline_starts[source_index] = inline_start;
        inline_start = inline_start + selected_inline_extent;
    }
    (line_inline_start, visual_indices, inline_starts)
}

#[must_use]
#[cfg(test)]
pub(super) fn layout_mixed_inline_run<S: LayoutScalar>(
    input: MixedInlineRunInputOf<S>,
) -> MixedInlineRunReportOf<S> {
    let available = resolved_inline_available(&input);
    layout_mixed_inline_run_from_available(
        input,
        available,
        |_, _| LogicalLineBandQueryResultOf {
            inline_start: S::ZERO,
            inline_end: available,
            next_transition: None,
        },
        |block, _| block,
    )
}

#[must_use]
pub(super) fn layout_mixed_inline_run_with_band_source<S, BandSource, ClearSource>(
    input: MixedInlineRunInputOf<S>,
    band_source: BandSource,
    clear_source: ClearSource,
) -> MixedInlineRunReportOf<S>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
{
    let available = resolved_inline_available(&input);
    layout_mixed_inline_run_from_available(input, available, band_source, clear_source)
}

fn layout_mixed_inline_run_from_available<S, BandSource, ClearSource>(
    input: MixedInlineRunInputOf<S>,
    available: S,
    mut band_source: BandSource,
    mut clear_source: ClearSource,
) -> MixedInlineRunReportOf<S>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
{
    let wraps = !matches!(input.available_inline_extent, AvailableOf::MaxContent);
    let mut selected_lines = Vec::new();
    let mut source_cursor = 0;
    let mut pending_strut = None;
    let mut block_cursor = S::ZERO;
    let mut float_strut = None;
    let mut continuation_inline_cursor = None;
    let mut advanced_phase_transition = false;
    let mut resolved_terminal_block_extent = S::ZERO;
    let mut float_edge_phase = None;

    while let Some(mut provisional) = select_next_inline_line(
        &input.participants,
        source_cursor,
        available,
        wraps,
        pending_strut,
        input.flow_axes,
    ) {
        if float_strut.is_some() {
            resolve_float_strut_phase(&mut provisional.line, float_strut, input.flow_axes);
        }
        let provisional_block_end =
            block_cursor + provisional.line.baseline + provisional.line.after_baseline;
        let queried_band = band_source(block_cursor, provisional_block_end);
        let band_inline_end = queried_band.inline_end.max(queried_band.inline_start);
        let band_available = band_inline_end - queried_band.inline_start;
        let has_float_transition = queried_band.next_transition.is_some();

        if band_available == S::ZERO
            && provisional.line.used_inline_extent > S::ZERO
            && let Some(next_transition) = queried_band
                .next_transition
                .filter(|transition| transition.is_finite() && *transition > block_cursor)
        {
            block_cursor = next_transition;
            continue;
        }

        let use_containing_band = band_available == S::ZERO
            && provisional.line.used_inline_extent > S::ZERO
            && queried_band.next_transition.is_none();
        let (band_inline_start, band_inline_end, mut selected) = if use_containing_band {
            (S::ZERO, available, provisional)
        } else {
            let selected = select_next_inline_line(
                &input.participants,
                source_cursor,
                band_available,
                wraps,
                pending_strut,
                input.flow_axes,
            )
            .expect("a provisional line remains selectable against its queried band");
            (queried_band.inline_start, band_inline_end, selected)
        };
        let next_float_strut = if float_strut.is_some()
            || (has_float_transition && starts_inline_strut_phase(&selected.line))
        {
            resolve_float_strut_phase(&mut selected.line, float_strut, input.flow_axes)
        } else {
            float_strut
        };
        resolve_completed_fallback_envelope(
            &mut selected.line,
            input.flow_axes,
            has_float_transition,
        );
        let selected_block_end =
            block_cursor + selected.line.baseline + selected.line.after_baseline;
        let continuation_overflows = continuation_inline_cursor
            .is_some_and(|cursor| cursor + selected.line.used_inline_extent > band_inline_end);
        if !advanced_phase_transition
            && float_strut.is_some()
            && continuation_overflows
            && let Some(next_transition) = queried_band.next_transition.filter(|transition| {
                transition.is_finite()
                    && *transition > block_cursor
                    && *transition < selected_block_end
            })
        {
            block_cursor = next_transition;
            advanced_phase_transition = true;
            continue;
        }

        let mut line = selected.line;
        if advanced_phase_transition && continuation_inline_cursor.is_some() {
            line.inline_start_override = Some(band_inline_start);
        }
        line.band = LogicalLineBandOf {
            inline_start: band_inline_start,
            inline_end: band_inline_end,
            block_start: block_cursor,
            block_end: selected_block_end,
        };
        source_cursor = selected.next_source_cursor;
        pending_strut = selected.pending_strut;
        let line_block_end = block_cursor + line.baseline + line.after_baseline;
        let line_inline_start =
            selected_line_inline_start(&line, input.flow_axes, input.text_align);
        let line_inline_end = line_inline_start + line.used_inline_extent;
        if has_float_transition && line.split_baseline_edge_phase.is_some() {
            float_edge_phase = line.split_baseline_edge_phase;
        }
        resolved_terminal_block_extent = resolved_float_terminal_block_extent(
            &line,
            float_strut,
            queried_band.next_transition,
            input.flow_axes,
            line_inline_start,
            line_inline_end,
        );
        continuation_inline_cursor = float_strut.is_some().then_some(line_inline_end);
        float_strut = next_float_strut;
        advanced_phase_transition = false;
        block_cursor =
            clear_source(line_block_end, line.post_line_clear_intent).max(line_block_end);
        selected_lines.push(line);
    }

    let mut inline_extent = S::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut fragments = Vec::new();
    let mut anchors = Vec::new();
    let mut atomics = Vec::new();
    let mut controls = Vec::new();
    let mut post_line_clear_intents = Vec::new();
    let mut line_bands = Vec::new();
    for (line_index, line) in selected_lines.into_iter().enumerate() {
        let block_start = line.band.block_start;
        let line_baseline = block_start + line.baseline;
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);
        let (line_inline_start, visual_indices, inline_starts) =
            resolved_inline_unit_slots(&line, input.flow_axes, input.text_align);
        inline_extent = inline_extent.max(line_inline_start + line.used_inline_extent);
        post_line_clear_intents.push(line.post_line_clear_intent);
        line_bands.push(line.band);
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
                    let item_block_start = match item.alignment {
                        InlineControlAlignment::Baseline => line
                            .fallback_line_band
                            .and_then(|band| {
                                item.fallback_block_start_in_band(input.flow_axes, band)
                            })
                            .map_or_else(
                                || line_baseline - item.baseline_offset(input.flow_axes),
                                |offset| block_start + offset,
                            ),
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
                    let control_block = if line.uses_float_strut_phase {
                        block_start + control.metrics().baseline()
                    } else {
                        control_block_position(
                            control.alignment(),
                            control.metrics(),
                            block_start,
                            line_baseline,
                            line.baseline + line.after_baseline,
                        )
                    };
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
    }

    MixedInlineRunReportOf {
        inline_extent,
        block_extent: block_cursor.max(resolved_terminal_block_extent),
        float_edge_phase,
        first_baseline,
        last_baseline,
        fragments,
        anchors,
        atomics,
        controls,
        post_line_clear_intents,
        line_bands,
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

impl<S: LayoutScalar> AtomicInlineBoxParticipant<S> {
    fn baseline_offset(self, flow_axes: FlowAxes) -> S {
        let logical_size = flow_axes.logical_size(self.size);
        let logical_margin = flow_axes.logical_edges(self.margin);
        self.first_baseline
            .unwrap_or(logical_size.block + logical_margin.block_end)
    }

    fn metrics(self, flow_axes: FlowAxes) -> InlineMetricContributionOf<S> {
        let logical_size = flow_axes.logical_size(self.size);
        let logical_margin = flow_axes.logical_edges(self.margin);
        InlineMetricContributionOf {
            baseline: logical_margin.block_start + self.baseline_offset(flow_axes),
            after_baseline: self.first_baseline.map_or(S::ZERO, |inner| {
                logical_size.block - inner + logical_margin.block_end
            }),
        }
    }

    fn fallback_block_start_in_band(
        self,
        flow_axes: FlowAxes,
        line_band: InlineMetricContributionOf<S>,
    ) -> Option<S> {
        if self.alignment != InlineControlAlignment::Baseline || self.first_baseline.is_some() {
            return None;
        }
        let logical_size = flow_axes.logical_size(self.size);
        let logical_margin = flow_axes.logical_edges(self.margin);
        let margin_box_extent =
            logical_margin.block_start + logical_size.block + logical_margin.block_end;
        let fallback_metrics = self.metrics(flow_axes);
        (margin_box_extent >= S::ZERO
            && margin_box_extent < line_band.extent()
            && fallback_metrics.baseline > line_band.baseline)
            .then(|| {
                (line_band.extent() - margin_box_extent) / S::from_f64(2.0)
                    + logical_margin.block_start
            })
    }
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
