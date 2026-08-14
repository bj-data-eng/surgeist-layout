use super::{
    AspectRatioOf, DefaultScalar, Edges, FlexBasisOf, GridTemplateAreas, LayoutScalar,
    LengthAutoOf, LengthOf, MaxSizeOf, MinSizeOf, NonNegativeFiniteScalarErrorOf, PreferredSizeOf,
    Size, TrackComponentOf,
};

#[cfg(test)]
use super::{
    FiniteScalarErrorOf, FlowAxes, LengthPercentageOf, NumericResolutionOf, PercentageBasisOf,
    PhysicalSide, ScrollRectOf,
};

mod alignment;
mod box_model;
mod flex;
mod grid;
mod inline;
mod scroll;

pub(crate) use alignment::item_order_permutation;
pub use alignment::{AlignContent, AlignItems, AlignSelf, ItemOrder, JustifyContent};
pub use box_model::{
    BoxSizing, Clear, Direction, Display, Float, FloatExclusion, FloatExclusionInterval,
    FloatExclusionIntervalError, FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf,
    FloatExclusionQuery, FloatExclusionQueryOf, Position,
};
pub use flex::{
    FlexDirection, FlexGrow, FlexGrowOf, FlexItemCollapse, FlexShrink, FlexShrinkOf, FlexWrap,
};
pub use grid::{
    GridAutoFlow, GridFlowTolerance, GridFlowToleranceOf, GridPlacement, RawGridLine,
    RawGridPlacement,
};
pub use inline::{
    AtomicInlineParticipation, AtomicInlineParticipationError, AtomicInlineParticipationErrorOf,
    AtomicInlineParticipationOf, BidiLevel, BidiLevelError, InlineBoundaryInput,
    InlineBoundaryInputOf, InlineBoundaryKind, InlineBreakKind, InlineBreakOpportunity,
    InlineBreakOpportunityOf, InlineMetrics, InlineMetricsError, InlineMetricsOf, InlineSegmentId,
    InlineTextInput, InlineTextInputError, InlineTextInputErrorOf, InlineTextInputOf,
    InlineWhitespaceEdge, LineBreakDisplay, LineBreakInput, LineBreakInputOf, ShapedInlineSegment,
    ShapedInlineSegmentOf, TextAlign, VerticalAlign, WritingMode,
};
pub use scroll::{
    ComputedOverflow, ComputedOverflowError, Overflow, OverflowClipBox, OverflowClipMargin,
    OverflowClipMarginOf, ScrollMargin, ScrollMarginError, ScrollMarginErrorOf, ScrollMarginOf,
    ScrollPadding, ScrollPaddingOf, ScrollPaddingValue, ScrollPaddingValueOf, ScrollSnapAlign,
    ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType,
    ScrollbarGutter, ScrollbarWidth, ScrollbarWidthOf,
};

