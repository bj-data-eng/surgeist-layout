use crate::support::oracle::grid::{
    AlignmentSafety, GridArea, GridItemRect, GridScenarioReport, LaneIntrinsicSizingReport,
    LaneItemRectInput, LanePlacementReport, PlacementCursor, PlacementReport, TrackAlignment,
    TrackSizingReport, align_tracks_report, compose_grid_scenario, compose_lane_item_rect,
};
use crate::support::oracle_tree::OracleTree;
use surgeist_layout::{
    AlignContent, AlignItems, Available, Compute, ComputeInput, ComputeOutput, Dimension,
    Direction, Display, Edges, GridAutoFlow, GridPlacement, Length, LengthAuto, NodeInput,
    NodeOutput, Overflow, Point, Position, RequestedAxis, RunMode, Size, SizingMode, SubgridTrack,
    TrackComponent, WritingMode, compute_grid, round_layout,
};

#[derive(Clone, Debug)]
pub struct GridLayoutComparison {
    root_display: Display,
    container: Size<f32>,
    root_size: Option<Size<Dimension>>,
    columns: Vec<TrackComponent>,
    rows: Vec<TrackComponent>,
    gap: Size<f32>,
    justify_content: AlignContent,
    align_content: AlignContent,
    auto_flow: GridAutoFlow,
    expected_columns: Option<TrackSizingReport>,
    expected_rows: Option<TrackSizingReport>,
    expected_lane_placement_reports: Vec<LanePlacementReport>,
    expected_lane_intrinsic_reports: Vec<LaneIntrinsicSizingReport>,
    children: Vec<GridLayoutNode>,
}

#[derive(Clone, Debug)]
pub struct GridLayoutNode {
    area: GridArea,
    measurement: Option<Size<f32>>,
    placement: ChildPlacement,
    display: Display,
    size: Size<Dimension>,
    justify_self: Option<AlignItems>,
    align_self: Option<AlignItems>,
    margin: Edges<LengthAuto>,
    padding: Edges<Length>,
    border: Edges<Length>,
    direction: Direction,
    writing_mode: WritingMode,
    overflow: Point<Overflow>,
    position: Position,
    columns: Vec<TrackComponent>,
    rows: Vec<TrackComponent>,
    gap: Size<Length>,
    expected_layout: Option<ExpectedLayout>,
    expected_final_layout: Option<ExpectedLayout>,
    children: Vec<GridLayoutNode>,
}

#[derive(Clone, Copy, Debug)]
enum ChildPlacement {
    Explicit,
    Auto,
    AutoSpan { column_span: usize, row_span: usize },
}

#[derive(Clone, Copy, Debug)]
struct ExpectedLayout {
    location: Point<f32>,
    size: Size<f32>,
}

