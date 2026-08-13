use crate::DefaultScalar;
use crate::geometry::PhysicalAxis;
use crate::node_input::{FloatExclusionIntervalErrorOf, InlineTextInputErrorOf};
use crate::output::RunMode;
use crate::scalar::LayoutScalar;
use crate::sizing::resolve::SizingResolutionError;
use crate::value::{LengthResolutionStatus, NonNegativeFiniteScalarErrorOf};

pub type LayoutResultOf<Node, T, S, M = core::convert::Infallible> =
    Result<T, LayoutErrorOf<Node, S, M>>;
pub type LayoutResult<Node, T, M> = LayoutResultOf<Node, T, DefaultScalar, M>;

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutErrorOf<Node, S: LayoutScalar = DefaultScalar, M = core::convert::Infallible> {
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    kind: LayoutErrorKindOf<S, M>,
}

pub type LayoutError<Node, M = core::convert::Infallible> = LayoutErrorOf<Node, DefaultScalar, M>;

impl<Node, S, M> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    pub const fn new(
        site: LayoutErrorSiteOf<Node>,
        operation: LayoutOperation,
        kind: LayoutErrorKindOf<S, M>,
    ) -> Self {
        Self {
            site,
            operation,
            kind,
        }
    }

    #[must_use]
    pub const fn site(&self) -> LayoutErrorSiteOf<Node>
    where
        Node: Copy,
    {
        self.site
    }

    #[must_use]
    pub const fn operation(&self) -> LayoutOperation {
        self.operation
    }

    #[must_use]
    pub const fn kind(&self) -> &LayoutErrorKindOf<S, M> {
        &self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutErrorSiteOf<Node> {
    Node(Node),
    ContainerSubject { container: Node, subject: Node },
    Standalone,
}

pub type LayoutErrorSite<Node> = LayoutErrorSiteOf<Node>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutOperation {
    RootLayout,
    ChildLayout,
    HiddenLayout,
    LeafMeasurement,
    ValueResolution,
    CacheAccess,
    CacheInvalidation,
    FloatExclusionQuery,
    RoundingFinalization,
    GridLanePlacement,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LayoutErrorKindOf<S: LayoutScalar = DefaultScalar, M = core::convert::Infallible> {
    InvalidInput(LayoutInvalidInputOf<S>),
    MissingContext(LayoutMissingContext),
    UnsupportedCapability(LayoutUnsupportedCapability),
    Measurement(M),
    InternalInvariant(LayoutInternalInvariant),
}

pub type LayoutErrorKind<M = core::convert::Infallible> = LayoutErrorKindOf<DefaultScalar, M>;

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum LayoutInvalidInputOf<S: LayoutScalar = DefaultScalar> {
    RootAvailability {
        axis: PhysicalAxis,
        error: NonNegativeFiniteScalarErrorOf<S>,
    },
    MeasurementOutput(InvalidMeasurementOutputOf<S>),
    InvalidNumeric {
        value: S,
    },
    InlineText(InlineTextInputErrorOf<S>),
    AtomicInlineParticipation {
        reason: AtomicInlineParticipationRoleError,
    },
    NonBoxNodeRole {
        reason: NonBoxNodeRoleError,
    },
    FloatExclusionRole {
        reason: FloatExclusionRoleError,
    },
    FloatExclusionProviderOutput {
        error: FloatExclusionIntervalErrorOf<S>,
    },
    InvalidationNodeNotReachable,
    TreeTopologyCycle,
}

pub type LayoutInvalidInput = LayoutInvalidInputOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicInlineParticipationRoleError {
    MissingForAtomicInline,
    UnexpectedForNonAtomic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonBoxNodeRoleError {
    NonCanonicalNodeInput,
    HasChildren,
    HasLeafMeasurement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatExclusionRoleError {
    Hidden,
    NonFloating,
    Absolute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutMissingContext {
    RequiredBasis,
    FloatExclusionProvider,
}

impl core::hash::Hash for PhysicalAxis {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SizingProperty {
    Preferred,
    Minimum,
    Maximum,
    FlexBasis,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SizingAlgorithm {
    Leaf,
    Block,
    Flex,
    Grid,
    GridLanes,
    Positioned,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CalcSizeBehaviorBasis {
    Auto,
    None,
    Content,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SizingBehavior {
    MinContent,
    MaxContent,
    FitContentFunction,
    Stretch,
    FitContent,
    Contain,
    CalcSize(CalcSizeBehaviorBasis),
}

/// A typed description of sizing behavior owned by a later algorithm initiative.
///
/// The payload is output-only; callers inspect descriptors returned through
/// [`LayoutUnsupportedCapability`] rather than constructing them.
///
/// ```compile_fail
/// use surgeist_layout::{
///     PhysicalAxis, SizingAlgorithm, SizingBehavior, SizingProperty,
///     UnsupportedSizingBehavior,
/// };
/// let _ = UnsupportedSizingBehavior {
///     property: SizingProperty::Preferred,
///     behavior: SizingBehavior::Stretch,
///     algorithm: SizingAlgorithm::Leaf,
///     axis: PhysicalAxis::Horizontal,
/// };
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{
///     PhysicalAxis, SizingAlgorithm, SizingBehavior, SizingProperty,
///     UnsupportedSizingBehavior,
/// };
/// let _ = UnsupportedSizingBehavior::new(
///     SizingProperty::Preferred,
///     SizingBehavior::Stretch,
///     SizingAlgorithm::Leaf,
///     PhysicalAxis::Horizontal,
/// );
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnsupportedSizingBehavior {
    property: SizingProperty,
    behavior: SizingBehavior,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
}

impl UnsupportedSizingBehavior {
    pub(crate) const fn new(
        property: SizingProperty,
        behavior: SizingBehavior,
        algorithm: SizingAlgorithm,
        axis: PhysicalAxis,
    ) -> Self {
        Self {
            property,
            behavior,
            algorithm,
            axis,
        }
    }

    #[must_use]
    pub const fn property(self) -> SizingProperty {
        self.property
    }

    #[must_use]
    pub const fn behavior(self) -> SizingBehavior {
        self.behavior
    }

    #[must_use]
    pub const fn algorithm(self) -> SizingAlgorithm {
        self.algorithm
    }

    #[must_use]
    pub const fn axis(self) -> PhysicalAxis {
        self.axis
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LayoutUnsupportedCapability {
    LaterFriBehavior,
    SizingBehavior(UnsupportedSizingBehavior),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutInternalInvariant {
    InvalidRootScrollGeometry,
    InvalidBlockScrollGeometry,
    InvalidRoundedScrollGeometry,
    InvalidRoundedInlineFragmentGeometry,
    MissingLeafMeasurementProvider,
    MissingStagedUnroundedOutput,
    MissingCachedInlineFragmentState,
    SubgridTrackInheritance,
    SubgridBaselineInheritance,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LeafMeasureErrorOf<S: LayoutScalar, M> {
    Provider(M),
    InvalidOutput(InvalidMeasurementOutputOf<S>),
}

pub type LeafMeasureError<M> = LeafMeasureErrorOf<DefaultScalar, M>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidMeasurementOutputOf<S: LayoutScalar = DefaultScalar> {
    axis: PhysicalAxis,
    error: NonNegativeFiniteScalarErrorOf<S>,
}

pub(crate) fn invalid_measurement_output<S, M>(
    axis: PhysicalAxis,
    error: NonNegativeFiniteScalarErrorOf<S>,
) -> LeafMeasureErrorOf<S, M>
where
    S: LayoutScalar,
{
    LeafMeasureErrorOf::InvalidOutput(InvalidMeasurementOutputOf { axis, error })
}

pub type InvalidMeasurementOutput = InvalidMeasurementOutputOf<DefaultScalar>;

impl<S: LayoutScalar> InvalidMeasurementOutputOf<S> {
    /// Returns the physical axis of the rejected measurement output.
    #[must_use]
    pub const fn axis(self) -> PhysicalAxis {
        self.axis
    }

    #[must_use]
    pub const fn error(self) -> NonNegativeFiniteScalarErrorOf<S> {
        self.error
    }
}

pub(crate) fn layout_own_geometry_error<Node, S, M, E>(
    node: Node,
    run_mode: RunMode,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    let (operation, invariant) = if run_mode == RunMode::PerformRootLayout {
        (
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        )
    } else {
        (
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        )
    };
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        operation,
        LayoutErrorKindOf::InternalInvariant(invariant),
    )
}

pub(crate) fn layout_child_geometry_error<Node, S, M, E>(
    container: Node,
    subject: Node,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    LayoutErrorOf::new(
        LayoutErrorSiteOf::ContainerSubject { container, subject },
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidBlockScrollGeometry),
    )
}

pub(crate) fn value_resolution_error<Node, S, M>(
    node: Node,
    status: LengthResolutionStatus<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    value_resolution_error_at_site(LayoutErrorSiteOf::Node(node), status)
}

pub(crate) fn value_resolution_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    status: LengthResolutionStatus<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = match status {
        LengthResolutionStatus::MissingBasis => {
            LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
        }
        LengthResolutionStatus::InvalidNumeric { value } => {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
        }
        LengthResolutionStatus::NonNumeric => {
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior)
        }
        LengthResolutionStatus::Resolved => {
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry)
        }
    };

    LayoutErrorOf::new(site, LayoutOperation::ValueResolution, kind)
}

pub(crate) fn sizing_resolution_error<Node, S, M>(
    node: Node,
    error: SizingResolutionError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    sizing_resolution_error_at_site(LayoutErrorSiteOf::Node(node), error)
}

pub(crate) fn sizing_resolution_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    error: SizingResolutionError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    match error {
        SizingResolutionError::Status(status) => value_resolution_error_at_site(site, status),
        SizingResolutionError::Unsupported(unsupported) => LayoutErrorOf::new(
            site,
            LayoutOperation::ValueResolution,
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
                unsupported,
            )),
        ),
    }
}
