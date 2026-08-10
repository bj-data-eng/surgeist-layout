use super::*;
use crate::geometry::{FlowAxes, LogicalAxis};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridAxisKind {
    Column,
    Row,
}

impl GridAxisKind {
    pub(super) const fn logical_axis(self) -> LogicalAxis {
        match self {
            Self::Column => LogicalAxis::Inline,
            Self::Row => LogicalAxis::Block,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct GridAxisMappingInput<'a, S: LayoutScalar = Scalar> {
    pub(super) queried_axis: GridAxisKind,
    pub(super) parent_style: &'a NodeInputOf<S>,
    pub(super) child_style: &'a NodeInputOf<S>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GridAxisMappingReport {
    pub(super) queried_axis: GridAxisKind,
    pub(super) parent_axis: GridAxisKind,
    pub(super) child_axis: GridAxisKind,
    pub(super) reversed: bool,
}

pub(super) fn map_grid_axis<S: LayoutScalar>(
    input: GridAxisMappingInput<'_, S>,
) -> GridAxisMappingReport {
    let parent_flow = FlowAxes::new(
        input.parent_style.writing_mode,
        input.parent_style.direction,
    );
    let child_flow = FlowAxes::new(input.child_style.writing_mode, input.child_style.direction);
    let logical_axis = input.queried_axis.logical_axis();
    let physical_axis = match logical_axis {
        LogicalAxis::Inline => child_flow.inline_axis(),
        LogicalAxis::Block => child_flow.block_axis(),
    };
    let parent_axis = if parent_flow.inline_axis() == physical_axis {
        GridAxisKind::Column
    } else {
        GridAxisKind::Row
    };

    GridAxisMappingReport {
        queried_axis: input.queried_axis,
        parent_axis,
        child_axis: input.queried_axis,
        reversed: parent_flow.physical_axis_progression(physical_axis)
            != child_flow.physical_axis_progression(physical_axis),
    }
}
