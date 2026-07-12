use super::{
    AvailableOf, Axis, CacheKeyContext, DefaultScalar, Edges, LayoutScalar, NonNegativeFiniteOf,
    NonNegativeFiniteScalarErrorOf, Point, ScrollGeometryOf, Size,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunMode {
    PerformRootLayout,
    PerformLayout,
    ComputeSize,
    PerformHiddenLayout,
}

impl RunMode {
    pub const fn is_perform_layout(self) -> bool {
        matches!(self, Self::PerformRootLayout | Self::PerformLayout)
    }

    pub const fn for_child(self) -> Self {
        match self {
            Self::PerformRootLayout => Self::PerformLayout,
            mode => mode,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SizingMode {
    ContentSize,
    InherentSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequestedAxis {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeInputOf<S: LayoutScalar = DefaultScalar> {
    pub(crate) run_mode: RunMode,
    pub(crate) sizing_mode: SizingMode,
    pub(crate) axis: RequestedAxis,
    pub(crate) known: Size<Option<S>>,
    pub(crate) parent: Size<Option<S>>,
    pub(crate) available: Size<AvailableOf<S>>,
}

pub type ComputeInput = ComputeInputOf<DefaultScalar>;

impl<S: LayoutScalar> ComputeInputOf<S> {
    pub(crate) const HIDDEN: Self = Self {
        run_mode: RunMode::PerformHiddenLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::NONE,
        available: Size::splat(AvailableOf::MAX_CONTENT),
    };

    pub fn leaf_layout(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        available: Size<AvailableOf<S>>,
    ) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Ok(Self {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: validate_optional_size(known)?,
            parent: validate_optional_size(parent)?,
            available: validate_root_available_size(available)?,
        })
    }

    pub fn leaf_content_size(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        available: Size<AvailableOf<S>>,
    ) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Ok(Self {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::ContentSize,
            axis: RequestedAxis::Both,
            known: validate_optional_size(known)?,
            parent: validate_optional_size(parent)?,
            available: validate_root_available_size(available)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayoutRoundingMode {
    #[default]
    NearestCssPixel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootAvailabilityErrorOf<S: LayoutScalar = DefaultScalar> {
    axis: Axis,
    scalar: NonNegativeFiniteScalarErrorOf<S>,
}

pub type RootAvailabilityError = RootAvailabilityErrorOf<DefaultScalar>;

impl<S: LayoutScalar> RootAvailabilityErrorOf<S> {
    #[must_use]
    pub const fn axis(&self) -> Axis {
        self.axis
    }

    #[must_use]
    pub const fn scalar(&self) -> NonNegativeFiniteScalarErrorOf<S> {
        self.scalar
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexItemRootContextOf<S: LayoutScalar = DefaultScalar> {
    viewport_available: Size<AvailableOf<S>>,
}

pub type FlexItemRootContext = FlexItemRootContextOf<DefaultScalar>;

impl<S: LayoutScalar> FlexItemRootContextOf<S> {
    pub fn under_viewport(
        viewport_available: Size<AvailableOf<S>>,
    ) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Ok(Self {
            viewport_available: validate_root_available_size(viewport_available)?,
        })
    }

    #[must_use]
    pub const fn viewport_available(&self) -> Size<AvailableOf<S>> {
        self.viewport_available
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutRootContextOf<S: LayoutScalar = DefaultScalar> {
    Viewport,
    FlexItemUnderViewport(FlexItemRootContextOf<S>),
}

pub type LayoutRootContext = LayoutRootContextOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutRootRequestOf<S: LayoutScalar = DefaultScalar> {
    available: Size<AvailableOf<S>>,
    context: LayoutRootContextOf<S>,
    rounding_mode: LayoutRoundingMode,
}

pub type LayoutRootRequest = LayoutRootRequestOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutRootRequestOf<S> {
    pub fn viewport(available: Size<AvailableOf<S>>) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Self::new(
            available,
            LayoutRootContextOf::Viewport,
            LayoutRoundingMode::NearestCssPixel,
        )
    }

    pub fn flex_item_under_viewport(
        available: Size<AvailableOf<S>>,
        context: FlexItemRootContextOf<S>,
    ) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Self::new(
            available,
            LayoutRootContextOf::FlexItemUnderViewport(context),
            LayoutRoundingMode::NearestCssPixel,
        )
    }

    pub fn with_rounding_mode(
        self,
        rounding_mode: LayoutRoundingMode,
    ) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Self::new(self.available, self.context, rounding_mode)
    }

    fn new(
        available: Size<AvailableOf<S>>,
        context: LayoutRootContextOf<S>,
        rounding_mode: LayoutRoundingMode,
    ) -> Result<Self, RootAvailabilityErrorOf<S>> {
        Ok(Self {
            available: validate_root_available_size(available)?,
            context,
            rounding_mode,
        })
    }

    #[must_use]
    pub const fn available(&self) -> Size<AvailableOf<S>> {
        self.available
    }

    #[must_use]
    pub const fn context(&self) -> LayoutRootContextOf<S> {
        self.context
    }

    #[must_use]
    pub const fn rounding_mode(&self) -> LayoutRoundingMode {
        self.rounding_mode
    }
}

fn validate_root_available_size<S>(
    available: Size<AvailableOf<S>>,
) -> Result<Size<AvailableOf<S>>, RootAvailabilityErrorOf<S>>
where
    S: LayoutScalar,
{
    Ok(Size::new(
        validate_root_available_axis(Axis::Horizontal, available.width)?,
        validate_root_available_axis(Axis::Vertical, available.height)?,
    ))
}

fn validate_optional_size<S>(
    size: Size<Option<S>>,
) -> Result<Size<Option<S>>, RootAvailabilityErrorOf<S>>
where
    S: LayoutScalar,
{
    Ok(Size::new(
        validate_optional_axis(Axis::Horizontal, size.width)?,
        validate_optional_axis(Axis::Vertical, size.height)?,
    ))
}

fn validate_optional_axis<S>(
    axis: Axis,
    value: Option<S>,
) -> Result<Option<S>, RootAvailabilityErrorOf<S>>
where
    S: LayoutScalar,
{
    value
        .map(|value| {
            NonNegativeFiniteOf::new(value)
                .map(NonNegativeFiniteOf::get)
                .map_err(|scalar| RootAvailabilityErrorOf { axis, scalar })
        })
        .transpose()
}

fn validate_root_available_axis<S>(
    axis: Axis,
    available: AvailableOf<S>,
) -> Result<AvailableOf<S>, RootAvailabilityErrorOf<S>>
where
    S: LayoutScalar,
{
    match available {
        AvailableOf::Definite(value) => NonNegativeFiniteOf::new(value)
            .map(|value| AvailableOf::Definite(value.get()))
            .map_err(|scalar| RootAvailabilityErrorOf { axis, scalar }),
        AvailableOf::MinContent => Ok(AvailableOf::MinContent),
        AvailableOf::MaxContent => Ok(AvailableOf::MaxContent),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollapsibleMarginOf<S: LayoutScalar = DefaultScalar> {
    positive: S,
    negative: S,
}

pub type CollapsibleMargin = CollapsibleMarginOf<DefaultScalar>;

impl<S: LayoutScalar> CollapsibleMarginOf<S> {
    pub const ZERO: Self = Self {
        positive: S::ZERO,
        negative: S::ZERO,
    };

    #[must_use]
    pub fn from_margin(margin: S) -> Self {
        if margin >= S::ZERO {
            Self {
                positive: margin,
                negative: S::ZERO,
            }
        } else {
            Self {
                positive: S::ZERO,
                negative: margin,
            }
        }
    }

    #[must_use]
    pub fn collapse_with_margin(self, margin: S) -> Self {
        self.collapse_with(Self::from_margin(margin))
    }

    #[must_use]
    pub fn collapse_with(self, other: Self) -> Self {
        Self {
            positive: self.positive.max(other.positive),
            negative: self.negative.min(other.negative),
        }
    }

    #[must_use]
    pub fn resolve(self) -> S {
        self.positive + self.negative
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselinesOf<S: LayoutScalar = DefaultScalar> {
    pub first: Point<Option<S>>,
    pub last: Point<Option<S>>,
}

pub type Baselines = BaselinesOf<DefaultScalar>;

impl<S: LayoutScalar> BaselinesOf<S> {
    pub const NONE: Self = Self {
        first: Point::NONE,
        last: Point::NONE,
    };

    #[must_use]
    pub const fn first(first: Point<Option<S>>) -> Self {
        Self {
            first,
            last: Point::NONE,
        }
    }

    #[must_use]
    pub const fn synthesized(size: Size<S>) -> Self {
        Self {
            first: Point::new(Some(size.width), Some(size.height)),
            last: Point::new(Some(S::ZERO), Some(S::ZERO)),
        }
    }

    #[must_use]
    pub fn first_or_synthesize_block(self, size: Size<S>) -> S {
        self.first.y.unwrap_or(size.height)
    }

    #[must_use]
    pub fn last_or_synthesize_block(self, _size: Size<S>) -> S {
        self.last.y.unwrap_or(S::ZERO)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeOutputOf<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub scroll_geometry: Option<ScrollGeometryOf<S>>,
    pub first_baselines: Point<Option<S>>,
    pub last_baselines: Point<Option<S>>,
    pub top_margin: CollapsibleMarginOf<S>,
    pub bottom_margin: CollapsibleMarginOf<S>,
    pub margins_can_collapse_through: bool,
}

pub type ComputeOutput = ComputeOutputOf<DefaultScalar>;

impl<S: LayoutScalar> ComputeOutputOf<S> {
    pub const HIDDEN: Self = Self {
        size: Size::<S>::ZERO,
        content_size: Size::<S>::ZERO,
        scroll_geometry: None,
        first_baselines: Point::NONE,
        last_baselines: Point::NONE,
        top_margin: CollapsibleMarginOf::ZERO,
        bottom_margin: CollapsibleMarginOf::ZERO,
        margins_can_collapse_through: false,
    };

    pub const DEFAULT: Self = Self::HIDDEN;

    #[must_use]
    pub const fn from_sizes_and_baselines(
        size: Size<S>,
        content_size: Size<S>,
        baselines: BaselinesOf<S>,
    ) -> Self {
        Self {
            size,
            content_size,
            scroll_geometry: None,
            first_baselines: baselines.first,
            last_baselines: baselines.last,
            top_margin: CollapsibleMarginOf::ZERO,
            bottom_margin: CollapsibleMarginOf::ZERO,
            margins_can_collapse_through: false,
        }
    }

    #[must_use]
    pub const fn from_sizes_and_first_baselines(
        size: Size<S>,
        content_size: Size<S>,
        first_baselines: Point<Option<S>>,
    ) -> Self {
        Self::from_sizes_and_baselines(size, content_size, BaselinesOf::first(first_baselines))
    }

    #[must_use]
    pub const fn from_sizes(size: Size<S>, content_size: Size<S>) -> Self {
        Self::from_sizes_and_baselines(size, content_size, BaselinesOf::NONE)
    }

    #[must_use]
    pub const fn from_outer_size(size: Size<S>) -> Self {
        Self::from_sizes(size, Size::<S>::ZERO)
    }

    #[must_use]
    pub const fn baselines(&self) -> BaselinesOf<S> {
        BaselinesOf {
            first: self.first_baselines,
            last: self.last_baselines,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeOutputOf<S: LayoutScalar = DefaultScalar> {
    pub order: u32,
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub scroll_geometry: Option<ScrollGeometryOf<S>>,
    pub scrollbar_size: Size<S>,
    pub border: Edges<S>,
    pub padding: Edges<S>,
    pub margin: Edges<S>,
}

pub type NodeOutput = NodeOutputOf<DefaultScalar>;

impl<S: LayoutScalar> NodeOutputOf<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self::with_order(0)
    }

    #[must_use]
    pub const fn with_order(order: u32) -> Self {
        Self {
            order,
            location: Point::<S>::ZERO,
            size: Size::<S>::ZERO,
            content_size: Size::<S>::ZERO,
            scroll_geometry: None,
            scrollbar_size: Size::<S>::ZERO,
            border: Edges::<S>::ZERO,
            padding: Edges::<S>::ZERO,
            margin: Edges::<S>::ZERO,
        }
    }

    #[must_use]
    pub fn content_box_size(self) -> Size<S> {
        Size::new(
            self.size.width
                - self.padding.left
                - self.padding.right
                - self.border.left
                - self.border.right,
            self.size.height
                - self.padding.top
                - self.padding.bottom
                - self.border.top
                - self.border.bottom,
        )
    }
}

impl<S: LayoutScalar> Default for NodeOutputOf<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutOutputEntryOf<Node, S: LayoutScalar = DefaultScalar> {
    node: Node,
    output: NodeOutputOf<S>,
}

pub type LayoutOutputEntry<Node> = LayoutOutputEntryOf<Node, DefaultScalar>;

impl<Node, S> LayoutOutputEntryOf<Node, S>
where
    Node: Copy,
    S: LayoutScalar,
{
    pub(crate) const fn new(node: Node, output: NodeOutputOf<S>) -> Self {
        Self { node, output }
    }

    #[must_use]
    pub const fn node(&self) -> Node {
        self.node
    }

    #[must_use]
    pub const fn output(&self) -> NodeOutputOf<S> {
        self.output
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutCacheStoreEntryOf<Node, S: LayoutScalar = DefaultScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    context: CacheKeyContext,
    output: ComputeOutputOf<S>,
}

pub type LayoutCacheStoreEntry<Node> = LayoutCacheStoreEntryOf<Node, DefaultScalar>;

impl<Node, S> LayoutCacheStoreEntryOf<Node, S>
where
    Node: Copy,
    S: LayoutScalar,
{
    pub(crate) const fn new(
        node: Node,
        input: ComputeInputOf<S>,
        context: CacheKeyContext,
        output: ComputeOutputOf<S>,
    ) -> Self {
        Self {
            node,
            input,
            context,
            output,
        }
    }

    #[must_use]
    pub const fn node(&self) -> Node {
        self.node
    }

    #[must_use]
    pub const fn input(&self) -> &ComputeInputOf<S> {
        &self.input
    }

    #[must_use]
    pub const fn context(&self) -> CacheKeyContext {
        self.context
    }

    #[must_use]
    pub const fn output(&self) -> ComputeOutputOf<S> {
        self.output
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutCacheClearEntry<Node> {
    node: Node,
}

impl<Node> LayoutCacheClearEntry<Node>
where
    Node: Copy,
{
    pub(crate) const fn new(node: Node) -> Self {
        Self { node }
    }

    #[must_use]
    pub const fn node(&self) -> Node {
        self.node
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompletedLayoutBatchOf<Node, S: LayoutScalar = DefaultScalar> {
    unrounded_entries: Vec<LayoutOutputEntryOf<Node, S>>,
    final_entries: Vec<LayoutOutputEntryOf<Node, S>>,
    cache_store_entries: Vec<LayoutCacheStoreEntryOf<Node, S>>,
    cache_clear_entries: Vec<LayoutCacheClearEntry<Node>>,
}

pub type CompletedLayoutBatch<Node> = CompletedLayoutBatchOf<Node, DefaultScalar>;

impl<Node, S> CompletedLayoutBatchOf<Node, S>
where
    S: LayoutScalar,
{
    pub(crate) fn from_entries(
        unrounded_entries: Vec<LayoutOutputEntryOf<Node, S>>,
        final_entries: Vec<LayoutOutputEntryOf<Node, S>>,
        cache_store_entries: Vec<LayoutCacheStoreEntryOf<Node, S>>,
        cache_clear_entries: Vec<LayoutCacheClearEntry<Node>>,
    ) -> Self {
        Self {
            unrounded_entries,
            final_entries,
            cache_store_entries,
            cache_clear_entries,
        }
    }

    #[must_use]
    pub fn unrounded_entries(&self) -> &[LayoutOutputEntryOf<Node, S>] {
        &self.unrounded_entries
    }

    #[must_use]
    pub fn final_entries(&self) -> &[LayoutOutputEntryOf<Node, S>] {
        &self.final_entries
    }

    #[must_use]
    pub fn cache_store_entries(&self) -> &[LayoutCacheStoreEntryOf<Node, S>] {
        &self.cache_store_entries
    }

    #[must_use]
    pub fn cache_clear_entries(&self) -> &[LayoutCacheClearEntry<Node>] {
        &self.cache_clear_entries
    }
}
