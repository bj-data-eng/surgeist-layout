use super::contracts::{Round, UnroundedInlineFragmentState};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf,
};
use crate::scalar::round_layout_coordinate;
use crate::scroll::rounding::rebuild_rounded_canonical_scroll_geometry;
use crate::{InlineFragmentOutputOf, LayoutScalar, Point, ScrollRectOf, Size, Traverse};

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
    let parent_cumulative_x = cumulative_x;
    let parent_cumulative_y = cumulative_y;
    let cumulative_x = cumulative_x + unrounded.location.x;
    let cumulative_y = cumulative_y + unrounded.location.y;

    layout.location.x = round_layout_coordinate(unrounded.location.x);
    layout.location.y = round_layout_coordinate(unrounded.location.y);
    layout.size.width = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x);
    layout.size.height = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y);
    layout.content_size.width =
        round_layout_coordinate(cumulative_x + unrounded.content_size.width)
            - round_layout_coordinate(cumulative_x);
    layout.content_size.height =
        round_layout_coordinate(cumulative_y + unrounded.content_size.height)
            - round_layout_coordinate(cumulative_y);
    layout.border.left = round_layout_coordinate(cumulative_x + unrounded.border.left)
        - round_layout_coordinate(cumulative_x);
    layout.border.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round_layout_coordinate(cumulative_y + unrounded.border.top)
        - round_layout_coordinate(cumulative_y);
    layout.border.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round_layout_coordinate(cumulative_x + unrounded.padding.left)
        - round_layout_coordinate(cumulative_x);
    layout.padding.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round_layout_coordinate(cumulative_y + unrounded.padding.top)
        - round_layout_coordinate(cumulative_y);
    layout.padding.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.padding.bottom);
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

    let fragment_phases = match tree.unrounded_inline_fragment_state(node) {
        UnroundedInlineFragmentState::Absent => None,
        UnroundedInlineFragmentState::Missing => {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingCachedInlineFragmentState,
                ),
            ));
        }
        UnroundedInlineFragmentState::Present(fragments) => {
            let unrounded_fragments = fragments.to_vec();
            let final_fragments = fragments
                .iter()
                .copied()
                .map(|fragment| {
                    round_inline_fragment(
                        node,
                        fragment,
                        Point::new(parent_cumulative_x, parent_cumulative_y),
                    )
                })
                .collect::<LayoutResultOf<_, Vec<_>, _, _>>()?;
            Some((unrounded_fragments, final_fragments))
        }
    };

    tree.set_final(node, layout);
    if let Some((unrounded_fragments, final_fragments)) = fragment_phases {
        tree.set_final_inline_fragments(node, unrounded_fragments, final_fragments);
    }

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        round_layout_inner(tree, child, cumulative_x, cumulative_y)?;
    }
    Ok(())
}

fn round_inline_fragment<Node, S, M>(
    node: Node,
    fragment: InlineFragmentOutputOf<S>,
    cumulative_origin: Point<S>,
) -> LayoutResultOf<Node, InlineFragmentOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let rect = fragment.rect();
    let origin = rect.origin();
    let size = rect.size();
    let rounded_origin = Point::new(
        round_layout_coordinate(cumulative_origin.x + origin.x)
            - round_layout_coordinate(cumulative_origin.x),
        round_layout_coordinate(cumulative_origin.y + origin.y)
            - round_layout_coordinate(cumulative_origin.y),
    );
    let rounded_end = Point::new(
        round_layout_coordinate(cumulative_origin.x + origin.x + size.width)
            - round_layout_coordinate(cumulative_origin.x),
        round_layout_coordinate(cumulative_origin.y + origin.y + size.height)
            - round_layout_coordinate(cumulative_origin.y),
    );
    let rounded_rect = ScrollRectOf::try_new(
        rounded_origin,
        Size::new(
            (rounded_end.x - rounded_origin.x).max(S::ZERO),
            (rounded_end.y - rounded_origin.y).max(S::ZERO),
        ),
    )
    .map_err(|_| invalid_rounded_inline_fragment_error(node))?;
    let baseline = fragment.baseline();
    let rounded_baseline = Point::new(
        round_layout_coordinate(cumulative_origin.x + baseline.x)
            - round_layout_coordinate(cumulative_origin.x),
        round_layout_coordinate(cumulative_origin.y + baseline.y)
            - round_layout_coordinate(cumulative_origin.y),
    );
    if !rounded_baseline.x.is_finite() || !rounded_baseline.y.is_finite() {
        return Err(invalid_rounded_inline_fragment_error(node));
    }
    Ok(InlineFragmentOutputOf::new(
        fragment.segment_id(),
        rounded_rect,
        rounded_baseline,
        fragment.line_index(),
        fragment.visual_index(),
        fragment.replacement_inline_extent(),
    ))
}

