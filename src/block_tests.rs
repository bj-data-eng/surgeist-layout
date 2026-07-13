use std::collections::HashMap;

use crate::block::resolve_in_flow_margin;
use crate::*;

fn assert_positive_physical_range(range: PhysicalScrollRange, maximum: Size) {
    assert_eq!(range.x().minimum(), 0.0);
    assert_eq!(range.x().maximum(), maximum.width);
    assert_eq!(range.y().minimum(), 0.0);
    assert_eq!(range.y().maximum(), maximum.height);
}

fn lp(absolute_px: Scalar, percent_fraction: Scalar) -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

fn lp64(absolute_px: f64, percent_fraction: f64) -> LengthPercentageOf<f64> {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

#[derive(Default)]
struct ScrollBlockTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    outputs: HashMap<u32, ComputeOutput>,
}

impl Traverse for ScrollBlockTree {
    type Node = u32;

    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for ScrollBlockTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        _input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok(self.outputs[&node])
    }
}

fn perform_scroll_block(tree: &mut ScrollBlockTree) -> ComputeOutput {
    crate::compute_block(
        tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(40.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(100.0), Available::definite(40.0)),
        ),
    )
    .unwrap()
}

fn child_scroll_geometry(
    overflow: Point<Overflow>,
    size: Size,
    scrollable_overflow: ScrollRect,
) -> ScrollGeometry {
    crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow,
        size,
        Edges::ZERO,
        Edges::ZERO,
        0.0,
        scrollable_overflow,
    )
    .unwrap()
}

#[test]
fn block_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            overflow: Point::new(Overflow::Scroll, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_visible_child_overflow_content_size() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(130.0, 70.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(30.0, 30.0));
}

#[test]
fn block_scroll_geometry_clips_hidden_child_overflow_from_parent_range() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(160.0, 90.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_preserves_negative_child_overflow_origin() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(-20.0),
                top: LengthAuto::px(-5.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            position: Position::Relative,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-20.0, -5.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 45.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_distinguishes_visible_hidden_clip_and_scroll() {
    fn run(overflow: Point<Overflow>) -> ScrollGeometry {
        let mut tree = ScrollBlockTree::default();
        tree.children.insert(1, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                overflow,
                size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
                ..NodeInput::default()
            },
        );

        perform_scroll_block(&mut tree).scroll_geometry.unwrap()
    }

    let visible = run(Point::new(Overflow::Visible, Overflow::Visible));
    assert_eq!(visible.overflow_clip(), None);
    assert_positive_physical_range(visible.physical_range(), Size::ZERO);

    let hidden = run(Point::new(Overflow::Hidden, Overflow::Hidden));
    assert_eq!(hidden.overflow_clip(), Some(hidden.scrollport()));
    assert_eq!(
        hidden
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(3.0, 4.0).unwrap()),
        PhysicalScrollOffset::try_new(0.0, 0.0).unwrap()
    );

    let clip = run(Point::new(Overflow::Clip, Overflow::Clip));
    assert_eq!(clip.overflow_clip(), Some(clip.scrollport()));
    assert_positive_physical_range(clip.physical_range(), Size::ZERO);

    let scroll = run(Point::new(Overflow::Scroll, Overflow::Scroll));
    assert_eq!(scroll.overflow_clip(), Some(scroll.scrollport()));
    assert_positive_physical_range(scroll.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_uses_node_local_padding_border_and_gutter_coordinates() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            padding: Edges::all(Length::px(2.0)),
            border: Edges::all(Length::px(3.0)),
            ..NodeInput::default()
        },
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().origin(), Point::new(13.0, 3.0));
    assert_eq!(geometry.scrollport().size(), Size::new(84.0, 34.0));
    assert_eq!(
        geometry.gutters().vertical(),
        Some(ScrollRect::new(Point::new(3.0, 3.0), Size::new(10.0, 34.0)).unwrap())
    );
}

#[test]
fn block_scroll_geometry_includes_absolute_child_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(35.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(45.0, 25.0)),
    );

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(135.0, 60.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(35.0, 20.0));
}

#[test]
fn block_scroll_geometry_includes_final_content_box_after_size_resolution() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            min_size: Size::new(Dimension::px(140.0), Dimension::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(60.0), Some(40.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(60.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(140.0, 80.0)).unwrap()
    );
}

#[test]
fn block_scroll_geometry_includes_inline_child_origin_bearing_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Hidden),
            size: Size::new(Dimension::px(40.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut inline_output = ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0));
    inline_output.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Visible, Overflow::Visible),
        Size::new(20.0, 10.0),
        ScrollRect::new(Point::new(-12.0, -3.0), Size::new(70.0, 26.0)).unwrap(),
    ));
    tree.outputs.insert(2, inline_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-12.0, -3.0)
    );
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(70.0, 26.0));
    assert_positive_physical_range(geometry.physical_range(), Size::new(0.0, 13.0));
}

#[test]
fn block_scroll_geometry_clips_hidden_inline_child_overflow_from_parent_range() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    let mut inline_output =
        ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(150.0, 80.0));
    inline_output.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(30.0, 10.0),
        ScrollRect::new(Point::new(-20.0, -7.0), Size::new(180.0, 92.0)).unwrap(),
    ));
    tree.outputs.insert(2, inline_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(100.0, 40.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn block_scroll_geometry_includes_segmented_inline_overflow_rects() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2, 3, 4, 5]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(5, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(80.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(Dimension::px(80.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::InlineBlock,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        5,
        NodeInput {
            display: Display::InlineBlock,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut first_inline = ComputeOutput::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    first_inline.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Visible, Overflow::Visible),
        Size::new(10.0, 10.0),
        ScrollRect::new(Point::new(-20.0, 0.0), Size::new(30.0, 10.0)).unwrap(),
    ));
    let mut second_inline = ComputeOutput::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    second_inline.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Visible, Overflow::Visible),
        Size::new(10.0, 10.0),
        ScrollRect::new(Point::new(-7.0, 0.0), Size::new(25.0, 12.0)).unwrap(),
    ));
    tree.outputs
        .insert(2, ComputeOutput::from_outer_size(Size::new(80.0, 50.0)));
    tree.outputs.insert(3, first_inline);
    tree.outputs.insert(5, second_inline);
    tree.styles.insert(4, NodeInput::default());

    struct SegmentedTree {
        inner: ScrollBlockTree,
        line_break: LineBreakInput,
    }

    impl Traverse for SegmentedTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a> = <ScrollBlockTree as Traverse>::Children<'a>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.inner.children(node)
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.inner.child_count(node)
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.inner.child(node, index)
        }
    }

    impl Compute for SegmentedTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            self.inner.node_input(node)
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            if node == 4 {
                LayoutInputOf::line_break(self.line_break)
            } else {
                self.inner.layout_input(node)
            }
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.inner.set_unrounded(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            self.inner.compute_child(node, input)
        }
    }

    let mut segmented = SegmentedTree {
        inner: tree,
        line_break: LineBreakInput::new()
            .with_clear(Clear::Left)
            .with_metrics(metrics),
    };

    let output = crate::compute_block(
        &mut segmented,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-20.0, 0.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(120.0, 80.0)
    );
}

