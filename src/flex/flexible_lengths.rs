use super::items::ResolvedFlexItem;
use super::{Constants, FlexAxes, LayoutScalar};
use crate::layout_math::MaxBeforeMinScalarClampExt;

pub(super) fn resolve_flexible_lengths<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let free_space = container_main_size - occupied_main_size(items, constants);
    if free_space.abs() < S::from_f64(0.0001) {
        return;
    }
    if free_space > S::ZERO {
        distribute_positive_free_space(items, constants);
    } else if free_space < S::ZERO {
        distribute_negative_free_space(items, constants);
    }
}

fn distribute_positive_free_space<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = constants
            .axes
            .with_main_size(item.target_size, item.flex_basis);
        if item.flex_grow_factor == S::ZERO || item.flex_basis > item.hypothetical_main_size {
            item.target_size = constants
                .axes
                .with_main_size(item.target_size, item.hypothetical_main_size);
            *frozen = true;
        }
    }

    loop {
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
        let mut free_space = container_main_size - flex_used_space(items, constants, &frozen);
        let grow_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_grow_factor)
            .fold(S::ZERO, |sum, value| sum + value);
        if grow_sum <= S::ZERO {
            return;
        }
        if grow_sum < S::ONE {
            let partial_free_space = initial_free_space * grow_sum;
            if partial_free_space.abs() < free_space.abs() {
                free_space = partial_free_space;
            }
        }

        let mut total_violation = S::ZERO;
        let mut violations = vec![S::ZERO; items.len()];
        for (index, (item, frozen)) in items.iter_mut().zip(&frozen).enumerate() {
            if *frozen {
                continue;
            }

            let grown_main_size = item.flex_basis + free_space * item.flex_grow_factor / grow_sum;
            let clamped = clamp_main_size(item, constants.axes, grown_main_size);
            item.target_size = constants.axes.with_main_size(item.target_size, clamped);
            let violation = clamped - grown_main_size;
            violations[index] = violation;
            total_violation = total_violation + violation;
        }

        freeze_violations(&mut frozen, &violations, total_violation);
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
    }
}

fn distribute_negative_free_space<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) {
    let mut frozen = vec![false; items.len()];
    let Some(container_main_size) = flex_main_size(constants) else {
        return;
    };
    let initial_free_space = container_main_size - flex_used_space(items, constants, &frozen);

    for (item, frozen) in items.iter_mut().zip(&mut frozen) {
        item.target_size = constants
            .axes
            .with_main_size(item.target_size, item.flex_basis);
        if item.flex_shrink_factor == S::ZERO || item.flex_basis < item.hypothetical_main_size {
            item.target_size = constants
                .axes
                .with_main_size(item.target_size, item.hypothetical_main_size);
            *frozen = true;
        }
    }

    loop {
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
        let mut free_space = container_main_size - flex_used_space(items, constants, &frozen);
        let shrink_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_shrink_factor)
            .fold(S::ZERO, |sum, value| sum + value);
        let scaled_shrink_sum = items
            .iter()
            .zip(&frozen)
            .filter(|(_, frozen)| !**frozen)
            .map(|(item, _)| item.flex_shrink_factor * item.flex_basis)
            .fold(S::ZERO, |sum, value| sum + value);
        if shrink_sum <= S::ZERO || scaled_shrink_sum <= S::ZERO {
            return;
        }
        if shrink_sum < S::ONE {
            let partial_free_space = initial_free_space * shrink_sum;
            if partial_free_space.abs() < free_space.abs() {
                free_space = partial_free_space;
            }
        }

        let mut total_violation = S::ZERO;
        let mut violations = vec![S::ZERO; items.len()];
        for (index, (item, frozen)) in items.iter_mut().zip(&frozen).enumerate() {
            if *frozen {
                continue;
            }

            let scaled_shrink = item.flex_shrink_factor * item.flex_basis;
            let shrunken_main_size =
                item.flex_basis + free_space * scaled_shrink / scaled_shrink_sum;
            let clamped =
                clamp_main_size(item, constants.axes, S::max(S::ZERO, shrunken_main_size));
            item.target_size = constants.axes.with_main_size(item.target_size, clamped);
            let violation = clamped - shrunken_main_size;
            violations[index] = violation;
            total_violation = total_violation + violation;
        }

        freeze_violations(&mut frozen, &violations, total_violation);
        if frozen.iter().all(|frozen| *frozen) {
            return;
        }
    }
}

fn flex_used_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
    frozen: &[bool],
) -> S {
    items
        .iter()
        .zip(frozen)
        .enumerate()
        .map(|(index, (item, frozen))| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.axes.main_size(constants.gap)
            };
            let main_size = if *frozen {
                constants.axes.main_size(item.target_size)
            } else {
                item.flex_basis
            };
            gap + main_size + constants.axes.main_edge_sum(item.margin)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

fn freeze_violations<S: LayoutScalar>(frozen: &mut [bool], violations: &[S], total_violation: S) {
    if total_violation == S::ZERO {
        for frozen in frozen {
            *frozen = true;
        }
    } else if total_violation > S::ZERO {
        for (frozen, violation) in frozen.iter_mut().zip(violations) {
            if *violation > S::ZERO {
                *frozen = true;
            }
        }
    } else {
        for (frozen, violation) in frozen.iter_mut().zip(violations) {
            if *violation < S::ZERO {
                *frozen = true;
            }
        }
    }
}

fn occupied_main_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 {
                S::ZERO
            } else {
                constants.axes.main_size(constants.gap)
            };
            gap + constants.axes.main_size(item.target_size)
                + constants.axes.main_edge_sum(item.margin)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

fn clamp_main_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    axes: FlexAxes,
    value: S,
) -> S {
    clamp_main_size_axes(
        value,
        item.automatic_min_main_size,
        axes.main_size(item.min_size),
        axes.main_size(item.max_size),
    )
}

pub(super) fn clamp_cross_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    value: S,
) -> S {
    value.clamp_max_before_min_optional(item.min_cross_size, item.max_cross_size)
}

pub(super) fn clamp_main_size_axes<S: LayoutScalar>(
    value: S,
    automatic_min: Option<S>,
    min: Option<S>,
    max: Option<S>,
) -> S {
    let value = max.map_or(value, |max| value.min(max));
    let value = automatic_min.map_or(value, |min| value.max(min));
    min.map_or(value, |min| value.max(min))
}

pub(super) fn flex_main_size<S: LayoutScalar>(constants: &Constants<S>) -> Option<S> {
    constants.axes.main_size(constants.node_inner_size)
}
