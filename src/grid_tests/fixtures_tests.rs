use super::*;

macro_rules! grid_container_projection {
    ($input:expr) => {
        GridContainerProjection::from_node($input)
    };
}

macro_rules! grid_item_projection {
    ($input:expr) => {
        GridItemProjection::from_node($input)
    };
}

macro_rules! grid_child_input {
    ($input:expr) => {
        GridChildInput::from_node($input)
    };
}

pub(super) fn default_grid_item_projection<S: LayoutScalar>() -> GridItemProjection<S> {
    grid_item_projection!(&NodeInputOf::default())
}

pub(super) fn single_grid_placement_context<S: LayoutScalar>(
    child: u32,
    style: &NodeInputOf<S>,
) -> GridPlacementContext<u32, S> {
    GridPlacementContext::new_with_child_inputs(
        vec![child],
        vec![ResolvedGridItemPlacement {
            column: GridPlacement::AUTO,
            row: GridPlacement::AUTO,
            absolute_column: GridPlacement::AUTO,
            absolute_row: GridPlacement::AUTO,
            in_flow: true,
        }],
        vec![grid_child_input!(style)],
    )
}

pub(super) fn subgrid_axis_report<S: LayoutScalar>(
    parent_style: &NodeInputOf<S>,
    child_style: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> SubgridAxisReport {
    super::subgrid_axis_report(
        &grid_container_projection!(parent_style),
        &grid_child_input!(child_style),
        axis,
    )
}

#[derive(Clone, Copy)]
pub(super) struct GridAxisMappingInput<'a, S: LayoutScalar = Scalar> {
    pub(super) queried_axis: GridAxisKind,
    pub(super) parent_style: &'a NodeInputOf<S>,
    pub(super) child_style: &'a NodeInputOf<S>,
}

pub(super) fn map_grid_axis<S: LayoutScalar>(
    input: GridAxisMappingInput<'_, S>,
) -> GridAxisMappingReport {
    super::map_grid_axis(super::axis::GridAxisMappingInput {
        queried_axis: input.queried_axis,
        parent_style: &grid_container_projection!(input.parent_style),
        child_style: &grid_item_projection!(input.child_style),
    })
}

#[derive(Clone, Copy)]
pub(super) struct SubgridEligibilityInput<'a, S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) parent_style: &'a NodeInputOf<S>,
    pub(super) has_parent_grid: bool,
    pub(super) child_style: &'a NodeInputOf<S>,
}

pub(super) fn subgrid_eligibility<S: LayoutScalar>(
    input: SubgridEligibilityInput<'_, S>,
) -> SubgridEligibility {
    let child_input = grid_child_input!(input.child_style);
    super::subgrid_eligibility(super::subgrid::SubgridEligibilityInput {
        axis: input.axis,
        parent_style: &grid_container_projection!(input.parent_style),
        has_parent_grid: input.has_parent_grid,
        child_style: child_input.item(),
        child_requested: child_input.subgrid_requested(input.axis),
    })
}

#[derive(Clone, Copy)]
pub(super) struct SubgridChildParentContextInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) item: SubgridItemReport<Node>,
    pub(super) child_style: &'a NodeInputOf<S>,
    pub(super) area: GridArea<S>,
    pub(super) content_box_size: Size<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) parent_named_columns: &'a NamedGridLines,
    pub(super) parent_named_rows: &'a NamedGridLines,
    pub(super) parent_area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) parent_baseline_groups: &'a GridBaselineGroups<S>,
    pub(super) margin: Edges<Option<S>>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
}

pub(super) fn with_projected_subgrid_child_input<Node: Copy, S: LayoutScalar, R>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    consume: impl FnOnce(super::child::SubgridChildParentContextInput<'_, Node, S>) -> R,
) -> R {
    let child_input = grid_child_input!(input.child_style);
    let child_container_style = child_input
        .nested_container()
        .expect("test subgrid child must retain its container projection input")
        .projection();
    consume(super::child::SubgridChildParentContextInput {
        item: input.item,
        child_style: child_input.item(),
        child_container_style: Some(child_container_style),
        area: input.area,
        content_box_size: input.content_box_size,
        columns: input.columns,
        rows: input.rows,
        gap: input.gap,
        parent_named_columns: input.parent_named_columns,
        parent_named_rows: input.parent_named_rows,
        parent_area_facts: input.parent_area_facts,
        parent_baseline_groups: input.parent_baseline_groups,
        margin: input.margin,
        border: input.border,
        padding: input.padding,
    })
}

pub(super) fn subgrid_child_parent_context<Node: Copy + PartialEq, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'_, Node, S>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>> {
    with_projected_subgrid_child_input(input, super::child::subgrid_child_parent_context)
}

pub(super) fn subgrid_child_parent_context_with_geometry<
    Node: Copy + PartialEq,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'_, Node, S>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>> {
    with_projected_subgrid_child_input(input, |input| {
        super::child::subgrid_child_parent_context_with_geometry(
            input,
            column_geometry,
            row_geometry,
        )
    })
}

pub(super) fn fri08_c01_placement_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::Definite(S::from_f64(240.0)),
        AvailableOf::Definite(S::from_f64(240.0)),
    ))
    .expect("finite placement viewport")
}

pub(super) fn fri08_c01_placement_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("placement output for node {node}"))
        .output()
}

pub(super) fn fri08_c01_placement_compute<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(tree, 1, fri08_c01_placement_request()).expect("valid grid placement")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C06RInheritedAxes {
    Columns,
    Rows,
    Both,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct GridTestRetainedState<S: LayoutScalar> {
    pub(super) unrounded: HashMap<u32, NodeOutputOf<S>>,
    pub(super) final_outputs: HashMap<u32, NodeOutputOf<S>>,
    pub(super) caches: HashMap<u32, CacheOf<S>>,
}

impl<S: LayoutScalar> GridTestRetainedState<S> {
    pub(super) fn prepare_grid_test_batch(&self, batch: &CompletedLayoutBatchOf<u32, S>) -> Self {
        let mut prepared = self.clone();
        for node in batch.invalidated_nodes() {
            prepared.unrounded.remove(node);
            prepared.final_outputs.remove(node);
            prepared.caches.remove(node);
        }
        for entry in batch.unrounded_entries() {
            prepared.unrounded.insert(entry.node(), entry.output());
        }
        for entry in batch.final_entries() {
            prepared.final_outputs.insert(entry.node(), entry.output());
        }
        for entry in batch.cache_clear_entries() {
            prepared.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            prepared
                .caches
                .entry(entry.node())
                .or_default()
                .store_with_context(entry.input(), entry.context(), entry.output());
        }
        prepared
    }
}

#[derive(Clone, Debug)]
pub(super) struct Fri08C06RAtomicTree<S: LayoutScalar> {
    pub(super) tree: PublicLayoutTreeOf<S>,
    pub(super) cache_queries: std::cell::RefCell<Vec<(u32, bool)>>,
    pub(super) retained: GridTestRetainedState<S>,
}

impl<S: LayoutScalar> Fri08C06RAtomicTree<S> {
    pub(super) fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            cache_queries: std::cell::RefCell::new(Vec::new()),
            retained: GridTestRetainedState::default(),
        }
    }

    pub(super) fn request() -> LayoutRootRequestOf<S> {
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::definite(S::from_f64(200.0))))
            .expect("finite inherited-placement viewport")
    }
}

impl<S: LayoutScalar> Traverse for Fri08C06RAtomicTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri08C06RAtomicTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.tree.has_leaf_measurement(node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.tree.measure_leaf(node, input)
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri08C06RAtomicTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = GridTestRetainedState<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        Ok(self.retained.prepare_grid_test_batch(batch))
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

pub(super) fn fri08_c06r_assert_cold_warm<S: LayoutScalar>(
    tree: PublicLayoutTreeOf<S>,
    expected_nodes: &[u32],
    assert_geometry: impl Fn(&CompletedLayoutBatchOf<u32, S>),
) {
    let mut tree = Fri08C06RAtomicTree::new(tree);
    let request = Fri08C06RAtomicTree::<S>::request();
    let cold = compute_layout(&tree, 1, request).expect("cold inherited placement succeeds");
    assert_geometry(&cold);
    assert_eq!(
        cold.final_entries()
            .iter()
            .map(LayoutOutputEntryOf::node)
            .collect::<Vec<_>>(),
        expected_nodes,
        "the successful batch publishes every source node exactly once"
    );
    let cold_unrounded = cold.unrounded_entries().to_vec();
    let cold_final = cold.final_entries().to_vec();
    cold.apply_to(&mut tree)
        .expect("cold inherited-placement batch commits atomically");

    tree.cache_queries.borrow_mut().clear();
    let warm = compute_layout(&tree, 1, request).expect("warm inherited placement succeeds");
    assert_geometry(&warm);
    assert_eq!(warm.unrounded_entries(), cold_unrounded);
    assert_eq!(warm.final_entries(), cold_final);
    assert!(
        tree.cache_queries.borrow().iter().any(|(_, hit)| *hit),
        "warm inherited placement must reuse committed cache state"
    );
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Fri08C03LanesTracks {
    Rows,
    Columns,
}

pub(super) fn fri08_c03_containing_block_percentage_child<S: LayoutScalar>(
    tracks: Fri08C03LanesTracks,
    writing_mode: WritingMode,
    direction: Direction,
    box_sizing: BoxSizing,
) -> (NodeOutputOf<S>, NodeOutputOf<S>) {
    let scalar = S::from_f64;
    let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
    let logical_container_size = LogicalSizeOf::new(scalar(100.0), scalar(80.0));
    let physical_container_size = flow_axes.physical_size(logical_container_size);
    let percentage = PreferredSizeOf::percent(S::ONE);
    let fixed = PreferredSizeOf::px(scalar(40.0));
    let logical_child_size = match tracks {
        Fri08C03LanesTracks::Rows => LogicalSizeOf::new(percentage, fixed),
        Fri08C03LanesTracks::Columns => LogicalSizeOf::new(fixed, percentage),
    };
    let physical_child_size = flow_axes.physical_size(logical_child_size);
    let (columns, rows) = match tracks {
        Fri08C03LanesTracks::Rows => (Vec::new(), vec![TrackComponentOf::px(scalar(40.0))]),
        Fri08C03LanesTracks::Columns => (vec![TrackComponentOf::px(scalar(40.0))], Vec::new()),
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::GridLanes,
                writing_mode,
                direction,
                box_sizing,
                size: physical_container_size.map(PreferredSizeOf::px),
                grid_template_columns: columns,
                grid_template_rows: rows,
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                box_sizing,
                size: physical_child_size,
                ..NodeInputOf::default()
            },
        )
        .measure(2, Size::ZERO);
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        ))
        .expect("max-content lanes viewport is valid"),
    );
    assert!(
        batch.is_ok(),
        "hybrid lanes containing block must resolve percentage child sizing: {batch:?}"
    );
    let batch = batch.expect("asserted successful lanes layout");
    (
        fri08_c01_placement_output(&batch, 1),
        fri08_c01_placement_output(&batch, 2),
    )
}