#[test]
fn block_scroll_geometry_includes_float_child_overflow_rect() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    let mut float_output = ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(30.0, 10.0));
    float_output.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Visible, Overflow::Visible),
        Size::new(30.0, 10.0),
        ScrollRect::new(Point::ZERO, Size::new(140.0, 55.0)).unwrap(),
    ));
    tree.outputs.insert(2, float_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(140.0, 55.0)
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(40.0, 15.0));
}

#[test]
fn block_float_child_node_output_recomputes_scroll_geometry() {
    let padding = Edges::all(Length::px(2.0));
    let border = Edges::all(Length::px(1.0));
    let resolved_padding = Edges::all(2.0);
    let resolved_border = Edges::all(1.0);
    let child_compute_overflow =
        ScrollRect::new(Point::new(-8.0, -3.0), Size::new(50.0, 20.0)).unwrap();
    let mut float_output = ComputeOutput::from_sizes(Size::new(30.0, 10.0), Size::new(70.0, 32.0));
    float_output.scroll_geometry = Some(
        crate::scroll::scroll_geometry_from_layout(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Point::new(Overflow::Hidden, Overflow::Hidden),
            Size::new(30.0, 10.0),
            resolved_padding,
            resolved_border,
            0.0,
            child_compute_overflow,
        )
        .unwrap(),
    );

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            float: Float::Left,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            padding,
            border,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, float_output);

    perform_scroll_block(&mut tree);

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size, Size::new(30.0, 10.0));
    assert_eq!(child_layout.content_size, Size::new(70.0, 32.0));
    assert_eq!(child_layout.padding, resolved_padding);
    assert_eq!(child_layout.border, resolved_border);

    let base_overflow = crate::scroll::scrollable_overflow_from_layout_content_size(
        Direction::Ltr,
        Point::new(Overflow::Hidden, Overflow::Hidden),
        child_layout.size,
        child_layout.padding,
        child_layout.border,
        0.0,
        child_layout.content_size,
    )
    .unwrap();
    let expected_overflow = crate::scroll::scroll_rect_union(base_overflow, child_compute_overflow)
        .expect("expected float child overflow union is valid");
    let expected_geometry = crate::scroll::scroll_geometry_from_layout(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Point::new(Overflow::Hidden, Overflow::Hidden),
        child_layout.size,
        child_layout.padding,
        child_layout.border,
        0.0,
        expected_overflow,
    )
    .unwrap();

    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport(), expected_geometry.scrollport());
    assert_eq!(geometry.scrollable_overflow(), expected_overflow);
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        child_compute_overflow.origin()
    );
    assert_eq!(
        geometry.physical_range(),
        expected_geometry.physical_range()
    );
}

#[test]
fn block_scroll_geometry_includes_absolute_margin_box_with_area_offset() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
            size: Size::new(Dimension::px(120.0), Dimension::px(80.0)),
            padding: Edges::all(Length::px(7.0)),
            border: Edges::all(Length::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            inset: Edges {
                left: LengthAuto::px(90.0),
                top: LengthAuto::px(60.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                left: LengthAuto::px(4.0),
                top: LengthAuto::px(3.0),
                right: LengthAuto::px(6.0),
                bottom: LengthAuto::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    let mut absolute_output =
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(50.0, 20.0));
    absolute_output.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Visible, Overflow::Visible),
        Size::new(20.0, 10.0),
        ScrollRect::new(Point::new(-2.0, -1.0), Size::new(60.0, 25.0)).unwrap(),
    ));
    tree.outputs.insert(2, absolute_output);

    let output = perform_scroll_block(&mut tree);

    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(12.0, 12.0)
    );
    assert_eq!(
        geometry.scrollable_overflow().size(),
        Size::new(145.0, 80.0)
    );
    assert_eq!(output.content_size, Size::new(144.0, 83.0));
}

#[test]
fn block_child_node_output_recomputes_child_scroll_geometry() {
    let mut child_output = ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = None;

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    perform_scroll_block(&mut tree);

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), Size::new(50.0, 20.0));
    assert_positive_physical_range(geometry.physical_range(), Size::new(30.0, 25.0));
}

#[test]
fn block_child_node_output_keeps_hidden_child_own_scroll_range() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(160.0, 90.0)),
    );

    perform_scroll_block(&mut tree);

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::new(Point::ZERO, Size::new(160.0, 90.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(110.0, 70.0));
}

#[test]
fn block_absolute_child_scroll_geometry_uses_final_node_output_size() {
    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(120.0, 30.0)),
    );

    perform_scroll_block(&mut tree);

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size.width, 100.0);
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size().width, 100.0);
    assert_positive_physical_range(geometry.physical_range(), Size::new(20.0, 20.0));
}

#[test]
fn block_child_node_output_preserves_child_scrollable_overflow_origin() {
    let child_overflow = ScrollRect::new(Point::new(-15.0, -4.0), Size::new(95.0, 49.0)).unwrap();
    let mut child_output = ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(80.0, 45.0));
    child_output.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(50.0, 20.0),
        child_overflow,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    perform_scroll_block(&mut tree);

    let geometry = tree.layouts[&2].scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollable_overflow().origin(),
        Point::new(-15.0, -4.0)
    );
    assert_eq!(geometry.scrollable_overflow().size(), Size::new(95.0, 49.0));
}

#[test]
fn block_inline_child_node_output_uses_final_inline_item_geometry() {
    let child_overflow = ScrollRect::new(Point::new(-9.0, -3.0), Size::new(74.0, 34.0)).unwrap();
    let mut child_output = ComputeOutput::from_sizes(Size::new(40.0, 12.0), Size::new(65.0, 31.0));
    child_output.scroll_geometry = Some(child_scroll_geometry(
        Point::new(Overflow::Hidden, Overflow::Hidden),
        Size::new(40.0, 12.0),
        child_overflow,
    ));

    let mut tree = ScrollBlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::InlineBlock,
            overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(2, child_output);

    perform_scroll_block(&mut tree);

    let child_layout = tree.layouts[&2];
    assert_eq!(child_layout.size, Size::new(40.0, 12.0));
    assert_eq!(child_layout.content_size, Size::new(65.0, 31.0));
    let geometry = child_layout.scroll_geometry.unwrap();
    assert_eq!(geometry.scrollport().size(), child_layout.size);
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
}

fn output_from_known_or(input: ComputeInput, fallback: Size) -> ComputeOutput {
    let size = Size::new(
        input.known().width.unwrap_or(fallback.width),
        input.known().height.unwrap_or(fallback.height),
    );
    ComputeOutput::from_sizes(size, size)
}

#[derive(Default)]
struct CalcBlockTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    inputs: HashMap<u32, Vec<ComputeInput>>,
}

impl Traverse for CalcBlockTree {
    type Node = u32;

    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for CalcBlockTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        Ok({
            self.inputs.entry(node).or_default().push(input);
            ComputeOutput::from_outer_size(Size::new(
                input.known().width.unwrap_or(0.0),
                input.known().height.unwrap_or(10.0),
            ))
        })
    }
}

#[test]
fn block_lays_out_atomic_inline_children_on_one_line() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 20.0));
}

