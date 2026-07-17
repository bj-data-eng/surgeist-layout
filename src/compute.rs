use super::{
    AspectRatioOf, AvailableOf, BoxSizing, CacheAccess, CollapsibleMarginOf, Compute,
    ComputeInputOf, ComputeOutputOf, DefaultScalar, Edges, LayoutCacheClearEntry,
    LayoutCacheStoreEntryOf, LayoutInputOf, LayoutOutputEntryOf, LayoutRootContextOf,
    LayoutRootRequestOf, LayoutScalar, LengthResolutionOf, LengthResolutionStatus, NodeInputOf,
    NodeOutputOf, NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf,
    PhysicalBlockMarginCollapseOf, Point, Position, Round, RunMode, Size, SizingMode, Traverse,
};
use crate::geometry::{FlowAxes, PhysicalAxis, PhysicalSide};
use crate::scroll::{
    CanonicalScrollBoxSourceOf, CanonicalScrollGeometryErrorOf, CanonicalScrollGeometrySourceOf,
    ClipMarginSourceOf, MeasuredLeafContentBoxInsetSourceOf, MeasuredLeafScrollGeometrySourceOf,
    OptimalRegionInsetOf, OptimalRegionInsetsOf, ScrollContributionAccumulatorOf, ScrollOriginAxes,
    ScrollOriginProgression, SettledAutoScrollbarState, canonical_measured_leaf_scroll_geometry,
    canonical_scroll_box_from_source, canonical_scroll_geometry_from_source,
    measured_leaf_content_box_inset, rebuild_canonical_scroll_geometry_for_border_box,
    rebuild_rounded_canonical_scroll_geometry,
};
use crate::sizing::{
    DispatchedSizingRequest, SizingDispatch, dispatch_flex_basis, dispatch_maximum_size,
    dispatch_minimum_size, dispatch_preferred_size,
};
use crate::{CompletedLayoutBatchOf, LayoutTree};
use crate::{FlexBasisOf, MaxSizeOf, MinSizeOf, PercentageBasisOf, PreferredSizeOf};

pub type LayoutResultOf<Node, T, S, M = core::convert::Infallible> =
    Result<T, LayoutErrorOf<Node, S, M>>;
pub type LayoutResult<Node, T, M> = LayoutResultOf<Node, T, DefaultScalar, M>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SizingResolutionError<S: LayoutScalar> {
    Status(LengthResolutionStatus<S>),
    Unsupported(UnsupportedSizingBehavior),
}

impl<S: LayoutScalar> From<LengthResolutionStatus<S>> for SizingResolutionError<S> {
    fn from(status: LengthResolutionStatus<S>) -> Self {
        Self::Status(status)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ResolvedPreferredSize<S: LayoutScalar> {
    Auto,
    Definite(S),
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ResolvedFlexBasis<S: LayoutScalar> {
    Auto,
    Content,
    Definite(S),
}

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
}

pub type LayoutInvalidInput = LayoutInvalidInputOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutMissingContext {
    RequiredBasis,
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
    MissingLeafMeasurementProvider,
    MissingStagedUnroundedOutput,
    SubgridTrackInheritance,
    SubgridBaselineInheritance,
}

#[expect(
    clippy::type_complexity,
    reason = "the public root boundary preserves the tree node, scalar, and provider error types"
)]
pub fn compute_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> LayoutResultOf<
    Tree::Node,
    CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>,
    Tree::Scalar,
    Tree::MeasureError,
>
where
    Tree: LayoutTree,
{
    let mut session = ComputeSession::new(tree);
    match request.context() {
        LayoutRootContextOf::Viewport => {
            compute_root(&mut session, root, request.available())?;
        }
        LayoutRootContextOf::FlexItemUnderViewport(context) => {
            compute_flex_item_root(&mut session, root, request.available(), context)?;
        }
    }

    match request.rounding_mode() {
        super::LayoutRoundingMode::NearestCssPixel => round_layout(&mut session, root)?,
    }

    Ok(session.complete())
}

struct ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    tree: &'a Tree,
    unrounded_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    final_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    cache_store_entries: Vec<LayoutCacheStoreEntryOf<Tree::Node, Tree::Scalar>>,
    cache_clear_entries: Vec<LayoutCacheClearEntry<Tree::Node>>,
}

impl<'a, Tree> ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    fn new(tree: &'a Tree) -> Self {
        Self {
            tree,
            unrounded_entries: Vec::new(),
            final_entries: Vec::new(),
            cache_store_entries: Vec::new(),
            cache_clear_entries: Vec::new(),
        }
    }

    fn complete(self) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        CompletedLayoutBatchOf::from_entries(
            self.unrounded_entries,
            self.final_entries,
            self.cache_store_entries,
            self.cache_clear_entries,
        )
    }

    fn staged_source_index(&self, node: Tree::Node) -> crate::SourceIndex {
        self.unrounded_entries
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .map(|entry| entry.output().source_index)
            .unwrap_or(crate::SourceIndex::ZERO)
    }

    fn compute_child_uncached(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        let style = self.node_input(node).clone();
        if self.tree.has_leaf_measurement(node) {
            return self.compute_tree_leaf(node, input, &style);
        }

        match style.display.inner_display() {
            super::Display::Block => crate::block::compute_block(self, node, input),
            super::Display::Flex => crate::flex::compute_flex(self, node, input),
            super::Display::Grid | super::Display::GridLanes => {
                crate::grid::compute_grid(self, node, input)
            }
            super::Display::None => compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
            ),
            super::Display::InlineBlock
            | super::Display::InlineGrid
            | super::Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }

    fn compute_tree_leaf(
        &self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        style: &NodeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        let resolved = resolve_leaf_values_for_input(input, style)
            .map_err(|error| sizing_resolution_error(node, error))?;

        let site = LayoutErrorSiteOf::Node(node);
        compute_leaf_with_resolved_values(site, input, style, resolved, |measure_input| match self
            .tree
            .measure_leaf(node, measure_input)
        {
            Some(Ok(output)) => Ok(output),
            Some(Err(error)) => Err(LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )),
            None => Err(LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingLeafMeasurementProvider,
                ),
            )),
        })
    }
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

