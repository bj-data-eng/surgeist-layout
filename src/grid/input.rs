use crate::node_projection::CommonBoxProjection;
use crate::{
    AlignContent, AlignItems, AspectRatioOf, BoxSizing, ComputedOverflow, Display, Edges, FlowAxes,
    GridAutoFlow, GridFlowToleranceOf, GridPlacement, GridTemplateAreas, ItemOrder, LayoutScalar,
    LengthAutoOf, LengthOf, MaxSizeOf, MinSizeOf, NodeInputOf, OverflowClipMarginOf, Position,
    PreferredSizeOf, RawGridPlacement, ScrollMarginOf, ScrollPaddingOf, ScrollSnapAlign,
    ScrollSnapStop, ScrollSnapType, ScrollbarGutter, ScrollbarWidthOf, Size, TrackComponentOf,
};

pub(super) trait GridOverflowFacts {
    fn computed_overflow(&self) -> ComputedOverflow;

    fn item_is_replaced(&self) -> bool;
}

impl<T: GridOverflowFacts + ?Sized> GridOverflowFacts for &T {
    fn computed_overflow(&self) -> ComputedOverflow {
        (*self).computed_overflow()
    }

    fn item_is_replaced(&self) -> bool {
        (*self).item_is_replaced()
    }
}

impl<S: LayoutScalar> GridOverflowFacts for NodeInputOf<S> {
    fn computed_overflow(&self) -> ComputedOverflow {
        self.overflow
    }

    fn item_is_replaced(&self) -> bool {
        self.item_is_replaced
    }
}

impl<S: LayoutScalar> GridOverflowFacts for GridContainerProjection<'_, S> {
    fn computed_overflow(&self) -> ComputedOverflow {
        self.overflow
    }

    fn item_is_replaced(&self) -> bool {
        self.item_is_replaced
    }
}

/// Grid-container facts settled at the grid algorithm entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridContainerProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) display: Display,
    pub(super) item_is_replaced: bool,
    pub(super) item_is_table: bool,
    pub(super) box_sizing: BoxSizing,
    pub(super) writing_mode: crate::WritingMode,
    pub(super) direction: crate::Direction,
    pub(super) overflow: ComputedOverflow,
    pub(super) overflow_clip_margin: OverflowClipMarginOf<S>,
    pub(super) scroll_padding: ScrollPaddingOf<S>,
    pub(super) scroll_margin: ScrollMarginOf<S>,
    pub(super) scroll_snap_type: ScrollSnapType,
    pub(super) scroll_snap_align: ScrollSnapAlign,
    pub(super) scroll_snap_stop: ScrollSnapStop,
    pub(super) position: Position,
    pub(super) inset: &'a Edges<LengthAutoOf<S>>,
    pub(super) size: &'a Size<PreferredSizeOf<S>>,
    pub(super) min_size: &'a Size<MinSizeOf<S>>,
    pub(super) max_size: &'a Size<MaxSizeOf<S>>,
    pub(super) aspect_ratio: &'a Option<AspectRatioOf<S>>,
    pub(super) margin: &'a Edges<LengthAutoOf<S>>,
    pub(super) padding: &'a Edges<LengthOf<S>>,
    pub(super) border: &'a Edges<LengthOf<S>>,
    pub(super) align_items: Option<AlignItems>,
    pub(super) justify_items: Option<AlignItems>,
    pub(super) align_content: Option<AlignContent>,
    pub(super) justify_content: Option<AlignContent>,
    pub(super) gap: &'a Size<LengthOf<S>>,
    pub(super) grid_template_columns: &'a [TrackComponentOf<S>],
    pub(super) grid_template_rows: &'a [TrackComponentOf<S>],
    pub(super) grid_template_areas: &'a GridTemplateAreas,
    pub(super) grid_auto_columns: &'a [TrackComponentOf<S>],
    pub(super) grid_auto_rows: &'a [TrackComponentOf<S>],
    pub(super) grid_auto_flow: GridAutoFlow,
    pub(super) grid_flow_tolerance: GridFlowToleranceOf<S>,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<'a, S: LayoutScalar> GridContainerProjection<'a, S> {
    #[must_use]
    pub(super) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            display: input.display,
            item_is_replaced: input.item_is_replaced,
            item_is_table: input.item_is_table,
            box_sizing: input.box_sizing,
            writing_mode: input.writing_mode,
            direction: input.direction,
            overflow: input.overflow,
            overflow_clip_margin: input.overflow_clip_margin,
            scroll_padding: input.scroll_padding,
            scroll_margin: input.scroll_margin,
            scroll_snap_type: input.scroll_snap_type,
            scroll_snap_align: input.scroll_snap_align,
            scroll_snap_stop: input.scroll_snap_stop,
            position: input.position,
            inset: &input.inset,
            size: &input.size,
            min_size: &input.min_size,
            max_size: &input.max_size,
            aspect_ratio: &input.aspect_ratio,
            margin: &input.margin,
            padding: &input.padding,
            border: &input.border,
            align_items: input.align_items,
            justify_items: input.justify_items,
            align_content: input.align_content,
            justify_content: input.justify_content,
            gap: &input.gap,
            grid_template_columns: &input.grid_template_columns,
            grid_template_rows: &input.grid_template_rows,
            grid_template_areas: &input.grid_template_areas,
            grid_auto_columns: &input.grid_auto_columns,
            grid_auto_rows: &input.grid_auto_rows,
            grid_auto_flow: input.grid_auto_flow,
            grid_flow_tolerance: input.grid_flow_tolerance,
            scrollbar_gutter: input.scrollbar_gutter,
            scrollbar_width: input.scrollbar_width,
        }
    }
}

