use super::*;

pub(super) type FlexTree<S = Scalar> = OracleTreeOf<S>;
pub(super) fn fri07_c01_composition_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("composition layout must publish node {node}"))
        .output()
}

pub(super) fn fri07_c02_collapse_round_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("collapsed-flex public layout must publish node {node}"))
        .output()
}

pub(super) fn fri07_c02_collapse_round_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(300.0))))
        .expect("collapsed-flex test viewport is finite")
}

pub(super) fn fri07_c02_collapse_round_item<S: LayoutScalar>(
    main: f64,
    cross: f64,
    collapse: FlexItemCollapse,
) -> NodeInputOf<S> {
    NodeInputOf {
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(main)),
            PreferredSizeOf::px(S::from_f64(cross)),
        ),
        flex_item_collapse: collapse,
        flex_grow: FlexGrowOf::ZERO,
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).expect("zero is a valid flex shrink"),
        ..NodeInputOf::default()
    }
}

pub(super) fn assert_fri07_c02_composition_finite_output<S: LayoutScalar>(
    output: NodeOutputOf<S>,
    context: &str,
) {
    let values = [
        output.location.x,
        output.location.y,
        output.size.width,
        output.size.height,
        output.content_size.width,
        output.content_size.height,
        output.border.top,
        output.border.right,
        output.border.bottom,
        output.border.left,
        output.padding.top,
        output.padding.right,
        output.padding.bottom,
        output.padding.left,
        output.margin.top,
        output.margin.right,
        output.margin.bottom,
        output.margin.left,
    ];
    assert!(
        values.into_iter().all(LayoutScalar::is_finite),
        "{context}: every published scalar is finite"
    );
    assert!(
        output.size.width >= S::ZERO
            && output.size.height >= S::ZERO
            && output.content_size.width >= S::ZERO
            && output.content_size.height >= S::ZERO,
        "{context}: published box sizes are non-negative"
    );
    if let Some(geometry) = output.scroll_geometry {
        for (name, rect) in [
            ("border", geometry.border_box()),
            ("padding", geometry.padding_box()),
            ("content", geometry.content_box()),
            ("scrollport", geometry.scrollport()),
            ("overflow", geometry.scrollable_overflow()),
        ] {
            assert!(
                rect.origin().x.is_finite()
                    && rect.origin().y.is_finite()
                    && rect.size().width.is_finite()
                    && rect.size().height.is_finite()
                    && rect.size().width >= S::ZERO
                    && rect.size().height >= S::ZERO,
                "{context}: {name} scroll box is finite and non-negative"
            );
        }
        let range = geometry.physical_range();
        assert!(
            range.x().minimum().is_finite()
                && range.x().maximum().is_finite()
                && range.y().minimum().is_finite()
                && range.y().maximum().is_finite(),
            "{context}: signed scroll range is finite"
        );
    }
}

pub(super) fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

pub(super) fn fri05_c04_flex_all_flow_axes() -> [FlowAxes; 10] {
    [
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
        FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysRl, Direction::Rtl),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr),
        FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl),
    ]
}

pub(super) fn fri05_c04_flex_overflow_at_flow_axes(
    flow_axes: FlowAxes,
    inline: Overflow,
    block: Overflow,
) -> ComputedOverflow {
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => computed_overflow(inline, block),
        PhysicalAxis::Vertical => computed_overflow(block, inline),
    }
}

pub(super) fn fri05_c04_flex_input(size: Size<f32>, flow_axes: FlowAxes) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(flow_axes, ParentFormattingContext::NoParent),
        size.map(Available::definite),
    )
}

pub(super) fn fri05_c04_assert_flow_range(
    geometry: ScrollGeometry,
    flow_axes: FlowAxes,
    inline: (f32, f32),
    block: (f32, f32),
    context: &str,
) {
    let expected = FlowRelativeScrollRange::try_new(inline.0, inline.1, block.0, block.1)
        .expect("expected flow range is ordered");
    assert_eq!(
        geometry.physical_range(),
        flow_axes.physical_scroll_range(expected),
        "{context}"
    );
    assert_eq!(
        flow_axes.flow_relative_scroll_range(geometry.physical_range()),
        expected,
        "{context}"
    );
}
