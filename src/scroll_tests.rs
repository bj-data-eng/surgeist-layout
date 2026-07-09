use crate::{
    Direction, Overflow, Point, ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry,
    ScrollOffset, ScrollOffsetOf, ScrollOverflowCouplingPolicy, ScrollOverflowExposure,
    ScrollRange, ScrollRangeOf, ScrollRect, ScrollUnsupportedFeature, ScrollbarGutterRects, Size,
    WritingMode,
};

#[test]
fn scroll_range_clamps_offsets_to_non_negative_maximum() {
    let range = ScrollRange::new(Size::new(120.0, 40.0)).unwrap();

    assert_eq!(
        range.clamp(ScrollOffset::new(Point::new(-10.0, 10.0))),
        ScrollOffset::new(Point::new(0.0, 10.0))
    );
    assert_eq!(
        range.clamp(ScrollOffset::new(Point::new(200.0, 99.0))),
        ScrollOffset::new(Point::new(120.0, 40.0))
    );
}

#[test]
fn scroll_range_rejects_negative_or_non_finite_maximum() {
    assert_eq!(
        ScrollRange::new(Size::new(-1.0, 0.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRange
    );
    assert_eq!(
        ScrollRange::new(Size::new(f32::INFINITY, 0.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRange
    );
}

#[test]
fn scroll_rect_rejects_negative_or_non_finite_size() {
    assert_eq!(
        ScrollRect::new(Point::ZERO, Size::new(10.0, -1.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRect
    );
    assert_eq!(
        ScrollRect::new(Point::new(f32::NAN, 0.0), Size::new(10.0, 1.0)).unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollRect
    );
}

#[test]
fn scroll_geometry_supports_f64() {
    let range = ScrollRangeOf::<f64>::new(Size::new(1_000_000_000_000.0, 0.5)).unwrap();

    assert_eq!(
        range.clamp(ScrollOffsetOf::<f64>::new(Point::new(
            2_000_000_000_000.0,
            1.0
        ))),
        ScrollOffsetOf::<f64>::new(Point::new(1_000_000_000_000.0, 0.5))
    );
}

#[test]
fn scroll_container_facts_distinguish_hidden_clip_and_scroll() {
    let hidden = ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap();
    let clip = ScrollContainerAxis::from_overflow(Overflow::Clip).unwrap();
    let scroll = ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap();
    let visible = ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap();

    assert_eq!(hidden.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(hidden.exposes_scroll_range());
    assert_eq!(clip.exposure(), ScrollOverflowExposure::ClipOnly);
    assert!(!clip.exposes_scroll_range());
    assert_eq!(scroll.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(scroll.exposes_scroll_range());
    assert_eq!(visible.exposure(), ScrollOverflowExposure::Visible);
    assert!(!visible.exposes_scroll_range());
}

#[test]
fn scroll_geometry_front_door_preserves_physical_rects_and_flow_metadata() {
    let scrollport = ScrollRect::new(Point::new(1.0, 2.0), Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let clip = ScrollRect::new(Point::new(1.0, 2.0), Size::new(80.0, 40.0)).unwrap();
    let range = ScrollRange::new(Size::new(40.0, 50.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);
    let geometry = ScrollGeometry::new(
        WritingMode::VerticalRl,
        Direction::Rtl,
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
        ),
        scrollport,
        Some(clip),
        overflow,
        range,
        gutters,
    )
    .unwrap();

    assert_eq!(geometry.writing_mode(), WritingMode::VerticalRl);
    assert_eq!(geometry.direction(), Direction::Rtl);
    assert_eq!(geometry.scrollport(), scrollport);
    assert_eq!(geometry.overflow_clip(), Some(clip));
    assert_eq!(geometry.scrollable_overflow(), overflow);
    assert_eq!(geometry.range(), range);
}

#[test]
fn scroll_geometry_rejects_clip_only_axis_with_non_zero_range() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = ScrollRange::new(Size::new(40.0, 0.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);

    assert_eq!(
        ScrollGeometry::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Clip).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ),
            scrollport,
            Some(scrollport),
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn scroll_geometry_rejects_visible_axis_with_non_zero_range() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = ScrollRange::new(Size::new(0.0, 50.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);

    assert_eq!(
        ScrollGeometry::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Scroll).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ),
            scrollport,
            None,
            overflow,
            range,
            gutters,
        )
        .unwrap_err(),
        ScrollUnsupportedFeature::InvalidScrollGeometry
    );
}

#[test]
fn phase_one_mixed_axis_boundary_is_root_pre_resolved() {
    assert_eq!(
        ScrollOverflowCouplingPolicy::PHASE_ONE,
        ScrollOverflowCouplingPolicy::RootPreResolved
    );
    assert_eq!(
        ScrollOverflowCouplingPolicy::LayoutOwnedVisibleToAutoCoupling.unsupported_feature(),
        Some(ScrollUnsupportedFeature::LayoutOwnedMixedAxisOverflowCoupling)
    );
}
