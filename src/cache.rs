use super::{
    Available, CalcGeneration, ComputeInput, ComputeOutput, RequestedAxis, RunMode, Scalar, Size,
    SizingMode,
};

const CACHE_SIZE: usize = 9;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CacheKeyContext {
    calc_generation: CalcGeneration,
}

impl CacheKeyContext {
    #[must_use]
    pub const fn new(calc_generation: CalcGeneration) -> Self {
        Self { calc_generation }
    }

    #[must_use]
    pub const fn static_no_calc() -> Self {
        Self::new(CalcGeneration::static_no_calc())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CacheKey {
    run_mode: RunMode,
    sizing_mode: SizingMode,
    axis: RequestedAxis,
    known: Size<Option<Scalar>>,
    parent: Size<Option<Scalar>>,
    available: Size<Available>,
    context: CacheKeyContext,
}

impl CacheKey {
    fn from_input(input: &ComputeInput, context: CacheKeyContext) -> Self {
        Self {
            run_mode: input.run_mode,
            sizing_mode: input.sizing_mode,
            axis: input.axis,
            known: input.known,
            parent: input.parent,
            available: input.available,
            context,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry<T> {
    key: CacheKey,
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
    pub fn get_with_context(
        &self,
        input: &ComputeInput,
        context: CacheKeyContext,
    ) -> Option<ComputeOutput> {
        match input.run_mode {
            RunMode::PerformRootLayout | RunMode::PerformLayout => self
                .final_layout
                .filter(|entry| matches_output(input, context, entry, entry.content.size))
                .map(|entry| entry.content),
            RunMode::ComputeSize => {
                for entry in self.measures.iter().flatten() {
                    if matches_output(input, context, entry, entry.content) {
                        return Some(ComputeOutput::from_outer_size(entry.content));
                    }
                }
                None
            }
            RunMode::PerformHiddenLayout => None,
        }
    }

    pub fn store_with_context(
        &mut self,
        input: &ComputeInput,
        context: CacheKeyContext,
        output: ComputeOutput,
    ) {
        let key = CacheKey::from_input(input, context);
        match input.run_mode {
            RunMode::PerformRootLayout | RunMode::PerformLayout => {
                self.empty = false;
                self.final_layout = Some(Entry {
                    key,
                    content: output,
                });
            }
            RunMode::ComputeSize => {
                self.empty = false;
                let slot = cache_slot(input.known, input.available);
                self.measures[slot] = Some(Entry {
                    key,
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

fn matches_output<T>(
    input: &ComputeInput,
    context: CacheKeyContext,
    entry: &Entry<T>,
    cached_size: Size,
) -> bool {
    let key = CacheKey::from_input(input, context);
    input.run_mode == entry.key.run_mode
        && input.sizing_mode == entry.key.sizing_mode
        && input.axis == entry.key.axis
        && input.parent == entry.key.parent
        && context == entry.key.context
        && (input.known.width == entry.key.known.width
            || input.known.width == Some(cached_size.width))
        && (input.known.height == entry.key.known.height
            || input.known.height == Some(cached_size.height))
        && (input.known.width.is_some()
            || entry.key.available.width.roughly_eq(key.available.width))
        && (input.known.height.is_some()
            || entry.key.available.height.roughly_eq(key.available.height))
}
