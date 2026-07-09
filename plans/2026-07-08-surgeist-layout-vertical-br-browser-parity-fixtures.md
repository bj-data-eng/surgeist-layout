# Vertical BR Browser Parity Fixtures Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add layout-owned browser parity fixtures for vertical `<br>` line breaks and generate checked XML expectations for the supported vertical break contract.

**Architecture:** This implements Phase 6 from `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`. The browser helper will keep legacy complex vertical `<br>` cases unsupported, but will allow explicitly layout-ready constrained fixtures to generate when the `<br>` parent is a block inline-run context and the fixture supplies complete inline metrics through computed style. The plan adds a small vertical `<br>` fixture set, manifest entries, regenerated XML, and updated report assertions without moving HTML, CSS, style, retained tree, or text-shaping ownership into layout.

**Tech Stack:** Rust 2024, `surgeist-layout` browser parity harness, constrained HTML fixtures, Chromium-based `surgeist-layout-generate`, JSON generation reports, Cargo test/clippy/fmt.

---

## Source References

- Specification: `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`
- Sequencing: `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`
- Previous implementation: `plans/2026-07-08-surgeist-layout-vertical-forced-break-implementation.md`
- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Workflow: `AGENTS.md`
- Fixture harness docs: `tests/layout/browser_parity/README.md`

## Scope

This plan does:

- add four Surgeist-owned constrained HTML fixtures for vertical `<br>` line breaks;
- explicitly opt those fixtures into vertical `<br>` generation with fixture-only metadata;
- keep old complex vertical `<br>` fixtures unsupported until their surrounding baseline/text/subgrid contracts are reviewed separately;
- regenerate XML through the supported generator;
- update generation-report assertions and parser/helper tests to reflect the newly supported fixture slice;
- run the checked-in XML corpus, including the newly generated vertical `<br>` fixtures.

This plan does not:

- remove all `Unsupported vertical <br> line-break semantics` entries from the full corpus;
- hand-edit generated XML;
- parse authored CSS or HTML in layout;
- add a style/retained/text dependency;
- implement vertical `clear`;
- implement a richer vertical baseline output model;
- add compatibility aliases or fallback lowering paths.

## Files

- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
  - Add a fixture-only opt-in for vertical `<br>` generation.
  - Keep unsupported classification for vertical `<br>` outside the opt-in.
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`
  - Update bundled helper smoke tests for the new vertical `<br>` opt-in.
- Create:
  - `tests/layout/browser_parity/html/block/block_br_vertical_rl_inline_block_metrics.html`
  - `tests/layout/browser_parity/html/block/block_br_vertical_lr_inline_block_metrics.html`
  - `tests/layout/browser_parity/html/block/block_br_vertical_rl_empty_lines_metrics.html`
  - `tests/layout/browser_parity/html/block/block_br_vertical_rl_rtl_inline_block_metrics.html`
- Modify: `tests/layout/browser_parity/corpus.toml`
  - Add four `[[cases]]` entries for the new fixtures.
- Modify: `tests/layout/browser_parity/support.rs`
  - Update generation-report tests so the vertical unsupported bucket remains for legacy complex cases but excludes the new layout-ready fixtures.
  - Add lowering coverage for vertical `<br>` fixture attributes if needed.
- Modify: `tests/layout/browser_parity.rs`
  - Update HTML inventory and full report generated/unsupported counts after regeneration.
- Modify: `tests/layout/browser_parity/README.md`
  - Correct the scoped generator example to use `SURGEIST_LAYOUT_GENERATE_FILTER`.
- Generate: `tests/layout/browser_parity/xml/block/block_br_vertical_*__*.xml`
  - Generated only by `surgeist-layout-generate`.
- Regenerate: `tests/layout/browser_parity/xml/generation-reports/all.json`
  - Generated only by `surgeist-layout-generate`.

Because changing `test_helper.js` changes the helper hash embedded in generated XML provenance comments, run full generation before committing. Expect broad generated XML comment churn if the generator refreshes every checked-in XML file.

## Fixture Contract

Use a fixture-only data attribute on the vertical line-break parent:

```html
data-surgeist-layout-ready-vertical-br="true"
```

This is not an app-facing API and must not be lowered into layout XML. It only tells the constrained browser generator that this fixture has been deliberately authored to stay inside layout-owned vertical `<br>` semantics:

- parent display is block;
- child participants are atomic inline boxes and `<br>` controls only;
- no non-whitespace text nodes;
- no vertical `clear`;
- computed `writing-mode` and `direction` on `<br>` match the containing inline flow;
- complete inline metrics are available through computed `font-size` and `line-height`.

The opt-in check must be limited to the direct supported parent. Do not use an ancestor search such as `closest(...)`, because that could silently promote nested complex vertical `<br>` cases under a marked ancestor.

## Task 1: Add Layout-Ready Vertical BR Fixture Opt-In

**Files:**
- Modify: `tests/layout/browser_parity/scripts/gentest/test_helper.js`

- [ ] **Step 1: Add the opt-in helper**

Add this helper near `hasSupportedBrLineBreakParent`:

```javascript
function hasLayoutReadyVerticalBrFixture(e) {
  return e.parentElement?.getAttribute?.('data-surgeist-layout-ready-vertical-br') === 'true';
}
```

- [ ] **Step 2: Narrow vertical unsupported classification**

Replace the vertical `<br>` branch in `unsupportedElementReason`:

```javascript
if (e.tagName === 'BR' && isVerticalWritingMode(computedStyle.writingMode)) {
  return "Unsupported vertical <br> line-break semantics";
}
```

with:

```javascript
if (
  e.tagName === 'BR' &&
  isVerticalWritingMode(computedStyle.writingMode) &&
  !hasLayoutReadyVerticalBrFixture(e)
) {
  return "Unsupported vertical <br> line-break semantics";
}
```

Keep this existing branch unchanged:

```javascript
if (e.tagName === 'BR' && !hasSupportedBrLineBreakParent(e)) {
  return "Unsupported <br> outside block inline-run semantics";
}
```

- [ ] **Step 3: Run the browser parity support test before fixtures**

Run:

```sh
cargo test -p surgeist-layout generation_report_uses_explicit_br_unsupported_buckets -- --nocapture
```

Expected: pass. The checked-in report has not been regenerated yet, so the vertical unsupported bucket should still be present.

## Task 2: Update Bundled Generator Helper Tests

**Files:**
- Modify: `tests/bin/surgeist-layout-generate/generator.rs`

- [ ] **Step 1: Keep the fake parent compatible with the opt-in helper**

In `br_helper_smoke_script`, update the fake `parent` object so it has a safe `getAttribute()` method. The default should return `null` so existing non-opt-in smoke tests stay unsupported.

- [ ] **Step 2: Update vertical BR smoke coverage**

Replace or extend `bundled_helper_keeps_vertical_br_explicitly_unsupported` so it proves both sides of the new fixture contract:

- a vertical `<br>` without the parent opt-in still reports `"Unsupported vertical <br> line-break semantics"`;
- a vertical `<br>` whose direct parent returns `"true"` for `data-surgeist-layout-ready-vertical-br` does not report an unsupported reason.

Do not remove the assertion that the stale generic `"Unsupported <br> line-break semantics"` string is absent.

- [ ] **Step 3: Run focused bundled-helper tests**

Run:

```sh
cargo test -p surgeist-layout --features layout-golden-generate bundled_helper_keeps_vertical_br -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate bundled_helper_keeps_unmodeled_br_parent_contexts_unsupported -- --nocapture
```

Expected: pass.

## Task 3: Add Four Vertical BR HTML Fixtures

**Files:**
- Create: `tests/layout/browser_parity/html/block/block_br_vertical_rl_inline_block_metrics.html`
- Create: `tests/layout/browser_parity/html/block/block_br_vertical_lr_inline_block_metrics.html`
- Create: `tests/layout/browser_parity/html/block/block_br_vertical_rl_empty_lines_metrics.html`
- Create: `tests/layout/browser_parity/html/block/block_br_vertical_rl_rtl_inline_block_metrics.html`

- [ ] **Step 1: Add vertical-rl inline-block fixture**

Create `tests/layout/browser_parity/html/block/block_br_vertical_rl_inline_block_metrics.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Block vertical-rl br inline block metrics</title>
</head>
<body>

