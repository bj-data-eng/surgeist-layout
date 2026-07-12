use core::ops::{Add, Sub};

use super::{Direction, FlexDirection, LayoutScalar, Scalar, WritingMode};

/// A physical x/y coordinate axis.
///
/// ```compile_fail
/// use surgeist_layout::PhysicalAxis;
/// let _ = PhysicalAxis::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalAxis {
    Horizontal,
    Vertical,
}

impl PhysicalAxis {
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

/// A flow-relative axis used by crate-private layout algorithms.
///
/// ```compile_fail
/// use surgeist_layout::LogicalAxis;
/// let _ = LogicalAxis::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogicalAxis {
    Inline,
    Block,
}

impl LogicalAxis {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-axis inversion for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) const fn other(self) -> Self {
        match self {
            Self::Inline => Self::Block,
            Self::Block => Self::Inline,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PhysicalProgression {
    Increasing,
    Decreasing,
}

impl PhysicalProgression {
    #[must_use]
    pub(crate) const fn is_decreasing(self) -> bool {
        matches!(self, Self::Decreasing)
    }
}

/// A side of a physical rectangle.
///
/// ```compile_fail
/// use surgeist_layout::PhysicalSide;
/// let _ = PhysicalSide::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl PhysicalSide {
    #[must_use]
    pub const fn axis(self) -> PhysicalAxis {
        match self {
            Self::Top | Self::Bottom => PhysicalAxis::Vertical,
            Self::Right | Self::Left => PhysicalAxis::Horizontal,
        }
    }

    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }

    const fn progression(self) -> PhysicalProgression {
        match self {
            Self::Top | Self::Left => PhysicalProgression::Increasing,
            Self::Right | Self::Bottom => PhysicalProgression::Decreasing,
        }
    }
}

/// Resolved physical axes and sides for one writing mode and used direction.
///
/// `FlowAxes` is constructed explicitly from a `WritingMode` and the already-
/// resolved used inline `Direction`; it has no context-free fallback.
///
/// ```compile_fail
/// use surgeist_layout::FlowAxes;
/// let _ = FlowAxes::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlowAxes {
    writing_mode: WritingMode,
    direction: Direction,
}

#[derive(Clone, Copy)]
struct FlowMapping {
    inline_axis: PhysicalAxis,
    inline_start: PhysicalSide,
    block_start: PhysicalSide,
    line_over: PhysicalSide,
}

impl FlowAxes {
    /// Resolves the physical mapping for `writing_mode` and used `direction`.
    #[must_use]
    pub const fn new(writing_mode: WritingMode, direction: Direction) -> Self {
        Self {
            writing_mode,
            direction,
        }
    }

    /// Returns the writing mode used to construct this mapping.
    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    /// Returns the used inline direction used to construct this mapping.
    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    /// Returns the physical axis containing the logical inline axis.
    #[must_use]
    pub const fn inline_axis(self) -> PhysicalAxis {
        self.mapping().inline_axis
    }

    /// Returns the physical axis containing the logical block axis.
    #[must_use]
    pub const fn block_axis(self) -> PhysicalAxis {
        self.inline_axis().other()
    }

