use super::{
    AspectRatioOf, AvailableOf, BoxSizing, CacheAccess, CalcResolutionOf, CalcResolutionStatus,
    CalcResolver, Compute, ComputeInputOf, ComputeOutputOf, Direction, LayoutInputOf, LayoutScalar,
    NoCalcResolver, NodeInputOf, NodeOutputOf, Position, Round, RunMode, Size, SizingMode,
    Traverse,
};
use crate::scroll::{
    ScrollbarReservationOf, content_box_inset_with_scrollbar, scrollbar_size_from_overflow,
};

pub fn compute_hidden<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
) -> ComputeOutputOf<<Tree as Traverse>::Scalar>
where
    Tree:
        Compute + CacheAccess<Node = <Tree as Traverse>::Node, Scalar = <Tree as Traverse>::Scalar>,
{
    tree.cache_clear(node);
    tree.set_unrounded(node, NodeOutputOf::with_order(0));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        match tree.layout_input(child) {
            LayoutInputOf::Box(_) => {
                tree.compute_child(child, ComputeInputOf::HIDDEN);
            }
            LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => {
                tree.cache_clear(child);
                tree.set_unrounded(child, NodeOutputOf::with_order(0));
            }
        }
    }

    ComputeOutputOf::HIDDEN
}

pub fn compute_root<Tree>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    available: Size<AvailableOf<Tree::Scalar>>,
) where
    Tree: Compute,
{
    let style = tree.node_input(root).clone();
    let known = Size::new(
        root_known_width(&style, available.width, tree.calc_resolver()),
        None,
    );
    let output = tree.compute_child(
        root,
        ComputeInputOf {
            run_mode: RunMode::PerformRootLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: super::RequestedAxis::Both,
            known,
            parent: available.map(AvailableOf::into_option),
            available,
        },
    );
    let parent_width = available.width.into_option();
    let inline_basis = Size::splat(parent_width);
    let padding = style.padding.zip_size(inline_basis, |length, basis| {
        resolve_length_or_zero_with(length, basis, tree.calc_resolver())
    });
    let border = style.border.zip_size(inline_basis, |length, basis| {
        resolve_length_or_zero_with(length, basis, tree.calc_resolver())
    });
    let margin = style.margin.zip_size(inline_basis, |length, basis| {
        resolve_auto_or_zero_with(length, basis, tree.calc_resolver())
    });
    let scrollbar_size = scrollbar_size_from_overflow(style.overflow, style.scrollbar_width);
    let location = super::Point::new(
        if style.direction.is_rtl() {
            parent_width.map_or(<Tree as Traverse>::Scalar::ZERO, |width| {
                width - output.size.width
            })
        } else {
            <Tree as Traverse>::Scalar::ZERO
        },
        <Tree as Traverse>::Scalar::ZERO,
    );

    tree.set_unrounded(
        root,
        NodeOutputOf {
            order: 0,
            location,
            size: output.size,
            content_size: output.content_size,
            scrollbar_size,
            padding,
            border,
            margin,
        },
    );
}

fn root_known_width<S>(
    style: &NodeInputOf<S>,
    available_width: AvailableOf<S>,
    resolver: &dyn CalcResolver<S>,
) -> Option<S>
where
    S: LayoutScalar,
{
    if style.display.is_inline_level()
        || !style.size.width.is_auto()
        || !style.min_size.width.is_auto()
    {
        return None;
    }

    let available_width = available_width.into_option()?;
    let parent = Size::splat(Some(available_width));
    let padding = style.padding.zip_inline_size(parent, |length, basis| {
        resolve_length_or_zero_with(length, basis, resolver)
    });
    let border = style.border.zip_inline_size(parent, |length, basis| {
        resolve_length_or_zero_with(length, basis, resolver)
    });
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = style
        .max_size
        .zip_map(parent, |dimension, basis| {
            resolve_dimension_with(dimension, basis, resolver)
        })
        .add_optional(box_sizing_adjustment);

    Some(available_width.clamp_optional(None, max_size.width))
}

pub fn round_layout<Tree>(tree: &mut Tree, root: <Tree as Traverse>::Node)
where
    Tree: Round,
{
    round_layout_inner(tree, root, Tree::Scalar::ZERO, Tree::Scalar::ZERO);
}

fn round_layout_inner<Tree>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    cumulative_x: Tree::Scalar,
    cumulative_y: Tree::Scalar,
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
fn round<S: LayoutScalar>(value: S) -> S {
    (value + S::from_f64(0.5)).floor()
}

pub fn compute_leaf<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    measure: impl FnOnce(Size<Option<S>>, Size<AvailableOf<S>>) -> Size<S>,
) -> ComputeOutputOf<S>
where
    S: LayoutScalar,
{
    compute_leaf_with_resolver(input, style, &NoCalcResolver, measure)
}

