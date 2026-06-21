# Surgeist Block Float BFC Placement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make non-float block formatting context children participate in CSS float exclusion and clearance so the imported WPT `zero-space-between-floats-*` block fixtures pass.

**Architecture:** Extend `crates/surgeist/src/layout/block.rs` with a small float exclusion context used during in-flow block child placement. Existing float children remain collected and laid out through the current `PendingFloat` path, but their margin-box bands are also recorded as active exclusions so later non-float BFC children can find the first horizontal opportunity that fits, including zero-width opportunities, and honor `clear:left` / `clear:right`.

**Tech Stack:** Rust, Surgeist layout engine, existing `OracleTree` layout tests, browser parity XML fixtures.

---

## File Structure

- Modify `crates/surgeist/src/layout/block.rs`: add active float exclusion tracking, placement for block formatting context children next to preceding floats, and clearance support for non-float children.
- Modify `crates/surgeist/tests/layout/block.rs`: add focused TDD tests that reproduce the four WPT block failures without running the full parity corpus.
- Read-only verification target `crates/surgeist/tests/layout_browser_parity.rs`: run the ignored parity harness with `SURGEIST_PARITY_FILTER=xml/wpt/block`.

## Current Failure Evidence

The block-only WPT run currently reports:

```text
16 browser parity fixtures failed
x mismatch: 8
y mismatch: 8
```

All failures are generated variants of:

```text
wpt/block/floats/zero-space-between-floats-001.html
wpt/block/floats/zero-space-between-floats-002.html
wpt/block/floats/zero-space-between-floats-003.html
wpt/block/floats/zero-space-between-floats-004.html
```

Representative fixture:

```html
<div id="container" style="position:relative; width:200px;">
  <div style="float:left; width:100px; height:200px;"></div>
  <div style="float:right; width:100px; height:200px;"></div>
  <div data-offset-x="100" data-offset-y="0" style="overflow:hidden; width:0; height:200px;"></div>
</div>
```

Expected: the zero-width `overflow:hidden` block establishes a BFC and fits in the zero-width opportunity between the left and right floats at `x=100`.

Current: Surgeist places it at `x=0`.

---

### Task 1: Add Focused Failing Tests

**Files:**
- Modify: `crates/surgeist/tests/layout/block.rs`

- [ ] **Step 1: Add focused tests for zero-width BFC placement between floats**

Append these tests near the existing float tests in `crates/surgeist/tests/layout/block.rs`:

```rust
#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats() {
    let mut tree = support::oracle_tree::OracleTree::new()
        .children(0, [1, 2, 3])
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
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(tree.final_layout(2).unwrap().location, Point::new(100.0, 0.0));
    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(100.0, 0.0));
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats_above_full_width_float() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(100.0, 0.0));
}
```

- [ ] **Step 2: Add focused tests for clearance on zero-width BFC children**

Append these tests near the tests from Step 1:

```rust
#[test]
fn block_bfc_zero_width_child_with_clear_left_sits_below_left_float_row() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: Clear::Left,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(0.0, 100.0));
}

#[test]
fn block_bfc_zero_width_child_with_clear_right_sits_below_all_right_floats() {
    let mut tree = support::oracle_tree::OracleTree::new()
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
                float: Float::Left,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(Dimension::percent(1.0), Dimension::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: Clear::Right,
                overflow: Point::new(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(Dimension::px(0.0), Dimension::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    compute_root(
        &mut tree,
        0,
        Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
    );
    round_layout(&mut tree, 0);

    assert_eq!(tree.final_layout(4).unwrap().location, Point::new(0.0, 200.0));
}
```

- [ ] **Step 3: Run tests and verify they fail for the expected reason**

Run:

```sh
cargo test -p surgeist --test layout block_bfc_zero_width_child -- --nocapture
```

Expected: tests fail because the zero-width BFC children are placed at `x=0` or `y=0` rather than the WPT expected coordinates.

- [ ] **Step 4: Commit the red tests**

Run:

```sh
git add crates/surgeist/tests/layout/block.rs
git commit -m "Add float BFC placement regression tests"
```

---

### Task 2: Implement Active Float Exclusion Placement

**Files:**
- Modify: `crates/surgeist/src/layout/block.rs`

- [ ] **Step 1: Add a lightweight active float model**

Add these structs near the existing `PendingFloat` / `FloatPlacement` types in `crates/surgeist/src/layout/block.rs`:

