use super::super::LayoutScalar;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Baseline,
    LastBaseline,
    Stretch,
}

impl AlignItems {
    #[must_use]
    pub const fn unsafe_position(self) -> Self {
        match self {
            Self::SafeEnd => Self::End,
            Self::SafeFlexEnd => Self::FlexEnd,
            Self::SafeCenter => Self::Center,
            Self::Baseline | Self::LastBaseline => self,
            position => position,
        }
    }

    /// Applies CSS safe alignment fallback for any layout scalar lane.
    ///
    /// This is intentionally non-const because generic scalar comparison is
    /// provided through the `LayoutScalar` contract.
    #[must_use]
    pub fn safe_fallback<S: LayoutScalar>(self, free_space: S) -> Self {
        if free_space < S::ZERO {
            match self {
                Self::SafeEnd | Self::SafeFlexEnd | Self::SafeCenter => Self::Start,
                position => position.unsafe_position(),
            }
        } else {
            self.unsafe_position()
        }
    }
}

pub type AlignSelf = AlignItems;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl AlignContent {
    #[must_use]
    pub const fn reversed(self) -> Self {
        match self {
            Self::Start => Self::End,
            Self::End => Self::Start,
            Self::FlexStart => Self::FlexEnd,
            Self::FlexEnd => Self::FlexStart,
            Self::SafeEnd => Self::Start,
            Self::SafeFlexEnd => Self::FlexStart,
            Self::Stretch => Self::End,
            style => style,
        }
    }

    #[must_use]
    pub const fn unsafe_position(self) -> Self {
        match self {
            Self::SafeEnd => Self::End,
            Self::SafeFlexEnd => Self::FlexEnd,
            Self::SafeCenter => Self::Center,
            position => position,
        }
    }

    /// Applies CSS safe alignment fallback for any layout scalar lane.
    ///
    /// This is intentionally non-const because generic scalar comparison is
    /// provided through the `LayoutScalar` contract.
    #[must_use]
    pub fn safe_fallback<S: LayoutScalar>(self, free_space: S) -> Self {
        if free_space < S::ZERO {
            match self {
                Self::SafeEnd | Self::SafeFlexEnd | Self::SafeCenter => Self::Start,
                position => position.unsafe_position(),
            }
        } else {
            self.unsafe_position()
        }
    }
}

pub type JustifyContent = AlignContent;

/// A layout item's signed order value, independent of its source identity.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemOrder(i32);

impl ItemOrder {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

pub(crate) fn item_order_permutation(
    items: &[(ItemOrder, crate::SourceIndex)],
) -> Vec<crate::SourceIndex> {
    let mut ordered = items.to_vec();
    ordered.sort_by_key(|&(item_order, source_index)| (item_order, source_index));
    ordered
        .into_iter()
        .map(|(_, source_index)| source_index)
        .collect()
}