pub(crate) fn compute_leaf_with_resolver<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolver: &dyn CalcResolver<S>,
    measure: impl FnOnce(Size<Option<S>>, Size<AvailableOf<S>>) -> Size<S>,
) -> ComputeOutputOf<S>
where
    S: LayoutScalar,
{
    let margin = style.margin.zip_inline_size(input.parent, |length, basis| {
        resolve_auto_or_zero_with(length, basis, resolver)
    });
    let padding = style
        .padding
        .zip_inline_size(input.parent, |length, basis| {
            resolve_length_or_zero_with(length, basis, resolver)
        });
    let border = style.border.zip_inline_size(input.parent, |length, basis| {
        resolve_length_or_zero_with(length, basis, resolver)
    });
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let scrollbar_reservation = ScrollbarReservationOf::from_overflow(
        style.overflow,
        style.scrollbar_width,
        Direction::Ltr,
    );
    let content_box_inset =
        content_box_inset_with_scrollbar(padding, border, scrollbar_reservation);
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
                .zip_map(input.parent, |dimension, basis| {
                    resolve_dimension_with(dimension, basis, resolver)
                })
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
            let style_min_size = style
                .min_size
                .zip_map(input.parent, |dimension, basis| {
                    resolve_dimension_with(dimension, basis, resolver)
                })
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
            let style_max_size = style
                .max_size
                .zip_map(input.parent, |dimension, basis| {
                    resolve_dimension_with(dimension, basis, resolver)
                })
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
        || padding.top > S::ZERO
        || padding.bottom > S::ZERO
        || border.top > S::ZERO
        || border.bottom > S::ZERO
        || matches!(node_size.height, Some(height) if height > S::ZERO)
        || matches!(node_min_size.height, Some(height) if height > S::ZERO);

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
        return ComputeOutputOf::from_outer_size(size);
    }

    let available = Size::new(
        input
            .known
            .width
            .map(AvailableOf::definite)
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
            .map(AvailableOf::definite)
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
                .map(|ratio| unclamped.width / ratio.get())
                .unwrap_or(S::ZERO),
        )
    };
    let aspect_size = Size::new(unclamped.width, aspect_height)
        .clamp_optional(node_min_size, node_max_size)
        .max_optional(padding_border_size.map(Some));

    let mut output = ComputeOutputOf::from_sizes(aspect_size, measured + padding.sum_axes());
    output.margins_can_collapse_through =
        !prevents_margin_collapse && aspect_size.height == S::ZERO && measured.height == S::ZERO;
    output
}

fn resolve_length_or_zero_with<S>(
    length: super::LengthOf<S>,
    basis: Option<S>,
    resolver: &dyn CalcResolver<S>,
) -> S
where
    S: LayoutScalar,
{
    resolution_or_zero(length.resolve_with_status(basis, resolver))
}

fn resolve_auto_or_zero_with<S>(
    length: super::LengthAutoOf<S>,
    basis: Option<S>,
    resolver: &dyn CalcResolver<S>,
) -> S
where
    S: LayoutScalar,
{
    resolution_optional(length.resolve_with_status(basis, resolver)).unwrap_or(S::ZERO)
}

fn resolve_dimension_with<S>(
    dimension: super::DimensionOf<S>,
    basis: Option<S>,
    resolver: &dyn CalcResolver<S>,
) -> Option<S>
where
    S: LayoutScalar,
{
    resolution_optional(dimension.resolve_with_status(basis, resolver))
}

fn resolution_or_zero<S: LayoutScalar>(resolution: CalcResolutionOf<S>) -> S {
    match resolution.status() {
        CalcResolutionStatus::Resolved => resolution
            .value
            .expect("resolved calc resolution must carry a value"),
        CalcResolutionStatus::MissingBasis | CalcResolutionStatus::NonNumeric => S::ZERO,
        CalcResolutionStatus::MissingResolver => {
            panic!("calc resolution requires an explicit resolver")
        }
        CalcResolutionStatus::MissingExpression => panic!("calc expression is missing"),
    }
}

fn resolution_optional<S: LayoutScalar>(resolution: CalcResolutionOf<S>) -> Option<S> {
    match resolution.status() {
        CalcResolutionStatus::Resolved => resolution.value,
        CalcResolutionStatus::MissingBasis | CalcResolutionStatus::NonNumeric => None,
        CalcResolutionStatus::MissingResolver => {
            panic!("calc resolution requires an explicit resolver")
        }
        CalcResolutionStatus::MissingExpression => panic!("calc expression is missing"),
    }
}

trait SizeOptionExt {
    type Scalar: LayoutScalar;

    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size<Self::Scalar>) -> Size<Self::Scalar>;
    fn add_optional(self, amount: Size<Self::Scalar>) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> SizeOptionExt for Size<Option<S>> {
    type Scalar = S;

    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size<S>) -> Size<S> {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| width + amount.width),
            self.height.map(|height| height + amount.height),
        )
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<S>>) -> Self {
        let Some(ratio) = aspect_ratio else {
            return self;
        };
        let ratio = ratio.get();
        match (self.width, self.height) {
            (Some(width), None) => Size::new(Some(width), Some(width / ratio)),
            (None, Some(height)) => Size::new(Some(height * ratio), Some(height)),
            _ => self,
        }
    }
}

trait SizeExt {
    type Scalar: LayoutScalar;

    fn clamp_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
    fn max_optional(self, min: Size<Option<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> SizeExt for Size<S> {
    type Scalar = S;

    fn clamp_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width.clamp_optional(min.width, max.width),
            self.height.clamp_optional(min.height, max.height),
        )
    }

    fn max_optional(self, min: Size<Option<S>>) -> Self {
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

impl<S: LayoutScalar> ScalarExt for S {
    fn clamp_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}

trait AvailableExt {
    type Scalar: LayoutScalar;

    fn sub_margin(self, margin: Self::Scalar) -> Self;
    fn set_optional(self, value: Option<Self::Scalar>) -> Self;
    fn map_definite(self, f: impl FnOnce(Self::Scalar) -> Self::Scalar) -> Self;
}

impl<S: LayoutScalar> AvailableExt for AvailableOf<S> {
    type Scalar = S;

    fn sub_margin(self, margin: S) -> Self {
        self.map_definite(|value| value - margin)
    }

    fn set_optional(self, value: Option<S>) -> Self {
        value.map_or(self, AvailableOf::definite)
    }

    fn map_definite(self, f: impl FnOnce(S) -> S) -> Self {
        match self {
            AvailableOf::Definite(value) => AvailableOf::Definite(f(value)),
            AvailableOf::MinContent => AvailableOf::MinContent,
            AvailableOf::MaxContent => AvailableOf::MaxContent,
        }
    }
}
