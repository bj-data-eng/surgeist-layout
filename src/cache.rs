use super::{
    AvailableOf, ComputeInputOf, ComputeOutputOf, DefaultScalar, LayoutScalar, RequestedAxis,
    RunMode, Size, SizingMode,
};
use crate::geometry::FlowAxes;

const CACHE_SIZE: usize = 9;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CacheKeyContext;

impl CacheKeyContext {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CacheKeyOf<S: LayoutScalar = DefaultScalar> {
    run_mode: RunMode,
    sizing_mode: SizingMode,
    axis: RequestedAxis,
    known: Size<Option<S>>,
    parent: Size<Option<S>>,
    containing_flow_axes: FlowAxes,
    available: Size<AvailableOf<S>>,
    context: CacheKeyContext,
}

impl<S: LayoutScalar> CacheKeyOf<S> {
    fn from_input(input: &ComputeInputOf<S>, context: CacheKeyContext) -> Self {
        Self {
            run_mode: input.run_mode(),
            sizing_mode: input.sizing_mode(),
            axis: input.requested_axis(),
            known: input.known(),
            parent: input.parent(),
            containing_flow_axes: input.containing_flow_axes(),
            available: input.available(),
            context,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EntryOf<S: LayoutScalar, T> {
    key: CacheKeyOf<S>,
    content: T,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CacheOf<S: LayoutScalar = DefaultScalar> {
    final_layout: Option<EntryOf<S, ComputeOutputOf<S>>>,
    measures: [Option<EntryOf<S, ComputeOutputOf<S>>>; CACHE_SIZE],
    empty: bool,
}

pub type Cache = CacheOf<DefaultScalar>;

impl<S: LayoutScalar> CacheOf<S> {
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
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        match input.run_mode() {
            RunMode::PerformRootLayout | RunMode::PerformLayout => self
                .final_layout
                .filter(|entry| matches_output(input, context, entry, entry.content.size))
                .map(|entry| entry.content),
            RunMode::ComputeSize => {
                for entry in self.measures.iter().flatten() {
                    if matches_output(input, context, entry, entry.content.size) {
                        return Some(entry.content);
                    }
                }
                None
            }
            RunMode::PerformHiddenLayout => None,
        }
    }

    pub fn store_with_context(
        &mut self,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
        output: ComputeOutputOf<S>,
    ) {
        let key = CacheKeyOf::from_input(input, context);
        match input.run_mode() {
            RunMode::PerformRootLayout | RunMode::PerformLayout => {
                self.empty = false;
                self.final_layout = Some(EntryOf {
                    key,
                    content: output,
                });
            }
            RunMode::ComputeSize => {
                self.empty = false;
                let slot = cache_slot(input.known(), input.available());
                self.measures[slot] = Some(EntryOf {
                    key,
                    content: output,
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

impl<S: LayoutScalar> Default for CacheOf<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClearState {
    Cleared,
    AlreadyEmpty,
}

fn cache_slot<S: LayoutScalar>(known: Size<Option<S>>, available: Size<AvailableOf<S>>) -> usize {
    let has_known_width = known.width.is_some();
    let has_known_height = known.height.is_some();

    if has_known_width && has_known_height {
        return 0;
    }

    if has_known_width && !has_known_height {
        return 1 + usize::from(available.height == AvailableOf::MIN_CONTENT);
    }

    if has_known_height && !has_known_width {
        return 3 + usize::from(available.width == AvailableOf::MIN_CONTENT);
    }

    match (available.width, available.height) {
        (
            AvailableOf::MaxContent | AvailableOf::Definite(_),
            AvailableOf::MaxContent | AvailableOf::Definite(_),
        ) => 5,
        (AvailableOf::MaxContent | AvailableOf::Definite(_), AvailableOf::MinContent) => 6,
        (AvailableOf::MinContent, AvailableOf::MaxContent | AvailableOf::Definite(_)) => 7,
        (AvailableOf::MinContent, AvailableOf::MinContent) => 8,
    }
}

fn matches_output<S: LayoutScalar, T>(
    input: &ComputeInputOf<S>,
    context: CacheKeyContext,
    entry: &EntryOf<S, T>,
    cached_size: Size<S>,
) -> bool {
    let key = CacheKeyOf::from_input(input, context);
    input.run_mode() == entry.key.run_mode
        && input.sizing_mode() == entry.key.sizing_mode
        && input.requested_axis() == entry.key.axis
        && input.parent() == entry.key.parent
        && input.containing_flow_axes() == entry.key.containing_flow_axes
        && context == entry.key.context
        && (input.known().width == entry.key.known.width
            || input.known().width == Some(cached_size.width))
        && (input.known().height == entry.key.known.height
            || input.known().height == Some(cached_size.height))
        && (input.known().width.is_some()
            || entry.key.available.width.roughly_eq(key.available.width))
        && (input.known().height.is_some()
            || entry.key.available.height.roughly_eq(key.available.height))
}