fn validate_numeric_property<S: LayoutScalar>(
    value: S,
) -> Result<S, NonNegativeFiniteScalarErrorOf<S>> {
    if !value.is_finite() {
        return Err(NonNegativeFiniteScalarErrorOf::NonFinite { value });
    }

    if value < S::ZERO {
        return Err(NonNegativeFiniteScalarErrorOf::Negative { value });
    }

    Ok(if value == S::ZERO { S::ZERO } else { value })
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeInputOf<S: LayoutScalar = DefaultScalar> {
    pub display: Display,
    pub atomic_inline_participation: Option<AtomicInlineParticipationOf<S>>,
    pub float_exclusion: FloatExclusion,
    pub item_is_table: bool,
    pub item_is_replaced: bool,
    pub item_order: ItemOrder,
    /// Normalized, layout-ready flex-layout participation for this node.
    ///
    /// This is not authored or computed CSS `visibility`.
    /// [`FlexItemCollapse::Normal`] is the default. Only in-flow children of
    /// flex containers consume [`FlexItemCollapse::Collapsed`]; other contexts
    /// preserve their existing behavior.
    pub flex_item_collapse: FlexItemCollapse,
    pub box_sizing: BoxSizing,
    pub direction: Direction,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub writing_mode: WritingMode,
    /// Atomic normalized computed overflow supplied by style lowering.
    pub overflow: ComputedOverflow,
    /// Normalized finite overflow clip reference box and margin.
    pub overflow_clip_margin: OverflowClipMarginOf<S>,
    /// Normalized gutter reservation policy.
    pub scrollbar_gutter: ScrollbarGutter,
    /// Explicit finite thickness from the caller's scrollbar environment.
    pub scrollbar_width: self::ScrollbarWidthOf<S>,
    /// Normalized physical scroll-padding edges.
    pub scroll_padding: ScrollPaddingOf<S>,
    /// Normalized finite signed physical target outsets.
    pub scroll_margin: ScrollMarginOf<S>,
    /// Container snap metadata; live selection remains root-owned.
    pub scroll_snap_type: ScrollSnapType,
    /// Target block/inline alignment metadata.
    pub scroll_snap_align: ScrollSnapAlign,
    /// Target pass-over metadata; live behavior remains root-owned.
    pub scroll_snap_stop: ScrollSnapStop,
    pub position: Position,
    pub float: Float,
    pub clear: Clear,
    pub inset: Edges<LengthAutoOf<S>>,
    pub size: Size<PreferredSizeOf<S>>,
    pub min_size: Size<MinSizeOf<S>>,
    pub max_size: Size<MaxSizeOf<S>>,
    pub aspect_ratio: Option<AspectRatioOf<S>>,
    pub margin: Edges<LengthAutoOf<S>>,
    pub padding: Edges<LengthOf<S>>,
    pub border: Edges<LengthOf<S>>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub justify_items: Option<AlignItems>,
    pub justify_self: Option<AlignSelf>,
    pub align_content: Option<AlignContent>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Size<LengthOf<S>>,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_basis: FlexBasisOf<S>,
    pub flex_grow: FlexGrowOf<S>,
    pub flex_shrink: FlexShrinkOf<S>,
    pub grid_template_columns: Vec<TrackComponentOf<S>>,
    pub grid_template_rows: Vec<TrackComponentOf<S>>,
    pub grid_template_areas: GridTemplateAreas,
    pub grid_auto_columns: Vec<TrackComponentOf<S>>,
    pub grid_auto_rows: Vec<TrackComponentOf<S>>,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_flow_tolerance: GridFlowToleranceOf<S>,
    pub grid_column: GridPlacement,
    pub grid_row: GridPlacement,
    pub raw_grid_column: RawGridPlacement,
    pub raw_grid_row: RawGridPlacement,
}

/// Property sizing fields use distinct public domains.
///
/// ```compile_fail
/// use surgeist_layout::{MaxSize, NodeInput, PreferredSize, Size};
/// let _ = NodeInput {
///     size: Size::splat(MaxSize::NONE),
///     ..NodeInput::DEFAULT
/// };
/// let _: PreferredSize = MaxSize::NONE;
/// ```
///
/// The removed broad sizing family has no compatibility reexport.
///
/// ```compile_fail
/// use surgeist_layout::Dimension;
/// let _ = Dimension::AUTO;
/// ```
///
/// ```compile_fail
/// use surgeist_layout::DimensionOf;
/// type Legacy = DimensionOf<f64>;
/// let _: Legacy = Legacy::AUTO;
/// ```
const _: () = ();

pub type NodeInput = NodeInputOf<DefaultScalar>;

impl NodeInputOf<DefaultScalar> {
    pub const DEFAULT: Self = Self {
        display: Display::Flex,
        atomic_inline_participation: None,
        float_exclusion: FloatExclusion::MarginBox,
        item_is_table: false,
        item_is_replaced: false,
        item_order: ItemOrder::ZERO,
        flex_item_collapse: FlexItemCollapse::Normal,
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
        text_align: TextAlign::Auto,
        vertical_align: VerticalAlign::Baseline,
        writing_mode: WritingMode::HorizontalTb,
        overflow: ComputedOverflow::VISIBLE,
        overflow_clip_margin: OverflowClipMarginOf {
            clip_box: OverflowClipBox::PaddingBox,
            margin: 0.0,
        },
        scrollbar_gutter: ScrollbarGutter::Auto,
        scrollbar_width: self::ScrollbarWidthOf::ZERO,
        scroll_padding: ScrollPaddingOf {
            top: ScrollPaddingValueOf::AUTO,
            right: ScrollPaddingValueOf::AUTO,
            bottom: ScrollPaddingValueOf::AUTO,
            left: ScrollPaddingValueOf::AUTO,
        },
        scroll_margin: ScrollMarginOf {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        },
        scroll_snap_type: ScrollSnapType::None,
        scroll_snap_align: ScrollSnapAlign {
            block: ScrollSnapAlignValue::None,
            inline: ScrollSnapAlignValue::None,
        },
        scroll_snap_stop: ScrollSnapStop::Normal,
        position: Position::Relative,
        float: Float::None,
        clear: Clear::None,
        inset: Edges::all(LengthAutoOf::AUTO),
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
        aspect_ratio: None,
        margin: Edges::all(LengthAutoOf::ZERO),
        padding: Edges::all(LengthOf::ZERO),
        border: Edges::all(LengthOf::ZERO),
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        gap: Size::new(LengthOf::NORMAL, LengthOf::NORMAL),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_basis: FlexBasisOf::AUTO,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::ONE,
        grid_template_columns: Vec::new(),
        grid_template_rows: Vec::new(),
        grid_template_areas: GridTemplateAreas { rows: Vec::new() },
        grid_auto_columns: Vec::new(),
        grid_auto_rows: Vec::new(),
        grid_auto_flow: GridAutoFlow::Row,
        grid_flow_tolerance: GridFlowToleranceOf::Normal { font_size: 16.0 },
        grid_column: GridPlacement::AUTO,
        grid_row: GridPlacement::AUTO,
        raw_grid_column: RawGridPlacement::AUTO,
        raw_grid_row: RawGridPlacement::AUTO,
    };
}

impl<S: LayoutScalar> Default for NodeInputOf<S> {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            atomic_inline_participation: None,
            float_exclusion: FloatExclusion::MarginBox,
            item_is_table: false,
            item_is_replaced: false,
            item_order: ItemOrder::ZERO,
            flex_item_collapse: FlexItemCollapse::Normal,
            box_sizing: BoxSizing::BorderBox,
            direction: Direction::Ltr,
            text_align: TextAlign::Auto,
            vertical_align: VerticalAlign::Baseline,
            writing_mode: WritingMode::HorizontalTb,
            overflow: ComputedOverflow::VISIBLE,
            overflow_clip_margin: OverflowClipMarginOf::default(),
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: self::ScrollbarWidthOf::ZERO,
            scroll_padding: ScrollPaddingOf::default(),
            scroll_margin: ScrollMarginOf::default(),
            scroll_snap_type: ScrollSnapType::None,
            scroll_snap_align: ScrollSnapAlign::default(),
            scroll_snap_stop: ScrollSnapStop::Normal,
            position: Position::Relative,
            float: Float::None,
            clear: Clear::None,
            inset: Edges::all(LengthAutoOf::AUTO),
            size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO),
            max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
            aspect_ratio: None,
            margin: Edges::all(LengthAutoOf::ZERO),
            padding: Edges::all(LengthOf::ZERO),
            border: Edges::all(LengthOf::ZERO),
            align_items: None,
            align_self: None,
            justify_items: None,
            justify_self: None,
            align_content: None,
            justify_content: None,
            gap: Size::new(LengthOf::NORMAL, LengthOf::NORMAL),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            flex_basis: FlexBasisOf::AUTO,
            flex_grow: FlexGrowOf::ZERO,
            flex_shrink: FlexShrinkOf::ONE,
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_template_areas: GridTemplateAreas { rows: Vec::new() },
            grid_auto_columns: Vec::new(),
            grid_auto_rows: Vec::new(),
            grid_auto_flow: GridAutoFlow::Row,
            grid_flow_tolerance: GridFlowToleranceOf::default(),
            grid_column: GridPlacement::AUTO,
            grid_row: GridPlacement::AUTO,
            raw_grid_column: RawGridPlacement::AUTO,
            raw_grid_row: RawGridPlacement::AUTO,
        }
    }
}