    #[must_use]
    pub(crate) fn block_axis_extent<T: Copy>(self, size: Size<T>) -> T {
        match self.block_axis() {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn block_axis_coordinate<T: Copy>(self, point: Point<T>) -> T {
        match self.block_axis() {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    pub(crate) fn line_over_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edge_at_side(edges, self.line_over())
    }

    #[must_use]
    pub(crate) fn line_under_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edge_at_side(edges, self.line_under())
    }

    /// Returns the physical side at logical inline start.
    #[must_use]
    pub const fn inline_start(self) -> PhysicalSide {
        self.mapping().inline_start
    }

    /// Returns the physical side at logical inline end.
    #[must_use]
    pub const fn inline_end(self) -> PhysicalSide {
        self.inline_start().opposite()
    }

    /// Returns the physical side at logical block start.
    #[must_use]
    pub const fn block_start(self) -> PhysicalSide {
        self.mapping().block_start
    }

    /// Returns the physical side at logical block end.
    #[must_use]
    pub const fn block_end(self) -> PhysicalSide {
        self.block_start().opposite()
    }

    /// Returns the physical side at the line-over edge.
    #[must_use]
    pub const fn line_over(self) -> PhysicalSide {
        self.mapping().line_over
    }

    /// Returns the physical side at the line-under edge.
    #[must_use]
    pub const fn line_under(self) -> PhysicalSide {
        self.line_over().opposite()
    }

    #[must_use]
    pub(crate) const fn physical_axis_progression(self, axis: PhysicalAxis) -> PhysicalProgression {
        match (self.inline_axis(), axis) {
            (PhysicalAxis::Horizontal, PhysicalAxis::Horizontal)
            | (PhysicalAxis::Vertical, PhysicalAxis::Vertical) => self.inline_start().progression(),
            (PhysicalAxis::Horizontal, PhysicalAxis::Vertical)
            | (PhysicalAxis::Vertical, PhysicalAxis::Horizontal) => {
                self.block_start().progression()
            }
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-axis progression for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) const fn logical_axis_progression(self, axis: LogicalAxis) -> PhysicalProgression {
        match axis {
            LogicalAxis::Inline => self.inline_start().progression(),
            LogicalAxis::Block => self.block_start().progression(),
        }
    }

    #[must_use]
    pub(crate) fn physical_size<S: LayoutScalar>(self, logical: LogicalSizeOf<S>) -> Size<S> {
        match self.inline_axis() {
            PhysicalAxis::Horizontal => Size::new(logical.inline, logical.block),
            PhysicalAxis::Vertical => Size::new(logical.block, logical.inline),
        }
    }

    #[must_use]
    pub(crate) fn logical_size<S: LayoutScalar>(self, physical: Size<S>) -> LogicalSizeOf<S> {
        match self.inline_axis() {
            PhysicalAxis::Horizontal => LogicalSizeOf::new(physical.width, physical.height),
            PhysicalAxis::Vertical => LogicalSizeOf::new(physical.height, physical.width),
        }
    }

    #[must_use]
    pub(crate) fn zip_physical_edges_with_inline_extent<T, U: Copy, R>(
        self,
        edges: Edges<T>,
        containing_physical_size: Size<U>,
        f: impl Fn(T, U) -> R,
    ) -> Edges<R> {
        let inline_extent = match self.inline_axis() {
            PhysicalAxis::Horizontal => containing_physical_size.width,
            PhysicalAxis::Vertical => containing_physical_size.height,
        };
        edges.map(|edge| f(edge, inline_extent))
    }

    #[must_use]
    pub(crate) fn physical_point<S: LayoutScalar>(
        self,
        logical_origin: LogicalPointOf<S>,
        logical_size: LogicalSizeOf<S>,
        containing_size: Size<S>,
    ) -> Point<S> {
        let inline_extent = physical_extent(containing_size, self.inline_axis());
        let block_extent = physical_extent(containing_size, self.block_axis());
        let inline = project_from_start(
            logical_origin.inline,
            logical_size.inline,
            inline_extent,
            self.inline_start(),
        );
        let block = project_from_start(
            logical_origin.block,
            logical_size.block,
            block_extent,
            self.block_start(),
        );

        match self.inline_axis() {
            PhysicalAxis::Horizontal => Point::new(inline, block),
            PhysicalAxis::Vertical => Point::new(block, inline),
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages physical-to-logical point projection for later flow-aware migrations."
        )
    )]
    #[must_use]
    pub(crate) fn logical_point<S: LayoutScalar>(
        self,
        physical_origin: Point<S>,
        physical_size: Size<S>,
        containing_size: Size<S>,
    ) -> LogicalPointOf<S> {
        let logical_size = self.logical_size(physical_size);
        let (inline_origin, block_origin) = match self.inline_axis() {
            PhysicalAxis::Horizontal => (physical_origin.x, physical_origin.y),
            PhysicalAxis::Vertical => (physical_origin.y, physical_origin.x),
        };
        let inline_extent = physical_extent(containing_size, self.inline_axis());
        let block_extent = physical_extent(containing_size, self.block_axis());

        LogicalPointOf::new(
            project_to_start(
                inline_origin,
                logical_size.inline,
                inline_extent,
                self.inline_start(),
            ),
            project_to_start(
                block_origin,
                logical_size.block,
                block_extent,
                self.block_start(),
            ),
        )
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-to-physical edge projection for later flow-aware migrations."
        )
    )]
    #[must_use]
    pub(crate) fn physical_edges<T>(self, logical: LogicalEdgesOf<T>) -> Edges<T> {
        let LogicalEdgesOf {
            inline_start,
            inline_end,
            block_start,
            block_end,
        } = logical;
        match (self.inline_start(), self.block_start()) {
            (PhysicalSide::Left, PhysicalSide::Top) => {
                Edges::new(block_start, inline_end, block_end, inline_start)
            }
            (PhysicalSide::Right, PhysicalSide::Top) => {
                Edges::new(block_start, inline_start, block_end, inline_end)
            }
            (PhysicalSide::Top, PhysicalSide::Right) => {
                Edges::new(inline_start, block_start, inline_end, block_end)
            }
            (PhysicalSide::Bottom, PhysicalSide::Right) => {
                Edges::new(inline_end, block_start, inline_start, block_end)
            }
            (PhysicalSide::Top, PhysicalSide::Left) => {
                Edges::new(inline_start, block_end, inline_end, block_start)
            }
            (PhysicalSide::Bottom, PhysicalSide::Left) => {
                Edges::new(inline_end, block_end, inline_start, block_start)
            }
            _ => unreachable!("flow mapping always assigns perpendicular logical sides"),
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages physical-to-logical edge projection for later flow-aware migrations."
        )
    )]
    #[must_use]
    pub(crate) fn logical_edges<T>(self, physical: Edges<T>) -> LogicalEdgesOf<T> {
        let Edges {
            top,
            right,
            bottom,
            left,
        } = physical;
        match (self.inline_start(), self.block_start()) {
            (PhysicalSide::Left, PhysicalSide::Top) => {
                LogicalEdgesOf::new(left, right, top, bottom)
            }
            (PhysicalSide::Right, PhysicalSide::Top) => {
                LogicalEdgesOf::new(right, left, top, bottom)
            }
            (PhysicalSide::Top, PhysicalSide::Right) => {
                LogicalEdgesOf::new(top, bottom, right, left)
            }
            (PhysicalSide::Bottom, PhysicalSide::Right) => {
                LogicalEdgesOf::new(bottom, top, right, left)
            }
            (PhysicalSide::Top, PhysicalSide::Left) => {
                LogicalEdgesOf::new(top, bottom, left, right)
            }
            (PhysicalSide::Bottom, PhysicalSide::Left) => {
                LogicalEdgesOf::new(bottom, top, left, right)
            }
            _ => unreachable!("flow mapping always assigns perpendicular logical sides"),
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-to-physical rectangle projection for later flow-aware migrations."
        )
    )]
    #[must_use]
    pub(crate) fn physical_rect<S: LayoutScalar>(
        self,
        logical: LogicalRectOf<S>,
        containing_size: Size<S>,
    ) -> (Point<S>, Size<S>) {
        (
            self.physical_point(logical.origin, logical.size, containing_size),
            self.physical_size(logical.size),
        )
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages physical-to-logical rectangle projection for later flow-aware migrations."
        )
    )]
    #[must_use]
    pub(crate) fn logical_rect<S: LayoutScalar>(
        self,
        physical_origin: Point<S>,
        physical_size: Size<S>,
        containing_size: Size<S>,
    ) -> LogicalRectOf<S> {
        LogicalRectOf::new(
            self.logical_point(physical_origin, physical_size, containing_size),
            self.logical_size(physical_size),
        )
    }

    const fn mapping(self) -> FlowMapping {
        match (self.writing_mode, self.direction) {
            (WritingMode::HorizontalTb, Direction::Ltr) => FlowMapping {
                inline_axis: PhysicalAxis::Horizontal,
                inline_start: PhysicalSide::Left,
                block_start: PhysicalSide::Top,
                line_over: PhysicalSide::Top,
            },
            (WritingMode::HorizontalTb, Direction::Rtl) => FlowMapping {
                inline_axis: PhysicalAxis::Horizontal,
                inline_start: PhysicalSide::Right,
                block_start: PhysicalSide::Top,
                line_over: PhysicalSide::Top,
            },
            (WritingMode::VerticalRl, Direction::Ltr)
            | (WritingMode::SidewaysRl, Direction::Ltr) => FlowMapping {
                inline_axis: PhysicalAxis::Vertical,
                inline_start: PhysicalSide::Top,
                block_start: PhysicalSide::Right,
                line_over: PhysicalSide::Right,
            },
            (WritingMode::VerticalRl, Direction::Rtl)
            | (WritingMode::SidewaysRl, Direction::Rtl) => FlowMapping {
                inline_axis: PhysicalAxis::Vertical,
                inline_start: PhysicalSide::Bottom,
                block_start: PhysicalSide::Right,
                line_over: PhysicalSide::Right,
            },
            (WritingMode::VerticalLr, Direction::Ltr) => FlowMapping {
                inline_axis: PhysicalAxis::Vertical,
                inline_start: PhysicalSide::Top,
                block_start: PhysicalSide::Left,
                line_over: PhysicalSide::Right,
            },
            (WritingMode::VerticalLr, Direction::Rtl) => FlowMapping {
                inline_axis: PhysicalAxis::Vertical,
                inline_start: PhysicalSide::Bottom,
                block_start: PhysicalSide::Left,
                line_over: PhysicalSide::Right,
            },
            (WritingMode::SidewaysLr, Direction::Ltr) => FlowMapping {
                inline_axis: PhysicalAxis::Vertical,
                inline_start: PhysicalSide::Bottom,
                block_start: PhysicalSide::Left,
                line_over: PhysicalSide::Left,
            },
            (WritingMode::SidewaysLr, Direction::Rtl) => FlowMapping {
                inline_axis: PhysicalAxis::Vertical,
                inline_start: PhysicalSide::Top,
                block_start: PhysicalSide::Left,
                line_over: PhysicalSide::Left,
            },
        }
    }
}