#[test]
fn f64_block_layout_preserves_fractional_child_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(100.0), DimensionOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(40.0), DimensionOf::px(5.25)),
                margin: Edges {
                    top: LengthAutoOf::px(large),
                    bottom: LengthAutoOf::px(0.25),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(40.0), DimensionOf::px(7.5)),
                margin: Edges {
                    top: LengthAutoOf::px(0.375),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the first child")
            .location,
        Point::new(0.0, large)
    );
    assert_eq!(
        tree.output(2)
            .expect("block layout must stage output for the second child")
            .location,
        Point::new(0.0, large + 5.25 + 0.375)
    );
    assert_eq!(
        tree.output(0)
            .expect("block layout must stage output for the root node")
            .size,
        Size::new(100.0, large + 5.25 + 0.375 + 7.5)
    );
}

#[test]
fn f64_block_layout_resolves_affine_values_without_narrowing() {
    let large = 16_777_217.25_f64;
    let container_width = 16_777_220.5_f64;
    let margin_left = lp64(large, 0.10);
    let width = lp64(large + 0.25, 0.50);
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(container_width), DimensionOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::value(width), DimensionOf::px(4.5)),
                margin: Edges {
                    left: LengthAutoOf::value(margin_left),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(
            AvailableOf::definite(container_width),
            AvailableOf::MAX_CONTENT,
        ),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the child")
            .location,
        Point::new(18_454_939.3, 0.0)
    );
    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the child")
            .size,
        Size::new(25_165_827.75, 4.5)
    );
}

#[test]
fn f64_inline_layout_preserves_large_atomic_inline_offsets() {
    let large = 16_777_217.25_f64;
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<f64> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(large + 20.0), DimensionOf::AUTO),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            1,
            NodeInputOf::<f64> {
                display: Display::InlineBlock,
                size: Size::new(DimensionOf::px(large), DimensionOf::px(10.5)),
                ..NodeInputOf::<f64>::default()
            },
        )
        .style(
            2,
            NodeInputOf::<f64> {
                display: Display::InlineBlock,
                size: Size::new(DimensionOf::px(9.75), DimensionOf::px(20.25)),
                ..NodeInputOf::<f64>::default()
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(
            AvailableOf::definite(large + 20.0),
            AvailableOf::MAX_CONTENT,
        ),
    )
    .unwrap();

    assert_eq!(
        tree.output(1)
            .expect("block layout must stage output for the first child")
            .location,
        Point::new(0.0, 9.75)
    );
    assert_eq!(
        tree.output(2)
            .expect("block layout must stage output for the second child")
            .location,
        Point::new(large, 0.0)
    );
    assert_eq!(
        tree.output(0)
            .expect("block layout must stage output for the root node")
            .size,
        Size::new(large + 20.0, 20.25)
    );
}

#[test]
fn vertical_rl_block_places_atomic_inline_run_at_inline_start_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                border: Edges::all(Length::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(55.0, 5.0)
    );
}

#[test]
fn inline_grid_uses_grid_tracks_and_participates_as_atomic_inline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_grid_lanes_uses_lanes_tracks_and_participates_as_atomic_inline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponent::px(40.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineGridLanes,
                grid_template_columns: vec![TrackComponent::px(10.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(40.0, 20.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::new(10.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn block_wraps_atomic_inline_children_between_items() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(40.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size.height, 20.0);
}

#[test]
fn block_atomic_inline_run_honors_line_break_child() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 2.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(20.0, 12.0)
    );
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 16.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 28.0));
}

#[test]
fn ordinary_block_child_receives_parent_non_horizontal_containing_flow() {
    let parent_flow_axes = crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                size: Size::new(Dimension::px(100.0), Dimension::px(80.0)),
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default());

    crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(100.0), Some(80.0)),
            crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::new(Available::definite(100.0), Available::definite(80.0)),
        ),
    )
    .unwrap();

    assert!(
        tree.inputs(1)
            .iter()
            .all(|input| input.containing_flow_axes() == parent_flow_axes)
    );
}

#[test]
fn block_line_break_conversion_with_metadata_preserves_current_output() {
    let metrics = InlineMetrics::from_line_height_and_baseline(24.0, 18.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_direction(Direction::Rtl)
                .with_writing_mode(WritingMode::HorizontalTb)
                .with_vertical_align(VerticalAlign::Top)
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 18.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 36.0));
}

#[test]
fn block_line_break_metrics_create_empty_line_height() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 15.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::default()
            },
        )
        .children(0, [1, 2])
        .line_break(1, LineBreakInput::new().with_metrics(metrics))
        .line_break(2, LineBreakInput::new().with_metrics(metrics));

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::splat(Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size.height, 40.0);
    assert_eq!(output.first_baselines.y, Some(15.0));
    assert_eq!(output.last_baselines.y, Some(35.0));
    assert_eq!(tree.layout(1).unwrap().location.y, 15.0);
    assert_eq!(tree.layout(2).unwrap().location.y, 35.0);
}

#[test]
fn block_inline_boundaries_are_reported_as_zero_size_inline_controls() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(18.0, 13.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.inputs(1), &[]);
    assert_eq!(tree.inputs(3), &[]);
    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 13.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 3.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 13.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(20.0, 1.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 18.0));
}

#[test]
fn block_inline_boundaries_before_overwide_first_inline_block_do_not_create_leading_line() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(50.0, 35.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(20.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(40.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(20.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 25.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(20.0, 50.0));
}

#[test]
fn vertical_block_inline_boundaries_use_parent_flow() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalRl),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(66.0, 0.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(70.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(66.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn vertical_lr_block_inline_boundaries_use_parent_flow() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalLr),
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            3,
            InlineBoundaryInput::new(InlineBoundaryKind::End, boundary_metrics)
                .with_writing_mode(WritingMode::VerticalLr),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(14.0, 0.0)
    );
    assert_eq!(tree.final_layout(1).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(14.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
}

#[test]
fn hidden_line_break_does_not_split_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new().hidden())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(tree.inputs(2), &[]);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 10.0));
}

#[test]
fn block_atomic_inline_run_never_computes_line_break_as_box() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new())
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(12.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(tree.inputs(2), &[]);
}

#[test]
fn vertical_rl_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(12.0), Dimension::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(66.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().location.x, 48.0);
}

#[test]
fn vertical_lr_line_break_is_laid_out_as_zero_size_inline_control() {
    let metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalLr)
                .with_metrics(metrics),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalLr,
                size: Size::new(Dimension::px(12.0), Dimension::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(14.0, 30.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().location.x, 20.0);
}

#[test]
#[should_panic(expected = "vertical line-break clear layout is not implemented")]
fn vertical_line_break_clear_panics_until_vertical_clear_is_modeled() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            1,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .with_clear(Clear::Both),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "vertical line-break clear layout is not implemented")]
fn vertical_parent_rejects_clear_even_when_line_break_input_defaults_horizontal() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(1, LineBreakInput::new().with_clear(Clear::Both));

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "line-break flow must match containing inline flow")]
fn vertical_parent_rejects_default_line_break_flow_until_input_is_layout_ready() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                ..NodeInput::DEFAULT
            },
        )
        .line_break(1, LineBreakInput::new());

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "inline boundary flow must match containing inline flow")]
fn vertical_parent_rejects_default_inline_boundary_flow_until_input_is_layout_ready() {
    let boundary_metrics = InlineMetrics::from_line_height_and_baseline(20.0, 14.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                ..NodeInput::DEFAULT
            },
        )
        .inline_boundary(
            1,
            InlineBoundaryInput::new(InlineBoundaryKind::Start, boundary_metrics),
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
}

