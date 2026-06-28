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
