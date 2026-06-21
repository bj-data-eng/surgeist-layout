use super::Scalar;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Available {
    Definite(Scalar),
    MinContent,
    MaxContent,
}

impl Available {
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    #[must_use]
    pub const fn definite(value: Scalar) -> Self {
        Self::Definite(value)
    }

    #[must_use]
    pub const fn into_option(self) -> Option<Scalar> {
        match self {
            Self::Definite(value) => Some(value),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub fn roughly_eq(self, other: Self) -> bool {
        match (self, other) {
            (Self::Definite(a), Self::Definite(b)) => (a - b).abs() < Scalar::EPSILON,
            (Self::MinContent, Self::MinContent) | (Self::MaxContent, Self::MaxContent) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Normal,
    Px(Scalar),
    Percent(Scalar),
}

impl Length {
    pub const NORMAL: Self = Self::Normal;
    pub const ZERO: Self = Self::Px(0.0);

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Percent(value)
    }

    #[must_use]
    pub fn resolve(self, basis: Scalar) -> Scalar {
        match self {
            Self::Normal => 0.0,
            Self::Px(value) => value,
            Self::Percent(value) => value * basis,
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthAuto {
    Px(Scalar),
    Percent(Scalar),
    Auto,
}

impl LengthAuto {
    pub const ZERO: Self = Self::Px(0.0);
    pub const AUTO: Self = Self::Auto;

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Percent(value)
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    #[must_use]
    pub fn resolve(self, basis: Scalar) -> Option<Scalar> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => Some(value * basis),
            Self::Auto => None,
        }
    }

    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl Default for LengthAuto {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Length> for LengthAuto {
    fn from(value: Length) -> Self {
        match value {
            Length::Normal => Self::ZERO,
            Length::Px(value) => Self::Px(value),
            Length::Percent(value) => Self::Percent(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dimension {
    Px(Scalar),
    Percent(Scalar),
    Fr(Scalar),
    Auto,
    MinContent,
    MaxContent,
}

impl Dimension {
    pub const ZERO: Self = Self::Px(0.0);
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Px(value)
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Percent(value)
    }

    #[must_use]
    pub const fn fr(value: Scalar) -> Self {
        Self::Fr(value)
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    #[must_use]
    pub fn resolve(self, basis: Scalar) -> Option<Scalar> {
        match self {
            Self::Px(value) => Some(value),
            Self::Percent(value) => Some(value * basis),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }

    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    #[must_use]
    pub const fn is_min_content(self) -> bool {
        matches!(self, Self::MinContent)
    }

    #[must_use]
    pub const fn is_max_content(self) -> bool {
        matches!(self, Self::MaxContent)
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Length> for Dimension {
    fn from(value: Length) -> Self {
        match value {
            Length::Normal => Self::ZERO,
            Length::Px(value) => Self::Px(value),
            Length::Percent(value) => Self::Percent(value),
        }
    }
}

impl From<LengthAuto> for Dimension {
    fn from(value: LengthAuto) -> Self {
        match value {
            LengthAuto::Px(value) => Self::Px(value),
            LengthAuto::Percent(value) => Self::Percent(value),
            LengthAuto::Auto => Self::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MinTrackSizing {
    Length(Length),
    Auto,
    MinContent,
    MaxContent,
}

impl MinTrackSizing {
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;
    pub const ZERO: Self = Self::Length(Length::ZERO);

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Length(Length::px(value))
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Length(Length::percent(value))
    }

    #[must_use]
    pub const fn is_intrinsic(self) -> bool {
        matches!(self, Self::Auto | Self::MinContent | Self::MaxContent)
    }

    #[must_use]
    pub fn definite(self, basis: Option<Scalar>) -> Option<Scalar> {
        match self {
            Self::Length(length) => basis
                .map(|basis| length.resolve(basis))
                .or_else(|| matches!(length, Length::Px(_)).then(|| length.resolve(0.0))),
            Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }
}

impl From<Length> for MinTrackSizing {
    fn from(value: Length) -> Self {
        Self::Length(value)
    }
}

impl From<Dimension> for MinTrackSizing {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Px(value) => Self::px(value),
            Dimension::Percent(value) => Self::percent(value),
            Dimension::Fr(_) | Dimension::Auto => Self::Auto,
            Dimension::MinContent => Self::MinContent,
            Dimension::MaxContent => Self::MaxContent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaxTrackSizing {
    Length(Length),
    Flex(Scalar),
    Auto,
    MinContent,
    MaxContent,
    FitContent(Length),
}

impl MaxTrackSizing {
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;
    pub const ZERO: Self = Self::Length(Length::ZERO);

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Length(Length::px(value))
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Length(Length::percent(value))
    }

    #[must_use]
    pub const fn fr(value: Scalar) -> Self {
        Self::Flex(value)
    }

    #[must_use]
    pub const fn fit_content(limit: Length) -> Self {
        Self::FitContent(limit)
    }

    #[must_use]
    pub const fn is_flexible(self) -> bool {
        matches!(self, Self::Flex(_))
    }

    #[must_use]
    pub const fn is_intrinsic(self) -> bool {
        matches!(
            self,
            Self::Auto | Self::MinContent | Self::MaxContent | Self::FitContent(_)
        )
    }

    #[must_use]
    pub fn definite(self, basis: Option<Scalar>) -> Option<Scalar> {
        match self {
            Self::Length(length) => basis
                .map(|basis| length.resolve(basis))
                .or_else(|| matches!(length, Length::Px(_)).then(|| length.resolve(0.0))),
            Self::Flex(_)
            | Self::Auto
            | Self::MinContent
            | Self::MaxContent
            | Self::FitContent(_) => None,
        }
    }

    #[must_use]
    pub fn fit_limit(self, basis: Option<Scalar>) -> Option<Scalar> {
        match self {
            Self::FitContent(limit) => basis
                .map(|basis| limit.resolve(basis))
                .or_else(|| matches!(limit, Length::Px(_)).then(|| limit.resolve(0.0))),
            _ => None,
        }
    }
}

impl From<Length> for MaxTrackSizing {
    fn from(value: Length) -> Self {
        Self::Length(value)
    }
}

impl From<Dimension> for MaxTrackSizing {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Px(value) => Self::px(value),
            Dimension::Percent(value) => Self::percent(value),
            Dimension::Fr(value) => Self::fr(value),
            Dimension::Auto => Self::Auto,
            Dimension::MinContent => Self::MinContent,
            Dimension::MaxContent => Self::MaxContent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSizing {
    pub min: MinTrackSizing,
    pub max: MaxTrackSizing,
}

impl TrackSizing {
    pub const AUTO: Self = Self {
        min: MinTrackSizing::AUTO,
        max: MaxTrackSizing::AUTO,
    };
    pub const MIN_CONTENT: Self = Self {
        min: MinTrackSizing::MIN_CONTENT,
        max: MaxTrackSizing::MIN_CONTENT,
    };
    pub const MAX_CONTENT: Self = Self {
        min: MinTrackSizing::MAX_CONTENT,
        max: MaxTrackSizing::MAX_CONTENT,
    };
    pub const ZERO: Self = Self {
        min: MinTrackSizing::ZERO,
        max: MaxTrackSizing::ZERO,
    };

    #[must_use]
    pub const fn new(min: MinTrackSizing, max: MaxTrackSizing) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::new(MinTrackSizing::px(value), MaxTrackSizing::px(value))
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::new(
            MinTrackSizing::percent(value),
            MaxTrackSizing::percent(value),
        )
    }

    #[must_use]
    pub const fn fr(value: Scalar) -> Self {
        Self::new(MinTrackSizing::AUTO, MaxTrackSizing::fr(value))
    }

    #[must_use]
    pub const fn fit_content(limit: Length) -> Self {
        Self::new(MinTrackSizing::AUTO, MaxTrackSizing::fit_content(limit))
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizing, max: MaxTrackSizing) -> Self {
        Self::new(min, max)
    }
}

impl Default for TrackSizing {
    fn default() -> Self {
        Self::AUTO
    }
}

impl From<Dimension> for TrackSizing {
    fn from(value: Dimension) -> Self {
        Self::new(value.into(), value.into())
    }
}

impl From<Length> for TrackSizing {
    fn from(value: Length) -> Self {
        Self::new(value.into(), value.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackRepeat {
    Count(usize),
    AutoFill,
    AutoFit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackRepetition {
    pub repeat: TrackRepeat,
    pub components: Vec<TrackComponent>,
}

impl TrackRepetition {
    #[must_use]
    pub fn count(count: usize, tracks: Vec<TrackSizing>) -> Self {
        Self::count_components(count, track_sizing_components_from_tracks(tracks))
    }

    #[must_use]
    pub fn auto_fill(tracks: Vec<TrackSizing>) -> Self {
        Self::auto_fill_components(track_sizing_components_from_tracks(tracks))
    }

    #[must_use]
    pub fn auto_fit(tracks: Vec<TrackSizing>) -> Self {
        Self::auto_fit_components(track_sizing_components_from_tracks(tracks))
    }

    #[must_use]
    pub fn count_components(count: usize, components: Vec<TrackComponent>) -> Self {
        Self {
            repeat: TrackRepeat::Count(count),
            components,
        }
    }

    #[must_use]
    pub fn auto_fill_components(components: Vec<TrackComponent>) -> Self {
        Self {
            repeat: TrackRepeat::AutoFill,
            components,
        }
    }

    #[must_use]
    pub fn auto_fit_components(components: Vec<TrackComponent>) -> Self {
        Self {
            repeat: TrackRepeat::AutoFit,
            components,
        }
    }

    #[must_use]
    pub fn sizing_tracks(&self) -> Vec<TrackSizing> {
        track_sizing_components(&self.components)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TrackComponent {
    LineNames(Vec<String>),
    Track(TrackSizing),
    Repeat(TrackRepetition),
    Subgrid(SubgridTrack),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrack {
    pub name_components: Vec<SubgridLineNameComponent>,
}

impl SubgridTrack {
    #[must_use]
    pub fn new(line_names: Vec<Vec<String>>) -> Self {
        Self {
            name_components: line_names
                .into_iter()
                .map(SubgridLineNameComponent::LineNames)
                .collect(),
        }
    }

    #[must_use]
    pub fn line_names(&self) -> Vec<Vec<String>> {
        let mut line_names = Vec::new();
        for component in &self.name_components {
            match component {
                SubgridLineNameComponent::LineNames(names) => line_names.push(names.clone()),
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::Count(count),
                    line_name_sets,
                } => {
                    for _ in 0..*count {
                        line_names.extend(line_name_sets.iter().cloned());
                    }
                }
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::AutoFill,
                    line_name_sets,
                } => line_names.extend(line_name_sets.iter().cloned()),
            }
        }
        line_names
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubgridLineNameComponent {
    LineNames(Vec<String>),
    Repeat {
        count: SubgridLineNameRepeatCount,
        line_name_sets: Vec<Vec<String>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridLineNameRepeatCount {
    Count(usize),
    AutoFill,
}

impl TrackComponent {
    pub const AUTO: Self = Self::Track(TrackSizing::AUTO);
    pub const MIN_CONTENT: Self = Self::Track(TrackSizing::MIN_CONTENT);
    pub const MAX_CONTENT: Self = Self::Track(TrackSizing::MAX_CONTENT);
    pub const ZERO: Self = Self::Track(TrackSizing::ZERO);

    #[must_use]
    pub const fn px(value: Scalar) -> Self {
        Self::Track(TrackSizing::px(value))
    }

    #[must_use]
    pub const fn percent(value: Scalar) -> Self {
        Self::Track(TrackSizing::percent(value))
    }

    #[must_use]
    pub const fn fr(value: Scalar) -> Self {
        Self::Track(TrackSizing::fr(value))
    }

    #[must_use]
    pub const fn fit_content(limit: Length) -> Self {
        Self::Track(TrackSizing::fit_content(limit))
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizing, max: MaxTrackSizing) -> Self {
        Self::Track(TrackSizing::minmax(min, max))
    }

    #[must_use]
    pub fn line_names(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::LineNames(names.into_iter().map(Into::into).collect())
    }
}

impl From<TrackSizing> for TrackComponent {
    fn from(value: TrackSizing) -> Self {
        Self::Track(value)
    }
}

impl From<Dimension> for TrackComponent {
    fn from(value: Dimension) -> Self {
        Self::Track(value.into())
    }
}

impl From<Length> for TrackComponent {
    fn from(value: Length) -> Self {
        Self::Track(value.into())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GridTemplateAreas {
    pub rows: Vec<GridTemplateAreaRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridTemplateAreaRow {
    pub cells: Vec<Option<String>>,
}

#[must_use]
pub fn track_sizing_components(components: &[TrackComponent]) -> Vec<TrackSizing> {
    let mut tracks = Vec::new();
    for component in components {
        match component {
            TrackComponent::Track(track) => tracks.push(*track),
            TrackComponent::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                match repetition.repeat {
                    TrackRepeat::Count(count) => {
                        for _ in 0..count {
                            tracks.extend(repeated_tracks.iter().copied());
                        }
                    }
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                        tracks.extend(repeated_tracks);
                    }
                }
            }
            TrackComponent::LineNames(_) | TrackComponent::Subgrid(_) => {}
        }
    }
    tracks
}

fn track_sizing_components_from_tracks(tracks: Vec<TrackSizing>) -> Vec<TrackComponent> {
    tracks.into_iter().map(TrackComponent::Track).collect()
}
