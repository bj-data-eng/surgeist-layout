use crate::{
    AspectRatioOf, Edges, LayoutScalar, LengthOf, LengthResolutionOf, LengthResolutionStatus, Size,
};

pub(crate) fn resolution_or_zero<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

pub(crate) fn resolution_optional<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(None),
    }
}

pub(crate) fn resolve_containing_padding_border<S: LayoutScalar, E>(
    containing_flow_axes: crate::geometry::FlowAxes,
    parent: Size<Option<S>>,
    padding: Edges<LengthOf<S>>,
    border: Edges<LengthOf<S>>,
    resolve_length: impl Fn(LengthOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
    mut transpose: impl FnMut(Edges<Result<S, LengthResolutionStatus<S>>>) -> Result<Edges<S>, E>,
) -> Result<(Edges<S>, Edges<S>), E> {
    let padding = transpose(containing_flow_axes.zip_physical_edges_with_inline_extent(
        padding,
        parent,
        &resolve_length,
    ))?;
    let border = transpose(containing_flow_axes.zip_physical_edges_with_inline_extent(
        border,
        parent,
        resolve_length,
    ))?;
    Ok((padding, border))
}

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

mod sealed {
    use crate::{LayoutScalar, Size};

    pub(crate) trait OptionalSizeMax {}

    impl<S: LayoutScalar> OptionalSizeMax for Size<Option<S>> {}

    pub(crate) trait OptionalMinimumSizeFloor {}

    impl<S: LayoutScalar> OptionalMinimumSizeFloor for Size<S> {}
}

pub(crate) trait OptionalSizeMaxExt: sealed::OptionalSizeMax {
    fn max_optional(self, min: Self) -> Self;
}

impl<S: LayoutScalar> OptionalSizeMaxExt for Size<Option<S>> {
    fn max_optional(self, min: Self) -> Self {
        Size::new(
            self.width
                .zip(min.width)
                .map(|(value, min)| value.max(min))
                .or(self.width),
            self.height
                .zip(min.height)
                .map(|(value, min)| value.max(min))
                .or(self.height),
        )
    }
}

pub(crate) trait OptionalMinimumSizeFloorExt: sealed::OptionalMinimumSizeFloor {
    type Scalar: LayoutScalar;

    fn max_optional(self, min: Size<Option<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> OptionalMinimumSizeFloorExt for Size<S> {
    type Scalar = S;

    fn max_optional(self, min: Size<Option<S>>) -> Self {
        Size::new(
            min.width.map_or(self.width, |min| self.width.max(min)),
            min.height.map_or(self.height, |min| self.height.max(min)),
        )
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

#[cfg(test)]
type ResolutionOrZeroFn<S> =
    fn(crate::LengthResolutionOf<S>) -> Result<S, crate::LengthResolutionStatus<S>>;

#[cfg(test)]
type ResolutionOptionalFn<S> =
    fn(crate::LengthResolutionOf<S>) -> Result<Option<S>, crate::LengthResolutionStatus<S>>;

#[cfg(test)]
pub(crate) fn assert_fri06_c13_t06_resolution_policy<S: LayoutScalar>(
    resolution_or_zero: ResolutionOrZeroFn<S>,
    resolution_optional: ResolutionOptionalFn<S>,
) {
    use crate::{LengthResolutionOf, LengthResolutionStatus};

    let resolved = S::from_f64(7.5);
    let negative = S::from_f64(-3.25);
    let invalid = LengthResolutionStatus::InvalidNumeric { value: S::INFINITY };

    assert_eq!(
        resolution_or_zero(LengthResolutionOf::definite(resolved, true)),
        Ok(resolved)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::definite(S::ZERO, false)),
        Ok(S::ZERO)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::definite(negative, false)),
        Ok(negative)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::unresolved(true)),
        Ok(S::ZERO)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::non_numeric()),
        Ok(S::ZERO)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::invalid_numeric(S::INFINITY, true)),
        Err(invalid)
    );

    assert_eq!(
        resolution_optional(LengthResolutionOf::definite(resolved, true)),
        Ok(Some(resolved))
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::definite(S::ZERO, false)),
        Ok(Some(S::ZERO))
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::definite(negative, false)),
        Ok(Some(negative))
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::unresolved(true)),
        Ok(None)
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::non_numeric()),
        Ok(None)
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::invalid_numeric(S::INFINITY, true)),
        Err(invalid)
    );
}

#[cfg(test)]
mod tests {
    use super::{OptionalMinimumSizeFloorExt, OptionalSizeMaxExt};
    use crate::{LayoutScalar, Size};

    fn assert_optional_size_operations<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(None, Some(scalar(12.0))).max_optional(Size::new(None, None)),
            Size::new(None, Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(Some(scalar(4.0)), Some(scalar(12.0)))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(9.0)), Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
    }

    #[test]
    fn optional_size_operations_preserve_f32() {
        assert_optional_size_operations::<f32>();
    }

    #[test]
    fn optional_size_operations_preserve_f64() {
        assert_optional_size_operations::<f64>();
    }
}
