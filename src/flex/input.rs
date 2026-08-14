use crate::node_projection::CommonBoxProjection;
use crate::scroll::{ScrollBoxProjection, ScrollTargetProjection};
use crate::{
    AlignContent, AlignItems, Compute, Display, FlexBasisOf, FlexDirection, FlexGrowOf,
    FlexItemCollapse, FlexShrinkOf, FlexWrap, ItemOrder, LayoutScalar, LengthOf, NodeInputOf, Size,
    SourceIndex, Traverse,
};

/// Flex-container facts settled at the flex algorithm entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FlexContainerProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) gap: &'a Size<LengthOf<S>>,
    pub(super) align_items: Option<AlignItems>,
    pub(super) align_content: Option<AlignContent>,
    pub(super) justify_content: Option<AlignContent>,
    pub(super) flex_direction: FlexDirection,
    pub(super) flex_wrap: FlexWrap,
}

impl<'a, S: LayoutScalar> FlexContainerProjection<'a, S> {
    #[must_use]
    fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            gap: &input.gap,
            align_items: input.align_items,
            align_content: input.align_content,
            justify_content: input.justify_content,
            flex_direction: input.flex_direction,
            flex_wrap: input.flex_wrap,
        }
    }
}

/// Flex-item facts retained once a child has a settled flex role.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FlexItemProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) display: Display,
    pub(super) item_order: ItemOrder,
    pub(super) collapse: FlexItemCollapse,
    pub(super) align_self: Option<AlignItems>,
    pub(super) flex_basis: &'a FlexBasisOf<S>,
    pub(super) flex_grow: FlexGrowOf<S>,
    pub(super) flex_shrink: FlexShrinkOf<S>,
}

impl<'a, S: LayoutScalar> FlexItemProjection<'a, S> {
    #[must_use]
    fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            display: input.display,
            item_order: input.item_order,
            collapse: input.flex_item_collapse,
            align_self: input.align_self,
            flex_basis: &input.flex_basis,
            flex_grow: input.flex_grow,
            flex_shrink: input.flex_shrink,
        }
    }
}

pub(super) fn with_flex_container_projection<Tree, M, Output>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    consume: impl FnOnce(FlexContainerProjection<'_, Tree::Scalar>) -> Output,
) -> Output
where
    Tree: Compute<M>,
{
    consume(FlexContainerProjection::from_node(tree.node_input(node)))
}

pub(super) fn with_flex_item_projection<Tree, M, Output>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    consume: impl FnOnce(&mut Tree, FlexItemProjection<'_, Tree::Scalar>) -> Output,
) -> Output
where
    Tree: Compute<M>,
{
    let complete_input = tree.node_input(node).clone();
    consume(tree, FlexItemProjection::from_node(&complete_input))
}

pub(super) fn with_flex_scroll_projections<Tree, M, Output>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    consume: impl FnOnce(
        ScrollBoxProjection<'_, Tree::Scalar>,
        ScrollTargetProjection<'_, Tree::Scalar>,
    ) -> Output,
) -> Output
where
    Tree: Compute<M>,
{
    let complete_input = tree.node_input(node);
    consume(
        ScrollBoxProjection::from_node(complete_input),
        ScrollTargetProjection::from_node(complete_input),
    )
}

pub(super) fn with_flex_item_scroll_projections<Tree, M, Output>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    consume: impl FnOnce(
        ScrollBoxProjection<'_, Tree::Scalar>,
        ScrollTargetProjection<'_, Tree::Scalar>,
    ) -> Output,
) -> Output
where
    Tree: Compute<M>,
{
    with_flex_scroll_projections::<Tree, M, Output>(tree, node, consume)
}

