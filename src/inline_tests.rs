use crate::Available;
use crate::inline::{
    AtomicInlineInput, AtomicInlineItem, atomic_inline_max_content_width,
    atomic_inline_min_content_width, layout_atomic_inline_items,
};
use crate::*;

#[test]
fn atomic_inline_line_aligns_items_to_max_baseline() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(200.0),
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(7.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 20.0), Edges::ZERO, Some(12.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 5.0));
    assert_eq!(report.items[1].location, Point::new(20.0, 0.0));
}

#[test]
fn atomic_inline_items_wrap_between_items_for_definite_width() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(25.0),
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(1, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_line_geometry_clamps_item_baseline_to_its_box() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(124.0, 64.0), Edges::ZERO, Some(94.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 0.0), Edges::ZERO, Some(0.0)),
        ],
    });

    assert_eq!(report.size, Size::new(134.0, 64.0));
    assert_eq!(report.first_baseline, Some(64.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(124.0, 64.0));
}

#[test]
fn atomic_inline_min_content_available_wraps_to_max_item_advance() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MIN_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(1, Size::new(60.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::new(2, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(60.0, 30.0));
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(30.0));
    assert_eq!(report.items[1].location, Point::new(0.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 20.0));
}

#[test]
fn atomic_inline_intrinsic_widths_use_max_item_and_sum() {
    let items = vec![
        AtomicInlineItem::new(
            0,
            Size::new(25.0, 10.0),
            Edges::new(0.0, 5.0, 0.0, 5.0),
            Some(10.0),
        ),
        AtomicInlineItem::new(
            1,
            Size::new(100.0, 10.0),
            Edges::new(0.0, 0.0, 0.0, 10.0),
            Some(10.0),
        ),
        AtomicInlineItem::new(2, Size::new(50.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(atomic_inline_min_content_width(&items), 110.0);
    assert_eq!(atomic_inline_max_content_width(&items), 195.0);
}

#[test]
fn atomic_inline_vertical_margins_participate_in_line_metrics() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![AtomicInlineItem::new(
            0,
            Size::new(20.0, 10.0),
            Edges::new(3.0, 0.0, 7.0, 0.0),
            Some(6.0),
        )],
    });

    assert_eq!(report.size, Size::new(20.0, 20.0));
    assert_eq!(report.first_baseline, Some(9.0));
    assert_eq!(report.last_baseline, Some(9.0));
    assert_eq!(report.items[0].location, Point::new(0.0, 3.0));
}

#[test]
fn atomic_inline_vertical_rl_places_line_against_right_edge() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::definite(70.0),
        writing_mode: WritingMode::VerticalRl,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
            AtomicInlineItem::new(1, Size::new(10.0, 0.0), Edges::ZERO, Some(0.0)),
            AtomicInlineItem::new(2, Size::new(20.0, 20.0), Edges::ZERO, Some(20.0)),
        ],
    });

    assert_eq!(report.size, Size::new(70.0, 40.0));
    assert_eq!(report.items[0].location, Point::new(50.0, 0.0));
    assert_eq!(report.items[1].location, Point::new(65.0, 20.0));
    assert_eq!(report.items[2].location, Point::new(50.0, 20.0));
}

#[test]
fn atomic_inline_empty_items_report_zero_size_and_no_baselines() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: Vec::new(),
    });

    assert_eq!(report.size, Size::ZERO);
    assert_eq!(report.content_size, Size::ZERO);
    assert_eq!(report.first_baseline, None);
    assert_eq!(report.last_baseline, None);
    assert!(report.items.is_empty());
}
