use crate::{
    DefaultScalar, FiniteScalarErrorOf, LayoutScalar, LengthPercentageOf, LengthResolutionOf,
    NonNegativeFiniteOf, NumericResolutionOf, PercentageBasisOf,
};
use core::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SizingCalculationError {
    EmptyArguments,
}

impl core::fmt::Display for SizingCalculationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyArguments => f.write_str("sizing calculation arguments must not be empty"),
        }
    }
}

impl std::error::Error for SizingCalculationError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CalcSizeCalculationErrorOf<S: LayoutScalar> {
    InvalidAbsolutePx(FiniteScalarErrorOf<S>),
    InvalidPercentFraction(FiniteScalarErrorOf<S>),
    InvalidSizeFraction(FiniteScalarErrorOf<S>),
}

impl<S: LayoutScalar> core::fmt::Display for CalcSizeCalculationErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidAbsolutePx(_) => f.write_str("absolute length coefficient must be finite"),
            Self::InvalidPercentFraction(_) => f.write_str("percentage coefficient must be finite"),
            Self::InvalidSizeFraction(_) => f.write_str("size coefficient must be finite"),
        }
    }
}

impl<S: LayoutScalar> std::error::Error for CalcSizeCalculationErrorOf<S> {}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Instruction<S: LayoutScalar> {
    Value(LengthPercentageOf<S>),
    Min(NonZeroUsize),
    Max(NonZeroUsize),
    Clamp {
        has_minimum: bool,
        has_maximum: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SizingCalculationOf<S: LayoutScalar> {
    instructions: Vec<Instruction<S>>,
    depends_on_basis: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CalcSizeCoefficients<S: LayoutScalar> {
    absolute_px: S,
    percent_fraction: S,
    size_fraction: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CalcSizeInstruction<S: LayoutScalar> {
    Value(CalcSizeCoefficients<S>),
    Min(NonZeroUsize),
    Max(NonZeroUsize),
    Clamp {
        has_minimum: bool,
        has_maximum: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalcSizeCalculationOf<S: LayoutScalar> {
    instructions: Vec<CalcSizeInstruction<S>>,
    depends_on_percentage_basis: bool,
    depends_on_size: bool,
}

pub type SizingCalculation = SizingCalculationOf<DefaultScalar>;
pub type CalcSizeCalculation = CalcSizeCalculationOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CalcSizeConstructionError {
    SizeReferenceWithAnyBasis,
}

impl core::fmt::Display for CalcSizeConstructionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SizeReferenceWithAnyBasis => {
                f.write_str("an Any calc-size basis cannot be combined with a size reference")
            }
        }
    }
}

impl std::error::Error for CalcSizeConstructionError {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PreferredSizeCalcBasis {
    Any,
    FullPercentage,
    Auto,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MinSizeCalcBasis {
    Any,
    FullPercentage,
    Auto,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaxSizeCalcBasis {
    Any,
    FullPercentage,
    None,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FlexBasisCalcBasis {
    Any,
    FullPercentage,
    Auto,
    Content,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

#[derive(Clone, Debug, PartialEq)]
enum PreferredSizeValue<S: LayoutScalar> {
    Zero,
    Calculation(SizingCalculationOf<S>),
    Auto,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(SizingCalculationOf<S>),
    CalcSize(PreferredSizeCalcBasis, CalcSizeCalculationOf<S>),
}

#[derive(Clone, Debug, PartialEq)]
enum MinSizeValue<S: LayoutScalar> {
    Zero,
    Calculation(SizingCalculationOf<S>),
    Auto,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(SizingCalculationOf<S>),
    CalcSize(MinSizeCalcBasis, CalcSizeCalculationOf<S>),
}

#[derive(Clone, Debug, PartialEq)]
enum MaxSizeValue<S: LayoutScalar> {
    Zero,
    Calculation(SizingCalculationOf<S>),
    None,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(SizingCalculationOf<S>),
    CalcSize(MaxSizeCalcBasis, CalcSizeCalculationOf<S>),
}

#[derive(Clone, Debug, PartialEq)]
enum FlexBasisValue<S: LayoutScalar> {
    Zero,
    Calculation(SizingCalculationOf<S>),
    Auto,
    Content,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(SizingCalculationOf<S>),
    CalcSize(FlexBasisCalcBasis, CalcSizeCalculationOf<S>),
}

/// A closed preferred-size value.
///
/// ```compile_fail
/// use surgeist_layout::PreferredSize;
/// let _ = PreferredSize::fr(1.0);
/// ```
///
/// ```compile_fail
/// use surgeist_layout::PreferredSize;
/// let _ = PreferredSize::CONTENT;
/// ```
///
/// ```compile_fail
/// use surgeist_layout::PreferredSize;
/// fn requires_copy<T: Copy>() {}
/// requires_copy::<PreferredSize>();
/// ```
///
/// ```
/// use surgeist_layout::{
///     CalcSizeCalculation, FlexBasis, MaxSize, MinSize, PreferredSize,
///     SizingCalculation,
/// };
///
/// let _preferred = PreferredSize::AUTO;
/// let _minimum = MinSize::ZERO;
/// let _maximum = MaxSize::NONE;
/// let _flex_basis = FlexBasis::CONTENT;
/// let _calculation: SizingCalculation = SizingCalculation::value(
///     surgeist_layout::LengthPercentageOf::ZERO,
/// );
/// let _calc_size = CalcSizeCalculation::size();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PreferredSizeOf<S: LayoutScalar = DefaultScalar> {
    value: PreferredSizeValue<S>,
}

/// A closed minimum-size value.
#[derive(Clone, Debug, PartialEq)]
pub struct MinSizeOf<S: LayoutScalar = DefaultScalar> {
    value: MinSizeValue<S>,
}

/// A closed maximum-size value.
///
/// ```compile_fail
/// use surgeist_layout::MaxSize;
/// let _ = MaxSize::AUTO;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MaxSizeOf<S: LayoutScalar = DefaultScalar> {
    value: MaxSizeValue<S>,
}

/// A closed flex-basis value.
///
/// ```compile_fail
/// use surgeist_layout::{FlexBasis, PreferredSize};
/// let _: PreferredSize = FlexBasis::AUTO.into();
/// ```
///
/// ```compile_fail
/// use surgeist_layout::FlexBasis;
/// let _ = FlexBasis::fr(1.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FlexBasisOf<S: LayoutScalar = DefaultScalar> {
    value: FlexBasisValue<S>,
}

pub type PreferredSize = PreferredSizeOf<DefaultScalar>;
pub type MinSize = MinSizeOf<DefaultScalar>;
pub type MaxSize = MaxSizeOf<DefaultScalar>;
pub type FlexBasis = FlexBasisOf<DefaultScalar>;

#[derive(Clone, Copy, Debug)]
pub(crate) enum PreferredSizeView<'a, S: LayoutScalar> {
    Zero,
    Calculation(&'a SizingCalculationOf<S>),
    Auto,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(&'a SizingCalculationOf<S>),
    CalcSize(PreferredSizeCalcBasis, &'a CalcSizeCalculationOf<S>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum MinSizeView<'a, S: LayoutScalar> {
    Zero,
    Calculation(&'a SizingCalculationOf<S>),
    Auto,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(&'a SizingCalculationOf<S>),
    CalcSize(MinSizeCalcBasis, &'a CalcSizeCalculationOf<S>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum MaxSizeView<'a, S: LayoutScalar> {
    Zero,
    Calculation(&'a SizingCalculationOf<S>),
    None,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(&'a SizingCalculationOf<S>),
    CalcSize(MaxSizeCalcBasis, &'a CalcSizeCalculationOf<S>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FlexBasisView<'a, S: LayoutScalar> {
    Zero,
    Calculation(&'a SizingCalculationOf<S>),
    Auto,
    Content,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
    FitContentFunction(&'a SizingCalculationOf<S>),
    CalcSize(FlexBasisCalcBasis, &'a CalcSizeCalculationOf<S>),
}

impl<S: LayoutScalar> PreferredSizeOf<S> {
    pub const AUTO: Self = Self::new(PreferredSizeValue::Auto);
    pub const MIN_CONTENT: Self = Self::new(PreferredSizeValue::MinContent);
    pub const MAX_CONTENT: Self = Self::new(PreferredSizeValue::MaxContent);
    pub const STRETCH: Self = Self::new(PreferredSizeValue::Stretch);
    pub const FIT_CONTENT: Self = Self::new(PreferredSizeValue::FitContent);
    pub const CONTAIN: Self = Self::new(PreferredSizeValue::Contain);

    const fn new(value: PreferredSizeValue<S>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn value(value: LengthPercentageOf<S>) -> Self {
        Self::calculation(SizingCalculationOf::value(value))
    }

    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::value(LengthPercentageOf::px(value).expect("trusted preferred-size test literal"))
    }

    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted preferred-size percentage test literal"),
        )
    }

    #[must_use]
    pub fn calculation(calculation: SizingCalculationOf<S>) -> Self {
        if calculation.is_zero_value() {
            Self::new(PreferredSizeValue::Zero)
        } else {
            Self::new(PreferredSizeValue::Calculation(calculation))
        }
    }

    #[must_use]
    pub fn fit_content_function(calculation: SizingCalculationOf<S>) -> Self {
        Self::new(PreferredSizeValue::FitContentFunction(calculation))
    }

    pub fn calc_size(
        basis: PreferredSizeCalcBasis,
        calculation: CalcSizeCalculationOf<S>,
    ) -> Result<Self, CalcSizeConstructionError> {
        reject_any_size_reference(basis == PreferredSizeCalcBasis::Any, &calculation)?;
        Ok(Self::new(PreferredSizeValue::CalcSize(basis, calculation)))
    }

    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self.view(), PreferredSizeView::Auto)
    }

    #[must_use]
    pub const fn is_min_content(&self) -> bool {
        matches!(self.view(), PreferredSizeView::MinContent)
    }

    #[must_use]
    pub const fn is_max_content(&self) -> bool {
        matches!(self.view(), PreferredSizeView::MaxContent)
    }

    #[must_use]
    pub fn is_calculation(&self) -> bool {
        match self.view() {
            PreferredSizeView::Zero => true,
            PreferredSizeView::Calculation(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_fit_content_function(&self) -> bool {
        match self.view() {
            PreferredSizeView::FitContentFunction(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_calc_size(&self) -> bool {
        match self.view() {
            PreferredSizeView::CalcSize(basis, value) => {
                let _ = (basis, value);
                true
            }
            _ => false,
        }
    }

    pub(crate) const fn view(&self) -> PreferredSizeView<'_, S> {
        match &self.value {
            PreferredSizeValue::Zero => PreferredSizeView::Zero,
            PreferredSizeValue::Calculation(value) => PreferredSizeView::Calculation(value),
            PreferredSizeValue::Auto => PreferredSizeView::Auto,
            PreferredSizeValue::MinContent => PreferredSizeView::MinContent,
            PreferredSizeValue::MaxContent => PreferredSizeView::MaxContent,
            PreferredSizeValue::Stretch => PreferredSizeView::Stretch,
            PreferredSizeValue::FitContent => PreferredSizeView::FitContent,
            PreferredSizeValue::Contain => PreferredSizeView::Contain,
            PreferredSizeValue::FitContentFunction(value) => {
                PreferredSizeView::FitContentFunction(value)
            }
            PreferredSizeValue::CalcSize(basis, value) => {
                PreferredSizeView::CalcSize(*basis, value)
            }
        }
    }

    pub(crate) fn resolve_simple_with_status(
        &self,
        basis: Option<S>,
    ) -> Result<LengthResolutionOf<S>, crate::LengthResolutionStatus<S>> {
        match self.view() {
            PreferredSizeView::Zero => Ok(LengthResolutionOf::definite(S::ZERO, false)),
            PreferredSizeView::Calculation(calculation) => {
                resolve_sizing_calculation(calculation, basis)
            }
            PreferredSizeView::Auto
            | PreferredSizeView::MinContent
            | PreferredSizeView::MaxContent => Ok(LengthResolutionOf::non_numeric()),
            PreferredSizeView::Stretch
            | PreferredSizeView::FitContent
            | PreferredSizeView::Contain
            | PreferredSizeView::FitContentFunction(_)
            | PreferredSizeView::CalcSize(_, _) => Err(crate::LengthResolutionStatus::NonNumeric),
        }
    }

    pub(crate) fn depends_on_basis(&self) -> bool {
        match self.view() {
            PreferredSizeView::Calculation(calculation) => calculation.depends_on_basis(),
            PreferredSizeView::Zero
            | PreferredSizeView::Auto
            | PreferredSizeView::MinContent
            | PreferredSizeView::MaxContent
            | PreferredSizeView::Stretch
            | PreferredSizeView::FitContent
            | PreferredSizeView::Contain
            | PreferredSizeView::FitContentFunction(_)
            | PreferredSizeView::CalcSize(_, _) => false,
        }
    }
}

impl<S: LayoutScalar> Default for PreferredSizeOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

impl<S: LayoutScalar> MinSizeOf<S> {
    pub const ZERO: Self = Self::new(MinSizeValue::Zero);
    pub const AUTO: Self = Self::new(MinSizeValue::Auto);
    pub const MIN_CONTENT: Self = Self::new(MinSizeValue::MinContent);
    pub const MAX_CONTENT: Self = Self::new(MinSizeValue::MaxContent);
    pub const STRETCH: Self = Self::new(MinSizeValue::Stretch);
    pub const FIT_CONTENT: Self = Self::new(MinSizeValue::FitContent);
    pub const CONTAIN: Self = Self::new(MinSizeValue::Contain);

    const fn new(value: MinSizeValue<S>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn value(value: LengthPercentageOf<S>) -> Self {
        Self::calculation(SizingCalculationOf::value(value))
    }

    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::value(LengthPercentageOf::px(value).expect("trusted minimum-size test literal"))
    }

    #[must_use]
    pub fn calculation(calculation: SizingCalculationOf<S>) -> Self {
        if calculation.is_zero_value() {
            Self::ZERO
        } else {
            Self::new(MinSizeValue::Calculation(calculation))
        }
    }

    #[must_use]
    pub fn fit_content_function(calculation: SizingCalculationOf<S>) -> Self {
        Self::new(MinSizeValue::FitContentFunction(calculation))
    }

    pub fn calc_size(
        basis: MinSizeCalcBasis,
        calculation: CalcSizeCalculationOf<S>,
    ) -> Result<Self, CalcSizeConstructionError> {
        reject_any_size_reference(basis == MinSizeCalcBasis::Any, &calculation)?;
        Ok(Self::new(MinSizeValue::CalcSize(basis, calculation)))
    }

    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self.view(), MinSizeView::Auto)
    }

    #[must_use]
    pub const fn is_min_content(&self) -> bool {
        matches!(self.view(), MinSizeView::MinContent)
    }

    #[must_use]
    pub const fn is_max_content(&self) -> bool {
        matches!(self.view(), MinSizeView::MaxContent)
    }

    #[must_use]
    pub fn is_calculation(&self) -> bool {
        match self.view() {
            MinSizeView::Zero => true,
            MinSizeView::Calculation(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_fit_content_function(&self) -> bool {
        match self.view() {
            MinSizeView::FitContentFunction(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_calc_size(&self) -> bool {
        match self.view() {
            MinSizeView::CalcSize(basis, value) => {
                let _ = (basis, value);
                true
            }
            _ => false,
        }
    }

    pub(crate) const fn view(&self) -> MinSizeView<'_, S> {
        match &self.value {
            MinSizeValue::Zero => MinSizeView::Zero,
            MinSizeValue::Calculation(value) => MinSizeView::Calculation(value),
            MinSizeValue::Auto => MinSizeView::Auto,
            MinSizeValue::MinContent => MinSizeView::MinContent,
            MinSizeValue::MaxContent => MinSizeView::MaxContent,
            MinSizeValue::Stretch => MinSizeView::Stretch,
            MinSizeValue::FitContent => MinSizeView::FitContent,
            MinSizeValue::Contain => MinSizeView::Contain,
            MinSizeValue::FitContentFunction(value) => MinSizeView::FitContentFunction(value),
            MinSizeValue::CalcSize(basis, value) => MinSizeView::CalcSize(*basis, value),
        }
    }

    pub(crate) fn resolve_simple_with_status(
        &self,
        basis: Option<S>,
    ) -> Result<LengthResolutionOf<S>, crate::LengthResolutionStatus<S>> {
        match self.view() {
            MinSizeView::Zero => Ok(LengthResolutionOf::definite(S::ZERO, false)),
            MinSizeView::Calculation(calculation) => resolve_sizing_calculation(calculation, basis),
            MinSizeView::Auto | MinSizeView::MinContent | MinSizeView::MaxContent => {
                Ok(LengthResolutionOf::non_numeric())
            }
            MinSizeView::Stretch
            | MinSizeView::FitContent
            | MinSizeView::Contain
            | MinSizeView::FitContentFunction(_)
            | MinSizeView::CalcSize(_, _) => Err(crate::LengthResolutionStatus::NonNumeric),
        }
    }
}

impl<S: LayoutScalar> Default for MinSizeOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

impl<S: LayoutScalar> MaxSizeOf<S> {
    pub const ZERO: Self = Self::new(MaxSizeValue::Zero);
    pub const NONE: Self = Self::new(MaxSizeValue::None);
    pub const MIN_CONTENT: Self = Self::new(MaxSizeValue::MinContent);
    pub const MAX_CONTENT: Self = Self::new(MaxSizeValue::MaxContent);
    pub const STRETCH: Self = Self::new(MaxSizeValue::Stretch);
    pub const FIT_CONTENT: Self = Self::new(MaxSizeValue::FitContent);
    pub const CONTAIN: Self = Self::new(MaxSizeValue::Contain);

    const fn new(value: MaxSizeValue<S>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn value(value: LengthPercentageOf<S>) -> Self {
        Self::calculation(SizingCalculationOf::value(value))
    }

    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::value(LengthPercentageOf::px(value).expect("trusted maximum-size test literal"))
    }

    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted maximum-size percentage test literal"),
        )
    }

    #[must_use]
    pub fn calculation(calculation: SizingCalculationOf<S>) -> Self {
        if calculation.is_zero_value() {
            Self::ZERO
        } else {
            Self::new(MaxSizeValue::Calculation(calculation))
        }
    }

    #[must_use]
    pub fn fit_content_function(calculation: SizingCalculationOf<S>) -> Self {
        Self::new(MaxSizeValue::FitContentFunction(calculation))
    }

    pub fn calc_size(
        basis: MaxSizeCalcBasis,
        calculation: CalcSizeCalculationOf<S>,
    ) -> Result<Self, CalcSizeConstructionError> {
        reject_any_size_reference(basis == MaxSizeCalcBasis::Any, &calculation)?;
        Ok(Self::new(MaxSizeValue::CalcSize(basis, calculation)))
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self.view(), MaxSizeView::None)
    }

    #[must_use]
    pub const fn is_min_content(&self) -> bool {
        matches!(self.view(), MaxSizeView::MinContent)
    }

    #[must_use]
    pub const fn is_max_content(&self) -> bool {
        matches!(self.view(), MaxSizeView::MaxContent)
    }

    #[must_use]
    pub fn is_calculation(&self) -> bool {
        match self.view() {
            MaxSizeView::Zero => true,
            MaxSizeView::Calculation(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_fit_content_function(&self) -> bool {
        match self.view() {
            MaxSizeView::FitContentFunction(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_calc_size(&self) -> bool {
        match self.view() {
            MaxSizeView::CalcSize(basis, value) => {
                let _ = (basis, value);
                true
            }
            _ => false,
        }
    }

    pub(crate) const fn view(&self) -> MaxSizeView<'_, S> {
        match &self.value {
            MaxSizeValue::Zero => MaxSizeView::Zero,
            MaxSizeValue::Calculation(value) => MaxSizeView::Calculation(value),
            MaxSizeValue::None => MaxSizeView::None,
            MaxSizeValue::MinContent => MaxSizeView::MinContent,
            MaxSizeValue::MaxContent => MaxSizeView::MaxContent,
            MaxSizeValue::Stretch => MaxSizeView::Stretch,
            MaxSizeValue::FitContent => MaxSizeView::FitContent,
            MaxSizeValue::Contain => MaxSizeView::Contain,
            MaxSizeValue::FitContentFunction(value) => MaxSizeView::FitContentFunction(value),
            MaxSizeValue::CalcSize(basis, value) => MaxSizeView::CalcSize(*basis, value),
        }
    }

    pub(crate) fn resolve_simple_with_status(
        &self,
        basis: Option<S>,
    ) -> Result<LengthResolutionOf<S>, crate::LengthResolutionStatus<S>> {
        match self.view() {
            MaxSizeView::Zero => Ok(LengthResolutionOf::definite(S::ZERO, false)),
            MaxSizeView::Calculation(calculation) => resolve_sizing_calculation(calculation, basis),
            MaxSizeView::None | MaxSizeView::MinContent | MaxSizeView::MaxContent => {
                Ok(LengthResolutionOf::non_numeric())
            }
            MaxSizeView::Stretch
            | MaxSizeView::FitContent
            | MaxSizeView::Contain
            | MaxSizeView::FitContentFunction(_)
            | MaxSizeView::CalcSize(_, _) => Err(crate::LengthResolutionStatus::NonNumeric),
        }
    }
}

impl<S: LayoutScalar> Default for MaxSizeOf<S> {
    fn default() -> Self {
        Self::NONE
    }
}

impl<S: LayoutScalar> FlexBasisOf<S> {
    pub const ZERO: Self = Self::new(FlexBasisValue::Zero);
    pub const AUTO: Self = Self::new(FlexBasisValue::Auto);
    pub const CONTENT: Self = Self::new(FlexBasisValue::Content);
    pub const MIN_CONTENT: Self = Self::new(FlexBasisValue::MinContent);
    pub const MAX_CONTENT: Self = Self::new(FlexBasisValue::MaxContent);
    pub const STRETCH: Self = Self::new(FlexBasisValue::Stretch);
    pub const FIT_CONTENT: Self = Self::new(FlexBasisValue::FitContent);
    pub const CONTAIN: Self = Self::new(FlexBasisValue::Contain);

    const fn new(value: FlexBasisValue<S>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn value(value: LengthPercentageOf<S>) -> Self {
        Self::calculation(SizingCalculationOf::value(value))
    }

    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::value(LengthPercentageOf::px(value).expect("trusted flex-basis test literal"))
    }

    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted flex-basis percentage test literal"),
        )
    }

    #[must_use]
    pub fn calculation(calculation: SizingCalculationOf<S>) -> Self {
        if calculation.is_zero_value() {
            Self::ZERO
        } else {
            Self::new(FlexBasisValue::Calculation(calculation))
        }
    }

    #[must_use]
    pub fn fit_content_function(calculation: SizingCalculationOf<S>) -> Self {
        Self::new(FlexBasisValue::FitContentFunction(calculation))
    }

    pub fn calc_size(
        basis: FlexBasisCalcBasis,
        calculation: CalcSizeCalculationOf<S>,
    ) -> Result<Self, CalcSizeConstructionError> {
        reject_any_size_reference(basis == FlexBasisCalcBasis::Any, &calculation)?;
        Ok(Self::new(FlexBasisValue::CalcSize(basis, calculation)))
    }

    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self.view(), FlexBasisView::Auto)
    }

    #[must_use]
    pub const fn is_content(&self) -> bool {
        matches!(self.view(), FlexBasisView::Content)
    }

    #[must_use]
    pub const fn is_min_content(&self) -> bool {
        matches!(self.view(), FlexBasisView::MinContent)
    }

    #[must_use]
    pub const fn is_max_content(&self) -> bool {
        matches!(self.view(), FlexBasisView::MaxContent)
    }

    #[must_use]
    pub fn is_calculation(&self) -> bool {
        match self.view() {
            FlexBasisView::Zero => true,
            FlexBasisView::Calculation(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_fit_content_function(&self) -> bool {
        match self.view() {
            FlexBasisView::FitContentFunction(value) => {
                let _ = value;
                true
            }
            _ => false,
        }
    }

    #[must_use]
    pub fn is_calc_size(&self) -> bool {
        match self.view() {
            FlexBasisView::CalcSize(basis, value) => {
                let _ = (basis, value);
                true
            }
            _ => false,
        }
    }

    pub(crate) const fn view(&self) -> FlexBasisView<'_, S> {
        match &self.value {
            FlexBasisValue::Zero => FlexBasisView::Zero,
            FlexBasisValue::Calculation(value) => FlexBasisView::Calculation(value),
            FlexBasisValue::Auto => FlexBasisView::Auto,
            FlexBasisValue::Content => FlexBasisView::Content,
            FlexBasisValue::MinContent => FlexBasisView::MinContent,
            FlexBasisValue::MaxContent => FlexBasisView::MaxContent,
            FlexBasisValue::Stretch => FlexBasisView::Stretch,
            FlexBasisValue::FitContent => FlexBasisView::FitContent,
            FlexBasisValue::Contain => FlexBasisView::Contain,
            FlexBasisValue::FitContentFunction(value) => FlexBasisView::FitContentFunction(value),
            FlexBasisValue::CalcSize(basis, value) => FlexBasisView::CalcSize(*basis, value),
        }
    }

    pub(crate) fn resolve_simple_with_status(
        &self,
        basis: Option<S>,
    ) -> Result<LengthResolutionOf<S>, crate::LengthResolutionStatus<S>> {
        match self.view() {
            FlexBasisView::Zero => Ok(LengthResolutionOf::definite(S::ZERO, false)),
            FlexBasisView::Calculation(calculation) => {
                resolve_sizing_calculation(calculation, basis)
            }
            FlexBasisView::Auto | FlexBasisView::MinContent | FlexBasisView::MaxContent => {
                Ok(LengthResolutionOf::non_numeric())
            }
            FlexBasisView::Content
            | FlexBasisView::Stretch
            | FlexBasisView::FitContent
            | FlexBasisView::Contain
            | FlexBasisView::FitContentFunction(_)
            | FlexBasisView::CalcSize(_, _) => Err(crate::LengthResolutionStatus::NonNumeric),
        }
    }
}

impl<S: LayoutScalar> Default for FlexBasisOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

fn reject_any_size_reference<S: LayoutScalar>(
    basis_is_any: bool,
    calculation: &CalcSizeCalculationOf<S>,
) -> Result<(), CalcSizeConstructionError> {
    if basis_is_any && calculation.depends_on_size() {
        Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
    } else {
        Ok(())
    }
}

fn resolve_sizing_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Result<LengthResolutionOf<S>, crate::LengthResolutionStatus<S>> {
    let basis = match basis {
        None => PercentageBasisOf::MISSING,
        Some(value) => match PercentageBasisOf::definite(value) {
            Ok(basis) => basis,
            Err(
                crate::NonNegativeFiniteScalarErrorOf::NonFinite { value }
                | crate::NonNegativeFiniteScalarErrorOf::Negative { value },
            ) => {
                return Ok(LengthResolutionOf::invalid_numeric(
                    value,
                    calculation.depends_on_basis(),
                ));
            }
        },
    };
    let resolution = calculation.resolve_against(basis);
    Ok(match resolution.status() {
        crate::LengthResolutionStatus::Resolved => LengthResolutionOf::definite(
            resolution
                .value
                .expect("resolved sizing calculation must carry a value")
                .max(S::ZERO),
            calculation.depends_on_basis(),
        ),
        crate::LengthResolutionStatus::MissingBasis
        | crate::LengthResolutionStatus::InvalidNumeric { .. } => resolution,
        crate::LengthResolutionStatus::NonNumeric => {
            unreachable!("validated sizing calculations are numeric programs")
        }
    })
}

impl<S: LayoutScalar> SizingCalculationOf<S> {
    #[must_use]
    pub fn value(value: LengthPercentageOf<S>) -> Self {
        Self {
            instructions: vec![Instruction::Value(value)],
            depends_on_basis: value.depends_on_basis(),
        }
    }

    pub fn min(arguments: Vec<Self>) -> Result<Self, SizingCalculationError> {
        Self::extremum(arguments, Instruction::Min)
    }

    pub fn max(arguments: Vec<Self>) -> Result<Self, SizingCalculationError> {
        Self::extremum(arguments, Instruction::Max)
    }

    #[must_use]
    pub fn clamp(minimum: Option<Self>, preferred: Self, maximum: Option<Self>) -> Self {
        let has_minimum = minimum.is_some();
        let has_maximum = maximum.is_some();
        let mut depends_on_basis = preferred.depends_on_basis;

        let mut instructions = if let Some(minimum) = minimum {
            depends_on_basis |= minimum.depends_on_basis;
            let mut instructions = minimum.instructions;
            instructions.extend(preferred.instructions);
            instructions
        } else {
            preferred.instructions
        };

        if let Some(maximum) = maximum {
            depends_on_basis |= maximum.depends_on_basis;
            instructions.extend(maximum.instructions);
        }
        instructions.push(Instruction::Clamp {
            has_minimum,
            has_maximum,
        });

        Self {
            instructions,
            depends_on_basis,
        }
    }

    #[must_use]
    pub const fn depends_on_basis(&self) -> bool {
        self.depends_on_basis
    }

    #[must_use]
    pub(crate) fn affine_value(&self) -> Option<LengthPercentageOf<S>> {
        match self.instructions.as_slice() {
            [Instruction::Value(value)] => Some(*value),
            _ => None,
        }
    }

    fn is_zero_value(&self) -> bool {
        matches!(
            self.instructions.as_slice(),
            [Instruction::Value(value)]
                if value.absolute_px() == S::ZERO && value.percent_fraction() == S::ZERO
        )
    }

    #[must_use]
    pub fn resolve_against(&self, basis: PercentageBasisOf<S>) -> LengthResolutionOf<S> {
        if self.depends_on_basis && matches!(basis, PercentageBasisOf::Missing) {
            return LengthResolutionOf::unresolved(true);
        }

        let mut values = Vec::new();
        for instruction in &self.instructions {
            match *instruction {
                Instruction::Value(value) => match value.resolve_against(basis) {
                    NumericResolutionOf::Resolved(value) => values.push(value),
                    NumericResolutionOf::MissingBasis { .. } => {
                        return LengthResolutionOf::unresolved(true);
                    }
                    NumericResolutionOf::InvalidNumeric { resolved, .. } => {
                        return LengthResolutionOf::invalid_numeric(
                            resolved,
                            self.depends_on_basis,
                        );
                    }
                },
                Instruction::Min(argument_count) => {
                    reduce_extremum(&mut values, argument_count, LayoutScalar::min);
                }
                Instruction::Max(argument_count) => {
                    reduce_extremum(&mut values, argument_count, LayoutScalar::max);
                }
                Instruction::Clamp {
                    has_minimum,
                    has_maximum,
                } => {
                    let maximum = has_maximum.then(|| pop_value(&mut values));
                    let preferred = pop_value(&mut values);
                    let minimum = has_minimum.then(|| pop_value(&mut values));
                    let bounded_above = maximum.map_or(preferred, |maximum| preferred.min(maximum));
                    values
                        .push(minimum.map_or(bounded_above, |minimum| minimum.max(bounded_above)));
                }
            }
        }

        debug_assert_eq!(values.len(), 1, "validated postfix program has one result");
        LengthResolutionOf::definite(pop_value(&mut values), self.depends_on_basis)
    }

    fn extremum(
        arguments: Vec<Self>,
        instruction: fn(NonZeroUsize) -> Instruction<S>,
    ) -> Result<Self, SizingCalculationError> {
        let Some(argument_count) = NonZeroUsize::new(arguments.len()) else {
            return Err(SizingCalculationError::EmptyArguments);
        };

        let mut arguments = arguments.into_iter();
        let first = arguments.next().expect("nonzero argument count");
        let mut instructions = first.instructions;
        let mut depends_on_basis = first.depends_on_basis;
        for argument in arguments {
            depends_on_basis |= argument.depends_on_basis;
            instructions.extend(argument.instructions);
        }
        instructions.push(instruction(argument_count));

        Ok(Self {
            instructions,
            depends_on_basis,
        })
    }
}

impl<S: LayoutScalar> CalcSizeCalculationOf<S> {
    #[must_use]
    pub fn value(value: LengthPercentageOf<S>) -> Self {
        Self::from_validated_coefficients(value.absolute_px(), value.percent_fraction(), S::ZERO)
    }

    #[must_use]
    pub fn size() -> Self {
        Self::from_validated_coefficients(S::ZERO, S::ZERO, S::ONE)
    }

    pub fn from_coefficients(
        absolute_px: S,
        percent_fraction: S,
        size_fraction: S,
    ) -> Result<Self, CalcSizeCalculationErrorOf<S>> {
        let absolute_px = finite_calc_size_coefficient(absolute_px)
            .map_err(CalcSizeCalculationErrorOf::InvalidAbsolutePx)?;
        let percent_fraction = finite_calc_size_coefficient(percent_fraction)
            .map_err(CalcSizeCalculationErrorOf::InvalidPercentFraction)?;
        let size_fraction = finite_calc_size_coefficient(size_fraction)
            .map_err(CalcSizeCalculationErrorOf::InvalidSizeFraction)?;
        Ok(Self::from_validated_coefficients(
            absolute_px,
            percent_fraction,
            size_fraction,
        ))
    }

    pub fn min(arguments: Vec<Self>) -> Result<Self, SizingCalculationError> {
        Self::extremum(arguments, CalcSizeInstruction::Min)
    }

    pub fn max(arguments: Vec<Self>) -> Result<Self, SizingCalculationError> {
        Self::extremum(arguments, CalcSizeInstruction::Max)
    }

    #[must_use]
    pub fn clamp(minimum: Option<Self>, preferred: Self, maximum: Option<Self>) -> Self {
        let has_minimum = minimum.is_some();
        let has_maximum = maximum.is_some();
        let mut depends_on_percentage_basis = preferred.depends_on_percentage_basis;
        let mut depends_on_size = preferred.depends_on_size;

        let mut instructions = if let Some(minimum) = minimum {
            depends_on_percentage_basis |= minimum.depends_on_percentage_basis;
            depends_on_size |= minimum.depends_on_size;
            let mut instructions = minimum.instructions;
            instructions.extend(preferred.instructions);
            instructions
        } else {
            preferred.instructions
        };

        if let Some(maximum) = maximum {
            depends_on_percentage_basis |= maximum.depends_on_percentage_basis;
            depends_on_size |= maximum.depends_on_size;
            instructions.extend(maximum.instructions);
        }
        instructions.push(CalcSizeInstruction::Clamp {
            has_minimum,
            has_maximum,
        });

        Self {
            instructions,
            depends_on_percentage_basis,
            depends_on_size,
        }
    }

    #[must_use]
    pub const fn depends_on_size(&self) -> bool {
        self.depends_on_size
    }

    #[must_use]
    pub fn resolve_against(
        &self,
        basis_size: Option<NonNegativeFiniteOf<S>>,
        percentage_basis: PercentageBasisOf<S>,
    ) -> LengthResolutionOf<S> {
        let depends_on_basis = self.depends_on_percentage_basis || self.depends_on_size;
        if self.depends_on_size && basis_size.is_none() {
            return LengthResolutionOf::unresolved(true);
        }

        let percentage_basis = percentage_basis
            .definite_value()
            .map_or(S::ZERO, NonNegativeFiniteOf::get);
        let basis_size = basis_size.map_or(S::ZERO, NonNegativeFiniteOf::get);
        let mut values = Vec::new();

        for instruction in &self.instructions {
            match *instruction {
                CalcSizeInstruction::Value(coefficients) => {
                    let value = match evaluate_calc_size_coefficients(
                        coefficients,
                        percentage_basis,
                        basis_size,
                    ) {
                        Ok(value) => value,
                        Err(value) => {
                            return LengthResolutionOf::invalid_numeric(value, depends_on_basis);
                        }
                    };
                    values.push(value);
                }
                CalcSizeInstruction::Min(argument_count) => {
                    reduce_extremum(&mut values, argument_count, LayoutScalar::min);
                    canonicalize_last(&mut values);
                }
                CalcSizeInstruction::Max(argument_count) => {
                    reduce_extremum(&mut values, argument_count, LayoutScalar::max);
                    canonicalize_last(&mut values);
                }
                CalcSizeInstruction::Clamp {
                    has_minimum,
                    has_maximum,
                } => {
                    let maximum = has_maximum.then(|| pop_value(&mut values));
                    let preferred = pop_value(&mut values);
                    let minimum = has_minimum.then(|| pop_value(&mut values));
                    let bounded_above = maximum.map_or(preferred, |maximum| preferred.min(maximum));
                    let result =
                        minimum.map_or(bounded_above, |minimum| minimum.max(bounded_above));
                    values.push(canonical_calc_size_zero(result));
                }
            }
        }

        debug_assert_eq!(values.len(), 1, "validated postfix program has one result");
        LengthResolutionOf::definite(pop_value(&mut values), depends_on_basis)
    }

    fn from_validated_coefficients(absolute_px: S, percent_fraction: S, size_fraction: S) -> Self {
        Self {
            instructions: vec![CalcSizeInstruction::Value(CalcSizeCoefficients {
                absolute_px,
                percent_fraction,
                size_fraction,
            })],
            depends_on_percentage_basis: percent_fraction != S::ZERO,
            depends_on_size: size_fraction != S::ZERO,
        }
    }

    fn extremum(
        arguments: Vec<Self>,
        instruction: fn(NonZeroUsize) -> CalcSizeInstruction<S>,
    ) -> Result<Self, SizingCalculationError> {
        let Some(argument_count) = NonZeroUsize::new(arguments.len()) else {
            return Err(SizingCalculationError::EmptyArguments);
        };

        let mut arguments = arguments.into_iter();
        let first = arguments.next().expect("nonzero argument count");
        let mut instructions = first.instructions;
        let mut depends_on_percentage_basis = first.depends_on_percentage_basis;
        let mut depends_on_size = first.depends_on_size;
        for argument in arguments {
            depends_on_percentage_basis |= argument.depends_on_percentage_basis;
            depends_on_size |= argument.depends_on_size;
            instructions.extend(argument.instructions);
        }
        instructions.push(instruction(argument_count));

        Ok(Self {
            instructions,
            depends_on_percentage_basis,
            depends_on_size,
        })
    }
}

fn finite_calc_size_coefficient<S: LayoutScalar>(value: S) -> Result<S, FiniteScalarErrorOf<S>> {
    if value.is_finite() {
        Ok(canonical_calc_size_zero(value))
    } else {
        Err(FiniteScalarErrorOf::NonFinite { value })
    }
}

fn evaluate_calc_size_coefficients<S: LayoutScalar>(
    coefficients: CalcSizeCoefficients<S>,
    percentage_basis: S,
    basis_size: S,
) -> Result<S, S> {
    let percentage = checked_calc_size_product(coefficients.percent_fraction, percentage_basis)?;
    let size = checked_calc_size_product(coefficients.size_fraction, basis_size)?;
    let value = checked_calc_size_sum(coefficients.absolute_px, percentage)?;
    checked_calc_size_sum(value, size)
}

fn checked_calc_size_product<S: LayoutScalar>(left: S, right: S) -> Result<S, S> {
    let value = left * right;
    if value.is_finite() {
        Ok(canonical_calc_size_zero(value))
    } else {
        Err(value)
    }
}

fn checked_calc_size_sum<S: LayoutScalar>(left: S, right: S) -> Result<S, S> {
    let value = left + right;
    if value.is_finite() {
        Ok(canonical_calc_size_zero(value))
    } else {
        Err(value)
    }
}

fn canonicalize_last<S: LayoutScalar>(values: &mut [S]) {
    let value = values.last_mut().expect("validated postfix arity");
    *value = canonical_calc_size_zero(*value);
}

fn canonical_calc_size_zero<S: LayoutScalar>(value: S) -> S {
    if value == S::ZERO { S::ZERO } else { value }
}

fn reduce_extremum<S: LayoutScalar>(
    values: &mut Vec<S>,
    argument_count: NonZeroUsize,
    combine: fn(S, S) -> S,
) {
    let start = values
        .len()
        .checked_sub(argument_count.get())
        .expect("validated postfix arity");
    let result = values[start + 1..]
        .iter()
        .copied()
        .fold(values[start], combine);
    values.truncate(start);
    values.push(result);
}

fn pop_value<S>(values: &mut Vec<S>) -> S {
    values.pop().expect("validated postfix arity")
}

#[cfg(test)]
mod tests {
    use super::{
        CalcSizeCalculationErrorOf, CalcSizeCalculationOf, CalcSizeConstructionError,
        FlexBasisCalcBasis, FlexBasisOf, FlexBasisView, MaxSizeCalcBasis, MaxSizeOf, MaxSizeView,
        MinSizeCalcBasis, MinSizeOf, MinSizeView, PreferredSizeCalcBasis, PreferredSizeOf,
        PreferredSizeView, SizingCalculationError, SizingCalculationOf,
    };
    use crate::{
        FiniteScalarErrorOf, LayoutScalar, LengthPercentageOf, LengthResolutionStatus,
        NonNegativeFiniteOf, PercentageBasisOf,
    };

    fn px_f32(value: f32) -> SizingCalculationOf<f32> {
        SizingCalculationOf::value(LengthPercentageOf::px(value).expect("finite px"))
    }

    fn px_f64(value: f64) -> SizingCalculationOf<f64> {
        SizingCalculationOf::value(LengthPercentageOf::px(value).expect("finite px"))
    }

    fn resolved_f32(calculation: &SizingCalculationOf<f32>) -> f32 {
        calculation
            .resolve_against(PercentageBasisOf::MISSING)
            .value
            .expect("basis-independent calculation")
    }

    fn calc_px_f32(value: f32) -> CalcSizeCalculationOf<f32> {
        CalcSizeCalculationOf::value(LengthPercentageOf::px(value).expect("finite px"))
    }

    fn calc_px_f64(value: f64) -> CalcSizeCalculationOf<f64> {
        CalcSizeCalculationOf::value(LengthPercentageOf::px(value).expect("finite px"))
    }

    fn size_f32(value: f32) -> NonNegativeFiniteOf<f32> {
        NonNegativeFiniteOf::new(value).expect("non-negative finite size")
    }

    fn size_f64(value: f64) -> NonNegativeFiniteOf<f64> {
        NonNegativeFiniteOf::new(value).expect("non-negative finite size")
    }

    fn resolved_calc_f32(
        calculation: &CalcSizeCalculationOf<f32>,
        basis_size: Option<NonNegativeFiniteOf<f32>>,
        percentage_basis: PercentageBasisOf<f32>,
    ) -> f32 {
        calculation
            .resolve_against(basis_size, percentage_basis)
            .value
            .expect("resolved calc-size calculation")
    }

    fn assert_property_sizing_lane<S: LayoutScalar>() {
        assert_eq!(PreferredSizeOf::<S>::default(), PreferredSizeOf::AUTO);
        assert_eq!(MinSizeOf::<S>::default(), MinSizeOf::AUTO);
        assert_eq!(MaxSizeOf::<S>::default(), MaxSizeOf::NONE);
        assert_eq!(FlexBasisOf::<S>::default(), FlexBasisOf::AUTO);
        assert_eq!(
            [
                PreferredSizeOf::<S>::AUTO,
                PreferredSizeOf::MIN_CONTENT,
                PreferredSizeOf::MAX_CONTENT,
                PreferredSizeOf::STRETCH,
                PreferredSizeOf::FIT_CONTENT,
                PreferredSizeOf::CONTAIN,
            ]
            .len(),
            6
        );
        assert_eq!(
            [
                MinSizeOf::<S>::AUTO,
                MinSizeOf::MIN_CONTENT,
                MinSizeOf::MAX_CONTENT,
                MinSizeOf::STRETCH,
                MinSizeOf::FIT_CONTENT,
                MinSizeOf::CONTAIN,
                MinSizeOf::ZERO,
            ]
            .len(),
            7
        );
        assert_eq!(
            [
                MaxSizeOf::<S>::NONE,
                MaxSizeOf::MIN_CONTENT,
                MaxSizeOf::MAX_CONTENT,
                MaxSizeOf::STRETCH,
                MaxSizeOf::FIT_CONTENT,
                MaxSizeOf::CONTAIN,
                MaxSizeOf::ZERO,
            ]
            .len(),
            7
        );
        assert_eq!(
            [
                FlexBasisOf::<S>::AUTO,
                FlexBasisOf::CONTENT,
                FlexBasisOf::MIN_CONTENT,
                FlexBasisOf::MAX_CONTENT,
                FlexBasisOf::STRETCH,
                FlexBasisOf::FIT_CONTENT,
                FlexBasisOf::CONTAIN,
                FlexBasisOf::ZERO,
            ]
            .len(),
            8
        );

        let value = LengthPercentageOf::px(S::ONE).expect("finite px");
        let calculation = SizingCalculationOf::value(value);
        for property in [
            PreferredSizeOf::<S>::value(value),
            PreferredSizeOf::calculation(calculation.clone()),
            PreferredSizeOf::fit_content_function(calculation.clone()),
        ] {
            assert!(!format!("{property:?}").is_empty());
            assert_eq!(property, property.clone());
        }
        assert_eq!(
            MinSizeOf::<S>::value(value),
            MinSizeOf::calculation(calculation.clone())
        );
        assert_eq!(
            MaxSizeOf::<S>::value(value),
            MaxSizeOf::calculation(calculation.clone())
        );
        assert_eq!(
            FlexBasisOf::<S>::value(value),
            FlexBasisOf::calculation(calculation.clone())
        );
        assert!(MinSizeOf::fit_content_function(calculation.clone()).is_fit_content_function());
        assert!(MaxSizeOf::fit_content_function(calculation.clone()).is_fit_content_function());
        assert!(FlexBasisOf::fit_content_function(calculation).is_fit_content_function());

        let dependent = CalcSizeCalculationOf::<S>::size();
        let independent = CalcSizeCalculationOf::value(LengthPercentageOf::<S>::ZERO);
        for basis in [
            PreferredSizeCalcBasis::FullPercentage,
            PreferredSizeCalcBasis::Auto,
            PreferredSizeCalcBasis::MinContent,
            PreferredSizeCalcBasis::MaxContent,
            PreferredSizeCalcBasis::Stretch,
            PreferredSizeCalcBasis::FitContent,
            PreferredSizeCalcBasis::Contain,
        ] {
            assert!(PreferredSizeOf::calc_size(basis, dependent.clone()).is_ok());
        }
        assert!(
            PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Any, independent.clone()).is_ok()
        );
        assert_eq!(
            PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Any, dependent.clone()),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );

        for basis in [
            MinSizeCalcBasis::FullPercentage,
            MinSizeCalcBasis::Auto,
            MinSizeCalcBasis::MinContent,
            MinSizeCalcBasis::MaxContent,
            MinSizeCalcBasis::Stretch,
            MinSizeCalcBasis::FitContent,
            MinSizeCalcBasis::Contain,
        ] {
            assert!(MinSizeOf::calc_size(basis, dependent.clone()).is_ok());
        }
        assert!(MinSizeOf::calc_size(MinSizeCalcBasis::Any, independent.clone()).is_ok());
        assert_eq!(
            MinSizeOf::calc_size(MinSizeCalcBasis::Any, dependent.clone()),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );

        for basis in [
            MaxSizeCalcBasis::FullPercentage,
            MaxSizeCalcBasis::None,
            MaxSizeCalcBasis::MinContent,
            MaxSizeCalcBasis::MaxContent,
            MaxSizeCalcBasis::Stretch,
            MaxSizeCalcBasis::FitContent,
            MaxSizeCalcBasis::Contain,
        ] {
            assert!(MaxSizeOf::calc_size(basis, dependent.clone()).is_ok());
        }
        assert!(MaxSizeOf::calc_size(MaxSizeCalcBasis::Any, independent.clone()).is_ok());
        assert_eq!(
            MaxSizeOf::calc_size(MaxSizeCalcBasis::Any, dependent.clone()),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );

        for basis in [
            FlexBasisCalcBasis::FullPercentage,
            FlexBasisCalcBasis::Auto,
            FlexBasisCalcBasis::Content,
            FlexBasisCalcBasis::MinContent,
            FlexBasisCalcBasis::MaxContent,
            FlexBasisCalcBasis::Stretch,
            FlexBasisCalcBasis::FitContent,
            FlexBasisCalcBasis::Contain,
        ] {
            assert!(FlexBasisOf::calc_size(basis, dependent.clone()).is_ok());
        }
        assert!(FlexBasisOf::calc_size(FlexBasisCalcBasis::Any, independent).is_ok());
        assert_eq!(
            FlexBasisOf::calc_size(FlexBasisCalcBasis::Any, dependent),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );
    }

    #[test]
    fn property_sizing_defaults_and_keyword_domains_are_exact() {
        assert_eq!(PreferredSizeOf::<f32>::default(), PreferredSizeOf::AUTO);
        assert_eq!(MinSizeOf::<f64>::default(), MinSizeOf::AUTO);
        assert_eq!(MaxSizeOf::<f32>::default(), MaxSizeOf::NONE);
        assert_eq!(FlexBasisOf::<f64>::default(), FlexBasisOf::AUTO);

        assert!(PreferredSizeOf::<f32>::MIN_CONTENT.is_min_content());
        assert!(PreferredSizeOf::<f64>::MAX_CONTENT.is_max_content());
        assert!(MinSizeOf::<f32>::AUTO.is_auto());
        assert!(MaxSizeOf::<f64>::NONE.is_none());
        assert!(FlexBasisOf::<f32>::CONTENT.is_content());

        let preferred_keywords = [
            PreferredSizeOf::<f32>::AUTO,
            PreferredSizeOf::MIN_CONTENT,
            PreferredSizeOf::MAX_CONTENT,
            PreferredSizeOf::STRETCH,
            PreferredSizeOf::FIT_CONTENT,
            PreferredSizeOf::CONTAIN,
        ];
        let minimum_keywords = [
            MinSizeOf::<f64>::AUTO,
            MinSizeOf::MIN_CONTENT,
            MinSizeOf::MAX_CONTENT,
            MinSizeOf::STRETCH,
            MinSizeOf::FIT_CONTENT,
            MinSizeOf::CONTAIN,
            MinSizeOf::ZERO,
        ];
        let maximum_keywords = [
            MaxSizeOf::<f32>::NONE,
            MaxSizeOf::MIN_CONTENT,
            MaxSizeOf::MAX_CONTENT,
            MaxSizeOf::STRETCH,
            MaxSizeOf::FIT_CONTENT,
            MaxSizeOf::CONTAIN,
            MaxSizeOf::ZERO,
        ];
        let flex_keywords = [
            FlexBasisOf::<f64>::AUTO,
            FlexBasisOf::CONTENT,
            FlexBasisOf::MIN_CONTENT,
            FlexBasisOf::MAX_CONTENT,
            FlexBasisOf::STRETCH,
            FlexBasisOf::FIT_CONTENT,
            FlexBasisOf::CONTAIN,
            FlexBasisOf::ZERO,
        ];
        assert_eq!(preferred_keywords.len(), 6);
        assert_eq!(minimum_keywords.len(), 7);
        assert_eq!(maximum_keywords.len(), 7);
        assert_eq!(flex_keywords.len(), 8);
        assert!(
            !format!(
                "{preferred_keywords:?}{minimum_keywords:?}{maximum_keywords:?}{flex_keywords:?}"
            )
            .is_empty()
        );
    }

    #[test]
    fn property_sizing_construction_and_calc_size_rules_cover_both_scalar_lanes() {
        let f32_value = LengthPercentageOf::px(12.0f32).expect("finite px");
        let f64_value = LengthPercentageOf::px(12.0f64).expect("finite px");
        assert_eq!(
            PreferredSizeOf::value(f32_value),
            PreferredSizeOf::calculation(px_f32(12.0))
        );
        assert_eq!(
            MinSizeOf::value(f64_value),
            MinSizeOf::calculation(px_f64(12.0))
        );

        let independent_f32 = CalcSizeCalculationOf::value(f32_value);
        let independent_f64 = CalcSizeCalculationOf::value(f64_value);
        let dependent_f32 = CalcSizeCalculationOf::<f32>::size();
        let dependent_f64 = CalcSizeCalculationOf::<f64>::size();

        assert!(PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Any, independent_f32).is_ok());
        assert!(MinSizeOf::calc_size(MinSizeCalcBasis::Auto, dependent_f32.clone()).is_ok());
        assert!(MaxSizeOf::calc_size(MaxSizeCalcBasis::None, dependent_f64.clone()).is_ok());
        assert!(FlexBasisOf::calc_size(FlexBasisCalcBasis::Content, dependent_f64).is_ok());
        assert_eq!(
            PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Any, dependent_f32),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );
        assert_eq!(
            MaxSizeOf::calc_size(MaxSizeCalcBasis::Any, CalcSizeCalculationOf::<f64>::size()),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );
        assert_eq!(
            MinSizeOf::calc_size(MinSizeCalcBasis::Any, CalcSizeCalculationOf::<f64>::size()),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );
        assert_eq!(
            FlexBasisOf::calc_size(
                FlexBasisCalcBasis::Any,
                CalcSizeCalculationOf::<f32>::size()
            ),
            Err(CalcSizeConstructionError::SizeReferenceWithAnyBasis)
        );

        assert!(FlexBasisOf::calc_size(FlexBasisCalcBasis::Any, independent_f64).is_ok());
        assert_eq!(
            MinSizeOf::<f32>::ZERO,
            MinSizeOf::value(LengthPercentageOf::ZERO)
        );
        assert_eq!(
            MaxSizeOf::<f64>::ZERO,
            MaxSizeOf::value(LengthPercentageOf::ZERO)
        );
        assert_eq!(
            FlexBasisOf::<f32>::ZERO,
            FlexBasisOf::value(LengthPercentageOf::ZERO)
        );

        let negative_zero_f32 = LengthPercentageOf::px(-0.0f32).expect("finite signed zero");
        let negative_zero_f64 = LengthPercentageOf::px(-0.0f64).expect("finite signed zero");
        assert_eq!(MinSizeOf::<f32>::ZERO, MinSizeOf::value(negative_zero_f32));
        assert_eq!(MaxSizeOf::<f32>::ZERO, MaxSizeOf::value(negative_zero_f32));
        assert_eq!(
            FlexBasisOf::<f32>::ZERO,
            FlexBasisOf::value(negative_zero_f32)
        );
        assert_eq!(MinSizeOf::<f64>::ZERO, MinSizeOf::value(negative_zero_f64));
        assert_eq!(MaxSizeOf::<f64>::ZERO, MaxSizeOf::value(negative_zero_f64));
        assert_eq!(
            FlexBasisOf::<f64>::ZERO,
            FlexBasisOf::value(negative_zero_f64)
        );
    }

    #[test]
    fn property_sizing_calc_size_accepts_every_non_any_basis_and_independent_any() {
        assert_property_sizing_lane::<f32>();
        assert_property_sizing_lane::<f64>();
    }

    #[test]
    fn property_sizing_exhaustive_views_preserve_constructor_semantics() {
        let calculation_f32 = px_f32(9.0);
        let calculation_f64 = px_f64(11.0);

        assert!(matches!(
            PreferredSizeOf::calculation(calculation_f32.clone()).view(),
            PreferredSizeView::Calculation(value) if value == &calculation_f32
        ));
        assert!(matches!(
            MinSizeOf::fit_content_function(calculation_f64.clone()).view(),
            MinSizeView::FitContentFunction(value) if value == &calculation_f64
        ));
        assert!(matches!(
            MaxSizeOf::calc_size(MaxSizeCalcBasis::None, CalcSizeCalculationOf::<f32>::size())
                .expect("valid basis")
                .view(),
            MaxSizeView::CalcSize(MaxSizeCalcBasis::None, value) if value.depends_on_size()
        ));
        assert!(matches!(
            FlexBasisOf::fit_content_function(calculation_f64.clone()).view(),
            FlexBasisView::FitContentFunction(value) if value == &calculation_f64
        ));
    }

    #[test]
    fn calc_size_calculation_constructors_cover_values_size_and_nonempty_extrema() {
        let value = CalcSizeCalculationOf::value(
            LengthPercentageOf::from_coefficients(5.0f32, 0.25).expect("finite length-percentage"),
        );
        let size = CalcSizeCalculationOf::<f32>::size();
        let one_min = CalcSizeCalculationOf::min(vec![calc_px_f32(7.0)]).expect("nonempty min");
        let many_min = CalcSizeCalculationOf::min(vec![calc_px_f32(8.0), calc_px_f32(-3.0)])
            .expect("nonempty min");
        let one_max = CalcSizeCalculationOf::max(vec![calc_px_f64(-4.0)]).expect("nonempty max");
        let many_max = CalcSizeCalculationOf::max(vec![calc_px_f64(-9.0), calc_px_f64(-2.0)])
            .expect("nonempty max");

        assert_eq!(
            resolved_calc_f32(
                &value,
                None,
                PercentageBasisOf::definite(20.0).expect("finite percentage basis")
            ),
            10.0
        );
        assert_eq!(
            resolved_calc_f32(&size, Some(size_f32(13.0)), PercentageBasisOf::MISSING),
            13.0
        );
        assert_eq!(
            resolved_calc_f32(&one_min, None, PercentageBasisOf::MISSING),
            7.0
        );
        assert_eq!(
            resolved_calc_f32(&many_min, None, PercentageBasisOf::MISSING),
            -3.0
        );
        assert_eq!(
            one_max
                .resolve_against(None, PercentageBasisOf::MISSING)
                .value,
            Some(-4.0)
        );
        assert_eq!(
            many_max
                .resolve_against(None, PercentageBasisOf::MISSING)
                .value,
            Some(-2.0)
        );
        assert_eq!(
            CalcSizeCalculationOf::<f32>::min(Vec::new()),
            Err(SizingCalculationError::EmptyArguments)
        );
        assert_eq!(
            CalcSizeCalculationOf::<f64>::max(Vec::new()),
            Err(SizingCalculationError::EmptyArguments)
        );
    }

    #[test]
    fn calc_size_calculation_invalid_coefficients_return_exact_owned_errors() {
        assert_eq!(
            CalcSizeCalculationOf::<f32>::from_coefficients(f32::INFINITY, 0.0, 0.0),
            Err(CalcSizeCalculationErrorOf::InvalidAbsolutePx(
                FiniteScalarErrorOf::NonFinite {
                    value: f32::INFINITY
                }
            ))
        );
        assert!(matches!(
            CalcSizeCalculationOf::<f64>::from_coefficients(0.0, f64::NAN, 0.0),
            Err(CalcSizeCalculationErrorOf::InvalidPercentFraction(
                FiniteScalarErrorOf::NonFinite { value }
            )) if value.is_nan()
        ));
        assert_eq!(
            CalcSizeCalculationOf::<f64>::from_coefficients(0.0, 0.0, f64::INFINITY),
            Err(CalcSizeCalculationErrorOf::InvalidSizeFraction(
                FiniteScalarErrorOf::NonFinite {
                    value: f64::INFINITY
                }
            ))
        );
    }

    #[test]
    fn calc_size_calculation_coefficients_canonicalize_signed_zero() {
        let f32_calculation = CalcSizeCalculationOf::from_coefficients(-0.0f32, -0.0, -0.0)
            .expect("finite coefficients");
        let f64_calculation = CalcSizeCalculationOf::from_coefficients(-0.0f64, -0.0, -0.0)
            .expect("finite coefficients");

        assert!(!f32_calculation.depends_on_size());
        assert!(!f64_calculation.depends_on_size());
        assert_eq!(
            resolved_calc_f32(&f32_calculation, None, PercentageBasisOf::MISSING).to_bits(),
            0.0f32.to_bits()
        );
        assert_eq!(
            f64_calculation
                .resolve_against(None, PercentageBasisOf::MISSING)
                .value
                .expect("resolved zero")
                .to_bits(),
            0.0f64.to_bits()
        );
    }

    #[test]
    fn calc_size_calculation_depends_on_size_is_exact_across_program() {
        let independent = CalcSizeCalculationOf::<f32>::from_coefficients(3.0, 0.5, 0.0)
            .expect("finite coefficients");
        let dependent = CalcSizeCalculationOf::<f32>::size();
        let nested = CalcSizeCalculationOf::clamp(
            Some(independent.clone()),
            CalcSizeCalculationOf::max(vec![independent, dependent]).expect("nonempty max"),
            None,
        );

        assert!(!calc_px_f32(1.0).depends_on_size());
        assert!(nested.depends_on_size());
    }

    #[test]
    fn calc_size_calculation_percentage_basis_is_explicit_and_missing_contributes_zero() {
        let calculation = CalcSizeCalculationOf::<f32>::from_coefficients(4.0, 0.5, 2.0)
            .expect("finite coefficients");

        assert_eq!(
            resolved_calc_f32(
                &calculation,
                Some(size_f32(3.0)),
                PercentageBasisOf::definite(20.0).expect("finite percentage basis")
            ),
            20.0
        );
        assert_eq!(
            resolved_calc_f32(
                &calculation,
                Some(size_f32(3.0)),
                PercentageBasisOf::MISSING
            ),
            10.0
        );
    }

    #[test]
    fn calc_size_calculation_clamp_supports_all_endpoints_and_minimum_wins() {
        let none = CalcSizeCalculationOf::clamp(None, calc_px_f32(12.0), None);
        let minimum =
            CalcSizeCalculationOf::clamp(Some(calc_px_f32(15.0)), calc_px_f32(12.0), None);
        let maximum =
            CalcSizeCalculationOf::clamp(None, calc_px_f32(12.0), Some(calc_px_f32(10.0)));
        let both = CalcSizeCalculationOf::clamp(
            Some(calc_px_f32(5.0)),
            calc_px_f32(12.0),
            Some(calc_px_f32(20.0)),
        );
        let conflicting = CalcSizeCalculationOf::clamp(
            Some(calc_px_f32(20.0)),
            calc_px_f32(15.0),
            Some(calc_px_f32(10.0)),
        );

        assert_eq!(
            resolved_calc_f32(&none, None, PercentageBasisOf::MISSING),
            12.0
        );
        assert_eq!(
            resolved_calc_f32(&minimum, None, PercentageBasisOf::MISSING),
            15.0
        );
        assert_eq!(
            resolved_calc_f32(&maximum, None, PercentageBasisOf::MISSING),
            10.0
        );
        assert_eq!(
            resolved_calc_f32(&both, None, PercentageBasisOf::MISSING),
            12.0
        );
        assert_eq!(
            resolved_calc_f32(&conflicting, None, PercentageBasisOf::MISSING),
            20.0
        );
    }

    #[test]
    fn calc_size_calculation_nested_program_keeps_negative_finite_result() {
        let minimum = CalcSizeCalculationOf::min(vec![calc_px_f32(-12.0), calc_px_f32(-8.0)])
            .expect("nonempty min");
        let preferred = CalcSizeCalculationOf::max(vec![calc_px_f32(-10.0), calc_px_f32(-6.0)])
            .expect("nonempty max");
        let nested =
            CalcSizeCalculationOf::clamp(Some(minimum), preferred, Some(calc_px_f32(-7.0)));

        assert_eq!(
            resolved_calc_f32(&nested, None, PercentageBasisOf::MISSING),
            -7.0
        );
    }

    #[test]
    fn calc_size_calculation_missing_size_is_syntactic_and_precedes_overflow() {
        let dominated =
            CalcSizeCalculationOf::min(vec![calc_px_f32(0.0), CalcSizeCalculationOf::size()])
                .expect("nonempty min");
        let canceling = CalcSizeCalculationOf::from_coefficients(10.0f32, 0.0, -1.0)
            .expect("finite coefficients");
        let overflowing = CalcSizeCalculationOf::from_coefficients(f32::MAX, f32::MAX, 1.0)
            .expect("finite coefficients");
        let nested = CalcSizeCalculationOf::clamp(
            Some(calc_px_f32(0.0)),
            CalcSizeCalculationOf::max(vec![dominated.clone(), canceling.clone()])
                .expect("nonempty max"),
            Some(calc_px_f32(20.0)),
        );

        for calculation in [dominated, canceling, nested, overflowing] {
            let resolution = calculation.resolve_against(
                None,
                PercentageBasisOf::definite(f32::MAX).expect("finite percentage basis"),
            );
            assert_eq!(resolution.value, None);
            assert_eq!(resolution.status(), LengthResolutionStatus::MissingBasis);
        }
    }

    #[test]
    fn calc_size_calculation_overflow_is_invalid_numeric_for_f32_and_f64() {
        let f32_calculation = CalcSizeCalculationOf::from_coefficients(f32::MAX, 1.0, 0.0)
            .expect("finite coefficients");
        let f64_calculation = CalcSizeCalculationOf::from_coefficients(f64::MAX, 0.0, 1.0)
            .expect("finite coefficients");

        assert_eq!(
            f32_calculation
                .resolve_against(
                    None,
                    PercentageBasisOf::definite(f32::MAX).expect("finite percentage basis")
                )
                .status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f32::INFINITY
            }
        );
        assert_eq!(
            f64_calculation
                .resolve_against(Some(size_f64(f64::MAX)), PercentageBasisOf::MISSING)
                .status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f64::INFINITY
            }
        );
    }

    #[test]
    fn calc_size_calculation_deep_nesting_evaluates_and_drops_iteratively() {
        const DEPTH: usize = 100_000;

        let mut calculation = calc_px_f32(3.0);
        for depth in 0..DEPTH {
            calculation = if depth % 2 == 0 {
                CalcSizeCalculationOf::max(vec![calculation]).expect("nonempty max")
            } else {
                CalcSizeCalculationOf::clamp(None, calculation, None)
            };
        }

        assert_eq!(
            resolved_calc_f32(&calculation, None, PercentageBasisOf::MISSING),
            3.0
        );
        drop(calculation);
    }

    #[test]
    fn sizing_calculation_value_and_nonempty_min_max_resolve_for_f32_and_f64() {
        let one_min = SizingCalculationOf::min(vec![px_f32(-3.0)]).expect("nonempty min");
        let many_min = SizingCalculationOf::min(vec![px_f32(8.0), px_f32(-4.0), px_f32(2.0)])
            .expect("nonempty min");
        let one_max = SizingCalculationOf::max(vec![px_f64(7.0)]).expect("nonempty max");
        let many_max = SizingCalculationOf::max(vec![px_f64(-8.0), px_f64(-2.0), px_f64(-5.0)])
            .expect("nonempty max");

        assert_eq!(resolved_f32(&one_min), -3.0);
        assert_eq!(resolved_f32(&many_min), -4.0);
        assert_eq!(
            one_max.resolve_against(PercentageBasisOf::MISSING).value,
            Some(7.0)
        );
        assert_eq!(
            many_max.resolve_against(PercentageBasisOf::MISSING).value,
            Some(-2.0)
        );
    }

    #[test]
    fn sizing_calculation_empty_min_and_max_return_typed_error() {
        assert_eq!(
            SizingCalculationOf::<f32>::min(Vec::new()),
            Err(SizingCalculationError::EmptyArguments)
        );
        assert_eq!(
            SizingCalculationOf::<f64>::max(Vec::new()),
            Err(SizingCalculationError::EmptyArguments)
        );
    }

    #[test]
    fn sizing_calculation_clamp_supports_every_optional_endpoint_form() {
        let no_endpoints = SizingCalculationOf::clamp(None, px_f32(12.0), None);
        let minimum_only = SizingCalculationOf::clamp(Some(px_f32(15.0)), px_f32(12.0), None);
        let maximum_only = SizingCalculationOf::clamp(None, px_f32(12.0), Some(px_f32(10.0)));
        let both = SizingCalculationOf::clamp(Some(px_f32(5.0)), px_f32(12.0), Some(px_f32(20.0)));

        assert_eq!(resolved_f32(&no_endpoints), 12.0);
        assert_eq!(resolved_f32(&minimum_only), 15.0);
        assert_eq!(resolved_f32(&maximum_only), 10.0);
        assert_eq!(resolved_f32(&both), 12.0);
    }

    #[test]
    fn sizing_calculation_clamp_conflicting_bounds_are_minimum_wins() {
        let calculation =
            SizingCalculationOf::clamp(Some(px_f32(20.0)), px_f32(15.0), Some(px_f32(10.0)));

        assert_eq!(resolved_f32(&calculation), 20.0);
    }

    #[test]
    fn sizing_calculation_nested_programs_resolve_without_property_range_clamp() {
        let inner_min =
            SizingCalculationOf::min(vec![px_f32(-12.0), px_f32(-8.0)]).expect("nonempty min");
        let inner_max =
            SizingCalculationOf::max(vec![px_f32(-10.0), px_f32(-6.0)]).expect("nonempty max");
        let calculation =
            SizingCalculationOf::clamp(Some(inner_min), inner_max, Some(px_f32(-7.0)));

        assert_eq!(resolved_f32(&calculation), -7.0);
    }

    #[test]
    fn sizing_calculation_missing_basis_is_syntactic_across_complete_program() {
        let percentage = SizingCalculationOf::value(
            LengthPercentageOf::from_percent_fraction(0.5).expect("finite percentage"),
        );
        let dominated =
            SizingCalculationOf::min(vec![px_f32(0.0), percentage]).expect("nonempty min");
        let nested = SizingCalculationOf::clamp(
            Some(px_f32(0.0)),
            SizingCalculationOf::max(vec![px_f32(10.0), dominated]).expect("nonempty max"),
            Some(px_f32(20.0)),
        );

        assert!(nested.depends_on_basis());
        let resolution = nested.resolve_against(PercentageBasisOf::MISSING);
        assert_eq!(resolution.value, None);
        assert!(resolution.depends_on_basis);
        assert_eq!(resolution.status(), LengthResolutionStatus::MissingBasis);
        assert_eq!(
            nested
                .resolve_against(PercentageBasisOf::definite(100.0).expect("finite basis"))
                .value,
            Some(10.0)
        );
    }

    #[test]
    fn sizing_calculation_all_zero_percentages_resolve_with_missing_basis_at_depth() {
        let zero_percentage = SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(7.0f64, -0.0).expect("finite coefficients"),
        );
        let nested = SizingCalculationOf::clamp(
            Some(px_f64(2.0)),
            SizingCalculationOf::max(vec![px_f64(4.0), zero_percentage]).expect("nonempty max"),
            Some(px_f64(9.0)),
        );

        assert!(!nested.depends_on_basis());
        let resolution = nested.resolve_against(PercentageBasisOf::MISSING);
        assert_eq!(resolution.value, Some(7.0));
        assert!(!resolution.depends_on_basis);
        assert_eq!(resolution.status(), LengthResolutionStatus::Resolved);
    }

    #[test]
    fn sizing_calculation_signed_zero_is_canonical_in_both_scalar_lanes() {
        let f32_calculation = SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(-0.0f32, -0.0).expect("finite coefficients"),
        );
        let f64_calculation = SizingCalculationOf::clamp(None, px_f64(-0.0), None);

        assert_eq!(
            f32_calculation
                .resolve_against(PercentageBasisOf::MISSING)
                .value
                .expect("resolved zero")
                .to_bits(),
            0.0f32.to_bits()
        );
        assert_eq!(
            f64_calculation
                .resolve_against(PercentageBasisOf::MISSING)
                .value
                .expect("resolved zero")
                .to_bits(),
            0.0f64.to_bits()
        );
    }

    #[test]
    fn sizing_calculation_finite_overflow_returns_typed_invalid_numeric() {
        let f32_calculation = SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients"),
        );
        let f64_calculation = SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(f64::MAX, 1.0).expect("finite coefficients"),
        );

        let f32_resolution = f32_calculation
            .resolve_against(PercentageBasisOf::definite(f32::MAX).expect("finite basis"));
        let f64_resolution = f64_calculation
            .resolve_against(PercentageBasisOf::definite(f64::MAX).expect("finite basis"));

        assert_eq!(f32_resolution.value, None);
        assert_eq!(
            f32_resolution.status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f32::INFINITY
            }
        );
        assert_eq!(f64_resolution.value, None);
        assert_eq!(
            f64_resolution.status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f64::INFINITY
            }
        );
    }

    #[test]
    fn sizing_calculation_deep_nesting_evaluates_and_drops_iteratively() {
        const DEPTH: usize = 100_000;

        let mut calculation = px_f32(3.0);
        for depth in 0..DEPTH {
            calculation = if depth % 2 == 0 {
                SizingCalculationOf::min(vec![calculation]).expect("nonempty min")
            } else {
                SizingCalculationOf::clamp(None, calculation, None)
            };
        }

        assert_eq!(resolved_f32(&calculation), 3.0);
        drop(calculation);
    }

    fn assert_fri04_c03_property_resolution_lane<S: LayoutScalar>(largest: S) {
        let px = |value: f64| {
            SizingCalculationOf::value(
                LengthPercentageOf::px(S::from_f64(value)).expect("finite sizing value"),
            )
        };
        let percentage = SizingCalculationOf::value(
            LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).expect("finite percentage"),
        );
        let nested = SizingCalculationOf::clamp(
            Some(SizingCalculationOf::min(vec![px(-20.0), px(-10.0)]).expect("nonempty minimum")),
            SizingCalculationOf::max(vec![px(12.0), px(18.0)]).expect("nonempty maximum"),
            Some(px(15.0)),
        );
        let nested_resolution = PreferredSizeOf::calculation(nested)
            .resolve_simple_with_status(Some(S::from_f64(100.0)))
            .expect("valid nested calculation remains numeric");
        assert_eq!(nested_resolution.status(), LengthResolutionStatus::Resolved);
        assert_eq!(nested_resolution.value, Some(S::from_f64(15.0)));

        let missing = MaxSizeOf::calculation(
            SizingCalculationOf::max(vec![px(10.0), percentage]).expect("nonempty maximum"),
        )
        .resolve_simple_with_status(None)
        .expect("basis-dependent calculation remains a numeric request");
        assert_eq!(missing.status(), LengthResolutionStatus::MissingBasis);

        let overflowing = SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(largest, S::ONE)
                .expect("finite overflow coefficients"),
        );
        let overflow = MinSizeOf::calculation(overflowing)
            .resolve_simple_with_status(Some(largest))
            .expect("overflow remains a numeric request");
        assert_eq!(
            overflow.status(),
            LengthResolutionStatus::InvalidNumeric { value: S::INFINITY }
        );
    }

    #[test]
    fn fri04_c03_leaf_root_nested_property_programs_preserve_status_in_both_scalar_lanes() {
        assert_fri04_c03_property_resolution_lane::<f32>(f32::MAX);
        assert_fri04_c03_property_resolution_lane::<f64>(f64::MAX);
    }

    #[test]
    fn fri04_c03_leaf_root_negative_property_results_clamp_in_both_scalar_lanes() {
        fn assert_lane<S: LayoutScalar>() {
            let negative = MaxSizeOf::calculation(
                SizingCalculationOf::min(vec![
                    SizingCalculationOf::value(
                        LengthPercentageOf::px(S::from_f64(-8.0)).expect("finite sizing value"),
                    ),
                    SizingCalculationOf::value(
                        LengthPercentageOf::px(S::from_f64(-3.0)).expect("finite sizing value"),
                    ),
                ])
                .expect("nonempty minimum"),
            )
            .resolve_simple_with_status(None)
            .expect("valid nested calculation remains numeric");

            assert_eq!(negative.status(), LengthResolutionStatus::Resolved);
            assert_eq!(negative.value, Some(S::ZERO));
        }

        assert_lane::<f32>();
        assert_lane::<f64>();
    }
}