#[test]
fn hidden_vertical_line_break_does_not_create_inline_control() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(80.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(10.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            2,
            LineBreakInput::new()
                .with_writing_mode(WritingMode::VerticalRl)
                .hidden(),
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(Dimension::px(12.0), Dimension::px(16.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(80.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(68.0, 30.0)
    );
}

fn inline_break_clear_tree(
    clear: Clear,
    float_side: Float,
) -> crate::test_support::layout_tree::OracleTree {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: float_side,
                size: Size::new(Dimension::px(80.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(clear)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
}

#[test]
fn line_break_clear_left_moves_following_inline_segment_below_left_float() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Left);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_right_moves_following_inline_segment_below_right_float() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Right);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_both_uses_greater_left_or_right_float_bottom() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4, 5])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(60.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(60.0), Dimension::px(70.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            4,
            LineBreakInput::new()
                .with_clear(Clear::Both)
                .with_metrics(metrics),
        )
        .style(
            5,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(5).unwrap().location,
        Point::new(0.0, 70.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 80.0));
}

#[test]
fn line_break_clear_at_run_end_moves_following_block_below_float() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(80.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(Clear::Left)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(25.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 50.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_left_ignores_right_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Right).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyRight,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(180.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(200.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_right_ignores_left_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Left).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyCenter,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(90.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(110.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(90.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_that_is_noop_after_line_height_preserves_alignment() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(80.0), Dimension::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(
            3,
            LineBreakInput::new()
                .with_clear(Clear::Left)
                .with_metrics(metrics),
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(180.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(200.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_none_preserves_existing_single_run_layout_near_float() {
    let mut tree = inline_break_clear_tree(Clear::None, Float::Left);

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(200.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(20.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn block_min_content_atomic_inline_run_uses_max_item_advance() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(40.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(60.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MIN_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(60.0, 30.0));
    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 10.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 20.0)
    );
}

#[test]
fn atomic_inline_auto_margins_resolve_to_zero() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                margin: Edges {
                    left: LengthAuto::AUTO,
                    right: LengthAuto::AUTO,
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    let child = tree.final_layout(1).unwrap();
    assert_eq!(child.location, Point::new(0.0, 0.0));
    assert_eq!(child.margin.left, 0.0);
    assert_eq!(child.margin.right, 0.0);
}

#[test]
fn inline_block_intrinsic_width_shrink_wraps_children() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(70.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(70.0, 20.0));
    assert_eq!(tree.final_layout(0).unwrap().size.width, 70.0);
}

#[test]
fn inline_block_uses_bottom_synthesized_baseline_when_child_has_no_baseline() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.y, 10.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
}

#[test]
fn inline_block_uses_inner_last_baseline_for_atomic_alignment() {
    let measured_inline_block = ComputeOutput::from_sizes_and_baselines(
        Size::new(10.0, 30.0),
        Size::new(10.0, 30.0),
        crate::Baselines {
            first: Point::new(None, Some(5.0)),
            last: Point::new(None, Some(25.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(25.0)),
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, measured_inline_block);

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(2).unwrap().location.y, 0.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 30.0);
}

#[test]
fn inline_block_keeps_child_margins_inside_atomic_wrapper() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().size, Size::new(20.0, 15.0));
    assert_eq!(tree.final_layout(2).unwrap().location.y, 5.0);
    assert_eq!(tree.final_layout(0).unwrap().size.height, 15.0);
}

#[test]
fn inline_grid_can_host_subgrid_descendant() {
    let subgrid_track = || {
        TrackComponent::Subgrid(crate::SubgridTrack {
            name_components: Vec::new(),
        })
    };
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponent::px(80.0)],
                grid_template_rows: vec![TrackComponent::px(30.0)],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![subgrid_track()],
                grid_template_rows: vec![subgrid_track()],
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(80.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(80.0, 30.0));
}

#[test]
fn block_positions_block_children_around_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(10.0)),
                margin: Edges {
                    bottom: LengthAuto::px(7.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(15.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(10.0)),
                margin: Edges {
                    top: LengthAuto::px(3.0),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(0.0, 27.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(10.0, 17.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 35.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 45.0));
}

#[test]
fn block_hidden_and_absolute_children_do_not_split_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(10.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::None,
                float: Float::Left,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                position: Position::Absolute,
                float: Float::Left,
                size: Size::new(Dimension::px(5.0), Dimension::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(10.0, 0.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(100.0, 10.0));
}

#[test]
fn block_rtl_atomic_inline_run_places_items_from_right_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn block_rtl_atomic_inline_run_mirrors_line_break_output_x() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(2, LineBreakInput::new().with_direction(Direction::Rtl))
        .style(
            3,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 2.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(80.0, 12.0)
    );
    assert_eq!(tree.final_layout(2).unwrap().size, Size::ZERO);
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(70.0, 16.0)
    );
}

#[test]
fn block_legacy_right_rtl_aligns_atomic_inline_run_to_physical_right_edge() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                direction: Direction::Rtl,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(80.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(50.0, 0.0)
    );
}

#[test]
fn block_atomic_inline_run_alignment_uses_resolved_inner_width() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                min_size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(50.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::MAX_CONTENT)).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location.x, 25.0);
}

#[test]
fn block_legacy_center_aligns_atomic_inline_run() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(Dimension::px(100.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::splat(Available::definite(100.0))).unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(25.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(45.0, 0.0)
    );
}

#[test]
fn block_inline_run_content_size_includes_visible_overflow_and_relative_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                overflow: Point::new(Overflow::Visible, Overflow::Visible),
                inset: Edges {
                    left: LengthAuto::px(15.0),
                    top: LengthAuto::px(5.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 30.0)),
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size, Size::new(95.0, 35.0));
}

#[test]
fn block_inline_run_content_size_accounts_for_negative_relative_inset_after_content() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(10.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                overflow: Point::new(Overflow::Visible, Overflow::Visible),
                inset: Edges {
                    top: LengthAuto::px(-5.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .measure(
            2,
            ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 30.0)),
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.content_size.height, 45.0);
}

#[test]
fn block_reports_inline_run_first_and_last_baselines() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::definite(100.0)),
            ),
        )
        .unwrap();

    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_reports_inline_run_baseline_including_padding() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.first_baselines.y, Some(30.0));
    assert_eq!(output.last_baselines.y, Some(30.0));
}

#[test]
fn block_definite_compute_size_keeps_inline_run_baselines() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(30.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(20.0));
}

#[test]
fn block_definite_compute_size_keeps_block_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        crate::Baselines {
            first: Point::new(None, Some(7.0)),
            last: Point::new(None, Some(17.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                ..NodeInput::DEFAULT
            },
        )
        .measure(1, child_output);

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(7.0));
    assert_eq!(output.last_baselines.y, Some(17.0));
}