impl Default for GridLayoutComparison {
    fn default() -> Self {
        Self {
            root_display: Display::Grid,
            container: Size::new(0.0, 0.0),
            root_size: None,
            columns: Vec::new(),
            rows: Vec::new(),
            gap: Size::new(0.0, 0.0),
            justify_content: AlignContent::Start,
            align_content: AlignContent::Start,
            auto_flow: GridAutoFlow::Row,
            expected_columns: None,
            expected_rows: None,
            expected_lane_placement_reports: Vec::new(),
            expected_lane_intrinsic_reports: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl GridLayoutComparison {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn container(mut self, container: Size<f32>) -> Self {
        self.container = container;
        self
    }

    pub fn root_display(mut self, display: Display) -> Self {
        assert!(
            matches!(display.inner_display(), Display::Grid | Display::GridLanes),
            "grid layout comparisons require a grid-like root display"
        );
        self.root_display = display;
        self
    }

    pub fn root_size(mut self, size: Size<Dimension>) -> Self {
        self.root_size = Some(size);
        self
    }

    pub fn columns(mut self, columns: Vec<TrackComponent>) -> Self {
        self.columns = columns;
        self
    }

    pub fn rows(mut self, rows: Vec<TrackComponent>) -> Self {
        self.rows = rows;
        self
    }

    pub fn gap(mut self, gap: Size<f32>) -> Self {
        self.gap = gap;
        self
    }

    pub fn justify_content(mut self, justify_content: AlignContent) -> Self {
        self.justify_content = justify_content;
        self
    }

    pub fn align_content(mut self, align_content: AlignContent) -> Self {
        self.align_content = align_content;
        self
    }

    pub fn auto_flow(mut self, auto_flow: GridAutoFlow) -> Self {
        self.auto_flow = auto_flow;
        self
    }

    pub fn expected_tracks(mut self, columns: TrackSizingReport, rows: TrackSizingReport) -> Self {
        self.expected_columns = Some(columns);
        self.expected_rows = Some(rows);
        self
    }

    pub fn expect_lane_placement_report(mut self, report: LanePlacementReport) -> Self {
        self.expected_lane_placement_reports.push(report);
        self
    }

    pub fn expect_lane_intrinsic_report(mut self, report: LaneIntrinsicSizingReport) -> Self {
        self.expected_lane_intrinsic_reports.push(report);
        self
    }

    pub fn assert_lane_reports(
        &self,
        placement_reports: &[LanePlacementReport],
        intrinsic_reports: &[LaneIntrinsicSizingReport],
    ) {
        assert_eq!(
            placement_reports, self.expected_lane_placement_reports,
            "lane placement reports"
        );
        assert_eq!(
            intrinsic_reports, self.expected_lane_intrinsic_reports,
            "lane intrinsic reports"
        );
    }

    pub fn child(mut self, area: GridArea) -> Self {
        self.children.push(GridLayoutNode::item(area));
        self
    }

    pub fn auto_child(mut self, expected_area: GridArea) -> Self {
        self.children.push(GridLayoutNode::auto_item(expected_area));
        self
    }

    pub fn auto_spanning_child(
        mut self,
        expected_area: GridArea,
        column_span: usize,
        row_span: usize,
    ) -> Self {
        assert!(column_span > 0, "column span must be positive");
        assert!(row_span > 0, "row span must be positive");
        self.children.push(GridLayoutNode::auto_spanning_item(
            expected_area,
            column_span,
            row_span,
        ));
        self
    }

    pub fn measured_child(mut self, area: GridArea, measurement: Size<f32>) -> Self {
        self.children
            .push(GridLayoutNode::item(area).measurement(measurement));
        self
    }

    pub fn node(mut self, node: GridLayoutNode) -> Self {
        self.children.push(node);
        self
    }

    pub fn assert_layout(self) {
        let expected_columns = self
            .expected_columns
            .clone()
            .expect("expected columns must be supplied");
        let expected_rows = self
            .expected_rows
            .clone()
            .expect("expected rows must be supplied");
        let scenario = self.expected_scenario(expected_columns, expected_rows);
        let mut tree = self.tree();

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(Some(self.container.width), Some(self.container.height)),
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
        );

        assert_size_close(output.size, self.container);
        tree.set_unrounded(
            1,
            NodeOutput {
                size: output.size,
                content_size: output.content_size,
                ..NodeOutput::new()
            },
        );
        for (index, expected) in scenario.item_rects.iter().enumerate() {
            let node = (index + 2) as u32;
            let child = &self.children[index];
            let actual = tree.layout(node).expect("child layout must be recorded");
            if let Some(expected_layout) = child.expected_layout {
                assert_node_output_close(node, actual.location, actual.size, expected_layout);
            } else {
                let expected_size = child
                    .measurement
                    .unwrap_or_else(|| Size::new(expected.width, expected.height));
                assert_rect_close(node, actual.location, actual.size, *expected, expected_size);
            }
        }

        let mut next_node = 2 + self.children.len() as u32;
        for child in &self.children {
            assert_nested_expected_layouts(&tree, child, &mut next_node);
        }

        if self
            .children
            .iter()
            .any(GridLayoutNode::has_expected_final_layout)
        {
            round_layout(&mut tree, 1);
            let mut next_node = 2 + self.children.len() as u32;
            for child in &self.children {
                assert_nested_expected_final_layouts(&tree, child, &mut next_node);
            }
        }
    }

    pub fn assert_layout_size(self, expected_size: Size<f32>) {
        let mut tree = self.tree();
        let output = compute_grid(
            &mut tree,
            1,
            ComputeInput {
                run_mode: RunMode::PerformLayout,
                sizing_mode: SizingMode::InherentSize,
                axis: RequestedAxis::Both,
                known: Size::NONE,
                parent: Size::new(None, None),
                available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            },
        );

        assert_size_close(output.size, expected_size);
        for (index, child) in self.children.iter().enumerate() {
            if let Some(expected) = child.expected_layout {
                let node = (index + 2) as u32;
                let actual = tree.layout(node).expect("child layout must be recorded");
                assert_node_output_close(node, actual.location, actual.size, expected);
            }
        }
    }

    fn expected_scenario(
        &self,
        expected_columns: TrackSizingReport,
        expected_rows: TrackSizingReport,
    ) -> GridScenarioReport {
        let placement = PlacementReport {
            areas: self.children.iter().map(|child| child.area).collect(),
            occupied: Vec::new(),
            implicit_columns_before: 0,
            implicit_columns_after: 0,
            implicit_rows_before: 0,
            implicit_rows_after: 0,
            cursor: PlacementCursor { column: 1, row: 1 },
        };
        let column_alignment = align_tracks_report(
            self.container.width,
            expected_columns
                .final_tracks
                .iter()
                .map(|track| track.size)
                .collect(),
            self.gap.width,
            track_alignment(self.justify_content),
            alignment_safety(self.justify_content),
        );
        let row_alignment = align_tracks_report(
            self.container.height,
            expected_rows
                .final_tracks
                .iter()
                .map(|track| track.size)
                .collect(),
            self.gap.height,
            track_alignment(self.align_content),
            alignment_safety(self.align_content),
        );

        compose_grid_scenario(
            placement,
            expected_columns,
            expected_rows,
            column_alignment,
            row_alignment,
        )
    }

    fn tree(&self) -> OracleTree {
        let child_nodes: Vec<u32> = (0..self.children.len())
            .map(|index| (index + 2) as u32)
            .collect();
        let mut tree = OracleTree::new().children(1, child_nodes);
        tree = tree.style(
            1,
            NodeInput {
                display: self.root_display,
                size: self.root_size.unwrap_or_else(|| {
                    Size::new(
                        Dimension::px(self.container.width),
                        Dimension::px(self.container.height),
                    )
                }),
                grid_template_columns: self.columns.clone(),
                grid_template_rows: self.rows.clone(),
                justify_content: Some(self.justify_content),
                align_content: Some(self.align_content),
                grid_auto_flow: self.auto_flow,
                gap: Size::new(Length::px(self.gap.width), Length::px(self.gap.height)),
                ..NodeInput::default()
            },
        );

        let mut next_node = 2 + self.children.len() as u32;
        for (index, child) in self.children.iter().enumerate() {
            let node = (index + 2) as u32;
            tree = append_node(tree, node, child, &mut next_node);
        }

        tree
    }
}

impl GridLayoutNode {
    pub fn item(area: GridArea) -> Self {
        Self::new(area, ChildPlacement::Explicit)
    }

    pub fn auto_item(expected_area: GridArea) -> Self {
        Self::new(expected_area, ChildPlacement::Auto)
    }

    pub fn auto_spanning_item(
        expected_area: GridArea,
        column_span: usize,
        row_span: usize,
    ) -> Self {
        assert!(column_span > 0, "column span must be positive");
        assert!(row_span > 0, "row span must be positive");
        Self::new(
            expected_area,
            ChildPlacement::AutoSpan {
                column_span,
                row_span,
            },
        )
    }

    pub fn grid(area: GridArea) -> Self {
        Self::item(area).display(Display::Grid)
    }

    pub fn grid_lanes(area: GridArea) -> Self {
        Self::item(area).display(Display::GridLanes)
    }

    pub fn subgrid(area: GridArea) -> Self {
        Self::grid(area)
            .columns(vec![subgrid_track()])
            .rows(vec![subgrid_track()])
    }

    fn new(area: GridArea, placement: ChildPlacement) -> Self {
        let default = NodeInput::default();
        Self {
            area,
            measurement: None,
            placement,
            display: default.display,
            size: default.size,
            justify_self: default.justify_self,
            align_self: default.align_self,
            margin: default.margin,
            padding: default.padding,
            border: default.border,
            direction: default.direction,
            writing_mode: default.writing_mode,
            overflow: default.overflow,
            position: default.position,
            columns: default.grid_template_columns,
            rows: default.grid_template_rows,
            gap: default.gap,
            expected_layout: None,
            expected_final_layout: None,
            children: Vec::new(),
        }
    }

    pub fn display(mut self, display: Display) -> Self {
        self.display = display;
        self
    }

    pub fn size(mut self, size: Size<Dimension>) -> Self {
        self.size = size;
        self
    }

    pub fn justify_self(mut self, justify_self: AlignItems) -> Self {
        self.justify_self = Some(justify_self);
        self
    }

    pub fn align_self(mut self, align_self: AlignItems) -> Self {
        self.align_self = Some(align_self);
        self
    }

    pub fn measurement(mut self, measurement: Size<f32>) -> Self {
        self.measurement = Some(measurement);
        self
    }

    pub fn margin(mut self, margin: Edges<LengthAuto>) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: Edges<Length>) -> Self {
        self.padding = padding;
        self
    }

    pub fn border(mut self, border: Edges<Length>) -> Self {
        self.border = border;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    pub fn overflow(mut self, overflow: Point<Overflow>) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn columns(mut self, columns: Vec<TrackComponent>) -> Self {
        self.columns = columns;
        self
    }

    pub fn rows(mut self, rows: Vec<TrackComponent>) -> Self {
        self.rows = rows;
        self
    }

    pub fn gap(mut self, gap: Size<Length>) -> Self {
        self.gap = gap;
        self
    }

    pub fn expect_layout(mut self, location: Point<f32>, size: Size<f32>) -> Self {
        self.expected_layout = Some(ExpectedLayout { location, size });
        self
    }

    pub fn expect_final_layout(mut self, location: Point<f32>, size: Size<f32>) -> Self {
        self.expected_final_layout = Some(ExpectedLayout { location, size });
        self
    }

    pub fn expect_lane_rect(self, input: LaneItemRectInput) -> Self {
        let rect = compose_lane_item_rect(input);
        self.expect_layout(
            Point::new(rect.x, rect.y),
            Size::new(rect.width, rect.height),
        )
    }

    pub fn child(mut self, child: GridLayoutNode) -> Self {
        self.children.push(child);
        self
    }

    fn grid_placement(&self) -> (GridPlacement, GridPlacement) {
        match self.placement {
            ChildPlacement::Explicit => (
                GridPlacement::line_span(self.area.column_start as isize, self.area.column_span),
                GridPlacement::line_span(self.area.row_start as isize, self.area.row_span),
            ),
            ChildPlacement::Auto => (GridPlacement::AUTO, GridPlacement::AUTO),
            ChildPlacement::AutoSpan {
                column_span,
                row_span,
            } => (
                auto_span_placement(column_span),
                auto_span_placement(row_span),
            ),
        }
    }

    fn node_input(&self) -> NodeInput {
        let (grid_column, grid_row) = self.grid_placement();
        NodeInput {
            display: self.display,
            direction: self.direction,
            writing_mode: self.writing_mode,
            overflow: self.overflow,
            position: self.position,
            size: self.size,
            justify_self: self.justify_self,
            align_self: self.align_self,
            margin: self.margin,
            padding: self.padding,
            border: self.border,
            grid_template_columns: self.columns.clone(),
            grid_template_rows: self.rows.clone(),
            gap: self.gap,
            grid_column,
            grid_row,
            ..NodeInput::default()
        }
    }

    fn has_expected_final_layout(&self) -> bool {
        self.expected_final_layout.is_some()
            || self
                .children
                .iter()
                .any(GridLayoutNode::has_expected_final_layout)
    }
}

fn append_node(
    mut tree: OracleTree,
    node: u32,
    child: &GridLayoutNode,
    next_node: &mut u32,
) -> OracleTree {
    let child_nodes: Vec<u32> = (0..child.children.len())
        .map(|_| {
            let node = *next_node;
            *next_node += 1;
            node
        })
        .collect();
    tree = tree.children(node, child_nodes.clone());
    tree = tree.style(node, child.node_input());
    if let Some(measurement) = child.measurement {
        tree = tree.measure(node, ComputeOutput::from_sizes(measurement, measurement));
    }
    for (node, child) in child_nodes.into_iter().zip(&child.children) {
        tree = append_node(tree, node, child, next_node);
    }
    tree
}

fn assert_nested_expected_layouts(tree: &OracleTree, parent: &GridLayoutNode, next_node: &mut u32) {
    let child_nodes: Vec<u32> = (0..parent.children.len())
        .map(|_| {
            let node = *next_node;
            *next_node += 1;
            node
        })
        .collect();

    for (node, child) in child_nodes.into_iter().zip(&parent.children) {
        if let Some(expected) = child.expected_layout {
            let actual = tree.layout(node).expect("nested layout must be recorded");
            assert_node_output_close(node, actual.location, actual.size, expected);
        }
        assert_nested_expected_layouts(tree, child, next_node);
    }
}

fn assert_nested_expected_final_layouts(
    tree: &OracleTree,
    parent: &GridLayoutNode,
    next_node: &mut u32,
) {
    let child_nodes: Vec<u32> = (0..parent.children.len())
        .map(|_| {
            let node = *next_node;
            *next_node += 1;
            node
        })
        .collect();

    for (node, child) in child_nodes.into_iter().zip(&parent.children) {
        if let Some(expected) = child.expected_final_layout {
            let actual = tree
                .final_layout(node)
                .expect("nested final layout must be recorded");
            assert_node_output_close(node, actual.location, actual.size, expected);
        }
        assert_nested_expected_final_layouts(tree, child, next_node);
    }
}

fn track_alignment(alignment: AlignContent) -> TrackAlignment {
    match alignment {
        AlignContent::Start | AlignContent::FlexStart | AlignContent::Stretch => {
            TrackAlignment::Start
        }
        AlignContent::End
        | AlignContent::FlexEnd
        | AlignContent::SafeEnd
        | AlignContent::SafeFlexEnd => TrackAlignment::End,
        AlignContent::Center | AlignContent::SafeCenter => TrackAlignment::Center,
        AlignContent::SpaceBetween => TrackAlignment::SpaceBetween,
        AlignContent::SpaceAround => TrackAlignment::SpaceAround,
        AlignContent::SpaceEvenly => TrackAlignment::SpaceEvenly,
    }
}

fn alignment_safety(alignment: AlignContent) -> AlignmentSafety {
    match alignment {
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
            AlignmentSafety::Safe
        }
        _ => AlignmentSafety::Unsafe,
    }
}

fn auto_span_placement(span: usize) -> GridPlacement {
    if span == 1 {
        GridPlacement::AUTO
    } else {
        GridPlacement::span(span)
    }
}

fn subgrid_track() -> TrackComponent {
    TrackComponent::Subgrid(SubgridTrack {
        name_components: Vec::new(),
    })
}

fn assert_rect_close(
    node: u32,
    location: Point<f32>,
    size: Size<f32>,
    expected: GridItemRect,
    expected_size: Size<f32>,
) {
    assert_close(location.x, expected.x, &format!("node {node} x"));
    assert_close(location.y, expected.y, &format!("node {node} y"));
    assert_close(
        size.width,
        expected_size.width,
        &format!("node {node} width"),
    );
    assert_close(
        size.height,
        expected_size.height,
        &format!("node {node} height"),
    );
}

fn assert_node_output_close(
    node: u32,
    location: Point<f32>,
    size: Size<f32>,
    expected: ExpectedLayout,
) {
    assert_close(location.x, expected.location.x, &format!("node {node} x"));
    assert_close(location.y, expected.location.y, &format!("node {node} y"));
    assert_close(
        size.width,
        expected.size.width,
        &format!("node {node} width"),
    );
    assert_close(
        size.height,
        expected.size.height,
        &format!("node {node} height"),
    );
}

fn assert_size_close(actual: Size<f32>, expected: Size<f32>) {
    assert_close(actual.width, expected.width, "root width");
    assert_close(actual.height, expected.height, "root height");
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "{label}: expected {expected}, got {actual}"
    );
}
