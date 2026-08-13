use super::LayoutRootRequestOf;
#[cfg(test)]
use super::{AvailableOf, Edges};
#[cfg(test)]
use super::{ComputeInputOf, LayoutScalar, NodeInputOf, Size};
use crate::engine;
use crate::error::LayoutResultOf;
#[cfg(test)]
use crate::error::{LayoutErrorKindOf, LayoutErrorSiteOf, LayoutOperation};
#[cfg(test)]
use crate::geometry::FlowAxes;
use crate::{CompletedLayoutBatchOf, LayoutTree};

#[cfg(test)]
use crate::AspectRatioOf;
#[cfg(test)]
use crate::layout_math::{
    MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt, OptionalMinimumSizeFloorExt,
    OptionalSizeExt,
};
#[cfg(test)]
use crate::measurement::compute_leaf;

type CompletedTreeBatch<Tree> =
    CompletedLayoutBatchOf<<Tree as super::Traverse>::Node, <Tree as super::Traverse>::Scalar>;

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
    compute_layout_invalidated(tree, root, request, &[])
}

pub fn compute_layout_invalidated<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    changed_nodes: &[Tree::Node],
) -> LayoutResultOf<Tree::Node, CompletedTreeBatch<Tree>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    let invalidated_nodes = engine::validate_layout_request(tree, root, changed_nodes)?;

    engine::compute_validated_layout(tree, root, request, invalidated_nodes)
}

#[cfg(test)]
mod fri08_c07_t03_optional_math_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)),)),
            Size::new(scalar(9.0), scalar(12.0))
        );
    }

    #[test]
    fn fri08_c07_t03_optional_math_compute_minimum_floor_preserves_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_compute_minimum_floor_preserves_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let optional = Size::new(None, Some(scalar(9.0)));

        assert_eq!(
            optional.or(Size::new(Some(scalar(4.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(4.0)), Some(scalar(9.0)))
        );
        assert_eq!(
            optional.unwrap_or(Size::new(scalar(6.0), scalar(7.0))),
            Size::new(scalar(6.0), scalar(9.0))
        );
        assert_eq!(
            optional.add_optional(Size::new(scalar(2.0), scalar(3.0))),
            Size::new(None, Some(scalar(12.0)))
        );

        let Some(ratio) = AspectRatioOf::new(scalar(2.0)) else {
            panic!("finite positive test aspect ratio must be accepted");
        };
        assert_eq!(
            Size::new(Some(scalar(12.0)), None).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(12.0)), Some(scalar(6.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(7.0))).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(14.0)), Some(scalar(7.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            scalar(5.0).clamp_max_before_min_optional(Some(scalar(10.0)), Some(scalar(3.0))),
            scalar(10.0)
        );
    }

    #[test]
    fn fri06_c13_t05_compute_optional_math_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_compute_optional_math_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fri06_c13_t06_leaf_missing_basis_counterexample<S: LayoutScalar>() {
        let style = NodeInputOf::<S> {
            padding: Edges::all(super::super::LengthOf::percent(S::from_f64(0.5))),
            ..NodeInputOf::default()
        };
        let context = super::super::ContainingLayoutContext::new(
            FlowAxes::new(
                super::super::WritingMode::HorizontalTb,
                super::super::Direction::Ltr,
            ),
            super::super::ParentFormattingContext::NoParent,
        );
        let available = Size::splat(AvailableOf::MAX_CONTENT);
        let measured = Size::new(S::from_f64(12.0), S::from_f64(8.0));

        let content_size_input =
            ComputeInputOf::leaf_content_size(Size::NONE, Size::NONE, context, available)
                .expect("indefinite intrinsic leaf input is valid");
        let content_size = compute_leaf(content_size_input, &style, |_input| Ok::<_, ()>(measured))
            .expect("ComputeSize explicitly treats missing edge basis as zero");
        assert_eq!(content_size.size, measured);

        let layout_input = ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, context, available)
            .expect("indefinite layout leaf input is valid");
        let error = compute_leaf(layout_input, &style, |_input| -> Result<Size<S>, ()> {
            panic!("layout missing-basis failure must precede measurement")
        })
        .expect_err("layout keeps missing edge basis fallible");
        assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::MissingContext(super::super::LayoutMissingContext::RequiredBasis,)
        );
    }

    #[test]
    fn fri06_c13_t06_leaf_missing_basis_compute_size_and_layout_differ_f32() {
        assert_fri06_c13_t06_leaf_missing_basis_counterexample::<f32>();
    }

    #[test]
    fn fri06_c13_t06_leaf_missing_basis_compute_size_and_layout_differ_f64() {
        assert_fri06_c13_t06_leaf_missing_basis_counterexample::<f64>();
    }
}
