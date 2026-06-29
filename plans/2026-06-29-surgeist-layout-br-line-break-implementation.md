# Surgeist Layout BR Line Break Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support HTML `<br>` in layout browser parity as a typed inline line-break primitive with basic style context, without treating it as an ordinary block, flex, grid, or measurable leaf box.

**Architecture:** Keep `NodeInputOf<S>` as box layout input. Add a separate `LayoutInputOf<S>` enum whose variants distinguish normal boxes from line-break primitives, and a small `LineBreakInput` model for line-break-only style context. Block atomic inline runs consume `LayoutInputOf<S>` for children, convert line-break children into forced inline breaks, and never dispatch a line break through normal box layout. Browser parity lowers `source-tag="br"` to `LayoutInput::LineBreak` after normal `surgeist-style` declaration resolution; unsupported vertical and outside-block contexts remain explicit unsupported buckets.

**Tech Stack:** Rust 2024, `surgeist-layout`, source-side tests in `src/*_tests.rs`, browser parity support in `tests/layout/browser_parity/support.rs`, generator helper in `tests/layout/browser_parity/scripts/gentest/test_helper.js`, generator tests in `tests/bin/surgeist-layout-generate/generator.rs`, generated XML under `tests/layout/browser_parity/xml`.

---

## Modeling Boundary

This plan deliberately does **not** add a public `inline_role` flag to `NodeInputOf<S>`. A public flag would make combinations such as `display: grid` plus "line break" constructible as ordinary box input, which is counter to `guidance/surgeist-rust-modeling-guide.md`.

The intended model is:

- `NodeInputOf<S>` remains layout-ready box input.
- `LineBreakInput` is layout-ready line-break input.
- `LayoutInputOf<S>` is the node-kind boundary:

```rust
pub enum LayoutInputOf<S: LayoutScalar = DefaultScalar> {
    Box(NodeInputOf<S>),
    LineBreak(LineBreakInput),
}
```

A line break can carry line-break style context, but it cannot carry flex/grid/block sizing fields because it is not a box. Any code that needs a box must ask for the `Box` variant. Any code that needs to handle inline flow children must match `LayoutInputOf<S>` explicitly.

The first implementation supports:

- Horizontal `<br>` as a forced break inside block atomic inline runs.
- Zero-size output for the line-break node so fixture geometry can compare it.
- `display: none` suppressing the line break through a line-break-specific display state.
- Normal style resolution for fields relevant to line-break context: `direction`, `writing-mode`, `vertical-align`, and `clear`; some fields are carried for future behavior.
- Browser parity generation only for horizontal `<br>` whose immediate parent has parseable `display: block`.

The first implementation does not support:

- Full mixed text and element inline layout.
- Generated content around `<br>`.
- Treating `<br>` as a block, flex, grid, or leaf box.
- Vertical-writing `<br>` line progression.
- Horizontal `<br>` outside a parseable `display: block` parent.
- Empty line-height-only behavior for leading, trailing, or consecutive `<br>` beyond explicitly tested atomic-inline behavior.
- Browser-specific `<br clear>` float clearance. Preserve `clear`, but do not claim float-clear behavior.

Current generated corpus evidence:

```text
240 unsupported variants / 60 unique HTML fixtures: Unsupported <br> line-break semantics
100 unsupported variants / 25 unique HTML fixtures: Unsupported mixed text/element content
16 unsupported variants / 4 unique HTML fixtures: Unsupported missing #test-root fixture root
```

The plan should unblock only the supported horizontal block-parent subset. Vertical and outside-context `<br>` fixtures remain unsupported with distinct reasons.

## Coordinator Workflow

Follow `AGENTS.md` for every implementation task. The coordinator assigns one worker for each task or tightly coupled task group, then assigns a separate scoped reviewer before committing. A task is committable only after:

- the worker reports changed files, commands run, results, and git status;
- the scoped reviewer is clean or all findings have been reconciled;
- the coordinator reruns the task's focused check after review fixes;
- the coordinator reviews `git diff --stat` and the relevant detailed diff.

Do not run or edit API artifact generation from this crate. Root owns public API artifact refresh during integration.