fn invalid_rounded_inline_fragment_error<Node, S, M>(node: Node) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RoundingFinalization,
        LayoutErrorKindOf::InternalInvariant(
            LayoutInternalInvariant::InvalidRoundedInlineFragmentGeometry,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InlineSegmentId;

    fn assert_fri06_c01_fragment_rounding_overflow<S: LayoutScalar>(largest: S) {
        let fragment = InlineFragmentOutputOf::new(
            InlineSegmentId::new(1),
            ScrollRectOf::try_new(Point::new(largest, S::ZERO), Size::ZERO).unwrap(),
            Point::new(largest, S::ZERO),
            0,
            0,
            None,
        );

        let error = round_inline_fragment::<u32, S, ()>(7, fragment, Point::new(largest, S::ZERO))
            .unwrap_err();

        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidRoundedInlineFragmentGeometry,
            )
        );
    }

    #[test]
    fn fri06_c02_rounding_overflow_returns_typed_error_without_panic() {
        assert_fri06_c01_fragment_rounding_overflow(f32::MAX);
        assert_fri06_c01_fragment_rounding_overflow(f64::MAX);
    }

    fn assert_fri06_mr02_layout_round_fragment_boundaries<S: LayoutScalar>() {
        for (value, expected) in [
            (-2.0, -2.0),
            (-1.51, -2.0),
            (-1.5, -1.0),
            (-1.49, -1.0),
            (-0.51, -1.0),
            (-0.5, 0.0),
            (-0.49, 0.0),
            (0.0, 0.0),
            (0.49, 0.0),
            (0.5, 1.0),
            (0.51, 1.0),
            (1.49, 1.0),
            (1.5, 2.0),
            (1.51, 2.0),
            (2.0, 2.0),
            (-1_048_576.5, -1_048_576.0),
            (1_048_576.5, 1_048_577.0),
        ] {
            let value = S::from_f64(value);
            let expected = S::from_f64(expected);
            let fragment = InlineFragmentOutputOf::new(
                InlineSegmentId::new(1),
                ScrollRectOf::try_new(Point::new(value, value), Size::ZERO).unwrap(),
                Point::new(value, value),
                0,
                0,
                None,
            );

            let rounded = round_inline_fragment::<u32, S, ()>(7, fragment, Point::ZERO).unwrap();

            assert_eq!(rounded.rect().origin(), Point::new(expected, expected));
            assert_eq!(rounded.baseline(), Point::new(expected, expected));
        }

        for (value, cumulative, expected) in [
            (0.25, 0.25, 1.0),
            (0.5, -0.25, 0.0),
            (-0.5, -0.25, -1.0),
            (1.49, 10.25, 2.0),
            (-1.49, -10.25, -2.0),
        ] {
            let value = S::from_f64(value);
            let cumulative = S::from_f64(cumulative);
            let expected = S::from_f64(expected);
            let fragment = InlineFragmentOutputOf::new(
                InlineSegmentId::new(1),
                ScrollRectOf::try_new(Point::new(value, value), Size::ZERO).unwrap(),
                Point::new(value, value),
                0,
                0,
                None,
            );

            let rounded = round_inline_fragment::<u32, S, ()>(
                7,
                fragment,
                Point::new(cumulative, cumulative),
            )
            .unwrap();

            assert_eq!(rounded.rect().origin(), Point::new(expected, expected));
            assert_eq!(rounded.baseline(), Point::new(expected, expected));
        }
    }

    #[test]
    fn fri06_mr02_layout_round_fragments_baselines_and_cumulative_origins_are_preserved() {
        assert_fri06_mr02_layout_round_fragment_boundaries::<f32>();
        assert_fri06_mr02_layout_round_fragment_boundaries::<f64>();
    }

    #[test]
    fn fri06_mr02_layout_round_fragment_overflow_preserves_typed_error() {
        assert_fri06_c01_fragment_rounding_overflow(f32::MAX);
        assert_fri06_c01_fragment_rounding_overflow(f64::MAX);
    }
}
