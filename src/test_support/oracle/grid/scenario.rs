//! Curated grid oracle scenarios composed from tested phase outputs.
//!
//! This module may combine explicit reports into final rectangles. It must not
//! traverse production trees, measure children, parse styles, or call production
//! layout algorithms.

use super::alignment::AlignmentReport;
use super::placement::{GridAxis, PlacementReport};
use super::subgrid::AxisEdges;
use super::tracks::TrackSizingReport;

#[derive(Clone, Debug, PartialEq)]
pub struct GridScenarioReport {
    pub placement: PlacementReport,
    pub columns: TrackSizingReport,
    pub rows: TrackSizingReport,
    pub column_alignment: AlignmentReport,
    pub row_alignment: AlignmentReport,
    pub item_rects: Vec<GridItemRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridItemRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl GridItemRect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[must_use]
pub fn compose_grid_scenario(
    placement: PlacementReport,
    columns: TrackSizingReport,
    rows: TrackSizingReport,
    column_alignment: AlignmentReport,
    row_alignment: AlignmentReport,
) -> GridScenarioReport {
    let item_rects = placement
        .areas
        .iter()
        .map(|area| {
            let (x, width) = axis_rect(
                area.column_start,
                area.column_span,
                &columns,
                &column_alignment,
                Axis::Column,
            );
            let (y, height) = axis_rect(
                area.row_start,
                area.row_span,
                &rows,
                &row_alignment,
                Axis::Row,
            );
            GridItemRect::new(x, y, width, height)
        })
        .collect();

    GridScenarioReport {
        placement,
        columns,
        rows,
        column_alignment,
        row_alignment,
        item_rects,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Axis {
    Column,
    Row,
}

fn axis_rect(
    start: usize,
    span: usize,
    tracks: &TrackSizingReport,
    alignment: &AlignmentReport,
    axis: Axis,
) -> (f32, f32) {
    assert!(span > 0, "{axis:?} span must be positive");
    assert!(start > 0, "{axis:?} start line must be 1-based");
    let start_index = start - 1;
    let end_index = start_index + span - 1;
    assert!(
        end_index < tracks.final_tracks.len(),
        "{axis:?} area must fit solved tracks"
    );
    assert!(
        end_index < alignment.offsets.len(),
        "{axis:?} area must fit aligned tracks"
    );

    let start_offset = alignment.offsets[start_index];
    let end_offset = alignment.offsets[end_index] + tracks.final_tracks[end_index].size;
    (start_offset, end_offset - start_offset)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubgridItemRectInput {
    pub inherited_axis: GridAxis,
    pub inherited_axis_offset: f32,
    pub standalone_axis_offset: f32,
    pub inherited_axis_size: f32,
    pub standalone_axis_size: f32,
    pub container_mbp_offset: AxisEdges,
    pub item_inline_offset: f32,
    pub item_block_offset: f32,
}

#[must_use]
pub fn compose_subgrid_item_rect(input: SubgridItemRectInput) -> SubgridItemRectReport {
    let inherited_axis_offset =
        input.inherited_axis_offset + input.container_mbp_offset.start + input.item_inline_offset;
    let standalone_axis_offset = input.standalone_axis_offset + input.item_block_offset;
    let rect = match input.inherited_axis {
        GridAxis::Column => GridItemRect::new(
            inherited_axis_offset,
            standalone_axis_offset,
            input.inherited_axis_size,
            input.standalone_axis_size,
        ),
        GridAxis::Row => GridItemRect::new(
            standalone_axis_offset,
            inherited_axis_offset,
            input.standalone_axis_size,
            input.inherited_axis_size,
        ),
    };

    SubgridItemRectReport {
        inherited_axis_offset,
        standalone_axis_offset,
        rect,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubgridItemRectReport {
    pub inherited_axis_offset: f32,
    pub standalone_axis_offset: f32,
    pub rect: GridItemRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneItemRectInput {
    pub grid_axis_start: f32,
    pub grid_axis_size: f32,
    pub lane_axis_offset: f32,
    pub lane_axis_size: f32,
    pub grid_axis_is_column: bool,
}

#[must_use]
pub fn compose_lane_item_rect(input: LaneItemRectInput) -> GridItemRect {
    if input.grid_axis_is_column {
        GridItemRect::new(
            input.grid_axis_start,
            input.lane_axis_offset,
            input.grid_axis_size,
            input.lane_axis_size,
        )
    } else {
        GridItemRect::new(
            input.lane_axis_offset,
            input.grid_axis_start,
            input.lane_axis_size,
            input.grid_axis_size,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineAlignedItemRectInput {
    pub area_x: f32,
    pub area_y: f32,
    pub area_width: f32,
    pub area_height: f32,
    pub item_width: f32,
    pub item_height: f32,
    pub normal_x_offset: f32,
    pub normal_y_offset: f32,
    pub baseline_y_offset: Option<f32>,
}

#[must_use]
pub fn compose_baseline_aligned_item_rect(input: BaselineAlignedItemRectInput) -> GridItemRect {
    let y_offset = input.baseline_y_offset.unwrap_or(input.normal_y_offset);

    GridItemRect::new(
        input.area_x + input.normal_x_offset,
        input.area_y + y_offset,
        input.item_width,
        input.item_height,
    )
}
