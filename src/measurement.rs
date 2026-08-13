use crate::error::{
    InvalidMeasurementOutputOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf,
    LayoutInternalInvariant, LayoutInvalidInputOf, LayoutOperation, LayoutResultOf,
    SizingAlgorithm, sizing_resolution_error, sizing_resolution_error_at_site,
};
use crate::geometry::{FlowAxes, PhysicalAxis};
use crate::layout_math::{
    MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt, OptionalMinimumSizeFloorExt,
    OptionalSizeExt,
};
use crate::scroll::{
    CanonicalScrollGeometryErrorOf, ClipMarginSourceOf, MeasuredLeafContentBoxInsetSourceOf,
    MeasuredLeafScrollGeometrySourceOf, OptimalRegionInsetsOf,
    canonical_measured_leaf_scroll_geometry, measured_leaf_content_box_inset,
};
use crate::sizing::resolve::{
    ResolvedPreferredSize, SizingResolutionError, resolution_optional_fallible,
    resolution_or_zero_fallible, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_sizing,
};
use crate::{
    AspectRatioOf, AvailableOf, BoxSizing, CollapsibleMarginOf, ComputeInputOf, ComputeOutputOf,
    DefaultScalar, Edges, LayoutScalar, LayoutTree, LengthResolutionOf, LengthResolutionStatus,
    NodeInputOf, NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf,
    PhysicalBlockMarginCollapseOf, Position, RunMode, Size, SizingMode,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafMeasureInputOf<S: LayoutScalar = DefaultScalar> {
    known_content_size: Size<Option<S>>,
    available_content_size: Size<MeasurementAvailableOf<S>>,
}

pub type LeafMeasureInput = LeafMeasureInputOf<DefaultScalar>;

impl<S: LayoutScalar> LeafMeasureInputOf<S> {
    #[must_use]
    pub const fn known_content_size(&self) -> Size<Option<S>> {
        self.known_content_size
    }

    #[must_use]
    pub const fn available_content_size(&self) -> Size<MeasurementAvailableOf<S>> {
        self.available_content_size
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeasurementAvailableOf<S: LayoutScalar = DefaultScalar> {
    Definite(NonNegativeFiniteOf<S>),
    MinContent,
    MaxContent,
}

pub type MeasurementAvailable = MeasurementAvailableOf<DefaultScalar>;

impl<S: LayoutScalar> MeasurementAvailableOf<S> {
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    pub fn definite(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self::Definite(NonNegativeFiniteOf::new(value)?))
    }

    #[must_use]
    pub const fn definite_value(self) -> Option<NonNegativeFiniteOf<S>> {
        match self {
            Self::Definite(value) => Some(value),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub const fn into_available(self) -> AvailableOf<S> {
        match self {
            Self::Definite(value) => AvailableOf::Definite(value.get()),
            Self::MinContent => AvailableOf::MinContent,
            Self::MaxContent => AvailableOf::MaxContent,
        }
    }

    fn from_content_space(value: AvailableOf<S>) -> Result<Self, S> {
        match value {
            AvailableOf::Definite(value) => {
                let Ok(value) = NonNegativeFiniteOf::new(finite_floor_at_zero(value)?) else {
                    unreachable!("finite content-space availability is non-negative")
                };
                Ok(Self::Definite(value))
            }
            AvailableOf::MinContent => Ok(Self::MinContent),
            AvailableOf::MaxContent => Ok(Self::MaxContent),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LeafMeasureErrorOf<S: LayoutScalar, M> {
    Provider(M),
    InvalidOutput(InvalidMeasurementOutputOf<S>),
}

pub type LeafMeasureError<M> = LeafMeasureErrorOf<DefaultScalar, M>;

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafResolvedValues<S: LayoutScalar> {
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    node_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    preferred_intrinsic_availability: Size<Option<AvailableOf<S>>>,
    flex_basis_intrinsic_availability: Size<Option<AvailableOf<S>>>,
    aspect_ratio: Option<AspectRatioOf<S>>,
}

fn resolve_leaf_values<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolve_auto: impl Fn(crate::LengthAutoOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
    resolve_length: impl Fn(crate::LengthOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    let margin = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.margin, input.parent(), resolve_auto),
    )?;
    let padding = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.padding, input.parent(), &resolve_length),
    )?;
    let border = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.border, input.parent(), resolve_length),
    )?;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };

    let (node_size, node_min_size, node_max_size, preferred_intrinsic_availability, aspect_ratio) =
        match input.sizing_mode() {
            SizingMode::ContentSize => (input.known(), Size::NONE, Size::NONE, Size::NONE, None),
            SizingMode::InherentSize => {
                let missing_basis_is_indefinite = input.run_mode() == RunMode::ComputeSize;
                let preferred = Size::new(
                    resolve_preferred_sizing(
                        &style.size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_preferred_sizing(
                        &style.size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                );
                let style_size = preferred
                    .map(|resolved| match resolved {
                        ResolvedPreferredSize::Definite(value) => Some(value),
                        ResolvedPreferredSize::Auto
                        | ResolvedPreferredSize::MinContent
                        | ResolvedPreferredSize::MaxContent => None,
                    })
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment);
                let preferred_intrinsic_availability = preferred.map(|resolved| match resolved {
                    ResolvedPreferredSize::MinContent => Some(AvailableOf::MIN_CONTENT),
                    ResolvedPreferredSize::MaxContent => Some(AvailableOf::MAX_CONTENT),
                    ResolvedPreferredSize::Auto | ResolvedPreferredSize::Definite(_) => None,
                });
                let style_min_size = Size::new(
                    resolve_minimum_optional(
                        &style.min_size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_minimum_optional(
                        &style.min_size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                )
                .apply_aspect_ratio(style.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let style_max_size = Size::new(
                    resolve_maximum_optional(
                        &style.max_size.width,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        missing_basis_is_indefinite,
                    )?,
                    resolve_maximum_optional(
                        &style.max_size.height,
                        SizingAlgorithm::Leaf,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        missing_basis_is_indefinite,
                    )?,
                )
                .add_optional(box_sizing_adjustment);

                (
                    input.known().or(style_size),
                    style_min_size,
                    style_max_size,
                    preferred_intrinsic_availability,
                    style.aspect_ratio,
                )
            }
        };
    let flex_basis_intrinsic =
        if input.parent_formatting_context() == crate::ParentFormattingContext::Flex {
            if style.flex_basis.is_min_content() {
                Some(AvailableOf::MIN_CONTENT)
            } else if style.flex_basis.is_max_content() {
                Some(AvailableOf::MAX_CONTENT)
            } else {
                None
            }
        } else {
            None
        };
    let flex_basis_intrinsic_availability = input
        .available()
        .map(|available| flex_basis_intrinsic.filter(|intrinsic| *intrinsic == available));

    Ok(LeafResolvedValues {
        margin,
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        preferred_intrinsic_availability,
        flex_basis_intrinsic_availability,
        aspect_ratio,
    })
}

fn resolve_leaf_values_for_input<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    resolve_leaf_values(
        input,
        style,
        |length, basis| resolve_leaf_auto(input, length, basis),
        |length, basis| resolve_leaf_length(input, length, basis),
    )
}

fn transpose_leaf_edges<S, E>(edges: Edges<Result<S, E>>) -> Result<Edges<S>, E> {
    Ok(Edges::new(
        edges.top?,
        edges.right?,
        edges.bottom?,
        edges.left?,
    ))
}

pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> Result<Size<S>, M>,
) -> LayoutResultOf<(), ComputeOutputOf<S>, S, M>
where
    S: LayoutScalar,
{
    let site = LayoutErrorSiteOf::Standalone;
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|error| sizing_resolution_error_at_site(site, error))?;

    compute_leaf_with_resolved_values(site, input, style, resolved, |measure_input| {
        measure(measure_input).map_err(|error| {
            LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )
        })
    })
}

pub(crate) fn compute_tree_leaf<Tree>(
    tree: &Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    style: &NodeInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|error| sizing_resolution_error(node, error))?;

    let site = LayoutErrorSiteOf::Node(node);
    compute_leaf_with_resolved_values(site, input, style, resolved, |measure_input| {
        match tree.measure_leaf(node, measure_input) {
            Some(Ok(output)) => Ok(output),
            Some(Err(error)) => Err(LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )),
            None => Err(LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingLeafMeasurementProvider,
                ),
            )),
        }
    })
}

