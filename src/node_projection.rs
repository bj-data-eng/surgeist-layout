use crate::{
    AspectRatioOf, BoxSizing, ComputedOverflow, Edges, FlowAxes, LayoutScalar, LengthAutoOf,
    LengthOf, MaxSizeOf, MinSizeOf, NodeInputOf, Position, PreferredSizeOf, Size,
};

/// Borrowed box facts whose semantics are identical across formatting algorithms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CommonBoxProjection<'a, S: LayoutScalar> {
    pub(crate) size: &'a Size<PreferredSizeOf<S>>,
    pub(crate) min_size: &'a Size<MinSizeOf<S>>,
    pub(crate) max_size: &'a Size<MaxSizeOf<S>>,
    pub(crate) aspect_ratio: &'a Option<AspectRatioOf<S>>,
    pub(crate) margin: &'a Edges<LengthAutoOf<S>>,
    pub(crate) padding: &'a Edges<LengthOf<S>>,
    pub(crate) border: &'a Edges<LengthOf<S>>,
    pub(crate) box_sizing: BoxSizing,
    pub(crate) flow_axes: FlowAxes,
    pub(crate) overflow: ComputedOverflow,
    pub(crate) position: Position,
    pub(crate) inset: &'a Edges<LengthAutoOf<S>>,
    pub(crate) item_is_replaced: bool,
    pub(crate) item_is_table: bool,
}

impl<'a, S: LayoutScalar> CommonBoxProjection<'a, S> {
    #[must_use]
    pub(crate) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            size: &input.size,
            min_size: &input.min_size,
            max_size: &input.max_size,
            aspect_ratio: &input.aspect_ratio,
            margin: &input.margin,
            padding: &input.padding,
            border: &input.border,
            box_sizing: input.box_sizing,
            flow_axes: FlowAxes::new(input.writing_mode, input.direction),
            overflow: input.overflow,
            position: input.position,
            inset: &input.inset,
            item_is_replaced: input.item_is_replaced,
            item_is_table: input.item_is_table,
        }
    }
}