/// Owns the complete public snapshot only long enough to lend a container projection.
pub(super) struct GridContainerInput<S: LayoutScalar> {
    source: NodeInputOf<S>,
}

impl<S: LayoutScalar> GridContainerInput<S> {
    #[must_use]
    pub(super) fn from_node(source: &NodeInputOf<S>) -> Self {
        Self {
            source: source.clone(),
        }
    }

    #[must_use]
    pub(super) fn projection(&self) -> GridContainerProjection<'_, S> {
        GridContainerProjection::from_node(&self.source)
    }

    pub(super) fn suppress_intrinsic_minimum(&mut self, axes: Size<bool>) {
        if axes.width {
            self.source.min_size.width = MinSizeOf::AUTO;
        }
        if axes.height {
            self.source.min_size.height = MinSizeOf::AUTO;
        }
    }
}

macro_rules! project_grid_container {
    ($tree:expr, $node:expr) => {
        $crate::grid::input::GridContainerInput::from_node($tree.node_input($node))
    };
}

pub(super) use project_grid_container;

macro_rules! project_grid_item {
    ($tree:expr, $node:expr) => {
        $crate::grid::input::GridItemProjection::from_node($tree.node_input($node))
    };
}

pub(super) use project_grid_item;

#[must_use]
pub(super) fn order_modified_indexes(
    items: &[(ItemOrder, crate::SourceIndex)],
) -> Vec<crate::SourceIndex> {
    crate::node_input::item_order_permutation(items)
}

/// Owned grid-item facts settled once while collecting one child.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct GridItemProjection<S: LayoutScalar> {
    pub(super) display: Display,
    pub(super) item_is_table: bool,
    pub(super) item_is_replaced: bool,
    pub(super) item_order: ItemOrder,
    pub(super) box_sizing: BoxSizing,
    pub(super) flow_axes: FlowAxes,
    pub(super) writing_mode: crate::WritingMode,
    pub(super) direction: crate::Direction,
    pub(super) overflow: ComputedOverflow,
    pub(super) overflow_clip_margin: OverflowClipMarginOf<S>,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
    pub(super) scroll_padding: ScrollPaddingOf<S>,
    pub(super) scroll_margin: ScrollMarginOf<S>,
    pub(super) scroll_snap_type: ScrollSnapType,
    pub(super) scroll_snap_align: ScrollSnapAlign,
    pub(super) scroll_snap_stop: ScrollSnapStop,
    pub(super) position: Position,
    pub(super) inset: Edges<LengthAutoOf<S>>,
    pub(super) size: Size<PreferredSizeOf<S>>,
    pub(super) min_size: Size<MinSizeOf<S>>,
    pub(super) max_size: Size<MaxSizeOf<S>>,
    pub(super) aspect_ratio: Option<AspectRatioOf<S>>,
    pub(super) margin: Edges<LengthAutoOf<S>>,
    pub(super) padding: Edges<LengthOf<S>>,
    pub(super) border: Edges<LengthOf<S>>,
    pub(super) align_self: Option<AlignItems>,
    pub(super) justify_self: Option<AlignItems>,
    pub(super) align_items: Option<AlignItems>,
    pub(super) justify_items: Option<AlignItems>,
    pub(super) align_content: Option<AlignContent>,
    pub(super) justify_content: Option<AlignContent>,
    pub(super) gap: Size<LengthOf<S>>,
    pub(super) grid_template_columns: Vec<TrackComponentOf<S>>,
    pub(super) grid_template_rows: Vec<TrackComponentOf<S>>,
    pub(super) grid_template_areas: GridTemplateAreas,
    pub(super) grid_auto_columns: Vec<TrackComponentOf<S>>,
    pub(super) grid_auto_rows: Vec<TrackComponentOf<S>>,
    pub(super) grid_auto_flow: GridAutoFlow,
    pub(super) grid_flow_tolerance: GridFlowToleranceOf<S>,
    pub(super) grid_column: GridPlacement,
    pub(super) grid_row: GridPlacement,
    pub(super) raw_grid_column: RawGridPlacement,
    pub(super) raw_grid_row: RawGridPlacement,
}

