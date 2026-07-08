# Line Break Clear Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Apply resolved `Clear` semantics to horizontal forced line breaks using the existing block float exclusion model.

**Architecture:** This implements Phase 3 from `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`. `LineBreakInputOf<S>::clear()` is already captured in `ForcedLineBreakControlOf<S>`; this plan makes block flow honor it by centralizing visible horizontal line-break classification, segmenting horizontal atomic inline runs only when a clear-bearing break can move the following line, laying out the segment through the break, applying `FloatExclusions::clearance_y`, and then continuing below the relevant floats. The change stays inside layout calculation: no HTML `clear` parsing, CSS lowering, fixture generation, vertical writing support, public API exposure, or text/font metric derivation.

**Tech Stack:** Rust 2024, `surgeist-layout`, existing `LayoutScalar` APIs, existing `FloatExclusions`, crate-local block tests, Cargo test/clippy/fmt.

---

## Source References

- Specification: `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`
- Sequencing: `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`
- Previous plan: `plans/2026-07-08-surgeist-layout-inline-control-contract-implementation.md`
- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Workflow: `AGENTS.md`

## Scope

This plan does:

- apply `Clear::Left`, `Clear::Right`, and `Clear::Both` for visible horizontal line-break nodes;
- preserve `Clear::None` behavior for existing inline runs;
- preserve no-op clear behavior when the requested side has no relevant active float;
- use the existing `FloatExclusions::clearance_y` machinery;
- keep line-break output zero-size;
- keep resolved clear as layout-ready input, not authored HTML/CSS syntax.

This plan does not:

- parse HTML `clear` attributes or CSS declarations;
- implement vertical line-break clear behavior;
- implement vertical forced-break layout;
- change browser parity HTML/XML generation;
- expose new public APIs;
- change font, text, inline metrics, or `vertical-align` behavior.

## Files

- Modify: `src/block.rs`
  - Add one centralized helper for visible horizontal line-break classification.
  - Reuse that helper in existing line-break branches and new clear segmentation.
  - Segment only when `LineBreakInputOf<S>::clear() != Clear::None` and existing float exclusions will move the following line.
  - Apply `FloatExclusions::clearance_y` after the segment that contains the clear-bearing break, including when the break is at the end of the atomic inline run.
- Modify: `src/block_tests.rs`
  - Add focused tests for relevant left/right/both clears.
  - Add run-end clear coverage.
  - Add no-op side and alignment preservation coverage.
  - Preserve existing hidden and vertical line-break tests.

No `src/inline.rs` change is required for this plan.

## Semantics

For a visible horizontal line break with `clear != Clear::None` and a relevant active float:

- inline content before the line break stays in the current atomic inline segment;
- the line-break node itself remains in that segment and receives its existing zero-size output at its committed insertion point;
- after the segment is placed, block flow advances `cursor_y` to the segment bottom;
- block flow applies `FloatExclusions::clearance_y(cursor_y, clear)`;
- following inline content or the next normal-flow sibling starts at the cleared `cursor_y`.

For `Clear::None`, existing behavior remains unchanged: the full atomic inline run is laid out as one run, and forced breaks inside that run split lines internally.

For `Clear::Left` with only right floats, or `Clear::Right` with only left floats, existing behavior remains unchanged. The implementation must not segment the run when clearance cannot move the following line, because segmenting can alter text alignment or RTL placement.

For vertical line breaks, preserve the existing explicit panic:

```rust
panic!("vertical line-break layout is not implemented");
```

## Task 1: Add Focused Block Tests

**Files:**
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Add a helper for line-break clear test trees**

Add this helper near the existing line-break block tests:

```rust
fn inline_break_clear_tree(
    clear: Clear,
    float_side: Float,
) -> crate::test_support::layout_tree::OracleTree {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: float_side,
                size: Size::new(Dimension::px(80.0), Dimension::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(3, LineBreakInput::new().with_clear(clear).with_metrics(metrics))
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
}
```

- [ ] **Step 2: Add behavior tests**

Add these tests near the helper:

