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
    use crate::{Direction, Position, WritingMode};

    fn assert_container_projection<S: LayoutScalar>() {
        let gap = Size::new(
            LengthOf::px(S::from_f64(7.25)),
            LengthOf::px(S::from_f64(3.5)),
        );
        let input = NodeInputOf::<S> {
            direction: Direction::Rtl,
            writing_mode: WritingMode::VerticalLr,
            position: Position::Relative,
            item_is_replaced: true,
            item_is_table: true,
            gap,
            align_items: Some(AlignItems::LastBaseline),
            align_content: Some(AlignContent::SpaceEvenly),
            justify_content: Some(AlignContent::SafeCenter),
            flex_direction: FlexDirection::ColumnReverse,
            flex_wrap: FlexWrap::WrapReverse,
            ..NodeInputOf::default()
        };

        let projection = FlexContainerProjection::from_node(&input);

        assert_eq!(projection.common, CommonBoxProjection::from_node(&input));
        assert_eq!(projection.gap, &gap);
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
            direction: Direction::Rtl,
            writing_mode: WritingMode::SidewaysLr,
            position: Position::Relative,
            item_is_replaced: true,
            item_is_table: true,
            item_order: ItemOrder::new(-17),
            flex_item_collapse: FlexItemCollapse::Collapsed,
            align_self: Some(AlignItems::Baseline),
            flex_basis: FlexBasisOf::MAX_CONTENT,
            flex_grow,
            flex_shrink,
            ..NodeInputOf::default()
        };

        let projection = FlexItemProjection::from_node(&input);

        assert_eq!(projection.common, CommonBoxProjection::from_node(&input));
        assert_eq!(projection.display, Display::Block);
        assert_eq!(projection.item_order, ItemOrder::new(-17));
        assert_eq!(projection.collapse, FlexItemCollapse::Collapsed);
        assert_eq!(projection.align_self, Some(AlignItems::Baseline));
        assert_eq!(projection.flex_basis, &FlexBasisOf::MAX_CONTENT);
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