#[must_use]
pub(super) fn flex_item_order_permutation(items: &[(ItemOrder, SourceIndex)]) -> Vec<SourceIndex> {
    crate::node_input::item_order_permutation(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AspectRatioOf, BoxSizing, ComputedOverflow, Direction, Edges, FlowAxes, LengthAutoOf,
        LengthPercentageOf, MaxSizeOf, MinSizeOf, Overflow, PhysicalAxis, PhysicalSide, Position,
        PreferredSizeOf, WritingMode,
    };

    fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    fn length_percentage<S: LayoutScalar>(value: f64) -> LengthPercentageOf<S> {
        LengthPercentageOf::px(scalar(value)).expect("finite test length")
    }

    fn length<S: LayoutScalar>(value: f64) -> LengthOf<S> {
        LengthOf::value(length_percentage(value))
    }

    fn length_auto<S: LayoutScalar>(value: f64) -> LengthAutoOf<S> {
        LengthAutoOf::value(length_percentage(value))
    }

    fn non_default_common_input<S: LayoutScalar>() -> NodeInputOf<S> {
        NodeInputOf {
            size: Size::new(
                PreferredSizeOf::value(length_percentage(11.0)),
                PreferredSizeOf::value(length_percentage(12.0)),
            ),
            min_size: Size::new(
                MinSizeOf::value(length_percentage(13.0)),
                MinSizeOf::value(length_percentage(14.0)),
            ),
            max_size: Size::new(
                MaxSizeOf::value(length_percentage(101.0)),
                MaxSizeOf::value(length_percentage(102.0)),
            ),
            aspect_ratio: Some(
                AspectRatioOf::new(scalar(1.25)).expect("positive finite aspect ratio"),
            ),
            margin: Edges::new(
                length_auto(1.0),
                length_auto(2.0),
                length_auto(3.0),
                length_auto(4.0),
            ),
            padding: Edges::new(length(5.0), length(6.0), length(7.0), length(8.0)),
            border: Edges::new(length(9.0), length(10.0), length(15.0), length(16.0)),
            box_sizing: BoxSizing::ContentBox,
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Rtl,
            overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Auto)
                .expect("canonical scrollable overflow pair"),
            position: Position::Absolute,
            inset: Edges::new(
                length_auto(21.0),
                length_auto(22.0),
                length_auto(23.0),
                length_auto(24.0),
            ),
            item_is_replaced: true,
            item_is_table: true,
            ..NodeInputOf::default()
        }
    }

    fn assert_common_projection<S: LayoutScalar>(
        common: CommonBoxProjection<'_, S>,
        input: &NodeInputOf<S>,
    ) {
        let expected_size = Size::new(
            PreferredSizeOf::value(length_percentage(11.0)),
            PreferredSizeOf::value(length_percentage(12.0)),
        );
        let expected_min_size = Size::new(
            MinSizeOf::value(length_percentage(13.0)),
            MinSizeOf::value(length_percentage(14.0)),
        );
        let expected_max_size = Size::new(
            MaxSizeOf::value(length_percentage(101.0)),
            MaxSizeOf::value(length_percentage(102.0)),
        );
        let expected_aspect_ratio =
            Some(AspectRatioOf::new(scalar(1.25)).expect("positive finite aspect ratio"));
        let expected_margin = Edges::new(
            length_auto(1.0),
            length_auto(2.0),
            length_auto(3.0),
            length_auto(4.0),
        );
        let expected_padding = Edges::new(length(5.0), length(6.0), length(7.0), length(8.0));
        let expected_border = Edges::new(length(9.0), length(10.0), length(15.0), length(16.0));
        let expected_overflow = ComputedOverflow::try_new(Overflow::Hidden, Overflow::Auto)
            .expect("canonical scrollable overflow pair");
        let expected_inset = Edges::new(
            length_auto(21.0),
            length_auto(22.0),
            length_auto(23.0),
            length_auto(24.0),
        );

        assert_eq!(*common.size, expected_size);
        assert_eq!(*common.min_size, expected_min_size);
        assert_eq!(*common.max_size, expected_max_size);
        assert_eq!(*common.aspect_ratio, expected_aspect_ratio);
        assert_eq!(*common.margin, expected_margin);
        assert_eq!(*common.padding, expected_padding);
        assert_eq!(*common.border, expected_border);
        assert_eq!(common.box_sizing, BoxSizing::ContentBox);
        assert_eq!(
            common.flow_axes,
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl)
        );
        assert_eq!(common.flow_axes.writing_mode(), WritingMode::SidewaysLr);
        assert_eq!(common.flow_axes.direction(), Direction::Rtl);
        assert_eq!(common.flow_axes.inline_axis(), PhysicalAxis::Vertical);
        assert_eq!(common.flow_axes.block_axis(), PhysicalAxis::Horizontal);
        assert_eq!(common.flow_axes.inline_start(), PhysicalSide::Top);
        assert_eq!(common.flow_axes.inline_end(), PhysicalSide::Bottom);
        assert_eq!(common.flow_axes.block_start(), PhysicalSide::Left);
        assert_eq!(common.flow_axes.block_end(), PhysicalSide::Right);
        assert_eq!(common.flow_axes.line_over(), PhysicalSide::Left);
        assert_eq!(common.flow_axes.line_under(), PhysicalSide::Right);
        assert_eq!(common.overflow, expected_overflow);
        assert_eq!(common.position, Position::Absolute);
        assert_eq!(*common.inset, expected_inset);
        assert!(common.item_is_replaced);
        assert!(common.item_is_table);

        assert!(core::ptr::eq(common.size, &input.size));
        assert!(core::ptr::eq(common.min_size, &input.min_size));
        assert!(core::ptr::eq(common.max_size, &input.max_size));
        assert!(core::ptr::eq(common.aspect_ratio, &input.aspect_ratio));
        assert!(core::ptr::eq(common.margin, &input.margin));
        assert!(core::ptr::eq(common.padding, &input.padding));
        assert!(core::ptr::eq(common.border, &input.border));
        assert!(core::ptr::eq(common.inset, &input.inset));
    }

    fn assert_container_projection<S: LayoutScalar>() {
        let gap = Size::new(
            LengthOf::px(S::from_f64(7.25)),
            LengthOf::px(S::from_f64(3.5)),
        );
        let input = NodeInputOf::<S> {
            gap,
            align_items: Some(AlignItems::LastBaseline),
            align_content: Some(AlignContent::SpaceEvenly),
            justify_content: Some(AlignContent::SafeCenter),
            flex_direction: FlexDirection::ColumnReverse,
            flex_wrap: FlexWrap::WrapReverse,
            ..non_default_common_input()
        };

        let projection = FlexContainerProjection::from_node(&input);

        assert_common_projection(projection.common, &input);
        assert_eq!(*projection.gap, gap);
        assert!(core::ptr::eq(projection.gap, &input.gap));
        assert_eq!(projection.align_items, Some(AlignItems::LastBaseline));
        assert_eq!(projection.align_content, Some(AlignContent::SpaceEvenly));
        assert_eq!(projection.justify_content, Some(AlignContent::SafeCenter));
        assert_eq!(projection.flex_direction, FlexDirection::ColumnReverse);
        assert_eq!(projection.flex_wrap, FlexWrap::WrapReverse);
    }

    fn assert_item_projection<S: LayoutScalar>() {
        let flex_grow = FlexGrowOf::try_new(S::from_f64(2.25)).expect("finite flex grow");
        let flex_shrink = FlexShrinkOf::try_new(S::from_f64(3.5)).expect("finite flex shrink");
        let input = NodeInputOf::<S> {
            display: Display::Block,
            item_order: ItemOrder::new(-17),
            flex_item_collapse: FlexItemCollapse::Collapsed,
            align_self: Some(AlignItems::Baseline),
            flex_basis: FlexBasisOf::MAX_CONTENT,
            flex_grow,
            flex_shrink,
            ..non_default_common_input()
        };

        let projection = FlexItemProjection::from_node(&input);

        assert_common_projection(projection.common, &input);
        assert_eq!(projection.display, Display::Block);
        assert_eq!(projection.item_order, ItemOrder::new(-17));
        assert_eq!(projection.collapse, FlexItemCollapse::Collapsed);
        assert_eq!(projection.align_self, Some(AlignItems::Baseline));
        assert_eq!(*projection.flex_basis, FlexBasisOf::MAX_CONTENT);
        assert!(core::ptr::eq(projection.flex_basis, &input.flex_basis));
        assert_eq!(projection.flex_grow, flex_grow);
        assert_eq!(projection.flex_shrink, flex_shrink);
    }

    #[test]
    fn node_projection_flex_container_selects_common_and_role_facts_in_both_scalar_lanes() {
        assert_container_projection::<f32>();
        assert_container_projection::<f64>();
    }

    #[test]
    fn node_projection_flex_item_selects_common_and_role_facts_in_both_scalar_lanes() {
        assert_item_projection::<f32>();
        assert_item_projection::<f64>();
    }
}