fn physical_extent<S: LayoutScalar>(size: Size<S>, axis: PhysicalAxis) -> S {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn edge_at_side<T: Copy>(edges: Edges<T>, side: PhysicalSide) -> T {
    match side {
        PhysicalSide::Top => edges.top,
        PhysicalSide::Right => edges.right,
        PhysicalSide::Bottom => edges.bottom,
        PhysicalSide::Left => edges.left,
    }
}

fn project_from_start<S: LayoutScalar>(origin: S, size: S, extent: S, start: PhysicalSide) -> S {
    if start.progression().is_decreasing() {
        extent - origin - size
    } else {
        origin
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "C01-T1 stages inverse point projection for later flow-aware migrations."
    )
)]
fn project_to_start<S: LayoutScalar>(origin: S, size: S, extent: S, start: PhysicalSide) -> S {
    if start.progression().is_decreasing() {
        extent - origin - size
    } else {
        origin
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LogicalPointOf<S: LayoutScalar = Scalar> {
    pub(crate) inline: S,
    pub(crate) block: S,
}

impl<S: LayoutScalar> LogicalPointOf<S> {
    #[must_use]
    pub(crate) const fn new(inline: S, block: S) -> Self {
        Self { inline, block }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-point mapping for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) fn map<R: LayoutScalar>(self, f: impl Fn(S) -> R) -> LogicalPointOf<R> {
        LogicalPointOf::new(f(self.inline), f(self.block))
    }
}

impl<U: LayoutScalar, T: LayoutScalar + Add<U>> Add<LogicalPointOf<U>> for LogicalPointOf<T>
where
    <T as Add<U>>::Output: LayoutScalar,
{
    type Output = LogicalPointOf<<T as Add<U>>::Output>;

    fn add(self, rhs: LogicalPointOf<U>) -> Self::Output {
        LogicalPointOf::new(self.inline + rhs.inline, self.block + rhs.block)
    }
}

impl<U: LayoutScalar, T: LayoutScalar + Sub<U>> Sub<LogicalPointOf<U>> for LogicalPointOf<T>
where
    <T as Sub<U>>::Output: LayoutScalar,
{
    type Output = LogicalPointOf<<T as Sub<U>>::Output>;

    fn sub(self, rhs: LogicalPointOf<U>) -> Self::Output {
        LogicalPointOf::new(self.inline - rhs.inline, self.block - rhs.block)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LogicalSizeOf<S: LayoutScalar = Scalar> {
    pub(crate) inline: S,
    pub(crate) block: S,
}

impl<S: LayoutScalar> LogicalSizeOf<S> {
    #[must_use]
    pub(crate) const fn new(inline: S, block: S) -> Self {
        Self { inline, block }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-size mapping for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) fn map<R: LayoutScalar>(self, f: impl Fn(S) -> R) -> LogicalSizeOf<R> {
        LogicalSizeOf::new(f(self.inline), f(self.block))
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-size pairwise mapping for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) fn zip_map<U: LayoutScalar, R: LayoutScalar>(
        self,
        other: LogicalSizeOf<U>,
        f: impl Fn(S, U) -> R,
    ) -> LogicalSizeOf<R> {
        LogicalSizeOf::new(f(self.inline, other.inline), f(self.block, other.block))
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-axis size selection for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) const fn axis(self, axis: LogicalAxis) -> S {
        match axis {
            LogicalAxis::Inline => self.inline,
            LogicalAxis::Block => self.block,
        }
    }
}

impl<U: LayoutScalar, T: LayoutScalar + Add<U>> Add<LogicalSizeOf<U>> for LogicalSizeOf<T>
where
    <T as Add<U>>::Output: LayoutScalar,
{
    type Output = LogicalSizeOf<<T as Add<U>>::Output>;

    fn add(self, rhs: LogicalSizeOf<U>) -> Self::Output {
        LogicalSizeOf::new(self.inline + rhs.inline, self.block + rhs.block)
    }
}

impl<U: LayoutScalar, T: LayoutScalar + Sub<U>> Sub<LogicalSizeOf<U>> for LogicalSizeOf<T>
where
    <T as Sub<U>>::Output: LayoutScalar,
{
    type Output = LogicalSizeOf<<T as Sub<U>>::Output>;

    fn sub(self, rhs: LogicalSizeOf<U>) -> Self::Output {
        LogicalSizeOf::new(self.inline - rhs.inline, self.block - rhs.block)
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "C01-T1 stages logical edge geometry for later flow-aware algorithm migrations."
    )
)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LogicalEdgesOf<T = Scalar> {
    pub(crate) inline_start: T,
    pub(crate) inline_end: T,
    pub(crate) block_start: T,
    pub(crate) block_end: T,
}

impl<T> LogicalEdgesOf<T> {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-edge construction for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) const fn new(inline_start: T, inline_end: T, block_start: T, block_end: T) -> Self {
        Self {
            inline_start,
            inline_end,
            block_start,
            block_end,
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-edge mapping for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) fn map<R>(self, f: impl Fn(T) -> R) -> LogicalEdgesOf<R> {
        LogicalEdgesOf::new(
            f(self.inline_start),
            f(self.inline_end),
            f(self.block_start),
            f(self.block_end),
        )
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "C01-T1 stages logical rectangle geometry for later flow-aware algorithm migrations."
    )
)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LogicalRectOf<S: LayoutScalar = Scalar> {
    pub(crate) origin: LogicalPointOf<S>,
    pub(crate) size: LogicalSizeOf<S>,
}

impl<S: LayoutScalar> LogicalRectOf<S> {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "C01-T1 stages logical-rectangle construction for later flow-aware algorithm migrations."
        )
    )]
    #[must_use]
    pub(crate) const fn new(origin: LogicalPointOf<S>, size: LogicalSizeOf<S>) -> Self {
        Self { origin, size }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Point<T = Scalar> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn map<R>(self, f: impl Fn(T) -> R) -> Point<R> {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }

    #[must_use]
    pub fn transpose(self) -> Point<T> {
        Point {
            x: self.y,
            y: self.x,
        }
    }

    #[must_use]
    pub fn main(self, direction: FlexDirection) -> T {
        if direction.is_row() { self.x } else { self.y }
    }

    #[must_use]
    pub fn cross(self, direction: FlexDirection) -> T {
        if direction.is_row() { self.y } else { self.x }
    }
}

impl<T> Point<Option<T>> {
    pub const NONE: Self = Self { x: None, y: None };
}

impl<S: LayoutScalar> Point<S> {
    pub const ZERO: Self = Self::new(S::ZERO, S::ZERO);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size<T = Scalar> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    #[must_use]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub fn map<R>(self, f: impl Fn(T) -> R) -> Size<R> {
        Size {
            width: f(self.width),
            height: f(self.height),
        }
    }

    #[must_use]
    pub fn zip_map<U, R>(self, other: Size<U>, f: impl Fn(T, U) -> R) -> Size<R> {
        Size {
            width: f(self.width, other.width),
            height: f(self.height, other.height),
        }
    }

    #[must_use]
    pub fn main(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.width
        } else {
            self.height
        }
    }

    #[must_use]
    pub fn cross(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.height
        } else {
            self.width
        }
    }
}