<div id="test-root" data-surgeist-layout-ready-vertical-br="true" style="display: block; writing-mode: vertical-rl; direction: ltr; width: 80px; font-size: 20px; line-height: 30px;">
  <span style="display: inline-block; width: 10px; height: 30px;"></span><br><span style="display: inline-block; width: 12px; height: 16px;"></span>
</div>

</body>
</html>
```

- [ ] **Step 2: Add vertical-lr inline-block fixture**

Create `tests/layout/browser_parity/html/block/block_br_vertical_lr_inline_block_metrics.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Block vertical-lr br inline block metrics</title>
</head>
<body>

<div id="test-root" data-surgeist-layout-ready-vertical-br="true" style="display: block; writing-mode: vertical-lr; direction: ltr; width: 80px; font-size: 20px; line-height: 30px;">
  <span style="display: inline-block; width: 10px; height: 30px;"></span><br><span style="display: inline-block; width: 12px; height: 16px;"></span>
</div>

</body>
</html>
```

- [ ] **Step 3: Add vertical-rl empty-lines fixture**

Create `tests/layout/browser_parity/html/block/block_br_vertical_rl_empty_lines_metrics.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Block vertical-rl br empty lines metrics</title>
</head>
<body>

<div id="test-root" data-surgeist-layout-ready-vertical-br="true" style="display: block; writing-mode: vertical-rl; direction: ltr; width: 80px; font-size: 20px; line-height: 30px;"><br><br></div>

</body>
</html>
```

- [ ] **Step 4: Add vertical-rl RTL fixture**

Create `tests/layout/browser_parity/html/block/block_br_vertical_rl_rtl_inline_block_metrics.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="../../scripts/gentest/test_helper.js"></script>
  <link rel="stylesheet" type="text/css" href="../../scripts/gentest/test_base_style.css">
  <title>Block vertical-rl rtl br inline block metrics</title>
</head>
<body>

<div id="test-root" data-surgeist-layout-ready-vertical-br="true" style="display: block; writing-mode: vertical-rl; direction: rtl; width: 80px; font-size: 20px; line-height: 30px;">
  <span style="display: inline-block; width: 10px; height: 30px;"></span><br><span style="display: inline-block; width: 12px; height: 16px;"></span>
</div>

</body>
</html>
```

- [ ] **Step 5: Add manifest entries**

Append these entries near the existing block `<br>` cases in `tests/layout/browser_parity/corpus.toml`:

```toml
[[cases]]
id = "block/block_br_vertical_rl_inline_block_metrics"
source_root = "surgeist"
source = "block/block_br_vertical_rl_inline_block_metrics.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "block/block_br_vertical_lr_inline_block_metrics"
source_root = "surgeist"
source = "block/block_br_vertical_lr_inline_block_metrics.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "block/block_br_vertical_rl_empty_lines_metrics"
source_root = "surgeist"
source = "block/block_br_vertical_rl_empty_lines_metrics.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "block/block_br_vertical_rl_rtl_inline_block_metrics"
source_root = "surgeist"
source = "block/block_br_vertical_rl_rtl_inline_block_metrics.html"
generator = "constrained-html"
status = "active"
```

## Task 4: Regenerate XML And Update Report Assertions

**Files:**
- Generate: `tests/layout/browser_parity/xml/block/block_br_vertical_rl_inline_block_metrics__border_box_ltr.xml`
- Generate: `tests/layout/browser_parity/xml/block/block_br_vertical_rl_inline_block_metrics__content_box_ltr.xml`
- Generate: `tests/layout/browser_parity/xml/block/block_br_vertical_rl_inline_block_metrics__border_box_rtl.xml`
- Generate: `tests/layout/browser_parity/xml/block/block_br_vertical_rl_inline_block_metrics__content_box_rtl.xml`
- Generate the same four variants for:
  - `block_br_vertical_lr_inline_block_metrics`
  - `block_br_vertical_rl_empty_lines_metrics`
  - `block_br_vertical_rl_rtl_inline_block_metrics`
- Regenerate: `tests/layout/browser_parity/xml/generation-reports/all.json`
- Modify: `tests/layout/browser_parity.rs`
- Modify: `tests/layout/browser_parity/support.rs`

- [ ] **Step 1: Run scoped generation first**

Run:

```sh
SURGEIST_LAYOUT_GENERATE_FILTER=block/block_br_vertical cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected:

