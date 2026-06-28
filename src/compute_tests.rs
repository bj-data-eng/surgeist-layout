use crate::compute::compute_leaf_with_resolver;
use crate::*;
use crate::{
    Available, CalcExpression, CalcTerm, ComputeInput, Dimension, LayoutCalcStore, NodeInput,
    RequestedAxis,
};

#[test]
fn leaf_calc_width_uses_tree_resolver() {
    let mut store = LayoutCalcStore::new();
    let width = store.push(CalcExpression::sum([
        CalcTerm::percent(0.5),
        CalcTerm::px(10.0),
    ]));
    let style = NodeInput {
        size: Size::new(Dimension::calc(width), Dimension::AUTO),
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

    let output = compute_leaf_with_resolver(input, &style, &store, |_known, _available| {
        Size::new(12.0, 8.0)
    });

    assert_eq!(output.size.width, 60.0);
}

#[test]
#[should_panic(expected = "calc resolution requires an explicit resolver")]
fn public_leaf_calc_width_requires_explicit_resolver() {
    let mut store = LayoutCalcStore::new();
    let width = store.push(CalcExpression::sum([CalcTerm::px(10.0)]));
    let style = NodeInput {
        size: Size::new(Dimension::calc(width), Dimension::AUTO),
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

    let _ = compute_leaf(input, &style, |_known, _available| Size::new(12.0, 8.0));
}
