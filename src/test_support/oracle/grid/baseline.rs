use super::placement::GridArea;
use super::subgrid::{OracleGapReport, OracleGridError, TrackSpan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineAlignment {
    None,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineGroupKind {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineFallback {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineItemFacts {
    pub id: &'static str,
    pub area: GridArea,
    pub block_size: f32,
    pub margin_before: f32,
    pub margin_after: f32,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
    pub synthesized_first: bool,
    pub synthesized_last: bool,
    pub alignment: BaselineAlignment,
    pub out_of_flow: bool,
    pub baseline_axis_auto_margins: bool,
    pub spans_intrinsic_track: bool,
    pub baseline_requires_unavailable_subgrid_layout: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineGeometry {
    pub available_span_size: f32,
    pub margin_box_size: f32,
    pub major_baseline: f32,
    pub minor_baseline: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BaselineShim {
    pub before: f32,
    pub after: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineParticipationReport {
    pub id: &'static str,
    pub alignment: BaselineAlignment,
    pub participates: bool,
    pub group: Option<BaselineGroupKind>,
    pub fallback: Option<BaselineFallback>,
    pub used_synthesized_baseline: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaselineGroupInput {
    pub track_count: usize,
    pub track_sizes: Vec<f32>,
    pub gap: f32,
    pub items: Vec<BaselineItemFacts>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaselineGroupReport {
    pub major: Vec<Option<f32>>,
    pub minor: Vec<Option<f32>>,
    pub participation: Vec<BaselineParticipationReport>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerBaselineFallbackItem {
    pub id: &'static str,
    pub area: GridArea,
    pub block_offset: f32,
    pub first_baseline: f32,
    pub last_baseline: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerBaselineInput {
    pub track_offsets: Vec<f32>,
    pub track_sizes: Vec<f32>,
    pub groups: BaselineGroupReport,
    pub fallback_items: Vec<ContainerBaselineFallbackItem>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerBaselineReport {
    pub first: Option<f32>,
    pub last: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselineInheritanceInput {
    pub parent_span: TrackSpan,
    pub reversed: bool,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub start_mbp: f32,
    pub end_mbp: f32,
    pub parent_major: Vec<Option<f32>>,
    pub parent_minor: Vec<Option<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselineInheritanceReport {
    pub parent_span: TrackSpan,
    pub reversed: bool,
    pub start_mbp: f32,
    pub end_mbp: f32,
    pub parent_gap: OracleGapReport,
    pub subgrid_gap: OracleGapReport,
    pub gap_difference: f32,
    pub sliced_major: Vec<Option<f32>>,
    pub sliced_minor: Vec<Option<f32>>,
    pub after_reversal_major: Vec<Option<f32>>,
    pub after_reversal_minor: Vec<Option<f32>>,
    pub after_mbp_major: Vec<Option<f32>>,
    pub after_mbp_minor: Vec<Option<f32>>,
    pub final_major: Vec<Option<f32>>,
    pub final_minor: Vec<Option<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselinePublicationInput {
    pub subgrid_span_in_parent: TrackSpan,
    pub subgrid_offset_in_parent: f32,
    pub reversed: bool,
    /// One-based track index within `subgrid_span_in_parent`.
    pub descendant_local_track: usize,
    pub descendant_track_offset_in_subgrid: f32,
    pub descendant_group: BaselineGroupKind,
    pub descendant_baseline_in_track: f32,
    pub inherited_axis_offset: f32,
    pub synthesized_cycle_fallback: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridBaselinePublicationReport {
    pub published: bool,
    pub ancestor_track: Option<usize>,
    pub group: Option<BaselineGroupKind>,
    pub baseline: Option<f32>,
}

fn uses_synthesized_aligned_baseline(item: &BaselineItemFacts) -> bool {
    match item.alignment {
        BaselineAlignment::First => item.first_baseline.is_none() || item.synthesized_first,
        BaselineAlignment::Last => item.last_baseline.is_none() || item.synthesized_last,
        BaselineAlignment::None => false,
    }
}

fn has_baseline_cycle_fallback(item: &BaselineItemFacts) -> bool {
    uses_synthesized_aligned_baseline(item)
        && (item.spans_intrinsic_track || item.baseline_requires_unavailable_subgrid_layout)
}

#[must_use]
pub fn baseline_participation(item: BaselineItemFacts) -> BaselineParticipationReport {
    let (group, fallback) = match item.alignment {
        BaselineAlignment::None => (None, None),
        BaselineAlignment::First => (
            Some(BaselineGroupKind::Major),
            Some(BaselineFallback::Start),
        ),
        BaselineAlignment::Last => (Some(BaselineGroupKind::Minor), Some(BaselineFallback::End)),
    };
    let used_synthesized_baseline = uses_synthesized_aligned_baseline(&item);
    let cycle_fallback = has_baseline_cycle_fallback(&item);
    let participates =
        group.is_some() && !item.out_of_flow && !item.baseline_axis_auto_margins && !cycle_fallback;
    let report_group = if participates { group } else { None };

    BaselineParticipationReport {
        id: item.id,
        alignment: item.alignment,
        participates,
        group: report_group,
        fallback: (!participates).then_some(fallback).flatten(),
        used_synthesized_baseline,
    }
}

#[must_use]
pub fn baseline_offset(
    group: BaselineGroupKind,
    shared_baseline: f32,
    geometry: BaselineGeometry,
) -> f32 {
    match group {
        BaselineGroupKind::Major => shared_baseline - geometry.major_baseline,
        BaselineGroupKind::Minor => {
            geometry.available_span_size
                - (shared_baseline - geometry.minor_baseline)
                - geometry.margin_box_size
        }
    }
}

#[must_use]
pub fn baseline_intrinsic_shim(
    group: BaselineGroupKind,
    shared_baseline: f32,
    geometry: BaselineGeometry,
) -> BaselineShim {
    match group {
        BaselineGroupKind::Major => BaselineShim {
            before: (shared_baseline - geometry.major_baseline).max(0.0),
            after: 0.0,
        },
        BaselineGroupKind::Minor => BaselineShim {
            before: 0.0,
            after: (shared_baseline - geometry.minor_baseline).max(0.0),
        },
    }
}

pub fn baseline_groups(input: BaselineGroupInput) -> Result<BaselineGroupReport, OracleGridError> {
    if input.track_count == 0 {
        return Err(OracleGridError::EmptyTrackList);
    }
    if input.track_sizes.len() != input.track_count {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let mut major = vec![None; input.track_count];
    let mut minor = vec![None; input.track_count];
    let mut participation = Vec::with_capacity(input.items.len());

    for item in input.items {
        let start_index = item
            .area
            .row_start
            .checked_sub(1)
            .ok_or(OracleGridError::SpanOutOfRange)?;
        if item.area.row_span == 0 {
            return Err(OracleGridError::SpanOutOfRange);
        }
        let end_exclusive = start_index
            .checked_add(item.area.row_span)
            .ok_or(OracleGridError::SpanOutOfRange)?;
        if end_exclusive > input.track_count {
            return Err(OracleGridError::SpanOutOfRange);
        }

        let report = baseline_participation(item);
        if let Some(group) = report.group {
            let available_span_size = input.track_sizes[start_index..end_exclusive]
                .iter()
                .sum::<f32>()
                + input.gap * (item.area.row_span - 1) as f32;
            let geometry = BaselineGeometry::from_item(item, available_span_size)?;

            match group {
                BaselineGroupKind::Major => {
                    push_group_contribution(&mut major[start_index], geometry.major_baseline);
                }
                BaselineGroupKind::Minor => {
                    push_group_contribution(&mut minor[end_exclusive - 1], geometry.minor_baseline);
                }
            }
        }
        participation.push(report);
    }

    Ok(BaselineGroupReport {
        major,
        minor,
        participation,
    })
}

pub fn container_baselines(
    input: ContainerBaselineInput,
) -> Result<ContainerBaselineReport, OracleGridError> {
    let track_count = input.track_offsets.len();
    if input.track_sizes.len() != track_count
        || input.groups.major.len() != track_count
        || input.groups.minor.len() != track_count
    {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let mut first_occupied_row = None;
    let mut last_occupied_row = None;
    for row in 0..track_count {
        if input.groups.major[row].is_some() || input.groups.minor[row].is_some() {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }

    for item in &input.fallback_items {
        let start = fallback_start_row(*item)?;
        let end = start
            .checked_add(item.area.row_span)
            .ok_or(OracleGridError::SpanOutOfRange)?;
        if end > track_count {
            return Err(OracleGridError::SpanOutOfRange);
        }
        include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, start);
        include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, end - 1);
    }

    let first = match first_occupied_row {
        Some(row) => input.groups.major[row]
            .map(|major| input.track_offsets[row] + major)
            .or_else(|| {
                input.groups.minor[row]
                    .map(|minor| input.track_offsets[row] + input.track_sizes[row] - minor)
            })
            .or_else(|| first_fallback_item(&input.fallback_items).map(|item| item.first_baseline)),
        None => None,
    };

    let last = match last_occupied_row {
        Some(row) => input.groups.minor[row]
            .map(|minor| input.track_offsets[row] + input.track_sizes[row] - minor)
            .or_else(|| input.groups.major[row].map(|major| input.track_offsets[row] + major))
            .or_else(|| last_fallback_item(&input.fallback_items).map(|item| item.last_baseline)),
        None => None,
    };

    Ok(ContainerBaselineReport { first, last })
}

pub fn inherit_subgrid_baselines(
    input: SubgridBaselineInheritanceInput,
) -> Result<SubgridBaselineInheritanceReport, OracleGridError> {
    let span_len = input.parent_span.checked_len()?;
    if input.parent_major.is_empty() || input.parent_minor.is_empty() {
        return Err(OracleGridError::EmptyTrackList);
    }
    if input.parent_major.len() != input.parent_minor.len()
        || input.parent_span.end > input.parent_major.len() + 1
        || span_len == 0
    {
        return Err(OracleGridError::SpanOutOfRange);
    }

    let start_index = input.parent_span.start - 1;
    let end_index = input.parent_span.end - 1;
    let sliced_major = input.parent_major[start_index..end_index].to_vec();
    let sliced_minor = input.parent_minor[start_index..end_index].to_vec();

    let mut after_reversal_major = sliced_major.clone();
    let mut after_reversal_minor = sliced_minor.clone();
    if input.reversed {
        after_reversal_major.reverse();
        after_reversal_minor.reverse();
    }

    let mut after_mbp_major = after_reversal_major.clone();
    if let Some(first_major) = after_mbp_major.first_mut().and_then(Option::as_mut) {
        *first_major += input.start_mbp;
    }

    let mut after_mbp_minor = after_reversal_minor.clone();
    if let Some(last_minor) = after_mbp_minor.last_mut().and_then(Option::as_mut) {
        *last_minor += input.end_mbp;
    }

    let gap_difference = (input.subgrid_gap.resolved - input.parent_gap.resolved) / 2.0;
    let mut final_major = after_mbp_major.clone();
    let mut final_minor = after_mbp_minor.clone();
    subtract_internal_gap_difference(&mut final_major, gap_difference);
    subtract_internal_gap_difference(&mut final_minor, gap_difference);

    Ok(SubgridBaselineInheritanceReport {
        parent_span: input.parent_span,
        reversed: input.reversed,
        start_mbp: input.start_mbp,
        end_mbp: input.end_mbp,
        parent_gap: input.parent_gap,
        subgrid_gap: input.subgrid_gap,
        gap_difference,
        sliced_major,
        sliced_minor,
        after_reversal_major,
        after_reversal_minor,
        after_mbp_major,
        after_mbp_minor,
        final_major,
        final_minor,
    })
}

pub fn publish_subgrid_baseline(
    input: SubgridBaselinePublicationInput,
) -> Result<SubgridBaselinePublicationReport, OracleGridError> {
    let span_len = input.subgrid_span_in_parent.checked_len()?;
    if input.descendant_local_track == 0 || input.descendant_local_track > span_len {
        return Err(OracleGridError::SpanOutOfRange);
    }

    if input.synthesized_cycle_fallback {
        return Ok(SubgridBaselinePublicationReport {
            published: false,
            ancestor_track: None,
            group: None,
            baseline: None,
        });
    }

    let local_index = input.descendant_local_track - 1;
    let ancestor_track = if input.reversed {
        input.subgrid_span_in_parent.start + (span_len - 1 - local_index)
    } else {
        input.subgrid_span_in_parent.start + local_index
    };
    let baseline = input.subgrid_offset_in_parent
        + input.inherited_axis_offset
        + input.descendant_track_offset_in_subgrid
        + input.descendant_baseline_in_track;

    Ok(SubgridBaselinePublicationReport {
        published: true,
        ancestor_track: Some(ancestor_track),
        group: Some(input.descendant_group),
        baseline: Some(baseline),
    })
}

fn push_group_contribution(group: &mut Option<f32>, contribution: f32) {
    *group = Some(group.map_or(contribution, |current| current.max(contribution)));
}

fn subtract_internal_gap_difference(groups: &mut [Option<f32>], gap_difference: f32) {
    if groups.len() < 2 {
        return;
    }

    for edge in 0..(groups.len() - 1) {
        subtract_group_coordinate(&mut groups[edge], gap_difference);
        subtract_group_coordinate(&mut groups[edge + 1], gap_difference);
    }
}

fn subtract_group_coordinate(group: &mut Option<f32>, amount: f32) {
    if let Some(coordinate) = group {
        *coordinate -= amount;
    }
}

fn include_occupied_row(first: &mut Option<usize>, last: &mut Option<usize>, row: usize) {
    *first = Some(first.map_or(row, |current| current.min(row)));
    *last = Some(last.map_or(row, |current| current.max(row)));
}

fn fallback_start_row(item: ContainerBaselineFallbackItem) -> Result<usize, OracleGridError> {
    if item.area.column_start == 0 || item.area.column_span == 0 || item.area.row_span == 0 {
        return Err(OracleGridError::SpanOutOfRange);
    }

    item.area
        .row_start
        .checked_sub(1)
        .ok_or(OracleGridError::SpanOutOfRange)
}

fn first_fallback_item(
    items: &[ContainerBaselineFallbackItem],
) -> Option<&ContainerBaselineFallbackItem> {
    items.iter().min_by_key(|item| fallback_start_key(item))
}

fn last_fallback_item(
    items: &[ContainerBaselineFallbackItem],
) -> Option<&ContainerBaselineFallbackItem> {
    items.iter().max_by_key(|item| fallback_end_key(item))
}

pub(super) fn fallback_start_key(item: &ContainerBaselineFallbackItem) -> (usize, usize) {
    (item.area.row_start, item.area.column_start)
}

pub(super) fn fallback_end_key(item: &ContainerBaselineFallbackItem) -> (usize, usize) {
    (
        item.area
            .row_start
            .saturating_add(item.area.row_span)
            .saturating_sub(1),
        item.area
            .column_start
            .saturating_add(item.area.column_span)
            .saturating_sub(1),
    )
}

impl BaselineGeometry {
    pub fn from_item(
        item: BaselineItemFacts,
        available_span_size: f32,
    ) -> Result<Self, OracleGridError> {
        if item.alignment == BaselineAlignment::None
            || item.out_of_flow
            || item.baseline_axis_auto_margins
            || has_baseline_cycle_fallback(&item)
        {
            return Err(OracleGridError::BaselineInferenceUnsupported);
        }

        let first = item.first_baseline.unwrap_or(item.block_size);
        let last = item.last_baseline.unwrap_or(0.0);
        if first < 0.0 || last < 0.0 || first > item.block_size || last > item.block_size {
            return Err(OracleGridError::BaselineInferenceUnsupported);
        }

        Ok(Self {
            available_span_size,
            margin_box_size: item.margin_before + item.block_size + item.margin_after,
            major_baseline: item.margin_before + first,
            minor_baseline: item.margin_after + item.block_size - last,
        })
    }
}
