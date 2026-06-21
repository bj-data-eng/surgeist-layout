# Surgeist Baseline Alignment Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add correct first/last baseline alignment support to Surgeist's layout engine, especially for grid, subgrid, and grid-lanes, without discarding the existing grid/subgrid/grid-lanes implementation.

**Architecture:** Treat baseline support as a cross-cutting layout output capability plus a grid-specific alignment phase. Keep the current track sizing, placement, subgrid inheritance, and grid-lanes structure intact; add explicit baseline vocabulary, group computation, baseline shims, and subgrid baseline propagation where the existing code already computes child layout and track contributions. Use WebKit's `GridBaselineAlignment`/`GridTrackSizingAlgorithm` flow as the primary reference, with Blink's `GridBaselineAccumulator` and `GridLayoutUtils` as a secondary sanity check.

**Tech Stack:** Rust under `crates/surgeist`, production layout modules in `crates/surgeist/src/layout`, style/CSS plumbing in `crates/surgeist/src/style` and `crates/surgeist/src/css`, browser parity support in `crates/surgeist/tests/layout_browser_parity`, verification with focused `cargo test -p surgeist ...` commands, `cargo fmt --check`, and `cargo clippy -p surgeist --all-targets --all-features -- -D warnings`.

---

## Source References

- Existing engine plan: `docs/superpowers/plans/2026-06-17-surgeist-grid-subgrid-lanes-engine-implementation.md`
- Oracle spec: `docs/superpowers/specs/2026-06-16-surgeist-subgrid-grid-lanes-oracle-design.md`
- Production layout baseline surfaces:
  - `crates/surgeist/src/layout/output.rs`
  - `crates/surgeist/src/layout/compute.rs`
  - `crates/surgeist/src/layout/block.rs`
  - `crates/surgeist/src/layout/flex.rs`
  - `crates/surgeist/src/layout/grid/child.rs`
  - `crates/surgeist/src/layout/grid/lanes.rs`
  - `crates/surgeist/src/layout/grid/tracks.rs`
  - `crates/surgeist/src/layout/grid/subgrid.rs`
- Style and parser surfaces:
  - `crates/surgeist/src/layout/node_input.rs`
  - `crates/surgeist/src/style/value.rs`
  - `crates/surgeist/src/style/adapters/layout.rs`
  - `crates/surgeist/src/css/mod.rs`
  - `crates/surgeist/tests/layout_browser_parity/support.rs`
- WebKit references:
  - `tmp/WebKit/Source/WebCore/rendering/GridBaselineAlignment.h`
  - `tmp/WebKit/Source/WebCore/rendering/GridTrackSizingAlgorithm.h`
  - `tmp/WebKit/Source/WebCore/rendering/GridTrackSizingAlgorithm.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderGrid.cpp`
  - `tmp/WebKit/Source/WebCore/rendering/RenderBlock.cpp`
- Blink references:
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_baseline_accumulator.h`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_layout_utils.cc`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_item.{cc,h}`
  - `tmp/chromium-inline/third_party/blink/renderer/core/layout/grid/grid_layout_algorithm.cc`

---

## Boundary Decisions

- [ ] Keep production code independent from the test oracle. Production may mirror oracle vocabulary but must not import `crates/surgeist/tests/support/oracle`.
- [ ] Keep `compute_grid` and `compute_grid_with_context` as the public production grid entry points.
- [ ] Do not collapse `last baseline` into `baseline`; add explicit first/last vocabulary.
- [ ] Do not silently treat `inline-grid`/`inline-block` browser fixtures as ordinary block fixtures when evaluating parity failures. For this plan, implement the layout-engine baseline support first and mark true inline formatting context behavior as a separate follow-up unless a focused baseline test requires it.
- [ ] Do not replace the current grid/subgrid/grid-lanes implementation. The implementation should add baseline data and baseline offset phases around existing placement and sizing.
- [ ] Commit after logical checkpoints. Run `git status --short --branch` and `git diff --check` before staging each commit.

---

## Implementation Overview

The current layout engine exposes only `ComputeOutput::first_baselines: Point<Option<Scalar>>`. Grid and grid-lanes use `output.first_baselines.y.unwrap_or(output.size.height)` as the child baseline and produce `Point::new(None, first_baseline)` as the container baseline. That is enough for simple first-baseline flex/grid cases, but it cannot model:

- `last baseline` alignment.
- Major/minor baseline groups.
- Baseline fallback/synthesis as an explicit rule.
- Baseline offsets that affect intrinsic track sizing.
- Subgrid baseline propagation through inherited axes.
- Baseline choice for the grid container itself.

WebKit's useful lesson is architectural: baseline-aligned grid items are detected before final placement, grouped by shared alignment context, cached into a baseline state, and queried for per-item baseline offsets during sizing and placement. Blink's useful lesson is data shape: keep first and last baselines, expose major/minor track baselines, synthesize missing baselines from border-box edges, and accumulate the container's first/last baseline from grid-order items.

Implement in this order:

1. Add explicit first/last baseline output and baseline helpers with compatibility for existing first-baseline callers.
2. Parse and lower `last baseline` alignment without changing behavior yet.
3. Add focused baseline fixtures and pure layout tests that fail for last-baseline and synthesized-baseline cases.
4. Add grid baseline grouping and offset computation for rows.
5. Thread row baseline offsets into intrinsic track sizing where baseline alignment changes an item's contribution.
6. Add container first/last baseline reporting.
7. Add subgrid baseline propagation for inherited row/column axes.
8. Extend grid-lanes baseline behavior only to the degree supported by WebKit/Blink and documented fallbacks.
9. Regenerate/check browser parity after the focused tests pass.

---

## Task 1: Add Explicit Baseline Output Types

**Files:**
- Modify `crates/surgeist/src/layout/output.rs`
- Modify `crates/surgeist/src/layout/mod.rs`
- Modify `crates/surgeist/src/layout/compute.rs`
- Modify `crates/surgeist/src/layout/block.rs`
- Modify `crates/surgeist/src/layout/flex.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/tests.rs`

- [ ] Add failing tests that prove a `ComputeOutput` can carry both first and last baselines while preserving existing constructor behavior.

Add tests in `crates/surgeist/src/layout/tests.rs`:

```rust
#[test]
fn compute_output_preserves_first_and_last_baselines() {
    let output = ComputeOutput::from_sizes_and_baselines(
        Size::new(40.0, 30.0),
        Size::ZERO,
        Baselines {
            first: Point::new(None, Some(8.0)),
            last: Point::new(None, Some(24.0)),
        },
    );

    assert_eq!(output.first_baselines.y, Some(8.0));
    assert_eq!(output.last_baselines.y, Some(24.0));
}