impl<S: LayoutScalar> GridItemProjection<S> {
    #[must_use]
    pub(super) fn from_node(input: &NodeInputOf<S>) -> Self {
        Self {
            display: input.display,
            item_is_table: input.item_is_table,
            item_is_replaced: input.item_is_replaced,
            item_order: input.item_order,
            box_sizing: input.box_sizing,
            flow_axes: FlowAxes::new(input.writing_mode, input.direction),
            writing_mode: input.writing_mode,
            direction: input.direction,
            overflow: input.overflow,
            overflow_clip_margin: input.overflow_clip_margin,
            scrollbar_gutter: input.scrollbar_gutter,
            scrollbar_width: input.scrollbar_width,
            scroll_padding: input.scroll_padding,
            scroll_margin: input.scroll_margin,
            scroll_snap_type: input.scroll_snap_type,
            scroll_snap_align: input.scroll_snap_align,
            scroll_snap_stop: input.scroll_snap_stop,
            position: input.position,
            inset: input.inset,
            size: input.size.clone(),
            min_size: input.min_size.clone(),
            max_size: input.max_size.clone(),
            aspect_ratio: input.aspect_ratio,
            margin: input.margin,
            padding: input.padding,
            border: input.border,
            align_self: input.align_self,
            justify_self: input.justify_self,
            align_items: input.align_items,
            justify_items: input.justify_items,
            align_content: input.align_content,
            justify_content: input.justify_content,
            gap: input.gap,
            grid_template_columns: input.grid_template_columns.clone(),
            grid_template_rows: input.grid_template_rows.clone(),
            grid_template_areas: input.grid_template_areas.clone(),
            grid_auto_columns: input.grid_auto_columns.clone(),
            grid_auto_rows: input.grid_auto_rows.clone(),
            grid_auto_flow: input.grid_auto_flow,
            grid_flow_tolerance: input.grid_flow_tolerance,
            grid_column: input.grid_column,
            grid_row: input.grid_row,
            raw_grid_column: input.raw_grid_column.clone(),
            raw_grid_row: input.raw_grid_row.clone(),
        }
    }