fn sizing_resolution_error_at_site<Node, S, M>(
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

fn percentage_basis<S: LayoutScalar>(basis: Option<S>) -> PercentageBasisOf<S> {
    basis.map_or(PercentageBasisOf::MISSING, |value| {
        PercentageBasisOf::definite(value)
            .expect("validated compute inputs carry non-negative finite parent sizes")
    })
}

fn resolve_dispatched_numeric<S: LayoutScalar>(
    request: DispatchedSizingRequest<'_, S>,
    basis: PercentageBasisOf<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let resolution = match request {
        DispatchedSizingRequest::Zero => return Ok(Some(S::ZERO)),
        DispatchedSizingRequest::Calculation(calculation) => calculation.resolve_against(basis),
        DispatchedSizingRequest::ResolvedCalcSize(resolution) => resolution,
        DispatchedSizingRequest::Auto | DispatchedSizingRequest::None => return Ok(None),
        DispatchedSizingRequest::Content
        | DispatchedSizingRequest::MinContent
        | DispatchedSizingRequest::MaxContent => {
            unreachable!("the property consumer must handle contextual supported states")
        }
    };

    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::MissingBasis if missing_basis_is_indefinite => Ok(None),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(SizingResolutionError::Status(resolution.status()))
        }
        LengthResolutionStatus::NonNumeric => {
            unreachable!("typed sizing dispatch never returns a nonnumeric numeric request")
        }
    }
}

pub(crate) fn resolve_preferred_sizing<S: LayoutScalar>(
    value: &PreferredSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<ResolvedPreferredSize<S>, SizingResolutionError<S>> {
    let basis = percentage_basis(basis);
    match dispatch_preferred_size(value, algorithm, axis, basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(DispatchedSizingRequest::Auto) => Ok(ResolvedPreferredSize::Auto),
        SizingDispatch::Supported(DispatchedSizingRequest::MinContent) => {
            Ok(ResolvedPreferredSize::MinContent)
        }
        SizingDispatch::Supported(DispatchedSizingRequest::MaxContent) => {
            Ok(ResolvedPreferredSize::MaxContent)
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, missing_basis_is_indefinite).map(|value| {
                value.map_or(ResolvedPreferredSize::Auto, ResolvedPreferredSize::Definite)
            })
        }
    }
}

pub(crate) fn resolve_preferred_optional<S: LayoutScalar>(
    value: &PreferredSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    match resolve_preferred_sizing(value, algorithm, axis, basis, missing_basis_is_indefinite)? {
        ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::MinContent
        | ResolvedPreferredSize::MaxContent => Ok(None),
        ResolvedPreferredSize::Definite(value) => Ok(Some(value)),
    }
}

pub(crate) fn resolve_minimum_optional<S: LayoutScalar>(
    value: &MinSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let basis = percentage_basis(basis);
    match dispatch_minimum_size(value, algorithm, axis, basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, missing_basis_is_indefinite)
        }
    }
}

pub(crate) fn resolve_maximum_optional<S: LayoutScalar>(
    value: &MaxSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let basis = percentage_basis(basis);
    match dispatch_maximum_size(value, algorithm, axis, basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, missing_basis_is_indefinite)
        }
    }
}

pub(crate) fn resolve_flex_basis<S: LayoutScalar>(
    value: &FlexBasisOf<S>,
    axis: PhysicalAxis,
    basis: Option<S>,
) -> Result<ResolvedFlexBasis<S>, SizingResolutionError<S>> {
    let percentage_basis = percentage_basis(basis);
    match dispatch_flex_basis(value, SizingAlgorithm::Flex, axis, percentage_basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(DispatchedSizingRequest::Auto) => Ok(ResolvedFlexBasis::Auto),
        SizingDispatch::Supported(DispatchedSizingRequest::Content) => {
            Ok(ResolvedFlexBasis::Content)
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, percentage_basis, true)
                .map(|value| value.map_or(ResolvedFlexBasis::Content, ResolvedFlexBasis::Definite))
        }
    }
}

fn root_scroll_error<Node, S, M, E>(node: Node, error: E) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    let kind =
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry);

    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        kind,
    )
}

impl<Tree> Traverse for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'b>
        = Tree::Children<'b>
    where
        Self: 'b;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.tree.children(node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<Tree> Compute<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        self.tree.layout_input(node)
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>) {
        if let Some(index) = self
            .unrounded_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.unrounded_entries.remove(index);
        }
        self.unrounded_entries
            .push(LayoutOutputEntryOf::new(node, layout));
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        let style = self.node_input(node).clone();
        if input.run_mode() == RunMode::PerformHiddenLayout || style.display == super::Display::None
        {
            return compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
            );
        }

        if input.run_mode().is_perform_layout() && self.child_count(node) != 0 {
            return self.compute_child_uncached(
                node,
                input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL),
            );
        }

        crate::traits::compute_cached(self, node, input, |session, node, input| {
            session.compute_child_uncached(
                node,
                input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL),
            )
        })
    }
}

