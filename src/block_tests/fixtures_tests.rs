use super::*;
pub(super) type BlockTree = OracleTree;
pub(super) type CalcBlockTree = OracleTree;

pub(super) fn computed_overflow(x: Overflow, y: Overflow) -> ComputedOverflow {
    ComputedOverflow::try_new(x, y).expect("test overflow pair must already be canonical")
}
pub(super) fn fri06_atomic_participation<S: LayoutScalar>() -> AtomicInlineParticipationOf<S> {
    AtomicInlineParticipationOf::try_new(
        BidiLevel::try_new(0).unwrap(),
        InlineBreakOpportunityOf::allowed(),
    )
    .unwrap()
}

pub(super) fn lp(absolute_px: Scalar, percent_fraction: Scalar) -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
        .expect("test coefficients are finite")
}

#[derive(Clone, Debug, Default)]
pub(super) enum ShapeProviderBehavior<S: LayoutScalar> {
    #[default]
    Missing,
    Failure,
    Empty,
    Interval {
        minimum: S,
        maximum: S,
    },
    Mismatch {
        query: FloatExclusionQueryOf<S>,
        minimum: S,
        maximum: S,
    },
    Bands(Vec<(S, S, S, S)>),
}

#[derive(Default)]
pub(super) struct PublicBlockTree<S: LayoutScalar> {
    children: HashMap<u32, Vec<u32>>,
    styles: HashMap<u32, NodeInputOf<S>>,
    layout_inputs: HashMap<u32, LayoutInputOf<S>>,
    leaf_nodes: HashSet<u32>,
    leaf_measurements: HashMap<u32, Size<S>>,
    shape_provider: ShapeProviderBehavior<S>,
    shape_queries: Mutex<Vec<(u32, FloatExclusionQueryOf<S>)>>,
}

impl<S: LayoutScalar> PublicBlockTree<S> {
    pub(super) fn with_children(
        mut self,
        node: u32,
        children: impl IntoIterator<Item = u32>,
    ) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub(super) fn with_style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.styles.insert(node, style);
        self
    }

    pub(super) fn with_layout_input(mut self, node: u32, input: LayoutInputOf<S>) -> Self {
        self.layout_inputs.insert(node, input);
        self
    }

    pub(super) fn with_measurement(mut self, node: u32, size: Size<S>) -> Self {
        self.leaf_nodes.insert(node);
        self.leaf_measurements.insert(node, size);
        self
    }

    pub(super) fn with_shape_provider(mut self, behavior: ShapeProviderBehavior<S>) -> Self {
        self.shape_provider = behavior;
        self
    }

    pub(super) fn shape_queries(&self) -> Vec<(u32, FloatExclusionQueryOf<S>)> {
        self.shape_queries.lock().unwrap().clone()
    }
}

impl<S: LayoutScalar> Traverse for PublicBlockTree<S> {
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

impl<S: LayoutScalar> LayoutTree for PublicBlockTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        &self.styles[&node]
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputOf<S> {
        self.layout_inputs
            .get(&node)
            .cloned()
            .unwrap_or_else(|| LayoutInputOf::box_input(self.styles[&node].clone()))
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.leaf_nodes.contains(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        _input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.leaf_measurements.get(&node).copied().map(Ok)
    }

    fn float_exclusion_interval(
        &self,
        node: Self::Node,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        self.shape_queries.lock().unwrap().push((node, query));
        match &self.shape_provider {
            ShapeProviderBehavior::Missing => None,
            ShapeProviderBehavior::Failure => Some(Err(())),
            ShapeProviderBehavior::Empty => Some(Ok(None)),
            ShapeProviderBehavior::Interval { minimum, maximum } => Some(Ok(
                FloatExclusionIntervalOf::try_new(query, *minimum, *maximum)
                    .expect("test provider endpoints are valid"),
            )),
            ShapeProviderBehavior::Mismatch {
                query,
                minimum,
                maximum,
            } => Some(Ok(FloatExclusionIntervalOf::try_new(
                *query, *minimum, *maximum,
            )
            .expect("mismatch provider endpoints are valid"))),
            ShapeProviderBehavior::Bands(bands) => {
                let interval = bands
                    .iter()
                    .find(|(band_minimum, band_maximum, _, _)| {
                        query.band_minimum() == *band_minimum
                            && query.band_maximum() == *band_maximum
                    })
                    .map(|(_, _, minimum, maximum)| {
                        FloatExclusionIntervalOf::try_new(query, *minimum, *maximum)
                            .expect("fixture band endpoints are valid")
                    });
                Some(Ok(interval.flatten()))
            }
        }
    }
}

pub(super) fn public_final_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("public layout batch contains the requested node")
        .output()
}

pub(super) fn scalar_value<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

pub(super) fn fri05_c03_block_union_content_size<S: LayoutScalar>(
    output: NodeOutputOf<S>,
) -> Size<S> {
    let geometry = output
        .scroll_geometry
        .expect("a performed block has canonical geometry");
    let anchor = geometry.content_box().origin();
    let overflow = geometry.scrollable_overflow();
    let overflow_origin = overflow.origin();
    let overflow_size = overflow.size();
    let overflow_end = Point::new(
        overflow_origin.x + overflow_size.width,
        overflow_origin.y + overflow_size.height,
    );

    Size::new(
        anchor.x.max(overflow_end.x) - anchor.x.min(overflow_origin.x),
        anchor.y.max(overflow_end.y) - anchor.y.min(overflow_origin.y),
    )
}

pub(super) fn all_writing_mode_directions() -> [(WritingMode, Direction); 10] {
    [
        (WritingMode::HorizontalTb, Direction::Ltr),
        (WritingMode::HorizontalTb, Direction::Rtl),
        (WritingMode::VerticalRl, Direction::Ltr),
        (WritingMode::VerticalRl, Direction::Rtl),
        (WritingMode::VerticalLr, Direction::Ltr),
        (WritingMode::VerticalLr, Direction::Rtl),
        (WritingMode::SidewaysRl, Direction::Ltr),
        (WritingMode::SidewaysRl, Direction::Rtl),
        (WritingMode::SidewaysLr, Direction::Ltr),
        (WritingMode::SidewaysLr, Direction::Rtl),
    ]
}
