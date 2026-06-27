use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridAlignment<S: LayoutScalar = Scalar> {
    pub(super) start: S,
    pub(super) gap: S,
}

pub(super) fn grid_alignment<S: LayoutScalar>(
    free_space: S,
    track_count: usize,
    base_gap: S,
    alignment: AlignContent,
) -> GridAlignment<S> {
    let alignment = alignment.safe_fallback(free_space);
    if track_count <= 1 || free_space <= S::ZERO {
        return GridAlignment {
            start: match alignment {
                AlignContent::Center => free_space / S::from_f64(2.0),
                AlignContent::End | AlignContent::FlexEnd => free_space,
                AlignContent::Start
                | AlignContent::FlexStart
                | AlignContent::Stretch
                | AlignContent::SpaceBetween
                | AlignContent::SpaceAround
                | AlignContent::SpaceEvenly => S::ZERO,
                AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                    unreachable!("safe_fallback returns unsafe content alignment")
                }
            },
            gap: base_gap,
        };
    }

    match alignment {
        AlignContent::SpaceBetween => GridAlignment {
            start: S::ZERO,
            gap: base_gap + free_space / S::from_usize(track_count - 1),
        },
        AlignContent::SpaceAround => {
            let distributed = free_space / S::from_usize(track_count);
            GridAlignment {
                start: distributed / S::from_f64(2.0),
                gap: base_gap + distributed,
            }
        }
        AlignContent::SpaceEvenly => {
            let distributed = free_space / S::from_usize(track_count + 1);
            GridAlignment {
                start: distributed,
                gap: base_gap + distributed,
            }
        }
        AlignContent::Center => GridAlignment {
            start: free_space / S::from_f64(2.0),
            gap: base_gap,
        },
        AlignContent::End | AlignContent::FlexEnd => GridAlignment {
            start: free_space,
            gap: base_gap,
        },
        AlignContent::Start | AlignContent::FlexStart | AlignContent::Stretch => GridAlignment {
            start: S::ZERO,
            gap: base_gap,
        },
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
            unreachable!("safe_fallback returns unsafe content alignment")
        }
    }
}
