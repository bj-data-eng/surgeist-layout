use crate::{
    AvailableOf, ComputeInputOf, ContainingLayoutContext, Direction, FlowAxes, LayoutErrorKindOf,
    LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation, LayoutScalar,
    LengthPercentageOf, NumericResolutionOf, ParentFormattingContext, PercentageBasisOf,
    RequestedAxis, RunMode, ScrollPaddingOf, ScrollPaddingValueOf, Size, SizingMode, WritingMode,
};

pub(crate) fn geometry_error_largest_finite<S: LayoutScalar>() -> S {
    if core::mem::size_of::<S>() == core::mem::size_of::<f32>() {
        S::from_f64(f32::MAX.into())
    } else {
        S::from_f64(f64::MAX)
    }
}

pub(crate) fn geometry_error_input<S: LayoutScalar>(run_mode: RunMode) -> ComputeInputOf<S> {
    let largest = geometry_error_largest_finite::<S>();
    ComputeInputOf::for_child(
        run_mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(largest)),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::splat(AvailableOf::definite(largest)),
    )
}

pub(crate) fn assert_geometry_error<S: LayoutScalar, M>(
    error: LayoutErrorOf<u32, S, M>,
    site: LayoutErrorSiteOf<u32>,
    operation: LayoutOperation,
    invariant: LayoutInternalInvariant,
) {
    assert_eq!(error.site(), site);
    assert_eq!(error.operation(), operation);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InternalInvariant(actual) if *actual == invariant
    ));
}

pub(crate) fn scroll_padding_inputs<S: LayoutScalar>() -> [ScrollPaddingOf<S>; 2] {
    let value = |value| {
        ScrollPaddingValueOf::value(
            LengthPercentageOf::px(S::from_f64(value))
                .unwrap_or_else(|_| panic!("test scroll padding must be finite")),
        )
    };

    [
        ScrollPaddingOf::new(
            value(11.0),
            ScrollPaddingValueOf::AUTO,
            value(33.0),
            ScrollPaddingValueOf::AUTO,
        ),
        ScrollPaddingOf::new(
            ScrollPaddingValueOf::AUTO,
            value(22.0),
            ScrollPaddingValueOf::AUTO,
            value(44.0),
        ),
    ]
}

pub(crate) fn assert_scroll_padding_inputs_exact<S: LayoutScalar>() {
    let [first, second] = scroll_padding_inputs::<S>();
    let basis = PercentageBasisOf::definite(S::from_f64(100.0))
        .unwrap_or_else(|_| panic!("test scroll-padding basis must be finite"));
    let resolved = |value: ScrollPaddingValueOf<S>| match value.resolve_against(basis) {
        NumericResolutionOf::Resolved(value) => value,
        status => panic!("test scroll-padding value must resolve exactly: {status:?}"),
    };

    assert!(!first.top().is_auto());
    assert!(first.right().is_auto());
    assert!(!first.bottom().is_auto());
    assert!(first.left().is_auto());
    assert_eq!(resolved(first.top()), S::from_f64(11.0));
    assert_eq!(resolved(first.right()), S::ZERO);
    assert_eq!(resolved(first.bottom()), S::from_f64(33.0));
    assert_eq!(resolved(first.left()), S::ZERO);

    assert!(second.top().is_auto());
    assert!(!second.right().is_auto());
    assert!(second.bottom().is_auto());
    assert!(!second.left().is_auto());
    assert_eq!(resolved(second.top()), S::ZERO);
    assert_eq!(resolved(second.right()), S::from_f64(22.0));
    assert_eq!(resolved(second.bottom()), S::ZERO);
    assert_eq!(resolved(second.left()), S::from_f64(44.0));
}
