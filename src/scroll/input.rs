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
