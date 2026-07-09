use crate::{
    Point, ScrollOffset, ScrollOffsetOf, ScrollRange, ScrollRangeOf, ScrollRect,
    ScrollUnsupportedFeature, Size,
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
