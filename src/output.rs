use super::{AvailableOf, DefaultScalar, Edges, LayoutScalar, Point, Size};

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
pub struct ComputeInputOf<S: LayoutScalar = DefaultScalar> {
    pub run_mode: RunMode,
    pub sizing_mode: SizingMode,
    pub axis: RequestedAxis,
    pub known: Size<Option<S>>,
    pub parent: Size<Option<S>>,
    pub available: Size<AvailableOf<S>>,
}

pub type ComputeInput = ComputeInputOf<DefaultScalar>;

impl<S: LayoutScalar> ComputeInputOf<S> {
    pub const HIDDEN: Self = Self {
        run_mode: RunMode::PerformHiddenLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::NONE,
        available: Size::splat(AvailableOf::MAX_CONTENT),
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollapsibleMarginOf<S: LayoutScalar = DefaultScalar> {
    positive: S,
    negative: S,
}

pub type CollapsibleMargin = CollapsibleMarginOf<DefaultScalar>;

impl<S: LayoutScalar> CollapsibleMarginOf<S> {
    pub const ZERO: Self = Self {
        positive: S::ZERO,
        negative: S::ZERO,
    };

    #[must_use]
    pub fn from_margin(margin: S) -> Self {
        if margin >= S::ZERO {
            Self {
                positive: margin,
                negative: S::ZERO,
            }
        } else {
            Self {
                positive: S::ZERO,
                negative: margin,
            }
        }
    }

    #[must_use]
    pub fn collapse_with_margin(self, margin: S) -> Self {
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
    pub fn resolve(self) -> S {
        self.positive + self.negative
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselinesOf<S: LayoutScalar = DefaultScalar> {
    pub first: Point<Option<S>>,
    pub last: Point<Option<S>>,
}

pub type Baselines = BaselinesOf<DefaultScalar>;

impl<S: LayoutScalar> BaselinesOf<S> {
    pub const NONE: Self = Self {
        first: Point::NONE,
        last: Point::NONE,
    };

    #[must_use]
    pub const fn first(first: Point<Option<S>>) -> Self {
        Self {
            first,
            last: Point::NONE,
        }
    }

    #[must_use]
    pub const fn synthesized(size: Size<S>) -> Self {
        Self {
            first: Point::new(Some(size.width), Some(size.height)),
            last: Point::new(Some(S::ZERO), Some(S::ZERO)),
        }
    }

    #[must_use]
    pub fn first_or_synthesize_block(self, size: Size<S>) -> S {
        self.first.y.unwrap_or(size.height)
    }

    #[must_use]
    pub fn last_or_synthesize_block(self, _size: Size<S>) -> S {
        self.last.y.unwrap_or(S::ZERO)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeOutputOf<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub first_baselines: Point<Option<S>>,
    pub last_baselines: Point<Option<S>>,
    pub top_margin: CollapsibleMarginOf<S>,
    pub bottom_margin: CollapsibleMarginOf<S>,
    pub margins_can_collapse_through: bool,
}

pub type ComputeOutput = ComputeOutputOf<DefaultScalar>;

impl<S: LayoutScalar> ComputeOutputOf<S> {
    pub const HIDDEN: Self = Self {
        size: Size::<S>::ZERO,
        content_size: Size::<S>::ZERO,
        first_baselines: Point::NONE,
        last_baselines: Point::NONE,
        top_margin: CollapsibleMarginOf::ZERO,
        bottom_margin: CollapsibleMarginOf::ZERO,
        margins_can_collapse_through: false,
    };

    pub const DEFAULT: Self = Self::HIDDEN;

    #[must_use]
    pub const fn from_sizes_and_baselines(
        size: Size<S>,
        content_size: Size<S>,
        baselines: BaselinesOf<S>,
    ) -> Self {
        Self {
            size,
            content_size,
            first_baselines: baselines.first,
            last_baselines: baselines.last,
            top_margin: CollapsibleMarginOf::ZERO,
            bottom_margin: CollapsibleMarginOf::ZERO,
            margins_can_collapse_through: false,
        }
    }

    #[must_use]
    pub const fn from_sizes_and_first_baselines(
        size: Size<S>,
        content_size: Size<S>,
        first_baselines: Point<Option<S>>,
    ) -> Self {
        Self::from_sizes_and_baselines(size, content_size, BaselinesOf::first(first_baselines))
    }

    #[must_use]
    pub const fn from_sizes(size: Size<S>, content_size: Size<S>) -> Self {
        Self::from_sizes_and_baselines(size, content_size, BaselinesOf::NONE)
    }

    #[must_use]
    pub const fn from_outer_size(size: Size<S>) -> Self {
        Self::from_sizes(size, Size::<S>::ZERO)
    }

    #[must_use]
    pub const fn baselines(&self) -> BaselinesOf<S> {
        BaselinesOf {
            first: self.first_baselines,
            last: self.last_baselines,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeOutputOf<S: LayoutScalar = DefaultScalar> {
    pub order: u32,
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub scrollbar_size: Size<S>,
    pub border: Edges<S>,
    pub padding: Edges<S>,
    pub margin: Edges<S>,
}

pub type NodeOutput = NodeOutputOf<DefaultScalar>;

impl<S: LayoutScalar> NodeOutputOf<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self::with_order(0)
    }

    #[must_use]
    pub const fn with_order(order: u32) -> Self {
        Self {
            order,
            location: Point::<S>::ZERO,
            size: Size::<S>::ZERO,
            content_size: Size::<S>::ZERO,
            scrollbar_size: Size::<S>::ZERO,
            border: Edges::<S>::ZERO,
            padding: Edges::<S>::ZERO,
            margin: Edges::<S>::ZERO,
        }
    }

    #[must_use]
    pub fn content_box_size(self) -> Size<S> {
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

impl<S: LayoutScalar> Default for NodeOutputOf<S> {
    fn default() -> Self {
        Self::new()
    }
}
