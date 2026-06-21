use super::{
    Available, BoxSizing, CacheAccess, Compute, ComputeInput, ComputeOutput, NodeInput, NodeOutput,
    Position, Round, RunMode, Scalar, Size, SizingMode, Traverse,
};

pub fn compute_hidden<Tree>(tree: &mut Tree, node: <Tree as Traverse>::Node) -> ComputeOutput
where
    Tree: Compute + CacheAccess<Node = <Tree as super::Traverse>::Node>,
{
    tree.cache_clear(node);
    tree.set_unrounded(node, NodeOutput::with_order(0));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        tree.compute_child(child, ComputeInput::HIDDEN);
    }

    ComputeOutput::HIDDEN
}

pub fn compute_root<Tree>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<Available>,
) where
    Tree: Compute,
{
    let style = tree.node_input(root).clone();
    let known = Size::new(root_known_width(&style, available.width), None);
    let output = tree.compute_child(
        root,
        ComputeInput {
            run_mode: RunMode::PerformRootLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: super::RequestedAxis::Both,
            known,
            parent: available.map(Available::into_option),
            available,
        },
    );
    let parent_width = available.width.into_option();
    let inline_basis = Size::splat(parent_width);
    let scrollbar_size = Size::new(
        if style.overflow.y == super::Overflow::Scroll {
            style.scrollbar_width
        } else {
            0.0
        },
        if style.overflow.x == super::Overflow::Scroll {
            style.scrollbar_width
        } else {
            0.0
        },
    );
    let location = super::Point::new(
        if style.direction.is_rtl() {
            parent_width.map_or(0.0, |width| width - output.size.width)
        } else {
            0.0
        },
        0.0,
    );

    tree.set_unrounded(
        root,
        NodeOutput {
            order: 0,
            location,
            size: output.size,
            content_size: output.content_size,
            scrollbar_size,
            padding: style.padding.zip_size(inline_basis, resolve_length_or_zero),
            border: style.border.zip_size(inline_basis, resolve_length_or_zero),
            margin: style.margin.zip_size(inline_basis, resolve_auto_or_zero),
        },
    );
}

fn root_known_width(style: &NodeInput, available_width: Available) -> Option<Scalar> {
    if style.display.is_inline_level()
        || !style.size.width.is_auto()
        || !style.min_size.width.is_auto()
    {
        return None;
    }

    let available_width = available_width.into_option()?;
    let parent = Size::splat(Some(available_width));
    let padding = style
        .padding
        .zip_inline_size(parent, resolve_length_or_zero);
    let border = style.border.zip_inline_size(parent, resolve_length_or_zero);
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = style
        .max_size
        .zip_map(parent, resolve_dimension)
        .add_optional(box_sizing_adjustment);

    Some(available_width.clamp_optional(None, max_size.width))
}

pub fn round_layout<Tree>(tree: &mut Tree, root: <Tree as Traverse>::Node)
where
    Tree: Round,
{
    round_layout_inner(tree, root, 0.0, 0.0);
}

