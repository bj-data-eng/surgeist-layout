use super::super::{
    DefaultScalar, FiniteScalarErrorOf, LayoutScalar, LengthPercentageOf,
    NonNegativeFiniteScalarErrorOf, NumericResolutionOf, PercentageBasisOf, PhysicalSide,
};
use super::validate_numeric_property;

/// A computed overflow keyword for one physical axis.
///
/// Pair-level construction goes through [`ComputedOverflow`]. Phase-specific
/// clipping and margin-collapse predicates are not available per axis.
///
/// ```compile_fail
/// use surgeist_layout::Overflow;
///
/// let _ = Overflow::Hidden.clips_contents();
/// ```
///
/// ```compile_fail
/// use surgeist_layout::Overflow;
///
/// let _ = Overflow::Hidden.blocks_margin_collapse();
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Scroll,
    Auto,
}

impl Overflow {
    #[must_use]
    pub const fn is_scrollable(self) -> bool {
        matches!(self, Self::Hidden | Self::Scroll | Self::Auto)
    }
}

/// A normalized canonical computed overflow pair for layout input.
///
/// Both axes are constructed atomically through [`Self::try_new`]; callers
/// cannot supply the former raw physical point or mutate one axis independently.
/// This is the post-cascade computed pair, not authored CSS. Layout privately
/// derives used axis values from this pair and `item_is_replaced`; that used
/// phase is observable only on layout-produced scroll geometry.
///
/// ```compile_fail
/// use surgeist_layout::{NodeInput, Overflow, Point};
///
/// let mut input = NodeInput::DEFAULT;
/// input.overflow = Point::new(Overflow::Visible, Overflow::Visible);
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{ComputedOverflow, Overflow};
///
/// let mut overflow = ComputedOverflow::try_new(Overflow::Visible, Overflow::Clip).unwrap();
/// overflow.x = Overflow::Hidden;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComputedOverflow {
    x: Overflow,
    y: Overflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComputedOverflowError {
    NonCanonicalPair { x: Overflow, y: Overflow },
}

impl core::fmt::Display for ComputedOverflowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonCanonicalPair { .. } => f.write_str(
                "computed overflow axes must both be visible/clip or both be hidden/scroll/auto",
            ),
        }
    }
}

impl std::error::Error for ComputedOverflowError {}

impl ComputedOverflow {
    pub const VISIBLE: Self = Self {
        x: Overflow::Visible,
        y: Overflow::Visible,
    };

    pub fn try_new(x: Overflow, y: Overflow) -> Result<Self, ComputedOverflowError> {
        if x.is_scrollable() == y.is_scrollable() {
            Ok(Self { x, y })
        } else {
            Err(ComputedOverflowError::NonCanonicalPair { x, y })
        }
    }

    #[must_use]
    pub const fn x(self) -> Overflow {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> Overflow {
        self.y
    }

    #[must_use]
    pub const fn establishes_independent_formatting_context(self) -> bool {
        self.x.is_scrollable() && self.y.is_scrollable()
    }
}

impl Default for ComputedOverflow {
    fn default() -> Self {
        Self::VISIBLE
    }
}

/// Reference box used by the overflow clip edge.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverflowClipBox {
    ContentBox,
    #[default]
    PaddingBox,
    BorderBox,
}

/// A validated overflow clip reference box and absolute margin.
///
/// The margin is finite and non-negative. Signed zero is normalized to the
/// scalar lane's canonical zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverflowClipMarginOf<S: LayoutScalar = DefaultScalar> {
    pub(super) clip_box: OverflowClipBox,
    pub(super) margin: S,
}

/// Default-scalar overflow clip margin.
pub type OverflowClipMargin = OverflowClipMarginOf<DefaultScalar>;

impl<S: LayoutScalar> OverflowClipMarginOf<S> {
    /// Constructs an overflow clip margin atomically.
    pub fn try_new(
        clip_box: OverflowClipBox,
        margin: S,
    ) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            clip_box,
            margin: validate_numeric_property(margin)?,
        })
    }

    /// Returns the overflow clip reference box.
    #[must_use]
    pub const fn clip_box(self) -> OverflowClipBox {
        self.clip_box
    }

    /// Returns the finite non-negative absolute margin.
    #[must_use]
    pub const fn margin(self) -> S {
        self.margin
    }
}

