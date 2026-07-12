use super::{
    AspectRatioOf, AvailableOf, Axis, BoxSizing, CacheAccess, Compute, ComputeInputOf,
    ComputeOutputOf, DefaultScalar, Direction, Edges, LayoutCacheClearEntry,
    LayoutCacheStoreEntryOf, LayoutInputOf, LayoutOutputEntryOf, LayoutRootContextOf,
    LayoutRootRequestOf, LayoutScalar, LengthResolutionOf, LengthResolutionStatus, NodeInputOf,
    NodeOutputOf, NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf, Point, Position, Round,
    RunMode, Size, SizingMode, Traverse,
};
use crate::scroll::{
    ScrollUnsupportedFeature, ScrollbarReservationOf, content_box_inset_with_scrollbar,
    round_scroll_geometry, scroll_geometry_from_layout, scroll_rect_union,
    scrollable_overflow_from_layout_content_size, scrollbar_size_from_overflow,
};
use crate::{CompletedLayoutBatchOf, LayoutTree};

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
        axis: Axis,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutUnsupportedCapability {
    LaterFriBehavior,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutInternalInvariant {
    InvalidRootScrollGeometry,
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
            super::Display::None => compute_hidden(self, node, input.containing_flow_axes()),
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
            .map_err(|status| value_resolution_error(node, status))?;

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

fn root_scroll_error<Node, S, M>(
    node: Node,
    error: ScrollUnsupportedFeature,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = if error.is_phase_one_deferred() {
        LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior)
    } else {
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry)
    };

    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RoundingFinalization,
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
            return compute_hidden(self, node, input.containing_flow_axes());
        }

        if input.run_mode().is_perform_layout() && self.child_count(node) != 0 {
            return self.compute_child_uncached(node, input);
        }

        crate::traits::compute_cached(self, node, input, |session, node, input| {
            session.compute_child_uncached(node, input)
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
        self.cache_store_entries
            .push(LayoutCacheStoreEntryOf::new(node, *input, context, output));
    }

    fn cache_clear(&mut self, node: Self::Node) {
        self.cache_clear_entries
            .push(LayoutCacheClearEntry::new(node));
    }
}

