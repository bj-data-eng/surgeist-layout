use crate::{AspectRatioOf, LayoutScalar, Size};

pub(crate) trait OptionalSizeExt {
    type Scalar: LayoutScalar;

    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size<Self::Scalar>) -> Size<Self::Scalar>;
    fn add_optional(self, amount: Size<Self::Scalar>) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> OptionalSizeExt for Size<Option<S>> {
    type Scalar = S;

    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size<S>) -> Size<S> {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| width + amount.width),
            self.height.map(|height| height + amount.height),
        )
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<S>>) -> Self {
        let Some(ratio) = aspect_ratio else {
            return self;
        };
        let ratio = ratio.get();
        match (self.width, self.height) {
            (Some(width), None) => Size::new(Some(width), Some(width / ratio)),
            (None, Some(height)) => Size::new(Some(height * ratio), Some(height)),
            _ => self,
        }
    }
}

pub(crate) trait UncheckedOptionalSizeSubExt {
    type Scalar: LayoutScalar;

    fn sub_optional_unchecked(self, amount: Size<Self::Scalar>) -> Self;
}

impl<S: LayoutScalar> UncheckedOptionalSizeSubExt for Size<Option<S>> {
    type Scalar = S;

    fn sub_optional_unchecked(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| width - amount.width),
            self.height.map(|height| height - amount.height),
        )
    }
}

pub(crate) trait MaxBeforeMinScalarClampExt {
    fn clamp_max_before_min_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl<S: LayoutScalar> MaxBeforeMinScalarClampExt for S {
    fn clamp_max_before_min_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}

pub(crate) trait MaxBeforeMinSizeClampExt {
    type Scalar: LayoutScalar;

    fn clamp_max_before_min_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
}

impl<S: LayoutScalar> MaxBeforeMinSizeClampExt for Size<S> {
    type Scalar = S;

    fn clamp_max_before_min_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width
                .clamp_max_before_min_optional(min.width, max.width),
            self.height
                .clamp_max_before_min_optional(min.height, max.height),
        )
    }
}

pub(crate) trait MaxBeforeMinOptionalSizeClampExt {
    type Scalar: LayoutScalar;

    fn clamp_max_before_min_optional(self, min: Self, max: Self) -> Self;
}

impl<S: LayoutScalar> MaxBeforeMinOptionalSizeClampExt for Size<Option<S>> {
    type Scalar = S;

    fn clamp_max_before_min_optional(self, min: Self, max: Self) -> Self {
        Size::new(
            self.width
                .map(|value| value.clamp_max_before_min_optional(min.width, max.width)),
            self.height
                .map(|value| value.clamp_max_before_min_optional(min.height, max.height)),
        )
    }
}