impl<Tree> Round<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<Self::Node, NodeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        self.unrounded_entries
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .map(LayoutOutputEntryOf::output)
            .ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::RoundingFinalization,
                    LayoutErrorKindOf::InternalInvariant(
                        LayoutInternalInvariant::MissingStagedUnroundedOutput,
                    ),
                )
            })
    }

    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>) {
        if let Some(index) = self
            .final_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.final_entries.remove(index);
        }
        self.final_entries
            .push(LayoutOutputEntryOf::new(node, layout));
    }
}

impl<Tree> CacheAccess<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;

    fn cache_context(&self) -> super::CacheKeyContext {
        self.tree.cache_context()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: super::CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        self.tree.cache_get(node, input, context)
    }

    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: super::CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    ) {
        if let Some(index) = self.cache_store_entries.iter().position(|entry| {
            entry.node() == node && entry.input() == input && entry.context() == context
        }) {
            self.cache_store_entries.remove(index);
        }
        self.cache_store_entries
            .push(LayoutCacheStoreEntryOf::new(node, *input, context, output));
    }

    fn cache_clear(&mut self, node: Self::Node) {
        if let Some(index) = self
            .cache_clear_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.cache_clear_entries.remove(index);
        }
        self.cache_clear_entries
            .push(LayoutCacheClearEntry::new(node));
    }
}

pub(crate) fn compute_hidden<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    source_index: crate::SourceIndex,
    containing_layout_context: crate::ContainingLayoutContext,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    ComputeOutputOf<<Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>
where
    Tree: Compute<M>
        + CacheAccess<M, Node = <Tree as Traverse>::Node, Scalar = <Tree as Traverse>::Scalar>,
{
    tree.cache_clear(node);
    tree.set_unrounded(node, NodeOutputOf::with_source_index(source_index));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        match tree.layout_input(child) {
            LayoutInputOf::Box(_) => {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::with_source_index(crate::SourceIndex::new(index)),
                );
                let descendant_context = crate::ContainingLayoutContext::new(
                    containing_layout_context.flow_axes(),
                    crate::ParentFormattingContext::NoParent,
                );
                tree.compute_child(child, ComputeInputOf::hidden(descendant_context))?;
            }
            LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => {
                tree.cache_clear(child);
                tree.set_unrounded(
                    child,
                    NodeOutputOf::with_source_index(crate::SourceIndex::new(index)),
                );
            }
        }
    }

    Ok(ComputeOutputOf::HIDDEN)
}

pub(crate) fn compute_root<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(root).clone();
    let containing_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let containing_layout_context = super::ContainingLayoutContext::new(
        containing_flow_axes,
        super::ParentFormattingContext::NoParent,
    );
    let parent = available.map(AvailableOf::into_option);
    let known =
        root_known_inline::<Tree, M>(tree, root, &style, containing_flow_axes, available, parent)?;
    let output = tree.compute_child(
        root,
        ComputeInputOf::root_layout(known, parent, containing_layout_context, available),
    )?;
    let root_edges = resolve_root_edges(tree, root, &style, containing_flow_axes, parent)?;
    let scroll_geometry = root_scroll_geometry(root, &style, output, &root_edges)?;
    let location = root_start_location(containing_flow_axes, output.size, available);

    tree.set_unrounded(
        root,
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location,
            size: output.size,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    );
    Ok(())
}

fn compute_flex_item_root<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
    context: super::FlexItemRootContextOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<M>,
{
    let style = tree.node_input(root).clone();
    let containing_flow_axes = context.parent_flow_axes();
    let containing_layout_context = super::ContainingLayoutContext::new(
        containing_flow_axes,
        super::ParentFormattingContext::Flex,
    );
    let parent = context.viewport_available().map(AvailableOf::into_option);
    let known =
        root_known_inline::<Tree, M>(tree, root, &style, containing_flow_axes, available, parent)?;
    let output = tree.compute_child(
        root,
        ComputeInputOf::flex_item_root(known, parent, containing_layout_context, available),
    )?;
    let root_edges = resolve_root_edges(tree, root, &style, containing_flow_axes, parent)?;
    let scroll_geometry = root_scroll_geometry(root, &style, output, &root_edges)?;
    tree.set_unrounded(
        root,
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location: Point::ZERO,
            size: output.size,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    );
    Ok(())
}

struct RootEdges<S: LayoutScalar> {
    padding: Edges<S>,
    border: Edges<S>,
    margin: Edges<S>,
}

fn root_scroll_geometry<Node, S, M>(
    node: Node,
    style: &NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    edges: &RootEdges<S>,
) -> LayoutResultOf<Node, Option<crate::ScrollGeometryOf<S>>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    if style.display == super::Display::None {
        return Ok(None);
    }

    if let Some(geometry) = output.scroll_geometry {
        if geometry.border_box().origin() == Point::ZERO
            && geometry.border_box().size() == output.size
        {
            return Ok(Some(geometry));
        }
        return rebuild_canonical_scroll_geometry_for_border_box(
            geometry,
            output.size,
            edges.border,
            edges.padding,
        )
        .map(Some)
        .map_err(|error| root_scroll_error(node, error));
    }

    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let settled_auto_scrollbars = SettledAutoScrollbarState::INITIAL;
    let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
        flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: output.size,
        border: edges.border,
        padding: edges.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars,
    })
    .map_err(|error| root_scroll_error(node, error))?;
    let content_box = scroll_box.content_box();
    let direct_content = crate::ScrollRectOf::try_new(
        content_box.origin(),
        Size::new(
            content_box.size().width.max(output.content_size.width),
            content_box.size().height.max(output.content_size.height),
        ),
    )
    .map_err(|error| root_scroll_error(node, error))?;
    // Root fallback overflow is content-anchored; the scrollport remains an
    // independently derived canonical box rather than an extra contribution.
    let mut contributions = ScrollContributionAccumulatorOf::new(direct_content);
    contributions.include_direct_line(direct_content);

    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: output.size,
        border: edges.border,
        padding: edges.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: leaf_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
        scroll_snap_type: style.scroll_snap_type,
        target_border_box: scroll_box.border_box(),
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
    .map(Some)
    .map_err(|error| root_scroll_error(node, error))
}