#[test]
fn compute_output_from_sizes_has_no_explicit_baselines() {
    let output = ComputeOutput::from_sizes(Size::new(40.0, 30.0), Size::ZERO);

    assert_eq!(output.first_baselines, Point::NONE);
    assert_eq!(output.last_baselines, Point::NONE);
}
```

Run:

```bash
cargo test -p surgeist --lib compute_output_preserves_first_and_last_baselines
cargo test -p surgeist --lib compute_output_from_sizes_has_no_explicit_baselines
```

Expected: fail because `Baselines` and `last_baselines` do not exist.

- [ ] Add the production baseline value type in `crates/surgeist/src/layout/output.rs`.

Expected code shape:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Baselines {
    pub first: Point<Option<Scalar>>,
    pub last: Point<Option<Scalar>>,
}

impl Baselines {
    pub const NONE: Self = Self {
        first: Point::NONE,
        last: Point::NONE,
    };

    pub const fn first(first: Point<Option<Scalar>>) -> Self {
        Self {
            first,
            last: Point::NONE,
        }
    }

    pub fn synthesized(size: Size) -> Self {
        Self {
            first: Point::new(Some(size.width), Some(size.height)),
            last: Point::new(Some(0.0), Some(0.0)),
        }
    }

    pub fn first_or_synthesize_block(self, size: Size) -> Scalar {
        self.first.y.unwrap_or(size.height)
    }

    pub fn last_or_synthesize_block(self, size: Size) -> Scalar {
        self.last.y.unwrap_or(0.0)
    }
}
```

Keep `ComputeOutput::first_baselines` for compatibility and add `ComputeOutput::last_baselines`:

```rust
pub struct ComputeOutput {
    pub size: Size,
    pub content_size: Size,
    pub first_baselines: Point<Option<Scalar>>,
    pub last_baselines: Point<Option<Scalar>>,
    pub top_margin: CollapsibleMargin,
    pub bottom_margin: CollapsibleMargin,
    pub margins_can_collapse_through: bool,
}
```

Change `from_sizes_and_baselines` to accept `Baselines`, and add a compatibility constructor:

```rust
pub const fn from_sizes_and_first_baselines(
    size: Size,
    content_size: Size,
    first_baselines: Point<Option<Scalar>>,
) -> Self {
    Self::from_sizes_and_baselines(size, content_size, Baselines::first(first_baselines))
}
```

- [ ] Update all call sites using `ComputeOutput::from_sizes_and_baselines` to pass `Baselines::first(...)`, or use `from_sizes_and_first_baselines`.

Expected grid call shape:

```rust
ComputeOutput::from_sizes_and_baselines(
    output_size,
    content_size,
    Baselines {
        first: Point::new(None, child_layout.first_baseline),
        last: Point::new(None, child_layout.last_baseline),
    },
)
```

- [ ] Update code that synthesizes child baselines by reading `output.first_baselines.y.unwrap_or(output.size.height)` to use a helper. This keeps synthesis intentional and searchable.

Expected helper usage:

```rust
let first_baseline = output.baselines().first_or_synthesize_block(output.size);
let last_baseline = output.baselines().last_or_synthesize_block(output.size);
```

If adding `ComputeOutput::baselines()` is cleaner:

```rust
impl ComputeOutput {
    pub const fn baselines(self) -> Baselines {
        Baselines {
            first: self.first_baselines,
            last: self.last_baselines,
        }
    }
}
```

- [ ] Run:

```bash
cargo test -p surgeist --lib compute_output_preserves_first_and_last_baselines
cargo test -p surgeist --lib compute_output_from_sizes_has_no_explicit_baselines
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/output.rs crates/surgeist/src/layout/mod.rs crates/surgeist/src/layout/compute.rs crates/surgeist/src/layout/block.rs crates/surgeist/src/layout/flex.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/tests.rs
git commit -m "Add explicit layout baselines"
```

---

## Task 2: Parse And Lower First/Last Baseline Alignment

**Files:**
- Modify `crates/surgeist/src/layout/node_input.rs`
- Modify `crates/surgeist/src/style/value.rs`
- Modify `crates/surgeist/src/style/adapters/layout.rs`
- Modify `crates/surgeist/src/css/mod.rs`
- Modify `crates/surgeist/tests/style.rs`
- Modify `crates/surgeist/tests/css.rs`
- Modify `crates/surgeist/tests/layout_browser_parity/support.rs`

- [ ] Add failing CSS/style tests for `baseline`, `first baseline`, and `last baseline`.

Add or extend tests in `crates/surgeist/tests/css.rs`:

```rust
#[test]
fn parses_first_and_last_baseline_item_alignment() {
    let sheet = parse_stylesheet(
        ".a { align-items: first baseline; align-self: last baseline; justify-items: baseline; }",
    )
    .expect("stylesheet should parse");

    let rule = &sheet.rules()[0];
    assert_eq!(
        rule.declarations().get(s::Property::AlignItems),
        Some(&s::Value::AlignItems(s::AlignItems::Baseline))
    );
    assert_eq!(
        rule.declarations().get(s::Property::AlignSelf),
        Some(&s::Value::AlignItems(s::AlignItems::LastBaseline))
    );
    assert_eq!(
        rule.declarations().get(s::Property::JustifyItems),
        Some(&s::Value::AlignItems(s::AlignItems::Baseline))
    );
}
```

Add a lowering test in `crates/surgeist/tests/style.rs`:

```rust
#[test]
fn lowers_last_baseline_alignment_to_layout() {
    let resolved = resolved_style_for_declarations([
        s::Declaration::new(
            s::Property::AlignSelf,
            s::Value::AlignItems(s::AlignItems::LastBaseline),
        ),
    ]);

    let layout = s::adapters::layout::to_layout(&resolved).expect("style lowers");
    assert_eq!(layout.align_self, Some(l::AlignItems::LastBaseline));
}
```

Use the existing local helper names in `tests/style.rs`; if `resolved_style_for_declarations` is not the exact helper, adapt only the helper call, not the assertion.

- [ ] Add `LastBaseline` to style and layout alignment enums.

Expected layout enum:

```rust
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Baseline,
    LastBaseline,
    Stretch,
}
```