## Task 1: Add Typed Layout Node Input Variants

**Files:**

- Modify: `src/node_input.rs`
- Modify: `src/lib.rs`
- Modify: `src/contract_tests.rs`

- [ ] **Step 1: Add line-break-specific input types**

In `src/node_input.rs`, near `Display`, add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineBreakDisplay {
    #[default]
    Break,
    None,
}

impl LineBreakDisplay {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineBreakInput {
    display: LineBreakDisplay,
    direction: Direction,
    writing_mode: WritingMode,
    vertical_align: VerticalAlign,
    clear: Clear,
}
```

Add constructors and accessors:

```rust
impl LineBreakInput {
    pub const DEFAULT: Self = Self {
        display: LineBreakDisplay::Break,
        direction: Direction::Ltr,
        writing_mode: WritingMode::HorizontalTb,
        vertical_align: VerticalAlign::Baseline,
        clear: Clear::None,
    };

    #[must_use]
    pub const fn new() -> Self {
        Self::DEFAULT
    }

    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.display = LineBreakDisplay::None;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn with_clear(mut self, clear: Clear) -> Self {
        self.clear = clear;
        self
    }

    #[must_use]
    pub const fn display(self) -> LineBreakDisplay {
        self.display
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn clear(self) -> Clear {
        self.clear
    }
}

impl Default for LineBreakInput {
    fn default() -> Self {
        Self::DEFAULT
    }
}
```

- [ ] **Step 2: Add layout node input enum**

In `src/node_input.rs`, after `NodeInputOf<S>` and its default impls, add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutInputOf<S: LayoutScalar = DefaultScalar> {
    Box(NodeInputOf<S>),
    LineBreak(LineBreakInput),
}

pub type LayoutInput = LayoutInputOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutInputOf<S> {
    #[must_use]
    pub const fn box_input(input: NodeInputOf<S>) -> Self {
        Self::Box(input)
    }

    #[must_use]
    pub const fn line_break(input: LineBreakInput) -> Self {
        Self::LineBreak(input)
    }

    #[must_use]
    pub const fn as_box(&self) -> Option<&NodeInputOf<S>> {
        match self {
            Self::Box(input) => Some(input),
            Self::LineBreak(_) => None,
        }
    }

    #[must_use]
    pub const fn as_line_break(&self) -> Option<LineBreakInput> {
        match self {
            Self::Box(_) => None,
            Self::LineBreak(input) => Some(*input),
        }
    }
}
```

- [ ] **Step 3: Reexport the new types**

In `src/lib.rs`, add these to the `pub use node_input::{ ... }` list:

```rust
LayoutInput, LayoutInputOf, LineBreakDisplay, LineBreakInput,
```

- [ ] **Step 4: Add contract tests**

In `src/contract_tests.rs`, add:

```rust
#[test]
fn line_break_input_defaults_to_visible_horizontal_break_context() {
    let input = LineBreakInput::default();
    assert_eq!(input.display(), LineBreakDisplay::Break);
    assert_eq!(input.direction(), Direction::Ltr);
    assert_eq!(input.writing_mode(), WritingMode::HorizontalTb);
    assert_eq!(input.vertical_align(), VerticalAlign::Baseline);
    assert_eq!(input.clear(), Clear::None);
}

#[test]
fn layout_input_distinguishes_box_from_line_break() {
    let box_input = LayoutInput::box_input(NodeInput::default());
    assert!(box_input.as_box().is_some());
    assert!(box_input.as_line_break().is_none());

    let line_break = LayoutInput::line_break(LineBreakInput::new().hidden());
    assert!(line_break.as_box().is_none());
    assert_eq!(
        line_break.as_line_break().unwrap().display(),
        LineBreakDisplay::None
    );
}

#[test]
fn node_input_does_not_carry_line_break_state() {
    let input = NodeInput {
        display: Display::Grid,
        ..NodeInput::default()
    };

    let layout_input = LayoutInput::box_input(input);
    assert!(layout_input.as_line_break().is_none());
}
```

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout line_break_input -- --nocapture
cargo test -p surgeist-layout layout_input -- --nocapture
```

Expected: the new contract tests pass.

- [ ] **Step 6: Request scoped review**

Ask a reviewer to check Task 1 against `guidance/surgeist-rust-modeling-guide.md`. They must verify that `NodeInputOf<S>` remains box input and line-break state is represented only by `LayoutInputOf<S>::LineBreak`.

- [ ] **Step 7: Commit**

```sh
git add src/node_input.rs src/lib.rs src/contract_tests.rs
git commit -m "Model line breaks as distinct layout input"
```

## Task 2: Make LayoutInput The Required Tree Boundary

**Files:**

- Modify: `src/traits.rs`
- Modify: `src/test_support/layout_tree.rs`
- Modify: `tests/layout/browser_parity/support.rs`
- Modify: every source or test file that implements `Compute`
- Modify: focused tests in existing test modules as needed

- [ ] **Step 1: Add required layout-input access to `Compute`**

In `src/traits.rs`, import `LayoutInputOf` and add a required method to
`Compute`:

```rust
fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar>;
```

Keep `node_input(&self, ...) -> &NodeInputOf<_>` unchanged for box algorithms
that already require a box. Do not provide a default `layout_input`
implementation that silently wraps `node_input` as a box. Every tree must answer
the node-kind question explicitly, even when its answer is always
`LayoutInputOf::Box(self.node_input(node).clone())`.

- [ ] **Step 2: Update all existing `Compute` implementors**

Update every `impl Compute for ...` in `src/**` and `tests/**` so the crate
compiles with the required method. For box-only test fixtures, implement:

```rust
fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
    LayoutInputOf::Box(self.node_input(node).clone())
}
```

This is not a compatibility escape hatch; it is an explicit statement that the
fixture only contains boxes. Any fixture that will contain line breaks must
return `LayoutInputOf::LineBreak(...)` for those nodes.

- [ ] **Step 3: Store strict layout inputs in oracle trees**

In `src/test_support/layout_tree.rs`, change `OracleTreeOf<S>` storage from
`styles: HashMap<u32, NodeInputOf<S>>` to
`layout_inputs: HashMap<u32, LayoutInputOf<S>>`. Rename the current `inputs`
field that records `ComputeInputOf<S>` calls to `compute_inputs` to avoid
ambiguity.

Remove the current f32 `DEFAULT_NODE_INPUT` fallback. The oracle should have one
scalar-independent invariant:

```text
Every oracle node that layout can inspect must have an explicit typed layout input.
```

This is an intentional test-support modeling fix, not just `<br>` plumbing. An
undeclared node should fail loudly, because silently defaulting to a box hides
fixture authoring mistakes and makes f32/f64 oracle behavior diverge.

Add builders:

```rust
pub fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
    self.layout_inputs.insert(node, LayoutInputOf::Box(style));
    self
}

pub fn line_break(mut self, node: u32, input: LineBreakInput) -> Self {
    self.layout_inputs.insert(node, LayoutInputOf::LineBreak(input));
    self
}
```

Update `node_input` to return the box input or panic with a clear message if a
line-break node is incorrectly used as a box or if a node has no declared layout
input:

```rust
match self.layout_inputs.get(&node) {
    Some(LayoutInputOf::Box(input)) => input,
    Some(LayoutInputOf::LineBreak(_)) => panic!("line break node has no box NodeInput"),
    None => panic!("oracle node {node} must define a layout input"),
}
```

Override `layout_input` with the same missing-node behavior:

```rust
fn layout_input(&self, node: Self::Node) -> LayoutInputOf<Self::Scalar> {
    self.layout_inputs
        .get(&node)
        .cloned()
        .unwrap_or_else(|| panic!("oracle node {node} must define a layout input"))
}
```

The final behavior should be the same for f32 and f64:

```text
node_input(missing)      -> panic
layout_input(missing)   -> panic
layout_input(box node)  -> LayoutInputOf::Box(...)
layout_input(line break)-> LayoutInputOf::LineBreak(...)
```

Audit source tests for implicit default child nodes after this change:

```sh
rg -n "OracleTree(::|Of::<).*new|\\.children\\(|\\.style\\(|\\.line_break\\(" src tests
rg -n "\\.styles\\.insert\\(" src tests
```

Every child that can be reached through `layout_input` must be declared with
`.style(...)` or `.line_break(...)`. For a default box, the declaration should
be explicit:

```rust
.style(node, NodeInput::default())
```

Replace any direct `tree.styles.insert(...)` writes with `.style(node, ...)`
before or during this task. If a test genuinely needs direct map access after
the rename, it must insert `LayoutInputOf::Box(...)` through an explicit helper
owned by the oracle test support, not by reaching into storage.

Do not implement this pattern:

```rust
self.layout_inputs
    .get(&node)
    .cloned()
    .unwrap_or_else(|| LayoutInputOf::Box(NodeInputOf::default()))
```

That form would make a missing layout input silently become a box and violates
the modeling boundary.

- [ ] **Step 4: Store strict layout inputs in browser parity `TestTree`**

In `tests/layout/browser_parity/support.rs`, update `TestNode` to store `layout_input: layout::LayoutInput` instead of only `node_input: layout::NodeInput`. Provide helpers:

```rust
fn box_input(&self) -> &layout::NodeInput {
    match &self.layout_input {
        layout::LayoutInput::Box(input) => input,
        layout::LayoutInput::LineBreak(_) => {
            panic!("line break node has no box NodeInput")
        }
    }
}
```

Implement `Compute::node_input` by returning `box_input()`, and implement
`layout_input` by cloning the stored enum. Synthetic text nodes remain
`LayoutInput::Box(layout::NodeInput::default())`.

- [ ] **Step 5: Add dispatcher tests**

Add tests proving:

```rust
#[test]
fn oracle_tree_line_break_input_is_not_box_input() {
    let tree = crate::test_support::layout_tree::OracleTree::new()
        .line_break(1, LineBreakInput::new());

    assert!(matches!(
        tree.layout_input(1),
        LayoutInput::LineBreak(_)
    ));
}
```

In browser parity support tests, add an equivalent assertion for a parsed `source-tag="br"` node after Task 5 if direct construction is not yet available. If direct construction is available in this task, add it here.

- [ ] **Step 6: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout layout_input -- --nocapture
cargo test -p surgeist-layout
```

Expected: focused tests pass, and the crate compiles with every `Compute`
implementor explicitly providing `layout_input`.

- [ ] **Step 7: Request scoped review**

Reviewer must verify this task does not add line-break layout behavior yet; it
only creates a strict typed input boundary and forces all tree implementors to
state whether nodes are boxes or line breaks. The reviewer must reject any
default trait method or helper that makes line-break support opt-in by silently
treating unknown nodes as boxes.

- [ ] **Step 8: Commit**

```sh
git add src tests
git commit -m "Thread typed layout input through test trees"
```

## Task 3: Teach Atomic Inline Layout About Forced Break Items

**Files:**

- Modify: `src/block.rs`
- Modify: `src/inline.rs`
- Modify: `src/inline_tests.rs`

- [ ] **Step 1: Convert atomic inline items to a closed algorithm enum**

In `src/inline.rs`, rename the current `AtomicInlineItem` struct to `AtomicInlineBoxItem` and introduce:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum AtomicInlineItem<S: LayoutScalar = DefaultScalar> {
    Box(AtomicInlineBoxItem<S>),
    ForcedLineBreak { order: u32 },
}
```

Move box-only methods (`advance`, `baseline`, `line_baseline`, `line_descent`) onto `AtomicInlineBoxItem<S>`. Keep `AtomicInlineItem::new(...)` for tests and add `AtomicInlineItem::forced_line_break(order)`.

- [ ] **Step 2: Add report item kind**

Add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AtomicInlineLayoutItemKind {
    Box,
    ForcedLineBreak,
}
```

Add `pub kind: AtomicInlineLayoutItemKind` to `AtomicInlineLayoutItem<S>` and set ordinary boxes to `Box`.

- [ ] **Step 3: Split horizontal lines at forced breaks**

In `layout_atomic_inline_items`, match `AtomicInlineItem::Box` and `AtomicInlineItem::ForcedLineBreak`. A forced break stores a pending zero-size break at the current line width, pushes the current line, and starts a new line. Do not add an empty trailing line after a final break unless a future line-height model requires it.

Emit forced-break report items as:

```rust
AtomicInlineLayoutItem {
    kind: AtomicInlineLayoutItemKind::ForcedLineBreak,
    order,
    location: Point::new(x, y + line.baseline),
    size: Size::ZERO,
    content_size: Size::ZERO,
    margin: Edges::ZERO,
    padding: Edges::ZERO,
    border: Edges::ZERO,
    scrollbar_size: Size::ZERO,
}
```

- [ ] **Step 4: Reject vertical forced breaks explicitly**

At the top of `layout_vertical_rl_atomic_inline_items`, add a debug assertion that no `ForcedLineBreak` reaches this path. Because `AtomicInlineItem` is now an enum, explicitly map only `Box` payloads for the existing vertical layout and use `unreachable!()` for `ForcedLineBreak` after the assertion. Do not silently drop forced breaks.

- [ ] **Step 5: Update intrinsic widths**

`atomic_inline_min_content_width` ignores forced breaks and returns the max single box advance. `atomic_inline_max_content_width` returns the maximum segment sum between forced breaks.

- [ ] **Step 6: Update block's existing box construction in the same task**

Update `layout_atomic_inline_run` in `src/block.rs` to construct `AtomicInlineBoxItem` and wrap it as `AtomicInlineItem::Box(...)`. This is required so the task compiles at its commit point. Do not add line-break child collection in this task.

- [ ] **Step 7: Add focused inline tests**

In `src/inline_tests.rs`, add:

```rust
#[test]
fn atomic_inline_forced_line_break_starts_next_line() {
    let report = layout_atomic_inline_items(AtomicInlineInput {
        available_width: Available::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        items: vec![
            AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
            AtomicInlineItem::forced_line_break(1),
            AtomicInlineItem::new(2, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        ],
    });

    assert_eq!(report.size, Size::new(30.0, 20.0));
    assert_eq!(report.items[1].kind, AtomicInlineLayoutItemKind::ForcedLineBreak);
    assert_eq!(report.items[1].location, Point::new(20.0, 10.0));
    assert_eq!(report.items[2].location, Point::new(0.0, 10.0));
}

#[test]
fn atomic_inline_intrinsic_widths_split_at_forced_line_breaks() {
    let items = vec![
        AtomicInlineItem::new(0, Size::new(20.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::new(1, Size::new(30.0, 10.0), Edges::ZERO, Some(10.0)),
        AtomicInlineItem::forced_line_break(2),
        AtomicInlineItem::new(3, Size::new(40.0, 10.0), Edges::ZERO, Some(10.0)),
    ];

    assert_eq!(atomic_inline_min_content_width(&items), 40.0);
    assert_eq!(atomic_inline_max_content_width(&items), 50.0);
}
```

- [ ] **Step 8: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout forced_line_break -- --nocapture
```

Expected: tests pass and the crate compiles.

- [ ] **Step 9: Request scoped review**

Reviewer must verify task scope includes the necessary `block.rs` compile adaptation and does not start line-break node collection.

- [ ] **Step 10: Commit**

```sh
git add src/block.rs src/inline.rs src/inline_tests.rs
git commit -m "Support forced breaks in atomic inline layout"
```

## Task 4: Collect LineBreak Layout Inputs In Block Atomic Inline Runs

**Files:**

- Modify: `src/block.rs`
- Modify: `src/block_tests.rs`

- [ ] **Step 1: Match child `LayoutInputOf<S>` in block flow**

In `layout_in_flow_children` and `layout_atomic_inline_run`, use `tree.layout_input(child)` for child flow decisions. Treat:

- `LayoutInputOf::Box(style)` with `style.display == Display::None` as hidden.
- `LayoutInputOf::LineBreak(input)` with `input.display().is_none()` as hidden.
- `LayoutInputOf::LineBreak(input)` as atomic inline-run participation only when not hidden and not vertical-writing.

Consume `LineBreakInput` during this classification step for hidden and
vertical-writing decisions. If a non-hidden line break has a non-horizontal
writing mode, panic with a clear message such as
`vertical line-break layout is not implemented`. Browser parity generation
should continue to quarantine vertical `<br>` fixtures before they reach this
path; the panic protects crate-local tests and custom trees from silently
skipping unsupported line-break semantics.

The later run-child value should carry only the node identity and report order
needed for write-back.

Do not pass line breaks to `compute_child`.

- [ ] **Step 2: Add run child enum**

Replace the tuple used for `run_children` with:

```rust
enum AtomicInlineRunChild<Node, S: LayoutScalar> {
    Box {
        child: Node,
        style: NodeInputOf<S>,
        output: ComputeOutputOf<S>,
    },
    LineBreak {
        child: Node,
        order: u32,
    },
}
```

For line breaks, push `AtomicInlineItem::forced_line_break(order)`.

- [ ] **Step 3: Match report items by order**

Because forced line breaks produce report items, build:

```rust
let report_items = report
    .items
    .iter()
    .map(|item| (item.order, *item))
    .collect::<BTreeMap<_, _>>();
```

Use it for both box and line-break write-back. A line break writes `NodeOutputOf::with_order(order)` with zero size at the forced-break report location.

- [ ] **Step 4: Add block tests**

Add tests:

- `block_atomic_inline_run_honors_line_break_child`
- `hidden_line_break_does_not_split_atomic_inline_run`
- `block_atomic_inline_run_never_computes_line_break_as_box`
- `vertical_line_break_panics_until_modeled`

Use `OracleTree::line_break(node, LineBreakInput::new())` for line-break nodes and `LineBreakInput::new().hidden()` for hidden line breaks.

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p surgeist-layout line_break -- --nocapture
```

Expected: block line-break tests pass.

- [ ] **Step 6: Request scoped review**

Reviewer must verify line breaks cannot fall through to block/flex/grid/leaf compute paths from block atomic inline runs.

- [ ] **Step 7: Commit**

```sh
git add src/block.rs src/block_tests.rs
git commit -m "Collect line breaks in block inline runs"
```

## Task 5: Lower Browser Parity BR Metadata To LayoutInput::LineBreak

**Files:**

- Modify: `tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Make `to_node_input` return `layout::LayoutInput`**

Rename or replace the helper currently returning `layout::NodeInput` so browser parity style attrs lower to `layout::LayoutInput`.

For non-`source-tag="br"` nodes:

```rust
Ok(layout::LayoutInput::Box(input))
```

For `source-tag="br"` nodes, first run the existing `surgeist-style::adapters::layout` lowering exactly as before. Then map selected fields into `LineBreakInput`:

```rust
let mut br = layout::LineBreakInput::new()
    .with_direction(input.direction)
    .with_writing_mode(input.writing_mode)
    .with_vertical_align(input.vertical_align)
    .with_clear(input.clear);

if input.display == layout::Display::None {
    br = br.hidden();
}

Ok(layout::LayoutInput::LineBreak(br))
```

Do not normalize `display: block` to `InlineBlock`; line-break participation is now carried by the enum variant, not by display.

- [ ] **Step 2: Update `TestNode` construction**

Store the returned `LayoutInput` from Step 1. Box nodes expose `NodeInput` through `Compute::node_input`. Line-break nodes are handled only by `layout_input` and must panic if accidentally requested as `node_input`.

- [ ] **Step 3: Replace BR rejection tests**

Replace `source_tag_br_is_rejected_until_line_break_semantics_are_modeled` with positive tests:

```rust
#[test]
fn source_tag_br_lowers_to_line_break_input() {
    let input = to_layout_input(
        &StyleAttrs {
            attrs: BTreeMap::from([("source-tag".to_string(), "br".to_string())]),
        },
        &mut s::adapters::layout::LayoutLoweringSession::new(),
    )
    .expect("source-tag br should lower");

    assert!(matches!(input, layout::LayoutInput::LineBreak(_)));
}

#[test]
fn source_tag_br_display_none_lowers_to_hidden_line_break() {
    let input = to_layout_input(
        &StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "none".to_string()),
            ]),
        },
        &mut s::adapters::layout::LayoutLoweringSession::new(),
    )
    .expect("display none br should lower");

    let layout::LayoutInput::LineBreak(input) = input else {
        panic!("br should lower to line break");
    };
    assert_eq!(input.display(), layout::LineBreakDisplay::None);
}
```

- [ ] **Step 4: Run focused support tests**

Run:

```sh
cargo test -p surgeist-layout --test layout source_tag_br -- --nocapture
cargo test -p surgeist-layout --test layout checked_fixture_enumerator_quarantines_unsupported_br_xml -- --nocapture
```

Expected: positive lowering tests pass and existing checked-fixture quarantine still passes.

- [ ] **Step 5: Request scoped review**

Reviewer must verify browser parity still uses the single style-lowering path and only maps `source-tag="br"` to `LayoutInput::LineBreak` after style lowering.

- [ ] **Step 6: Commit**

```sh
git add tests/layout/browser_parity/support.rs
git commit -m "Lower browser parity br as line break input"
```

## Task 6: Narrow Generator Unsupported BR Buckets

**Files:**

- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`

- [ ] **Step 1: Keep unsupported contexts explicit**

In `test_helper.js`, change `unsupportedElementReason(e)` to `unsupportedElementReason(e, computedStyle)`.

Use:

```javascript
function unsupportedElementReason(e, computedStyle) {
  if (e.tagName === 'BR' && isVerticalWritingMode(computedStyle.writingMode)) {
    return "Unsupported vertical <br> line-break semantics";
  }
  if (e.tagName === 'BR' && !hasSupportedBrLineBreakParent(e)) {
    return "Unsupported <br> outside block inline-run semantics";
  }
  return undefined;
}

function hasSupportedBrLineBreakParent(e) {
  const parent = e.parentElement;
  if (!parent) return false;
  return getComputedStyle(parent).display === "block";
}
```

Do not remove `tagName: e.tagName.toLowerCase()`.

- [ ] **Step 2: Update generator helper tests**

Replace the generic BR rejection assertion with tests that check:

- no stale generic `Unsupported <br> line-break semantics` string remains;
- vertical BR remains explicitly unsupported;
- outside-block BR remains explicitly unsupported;
- `tagName` is still captured.

- [ ] **Step 3: Run generator tests**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_describes_br_as_source_tag_without_measured_box_special_case -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_keeps_vertical_br_explicitly_unsupported -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_keeps_unmodeled_br_parent_contexts_unsupported -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate unsupported_browser_semantics_are_reported_without_xml_generation -- --nocapture
```

Expected: all pass.

- [ ] **Step 4: Request scoped review**

Reviewer must verify the generator does not globally unquarantine horizontal BR; only parseable `display: block` parent contexts are generated.

- [ ] **Step 5: Commit**

```sh
git add tests/layout/browser_parity/scripts/gentest/test_helper.js tests/bin/surgeist-layout-generate/generator.rs
git commit -m "Narrow br fixture generation support"
```

## Task 7: Regenerate Browser Parity XML And Reports

**Files:**

- Modify: `tests/layout/browser_parity/xml/**`
- Modify: `tests/layout/browser_parity/xml/generation-reports/*.json`
- Modify if counts changed: `tests/layout/browser_parity/README.md`
- Modify: `tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Regenerate affected subgrid fixtures**

Run:

```sh
SURGEIST_PARITY_FILTER=subgrid_baseline cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: supported horizontal block-parent BR XML is generated. Vertical and outside-block contexts remain unsupported with distinct reasons.

- [ ] **Step 2: Regenerate full report**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: the stale generic `Unsupported <br> line-break semantics` bucket is gone.

Record:

```sh
jq -r '(.unsupported // [])[] | (.reason // .kind // .error // "unknown")' \
  tests/layout/browser_parity/xml/generation-reports/all.json \
  | sort | uniq -c | sort -nr
```

Expected remaining BR reasons:

```text
Unsupported vertical <br> line-break semantics
Unsupported <br> outside block inline-run semantics
```

- [ ] **Step 3: Replace checked-fixture quarantine test**

Replace `checked_fixture_enumerator_quarantines_unsupported_br_xml` with a report assertion proving the stale generic BR bucket is absent.

- [ ] **Step 4: Run parsing and targeted parity tests**

Run:

```sh
cargo test -p surgeist-layout --test layout parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout browser_parity_generation_report_counts_full_scope -- --nocapture
cargo test -p surgeist-layout --test layout generation_report_no_longer_classifies_horizontal_br_as_unsupported -- --nocapture
SURGEIST_PARITY_FILTER=subgrid_baseline cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: parse/report tests pass. The ignored parity sweep must not fail due to parse errors, stale generic unsupported reasons, or line-break nodes falling into box dispatch. Concrete geometry mismatches may be recorded.

- [ ] **Step 5: Request scoped review**

Reviewer must verify generated files are generator-owned outputs and that unsupported BR buckets match the modeled scope.

- [ ] **Step 6: Commit**

```sh
git add tests/layout/browser_parity/xml tests/layout/browser_parity/README.md tests/layout/browser_parity/support.rs
git commit -m "Regenerate parity fixtures with br line breaks"
```

## Task 8: Record Cross-Crate Follow-Up

**Files:**

- Create: `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`

- [ ] **Step 1: Create the ledger**

Create:

```markdown
# Surgeist Layout BR Line Break Cross-Crate Ledger

This ledger records follow-up work outside `surgeist-layout` discovered while
implementing `plans/2026-06-29-surgeist-layout-br-line-break-implementation.md`.

## Entries

### HTML/style adapter needs to lower real `<br>` to `LayoutInput::LineBreak`

- Status: `open`
- Owning crate: root `surgeist` or future HTML/DOM adapter crate
- Affected API: `surgeist_layout::LayoutInputOf::LineBreak`
- Observed behavior: layout browser parity can lower `source-tag="br"` fixture
  metadata after style resolution, but production HTML tree construction outside
  this crate still needs to map real HTML `<br>` elements to layout input.
- Expected behavior: the real adapter should preserve normal style resolution
  for the element, then construct `LayoutInput::LineBreak(LineBreakInput)`.
  `display: none` should map to hidden line-break input.
- Required owning change: add a root or adapter implementation plan after this
  layout API lands. Do not implement that adapter from the layout crate project.
```

- [ ] **Step 2: Commit**

```sh
git add plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md
git commit -m "Record br adapter follow-up"
```

## Task 9: Final Verification And Holistic Review

**Files:**

- Review all files changed by Tasks 1-8.

- [ ] **Step 1: Run focused checks**

Run:

```sh
cargo test -p surgeist-layout line_break -- --nocapture
cargo test -p surgeist-layout layout_input -- --nocapture
cargo test -p surgeist-layout --test layout source_tag_br -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- --nocapture
```

Expected: all pass.

- [ ] **Step 2: Run baseline checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
```

Expected: all pass.

- [ ] **Step 3: Run generated-fixture check**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

Expected: the pinned Taffy corpus check passes.

- [ ] **Step 4: Request final holistic clean-context review**

Reviewer must inspect the final result against this plan, `AGENTS.md`, and `guidance/surgeist-rust-modeling-guide.md`. They must check:

- `NodeInputOf<S>` remains box input and does not carry line-break flags.
- `LayoutInputOf<S>` is the explicit box-vs-line-break node-kind boundary.
- No line-break node can be accidentally computed as block/flex/grid/leaf layout.
- Browser parity still uses style resolver and `surgeist-style::adapters::layout` before mapping `source-tag="br"` to line-break input.
- Unsupported vertical/outside-block contexts remain explicit.
- Task commits are logical and reviewed.

- [ ] **Step 5: Address findings**

Assign workers and reviewers for any findings. Do not complete the implementation until the final holistic reviewer is clean.

Completion is when all tasks are implemented, required verification passes, generated artifacts are current, cross-crate follow-up is recorded, and the final holistic clean-context review is clean.