    #[must_use]
    pub(super) fn nested_container_projection(&self) -> GridContainerProjection<'_, S> {
        GridContainerProjection {
            common: CommonBoxProjection {
                size: &self.size,
                min_size: &self.min_size,
                max_size: &self.max_size,
                aspect_ratio: &self.aspect_ratio,
                margin: &self.margin,
                padding: &self.padding,
                border: &self.border,
                box_sizing: self.box_sizing,
                flow_axes: self.flow_axes,
                overflow: self.overflow,
                position: self.position,
                inset: &self.inset,
                item_is_replaced: self.item_is_replaced,
                item_is_table: self.item_is_table,
            },
            display: self.display,
            item_is_replaced: self.item_is_replaced,
            item_is_table: self.item_is_table,
            box_sizing: self.box_sizing,
            writing_mode: self.writing_mode,
            direction: self.direction,
            overflow: self.overflow,
            overflow_clip_margin: self.overflow_clip_margin,
            scroll_padding: self.scroll_padding,
            scroll_margin: self.scroll_margin,
            scroll_snap_type: self.scroll_snap_type,
            scroll_snap_align: self.scroll_snap_align,
            scroll_snap_stop: self.scroll_snap_stop,
            position: self.position,
            inset: &self.inset,
            size: &self.size,
            min_size: &self.min_size,
            max_size: &self.max_size,
            aspect_ratio: &self.aspect_ratio,
            margin: &self.margin,
            padding: &self.padding,
            border: &self.border,
            align_items: self.align_items,
            justify_items: self.justify_items,
            align_content: self.align_content,
            justify_content: self.justify_content,
            gap: &self.gap,
            grid_template_columns: &self.grid_template_columns,
            grid_template_rows: &self.grid_template_rows,
            grid_template_areas: &self.grid_template_areas,
            grid_auto_columns: &self.grid_auto_columns,
            grid_auto_rows: &self.grid_auto_rows,
            grid_auto_flow: self.grid_auto_flow,
            grid_flow_tolerance: self.grid_flow_tolerance,
            scrollbar_gutter: self.scrollbar_gutter,
            scrollbar_width: self.scrollbar_width,
        }
    }

    pub(super) fn with_scroll_projections<R>(
        &self,
        consume: impl FnOnce(
            crate::scroll::ScrollBoxProjection<'_, S>,
            crate::scroll::ScrollTargetProjection<'_, S>,
        ) -> R,
    ) -> R {
        let source = NodeInputOf {
            item_is_table: self.item_is_table,
            item_is_replaced: self.item_is_replaced,
            box_sizing: self.box_sizing,
            writing_mode: self.writing_mode,
            direction: self.direction,
            overflow: self.overflow,
            overflow_clip_margin: self.overflow_clip_margin,
            scrollbar_gutter: self.scrollbar_gutter,
            scrollbar_width: self.scrollbar_width,
            scroll_padding: self.scroll_padding,
            scroll_margin: self.scroll_margin,
            scroll_snap_type: self.scroll_snap_type,
            scroll_snap_align: self.scroll_snap_align,
            scroll_snap_stop: self.scroll_snap_stop,
            position: self.position,
            inset: self.inset,
            size: self.size.clone(),
            min_size: self.min_size.clone(),
            max_size: self.max_size.clone(),
            aspect_ratio: self.aspect_ratio,
            margin: self.margin,
            padding: self.padding,
            border: self.border,
            ..NodeInputOf::default()
        };
        consume(
            crate::scroll::ScrollBoxProjection::from_node(&source),
            crate::scroll::ScrollTargetProjection::from_node(&source),
        )
    }
}

impl<S: LayoutScalar> GridOverflowFacts for GridItemProjection<S> {
    fn computed_overflow(&self) -> ComputedOverflow {
        self.overflow
    }

    fn item_is_replaced(&self) -> bool {
        self.item_is_replaced
    }
}

#[cfg(test)]
mod tests {
    use super::{GridContainerProjection, GridItemProjection};
    use crate::{
        AlignContent, AlignItems, AspectRatioOf, BoxSizing, ComputedOverflow, Direction, Display,
        Edges, FlowAxes, GridAutoFlow, GridFlowToleranceOf, GridPlacement, GridTemplateAreaRow,
        GridTemplateAreas, ItemOrder, LayoutScalar, LengthAutoOf, LengthOf, LengthPercentageOf,
        MaxSizeOf, MinSizeOf, NodeInputOf, Overflow, OverflowClipBox, OverflowClipMarginOf,
        PhysicalAxis, PhysicalSide, Position, PreferredSizeOf, RawGridLine, RawGridPlacement,
        ScrollMarginOf, ScrollPaddingOf, ScrollPaddingValueOf, ScrollSnapAlign,
        ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType,
        ScrollbarGutter, ScrollbarWidthOf, Size, TrackComponentOf, WritingMode,
    };

    fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    fn length_percentage<S: LayoutScalar>(value: f64) -> LengthPercentageOf<S> {
        LengthPercentageOf::px(scalar(value)).expect("finite test length")
    }

    fn length<S: LayoutScalar>(value: f64) -> LengthOf<S> {
        LengthOf::value(length_percentage(value))
    }