Expected style enum:

```rust
pub enum AlignItems {
    Auto,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Baseline,
    LastBaseline,
    Stretch,
}
```

Keep `baseline` and `first baseline` as first-baseline alignment. Do not introduce a separate `FirstBaseline` variant unless a later task proves it reduces branching; `Baseline` should mean CSS first baseline.

- [ ] Update `AlignItems::safe_fallback` and `unsafe_position`.

Expected behavior:

```rust
Self::Baseline | Self::LastBaseline => self,
```

- [ ] Update CSS parsing so multi-token baseline positions work.

Expected parser behavior:

```text
baseline -> AlignItems::Baseline
first baseline -> AlignItems::Baseline
last baseline -> AlignItems::LastBaseline
safe center -> AlignItems::SafeCenter
safe end -> AlignItems::SafeEnd
safe flex-end -> AlignItems::SafeFlexEnd
```

- [ ] Update browser parity support parsing in `crates/surgeist/tests/layout_browser_parity/support.rs`.

Expected parsing behavior:

```rust
(_, "baseline") => Ok(layout::AlignItems::Baseline),
(_, "first baseline") => Ok(layout::AlignItems::Baseline),
(_, "last baseline") => Ok(layout::AlignItems::LastBaseline),
```

Preserve existing support for `safe center`, `safe end`, and `safe flex-end`.

- [ ] Update display parsing only if necessary to keep current uncommitted parity support compiling. Do not add new inline-block/inline-grid behavior in this task beyond whatever exists in the working tree.

- [ ] Run:

```bash
cargo test -p surgeist --test css parses_first_and_last_baseline_item_alignment
cargo test -p surgeist --test style lowers_last_baseline_alignment_to_layout
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/node_input.rs crates/surgeist/src/style/value.rs crates/surgeist/src/style/adapters/layout.rs crates/surgeist/src/css/mod.rs crates/surgeist/tests/style.rs crates/surgeist/tests/css.rs crates/surgeist/tests/layout_browser_parity/support.rs
git commit -m "Parse first and last baseline alignment"
```

---

## Baseline Coordinate Convention

Before Task 3 changes grid behavior, establish this convention in code comments and tests:

- `ComputeOutput.first_baselines.y` is a border-box distance from the item's block-start edge to the first baseline.
- `ComputeOutput.last_baselines.y` is a border-box distance from the item's block-start edge to the last baseline.
- A grid item's major baseline contribution is `block_start_margin + first_baseline_from_block_start`.
- A grid item's minor baseline contribution is `block_end_margin + (border_box_block_size - last_baseline_from_block_start)`.
- Baseline groups store distances from the shared alignment context's start edge for major groups and from the shared alignment context's end edge for minor groups.
- Placement offsets are computed from the whole spanned alignment area, including row gaps, not from only the first or last row.

