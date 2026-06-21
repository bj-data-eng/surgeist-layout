# Surgeist Flex Parity Backlog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce the current WPT flex parity backlog by fixing the first evidence-backed flex failure clusters without masking unsupported display-model work.

**Architecture:** Treat flex parity failures as separate root-cause clusters. First ensure generated WPT XML is fresh with respect to linked support resources, then regenerate the stale-looking flex abspos subset, then fix only layout defects proven by RED tests. Flex-grow factors below one must be re-clustered because the engine already has focused coverage for the obvious spec rule.

**Tech Stack:** Rust in `crates/surgeist`, generator code in `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`, focused layout tests in `crates/surgeist/tests/layout/flex.rs`, parity runner in `crates/surgeist/tests/layout_browser_parity.rs`, WPT corpus under `crates/surgeist/tests/layout_browser_parity/wpt` and `xml/wpt`, verification with `cargo test`, `cargo fmt --check`, and filtered `SURGEIST_PARITY_FILTER=xml/wpt/flex/...` runs.

---

## Evidence Snapshot

Fresh flex parity:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Current result: `9340` failures out of `10872` flex XML files.

Failure buckets:

```text
height mismatch: 744
unsupported content alignment: 32
unsupported display: 800
width mismatch: 990
x mismatch: 5588
y mismatch: 1186
```

Representative clusters:

- `flex__abspos`: `220` failures; `flex__abspos__position-absolute-002` alone has `156` failures, mostly x position.
- `flex__alignment__multiline-align-self`: `2848` failures, mostly x; likely cross-axis/writing-mode/column-wrap behavior.
- `flex__align-content-wrap-003`: `480` failures, x/y line alignment.
- `flex__flex-factor-less-than-one`: `156` failures, but the engine already has grow-sum-below-one code and tests, so this must be re-clustered before implementation.
- `flex__percentage-heights-001`: `44` failures, height.
- `unsupported display` includes `inline-flex` and `table`; do not solve this inside flex layout math.

Critical corpus finding:

```text
crates/surgeist/tests/layout_browser_parity/wpt/flex/abspos/position-absolute-002.html
```

links:

```html
<link href="../../flex/support/flexbox.css" rel="stylesheet">
```

and the checked-in support file contains:

```css
.flexbox {
    display: -webkit-flex;
    display: flex;
}
```

but generated XML such as:

```text
crates/surgeist/tests/layout_browser_parity/xml/wpt/flex/flex__abspos__position-absolute-002__items-10__content_box_ltr.xml
```

currently records the `.flexbox` element as:

```xml
<div source-tag="div" display="block" ...>
```

This must be resolved before changing flex abspos math for that cluster.

## File Map

- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
  - Add targeted flex support CSS import coverage.
  - Include linked WPT resources in generated XML freshness/provenance so stale XML is caught when support CSS changes.
- Modify if regenerated output changes: `crates/surgeist/tests/layout_browser_parity/xml/wpt/flex/flex__abspos__position-absolute-002__*.xml`
  - Regenerate with `SURGEIST_LAYOUT_GENERATE_FILTER=wpt/flex/flex__abspos__position-absolute-002`.
  - Alternate valid filter: `SURGEIST_LAYOUT_GENERATE_FILTER=wpt/flex/abspos/position-absolute-002`.
- Modify after RED only: `crates/surgeist/tests/layout/flex.rs`
  - Add focused tests for real abspos static-position or flex-factor defects.
- Modify after RED only: `crates/surgeist/src/layout/flex.rs`
  - Fix only the root-cause flex math proven by focused tests.
- Read-only verification target: `crates/surgeist/tests/layout_browser_parity.rs`
  - Run filtered parity after each cluster.