pub(crate) fn compute_hidden<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    containing_flow_axes: crate::geometry::FlowAxes,
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
    tree.set_unrounded(node, NodeOutputOf::with_order(0));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        match tree.layout_input(child) {
            LayoutInputOf::Box(_) => {
                tree.compute_child(child, ComputeInputOf::hidden(containing_flow_axes))?;
            }
            LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => {
                tree.cache_clear(child);
                tree.set_unrounded(child, NodeOutputOf::with_order(0));
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
    let containing_flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
    let known = Size::new(
        root_known_width::<Tree, M>(tree, root, &style, available.width)?,
        None,
    );
    let output = tree.compute_child(
        root,
        ComputeInputOf::root_layout(
            known,
            available.map(AvailableOf::into_option),
            containing_flow_axes,
            available,
        ),
    )?;
    let parent_width = available.width.into_option();
    let inline_basis = Size::splat(parent_width);
    let padding = style
        .padding
        .zip_size(inline_basis, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, root)?;
    let border = style
        .border
        .zip_size(inline_basis, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, root)?;
    let margin = style
        .margin
        .zip_size(inline_basis, |length, basis| {
            resolve_auto_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, root)?;
    let scrollbar_size = scrollbar_size_from_overflow(style.overflow, style.scrollbar_width.get());
    let scrollable_overflow = scrollable_overflow_from_layout_content_size(
        style.direction,
        style.overflow,
        output.size,
        padding,
        border,
        style.scrollbar_width.get(),
        output.content_size,
    )
    .map_err(|error| root_scroll_error(root, error))?;
    let scrollable_overflow = output
        .scroll_geometry
        .map(|geometry| scroll_rect_union(scrollable_overflow, geometry.scrollable_overflow()))
        .transpose()
        .map_err(|error| root_scroll_error(root, error))?
        .unwrap_or(scrollable_overflow);
    let scroll_geometry = Some(
        scroll_geometry_from_layout(
            style.writing_mode,
            style.direction,
            style.overflow,
            output.size,
            padding,
            border,
            style.scrollbar_width.get(),
            scrollable_overflow,
        )
        .map_err(|error| root_scroll_error(root, error))?,
    );
    let location = super::Point::new(
        if style.direction.is_rtl() {
            parent_width.map_or(<Tree as Traverse>::Scalar::ZERO, |width| {
                width - output.size.width
            })
        } else {
            <Tree as Traverse>::Scalar::ZERO
        },
        <Tree as Traverse>::Scalar::ZERO,
    );

    tree.set_unrounded(
        root,
        NodeOutputOf {
            order: 0,
            location,
            size: output.size,
            content_size: output.content_size,
            scroll_geometry,
            scrollbar_size,
            padding,
            border,
            margin,
        },
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
    let containing_flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
    let output = tree.compute_child(
        root,
        ComputeInputOf::flex_item_root(
            context.viewport_available().map(AvailableOf::into_option),
            containing_flow_axes,
            available,
        ),
    )?;
    tree.set_unrounded(
        root,
        NodeOutputOf {
            order: 0,
            location: Point::ZERO,
            size: output.size,
            content_size: output.content_size,
            ..NodeOutputOf::new()
        },
    );
    Ok(())
}

fn root_known_width<Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    available_width: AvailableOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Option<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<M>,
{
    if style.display.is_inline_level()
        || !style.size.width.is_auto()
        || !style.min_size.width.is_auto()
    {
        return Ok(None);
    }

    let Some(available_width) = available_width.into_option() else {
        return Ok(None);
    };
    let parent = Size::splat(Some(available_width));
    let padding = crate::geometry::FlowAxes::new(style.writing_mode, style.direction)
        .zip_physical_edges_with_inline_extent(style.padding, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = crate::geometry::FlowAxes::new(style.writing_mode, style.direction)
        .zip_physical_edges_with_inline_extent(style.border, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = style
        .max_size
        .zip_map(parent, |dimension, basis| {
            resolve_dimension_fallible(dimension, basis)
        })
        .transpose_with_node(tree, node)?
        .add_optional(box_sizing_adjustment);

    Ok(Some(available_width.clamp_optional(None, max_size.width)))
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
    layout.scroll_geometry = unrounded
        .scroll_geometry
        .map(|geometry| round_scroll_geometry(geometry, Point::new(cumulative_x, cumulative_y)))
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
    axis: super::Axis,
    error: NonNegativeFiniteScalarErrorOf<S>,
}

pub type InvalidMeasurementOutput = InvalidMeasurementOutputOf<DefaultScalar>;

impl<S: LayoutScalar> InvalidMeasurementOutputOf<S> {
    #[must_use]
    pub const fn axis(self) -> super::Axis {
        self.axis
    }

    #[must_use]
    pub const fn error(self) -> NonNegativeFiniteScalarErrorOf<S> {
        self.error
    }
}

struct LeafResolvedValues<S: LayoutScalar> {
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    node_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    aspect_ratio: Option<AspectRatioOf<S>>,
}

fn resolve_leaf_values<S, E>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolve_auto: impl Fn(super::LengthAutoOf<S>, Option<S>) -> Result<S, E>,
    resolve_length: impl Fn(super::LengthOf<S>, Option<S>) -> Result<S, E>,
    resolve_dimension: impl Fn(super::DimensionOf<S>, Option<S>) -> Result<Option<S>, E>,
) -> Result<LeafResolvedValues<S>, E>
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

    let (node_size, node_min_size, node_max_size, aspect_ratio) = match input.sizing_mode() {
        SizingMode::ContentSize => (input.known(), Size::NONE, Size::NONE, None),
        SizingMode::InherentSize => {
            let style_size =
                transpose_leaf_size(style.size.zip_map(input.parent(), |dimension, basis| {
                    resolve_dimension(dimension, basis)
                }))?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
            let style_min_size =
                transpose_leaf_size(style.min_size.zip_map(input.parent(), |dimension, basis| {
                    resolve_dimension(dimension, basis)
                }))?
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
            let style_max_size =
                transpose_leaf_size(style.max_size.zip_map(input.parent(), |dimension, basis| {
                    resolve_dimension(dimension, basis)
                }))?
                .add_optional(box_sizing_adjustment);

            (
                input.known().or(style_size),
                style_min_size,
                style_max_size,
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
        aspect_ratio,
    })
}

fn resolve_leaf_values_for_input<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
) -> Result<LeafResolvedValues<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    resolve_leaf_values(
        input,
        style,
        |length, basis| resolve_leaf_auto(input, length, basis),
        |length, basis| resolve_leaf_length(input, length, basis),
        |dimension, basis| resolve_leaf_dimension(input, dimension, basis),
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

fn transpose_leaf_size<S, E>(size: Size<Result<Option<S>, E>>) -> Result<Size<Option<S>>, E> {
    Ok(Size::new(size.width?, size.height?))
}

pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    measure: impl FnOnce(LeafMeasureInputOf<S>) -> Result<Size<S>, M>,
) -> LayoutResultOf<(), ComputeOutputOf<S>, S, M>
where
    S: LayoutScalar,
{
    let site = LayoutErrorSiteOf::Standalone;
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|status| value_resolution_error_at_site(site, status))?;

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
    measure: impl FnOnce(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let LeafResolvedValues {
        margin,
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        aspect_ratio,
    } = resolved;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
        style.overflow,
        style.scrollbar_width.get(),
        Direction::Ltr,
    );
    let content_box_inset =
        content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
    let content_box_inset_size = content_box_inset.sum_axes();

    let prevents_margin_collapse = style.display != super::Display::Block
        || style.overflow.x.blocks_margin_collapse()
        || style.overflow.y.blocks_margin_collapse()
        || style.position == Position::Absolute
        || padding.top > S::ZERO
        || padding.bottom > S::ZERO
        || border.top > S::ZERO
        || border.bottom > S::ZERO
        || matches!(node_size.height, Some(height) if height > S::ZERO)
        || matches!(node_min_size.height, Some(height) if height > S::ZERO);

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

    let available = Size::new(
        input
            .known()
            .width
            .map(AvailableOf::definite)
            .unwrap_or(input.available().width)
            .sub_margin(margin.horizontal_sum())
            .set_optional(input.known().width)
            .set_optional(node_size.width)
            .map_definite(|value| {
                value.clamp_optional(node_min_size.width, node_max_size.width)
                    - content_box_inset.horizontal_sum()
            }),
        input
            .known()
            .height
            .map(AvailableOf::definite)
            .unwrap_or(input.available().height)
            .sub_margin(margin.vertical_sum())
            .set_optional(input.known().height)
            .set_optional(node_size.height)
            .map_definite(|value| {
                value.clamp_optional(node_min_size.height, node_max_size.height)
                    - content_box_inset.vertical_sum()
            }),
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
    let measured = validate_measurement_output(measure(measurement_input)?)
        .map_err(|error| leaf_measurement_error_at_site(site, error))?;
    let unclamped = input
        .known()
        .or(node_size)
        .unwrap_or(measured + content_box_inset_size);
    let height_is_definite = input.known().height.is_some() || node_size.height.is_some();
    let aspect_height = if height_is_definite {
        unclamped.height
    } else {
        unclamped.height.max(
            aspect_ratio
                .map(|ratio| unclamped.width / ratio.get())
                .unwrap_or(S::ZERO),
        )
    };
    let aspect_size = Size::new(unclamped.width, aspect_height)
        .clamp_optional(node_min_size, node_max_size)
        .max_optional(padding_border_size.map(Some));

    let mut output = ComputeOutputOf::from_sizes(aspect_size, measured + padding.sum_axes());
    output.margins_can_collapse_through =
        !prevents_margin_collapse && aspect_size.height == S::ZERO && measured.height == S::ZERO;
    Ok(output)
}

fn validate_measurement_output<S, M>(measured: Size<S>) -> Result<Size<S>, LeafMeasureErrorOf<S, M>>
where
    S: LayoutScalar,
{
    let width = NonNegativeFiniteOf::new(measured.width)
        .map_err(|error| invalid_measurement_output(super::Axis::Horizontal, error))?;
    let height = NonNegativeFiniteOf::new(measured.height)
        .map_err(|error| invalid_measurement_output(super::Axis::Vertical, error))?;

    Ok(Size::new(width.get(), height.get()))
}

fn invalid_measurement_output<S, M>(
    axis: super::Axis,
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

fn resolve_leaf_dimension<S>(
    input: ComputeInputOf<S>,
    dimension: super::DimensionOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    resolve_leaf_optional(input, dimension.resolve_with_status(basis))
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

fn resolve_dimension_fallible<S>(
    dimension: super::DimensionOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    resolution_optional_fallible(dimension.resolve_with_status(basis))
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
