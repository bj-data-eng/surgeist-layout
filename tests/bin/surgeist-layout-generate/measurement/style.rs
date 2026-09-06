//! Typed captured style values; semantic strings are kept intact for the fixture reader.
use super::{
    Attributes, Context, MeasurementError, MeasurementErrorKind, finite, nonnegative, number,
    present, validate_xml_text,
};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Style {
    #[serde(default, deserialize_with = "present")]
    pub display: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub box_sizing: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub position: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub direction: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub writing_mode: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub order: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub flex_item_collapse: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub css_float: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub clear: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub text_align: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub vertical_align: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub font_family: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub flex_direction: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub flex_wrap: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub overflow_x: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub overflow_y: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub overflow_clip_margin: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scrollbar_gutter: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_padding_top: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_padding_right: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_padding_bottom: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_padding_left: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_margin_top: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_margin_right: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_margin_bottom: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_margin_left: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_snap_type: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_snap_align: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scroll_snap_stop: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub align_items: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub align_self: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub justify_items: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub justify_self: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub align_content: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub justify_content: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub scrollbar_width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    pub flex_grow: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    pub flex_shrink: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    pub aspect_ratio: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    pub font_size: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    pub line_height: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    pub flex_basis: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    pub grid_template_rows: Option<Vec<Track>>,
    #[serde(default, deserialize_with = "present")]
    pub grid_template_columns: Option<Vec<Track>>,
    #[serde(default, deserialize_with = "present")]
    pub grid_auto_rows: Option<Vec<Track>>,
    #[serde(default, deserialize_with = "present")]
    pub grid_auto_columns: Option<Vec<Track>>,
    #[serde(default, deserialize_with = "present")]
    pub grid_row_start: Option<Position>,
    #[serde(default, deserialize_with = "present")]
    pub grid_row_end: Option<Position>,
    #[serde(default, deserialize_with = "present")]
    pub grid_column_start: Option<Position>,
    #[serde(default, deserialize_with = "present")]
    pub grid_column_end: Option<Position>,
    #[serde(default, deserialize_with = "present")]
    pub inline_metrics: Option<InlineMetrics>,
    #[serde(default, deserialize_with = "present")]
    pub gap: Option<Gap>,
    #[serde(default, deserialize_with = "present")]
    pub size: Option<Size>,
    #[serde(default, deserialize_with = "present")]
    pub min_size: Option<Size>,
    #[serde(default, deserialize_with = "present")]
    pub max_size: Option<Size>,
    #[serde(default, deserialize_with = "present")]
    pub margin: Option<Edges>,
    #[serde(default, deserialize_with = "present")]
    pub padding: Option<Edges>,
    #[serde(default, deserialize_with = "present")]
    pub border: Option<Edges>,
    #[serde(default, deserialize_with = "present")]
    pub inset: Option<Edges>,
    #[serde(default, deserialize_with = "present")]
    pub logical_margin: Option<LogicalMargin>,
    #[serde(default, deserialize_with = "present")]
    pub grid_auto_flow: Option<AutoFlow>,
    #[serde(default, deserialize_with = "present")]
    pub grid_template_areas: Option<Vec<Vec<Option<String>>>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Dimension {
    unit: Unit,
    #[serde(default, deserialize_with = "present")]
    value: Option<Scalar>,
}
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Unit {
    Auto,
    None,
    Content,
    MaxContent,
    MinContent,
    Stretch,
    FitContent,
    Contain,
    Px,
    Percent,
    Fraction,
    Calc,
    Sizing,
    AutoFill,
    AutoFit,
    Integer,
}

#[derive(Clone, Copy)]
enum DimensionUse {
    Length,
    LengthAuto,
    Preferred,
    Maximum,
    FlexBasis,
    TrackMinimum,
    TrackMaximum,
}

impl DimensionUse {
    fn accepts(self, unit: Unit) -> bool {
        if matches!(unit, Unit::Px | Unit::Percent | Unit::Calc | Unit::Sizing) {
            return true;
        }
        match self {
            Self::Length => false,
            Self::LengthAuto => unit == Unit::Auto,
            Self::Preferred | Self::FlexBasis => {
                matches!(
                    unit,
                    Unit::Auto
                        | Unit::MinContent
                        | Unit::MaxContent
                        | Unit::Stretch
                        | Unit::FitContent
                        | Unit::Contain
                ) || matches!(self, Self::FlexBasis) && unit == Unit::Content
            }
            Self::Maximum => matches!(
                unit,
                Unit::None
                    | Unit::MinContent
                    | Unit::MaxContent
                    | Unit::Stretch
                    | Unit::FitContent
                    | Unit::Contain
            ),
            Self::TrackMinimum => matches!(unit, Unit::Auto | Unit::MinContent | Unit::MaxContent),
            Self::TrackMaximum => matches!(
                unit,
                Unit::Auto | Unit::MinContent | Unit::MaxContent | Unit::Fraction
            ),
        }
    }
}

fn validate_dimension_use(
    unit: Unit,
    usage: DimensionUse,
    context: &Context<'_>,
    field: &str,
) -> Result<(), MeasurementError> {
    context.ensure(
        usage.accepts(unit),
        field,
        "dimension unit is not valid for this property",
    )
}

impl Dimension {
    pub fn validate_viewport_wire(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        self.validate(context, field)?;
        context.ensure(
            matches!(self.unit, Unit::Px | Unit::MinContent | Unit::MaxContent),
            field,
            "viewport dimension must be px, min-content, or max-content",
        )
    }

    pub fn validate_viewport_semantic(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        self.validate_nonnegative(context, field)
    }

    fn validate_xml_strings(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        if let Some(Scalar::Text(value)) = &self.value {
            validate_xml_text(value, context, &format!("{field}.value"))?;
        }
        Ok(())
    }

    fn validate_for(
        &self,
        usage: DimensionUse,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        self.validate(context, field)?;
        validate_dimension_use(self.unit, usage, context, field)
    }

    fn validate_nonnegative(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        if let Some(Scalar::Number(value)) = self.value {
            nonnegative(value, context, field)?;
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Scalar {
    Number(f64),
    Text(String),
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct InlineMetrics {
    baseline: f64,
    line_height: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Track {
    kind: TrackKind,
    #[serde(default, deserialize_with = "present")]
    unit: Option<Unit>,
    #[serde(default, deserialize_with = "present")]
    value: Option<Scalar>,
    #[serde(default, deserialize_with = "present")]
    name: Option<Function>,
    #[serde(default, deserialize_with = "present")]
    arguments: Option<Vec<Track>>,
    #[serde(default, deserialize_with = "present")]
    names: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present")]
    line_names: Option<Vec<Vec<String>>>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum TrackKind {
    Scalar,
    LineNames,
    Subgrid,
    Function,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Function {
    FitContent,
    Minmax,
    Repeat,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Position {
    kind: PositionKind,
    #[serde(default, deserialize_with = "present")]
    name: Option<String>,
    #[serde(default, deserialize_with = "present")]
    value: Option<i64>,
    #[serde(default, deserialize_with = "present")]
    occurrence: Option<i64>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum PositionKind {
    Auto,
    Span,
    Line,
    NamedLine,
    NamedSpan,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AutoFlow {
    direction: FlowDirection,
    algorithm: FlowAlgorithm,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FlowDirection {
    Row,
    Column,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FlowAlgorithm {
    Sparse,
    Dense,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Size {
    #[serde(default, deserialize_with = "present")]
    width: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    height: Option<Dimension>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Gap {
    #[serde(default, deserialize_with = "present")]
    row: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    column: Option<Dimension>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Edges {
    #[serde(default, deserialize_with = "present")]
    top: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    right: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    bottom: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    left: Option<Dimension>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LogicalMargin {
    #[serde(default, deserialize_with = "present")]
    inline_start: Option<Dimension>,
    #[serde(default, deserialize_with = "present")]
    inline_end: Option<Dimension>,
}

impl Dimension {
    pub fn validate(&self, context: &Context<'_>, field: &str) -> Result<String, MeasurementError> {
        dimension(self.unit, self.value.as_ref(), context, field, false)
    }
}
fn dimension(
    unit: Unit,
    value: Option<&Scalar>,
    context: &Context<'_>,
    field: &str,
    repetition: bool,
) -> Result<String, MeasurementError> {
    let invalid = || {
        context.error(
            field,
            MeasurementErrorKind::InvalidValue,
            "dimension unit and value do not form a legal alternative",
        )
    };
    match (unit, value) {
        (Unit::Auto, None) => Ok("auto".into()),
        (Unit::None, None) => Ok("none".into()),
        (Unit::Content, None) => Ok("content".into()),
        (Unit::MaxContent, None) => Ok("max-content".into()),
        (Unit::MinContent, None) => Ok("min-content".into()),
        (Unit::Stretch, None) => Ok("stretch".into()),
        (Unit::FitContent, None) => Ok("fit-content".into()),
        (Unit::Contain, None) => Ok("contain".into()),
        (Unit::Px, Some(Scalar::Number(value))) => {
            Ok(format!("{}px", number(finite(*value, context, field)?)))
        }
        (Unit::Percent, Some(Scalar::Number(value))) => Ok(format!(
            "{}%",
            number(finite(*value * 100.0, context, field)?)
        )),
        (Unit::Fraction, Some(Scalar::Number(value))) => Ok(format!(
            "{}fr",
            number(nonnegative(*value, context, field)?)
        )),
        (Unit::Calc | Unit::Sizing, Some(Scalar::Text(value))) if !value.trim().is_empty() => {
            Ok(value.clone())
        }
        (Unit::AutoFill, None) if repetition => Ok("auto-fill".into()),
        (Unit::AutoFit, None) if repetition => Ok("auto-fit".into()),
        (Unit::Integer, Some(Scalar::Number(value)))
            if repetition
                && value.fract() == 0.0
                && *value > 0.0
                && *value <= f64::from(u16::MAX) =>
        {
            Ok(number(*value))
        }
        _ => Err(invalid()),
    }
}
impl Track {
    fn validate(
        &self,
        context: &Context<'_>,
        field: &str,
        repetition: bool,
    ) -> Result<String, MeasurementError> {
        let invalid = || {
            context.error(
                field,
                MeasurementErrorKind::InvalidValue,
                "track fields do not form a legal alternative",
            )
        };
        match self.kind {
            TrackKind::Scalar => {
                if self.name.is_some()
                    || self.arguments.is_some()
                    || self.names.is_some()
                    || self.line_names.is_some()
                {
                    return Err(invalid());
                }
                let unit = self.unit.ok_or_else(invalid)?;
                if !repetition {
                    validate_dimension_use(unit, DimensionUse::TrackMaximum, context, field)?;
                }
                dimension(unit, self.value.as_ref(), context, field, repetition)
            }
            TrackKind::LineNames => {
                if self.unit.is_some()
                    || self.value.is_some()
                    || self.name.is_some()
                    || self.arguments.is_some()
                    || self.line_names.is_some()
                {
                    return Err(invalid());
                }
                let names = self.names.as_ref().ok_or_else(invalid)?;
                validate_names(names, context, field)?;
                Ok(format!("[{}]", names.join(" ")))
            }
            TrackKind::Subgrid => {
                if self.unit.is_some()
                    || self.value.is_some()
                    || self.name.is_some()
                    || self.arguments.is_some()
                    || self.names.is_some()
                {
                    return Err(invalid());
                }
                let mut result = "subgrid".to_string();
                for names in self.line_names.as_deref().ok_or_else(invalid)? {
                    validate_names(names, context, field)?;
                    result.push_str(&format!(" [{}]", names.join(" ")));
                }
                Ok(result)
            }
            TrackKind::Function => {
                if self.unit.is_some()
                    || self.value.is_some()
                    || self.names.is_some()
                    || self.line_names.is_some()
                {
                    return Err(invalid());
                }
                let args = self.arguments.as_ref().ok_or_else(invalid)?;
                match self.name.as_ref().ok_or_else(invalid)? {
                    Function::FitContent if args.len() == 1 => Ok(format!(
                        "fit-content({})",
                        args[0].scalar(
                            DimensionUse::Length,
                            context,
                            &format!("{field}.arguments[0]")
                        )?
                    )),
                    Function::Minmax if args.len() == 2 => Ok(format!(
                        "minmax({},{})",
                        args[0].scalar(
                            DimensionUse::TrackMinimum,
                            context,
                            &format!("{field}.arguments[0]")
                        )?,
                        args[1].scalar(
                            DimensionUse::TrackMaximum,
                            context,
                            &format!("{field}.arguments[1]")
                        )?
                    )),
                    Function::Repeat if args.len() >= 2 => {
                        let repetition =
                            args[0].validate(context, &format!("{field}.arguments[0]"), true)?;
                        context.ensure(
                            matches!(
                                args[0].unit,
                                Some(Unit::Integer | Unit::AutoFill | Unit::AutoFit)
                            ),
                            field,
                            "repeat requires a repetition count",
                        )?;
                        let tracks = args
                            .iter()
                            .enumerate()
                            .skip(1)
                            .map(|(index, track)| {
                                track.validate(
                                    context,
                                    &format!("{field}.arguments[{index}]"),
                                    false,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(format!("repeat({repetition}, {})", tracks.join(" ")))
                    }
                    _ => Err(context.error(
                        format!("{field}.arguments"),
                        MeasurementErrorKind::InvalidValue,
                        "invalid track function arity",
                    )),
                }
            }
        }
    }
    fn scalar(
        &self,
        usage: DimensionUse,
        context: &Context<'_>,
        field: &str,
    ) -> Result<String, MeasurementError> {
        context.ensure(
            self.kind == TrackKind::Scalar,
            field,
            "track function requires a scalar argument",
        )?;
        let value = self.validate(context, field, false)?;
        let unit = context.required(self.unit, field)?;
        validate_dimension_use(unit, usage, context, field)?;
        Ok(value)
    }
}
fn validate_names(
    names: &[String],
    context: &Context<'_>,
    field: &str,
) -> Result<(), MeasurementError> {
    context.ensure(
        names
            .iter()
            .all(|name| !name.is_empty() && !name.chars().any(char::is_whitespace)),
        field,
        "grid names must be nonempty tokens",
    )
}

fn validate_alignment(
    value: &str,
    content: bool,
    context: &Context<'_>,
    field: &str,
) -> Result<(), MeasurementError> {
    let prefixed = value
        .strip_prefix("safe ")
        .or_else(|| value.strip_prefix("unsafe "));
    let keyword = prefixed.unwrap_or(value);
    let valid = matches!(
        keyword,
        "start" | "end" | "flex-start" | "flex-end" | "center" | "stretch"
    ) || content
        && matches!(keyword, "space-between" | "space-around" | "space-evenly")
        || !content
            && prefixed.is_none()
            && matches!(keyword, "baseline" | "first baseline" | "last baseline");
    context.ensure(valid, field, "unknown alignment discriminant")
}

fn valid_area_name(name: &str) -> bool {
    if matches!(
        name,
        "auto"
            | "default"
            | "inherit"
            | "initial"
            | "none"
            | "revert"
            | "revert-layer"
            | "span"
            | "unset"
    ) {
        return false;
    }
    let mut bytes = name.strip_prefix('-').unwrap_or(name).bytes();
    let start = match bytes.next() {
        Some(first) if first.is_ascii_alphabetic() || first == b'_' => true,
        Some(b'-') => bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_'),
        _ => false,
    };
    start
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn validate_areas(
    rows: &[Vec<Option<String>>],
    context: &Context<'_>,
) -> Result<(), MeasurementError> {
    let width = rows.first().map_or(0, Vec::len);
    context.ensure(
        width > 0,
        "style.gridTemplateAreas",
        "grid area table must not be empty",
    )?;
    let mut bounds = std::collections::BTreeMap::<&str, (usize, usize, usize, usize, usize)>::new();
    for (row_index, row) in rows.iter().enumerate() {
        context.ensure(
            row.len() == width,
            &format!("style.gridTemplateAreas[{row_index}]"),
            "grid area rows must have equal widths",
        )?;
        for (column_index, name) in row.iter().enumerate() {
            let Some(name) = name else {
                continue;
            };
            context.ensure(
                valid_area_name(name),
                &format!("style.gridTemplateAreas[{row_index}][{column_index}]"),
                "grid area cell must be an identifier or null",
            )?;
            let entry =
                bounds
                    .entry(name)
                    .or_insert((row_index, row_index, column_index, column_index, 0));
            entry.0 = entry.0.min(row_index);
            entry.1 = entry.1.max(row_index);
            entry.2 = entry.2.min(column_index);
            entry.3 = entry.3.max(column_index);
            entry.4 += 1;
        }
    }
    for (name, (first_row, last_row, first_column, last_column, count)) in bounds {
        let area = (last_row - first_row + 1).checked_mul(last_column - first_column + 1);
        context.ensure(
            area == Some(count),
            "style.gridTemplateAreas",
            &format!("grid area `{name}` must form one rectangle"),
        )?;
    }
    Ok(())
}

impl Style {
    fn validate_xml_strings(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        for (field, value) in [
            ("display", &self.display),
            ("boxSizing", &self.box_sizing),
            ("position", &self.position),
            ("direction", &self.direction),
            ("writingMode", &self.writing_mode),
            ("order", &self.order),
            ("flexItemCollapse", &self.flex_item_collapse),
            ("cssFloat", &self.css_float),
            ("clear", &self.clear),
            ("textAlign", &self.text_align),
            ("verticalAlign", &self.vertical_align),
            ("fontFamily", &self.font_family),
            ("flexDirection", &self.flex_direction),
            ("flexWrap", &self.flex_wrap),
            ("overflowX", &self.overflow_x),
            ("overflowY", &self.overflow_y),
            ("overflowClipMargin", &self.overflow_clip_margin),
            ("scrollbarGutter", &self.scrollbar_gutter),
            ("scrollPaddingTop", &self.scroll_padding_top),
            ("scrollPaddingRight", &self.scroll_padding_right),
            ("scrollPaddingBottom", &self.scroll_padding_bottom),
            ("scrollPaddingLeft", &self.scroll_padding_left),
            ("scrollMarginTop", &self.scroll_margin_top),
            ("scrollMarginRight", &self.scroll_margin_right),
            ("scrollMarginBottom", &self.scroll_margin_bottom),
            ("scrollMarginLeft", &self.scroll_margin_left),
            ("scrollSnapType", &self.scroll_snap_type),
            ("scrollSnapAlign", &self.scroll_snap_align),
            ("scrollSnapStop", &self.scroll_snap_stop),
            ("alignItems", &self.align_items),
            ("alignSelf", &self.align_self),
            ("justifyItems", &self.justify_items),
            ("justifySelf", &self.justify_self),
            ("alignContent", &self.align_content),
            ("justifyContent", &self.justify_content),
        ] {
            if let Some(value) = value {
                validate_xml_text(value, context, &format!("style.{field}"))?;
            }
        }
        for (field, value) in [
            ("fontSize", &self.font_size),
            ("lineHeight", &self.line_height),
            ("flexBasis", &self.flex_basis),
        ] {
            if let Some(value) = value {
                value.validate_xml_strings(context, &format!("style.{field}"))?;
            }
        }
        for (field, size) in [
            ("size", &self.size),
            ("minSize", &self.min_size),
            ("maxSize", &self.max_size),
        ] {
            if let Some(size) = size {
                for (axis, value) in [("width", &size.width), ("height", &size.height)] {
                    if let Some(value) = value {
                        value.validate_xml_strings(context, &format!("style.{field}.{axis}"))?;
                    }
                }
            }
        }
        for (field, edges) in [
            ("margin", &self.margin),
            ("padding", &self.padding),
            ("border", &self.border),
            ("inset", &self.inset),
        ] {
            if let Some(edges) = edges {
                for (edge, value) in [
                    ("top", &edges.top),
                    ("right", &edges.right),
                    ("bottom", &edges.bottom),
                    ("left", &edges.left),
                ] {
                    if let Some(value) = value {
                        value.validate_xml_strings(context, &format!("style.{field}.{edge}"))?;
                    }
                }
            }
        }
        if let Some(gap) = &self.gap {
            for (field, value) in [("row", &gap.row), ("column", &gap.column)] {
                if let Some(value) = value {
                    value.validate_xml_strings(context, &format!("style.gap.{field}"))?;
                }
            }
        }
        if let Some(margin) = &self.logical_margin {
            for (field, value) in [
                ("inlineStart", &margin.inline_start),
                ("inlineEnd", &margin.inline_end),
            ] {
                if let Some(value) = value {
                    value.validate_xml_strings(context, &format!("style.logicalMargin.{field}"))?;
                }
            }
        }
        for (field, tracks) in [
            ("gridTemplateRows", &self.grid_template_rows),
            ("gridTemplateColumns", &self.grid_template_columns),
            ("gridAutoRows", &self.grid_auto_rows),
            ("gridAutoColumns", &self.grid_auto_columns),
        ] {
            if let Some(tracks) = tracks {
                for (index, track) in tracks.iter().enumerate() {
                    track.validate_xml_strings(context, &format!("style.{field}[{index}]"))?;
                }
            }
        }
        for (field, position) in [
            ("gridRowStart", &self.grid_row_start),
            ("gridRowEnd", &self.grid_row_end),
            ("gridColumnStart", &self.grid_column_start),
            ("gridColumnEnd", &self.grid_column_end),
        ] {
            if let Some(position) = position
                && let Some(name) = &position.name
            {
                validate_xml_text(name, context, &format!("style.{field}.name"))?;
            }
        }
        // Grid-area names already have a closed ASCII identifier grammar.
        Ok(())
    }

    fn validate_nonnegative_extents(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        for (field, value) in [
            ("fontSize", &self.font_size),
            ("lineHeight", &self.line_height),
            ("flexBasis", &self.flex_basis),
        ] {
            if let Some(value) = value {
                value.validate_nonnegative(context, &format!("style.{field}"))?;
            }
        }
        for (field, size) in [
            ("size", &self.size),
            ("minSize", &self.min_size),
            ("maxSize", &self.max_size),
        ] {
            if let Some(size) = size {
                for (axis, value) in [("width", &size.width), ("height", &size.height)] {
                    if let Some(value) = value {
                        value.validate_nonnegative(context, &format!("style.{field}.{axis}"))?;
                    }
                }
            }
        }
        for (field, edges) in [("padding", &self.padding), ("border", &self.border)] {
            if let Some(edges) = edges {
                for (edge, value) in [
                    ("top", &edges.top),
                    ("right", &edges.right),
                    ("bottom", &edges.bottom),
                    ("left", &edges.left),
                ] {
                    if let Some(value) = value {
                        value.validate_nonnegative(context, &format!("style.{field}.{edge}"))?;
                    }
                }
            }
        }
        if let Some(gap) = &self.gap {
            for (field, value) in [("row", &gap.row), ("column", &gap.column)] {
                if let Some(value) = value {
                    value.validate_nonnegative(context, &format!("style.gap.{field}"))?;
                }
            }
        }
        for (field, tracks) in [
            ("gridTemplateRows", &self.grid_template_rows),
            ("gridTemplateColumns", &self.grid_template_columns),
            ("gridAutoRows", &self.grid_auto_rows),
            ("gridAutoColumns", &self.grid_auto_columns),
        ] {
            if let Some(tracks) = tracks {
                for (index, track) in tracks.iter().enumerate() {
                    track.validate_nonnegative(context, &format!("style.{field}[{index}]"))?;
                }
            }
        }
        Ok(())
    }
}

impl Track {
    fn validate_xml_strings(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        if let Some(Scalar::Text(value)) = &self.value {
            validate_xml_text(value, context, &format!("{field}.value"))?;
        }
        if let Some(names) = &self.names {
            for (index, name) in names.iter().enumerate() {
                validate_xml_text(name, context, &format!("{field}.names[{index}]"))?;
            }
        }
        if let Some(lines) = &self.line_names {
            for (line, names) in lines.iter().enumerate() {
                for (index, name) in names.iter().enumerate() {
                    validate_xml_text(
                        name,
                        context,
                        &format!("{field}.lineNames[{line}][{index}]"),
                    )?;
                }
            }
        }
        if let Some(arguments) = &self.arguments {
            for (index, value) in arguments.iter().enumerate() {
                value.validate_xml_strings(context, &format!("{field}.arguments[{index}]"))?;
            }
        }
        Ok(())
    }

    fn validate_nonnegative(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<(), MeasurementError> {
        if let Some(Scalar::Number(value)) = self.value {
            nonnegative(value, context, field)?;
        }
        if let Some(arguments) = &self.arguments {
            for (index, value) in arguments.iter().enumerate() {
                value.validate_nonnegative(context, &format!("{field}.arguments[{index}]"))?;
            }
        }
        Ok(())
    }
}
impl Position {
    fn validate(
        &self,
        context: &Context<'_>,
        field: &str,
    ) -> Result<Option<String>, MeasurementError> {
        let invalid = || {
            context.error(
                field,
                MeasurementErrorKind::InvalidValue,
                "placement fields do not form a legal alternative",
            )
        };
        match self.kind {
            PositionKind::Auto
                if self.name.is_none() && self.value.is_none() && self.occurrence.is_none() =>
            {
                Ok(None)
            }
            PositionKind::Line if self.name.is_none() && self.occurrence.is_none() => {
                let value = self.value.ok_or_else(invalid)?;
                context.ensure(
                    value != 0 && i16::try_from(value).is_ok(),
                    field,
                    "grid line must be a nonzero i16",
                )?;
                Ok(Some(value.to_string()))
            }
            PositionKind::Span if self.name.is_none() && self.occurrence.is_none() => {
                let value = self.value.ok_or_else(invalid)?;
                context.ensure(
                    value > 0 && u16::try_from(value).is_ok(),
                    field,
                    "grid span must be a positive u16",
                )?;
                Ok(Some(format!("span {value}")))
            }
            PositionKind::NamedLine | PositionKind::NamedSpan => {
                let name = self.name.as_ref().ok_or_else(invalid)?;
                validate_names(std::slice::from_ref(name), context, field)?;
                context.ensure(
                    self.value.is_none(),
                    field,
                    "named placement cannot carry numeric value",
                )?;
                match (&self.kind, self.occurrence) {
                    (PositionKind::NamedLine, None) => Ok(Some(name.clone())),
                    (PositionKind::NamedSpan, None) => Ok(Some(format!("span {name}"))),
                    (PositionKind::NamedLine, Some(value)) => {
                        context.ensure(
                            value != 0 && i16::try_from(value).is_ok(),
                            field,
                            "named grid line occurrence must be a nonzero i16",
                        )?;
                        Ok(Some(format!("{name} {value}")))
                    }
                    (PositionKind::NamedSpan, Some(value)) => {
                        context.ensure(
                            value > 0 && u16::try_from(value).is_ok(),
                            field,
                            "named grid span occurrence must be a positive u16",
                        )?;
                        Ok(Some(format!("span {value} {name}")))
                    }
                    _ => unreachable!("matched named placement kinds"),
                }
            }
            _ => Err(invalid()),
        }
    }
}
fn add(attrs: &mut Attributes, key: &'static str, value: Option<String>, elide: Option<&str>) {
    if let Some(value) = value
        && elide != Some(value.as_str())
    {
        attrs.push((key, value));
    }
}
fn optional_dimension(
    value: Option<&Dimension>,
    context: &Context<'_>,
    field: &str,
) -> Result<Option<String>, MeasurementError> {
    value
        .map(|value| value.validate(context, field))
        .transpose()
}
pub(super) fn validate_writing_mode(
    value: &str,
    context: &Context<'_>,
    field: &str,
) -> Result<(), MeasurementError> {
    context.ensure(
        matches!(
            value,
            "horizontal-tb" | "vertical-rl" | "vertical-lr" | "sideways-rl" | "sideways-lr"
        ),
        field,
        "unknown writing mode",
    )
}

impl Style {
    pub fn writing_mode(&self) -> &str {
        self.writing_mode.as_deref().unwrap_or("horizontal-tb")
    }
    pub fn is_grid(&self) -> bool {
        matches!(
            self.display.as_deref(),
            Some("grid" | "inline-grid" | "grid-lanes" | "inline-grid-lanes")
        )
    }
    pub fn has_scroll_overflow(&self) -> bool {
        [self.overflow_x.as_deref(), self.overflow_y.as_deref()]
            .into_iter()
            .any(|value| matches!(value, Some("hidden" | "scroll" | "auto")))
    }
    pub fn has_scroll_fields(&self) -> bool {
        self.overflow_x.is_some()
            || self.overflow_y.is_some()
            || self.overflow_clip_margin.is_some()
            || self.scrollbar_width.is_some()
            || self.scrollbar_gutter.is_some()
            || self.scroll_padding_top.is_some()
            || self.scroll_padding_right.is_some()
            || self.scroll_padding_bottom.is_some()
            || self.scroll_padding_left.is_some()
            || self.scroll_margin_top.is_some()
            || self.scroll_margin_right.is_some()
            || self.scroll_margin_bottom.is_some()
            || self.scroll_margin_left.is_some()
            || self.scroll_snap_type.is_some()
            || self.scroll_snap_align.is_some()
            || self.scroll_snap_stop.is_some()
    }
    pub fn validate_wire(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        for (field, value, choices) in [
            (
                "display",
                self.display.as_deref(),
                &[
                    "none",
                    "block",
                    "inline",
                    "inline-block",
                    "flex",
                    "inline-flex",
                    "grid",
                    "inline-grid",
                    "grid-lanes",
                    "inline-grid-lanes",
                    "contents",
                    "flow-root",
                    "table",
                    "table-row",
                    "table-cell",
                    "list-item",
                ][..],
            ),
            ("direction", self.direction.as_deref(), &["ltr", "rtl"][..]),
            (
                "boxSizing",
                self.box_sizing.as_deref(),
                &["border-box", "content-box"][..],
            ),
            (
                "flexDirection",
                self.flex_direction.as_deref(),
                &["row", "row-reverse", "column", "column-reverse"][..],
            ),
            (
                "flexWrap",
                self.flex_wrap.as_deref(),
                &["nowrap", "wrap", "wrap-reverse"][..],
            ),
            (
                "overflowX",
                self.overflow_x.as_deref(),
                &["visible", "hidden", "clip", "scroll", "auto"][..],
            ),
            (
                "overflowY",
                self.overflow_y.as_deref(),
                &["visible", "hidden", "clip", "scroll", "auto"][..],
            ),
            (
                "position",
                self.position.as_deref(),
                &["static", "relative", "absolute", "fixed", "sticky"][..],
            ),
            (
                "flexItemCollapse",
                self.flex_item_collapse.as_deref(),
                &["collapsed"][..],
            ),
            (
                "cssFloat",
                self.css_float.as_deref(),
                &["none", "left", "right", "inline-start", "inline-end"][..],
            ),
            (
                "clear",
                self.clear.as_deref(),
                &[
                    "none",
                    "left",
                    "right",
                    "inline-start",
                    "inline-end",
                    "both",
                ][..],
            ),
            (
                "textAlign",
                self.text_align.as_deref(),
                &[
                    "left",
                    "right",
                    "center",
                    "-webkit-left",
                    "-webkit-right",
                    "-webkit-center",
                ][..],
            ),
            (
                "verticalAlign",
                self.vertical_align.as_deref(),
                &["baseline", "top", "bottom"][..],
            ),
        ] {
            if let Some(value) = value {
                context.ensure(
                    choices.contains(&value),
                    &format!("style.{field}"),
                    "unknown style discriminant",
                )?;
            }
        }
        if let Some(mode) = &self.writing_mode {
            validate_writing_mode(mode, context, "style.writingMode")?;
        }
        for (field, value, content) in [
            ("alignItems", &self.align_items, false),
            ("alignSelf", &self.align_self, false),
            ("justifyItems", &self.justify_items, false),
            ("justifySelf", &self.justify_self, false),
            ("alignContent", &self.align_content, true),
            ("justifyContent", &self.justify_content, true),
        ] {
            if let Some(value) = value {
                validate_alignment(value, content, context, &format!("style.{field}"))?;
            }
        }
        // Wire alternatives must be meaningful even for an unsupported node;
        // coupled measurement metrics are validated only for supported cases.
        for (field, value, usage) in [
            ("fontSize", &self.font_size, DimensionUse::Length),
            ("lineHeight", &self.line_height, DimensionUse::Length),
            ("flexBasis", &self.flex_basis, DimensionUse::FlexBasis),
        ] {
            if let Some(value) = value {
                value.validate_for(usage, context, &format!("style.{field}"))?;
            }
        }
        for (field, size, usage) in [
            ("size", &self.size, DimensionUse::Preferred),
            ("minSize", &self.min_size, DimensionUse::Preferred),
            ("maxSize", &self.max_size, DimensionUse::Maximum),
        ] {
            if let Some(size) = size {
                for (axis, value) in [("width", &size.width), ("height", &size.height)] {
                    if let Some(value) = value {
                        value.validate_for(usage, context, &format!("style.{field}.{axis}"))?;
                    }
                }
            }
        }
        for (field, edges, usage) in [
            ("margin", &self.margin, DimensionUse::LengthAuto),
            ("padding", &self.padding, DimensionUse::Length),
            ("border", &self.border, DimensionUse::Length),
            ("inset", &self.inset, DimensionUse::LengthAuto),
        ] {
            if let Some(edges) = edges {
                for (edge, value) in [
                    ("top", &edges.top),
                    ("right", &edges.right),
                    ("bottom", &edges.bottom),
                    ("left", &edges.left),
                ] {
                    if let Some(value) = value {
                        value.validate_for(usage, context, &format!("style.{field}.{edge}"))?;
                    }
                }
            }
        }
        if let Some(gap) = &self.gap {
            for (field, value) in [("row", &gap.row), ("column", &gap.column)] {
                if let Some(value) = value {
                    value.validate_for(
                        DimensionUse::Length,
                        context,
                        &format!("style.gap.{field}"),
                    )?;
                }
            }
        }
        if let Some(margin) = &self.logical_margin {
            for (field, value) in [
                ("inlineStart", &margin.inline_start),
                ("inlineEnd", &margin.inline_end),
            ] {
                if let Some(value) = value {
                    value.validate_for(
                        DimensionUse::LengthAuto,
                        context,
                        &format!("style.logicalMargin.{field}"),
                    )?;
                }
            }
        }
        for (field, tracks) in [
            ("gridTemplateRows", &self.grid_template_rows),
            ("gridTemplateColumns", &self.grid_template_columns),
            ("gridAutoRows", &self.grid_auto_rows),
            ("gridAutoColumns", &self.grid_auto_columns),
        ] {
            if let Some(tracks) = tracks {
                for (index, track) in tracks.iter().enumerate() {
                    track.validate(context, &format!("style.{field}[{index}]"), false)?;
                }
            }
        }
        for (field, position) in [
            ("gridRowStart", &self.grid_row_start),
            ("gridRowEnd", &self.grid_row_end),
            ("gridColumnStart", &self.grid_column_start),
            ("gridColumnEnd", &self.grid_column_end),
        ] {
            if let Some(position) = position {
                position.validate(context, &format!("style.{field}"))?;
            }
        }
        if let Some(rows) = &self.grid_template_areas {
            validate_areas(rows, context)?;
        }
        Ok(())
    }
    pub fn attrs(
        &self,
        context: &Context<'_>,
        parent_mode: &str,
        shape: bool,
        inline_metrics: bool,
    ) -> Result<Attributes, MeasurementError> {
        self.validate_xml_strings(context)?;
        self.validate_nonnegative_extents(context)?;
        let mut attrs = Vec::new();
        for (key, value, elide) in [
            ("display", &self.display, None),
            ("box-sizing", &self.box_sizing, Some("border-box")),
            ("direction", &self.direction, None),
            ("order", &self.order, Some("0")),
            ("flex-item-collapse", &self.flex_item_collapse, None),
        ] {
            add(&mut attrs, key, value.clone(), elide);
        }
        if self.writing_mode() != "horizontal-tb" || parent_mode != "horizontal-tb" {
            attrs.push(("writing-mode", self.writing_mode().into()));
        }
        add(
            &mut attrs,
            "position",
            self.position.clone(),
            Some("relative"),
        );
        add(&mut attrs, "float", self.css_float.clone(), None);
        if shape {
            attrs.push(("float-exclusion", "shape".into()));
        }
        add(&mut attrs, "clear", self.clear.clone(), None);
        add(
            &mut attrs,
            "flex-direction",
            self.flex_direction.clone(),
            Some("row"),
        );
        add(
            &mut attrs,
            "flex-wrap",
            self.flex_wrap.clone(),
            Some("nowrap"),
        );
        if [self.overflow_x.as_deref(), self.overflow_y.as_deref()]
            .into_iter()
            .flatten()
            .any(|value| value != "visible")
        {
            attrs.push((
                "overflow-x",
                self.overflow_x.clone().unwrap_or_else(|| "visible".into()),
            ));
            attrs.push((
                "overflow-y",
                self.overflow_y.clone().unwrap_or_else(|| "visible".into()),
            ));
            if let Some(value) = self.scrollbar_width {
                attrs.push((
                    "scrollbar-width",
                    number(nonnegative(value, context, "style.scrollbarWidth")?),
                ));
            }
        }
        for (key, value, initial) in [
            ("overflow-clip-margin", &self.overflow_clip_margin, "0px"),
            ("scrollbar-gutter", &self.scrollbar_gutter, "auto"),
            ("scroll-padding-top", &self.scroll_padding_top, "auto"),
            ("scroll-padding-right", &self.scroll_padding_right, "auto"),
            ("scroll-padding-bottom", &self.scroll_padding_bottom, "auto"),
            ("scroll-padding-left", &self.scroll_padding_left, "auto"),
            ("scroll-margin-top", &self.scroll_margin_top, "0px"),
            ("scroll-margin-right", &self.scroll_margin_right, "0px"),
            ("scroll-margin-bottom", &self.scroll_margin_bottom, "0px"),
            ("scroll-margin-left", &self.scroll_margin_left, "0px"),
            ("scroll-snap-type", &self.scroll_snap_type, "none"),
            ("scroll-snap-align", &self.scroll_snap_align, "none"),
            ("scroll-snap-stop", &self.scroll_snap_stop, "normal"),
        ] {
            add(&mut attrs, key, value.clone(), Some(initial));
        }
        add(&mut attrs, "text-align", self.text_align.clone(), None);
        add(
            &mut attrs,
            "vertical-align",
            self.vertical_align.clone(),
            Some("baseline"),
        );
        if let Some(family) = &self.font_family {
            let family = family.replace('"', "");
            let primary = family
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            if matches!(primary.as_str(), "ahem" | "monospace") {
                add(&mut attrs, "font-family", Some(primary), Some("ahem"));
            }
        }
        add(
            &mut attrs,
            "font-size",
            optional_dimension(self.font_size.as_ref(), context, "style.fontSize")?,
            Some("10px"),
        );
        add(
            &mut attrs,
            "line-height",
            optional_dimension(self.line_height.as_ref(), context, "style.lineHeight")?,
            Some("10px"),
        );
        if let Some(metrics) = &self.inline_metrics {
            nonnegative(metrics.baseline, context, "style.inlineMetrics.baseline")?;
            nonnegative(
                metrics.line_height,
                context,
                "style.inlineMetrics.lineHeight",
            )?;
            context.ensure(
                metrics.baseline <= metrics.line_height,
                "style.inlineMetrics.lineHeight",
                "line height must cover baseline",
            )?;
            if inline_metrics {
                attrs.extend([
                    ("inline-baseline", format!("{}px", number(metrics.baseline))),
                    (
                        "inline-line-height",
                        format!("{}px", number(metrics.line_height)),
                    ),
                ]);
            }
        }
        for (key, value) in [
            ("align-items", &self.align_items),
            ("align-self", &self.align_self),
            ("justify-items", &self.justify_items),
            ("justify-self", &self.justify_self),
            ("align-content", &self.align_content),
            ("justify-content", &self.justify_content),
        ] {
            add(&mut attrs, key, value.clone(), None);
        }
        for (key, field, value, initial) in [
            ("flex-grow", "flexGrow", self.flex_grow, "0"),
            ("flex-shrink", "flexShrink", self.flex_shrink, "1"),
        ] {
            if let Some(value) = value {
                add(
                    &mut attrs,
                    key,
                    Some(number(nonnegative(
                        value,
                        context,
                        &format!("style.{field}"),
                    )?)),
                    Some(initial),
                );
            }
        }
        add(
            &mut attrs,
            "flex-basis",
            optional_dimension(self.flex_basis.as_ref(), context, "style.flexBasis")?,
            Some("auto"),
        );
        for (key, field, size) in [
            ("", "size", &self.size),
            ("min-", "minSize", &self.min_size),
            ("max-", "maxSize", &self.max_size),
        ] {
            if let Some(size) = size {
                for (dimension, value) in [("width", &size.width), ("height", &size.height)] {
                    let key = match (key, dimension) {
                        ("", "width") => "width",
                        ("", "height") => "height",
                        ("min-", "width") => "min-width",
                        ("min-", "height") => "min-height",
                        ("max-", "width") => "max-width",
                        _ => "max-height",
                    };
                    add(
                        &mut attrs,
                        key,
                        optional_dimension(
                            value.as_ref(),
                            context,
                            &format!("style.{field}.{dimension}"),
                        )?,
                        Some("auto"),
                    );
                }
            }
        }
        if let Some(ratio) = self.aspect_ratio {
            context.ensure(
                ratio > 0.0,
                "style.aspectRatio",
                "aspect ratio must be positive",
            )?;
            attrs.push((
                "aspect-ratio",
                number(finite(ratio, context, "style.aspectRatio")?),
            ));
        }
        if let Some(gap) = &self.gap {
            add(
                &mut attrs,
                "row-gap",
                optional_dimension(gap.row.as_ref(), context, "style.gap.row")?,
                None,
            );
            add(
                &mut attrs,
                "column-gap",
                optional_dimension(gap.column.as_ref(), context, "style.gap.column")?,
                None,
            );
        }
        self.edge_attrs(&mut attrs, "margin", self.margin.as_ref(), context)?;
        if let Some(logical) = &self.logical_margin {
            let rtl = self.direction.as_deref() == Some("rtl");
            let (start, end) = match (self.writing_mode(), rtl) {
                ("vertical-rl" | "vertical-lr" | "sideways-rl", false) => {
                    ("margin-top", "margin-bottom")
                }
                ("vertical-rl" | "vertical-lr" | "sideways-rl", true) => {
                    ("margin-bottom", "margin-top")
                }
                ("sideways-lr", false) => ("margin-bottom", "margin-top"),
                ("sideways-lr", true) => ("margin-top", "margin-bottom"),
                (_, false) => ("margin-left", "margin-right"),
                (_, true) => ("margin-right", "margin-left"),
            };
            for (key, value, field) in [
                (start, &logical.inline_start, "inlineStart"),
                (end, &logical.inline_end, "inlineEnd"),
            ] {
                if let Some(value) = optional_dimension(
                    value.as_ref(),
                    context,
                    &format!("style.logicalMargin.{field}"),
                )? {
                    if let Some((_, existing)) = attrs.iter_mut().find(|(attr, _)| *attr == key) {
                        *existing = value;
                    } else {
                        attrs.push((key, value));
                    }
                }
            }
        }
        self.edge_attrs(&mut attrs, "padding", self.padding.as_ref(), context)?;
        self.edge_attrs(&mut attrs, "border", self.border.as_ref(), context)?;
        self.edge_attrs(&mut attrs, "", self.inset.as_ref(), context)?;
        if let Some(flow) = &self.grid_auto_flow {
            let direction = match flow.direction {
                FlowDirection::Row => "row",
                FlowDirection::Column => "column",
            };
            attrs.push((
                "grid-auto-flow",
                match flow.algorithm {
                    FlowAlgorithm::Sparse => direction.into(),
                    FlowAlgorithm::Dense => format!("{direction} dense"),
                },
            ));
        }
        self.track_attr(
            &mut attrs,
            "grid-template-rows",
            "gridTemplateRows",
            self.grid_template_rows.as_deref(),
            context,
        )?;
        self.track_attr(
            &mut attrs,
            "grid-template-columns",
            "gridTemplateColumns",
            self.grid_template_columns.as_deref(),
            context,
        )?;
        if let Some(rows) = &self.grid_template_areas {
            let mut values = Vec::new();
            for (index, row) in rows.iter().enumerate() {
                context.ensure(
                    !row.is_empty(),
                    &format!("style.gridTemplateAreas[{index}]"),
                    "grid area row must not be empty",
                )?;
                let values = row
                    .iter()
                    .map(|cell| cell.as_deref().unwrap_or("."))
                    .collect::<Vec<_>>();
                context.ensure(
                    values
                        .iter()
                        .all(|value| !value.is_empty() && !value.chars().any(char::is_whitespace)),
                    "style.gridTemplateAreas",
                    "grid area cells must be names or null",
                )?;
            }
            for row in rows {
                values.push(
                    row.iter()
                        .map(|cell| cell.as_deref().unwrap_or("."))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
            if !values.is_empty() {
                attrs.push(("grid-template-areas", values.join(" / ")));
            }
        }
        self.track_attr(
            &mut attrs,
            "grid-auto-rows",
            "gridAutoRows",
            self.grid_auto_rows.as_deref(),
            context,
        )?;
        self.track_attr(
            &mut attrs,
            "grid-auto-columns",
            "gridAutoColumns",
            self.grid_auto_columns.as_deref(),
            context,
        )?;
        for (key, field, value) in [
            ("grid-row-start", "gridRowStart", &self.grid_row_start),
            ("grid-row-end", "gridRowEnd", &self.grid_row_end),
            (
                "grid-column-start",
                "gridColumnStart",
                &self.grid_column_start,
            ),
            ("grid-column-end", "gridColumnEnd", &self.grid_column_end),
        ] {
            if let Some(value) = value {
                add(
                    &mut attrs,
                    key,
                    value.validate(context, &format!("style.{field}"))?,
                    None,
                );
            }
        }
        Ok(attrs)
    }
    fn edge_attrs(
        &self,
        attrs: &mut Attributes,
        prefix: &str,
        edges: Option<&Edges>,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        if let Some(edges) = edges {
            for (name, value) in [
                ("top", &edges.top),
                ("left", &edges.left),
                ("bottom", &edges.bottom),
                ("right", &edges.right),
            ] {
                let key = match (prefix, name) {
                    ("margin", "top") => "margin-top",
                    ("margin", "right") => "margin-right",
                    ("margin", "bottom") => "margin-bottom",
                    ("margin", "left") => "margin-left",
                    ("padding", "top") => "padding-top",
                    ("padding", "right") => "padding-right",
                    ("padding", "bottom") => "padding-bottom",
                    ("padding", "left") => "padding-left",
                    ("border", "top") => "border-top",
                    ("border", "right") => "border-right",
                    ("border", "bottom") => "border-bottom",
                    ("border", "left") => "border-left",
                    ("", "top") => "top",
                    ("", "right") => "right",
                    ("", "bottom") => "bottom",
                    _ => "left",
                };
                add(
                    attrs,
                    key,
                    optional_dimension(
                        value.as_ref(),
                        context,
                        &format!(
                            "style.{}.{name}",
                            if prefix.is_empty() { "inset" } else { prefix }
                        ),
                    )?,
                    None,
                );
            }
        }
        Ok(())
    }
    fn track_attr(
        &self,
        attrs: &mut Attributes,
        key: &'static str,
        field: &str,
        tracks: Option<&[Track]>,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        if let Some(tracks) = tracks {
            let values = tracks
                .iter()
                .enumerate()
                .map(|(index, track)| {
                    track.validate(context, &format!("style.{field}[{index}]"), false)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if !values.is_empty() {
                attrs.push((key, values.join(" ")));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub(super) fn test_track(raw: &str) -> Result<String, MeasurementError> {
    let value: Track = serde_json::from_str(raw).map_err(super::test_decode_error)?;
    value.validate(
        &Context {
            case_id: "style-test",
            variant: "test",
            node_path: "root".into(),
        },
        "track",
        false,
    )
}
#[cfg(test)]
pub(super) fn test_position(raw: &str) -> Result<Option<String>, MeasurementError> {
    let value: Position = serde_json::from_str(raw).map_err(super::test_decode_error)?;
    value.validate(
        &Context {
            case_id: "style-test",
            variant: "test",
            node_path: "root".into(),
        },
        "position",
    )
}

#[cfg(test)]
#[path = "style_tests.rs"]
mod tests;
