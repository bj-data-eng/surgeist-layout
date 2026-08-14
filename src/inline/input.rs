use crate::block::BlockChildProjection;
use crate::{
    Compute, InlineBoundaryInputOf, InlineTextInputOf, LayoutInputOf, LayoutScalar,
    LineBreakInputOf, Traverse,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InlineParticipantKindOf<'a, S: LayoutScalar> {
    Box(Box<BlockChildProjection<'a, S>>),
    InlineText(InlineTextInputOf<S>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}

/// One settled box, text, break, or boundary participant in an inline context.
///
/// The owned lookup result anchors the borrowed common-box facts exposed by
/// [`Self::kind`] without copying them into another projection.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InlineParticipantProjection<S: LayoutScalar> {
    input: LayoutInputOf<S>,
}

impl<S: LayoutScalar> InlineParticipantProjection<S> {
    #[must_use]
    pub(crate) fn lookup<Tree, M>(tree: &Tree, node: <Tree as Traverse>::Node) -> Self
    where
        Tree: Compute<M, Scalar = S>,
    {
        Self {
            input: tree.layout_input(node),
        }
    }

    #[must_use]
    pub(crate) fn kind(&self) -> InlineParticipantKindOf<'_, S> {
        match &self.input {
            LayoutInputOf::Box(input) => {
                InlineParticipantKindOf::Box(Box::new(BlockChildProjection::from_node(input)))
            }
            LayoutInputOf::InlineText(input) => InlineParticipantKindOf::InlineText(input.clone()),
            LayoutInputOf::LineBreak(input) => InlineParticipantKindOf::LineBreak(*input),
            LayoutInputOf::InlineBoundary(input) => InlineParticipantKindOf::InlineBoundary(*input),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;
    use crate::{
        BidiLevel, Clear, ComputeInputOf, ComputeOutputOf, Direction, InlineBoundaryKind,
        InlineBreakOpportunityOf, InlineMetricsOf, InlineSegmentId, InlineWhitespaceEdge,
        LayoutResultOf, NodeInputOf, NodeOutputOf, ShapedInlineSegmentOf, VerticalAlign,
        WritingMode,
    };

    struct ProjectionTree<S: LayoutScalar> {
        node_input: NodeInputOf<S>,
        layout_input: LayoutInputOf<S>,
    }

    impl<S: LayoutScalar> Traverse for ProjectionTree<S> {
        type Node = ();
        type Scalar = S;
        type Children<'a>
            = core::iter::Empty<()>
        where
            Self: 'a;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            core::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            panic!("projection test tree has no children")
        }
    }

    impl<S: LayoutScalar> Compute<Infallible> for ProjectionTree<S> {
        fn node_input(&self, _node: Self::Node) -> &NodeInputOf<S> {
            &self.node_input
        }

        fn layout_input(&self, _node: Self::Node) -> LayoutInputOf<S> {
            self.layout_input.clone()
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutputOf<S>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInputOf<S>,
        ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S, Infallible> {
            panic!("projection lookup does not compute children")
        }
    }

    fn lookup<S: LayoutScalar>(input: LayoutInputOf<S>) -> InlineParticipantProjection<S> {
        InlineParticipantProjection::lookup::<_, Infallible>(
            &ProjectionTree {
                node_input: NodeInputOf::default(),
                layout_input: input,
            },
            (),
        )
    }

    fn assert_non_box_projection_values<S: LayoutScalar>() {
        let text_metrics = InlineMetricsOf::try_new(S::from_f64(7.0), S::from_f64(11.0))
            .expect("valid non-default text metrics");
        let text_segment = ShapedInlineSegmentOf::try_new(
            InlineSegmentId::new(17),
            S::from_f64(23.0),
            text_metrics,
            BidiLevel::try_new(5).expect("valid bidi level"),
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(2.0))
                .expect("valid replacement break"),
        )
        .expect("valid shaped inline segment");
        let text = InlineTextInputOf::try_new(vec![text_segment])
            .expect("nonempty inline text with unique segment identity");
        let text_projection = lookup(LayoutInputOf::inline_text(text.clone()));
        let InlineParticipantKindOf::InlineText(projected_text) = text_projection.kind() else {
            panic!("inline text input must retain its participant kind");
        };
        assert_eq!(projected_text, text);
        assert_eq!(projected_text.segments(), &[text_segment]);

        let break_metrics = InlineMetricsOf::try_new(S::from_f64(13.0), S::from_f64(19.0))
            .expect("valid non-default line-break metrics");
        let line_break = LineBreakInputOf::new()
            .with_metrics(break_metrics)
            .hidden()
            .with_direction(Direction::Rtl)
            .with_writing_mode(WritingMode::VerticalLr)
            .with_vertical_align(VerticalAlign::Top)
            .with_clear(Clear::Right);
        let line_break_projection = lookup(LayoutInputOf::line_break(line_break));
        let InlineParticipantKindOf::LineBreak(projected_break) = line_break_projection.kind()
        else {
            panic!("line-break input must retain its participant kind");
        };
        assert_eq!(projected_break, line_break);
        assert_eq!(projected_break.metrics(), break_metrics);
        assert_eq!(projected_break.direction(), Direction::Rtl);
        assert_eq!(projected_break.writing_mode(), WritingMode::VerticalLr);
        assert_eq!(projected_break.vertical_align(), VerticalAlign::Top);
        assert_eq!(projected_break.clear(), Clear::Right);

        let boundary_metrics = InlineMetricsOf::try_new(S::from_f64(29.0), S::from_f64(31.0))
            .expect("valid non-default inline-boundary metrics");
        let boundary = InlineBoundaryInputOf::new(InlineBoundaryKind::End, boundary_metrics)
            .with_writing_mode(WritingMode::SidewaysRl)
            .with_direction(Direction::Rtl)
            .with_vertical_align(VerticalAlign::Bottom);
        let boundary_projection = lookup(LayoutInputOf::inline_boundary(boundary));
        let InlineParticipantKindOf::InlineBoundary(projected_boundary) =
            boundary_projection.kind()
        else {
            panic!("inline-boundary input must retain its participant kind");
        };
        assert_eq!(projected_boundary, boundary);
        assert_eq!(projected_boundary.kind(), InlineBoundaryKind::End);
        assert_eq!(projected_boundary.metrics(), boundary_metrics);
        assert_eq!(projected_boundary.writing_mode(), WritingMode::SidewaysRl);
        assert_eq!(projected_boundary.direction(), Direction::Rtl);
        assert_eq!(projected_boundary.vertical_align(), VerticalAlign::Bottom);
    }

    #[test]
    fn node_projection_block_inline_non_box_f32_retains_each_participant_value() {
        assert_non_box_projection_values::<f32>();
    }

    #[test]
    fn node_projection_block_inline_non_box_f64_retains_each_participant_value() {
        assert_non_box_projection_values::<f64>();
    }
}
