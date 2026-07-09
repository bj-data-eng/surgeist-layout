use crate::scroll::{ScrollBoxRects, ScrollbarReservation};
use crate::{
    Direction, Edges, Overflow, Point, ScrollContainerAxis, ScrollContainerFacts, ScrollGeometry,
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
    assert!(hidden.clips_overflow());
    assert_eq!(clip.exposure(), ScrollOverflowExposure::ClipOnly);
    assert!(!clip.exposes_scroll_range());
    assert!(clip.clips_overflow());
    assert_eq!(scroll.exposure(), ScrollOverflowExposure::ScrollableClip);
    assert!(scroll.exposes_scroll_range());
    assert!(scroll.clips_overflow());
    assert_eq!(visible.exposure(), ScrollOverflowExposure::Visible);
    assert!(!visible.exposes_scroll_range());
    assert!(!visible.clips_overflow());
    assert!(ScrollContainerFacts::new(hidden, visible).requires_overflow_clip());
    assert!(!ScrollContainerFacts::new(visible, visible).requires_overflow_clip());
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
fn scroll_geometry_rejects_clipping_axis_without_clip_rect() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = ScrollRange::new(Size::new(0.0, 0.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);

    for overflow_x in [Overflow::Hidden, Overflow::Clip, Overflow::Scroll] {
        assert_eq!(
            ScrollGeometry::new(
                WritingMode::HorizontalTb,
                Direction::Ltr,
                ScrollContainerFacts::new(
                    ScrollContainerAxis::from_overflow(overflow_x).unwrap(),
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

    assert_eq!(
        ScrollGeometry::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            ScrollContainerFacts::new(
                ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
                ScrollContainerAxis::from_overflow(Overflow::Hidden).unwrap(),
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
fn scroll_geometry_allows_visible_axes_without_clip_rect() {
    let scrollport = ScrollRect::new(Point::ZERO, Size::new(80.0, 40.0)).unwrap();
    let overflow = ScrollRect::new(Point::ZERO, Size::new(120.0, 90.0)).unwrap();
    let range = ScrollRange::new(Size::new(0.0, 0.0)).unwrap();
    let gutters = ScrollbarGutterRects::new(None, None);

    let geometry = ScrollGeometry::new(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        ScrollContainerFacts::new(
            ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
            ScrollContainerAxis::from_overflow(Overflow::Visible).unwrap(),
        ),
        scrollport,
        None,
        overflow,
        range,
        gutters,
    )
    .unwrap();

    assert_eq!(geometry.overflow_clip(), None);
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

#[test]
fn phase_one_reports_deferred_scroll_features_explicitly() {
    let deferred = [
        ScrollUnsupportedFeature::OverflowAuto,
        ScrollUnsupportedFeature::OverflowClipMargin,
        ScrollUnsupportedFeature::ScrollbarGutterStable,
        ScrollUnsupportedFeature::ScrollbarGutterBothEdges,
        ScrollUnsupportedFeature::ScrollPadding,
        ScrollUnsupportedFeature::ScrollMargin,
        ScrollUnsupportedFeature::ScrollSnap,
    ];

    for feature in deferred {
        assert!(feature.is_phase_one_deferred());
    }

    assert!(ScrollUnsupportedFeature::LayoutOwnedMixedAxisOverflowCoupling.is_phase_one_deferred());
    assert!(!ScrollUnsupportedFeature::InvalidScrollRect.is_phase_one_deferred());
    assert!(!ScrollUnsupportedFeature::InvalidScrollRange.is_phase_one_deferred());
    assert!(!ScrollUnsupportedFeature::InvalidScrollGeometry.is_phase_one_deferred());
}

#[test]
fn scrollbar_size_uses_scroll_overflow_on_opposite_physical_axis() {
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            Point::new(Overflow::Visible, Overflow::Scroll),
            15.0,
        ),
        Size::new(15.0, 0.0)
    );
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            Point::new(Overflow::Scroll, Overflow::Visible),
            15.0,
        ),
        Size::new(0.0, 15.0)
    );
    assert_eq!(
        crate::scroll::scrollbar_size_from_overflow(
            Point::new(Overflow::Scroll, Overflow::Scroll),
            15.0,
        ),
        Size::new(15.0, 15.0)
    );
}

#[test]
fn scrollbar_reservation_places_inline_gutter_by_direction() {
    let ltr = ScrollbarReservation::from_overflow(
        Point::new(Overflow::Visible, Overflow::Scroll),
        12.0,
        Direction::Ltr,
    );
    let rtl = ScrollbarReservation::from_overflow(
        Point::new(Overflow::Visible, Overflow::Scroll),
        12.0,
        Direction::Rtl,
    );

    assert_eq!(ltr.size(), Size::new(12.0, 0.0));
    assert_eq!(ltr.inset(), Edges::new(0.0, 12.0, 0.0, 0.0));
    assert_eq!(rtl.size(), Size::new(12.0, 0.0));
    assert_eq!(rtl.inset(), Edges::new(0.0, 0.0, 0.0, 12.0));
}

#[test]
fn content_box_inset_includes_padding_border_and_scrollbar_reservation() {
    let padding = Edges::new(1.0, 2.0, 3.0, 4.0);
    let border = Edges::new(5.0, 6.0, 7.0, 8.0);
    let reservation = ScrollbarReservation::from_overflow(
        Point::new(Overflow::Visible, Overflow::Scroll),
        9.0,
        Direction::Ltr,
    );

    assert_eq!(
        crate::scroll::content_box_inset_with_scrollbar(padding, border, reservation),
        Edges::new(6.0, 17.0, 10.0, 12.0)
    );
}

#[test]
fn scrollbar_box_rects_derive_ltr_scrollport_and_gutter_rects() {
    let rects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0)).unwrap(),
        Edges::new(2.0, 3.0, 4.0, 5.0),
        Edges::all(1.0),
        ScrollbarReservation::from_overflow(
            Point::new(Overflow::Scroll, Overflow::Scroll),
            10.0,
            Direction::Ltr,
        ),
    )
    .unwrap();

    assert_eq!(
        rects.border_box(),
        ScrollRect::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0)).unwrap()
    );
    assert_eq!(
        rects.padding_box(),
        ScrollRect::new(Point::new(11.0, 21.0), Size::new(98.0, 78.0)).unwrap()
    );
    assert_eq!(
        rects.content_box(),
        ScrollRect::new(Point::new(16.0, 23.0), Size::new(80.0, 62.0)).unwrap()
    );
    assert_eq!(
        rects.scrollport(),
        ScrollRect::new(Point::new(11.0, 21.0), Size::new(88.0, 68.0)).unwrap()
    );
    assert_eq!(
        rects.gutters().vertical(),
        Some(ScrollRect::new(Point::new(99.0, 21.0), Size::new(10.0, 68.0)).unwrap())
    );
    assert_eq!(
        rects.gutters().horizontal(),
        Some(ScrollRect::new(Point::new(11.0, 89.0), Size::new(88.0, 10.0)).unwrap())
    );
}