fn compute_leaf_with_resolved_values<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: LeafResolvedValues<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let LeafResolvedValues {
        padding,
        border,
        node_size,
        node_min_size,
        node_max_size,
        ..
    } = resolved;
    let padding_border = padding + border;
    let padding_border_size = padding_border.sum_axes();
    let leaf_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let block_start = leaf_flow_axes.block_start();
    let block_end = leaf_flow_axes.block_end();
    let node_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_size.width,
        PhysicalAxis::Vertical => node_size.height,
    };
    let node_min_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_min_size.width,
        PhysicalAxis::Vertical => node_min_size.height,
    };

    let prevents_margin_collapse = input.parent_formatting_context()
        != crate::ParentFormattingContext::BlockFlow
        || style.display != crate::Display::Block
        || !style.item_is_replaced && style.overflow.establishes_independent_formatting_context()
        || style.position == Position::Absolute
        || padding.at_physical_side(block_start) > S::ZERO
        || padding.at_physical_side(block_end) > S::ZERO
        || border.at_physical_side(block_start) > S::ZERO
        || border.at_physical_side(block_end) > S::ZERO
        || matches!(node_block_size, Some(size) if size > S::ZERO)
        || matches!(node_min_block_size, Some(size) if size > S::ZERO);

    if input.run_mode() == RunMode::ComputeSize
        && prevents_margin_collapse
        && let Size {
            width: Some(width),
            height: Some(height),
        } = node_size
    {
        let size = Size::new(width, height)
            .clamp_max_before_min_optional(node_min_size, node_max_size)
            .max_optional(padding_border_size.map(Some));
        return Ok(ComputeOutputOf::from_outer_size(size));
    }

    let mut pass_input = input;
    let mut reusable_measurement = None;
    loop {
        let pass = leaf_pass_input(site, pass_input, style, &resolved)?;
        let measured = match reusable_measurement.take() {
            Some((measurement_input, measured)) if measurement_input == pass.measurement_input => {
                measured
            }
            _ => validate_measurement_output(measure(pass.measurement_input)?)
                .map_err(|error| leaf_measurement_error_at_site(site, error))?,
        };
        let unclamped = pass_input
            .known()
            .or(resolved.node_size)
            .unwrap_or(measured + pass.content_box_inset_size);
        let height_is_definite =
            pass_input.known().height.is_some() || resolved.node_size.height.is_some();
        let aspect_height = if height_is_definite {
            unclamped.height
        } else {
            unclamped.height.max(
                resolved
                    .aspect_ratio
                    .map(|ratio| unclamped.width / ratio.get())
                    .unwrap_or(S::ZERO),
            )
        };
        let aspect_size = Size::new(unclamped.width, aspect_height)
            .clamp_max_before_min_optional(resolved.node_min_size, resolved.node_max_size)
            .max_optional(padding_border_size.map(Some));

        let mut output =
            ComputeOutputOf::from_sizes(aspect_size, measured + resolved.padding.sum_axes());
        let can_collapse_through = !prevents_margin_collapse
            && leaf_flow_axes.logical_size(aspect_size).block == S::ZERO
            && leaf_flow_axes.logical_size(measured).block == S::ZERO;
        output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
            leaf_flow_axes,
            CollapsibleMarginOf::ZERO,
            CollapsibleMarginOf::ZERO,
            can_collapse_through,
        );
        let geometry =
            canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
                flow_axes: leaf_flow_axes,
                computed_overflow: style.overflow,
                item_is_replaced: style.item_is_replaced,
                border_box_size: aspect_size,
                border: resolved.border,
                padding: resolved.padding,
                scrollbar_gutter: style.scrollbar_gutter,
                scrollbar_width: style.scrollbar_width,
                settled_auto_scrollbars: pass_input.settled_auto_scrollbars(),
                clip_margin: ClipMarginSourceOf::new(
                    style.overflow_clip_margin.clip_box(),
                    style.overflow_clip_margin.margin(),
                ),
                scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
                measured_content_size: measured,
                scroll_snap_type: style.scroll_snap_type,
                target_scroll_margin: style.scroll_margin,
                target_snap_align: style.scroll_snap_align,
                target_snap_stop: style.scroll_snap_stop,
            })
            .map_err(|error| leaf_scroll_error_at_site(site, pass_input.run_mode(), error))?;
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || style.scrollbar_width.get() == S::ZERO
        {
            if input.run_mode().is_perform_layout() {
                output.content_size = geometry.canonical_content_size().map_err(|error| {
                    leaf_scroll_error_at_site(site, pass_input.run_mode(), error)
                })?;
                output.scroll_geometry = Some(geometry);
            }
            return Ok(output);
        }

        reusable_measurement = Some((pass.measurement_input, measured));
        pass_input = pass_input.with_settled_auto_scrollbars(next_state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafPassInputOf<S: LayoutScalar> {
    content_box_inset_size: Size<S>,
    measurement_input: LeafMeasureInputOf<S>,
}

fn leaf_pass_input<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: &LeafResolvedValues<S>,
) -> LayoutResultOf<Node, LeafPassInputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let content_box_inset = measured_leaf_content_box_inset(MeasuredLeafContentBoxInsetSourceOf {
        flow_axes: FlowAxes::new(style.writing_mode, style.direction),
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: input.settled_auto_scrollbars(),
        padding: resolved.padding,
        border: resolved.border,
    });
    let content_box_inset_size = content_box_inset.sum_axes();
    let available = Size::new(
        input
            .known()
            .width
            .map(AvailableOf::definite)
            .unwrap_or(input.available().width)
            .sub_margin(resolved.margin.horizontal_sum())
            .set_optional(input.known().width)
            .set_optional(resolved.node_size.width)
            .map_definite(|value| {
                value.clamp_max_before_min_optional(
                    resolved.node_min_size.width,
                    resolved.node_max_size.width,
                ) - content_box_inset.horizontal_sum()
            }),
        input
            .known()
            .height
            .map(AvailableOf::definite)
            .unwrap_or(input.available().height)
            .sub_margin(resolved.margin.vertical_sum())
            .set_optional(input.known().height)
            .set_optional(resolved.node_size.height)
            .map_definite(|value| {
                value.clamp_max_before_min_optional(
                    resolved.node_min_size.height,
                    resolved.node_max_size.height,
                ) - content_box_inset.vertical_sum()
            }),
    )
    .zip_map(
        resolved.preferred_intrinsic_availability,
        |available, intrinsic| intrinsic.unwrap_or(available),
    )
    .zip_map(
        resolved.flex_basis_intrinsic_availability,
        |available, intrinsic| intrinsic.unwrap_or(available),
    );

    let known_content_size = match input.run_mode() {
        RunMode::ComputeSize => input.known(),
        RunMode::PerformRootLayout | RunMode::PerformLayout => Size::NONE,
        RunMode::PerformHiddenLayout => {
            unreachable!("hidden layout uses ComputeOutput::HIDDEN")
        }
    }
    .zip_map(content_box_inset_size, |value, inset| {
        value.map(|value| finite_floor_at_zero(value - inset))
    });
    let known_content_size =
        transpose_leaf_known_content_size(known_content_size).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let available_content_size =
        measurement_available_content_size(available).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let measurement_input = LeafMeasureInputOf {
        known_content_size,
        available_content_size,
    };
    Ok(LeafPassInputOf {
        content_box_inset_size,
        measurement_input,
    })
}

