use crate::*;
use crate::{Available, ComputeInput, Dimension, LengthPercentageOf, NodeInput, RequestedAxis};

#[test]
fn leaf_affine_width_resolves_against_parent_basis() {
    let width = LengthPercentageOf::from_coefficients(10.0, 0.5).expect("finite coefficients");
    let style = NodeInput {
        size: Size::new(Dimension::value(width), Dimension::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(100.0), None),
        available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    };

    let output = compute_leaf(input, &style, |_known, _available| Size::new(12.0, 8.0));

    assert_eq!(output.size.width, 60.0);
}

#[test]
fn public_leaf_affine_px_width_needs_no_resolver() {
    let width = LengthPercentageOf::px(10.0).expect("finite px");
    let style = NodeInput {
        size: Size::new(Dimension::value(width), Dimension::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(100.0), None),
        available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    };

    let output = compute_leaf(input, &style, |_known, _available| Size::new(12.0, 8.0));

    assert_eq!(output.size.width, 10.0);
}