pub(super) fn fri08_c03_containing_block_rows_child<S: LayoutScalar>(
    child_style: NodeInputOf<S>,
    measured_size: Size<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let scalar = S::from_f64;
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInputOf {
                display: Display::GridLanes,
                size: Size::new(
                    PreferredSizeOf::px(scalar(100.0)),
                    PreferredSizeOf::px(scalar(80.0)),
                ),
                grid_template_rows: vec![TrackComponentOf::px(scalar(80.0))],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(2, child_style)
        .measure(2, measured_size);
    compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        ))
        .expect("max-content lanes viewport is valid"),
    )
    .expect("hybrid containing-block control layout succeeds")
}

pub(super) fn assert_fri08_c03_containing_block_percentage_children<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let (container, child) = fri08_c03_containing_block_percentage_child::<S>(
                Fri08C03LanesTracks::Rows,
                WritingMode::HorizontalTb,
                direction,
                box_sizing,
            );
            assert_eq!(container.size, Size::new(scalar(100.0), scalar(80.0)));
            assert_eq!(
                child.size,
                Size::new(scalar(100.0), scalar(40.0)),
                "rows-only {box_sizing:?} {direction:?} child must use the 100px content width"
            );
            assert_eq!(
                child.location,
                Point::ZERO,
                "rows-only {box_sizing:?} {direction:?} child must start at the content origin"
            );
        }
    }

    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for tracks in [Fri08C03LanesTracks::Rows, Fri08C03LanesTracks::Columns] {
                let (container, child) = fri08_c03_containing_block_percentage_child::<S>(
                    tracks,
                    writing_mode,
                    direction,
                    BoxSizing::BorderBox,
                );
                let flow_axes = crate::geometry::FlowAxes::new(writing_mode, direction);
                assert_eq!(
                    flow_axes.logical_size(container.size),
                    LogicalSizeOf::new(scalar(100.0), scalar(80.0))
                );
                let expected_child = match tracks {
                    Fri08C03LanesTracks::Rows => LogicalSizeOf::new(scalar(100.0), scalar(40.0)),
                    Fri08C03LanesTracks::Columns => LogicalSizeOf::new(scalar(40.0), scalar(80.0)),
                };
                assert_eq!(
                    flow_axes.logical_size(child.size),
                    expected_child,
                    "{writing_mode:?} {direction:?} {tracks:?} must preserve the hybrid logical axes"
                );
            }
        }
    }
}

pub(super) fn assert_fri08_c03_containing_block_percentage_controls<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let percent = |fraction| {
        LengthPercentageOf::from_percent_fraction(scalar(fraction))
            .expect("finite percentage control")
    };

    let edges = fri08_c03_containing_block_rows_child(
        NodeInputOf {
            display: Display::Block,
            box_sizing: BoxSizing::ContentBox,
            size: Size::new(
                PreferredSizeOf::value(percent(0.5)),
                PreferredSizeOf::px(scalar(20.0)),
            ),
            min_size: Size::new(MinSizeOf::value(percent(0.4)), MinSizeOf::AUTO),
            max_size: Size::new(MaxSizeOf::value(percent(0.6)), MaxSizeOf::NONE),
            margin: Edges::new(
                LengthAutoOf::ZERO,
                LengthAutoOf::value(percent(0.05)),
                LengthAutoOf::ZERO,
                LengthAutoOf::value(percent(0.05)),
            ),
            padding: Edges::new(
                LengthOf::ZERO,
                LengthOf::value(percent(0.1)),
                LengthOf::ZERO,
                LengthOf::value(percent(0.1)),
            ),
            border: Edges::new(
                LengthOf::ZERO,
                LengthOf::px(scalar(2.0)),
                LengthOf::ZERO,
                LengthOf::px(scalar(2.0)),
            ),
            justify_self: Some(AlignItems::Start),
            align_self: Some(AlignItems::Start),
            ..NodeInputOf::default()
        },
        Size::ZERO,
    );
    let edges = fri08_c01_placement_output(&edges, 2);
    assert_eq!(edges.size, Size::new(scalar(74.0), scalar(20.0)));
    assert_eq!(edges.location, Point::new(scalar(5.0), S::ZERO));
    assert_eq!(
        edges.padding,
        Edges::new(S::ZERO, scalar(10.0), S::ZERO, scalar(10.0),)
    );
    assert_eq!(
        edges.margin,
        Edges::new(S::ZERO, scalar(5.0), S::ZERO, scalar(5.0))
    );

    for (preferred, minimum, maximum, expected) in [(0.1, 0.4, 0.9, 40.0), (1.0, 0.2, 0.6, 60.0)] {
        let batch = fri08_c03_containing_block_rows_child(
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::value(percent(preferred)),
                    PreferredSizeOf::px(scalar(20.0)),
                ),
                min_size: Size::new(MinSizeOf::value(percent(minimum)), MinSizeOf::AUTO),
                max_size: Size::new(MaxSizeOf::value(percent(maximum)), MaxSizeOf::NONE),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
            Size::ZERO,
        );
        assert_eq!(
            fri08_c01_placement_output(&batch, 2).size.width,
            scalar(expected),
            "percentage min/max must share the 100px hybrid width basis"
        );
    }

    for (item_is_replaced, align_self, expected_height) in [
        (false, None, 80.0),
        (true, None, 20.0),
        (true, Some(AlignItems::Stretch), 80.0),
    ] {
        let batch = fri08_c03_containing_block_rows_child(
            NodeInputOf {
                display: Display::Block,
                item_is_replaced,
                size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
                justify_self: Some(AlignItems::Start),
                align_self,
                ..NodeInputOf::default()
            },
            Size::new(scalar(20.0), scalar(20.0)),
        );
        assert_eq!(
            fri08_c01_placement_output(&batch, 2).size.height,
            scalar(expected_height),
            "replaced={item_is_replaced} align-self={align_self:?} stretch height"
        );
    }

    let aspect = fri08_c03_containing_block_rows_child(
        NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::value(percent(0.5)), PreferredSizeOf::AUTO),
            aspect_ratio: AspectRatioOf::new(scalar(2.0)),
            justify_self: Some(AlignItems::Start),
            align_self: Some(AlignItems::Start),
            ..NodeInputOf::default()
        },
        Size::ZERO,
    );
    assert_eq!(
        fri08_c01_placement_output(&aspect, 2).size,
        Size::new(scalar(50.0), scalar(25.0)),
        "aspect-ratio preflight must use the same resolved percentage width"
    );
}

pub(super) fn fri08_c03_auto_fit_named_repeat<S: LayoutScalar>(
    kind: TrackRepeat,
) -> TrackComponentOf<S> {
    let components = vec![
        TrackComponentOf::line_names(["slot"]),
        TrackComponentOf::px(S::from_f64(40.0)),
    ];
    let repetition = match kind {
        TrackRepeat::AutoFit => TrackRepetitionOf::auto_fit_components(components),
        TrackRepeat::AutoFill => TrackRepetitionOf::auto_fill_components(components),
        TrackRepeat::Count(_) => unreachable!("auto-repeat test helper requires an auto kind"),
    }
    .expect("valid named auto-repeat");
    TrackComponentOf::Repeat(repetition)
}

pub(super) fn fri08_c03_auto_fit_batch<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    viewport: Size<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(
        tree,
        1,
        LayoutRootRequestOf::viewport(viewport.map(AvailableOf::definite))
            .expect("finite lanes auto-fit viewport"),
    )
    .expect("valid lanes auto-fit layout")
}

#[derive(Clone, Debug)]
pub(super) struct Fri08C04StandaloneIntrinsicMinimumTree<S: LayoutScalar> {
    pub(super) children: HashMap<u32, Vec<u32>>,
    pub(super) styles: HashMap<u32, NodeInputOf<S>>,
}

