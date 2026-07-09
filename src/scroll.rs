use super::{DefaultScalar, LayoutScalar, Point, Size};

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
