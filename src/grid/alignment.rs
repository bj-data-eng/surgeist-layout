use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridAlignment {
    pub(super) start: Scalar,
    pub(super) gap: Scalar,
}

pub(super) fn grid_alignment(
    free_space: Scalar,
    track_count: usize,
    base_gap: Scalar,
    alignment: AlignContent,
) -> GridAlignment {
    let alignment = alignment.safe_fallback(free_space);
    if track_count <= 1 || free_space <= 0.0 {
        return GridAlignment {
            start: match alignment {
                AlignContent::Center => free_space / 2.0,
                AlignContent::End | AlignContent::FlexEnd => free_space,
                AlignContent::Start
                | AlignContent::FlexStart
                | AlignContent::Stretch
                | AlignContent::SpaceBetween
                | AlignContent::SpaceAround
                | AlignContent::SpaceEvenly => 0.0,
                AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                    unreachable!("safe_fallback returns unsafe content alignment")
                }
            },
            gap: base_gap,
        };
    }

    match alignment {
        AlignContent::SpaceBetween => GridAlignment {
            start: 0.0,
            gap: base_gap + free_space / (track_count - 1) as Scalar,
        },
        AlignContent::SpaceAround => {
            let distributed = free_space / track_count as Scalar;
            GridAlignment {
                start: distributed / 2.0,
                gap: base_gap + distributed,
            }
        }
        AlignContent::SpaceEvenly => {
            let distributed = free_space / (track_count + 1) as Scalar;
            GridAlignment {
                start: distributed,
                gap: base_gap + distributed,
            }
        }
        AlignContent::Center => GridAlignment {
            start: free_space / 2.0,
            gap: base_gap,
        },
        AlignContent::End | AlignContent::FlexEnd => GridAlignment {
            start: free_space,
            gap: base_gap,
        },
        AlignContent::Start | AlignContent::FlexStart | AlignContent::Stretch => GridAlignment {
            start: 0.0,
            gap: base_gap,
        },
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
            unreachable!("safe_fallback returns unsafe content alignment")
        }
    }
}