impl<S: LayoutScalar> Fri08C04StandaloneIntrinsicMinimumTree<S> {
    pub(super) fn new(minimum: MinSizeOf<S>) -> Self {
        let mut children = HashMap::new();
        children.insert(1, vec![2]);
        children.insert(2, vec![3]);
        children.insert(3, vec![4, 5]);
        children.insert(4, Vec::new());
        children.insert(5, Vec::new());

        let mut styles = HashMap::new();
        styles.insert(
            1,
            NodeInputOf {
                display: Display::InlineGrid,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
        styles.insert(
            2,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                grid_column: GridPlacement::try_line(1).expect("single inherited column"),
                grid_row: GridPlacement::try_line_span(1, 2).expect("two root rows"),
                ..NodeInputOf::default()
            },
        );
        styles.insert(
            3,
            NodeInputOf {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: subgrid_track_of(),
                grid_template_areas: GridTemplateAreas {
                    rows: vec![
                        GridTemplateAreaRow {
                            cells: vec![Some("first".to_string())],
                        },
                        GridTemplateAreaRow {
                            cells: vec![Some("second".to_string())],
                        },
                    ],
                },
                grid_column: GridPlacement::try_line(1).expect("single inherited column"),
                grid_row: GridPlacement::try_line_span(1, 2).expect("two inherited rows"),
                grid_auto_flow: GridAutoFlow::Column,
                min_size: Size::new(minimum, MinSizeOf::AUTO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
        for (node, area) in [(4, "first"), (5, "second")] {
            styles.insert(
                node,
                NodeInputOf {
                    item_is_replaced: true,
                    grid_column: GridPlacement::try_line(1).expect("standalone local column"),
                    raw_grid_row: RawGridPlacement::new(
                        RawGridLine::BareIdent(area.to_string()),
                        RawGridLine::BareIdent(area.to_string()),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }

        Self { children, styles }
    }
}

impl<S: LayoutScalar> Traverse for Fri08C04StandaloneIntrinsicMinimumTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for Fri08C04StandaloneIntrinsicMinimumTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        self.styles
            .get(&node)
            .unwrap_or_else(|| panic!("standalone intrinsic node {node} must have style"))
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
        LayoutInputOf::box_input(self.node_input(node).clone())
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 4 | 5)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        if !matches!(node, 4 | 5) {
            return None;
        }
        let width = match input.available_content_size().width {
            MeasurementAvailableOf::MinContent => Self::Scalar::from_f64(20.0),
            MeasurementAvailableOf::MaxContent => Self::Scalar::from_f64(80.0),
            MeasurementAvailableOf::Definite(value) => {
                value.get().min(Self::Scalar::from_f64(80.0))
            }
        };
        Some(Ok(Size::new(width, Self::Scalar::from_f64(50.0))))
    }
}

pub(super) fn fri08_c04_standalone_intrinsic_minimum_width<S: LayoutScalar>(
    minimum: MinSizeOf<S>,
) -> S {
    let tree = Fri08C04StandaloneIntrinsicMinimumTree::new(minimum);
    let batch = compute_layout(
        &tree,
        1,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MIN_CONTENT))
            .expect("min-content standalone viewport"),
    )
    .expect("one-axis subgrid intrinsic minimum layout succeeds");
    fri08_c01_placement_output(&batch, 1).size.width
}

pub(super) fn fri08_c04_standalone_nested_tree<S: LayoutScalar>(
    writing_mode: WritingMode,
    root_direction: Direction,
    child_direction: Direction,
    standalone_minimum: MinSizeOf<S>,
    inherit_other_axis: bool,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let flow_axes = FlowAxes::new(writing_mode, child_direction);
    let inherited_span = GridPlacement::try_line_span(1, 2).expect("two inherited rows");
    let single_column = GridPlacement::try_line(1).expect("single inherited column");
    let leaf_size = flow_axes.physical_size(LogicalSizeOf::new(scalar(100.0), scalar(50.0)));
    PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [3])
        .children(3, [4, 5])
        .style(
            1,
            NodeInputOf {
                display: Display::InlineGrid,
                writing_mode,
                direction: root_direction,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                writing_mode,
                direction: child_direction,
                grid_template_columns: subgrid_track_of(),
                grid_template_rows: if inherit_other_axis {
                    subgrid_track_of()
                } else {
                    vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO]
                },
                grid_column: single_column,
                grid_row: inherited_span,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                writing_mode,
                direction: child_direction,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: subgrid_track_of(),
                grid_template_areas: GridTemplateAreas {
                    rows: vec![
                        GridTemplateAreaRow {
                            cells: vec![Some("first".to_string())],
                        },
                        GridTemplateAreaRow {
                            cells: vec![Some("second".to_string())],
                        },
                    ],
                },
                grid_column: single_column,
                grid_row: inherited_span,
                grid_auto_flow: GridAutoFlow::Column,
                min_size: flow_axes
                    .physical_size(LogicalSizeOf::new(standalone_minimum, MinSizeOf::AUTO)),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                writing_mode,
                direction: child_direction,
                item_order: ItemOrder::new(7),
                item_is_replaced: true,
                grid_column: GridPlacement::try_line(1).expect("standalone local column"),
                raw_grid_row: RawGridPlacement::new(
                    RawGridLine::BareIdent("first".to_string()),
                    RawGridLine::BareIdent("first".to_string()),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                writing_mode,
                direction: child_direction,
                item_order: ItemOrder::new(-7),
                item_is_replaced: true,
                grid_column: GridPlacement::try_line(1).expect("standalone local column"),
                raw_grid_row: RawGridPlacement::new(
                    RawGridLine::BareIdent("second".to_string()),
                    RawGridLine::BareIdent("second".to_string()),
                ),
                ..NodeInputOf::default()
            },
        )
        .measure(4, leaf_size)
        .measure(5, leaf_size)
}

pub(super) fn assert_fri08_c04_standalone_nested_flows<S: LayoutScalar>() {
    let scalar = S::from_f64;
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for root_direction in [Direction::Ltr, Direction::Rtl] {
            let child_direction = match root_direction {
                Direction::Ltr => Direction::Rtl,
                Direction::Rtl => Direction::Ltr,
            };
            for inherit_other_axis in [false, true] {
                for standalone_minimum in [
                    MinSizeOf::AUTO,
                    MinSizeOf::px(scalar(20.0)),
                    MinSizeOf::MIN_CONTENT,
                    MinSizeOf::MAX_CONTENT,
                ] {
                    let tree = fri08_c04_standalone_nested_tree::<S>(
                        writing_mode,
                        root_direction,
                        child_direction,
                        standalone_minimum.clone(),
                        inherit_other_axis,
                    );
                    for available in [
                        AvailableOf::MIN_CONTENT,
                        AvailableOf::MAX_CONTENT,
                        AvailableOf::Definite(scalar(240.0)),
                    ] {
                        let batch = compute_layout(
                            &tree,
                            1,
                            LayoutRootRequestOf::viewport(Size::splat(available))
                                .expect("finite standalone viewport"),
                        )
                        .expect("nested standalone boundary is supported");
                        let root = fri08_c01_placement_output(&batch, 1);
                        let logical =
                            FlowAxes::new(writing_mode, root_direction).logical_size(root.size);
                        assert_eq!(
                            logical,
                            LogicalSizeOf::new(scalar(100.0), scalar(100.0)),
                            "{writing_mode:?} {root_direction:?} {inherit_other_axis:?} {standalone_minimum:?} {available:?}: {:?}",
                            batch.final_entries()
                        );
                        assert_eq!(
                            batch
                                .final_entries()
                                .iter()
                                .map(LayoutOutputEntryOf::node)
                                .collect::<Vec<_>>(),
                            [1, 2, 3, 4, 5],
                            "standalone local descendants remain source-associated and publish once"
                        );
                    }
                }
            }
        }
    }
}

pub(super) fn fri08_c03_intrinsic_facts<S: LayoutScalar>(
    minimum: f64,
    min_content: f64,
    max_content: f64,
) -> LaneContributionFactsOf<S> {
    LaneContributionFactsOf {
        min_content: S::from_f64(min_content),
        max_content: S::from_f64(max_content),
        min_size: S::from_f64(minimum),
        automatic_minimum_applies: false,
    }
}

pub(super) fn fri08_c03_intrinsic_projected_item<S: LayoutScalar>(
    id: &'static str,
    span: usize,
    candidate_starts: Option<Vec<usize>>,
    baseline_role: LaneIntrinsicBaselineRole,
    edges: LaneIntrinsicEdgeFactsOf<S>,
    contribution: LaneContributionFactsOf<S>,
) -> ProjectedLaneIntrinsicItemOf<S> {
    ProjectedLaneIntrinsicItemOf {
        id,
        kind: LaneIntrinsicItemKind::Indefinite {
            span: LaneTrackSpanLength::new(span).expect("intrinsic span is nonzero"),
        },
        candidate_starts,
        contribution,
        baseline_role,
        edges,
        contribution_kind: IntrinsicSpanContribution::MinContent {
            prioritize_min_tracks: false,
        },
    }
}

pub(super) fn assert_fri08_c03_nested_candidate_bounds_edges_and_reversal<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let projection = NestedLaneIntrinsicProjectionOf {
        root_track_count: 5,
        axis: GridAxisKind::Column,
        wrapper_span: 3,
        wrapper_starts: vec![1],
        reversed: false,
        parent_gap: S::ZERO,
        accumulated_edges: LaneIntrinsicEdgeFactsOf::default(),
        wrapper_edges: LaneIntrinsicEdgeFactsOf {
            start_mbp: scalar(2.0),
            end_mbp: scalar(5.0),
            start_half_gap: scalar(3.0),
            end_half_gap: scalar(7.0),
        },
    };
    let groups = nested_lane_candidate_groups(&projection, 1, [0, 1, 2]);
    assert_eq!(groups.len(), 3);
    assert!(groups.iter().any(|group| {
        group.starts == [1]
            && group.edges.start_mbp == scalar(2.0)
            && group.edges.start_half_gap == scalar(3.0)
            && group.edges.end_mbp == S::ZERO
    }));
    assert!(groups.iter().any(|group| {
        group.starts == [2] && group.edges == LaneIntrinsicEdgeFactsOf::default()
    }));
    assert!(groups.iter().any(|group| {
        group.starts == [3]
            && group.edges.end_mbp == scalar(5.0)
            && group.edges.end_half_gap == scalar(7.0)
            && group.edges.start_mbp == S::ZERO
    }));

    let reversed = NestedLaneIntrinsicProjectionOf {
        reversed: true,
        ..projection
    };
    let reversed_groups = nested_lane_candidate_groups(&reversed, 1, [0, 1, 2]);
    assert!(reversed_groups.iter().any(|group| {
        group.starts == [3]
            && group.edges.end_mbp == scalar(2.0)
            && group.edges.end_half_gap == scalar(3.0)
    }));
    assert!(reversed_groups.iter().any(|group| {
        group.starts == [1]
            && group.edges.start_mbp == scalar(5.0)
            && group.edges.start_half_gap == scalar(7.0)
    }));
}

#[derive(Clone, Copy)]
pub(super) struct Fri08C03NestedFlowCase {
    pub(super) root_direction: Direction,
    pub(super) first_wrapper_mode: WritingMode,
    pub(super) first_wrapper_direction: Direction,
    pub(super) second_wrapper_mode: WritingMode,
    pub(super) second_wrapper_direction: Direction,
    pub(super) inherited_axis: GridAxisKind,
}

pub(super) fn fri08_c03_nested_subgrid_component<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))]
}

pub(super) fn fri08_c03_nested_axis_placement(
    axis: GridAxisKind,
    start: isize,
    span: usize,
) -> (GridPlacement, GridPlacement) {
    let inherited = GridPlacement::try_line_span(start, span).expect("valid nested test span");
    let companion = GridPlacement::try_line(1).expect("valid nested companion line");
    match axis {
        GridAxisKind::Column => (inherited, companion),
        GridAxisKind::Row => (companion, inherited),
    }
}

pub(super) fn fri08_c03_nested_wrapper_style<S: LayoutScalar>(
    mode: WritingMode,
    direction: Direction,
    axis: GridAxisKind,
    gap: f64,
    placement: (GridPlacement, GridPlacement),
    physical_edges: (f64, f64),
) -> NodeInputOf<S> {
    let scalar = S::from_f64;
    let (columns, rows, auto_flow) = match axis {
        GridAxisKind::Column => (
            fri08_c03_nested_subgrid_component(),
            vec![TrackComponentOf::px(scalar(10.0))],
            GridAutoFlow::Row,
        ),
        GridAxisKind::Row => (
            vec![TrackComponentOf::px(scalar(10.0))],
            fri08_c03_nested_subgrid_component(),
            GridAutoFlow::Column,
        ),
    };
    let (left, right) = physical_edges;
    let has_physical_edges = left != 0.0 || right != 0.0;
    NodeInputOf {
        display: Display::GridLanes,
        writing_mode: mode,
        direction,
        grid_auto_flow: auto_flow,
        grid_template_columns: columns,
        grid_template_rows: rows,
        grid_column: placement.0,
        grid_row: placement.1,
        gap: Size::new(LengthOf::px(scalar(gap)), LengthOf::ZERO),
        margin: Edges::new(
            LengthAutoOf::ZERO,
            LengthAutoOf::px(scalar(if has_physical_edges { right - 6.0 } else { 0.0 })),
            LengthAutoOf::ZERO,
            LengthAutoOf::px(scalar(if has_physical_edges { left - 4.0 } else { 0.0 })),
        ),
        padding: Edges::new(
            LengthOf::ZERO,
            LengthOf::px(scalar(if has_physical_edges { 5.0 } else { 0.0 })),
            LengthOf::ZERO,
            LengthOf::px(scalar(if has_physical_edges { 3.0 } else { 0.0 })),
        ),
        border: Edges::new(
            LengthOf::ZERO,
            LengthOf::px(scalar(if has_physical_edges { 1.0 } else { 0.0 })),
            LengthOf::ZERO,
            LengthOf::px(scalar(if has_physical_edges { 1.0 } else { 0.0 })),
        ),
        ..NodeInputOf::default()
    }
}

