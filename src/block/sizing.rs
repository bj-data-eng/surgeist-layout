use crate::geometry::PhysicalAxis;
use crate::layout_math::{resolution_optional, resolution_or_zero};
use crate::sizing::resolve::{
    SizingResolutionError, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::{
    LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionStatus, MaxSizeOf, MinSizeOf,
    PreferredSizeOf, Size, SizingAlgorithm,
};

pub(super) fn max_content_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

pub(super) fn preferred_size<S: LayoutScalar>(
    size: &Size<PreferredSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_preferred_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_preferred_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

pub(super) fn minimum_size<S: LayoutScalar>(
    size: &Size<MinSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_minimum_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_minimum_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

pub(super) fn maximum_size<S: LayoutScalar>(
    size: &Size<MaxSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    missing_basis_is_indefinite: bool,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    Size::new(
        resolve_maximum_optional(
            &size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            basis.width,
            missing_basis_is_indefinite,
        ),
        resolve_maximum_optional(
            &size.height,
            algorithm,
            PhysicalAxis::Vertical,
            basis.height,
            missing_basis_is_indefinite,
        ),
    )
}

pub(super) fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

pub(super) fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

pub(super) trait BlockOptionalSizeSubExt<S: LayoutScalar> {
    fn sub_optional_clamped_to_zero(self, amount: Size<S>) -> Self;
}

impl<S: LayoutScalar> BlockOptionalSizeSubExt<S> for Size<Option<S>> {
    fn sub_optional_clamped_to_zero(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| (width - amount.width).max(S::ZERO)),
            self.height
                .map(|height| (height - amount.height).max(S::ZERO)),
        )
    }
}

#[cfg(test)]
mod characterization_tests {
    use super::*;
    use crate::AspectRatioOf;
    use crate::layout_math::{
        MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt, OptionalSizeExt, OptionalSizeMaxExt,
    };

    fn characterize_optional_max<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(None, Some(scalar(12.0))).max_optional(Size::new(Some(scalar(9.0)), None)),
            Size::new(None, Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(Some(scalar(4.0)), Some(scalar(12.0)))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(9.0)), Some(scalar(12.0)))
        );
    }

    fn characterize_optional_math<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let optional = Size::new(Some(scalar(8.0)), None);

        assert_eq!(
            optional.or(Size::new(Some(scalar(3.0)), Some(scalar(5.0)))),
            Size::new(Some(scalar(8.0)), Some(scalar(5.0)))
        );
        assert_eq!(
            optional.unwrap_or(Size::new(scalar(13.0), scalar(21.0))),
            Size::new(scalar(8.0), scalar(21.0))
        );
        assert_eq!(
            optional.add_optional(Size::new(scalar(2.0), scalar(3.0))),
            Size::new(Some(scalar(10.0)), None)
        );

        let Some(ratio) = AspectRatioOf::new(scalar(2.0)) else {
            panic!("finite positive test aspect ratio must be accepted");
        };
        assert_eq!(
            Size::new(Some(scalar(12.0)), None).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(12.0)), Some(scalar(6.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(7.0))).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(14.0)), Some(scalar(7.0)))
        );

        assert_eq!(
            Size::new(Some(scalar(2.0)), Some(scalar(9.0)))
                .sub_optional_clamped_to_zero(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(Some(S::ZERO), Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            scalar(5.0).clamp_max_before_min_optional(Some(scalar(10.0)), Some(scalar(3.0))),
            scalar(10.0)
        );
    }

    #[test]
    fn fri08_c07_t03_optional_math_block_componentwise_max_preserves_f32() {
        characterize_optional_max::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_block_componentwise_max_preserves_f64() {
        characterize_optional_max::<f64>();
    }

    #[test]
    fn fri06_c13_t05_block_optional_math_and_zero_clamp_preserve_f32() {
        characterize_optional_math::<f32>();
    }

    #[test]
    fn fri06_c13_t05_block_optional_math_and_zero_clamp_preserve_f64() {
        characterize_optional_math::<f64>();
    }
}