## Task 1: Track WPT Support Resources In Generated XML

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`

- [ ] **Step 1: Add targeted flex support CSS import coverage**

Add this test near the existing WPT import tests in `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`:

```rust
#[test]
fn import_wpt_from_source_keeps_relative_flex_support_stylesheet_available() {
    let root = std::env::temp_dir().join(format!(
        "surgeist-layout-wpt-flex-support-css-{}",
        std::process::id()
    ));
    let source_root = root.join("source");
    let corpus_root = root.join("corpus");
    let wpt_root = corpus_root.join("wpt");
    fs::create_dir_all(source_root.join("css/css-flexbox/abspos")).expect("source dir");
    fs::create_dir_all(source_root.join("css/flex/support")).expect("support dir");
    fs::create_dir_all(source_root.join("resources")).expect("resource dir");
    fs::create_dir_all(wpt_root.join("manifests")).expect("manifest dir");
    fs::write(
        source_root.join("css/css-flexbox/abspos/example.html"),
        r#"<!doctype html>
<link href="../../flex/support/flexbox.css" rel="stylesheet">
<script src="../../resources/check-layout-th.js"></script>
<div class="flexbox" data-expected-width="100"></div>
"#,
    )
    .expect("source fixture");
    fs::write(
        source_root.join("css/flex/support/flexbox.css"),
        ".flexbox { display: flex; width: 100px; }\n",
    )
    .expect("support stylesheet");
    for resource in WPT_RESOURCE_FILES {
        fs::write(source_root.join(resource), format!("// {resource}\n")).expect("resource");
    }
    fs::write(
        wpt_root.join("manifests/flex.toml"),
        r#"
domain = "flex"

[[cases]]
id = "flex__abspos__example"
path = "flex/abspos/example.html"
upstream_path = "css/css-flexbox/abspos/example.html"
upstream_commit = "f01d00b6963a"
generator = "wpt-multi-assertion"
status = "active"

[[cases.assertions]]
id = "items"
selector = "[data-expected-width]"
expect = "layout"
count = 1
"#,
    )
    .expect("manifest");
    let config = Config {
        root: corpus_root,
        html_root: root.join("corpus/html"),
        wpt_root: wpt_root.clone(),
        xml_root: root.join("corpus/xml"),
        filter: None,
        browser_cache: PathBuf::from("target/surgeist-browser"),
        browser_path: None,
        browser_version: None,
    };

    import_wpt_from_verified_source(&config, &source_root).expect("WPT import");

    let imported =
        fs::read_to_string(wpt_root.join("flex/abspos/example.html")).expect("imported fixture");
    assert!(
        imported.contains(r#"href="../../flex/support/flexbox.css""#)
            || imported.contains(r#"href="../../resources/css/flex/support/flexbox.css""#)
    );
    assert_eq!(
        fs::read_to_string(wpt_root.join("flex/support/flexbox.css"))
            .or_else(|_| {
                fs::read_to_string(wpt_root.join("resources/css/flex/support/flexbox.css"))
            })
            .expect("support stylesheet"),
        ".flexbox { display: flex; width: 100px; }\n"
    );
    check_wpt_corpus_against_verified_source(&config, &source_root)
        .expect("fresh import should check");
    fs::remove_dir_all(root).ok();
}
```

- [ ] **Step 2: Run targeted import coverage**

Run:

```sh
cargo test -p surgeist --bin surgeist-layout-generate import_wpt_from_source_keeps_relative_flex_support_stylesheet_available -- --nocapture
```

Expected: pass if existing import logic already preserves this path; fail only if the support stylesheet path is not imported or validated.

- [ ] **Step 3: Add RED freshness test for linked WPT resources**

Add a generator test proving this behavior:

```text
Given generated WPT XML whose source HTML is unchanged but whose linked support CSS changed,
check_corpus rejects the XML as stale.
```

Required implementation shape:

- Build a temp WPT corpus with one fixture linking `../../flex/support/flexbox.css`.
- Write generated XML with provenance matching the original fixture and support CSS.
- Mutate the checked-in support CSS under the temp corpus.
- Run the existing corpus freshness check.
- Assert the check errors because linked support provenance is stale.

Suggested assertion:

```rust
let error = check_corpus(&config).expect_err("changed support CSS should stale generated XML");
assert!(
    error.contains("stale") || error.contains("support"),
    "unexpected error: {error}"
);
```

- [ ] **Step 4: Implement linked-resource provenance**

Modify `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs` so generated XML provenance includes deterministic hashes of WPT linked dependencies used by the source fixture.

Requirements:

- Keep existing `source-sha256`, `helper-sha256`, and `browser` provenance intact.
- Add a deterministic linked-resource component, sorted by local relative path, for WPT fixtures.
- Freshness validation must compare the recorded dependency hashes against the current checked-in `wpt/` files.
- If a fixture has no linked resources, existing XML remains valid unless source/helper/browser provenance changed.
- Do not require network or upstream WPT checkout for freshness validation; it must use checked-in corpus files.

- [ ] **Step 5: Run generator tests**

Run:

```sh
cargo test -p surgeist --bin surgeist-layout-generate
```

Expected: all generator tests pass.

- [ ] **Step 6: Commit**

Run:

```sh
git status --short
git add crates/surgeist/src/bin/surgeist-layout-generate/generator.rs
git commit -m "Track WPT support resources in parity XML"
```

## Task 2: Regenerate And Measure Flex Abspos XML

**Files:**
- Modify if generated: `crates/surgeist/tests/layout_browser_parity/xml/wpt/flex/flex__abspos__position-absolute-002__*.xml`

- [ ] **Step 1: Regenerate the stale-looking subset**

Run:

```sh
SURGEIST_LAYOUT_GENERATE_FILTER=wpt/flex/flex__abspos__position-absolute-002 cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
```

Expected:

- Generation succeeds.
- Generated XML for `flex__abspos__position-absolute-002` records source `.flexbox` elements as `display="flex"`.
- No unrelated XML is rewritten.

- [ ] **Step 2: Inspect the generated diff**

Run:

```sh
rg -n 'display="flex"|display="block"' crates/surgeist/tests/layout_browser_parity/xml/wpt/flex/flex__abspos__position-absolute-002__*.xml
git diff --stat
git diff -- crates/surgeist/tests/layout_browser_parity/xml/wpt/flex | sed -n '1,220p'
```

Expected: the diff is limited to regenerated `position-absolute-002` XML and provenance. If display remains `block`, stop and debug browser generation before proceeding.

- [ ] **Step 3: Run focused parity after regeneration**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__abspos__position-absolute-002 cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: either the subset improves materially from `156` failures, or remaining failures now reflect real flex abspos math with faithful `display="flex"` input.

- [ ] **Step 4: Commit regenerated XML**

Run:

```sh
git status --short
git add crates/surgeist/tests/layout_browser_parity/xml/wpt/flex
git commit -m "Regenerate flex abspos parity XML"
```

## Task 3: Re-Cluster Flex-Grow Factors Below One

**Files:**
- Read: `crates/surgeist/src/layout/flex.rs`
- Read: `crates/surgeist/tests/layout/flex.rs`
- Modify after RED only: `crates/surgeist/tests/layout/flex.rs`
- Modify after RED only: `crates/surgeist/src/layout/flex.rs`

- [ ] **Step 1: Verify existing grow-factor coverage**

Run:

```sh
cargo test -p surgeist --test layout flex_grow_factors_below_one -- --nocapture
```

Expected: existing focused coverage passes. If no tests run, locate the exact existing grow-factor-below-one test name near the flex-grow tests and run it. Do not implement the old "sum grow < 1" fix unless a new RED test proves the current implementation is wrong.

- [ ] **Step 2: Re-run the WPT subset and capture first failures**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__flex-factor-less-than-one cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: record the first 10 failures, including XML path, expected value, actual value, and node path.

- [ ] **Step 3: Inspect representative XML and source**

Open the first failing XML and source HTML:

```sh
sed -n '1,120p' crates/surgeist/tests/layout_browser_parity/xml/wpt/flex/<first-failing-xml>.xml
sed -n '1,180p' crates/surgeist/tests/layout_browser_parity/wpt/flex/flex-factor-less-than-one.html
```

Classify the root cause as one of:

- supported flex-grow math bug not covered by existing focused tests
- fixture/generator issue
- min/max/auto-size interaction
- unsupported display or content alignment
- false cluster caused by stale XML

- [ ] **Step 4: Add a RED test only for the classified root cause**

If the root cause is a real supported flex math bug, add the smallest focused test in `crates/surgeist/tests/layout/flex.rs` that reproduces the representative XML failure. Run it and verify it fails:

```sh
cargo test -p surgeist --test layout <new_test_name> -- --nocapture
```

Expected: fail for the same reason as the WPT subset.

- [ ] **Step 5: Implement only the proven fix**

Modify `crates/surgeist/src/layout/flex.rs` only for the classified root cause. Keep existing grow-sum-below-one scaling intact unless the RED test proves that exact code is wrong.

- [ ] **Step 6: Verify and commit if code changed**

If this task produced a code fix, run:

```sh
cargo test -p surgeist --test layout <new_test_name> -- --nocapture
cargo test -p surgeist --test layout flex_grow -- --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__flex-factor-less-than-one cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: focused tests pass and the WPT subset improves from `156`.

Then commit:

```sh
git status --short
git add crates/surgeist/src/layout/flex.rs crates/surgeist/tests/layout/flex.rs
git commit -m "Fix flex factor parity case"
```

If this task proves the failures belong to generator, unsupported display, or a deferred sizing bucket, do not change flex layout code. Add the finding to the final residual bucket notes instead.

## Task 4: Fix Real Flex Abspos Static Position Failures

**Files:**
- Modify: `crates/surgeist/tests/layout/flex.rs`
- Modify: `crates/surgeist/src/layout/flex.rs`

Only start this task after Task 2 proves the `position-absolute-002` XML is faithful.

- [ ] **Step 1: Add a failing row static-position test**

Add this focused test near the existing absolute flex tests:

```rust
#[test]
fn flex_absolute_child_static_position_ignores_flow_siblings_in_main_axis() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            output_from_known_or(input, Size::ZERO)
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            align_items: Some(AlignItems::Center),
            ..NodeInput::default()
        },
    );
    for node in [2, 4] {
        tree.styles.insert(
            node,
            NodeInput {
                size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
                flex_shrink: 0.0,
                ..NodeInput::default()
            },
        );
    }
    tree.styles.insert(
        3,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
            flex_shrink: 0.0,
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(100.0)),
            available: Size::new(Available::definite(100.0), Available::definite(100.0)),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 40.0));
    assert_eq!(tree.layouts[&4].location, Point::new(20.0, 40.0));
}
```

- [ ] **Step 2: Add a flex-end static-position test only if parity proves it**

If regenerated XML still fails `align-items:flex-end` abspos cases, add this test:

```rust
#[test]
fn flex_absolute_child_static_cross_position_honors_flex_end_alignment() {
    #[derive(Default)]
    struct FlexTree {
        children: HashMap<u32, Vec<u32>>,
        styles: HashMap<u32, NodeInput>,
        layouts: HashMap<u32, NodeOutput>,
    }

    impl Traverse for FlexTree {
        type Node = u32;
        type Children<'a> = std::iter::Copied<std::slice::Iter<'a, u32>>;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl Compute for FlexTree {
        fn node_input(&self, node: Self::Node) -> &NodeInput {
            &self.styles[&node]
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(&mut self, _node: Self::Node, input: ComputeInput) -> ComputeOutput {
            output_from_known_or(input, Size::ZERO)
        }
    }

    let mut tree = FlexTree::default();
    tree.children.insert(1, vec![2, 3, 4]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.children.insert(4, vec![]);
    tree.styles.insert(
        1,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(100.0)),
            align_items: Some(AlignItems::FlexEnd),
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        2,
        NodeInput {
            size: Size::new(Dimension::px(100.0), Dimension::px(20.0)),
            flex_shrink: 0.0,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        3,
        NodeInput {
            position: Position::Absolute,
            size: Size::new(Dimension::px(20.0), Dimension::px(20.0)),
            flex_shrink: 0.0,
            ..NodeInput::default()
        },
    );
    tree.styles.insert(
        4,
        NodeInput {
            size: Size::new(Dimension::px(50.0), Dimension::px(20.0)),
            flex_shrink: 0.0,
            ..NodeInput::default()
        },
    );

    compute_flex(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(100.0), Some(100.0)),
            available: Size::new(Available::definite(100.0), Available::definite(100.0)),
        },
    );

    assert_eq!(tree.layouts[&2].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&3].location, Point::new(0.0, 80.0));
    assert_eq!(tree.layouts[&4].location, Point::new(100.0, 80.0));
}
```

- [ ] **Step 3: Run new tests and verify RED**

Run:

```sh
cargo test -p surgeist --test layout flex_absolute_child_static_position_ignores_flow_siblings_in_main_axis -- --nocapture
```

Expected: FAIL if the engine currently lets absolute children consume/order flow space or uses the wrong static-position rectangle.

- [ ] **Step 4: Implement the smallest abspos correction**

In `crates/surgeist/src/layout/flex.rs`, keep absolute children excluded from `resolve_lines`, and adjust only `layout_absolute_children`, `absolute_main_alignment`, or `absolute_cross_alignment` as needed.

Required behavior:

- Absolutely positioned flex children do not participate in flex line formation.
- With both main-axis insets auto, the static position uses flex container main-axis content alignment as if the abspos child were the only flex item.
- With both cross-axis insets auto, the static cross position uses `align-self` or container `align-items`; `wrap-reverse` flips cross-start/end interpretation.
- Explicit `top/right/bottom/left` insets continue to win over static alignment.

- [ ] **Step 5: Run focused abspos tests**

Run:

```sh
cargo test -p surgeist --test layout flex_absolute_child -- --nocapture
```

Expected: all flex absolute-child tests pass.

- [ ] **Step 6: Run the abspos WPT subset**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__abspos cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: failures improve from `220`. If not, stop and re-cluster abspos failures before adding another fix.

- [ ] **Step 7: Commit**

Run:

```sh
git status --short
git add crates/surgeist/src/layout/flex.rs crates/surgeist/tests/layout/flex.rs
git commit -m "Fix flex absolute static positioning"
```

## Task 5: Review, Measure, And Document Residual Flex Buckets

**Files:**
- Modify if useful: `docs/superpowers/plans/2026-06-19-surgeist-flex-parity-backlog.md`

- [ ] **Step 1: Run focused regression suites**

Run:

```sh
cargo fmt --check
cargo test -p surgeist --bin surgeist-layout-generate
cargo test -p surgeist --test layout flex -- --nocapture
```

Expected: all pass.

- [ ] **Step 2: Run focused parity buckets**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__abspos cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__flex-factor-less-than-one cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: both improve from the evidence snapshot or are explicitly bucketed.

- [ ] **Step 3: Run full flex parity**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: failure count improves from `9340`; capture the new count and top residual buckets.

- [ ] **Step 4: Dispatch clean-context reviewers**

Dispatch one spec reviewer:

```text
Review the Surgeist flex parity changes against docs/superpowers/plans/2026-06-19-surgeist-flex-parity-backlog.md.
Focus on corpus freshness, regenerated flex abspos XML fidelity, flex-factor re-clustering, and abspos static-position requirements.
Report Critical/Important/Minor findings with file/line references.
```

Dispatch one code reviewer:

```text
Review the Surgeist flex parity code changes for correctness, maintainability, and regression risk.
Focus on crates/surgeist/src/bin/surgeist-layout-generate/generator.rs, crates/surgeist/src/layout/flex.rs, and crates/surgeist/tests/layout/flex.rs.
Report Critical/Important/Minor findings with file/line references.
```

- [ ] **Step 5: Apply review recommendations**

Fix all Critical and Important review findings with focused tests first. Commit each logical correction:

```sh
git status --short
git add <changed files>
git commit -m "Address flex parity review"
```

- [ ] **Step 6: Final verification**

Run:

```sh
cargo fmt --check
cargo test -p surgeist --bin surgeist-layout-generate
cargo test -p surgeist --test layout flex -- --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__abspos cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__flex-factor-less-than-one cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
SURGEIST_PARITY_FILTER=xml/wpt/flex cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: focused checks pass or have explicitly documented residual unsupported cases; full flex parity improves from the starting `9340` failures.

## Residual Buckets Not In This Plan

Do not hide these. Bucket them after Task 5:

- `inline-flex`: display-model and inline formatting work, not a flex layout-only fix.
- `table`: unsupported table display, not flex.
- `multiline-align-self` and broad `align-content-*`: likely cross-axis writing-mode and column-wrap line packing; tackle after abspos/grow because the surface is larger and riskier.
- `percentage-heights-001`: percentage resolution in flex sizing; separate after the grow and abspos clusters are stable.