pub(super) fn fri08_c03_nested_projection_tree<S: LayoutScalar>(
    flow: Fri08C03NestedFlowCase,
    tolerance: GridFlowToleranceOf<S>,
    with_edges: bool,
) -> PublicLayoutTreeOf<S> {
    let scalar = S::from_f64;
    let root_gap = if with_edges { 10.0 } else { 0.0 };
    let first_gap = if with_edges { 14.0 } else { 0.0 };
    let second_gap = if with_edges { 22.0 } else { 0.0 };
    let first_edges = if with_edges { (6.0, 10.0) } else { (0.0, 0.0) };
    let second_edges = if with_edges { (12.0, 16.0) } else { (0.0, 0.0) };
    let root_wrapper_placement = fri08_c03_nested_axis_placement(GridAxisKind::Column, 1, 3);
    let nested_wrapper_placement = fri08_c03_nested_axis_placement(flow.inherited_axis, 1, 3);
    let first_leaf_placement = fri08_c03_nested_axis_placement(flow.inherited_axis, 1, 1);
    let last_leaf_placement = fri08_c03_nested_axis_placement(flow.inherited_axis, 3, 1);

    let mut tree = PublicLayoutTreeOf::new()
        .children(1, [2, 6, 7, 8])
        .children(2, [3])
        .children(3, [4, 5])
        .style(
            1,
            NodeInputOf {
                display: Display::GridLanes,
                direction: flow.root_direction,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(10.0))),
                grid_template_columns: vec![
                    TrackComponentOf::AUTO,
                    TrackComponentOf::AUTO,
                    TrackComponentOf::AUTO,
                ],
                grid_template_rows: vec![TrackComponentOf::px(scalar(10.0))],
                gap: Size::new(LengthOf::px(scalar(root_gap)), LengthOf::ZERO),
                grid_flow_tolerance: tolerance,
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            fri08_c03_nested_wrapper_style(
                flow.first_wrapper_mode,
                flow.first_wrapper_direction,
                flow.inherited_axis,
                first_gap,
                root_wrapper_placement,
                first_edges,
            ),
        )
        .style(
            3,
            fri08_c03_nested_wrapper_style(
                flow.second_wrapper_mode,
                flow.second_wrapper_direction,
                flow.inherited_axis,
                second_gap,
                nested_wrapper_placement,
                second_edges,
            ),
        );
    for (node, placement, measurement, order) in [
        (4, first_leaf_placement, 20.0, 7),
        (5, last_leaf_placement, 40.0, -7),
    ] {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    grid_column: placement.0,
                    grid_row: placement.1,
                    item_order: ItemOrder::new(order),
                    item_is_replaced: true,
                    justify_self: Some(AlignItems::Start),
                    align_self: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            )
            .measure(node, Size::new(scalar(measurement), scalar(10.0)));
    }
    for (index, node) in [6, 7, 8].into_iter().enumerate() {
        let placement =
            GridPlacement::try_line(isize::try_from(index + 1).unwrap()).expect("valid probe line");
        tree = tree
            .style(
                node,
                NodeInputOf {
                    grid_column: placement,
                    grid_row: GridPlacement::try_line(1).expect("valid probe row"),
                    min_size: Size::ZERO.map(MinSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .measure(node, Size::ZERO);
    }
    tree
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C03NestedMeasureMode {
    Values,
    ProviderError,
    NonFinite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C03NestedMeasureError {
    Provider,
}

#[derive(Clone, Debug)]
pub(super) struct Fri08C03NestedAtomicTree<S: LayoutScalar> {
    pub(super) tree: PublicLayoutTreeOf<S>,
    pub(super) measure_mode: std::cell::Cell<Fri08C03NestedMeasureMode>,
    pub(super) measurement_requests: std::cell::RefCell<Vec<(u32, LeafMeasureInputOf<S>)>>,
    pub(super) cache_queries: std::cell::RefCell<Vec<(u32, bool)>>,
    pub(super) retained: GridTestRetainedState<S>,
}

impl<S: LayoutScalar> Fri08C03NestedAtomicTree<S> {
    pub(super) fn new() -> Self {
        Self::with_tree(fri08_c03_nested_projection_tree(
            Fri08C03NestedFlowCase {
                root_direction: Direction::Ltr,
                first_wrapper_mode: WritingMode::HorizontalTb,
                first_wrapper_direction: Direction::Ltr,
                second_wrapper_mode: WritingMode::HorizontalTb,
                second_wrapper_direction: Direction::Ltr,
                inherited_axis: GridAxisKind::Column,
            },
            GridFlowToleranceOf::Length(LengthOf::ZERO),
            false,
        ))
    }

    pub(super) fn with_tree(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            measure_mode: std::cell::Cell::new(Fri08C03NestedMeasureMode::Values),
            measurement_requests: std::cell::RefCell::new(Vec::new()),
            cache_queries: std::cell::RefCell::new(Vec::new()),
            retained: GridTestRetainedState::default(),
        }
    }

    pub(super) fn request() -> LayoutRootRequestOf<S> {
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
            .expect("nested atomic max-content viewport")
    }
}

impl<S: LayoutScalar> Traverse for Fri08C03NestedAtomicTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri08C03NestedAtomicTree<S> {
    type MeasureError = Fri08C03NestedMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        matches!(node, 4 | 5)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if !matches!(node, 4 | 5) {
            return None;
        }
        self.measurement_requests.borrow_mut().push((node, input));
        match self.measure_mode.get() {
            Fri08C03NestedMeasureMode::ProviderError if node == 4 => {
                Some(Err(Fri08C03NestedMeasureError::Provider))
            }
            Fri08C03NestedMeasureMode::NonFinite if node == 4 => {
                Some(Ok(Size::new(S::from_f64(f64::NAN), S::from_f64(10.0))))
            }
            _ => Some(Ok(Size::new(
                S::from_f64(if node == 4 { 20.0 } else { 40.0 }),
                S::from_f64(10.0),
            ))),
        }
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri08C03NestedAtomicTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = GridTestRetainedState<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        Ok(self.retained.prepare_grid_test_batch(batch))
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C04BaselineMeasureMode {
    Values,
    ProviderError,
    NonFinite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C04BaselineMeasureError {
    Provider,
}

#[derive(Clone, Debug)]
pub(super) struct Fri08C04BaselineTree<S: LayoutScalar> {
    pub(super) tree: PublicLayoutTreeOf<S>,
    pub(super) measurements: HashMap<u32, Size<S>>,
    pub(super) failing_node: u32,
    pub(super) measure_mode: std::cell::Cell<Fri08C04BaselineMeasureMode>,
    pub(super) measurement_requests: std::cell::RefCell<Vec<u32>>,
    pub(super) cache_queries: std::cell::RefCell<Vec<(u32, bool)>>,
    pub(super) retained: GridTestRetainedState<S>,
}

impl<S: LayoutScalar> Traverse for Fri08C04BaselineTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri08C04BaselineTree<S> {
    type MeasureError = Fri08C04BaselineMeasureError;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measurements.contains_key(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        let measured = self.measurements.get(&node).copied()?;
        self.measurement_requests.borrow_mut().push(node);
        Some(match self.measure_mode.get() {
            Fri08C04BaselineMeasureMode::ProviderError if node == self.failing_node => {
                Err(Fri08C04BaselineMeasureError::Provider)
            }
            Fri08C04BaselineMeasureMode::NonFinite if node == self.failing_node => {
                Ok(Size::new(S::from_f64(f64::NAN), measured.height))
            }
            _ => Ok(measured),
        })
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .retained
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        self.cache_queries
            .borrow_mut()
            .push((node, output.is_some()));
        output
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<NodeOutputOf<S>> {
        self.retained.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Fri08C04BaselineTree<S> {
    type Error = core::convert::Infallible;
    type Prepared = GridTestRetainedState<S>;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        Ok(self.retained.prepare_grid_test_batch(batch))
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.retained = prepared;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C04BaselineParentAxis {
    Column,
    Row,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Fri08C04BaselineFlowCase {
    pub(super) parent_axis: Fri08C04BaselineParentAxis,
    pub(super) root_writing_mode: WritingMode,
    pub(super) root_direction: Direction,
    pub(super) child_writing_mode: WritingMode,
    pub(super) child_direction: Direction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fri08C04BaselineAreaTopology {
    ExpandedNonUniform,
    NonAreaNonUniform,
    NonAreaOrthogonalControl,
    NonAreaUniform,
    UniformAreaExpanded,
    FullyExplicitNonUniformArea,
    OrthogonalOnlyInheritedArea,
}

pub(super) fn fri08_c04_baseline_area_implicit_tree<S: LayoutScalar>(
    case: Fri08C04BaselineFlowCase,
    alignment: AlignItems,
) -> Fri08C04BaselineTree<S> {
    fri08_c04_baseline_area_topology_tree(
        case,
        alignment,
        Fri08C04BaselineAreaTopology::ExpandedNonUniform,
    )
}

pub(super) fn fri08_c04_baseline_area_topology_tree<S: LayoutScalar>(
    case: Fri08C04BaselineFlowCase,
    alignment: AlignItems,
    topology: Fri08C04BaselineAreaTopology,
) -> Fri08C04BaselineTree<S> {
    let scalar = S::from_f64;
    let root_axes = FlowAxes::new(case.root_writing_mode, case.root_direction);
    let child_axes = FlowAxes::new(case.child_writing_mode, case.child_direction);
    let (direct_writing_mode, direct_direction) = match case.parent_axis {
        Fri08C04BaselineParentAxis::Row => (case.root_writing_mode, case.root_direction),
        Fri08C04BaselineParentAxis::Column => (case.child_writing_mode, case.child_direction),
    };
    let direct_axes = FlowAxes::new(direct_writing_mode, direct_direction);
    let root_gap = root_axes.physical_size(LogicalSizeOf::new(scalar(12.0), scalar(10.0)));
    let composed_edges = case.root_writing_mode == WritingMode::HorizontalTb
        && case.parent_axis == Fri08C04BaselineParentAxis::Row;
    let child_gap = child_axes.physical_size(LogicalSizeOf::new(scalar(6.0), scalar(20.0)));
    let (direct_column, direct_row, nested_column, nested_row, implicit_column, implicit_row) =
        match case.parent_axis {
            Fri08C04BaselineParentAxis::Row => (
                GridPlacement::try_line(1).expect("direct area column"),
                GridPlacement::try_line(2).expect("direct baseline row"),
                GridPlacement::try_line(2).expect("subgrid area column"),
                GridPlacement::try_line_span(1, 2).expect("subgrid inherited rows"),
                GridPlacement::try_line(3).expect("implicit column"),
                GridPlacement::try_line(1).expect("implicit row"),
            ),
            Fri08C04BaselineParentAxis::Column => (
                GridPlacement::try_line(2).expect("direct baseline column"),
                GridPlacement::try_line(1).expect("direct area row"),
                GridPlacement::try_line_span(1, 2).expect("subgrid inherited columns"),
                GridPlacement::try_line(2).expect("subgrid area row"),
                GridPlacement::try_line(1).expect("implicit column"),
                GridPlacement::try_line(3).expect("implicit row"),
            ),
        };
    let direct_alignment = match case.parent_axis {
        Fri08C04BaselineParentAxis::Column => (Some(alignment), Some(AlignItems::Start)),
        Fri08C04BaselineParentAxis::Row => (Some(AlignItems::Start), Some(alignment)),
    };
    let two_by_two_areas = GridTemplateAreas {
        rows: vec![
            GridTemplateAreaRow {
                cells: vec![Some("alpha".to_string()), Some("beta".to_string())],
            },
            GridTemplateAreaRow {
                cells: vec![Some("gamma".to_string()), Some("delta".to_string())],
            },
        ],
    };
    let orthogonal_areas = GridTemplateAreas {
        rows: vec![GridTemplateAreaRow {
            cells: vec![Some("left".to_string()), Some("right".to_string())],
        }],
    };
    let orthogonal_control = matches!(
        topology,
        Fri08C04BaselineAreaTopology::NonAreaOrthogonalControl
            | Fri08C04BaselineAreaTopology::OrthogonalOnlyInheritedArea
    );
    let uniform_tracks = matches!(
        topology,
        Fri08C04BaselineAreaTopology::NonAreaUniform
            | Fri08C04BaselineAreaTopology::UniformAreaExpanded
    );
    let root_areas = match topology {
        Fri08C04BaselineAreaTopology::ExpandedNonUniform
        | Fri08C04BaselineAreaTopology::UniformAreaExpanded
        | Fri08C04BaselineAreaTopology::FullyExplicitNonUniformArea => two_by_two_areas,
        Fri08C04BaselineAreaTopology::NonAreaNonUniform
        | Fri08C04BaselineAreaTopology::NonAreaOrthogonalControl
        | Fri08C04BaselineAreaTopology::NonAreaUniform
        | Fri08C04BaselineAreaTopology::OrthogonalOnlyInheritedArea => GridTemplateAreas::default(),
    };
    let root_columns = match topology {
        Fri08C04BaselineAreaTopology::ExpandedNonUniform
        | Fri08C04BaselineAreaTopology::UniformAreaExpanded => {
            vec![TrackComponentOf::px(scalar(60.0))]
        }
        Fri08C04BaselineAreaTopology::NonAreaNonUniform
        | Fri08C04BaselineAreaTopology::NonAreaOrthogonalControl
        | Fri08C04BaselineAreaTopology::NonAreaUniform
        | Fri08C04BaselineAreaTopology::FullyExplicitNonUniformArea
        | Fri08C04BaselineAreaTopology::OrthogonalOnlyInheritedArea => vec![
            TrackComponentOf::px(scalar(60.0)),
            TrackComponentOf::px(scalar(if uniform_tracks { 60.0 } else { 70.0 })),
        ],
    };
    let root_rows = match topology {
        Fri08C04BaselineAreaTopology::ExpandedNonUniform
        | Fri08C04BaselineAreaTopology::UniformAreaExpanded => {
            vec![TrackComponentOf::px(scalar(40.0))]
        }
        Fri08C04BaselineAreaTopology::NonAreaNonUniform
        | Fri08C04BaselineAreaTopology::NonAreaOrthogonalControl
        | Fri08C04BaselineAreaTopology::NonAreaUniform
        | Fri08C04BaselineAreaTopology::FullyExplicitNonUniformArea
        | Fri08C04BaselineAreaTopology::OrthogonalOnlyInheritedArea => vec![
            TrackComponentOf::px(scalar(40.0)),
            TrackComponentOf::px(scalar(if uniform_tracks { 40.0 } else { 50.0 })),
        ],
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2, 4, 3])
        .children(2, [])
        .children(3, [5])
        .children(4, [6])
        .children(5, [7])
        .children(6, [])
        .children(7, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: case.root_writing_mode,
                direction: case.root_direction,
                size: root_axes
                    .physical_size(LogicalSizeOf::new(scalar(220.0), scalar(150.0)))
                    .map(PreferredSizeOf::px),
                grid_template_columns: root_columns,
                grid_template_rows: root_rows,
                grid_template_areas: root_areas,
                grid_auto_columns: vec![TrackComponentOf::px(scalar(if composed_edges {
                    if uniform_tracks { 60.0 } else { 70.0 }
                } else {
                    60.0
                }))],
                grid_auto_rows: vec![TrackComponentOf::px(scalar(if composed_edges {
                    if uniform_tracks { 40.0 } else { 50.0 }
                } else {
                    40.0
                }))],
                gap: root_gap.map(LengthOf::px),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: case.root_writing_mode,
                direction: case.root_direction,
                grid_column: implicit_column,
                grid_row: implicit_row,
                item_order: ItemOrder::new(0),
                size: root_axes
                    .physical_size(LogicalSizeOf::new(scalar(8.0), scalar(8.0)))
                    .map(PreferredSizeOf::px),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: direct_writing_mode,
                direction: direct_direction,
                grid_column: direct_column,
                grid_row: direct_row,
                item_order: ItemOrder::new(7),
                justify_self: direct_alignment.0,
                align_self: direct_alignment.1,
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: vec![TrackComponentOf::AUTO],
                justify_items: Some(AlignItems::Start),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: case.child_writing_mode,
                direction: case.child_direction,
                grid_column: nested_column,
                grid_row: nested_row,
                item_order: ItemOrder::new(-7),
                grid_template_columns: if orthogonal_control {
                    vec![
                        TrackComponentOf::px(scalar(32.0)),
                        TrackComponentOf::px(scalar(32.0)),
                    ]
                } else {
                    vec![TrackComponentOf::px(scalar(32.0))]
                },
                grid_template_rows: subgrid_track_of(),
                grid_template_areas: if topology
                    == Fri08C04BaselineAreaTopology::OrthogonalOnlyInheritedArea
                {
                    orthogonal_areas
                } else {
                    GridTemplateAreas::default()
                },
                gap: child_gap.map(LengthOf::px),
                margin: if composed_edges {
                    Edges::new(
                        LengthAutoOf::px(scalar(3.0)),
                        LengthAutoOf::px(scalar(5.0)),
                        LengthAutoOf::px(scalar(7.0)),
                        LengthAutoOf::px(scalar(11.0)),
                    )
                } else {
                    Edges::all(LengthAutoOf::ZERO)
                },
                border: if composed_edges {
                    Edges::new(
                        LengthOf::px(scalar(2.0)),
                        LengthOf::px(scalar(1.0)),
                        LengthOf::px(scalar(4.0)),
                        LengthOf::px(scalar(3.0)),
                    )
                } else {
                    Edges::all(LengthOf::ZERO)
                },
                padding: if composed_edges {
                    Edges::new(
                        LengthOf::px(scalar(5.0)),
                        LengthOf::px(scalar(2.0)),
                        LengthOf::px(scalar(6.0)),
                        LengthOf::px(scalar(4.0)),
                    )
                } else {
                    Edges::all(LengthOf::ZERO)
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            5,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: case.child_writing_mode,
                direction: case.child_direction,
                grid_column: GridPlacement::try_line(1).expect("standalone local column"),
                grid_row: GridPlacement::try_line(2).expect("inherited local row"),
                item_order: ItemOrder::new(-9),
                justify_self: Some(AlignItems::Start),
                align_self: Some(alignment),
                grid_template_columns: vec![TrackComponentOf::AUTO],
                grid_template_rows: if orthogonal_control {
                    subgrid_track_of()
                } else {
                    vec![TrackComponentOf::AUTO]
                },
                margin: if orthogonal_control {
                    Edges::all(LengthAutoOf::px(scalar(1.0)))
                } else {
                    Edges::all(LengthAutoOf::ZERO)
                },
                border: if orthogonal_control {
                    Edges::all(LengthOf::px(scalar(1.0)))
                } else {
                    Edges::all(LengthOf::ZERO)
                },
                padding: if orthogonal_control {
                    Edges::all(LengthOf::px(scalar(1.0)))
                } else {
                    Edges::all(LengthOf::ZERO)
                },
                justify_items: Some(AlignItems::Start),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            6,
            NodeInputOf {
                writing_mode: direct_writing_mode,
                direction: direct_direction,
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            7,
            NodeInputOf {
                writing_mode: case.child_writing_mode,
                direction: case.child_direction,
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        );
    let measurements = HashMap::from([
        (
            6,
            direct_axes.physical_size(LogicalSizeOf::new(scalar(20.0), scalar(30.0))),
        ),
        (
            7,
            child_axes.physical_size(LogicalSizeOf::new(scalar(18.0), scalar(12.0))),
        ),
    ]);
    Fri08C04BaselineTree {
        tree,
        measurements,
        failing_node: 7,
        measure_mode: std::cell::Cell::new(Fri08C04BaselineMeasureMode::Values),
        measurement_requests: std::cell::RefCell::new(Vec::new()),
        cache_queries: std::cell::RefCell::new(Vec::new()),
        retained: GridTestRetainedState::default(),
    }
}

pub(super) fn fri08_c04_baseline_physical_edge<S: LayoutScalar>(
    output: NodeOutputOf<S>,
    flow_axes: FlowAxes,
    alignment: AlignItems,
) -> S {
    let decreasing = flow_axes
        .logical_axis_progression(LogicalAxis::Block)
        .is_decreasing();
    let (origin, extent) = match flow_axes.block_axis() {
        PhysicalAxis::Horizontal => (output.location.x, output.size.width),
        PhysicalAxis::Vertical => (output.location.y, output.size.height),
    };
    match alignment {
        AlignItems::Baseline if decreasing => origin,
        AlignItems::Baseline => origin + extent,
        AlignItems::LastBaseline if decreasing => origin + extent,
        AlignItems::LastBaseline => origin,
        _ => unreachable!("baseline edge helper requires a first/last role"),
    }
}

pub(super) fn fri08_c04_baseline_world_coordinate<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    case: Fri08C04BaselineFlowCase,
    alignment: AlignItems,
) -> (S, S) {
    let nested = fri08_c01_placement_output(batch, 3);
    let direct_container = fri08_c01_placement_output(batch, 4);
    let direct_leaf = fri08_c01_placement_output(batch, 6);
    let descendant_container = fri08_c01_placement_output(batch, 5);
    let descendant_leaf = fri08_c01_placement_output(batch, 7);
    let direct_axes = match case.parent_axis {
        Fri08C04BaselineParentAxis::Row => {
            FlowAxes::new(case.root_writing_mode, case.root_direction)
        }
        Fri08C04BaselineParentAxis::Column => {
            FlowAxes::new(case.child_writing_mode, case.child_direction)
        }
    };
    let child_axes = FlowAxes::new(case.child_writing_mode, case.child_direction);
    let nested_origin = match child_axes.block_axis() {
        PhysicalAxis::Horizontal => nested.location.x,
        PhysicalAxis::Vertical => nested.location.y,
    };
    let direct_origin = match direct_axes.block_axis() {
        PhysicalAxis::Horizontal => direct_container.location.x,
        PhysicalAxis::Vertical => direct_container.location.y,
    };
    let descendant_origin = match child_axes.block_axis() {
        PhysicalAxis::Horizontal => descendant_container.location.x,
        PhysicalAxis::Vertical => descendant_container.location.y,
    };
    (
        direct_origin + fri08_c04_baseline_physical_edge(direct_leaf, direct_axes, alignment),
        nested_origin
            + descendant_origin
            + fri08_c04_baseline_physical_edge(descendant_leaf, child_axes, alignment),
    )
}

pub(super) fn assert_fri08_c04_baseline_area_topology_controls<S: LayoutScalar>() {
    let case = Fri08C04BaselineFlowCase {
        parent_axis: Fri08C04BaselineParentAxis::Row,
        root_writing_mode: WritingMode::HorizontalTb,
        root_direction: Direction::Ltr,
        child_writing_mode: WritingMode::HorizontalTb,
        child_direction: Direction::Rtl,
    };
    let request = LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
        .expect("baseline topology-control viewport");
    for alignment in [AlignItems::Baseline, AlignItems::LastBaseline] {
        for (reference, area_topology) in [
            (
                Fri08C04BaselineAreaTopology::NonAreaUniform,
                Fri08C04BaselineAreaTopology::UniformAreaExpanded,
            ),
            (
                Fri08C04BaselineAreaTopology::NonAreaNonUniform,
                Fri08C04BaselineAreaTopology::FullyExplicitNonUniformArea,
            ),
            (
                Fri08C04BaselineAreaTopology::NonAreaOrthogonalControl,
                Fri08C04BaselineAreaTopology::OrthogonalOnlyInheritedArea,
            ),
        ] {
            let reference = compute_layout(
                &fri08_c04_baseline_area_topology_tree::<S>(case, alignment, reference),
                1,
                request,
            )
            .expect("non-area baseline topology control succeeds");
            let area = compute_layout(
                &fri08_c04_baseline_area_topology_tree::<S>(case, alignment, area_topology),
                1,
                request,
            )
            .expect("area baseline topology control succeeds");
            assert_eq!(
                area.final_entries(),
                reference.final_entries(),
                "{area_topology:?} naming facts do not change {alignment:?} production layout when they do not create a non-uniform mapped-axis track"
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Fri08C02TrackAxis {
    Columns,
    Rows,
}

pub(super) fn fri08_c02_fit_content_track<S: LayoutScalar>(
    absolute_px: f64,
    percent_fraction: f64,
) -> TrackComponentOf<S> {
    TrackComponentOf::fit_content(SizingCalculationOf::value(
        LengthPercentageOf::from_coefficients(
            S::from_f64(absolute_px),
            S::from_f64(percent_fraction),
        )
        .expect("finite fit-content test limit"),
    ))
}

pub(super) fn fri08_c02_flex_track<S: LayoutScalar>(factor: f64) -> TrackComponentOf<S> {
    TrackComponentOf::flex(
        TrackFlexFactorOf::try_new(S::from_f64(factor)).expect("finite flex test factor"),
    )
}

pub(super) fn fri08_c02_track_mix_tree<S: LayoutScalar>(
    display: Display,
    axis: Fri08C02TrackAxis,
    writing_mode: WritingMode,
    fit_limit: (f64, f64),
    definite_axis_size: Option<f64>,
    companion_tracks: Vec<TrackComponentOf<S>>,
    measurements: &[f64],
) -> (PublicLayoutTreeOf<S>, crate::geometry::FlowAxes, Size<S>) {
    let scalar = S::from_f64;
    let flow_axes = crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr);
    let track_axis_size = definite_axis_size.unwrap_or_else(|| {
        measurements
            .iter()
            .copied()
            .fold(0.0_f64, |sum, value| sum + value)
    });
    let logical_container_size = match axis {
        Fri08C02TrackAxis::Columns => LogicalSizeOf::new(scalar(track_axis_size), scalar(10.0)),
        Fri08C02TrackAxis::Rows => LogicalSizeOf::new(scalar(10.0), scalar(track_axis_size)),
    };
    let physical_container_size = flow_axes.physical_size(logical_container_size);
    let mut sizing_tracks = vec![fri08_c02_fit_content_track(fit_limit.0, fit_limit.1)];
    sizing_tracks.extend(companion_tracks);
    let (columns, rows) = match axis {
        Fri08C02TrackAxis::Columns => (sizing_tracks, vec![TrackComponentOf::px(scalar(10.0))]),
        Fri08C02TrackAxis::Rows => (vec![TrackComponentOf::px(scalar(10.0))], sizing_tracks),
    };
    let logical_root_size = match axis {
        Fri08C02TrackAxis::Columns => LogicalSizeOf::new(
            definite_axis_size.map_or(PreferredSizeOf::AUTO, |value| {
                PreferredSizeOf::px(scalar(value))
            }),
            PreferredSizeOf::px(scalar(10.0)),
        ),
        Fri08C02TrackAxis::Rows => LogicalSizeOf::new(
            PreferredSizeOf::px(scalar(10.0)),
            definite_axis_size.map_or(PreferredSizeOf::AUTO, |value| {
                PreferredSizeOf::px(scalar(value))
            }),
        ),
    };
    let physical_root_size = flow_axes.physical_size(logical_root_size);
    let children = (0..measurements.len())
        .map(|index| index as u32 + 2)
        .collect::<Vec<_>>();
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, children.iter().copied())
        .style(
            1,
            NodeInputOf {
                display,
                writing_mode,
                size: physical_root_size,
                grid_template_columns: columns,
                grid_template_rows: rows,
                grid_auto_flow: match axis {
                    Fri08C02TrackAxis::Columns => GridAutoFlow::Row,
                    Fri08C02TrackAxis::Rows => GridAutoFlow::Column,
                },
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
    for (index, measurement) in measurements.iter().copied().enumerate() {
        let node = index as u32 + 2;
        let line = isize::try_from(index + 1).expect("small test track index");
        let placement = GridPlacement::try_line(line).expect("valid test grid line");
        let style = match axis {
            Fri08C02TrackAxis::Columns => NodeInputOf {
                grid_column: placement,
                grid_row: GridPlacement::try_line(1).expect("first test row"),
                ..NodeInputOf::default()
            },
            Fri08C02TrackAxis::Rows => NodeInputOf {
                grid_column: GridPlacement::try_line(1).expect("first test column"),
                grid_row: placement,
                ..NodeInputOf::default()
            },
        };
        let logical_measurement = match axis {
            Fri08C02TrackAxis::Columns => LogicalSizeOf::new(scalar(measurement), scalar(10.0)),
            Fri08C02TrackAxis::Rows => LogicalSizeOf::new(scalar(10.0), scalar(measurement)),
        };
        tree = tree
            .style(node, style)
            .measure(node, flow_axes.physical_size(logical_measurement));
    }
    (tree, flow_axes, physical_container_size)
}

pub(super) fn fri08_c02_track_sizes<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    flow_axes: crate::geometry::FlowAxes,
    viewport: Size<S>,
    axis: Fri08C02TrackAxis,
    count: usize,
) -> Vec<S> {
    let batch = compute_layout(
        tree,
        1,
        LayoutRootRequestOf::viewport(viewport.map(AvailableOf::definite))
            .expect("finite track sizing viewport"),
    )
    .expect("valid public grid track sizing");
    (0..count)
        .map(|index| {
            let logical_size =
                flow_axes.logical_size(fri08_c01_placement_output(&batch, index as u32 + 2).size);
            match axis {
                Fri08C02TrackAxis::Columns => logical_size.inline,
                Fri08C02TrackAxis::Rows => logical_size.block,
            }
        })
        .collect()
}

pub(super) fn fri08_c02_stretch_track<S: LayoutScalar>(
    minimum: MinTrackSizingOf<S>,
) -> TrackComponentOf<S> {
    TrackComponentOf::minmax(minimum, MaxTrackSizingOf::AUTO)
}

pub(super) struct Fri08C02StretchTreeInput<'a, S: LayoutScalar> {
    pub(super) display: Display,
    pub(super) axis: Fri08C02TrackAxis,
    pub(super) writing_mode: WritingMode,
    pub(super) definite_axis_size: Option<f64>,
    pub(super) viewport_axis_size: f64,
    pub(super) gap: f64,
    pub(super) alignment: Option<AlignContent>,
    pub(super) tracks: Vec<TrackComponentOf<S>>,
    pub(super) measurements: &'a [f64],
}

pub(super) fn fri08_c02_stretch_tree<S: LayoutScalar>(
    input: Fri08C02StretchTreeInput<'_, S>,
) -> (PublicLayoutTreeOf<S>, crate::geometry::FlowAxes, Size<S>) {
    let Fri08C02StretchTreeInput {
        display,
        axis,
        writing_mode,
        definite_axis_size,
        viewport_axis_size,
        gap,
        alignment,
        tracks,
        measurements,
    } = input;
    let scalar = S::from_f64;
    let flow_axes = crate::geometry::FlowAxes::new(writing_mode, Direction::Ltr);
    let logical_root_size = match axis {
        Fri08C02TrackAxis::Columns => LogicalSizeOf::new(
            definite_axis_size.map_or(PreferredSizeOf::AUTO, |size| {
                PreferredSizeOf::px(scalar(size))
            }),
            PreferredSizeOf::px(scalar(10.0)),
        ),
        Fri08C02TrackAxis::Rows => LogicalSizeOf::new(
            PreferredSizeOf::px(scalar(10.0)),
            definite_axis_size.map_or(PreferredSizeOf::AUTO, |size| {
                PreferredSizeOf::px(scalar(size))
            }),
        ),
    };
    let physical_root_size = flow_axes.physical_size(logical_root_size);
    let (columns, rows) = match axis {
        Fri08C02TrackAxis::Columns => (tracks, vec![TrackComponentOf::px(scalar(10.0))]),
        Fri08C02TrackAxis::Rows => (vec![TrackComponentOf::px(scalar(10.0))], tracks),
    };
    let children = (0..measurements.len())
        .map(|index| index as u32 + 2)
        .collect::<Vec<_>>();
    let mut tree = PublicLayoutTreeOf::new()
        .children(1, children.iter().copied())
        .style(
            1,
            NodeInputOf {
                display,
                writing_mode,
                size: physical_root_size,
                grid_template_columns: columns,
                grid_template_rows: rows,
                grid_auto_flow: match axis {
                    Fri08C02TrackAxis::Columns => GridAutoFlow::Row,
                    Fri08C02TrackAxis::Rows => GridAutoFlow::Column,
                },
                gap: flow_axes.physical_size(LogicalSizeOf::new(
                    LengthOf::px(scalar(gap)),
                    LengthOf::ZERO,
                )),
                justify_content: alignment,
                align_content: alignment,
                ..NodeInputOf::default()
            },
        );
    for (index, measurement) in measurements.iter().copied().enumerate() {
        let node = index as u32 + 2;
        let line = isize::try_from(index + 1).expect("small stretch track index");
        let placement = GridPlacement::try_line(line).expect("valid stretch track line");
        let style = match axis {
            Fri08C02TrackAxis::Columns => NodeInputOf {
                grid_column: placement,
                grid_row: GridPlacement::try_line(1).expect("single stretch row"),
                ..NodeInputOf::default()
            },
            Fri08C02TrackAxis::Rows => NodeInputOf {
                grid_column: GridPlacement::try_line(1).expect("single stretch column"),
                grid_row: placement,
                ..NodeInputOf::default()
            },
        };
        let logical_measurement = match axis {
            Fri08C02TrackAxis::Columns => LogicalSizeOf::new(scalar(measurement), scalar(10.0)),
            Fri08C02TrackAxis::Rows => LogicalSizeOf::new(scalar(10.0), scalar(measurement)),
        };
        tree = tree
            .style(node, style)
            .measure(node, flow_axes.physical_size(logical_measurement));
    }
    let logical_viewport = match axis {
        Fri08C02TrackAxis::Columns => LogicalSizeOf::new(scalar(viewport_axis_size), scalar(10.0)),
        Fri08C02TrackAxis::Rows => LogicalSizeOf::new(scalar(10.0), scalar(viewport_axis_size)),
    };
    (tree, flow_axes, flow_axes.physical_size(logical_viewport))
}

pub(super) fn assert_fri08_c02_stretch_intrinsic_minimums<S: LayoutScalar>() {
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for axis in [Fri08C02TrackAxis::Columns, Fri08C02TrackAxis::Rows] {
            for minimum in [
                MinTrackSizingOf::<S>::MIN_CONTENT,
                MinTrackSizingOf::<S>::MAX_CONTENT,
            ] {
                let (tree, flow_axes, viewport) =
                    fri08_c02_stretch_tree(Fri08C02StretchTreeInput {
                        display: Display::Grid,
                        axis,
                        writing_mode,
                        definite_axis_size: Some(100.0),
                        viewport_axis_size: 100.0,
                        gap: 0.0,
                        alignment: None,
                        tracks: vec![fri08_c02_stretch_track(minimum)],
                        measurements: &[20.0],
                    });
                let size = fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 1)[0];
                assert_eq!(size, S::from_f64(100.0));
                assert!(size > S::from_f64(20.0));
            }
        }
    }
}

pub(super) fn assert_fri08_c02_fit_content_flex_composes<S: LayoutScalar>(
    axis: Fri08C02TrackAxis,
    writing_mode: WritingMode,
) {
    let (tree, flow_axes, viewport) = fri08_c02_track_mix_tree(
        Display::Grid,
        axis,
        writing_mode,
        (50.0, 0.0),
        Some(200.0),
        vec![fri08_c02_flex_track::<S>(1.0)],
        &[20.0, 0.0],
    );
    let sizes = fri08_c02_track_sizes(&tree, flow_axes, viewport, axis, 2);
    assert_eq!(sizes, [S::from_f64(20.0), S::from_f64(180.0)]);
}

pub(super) fn fri08_c02_auto_fit_repeat<S: LayoutScalar>() -> TrackComponentOf<S> {
    TrackComponentOf::Repeat(
        TrackRepetitionOf::auto_fit_components(vec![TrackComponentOf::px(S::from_f64(40.0))])
            .expect("valid fixed auto-fit repetition"),
    )
}

pub(super) fn fri08_c02_auto_fit_output<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    viewport: Size<S>,
    node: u32,
) -> NodeOutputOf<S> {
    let batch = compute_layout(
        tree,
        1,
        LayoutRootRequestOf::viewport(viewport.map(AvailableOf::definite))
            .expect("finite auto-fit viewport"),
    )
    .expect("valid auto-fit grid layout");
    fri08_c01_placement_output(&batch, node)
}

pub(super) fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}

pub(super) fn fri06_c07_height_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .expect("public layout batch contains the requested node")
        .output()
}

pub(super) fn fri06_mr02_geometry_error_largest_finite<S: LayoutScalar>() -> S {
    if core::mem::size_of::<S>() == core::mem::size_of::<f32>() {
        S::from_f64(f32::MAX.into())
    } else {
        S::from_f64(f64::MAX)
    }
}

pub(super) fn fri06_mr02_geometry_error_input<S: LayoutScalar>(
    run_mode: RunMode,
) -> ComputeInputOf<S> {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
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

pub(super) fn fri06_mr02_geometry_error_assert<S: LayoutScalar, M>(
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

pub(super) fn assert_fri06_mr02_geometry_error_grid_own<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite();
    let style = NodeInputOf {
        display: Display::Grid,
        size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        border: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    };

    for (run_mode, operation, invariant) in [
        (
            RunMode::PerformRootLayout,
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (
            RunMode::PerformLayout,
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    ] {
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(7, [])
            .style(7, style.clone());
        let error = compute_grid(&mut tree, 7, fri06_mr02_geometry_error_input(run_mode))
            .expect_err("overflowing grid geometry must fail");

        fri06_mr02_geometry_error_assert(error, LayoutErrorSiteOf::Node(7), operation, invariant);
    }
}

pub(super) fn fri05_c05_grid_sizing_input(size: Size<Option<Scalar>>) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        size,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        size.map(|value| value.map_or(Available::MAX_CONTENT, Available::Definite)),
    )
}

pub(super) fn track_component_flex<S: LayoutScalar>(value: S) -> TrackComponentOf<S> {
    TrackComponentOf::flex(TrackFlexFactorOf::try_new(value).expect("valid test track flex factor"))
}

pub(super) fn fri04_c04_grid_dispatch_input(parent: Size<Option<f32>>) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        parent,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::new(
            parent
                .width
                .map_or(Available::MAX_CONTENT, Available::Definite),
            parent
                .height
                .map_or(Available::MAX_CONTENT, Available::Definite),
        ),
    )
}

pub(super) fn fri04_c04_grid_dispatch_assert_error(
    display: Display,
    style: NodeInput,
    expected_property: SizingProperty,
    expected_behavior: SizingBehavior,
    expected_algorithm: SizingAlgorithm,
    expected_axis: PhysicalAxis,
    expected_node: u32,
) {
    let (mut tree, node) = if expected_node == 0 {
        (
            OracleTree::new()
                .children(0, [])
                .style(0, NodeInput { display, ..style }),
            0,
        )
    } else {
        (
            OracleTree::new()
                .children(0, [1])
                .children(1, [])
                .style(
                    0,
                    NodeInput {
                        display,
                        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                        grid_template_columns: vec![TrackComponent::px(100.0)],
                        grid_template_rows: vec![TrackComponent::px(80.0)],
                        ..NodeInput::default()
                    },
                )
                .style(1, style),
            0,
        )
    };
    let error = compute_grid(
        &mut tree,
        node,
        fri04_c04_grid_dispatch_input(Size::new(Some(100.0), Some(80.0))),
    )
    .expect_err("later-owned grid sizing must be rejected");
    assert_eq!(error.site(), LayoutErrorSite::Node(expected_node));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    let LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
        unsupported,
    )) = error.kind()
    else {
        panic!("expected exact sizing capability, got {:?}", error.kind());
    };
    assert_eq!(
        (
            unsupported.property(),
            unsupported.behavior(),
            unsupported.algorithm(),
            unsupported.axis(),
        ),
        (
            expected_property,
            expected_behavior,
            expected_algorithm,
            expected_axis,
        )
    );
}

pub(super) enum Fri04C04GridSizingValue {
    Preferred(PreferredSize),
    Minimum(MinSize),
    Maximum(MaxSize),
}

pub(super) fn fri04_c04_grid_dispatch_style(
    value: Fri04C04GridSizingValue,
    axis: PhysicalAxis,
) -> NodeInput {
    let mut style = NodeInput::default();
    match (value, axis) {
        (Fri04C04GridSizingValue::Preferred(value), PhysicalAxis::Horizontal) => {
            style.size.width = value;
        }
        (Fri04C04GridSizingValue::Preferred(value), PhysicalAxis::Vertical) => {
            style.size.height = value;
        }
        (Fri04C04GridSizingValue::Minimum(value), PhysicalAxis::Horizontal) => {
            style.min_size.width = value;
        }
        (Fri04C04GridSizingValue::Minimum(value), PhysicalAxis::Vertical) => {
            style.min_size.height = value;
        }
        (Fri04C04GridSizingValue::Maximum(value), PhysicalAxis::Horizontal) => {
            style.max_size.width = value;
        }
        (Fri04C04GridSizingValue::Maximum(value), PhysicalAxis::Vertical) => {
            style.max_size.height = value;
        }
    }
    style
}

pub(super) fn lp(absolute_px: Scalar, percent_fraction: Scalar) -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

pub(super) fn invalid_numeric_lp() -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(f32::MAX, f32::MAX).expect("test coefficients are finite")
}

pub(super) fn fri04_c03_grid_track_value(value: Scalar) -> SizingCalculation {
    SizingCalculation::value(LengthPercentageOf::px(value).expect("test sizing value is finite"))
}

pub(super) fn fri04_c03_grid_track_nested(
    minimum: Scalar,
    preferred: Scalar,
    maximum: Scalar,
) -> SizingCalculation {
    let preferred = SizingCalculation::max(vec![
        fri04_c03_grid_track_value(preferred),
        SizingCalculation::min(vec![
            fri04_c03_grid_track_value(preferred - 5.0),
            fri04_c03_grid_track_value(preferred + 5.0),
        ])
        .expect("nested minimum is nonempty"),
    ])
    .expect("nested maximum is nonempty");
    SizingCalculation::clamp(
        Some(fri04_c03_grid_track_value(minimum)),
        preferred,
        Some(fri04_c03_grid_track_value(maximum)),
    )
}

pub(super) fn fri04_c03_grid_track_percentage_nested(
    minimum: Scalar,
    percentage: Scalar,
    maximum: Scalar,
) -> SizingCalculation {
    SizingCalculation::clamp(
        Some(fri04_c03_grid_track_value(minimum)),
        SizingCalculation::max(vec![
            fri04_c03_grid_track_value(minimum - 5.0),
            SizingCalculation::value(
                LengthPercentageOf::from_percent_fraction(percentage)
                    .expect("test percentage is finite"),
            ),
        ])
        .expect("nested maximum is nonempty"),
        Some(fri04_c03_grid_track_value(maximum)),
    )
}

pub(super) fn baseline_measure(
    width: Scalar,
    height: Scalar,
    first_baseline: Option<Scalar>,
    last_baseline_from_bottom: Option<Scalar>,
) -> ComputeOutput {
    ComputeOutput::from_sizes_and_baselines(
        Size::new(width, height),
        Size::new(width, height),
        crate::Baselines {
            first: Point::new(None, first_baseline),
            last: Point::new(
                None,
                last_baseline_from_bottom.map(|from_bottom| height - from_bottom),
            ),
        },
    )
}

pub(super) fn vertical_baseline_measure(
    width: Scalar,
    height: Scalar,
    first_baseline: Option<Scalar>,
    last_baseline_from_right: Option<Scalar>,
) -> ComputeOutput {
    ComputeOutput::from_sizes_and_baselines(
        Size::new(width, height),
        Size::new(width, height),
        crate::Baselines {
            first: Point::new(first_baseline, None),
            last: Point::new(
                last_baseline_from_right.map(|from_right| width - from_right),
                None,
            ),
        },
    )
}

pub(super) fn compute_oracle_grid(tree: &mut OracleTree) {
    compute_root(
        tree,
        1,
        Size::new(Available::Definite(120.0), Available::Definite(120.0)),
    )
    .unwrap();
    round_layout(tree, 1).unwrap();
}

pub(super) fn compute_oracle_grid_output(tree: &mut OracleTree) -> ComputeOutput {
    crate::compute_grid(
        tree,
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(120.0), Some(120.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            Size::new(Available::Definite(120.0), Available::Definite(120.0)),
        ),
    )
    .unwrap()
}

pub(super) fn final_y(tree: &OracleTree, node: u32) -> Scalar {
    tree.final_layout(node)
        .expect("node should have a final layout")
        .location
        .y
}

pub(super) fn empty_subgrid_track() -> TrackComponent {
    TrackComponent::Subgrid(crate::SubgridTrack {
        name_components: Vec::new(),
    })
}

pub(super) fn inherited_placement_member(
    source: u32,
    axis: GridAxisKind,
    role: AncestorBaselineRole,
    selected_track: usize,
    target: f32,
) -> AncestorBaselineMember<u32> {
    let child_flow_axes = match axis {
        GridAxisKind::Column => FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr),
        GridAxisKind::Row => FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    };
    let physical_baseline = match (axis, role) {
        (GridAxisKind::Column, AncestorBaselineRole::First) => Point::new(Some(target), None),
        (GridAxisKind::Column, AncestorBaselineRole::Last) => {
            Point::new(Some(100.0 - target), None)
        }
        (GridAxisKind::Row, AncestorBaselineRole::First) => Point::new(None, Some(target)),
        (GridAxisKind::Row, AncestorBaselineRole::Last) => Point::new(None, Some(100.0 - target)),
    };
    ancestor_baseline_member(AncestorBaselineMemberInput {
        source,
        axis,
        ancestor_span: GridTrackSpan::new(selected_track + 1, selected_track + 2),
        alignment: match role {
            AncestorBaselineRole::First => AlignItems::Baseline,
            AncestorBaselineRole::Last => AlignItems::LastBaseline,
        },
        block_auto_margins: false,
        synthesized_baseline_cycle: false,
        output: ComputeOutput::from_sizes_and_baselines(
            Size::new(100.0, 100.0),
            Size::new(100.0, 100.0),
            Baselines {
                first: if role == AncestorBaselineRole::First {
                    physical_baseline
                } else {
                    Point::NONE
                },
                last: if role == AncestorBaselineRole::Last {
                    physical_baseline
                } else {
                    Point::NONE
                },
            },
        ),
        margin: Edges::all(0.0),
        child_flow_axes,
        containing_flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        start_adjustment: 0.0,
        end_adjustment: 0.0,
    })
    .expect("placement fixture member participates")
}

pub(super) fn inherited_placement_group(
    axis: GridAxisKind,
    role: AncestorBaselineRole,
    selected_track: usize,
    target: f32,
) -> AncestorBaselineGroup<u32> {
    AncestorBaselineGroup::reduce(
        1_u32,
        axis,
        match axis {
            GridAxisKind::Column => PhysicalAxis::Horizontal,
            GridAxisKind::Row => PhysicalAxis::Vertical,
        },
        4,
        [inherited_placement_member(
            91,
            axis,
            role,
            selected_track,
            target,
        )],
    )
}

macro_rules! owner_placement_boundary {
    (
        $parent_grid:expr,
        $current_grid:expr,
        $parent_span:expr,
        $reversed:expr,
        $parent_progression:expr,
        $current_progression:expr,
        $parent_first_frame_origins:expr,
        $parent_last_frame_origins:expr,
        $current_first_frame_origins:expr,
        $current_last_frame_origins:expr,
        $parent_gap:expr,
        $current_gap:expr,
        $start_mbp:expr,
        $end_mbp:expr $(,)?
    ) => {
        OwnerToCurrentPlacementBoundaryInput {
            parent_grid: $parent_grid,
            current_grid: $current_grid,
            parent_axis: GridAxisKind::Row,
            current_axis: GridAxisKind::Row,
            physical_axis: PhysicalAxis::Vertical,
            parent_progression: $parent_progression,
            current_progression: $current_progression,
            parent_span: $parent_span,
            reversed: $reversed,
            parent_first_frame_origins: $parent_first_frame_origins,
            parent_last_frame_origins: $parent_last_frame_origins,
            current_first_frame_origins: $current_first_frame_origins,
            current_last_frame_origins: $current_last_frame_origins,
            parent_boundary_gutters: &vec![
                $parent_gap;
                ($parent_first_frame_origins).len().saturating_sub(1)
            ],
            current_boundary_gutters: &vec![
                $current_gap;
                ($current_first_frame_origins).len().saturating_sub(1)
            ],
            parent_gap: $parent_gap,
            current_gap: $current_gap,
            start_mbp: $start_mbp,
            end_mbp: $end_mbp,
            inherited: true,
        }
    };
}

pub(super) fn inherited_placement_mapping(
    axis: GridAxisKind,
    reversed: bool,
    parent_span: GridTrackSpan,
    parent_gap: f32,
    current_gap: f32,
) -> CheckedOwnerToCurrentPlacementMap<u32, f32> {
    let physical_axis = match axis {
        GridAxisKind::Column => PhysicalAxis::Horizontal,
        GridAxisKind::Row => PhysicalAxis::Vertical,
    };
    let identity = CheckedOwnerToCurrentPlacementMap::identity(
        1_u32,
        axis,
        physical_axis,
        PhysicalProgression::Increasing,
        4,
    );
    let current_count = parent_span.checked_len().unwrap_or(0);
    let parent_origins = vec![0.0; 4];
    let current_origins = vec![0.0; current_count];
    let parent_boundary_gutters = vec![parent_gap; 3];
    let current_boundary_gutters = vec![current_gap; current_count.saturating_sub(1)];
    identity
        .compose(OwnerToCurrentPlacementBoundaryInput {
            parent_grid: 1,
            current_grid: 7,
            parent_axis: axis,
            current_axis: axis,
            physical_axis,
            parent_progression: PhysicalProgression::Increasing,
            current_progression: PhysicalProgression::Increasing,
            parent_span,
            reversed,
            parent_first_frame_origins: &parent_origins,
            parent_last_frame_origins: &parent_origins,
            current_first_frame_origins: &current_origins,
            current_last_frame_origins: &current_origins,
            parent_boundary_gutters: &parent_boundary_gutters,
            current_boundary_gutters: &current_boundary_gutters,
            parent_gap,
            current_gap,
            start_mbp: 0.0,
            end_mbp: 0.0,
            inherited: true,
        })
        .expect("placement fixture map composes")
}

pub(super) fn inherited_placement_witness(
    axis: GridAxisKind,
    role: AncestorBaselineRole,
    selected_local_track: usize,
) -> CurrentGridDirectWitness<u32> {
    CurrentGridDirectWitness::new(
        7,
        11,
        axis,
        GridTrackSpan::new(selected_local_track, selected_local_track + 1),
        role,
    )
}

pub(super) fn derive_inherited_placement(
    group: &AncestorBaselineGroup<u32>,
    axis: GridAxisKind,
    role: AncestorBaselineRole,
    selected_local_track: usize,
    reversed: bool,
    parent_gap: f32,
    current_gap: f32,
) -> Result<
    InheritedCurrentGridBaselinePlacement<u32, f32>,
    InheritedCurrentGridBaselinePlacementError,
> {
    InheritedCurrentGridBaselinePlacement::try_derive(
        group,
        InheritedCurrentGridBaselinePlacementInput {
            axis,
            physical_axis: match axis {
                GridAxisKind::Column => PhysicalAxis::Horizontal,
                GridAxisKind::Row => PhysicalAxis::Vertical,
            },
            mapping: inherited_placement_mapping(
                axis,
                reversed,
                GridTrackSpan::new(0, 4),
                parent_gap,
                current_gap,
            ),
            direct_witness: inherited_placement_witness(axis, role, selected_local_track),
            current_grid: 7,
            item: 11,
        },
    )
}

pub(super) fn fri06_c12_t08_inherited_baseline_gap_position(
    parent_gap: f32,
    child_gap: f32,
) -> f32 {
    let mut tree = OracleTree::new()
        .children(1, [2, 3])
        .children(2, [4])
        .children(3, [5])
        .children(4, [])
        .children(5, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::px(60.0), TrackComponent::px(60.0)],
                grid_template_rows: vec![TrackComponent::px(40.0), TrackComponent::px(40.0)],
                gap: Size::new(Length::ZERO, Length::px(parent_gap)),
                align_items: Some(AlignItems::Start),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Grid,
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                gap: Size::new(Length::ZERO, Length::px(parent_gap)),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(2).expect("valid grid line"),
                grid_row: GridPlacement::try_lines(1, 3).expect("valid grid lines"),
                grid_template_columns: vec![TrackComponent::px(60.0)],
                grid_template_rows: vec![empty_subgrid_track()],
                gap: Size::new(Length::ZERO, Length::px(child_gap)),
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid grid line"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .measure(4, baseline_measure(30.0, 20.0, Some(30.0), None))
        .measure(5, baseline_measure(30.0, 20.0, Some(8.0), None));

    compute_oracle_grid(&mut tree);

    final_y(&tree, 5)
}

pub(super) fn subgrid_track() -> Vec<TrackComponent> {
    subgrid_track_of()
}

pub(super) fn subgrid_track_of<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack {
        name_components: Vec::new(),
    })]
}

pub(super) fn fri08_c01_topology_for_style<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    column_basis: Option<S>,
    row_basis: Option<S>,
) -> topology::ExpandedGridTopology<S> {
    let columns = expand_track_components_with_origins(
        &style.grid_template_columns,
        column_basis,
        S::ZERO,
        None,
    )
    .expect("valid column topology input");
    let rows =
        expand_track_components_with_origins(&style.grid_template_rows, row_basis, S::ZERO, None)
            .expect("valid row topology input");
    let named = named::build_grid_named_context(
        &grid_container_projection!(style),
        columns.tracks.len(),
        rows.tracks.len(),
        &GridParentContext::none(),
    )
    .expect("valid named topology input");
    topology::ExpandedGridTopology::new(topology::ExpandedGridTopologyInput {
        container: GridContainerProjection::from_node(style),
        columns,
        rows,
        named,
        column_basis,
        row_basis,
        column_gap: S::ZERO,
        row_gap: S::ZERO,
        inherited_columns: false,
        inherited_rows: false,
    })
    .expect("valid canonical topology")
}