    fn length_auto<S: LayoutScalar>(value: f64) -> LengthAutoOf<S> {
        LengthAutoOf::value(length_percentage(value))
    }

    fn assert_non_default_projection_values<S: LayoutScalar>() {
        let expected_size = Size::new(
            PreferredSizeOf::value(length_percentage(11.0)),
            PreferredSizeOf::value(length_percentage(12.0)),
        );
        let expected_min_size = Size::new(
            MinSizeOf::value(length_percentage(13.0)),
            MinSizeOf::value(length_percentage(14.0)),
        );
        let expected_max_size = Size::new(
            MaxSizeOf::value(length_percentage(101.0)),
            MaxSizeOf::value(length_percentage(102.0)),
        );
        let expected_aspect_ratio =
            Some(AspectRatioOf::new(scalar(1.25)).expect("positive finite aspect ratio"));
        let expected_margin = Edges::new(
            length_auto(1.0),
            length_auto(2.0),
            length_auto(3.0),
            length_auto(4.0),
        );
        let expected_padding = Edges::new(length(5.0), length(6.0), length(7.0), length(8.0));
        let expected_border = Edges::new(length(9.0), length(10.0), length(15.0), length(16.0));
        let expected_inset = Edges::new(
            length_auto(21.0),
            length_auto(22.0),
            length_auto(23.0),
            length_auto(24.0),
        );
        let expected_overflow = ComputedOverflow::try_new(Overflow::Hidden, Overflow::Auto)
            .expect("canonical scrollable overflow pair");
        let expected_template_columns = vec![
            TrackComponentOf::line_names(["hero-start"]),
            TrackComponentOf::MAX_CONTENT,
        ];
        let expected_template_rows = vec![TrackComponentOf::MIN_CONTENT];
        let expected_template_areas = GridTemplateAreas {
            rows: vec![GridTemplateAreaRow {
                cells: vec![Some(String::from("hero")), None],
            }],
        };
        let expected_auto_columns = vec![TrackComponentOf::AUTO];
        let expected_auto_rows = vec![
            TrackComponentOf::line_names(["implicit-row"]),
            TrackComponentOf::AUTO,
        ];
        let expected_gap = Size::new(length(31.0), length(32.0));
        let expected_tolerance = GridFlowToleranceOf::Percent(scalar(0.125));
        let expected_scrollbar_width =
            ScrollbarWidthOf::try_new(scalar(7.5)).expect("finite scrollbar width");

        let input = NodeInputOf::<S> {
            display: Display::InlineGridLanes,
            size: expected_size.clone(),
            min_size: expected_min_size.clone(),
            max_size: expected_max_size.clone(),
            aspect_ratio: expected_aspect_ratio,
            margin: expected_margin,
            padding: expected_padding,
            border: expected_border,
            box_sizing: BoxSizing::ContentBox,
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Rtl,
            overflow: expected_overflow,
            position: Position::Absolute,
            inset: expected_inset,
            item_is_replaced: true,
            item_is_table: true,
            align_items: Some(AlignItems::LastBaseline),
            justify_items: Some(AlignItems::SafeCenter),
            align_content: Some(AlignContent::SpaceEvenly),
            justify_content: Some(AlignContent::SafeFlexEnd),
            gap: expected_gap,
            grid_template_columns: expected_template_columns.clone(),
            grid_template_rows: expected_template_rows.clone(),
            grid_template_areas: expected_template_areas.clone(),
            grid_auto_columns: expected_auto_columns.clone(),
            grid_auto_rows: expected_auto_rows.clone(),
            grid_auto_flow: GridAutoFlow::ColumnDense,
            grid_flow_tolerance: expected_tolerance,
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: expected_scrollbar_width,
            ..NodeInputOf::default()
        };

        let projection = GridContainerProjection::from_node(&input);

        assert_eq!(*projection.common.size, expected_size);
        assert_eq!(*projection.common.min_size, expected_min_size);
        assert_eq!(*projection.common.max_size, expected_max_size);
        assert_eq!(*projection.common.aspect_ratio, expected_aspect_ratio);
        assert_eq!(*projection.common.margin, expected_margin);
        assert_eq!(*projection.common.padding, expected_padding);
        assert_eq!(*projection.common.border, expected_border);
        assert_eq!(projection.common.box_sizing, BoxSizing::ContentBox);
        assert_eq!(
            projection.common.flow_axes,
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl)
        );
        assert_eq!(
            projection.common.flow_axes.writing_mode(),
            WritingMode::SidewaysLr
        );
        assert_eq!(projection.common.flow_axes.direction(), Direction::Rtl);
        assert_eq!(
            projection.common.flow_axes.inline_axis(),
            PhysicalAxis::Vertical
        );
        assert_eq!(
            projection.common.flow_axes.block_axis(),
            PhysicalAxis::Horizontal
        );
        assert_eq!(
            projection.common.flow_axes.inline_start(),
            PhysicalSide::Top
        );
        assert_eq!(
            projection.common.flow_axes.inline_end(),
            PhysicalSide::Bottom
        );
        assert_eq!(
            projection.common.flow_axes.block_start(),
            PhysicalSide::Left
        );
        assert_eq!(projection.common.flow_axes.block_end(), PhysicalSide::Right);
        assert_eq!(projection.common.flow_axes.line_over(), PhysicalSide::Left);
        assert_eq!(
            projection.common.flow_axes.line_under(),
            PhysicalSide::Right
        );
        assert_eq!(projection.common.overflow, expected_overflow);
        assert_eq!(projection.common.position, Position::Absolute);
        assert_eq!(*projection.common.inset, expected_inset);
        assert!(projection.common.item_is_replaced);
        assert!(projection.common.item_is_table);

