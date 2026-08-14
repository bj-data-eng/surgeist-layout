use crate::node_projection::CommonBoxProjection;
use crate::{
    AlignContent, AlignItems, ComputedOverflow, Display, GridAutoFlow, GridFlowToleranceOf,
    GridTemplateAreas, LayoutScalar, LengthOf, NodeInputOf, ScrollbarGutter, ScrollbarWidthOf,
    Size, TrackComponentOf,
};

pub(super) trait GridOverflowFacts {
    fn computed_overflow(&self) -> ComputedOverflow;

    fn item_is_replaced(&self) -> bool;
}

impl<S: LayoutScalar> GridOverflowFacts for NodeInputOf<S> {
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

#[cfg(test)]
mod tests {
    use super::GridContainerProjection;
    use crate::{
        AlignContent, AlignItems, AspectRatioOf, BoxSizing, ComputedOverflow, Direction, Display,
        Edges, FlowAxes, GridAutoFlow, GridFlowToleranceOf, GridTemplateAreaRow, GridTemplateAreas,
        LayoutScalar, LengthAutoOf, LengthOf, LengthPercentageOf, MaxSizeOf, MinSizeOf,
        NodeInputOf, Overflow, PhysicalAxis, PhysicalSide, Position, PreferredSizeOf,
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
}