fn round_layout_inner<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    cumulative_x: Scalar,
    cumulative_y: Scalar,
) where
    Tree: Round,
{
    let unrounded = tree.unrounded(node);
    let mut layout = unrounded;
    let cumulative_x = cumulative_x + unrounded.location.x;
    let cumulative_y = cumulative_y + unrounded.location.y;

    layout.location.x = round(unrounded.location.x);
    layout.location.y = round(unrounded.location.y);
    layout.size.width = round(cumulative_x + unrounded.size.width) - round(cumulative_x);
    layout.size.height = round(cumulative_y + unrounded.size.height) - round(cumulative_y);
    layout.content_size.width =
        round(cumulative_x + unrounded.content_size.width) - round(cumulative_x);
    layout.content_size.height =
        round(cumulative_y + unrounded.content_size.height) - round(cumulative_y);
    layout.scrollbar_size.width = round(unrounded.scrollbar_size.width);
    layout.scrollbar_size.height = round(unrounded.scrollbar_size.height);
    layout.border.left = round(cumulative_x + unrounded.border.left) - round(cumulative_x);
    layout.border.right = round(cumulative_x + unrounded.size.width)
        - round(cumulative_x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round(cumulative_y + unrounded.border.top) - round(cumulative_y);
    layout.border.bottom = round(cumulative_y + unrounded.size.height)
        - round(cumulative_y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round(cumulative_x + unrounded.padding.left) - round(cumulative_x);
    layout.padding.right = round(cumulative_x + unrounded.size.width)
        - round(cumulative_x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round(cumulative_y + unrounded.padding.top) - round(cumulative_y);
    layout.padding.bottom = round(cumulative_y + unrounded.size.height)
        - round(cumulative_y + unrounded.size.height - unrounded.padding.bottom);

    tree.set_final(node, layout);

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        round_layout_inner(tree, child, cumulative_x, cumulative_y);
    }
}

#[inline]
fn round(value: Scalar) -> Scalar {
    (value + 0.5).floor()
}

pub fn compute_leaf(
    input: ComputeInput,
    style: &NodeInput,
    measure: impl FnOnce(Size<Option<Scalar>>, Size<Available>) -> Size,
) -> ComputeOutput {
    let margin = style
        .margin
        .zip_inline_size(input.parent, resolve_auto_or_zero);
    let padding = style
        .padding
        .zip_inline_size(input.parent, resolve_length_or_zero);
    let border = style
        .border
        .zip_inline_size(input.parent, resolve_length_or_zero);
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let scrollbar_gutter = Size::new(
        if style.overflow.y == super::Overflow::Scroll {
            style.scrollbar_width
        } else {
            0.0
        },
        if style.overflow.x == super::Overflow::Scroll {
            style.scrollbar_width
        } else {
            0.0
        },
    );
    let mut content_box_inset = padding_border;
    content_box_inset.right += scrollbar_gutter.width;
    content_box_inset.bottom += scrollbar_gutter.height;
    let content_box_inset_size = content_box_inset.sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };

    let (node_size, node_min_size, node_max_size, aspect_ratio) = match input.sizing_mode {
        SizingMode::ContentSize => (input.known, Size::NONE, Size::NONE, None),
        SizingMode::InherentSize => {
            let style_size = style
                .size
                .zip_map(input.parent, resolve_dimension)
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
            let style_min_size = style
                .min_size
                .zip_map(input.parent, resolve_dimension)
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
            let style_max_size = style
                .max_size
                .zip_map(input.parent, resolve_dimension)
                .add_optional(box_sizing_adjustment);

            (
                input.known.or(style_size),
                style_min_size,
                style_max_size,
                style.aspect_ratio,
            )
        }
    };

    let prevents_margin_collapse = style.display != super::Display::Block
        || style.overflow.x.blocks_margin_collapse()
        || style.overflow.y.blocks_margin_collapse()
        || style.position == Position::Absolute
        || padding.top > 0.0
        || padding.bottom > 0.0
        || border.top > 0.0
        || border.bottom > 0.0
        || matches!(node_size.height, Some(height) if height > 0.0)
        || matches!(node_min_size.height, Some(height) if height > 0.0);

    if input.run_mode == RunMode::ComputeSize
        && prevents_margin_collapse
        && let Size {
            width: Some(width),
            height: Some(height),
        } = node_size
    {
        let size = Size::new(width, height)
            .clamp_optional(node_min_size, node_max_size)
            .max_optional(padding_border_size.map(Some));
        return ComputeOutput::from_outer_size(size);
    }

    let available = Size::new(
        input
            .known
            .width
            .map(Available::definite)
            .unwrap_or(input.available.width)
            .sub_margin(margin.horizontal_sum())
            .set_optional(input.known.width)
            .set_optional(node_size.width)
            .map_definite(|value| {
                value.clamp_optional(node_min_size.width, node_max_size.width)
                    - content_box_inset.horizontal_sum()
            }),
        input
            .known
            .height
            .map(Available::definite)
            .unwrap_or(input.available.height)
            .sub_margin(margin.vertical_sum())
            .set_optional(input.known.height)
            .set_optional(node_size.height)
            .map_definite(|value| {
                value.clamp_optional(node_min_size.height, node_max_size.height)
                    - content_box_inset.vertical_sum()
            }),
    );

    let measured = measure(
        match input.run_mode {
            RunMode::ComputeSize => input.known,
            RunMode::PerformRootLayout | RunMode::PerformLayout => Size::NONE,
            RunMode::PerformHiddenLayout => {
                unreachable!("hidden layout uses ComputeOutput::HIDDEN")
            }
        },
        available,
    );
    let unclamped = input
        .known
        .or(node_size)
        .unwrap_or(measured + content_box_inset_size);
    let height_is_definite = input.known.height.is_some() || node_size.height.is_some();
    let aspect_height = if height_is_definite {
        unclamped.height
    } else {
        unclamped.height.max(
            aspect_ratio
                .map(|ratio| unclamped.width / ratio)
                .unwrap_or(0.0),
        )
    };
    let aspect_size = Size::new(unclamped.width, aspect_height)
        .clamp_optional(node_min_size, node_max_size)
        .max_optional(padding_border_size.map(Some));

    let mut output = ComputeOutput::from_sizes(aspect_size, measured + padding.sum_axes());
    output.margins_can_collapse_through =
        !prevents_margin_collapse && aspect_size.height == 0.0 && measured.height == 0.0;
    output
}

fn resolve_length_or_zero(length: super::Length, basis: Option<Scalar>) -> Scalar {
    match length {
        super::Length::Normal => 0.0,
        super::Length::Px(value) => value,
        super::Length::Percent(value) => basis.map_or(0.0, |basis| value * basis),
    }
}

fn resolve_auto_or_zero(length: super::LengthAuto, basis: Option<Scalar>) -> Scalar {
    match length {
        super::LengthAuto::Px(value) => value,
        super::LengthAuto::Percent(value) => basis.map_or(0.0, |basis| value * basis),
        super::LengthAuto::Auto => 0.0,
    }
}

fn resolve_dimension(dimension: super::Dimension, basis: Option<Scalar>) -> Option<Scalar> {
    match dimension {
        super::Dimension::Px(value) => Some(value),
        super::Dimension::Percent(value) => basis.map(|basis| value * basis),
        super::Dimension::Fr(_)
        | super::Dimension::Auto
        | super::Dimension::MinContent
        | super::Dimension::MaxContent => None,
    }
}

trait SizeOptionExt {
    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size) -> Size;
    fn add_optional(self, amount: Size) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<Scalar>) -> Self;
}

