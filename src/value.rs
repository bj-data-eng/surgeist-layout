use super::{DefaultScalar, LayoutScalar};
use core::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatioOf<S: LayoutScalar = DefaultScalar>(S);

pub type AspectRatio = AspectRatioOf<DefaultScalar>;

impl<S: LayoutScalar> AspectRatioOf<S> {
    #[must_use]
    pub fn new(value: S) -> Option<Self> {
        (value.is_finite() && value > S::ZERO).then_some(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridLine(isize);

impl GridLine {
    #[must_use]
    pub const fn new(value: isize) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    #[must_use]
    pub const fn get(self) -> isize {
        self.0
    }
}

impl TryFrom<isize> for GridLine {
    type Error = ();

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridSpan(NonZeroUsize);

impl GridSpan {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

impl TryFrom<usize> for GridSpan {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableOf<S: LayoutScalar = DefaultScalar> {
    Definite(S),
    MinContent,
    MaxContent,
}

pub type Available = AvailableOf<DefaultScalar>;

impl<S: LayoutScalar> AvailableOf<S> {
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    #[must_use]
    pub const fn definite(value: S) -> Self {
        Self::Definite(value)
    }

    #[must_use]
    pub const fn into_option(self) -> Option<S> {
        match self {
            Self::Definite(value) => Some(value),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub fn roughly_eq(self, other: Self) -> bool {
        match (self, other) {
            (Self::Definite(a), Self::Definite(b)) => (a - b).abs() < S::EPSILON,
            (Self::MinContent, Self::MinContent) | (Self::MaxContent, Self::MaxContent) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CalcId(u32);

impl CalcId {
    pub(crate) const fn from_store_index(index: u32) -> Self {
        Self(index)
    }

    #[cfg(test)]
    #[must_use]
    pub const fn from_raw_for_tests(index: u32) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalcResolutionStatus {
    Resolved,
    MissingBasis,
    MissingResolver,
    MissingExpression,
    NonNumeric,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CalcUnresolvedReason {
    Basis,
    Resolver,
    Expression,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedLengthAutoOf<S: LayoutScalar = DefaultScalar> {
    Auto,
    Resolved(S),
    Unresolved(CalcUnresolvedReason),
}

pub type ResolvedLengthAuto = ResolvedLengthAutoOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalcResolutionOf<S: LayoutScalar = DefaultScalar> {
    pub value: Option<S>,
    pub depends_on_basis: bool,
    status: CalcResolutionStatus,
}

pub type CalcResolution = CalcResolutionOf<DefaultScalar>;

impl<S: LayoutScalar> CalcResolutionOf<S> {
    #[must_use]
    pub const fn definite(value: S, depends_on_basis: bool) -> Self {
        Self {
            value: Some(value),
            depends_on_basis,
            status: CalcResolutionStatus::Resolved,
        }
    }

    #[must_use]
    pub const fn unresolved(depends_on_basis: bool) -> Self {
        Self {
            value: None,
            depends_on_basis,
            status: CalcResolutionStatus::MissingBasis,
        }
    }

    #[must_use]
    pub const fn missing_expression() -> Self {
        Self {
            value: None,
            depends_on_basis: false,
            status: CalcResolutionStatus::MissingExpression,
        }
    }

    #[must_use]
    pub const fn missing_resolver() -> Self {
        Self {
            value: None,
            depends_on_basis: false,
            status: CalcResolutionStatus::MissingResolver,
        }
    }

    #[must_use]
    pub const fn non_numeric() -> Self {
        Self {
            value: None,
            depends_on_basis: false,
            status: CalcResolutionStatus::NonNumeric,
        }
    }

    #[must_use]
    pub const fn status(self) -> CalcResolutionStatus {
        self.status
    }

    #[must_use]
    pub const fn is_missing_expression(self) -> bool {
        matches!(self.status, CalcResolutionStatus::MissingExpression)
    }

    #[must_use]
    pub const fn unresolved_reason(self) -> Option<CalcUnresolvedReason> {
        match self.status {
            CalcResolutionStatus::Resolved | CalcResolutionStatus::NonNumeric => None,
            CalcResolutionStatus::MissingBasis => Some(CalcUnresolvedReason::Basis),
            CalcResolutionStatus::MissingResolver => Some(CalcUnresolvedReason::Resolver),
            CalcResolutionStatus::MissingExpression => Some(CalcUnresolvedReason::Expression),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CalcGeneration(u64);

impl CalcGeneration {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn static_no_calc() -> Self {
        Self(0)
    }
}

pub trait CalcResolver<S: LayoutScalar = DefaultScalar> {
    fn resolve_calc(&self, id: CalcId, basis: Option<S>) -> CalcResolutionOf<S>;
    fn calc_generation(&self) -> CalcGeneration;
    fn calc_depends_on_basis(&self, id: CalcId) -> bool;
    fn calc_percent_fraction(&self, id: CalcId) -> Option<S> {
        Some(if self.calc_depends_on_basis(id) {
            S::ONE
        } else {
            S::ZERO
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoCalcResolver;

impl<S: LayoutScalar> CalcResolver<S> for NoCalcResolver {
    fn resolve_calc(&self, _id: CalcId, _basis: Option<S>) -> CalcResolutionOf<S> {
        CalcResolutionOf::missing_resolver()
    }

    fn calc_generation(&self) -> CalcGeneration {
        CalcGeneration::static_no_calc()
    }

    fn calc_depends_on_basis(&self, _id: CalcId) -> bool {
        false
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutCalcStoreOf<S: LayoutScalar = DefaultScalar> {
    expressions: Vec<CalcExpressionOf<S>>,
}

pub type LayoutCalcStore = LayoutCalcStoreOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutCalcStoreOf<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }

    pub fn push(&mut self, expression: CalcExpressionOf<S>) -> CalcId {
        let index = u32::try_from(self.expressions.len())
            .expect("layout calc store exhausted CalcId range");
        let id = CalcId::from_store_index(index);
        self.expressions.push(expression);
        id
    }

    #[must_use]
    pub fn get(&self, id: CalcId) -> Option<&CalcExpressionOf<S>> {
        self.expressions.get(id.index() as usize)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.expressions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }
}

impl<S: LayoutScalar> CalcResolver<S> for LayoutCalcStoreOf<S> {
    fn resolve_calc(&self, id: CalcId, basis: Option<S>) -> CalcResolutionOf<S> {
        self.get(id)
            .map_or(CalcResolutionOf::missing_expression(), |expression| {
                expression.resolve(basis)
            })
    }

    fn calc_generation(&self) -> CalcGeneration {
        CalcGeneration::new(self.len() as u64)
    }

    fn calc_depends_on_basis(&self, id: CalcId) -> bool {
        self.get(id).is_some_and(CalcExpressionOf::depends_on_basis)
    }

    fn calc_percent_fraction(&self, id: CalcId) -> Option<S> {
        self.get(id).map(CalcExpressionOf::percent_fraction)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalcExpressionOf<S: LayoutScalar = DefaultScalar> {
    terms: Vec<CalcTermOf<S>>,
}

pub type CalcExpression = CalcExpressionOf<DefaultScalar>;

impl<S: LayoutScalar> CalcExpressionOf<S> {
    #[must_use]
    pub fn sum(terms: impl IntoIterator<Item = CalcTermOf<S>>) -> Self {
        Self {
            terms: terms.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn depends_on_basis(&self) -> bool {
        self.terms
            .iter()
            .any(|term| matches!(term, CalcTermOf::Percent(_)))
    }

    #[must_use]
    pub fn percent_fraction(&self) -> S {
        self.terms.iter().fold(S::ZERO, |sum, term| match *term {
            CalcTermOf::Px(_) => sum,
            CalcTermOf::Percent(percent) => sum + percent,
        })
    }

    #[must_use]
    pub fn resolve(&self, basis: Option<S>) -> CalcResolutionOf<S> {
        let mut value = S::ZERO;
        let mut depends_on_basis = false;

        for term in &self.terms {
            match *term {
                CalcTermOf::Px(px) => value = value + px,
                CalcTermOf::Percent(percent) => {
                    depends_on_basis = true;
                    let Some(basis) = basis else {
                        return CalcResolutionOf::unresolved(true);
                    };
                    value = value + percent * basis;
                }
            }
        }

        CalcResolutionOf::definite(value, depends_on_basis)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CalcTermOf<S: LayoutScalar = DefaultScalar> {
    Px(S),
    Percent(S),
}

pub type CalcTerm = CalcTermOf<DefaultScalar>;

impl<S: LayoutScalar> CalcTermOf<S> {
    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Percent(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthOf<S: LayoutScalar = DefaultScalar> {
    Normal,
    Px(S),
    Percent(S),
    Calc(CalcId),
}

pub type Length = LengthOf<DefaultScalar>;

impl<S: LayoutScalar> LengthOf<S> {
    pub const NORMAL: Self = Self::Normal;
    pub const ZERO: Self = Self::Px(S::ZERO);

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Percent(value)
    }

    #[must_use]
    pub const fn calc(id: CalcId) -> Self {
        Self::Calc(id)
    }

    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        matches!(self, Self::Percent(_) | Self::Calc(_))
    }

    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &dyn CalcResolver<S>) -> bool {
        match self {
            Self::Calc(id) => resolver.calc_depends_on_basis(id),
            _ => self.depends_on_basis(),
        }
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &dyn CalcResolver<S>) -> S {
        match self {
            Self::Percent(value) => value,
            Self::Calc(id) => resolver.calc_percent_fraction(id).unwrap_or_else(|| {
                if resolver.calc_depends_on_basis(id) {
                    S::ONE
                } else {
                    S::ZERO
                }
            }),
            Self::Normal | Self::Px(_) => S::ZERO,
        }
    }

    #[must_use]
    pub const fn requires_resolver(self) -> bool {
        matches!(self, Self::Calc(_))
    }

    #[must_use]
    pub fn resolve(self, basis: S) -> S {
        match self {
            Self::Normal => S::ZERO,
            Self::Px(value) => value,
            Self::Percent(value) => value * basis,
            Self::Calc(_) => panic!("calc values require an explicit resolver"),
        }
    }

    #[must_use]
    pub fn resolve_or_zero(self, basis: Option<S>) -> S {
        self.resolve_optional(basis).unwrap_or(S::ZERO)
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Normal => Some(S::ZERO),
            Self::Px(value) => Some(value),
            Self::Percent(value) => basis.map(|basis| value * basis),
            Self::Calc(_) => panic!("calc values require an explicit resolver"),
        }
    }

    #[must_use]
    pub fn resolve_with(self, basis: Option<S>, resolver: &dyn CalcResolver<S>) -> Option<S> {
        self.resolve_with_status(basis, resolver).value
    }

    #[must_use]
    pub fn resolve_with_status(
        self,
        basis: Option<S>,
        resolver: &dyn CalcResolver<S>,
    ) -> CalcResolutionOf<S> {
        match self {
            Self::Normal => CalcResolutionOf::definite(S::ZERO, false),
            Self::Px(value) => CalcResolutionOf::definite(value, false),
            Self::Percent(value) => basis.map_or(CalcResolutionOf::unresolved(true), |basis| {
                CalcResolutionOf::definite(value * basis, true)
            }),
            Self::Calc(id) => resolver.resolve_calc(id, basis),
        }
    }
}

impl<S: LayoutScalar> Default for LengthOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthAutoOf<S: LayoutScalar = DefaultScalar> {
    Px(S),
    Percent(S),
    Calc(CalcId),
    Auto,
}

pub type LengthAuto = LengthAutoOf<DefaultScalar>;

impl<S: LayoutScalar> LengthAutoOf<S> {
    pub const ZERO: Self = Self::Px(S::ZERO);
    pub const AUTO: Self = Self::Auto;

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Percent(value)
    }

    #[must_use]
    pub const fn calc(id: CalcId) -> Self {
        Self::Calc(id)
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        matches!(self, Self::Percent(_) | Self::Calc(_))
    }

    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &dyn CalcResolver<S>) -> bool {
        match self {
            Self::Calc(id) => resolver.calc_depends_on_basis(id),
            _ => self.depends_on_basis(),
        }
    }

    #[must_use]
    pub const fn requires_resolver(self) -> bool {
        matches!(self, Self::Calc(_))
    }

    #[must_use]
    pub fn resolve(self, basis: S) -> Option<S> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => Some(value * basis),
            Self::Calc(_) => panic!("calc values require an explicit resolver"),
            Self::Auto => None,
        }
    }

    #[must_use]
    pub fn resolve_or_zero(self, basis: Option<S>) -> S {
        self.resolve_optional(basis).unwrap_or(S::ZERO)
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => basis.map(|basis| value * basis),
            Self::Calc(_) => panic!("calc values require an explicit resolver"),
            Self::Auto => None,
        }
    }

    #[must_use]
    pub fn resolve_with(self, basis: Option<S>, resolver: &dyn CalcResolver<S>) -> Option<S> {
        self.resolve_with_status(basis, resolver).value
    }

    #[must_use]
    pub fn resolve_with_status(
        self,
        basis: Option<S>,
        resolver: &dyn CalcResolver<S>,
    ) -> CalcResolutionOf<S> {
        match self {
            Self::Px(value) => CalcResolutionOf::definite(value, false),
            Self::Percent(value) => basis.map_or(CalcResolutionOf::unresolved(true), |basis| {
                CalcResolutionOf::definite(value * basis, true)
            }),
            Self::Calc(id) => resolver.resolve_calc(id, basis),
            Self::Auto => CalcResolutionOf::non_numeric(),
        }
    }

    #[must_use]
    pub fn resolve_auto_with_status(
        self,
        basis: Option<S>,
        resolver: &dyn CalcResolver<S>,
    ) -> ResolvedLengthAutoOf<S> {
        if self.is_auto() {
            return ResolvedLengthAutoOf::Auto;
        }

        let resolution = self.resolve_with_status(basis, resolver);
        if let Some(value) = resolution.value {
            return ResolvedLengthAutoOf::Resolved(value);
        }
        if let Some(reason) = resolution.unresolved_reason() {
            return ResolvedLengthAutoOf::Unresolved(reason);
        }

        panic!("non-auto length resolution produced non-numeric status")
    }

    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl<S: LayoutScalar> Default for LengthAutoOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for LengthAutoOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        match value {
            LengthOf::Normal => Self::ZERO,
            LengthOf::Px(value) => Self::Px(value),
            LengthOf::Percent(value) => Self::Percent(value),
            LengthOf::Calc(id) => Self::Calc(id),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DimensionOf<S: LayoutScalar = DefaultScalar> {
    Px(S),
    Percent(S),
    Calc(CalcId),
    Fr(S),
    Auto,
    MinContent,
    MaxContent,
}

pub type Dimension = DimensionOf<DefaultScalar>;

impl<S: LayoutScalar> DimensionOf<S> {
    pub const ZERO: Self = Self::Px(S::ZERO);
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Percent(value)
    }

    #[must_use]
    pub const fn calc(id: CalcId) -> Self {
        Self::Calc(id)
    }

    #[must_use]
    pub const fn fr(value: S) -> Self {
        Self::Fr(value)
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        matches!(self, Self::Percent(_) | Self::Calc(_))
    }

    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &dyn CalcResolver<S>) -> bool {
        match self {
            Self::Calc(id) => resolver.calc_depends_on_basis(id),
            _ => self.depends_on_basis(),
        }
    }

    #[must_use]
    pub const fn requires_resolver(self) -> bool {
        matches!(self, Self::Calc(_))
    }

    #[must_use]
    pub fn resolve(self, basis: S) -> Option<S> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => Some(value * basis),
            Self::Calc(_) => panic!("calc values require an explicit resolver"),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => basis.map(|basis| value * basis),
            Self::Calc(_) => panic!("calc values require an explicit resolver"),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub fn resolve_with(self, basis: Option<S>, resolver: &dyn CalcResolver<S>) -> Option<S> {
        self.resolve_with_status(basis, resolver).value
    }

    #[must_use]
    pub fn resolve_with_status(
        self,
        basis: Option<S>,
        resolver: &dyn CalcResolver<S>,
    ) -> CalcResolutionOf<S> {
        match self {
            Self::Px(value) => CalcResolutionOf::definite(value, false),
            Self::Percent(value) => basis.map_or(CalcResolutionOf::unresolved(true), |basis| {
                CalcResolutionOf::definite(value * basis, true)
            }),
            Self::Calc(id) => resolver.resolve_calc(id, basis),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => {
                CalcResolutionOf::non_numeric()
            }
        }
    }

    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    #[must_use]
    pub const fn is_min_content(self) -> bool {
        matches!(self, Self::MinContent)
    }

    #[must_use]
    pub const fn is_max_content(self) -> bool {
        matches!(self, Self::MaxContent)
    }
}

impl<S: LayoutScalar> Default for DimensionOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for DimensionOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        match value {
            LengthOf::Normal => Self::ZERO,
            LengthOf::Px(value) => Self::Px(value),
            LengthOf::Percent(value) => Self::Percent(value),
            LengthOf::Calc(id) => Self::Calc(id),
        }
    }
}

impl<S: LayoutScalar> From<LengthAutoOf<S>> for DimensionOf<S> {
    fn from(value: LengthAutoOf<S>) -> Self {
        match value {
            LengthAutoOf::Px(value) => Self::Px(value),
            LengthAutoOf::Percent(value) => Self::Percent(value),
            LengthAutoOf::Calc(id) => Self::Calc(id),
            LengthAutoOf::Auto => Self::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MinTrackSizingOf<S: LayoutScalar = DefaultScalar> {
    Length(LengthOf<S>),
    Auto,
    MinContent,
    MaxContent,
}

pub type MinTrackSizing = MinTrackSizingOf<DefaultScalar>;

impl<S: LayoutScalar> MinTrackSizingOf<S> {
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;
    pub const ZERO: Self = Self::Length(LengthOf::ZERO);

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Length(LengthOf::px(value))
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Length(LengthOf::percent(value))
    }

    #[must_use]
    pub const fn is_intrinsic(self) -> bool {
        matches!(self, Self::Auto | Self::MinContent | Self::MaxContent)
    }

    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        match self {
            Self::Length(length) => length.depends_on_basis(),
            Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &dyn CalcResolver<S>) -> bool {
        match self {
            Self::Length(length) => length.depends_on_basis_with(resolver),
            Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &dyn CalcResolver<S>) -> S {
        match self {
            Self::Length(length) => length.percent_fraction_with(resolver),
            Self::Auto | Self::MinContent | Self::MaxContent => S::ZERO,
        }
    }

    #[must_use]
    pub fn definite(self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Length(length) => length.resolve_optional(basis),
            Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for MinTrackSizingOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        Self::Length(value)
    }
}

impl<S: LayoutScalar> From<DimensionOf<S>> for MinTrackSizingOf<S> {
    fn from(value: DimensionOf<S>) -> Self {
        match value {
            DimensionOf::Px(value) => Self::px(value),
            DimensionOf::Percent(value) => Self::percent(value),
            DimensionOf::Calc(id) => Self::Length(LengthOf::calc(id)),
            DimensionOf::Fr(_) | DimensionOf::Auto => Self::Auto,
            DimensionOf::MinContent => Self::MinContent,
            DimensionOf::MaxContent => Self::MaxContent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaxTrackSizingOf<S: LayoutScalar = DefaultScalar> {
    Length(LengthOf<S>),
    Flex(S),
    Auto,
    MinContent,
    MaxContent,
    FitContent(LengthOf<S>),
}

pub type MaxTrackSizing = MaxTrackSizingOf<DefaultScalar>;

impl<S: LayoutScalar> MaxTrackSizingOf<S> {
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;
    pub const ZERO: Self = Self::Length(LengthOf::ZERO);

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Length(LengthOf::px(value))
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Length(LengthOf::percent(value))
    }

    #[must_use]
    pub const fn fr(value: S) -> Self {
        Self::Flex(value)
    }

    #[must_use]
    pub const fn fit_content(limit: LengthOf<S>) -> Self {
        Self::FitContent(limit)
    }

    #[must_use]
    pub const fn is_flexible(self) -> bool {
        matches!(self, Self::Flex(_))
    }

    #[must_use]
    pub const fn is_intrinsic(self) -> bool {
        matches!(
            self,
            Self::Auto | Self::MinContent | Self::MaxContent | Self::FitContent(_)
        )
    }

    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        match self {
            Self::Length(length) | Self::FitContent(length) => length.depends_on_basis(),
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &dyn CalcResolver<S>) -> bool {
        match self {
            Self::Length(length) | Self::FitContent(length) => {
                length.depends_on_basis_with(resolver)
            }
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &dyn CalcResolver<S>) -> S {
        match self {
            Self::Length(length) | Self::FitContent(length) => {
                length.percent_fraction_with(resolver)
            }
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => S::ZERO,
        }
    }

    #[must_use]
    pub fn definite(self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Length(length) => length.resolve_optional(basis),
            Self::Flex(_)
            | Self::Auto
            | Self::MinContent
            | Self::MaxContent
            | Self::FitContent(_) => None,
        }
    }

    #[must_use]
    pub fn fit_limit(self, basis: Option<S>) -> Option<S> {
        match self {
            Self::FitContent(limit) => limit.resolve_optional(basis),
            _ => None,
        }
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for MaxTrackSizingOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        Self::Length(value)
    }
}

impl<S: LayoutScalar> From<DimensionOf<S>> for MaxTrackSizingOf<S> {
    fn from(value: DimensionOf<S>) -> Self {
        match value {
            DimensionOf::Px(value) => Self::px(value),
            DimensionOf::Percent(value) => Self::percent(value),
            DimensionOf::Calc(id) => Self::Length(LengthOf::calc(id)),
            DimensionOf::Fr(value) => Self::fr(value),
            DimensionOf::Auto => Self::Auto,
            DimensionOf::MinContent => Self::MinContent,
            DimensionOf::MaxContent => Self::MaxContent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSizingOf<S: LayoutScalar = DefaultScalar> {
    pub min: MinTrackSizingOf<S>,
    pub max: MaxTrackSizingOf<S>,
}

pub type TrackSizing = TrackSizingOf<DefaultScalar>;

impl<S: LayoutScalar> TrackSizingOf<S> {
    pub const AUTO: Self = Self {
        min: MinTrackSizingOf::AUTO,
        max: MaxTrackSizingOf::AUTO,
    };
    pub const MIN_CONTENT: Self = Self {
        min: MinTrackSizingOf::MIN_CONTENT,
        max: MaxTrackSizingOf::MIN_CONTENT,
    };
    pub const MAX_CONTENT: Self = Self {
        min: MinTrackSizingOf::MAX_CONTENT,
        max: MaxTrackSizingOf::MAX_CONTENT,
    };
    pub const ZERO: Self = Self {
        min: MinTrackSizingOf::ZERO,
        max: MaxTrackSizingOf::ZERO,
    };

    #[must_use]
    pub const fn new(min: MinTrackSizingOf<S>, max: MaxTrackSizingOf<S>) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::new(MinTrackSizingOf::px(value), MaxTrackSizingOf::px(value))
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::new(
            MinTrackSizingOf::percent(value),
            MaxTrackSizingOf::percent(value),
        )
    }

    #[must_use]
    pub const fn fr(value: S) -> Self {
        Self::new(MinTrackSizingOf::AUTO, MaxTrackSizingOf::fr(value))
    }

    #[must_use]
    pub const fn fit_content(limit: LengthOf<S>) -> Self {
        Self::new(MinTrackSizingOf::AUTO, MaxTrackSizingOf::fit_content(limit))
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizingOf<S>, max: MaxTrackSizingOf<S>) -> Self {
        Self::new(min, max)
    }

    #[must_use]
    pub const fn depends_on_basis(self) -> bool {
        self.min.depends_on_basis() || self.max.depends_on_basis()
    }

    #[must_use]
    pub fn depends_on_basis_with(self, resolver: &dyn CalcResolver<S>) -> bool {
        self.min.depends_on_basis_with(resolver) || self.max.depends_on_basis_with(resolver)
    }

    #[must_use]
    pub fn percent_fraction_with(self, resolver: &dyn CalcResolver<S>) -> S {
        self.min
            .percent_fraction_with(resolver)
            .max(self.max.percent_fraction_with(resolver))
    }
}

impl<S: LayoutScalar> Default for TrackSizingOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

impl<S: LayoutScalar> From<DimensionOf<S>> for TrackSizingOf<S> {
    fn from(value: DimensionOf<S>) -> Self {
        Self::new(value.into(), value.into())
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for TrackSizingOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        Self::new(value.into(), value.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackRepeat {
    Count(TrackRepeatCount),
    AutoFill,
    AutoFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackRepeatCount(NonZeroUsize);

impl TrackRepeatCount {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackComponentListOf<S: LayoutScalar = DefaultScalar>(Vec<TrackComponentOf<S>>);

pub type TrackComponentList = TrackComponentListOf<DefaultScalar>;

impl<S: LayoutScalar> TrackComponentListOf<S> {
    #[must_use]
    pub fn as_slice(&self) -> &[TrackComponentOf<S>] {
        &self.0
    }

    fn into_vec(self) -> Vec<TrackComponentOf<S>> {
        self.0
    }
}

impl<S: LayoutScalar> TryFrom<Vec<TrackComponentOf<S>>> for TrackComponentListOf<S> {
    type Error = TrackRepetitionError;

    fn try_from(value: Vec<TrackComponentOf<S>>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(TrackRepetitionError::EmptyComponents)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackRepetitionError {
    ZeroCount,
    EmptyComponents,
}

impl core::fmt::Display for TrackRepetitionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroCount => f.write_str("track repeat count must be greater than zero"),
            Self::EmptyComponents => f.write_str("track repeat components must not be empty"),
        }
    }
}

impl std::error::Error for TrackRepetitionError {}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackRepetitionOf<S: LayoutScalar = DefaultScalar> {
    repeat: TrackRepeat,
    components: TrackComponentListOf<S>,
}

pub type TrackRepetition = TrackRepetitionOf<DefaultScalar>;

impl<S: LayoutScalar> TrackRepetitionOf<S> {
    pub fn count(
        count: usize,
        tracks: Vec<TrackSizingOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        Self::count_components(count, track_sizing_components_from_tracks(tracks))
    }

    pub fn auto_fill(tracks: Vec<TrackSizingOf<S>>) -> Result<Self, TrackRepetitionError> {
        Self::auto_fill_components(track_sizing_components_from_tracks(tracks))
    }

    pub fn auto_fit(tracks: Vec<TrackSizingOf<S>>) -> Result<Self, TrackRepetitionError> {
        Self::auto_fit_components(track_sizing_components_from_tracks(tracks))
    }

    pub fn count_components(
        count: usize,
        components: Vec<TrackComponentOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        let count = TrackRepeatCount::new(count).ok_or(TrackRepetitionError::ZeroCount)?;
        let components = TrackComponentListOf::try_from(components)?;
        Ok(Self::from_validated(TrackRepeat::Count(count), components))
    }

    pub fn auto_fill_components(
        components: Vec<TrackComponentOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        let components = TrackComponentListOf::try_from(components)?;
        Ok(Self::from_validated(TrackRepeat::AutoFill, components))
    }

    pub fn auto_fit_components(
        components: Vec<TrackComponentOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        let components = TrackComponentListOf::try_from(components)?;
        Ok(Self::from_validated(TrackRepeat::AutoFit, components))
    }

    #[must_use]
    pub const fn from_validated(repeat: TrackRepeat, components: TrackComponentListOf<S>) -> Self {
        Self { components, repeat }
    }

    #[must_use]
    pub fn sizing_tracks(&self) -> Vec<TrackSizingOf<S>> {
        track_sizing_components_of(self.components.as_slice())
    }

    #[must_use]
    pub const fn repeat(&self) -> TrackRepeat {
        self.repeat
    }

    #[must_use]
    pub fn components(&self) -> &[TrackComponentOf<S>] {
        self.components.as_slice()
    }

    #[must_use]
    pub fn into_components(self) -> Vec<TrackComponentOf<S>> {
        self.components.into_vec()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TrackComponentOf<S: LayoutScalar = DefaultScalar> {
    LineNames(Vec<String>),
    Track(TrackSizingOf<S>),
    Repeat(TrackRepetitionOf<S>),
    Subgrid(SubgridTrack),
}

pub type TrackComponent = TrackComponentOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrack {
    pub name_components: Vec<SubgridLineNameComponent>,
}

impl SubgridTrack {
    #[must_use]
    pub fn new(line_names: Vec<Vec<String>>) -> Self {
        Self {
            name_components: line_names
                .into_iter()
                .map(SubgridLineNameComponent::LineNames)
                .collect(),
        }
    }

    #[must_use]
    pub fn line_names(&self) -> Vec<Vec<String>> {
        let mut line_names = Vec::new();
        for component in &self.name_components {
            match component {
                SubgridLineNameComponent::LineNames(names) => line_names.push(names.clone()),
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::Count(count),
                    line_name_sets,
                } => {
                    for _ in 0..*count {
                        line_names.extend(line_name_sets.iter().cloned());
                    }
                }
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::AutoFill,
                    line_name_sets,
                } => line_names.extend(line_name_sets.iter().cloned()),
            }
        }
        line_names
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubgridLineNameComponent {
    LineNames(Vec<String>),
    Repeat {
        count: SubgridLineNameRepeatCount,
        line_name_sets: Vec<Vec<String>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridLineNameRepeatCount {
    Count(usize),
    AutoFill,
}

impl<S: LayoutScalar> TrackComponentOf<S> {
    pub const AUTO: Self = Self::Track(TrackSizingOf::AUTO);
    pub const MIN_CONTENT: Self = Self::Track(TrackSizingOf::MIN_CONTENT);
    pub const MAX_CONTENT: Self = Self::Track(TrackSizingOf::MAX_CONTENT);
    pub const ZERO: Self = Self::Track(TrackSizingOf::ZERO);

    #[must_use]
    pub const fn px(value: S) -> Self {
        Self::Track(TrackSizingOf::px(value))
    }

    #[must_use]
    pub const fn percent(value: S) -> Self {
        Self::Track(TrackSizingOf::percent(value))
    }

    #[must_use]
    pub const fn fr(value: S) -> Self {
        Self::Track(TrackSizingOf::fr(value))
    }

    #[must_use]
    pub const fn fit_content(limit: LengthOf<S>) -> Self {
        Self::Track(TrackSizingOf::fit_content(limit))
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizingOf<S>, max: MaxTrackSizingOf<S>) -> Self {
        Self::Track(TrackSizingOf::minmax(min, max))
    }

    #[must_use]
    pub fn line_names(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::LineNames(names.into_iter().map(Into::into).collect())
    }
}

impl<S: LayoutScalar> From<TrackSizingOf<S>> for TrackComponentOf<S> {
    fn from(value: TrackSizingOf<S>) -> Self {
        Self::Track(value)
    }
}

impl<S: LayoutScalar> From<DimensionOf<S>> for TrackComponentOf<S> {
    fn from(value: DimensionOf<S>) -> Self {
        Self::Track(value.into())
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for TrackComponentOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        Self::Track(value.into())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GridTemplateAreas {
    pub rows: Vec<GridTemplateAreaRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridTemplateAreaRow {
    pub cells: Vec<Option<String>>,
}

#[must_use]
pub fn track_sizing_components(components: &[TrackComponent]) -> Vec<TrackSizing> {
    track_sizing_components_of(components)
}

#[must_use]
pub fn track_sizing_components_of<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Vec<TrackSizingOf<S>> {
    let mut tracks = Vec::new();
    for component in components {
        match component {
            TrackComponentOf::Track(track) => tracks.push(*track),
            TrackComponentOf::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                match repetition.repeat() {
                    TrackRepeat::Count(count) => {
                        for _ in 0..count.get() {
                            tracks.extend(repeated_tracks.iter().copied());
                        }
                    }
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                        tracks.extend(repeated_tracks);
                    }
                }
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    tracks
}

fn track_sizing_components_from_tracks<S: LayoutScalar>(
    tracks: Vec<TrackSizingOf<S>>,
) -> Vec<TrackComponentOf<S>> {
    tracks.into_iter().map(TrackComponentOf::Track).collect()
}