- command succeeds;
- 16 new XML files are generated under `tests/layout/browser_parity/xml/block/`;
- scoped report `tests/layout/browser_parity/xml/generation-reports/block_block_br_vertical.json` exists;
- the scoped report has `"unsupported": []` and `"failed_to_generate": []`.

- [ ] **Step 2: Run focused parity for the new XML**

Run:

```sh
SURGEIST_PARITY_FILTER=block/block_br_vertical cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: pass. If any new vertical fixture fails geometry, do not loosen tolerance and do not add expected-fail. Inspect whether the fixture exceeds the Phase 6 layout-owned contract. Keep or revise only fixtures that pass with the current layout-owned behavior.

- [ ] **Step 3: Run full generation**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Expected:

- command succeeds;
- `tests/layout/browser_parity/xml/generation-reports/all.json` is regenerated;
- no `failed_to_generate` entries are present;
- legacy complex vertical `<br>` cases can remain under `"Unsupported vertical <br> line-break semantics"`.

- [ ] **Step 4: Update full-report count assertions**

Read the regenerated counts:

```sh
jq '.summary' tests/layout/browser_parity/xml/generation-reports/all.json
```

Update `browser_parity_generation_report_counts_full_scope` in `tests/layout/browser_parity.rs` to the regenerated exact values.

If only the four new cases became active and no legacy case changed bucket, the expected values should become:

```rust
assert_eq!(report_json["summary"]["generated"], 5048);
assert_eq!(report_json["summary"]["unsupported"], 356);
```

If generation reports a different value, stop and inspect the bucket diff before updating the assertion. Different values are acceptable only when explained by the generated report and consistent with the scope above.

- [ ] **Step 5: Update HTML inventory assertion**

Update `browser_parity_html_corpus_inventory_is_documented` in `tests/layout/browser_parity.rs` for the four added block fixtures. The current `taffy_plus_local_count` assertion should increase by four, and the message should describe the added vertical BR coverage rather than leaving the old fixture-count wording stale.

- [ ] **Step 6: Update BR unsupported report test**

Modify `generation_report_uses_explicit_br_unsupported_buckets` in `tests/layout/browser_parity/support.rs` so it:

- still rejects stale `"Unsupported <br> line-break semantics"`;
- still requires `"Unsupported <br> outside block inline-run semantics"`;
- still allows or requires `"Unsupported vertical <br> line-break semantics"` for legacy complex cases;
- proves the four new sources are not in the unsupported bucket.

Use this helper inside the test:

```rust
let unsupported_sources = unsupported
    .iter()
    .filter_map(|entry| entry.get("source").and_then(serde_json::Value::as_str))
    .collect::<Vec<_>>();
for source in [
    "html/block/block_br_vertical_rl_inline_block_metrics.html",
    "html/block/block_br_vertical_lr_inline_block_metrics.html",
    "html/block/block_br_vertical_rl_empty_lines_metrics.html",
    "html/block/block_br_vertical_rl_rtl_inline_block_metrics.html",
] {
    assert!(
        !unsupported_sources.contains(&source),
        "{source} should generate rather than remain unsupported"
    );
}
```

- [ ] **Step 7: Verify manifest, report, and source discovery after full generation**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```

Expected: pass. This command must run after generation because it checks generated XML/report freshness, including embedded helper provenance.

- [ ] **Step 8: Verify generated XML carries layout-ready vertical break data**

Run:

```sh
rg -n 'source-tag="br".*writing-mode="vertical|inline-baseline=|inline-line-height=' tests/layout/browser_parity/xml/block/block_br_vertical_*.xml
```

Expected: every new XML variant has `source-tag="br"` entries with `writing-mode="vertical-rl"` or `writing-mode="vertical-lr"` and complete `inline-baseline`/`inline-line-height` attributes.