```rust
#[derive(Clone, Copy, Debug)]
struct ActiveFloat {
    side: Float,
    x: Scalar,
    y: Scalar,
    width: Scalar,
    height: Scalar,
}

impl ActiveFloat {
    fn bottom(self) -> Scalar {
        self.y + self.height
    }

    fn overlaps_y(self, y: Scalar) -> bool {
        y >= self.y && y < self.bottom()
    }
}

#[derive(Clone, Debug)]
struct FloatExclusions {
    content_width: Scalar,
    inset: Edges,
    active: Vec<ActiveFloat>,
    placer: FloatPlacement,
}
```

- [ ] **Step 2: Route float placement through `FloatExclusions`**

Add methods that preserve existing float placement but record each placed float band:

```rust
impl FloatExclusions {
    fn new(content_width: Scalar, inset: Edges) -> Self {
        Self {
            content_width,
            inset,
            active: Vec::new(),
            placer: FloatPlacement::new(content_width, inset),
        }
    }

    fn place_float<Node>(&mut self, float: &PendingFloat<Node>) -> Point {
        let location = self.placer.place(float);
        let margin_box = float.size + float.margin.sum_axes();
        self.active.push(ActiveFloat {
            side: float.side,
            x: location.x - float.margin.left,
            y: location.y - float.margin.top,
            width: margin_box.width,
            height: margin_box.height,
        });
        location
    }
}
```

Update `layout_floats` to use `FloatExclusions::place_float` instead of `FloatPlacement::place`.

- [ ] **Step 3: Maintain float exclusions during in-flow layout**

Inside `layout_in_flow_children`, initialize:

```rust
let content_width = inner_width
    .or(input.available.width.into_option())
    .unwrap_or(0.0);
let mut float_exclusions = FloatExclusions::new(content_width, constants.content_box_inset);
```

When a float child is encountered and `pending_floats` is pushed, also place it in `float_exclusions` immediately. Reuse the returned location only for exclusion tracking; keep the existing deferred final `set_unrounded` through `layout_floats`.

- [ ] **Step 4: Add non-float block opportunity search**

Add a method on `FloatExclusions`:

```rust
fn place_bfc_block(
    &self,
    y: Scalar,
    size: Size,
    margin: Edges,
    clear: Clear,
    fallback_x: Scalar,
) -> Point {
    let mut candidate_y = self.clearance_y(y, clear);
    loop {
        let (left_edge, right_edge, next_y) = self.available_band(candidate_y);
        let margin_box_width = size.width + margin.horizontal_sum();
        if margin_box_width <= (right_edge - left_edge).max(0.0) {
            return Point::new(left_edge + margin.left, candidate_y + margin.top);
        }
        if let Some(next_y) = next_y {
            candidate_y = next_y;
        } else {
            return Point::new(fallback_x, candidate_y + margin.top);
        }
    }
}
```

Add helper methods:

```rust
fn clearance_y(&self, y: Scalar, clear: Clear) -> Scalar {
    let clears_left = matches!(clear, Clear::Left | Clear::Both);
    let clears_right = matches!(clear, Clear::Right | Clear::Both);
    if !clears_left && !clears_right {
        return y;
    }
    self.active
        .iter()
        .filter(|float| {
            (clears_left && float.side == Float::Left) || (clears_right && float.side == Float::Right)
        })
        .map(|float| float.bottom())
        .fold(y, Scalar::max)
}

fn available_band(&self, y: Scalar) -> (Scalar, Scalar, Option<Scalar>) {
    let mut left_edge = self.inset.left;
    let mut right_edge = self.inset.left + self.content_width;
    let mut next_y = None;
    for float in self.active.iter().copied().filter(|float| float.overlaps_y(y)) {
        match float.side {
            Float::Left => left_edge = left_edge.max(float.x + float.width),
            Float::Right => right_edge = right_edge.min(float.x),
            Float::None => {}
        }
        next_y = Some(next_y.map_or(float.bottom(), |current| current.min(float.bottom())));
    }
    (left_edge, right_edge, next_y)
}
```

- [ ] **Step 5: Use opportunity search for BFC-making non-float children**

In the non-float block child path, compute the current fallback `location` as today, then replace it only when the child establishes a BFC or has `clear`:

```rust
let fallback_location = Point::new(
    in_flow_child_x(output.size, child_margin, &layout_constants) + inset_offset.x,
    cursor_y + inset_offset.y,
);
let establishes_bfc = child_style.overflow.x.blocks_margin_collapse()
    || child_style.overflow.y.blocks_margin_collapse()
    || child_style.display.is_flow_root_like();
let location = if establishes_bfc || child_style.clear != Clear::None {
    let placement = float_exclusions.place_bfc_block(
        cursor_y,
        output.size,
        child_margin,
        child_style.clear,
        fallback_location.x - inset_offset.x,
    );
    Point::new(placement.x + inset_offset.x, placement.y + inset_offset.y)
} else {
    fallback_location
};
```