```rust
#[test]
fn line_break_clear_left_moves_following_inline_segment_below_left_float() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Left);

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(20.0, 10.0));
    assert_eq!(tree.final_layout(3).unwrap().size, Size::ZERO);
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(0.0, 50.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_right_moves_following_inline_segment_below_right_float() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Right);

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(20.0, 10.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(0.0, 50.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_both_uses_greater_left_or_right_float_bottom() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4, 5])
        .style(0, NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        })
        .style(1, NodeInput {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(Dimension::px(60.0), Dimension::px(30.0)),
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::Block,
            float: Float::Right,
            size: Size::new(Dimension::px(60.0), Dimension::px(70.0)),
            ..NodeInput::DEFAULT
        })
        .style(3, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        })
        .line_break(4, LineBreakInput::new().with_clear(Clear::Both).with_metrics(metrics))
        .style(5, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(20.0, 10.0));
    assert_eq!(tree.final_layout(5).unwrap().location, Point::new(0.0, 70.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 80.0));
}

#[test]
fn line_break_clear_at_run_end_moves_following_block_below_float() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(0, NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        })
        .style(1, NodeInput {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(Dimension::px(80.0), Dimension::px(50.0)),
            ..NodeInput::DEFAULT
        })
        .style(2, NodeInput {
            display: Display::InlineBlock,
            size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        })
        .line_break(3, LineBreakInput::new().with_clear(Clear::Left).with_metrics(metrics))
        .style(4, NodeInput {
            display: Display::Block,
            size: Size::new(Dimension::px(25.0), Dimension::px(10.0)),
            ..NodeInput::DEFAULT
        });

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(20.0, 10.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(0.0, 50.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 60.0));
}

#[test]
fn line_break_clear_left_ignores_right_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Left, Float::Right).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyRight,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(180.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(200.0, 10.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(180.0, 10.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_right_ignores_left_float_and_preserves_alignment() {
    let mut tree = inline_break_clear_tree(Clear::Right, Float::Left).style(
        0,
        NodeInput {
            display: Display::Block,
            text_align: TextAlign::LegacyCenter,
            size: Size::new(Dimension::px(200.0), Dimension::AUTO),
            ..NodeInput::DEFAULT
        },
    );

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(90.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(110.0, 10.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(90.0, 10.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_that_is_noop_after_line_height_preserves_alignment() {
    let metrics = InlineMetrics::from_line_height_and_baseline(10.0, 10.0).unwrap();
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(Dimension::px(200.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(Dimension::px(80.0), Dimension::px(5.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(20.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .line_break(3, LineBreakInput::new().with_clear(Clear::Left).with_metrics(metrics))
        .style(
            4,
            NodeInput {
                display: Display::InlineBlock,
                size: Size::new(Dimension::px(15.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(180.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(200.0, 10.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(180.0, 10.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}

#[test]
fn line_break_clear_none_preserves_existing_single_run_layout_near_float() {
    let mut tree = inline_break_clear_tree(Clear::None, Float::Left);

    compute_root(&mut tree, 0, Size::new(Available::definite(200.0), Available::MAX_CONTENT));
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(20.0, 10.0));
    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(0.0, 10.0));
    assert_eq!(tree.final_layout(0).unwrap().size, Size::new(200.0, 20.0));
}
```

- [ ] **Step 3: Run the red tests**

Run:

```sh
cargo test -p surgeist-layout line_break_clear_ -- --nocapture
```

Expected:

- tests with relevant floats fail because following content still starts at the normal next line instead of below relevant floats;
- no-relevant-float and `Clear::None` preservation tests pass.

Do not change expected values to match current failing behavior.

## Task 2: Segment Atomic Inline Runs At Moving Clear Breaks

**Files:**
- Modify: `src/block.rs`

- [ ] **Step 1: Add centralized visible horizontal line-break classification**

Add this helper near `atomic_inline_run_end`:

```rust
fn visible_horizontal_line_break<Tree>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
) -> Option<LineBreakInputOf<<Tree as Compute>::Scalar>>
where
    Tree: Compute,
{
    let LayoutInputOf::LineBreak(line_break) = tree.layout_input(child) else {
        return None;
    };
    if line_break.display().is_none() {
        return None;
    }
    if line_break.writing_mode() != WritingMode::HorizontalTb {
        panic!("vertical line-break layout is not implemented");
    }
    Some(line_break)
}
```

Use this helper anywhere new code needs to classify visible horizontal line breaks. Where existing code must also write hidden line-break output, keep the hidden branch before calling this helper.

Also update the existing line-break branches in `atomic_inline_run_end`,
`layout_in_flow_children`, and `layout_atomic_inline_run` to call
`visible_horizontal_line_break` after their hidden-line-break handling instead
of repeating the writing-mode check inline. The intent is one visible-horizontal
classification helper, not a third interpretation point.

