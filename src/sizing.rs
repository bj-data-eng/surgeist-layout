use crate::{
    FiniteScalarErrorOf, LayoutScalar, LengthPercentageOf, LengthResolutionOf, NonNegativeFiniteOf,
    NumericResolutionOf, PercentageBasisOf,
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
        CalcSizeCalculationErrorOf, CalcSizeCalculationOf, SizingCalculationError,
        SizingCalculationOf,
    };
    use crate::{
        FiniteScalarErrorOf, LengthPercentageOf, LengthResolutionStatus, NonNegativeFiniteOf,
        PercentageBasisOf,
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
}