pub(super) fn local_line_names(line_names: &[Vec<named::LineNameEntry>]) -> Vec<Vec<&str>> {
    line_names
        .iter()
        .map(|entries| entry_names(entries))
        .collect()
}

pub(super) fn entry_names(entries: &[named::LineNameEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.name.as_str()).collect()
}

pub(super) fn tagged_baseline<S: LayoutScalar>(
    axis: PhysicalAxis,
    coordinate: S,
) -> crate::output::PhysicalBaseline<S> {
    let flow_axes = match axis {
        PhysicalAxis::Horizontal => {
            crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr)
        }
        PhysicalAxis::Vertical => {
            crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
        }
    };
    let baselines = match axis {
        PhysicalAxis::Horizontal => BaselinesOf {
            first: Point::new(Some(coordinate), None),
            last: Point::NONE,
        },
        PhysicalAxis::Vertical => BaselinesOf {
            first: Point::new(None, Some(coordinate)),
            last: Point::NONE,
        },
    };

    baselines
        .first_block_baseline(flow_axes)
        .expect("the tagged physical baseline is present")
}

pub(super) fn tagged_group<S: LayoutScalar>(
    axis: PhysicalAxis,
    first: Option<S>,
    last: Option<S>,
) -> TrackBaselineGroup<S> {
    TrackBaselineGroup {
        first: first.map(|coordinate| tagged_baseline(axis, coordinate)),
        last: last.map(|coordinate| tagged_baseline(axis, coordinate)),
    }
}

pub(super) fn traversal_leaf(node: u32, start: usize, end: usize) -> SubgridTraversalChild<u32> {
    SubgridTraversalChild::Leaf(SubgridTraversalLeaf {
        node,
        style: default_grid_item_projection(),
        span_in_parent: GridTrackSpan::new(start, end),
        available_inline_size: None,
        available_inline_size_is_known: false,
        align_self: AlignItems::Stretch,
    })
}