- [ ] **Step 2: Add clear candidate helpers**

Add these helpers near `atomic_inline_run_end`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AtomicInlineClearCandidate {
    end: usize,
    clear: Clear,
}

fn next_atomic_inline_clear_candidate<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    start: usize,
    run_end: usize,
) -> Option<AtomicInlineClearCandidate>
where
    Tree: Compute,
{
    for index in start..run_end {
        if let Some(line_break) = visible_horizontal_line_break(tree, children[index]) {
            let clear = line_break.clear();
            if clear != Clear::None {
                return Some(AtomicInlineClearCandidate {
                    end: index + 1,
                    clear,
                });
            }
        }
    }
    None
}

fn atomic_inline_run_contains_clear<Tree>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
    run_start: usize,
    run_end: usize,
) -> bool
where
    Tree: Compute,
{
    next_atomic_inline_clear_candidate(tree, children, run_start, run_end).is_some()
}
```

These helpers must not inspect HTML source tags, fixture attributes, style crate types, or retained crate types.

- [ ] **Step 3: Add a segmented inline run placement helper**

Add this helper near `layout_atomic_inline_run`:

```rust
struct AtomicInlineSegmentsContext<'a, S: LayoutScalar> {
    order_start: u32,
    cursor_y: S,
    constants: &'a Constants<S>,
    input: ComputeInputOf<S>,
    node_inner_size: Size<Option<S>>,
    set_layout: bool,
}

fn layout_atomic_inline_segments<Tree, S>(
    tree: &mut Tree,
    run: &[<Tree as Traverse>::Node],
    context: AtomicInlineSegmentsContext<'_, S>,
    float_exclusions: &FloatExclusions<S>,
) -> InlineRunPlacement<<Tree as Traverse>::Node, S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    let AtomicInlineSegmentsContext {
        order_start,
        mut cursor_y,
        constants,
        input,
        node_inner_size,
        set_layout,
    } = context;
    let mut offset = 0;
    let mut content_size = Size::ZERO;
    let mut static_positions = Vec::new();
    let mut first_baseline = None;
    let mut last_baseline = None;
    let start_y = cursor_y;

    while offset < run.len() {
        let mut segment_end = run.len();
        let mut segment_clear = Clear::None;
        let mut scan_start = offset;
        while let Some(candidate) =
            next_atomic_inline_clear_candidate(tree, run, scan_start, run.len())
        {
            let probe = layout_atomic_inline_run(
                tree,
                &run[offset..candidate.end],
                AtomicInlineRunContext {
                    order_start: order_start + offset as u32,
                    cursor_y,
                    constants,
                    input,
                    node_inner_size,
                    set_layout: false,
                },
            );
            let segment_bottom = cursor_y + probe.size.height;
            if float_exclusions.clearance_y(segment_bottom, candidate.clear) > segment_bottom {
                segment_end = candidate.end;
                segment_clear = candidate.clear;
                break;
            }
            scan_start = candidate.end;
        }

        let placement = layout_atomic_inline_run(
            tree,
            &run[offset..segment_end],
            AtomicInlineRunContext {
                order_start: order_start + offset as u32,
                cursor_y,
                constants,
                input,
                node_inner_size,
                set_layout,
            },
        );

        content_size.width = content_size.width.max(placement.content_size.width);
        content_size.height = content_size.height.max(placement.content_size.height);
        static_positions.extend(placement.static_positions);
        if let Some(baseline) = placement.first_baseline {
            first_baseline.get_or_insert(cursor_y - start_y + baseline);
        }
        if let Some(baseline) = placement.last_baseline {
            last_baseline = Some(cursor_y - start_y + baseline);
        }

        cursor_y = cursor_y + placement.size.height;
        if segment_clear != Clear::None {
            cursor_y = float_exclusions.clearance_y(cursor_y, segment_clear);
            content_size.height = content_size
                .height
                .max(cursor_y - constants.content_box_inset.top);
        }
        offset = segment_end;
    }

    InlineRunPlacement {
        size: Size::new(content_size.width, cursor_y - start_y),
        content_size,
        static_positions,
        first_baseline,
        last_baseline,
    }
}
```

The `clear != Clear::None` branch must run even when the clear-bearing line break is the last child in the atomic inline run, because the next normal-flow sibling must start after clearance.

- [ ] **Step 4: Add a clear-aware run wrapper**

Add this helper near `layout_atomic_inline_run`:

```rust
fn layout_atomic_inline_run_with_clear<Tree, S>(
    tree: &mut Tree,
    children: &[<Tree as Traverse>::Node],
    run_start: usize,
    run_end: usize,
    context: AtomicInlineRunContext<'_, S>,
    float_exclusions: &FloatExclusions<S>,
) -> InlineRunPlacement<<Tree as Traverse>::Node, S>
where
    Tree: Compute<Scalar = S>,
    S: LayoutScalar,
{
    if !atomic_inline_run_contains_clear(tree, children, run_start, run_end) {
        return layout_atomic_inline_run(tree, &children[run_start..run_end], context);
    }

    layout_atomic_inline_segments(
        tree,
        &children[run_start..run_end],
        AtomicInlineSegmentsContext {
            order_start: context.order_start,
            cursor_y: context.cursor_y,
            constants: context.constants,
            input: context.input,
            node_inner_size: context.node_inner_size,
            set_layout: context.set_layout,
        },
        float_exclusions,
    )
}
```

If borrow checking requires destructuring `context` before the early return, keep the same direct-run fast path for runs without clear-bearing line breaks. Runs with clear-bearing line breaks may enter the segmented helper, but that helper must preserve alignment by probing each candidate and using a full direct run when no candidate clear moves the post-segment line bottom.

- [ ] **Step 5: Route block inline-run branches through the wrapper**

In both inline-run branches inside `layout_in_flow_children`:

- the branch where the current child is `LayoutInputOf::LineBreak`;
- the branch where `child_style.display.is_inline_level() && child_style.float.is_none()`;

replace direct calls to `layout_atomic_inline_run(...)` with `layout_atomic_inline_run_with_clear(...)`.

Example replacement:

```rust
let placement = layout_atomic_inline_run_with_clear(
    tree,
    children,
    run_start,
    index,
    AtomicInlineRunContext {
        order_start: run_start as u32,
        cursor_y,
        constants,
        input,
        node_inner_size,
        set_layout,
    },
    &float_exclusions,
);
```

Do not remove `layout_atomic_inline_run`; the clear-aware wrapper should reuse it.

## Task 3: Verify Behavior And Boundaries

**Files:**
- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout line_break_clear_ -- --nocapture
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout block_bfc_clear -- --nocapture
```

