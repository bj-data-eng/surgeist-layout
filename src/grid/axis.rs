use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridAxisKind {
    Column,
    Row,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GridAxisMappingError {
    #[expect(
        dead_code,
        reason = "reserved for staged vertical writing-mode grid axis validation parity"
    )]
    VerticalWritingModeUnsupported,
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
) -> Result<GridAxisMappingReport, GridAxisMappingError> {
    let parent_axis = if input.parent_style.writing_mode.is_vertical()
        != input.child_style.writing_mode.is_vertical()
    {
        match input.queried_axis {
            GridAxisKind::Column => GridAxisKind::Row,
            GridAxisKind::Row => GridAxisKind::Column,
        }
    } else {
        input.queried_axis
    };

    Ok(GridAxisMappingReport {
        queried_axis: input.queried_axis,
        parent_axis,
        child_axis: input.queried_axis,
        reversed: axis_flipped(input.parent_style, parent_axis)
            != axis_flipped(input.child_style, input.queried_axis),
    })
}

const fn axis_flipped<S: LayoutScalar>(style: &NodeInputOf<S>, axis: GridAxisKind) -> bool {
    match (style.writing_mode, axis) {
        (crate::WritingMode::HorizontalTb, GridAxisKind::Column) => style.direction.is_rtl(),
        (crate::WritingMode::VerticalLr, GridAxisKind::Column)
        | (crate::WritingMode::VerticalRl, GridAxisKind::Column) => style.direction.is_rtl(),
        (crate::WritingMode::VerticalRl, GridAxisKind::Row) => true,
        _ => false,
    }
}