#[test]
fn scrollbar_box_rects_shift_rtl_scrollport_after_left_gutter() {
    let rects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap(),
        Edges::ZERO,
        Edges::ZERO,
        ScrollbarReservation::from_overflow(
            Point::new(Overflow::Visible, Overflow::Scroll),
            12.0,
            Direction::Rtl,
        ),
    )
    .unwrap();

    assert_eq!(
        rects.scrollport(),
        ScrollRect::new(Point::new(12.0, 0.0), Size::new(88.0, 40.0)).unwrap()
    );
    assert_eq!(
        rects.gutters().vertical(),
        Some(ScrollRect::new(Point::ZERO, Size::new(12.0, 40.0)).unwrap())
    );
    assert_eq!(rects.gutters().horizontal(), None);
}

#[test]
fn scrollbar_box_rects_clamp_overlarge_insets_to_empty_rects() {
    let rects: ScrollBoxRects = crate::scroll::scroll_box_rects_from_border_box(
        ScrollRect::new(Point::ZERO, Size::new(10.0, 10.0)).unwrap(),
        Edges::all(20.0),
        Edges::all(20.0),
        ScrollbarReservation::from_overflow(
            Point::new(Overflow::Scroll, Overflow::Scroll),
            20.0,
            Direction::Ltr,
        ),
    )
    .unwrap();

    assert_eq!(rects.content_box().size(), Size::ZERO);
    assert_eq!(rects.scrollport().size(), Size::ZERO);
}