type RootKnownInlineResult<Node, S, M> = LayoutResultOf<Node, Size<Option<S>>, S, M>;

fn resolve_root_edges<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    parent: Size<Option<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, RootEdges<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.margin, parent, |length, basis| {
            resolve_auto_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;

    Ok(RootEdges {
        padding,
        border,
        margin,
    })
}

fn root_known_inline<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    fill_available: Size<AvailableOf<Tree::Scalar>>,
    percentage_parent: Size<Option<Tree::Scalar>>,
) -> RootKnownInlineResult<<Tree as Traverse>::Node, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    let inline_axis = containing_flow_axes.inline_axis();
    if style.display.is_inline_level()
        || style.item_is_replaced
        || !root_physical_axis_value(style.size.clone(), inline_axis).is_auto()
        || !root_physical_axis_value(style.min_size.clone(), inline_axis).is_auto()
    {
        return Ok(Size::NONE);
    }

    let Some(available_inline) =
        root_physical_axis_value(fill_available, inline_axis).into_option()
    else {
        return Ok(Size::NONE);
    };
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = Size::new(
        resolve_maximum_optional(
            &style.max_size.width,
            SizingAlgorithm::Block,
            PhysicalAxis::Horizontal,
            percentage_parent.width,
            false,
        ),
        resolve_maximum_optional(
            &style.max_size.height,
            SizingAlgorithm::Block,
            PhysicalAxis::Vertical,
            percentage_parent.height,
            false,
        ),
    );
    if matches!(max_size.width, Err(SizingResolutionError::Unsupported(_)))
        || matches!(max_size.height, Err(SizingResolutionError::Unsupported(_)))
    {
        // The root optimization cannot know whether a root with block display is
        // measured as a leaf. Defer contextual rejection to the actual consumer.
        return Ok(Size::NONE);
    }
    let max_size = Size::new(
        max_size
            .width
            .map_err(|error| sizing_resolution_error(node, error))?,
        max_size
            .height
            .map_err(|error| sizing_resolution_error(node, error))?,
    )
    .add_optional(box_sizing_adjustment);

    Ok(root_known_on_axis(
        inline_axis,
        available_inline.clamp_optional(None, root_physical_axis_value(max_size, inline_axis)),
    ))
}

fn root_physical_axis_value<T>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn root_known_on_axis<S: LayoutScalar>(axis: PhysicalAxis, value: S) -> Size<Option<S>> {
    match axis {
        PhysicalAxis::Horizontal => Size::new(Some(value), None),
        PhysicalAxis::Vertical => Size::new(None, Some(value)),
    }
}

fn root_start_location<S: LayoutScalar>(
    containing_flow_axes: FlowAxes,
    root_size: Size<S>,
    available: Size<AvailableOf<S>>,
) -> Point<S> {
    Point::new(
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Horizontal,
        ),
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Vertical,
        ),
    )
}

fn root_start_coordinate<S: LayoutScalar>(
    inline_start: PhysicalSide,
    block_start: PhysicalSide,
    root_size: Size<S>,
    available: Size<AvailableOf<S>>,
    axis: PhysicalAxis,
) -> S {
    let start_side = if inline_start.axis() == axis {
        inline_start
    } else {
        block_start
    };
    match start_side {
        PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
        PhysicalSide::Right | PhysicalSide::Bottom => root_physical_axis_value(available, axis)
            .into_option()
            .map_or(S::ZERO, |extent| {
                extent - root_physical_axis_value(root_size, axis)
            }),
    }
}

pub(crate) fn round_layout<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    round_layout_inner(tree, root, Tree::Scalar::ZERO, Tree::Scalar::ZERO)
}