        assert!(core::ptr::eq(projection.common.size, &input.size));
        assert!(core::ptr::eq(projection.common.min_size, &input.min_size));
        assert!(core::ptr::eq(projection.common.max_size, &input.max_size));
        assert!(core::ptr::eq(
            projection.common.aspect_ratio,
            &input.aspect_ratio
        ));
        assert!(core::ptr::eq(projection.common.margin, &input.margin));
        assert!(core::ptr::eq(projection.common.padding, &input.padding));
        assert!(core::ptr::eq(projection.common.border, &input.border));
        assert!(core::ptr::eq(projection.common.inset, &input.inset));

        assert_eq!(projection.display, Display::InlineGridLanes);
        assert_eq!(projection.align_items, Some(AlignItems::LastBaseline));
        assert_eq!(projection.justify_items, Some(AlignItems::SafeCenter));
        assert_eq!(projection.align_content, Some(AlignContent::SpaceEvenly));
        assert_eq!(projection.justify_content, Some(AlignContent::SafeFlexEnd));
        assert_eq!(*projection.gap, expected_gap);
        assert_eq!(projection.grid_template_columns, expected_template_columns);
        assert_eq!(projection.grid_template_rows, expected_template_rows);
        assert_eq!(*projection.grid_template_areas, expected_template_areas);
        assert_eq!(projection.grid_auto_columns, expected_auto_columns);
        assert_eq!(projection.grid_auto_rows, expected_auto_rows);
        assert_eq!(projection.grid_auto_flow, GridAutoFlow::ColumnDense);
        assert_eq!(projection.grid_flow_tolerance, expected_tolerance);
        assert_eq!(
            projection.scrollbar_gutter,
            ScrollbarGutter::StableBothEdges
        );
        assert_eq!(projection.scrollbar_width, expected_scrollbar_width);

