use super::grid::{ContributionSize, GridArea, ItemContributionFacts};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlineSize {
    pub width: f32,
    pub height: f32,
}

impl InlineSize {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlinePoint {
    pub x: f32,
    pub y: f32,
}

impl InlinePoint {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlineEdges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl InlineEdges {
    pub const ZERO: Self = Self::all(0.0);

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn horizontal_sum(self) -> f32 {
        self.left + self.right
    }

    pub const fn vertical_sum(self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineItemFacts {
    pub id: &'static str,
    pub size: InlineSize,
    pub margin: InlineEdges,
    pub first_baseline: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineMetrics {
    pub id: &'static str,
    pub advance: f32,
    pub baseline: f32,
    pub descent: f32,
    pub margin_box_size: InlineSize,
    pub synthesized_baseline: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtomicInlineError {
    BaselineOutOfRange {
        id: &'static str,
        first_baseline: f32,
        height: f32,
    },
}

impl AtomicInlineMetrics {
    pub fn try_from_item(item: AtomicInlineItemFacts) -> Result<Self, AtomicInlineError> {
        if let Some(first_baseline) = item.first_baseline
            && !(0.0..=item.size.height).contains(&first_baseline)
        {
            return Err(AtomicInlineError::BaselineOutOfRange {
                id: item.id,
                first_baseline,
                height: item.size.height,
            });
        }

        Ok(Self::from_valid_item(item))
    }

    pub fn from_item(item: AtomicInlineItemFacts) -> Self {
        Self::try_from_item(item).expect("atomic inline item baseline must be inside border box")
    }

    fn from_valid_item(item: AtomicInlineItemFacts) -> Self {
        let first_baseline = item.first_baseline.unwrap_or(item.size.height);
        Self {
            id: item.id,
            advance: item.margin.left + item.size.width + item.margin.right,
            baseline: item.margin.top + first_baseline,
            descent: item.margin.bottom + item.size.height - first_baseline,
            margin_box_size: InlineSize::new(
                item.size.width + item.margin.horizontal_sum(),
                item.size.height + item.margin.vertical_sum(),
            ),
            synthesized_baseline: item.first_baseline.is_none(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineOuterDisplay {
    Block,
    Grid,
    GridLanes,
}

impl InlineOuterDisplay {
    pub const fn inner_context(self) -> InnerFormattingContext {
        match self {
            Self::Block => InnerFormattingContext::Block,
            Self::Grid => InnerFormattingContext::Grid,
            Self::GridLanes => InnerFormattingContext::GridLanes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InnerFormattingContext {
    Block,
    Grid,
    GridLanes,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineWrapperFacts {
    pub id: &'static str,
    pub outer_display: InlineOuterDisplay,
    pub outer_size: InlineSize,
    pub margin: InlineEdges,
    pub first_baseline: Option<f32>,
}

impl AtomicInlineWrapperFacts {
    pub fn new(
        id: &'static str,
        outer_display: InlineOuterDisplay,
        outer_size: InlineSize,
        margin: InlineEdges,
        first_baseline: Option<f32>,
    ) -> Self {
        Self {
            id,
            outer_display,
            outer_size,
            margin,
            first_baseline,
        }
    }

    pub const fn inner_context(self) -> InnerFormattingContext {
        self.outer_display.inner_context()
    }

    pub fn as_item(self) -> AtomicInlineItemFacts {
        AtomicInlineItemFacts {
            id: self.id,
            size: self.outer_size,
            margin: self.margin,
            first_baseline: self.first_baseline,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineGridItemFacts {
    pub id: &'static str,
    pub outer_display: InlineOuterDisplay,
    pub item: ItemContributionFacts,
}

impl AtomicInlineGridItemFacts {
    pub const fn inner_context(self) -> InnerFormattingContext {
        self.outer_display.inner_context()
    }
}

pub fn atomic_inline_grid_item_facts(
    wrapper: AtomicInlineWrapperFacts,
    area: GridArea,
    min_content_inline_size: f32,
    max_content_inline_size: f32,
) -> AtomicInlineGridItemFacts {
    AtomicInlineGridItemFacts {
        id: wrapper.id,
        outer_display: wrapper.outer_display,
        item: ItemContributionFacts {
            area,
            min_content: min_content_inline_size,
            max_content: max_content_inline_size,
            preferred: ContributionSize::Definite(wrapper.outer_size.width),
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Infinite,
            margin_before: wrapper.margin.left,
            margin_after: wrapper.margin.right,
            automatic_minimum_applies: true,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineAvailable {
    Definite(f32),
    MinContent,
    MaxContent,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomicInlineInput {
    pub available_width: InlineAvailable,
    pub items: Vec<AtomicInlineItemFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlineLine {
    pub start_item: usize,
    pub end_item: usize,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub descent: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicInlinePositionedItem {
    pub id: &'static str,
    pub line_index: usize,
    pub location: InlinePoint,
    pub size: InlineSize,
    pub margin: InlineEdges,
    pub first_baseline: f32,
    pub synthesized_baseline: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomicInlineReport {
    pub size: InlineSize,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
    pub lines: Vec<AtomicInlineLine>,
    pub items: Vec<AtomicInlinePositionedItem>,
}

pub fn layout_atomic_inline(input: AtomicInlineInput) -> AtomicInlineReport {
    let metrics = input
        .items
        .iter()
        .copied()
        .map(AtomicInlineMetrics::from_item)
        .collect::<Vec<_>>();
    let line_ranges = line_ranges(&metrics, input.available_width);
    build_report(&input.items, &metrics, &line_ranges)
}

pub fn atomic_inline_min_content_width(items: &[AtomicInlineItemFacts]) -> f32 {
    items
        .iter()
        .copied()
        .map(AtomicInlineMetrics::from_item)
        .map(|metrics| metrics.advance)
        .fold(0.0, f32::max)
}

pub fn atomic_inline_max_content_width(items: &[AtomicInlineItemFacts]) -> f32 {
    items
        .iter()
        .copied()
        .map(AtomicInlineMetrics::from_item)
        .map(|metrics| metrics.advance)
        .sum()
}

fn line_ranges(metrics: &[AtomicInlineMetrics], available: InlineAvailable) -> Vec<(usize, usize)> {
    if metrics.is_empty() {
        return Vec::new();
    }

    let Some(width) = wrap_width(metrics, available) else {
        return vec![(0, metrics.len())];
    };

    let mut ranges = Vec::new();
    let mut start = 0;
    let mut current = 0.0;
    for (index, item) in metrics.iter().enumerate() {
        if index > start && current + item.advance > width {
            ranges.push((start, index));
            start = index;
            current = 0.0;
        }
        current += item.advance;
    }
    ranges.push((start, metrics.len()));
    ranges
}

fn wrap_width(metrics: &[AtomicInlineMetrics], available: InlineAvailable) -> Option<f32> {
    match available {
        InlineAvailable::Definite(width) => Some(width),
        InlineAvailable::MinContent => Some(
            metrics
                .iter()
                .map(|metrics| metrics.advance)
                .fold(0.0, f32::max),
        ),
        InlineAvailable::MaxContent => None,
    }
}

fn build_report(
    items: &[AtomicInlineItemFacts],
    metrics: &[AtomicInlineMetrics],
    line_ranges: &[(usize, usize)],
) -> AtomicInlineReport {
    let mut lines = Vec::with_capacity(line_ranges.len());
    let mut positioned_items = Vec::with_capacity(items.len());
    let mut y = 0.0;
    let mut report_width = 0.0_f32;

    for (line_index, &(start_item, end_item)) in line_ranges.iter().enumerate() {
        let line_metrics = &metrics[start_item..end_item];
        let baseline = line_metrics
            .iter()
            .map(|metrics| metrics.baseline)
            .fold(0.0, f32::max);
        let descent = line_metrics
            .iter()
            .map(|metrics| metrics.descent)
            .fold(0.0, f32::max);
        let width = line_metrics.iter().map(|metrics| metrics.advance).sum();
        let height = baseline + descent;
        let line = AtomicInlineLine {
            start_item,
            end_item,
            y,
            width,
            height,
            baseline,
            descent,
        };

        let mut inline_cursor = 0.0;
        for item_index in start_item..end_item {
            let item = items[item_index];
            let item_metrics = metrics[item_index];
            let first_baseline = item.first_baseline.unwrap_or(item.size.height);
            positioned_items.push(AtomicInlinePositionedItem {
                id: item.id,
                line_index,
                location: InlinePoint::new(
                    inline_cursor + item.margin.left,
                    y + baseline - item_metrics.baseline + item.margin.top,
                ),
                size: item.size,
                margin: item.margin,
                first_baseline,
                synthesized_baseline: item.first_baseline.is_none(),
            });
            inline_cursor += item_metrics.advance;
        }

        lines.push(line);
        report_width = report_width.max(width);
        y += height;
    }

    AtomicInlineReport {
        size: InlineSize::new(report_width, y),
        first_baseline: lines.first().map(|line| line.y + line.baseline),
        last_baseline: lines.last().map(|line| line.y + line.baseline),
        lines,
        items: positioned_items,
    }
}
