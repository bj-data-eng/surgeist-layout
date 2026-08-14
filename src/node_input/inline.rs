use std::collections::HashSet;

use super::super::{DefaultScalar, LayoutScalar, NonNegativeFiniteScalarErrorOf};
use super::{Clear, Direction, validate_numeric_property};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineBreakDisplay {
    #[default]
    Break,
    None,
}

impl LineBreakDisplay {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlign {
    #[default]
    Auto,
    LegacyLeft,
    LegacyRight,
    LegacyCenter,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
    Bottom,
}

/// The writing-mode state supplied to layout.
///
/// The five supported values are `HorizontalTb`, `VerticalRl`, `VerticalLr`,
/// `SidewaysRl`, and `SidewaysLr`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

impl WritingMode {
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        !matches!(self, Self::HorizontalTb)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineMetricsOf<S: LayoutScalar = DefaultScalar> {
    baseline: S,
    line_extent: S,
}

pub type InlineMetrics = InlineMetricsOf<DefaultScalar>;

/// Caller-local identity for one shaped inline segment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InlineSegmentId(u64);

impl InlineSegmentId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A validated Unicode bidi embedding level.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BidiLevel(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BidiLevelError {
    OutOfRange { level: u8 },
}

impl BidiLevel {
    pub const fn try_new(level: u8) -> Result<Self, BidiLevelError> {
        if level <= 125 {
            Ok(Self(level))
        } else {
            Err(BidiLevelError::OutOfRange { level })
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn is_rtl(self) -> bool {
        self.0 % 2 == 1
    }
}

/// Whitespace behavior at a selected line edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineWhitespaceEdge {
    Preserve,
    DiscardAtLineStart,
    DiscardAtLineEnd,
    DiscardAtBoth,
}

/// The closed kind of break opportunity following one inline participant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBreakKind {
    Prohibited,
    Allowed,
    AllowedWithReplacement,
    Mandatory,
}

/// A validated break opportunity following one inline participant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBreakOpportunityOf<S: LayoutScalar = DefaultScalar> {
    kind: InlineBreakKind,
    replacement_inline_extent: Option<S>,
}

pub type InlineBreakOpportunity = InlineBreakOpportunityOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBreakOpportunityOf<S> {
    #[must_use]
    pub const fn prohibited() -> Self {
        Self {
            kind: InlineBreakKind::Prohibited,
            replacement_inline_extent: None,
        }
    }

    #[must_use]
    pub const fn allowed() -> Self {
        Self {
            kind: InlineBreakKind::Allowed,
            replacement_inline_extent: None,
        }
    }

    pub fn try_allowed_with_replacement(extent: S) -> Result<Self, InlineTextInputErrorOf<S>> {
        Ok(Self {
            kind: InlineBreakKind::AllowedWithReplacement,
            replacement_inline_extent: Some(validate_numeric_property(extent).map_err(
                |error| InlineTextInputErrorOf::InvalidReplacementInlineExtent { error },
            )?),
        })
    }

    #[must_use]
    pub const fn mandatory() -> Self {
        Self {
            kind: InlineBreakKind::Mandatory,
            replacement_inline_extent: None,
        }
    }

    #[must_use]
    pub const fn kind(self) -> InlineBreakKind {
        self.kind
    }

    #[must_use]
    pub const fn replacement_inline_extent(self) -> Option<S> {
        self.replacement_inline_extent
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineTextInputErrorOf<S: LayoutScalar = DefaultScalar> {
    Empty,
    InvalidReplacementInlineExtent {
        error: NonNegativeFiniteScalarErrorOf<S>,
    },
    DuplicateSegmentId {
        segment_id: InlineSegmentId,
    },
    InvalidInlineExtent {
        segment_id: InlineSegmentId,
        error: NonNegativeFiniteScalarErrorOf<S>,
    },
    ReplacementWithDiscardableWhitespace {
        segment_id: InlineSegmentId,
        whitespace_edge: InlineWhitespaceEdge,
    },
}

pub type InlineTextInputError = InlineTextInputErrorOf<DefaultScalar>;

/// One indivisible layout-ready shaped inline segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapedInlineSegmentOf<S: LayoutScalar = DefaultScalar> {
    segment_id: InlineSegmentId,
    inline_extent: S,
    metrics: InlineMetricsOf<S>,
    bidi_level: BidiLevel,
    whitespace_edge: InlineWhitespaceEdge,
    following_break: InlineBreakOpportunityOf<S>,
}

pub type ShapedInlineSegment = ShapedInlineSegmentOf<DefaultScalar>;

impl<S: LayoutScalar> ShapedInlineSegmentOf<S> {
    pub fn try_new(
        segment_id: InlineSegmentId,
        inline_extent: S,
        metrics: InlineMetricsOf<S>,
        bidi_level: BidiLevel,
        whitespace_edge: InlineWhitespaceEdge,
        following_break: InlineBreakOpportunityOf<S>,
    ) -> Result<Self, InlineTextInputErrorOf<S>> {
        let inline_extent = validate_numeric_property(inline_extent)
            .map_err(|error| InlineTextInputErrorOf::InvalidInlineExtent { segment_id, error })?;
        if following_break.kind() == InlineBreakKind::AllowedWithReplacement
            && whitespace_edge != InlineWhitespaceEdge::Preserve
        {
            return Err(
                InlineTextInputErrorOf::ReplacementWithDiscardableWhitespace {
                    segment_id,
                    whitespace_edge,
                },
            );
        }
        Ok(Self {
            segment_id,
            inline_extent,
            metrics,
            bidi_level,
            whitespace_edge,
            following_break,
        })
    }

    #[must_use]
    pub const fn segment_id(self) -> InlineSegmentId {
        self.segment_id
    }
    #[must_use]
    pub const fn inline_extent(self) -> S {
        self.inline_extent
    }
    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
    #[must_use]
    pub const fn bidi_level(self) -> BidiLevel {
        self.bidi_level
    }
    #[must_use]
    pub const fn whitespace_edge(self) -> InlineWhitespaceEdge {
        self.whitespace_edge
    }
    #[must_use]
    pub const fn following_break(self) -> InlineBreakOpportunityOf<S> {
        self.following_break
    }
}

/// A nonempty ordered collection of shaped inline segments.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineTextInputOf<S: LayoutScalar = DefaultScalar> {
    segments: Vec<ShapedInlineSegmentOf<S>>,
}

pub type InlineTextInput = InlineTextInputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineTextInputOf<S> {
    pub fn try_new(
        segments: Vec<ShapedInlineSegmentOf<S>>,
    ) -> Result<Self, InlineTextInputErrorOf<S>> {
        if segments.is_empty() {
            return Err(InlineTextInputErrorOf::Empty);
        }
        let mut segment_ids = HashSet::<InlineSegmentId>::with_capacity(segments.len());
        for segment in &segments {
            if !segment_ids.insert(segment.segment_id) {
                return Err(InlineTextInputErrorOf::DuplicateSegmentId {
                    segment_id: segment.segment_id,
                });
            }
        }
        Ok(Self { segments })
    }

    #[must_use]
    pub fn segments(&self) -> &[ShapedInlineSegmentOf<S>] {
        &self.segments
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtomicInlineParticipationErrorOf<S: LayoutScalar = DefaultScalar> {
    ReplacementNotAllowed { replacement_inline_extent: S },
}

pub type AtomicInlineParticipationError = AtomicInlineParticipationErrorOf<DefaultScalar>;

/// Paragraph participation facts for one atomic inline box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineParticipationOf<S: LayoutScalar = DefaultScalar> {
    bidi_level: BidiLevel,
    following_break: InlineBreakOpportunityOf<S>,
}

pub type AtomicInlineParticipation = AtomicInlineParticipationOf<DefaultScalar>;

impl<S: LayoutScalar> AtomicInlineParticipationOf<S> {
    pub fn try_new(
        bidi_level: BidiLevel,
        following_break: InlineBreakOpportunityOf<S>,
    ) -> Result<Self, AtomicInlineParticipationErrorOf<S>> {
        if let Some(replacement_inline_extent) = following_break.replacement_inline_extent() {
            return Err(AtomicInlineParticipationErrorOf::ReplacementNotAllowed {
                replacement_inline_extent,
            });
        }
        Ok(Self {
            bidi_level,
            following_break,
        })
    }

    #[must_use]
    pub const fn bidi_level(self) -> BidiLevel {
        self.bidi_level
    }
    #[must_use]
    pub const fn following_break(self) -> InlineBreakOpportunityOf<S> {
        self.following_break
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineMetricsError<S: LayoutScalar = DefaultScalar> {
    NonFinite { value: S },
    Negative { value: S },
    BaselineExceedsLineExtent { baseline: S, line_extent: S },
    BaselineExceedsLineHeight { baseline: S, line_height: S },
}

impl<S: LayoutScalar> InlineMetricsOf<S> {
    pub fn try_new(baseline: S, line_extent: S) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(baseline)?;
        validate_non_negative_finite(line_extent)?;

        if baseline > line_extent {
            return Err(InlineMetricsError::BaselineExceedsLineExtent {
                baseline,
                line_extent,
            });
        }

        Ok(Self {
            baseline,
            line_extent,
        })
    }

    pub fn from_ascent_descent(ascent: S, descent: S) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(ascent)?;
        validate_non_negative_finite(descent)?;
        Self::try_new(ascent, ascent + descent)
    }

    pub fn from_line_height_and_baseline(
        line_height: S,
        baseline: S,
    ) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(line_height)?;
        validate_non_negative_finite(baseline)?;

        if baseline > line_height {
            return Err(InlineMetricsError::BaselineExceedsLineHeight {
                baseline,
                line_height,
            });
        }

        Ok(Self {
            baseline,
            line_extent: line_height,
        })
    }

    #[must_use]
    pub const fn baseline(self) -> S {
        self.baseline
    }

    #[must_use]
    pub const fn line_extent(self) -> S {
        self.line_extent
    }

    #[must_use]
    pub fn after_baseline(self) -> S {
        self.line_extent - self.baseline
    }
}

impl<S: LayoutScalar> Default for InlineMetricsOf<S> {
    fn default() -> Self {
        Self::from_line_height_and_baseline(S::from_f64(16.0), S::from_f64(12.0))
            .expect("default inline metrics are valid")
    }
}

fn validate_non_negative_finite<S: LayoutScalar>(value: S) -> Result<(), InlineMetricsError<S>> {
    if !value.is_finite() {
        return Err(InlineMetricsError::NonFinite { value });
    }
    if value < S::ZERO {
        return Err(InlineMetricsError::Negative { value });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineBreakInputOf<S: LayoutScalar = DefaultScalar> {
    display: LineBreakDisplay,
    direction: Direction,
    writing_mode: WritingMode,
    vertical_align: VerticalAlign,
    clear: Clear,
    metrics: InlineMetricsOf<S>,
}

pub type LineBreakInput = LineBreakInputOf<DefaultScalar>;

impl<S: LayoutScalar> LineBreakInputOf<S> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_metrics(mut self, metrics: InlineMetricsOf<S>) -> Self {
        self.metrics = metrics;
        self
    }

    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.display = LineBreakDisplay::None;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn with_clear(mut self, clear: Clear) -> Self {
        self.clear = clear;
        self
    }

    #[must_use]
    pub const fn display(self) -> LineBreakDisplay {
        self.display
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn clear(self) -> Clear {
        self.clear
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}

impl<S: LayoutScalar> Default for LineBreakInputOf<S> {
    fn default() -> Self {
        Self {
            display: LineBreakDisplay::Break,
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            vertical_align: VerticalAlign::Baseline,
            clear: Clear::None,
            metrics: InlineMetricsOf::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoundaryKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryInputOf<S: LayoutScalar = DefaultScalar> {
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    vertical_align: VerticalAlign,
    metrics: InlineMetricsOf<S>,
}

pub type InlineBoundaryInput = InlineBoundaryInputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBoundaryInputOf<S> {
    #[must_use]
    pub const fn new(kind: InlineBoundaryKind, metrics: InlineMetricsOf<S>) -> Self {
        Self {
            kind,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            vertical_align: VerticalAlign::Baseline,
            metrics,
        }
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}
