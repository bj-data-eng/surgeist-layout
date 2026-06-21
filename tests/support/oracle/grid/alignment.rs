use super::tracks::SolvedTracks;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignmentSafety {
    Unsafe,
    Safe,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlignmentReport {
    pub leading_offset: f32,
    pub distributed_gap: f32,
    pub offsets: Vec<f32>,
    pub safe_fallback_used: bool,
}

#[must_use]
pub fn align_tracks(
    container: f32,
    sizes: Vec<f32>,
    gap: f32,
    alignment: TrackAlignment,
) -> SolvedTracks {
    let report = align_tracks_report(
        container,
        sizes.clone(),
        gap,
        alignment,
        AlignmentSafety::Unsafe,
    );
    let mut solved = SolvedTracks::new(sizes, report.distributed_gap);
    for (offset, expected) in solved.offsets_mut().iter_mut().zip(report.offsets) {
        *offset = expected;
    }
    solved
}

#[must_use]
pub fn align_tracks_report(
    container: f32,
    sizes: Vec<f32>,
    gap: f32,
    alignment: TrackAlignment,
    safety: AlignmentSafety,
) -> AlignmentReport {
    let track_count = sizes.len();
    let occupied = sizes.iter().sum::<f32>() + gap * track_count.saturating_sub(1) as f32;
    let overflows = occupied > container;
    let safe_fallback_used = safety == AlignmentSafety::Safe && overflows;
    let alignment = if safe_fallback_used {
        TrackAlignment::Start
    } else {
        alignment
    };
    let free = (container - occupied).max(0.0);

    let (leading_offset, distributed_gap) = match alignment {
        TrackAlignment::Start => (0.0, gap),
        TrackAlignment::End => (free, gap),
        TrackAlignment::Center => (free / 2.0, gap),
        TrackAlignment::SpaceBetween if track_count > 1 => {
            (0.0, gap + free / (track_count - 1) as f32)
        }
        TrackAlignment::SpaceBetween => (0.0, gap),
        TrackAlignment::SpaceAround if track_count > 0 => {
            let space = free / track_count as f32;
            (space / 2.0, gap + space)
        }
        TrackAlignment::SpaceAround => (0.0, gap),
        TrackAlignment::SpaceEvenly if track_count > 0 => {
            let space = free / (track_count + 1) as f32;
            (space, gap + space)
        }
        TrackAlignment::SpaceEvenly => (0.0, gap),
    };

    let mut offset = leading_offset;
    let mut offsets = Vec::with_capacity(sizes.len());
    for size in sizes {
        offsets.push(offset);
        offset += size + distributed_gap;
    }

    AlignmentReport {
        leading_offset,
        distributed_gap,
        offsets,
        safe_fallback_used,
    }
}