fn leaf_scroll_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    run_mode: RunMode,
    _error: CanonicalScrollGeometryErrorOf<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let (operation, invariant) = match (site, run_mode) {
        (LayoutErrorSiteOf::Standalone, _) => (
            LayoutOperation::LeafMeasurement,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (_, RunMode::PerformRootLayout) => (
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        _ => (
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    };
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InternalInvariant(invariant),
    )
}

fn validate_measurement_output<S, M>(measured: Size<S>) -> Result<Size<S>, LeafMeasureErrorOf<S, M>>
where
    S: LayoutScalar,
{
    let width = NonNegativeFiniteOf::new(measured.width).map_err(|error| {
        LeafMeasureErrorOf::InvalidOutput(InvalidMeasurementOutputOf::new(
            PhysicalAxis::Horizontal,
            error,
        ))
    })?;
    let height = NonNegativeFiniteOf::new(measured.height).map_err(|error| {
        LeafMeasureErrorOf::InvalidOutput(InvalidMeasurementOutputOf::new(
            PhysicalAxis::Vertical,
            error,
        ))
    })?;

    Ok(Size::new(width.get(), height.get()))
}

fn finite_floor_at_zero<S>(value: S) -> Result<S, S>
where
    S: LayoutScalar,
{
    if value.is_finite() {
        Ok(value.max(S::ZERO))
    } else {
        Err(value)
    }
}

fn transpose_leaf_known_content_size<S>(
    size: Size<Option<Result<S, S>>>,
) -> Result<Size<Option<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(size.width.transpose()?, size.height.transpose()?))
}

fn measurement_available_content_size<S>(
    available: Size<AvailableOf<S>>,
) -> Result<Size<MeasurementAvailableOf<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(
        MeasurementAvailableOf::from_content_space(available.width)?,
        MeasurementAvailableOf::from_content_space(available.height)?,
    ))
}

fn invalid_numeric_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    value: S,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }),
    )
}

fn leaf_measurement_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    error: LeafMeasureErrorOf<S, M>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = match error {
        LeafMeasureErrorOf::Provider(error) => LayoutErrorKindOf::Measurement(error),
        LeafMeasureErrorOf::InvalidOutput(error) => {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(error))
        }
    };
    LayoutErrorOf::new(site, LayoutOperation::LeafMeasurement, kind)
}

fn resolve_leaf_auto<S>(
    input: ComputeInputOf<S>,
    length: crate::LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolve_leaf_optional(input, length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

fn resolve_leaf_length<S>(
    input: ComputeInputOf<S>,
    length: crate::LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    let resolution = length.resolve_with_status(basis);
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(S::ZERO);
    }

    resolution_or_zero_fallible(resolution)
}

fn resolve_leaf_optional<S>(
    input: ComputeInputOf<S>,
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(None);
    }

    resolution_optional_fallible(resolution)
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