fn round_layout_inner<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    cumulative_x: Tree::Scalar,
    cumulative_y: Tree::Scalar,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    let unrounded = tree.unrounded(node)?;
    let mut layout = unrounded;
    let cumulative_x = cumulative_x + unrounded.location.x;
    let cumulative_y = cumulative_y + unrounded.location.y;

    layout.location.x = round(unrounded.location.x);
    layout.location.y = round(unrounded.location.y);
    layout.size.width = round(cumulative_x + unrounded.size.width) - round(cumulative_x);
    layout.size.height = round(cumulative_y + unrounded.size.height) - round(cumulative_y);
    layout.content_size.width =
        round(cumulative_x + unrounded.content_size.width) - round(cumulative_x);
    layout.content_size.height =
        round(cumulative_y + unrounded.content_size.height) - round(cumulative_y);
    layout.scrollbar_size.width = round(unrounded.scrollbar_size.width);
    layout.scrollbar_size.height = round(unrounded.scrollbar_size.height);
    layout.border.left = round(cumulative_x + unrounded.border.left) - round(cumulative_x);
    layout.border.right = round(cumulative_x + unrounded.size.width)
        - round(cumulative_x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round(cumulative_y + unrounded.border.top) - round(cumulative_y);
    layout.border.bottom = round(cumulative_y + unrounded.size.height)
        - round(cumulative_y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round(cumulative_x + unrounded.padding.left) - round(cumulative_x);
    layout.padding.right = round(cumulative_x + unrounded.size.width)
        - round(cumulative_x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round(cumulative_y + unrounded.padding.top) - round(cumulative_y);
    layout.padding.bottom = round(cumulative_y + unrounded.size.height)
        - round(cumulative_y + unrounded.size.height - unrounded.padding.bottom);
    let scroll_geometry = unrounded
        .scroll_geometry
        .map(|geometry| {
            rebuild_rounded_canonical_scroll_geometry(
                geometry,
                Point::new(cumulative_x, cumulative_y),
            )
        })
        .transpose()
        .map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidRoundedScrollGeometry,
                ),
            )
        })?;
    layout = layout.with_scroll_geometry(scroll_geometry);

    tree.set_final(node, layout);

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        round_layout_inner(tree, child, cumulative_x, cumulative_y)?;
    }
    Ok(())
}

#[inline]
fn round<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafMeasureInputOf<S: LayoutScalar = DefaultScalar> {
    known_content_size: Size<Option<S>>,
    available_content_size: Size<MeasurementAvailableOf<S>>,
}

pub type LeafMeasureInput = LeafMeasureInputOf<DefaultScalar>;

impl<S: LayoutScalar> LeafMeasureInputOf<S> {
    #[must_use]
    pub const fn known_content_size(&self) -> Size<Option<S>> {
        self.known_content_size
    }

    #[must_use]
    pub const fn available_content_size(&self) -> Size<MeasurementAvailableOf<S>> {
        self.available_content_size
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeasurementAvailableOf<S: LayoutScalar = DefaultScalar> {
    Definite(NonNegativeFiniteOf<S>),
    MinContent,
    MaxContent,
}

pub type MeasurementAvailable = MeasurementAvailableOf<DefaultScalar>;

impl<S: LayoutScalar> MeasurementAvailableOf<S> {
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    pub fn definite(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self::Definite(NonNegativeFiniteOf::new(value)?))
    }

    #[must_use]
    pub const fn definite_value(self) -> Option<NonNegativeFiniteOf<S>> {
        match self {
            Self::Definite(value) => Some(value),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub const fn into_available(self) -> AvailableOf<S> {
        match self {
            Self::Definite(value) => AvailableOf::Definite(value.get()),
            Self::MinContent => AvailableOf::MinContent,
            Self::MaxContent => AvailableOf::MaxContent,
        }
    }

    fn from_content_space(value: AvailableOf<S>) -> Result<Self, S> {
        match value {
            AvailableOf::Definite(value) => Ok(Self::Definite(
                NonNegativeFiniteOf::new(finite_floor_at_zero(value)?)
                    .expect("finite content-space availability is non-negative"),
            )),
            AvailableOf::MinContent => Ok(Self::MinContent),
            AvailableOf::MaxContent => Ok(Self::MaxContent),
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafResolvedValues<S: LayoutScalar> {
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    node_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    preferred_intrinsic_availability: Size<Option<AvailableOf<S>>>,
    aspect_ratio: Option<AspectRatioOf<S>>,
}

fn resolve_leaf_values<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolve_auto: impl Fn(super::LengthAutoOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
    resolve_length: impl Fn(super::LengthOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    let margin = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.margin, input.parent(), resolve_auto),
    )?;
    let padding = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.padding, input.parent(), &resolve_length),
    )?;
    let border = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.border, input.parent(), resolve_length),
    )?;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };

    let (node_size, node_min_size, node_max_size, preferred_intrinsic_availability, aspect_ratio) =
        match input.sizing_mode() {
            SizingMode::ContentSize => (input.known(), Size::NONE, Size::NONE, Size::NONE, None),
            SizingMode::InherentSize => {
                let missing_basis_is_indefinite = input.run_mode() == RunMode::ComputeSize;
                let preferred = Size::new(
                    resolve_preferred_sizing(
                        &style.size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_preferred_sizing(
                        &style.size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                );
                let style_size = preferred
                    .map(|resolved| match resolved {
                        ResolvedPreferredSize::Definite(value) => Some(value),
                        ResolvedPreferredSize::Auto
                        | ResolvedPreferredSize::MinContent
                        | ResolvedPreferredSize::MaxContent => None,
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let preferred_intrinsic_availability = preferred.map(|resolved| match resolved {
                    ResolvedPreferredSize::MinContent => Some(AvailableOf::MIN_CONTENT),
                    ResolvedPreferredSize::MaxContent => Some(AvailableOf::MAX_CONTENT),
                    ResolvedPreferredSize::Auto | ResolvedPreferredSize::Definite(_) => None,
                });
                let style_min_size = Size::new(
                    resolve_minimum_optional(
                        &style.min_size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_minimum_optional(
                        &style.min_size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                )
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let style_max_size = Size::new(
                    resolve_maximum_optional(
                        &style.max_size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_maximum_optional(
                        &style.max_size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                )
                .add_optional(box_sizing_adjustment);

                (
                    input.known().or(style_size),
                    style_min_size,
                    style_max_size,
                    preferred_intrinsic_availability,
                    style.aspect_ratio,
                )
            }
        };

    Ok(LeafResolvedValues {
        margin,
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        preferred_intrinsic_availability,
        aspect_ratio,
    })
}

fn resolve_leaf_values_for_input<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    resolve_leaf_values(
        input,
        style,
        |length, basis| resolve_leaf_auto(input, length, basis),
        |length, basis| resolve_leaf_length(input, length, basis),
    )
}

fn transpose_leaf_edges<S, E>(edges: Edges<Result<S, E>>) -> Result<Edges<S>, E> {
    Ok(Edges::new(
        edges.top?,
        edges.right?,
        edges.bottom?,
        edges.left?,
    ))
}

fn edge_at_physical_side<T: Copy>(edges: Edges<T>, side: PhysicalSide) -> T {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> Result<Size<S>, M>,
) -> LayoutResultOf<(), ComputeOutputOf<S>, S, M>
where
    S: LayoutScalar,
{
    let site = LayoutErrorSiteOf::Standalone;
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|error| sizing_resolution_error_at_site(site, error))?;

    compute_leaf_with_resolved_values(site, input, style, resolved, |measure_input| {
        measure(measure_input).map_err(|error| {
            LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )
        })
    })
}

fn compute_leaf_with_resolved_values<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: LeafResolvedValues<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let LeafResolvedValues {
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        ..
    } = resolved;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let leaf_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let block_start = leaf_flow_axes.block_start();
    let block_end = leaf_flow_axes.block_end();
    let node_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_size.width,
        PhysicalAxis::Vertical => node_size.height,
    };
    let node_min_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_min_size.width,
        PhysicalAxis::Vertical => node_min_size.height,
    };

    let prevents_margin_collapse = input.parent_formatting_context()
        != super::ParentFormattingContext::BlockFlow
        || style.display != super::Display::Block
        || !style.item_is_replaced && style.overflow.establishes_independent_formatting_context()
        || style.position == Position::Absolute
        || edge_at_physical_side(padding, block_start) > S::ZERO
        || edge_at_physical_side(padding, block_end) > S::ZERO
        || edge_at_physical_side(border, block_start) > S::ZERO
        || edge_at_physical_side(border, block_end) > S::ZERO
        || matches!(node_block_size, Some(size) if size > S::ZERO)
        || matches!(node_min_block_size, Some(size) if size > S::ZERO);

    if input.run_mode() == RunMode::ComputeSize
        && prevents_margin_collapse
        && let Size {
            width: Some(width),
            height: Some(height),
        } = node_size
    {
        let size = Size::new(width, height)
            .clamp_optional(node_min_size, node_max_size)
            .max_optional(padding_border_size.map(Some));
        return Ok(ComputeOutputOf::from_outer_size(size));
    }

    let mut pass_input = input;
    let mut reusable_measurement = None;
    loop {
        let pass = leaf_pass_input(site, pass_input, style, &resolved)?;
        let measured = match reusable_measurement.take() {
            Some((measurement_input, measured)) if measurement_input == pass.measurement_input => {
                measured
            }
            _ => validate_measurement_output(measure(pass.measurement_input)?)
                .map_err(|error| leaf_measurement_error_at_site(site, error))?,
        };
        let unclamped = pass_input
            .known()
            .or(resolved.node_size)
            .unwrap_or(measured + pass.content_box_inset_size);
        let height_is_definite =
            pass_input.known().height.is_some() || resolved.node_size.height.is_some();
        let aspect_height = if height_is_definite {
            unclamped.height
        } else {
            unclamped.height.max(
                resolved
                    .aspect_ratio
                    .map(|ratio| unclamped.width / ratio.get())
                    .unwrap_or(S::ZERO),
            )
        };
        let aspect_size = Size::new(unclamped.width, aspect_height)
            .clamp_optional(resolved.node_min_size, resolved.node_max_size)
            .max_optional(padding_border_size.map(Some));

        let mut output =
            ComputeOutputOf::from_sizes(aspect_size, measured + resolved.padding.sum_axes());
        let can_collapse_through = !prevents_margin_collapse
            && leaf_flow_axes.logical_size(aspect_size).block == S::ZERO
            && leaf_flow_axes.logical_size(measured).block == S::ZERO;
        output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
            leaf_flow_axes,
            CollapsibleMarginOf::ZERO,
            CollapsibleMarginOf::ZERO,
            can_collapse_through,
        );
        let geometry =
            canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
                flow_axes: leaf_flow_axes,
                computed_overflow: style.overflow,
                item_is_replaced: style.item_is_replaced,
                border_box_size: aspect_size,
                border: resolved.border,
                padding: resolved.padding,
                scrollbar_gutter: style.scrollbar_gutter,
                scrollbar_width: style.scrollbar_width,
                settled_auto_scrollbars: pass_input.settled_auto_scrollbars(),
                clip_margin: ClipMarginSourceOf::new(
                    style.overflow_clip_margin.clip_box(),
                    style.overflow_clip_margin.margin(),
                ),
                scroll_padding: leaf_scroll_padding(style.scroll_padding),
                measured_content_size: measured,
                scroll_snap_type: style.scroll_snap_type,
                target_scroll_margin: style.scroll_margin,
                target_snap_align: style.scroll_snap_align,
                target_snap_stop: style.scroll_snap_stop,
            })
            .map_err(|error| leaf_scroll_error_at_site(site, pass_input.run_mode(), error))?;
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || style.scrollbar_width.get() == S::ZERO
        {
            if input.run_mode().is_perform_layout() {
                output.scroll_geometry = Some(geometry);
            }
            return Ok(output);
        }

