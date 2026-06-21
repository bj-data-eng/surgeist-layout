use super::{Available, ComputeInput, ComputeOutput, RunMode, Scalar, Size};

const CACHE_SIZE: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry<T> {
    known: Size<Option<Scalar>>,
    available: Size<Available>,
    content: T,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cache {
    final_layout: Option<Entry<ComputeOutput>>,
    measures: [Option<Entry<Size>>; CACHE_SIZE],
    empty: bool,
}

impl Cache {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            final_layout: None,
            measures: [None; CACHE_SIZE],
            empty: true,
        }
    }

    #[must_use]
    pub fn get(&self, input: &ComputeInput) -> Option<ComputeOutput> {
        match input.run_mode {
            RunMode::PerformRootLayout | RunMode::PerformLayout => self
                .final_layout
                .filter(|entry| matches_output(input, entry, entry.content.size))
                .map(|entry| entry.content),
            RunMode::ComputeSize => {
                for entry in self.measures.iter().flatten() {
                    if matches_output(input, entry, entry.content) {
                        return Some(ComputeOutput::from_outer_size(entry.content));
                    }
                }
                None
            }
            RunMode::PerformHiddenLayout => None,
        }
    }

    pub fn store(&mut self, input: &ComputeInput, output: ComputeOutput) {
        match input.run_mode {
            RunMode::PerformRootLayout | RunMode::PerformLayout => {
                self.empty = false;
                self.final_layout = Some(Entry {
                    known: input.known,
                    available: input.available,
                    content: output,
                });
            }
            RunMode::ComputeSize => {
                self.empty = false;
                let slot = cache_slot(input.known, input.available);
                self.measures[slot] = Some(Entry {
                    known: input.known,
                    available: input.available,
                    content: output.size,
                });
            }
            RunMode::PerformHiddenLayout => {}
        }
    }

    pub fn clear(&mut self) -> ClearState {
        if self.empty {
            return ClearState::AlreadyEmpty;
        }
        self.empty = true;
        self.final_layout = None;
        self.measures = [None; CACHE_SIZE];
        ClearState::Cleared
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.final_layout.is_none() && self.measures.iter().all(Option::is_none)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClearState {
    Cleared,
    AlreadyEmpty,
}

fn cache_slot(known: Size<Option<Scalar>>, available: Size<Available>) -> usize {
    let has_known_width = known.width.is_some();
    let has_known_height = known.height.is_some();

    if has_known_width && has_known_height {
        return 0;
    }

    if has_known_width && !has_known_height {
        return 1 + usize::from(available.height == Available::MIN_CONTENT);
    }

    if has_known_height && !has_known_width {
        return 3 + usize::from(available.width == Available::MIN_CONTENT);
    }

    match (available.width, available.height) {
        (
            Available::MaxContent | Available::Definite(_),
            Available::MaxContent | Available::Definite(_),
        ) => 5,
        (Available::MaxContent | Available::Definite(_), Available::MinContent) => 6,
        (Available::MinContent, Available::MaxContent | Available::Definite(_)) => 7,
        (Available::MinContent, Available::MinContent) => 8,
    }
}

fn matches_output<T>(input: &ComputeInput, entry: &Entry<T>, cached_size: Size) -> bool {
    (input.known.width == entry.known.width || input.known.width == Some(cached_size.width))
        && (input.known.height == entry.known.height
            || input.known.height == Some(cached_size.height))
        && (input.known.width.is_some() || entry.available.width.roughly_eq(input.available.width))
        && (input.known.height.is_some()
            || entry.available.height.roughly_eq(input.available.height))
}
