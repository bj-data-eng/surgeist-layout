use super::placement::GridAxis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleWritingMode {
    HorizontalTb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisMappingInput {
    pub queried_axis: GridAxis,
    pub parent_writing_mode: OracleWritingMode,
    pub child_writing_mode: OracleWritingMode,
    pub parent_direction: OracleDirection,
    pub child_direction: OracleDirection,
    pub parent_flipped_in_resolved_axis: bool,
    pub child_flipped_in_resolved_axis: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisMappingReport {
    pub queried_axis: GridAxis,
    pub parent_axis: GridAxis,
    pub child_axis: GridAxis,
    pub parent_writing_mode: OracleWritingMode,
    pub child_writing_mode: OracleWritingMode,
    pub parent_direction: OracleDirection,
    pub child_direction: OracleDirection,
    pub parent_flipped_in_resolved_axis: bool,
    pub child_flipped_in_resolved_axis: bool,
    pub reversed: bool,
}

pub fn map_axis(input: AxisMappingInput) -> AxisMappingReport {
    AxisMappingReport {
        queried_axis: input.queried_axis,
        parent_axis: input.queried_axis,
        child_axis: input.queried_axis,
        parent_writing_mode: input.parent_writing_mode,
        child_writing_mode: input.child_writing_mode,
        parent_direction: input.parent_direction,
        child_direction: input.child_direction,
        parent_flipped_in_resolved_axis: input.parent_flipped_in_resolved_axis,
        child_flipped_in_resolved_axis: input.child_flipped_in_resolved_axis,
        reversed: input.parent_flipped_in_resolved_axis != input.child_flipped_in_resolved_axis,
    }
}

#[must_use]
pub const fn opposite_axis(axis: GridAxis) -> GridAxis {
    match axis {
        GridAxis::Column => GridAxis::Row,
        GridAxis::Row => GridAxis::Column,
    }
}