fn assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let child_output = ComputeOutputOf::from_sizes_and_baselines(
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        Size::new(S::from_f64(70.0), S::from_f64(110.0)),
        BaselinesOf {
            first: Point::new(Some(S::from_f64(7.0)), None),
            last: Point::new(Some(S::from_f64(11.0)), None),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(S::from_f64(120.0)), DimensionOf::AUTO),
                padding: Edges {
                    top: LengthOf::px(S::from_f64(5.0)),
                    left: LengthOf::px(S::from_f64(3.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode,
                margin: Edges {
                    top: LengthAutoOf::px(S::from_f64(17.0)),
                    left: LengthAutoOf::px(S::from_f64(11.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        )
        .measure(1, child_output);

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(160.0))),
            crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(
        tree.layout(1).expect("child layout is staged").location,
        Point::new(S::from_f64(14.0), S::from_f64(22.0))
    );
    assert_eq!(
        output.first_baselines,
        Point::new(Some(S::from_f64(21.0)), None)
    );
    assert_eq!(
        output.last_baselines,
        Point::new(Some(S::from_f64(25.0)), None)
    );
}

#[test]
fn orthogonal_baseline_block_translation_uses_physical_x_for_f32() {
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>(
        WritingMode::VerticalRl,
    );
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn orthogonal_baseline_block_translation_uses_physical_x_for_f64() {
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>(
        WritingMode::VerticalRl,
    );
    assert_block_translates_orthogonal_child_baselines_on_the_child_block_axis::<f64>(
        WritingMode::SidewaysLr,
    );
}

fn assert_block_aggregates_physical_baselines_on_both_axes<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1, 2])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(S::from_f64(140.0)), DimensionOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                writing_mode: WritingMode::VerticalRl,
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(5.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf::<S> {
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(19.0)),
                    LengthAutoOf::px(S::from_f64(7.0)),
                    LengthAutoOf::px(S::from_f64(23.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            1,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(20.0)),
                Size::new(S::from_f64(70.0), S::from_f64(20.0)),
                BaselinesOf {
                    first: Point::new(Some(S::from_f64(7.0)), None),
                    last: Point::new(Some(S::from_f64(11.0)), None),
                },
            ),
        )
        .measure(
            2,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(30.0), S::from_f64(40.0)),
                Size::new(S::from_f64(30.0), S::from_f64(40.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(9.0))),
                    last: Point::new(None, Some(S::from_f64(15.0))),
                },
            ),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(140.0)), Some(S::from_f64(200.0))),
            crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::new(
                AvailableOf::definite(S::from_f64(140.0)),
                AvailableOf::definite(S::from_f64(200.0)),
            ),
        ),
    )
    .expect("block layout succeeds");
    let first_child = tree.layout(1).expect("first child layout is staged");
    let second_child = tree.layout(2).expect("second child layout is staged");

    assert_eq!(
        output.first_baselines,
        Point::new(
            Some(first_child.location.x + S::from_f64(7.0)),
            Some(second_child.location.y + S::from_f64(9.0)),
        )
    );
    assert_eq!(
        output.last_baselines,
        Point::new(
            Some(first_child.location.x + S::from_f64(11.0)),
            Some(second_child.location.y + S::from_f64(15.0)),
        )
    );
}

#[test]
fn physical_baseline_block_aggregates_both_axes_for_f32() {
    assert_block_aggregates_physical_baselines_on_both_axes::<f32>();
}

#[test]
fn physical_baseline_block_aggregates_both_axes_for_f64() {
    assert_block_aggregates_physical_baselines_on_both_axes::<f64>();
}

fn assert_block_preserves_a_child_y_baseline<S: LayoutScalar>()
where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(DimensionOf::px(S::from_f64(120.0)), DimensionOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf::<S> {
                writing_mode: WritingMode::HorizontalTb,
                margin: Edges::new(
                    LengthAutoOf::px(S::from_f64(17.0)),
                    LengthAutoOf::px(S::from_f64(5.0)),
                    LengthAutoOf::px(S::from_f64(13.0)),
                    LengthAutoOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(
            1,
            ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(70.0), S::from_f64(40.0)),
                Size::new(S::from_f64(70.0), S::from_f64(40.0)),
                BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(9.0))),
                    last: Point::new(None, Some(S::from_f64(15.0))),
                },
            ),
        );

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(120.0)), Some(S::from_f64(160.0))),
            crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            Size::new(
                AvailableOf::definite(S::from_f64(120.0)),
                AvailableOf::definite(S::from_f64(160.0)),
            ),
        ),
    )
    .expect("block layout succeeds");
    let child = tree.layout(1).expect("child layout is staged");

    assert_eq!(
        output.first_baselines,
        Point::new(None, Some(child.location.y + S::from_f64(9.0)))
    );
    assert_eq!(
        output.last_baselines,
        Point::new(None, Some(child.location.y + S::from_f64(15.0)))
    );
}

#[test]
fn physical_baseline_block_preserves_y_for_f32() {
    assert_block_preserves_a_child_y_baseline::<f32>();
}

#[test]
fn physical_baseline_block_preserves_y_for_f64() {
    assert_block_preserves_a_child_y_baseline::<f64>();
}

#[test]
fn block_definite_compute_size_keeps_non_empty_flex_child_baselines() {
    let child_output = ComputeOutput::from_sizes_and_baselines(
        Size::new(30.0, 20.0),
        Size::new(30.0, 20.0),
        crate::Baselines {
            first: Point::new(None, Some(9.0)),
            last: Point::new(None, Some(19.0)),
        },
    );
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .children(1, [2])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Flex,
                ..NodeInput::DEFAULT
            },
        )
        .style(2, NodeInput::DEFAULT)
        .measure(1, child_output);

    let output = tree
        .compute_child(
            0,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::splat(Available::MAX_CONTENT),
            ),
        )
        .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(output.first_baselines.y, Some(9.0));
    assert_eq!(output.last_baselines.y, Some(19.0));
}

#[test]
fn block_layout_stacks_in_flow_children_vertically() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                self.outputs[&node]
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                right: Length::px(5.0),
                bottom: Length::px(7.0),
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(30.0, 12.0), Size::new(30.0, 12.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 41.0));
    assert_eq!(output.content_size, Size::new(30.0, 29.0));
    assert_eq!(tree.layouts[&2].location, Point::new(18.0, 6.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.left, 6.0);
    assert_eq!(tree.layouts[&3].location, Point::new(12.0, 21.0));
    assert_eq!(tree.layouts[&3].size, Size::new(30.0, 12.0));
    assert_eq!(tree.inputs[&2][0].parent(), Size::new(Some(82.0), None));
    assert_eq!(tree.inputs[&3][0].parent(), Size::new(Some(82.0), None));
}

#[test]
fn block_in_flow_affine_margin_resolves_against_containing_block_width() {
    let mut tree = CalcBlockTree::default();
    let margin_left = lp(-4.0, 0.1);
    let width = lp(20.0, 0.5);
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::value(width), Dimension::AUTO),
            margin: Edges {
                left: LengthAuto::value(margin_left),
                right: LengthAuto::ZERO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(200.0), None),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::Definite(200.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs[&2][0].known(), Size::new(Some(120.0), None));
    assert_eq!(tree.layouts[&2].location, Point::new(16.0, 0.0));
    assert_eq!(tree.layouts[&2].margin.left, 16.0);
    assert_eq!(tree.layouts[&2].size, Size::new(120.0, 10.0));
}

#[test]
fn block_container_affine_padding_uses_parent_basis() {
    let mut tree = CalcBlockTree::default();
    let padding = lp(2.0, 0.1);
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            padding: Edges::all(Length::value(padding)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(1, NodeInput::default());

    let output = crate::compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.content_size.width, 76.0);
}