If there is no `is_flow_root_like` helper, do not add a broad display abstraction. Use the overflow condition plus `clear` for this task, because the failing WPT cases use `overflow:hidden`.

- [ ] **Step 6: Ensure `cursor_y` advances from the actual float-aware location**

Replace:

```rust
let child_bottom = cursor_y + output.size.height;
```

with:

```rust
let child_bottom = (location.y - inset_offset.y) + output.size.height;
```

Keep margin collapse behavior unchanged except for the new clearance placement.

- [ ] **Step 7: Run focused tests**

Run:

```sh
cargo test -p surgeist --test layout block_bfc_zero_width_child -- --nocapture
```

Expected: the four new tests pass.

- [ ] **Step 8: Run focused WPT block parity**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/block cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: block WPT parity passes all 56 generated block fixtures.

- [ ] **Step 9: Commit implementation**

Run:

```sh
git add crates/surgeist/src/layout/block.rs crates/surgeist/tests/layout/block.rs
git commit -m "Place block BFCs around floats"
```

---

### Task 3: Regression Sweep and Review Fixes

**Files:**
- Modify only if review finds concrete issues:
  - `crates/surgeist/src/layout/block.rs`
  - `crates/surgeist/tests/layout/block.rs`

- [ ] **Step 1: Run focused layout tests**

Run:

```sh
cargo test -p surgeist --test layout block -- --nocapture
```

Expected: all block layout tests pass.

- [ ] **Step 2: Run WPT block parity**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/block cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: all 56 WPT block XML fixtures pass.

- [ ] **Step 3: Run local browser parity smoke**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity -- runs_browser_parity_smoke_fixture_against_surgeist_layout runs_subgrid_relative_rtl_abspos_fixture_against_surgeist_layout runs_grid_multiline_baseline_fixture_against_surgeist_layout -- --nocapture
```

Expected: the selected parity smoke tests pass.

- [ ] **Step 4: Request spec compliance review**

Dispatch a clean-context reviewer with:

```text
Review whether the implementation from BASE_SHA..HEAD_SHA satisfies docs/superpowers/plans/2026-06-19-surgeist-block-float-bfc-placement.md. Focus on whether it implements float-aware placement only for non-float BFC/clearance children, preserves existing float placement, and makes the four WPT block float fixtures pass without masking unsupported behavior.
```

- [ ] **Step 5: Implement all accepted spec review recommendations**

If the reviewer finds issues, add or update focused tests first, verify they fail when appropriate, then update implementation and rerun:

```sh
cargo test -p surgeist --test layout block_bfc_zero_width_child -- --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/block cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

- [ ] **Step 6: Request code quality review**

Dispatch a clean-context reviewer with:

```text
Review the implementation from BASE_SHA..HEAD_SHA for maintainability and risk. Focus on float placement state duplication, margin/cursor correctness, interaction with existing margin collapse behavior, and whether the tests cover the intended bug without overfitting.
```

- [ ] **Step 7: Implement all accepted code review recommendations**

Apply fixes with focused tests when behavior changes. Rerun the same focused block and parity commands.

- [ ] **Step 8: Final verification**

Run:

```sh
cargo fmt
cargo test -p surgeist --test layout block -- --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/block cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: all pass.

- [ ] **Step 9: Commit review fixes if any**

Run:

```sh
git status --short
git add crates/surgeist/src/layout/block.rs crates/surgeist/tests/layout/block.rs
git commit -m "Refine block float BFC placement"
```

Only commit if review fixes produced changes.

---

## Self-Review

- Spec coverage: The plan covers the exact four failing WPT block float cases, including zero-width opportunities and `clear:left` / `clear:right`.
- Placeholder scan: No `TBD`, vague “add tests”, or unspecified verification steps remain.
- Type consistency: The plan uses existing Surgeist layout types visible in `block.rs` and `tests/layout/block.rs`: `NodeInput`, `Display`, `Float`, `Clear`, `Overflow`, `Point`, `Size`, `Dimension`, `Available`, and `Scalar`.
- Risk note: This is intentionally not a complete CSS float implementation. It is scoped to preceding floats affecting non-float BFC/clearance placement in block layout, because that is the only current block WPT failure cluster.
