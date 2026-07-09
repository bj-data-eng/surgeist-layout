use super::{DefaultScalar, Direction, LayoutScalar, Overflow, Point, Size, WritingMode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollUnsupportedFeature {
    InvalidScrollRect,
    InvalidScrollRange,
    InvalidScrollGeometry,
    OverflowAuto,
    OverflowClipMargin,
    ScrollbarGutterStable,
    ScrollbarGutterBothEdges,
    ScrollPadding,
    ScrollMargin,
    ScrollSnap,
    LayoutOwnedMixedAxisOverflowCoupling,
}

impl ScrollUnsupportedFeature {
    #[must_use]
    pub const fn is_phase_one_deferred(self) -> bool {
        matches!(
            self,
            Self::OverflowAuto
                | Self::OverflowClipMargin
                | Self::ScrollbarGutterStable
                | Self::ScrollbarGutterBothEdges
                | Self::ScrollPadding
                | Self::ScrollMargin
                | Self::ScrollSnap
                | Self::LayoutOwnedMixedAxisOverflowCoupling
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRectOf<S: LayoutScalar = DefaultScalar> {
    origin: Point<S>,
    size: Size<S>,
}

pub type ScrollRect = ScrollRectOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollRectOf<S> {
    pub fn new(origin: Point<S>, size: Size<S>) -> Result<Self, ScrollUnsupportedFeature> {
        if !origin.x.is_finite()
            || !origin.y.is_finite()
            || !size.width.is_finite()
            || !size.height.is_finite()
            || size.width < S::ZERO
            || size.height < S::ZERO
        {
            return Err(ScrollUnsupportedFeature::InvalidScrollRect);
        }

        Ok(Self { origin, size })
    }

    #[must_use]
    pub const fn origin(self) -> Point<S> {
        self.origin
    }

    #[must_use]
    pub const fn size(self) -> Size<S> {
        self.size
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollOffsetOf<S: LayoutScalar = DefaultScalar> {
    position: Point<S>,
}

pub type ScrollOffset = ScrollOffsetOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollOffsetOf<S> {
    #[must_use]
    pub const fn new(position: Point<S>) -> Self {
        Self { position }
    }

    #[must_use]
    pub const fn position(self) -> Point<S> {
        self.position
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRangeOf<S: LayoutScalar = DefaultScalar> {
    maximum_offset: Size<S>,
}

pub type ScrollRange = ScrollRangeOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollRangeOf<S> {
    pub fn new(maximum_offset: Size<S>) -> Result<Self, ScrollUnsupportedFeature> {
        if !maximum_offset.width.is_finite()
            || !maximum_offset.height.is_finite()
            || maximum_offset.width < S::ZERO
            || maximum_offset.height < S::ZERO
        {
            return Err(ScrollUnsupportedFeature::InvalidScrollRange);
        }

        Ok(Self { maximum_offset })
    }

    #[must_use]
    pub const fn maximum_offset(self) -> Size<S> {
        self.maximum_offset
    }

    #[must_use]
    pub fn clamp(self, offset: ScrollOffsetOf<S>) -> ScrollOffsetOf<S> {
        let position = offset.position();
        ScrollOffsetOf::new(Point::new(
            position.x.max(S::ZERO).min(self.maximum_offset.width),
            position.y.max(S::ZERO).min(self.maximum_offset.height),
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollOverflowExposure {
    Visible,
    ClipOnly,
    ScrollableClip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollContainerAxis {
    exposure: ScrollOverflowExposure,
}

impl ScrollContainerAxis {
    pub const VISIBLE: Self = Self {
        exposure: ScrollOverflowExposure::Visible,
    };

    #[must_use]
    pub const fn exposure(self) -> ScrollOverflowExposure {
        self.exposure
    }

    #[must_use]
    pub const fn exposes_scroll_range(self) -> bool {
        matches!(self.exposure, ScrollOverflowExposure::ScrollableClip)
    }

    #[must_use]
    pub const fn clips_overflow(self) -> bool {
        matches!(
            self.exposure,
            ScrollOverflowExposure::ClipOnly | ScrollOverflowExposure::ScrollableClip
        )
    }

    pub const fn from_overflow(overflow: Overflow) -> Result<Self, ScrollUnsupportedFeature> {
        Ok(Self {
            exposure: match overflow {
                Overflow::Visible => ScrollOverflowExposure::Visible,
                Overflow::Clip => ScrollOverflowExposure::ClipOnly,
                Overflow::Hidden | Overflow::Scroll => ScrollOverflowExposure::ScrollableClip,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollContainerFacts {
    x: ScrollContainerAxis,
    y: ScrollContainerAxis,
}

impl ScrollContainerFacts {
    #[must_use]
    pub const fn new(x: ScrollContainerAxis, y: ScrollContainerAxis) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(self) -> ScrollContainerAxis {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> ScrollContainerAxis {
        self.y
    }

    #[must_use]
    pub fn accepts_range<S: LayoutScalar>(self, range: ScrollRangeOf<S>) -> bool {
        let maximum = range.maximum_offset();
        (self.x.exposes_scroll_range() || maximum.width == S::ZERO)
            && (self.y.exposes_scroll_range() || maximum.height == S::ZERO)
    }

    #[must_use]
    pub const fn requires_overflow_clip(self) -> bool {
        self.x.clips_overflow() || self.y.clips_overflow()
    }

    #[must_use]
    pub const fn accepts_overflow_clip<S: LayoutScalar>(
        self,
        overflow_clip: Option<ScrollRectOf<S>>,
    ) -> bool {
        !self.requires_overflow_clip() || overflow_clip.is_some()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollOverflowCouplingPolicy {
    RootPreResolved,
    LayoutOwnedVisibleToAutoCoupling,
}

impl ScrollOverflowCouplingPolicy {
    pub const PHASE_ONE: Self = Self::RootPreResolved;

    #[must_use]
    pub const fn unsupported_feature(self) -> Option<ScrollUnsupportedFeature> {
        match self {
            Self::RootPreResolved => None,
            Self::LayoutOwnedVisibleToAutoCoupling => {
                Some(ScrollUnsupportedFeature::LayoutOwnedMixedAxisOverflowCoupling)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarGutterRectsOf<S: LayoutScalar = DefaultScalar> {
    horizontal: Option<ScrollRectOf<S>>,
    vertical: Option<ScrollRectOf<S>>,
}

pub type ScrollbarGutterRects = ScrollbarGutterRectsOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarGutterRectsOf<S> {
    #[must_use]
    pub const fn new(
        horizontal: Option<ScrollRectOf<S>>,
        vertical: Option<ScrollRectOf<S>>,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub const fn horizontal(self) -> Option<ScrollRectOf<S>> {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> Option<ScrollRectOf<S>> {
        self.vertical
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollGeometryOf<S: LayoutScalar = DefaultScalar> {
    writing_mode: WritingMode,
    direction: Direction,
    container: ScrollContainerFacts,
    scrollport: ScrollRectOf<S>,
    overflow_clip: Option<ScrollRectOf<S>>,
    scrollable_overflow: ScrollRectOf<S>,
    range: ScrollRangeOf<S>,
    gutters: ScrollbarGutterRectsOf<S>,
}

pub type ScrollGeometry = ScrollGeometryOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollGeometryOf<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        writing_mode: WritingMode,
        direction: Direction,
        container: ScrollContainerFacts,
        scrollport: ScrollRectOf<S>,
        overflow_clip: Option<ScrollRectOf<S>>,
        scrollable_overflow: ScrollRectOf<S>,
        range: ScrollRangeOf<S>,
        gutters: ScrollbarGutterRectsOf<S>,
    ) -> Result<Self, ScrollUnsupportedFeature> {
        if !container.accepts_range(range) {
            return Err(ScrollUnsupportedFeature::InvalidScrollGeometry);
        }
        if !container.accepts_overflow_clip(overflow_clip) {
            return Err(ScrollUnsupportedFeature::InvalidScrollGeometry);
        }

        Ok(Self {
            writing_mode,
            direction,
            container,
            scrollport,
            overflow_clip,
            scrollable_overflow,
            range,
            gutters,
        })
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
    pub const fn container(self) -> ScrollContainerFacts {
        self.container
    }

    #[must_use]
    pub const fn scrollport(self) -> ScrollRectOf<S> {
        self.scrollport
    }

    #[must_use]
    pub const fn overflow_clip(self) -> Option<ScrollRectOf<S>> {
        self.overflow_clip
    }

    #[must_use]
    pub const fn scrollable_overflow(self) -> ScrollRectOf<S> {
        self.scrollable_overflow
    }

    #[must_use]
    pub const fn range(self) -> ScrollRangeOf<S> {
        self.range
    }

    #[must_use]
    pub const fn gutters(self) -> ScrollbarGutterRectsOf<S> {
        self.gutters
    }
}