## Task 5: Final Verification And Review

**Files:**
- Modify: `tests/layout/browser_parity/README.md`
- Inspect other files only unless failures require task-local edits.

- [ ] **Step 1: Correct the scoped generation command in docs**

In `tests/layout/browser_parity/README.md`, change the scoped regeneration example from `SURGEIST_PARITY_FILTER=subgrid cargo run ... surgeist-layout-generate` to:

```sh
SURGEIST_LAYOUT_GENERATE_FILTER=subgrid cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Do not change the parity-test filter examples; those still use `SURGEIST_PARITY_FILTER`.

- [ ] **Step 2: Run focused checks**

Run:

```sh
cargo test -p surgeist-layout generation_report_uses_explicit_br_unsupported_buckets -- --nocapture
cargo test -p surgeist-layout browser_parity_generation_report_counts_full_scope -- --nocapture
cargo test -p surgeist-layout browser_parity_html_corpus_inventory_is_documented -- --nocapture
cargo test -p surgeist-layout --features layout-golden-generate bundled_helper_keeps_vertical_br -- --nocapture
SURGEIST_PARITY_FILTER=block/block_br_vertical cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
```

Expected: all pass.

- [ ] **Step 3: Run full checks**

Run:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

Expected: all pass. `git status` should show only task-owned source, fixture, generated XML, and report changes before the coordinator commits.

- [ ] **Step 4: Scoped review**

Ask a clean-context reviewer to inspect the task diff against:

- this plan;
- `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`;
- `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`;
- `plans/2026-07-08-surgeist-layout-vertical-forced-break-implementation.md`;
- `guidance/surgeist-rust-modeling-guide.md`;
- `AGENTS.md`.

The reviewer must confirm:

- vertical `<br>` parity is enabled only for layout-ready constrained fixtures;
- legacy complex vertical `<br>` cases are not silently promoted;
- generated XML was produced by the generator and not hand-edited;
- helper changes are fixture-harness classification only, not app-facing layout semantics;
- generator bundled-helper tests prove default vertical `<br>` stays unsupported and direct-parent opt-in is supported;
- no HTML/CSS/style/text ownership moved into layout;
- new XML passes the filtered parity run.

- [ ] **Step 5: Commit after clean scoped review**

After worker checks and scoped review are clean, commit:

```sh
git add tests/layout/browser_parity/scripts/gentest/test_helper.js \
  tests/bin/surgeist-layout-generate/generator.rs \
  tests/layout/browser_parity/html/block/block_br_vertical_rl_inline_block_metrics.html \
  tests/layout/browser_parity/html/block/block_br_vertical_lr_inline_block_metrics.html \
  tests/layout/browser_parity/html/block/block_br_vertical_rl_empty_lines_metrics.html \
  tests/layout/browser_parity/html/block/block_br_vertical_rl_rtl_inline_block_metrics.html \
  tests/layout/browser_parity/corpus.toml \
  tests/layout/browser_parity/support.rs \
  tests/layout/browser_parity.rs \
  tests/layout/browser_parity/README.md \
  tests/layout/browser_parity/xml
git commit -m "Add vertical br browser parity fixtures"
```

## Final Holistic Review Gate

After the scoped commit, assign a final clean-context holistic reviewer. The final reviewer must inspect the complete result against:

- this implementation plan;
- the inline control item spec;
- the sequencing plan;
- the vertical forced-break implementation;
- modeling guidance;
- the actual code and generated artifacts.

Completion requires the final reviewer to come back clean and these final checks to pass:

```sh
cargo test -p surgeist-layout
cargo clippy -p surgeist-layout --all-targets -- -D warnings
cargo fmt --check
git diff --check
git status --short --branch
```

## Cross-Crate Notes

No blocking cross-crate work is required for these layout-owned constrained fixtures. Root/style/retained/text still own real application `<br>` classification, computed writing-mode/direction/clear/vertical-align lowering, and production inline metrics. This plan only proves that layout can consume generated layout-ready vertical `<br>` fixture data when those upstream contracts are already represented in XML.
