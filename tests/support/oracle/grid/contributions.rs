//! Explicit grid item contribution facts and contribution arithmetic.

use super::placement::GridArea;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemContributionFacts {
    pub area: GridArea,
    pub min_content: f32,
    pub max_content: f32,
    pub preferred: ContributionSize,
    pub min_size: ContributionSize,
    pub max_size: ContributionSize,
    pub margin_before: f32,
    pub margin_after: f32,
    pub automatic_minimum_applies: bool,
}

impl ItemContributionFacts {
    #[must_use]
    pub fn contributions(self) -> ItemContributions {
        let margins = self.margin_before + self.margin_after;
        let lower = definite_or(self.min_size, 0.0);
        let upper = upper_limit(self.max_size, self.preferred);
        let minimum_inner = if matches!(self.min_size, ContributionSize::Definite(_)) {
            lower
        } else if self.automatic_minimum_applies {
            self.min_content
        } else {
            0.0
        };

        ItemContributions {
            minimum: clamp_to_limit(minimum_inner, lower, upper) + margins,
            min_content: self.min_content + margins,
            max_content: self.max_content + margins,
            limited_min_content: clamp_to_limit(self.min_content, lower, upper) + margins,
            limited_max_content: clamp_to_limit(self.max_content, lower, upper) + margins,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContributionSize {
    Auto,
    Definite(f32),
    Infinite,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemContributions {
    pub minimum: f32,
    pub min_content: f32,
    pub max_content: f32,
    pub limited_min_content: f32,
    pub limited_max_content: f32,
}

fn definite_or(size: ContributionSize, fallback: f32) -> f32 {
    match size {
        ContributionSize::Definite(value) => value,
        ContributionSize::Auto | ContributionSize::Infinite => fallback,
    }
}

fn upper_limit(max_size: ContributionSize, preferred: ContributionSize) -> Option<f32> {
    match max_size {
        ContributionSize::Definite(value) => Some(value),
        ContributionSize::Infinite => match preferred {
            ContributionSize::Definite(value) => Some(value),
            ContributionSize::Auto | ContributionSize::Infinite => None,
        },
        ContributionSize::Auto => match preferred {
            ContributionSize::Definite(value) => Some(value),
            ContributionSize::Auto | ContributionSize::Infinite => None,
        },
    }
}

fn clamp_to_limit(value: f32, lower: f32, upper: Option<f32>) -> f32 {
    let value = value.max(lower);
    upper.map_or(value, |upper| value.min(upper))
}