Add these helper shapes in `crates/surgeist/src/layout/grid/child.rs` before implementing placement:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BaselineGroupKind {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BaselineParticipation {
    pub(super) participates: bool,
    pub(super) group: Option<BaselineGroupKind>,
    pub(super) synthesized: bool,
    pub(super) fallback_alignment: Option<AlignItems>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BaselineGeometry {
    pub(super) available_span_size: Scalar,
    pub(super) margin_box_size: Scalar,
    pub(super) major_baseline: Scalar,
    pub(super) minor_baseline: Scalar,
}
```

Required helper behavior:

```rust
fn baseline_offset(
    group_kind: BaselineGroupKind,
    shared_baseline: Scalar,
    geometry: BaselineGeometry,
) -> Scalar {
    match group_kind {
        BaselineGroupKind::Major => shared_baseline - geometry.major_baseline,
        BaselineGroupKind::Minor => {
            let baseline_delta = shared_baseline - geometry.minor_baseline;
            geometry.available_span_size - baseline_delta - geometry.margin_box_size
        }
    }
}
```

This follows WebKit's baseline-alignment flow while using Blink's compact `ComputeBaselineOffset` formula as the easiest executable shape: major groups use track baseline minus item baseline; minor groups use the available alignment area minus baseline delta minus item size.

Participation rules:

- Out-of-flow items never participate.
- Items with auto margins in the baseline axis never participate.
- `AlignItems::Baseline` participates as `BaselineGroupKind::Major`.
- `AlignItems::LastBaseline` participates as `BaselineGroupKind::Minor`.
- Synthesized baselines participate only when the item does not create an intrinsic sizing cycle. When a baseline-less item spans an intrinsic/flexible track in the queried axis, fall back to start alignment for major groups and end alignment for minor groups.
- The production code must carry whether the used baseline was synthesized so track sizing can avoid cycle-producing shims.

WebKit references for these rules:

- `RenderGrid::isBaselineAlignmentForGridItem` excludes out-of-flow and auto-margin items.
- `GridTrackSizingAlgorithm::canParticipateInBaselineAlignment` rejects synthesized-baseline cases that would create intrinsic sizing cycles.
- `GridTrackSizingAlgorithm::baselineOffsetForGridItem` is the behavior to match when the item is eligible.

Required tests before any placement edits:

- major offset with nonzero block-start and block-end margins.
- minor offset with nonzero block-start and block-end margins.
- major offset for a row-spanning item across a row gap.
- minor offset for a row-spanning item across a row gap.
- absolutely positioned grid item requests baseline but does not participate.
- baseline-less item in an intrinsic row falls back instead of producing a shim.

---

## Task 3: Add Focused Grid Baseline Unit Tests

**Files:**
- Modify `crates/surgeist/src/layout/grid/tests.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/lanes.rs`

- [ ] Add failing unit tests for baseline group computation before changing layout behavior.

Add tests in `crates/surgeist/src/layout/grid/tests.rs`:

```rust
#[test]
fn row_baselines_choose_first_baseline_for_first_group() {
    let items = vec![
        baseline_test_item(0, 0, 1, AlignItems::Baseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 1, AlignItems::Baseline, 14.0, 20.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2);

    assert_eq!(groups.rows[0].first, Some(14.0));
}

#[test]
fn row_baselines_choose_last_baseline_for_last_group() {
    let items = vec![
        baseline_test_item(0, 0, 2, AlignItems::LastBaseline, 8.0, 22.0, 30.0),
        baseline_test_item(1, 0, 2, AlignItems::LastBaseline, 8.0, 18.0, 30.0),
    ];

    let groups = baseline_groups(&items, 2);

    assert_eq!(groups.rows[1].last, Some(12.0));
}
```

Define the helper in the test module, matching the production `PendingGridItem` shape after Task 1:

```rust
fn baseline_test_item(
    row: usize,
    column: usize,
    row_span: usize,
    align_self: AlignItems,
    first: Scalar,
    last: Scalar,
    height: Scalar,
) -> PendingGridItem<()> {
    PendingGridItem {
        node: (),
        order: 0,
        area: GridArea {
            row,
            column,
            row_end: row + row_span,
            column_end: column + 1,
            size: Size::new(40.0, height),
        },
        output: ComputeOutput::from_sizes_and_baselines(
            Size::new(40.0, height),
            Size::ZERO,
            Baselines {
                first: Point::new(None, Some(first)),
                last: Point::new(None, Some(last)),
            },
        ),
        horizontal_axis: ResolvedGridItemAxis {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        vertical_axis: ResolvedGridItemAxis {
            offset: 0.0,
            margin_start: 0.0,
            margin_end: 0.0,
        },
        relative_offset: Point::ZERO,
        first_baseline: first,
        last_baseline: last,
        block_auto_margins: false,
        baseline_participation: BaselineParticipation {
            participates: matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline),
            group: match align_self {
                AlignItems::Baseline => Some(BaselineGroupKind::Major),
                AlignItems::LastBaseline => Some(BaselineGroupKind::Minor),
                _ => None,
            },
            synthesized: false,
            fallback_alignment: None,
        },
        margin: Edges::ZERO,
        scrollbar_size: Size::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        overflow: Point::new(Overflow::Visible, Overflow::Visible),
        align_self,
    }
}
```

If private visibility makes this awkward, move the helper tests into `crates/surgeist/src/layout/grid/child.rs` under `#[cfg(test)]` instead of weakening production visibility.

- [ ] Replace `row_baselines` with a first/last baseline group model.

Expected code shape in `crates/surgeist/src/layout/grid/child.rs`:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TrackBaselineGroup {
    pub(super) first: Option<Scalar>,
    pub(super) last: Option<Scalar>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GridBaselineGroups {
    pub(super) rows: Vec<TrackBaselineGroup>,
    pub(super) columns: Vec<TrackBaselineGroup>,
}
```

For the first increment, implement row groups only and keep `columns` filled with defaults. That matches current horizontal writing-mode behavior while leaving the data shape ready for column-axis baseline alignment.

Rules:

- `AlignItems::Baseline` participates in the start-most row shared alignment context.
- `AlignItems::LastBaseline` participates in the end-most row shared alignment context.
- First-baseline group stores the max margin-box distance from alignment-context start to first baseline.
- Last-baseline group stores the max margin-box distance from alignment-context end to last baseline.
- Items with auto margins in the block axis, out-of-flow items, and intrinsic-cycle synthesized-baseline items must not participate. If margin-auto state is no longer available after resolution, carry booleans into `PendingGridItem`.

- [ ] Update `PendingGridItem` to store both baseline facts.

Expected fields:

```rust
pub(super) first_baseline: Scalar,
pub(super) last_baseline: Scalar,
pub(super) block_auto_margins: bool,
pub(super) baseline_participation: BaselineParticipation,
```

Calculate with explicit synthesis:

```rust
let baselines = output.baselines();
let first_baseline = baselines.first_or_synthesize_block(output.size);
let last_baseline = baselines.last_or_synthesize_block(output.size);
let major_baseline = vertical_axis.margin_start + first_baseline;
let minor_baseline = vertical_axis.margin_end + output.size.height - last_baseline;
```

Store raw border-box baselines and derived margin-box geometry separately. Do not put margins into `first_baseline` or `last_baseline`.

- [ ] Run:

```bash
cargo test -p surgeist --lib row_baselines_choose_first_baseline_for_first_group
cargo test -p surgeist --lib row_baselines_choose_last_baseline_for_last_group
cargo test -p surgeist --test layout -- grid
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/tests.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/lanes.rs
git commit -m "Model grid baseline groups"
```

---

## Task 4: Apply Baseline Offsets During Grid Child Placement

**Files:**
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] Add focused layout tests where two same-row items align to first baseline.

Add a test in `crates/surgeist/tests/layout/grid.rs` using the existing test tree helpers:

```rust
#[test]
fn grid_aligns_items_to_shared_first_baseline() {
    let mut tree = tree(
        node(display_grid())
            .style(|s| {
                s.size.width = Dimension::Px(120.0);
                s.grid_template_columns = vec![TrackComponent::Track(TrackSizing::fixed(60.0))];
                s.grid_template_rows = vec![TrackComponent::Track(TrackSizing::fixed(40.0))];
                s.align_items = Some(AlignItems::Baseline);
            })
            .child(leaf_with_measure(30.0, 20.0, Some(8.0), None))
            .child(leaf_with_measure(30.0, 30.0, Some(14.0), None)),
    );

    compute_root(&mut tree, 0, Size::new(Available::Definite(120.0), Available::Definite(80.0)));

    assert_eq!(tree.final_layout(1).location.y, 6.0);
    assert_eq!(tree.final_layout(2).location.y, 0.0);
}
```

Use the existing helper names in `tests/layout/grid.rs`; if no leaf helper can provide baselines yet, add one to the test support module rather than hard-coding production behavior.

- [ ] Add a focused layout test where two same-row items align to last baseline.

Expected relationship:

```text
item A height 20 last baseline 4 from bottom
item B height 30 last baseline 10 from bottom
shared last-baseline group = max(4, 10) = 10
in a 40px row, item A top offset = 40 - (10 - 4) - 20 = 14
in a 40px row, item B top offset = 40 - (10 - 10) - 30 = 10
```

- [ ] Add focused layout tests for margins and spanning rows.

Required cases:

```text
first baseline with margin-top 3 and margin-bottom 5
last baseline with margin-top 3 and margin-bottom 5
first baseline spanning two rows with a 7px row gap
last baseline spanning two rows with a 7px row gap
```

Each assertion should verify the final item `location.y`, not only the computed group value.

- [ ] Implement baseline offset computation in `layout_grid_children`.

Do not inline one-off formulas at the placement site. Call the helper from the Baseline Coordinate Convention section.

Expected usage:

```rust
let baseline_offset = item
    .baseline_participation
    .group
    .and_then(|group_kind| {
        let shared = groups.shared_baseline(group_kind, item.area)?;
        Some(baseline_offset(group_kind, shared, item.baseline_geometry(rows, gap.height)))
    });
```

Expected geometry helper:

```rust
fn spanned_track_size(tracks: &[Scalar], start: usize, end: usize, gap: Scalar) -> Scalar {
    let track_sum = tracks[start..end].iter().copied().sum::<Scalar>();
    let gap_sum = gap * end.saturating_sub(start + 1) as Scalar;
    track_sum + gap_sum
}
```

The helper must be tested directly:

```rust
fn baseline_aligned_block_offset(
    item: &PendingGridItem<impl Copy>,
    groups: &GridBaselineGroups,
    rows: &[Scalar],
    row_gap: Scalar,
) -> Option<Scalar>
```

The helper must cover:

- first baseline, single-row item
- first baseline, spanning item
- last baseline, single-row item
- last baseline, spanning item
- first and last baseline with nonzero margins
- no group baseline returns `None`

- [ ] Preserve existing non-baseline alignment behavior. `AlignItems::Baseline` and `AlignItems::LastBaseline` should fall back to the normal axis offset when no group baseline exists.

- [ ] Preserve WebKit participation behavior. Out-of-flow items, block-axis auto-margin items, and intrinsic-cycle synthesized-baseline items should use their fallback alignment instead of baseline offsets.

- [ ] Run:

```bash
cargo test -p surgeist --test layout -- grid_aligns_items_to_shared_first_baseline
cargo test -p surgeist --test layout -- grid_aligns_items_to_shared_last_baseline
cargo test -p surgeist --lib baseline_aligned_block_offset
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
cargo fmt --check
```

Expected parity command: may still fail on broader subgrid/inline issues; record the failure count and top failure buckets in the commit message body or a short note in the working log. The focused tests must pass.

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/tests.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Align grid items by baselines"
```

---

## Task 5: Include Baseline Shim In Track Sizing Contributions

**Files:**
- Modify `crates/surgeist/src/layout/grid/tracks.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] Add a failing layout test where first-baseline alignment increases an auto row's required size.

Scenario:

```text
grid auto row
item A: height 20, first baseline 18
item B: height 30, first baseline 6
shared first baseline = 18
item B needs 12px top shim; row must fit 12 + 30 = 42
```

Expected assertion:

```rust
assert_eq!(tree.final_layout(0).size.height, 42.0);
assert_eq!(tree.final_layout(2).location.y, 12.0);
```

- [ ] Add a failing layout test where last-baseline alignment increases an auto row's required size.

Scenario:

```text
grid auto row
item A: height 20, last baseline 2 from bottom
item B: height 30, last baseline 12 from bottom
shared last-baseline distance = 12
item A needs 10px bottom shim; row must fit 20 + 10 = 30, which ties item B
```

Use a case where the shim changes the row size by at least 1px.

- [ ] Add failing tests for participation fallbacks during track sizing.

Required cases:

```text
absolute positioned child with align-self: baseline does not affect row baseline shim
child with auto block-axis margin and align-self: baseline does not affect row baseline shim
baseline-less child spanning an intrinsic row uses fallback alignment and does not create a baseline shim
baseline-less child in a fixed row may synthesize for final placement but still does not grow intrinsic track sizing
```

These tests should fail before the `BaselineParticipation` gating is wired into track sizing.

- [ ] Add a small production helper that computes baseline shim facts without requiring final child placement.

Expected type:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BaselineShim {
    pub(super) before: Scalar,
    pub(super) after: Scalar,
}
```

Expected helper:

```rust
fn baseline_shim_for_intrinsic_contribution(
    participation: BaselineParticipation,
    geometry: BaselineGeometry,
    shared: TrackBaselineGroup,
) -> BaselineShim
```

Rules:

- Nonparticipants return zero.
- Major/first-baseline shim grows before the item: `shared.first - geometry.major_baseline`, clamped at zero.
- Minor/last-baseline shim grows after the item: `shared.last - geometry.minor_baseline`, clamped at zero.
- Synthesized-baseline intrinsic-cycle fallbacks return zero and use fallback alignment later in placement.

- [ ] Thread baseline shim into row intrinsic contribution sizing.

Use WebKit as the conceptual reference: `GridTrackSizingAlgorithm::baselineOffsetForGridItem` is added to item logical size contributions. In Surgeist, the exact insertion point is where `row_intrinsic_sizes` and row track lower bounds are computed in `resolve_grid_track_sizes`.

Expected contribution shape:

```rust
let contribution = item_contribution + baseline_shim.before + baseline_shim.after;
```

Do not add baseline shim to fixed-size track breadth unless the existing track sizing code already treats the item contribution as a lower bound for that track.

- [ ] Keep the row-only limitation explicit in code comments.

Comment to add near the helper:

```rust
// Surgeist currently lays out horizontal writing mode only. Column-axis
// baseline groups use the same data model but are not applied until vertical
// writing-mode grid tests are introduced.
```

- [ ] Run:

```bash
cargo test -p surgeist --test layout -- grid_baseline_increases_auto_row_size
cargo test -p surgeist --test layout -- grid_last_baseline_increases_auto_row_size
cargo test -p surgeist --lib baseline_shim_for_intrinsic_contribution
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_oracle
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/tests.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Size grid rows with baseline shims"
```

---

## Task 6: Report Grid Container First And Last Baselines

**Files:**
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] Add failing tests for container baseline reporting.

First-baseline selection rule:

- Prefer the first major row baseline group on the first occupied row, matching Blink's accumulator and WebKit's baseline-aligned grid-item preference.
- If no major group exists there, use the first grid-order item in the first occupied row that participates in first-baseline alignment and whose shared context is that first row.
- If no baseline-aligned item qualifies, use the first grid-order item in the first occupied row.
- Synthesize from border-box bottom when the child has no first baseline.

Last-baseline selection rule:

- Prefer the last minor row baseline group on the last occupied row, matching Blink's accumulator and WebKit's last-baseline item preference.
- If no minor group exists there, use the last grid-order item in the last occupied row that participates in last-baseline alignment and whose shared context is that last row.
- If no last-baseline-aligned item qualifies, use the last grid-order item that occupies the last row.
- For spanning fallback items, order last-baseline fallback candidates by their occupied end row and end column, not by their start row and start column. This matches WebKit's last occupied row scan and Blink's end-line accumulator shape.
- Synthesize from border-box top when the child has no last baseline.

Add tests:

```rust
#[test]
fn grid_reports_first_baseline_from_first_row_grid_order() {
    /* root grid with two rows; first row child baseline 7; second row child baseline 9 */
    assert_eq!(tree.compute_output(0).first_baselines.y, Some(7.0));
}

#[test]
fn grid_reports_last_baseline_from_last_row_grid_order() {
    /* root grid with two rows; last row child last baseline 22 at row offset 40 */
    assert_eq!(tree.compute_output(0).last_baselines.y, Some(62.0));
}

#[test]
fn grid_reports_first_baseline_from_shared_major_group_before_fallback_item() {
    /* first row shared major group is 14px from row start; first child fallback baseline is 8px */
    assert_eq!(tree.compute_output(0).first_baselines.y, Some(14.0));
}

#[test]
fn grid_reports_last_baseline_from_shared_minor_group_before_fallback_item() {
    /* last row starts at 40px, height is 30px, shared minor group is 6px from row end */
    assert_eq!(tree.compute_output(0).last_baselines.y, Some(64.0));
}

#[test]
fn grid_reports_last_baseline_from_spanning_item_that_occupies_last_row() {
    /*
    two-row grid; item A starts in row 2 with last baseline at 54;
    item B starts in row 1, spans through row 2, and has last baseline at 72.
    No shared minor group exists, so the spanning item wins because its occupied
    end row is the last row.
    */
    assert_eq!(tree.compute_output(0).last_baselines.y, Some(72.0));
}
```

Use existing test-tree helpers; if they do not expose root `ComputeOutput`, add a test-only accessor in the test support tree rather than modifying `NodeOutput`.

- [ ] Replace `first_grid_baseline` with `grid_container_baselines`.

Expected return type:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct GridContainerBaselines {
    pub(super) first: Option<Scalar>,
    pub(super) last: Option<Scalar>,
}
```

Expected function:

```rust
fn grid_container_baselines<Node>(
    items: &[PendingGridItem<Node>],
    groups: &GridBaselineGroups,
    row_offsets: &[Scalar],
    rows: &[Scalar],
) -> GridContainerBaselines
```

Implementation order:

1. Find the first and last occupied row set.
2. If the first occupied row has a major group baseline, return `row_offsets[row] + group.first`.
3. If the last occupied row has a minor group baseline, return `row_offsets[row] + rows[row] - group.last`.
4. Fall back to first-baseline item geometry using start-row/start-column grid order and resolved item block offset plus raw border-box baseline.
5. Fall back to last-baseline item geometry using occupied end-row/end-column grid order and resolved item block offset plus raw border-box baseline.

If final item block offsets are not stored before calling the function, carry the resolved block offset in `PendingGridItem` after baseline placement and compute container baselines after final offsets are known.

- [ ] Update `GridChildrenLayout` to carry both baselines.

Expected shape:

```rust
pub(super) struct GridChildrenLayout {
    pub(super) visible_content_size: Size,
    pub(super) first_baseline: Option<Scalar>,
    pub(super) last_baseline: Option<Scalar>,
}
```

- [ ] Update `compute_grid_with_context` and `compute_grid_lanes_with_context` to emit both first and last baselines.

Expected call:

```rust
ComputeOutput::from_sizes_and_baselines(
    output_size,
    content_size,
    Baselines {
        first: Point::new(None, child_layout.first_baseline),
        last: Point::new(None, child_layout.last_baseline),
    },
)
```

- [ ] Run:

```bash
cargo test -p surgeist --test layout -- grid_reports_first_baseline_from_first_row_grid_order
cargo test -p surgeist --test layout -- grid_reports_last_baseline_from_last_row_grid_order
cargo test -p surgeist --test layout -- grid_reports_first_baseline_from_shared_major_group_before_fallback_item
cargo test -p surgeist --test layout -- grid_reports_last_baseline_from_shared_minor_group_before_fallback_item
cargo test -p surgeist --test layout -- grid_reports_last_baseline_from_spanning_item_that_occupies_last_row
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Report grid first and last baselines"
```

---

## Task 7: Propagate Baselines Through Subgrid

**Files:**
- Modify `crates/surgeist/src/layout/grid/subgrid.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/mod.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`
- Modify `crates/surgeist/tests/layout_oracle.rs`

- [ ] Add failing tests for parent baseline inheritance into a direct row subgrid.

Scenario:

```text
outer grid rows: 40px 40px
subgrid spans both rows and is row-subgridded
outer grid has a major row baseline group in the inherited row range
subgrid child baseline-aligns against that inherited group
subgrid child offset should match the parent shared baseline translated into subgrid-local coordinates
```

Expected assertion:

```rust
assert_eq!(subgrid_child.location.y, 6.0);
```

Use a parent row major baseline of `14.0` and a subgrid child first baseline of `8.0`, so the expected translated offset is `14.0 - 8.0 = 6.0`.

- [ ] Add failing tests for descendant baseline publication back to the ancestor grid.

Scenario:

```text
outer grid rows: 40px 40px
subgrid spans both rows and is row-subgridded
leaf inside subgrid participates in first baseline in the second inherited row
outer sibling baseline-aligns in the same outer row
outer row baseline group must include the subgrid descendant's baseline translated into outer row coordinates
```

Expected assertion:

```rust
assert_eq!(outer_sibling.location.y, 9.0);
```

Use a published descendant major baseline of `17.0` in the ancestor row and an outer sibling first baseline of `8.0`, so the sibling offset is `17.0 - 8.0 = 9.0`.

- [ ] Add failing tests for reversed and adjusted subgrid cases.

Required cases:

```text
RTL or opposite-direction row subgrid reverses inherited baseline group order
subgrid margin/border/padding shifts inherited baseline coordinates exactly once
subgrid gap differs from parent gap and translates descendant publication across internal edges
baseline-less descendant uses synthesized fallback only when it does not create an intrinsic cycle
```

- [ ] Add failing tests for direct column-subgrid baseline data preservation.

Column-axis application may stay disabled for horizontal writing mode, but the subgrid context should carry inherited column baseline groups without dropping them.

Expected assertion:

```rust
assert_eq!(context.columns.unwrap().major_baselines.len(), 2);
assert_eq!(context.columns.unwrap().minor_baselines.len(), 2);
```

Keep this as a unit test in `grid/subgrid.rs` if easier than a full layout test.

- [ ] Extend `InheritedGridAxis` to carry baseline groups.

Expected shape:

```rust
struct InheritedGridAxis {
    offset: Scalar,
    gap: Scalar,
    tracks: Vec<Scalar>,
    major_baselines: Vec<Option<Scalar>>,
    minor_baselines: Vec<Option<Scalar>>,
}
```

- [ ] Implement parent track baseline inheritance into subgrid layout.

Use WebKit's `RenderGrid::columnAxisBaselineOffsetForGridItem` / `rowAxisBaselineOffsetForGridItem` as the behavior reference: when an item is in a subgridded axis, the baseline offset routes through the outer grid's corresponding axis. Use Blink's `CreateSubgridBaselines` as the data-shape reference: parent track range, gutter, margins, border/padding, track direction, and opposite-direction state all affect the child baseline collection.

Expected helper:

```rust
fn inherit_subgrid_baselines(
    parent_major: &[Option<Scalar>],
    parent_minor: &[Option<Scalar>],
    parent_span: GridTrackSpan,
    reversed: bool,
    parent_gap: Scalar,
    subgrid_gap: Scalar,
    start_mbp: Scalar,
    end_mbp: Scalar,
) -> InheritedBaselineGroups
```

Rules:

- Slice parent major/minor arrays over the inherited parent span.
- Reverse group order when inherited tracks are reversed.
- Preserve major/minor meaning relative to the resolved subgrid axis.
- Adjust start/end baselines by subgrid margin/border/padding exactly once.
- Adjust internal baseline coordinates for signed parent/subgrid gap differences using the same edge math as inherited track sizing: compute `(subgrid_gap - parent_gap) / 2` and subtract that signed value from both sides of each internal edge. Positive values reduce inherited baseline coordinates; negative values increase them.
- Preserve `None` entries; do not synthesize parent track baselines while inheriting.

Add tests:

```rust
#[test]
fn subgrid_baselines_apply_negative_gap_difference_to_internal_edges() {
    /*
    parent gap 20, subgrid gap 10 => signed half difference -5.
    A two-track inherited row baseline group should add 5 to coordinates
    adjacent to the internal edge after MBP adjustment, matching the oracle.
    */
    let rows = context.rows.as_ref().unwrap();
    assert_eq!(rows.major_baselines, vec![Some(18.0), Some(25.0)]);
    assert_eq!(rows.minor_baselines, vec![Some(10.0), Some(25.0)]);
}
```

- [ ] Implement descendant baseline publication back to the ancestor grid.

Expected helper:

```rust
fn publish_subgrid_descendant_baseline(
    descendant: DescendantBaseline,
    subgrid_area: GridArea,
    subgrid_offset: Scalar,
    inherited_axis: &InheritedGridAxis,
) -> PublishedBaseline
```

Rules:

- Translate descendant row/column baseline context into the ancestor's track index.
- Add the subgrid's resolved grid-area offset and inherited axis offset.
- Account for reversed inherited axis.
- Account for gap differences already applied in inherited track coordinates; do not apply them a second time.
- If the descendant baseline was synthesized and would create an intrinsic sizing cycle, publish no baseline and use fallback alignment.

- [ ] Add baseline groups into child grid context.

Expected context shape:

```rust
#[derive(Clone, Debug)]
struct GridParentContext {
    columns: Option<InheritedGridAxis>,
    rows: Option<InheritedGridAxis>,
}
```

Existing type already has this shape; extend its axis value only.

- [ ] During subgrid child layout, merge descendant baseline facts back into the parent pending item when the child is a subgrid in that axis.

Expected behavior:

- A row-subgrid child exposes first/last baselines in the parent's row coordinate space.
- If the subgrid has no qualifying descendant baseline, fall back to the subgrid container's own synthesized baseline.
- Do not double-apply the subgrid's margin.

- [ ] Run:

```bash
cargo test -p surgeist --lib subgrid
cargo test -p surgeist --test layout -- subgrid
cargo test -p surgeist --test layout -- subgrid_baselines_apply_negative_gap_difference_to_internal_edges
cargo test -p surgeist --test layout -- baseline
cargo test -p surgeist --test layout_oracle
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/subgrid.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/mod.rs crates/surgeist/src/layout/grid/tests.rs crates/surgeist/tests/layout/grid.rs crates/surgeist/tests/layout_oracle.rs
git commit -m "Propagate subgrid baselines"
```

---

## Task 8: Define Grid-Lanes Baseline Fallbacks

**Files:**
- Modify `crates/surgeist/src/layout/grid/lanes.rs`
- Modify `crates/surgeist/src/layout/grid/child.rs`
- Modify `crates/surgeist/src/layout/grid/tests.rs`
- Modify `crates/surgeist/tests/layout/grid.rs`

- [ ] Add tests matching the current intended support boundary.

WebKit currently skips masonry baseline calculations in `RenderGrid::columnAxisBaselineOffsetForGridItem` and `rowAxisBaselineOffsetForGridItem` when masonry is detected. Surgeist should do the same for grid-lanes until the grid-lanes baseline behavior is deliberately designed.

Add tests:

```rust
#[test]
fn grid_lanes_does_not_apply_lane_axis_baseline_offsets() {
    /* grid-lanes column flow; lane-axis placement ignores baseline offsets */
    assert_eq!(tree.final_layout(child_a).location.y, 0.0);
    assert_eq!(tree.final_layout(child_b).location.y, 30.0);
}

#[test]
fn grid_lanes_still_reports_synthesized_container_baselines() {
    /* first child height 20 at y=0; last child height 30 at y=30 */
    assert_eq!(output.first_baselines.y, Some(20.0));
    assert_eq!(output.last_baselines.y, Some(30.0));
}
```

- [ ] Make the fallback explicit in `lanes.rs`.

Add a comment near `layout_grid_lanes_children`:

```rust
// WebKit currently skips masonry baseline offset calculations. Surgeist keeps
// grid-lanes baseline offsets disabled for lane-axis placement, but still
// reports synthesized container baselines from final item geometry.
```

- [ ] Ensure `GridChildrenLayout` returned from `layout_grid_lanes_children` carries first/last container baselines based on final item geometry.
- [ ] Use the same fallback ordering as ordinary grid container baselines: first baseline uses start-row/start-column order, while last baseline uses occupied end-row/end-column order so spanning lane items can supply the last container baseline from the row they occupy.

Add test:

```rust
#[test]
fn grid_lanes_reports_last_baseline_from_spanning_item_end_edge() {
    /*
    grid-lanes final geometry has item A starting later but ending earlier,
    and item B starting earlier while spanning to the last occupied lane row.
    The container last baseline must come from item B's final block offset
    plus explicit or synthesized last baseline.
    */
    assert_eq!(output.last_baselines.y, Some(96.0));
}
```

- [ ] Run:

```bash
cargo test -p surgeist --test layout -- grid_lanes
cargo test -p surgeist --test layout -- baseline
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout -- grid_lanes_reports_last_baseline_from_spanning_item_end_edge
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/grid/lanes.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/tests.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Document grid-lanes baseline fallback"
```

---

## Task 9: Fix Browser Parity Harness Baseline Cases

**Files:**
- Modify `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
- Modify or add `crates/surgeist/tests/layout_browser_parity/README.md`

**Superseded corpus rule:** The parity corpus consolidation plan replaces the
older hand-edited XML workflow. Generated XML is output only; do not correct
browser fail-list values by editing XML. Known browser-bad captures or bad WPT
references must be represented through source fixtures, manifests, expected
failure/quarantine metadata, or generator reports, then regenerated with
`surgeist-layout-generate`.

- [ ] Confirm the harness can parse all currently checked-in subgrid XML.

Run:

```bash
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Expected: pass.

- [ ] Add focused parser tests for:

- `align-items="last baseline"`
- `align-self="first baseline"`
- `display="inline-grid"`
- `display="inline-block"`
- `grid-template-columns="subgrid [a] [b]"`

Expected behavior:

- `first baseline` parses as `layout::AlignItems::Baseline`.
- `last baseline` parses as `layout::AlignItems::LastBaseline`.
- Inline display parsing remains explicitly documented as a parity harness simplification until inline formatting context support exists.
- Inline formatting context differences are assigned to an explicit parity bucket such as `UnsupportedInlineFormattingContext`; they are not counted as grid baseline mismatches.
- `subgrid` parser preserves line names and does not create a fake auto track.

- [ ] Add a parity classification for inline formatting context differences.

Expected behavior:

```text
fixtures containing inline-grid or inline-block parse successfully
fixtures whose expected rectangles require inline formatting context behavior are reported as UnsupportedInlineFormattingContext
baseline-specific fixture regeneration excludes this bucket
```

Do not regenerate inline-related XML expectations under this baseline task. If an inline fixture also exercises baseline behavior, keep it as evidence for the future inline formatting plan and add a non-inline focused baseline fixture for this task.

- [ ] Keep generator helper injection from the current working tree if it remains necessary.

Expected generator behavior:

```rust
if typeof getTestData !== "function" {
    inject tests/layout_browser_parity/scripts/gentest/test_helper.js
}
```

This supports older WPT-derived standalone fixtures without editing every HTML file.

- [ ] Regenerate only the targeted non-inline subgrid baseline fixtures after baseline parsing is correct.

Run:

```bash
SURGEIST_LAYOUT_GENERATE_FILTER=subgrid cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
```

After generation, inspect changed files before staging:

```bash
git diff --name-only -- crates/surgeist/tests/layout_browser_parity
```

Only keep files that are not in the inline formatting bucket and are relevant to baseline/subgrid behavior.

- [ ] If a browser fail-list fixture produces incorrect generated values,
      capture the reason in the corpus manifest/reporting path and regenerate.
      Treat generated XML as output only.

- [ ] Run:

```bash
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
SURGEIST_PARITY_FILTER=subgrid cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
cargo fmt --check
```

- [ ] Commit:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/README.md
git commit -m "Update subgrid parity baselines"
```

Before the commit, stage only reviewed source fixture, manifest, report, and
generated-output paths that are produced by the repeatable generator. Do not use
hand-edited XML as the correction mechanism.

---

## Task 10: Full Verification And Clippy

**Files:**
- No source files expected unless verification finds issues.

- [ ] Run focused checks:

```bash
cargo test -p surgeist --lib
cargo test -p surgeist --test layout -- grid
cargo test -p surgeist --test layout_oracle
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
SURGEIST_PARITY_FILTER=subgrid cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored
```

- [ ] Run broad checks:

```bash
cargo test -p surgeist
cargo clippy -p surgeist --all-targets --all-features -- -D warnings
cargo fmt --check
```

- [ ] If any command fails, fix the smallest relevant source slice and rerun the failed command plus its nearest focused predecessor.

- [ ] Commit fixes if any were needed:

```bash
git status --short --branch
git diff --check
git add crates/surgeist/src/layout/output.rs crates/surgeist/src/layout/grid/child.rs crates/surgeist/src/layout/grid/tracks.rs crates/surgeist/tests/layout/grid.rs
git commit -m "Fix baseline verification issues"
```

Adjust the `git add` file list to the exact verification-fix files before running it; the listed paths are the most likely files for this plan's final verification fixes.

---

## Clean-Context Review Requirement

Before marking this goal complete:

- [ ] Dispatch a clean-context reviewer with this plan, the existing engine plan, and the WebKit/Blink reference file list.
- [ ] Ask the reviewer to check:
  - Whether the plan correctly preserves existing grid/subgrid/grid-lanes work.
  - Whether first/last baseline semantics match WebKit/Blink closely enough for Surgeist's current horizontal writing-mode scope.
  - Whether subgrid baseline propagation is described at the right level of detail.
  - Whether inline-grid/inline-block is correctly treated as a separate inline formatting concern rather than hidden under baseline work.
  - Whether the test strategy is broad enough and ordered correctly.
- [ ] Implement every accepted recommendation in this plan file before marking the goal complete.
- [ ] If a recommendation is rejected, record the reason in a short "Reviewer Notes" section at the bottom of this file.

---

## Self-Review

- Spec coverage: This plan covers the engine side of baseline support missing from the prior grid/subgrid/grid-lanes implementation plan. It does not replace placement, track inheritance, or lane placement work already completed/planned.
- Placeholder scan: No task uses empty future-work labels or generic test instructions without concrete test scenarios and expected commands.
- Type consistency: `Baseline` means CSS first baseline; `LastBaseline` is explicit. `ComputeOutput` keeps `first_baselines` for compatibility and adds `last_baselines`.
- Scope boundary: Full inline formatting context layout is not included. The plan requires explicit documentation and parser support so parity failures are not hidden by accidental display collapsing.
- Oracle completion delta: The completed oracle corrected two edge cases that this engine plan must preserve: signed subgrid gap differences, including negative gap differences that increase coordinates, and last-baseline fallback ordering by occupied end row/end column for spanning ordinary-grid and grid-lanes items.