impl<S: LayoutScalar> Default for OverflowClipMarginOf<S> {
    fn default() -> Self {
        Self {
            clip_box: OverflowClipBox::PaddingBox,
            margin: S::ZERO,
        }
    }
}

/// Normalized scrollbar gutter reservation policy supplied to layout.
///
/// The companion [`ScrollbarWidthOf`] carries the explicit finite physical
/// thickness selected by the caller's overlay/classic host environment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarGutter {
    #[default]
    Auto,
    Stable,
    StableBothEdges,
}

/// One physical scroll-padding edge in normalized layout input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollPaddingValueOf<S: LayoutScalar = DefaultScalar> {
    Value(LengthPercentageOf<S>),
    Auto,
}

/// Default-scalar scroll-padding edge value.
pub type ScrollPaddingValue = ScrollPaddingValueOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollPaddingValueOf<S> {
    /// The CSS initial `auto` value.
    pub const AUTO: Self = Self::Auto;

    /// Wraps an intrinsically validated length-percentage value.
    #[must_use]
    pub const fn value(value: LengthPercentageOf<S>) -> Self {
        Self::Value(value)
    }

    /// Constructs the CSS initial `auto` value.
    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    /// Returns whether this edge retains the `auto` value.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Resolves this edge against its corresponding physical dimension.
    ///
    /// Callers pass scrollport width for left/right and scrollport height for
    /// top/bottom. Product `auto` resolves to zero. A negative numeric result
    /// is clamped to canonical zero, while missing-basis and invalid-numeric
    /// outcomes retain the underlying length-percentage diagnostic.
    #[must_use]
    pub fn resolve_against(self, basis: PercentageBasisOf<S>) -> NumericResolutionOf<S> {
        let resolution = match self {
            Self::Value(value) => value.resolve_against(basis),
            Self::Auto => NumericResolutionOf::Resolved(S::ZERO),
        };

        match resolution {
            NumericResolutionOf::Resolved(value) if value < S::ZERO => {
                NumericResolutionOf::Resolved(S::ZERO)
            }
            NumericResolutionOf::Resolved(value) if value == S::ZERO => {
                NumericResolutionOf::Resolved(S::ZERO)
            }
            resolution => resolution,
        }
    }
}

impl<S: LayoutScalar> Default for ScrollPaddingValueOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

/// Four normalized physical scroll-padding edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollPaddingOf<S: LayoutScalar = DefaultScalar> {
    pub(super) top: ScrollPaddingValueOf<S>,
    pub(super) right: ScrollPaddingValueOf<S>,
    pub(super) bottom: ScrollPaddingValueOf<S>,
    pub(super) left: ScrollPaddingValueOf<S>,
}