impl SizeOptionExt for Size<Option<Scalar>> {
    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size) -> Size {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size) -> Self {
        Size::new(
            self.width.map(|width| width + amount.width),
            self.height.map(|height| height + amount.height),
        )
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<Scalar>) -> Self {
        let Some(ratio) = aspect_ratio else {
            return self;
        };
        match (self.width, self.height) {
            (Some(width), None) => Size::new(Some(width), Some(width / ratio)),
            (None, Some(height)) => Size::new(Some(height * ratio), Some(height)),
            _ => self,
        }
    }
}

trait SizeExt {
    fn clamp_optional(self, min: Size<Option<Scalar>>, max: Size<Option<Scalar>>) -> Self;
    fn max_optional(self, min: Size<Option<Scalar>>) -> Self;
}

impl SizeExt for Size {
    fn clamp_optional(self, min: Size<Option<Scalar>>, max: Size<Option<Scalar>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }

    fn max_optional(self, min: Size<Option<Scalar>>) -> Self {
        Size::new(
            min.width.map_or(self.width, |min| self.width.max(min)),
            min.height.map_or(self.height, |min| self.height.max(min)),
        )
    }
}

trait ScalarExt {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl ScalarExt for Scalar {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}

trait AvailableExt {
    fn sub_margin(self, margin: Scalar) -> Self;
    fn set_optional(self, value: Option<Scalar>) -> Self;
    fn map_definite(self, f: impl FnOnce(Scalar) -> Scalar) -> Self;
}

impl AvailableExt for Available {
    fn sub_margin(self, margin: Scalar) -> Self {
        self.map_definite(|value| value - margin)
    }

    fn set_optional(self, value: Option<Scalar>) -> Self {
        value.map_or(self, Available::definite)
    }

    fn map_definite(self, f: impl FnOnce(Scalar) -> Scalar) -> Self {
        match self {
            Available::Definite(value) => Available::Definite(f(value)),
            Available::MinContent => Available::MinContent,
            Available::MaxContent => Available::MaxContent,
        }
    }
}