Expected:

- new line-break clear tests pass;
- existing line-break tests pass, including vertical panic coverage;
- existing block clear tests pass.

- [ ] **Step 2: Search for boundary drift**

Run:

```sh
rg -n "source-tag|HTML|html|font|line-height|surgeist_style|surgeist-style|surgeist_retained|surgeist-retained|clear\\(" src/block.rs src/block_tests.rs
```

Expected:

- no style or retained dependency appears;
- no HTML/source-tag parsing appears;
- `clear(` hits are limited to layout-owned `Clear` APIs and existing `FloatExclusions::clearance_y` usage.

- [ ] **Step 3: Run final checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected:

- tests pass;
- clippy passes with `-D warnings`;
- formatting is clean;
- no whitespace errors;
- git status shows only intentional changes before commit.

## Commit Point

After the worker completes all tasks and the scoped reviewer comes back clean, the coordinator should commit:

```sh
git add src/block.rs src/block_tests.rs
git commit -m "Apply clear to line breaks"
```

Do not commit before the review cycle is clean.

## Review Checklist

The clean-context reviewer should verify:

- implementation matches this plan and Phase 3 of the sequencing document;
- `Clear::None` behavior remains unchanged;
- no-op clear side behavior remains unchanged, including alignment-sensitive cases;
- `Clear::Left`, `Clear::Right`, and `Clear::Both` move following inline content or following normal-flow siblings below relevant floats;
- line-break node output remains zero-size and at its committed insertion point;
- vertical line-break behavior remains explicitly unsupported;
- no HTML/CSS parsing, fixture generation, public API exposure, or text/font metric work was added;
- the implementation reuses existing `FloatExclusions::clearance_y` rather than creating a second clearance model;
- final checks listed above were run and passed.

## Follow-Up Plans

After this plan is implemented and reviewed cleanly, the next derived plan should be Phase 4 from the sequencing document: extract logical inline axes before vertical forced-break support.