        reusable_measurement = Some((pass.measurement_input, measured));
        pass_input = pass_input.with_settled_auto_scrollbars(next_state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafPassInputOf<S: LayoutScalar> {
    content_box_inset_size: Size<S>,
    measurement_input: LeafMeasureInputOf<S>,
}

fn leaf_scroll_padding<S: LayoutScalar>(
    scroll_padding: crate::ScrollPaddingOf<S>,
) -> OptimalRegionInsetsOf<S> {
    fn inset<S: LayoutScalar>(value: crate::ScrollPaddingValueOf<S>) -> OptimalRegionInsetOf<S> {
        match value {
            crate::ScrollPaddingValueOf::Value(value) => OptimalRegionInsetOf::Value(value),
            crate::ScrollPaddingValueOf::Auto => OptimalRegionInsetOf::Auto,
        }
    }

    OptimalRegionInsetsOf::new(
        inset(scroll_padding.top()),
        inset(scroll_padding.right()),
        inset(scroll_padding.bottom()),
        inset(scroll_padding.left()),
    )
}

fn leaf_pass_input<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: &LeafResolvedValues<S>,
) -> LayoutResultOf<Node, LeafPassInputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let content_box_inset = measured_leaf_content_box_inset(MeasuredLeafContentBoxInsetSourceOf {
        flow_axes: FlowAxes::new(style.writing_mode, style.direction),
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: input.settled_auto_scrollbars(),
        padding: resolved.padding,
        border: resolved.border,
    });
    let content_box_inset_size = content_box_inset.sum_axes();
    let available = Size::new(
        input
            .known()
            .width
            .map(AvailableOf::definite)
            .unwrap_or(input.available().width)
            .sub_margin(resolved.margin.horizontal_sum())
            .set_optional(input.known().width)
            .set_optional(resolved.node_size.width)
            .map_definite(|value| {
                value.clamp_optional(resolved.node_min_size.width, resolved.node_max_size.width)
                    - content_box_inset.horizontal_sum()
            }),
        input
            .known()
            .height
            .map(AvailableOf::definite)
            .unwrap_or(input.available().height)
            .sub_margin(resolved.margin.vertical_sum())
            .set_optional(input.known().height)
            .set_optional(resolved.node_size.height)
            .map_definite(|value| {
                value.clamp_optional(resolved.node_min_size.height, resolved.node_max_size.height)
                    - content_box_inset.vertical_sum()
            }),
    )
    .zip_map(
        resolved.preferred_intrinsic_availability,
        |available, intrinsic| intrinsic.unwrap_or(available),
    );

    let known_content_size = match input.run_mode() {
        RunMode::ComputeSize => input.known(),
        RunMode::PerformRootLayout | RunMode::PerformLayout => Size::NONE,
        RunMode::PerformHiddenLayout => {
            unreachable!("hidden layout uses ComputeOutput::HIDDEN")
        }
    }
    .zip_map(content_box_inset_size, |value, inset| {
        value.map(|value| finite_floor_at_zero(value - inset))
    });
    let known_content_size =
        transpose_leaf_known_content_size(known_content_size).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let available_content_size =
        measurement_available_content_size(available).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let measurement_input = LeafMeasureInputOf {
        known_content_size,
        available_content_size,
    };
    Ok(LeafPassInputOf {
        content_box_inset_size,
        measurement_input,
    })
}

fn leaf_scroll_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    run_mode: RunMode,
    _error: CanonicalScrollGeometryErrorOf<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let (operation, invariant) = match (site, run_mode) {
        (LayoutErrorSiteOf::Standalone, _) => (
            LayoutOperation::LeafMeasurement,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (_, RunMode::PerformRootLayout) => (
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        _ => (
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    };
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InternalInvariant(invariant),
    )
}

fn validate_measurement_output<S, M>(measured: Size<S>) -> Result<Size<S>, LeafMeasureErrorOf<S, M>>
where
    S: LayoutScalar,
{
    let width = NonNegativeFiniteOf::new(measured.width)
        .map_err(|error| invalid_measurement_output(PhysicalAxis::Horizontal, error))?;
    let height = NonNegativeFiniteOf::new(measured.height)
        .map_err(|error| invalid_measurement_output(PhysicalAxis::Vertical, error))?;

    Ok(Size::new(width.get(), height.get()))
}

fn invalid_measurement_output<S, M>(
    axis: PhysicalAxis,
    error: NonNegativeFiniteScalarErrorOf<S>,
) -> LeafMeasureErrorOf<S, M>
where
    S: LayoutScalar,
{
    LeafMeasureErrorOf::InvalidOutput(InvalidMeasurementOutputOf { axis, error })
}

fn finite_floor_at_zero<S>(value: S) -> Result<S, S>
where
    S: LayoutScalar,
{
    if value.is_finite() {
        Ok(value.max(S::ZERO))
    } else {
        Err(value)
    }
}

fn transpose_leaf_known_content_size<S>(
    size: Size<Option<Result<S, S>>>,
) -> Result<Size<Option<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(size.width.transpose()?, size.height.transpose()?))
}

fn measurement_available_content_size<S>(
    available: Size<AvailableOf<S>>,
) -> Result<Size<MeasurementAvailableOf<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(
        MeasurementAvailableOf::from_content_space(available.width)?,
        MeasurementAvailableOf::from_content_space(available.height)?,
    ))
}

fn invalid_numeric_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    value: S,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }),
    )
}

