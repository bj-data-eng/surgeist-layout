use super::{AvailableOf, DefaultScalar, Edges, LayoutScalar, Point, Size, WritingMode};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct AtomicInlineInput<S: LayoutScalar = DefaultScalar> {
    pub available_width: AvailableOf<S>,
    pub writing_mode: WritingMode,
    pub items: Vec<AtomicInlineItem<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineItem<S: LayoutScalar = DefaultScalar> {
    pub order: u32,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
    pub first_baseline: Option<S>,
}

impl<S: LayoutScalar> AtomicInlineItem<S> {
    #[cfg(test)]
    pub(super) const fn new(
        order: u32,
        size: Size<S>,
        margin: Edges<S>,
        first_baseline: Option<S>,
    ) -> Self {
        Self {
            order,
            size,
            content_size: size,
            margin,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline,
        }
    }

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineLayoutItem<S: LayoutScalar = DefaultScalar> {
    pub order: u32,
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct AtomicInlineReport<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub items: Vec<AtomicInlineLayoutItem<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingInlineItem<S: LayoutScalar = DefaultScalar> {
    item: AtomicInlineItem<S>,
    x: S,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct InlineLine<S: LayoutScalar = DefaultScalar> {
    items: Vec<PendingInlineItem<S>>,
    width: S,
    baseline: S,
    descent: S,
}

impl<S: LayoutScalar> InlineLine<S> {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn push(&mut self, item: AtomicInlineItem<S>) {
        let baseline = item.line_baseline();
        self.baseline = self.baseline.max(baseline);
        self.descent = self.descent.max(item.line_descent());
        self.items.push(PendingInlineItem {
            item,
            x: self.width + item.margin.left,
        });
        self.width = self.width + item.advance();
    }

    #[must_use]
    fn height(&self) -> S {
        self.baseline + self.descent
    }
}

#[must_use]
pub(super) fn layout_atomic_inline_items<S: LayoutScalar>(
    input: AtomicInlineInput<S>,
) -> AtomicInlineReport<S> {
    if input.writing_mode == WritingMode::VerticalRl {
        return layout_vertical_rl_atomic_inline_items(input);
    }

    let available_width = match input.available_width {
        AvailableOf::Definite(width) => Some(width),
        AvailableOf::MinContent => Some(atomic_inline_min_content_width(&input.items)),
        AvailableOf::MaxContent => None,
    };
    let mut lines = Vec::new();
    let mut line = InlineLine::<S>::default();

    for item in input.items {
        let advance = item.advance();
        if let Some(available_width) = available_width
            && !line.is_empty()
            && line.width + advance > available_width
        {
            lines.push(line);
            line = InlineLine::<S>::default();
        }

        line.push(item);
    }

    if !line.is_empty() {
        lines.push(line);
    }

    let mut y = S::ZERO;
    let mut width = S::ZERO;
    let mut items = Vec::new();
    let mut first_baseline = None;
    let mut last_baseline = None;

    for line in lines {
        width = width.max(line.width);
        let line_baseline = y + line.baseline;
        let line_height = line.height();
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);

        for pending in line.items {
            let item = pending.item;
            items.push(AtomicInlineLayoutItem {
                order: item.order,
                location: Point::new(pending.x, y + line.baseline - item.baseline()),
                size: item.size,
                content_size: item.content_size,
                margin: item.margin,
                padding: item.padding,
                border: item.border,
                scrollbar_size: item.scrollbar_size,
            });
        }

        y = y + line_height;
    }

    let content_size = Size::new(width, y);

    AtomicInlineReport {
        size: content_size,
        content_size,
        first_baseline,
        last_baseline,
        items,
    }
}

fn layout_vertical_rl_atomic_inline_items<S: LayoutScalar>(
    input: AtomicInlineInput<S>,
) -> AtomicInlineReport<S> {
    let line_width = input
        .items
        .iter()
        .map(|item| item.margin.left + item.size.width + item.margin.right)
        .fold(S::ZERO, S::max);
    let container_width = match input.available_width {
        AvailableOf::Definite(width) => width.max(line_width),
        AvailableOf::MinContent | AvailableOf::MaxContent => line_width,
    };
    let line_x = (container_width - line_width).max(S::ZERO);
    let mut y = S::ZERO;
    let mut items = Vec::with_capacity(input.items.len());
    let mut first_baseline = None;
    let mut last_baseline = None;

    for item in input.items {
        y = y + item.margin.top;
        let mut item_x = line_x + line_width - item.margin.right - item.size.width;
        if item.size.height == S::ZERO {
            item_x = item_x + item.size.width / S::from_f64(2.0);
        }
        let baseline = y + item.baseline();
        first_baseline.get_or_insert(baseline);
        last_baseline = Some(baseline);
        items.push(AtomicInlineLayoutItem {
            order: item.order,
            location: Point::new(item_x, y),
            size: item.size,
            content_size: item.content_size,
            margin: item.margin,
            padding: item.padding,
            border: item.border,
            scrollbar_size: item.scrollbar_size,
        });
        y = y + item.size.height + item.margin.bottom;
    }

    let content_size = Size::new(container_width, y);

    AtomicInlineReport {
        size: content_size,
        content_size,
        first_baseline,
        last_baseline,
        items,
    }
}

#[must_use]
pub(super) fn atomic_inline_min_content_width<S: LayoutScalar>(items: &[AtomicInlineItem<S>]) -> S {
    items
        .iter()
        .map(|item| item.advance())
        .fold(S::ZERO, S::max)
}

#[must_use]
#[cfg(test)]
pub(super) fn atomic_inline_max_content_width<S: LayoutScalar>(items: &[AtomicInlineItem<S>]) -> S {
    items
        .iter()
        .map(|item| item.advance())
        .fold(S::ZERO, |sum, advance| sum + advance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Available;

    #[test]
    fn atomic_inline_line_aligns_items_to_max_baseline() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::definite(200.0),
            writing_mode: WritingMode::HorizontalTb,
            items: vec![
                AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(7.0)),
                AtomicInlineItem::new(1, Size::new(10.0, 20.0), Edges::ZERO, Some(12.0)),
            ],
        });

        assert_eq!(report.size, Size::new(30.0, 20.0));
        assert_eq!(report.first_baseline, Some(12.0));
        assert_eq!(report.items[0].location, Point::new(0.0, 5.0));
        assert_eq!(report.items[1].location, Point::new(20.0, 0.0));
    }

    #[test]
    fn atomic_inline_items_wrap_between_items_for_definite_width() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::definite(25.0),
            writing_mode: WritingMode::HorizontalTb,
            items: vec![
                AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
                AtomicInlineItem::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            ],
        });

        assert_eq!(report.size, Size::new(20.0, 20.0));
        assert_eq!(report.first_baseline, Some(10.0));
        assert_eq!(report.last_baseline, Some(20.0));
        assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
    }

    #[test]
    fn atomic_inline_line_geometry_clamps_item_baseline_to_its_box() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::MAX_CONTENT,
            writing_mode: WritingMode::HorizontalTb,
            items: vec![
                AtomicInlineItem::new(0, Size::new(124.0, 64.0), Edges::ZERO, Some(94.0)),
                AtomicInlineItem::new(1, Size::new(10.0, 0.0), Edges::ZERO, Some(0.0)),
            ],
        });

        assert_eq!(report.size, Size::new(134.0, 64.0));
        assert_eq!(report.first_baseline, Some(64.0));
        assert_eq!(report.items[0].location, Point::new(0.0, 0.0));
        assert_eq!(report.items[1].location, Point::new(124.0, 64.0));
    }

    #[test]
    fn atomic_inline_min_content_available_wraps_to_max_item_advance() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::MIN_CONTENT,
            writing_mode: WritingMode::HorizontalTb,
            items: vec![
                AtomicInlineItem::new(0, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
                AtomicInlineItem::new(1, Size::new(60.0, 10.0), Edges::ZERO, Some(10.0)),
                AtomicInlineItem::new(2, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            ],
        });

        assert_eq!(report.size, Size::new(60.0, 30.0));
        assert_eq!(report.first_baseline, Some(10.0));
        assert_eq!(report.last_baseline, Some(30.0));
        assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
        assert_eq!(report.items[2].location, Point::new(0.0, 20.0));
    }

    #[test]
    fn atomic_inline_intrinsic_widths_use_max_item_and_sum() {
        let items = vec![
            AtomicInlineItem::new(
                0,
                Size::new(25.0, 10.0),
                Edges::new(0.0, 5.0, 0.0, 5.0),
                Some(10.0),
            ),
            AtomicInlineItem::new(
                1,
                Size::new(100.0, 10.0),
                Edges::new(0.0, 0.0, 0.0, 10.0),
                Some(10.0),
            ),
            AtomicInlineItem::new(2, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
        ];

        assert_eq!(atomic_inline_min_content_width(&items), 110.0);
        assert_eq!(atomic_inline_max_content_width(&items), 195.0);
    }

    #[test]
    fn atomic_inline_vertical_margins_participate_in_line_metrics() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::MAX_CONTENT,
            writing_mode: WritingMode::HorizontalTb,
            items: vec![AtomicInlineItem::new(
                0,
                Size::new(20.0, 10.0),
                Edges::new(3.0, 0.0, 7.0, 0.0),
                Some(6.0),
            )],
        });

        assert_eq!(report.size, Size::new(20.0, 20.0));
        assert_eq!(report.first_baseline, Some(9.0));
        assert_eq!(report.last_baseline, Some(9.0));
        assert_eq!(report.items[0].location, Point::new(0.0, 3.0));
    }

    #[test]
    fn atomic_inline_vertical_rl_places_line_against_right_edge() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::definite(70.0),
            writing_mode: WritingMode::VerticalRl,
            items: vec![
                AtomicInlineItem::new(0, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
                AtomicInlineItem::new(1, Size::new(10.0, 0.0), Edges::ZERO, Some(0.0)),
                AtomicInlineItem::new(2, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
            ],
        });

        assert_eq!(report.size, Size::new(70.0, 40.0));
        assert_eq!(report.items[0].location, Point::new(50.0, 0.0));
        assert_eq!(report.items[1].location, Point::new(65.0, 20.0));
        assert_eq!(report.items[2].location, Point::new(50.0, 20.0));
    }

    #[test]
    fn atomic_inline_empty_items_report_zero_size_and_no_baselines() {
        let report = layout_atomic_inline_items(AtomicInlineInput {
            available_width: Available::MAX_CONTENT,
            writing_mode: WritingMode::HorizontalTb,
            items: Vec::new(),
        });

        assert_eq!(report.size, Size::ZERO);
        assert_eq!(report.content_size, Size::ZERO);
        assert_eq!(report.first_baseline, None);
        assert_eq!(report.last_baseline, None);
        assert!(report.items.is_empty());
    }
}