#[test]
fn block_auto_width_includes_in_flow_child_horizontal_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            item_is_table: true,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(3.0, 0.0));
    assert_eq!(output.size, Size::new(32.0, 10.0));
    assert_eq!(output.content_size, Size::new(32.0, 10.0));
}

#[test]
fn block_float_contributes_to_intrinsic_width_and_places_from_right_edge() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::AUTO, Dimension::px(80.0)),
            border: Edges::all(Length::px(2.0)),
            ..NodeInput::default()
        },
    );
    for node in [2, 3, 4] {
        tree.styles.insert(
            node,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::default()
            },
        );
        tree.outputs.insert(
            node,
            ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
        );
    }

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(154.0, 80.0));
    assert_eq!(tree.layouts[&2].location, Point::new(102.0, 2.0));
    assert_eq!(tree.layouts[&3].location, Point::new(52.0, 2.0));
    assert_eq!(tree.layouts[&4].location, Point::new(2.0, 2.0));
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats_above_full_width_float() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 200.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_overflow_clip_zero_width_child_ignores_float_exclusion_without_clear() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Clip, Overflow::Clip),
                size: Size::new(Dimension::px(0.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_right_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(150.0, 0.0)
    );
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_center_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(75.0, 0.0)
    );
}

#[test]
fn block_bfc_float_content_size_height_excludes_container_top_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                border: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(50.0), Dimension::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().content_size.height, 30.0);
}

#[test]
fn block_bfc_clear_only_visible_child_keeps_normal_x_while_clearing_y() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(50.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(150.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: Point::new(Overflow::Visible, Overflow::Visible),
                size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(150.0, 50.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 70.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_left_sits_below_left_float_row() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 100.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_right_sits_below_all_right_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Right,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    )
    .unwrap();
    round_layout(&mut tree, 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 200.0)
    );
}

#[test]
fn block_layout_collapses_adjacent_in_flow_vertical_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 20.0));
    assert_eq!(output.size, Size::new(100.0, 30.0));
    assert_eq!(output.content_size, Size::new(100.0, 30.0));
}

#[test]
fn block_layout_collapses_first_child_top_margin_through_parent() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(output.top_margin.resolve(), 10.0);
    assert_eq!(output.bottom_margin.resolve(), 0.0);
}

#[test]
fn block_scroll_container_keeps_first_child_top_margin_inside() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 15.0));
    assert_eq!(output.content_size, Size::new(100.0, 15.0));
    assert_eq!(output.top_margin.resolve(), 0.0);
    assert_eq!(output.bottom_margin.resolve(), 0.0);
    assert!(!output.margins_can_collapse_through);
}

#[test]
fn block_rtl_scrollbar_gutter_uses_left_inset() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(10.0),
                ))
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            overflow: Point::new(Overflow::Visible, Overflow::Scroll),
            scrollbar_width: crate::ScrollbarWidthOf::try_new(17.0).unwrap(),
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(17.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(83.0, 10.0));
}

#[test]
fn block_layout_collapses_last_child_bottom_margin_through_parent() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                bottom: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 5.0), Size::new(100.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(output.size, Size::new(100.0, 5.0));
    assert_eq!(output.top_margin.resolve(), 0.0);
    assert_eq!(output.bottom_margin.resolve(), 10.0);
}

#[test]
fn block_layout_keeps_grid_child_margins_inside_parent_flow() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(ComputeOutput::from_outer_size(Size::new(50.0, 20.0)))
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Grid,
            margin: Edges {
                top: LengthAuto::px(10.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(50.0, 30.0));
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.top, 10.0);
}

#[test]
fn block_layout_collapses_margins_through_empty_in_flow_child() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            border: Edges {
                top: Length::px(1.0),
                right: Length::ZERO,
                bottom: Length::px(1.0),
                left: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(10.0),
                bottom: LengthAuto::px(5.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(7.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );
    let mut empty_output = ComputeOutput::from_sizes(Size::new(100.0, 0.0), Size::new(100.0, 0.0));
    empty_output.margins_can_collapse_through = true;
    tree.outputs.insert(2, empty_output);
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(100.0, 10.0), Size::new(100.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 11.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 11.0));
    assert_eq!(output.size, Size::new(100.0, 22.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
}

#[test]
fn block_empty_auto_height_can_collapse_through() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 0.0));
    assert!(output.margins_can_collapse_through);
}

#[test]
fn block_with_padding_reports_own_margins_when_child_collapse_is_blocked() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("empty block should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            margin: Edges {
                top: LengthAuto::px(8.0),
                bottom: LengthAuto::px(6.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            padding: Edges {
                top: Length::px(1.0),
                bottom: Length::px(1.0),
                ..Edges::all(Length::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 2.0));
    assert_eq!(output.top_margin.resolve(), 8.0);
    assert_eq!(output.bottom_margin.resolve(), 6.0);
    assert!(!output.margins_can_collapse_through);
}

fn assert_collapsible_percentage_margins_use_containing_inline_extent<S: LayoutScalar>(
    writing_mode: WritingMode,
) where
    crate::test_support::layout_tree::OracleTreeOf<S>: Compute + Traverse<Node = u32, Scalar = S>,
{
    let top_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.25))
        .expect("test coefficients are finite");
    let bottom_margin = LengthPercentageOf::<S>::from_coefficients(S::ZERO, S::from_f64(0.5))
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(1, [])
        .style(
            1,
            NodeInputOf::<S> {
                display: Display::Block,
                size: Size::new(DimensionOf::px(S::from_f64(100.0)), DimensionOf::AUTO),
                margin: Edges {
                    top: LengthAutoOf::value(top_margin),
                    bottom: LengthAutoOf::value(bottom_margin),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                padding: Edges {
                    top: LengthOf::px(S::from_f64(1.0)),
                    bottom: LengthOf::px(S::from_f64(1.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(40.0)), Some(S::from_f64(120.0))),
            crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr),
            Size::new(
                AvailableOf::definite(S::from_f64(40.0)),
                AvailableOf::definite(S::from_f64(120.0)),
            ),
        ),
    )
    .expect("block layout succeeds");

    assert_eq!(output.top_margin.resolve(), S::from_f64(30.0));
    assert_eq!(output.bottom_margin.resolve(), S::from_f64(60.0));
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f32() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f32>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn collapsible_percentage_margins_use_non_horizontal_containing_inline_extent_for_f64() {
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::VerticalRl,
    );
    assert_collapsible_percentage_margins_use_containing_inline_extent::<f64>(
        WritingMode::SidewaysLr,
    );
}

#[test]
fn block_in_flow_invalid_numeric_horizontal_margin_uses_zero_fallback() {
    let invalid_margin = LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX)
        .expect("test coefficients are finite");
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(f32::MAX), Dimension::AUTO),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(10.0), Dimension::AUTO),
                margin: Edges {
                    left: LengthAuto::value(invalid_margin),
                    ..Edges::all(LengthAuto::ZERO)
                },
                ..NodeInput::default()
            },
        );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(f32::MAX), None),
            crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::new(Available::definite(f32::MAX), Available::MAX_CONTENT),
        ),
    )
    .expect("the in-flow invalid-numeric margin falls back to zero");

    assert_eq!(
        tree.output(2)
            .expect("child block receives an in-flow layout")
            .margin
            .left,
        0.0
    );
}

