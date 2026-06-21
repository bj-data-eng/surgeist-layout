use super::{Available, Edges, Point, Scalar, Size};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunMode {
    PerformRootLayout,
    PerformLayout,
    ComputeSize,
    PerformHiddenLayout,
}

impl RunMode {
    pub const fn is_perform_layout(self) -> bool {
        matches!(self, Self::PerformRootLayout | Self::PerformLayout)
    }

    pub const fn for_child(self) -> Self {
        match self {
            Self::PerformRootLayout => Self::PerformLayout,
            mode => mode,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SizingMode {
    ContentSize,
    InherentSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestedAxis {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeInput {
    pub run_mode: RunMode,
    pub sizing_mode: SizingMode,
    pub axis: RequestedAxis,
    pub known: Size<Option<Scalar>>,
    pub parent: Size<Option<Scalar>>,
    pub available: Size<Available>,
}

impl ComputeInput {
    pub const HIDDEN: Self = Self {
        run_mode: RunMode::PerformHiddenLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::NONE,
        available: Size::splat(Available::MAX_CONTENT),
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollapsibleMargin {
    positive: Scalar,
    negative: Scalar,
}

impl CollapsibleMargin {
    pub const ZERO: Self = Self {
        positive: 0.0,
        negative: 0.0,
    };

    #[must_use]
    pub fn from_margin(margin: Scalar) -> Self {
        if margin >= 0.0 {
            Self {
                positive: margin,
                negative: 0.0,
            }
        } else {
            Self {
                positive: 0.0,
                negative: margin,
            }
        }
    }

    #[must_use]
    pub fn collapse_with_margin(self, margin: Scalar) -> Self {
        self.collapse_with(Self::from_margin(margin))
    }

    #[must_use]
    pub fn collapse_with(self, other: Self) -> Self {
        Self {
            positive: self.positive.max(other.positive),
            negative: self.negative.min(other.negative),
        }
    }

    #[must_use]
    pub fn resolve(self) -> Scalar {
        self.positive + self.negative
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Baselines {
    pub first: Point<Option<Scalar>>,
    pub last: Point<Option<Scalar>>,
}

impl Baselines {
    pub const NONE: Self = Self {
        first: Point::NONE,
        last: Point::NONE,
    };

    #[must_use]
    pub const fn first(first: Point<Option<Scalar>>) -> Self {
        Self {
            first,
            last: Point::NONE,
        }
    }

    #[must_use]
    pub const fn synthesized(size: Size) -> Self {
        Self {
            first: Point::new(Some(size.width), Some(size.height)),
            last: Point::new(Some(0.0), Some(0.0)),
        }
    }

    #[must_use]
    pub fn first_or_synthesize_block(self, size: Size) -> Scalar {
        self.first.y.unwrap_or(size.height)
    }

    #[must_use]
    pub fn last_or_synthesize_block(self, _size: Size) -> Scalar {
        self.last.y.unwrap_or(0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeOutput {
    pub size: Size,
    pub content_size: Size,
    pub first_baselines: Point<Option<Scalar>>,
    pub last_baselines: Point<Option<Scalar>>,
    pub top_margin: CollapsibleMargin,
    pub bottom_margin: CollapsibleMargin,
    pub margins_can_collapse_through: bool,
}

impl ComputeOutput {
    pub const HIDDEN: Self = Self {
        size: Size::ZERO,
        content_size: Size::ZERO,
        first_baselines: Point::NONE,
        last_baselines: Point::NONE,
        top_margin: CollapsibleMargin::ZERO,
        bottom_margin: CollapsibleMargin::ZERO,
        margins_can_collapse_through: false,
    };

    pub const DEFAULT: Self = Self::HIDDEN;

    #[must_use]
    pub const fn from_sizes_and_baselines(
        size: Size,
        content_size: Size,
        baselines: Baselines,
    ) -> Self {
        Self {
            size,
            content_size,
            first_baselines: baselines.first,
            last_baselines: baselines.last,
            top_margin: CollapsibleMargin::ZERO,
            bottom_margin: CollapsibleMargin::ZERO,
            margins_can_collapse_through: false,
        }
    }

    #[must_use]
    pub const fn from_sizes_and_first_baselines(
        size: Size,
        content_size: Size,
        first_baselines: Point<Option<Scalar>>,
    ) -> Self {
        Self::from_sizes_and_baselines(size, content_size, Baselines::first(first_baselines))
    }

    #[must_use]
    pub const fn from_sizes(size: Size, content_size: Size) -> Self {
        Self::from_sizes_and_baselines(size, content_size, Baselines::NONE)
    }

    #[must_use]
    pub const fn from_outer_size(size: Size) -> Self {
        Self::from_sizes(size, Size::ZERO)
    }

    #[must_use]
    pub const fn baselines(&self) -> Baselines {
        Baselines {
            first: self.first_baselines,
            last: self.last_baselines,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeOutput {
    pub order: u32,
    pub location: Point,
    pub size: Size,
    pub content_size: Size,
    pub scrollbar_size: Size,
    pub border: Edges,
    pub padding: Edges,
    pub margin: Edges,
}

impl NodeOutput {
    #[must_use]
    pub const fn new() -> Self {
        Self::with_order(0)
    }

    #[must_use]
    pub const fn with_order(order: u32) -> Self {
        Self {
            order,
            location: Point::ZERO,
            size: Size::ZERO,
            content_size: Size::ZERO,
            scrollbar_size: Size::ZERO,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
        }
    }

    #[must_use]
    pub fn content_box_size(self) -> Size {
        Size::new(
            self.size.width
                - self.padding.left
                - self.padding.right
                - self.border.left
                - self.border.right,
            self.size.height
                - self.padding.top
                - self.padding.bottom
                - self.border.top
                - self.border.bottom,
        )
    }
}

impl Default for NodeOutput {
    fn default() -> Self {
        Self::new()
    }
}