impl<S: LayoutScalar> NodeInputOf<S> {
    /// Returns the sole canonical companion for a non-box layout input.
    #[must_use]
    pub fn non_box() -> Self {
        Self {
            display: Display::None,
            flex_item_collapse: FlexItemCollapse::Normal,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod property_field_migration_tests {
    use super::*;
    use crate::{
        CalcSizeCalculation, FlexBasisOf, LengthPercentageOf, LengthResolutionStatus, MaxSizeOf,
        MinSizeOf, PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation,
    };

    fn assert_field_types<S: LayoutScalar>(input: &NodeInputOf<S>) {
        let _: &Size<PreferredSizeOf<S>> = &input.size;
        let _: &Size<MinSizeOf<S>> = &input.min_size;
        let _: &Size<MaxSizeOf<S>> = &input.max_size;
        let _: &FlexBasisOf<S> = &input.flex_basis;
    }

    fn assert_generic_defaults<S: LayoutScalar>() {
        let input = NodeInputOf::<S>::default();
        assert_field_types(&input);
        assert_eq!(
            input.size,
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
        );
        assert_eq!(input.min_size, Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO));
        assert_eq!(input.max_size, Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE));
        assert_eq!(input.flex_basis, FlexBasisOf::AUTO);
    }

    #[test]
    fn property_field_migration_default_scalar_uses_exact_domains_and_initial_values() {
        let input = &NodeInput::DEFAULT;
        assert_field_types(input);
        assert_eq!(
            input.size,
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
        );
        assert_eq!(input.min_size, Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO));
        assert_eq!(input.max_size, Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE));
        assert_eq!(input.flex_basis, FlexBasisOf::AUTO);
    }

    #[test]
    fn property_field_migration_generic_scalar_uses_exact_domains_and_initial_values() {
        assert_generic_defaults::<f32>();
        assert_generic_defaults::<f64>();
    }

    #[test]
    fn property_field_migration_numeric_calculations_resolve_while_later_states_stay_unsupported() {
        let nested =
            SizingCalculation::min(vec![SizingCalculation::value(LengthPercentageOf::ZERO)])
                .expect("nonempty sizing calculation");
        let preferred = PreferredSizeOf::calculation(nested);
        let resolution = preferred
            .resolve_simple_with_status(None)
            .expect("valid numeric calculation resolves");
        assert_eq!(resolution.status(), LengthResolutionStatus::Resolved);
        assert_eq!(resolution.value, Some(0.0));

        let calc_size =
            PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Auto, CalcSizeCalculation::size())
                .expect("valid preferred calc-size");
        assert_eq!(
            calc_size.resolve_simple_with_status(None),
            Err(LengthResolutionStatus::NonNumeric),
        );
        assert_eq!(
            FlexBasisOf::<f32>::CONTENT.resolve_simple_with_status(None),
            Err(LengthResolutionStatus::NonNumeric),
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutInputOf<S: LayoutScalar = DefaultScalar> {
    Box(std::boxed::Box<NodeInputOf<S>>),
    InlineText(InlineTextInputOf<S>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}

pub type LayoutInput = LayoutInputOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutInputOf<S> {
    #[must_use]
    pub fn box_input(input: NodeInputOf<S>) -> Self {
        Self::Box(std::boxed::Box::new(input))
    }

    #[must_use]
    pub const fn line_break(input: LineBreakInputOf<S>) -> Self {
        Self::LineBreak(input)
    }

    #[must_use]
    pub const fn inline_text(input: InlineTextInputOf<S>) -> Self {
        Self::InlineText(input)
    }

    #[must_use]
    pub const fn inline_boundary(input: InlineBoundaryInputOf<S>) -> Self {
        Self::InlineBoundary(input)
    }

    #[must_use]
    pub fn as_box(&self) -> Option<&NodeInputOf<S>> {
        match self {
            Self::Box(input) => Some(input.as_ref()),
            Self::InlineText(_) | Self::LineBreak(_) | Self::InlineBoundary(_) => None,
        }
    }

    #[must_use]
    pub const fn as_inline_text(&self) -> Option<&InlineTextInputOf<S>> {
        match self {
            Self::InlineText(input) => Some(input),
            Self::Box(_) | Self::LineBreak(_) | Self::InlineBoundary(_) => None,
        }
    }

    #[must_use]
    pub const fn as_line_break(&self) -> Option<LineBreakInputOf<S>> {
        match self {
            Self::Box(_) | Self::InlineText(_) | Self::InlineBoundary(_) => None,
            Self::LineBreak(input) => Some(*input),
        }
    }

    #[must_use]
    pub const fn as_inline_boundary(&self) -> Option<InlineBoundaryInputOf<S>> {
        match self {
            Self::Box(_) | Self::InlineText(_) | Self::LineBreak(_) => None,
            Self::InlineBoundary(input) => Some(*input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point, SourceIndex};

    fn assert_fri06_mr02_signed_zero_float_exclusion_boundaries<S: LayoutScalar>() {
        let margin_box = ScrollRectOf::try_new(
            Point::new(S::from_f64(-10.0), S::from_f64(-8.0)),
            Size::new(S::from_f64(20.0), S::from_f64(16.0)),
        )
        .expect("finite float margin box");
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let query = FloatExclusionQueryOf::try_new(margin_box, axes, -S::ZERO, S::from_f64(6.0))
            .expect("finite ordered band");
        assert_eq!(query.band_minimum(), S::ZERO);
        assert!(!query.band_minimum().to_f64().is_sign_negative());
        assert_eq!(query.band_maximum(), S::from_f64(6.0));

        let signed = FloatExclusionIntervalOf::try_new(query, -S::ZERO, S::from_f64(4.0))
            .expect("finite interval")
            .expect("overlapping interval");
        assert_eq!(signed.minimum(), S::ZERO);
        assert!(!signed.minimum().to_f64().is_sign_negative());
        assert_eq!(signed.maximum(), S::from_f64(4.0));

        let clipped =
            FloatExclusionIntervalOf::try_new(query, S::from_f64(-12.0), S::from_f64(-3.0))
                .expect("finite interval")
                .expect("partially overlapping interval");
        assert_eq!(clipped.minimum(), S::from_f64(-10.0));
        assert_eq!(clipped.maximum(), S::from_f64(-3.0));

        assert!(matches!(
            FloatExclusionQueryOf::try_new(margin_box, axes, S::NAN, S::INFINITY),
            Err(FloatExclusionIntervalErrorOf::NonFiniteBandMinimum { value })
                if value.to_f64().is_nan()
        ));
        assert!(matches!(
            FloatExclusionQueryOf::try_new(margin_box, axes, S::ZERO, -S::INFINITY),
            Err(FloatExclusionIntervalErrorOf::NonFiniteBandMaximum { value })
                if value == -S::INFINITY
        ));
        assert!(matches!(
            FloatExclusionQueryOf::try_new(margin_box, axes, S::ONE, S::ZERO),
            Err(FloatExclusionIntervalErrorOf::InvertedBand { minimum, maximum })
                if minimum == S::ONE && maximum == S::ZERO
        ));
        assert!(matches!(
            FloatExclusionIntervalOf::try_new(query, -S::INFINITY, S::NAN),
            Err(FloatExclusionIntervalErrorOf::NonFiniteIntervalMinimum { value })
                if value == -S::INFINITY
        ));
        assert!(matches!(
            FloatExclusionIntervalOf::try_new(query, S::ZERO, S::NAN),
            Err(FloatExclusionIntervalErrorOf::NonFiniteIntervalMaximum { value })
                if value.to_f64().is_nan()
        ));
        assert!(matches!(
            FloatExclusionIntervalOf::try_new(query, S::ONE, S::ZERO),
            Err(FloatExclusionIntervalErrorOf::InvertedInterval { minimum, maximum })
                if minimum == S::ONE && maximum == S::ZERO
        ));
    }

    #[test]
    fn fri06_mr02_signed_zero_float_exclusion_validation_clipping_and_order_are_preserved() {
        assert_fri06_mr02_signed_zero_float_exclusion_boundaries::<f32>();
        assert_fri06_mr02_signed_zero_float_exclusion_boundaries::<f64>();
    }

    fn assert_fri05_c01_node_input_fields_and_defaults<S: LayoutScalar>(input: &NodeInputOf<S>) {
        let _: &ComputedOverflow = &input.overflow;
        let _: &OverflowClipMarginOf<S> = &input.overflow_clip_margin;
        let _: &ScrollbarGutter = &input.scrollbar_gutter;
        let _: &ScrollbarWidthOf<S> = &input.scrollbar_width;
        let _: &ScrollPaddingOf<S> = &input.scroll_padding;
        let _: &ScrollMarginOf<S> = &input.scroll_margin;
        let _: &ScrollSnapType = &input.scroll_snap_type;
        let _: &ScrollSnapAlign = &input.scroll_snap_align;
        let _: &ScrollSnapStop = &input.scroll_snap_stop;

        assert_eq!(input.overflow, ComputedOverflow::VISIBLE);
        assert_eq!(
            input.overflow_clip_margin,
            OverflowClipMarginOf::<S>::default()
        );
        assert_eq!(input.scrollbar_gutter, ScrollbarGutter::Auto);
        assert_eq!(input.scrollbar_width, ScrollbarWidthOf::<S>::ZERO);
        assert_eq!(input.scroll_padding, ScrollPaddingOf::<S>::default());
        assert_eq!(input.scroll_margin, ScrollMarginOf::<S>::default());
        assert_eq!(input.scroll_snap_type, ScrollSnapType::None);
        assert_eq!(input.scroll_snap_align, ScrollSnapAlign::default());
        assert_eq!(input.scroll_snap_stop, ScrollSnapStop::Normal);
    }

    #[test]
    fn fri05_c01_node_input_default_and_generic_fields_have_exact_domains_and_initial_values() {
        assert_fri05_c01_node_input_fields_and_defaults(&NodeInput::DEFAULT);
        assert_fri05_c01_node_input_fields_and_defaults(&NodeInputOf::<f32>::default());
        assert_fri05_c01_node_input_fields_and_defaults(&NodeInputOf::<f64>::default());
    }

    fn assert_canonical_zero<S: LayoutScalar>(value: S) {
        assert_eq!(value, S::ZERO);
        assert_eq!(value.to_f64().to_bits(), 0.0f64.to_bits());
    }

    fn assert_scroll_input_scalar_traits<S: LayoutScalar>() {
        fn assert_value<T: Clone + Copy + core::fmt::Debug + PartialEq>() {}
        fn assert_error<T: Clone + Copy + core::fmt::Debug + PartialEq + std::error::Error>() {}

        assert_value::<OverflowClipMarginOf<S>>();
        assert_value::<ScrollPaddingValueOf<S>>();
        assert_value::<ScrollPaddingOf<S>>();
        assert_value::<ScrollMarginOf<S>>();
        assert_error::<ScrollMarginErrorOf<S>>();
    }

    fn assert_clip_margin_contract<S: LayoutScalar>() {
        let default = OverflowClipMarginOf::<S>::default();
        assert_eq!(default.clip_box(), OverflowClipBox::PaddingBox);
        assert_canonical_zero(default.margin());

        for clip_box in [
            OverflowClipBox::ContentBox,
            OverflowClipBox::PaddingBox,
            OverflowClipBox::BorderBox,
        ] {
            let clip_margin = OverflowClipMarginOf::try_new(clip_box, S::from_f64(4.5))
                .expect("finite non-negative clip margin");
            assert_eq!(clip_margin.clip_box(), clip_box);
            assert_eq!(clip_margin.margin(), S::from_f64(4.5));
        }

        let signed_zero = OverflowClipMarginOf::try_new(OverflowClipBox::BorderBox, -S::ZERO)
            .expect("signed zero clip margin is valid");
        assert_canonical_zero(signed_zero.margin());

        assert_eq!(
            OverflowClipMarginOf::try_new(OverflowClipBox::ContentBox, S::from_f64(-1.0)),
            Err(NonNegativeFiniteScalarErrorOf::Negative {
                value: S::from_f64(-1.0),
            })
        );
        for value in [S::NAN, S::INFINITY, -S::INFINITY] {
            assert!(matches!(
                OverflowClipMarginOf::try_new(OverflowClipBox::PaddingBox, value),
                Err(NonNegativeFiniteScalarErrorOf::NonFinite { value: rejected })
                    if !rejected.is_finite()
            ));
        }
    }

    #[test]
    fn fri05_c01_scroll_input_clip_margin_is_validated_in_both_scalar_lanes() {
        assert_clip_margin_contract::<f32>();
        assert_clip_margin_contract::<f64>();
    }

    fn assert_padding_contract<S: LayoutScalar>(largest_finite: S) {
        let auto = ScrollPaddingValueOf::<S>::AUTO;
        assert_eq!(ScrollPaddingValueOf::<S>::default(), auto);
        assert_eq!(ScrollPaddingValueOf::<S>::auto(), auto);
        assert!(auto.is_auto());
        assert_eq!(
            auto.resolve_against(PercentageBasisOf::MISSING),
            NumericResolutionOf::Resolved(S::ZERO)
        );

        let quarter = LengthPercentageOf::from_percent_fraction(S::from_f64(0.25))
            .expect("finite percentage");
        let value = ScrollPaddingValueOf::value(quarter);
        assert!(!value.is_auto());
        assert_eq!(
            value.resolve_against(
                PercentageBasisOf::definite(S::from_f64(200.0)).expect("finite width")
            ),
            NumericResolutionOf::Resolved(S::from_f64(50.0))
        );
        assert_eq!(
            value.resolve_against(
                PercentageBasisOf::definite(S::from_f64(80.0)).expect("finite height")
            ),
            NumericResolutionOf::Resolved(S::from_f64(20.0))
        );
        assert_eq!(
            value.resolve_against(PercentageBasisOf::MISSING),
            NumericResolutionOf::MissingBasis { value: quarter }
        );

        let negative = LengthPercentageOf::from_coefficients(S::from_f64(-30.0), S::from_f64(0.1))
            .expect("finite negative calculation");
        let NumericResolutionOf::Resolved(clamped) = ScrollPaddingValueOf::value(negative)
            .resolve_against(
                PercentageBasisOf::definite(S::from_f64(100.0)).expect("finite basis"),
            )
        else {
            panic!("negative used scroll padding must resolve");
        };
        assert_canonical_zero(clamped);

        let overflowing = LengthPercentageOf::from_percent_fraction(largest_finite)
            .expect("largest finite coefficient");
        let basis = PercentageBasisOf::definite(S::from_f64(2.0)).expect("finite basis");
        let NumericResolutionOf::InvalidNumeric {
            value: invalid_value,
            basis: invalid_basis,
            resolved,
        } = ScrollPaddingValueOf::value(overflowing).resolve_against(basis)
        else {
            panic!("non-finite evaluation must remain invalid");
        };
        assert_eq!(invalid_value, overflowing);
        assert_eq!(invalid_basis, basis);
        assert!(!resolved.is_finite());

        let padding = ScrollPaddingOf::new(
            auto,
            value,
            ScrollPaddingValueOf::value(negative),
            ScrollPaddingValueOf::value(LengthPercentageOf::ZERO),
        );
        assert_eq!(padding.top(), auto);
        assert_eq!(padding.right(), value);
        assert_eq!(padding.bottom(), ScrollPaddingValueOf::value(negative));
        assert_eq!(
            padding.left(),
            ScrollPaddingValueOf::value(LengthPercentageOf::ZERO)
        );

        let default = ScrollPaddingOf::<S>::default();
        assert_eq!(default.top(), auto);
        assert_eq!(default.right(), auto);
        assert_eq!(default.bottom(), auto);
        assert_eq!(default.left(), auto);
    }

    #[test]
    fn fri05_c01_scroll_input_padding_resolves_physical_bases_in_both_scalar_lanes() {
        assert_padding_contract::<f32>(f32::MAX);
        assert_padding_contract::<f64>(f64::MAX);
    }

    fn assert_same_non_finite<S: LayoutScalar>(actual: S, expected: S) {
        if expected.to_f64().is_nan() {
            assert!(actual.to_f64().is_nan());
        } else {
            assert_eq!(actual.to_f64(), expected.to_f64());
        }
    }

    fn assert_scroll_margin_contract<S: LayoutScalar>() {
        let margin = ScrollMarginOf::try_new(
            S::from_f64(-4.0),
            S::from_f64(2.0),
            -S::ZERO,
            S::from_f64(6.0),
        )
        .expect("finite signed scroll margins");
        assert_eq!(margin.top(), S::from_f64(-4.0));
        assert_eq!(margin.right(), S::from_f64(2.0));
        assert_canonical_zero(margin.bottom());
        assert_eq!(margin.left(), S::from_f64(6.0));

        let default = ScrollMarginOf::<S>::default();
        assert_canonical_zero(default.top());
        assert_canonical_zero(default.right());
        assert_canonical_zero(default.bottom());
        assert_canonical_zero(default.left());

        for (edge, values, rejected, edge_name) in [
            (
                PhysicalSide::Top,
                [S::NAN, S::ZERO, S::ZERO, S::ZERO],
                S::NAN,
                "top",
            ),
            (
                PhysicalSide::Right,
                [S::ZERO, S::INFINITY, S::ZERO, S::ZERO],
                S::INFINITY,
                "right",
            ),
            (
                PhysicalSide::Bottom,
                [S::ZERO, S::ZERO, -S::INFINITY, S::ZERO],
                -S::INFINITY,
                "bottom",
            ),
            (
                PhysicalSide::Left,
                [S::ZERO, S::ZERO, S::ZERO, S::NAN],
                S::NAN,
                "left",
            ),
        ] {
            let error = ScrollMarginOf::try_new(values[0], values[1], values[2], values[3])
                .expect_err("non-finite aggregate edge must fail atomically");
            assert_eq!(error.edge(), edge);
            let FiniteScalarErrorOf::NonFinite { value } = error.error();
            assert_same_non_finite(value, rejected);
            assert_eq!(
                error.to_string(),
                format!("scroll margin {edge_name} edge must be finite")
            );

            let source = std::error::Error::source(&error)
                .expect("scroll margin diagnostic preserves its scalar source")
                .downcast_ref::<FiniteScalarErrorOf<S>>()
                .expect("source has the exact finite-scalar type");
            let FiniteScalarErrorOf::NonFinite { value } = *source;
            assert_same_non_finite(value, rejected);
        }
    }

    #[test]
    fn fri05_c01_scroll_input_signed_margin_is_atomic_in_both_scalar_lanes() {
        assert_scroll_margin_contract::<f32>();
        assert_scroll_margin_contract::<f64>();
    }

    #[test]
    fn fri05_c01_scroll_input_closed_enums_cover_states_defaults_and_traits() {
        fn assert_closed<T: Clone + Copy + core::fmt::Debug + Eq + PartialEq>() {}

        assert_closed::<OverflowClipBox>();
        assert_closed::<ScrollbarGutter>();
        assert_closed::<ScrollSnapAxis>();
        assert_closed::<ScrollSnapStrictness>();
        assert_closed::<ScrollSnapType>();
        assert_closed::<ScrollSnapAlignValue>();
        assert_closed::<ScrollSnapAlign>();
        assert_closed::<ScrollSnapStop>();
        assert_scroll_input_scalar_traits::<f32>();
        assert_scroll_input_scalar_traits::<f64>();

        assert_eq!(OverflowClipBox::default(), OverflowClipBox::PaddingBox);
        assert_eq!(ScrollbarGutter::default(), ScrollbarGutter::Auto);
        assert_eq!(ScrollSnapAlignValue::default(), ScrollSnapAlignValue::None);
        assert_eq!(ScrollSnapType::default(), ScrollSnapType::None);
        assert_eq!(ScrollSnapStop::default(), ScrollSnapStop::Normal);

        let clip_boxes = [
            OverflowClipBox::ContentBox,
            OverflowClipBox::PaddingBox,
            OverflowClipBox::BorderBox,
        ];
        let gutters = [
            ScrollbarGutter::Auto,
            ScrollbarGutter::Stable,
            ScrollbarGutter::StableBothEdges,
        ];
        let axes = [
            ScrollSnapAxis::X,
            ScrollSnapAxis::Y,
            ScrollSnapAxis::Block,
            ScrollSnapAxis::Inline,
            ScrollSnapAxis::Both,
        ];
        let strictnesses = [
            ScrollSnapStrictness::Proximity,
            ScrollSnapStrictness::Mandatory,
        ];
        let alignments = [
            ScrollSnapAlignValue::None,
            ScrollSnapAlignValue::Start,
            ScrollSnapAlignValue::End,
            ScrollSnapAlignValue::Center,
        ];
        let stops = [ScrollSnapStop::Normal, ScrollSnapStop::Always];

        assert_eq!(clip_boxes.len(), 3);
        assert_eq!(gutters.len(), 3);
        assert_eq!(alignments.len(), 4);
        assert_eq!(stops.len(), 2);
        for axis in axes {
            for strictness in strictnesses {
                assert_eq!(
                    ScrollSnapType::Enabled { axis, strictness },
                    ScrollSnapType::Enabled { axis, strictness }
                );
            }
        }
    }

    #[test]
    fn fri05_c01_scroll_input_snap_alignment_keeps_block_and_inline_roles() {
        let alignment =
            ScrollSnapAlign::new(ScrollSnapAlignValue::Start, ScrollSnapAlignValue::End);
        assert_eq!(alignment.block(), ScrollSnapAlignValue::Start);
        assert_eq!(alignment.inline(), ScrollSnapAlignValue::End);

        let default = ScrollSnapAlign::default();
        assert_eq!(default.block(), ScrollSnapAlignValue::None);
        assert_eq!(default.inline(), ScrollSnapAlignValue::None);
    }

    #[test]
    fn fri05_c01_computed_overflow_accepts_exact_canonical_pair_table() {
        let values = [
            Overflow::Visible,
            Overflow::Clip,
            Overflow::Hidden,
            Overflow::Scroll,
            Overflow::Auto,
        ];
        let accepted = [
            [true, true, false, false, false],
            [true, true, false, false, false],
            [false, false, true, true, true],
            [false, false, true, true, true],
            [false, false, true, true, true],
        ];
        let mut accepted_count = 0;
        let mut rejected_count = 0;

        for (x_index, x) in values.into_iter().enumerate() {
            for (y_index, y) in values.into_iter().enumerate() {
                let result = ComputedOverflow::try_new(x, y);
                if accepted[x_index][y_index] {
                    let pair = result.expect("canonical pair must be accepted");
                    assert_eq!((pair.x(), pair.y()), (x, y));
                    accepted_count += 1;
                } else {
                    assert_eq!(
                        result,
                        Err(ComputedOverflowError::NonCanonicalPair { x, y })
                    );
                    rejected_count += 1;
                }
            }
        }

        assert_eq!(accepted_count, 13);
        assert_eq!(rejected_count, 12);
    }

    #[test]
    fn fri05_c01_computed_overflow_visible_default_traits_and_diagnostics_are_exact() {
        fn assert_value_traits<T: Clone + Copy + core::fmt::Debug + Eq + PartialEq>() {}
        fn assert_error_traits<
            T: Clone + Copy + core::fmt::Debug + Eq + PartialEq + std::error::Error,
        >() {
        }

        const VISIBLE_X: Overflow = ComputedOverflow::VISIBLE.x();
        const VISIBLE_Y: Overflow = ComputedOverflow::VISIBLE.y();

        assert_value_traits::<ComputedOverflow>();
        assert_error_traits::<ComputedOverflowError>();
        assert_eq!(ComputedOverflow::default(), ComputedOverflow::VISIBLE);
        assert_eq!(
            (VISIBLE_X, VISIBLE_Y),
            (Overflow::Visible, Overflow::Visible)
        );

        let pair = ComputedOverflow::try_new(Overflow::Clip, Overflow::Visible)
            .expect("visible/clip pair is canonical");
        assert_eq!(
            format!("{pair:?}"),
            "ComputedOverflow { x: Clip, y: Visible }"
        );

        let error = ComputedOverflowError::NonCanonicalPair {
            x: Overflow::Visible,
            y: Overflow::Auto,
        };
        assert_eq!(
            error.to_string(),
            "computed overflow axes must both be visible/clip or both be hidden/scroll/auto"
        );
        assert_eq!(
            format!("{error:?}"),
            "NonCanonicalPair { x: Visible, y: Auto }"
        );
    }

    #[test]
    fn fri05_c01_computed_overflow_scrollability_and_block_pair_predicate_are_exact() {
        for (overflow, expected) in [
            (Overflow::Visible, false),
            (Overflow::Clip, false),
            (Overflow::Hidden, true),
            (Overflow::Scroll, true),
            (Overflow::Auto, true),
        ] {
            assert_eq!(overflow.is_scrollable(), expected);
        }

        for x in [Overflow::Visible, Overflow::Clip] {
            for y in [Overflow::Visible, Overflow::Clip] {
                let pair = ComputedOverflow::try_new(x, y).expect("pair is canonical");
                assert!(!pair.establishes_independent_formatting_context());
            }
        }

        for x in [Overflow::Hidden, Overflow::Scroll, Overflow::Auto] {
            for y in [Overflow::Hidden, Overflow::Scroll, Overflow::Auto] {
                let pair = ComputedOverflow::try_new(x, y).expect("pair is canonical");
                assert!(pair.establishes_independent_formatting_context());
            }
        }
    }

    #[test]
    fn item_order_permutation_is_signed_total_and_stable() {
        let items = [
            (ItemOrder::ZERO, SourceIndex::new(4)),
            (ItemOrder::new(-1), SourceIndex::new(3)),
            (ItemOrder::new(1), SourceIndex::new(2)),
            (ItemOrder::new(-1), SourceIndex::new(1)),
            (ItemOrder::ZERO, SourceIndex::new(0)),
            (ItemOrder::new(i32::MIN), SourceIndex::new(6)),
            (ItemOrder::new(i32::MAX), SourceIndex::new(5)),
        ];
        assert_eq!(
            item_order_permutation(&items),
            [6, 1, 3, 0, 4, 2, 5].map(SourceIndex::new)
        );

        let all_zero = [
            (ItemOrder::ZERO, SourceIndex::new(2)),
            (ItemOrder::default(), SourceIndex::new(0)),
            (ItemOrder::ZERO, SourceIndex::new(1)),
        ];
        assert_eq!(
            item_order_permutation(&all_zero),
            [0, 1, 2].map(SourceIndex::new)
        );
        assert_eq!(item_order_permutation(&[]), Vec::new());
    }
}