#[test]
fn block_layout_positions_in_flow_children_from_right_edge_in_rtl() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::ZERO,
                right: Length::px(5.0),
                bottom: Length::ZERO,
                left: Length::px(11.0),
            },
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::px(7.0),
                bottom: LengthAuto::ZERO,
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(tree.layouts[&2].location, Point::new(67.0, 1.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.right, 7.0);
}

#[test]
fn block_layout_expands_horizontal_auto_margins_for_in_flow_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::ZERO,
                right: LengthAuto::AUTO,
                bottom: LengthAuto::ZERO,
                left: LengthAuto::AUTO,
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(output.content_size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
}

#[test]
fn block_content_size_includes_visible_child_overflow_content() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(120.0, 24.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(40.0, 10.0));
    assert_eq!(output.content_size, Size::new(120.0, 24.0));
}

#[test]
fn block_relative_child_inset_offsets_final_layout_location() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(3.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::ZERO,
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(6.0),
            },
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(13.0, 3.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn block_layout_stretches_auto_width_in_flow_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_sizes(
                    Size::new(input.known().width.unwrap(), 10.0),
                    Size::new(input.known().width.unwrap(), 10.0),
                )
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::ZERO,
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::ZERO,
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                left: LengthAuto::px(3.0),
                right: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs[&2][0].known().width, Some(76.0));
    assert_eq!(tree.layouts[&2].size, Size::new(76.0, 10.0));
    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 0.0));
    assert_eq!(output.content_size, Size::new(88.0, 10.0));
    assert_eq!(output.size, Size::new(100.0, 10.0));
}

#[test]
fn block_compute_size_uses_in_flow_children_for_auto_height() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_sizes(
                    Size::new(input.known().width.unwrap(), 10.0),
                    Size::new(input.known().width.unwrap(), 10.0),
                )
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            padding: Edges {
                top: Length::px(3.0),
                left: Length::px(5.0),
                right: Length::px(7.0),
                bottom: Length::px(7.0),
            },
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            margin: Edges {
                top: LengthAuto::px(2.0),
                right: LengthAuto::px(9.0),
                bottom: LengthAuto::px(4.0),
                left: LengthAuto::px(3.0),
            },
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.inputs[&2][0].run_mode(), RunMode::ComputeSize);
    assert_eq!(tree.inputs[&2][0].known().width, Some(76.0));
    assert_eq!(output.size, Size::new(100.0, 26.0));
    assert_eq!(output.content_size, Size::ZERO);
    assert!(tree.layouts.is_empty());
}

#[test]
fn block_compute_size_uses_definite_min_max_without_measuring_children() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite min/max compute-size should not measure children")
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            min_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            max_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(2, NodeInput::default());

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(500.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 40.0));
    assert_eq!(output.content_size, Size::ZERO);
}

#[test]
fn block_definite_compute_size_keeps_grid_children_on_fast_path_until_grid_baselines() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            panic!("definite grid compute-size should stay on the fast path")
        }
    }

    for display in [Display::Grid, Display::GridLanes] {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![3]);
        tree.children.insert(3, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                min_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
                max_size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display,
                ..NodeInput::default()
            },
        );
        tree.styles.insert(3, NodeInput::default());

        let output = crate::compute_block(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(500.0), Some(400.0)),
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::new(Available::definite(500.0), Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        assert_eq!(output.size, Size::new(100.0, 40.0));
        assert_eq!(output.content_size, Size::ZERO);
    }
}

#[test]
fn block_auto_height_clamps_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            max_size: Size::new(Dimension::AUTO, Dimension::px(12.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(100.0, 20.0), Size::new(100.0, 20.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(100.0, 20.0));
    assert_eq!(tree.layouts[&2].size, Size::new(100.0, 20.0));
}

#[test]
fn block_auto_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutput) {}

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(500.0), Some(400.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(50.0, 25.0));
}

#[test]
fn block_legacy_text_align_offsets_table_child_in_free_inline_space() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(ComputeOutput::from_outer_size(Size::new(60.0, 10.0)))
        }
    }

    fn run(text_align: TextAlign, direction: Direction) -> NodeOutput {
        let mut tree = BlockTree::default();
        tree.children.insert(1, vec![2]);
        tree.children.insert(2, vec![]);
        tree.styles.insert(
            1,
            NodeInput {
                display: Display::Block,
                direction,
                text_align,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::default()
            },
        );
        tree.styles.insert(
            2,
            NodeInput {
                display: Display::Block,
                item_is_table: true,
                ..NodeInput::default()
            },
        );

        crate::compute_block(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(300.0), Some(200.0)),
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                Size::new(Available::definite(300.0), Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        tree.layouts[&2]
    }

    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Ltr).location.x,
        70.0
    );
    assert_eq!(
        run(TextAlign::LegacyRight, Direction::Ltr).location.x,
        140.0
    );
    assert_eq!(
        run(TextAlign::LegacyCenter, Direction::Rtl).location.x,
        70.0
    );
    assert_eq!(run(TextAlign::LegacyLeft, Direction::Rtl).location.x, 0.0);
}

#[test]
fn block_layout_lays_out_absolute_children_without_flow_contribution_and_hides_display_none() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                if input.run_mode() == RunMode::PerformHiddenLayout {
                    ComputeOutput::HIDDEN
                } else {
                    self.outputs[&node]
                }
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            overflow: Point::new(Overflow::Visible, Overflow::Visible),
            inset: Edges {
                left: LengthAuto::px(7.0),
                top: LengthAuto::px(9.0),
                ..Edges::all(LengthAuto::AUTO)
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(40.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(80.0, 32.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(output.content_size, Size::new(87.0, 41.0));
    assert_eq!(tree.layouts[&2].location, Point::new(1.0, 1.0));
    assert_eq!(tree.layouts[&3].location, Point::new(8.0, 10.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 10.0));
    assert_eq!(tree.layouts[&4], NodeOutput::with_order(2));
    assert_eq!(
        tree.inputs[&4],
        vec![ComputeInput::hidden(crate::geometry::FlowAxes::new(
            crate::WritingMode::HorizontalTb,
            crate::Direction::Ltr,
        ))]
    );
}

#[test]
fn block_absolute_child_without_insets_uses_static_position_after_flow() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(5.0)),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(98.0, 10.0), Size::new(98.0, 10.0)),
    );
    tree.outputs.insert(
        3,
        ComputeOutput::from_sizes(Size::new(20.0, 5.0), Size::new(20.0, 5.0)),
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 12.0));
    assert_eq!(tree.layouts[&2].location, Point::new(1.0, 1.0));
    assert_eq!(tree.layouts[&3].location, Point::new(1.0, 11.0));
    assert_eq!(tree.layouts[&3].size, Size::new(20.0, 5.0));
}

#[test]
fn block_absolute_child_auto_size_applies_aspect_ratio_to_max_size() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        outputs: HashMap<u32, ComputeOutput>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            _input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok(self.outputs[&node])
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            aspect_ratio: AspectRatio::new(2.0),
            max_size: Size::new(Dimension::px(50.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.outputs.insert(
        2,
        ComputeOutput::from_sizes(Size::new(80.0, 40.0), Size::new(80.0, 40.0)),
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&2].size, Size::new(50.0, 25.0));
}