impl<T> Size<Option<T>> {
    pub const NONE: Self = Self {
        width: None,
        height: None,
    };
}

impl<T> Size<Option<T>> {
    #[must_use]
    pub const fn from_cross(direction: FlexDirection, value: Option<T>) -> Self {
        if direction.is_row() {
            Self {
                width: None,
                height: value,
            }
        } else {
            Self {
                width: value,
                height: None,
            }
        }
    }
}

impl<S: LayoutScalar> Size<S> {
    pub const ZERO: Self = Self::new(S::ZERO, S::ZERO);
}

impl<T: Copy> Size<T> {
    pub const fn splat(value: T) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}

impl<U, T: Add<U>> Add<Size<U>> for Size<T> {
    type Output = Size<T::Output>;

    fn add(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<U, T: Sub<U>> Sub<Size<U>> for Size<T> {
    type Output = Size<T::Output>;

    fn sub(self, rhs: Size<U>) -> Self::Output {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Edges<T = Scalar> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Edges<T> {
    #[must_use]
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub fn map<R>(self, f: impl Fn(T) -> R) -> Edges<R> {
        Edges {
            top: f(self.top),
            right: f(self.right),
            bottom: f(self.bottom),
            left: f(self.left),
        }
    }

    #[must_use]
    pub fn zip_size<U, R>(self, size: Size<U>, f: impl Fn(T, U) -> R) -> Edges<R>
    where
        U: Copy,
    {
        Edges {
            top: f(self.top, size.height),
            right: f(self.right, size.width),
            bottom: f(self.bottom, size.height),
            left: f(self.left, size.width),
        }
    }
}

impl<U, T: Add<U>> Add<Edges<U>> for Edges<T> {
    type Output = Edges<T::Output>;

    fn add(self, rhs: Edges<U>) -> Self::Output {
        Edges {
            top: self.top + rhs.top,
            right: self.right + rhs.right,
            bottom: self.bottom + rhs.bottom,
            left: self.left + rhs.left,
        }
    }
}

impl<T: Copy> Edges<T> {
    #[must_use]
    pub const fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

impl<T> Edges<T>
where
    T: Add<Output = T> + Copy,
{
    #[must_use]
    pub fn horizontal_sum(self) -> T {
        self.left + self.right
    }

    #[must_use]
    pub fn vertical_sum(self) -> T {
        self.top + self.bottom
    }

    #[must_use]
    pub fn sum_axes(self) -> Size<T> {
        Size::new(self.horizontal_sum(), self.vertical_sum())
    }

    #[must_use]
    pub fn main_sum(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.horizontal_sum()
        } else {
            self.vertical_sum()
        }
    }

    #[must_use]
    pub fn cross_sum(self, direction: FlexDirection) -> T {
        if direction.is_row() {
            self.vertical_sum()
        } else {
            self.horizontal_sum()
        }
    }
}

impl<S: LayoutScalar> Edges<S> {
    pub const ZERO: Self = Self::all(S::ZERO);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction, WritingMode};

    #[test]
    fn flow_axes_cover_the_normative_ten_row_mapping() {
        let rows = [
            (
                WritingMode::HorizontalTb,
                Direction::Ltr,
                PhysicalAxis::Horizontal,
                PhysicalAxis::Vertical,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
            ),
            (
                WritingMode::HorizontalTb,
                Direction::Rtl,
                PhysicalAxis::Horizontal,
                PhysicalAxis::Vertical,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
            ),
            (
                WritingMode::VerticalRl,
                Direction::Ltr,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Left,
            ),
            (
                WritingMode::VerticalRl,
                Direction::Rtl,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Bottom,
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Left,
            ),
            (
                WritingMode::VerticalLr,
                Direction::Ltr,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Right,
                PhysicalSide::Left,
            ),
            (
                WritingMode::VerticalLr,
                Direction::Rtl,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Bottom,
                PhysicalSide::Top,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Right,
                PhysicalSide::Left,
            ),
            (
                WritingMode::SidewaysRl,
                Direction::Ltr,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Left,
            ),
            (
                WritingMode::SidewaysRl,
                Direction::Rtl,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Bottom,
                PhysicalSide::Top,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Left,
            ),
            (
                WritingMode::SidewaysLr,
                Direction::Ltr,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Bottom,
                PhysicalSide::Top,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Right,
            ),
            (
                WritingMode::SidewaysLr,
                Direction::Rtl,
                PhysicalAxis::Vertical,
                PhysicalAxis::Horizontal,
                PhysicalSide::Top,
                PhysicalSide::Bottom,
                PhysicalSide::Left,
                PhysicalSide::Right,
                PhysicalSide::Left,
                PhysicalSide::Right,
            ),
        ];

        for (
            writing_mode,
            direction,
            inline_axis,
            block_axis,
            inline_start,
            inline_end,
            block_start,
            block_end,
            line_over,
            line_under,
        ) in rows
        {
            let axes = FlowAxes::new(writing_mode, direction);
            assert_eq!(axes.writing_mode(), writing_mode);
            assert_eq!(axes.direction(), direction);
            assert_eq!(axes.inline_axis(), inline_axis);
            assert_eq!(axes.block_axis(), block_axis);
            assert_eq!(axes.inline_start(), inline_start);
            assert_eq!(axes.inline_end(), inline_end);
            assert_eq!(axes.block_start(), block_start);
            assert_eq!(axes.block_end(), block_end);
            assert_eq!(axes.line_over(), line_over);
            assert_eq!(axes.line_under(), line_under);
            assert_eq!(axes.inline_start().axis(), axes.inline_axis());
            assert_eq!(axes.inline_end().axis(), axes.inline_axis());
            assert_eq!(axes.block_start().axis(), axes.block_axis());
            assert_eq!(axes.block_end().axis(), axes.block_axis());
            assert_eq!(axes.line_over().axis(), axes.block_axis());
            assert_eq!(axes.line_under().axis(), axes.block_axis());
        }
    }

    fn assert_flow_axes_round_trip<S: LayoutScalar>(
        writing_mode: WritingMode,
        direction: Direction,
    ) {
        let axes = FlowAxes::new(writing_mode, direction);
        let containing_size = Size::new(S::from_f64(70.0), S::from_f64(110.0));
        let logical = LogicalRectOf::new(
            LogicalPointOf::new(S::from_f64(9.0), S::from_f64(13.0)),
            LogicalSizeOf::new(S::from_f64(17.0), S::from_f64(23.0)),
        );

        let (physical_origin, physical_size) = axes.physical_rect(logical, containing_size);
        if (writing_mode, direction) == (WritingMode::SidewaysLr, Direction::Ltr) {
            assert_eq!(
                physical_origin,
                Point::new(S::from_f64(13.0), S::from_f64(84.0))
            );
            assert_eq!(
                physical_size,
                Size::new(S::from_f64(23.0), S::from_f64(17.0))
            );
        }
        assert_eq!(
            axes.logical_rect(physical_origin, physical_size, containing_size),
            logical
        );
        assert_eq!(
            axes.logical_size(axes.physical_size(logical.size)),
            logical.size
        );

        let logical_edges = LogicalEdgesOf::new(
            S::from_f64(2.0),
            S::from_f64(3.0),
            S::from_f64(5.0),
            S::from_f64(7.0),
        );
        assert_eq!(
            axes.logical_edges(axes.physical_edges(logical_edges)),
            logical_edges
        );
        assert_eq!(
            axes.logical_point(
                axes.physical_point(logical.origin, logical.size, containing_size),
                axes.physical_size(logical.size),
                containing_size,
            ),
            logical.origin
        );
    }

    #[test]
    fn flow_axes_round_trip_size_edges_point_and_rect_for_all_rows_and_scalar_lanes() {
        for (writing_mode, direction) in [
            (WritingMode::HorizontalTb, Direction::Ltr),
            (WritingMode::HorizontalTb, Direction::Rtl),
            (WritingMode::VerticalRl, Direction::Ltr),
            (WritingMode::VerticalRl, Direction::Rtl),
            (WritingMode::VerticalLr, Direction::Ltr),
            (WritingMode::VerticalLr, Direction::Rtl),
            (WritingMode::SidewaysRl, Direction::Ltr),
            (WritingMode::SidewaysRl, Direction::Rtl),
            (WritingMode::SidewaysLr, Direction::Ltr),
            (WritingMode::SidewaysLr, Direction::Rtl),
        ] {
            assert_flow_axes_round_trip::<f32>(writing_mode, direction);
            assert_flow_axes_round_trip::<f64>(writing_mode, direction);
        }
    }

    #[test]
    fn logical_geometry_helpers_preserve_axis_and_component_semantics() {
        assert_eq!(LogicalAxis::Inline.other(), LogicalAxis::Block);
        assert_eq!(LogicalAxis::Block.other(), LogicalAxis::Inline);

        let axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Ltr);
        assert_eq!(
            axes.logical_axis_progression(LogicalAxis::Inline),
            PhysicalProgression::Decreasing
        );
        assert_eq!(
            axes.logical_axis_progression(LogicalAxis::Block),
            PhysicalProgression::Increasing
        );

        let point = LogicalPointOf::new(2.0, 3.0);
        assert_eq!(
            point.map(|value| value * 2.0),
            LogicalPointOf::new(4.0, 6.0)
        );

        let size = LogicalSizeOf::new(5.0, 7.0);
        assert_eq!(size.map(|value| value + 1.0), LogicalSizeOf::new(6.0, 8.0));
        assert_eq!(
            size.zip_map(LogicalSizeOf::new(11.0, 13.0), |left, right| left + right),
            LogicalSizeOf::new(16.0, 20.0)
        );
        assert_eq!(size.axis(LogicalAxis::Inline), 5.0);
        assert_eq!(size.axis(LogicalAxis::Block), 7.0);

        let edges = LogicalEdgesOf::new(2.0, 3.0, 5.0, 7.0);
        assert_eq!(
            edges.map(|value| value * 2.0),
            LogicalEdgesOf::new(4.0, 6.0, 10.0, 14.0)
        );
    }

    #[test]
    fn containing_flow_uses_its_logical_inline_extent_for_every_physical_edge() {
        let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        let edges = Edges::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(
            axes.zip_physical_edges_with_inline_extent(
                edges,
                Size::new(40.0, 60.0),
                |edge, basis| edge * basis,
            ),
            Edges::new(60.0, 120.0, 180.0, 240.0)
        );
    }
}
