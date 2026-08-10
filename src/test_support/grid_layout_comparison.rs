use std::collections::HashMap;

use super::oracle::grid::{
    AlignmentSafety, GridArea, GridItemRect, GridScenarioReport, LaneIntrinsicSizingReport,
    LaneItemRectInput, LanePlacementReport, PlacementCursor, PlacementReport, TrackAlignment,
    TrackSizingReport, align_tracks_report, compose_grid_scenario, compose_lane_item_rect,
};
use crate::test_support::layout_tree::{OracleTree, OracleTreeOf};
use crate::{
    AlignContent, AlignItems, Available, Compute, ComputeInput, ComputeOutput, ComputedOverflow,
    Direction, Display, Edges, GridAutoFlow, GridPlacement, Length, LengthAuto, NodeInput,
    NodeOutput, NodeOutputOf, Point, Position, PreferredSize, RequestedAxis, Round, RunMode, Size,
    SizingMode, SubgridTrack, TrackComponent, WritingMode, compute_grid, round_layout,
};

type Scalar = f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonTolerance {
    value: Scalar,
}

impl ComparisonTolerance {
    pub const fn browser_parity() -> Self {
        Self { value: 0.1 }
    }

    pub const fn oracle_grid() -> Self {
        Self { value: 0.000_1 }
    }

    pub fn contains(self, delta: Scalar) -> bool {
        delta.abs() <= self.value
    }
}

#[derive(Clone, Debug)]
pub struct GridLayoutComparison {
    root_display: Display,
    container: Size<f32>,
    root_size: Option<Size<PreferredSize>>,
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
    size: Size<PreferredSize>,
    justify_self: Option<AlignItems>,
    align_self: Option<AlignItems>,
    margin: Edges<LengthAuto>,
    padding: Edges<Length>,
    border: Edges<Length>,
    direction: Direction,
    writing_mode: WritingMode,
    overflow: ComputedOverflow,
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

    pub fn root_size(mut self, size: Size<PreferredSize>) -> Self {
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
        let (mut tree, identities) = self.tree();

        let output = compute_grid(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(self.container.width), Some(self.container.height)),
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();

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
            let child = &self.children[index];
            let node = identities.node(child, &[index]);
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

        walk_grid_comparison_expectations(
            &tree,
            &self.children,
            &identities,
            GridComparisonPhase::Unrounded,
        );

        if self
            .children
            .iter()
            .any(GridLayoutNode::has_expected_final_layout)
        {
            round_layout(&mut tree, 1).unwrap();
            walk_grid_comparison_expectations(
                &tree,
                &self.children,
                &identities,
                GridComparisonPhase::Final,
            );
        }
    }