fn leaf_measurement_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    error: LeafMeasureErrorOf<S, M>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = match error {
        LeafMeasureErrorOf::Provider(error) => LayoutErrorKindOf::Measurement(error),
        LeafMeasureErrorOf::InvalidOutput(error) => {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(error))
        }
    };
    LayoutErrorOf::new(site, LayoutOperation::LeafMeasurement, kind)
}

fn resolve_length_or_zero_fallible<S>(
    length: super::LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    resolution_or_zero_fallible(length.resolve_with_status(basis))
}

fn resolve_auto_or_zero_fallible<S>(
    length: super::LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolution_optional_fallible(length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

fn resolve_leaf_auto<S>(
    input: ComputeInputOf<S>,
    length: super::LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolve_leaf_optional(input, length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

fn resolve_leaf_length<S>(
    input: ComputeInputOf<S>,
    length: super::LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    let resolution = length.resolve_with_status(basis);
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(S::ZERO);
    }

    resolution_or_zero_fallible(resolution)
}

fn resolve_leaf_optional<S>(
    input: ComputeInputOf<S>,
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(None);
    }

    resolution_optional_fallible(resolution)
}

fn resolution_or_zero_fallible<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(resolution.status())
        }
        LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

fn resolution_optional_fallible<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(resolution.status())
        }
        LengthResolutionStatus::NonNumeric => Ok(None),
    }
}

pub(crate) trait SizeResultExt<S: LayoutScalar> {
    type Output;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>;
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<S, LengthResolutionStatus<S>>> {
    type Output = Size<S>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|status| value_resolution_error(node, status))?,
            self.height
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<Option<S>, LengthResolutionStatus<S>>> {
    type Output = Size<Option<S>>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|status| value_resolution_error(node, status))?,
            self.height
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<Option<S>, SizingResolutionError<S>>> {
    type Output = Size<Option<S>>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|error| sizing_resolution_error(node, error))?,
            self.height
                .map_err(|error| sizing_resolution_error(node, error))?,
        ))
    }
}

pub(crate) trait EdgesResultExt<S: LayoutScalar> {
    type Output;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>;
}

impl<S: LayoutScalar> EdgesResultExt<S> for super::Edges<Result<S, LengthResolutionStatus<S>>> {
    type Output = super::Edges<S>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(super::Edges::new(
            self.top
                .map_err(|status| value_resolution_error(node, status))?,
            self.right
                .map_err(|status| value_resolution_error(node, status))?,
            self.bottom
                .map_err(|status| value_resolution_error(node, status))?,
            self.left
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> EdgesResultExt<S>
    for super::Edges<Result<Option<S>, LengthResolutionStatus<S>>>
{
    type Output = super::Edges<Option<S>>;

    fn transpose_with_node<Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<M, Scalar = S>,
    {
        Ok(super::Edges::new(
            self.top
                .map_err(|status| value_resolution_error(node, status))?,
            self.right
                .map_err(|status| value_resolution_error(node, status))?,
            self.bottom
                .map_err(|status| value_resolution_error(node, status))?,
            self.left
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

trait SizeOptionExt {
    type Scalar: LayoutScalar;

    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size<Self::Scalar>) -> Size<Self::Scalar>;
    fn add_optional(self, amount: Size<Self::Scalar>) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> SizeOptionExt for Size<Option<S>> {
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

trait SizeExt {
    type Scalar: LayoutScalar;

    fn clamp_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
    fn max_optional(self, min: Size<Option<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> SizeExt for Size<S> {
    type Scalar = S;

    fn clamp_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }

    fn max_optional(self, min: Size<Option<S>>) -> Self {
        Size::new(
            min.width.map_or(self.width, |min| self.width.max(min)),
            min.height.map_or(self.height, |min| self.height.max(min)),
        )
    }
}

trait ScalarExt {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl<S: LayoutScalar> ScalarExt for S {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}

trait AvailableExt {
    type Scalar: LayoutScalar;

    fn sub_margin(self, margin: Self::Scalar) -> Self;
    fn set_optional(self, value: Option<Self::Scalar>) -> Self;
    fn map_definite(self, f: impl FnOnce(Self::Scalar) -> Self::Scalar) -> Self;
}

impl<S: LayoutScalar> AvailableExt for AvailableOf<S> {
    type Scalar = S;

    fn sub_margin(self, margin: S) -> Self {
        self.map_definite(|value| value - margin)
    }

    fn set_optional(self, value: Option<S>) -> Self {
        value.map_or(self, AvailableOf::definite)
    }

    fn map_definite(self, f: impl FnOnce(S) -> S) -> Self {
        match self {
            AvailableOf::Definite(value) => AvailableOf::Definite(f(value)),
            AvailableOf::MinContent => AvailableOf::MinContent,
            AvailableOf::MaxContent => AvailableOf::MaxContent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutInputOf, LayoutTree, NodeInput, Traverse};

    struct EmptyTree {
        input: NodeInput,
    }

    impl Traverse for EmptyTree {
        type Node = u32;
        type Scalar = DefaultScalar;
        type Children<'a> = std::iter::Empty<u32>;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("empty test tree has no children")
        }
    }

    impl LayoutTree for EmptyTree {
        type MeasureError = ();

        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.input
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.input.clone())
        }
    }

    #[test]
    fn compute_session_rejects_missing_staged_unrounded_output() {
        let tree = EmptyTree {
            input: NodeInput::default(),
        };
        let session = ComputeSession::new(&tree);

        let error = Round::unrounded(&session, 0).unwrap_err();

        assert_eq!(error.site(), LayoutErrorSite::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::MissingStagedUnroundedOutput,
            )
        );
    }
}
