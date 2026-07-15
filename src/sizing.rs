use crate::{
    LayoutScalar, LengthPercentageOf, LengthResolutionOf, NumericResolutionOf, PercentageBasisOf,
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
    use super::{SizingCalculationError, SizingCalculationOf};
    use crate::{LengthPercentageOf, LengthResolutionStatus, PercentageBasisOf};

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