#[test]
fn block_absolute_child_auto_size_resolves_from_opposing_insets() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(50.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                right: LengthAuto::px(17.0),
                top: LengthAuto::px(13.0),
                bottom: LengthAuto::px(11.0),
            },
            size: Size::new(Dimension::AUTO, Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    let output = crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(100.0, 50.0));
    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(74.0), Some(24.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(8.0, 14.0));
    assert_eq!(tree.layouts[&2].size, Size::new(74.0, 24.0));
}

#[test]
fn block_absolute_child_applies_aspect_ratio_to_inset_derived_width() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_outer_size(Size::new(
                    input.known().width.unwrap_or(0.0),
                    input.known().height.unwrap_or(0.0),
                ))
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(10.0),
                right: LengthAuto::px(10.0),
                top: LengthAuto::AUTO,
                bottom: LengthAuto::AUTO,
            },
            aspect_ratio: AspectRatio::new(2.0),
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(80.0), Some(40.0))
    );
    assert_eq!(tree.layouts[&2].size, Size::new(80.0, 40.0));
}

#[test]
fn block_absolute_child_expands_horizontal_auto_margins() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(20.0), Some(10.0))
    );
    assert_eq!(tree.layouts[&2].margin.left, 40.0);
    assert_eq!(tree.layouts[&2].margin.right, 40.0);
    assert_eq!(tree.layouts[&2].location, Point::new(40.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[test]
fn block_absolute_child_large_width_keeps_horizontal_auto_margins_zero() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                ComputeOutput::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0))
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(0.0),
                right: LengthAuto::px(0.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(Dimension::px(70.0), Dimension::px(10.0)),
            margin: Edges {
                left: LengthAuto::AUTO,
                right: LengthAuto::AUTO,
                top: LengthAuto::ZERO,
                bottom: LengthAuto::ZERO,
            },
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(70.0), Some(10.0))
    );
    assert_eq!(tree.layouts[&2].margin.left, 0.0);
    assert_eq!(tree.layouts[&2].margin.right, 0.0);
    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 0.0));
    assert_eq!(tree.layouts[&2].size, Size::new(70.0, 10.0));
}

#[test]
fn block_absolute_child_with_opposing_horizontal_insets_honors_rtl_end_edge() {
    #[derive(Default)]
    struct BlockTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
        inputs: HashMap<u32, Vec<ComputeInput>>,
    }

    impl Traverse for BlockTree {
        type Node = u32;

        type Scalar = Scalar;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for BlockTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
            LayoutInputOf::box_input(self.node_input(node).clone())
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                self.inputs.entry(node).or_default().push(input);
                output_from_known_or(input, Size::ZERO)
            })
        }
    }

    let mut tree = BlockTree::default();
    tree.children.insert(1, vec![2]);
    tree.children.insert(2, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(Dimension::px(100.0), Dimension::px(40.0)),
            border: Edges::all(Length::px(1.0)),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            display: Display::Block,
            position: Position::Absolute,
            inset: Edges {
                left: LengthAuto::px(7.0),
                right: LengthAuto::px(17.0),
                top: LengthAuto::px(0.0),
                bottom: LengthAuto::AUTO,
            },
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::default()
        },
    );

    crate::compute_block(
        &mut tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(300.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(
        tree.inputs[&2][0].known(),
        Size::new(Some(20.0), Some(10.0))
    );
    assert_eq!(tree.layouts[&2].location, Point::new(62.0, 1.0));
    assert_eq!(tree.layouts[&2].size, Size::new(20.0, 10.0));
}

#[derive(Default)]
struct CalcLeafTree {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInput>,
    layouts: HashMap<u32, NodeOutput>,
    invalid_leaf_measurement: bool,
}

impl Traverse for CalcLeafTree {
    type Node = u32;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map_or([].as_slice(), Vec::as_slice)
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl Compute for CalcLeafTree {
    fn node_input(&self, node: Self::Node) -> &NodeInput {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
        self.layouts.insert(node, layout);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInput,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        if self.child_count(node) > 0 {
            return compute_block(self, node, input);
        }

        let style = self.styles[&node].clone();
        let invalid_leaf_measurement = self.invalid_leaf_measurement;
        compute_leaf(input, &style, |measure_input| {
            let known = measure_input.known_content_size();
            let available = measure_input
                .available_content_size()
                .map(MeasurementAvailable::into_available);
            Ok::<_, core::convert::Infallible>(Size::new(
                if invalid_leaf_measurement {
                    f32::NAN
                } else {
                    known
                        .width
                        .or_else(|| available.width.into_option())
                        .unwrap_or(0.0)
                },
                known.height.unwrap_or(10.0),
            ))
        })
        .map_err(|error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                error.operation(),
                error.kind().clone(),
            )
        })
    }
}

#[test]
fn calc_leaf_tree_propagates_leaf_measurement_error_instead_of_panicking() {
    let mut tree = CalcLeafTree {
        invalid_leaf_measurement: true,
        ..CalcLeafTree::default()
    };
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(0, NodeInput::default());
    tree.styles.insert(1, NodeInput::default());

    let error = compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::MeasurementOutput(output))
            if output.axis() == PhysicalAxis::Horizontal
    ));
}

#[test]
fn block_inline_affine_leaf_uses_public_leaf_path() {
    let mut tree = CalcLeafTree::default();
    let width = lp(10.0, 0.5);
    tree.children.insert(0, vec![1]);
    tree.children.insert(1, vec![]);
    tree.styles.insert(
        0,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::AUTO),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        1,
        NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::value(width), Dimension::AUTO),
            ..NodeInput::default()
        },
    );

    let output = compute_block(
        &mut tree,
        0,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(100.0), None),
            Size::new(Some(100.0), None),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::new(Available::definite(100.0), Available::MAX_CONTENT),
        ),
    )
    .unwrap();

    assert_eq!(tree.layouts[&1].size.width, 60.0);
    assert_eq!(output.content_size.width, 60.0);
}

#[test]
fn unresolved_symbolic_vertical_margin_is_not_treated_as_auto_margin() {
    let mut tree = CalcLeafTree::default();
    let margin = lp(0.0, 0.25);
    tree.styles.insert(
        1,
        NodeInput {
            margin: Edges {
                top: LengthAuto::value(margin),
                ..Edges::<Scalar>::ZERO.map(|_| LengthAuto::px(0.0))
            },
            ..NodeInput::default()
        },
    );

    let resolved =
        crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr)
            .zip_physical_edges_with_inline_extent(
                tree.styles[&1].margin,
                Size::new(None, None),
                |length, basis| length.resolve_auto_with_status(basis),
            );
    let resolved = resolve_in_flow_margin(resolved, Size::new(10.0, 10.0), None);

    assert_eq!(resolved.top, 0.0);
}

#[test]
fn invalid_numeric_margin_keeps_explicit_failure_without_panicking() {
    let margin = LengthAuto::value(
        LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX).expect("finite coefficients"),
    )
    .resolve_auto_with_status(Some(10.0));

    let resolved = resolve_in_flow_margin(
        Edges {
            top: margin,
            ..Edges::<Scalar>::ZERO.map(|_| ResolvedLengthAuto::Resolved(0.0))
        },
        Size::new(10.0, 10.0),
        Some(10.0),
    );

    assert_eq!(resolved.top, 0.0);
}