/// Default-scalar physical scroll padding.
pub type ScrollPadding = ScrollPaddingOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollPaddingOf<S> {
    /// Constructs four physical edges in top/right/bottom/left order.
    #[must_use]
    pub const fn new(
        top: ScrollPaddingValueOf<S>,
        right: ScrollPaddingValueOf<S>,
        bottom: ScrollPaddingValueOf<S>,
        left: ScrollPaddingValueOf<S>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn top(self) -> ScrollPaddingValueOf<S> {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> ScrollPaddingValueOf<S> {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> ScrollPaddingValueOf<S> {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> ScrollPaddingValueOf<S> {
        self.left
    }
}

impl<S: LayoutScalar> Default for ScrollPaddingOf<S> {
    fn default() -> Self {
        Self::new(
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::AUTO,
        )
    }
}

/// Atomic scroll-margin construction failure for one physical edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollMarginErrorOf<S: LayoutScalar = DefaultScalar> {
    edge: PhysicalSide,
    source: FiniteScalarErrorOf<S>,
}

/// Default-scalar scroll-margin construction error.
pub type ScrollMarginError = ScrollMarginErrorOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollMarginErrorOf<S> {
    /// Returns the exact rejected physical edge.
    #[must_use]
    pub const fn edge(&self) -> PhysicalSide {
        self.edge
    }

    /// Returns the preserved scalar validation error.
    #[must_use]
    pub const fn error(&self) -> FiniteScalarErrorOf<S> {
        self.source
    }
}

impl<S: LayoutScalar> core::fmt::Display for ScrollMarginErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let edge = match self.edge {
            PhysicalSide::Top => "top",
            PhysicalSide::Right => "right",
            PhysicalSide::Bottom => "bottom",
            PhysicalSide::Left => "left",
        };
        write!(f, "scroll margin {edge} edge must be finite")
    }
}

impl<S: LayoutScalar> std::error::Error for ScrollMarginErrorOf<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Four finite signed absolute physical scroll-margin outsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollMarginOf<S: LayoutScalar = DefaultScalar> {
    pub(super) top: S,
    pub(super) right: S,
    pub(super) bottom: S,
    pub(super) left: S,
}

/// Default-scalar physical scroll margin.
pub type ScrollMargin = ScrollMarginOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollMarginOf<S> {
    /// Constructs all four physical edges atomically.
    pub fn try_new(top: S, right: S, bottom: S, left: S) -> Result<Self, ScrollMarginErrorOf<S>> {
        Ok(Self {
            top: validate_scroll_margin_edge(PhysicalSide::Top, top)?,
            right: validate_scroll_margin_edge(PhysicalSide::Right, right)?,
            bottom: validate_scroll_margin_edge(PhysicalSide::Bottom, bottom)?,
            left: validate_scroll_margin_edge(PhysicalSide::Left, left)?,
        })
    }

    #[must_use]
    pub const fn top(self) -> S {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> S {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> S {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> S {
        self.left
    }
}

impl<S: LayoutScalar> Default for ScrollMarginOf<S> {
    fn default() -> Self {
        Self {
            top: S::ZERO,
            right: S::ZERO,
            bottom: S::ZERO,
            left: S::ZERO,
        }
    }
}

fn validate_scroll_margin_edge<S: LayoutScalar>(
    edge: PhysicalSide,
    value: S,
) -> Result<S, ScrollMarginErrorOf<S>> {
    if !value.is_finite() {
        return Err(ScrollMarginErrorOf {
            edge,
            source: FiniteScalarErrorOf::NonFinite { value },
        });
    }

    Ok(if value == S::ZERO { S::ZERO } else { value })
}

/// Axis selected by an enabled scroll snap type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollSnapAxis {
    X,
    Y,
    Block,
    Inline,
    Both,
}

/// Strictness selected by an enabled scroll snap type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollSnapStrictness {
    Proximity,
    Mandatory,
}

/// Normalized scroll snap metadata for a scroll container.
///
/// Layout records geometry for this value but does not select a live snap
/// position or retain a current scroll offset.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollSnapType {
    #[default]
    None,
    Enabled {
        axis: ScrollSnapAxis,
        strictness: ScrollSnapStrictness,
    },
}

/// One semantic block- or inline-axis snap alignment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollSnapAlignValue {
    #[default]
    None,
    Start,
    End,
    Center,
}

/// Explicit semantic block and inline snap alignments for one target.
///
/// These roles remain semantic until root associates and transforms the target
/// against its eventual snap container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollSnapAlign {
    pub(super) block: ScrollSnapAlignValue,
    pub(super) inline: ScrollSnapAlignValue,
}

impl ScrollSnapAlign {
    /// Constructs semantic block and inline values without physical mapping.
    #[must_use]
    pub const fn new(block: ScrollSnapAlignValue, inline: ScrollSnapAlignValue) -> Self {
        Self { block, inline }
    }

    #[must_use]
    pub const fn block(self) -> ScrollSnapAlignValue {
        self.block
    }

    #[must_use]
    pub const fn inline(self) -> ScrollSnapAlignValue {
        self.inline
    }
}

impl Default for ScrollSnapAlign {
    fn default() -> Self {
        Self::new(ScrollSnapAlignValue::None, ScrollSnapAlignValue::None)
    }
}

/// Whether a scroll snap target may be skipped during an active snap operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollSnapStop {
    #[default]
    Normal,
    Always,
}

/// Explicit finite non-negative physical scrollbar thickness for layout.
///
/// Root or a standalone caller lowers its host scrollbar environment before
/// constructing this value. Zero represents overlay or disabled thickness;
/// layout never probes host UI metrics or supplies an implicit classic width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarWidthOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type ScrollbarWidth = ScrollbarWidthOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollbarWidthOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self {
            value: validate_numeric_property(value)?,
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for ScrollbarWidthOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}