    pub fn assert_layout_size(self, expected_size: Size<f32>) {
        let (mut tree, identities) = self.tree();
        let output = compute_grid(
            &mut tree,
            1,
            ComputeInput::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(None, None),
                crate::ContainingLayoutContext::new(
                    crate::geometry::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        crate::Direction::Ltr,
                    ),
                    crate::ParentFormattingContext::NoParent,
                ),
                Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
            ),
        )
        .unwrap();

        assert_size_close(output.size, expected_size);
        for (index, child) in self.children.iter().enumerate() {
            if let Some(expected) = child.expected_layout {
                let node = identities.node(child, &[index]);
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

    fn tree(&self) -> (OracleTree, GridComparisonIdentityMap<'_>) {
        let identities = build_grid_comparison_identity_map(&self.children);
        let child_nodes = self
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| identities.node(child, &[index]))
            .collect::<Vec<_>>();
        let mut tree = OracleTree::new().children(1, child_nodes);
        tree = tree.style(
            1,
            NodeInput {
                display: self.root_display,
                size: self.root_size.clone().unwrap_or_else(|| {
                    Size::new(
                        PreferredSize::px(self.container.width),
                        PreferredSize::px(self.container.height),
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

        for (node, child) in &identities.source_order {
            let child_nodes = child
                .children
                .iter()
                .enumerate()
                .map(|(index, descendant)| {
                    let mut path = identities.path(child).to_vec();
                    path.push(index);
                    identities.node(descendant, &path)
                })
                .collect::<Vec<_>>();
            tree = tree.children(*node, child_nodes);
            tree = tree.style(*node, child.node_input());
            if let Some(measurement) = child.measurement {
                tree = tree.measure(*node, ComputeOutput::from_sizes(measurement, measurement));
            }
        }

        (tree, identities)
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

    pub fn size(mut self, size: Size<PreferredSize>) -> Self {
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

    pub fn overflow(mut self, overflow: ComputedOverflow) -> Self {
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
                GridPlacement::try_line_span(
                    self.area.column_start as isize,
                    self.area.column_span,
                )
                .expect("generated grid column line/span must be valid"),
                GridPlacement::try_line_span(self.area.row_start as isize, self.area.row_span)
                    .expect("generated grid row line/span must be valid"),
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
            size: self.size.clone(),
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

#[derive(Debug, Default)]
struct GridComparisonIdentityMap<'a> {
    nodes_by_source: HashMap<*const GridLayoutNode, u32>,
    sources_by_node: HashMap<u32, *const GridLayoutNode>,
    paths_by_source: HashMap<*const GridLayoutNode, Vec<usize>>,
    source_order: Vec<(u32, &'a GridLayoutNode)>,
}

impl<'a> GridComparisonIdentityMap<'a> {
    fn record(&mut self, source: &'a GridLayoutNode, node: u32, path: &[usize]) {
        let source_identity = std::ptr::from_ref(source);
        assert!(
            !self.nodes_by_source.contains_key(&source_identity),
            "duplicate grid comparison source identity at {}",
            format_grid_comparison_path(path)
        );
        assert!(
            !self.sources_by_node.contains_key(&node),
            "duplicate grid comparison node identity {node} at {}",
            format_grid_comparison_path(path)
        );
        self.nodes_by_source.insert(source_identity, node);
        self.sources_by_node.insert(node, source_identity);
        self.paths_by_source.insert(source_identity, path.to_vec());
    }

    fn node(&self, source: &GridLayoutNode, path: &[usize]) -> u32 {
        let source_identity = std::ptr::from_ref(source);
        self.nodes_by_source
            .get(&source_identity)
            .copied()
            .unwrap_or_else(|| {
                panic!(
                    "missing grid comparison source identity at {}",
                    format_grid_comparison_path(path)
                )
            })
    }

    fn path(&self, source: &GridLayoutNode) -> &[usize] {
        let source_identity = std::ptr::from_ref(source);
        self.paths_by_source
            .get(&source_identity)
            .map(Vec::as_slice)
            .unwrap_or_else(|| panic!("grid comparison source path must be recorded"))
    }
}

fn build_grid_comparison_identity_map(
    children: &[GridLayoutNode],
) -> GridComparisonIdentityMap<'_> {
    let mut identities = GridComparisonIdentityMap::default();
    let mut next_node = 2;
    for (index, child) in children.iter().enumerate() {
        identities.record(child, next_node, &[index]);
        next_node += 1;
    }
    for (index, child) in children.iter().enumerate() {
        record_grid_comparison_descendant_identities(
            child,
            &[index],
            &mut next_node,
            &mut identities,
        );
    }
    let mut pending = children
        .iter()
        .enumerate()
        .rev()
        .map(|(index, child)| (child, vec![index]))
        .collect::<Vec<_>>();
    while let Some((child, path)) = pending.pop() {
        identities
            .source_order
            .push((identities.node(child, &path), child));
        for (index, descendant) in child.children.iter().enumerate().rev() {
            let mut descendant_path = path.clone();
            descendant_path.push(index);
            pending.push((descendant, descendant_path));
        }
    }
    identities
}

fn record_grid_comparison_descendant_identities<'a>(
    parent: &'a GridLayoutNode,
    parent_path: &[usize],
    next_node: &mut u32,
    identities: &mut GridComparisonIdentityMap<'a>,
) {
    for (index, child) in parent.children.iter().enumerate() {
        let mut path = parent_path.to_vec();
        path.push(index);
        identities.record(child, *next_node, &path);
        *next_node += 1;
    }
    for (index, child) in parent.children.iter().enumerate() {
        let mut path = parent_path.to_vec();
        path.push(index);
        record_grid_comparison_descendant_identities(child, &path, next_node, identities);
    }
}

#[derive(Clone, Copy, Debug)]
enum GridComparisonPhase {
    Unrounded,
    Final,
}

impl GridComparisonPhase {
    fn layout(self, node: &GridLayoutNode) -> Option<ExpectedLayout> {
        match self {
            Self::Unrounded => node.expected_layout,
            Self::Final => node.expected_final_layout,
        }
    }

    fn output<S: crate::LayoutScalar>(self, tree: &OracleTreeOf<S>, node: u32) -> NodeOutputOf<S> {
        match self {
            Self::Unrounded => tree
                .layout(node)
                .unwrap_or_else(|| panic!("nested layout must be recorded")),
            Self::Final => tree
                .final_layout(node)
                .unwrap_or_else(|| panic!("nested final layout must be recorded")),
        }
    }
}

fn walk_grid_comparison_expectations(
    tree: &OracleTreeOf<impl crate::LayoutScalar + std::fmt::Display>,
    children: &[GridLayoutNode],
    identities: &GridComparisonIdentityMap<'_>,
    phase: GridComparisonPhase,
) {
    let mut pending = Vec::new();
    for (root_index, root) in children.iter().enumerate().rev() {
        for (child_index, child) in root.children.iter().enumerate().rev() {
            pending.push((child, vec![root_index, child_index]));
        }
    }

    while let Some((child, path)) = pending.pop() {
        let node = identities.node(child, &path);
        if let Some(expected) = phase.layout(child) {
            assert_node_output_close_of(node, phase.output(tree, node), expected);
        }
        for (child_index, descendant) in child.children.iter().enumerate().rev() {
            let mut descendant_path = path.clone();
            descendant_path.push(child_index);
            pending.push((descendant, descendant_path));
        }
    }
}

fn format_grid_comparison_path(path: &[usize]) -> String {
    let mut formatted = String::from("root");
    for index in path {
        formatted.push('/');
        formatted.push_str(&index.to_string());
    }
    formatted
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
        GridPlacement::try_span(span).expect("generated grid span must be valid")
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

fn assert_node_output_close_of<S>(node: u32, actual: NodeOutputOf<S>, expected: ExpectedLayout)
where
    S: crate::LayoutScalar + std::fmt::Display,
{
    assert_close_of(
        actual.location.x,
        S::from_f32(expected.location.x),
        &format!("node {node} x"),
    );
    assert_close_of(
        actual.location.y,
        S::from_f32(expected.location.y),
        &format!("node {node} y"),
    );
    assert_close_of(
        actual.size.width,
        S::from_f32(expected.size.width),
        &format!("node {node} width"),
    );
    assert_close_of(
        actual.size.height,
        S::from_f32(expected.size.height),
        &format!("node {node} height"),
    );
}

fn assert_size_close(actual: Size<f32>, expected: Size<f32>) {
    assert_close(actual.width, expected.width, "root width");
    assert_close(actual.height, expected.height, "root height");
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    let tolerance = ComparisonTolerance::oracle_grid();
    assert!(
        tolerance.contains(actual - expected),
        "{label}: expected {expected}, got {actual}"
    );
}

fn assert_close_of<S>(actual: S, expected: S, label: &str)
where
    S: crate::LayoutScalar + std::fmt::Display,
{
    let tolerance = S::from_f32(ComparisonTolerance::oracle_grid().value);
    assert!(
        (actual - expected).abs() <= tolerance,
        "{label}: expected {expected}, got {actual}"
    );
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;
    use crate::{ContainingLayoutContext, LayoutScalar, ParentFormattingContext, Traverse};

    fn comparison_fixture() -> GridLayoutComparison {
        GridLayoutComparison::new()
            .node(
                GridLayoutNode::grid(GridArea::new(1, 1, 1, 1))
                    .child(
                        GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                            .measurement(Size::new(13.0, 17.0))
                            .expect_layout(Point::new(1.0, 2.0), Size::new(13.0, 17.0)),
                    )
                    .child(GridLayoutNode::item(GridArea::new(1, 1, 1, 1)))
                    .child(
                        GridLayoutNode::grid(GridArea::new(1, 1, 1, 1)).child(
                            GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                                .expect_layout(Point::new(3.0, 4.0), Size::new(19.0, 23.0))
                                .expect_final_layout(Point::new(3.0, 4.0), Size::new(19.0, 23.0)),
                        ),
                    ),
            )
            .node(
                GridLayoutNode::grid(GridArea::new(1, 1, 1, 1)).child(
                    GridLayoutNode::item(GridArea::new(1, 1, 1, 1))
                        .expect_layout(Point::new(5.0, 6.0), Size::new(29.0, 31.0)),
                ),
            )
    }

    fn compute_input() -> ComputeInput {
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(Available::MAX_CONTENT),
        )
    }

    fn panic_text(payload: Box<dyn Any + Send>) -> String {
        if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = payload.downcast_ref::<&str>() {
            (*message).to_string()
        } else {
            panic!("panic payload must contain text")
        }
    }

    fn captured_panic(action: impl FnOnce()) -> String {
        match catch_unwind(AssertUnwindSafe(action)) {
            Ok(()) => panic!("comparison action must fail"),
            Err(payload) => panic_text(payload),
        }
    }

    fn output<S: LayoutScalar>(location: (f64, f64), size: (f64, f64)) -> NodeOutputOf<S> {
        NodeOutputOf {
            location: Point::new(S::from_f64(location.0), S::from_f64(location.1)),
            size: Size::new(S::from_f64(size.0), S::from_f64(size.1)),
            ..NodeOutputOf::default()
        }
    }

    fn phase_tree<S>() -> OracleTreeOf<S>
    where
        S: LayoutScalar,
    {
        let mut tree = OracleTreeOf::new()
            .unrounded(4, output((1.0, 2.0), (13.0, 17.0)))
            .unrounded(7, output((3.0, 4.0), (19.0, 23.0)))
            .unrounded(8, output((5.0, 6.0), (29.0, 31.0)));
        <OracleTreeOf<S> as Round>::set_final(&mut tree, 7, output((3.0, 4.0), (19.0, 23.0)));
        tree
    }

    fn assert_phase_walks<S>()
    where
        S: LayoutScalar + std::fmt::Display,
    {
        let comparison = comparison_fixture();
        let identities = build_grid_comparison_identity_map(&comparison.children);
        let tree = phase_tree::<S>();

        walk_grid_comparison_expectations(
            &tree,
            &comparison.children,
            &identities,
            GridComparisonPhase::Unrounded,
        );
        walk_grid_comparison_expectations(
            &tree,
            &comparison.children,
            &identities,
            GridComparisonPhase::Final,
        );
    }

    #[test]
    fn fri08_c08_t04_comparison_walk_preserves_source_identity_child_order_and_measurements() {
        let comparison = comparison_fixture();
        let identities = build_grid_comparison_identity_map(&comparison.children);
        let (mut tree, built_identities) = comparison.tree();

        assert_eq!(
            <OracleTree as Traverse>::children(&tree, 1).collect::<Vec<_>>(),
            [2, 3]
        );
        assert_eq!(
            <OracleTree as Traverse>::children(&tree, 2).collect::<Vec<_>>(),
            [4, 5, 6]
        );
        assert_eq!(
            <OracleTree as Traverse>::children(&tree, 6).collect::<Vec<_>>(),
            [7]
        );
        assert_eq!(
            <OracleTree as Traverse>::children(&tree, 3).collect::<Vec<_>>(),
            [8]
        );
        assert_eq!(identities.node(&comparison.children[0], &[0]), 2);
        assert_eq!(identities.node(&comparison.children[1], &[1]), 3);
        assert_eq!(
            identities.node(&comparison.children[0].children[0], &[0, 0]),
            4
        );
        assert_eq!(
            identities.node(&comparison.children[0].children[2], &[0, 2]),
            6
        );
        assert_eq!(
            identities.node(&comparison.children[0].children[2].children[0], &[0, 2, 0]),
            7
        );
        assert_eq!(identities.path(&comparison.children[1].children[0]), [1, 0]);
        assert_eq!(
            built_identities
                .source_order
                .iter()
                .map(|(node, _)| *node)
                .collect::<Vec<_>>(),
            [2, 4, 5, 6, 7, 3, 8]
        );

        let measured = tree.compute_child(4, compute_input()).unwrap();
        assert_eq!(measured.size, Size::new(13.0, 17.0));
        assert_eq!(tree.inputs(4), &[compute_input()]);
    }

    #[test]
    fn fri08_c08_t04_comparison_walk_skips_absent_values_and_selects_phase_outputs_for_both_scalars()
     {
        assert_phase_walks::<f32>();
        assert_phase_walks::<f64>();
    }

    #[test]
    fn fri08_c08_t04_comparison_walk_reports_missing_and_duplicate_identities() {
        let comparison = comparison_fixture();
        let identities = build_grid_comparison_identity_map(&comparison.children);
        let missing_unrounded_message = captured_panic(|| {
            walk_grid_comparison_expectations(
                &OracleTreeOf::<f32>::new(),
                &comparison.children,
                &identities,
                GridComparisonPhase::Unrounded,
            );
        });
        assert_eq!(missing_unrounded_message, "nested layout must be recorded");
        let missing_final_message = captured_panic(|| {
            walk_grid_comparison_expectations(
                &OracleTreeOf::<f32>::new(),
                &comparison.children,
                &identities,
                GridComparisonPhase::Final,
            );
        });
        assert_eq!(
            missing_final_message,
            "nested final layout must be recorded"
        );
        assert!(ComparisonTolerance::oracle_grid().contains(0.000_1));
        assert!(!ComparisonTolerance::oracle_grid().contains(0.000_2));
        assert!(ComparisonTolerance::browser_parity().contains(0.1));
        assert!(!ComparisonTolerance::browser_parity().contains(0.2));

        let missing = GridComparisonIdentityMap::default();
        let missing_message = captured_panic(|| {
            walk_grid_comparison_expectations(
                &phase_tree::<f32>(),
                &comparison.children,
                &missing,
                GridComparisonPhase::Unrounded,
            );
        });
        assert_eq!(
            missing_message,
            "missing grid comparison source identity at root/0/0"
        );

        let first = GridLayoutNode::item(GridArea::new(1, 1, 1, 1));
        let second = GridLayoutNode::item(GridArea::new(1, 1, 1, 1));
        let mut duplicate_source = GridComparisonIdentityMap::default();
        duplicate_source.record(&first, 4, &[0, 0]);
        let duplicate_source_message = captured_panic(|| {
            duplicate_source.record(&first, 5, &[0, 1]);
        });
        assert_eq!(
            duplicate_source_message,
            "duplicate grid comparison source identity at root/0/1"
        );

        let mut duplicate_node = GridComparisonIdentityMap::default();
        duplicate_node.record(&first, 4, &[0, 0]);
        let duplicate_node_message = captured_panic(|| {
            duplicate_node.record(&second, 4, &[0, 1]);
        });
        assert_eq!(
            duplicate_node_message,
            "duplicate grid comparison node identity 4 at root/0/1"
        );
    }

    #[test]
    fn fri08_c08_t04_comparison_walk_reports_nested_mismatches_in_deterministic_preorder() {
        let comparison = comparison_fixture();
        let identities = build_grid_comparison_identity_map(&comparison.children);
        let first_message = captured_panic(|| {
            let tree = OracleTreeOf::<f64>::new()
                .unrounded(4, output((101.0, 2.0), (13.0, 17.0)))
                .unrounded(7, output((103.0, 4.0), (19.0, 23.0)))
                .unrounded(8, output((105.0, 6.0), (29.0, 31.0)));
            walk_grid_comparison_expectations(
                &tree,
                &comparison.children,
                &identities,
                GridComparisonPhase::Unrounded,
            );
        });
        assert_eq!(first_message, "node 4 x: expected 1, got 101");
        assert_eq!(identities.path(&comparison.children[0].children[0]), [0, 0]);

        let second_message = captured_panic(|| {
            let tree = OracleTreeOf::<f64>::new()
                .unrounded(4, output((1.0, 2.0), (13.0, 17.0)))
                .unrounded(7, output((103.0, 4.0), (19.0, 23.0)))
                .unrounded(8, output((105.0, 6.0), (29.0, 31.0)));
            walk_grid_comparison_expectations(
                &tree,
                &comparison.children,
                &identities,
                GridComparisonPhase::Unrounded,
            );
        });
        assert_eq!(second_message, "node 7 x: expected 3, got 103");
        assert_eq!(
            identities.path(&comparison.children[0].children[2].children[0]),
            [0, 2, 0]
        );
    }
}