        assert!(core::ptr::eq(projection.gap, &input.gap));
        assert!(core::ptr::eq(
            projection.grid_template_columns,
            input.grid_template_columns.as_slice()
        ));
        assert!(core::ptr::eq(
            projection.grid_template_rows,
            input.grid_template_rows.as_slice()
        ));
        assert!(core::ptr::eq(
            projection.grid_template_areas,
            &input.grid_template_areas
        ));
        assert!(core::ptr::eq(
            projection.grid_auto_columns,
            input.grid_auto_columns.as_slice()
        ));
        assert!(core::ptr::eq(
            projection.grid_auto_rows,
            input.grid_auto_rows.as_slice()
        ));
    }

    #[test]
    fn node_projection_grid_container_f32_selects_exact_non_default_values() {
        assert_non_default_projection_values::<f32>();
    }

    #[test]
    fn node_projection_grid_container_f64_selects_exact_non_default_values() {
        assert_non_default_projection_values::<f64>();
    }

    fn assert_non_default_item_projection_values<S: LayoutScalar>() {
        let expected_size = Size::new(
            PreferredSizeOf::value(length_percentage(41.0)),
            PreferredSizeOf::value(length_percentage(42.0)),
        );
        let expected_min_size = Size::new(
            MinSizeOf::value(length_percentage(43.0)),
            MinSizeOf::value(length_percentage(44.0)),
        );
        let expected_max_size = Size::new(
            MaxSizeOf::value(length_percentage(141.0)),
            MaxSizeOf::value(length_percentage(142.0)),
        );
        let expected_margin = Edges::new(
            length_auto(45.0),
            length_auto(46.0),
            length_auto(47.0),
            length_auto(48.0),
        );
        let expected_padding = Edges::new(length(51.0), length(52.0), length(53.0), length(54.0));
        let expected_border = Edges::new(length(55.0), length(56.0), length(57.0), length(58.0));
        let expected_inset = Edges::new(
            length_auto(61.0),
            length_auto(62.0),
            length_auto(63.0),
            length_auto(64.0),
        );
        let expected_aspect_ratio =
            Some(AspectRatioOf::new(scalar(1.75)).expect("positive finite aspect ratio"));
        let expected_gap = Size::new(length(65.0), length(66.0));
        let expected_template_columns = vec![TrackComponentOf::MIN_CONTENT];
        let expected_template_rows = vec![TrackComponentOf::MAX_CONTENT];
        let expected_auto_columns = vec![TrackComponentOf::AUTO];
        let expected_auto_rows = vec![TrackComponentOf::MIN_CONTENT];
        let expected_template_areas = GridTemplateAreas {
            rows: vec![GridTemplateAreaRow {
                cells: vec![Some(String::from("feature"))],
            }],
        };
        let expected_tolerance = GridFlowToleranceOf::Percent(scalar(0.25));
        let expected_overflow_clip_margin =
            OverflowClipMarginOf::try_new(OverflowClipBox::BorderBox, scalar(6.5))
                .expect("finite overflow clip margin");
        let expected_scroll_padding = ScrollPaddingOf::new(
            ScrollPaddingValueOf::value(length_percentage(0.11)),
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::value(length_percentage(0.13)),
            ScrollPaddingValueOf::value(length_percentage(0.14)),
        );
        let expected_scroll_margin =
            ScrollMarginOf::try_new(scalar(71.0), scalar(72.0), scalar(73.0), scalar(74.0))
                .expect("finite scroll margin");
        let expected_snap_type = ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Inline,
            strictness: ScrollSnapStrictness::Mandatory,
        };
        let expected_snap_align =
            ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
        let expected_scrollbar_width =
            ScrollbarWidthOf::try_new(scalar(8.0)).expect("finite scrollbar width");
        let expected_column = GridPlacement::try_lines(2, 5).expect("valid grid lines");
        let expected_row = GridPlacement::try_line_span(-3, 2).expect("valid line and span");
        let expected_raw_column = RawGridPlacement::new(
            RawGridLine::NamedLine {
                name: String::from("hero"),
                index: 2,
            },
            RawGridLine::NamedSpan {
                name: String::from("hero"),
                index: 3,
            },
        );
        let expected_raw_row = RawGridPlacement::lines(-4, -1);
        let expected_overflow = ComputedOverflow::try_new(Overflow::Hidden, Overflow::Scroll)
            .expect("canonical non-default overflow pair");

        let input = NodeInputOf::<S> {
            display: Display::InlineGrid,
            item_is_replaced: true,
            item_is_table: true,
            item_order: ItemOrder::new(-17),
            box_sizing: BoxSizing::ContentBox,
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            overflow: expected_overflow,
            overflow_clip_margin: expected_overflow_clip_margin,
            position: Position::Absolute,
            size: expected_size.clone(),
            min_size: expected_min_size.clone(),
            max_size: expected_max_size.clone(),
            aspect_ratio: expected_aspect_ratio,
            margin: expected_margin,
            padding: expected_padding,
            border: expected_border,
            inset: expected_inset,
            align_self: Some(AlignItems::LastBaseline),
            justify_self: Some(AlignItems::SafeCenter),
            align_items: Some(AlignItems::Baseline),
            justify_items: Some(AlignItems::SafeFlexEnd),
            align_content: Some(AlignContent::SpaceAround),
            justify_content: Some(AlignContent::SpaceBetween),
            gap: expected_gap,
            grid_template_columns: expected_template_columns.clone(),
            grid_template_rows: expected_template_rows.clone(),
            grid_template_areas: expected_template_areas.clone(),
            grid_auto_columns: expected_auto_columns.clone(),
            grid_auto_rows: expected_auto_rows.clone(),
            grid_auto_flow: GridAutoFlow::ColumnDense,
            grid_flow_tolerance: expected_tolerance,
            grid_column: expected_column,
            grid_row: expected_row,
            raw_grid_column: expected_raw_column.clone(),
            raw_grid_row: expected_raw_row.clone(),
            scroll_padding: expected_scroll_padding,
            scroll_margin: expected_scroll_margin,
            scroll_snap_type: expected_snap_type,
            scroll_snap_align: expected_snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            scrollbar_gutter: ScrollbarGutter::Stable,
            scrollbar_width: expected_scrollbar_width,
            ..NodeInputOf::default()
        };

        let projection = GridItemProjection::from_node(&input);

        assert_eq!(projection.display, Display::InlineGrid);
        assert!(projection.item_is_replaced);
        assert!(projection.item_is_table);
        assert_eq!(projection.item_order, ItemOrder::new(-17));
        assert_eq!(projection.box_sizing, BoxSizing::ContentBox);
        assert_eq!(
            projection.flow_axes,
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl)
        );
        assert_eq!(projection.overflow, expected_overflow);
        assert_eq!(
            projection.overflow_clip_margin,
            expected_overflow_clip_margin
        );
        assert_eq!(projection.position, Position::Absolute);
        assert_eq!(projection.size, expected_size);
        assert_eq!(projection.min_size, expected_min_size);
        assert_eq!(projection.max_size, expected_max_size);
        assert_eq!(projection.aspect_ratio, expected_aspect_ratio);
        assert_eq!(projection.margin, expected_margin);
        assert_eq!(projection.padding, expected_padding);
        assert_eq!(projection.border, expected_border);
        assert_eq!(projection.inset, expected_inset);
        assert_eq!(projection.align_self, Some(AlignItems::LastBaseline));
        assert_eq!(projection.justify_self, Some(AlignItems::SafeCenter));
        assert_eq!(projection.align_items, Some(AlignItems::Baseline));
        assert_eq!(projection.justify_items, Some(AlignItems::SafeFlexEnd));
        assert_eq!(projection.align_content, Some(AlignContent::SpaceAround));
        assert_eq!(projection.justify_content, Some(AlignContent::SpaceBetween));
        assert_eq!(projection.gap, expected_gap);
        assert_eq!(projection.grid_template_columns, expected_template_columns);
        assert_eq!(projection.grid_template_rows, expected_template_rows);
        assert_eq!(projection.grid_template_areas, expected_template_areas);
        assert_eq!(projection.grid_auto_columns, expected_auto_columns);
        assert_eq!(projection.grid_auto_rows, expected_auto_rows);
        assert_eq!(projection.grid_auto_flow, GridAutoFlow::ColumnDense);
        assert_eq!(projection.grid_flow_tolerance, expected_tolerance);
        assert_eq!(projection.grid_column, expected_column);
        assert_eq!(projection.grid_row, expected_row);
        assert_eq!(projection.raw_grid_column, expected_raw_column);
        assert_eq!(projection.raw_grid_row, expected_raw_row);
        assert_eq!(projection.scroll_padding, expected_scroll_padding);
        assert_eq!(projection.scroll_margin, expected_scroll_margin);
        assert_eq!(projection.scroll_snap_type, expected_snap_type);
        assert_eq!(projection.scroll_snap_align, expected_snap_align);
        assert_eq!(projection.scroll_snap_stop, ScrollSnapStop::Always);
        assert_eq!(projection.scrollbar_gutter, ScrollbarGutter::Stable);
        assert_eq!(projection.scrollbar_width, expected_scrollbar_width);
    }

    #[test]
    fn node_projection_grid_item_f32_selects_exact_non_default_values() {
        assert_non_default_item_projection_values::<f32>();
    }

    #[test]
    fn node_projection_grid_item_f64_selects_exact_non_default_values() {
        assert_non_default_item_projection_values::<f64>();
    }
}
