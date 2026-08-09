use crate::{LayoutScalar, LengthResolutionStatus, Scalar, TrackComponentOf, TrackSizingOf};

use super::named::{GridAreaNameFacts, GridNamedContext, NamedGridLines};
use super::tracks::{AutoRepeatTrackOrigin, TrackExpansionOf, expand_track_components};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExplicitTrackSizingOrigin {
    AuthoredTemplate,
    Inherited,
    TemplateAreaAutoPattern { pattern_index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExplicitTrackOrigin {
    pub(super) sizing: ExplicitTrackSizingOrigin,
    pub(super) auto_repeat: Option<AutoRepeatTrackOrigin>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ExpandedGridTopology<S: LayoutScalar = Scalar> {
    pub(super) column_tracks: Vec<TrackSizingOf<S>>,
    pub(super) row_tracks: Vec<TrackSizingOf<S>>,
    pub(super) explicit_columns: usize,
    pub(super) explicit_rows: usize,
    pub(super) named_columns: NamedGridLines,
    pub(super) named_rows: NamedGridLines,
    pub(super) area_facts: Option<GridAreaNameFacts>,
    pub(super) column_origins: Vec<ExplicitTrackOrigin>,
    pub(super) row_origins: Vec<ExplicitTrackOrigin>,
}

pub(super) struct ExpandedGridTopologyInput<'a, S: LayoutScalar = Scalar> {
    pub(super) columns: TrackExpansionOf<S>,
    pub(super) rows: TrackExpansionOf<S>,
    pub(super) named: GridNamedContext,
    pub(super) auto_columns: &'a [TrackComponentOf<S>],
    pub(super) auto_rows: &'a [TrackComponentOf<S>],
    pub(super) column_basis: Option<S>,
    pub(super) row_basis: Option<S>,
    pub(super) column_gap: S,
    pub(super) row_gap: S,
    pub(super) inherited_columns: bool,
    pub(super) inherited_rows: bool,
}

struct ExpandedAxisTopology<S: LayoutScalar = Scalar> {
    tracks: Vec<TrackSizingOf<S>>,
    origins: Vec<ExplicitTrackOrigin>,
    named_lines: NamedGridLines,
}

impl<S: LayoutScalar> ExpandedGridTopology<S> {
    pub(super) fn new(
        input: ExpandedGridTopologyInput<'_, S>,
    ) -> Result<Self, LengthResolutionStatus<S>> {
        let GridNamedContext {
            columns: named_columns,
            rows: named_rows,
            area_facts,
        } = input.named;
        let columns = complete_explicit_axis(
            input.columns,
            named_columns,
            input.auto_columns,
            input.column_basis,
            input.column_gap,
            input.inherited_columns,
        )?;
        let rows = complete_explicit_axis(
            input.rows,
            named_rows,
            input.auto_rows,
            input.row_basis,
            input.row_gap,
            input.inherited_rows,
        )?;
        let topology = Self {
            explicit_columns: columns.tracks.len(),
            explicit_rows: rows.tracks.len(),
            column_tracks: columns.tracks,
            row_tracks: rows.tracks,
            named_columns: columns.named_lines,
            named_rows: rows.named_lines,
            area_facts,
            column_origins: columns.origins,
            row_origins: rows.origins,
        };
        debug_assert!(topology.has_complete_origin_evidence());
        Ok(topology)
    }

    pub(super) fn has_complete_origin_evidence(&self) -> bool {
        axis_origin_evidence_is_complete(
            &self.column_tracks,
            self.explicit_columns,
            &self.named_columns,
            &self.column_origins,
        ) && axis_origin_evidence_is_complete(
            &self.row_tracks,
            self.explicit_rows,
            &self.named_rows,
            &self.row_origins,
        )
    }

    #[cfg(test)]
    pub(super) fn from_test_parts(
        column_tracks: Vec<TrackSizingOf<S>>,
        row_tracks: Vec<TrackSizingOf<S>>,
        named_columns: NamedGridLines,
        named_rows: NamedGridLines,
        area_facts: Option<GridAreaNameFacts>,
    ) -> Self {
        let explicit_columns = column_tracks.len();
        let explicit_rows = row_tracks.len();
        Self {
            column_tracks,
            row_tracks,
            explicit_columns,
            explicit_rows,
            named_columns,
            named_rows,
            area_facts,
            column_origins: vec![
                ExplicitTrackOrigin {
                    sizing: ExplicitTrackSizingOrigin::AuthoredTemplate,
                    auto_repeat: None,
                };
                explicit_columns
            ],
            row_origins: vec![
                ExplicitTrackOrigin {
                    sizing: ExplicitTrackSizingOrigin::AuthoredTemplate,
                    auto_repeat: None,
                };
                explicit_rows
            ],
        }
    }
}

fn complete_explicit_axis<S: LayoutScalar>(
    expansion: TrackExpansionOf<S>,
    named_lines: NamedGridLines,
    auto_components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    inherited: bool,
) -> Result<ExpandedAxisTopology<S>, LengthResolutionStatus<S>> {
    let mut tracks = Vec::with_capacity(named_lines.explicit_track_count);
    let mut origins = Vec::with_capacity(named_lines.explicit_track_count);
    for track in expansion.tracks {
        tracks.push(track.sizing);
        origins.push(ExplicitTrackOrigin {
            sizing: if inherited {
                ExplicitTrackSizingOrigin::Inherited
            } else {
                ExplicitTrackSizingOrigin::AuthoredTemplate
            },
            auto_repeat: track.auto_repeat,
        });
    }

    if !inherited && tracks.len() < named_lines.explicit_track_count {
        let auto_pattern = expand_track_components(auto_components, basis, gap, None)?;
        let missing = named_lines.explicit_track_count - tracks.len();
        for pattern_offset in 0..missing {
            let (sizing, pattern_index) = if auto_pattern.is_empty() {
                (TrackSizingOf::AUTO, 0)
            } else {
                let pattern_index = pattern_offset % auto_pattern.len();
                (auto_pattern[pattern_index].clone(), pattern_index)
            };
            tracks.push(sizing);
            origins.push(ExplicitTrackOrigin {
                sizing: ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index },
                auto_repeat: None,
            });
        }
    }

    debug_assert_eq!(tracks.len(), named_lines.explicit_track_count);
    Ok(ExpandedAxisTopology {
        tracks,
        origins,
        named_lines,
    })
}

fn axis_origin_evidence_is_complete<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    explicit_count: usize,
    named_lines: &NamedGridLines,
    origins: &[ExplicitTrackOrigin],
) -> bool {
    tracks.len() == explicit_count
        && origins.len() == explicit_count
        && named_lines.explicit_track_count == explicit_count
        && origins.iter().all(|origin| match origin.sizing {
            ExplicitTrackSizingOrigin::AuthoredTemplate => true,
            ExplicitTrackSizingOrigin::Inherited => origin.auto_repeat.is_none(),
            ExplicitTrackSizingOrigin::TemplateAreaAutoPattern { pattern_index } => {
                origin.auto_repeat.is_none() && pattern_index < explicit_count.max(1)
            }
        })
}
