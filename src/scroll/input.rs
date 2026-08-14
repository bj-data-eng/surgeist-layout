use crate::{
    LayoutScalar, NodeInputOf, OverflowClipMarginOf, ScrollMarginOf, ScrollPaddingOf,
    ScrollSnapAlign, ScrollSnapStop, ScrollSnapType, ScrollbarGutter, ScrollbarWidthOf,
    node_projection::CommonBoxProjection,
};

/// Borrowed scroll-container facts for canonical box and range construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollBoxProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) overflow_clip_margin: &'a OverflowClipMarginOf<S>,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
    pub(super) scroll_padding: &'a ScrollPaddingOf<S>,
    pub(super) scroll_snap_type: ScrollSnapType,
}

impl<'a, S: LayoutScalar> ScrollBoxProjection<'a, S> {
    #[must_use]
    pub(crate) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            overflow_clip_margin: &input.overflow_clip_margin,
            scrollbar_gutter: input.scrollbar_gutter,
            scrollbar_width: input.scrollbar_width,
            scroll_padding: &input.scroll_padding,
            scroll_snap_type: input.scroll_snap_type,
        }
    }
}

/// Borrowed scroll-target facts retained with canonical target geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollTargetProjection<'a, S: LayoutScalar> {
    pub(super) flow_axes: crate::FlowAxes,
    pub(super) scroll_margin: &'a ScrollMarginOf<S>,
    pub(super) snap_align: ScrollSnapAlign,
    pub(super) snap_stop: ScrollSnapStop,
}

impl<'a, S: LayoutScalar> ScrollTargetProjection<'a, S> {
    #[must_use]
    pub(crate) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            flow_axes: crate::FlowAxes::new(input.writing_mode, input.direction),
            scroll_margin: &input.scroll_margin,
            snap_align: input.scroll_snap_align,
            snap_stop: input.scroll_snap_stop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AspectRatioOf, BoxSizing, ComputedOverflow, Direction, Edges, FlowAxes, LengthAutoOf,
        LengthOf, LengthPercentageOf, MaxSizeOf, MinSizeOf, Overflow, OverflowClipBox,
        PhysicalAxis, PhysicalSide, Position, PreferredSizeOf, ScrollPaddingValueOf,
        ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStrictness, Size, WritingMode,
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

    fn assert_sideways_lr_rtl_mapping(axes: FlowAxes) {
        assert_eq!(axes.writing_mode(), WritingMode::SidewaysLr);
        assert_eq!(axes.direction(), Direction::Rtl);
        assert_eq!(axes.inline_axis(), PhysicalAxis::Vertical);
        assert_eq!(axes.block_axis(), PhysicalAxis::Horizontal);
        assert_eq!(axes.inline_start(), PhysicalSide::Top);
        assert_eq!(axes.inline_end(), PhysicalSide::Bottom);
        assert_eq!(axes.block_start(), PhysicalSide::Left);
        assert_eq!(axes.block_end(), PhysicalSide::Right);
        assert_eq!(axes.line_over(), PhysicalSide::Left);
        assert_eq!(axes.line_under(), PhysicalSide::Right);
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
        let expected_overflow_clip_margin =
            OverflowClipMarginOf::try_new(OverflowClipBox::BorderBox, scalar(31.0))
                .expect("finite non-negative clip margin");
        let expected_scrollbar_width =
            ScrollbarWidthOf::try_new(scalar(32.0)).expect("finite non-negative scrollbar width");
        let expected_scroll_padding = ScrollPaddingOf::new(
            ScrollPaddingValueOf::value(length_percentage(33.0)),
            ScrollPaddingValueOf::value(length_percentage(34.0)),
            ScrollPaddingValueOf::value(length_percentage(35.0)),
            ScrollPaddingValueOf::value(length_percentage(36.0)),
        );
        let expected_scroll_snap_type = ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Inline,
            strictness: ScrollSnapStrictness::Mandatory,
        };
        let expected_scroll_margin =
            ScrollMarginOf::try_new(scalar(-41.0), scalar(42.0), scalar(-43.0), scalar(44.0))
                .expect("finite signed scroll margins");
        let expected_snap_align =
            ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);

        let input = NodeInputOf::<S> {
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
            overflow_clip_margin: expected_overflow_clip_margin,
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: expected_scrollbar_width,
            scroll_padding: expected_scroll_padding,
            scroll_snap_type: expected_scroll_snap_type,
            scroll_margin: expected_scroll_margin,
            scroll_snap_align: expected_snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInputOf::default()
        };

        let common = CommonBoxProjection::from_node(&input);
        let scroll_box = ScrollBoxProjection::from_node(&input);
        let scroll_target = ScrollTargetProjection::from_node(&input);

        assert_eq!(*common.size, expected_size);
        assert_eq!(*common.min_size, expected_min_size);
        assert_eq!(*common.max_size, expected_max_size);
        assert_eq!(*common.aspect_ratio, expected_aspect_ratio);
        assert_eq!(*common.margin, expected_margin);
        assert_eq!(*common.padding, expected_padding);
        assert_eq!(*common.border, expected_border);
        assert_eq!(common.box_sizing, BoxSizing::ContentBox);
        assert_eq!(common.overflow, expected_overflow);
        assert_eq!(common.position, Position::Absolute);
        assert_eq!(*common.inset, expected_inset);
        assert!(common.item_is_replaced);
        assert!(common.item_is_table);
        assert_sideways_lr_rtl_mapping(common.flow_axes);

        assert!(core::ptr::eq(common.size, &input.size));
        assert!(core::ptr::eq(common.min_size, &input.min_size));
        assert!(core::ptr::eq(common.max_size, &input.max_size));
        assert!(core::ptr::eq(common.aspect_ratio, &input.aspect_ratio));
        assert!(core::ptr::eq(common.margin, &input.margin));
        assert!(core::ptr::eq(common.padding, &input.padding));
        assert!(core::ptr::eq(common.border, &input.border));
        assert!(core::ptr::eq(common.inset, &input.inset));

        assert_eq!(scroll_box.common, common);
        assert!(core::ptr::eq(scroll_box.common.size, common.size));
        assert_eq!(
            *scroll_box.overflow_clip_margin,
            expected_overflow_clip_margin
        );
        assert_eq!(
            scroll_box.scrollbar_gutter,
            ScrollbarGutter::StableBothEdges
        );
        assert_eq!(scroll_box.scrollbar_width, expected_scrollbar_width);
        assert_eq!(*scroll_box.scroll_padding, expected_scroll_padding);
        assert_eq!(scroll_box.scroll_snap_type, expected_scroll_snap_type);
        assert!(core::ptr::eq(
            scroll_box.overflow_clip_margin,
            &input.overflow_clip_margin
        ));
        assert!(core::ptr::eq(
            scroll_box.scroll_padding,
            &input.scroll_padding
        ));

        assert_eq!(*scroll_target.scroll_margin, expected_scroll_margin);
        assert_eq!(scroll_target.snap_align, expected_snap_align);
        assert_eq!(scroll_target.snap_stop, ScrollSnapStop::Always);
        assert_sideways_lr_rtl_mapping(scroll_target.flow_axes);
        assert_eq!(scroll_target.flow_axes, common.flow_axes);
        assert!(core::ptr::eq(
            scroll_target.scroll_margin,
            &input.scroll_margin
        ));
    }

    #[test]
    fn node_projection_common_scroll_f32_selects_exact_non_default_values() {
        assert_non_default_projection_values::<f32>();
    }

    #[test]
    fn node_projection_common_scroll_f64_selects_exact_non_default_values() {
        assert_non_default_projection_values::<f64>();
    }
}
