# Surgeist WPT Single Assertion Generation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Surgeist WPT parity generation source-faithful by removing synthetic WPT border/content and LTR/RTL fan-out while preserving one generated fixture per numeric WPT assertion.

**Architecture:** WPT HTML is the source of truth for direction, box sizing, and authored styles. The generator should fan out only when a WPT HTML file contains multiple numeric assertions, producing one XML fixture per assertion with no extra body class mutation. Existing local Surgeist HTML fixtures may continue to use their legacy `border_box_ltr`, `content_box_ltr`, `border_box_rtl`, and `content_box_rtl` variants.

**Tech Stack:** Rust generator in `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`, browser helper JavaScript in `crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js`, generated parity XML and reports under `crates/surgeist/tests/layout_browser_parity/xml`, verification with `cargo test`, `cargo fmt --check`, `cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate`, and filtered `layout_browser_parity` runs.

---

## Current State

The working tree currently contains generator/helper changes for source-faithful WPT output. Earlier partial generated XML/report fallout from the rejected WPT variant fan-out behavior has been cleaned; keep generated output clean unless a full successful regeneration is intentionally retained.

Keep these already-correct generator directions:

- Browser generation should load fixture HTML through the document-write path with a synthesized base URL so linked CSS and WPT resources resolve without `page.goto(file://...)`.
- WPT root coordinate expectations should be normalized to local root coordinates instead of inheriting full-page browser offsets.
- Linked WPT resources and helper hashes should remain part of freshness/provenance checking.
- External or opaque stylesheet fallback should use non-initial computed values conservatively.

Change this behavior:

- `getSurgeistWptCases()` must no longer synthesize four WPT variants by mutating `document.body.className`.
- WPT generation must no longer duplicate the same source data into `borderBoxLtrData`, `contentBoxLtrData`, `borderBoxRtlData`, and `contentBoxRtlData`.
- Multi-assertion WPT tests still fan out by assertion id and matched element index, e.g. `items-1`, `items-2`, so each generated XML has a single numeric assertion target.

## File Map

- Modify: `crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js`
  - Return a WPT case map where each case id maps to one source-faithful `data` object.
  - Preserve unsupported-case reporting per assertion.
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
  - Update WPT case parsing/writing to consume a single WPT data key.
  - Update tests that currently assert duplicated variant keys.
  - Keep legacy four-variant generation for non-WPT local Surgeist fixtures.
- Regenerate: `crates/surgeist/tests/layout_browser_parity/xml/wpt/**`
  - Regenerate after the generator shape is correct.
  - Remove stale WPT variant XML files that no longer correspond to generated cases.
- Regenerate: `crates/surgeist/tests/layout_browser_parity/xml/generation-reports/*.json`
  - Reports must match the regenerated WPT corpus and current helper hash.
- Read-only: `crates/surgeist/tests/layout_browser_parity/wpt/**`
  - WPT HTML inputs stay source-faithful; do not edit them to simulate box or direction variants.

## Task 1: Clean Rejected Partial Regeneration

**Files:**
- Revert generated outputs only: `crates/surgeist/tests/layout_browser_parity/xml/wpt/**`
- Remove generated report if only produced during the abandoned run: `crates/surgeist/tests/layout_browser_parity/xml/generation-reports/wpt_flex_flex__abspos__position-absolute-002.json`

- [x] **Step 1: Inspect dirty generated output count**

Run:

```sh
git status --porcelain=v1 crates/surgeist/tests/layout_browser_parity/xml | wc -l
```

Current result on 2026-06-19: no dirty generated XML/report files remained.

- [x] **Step 2: Revert generated XML/report fallout from the abandoned fan-out run**

Run:

```sh
git restore -- crates/surgeist/tests/layout_browser_parity/xml/wpt crates/surgeist/tests/layout_browser_parity/xml/generation-reports
rm -f crates/surgeist/tests/layout_browser_parity/xml/generation-reports/wpt_flex_flex__abspos__position-absolute-002.json
```

This step reverts only generated corpus output from the current abandoned run. Do not revert `generator.rs`, `test_helper.js`, docs, or any engine files. No generated fallout remained when Worker A inspected the tree.

- [x] **Step 3: Verify only source changes remain**

Run:

```sh
git status --short -- crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js crates/surgeist/tests/layout_browser_parity/xml
```

Observed: modified `generator.rs` and `test_helper.js`, plus this plan; no modified XML files.

- [x] **Step 4: Remove any remaining untracked generated fallout after listing it**

Run:

```sh
git status --porcelain=v1 crates/surgeist/tests/layout_browser_parity/xml
```

Observed: no output. If untracked generated WPT XML or abandoned scoped generation reports appear in a later run, list them first, then remove only those generated files.

## Task 2: Change WPT Helper Output To One Data Object Per Assertion

**Files:**
- Modify: `crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js`
- Modify tests in: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`

- [x] **Step 1: Update `getSurgeistWptCases` contract**

In `crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js`, change WPT case output from four variant keys to one source-faithful key:

```js
function getSurgeistWptCases(assertions) {
  const cases = {};
  for (const assertion of assertions) {
    const elements = Array.from(document.querySelectorAll(assertion.selector));
    if (elements.length === 0) {
      const count = Math.max(1, assertion.count || 1);
      for (let index = 0; index < count; index++) {
        const id = count === 1 ? assertion.id : `${assertion.id}-${index + 1}`;
        cases[id] = {
          data: unsupportedTestData(`missing WPT assertion selector ${assertion.selector}`),
        };
      }
      continue;
    }
    elements.forEach((element, index) => {
      const id = elements.length === 1 ? assertion.id : `${assertion.id}-${index + 1}`;
      const root = wptAssertionRoot(element);
      cases[id] = { data: describeElement(root, element) };
    });
  }
  return JSON.stringify(cases);
}
```

- [x] **Step 2: Update helper contract tests**

In `generator.rs`, update the test that checks WPT numeric helper behavior so it asserts:

```rust
assert!(TEST_HELPER_SOURCE.contains("cases[id] = { data: describeElement(root, element) }"));
assert!(!TEST_HELPER_SOURCE.contains("borderBoxRtlData: sourceData"));
assert!(TEST_HELPER_SOURCE.contains("document.body.className = \"border-box ltr\""));
let wpt_helper = TEST_HELPER_SOURCE
    .split("function getSurgeistWptCases")
    .nth(1)
    .expect("WPT helper function should exist");
assert!(!wpt_helper.contains("document.body.className"));
assert!(wpt_helper.contains("const count = Math.max(1, assertion.count || 1)"));
```

- [x] **Step 3: Pass WPT assertion counts into the helper**

In `describe_wpt_case`, serialize each assertion as:

```rust
serde_json::json!({
    "id": assertion.id,
    "selector": assertion.selector,
    "count": assertion.count,
})
```

This lets missing-selector cases still report one unsupported generated case per expected numeric assertion id.

- [x] **Step 4: Run focused helper contract test**

Run:

```sh
cargo test -p surgeist --features layout-golden-generate --bin surgeist-layout-generate bundled_helper_records_wpt_numeric_expectations_from_data_attrs -- --nocapture
```

Expected: pass.

Observed: passed on 2026-06-19.

## Task 3: Update Rust WPT XML Writing For Single WPT Data

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`

- [x] **Step 1: Locate WPT variant iteration**

Inspect the existing WPT variant loop:

```sh
rg -n 'border_box_ltr|content_box_ltr|borderBoxLtrData|contentBoxLtrData|write_wpt' crates/surgeist/src/bin/surgeist-layout-generate/generator.rs
```

Keep the four local fixture variants for non-WPT generation. Change only the WPT writer path.

- [x] **Step 2: Replace WPT variant iteration with a single source variant**

Update the WPT XML writing path so each WPT case id reads only `data` from the browser JSON. The output filename should no longer include `__border_box_ltr`, `__content_box_ltr`, `__border_box_rtl`, or `__content_box_rtl`.

Required shape:

```rust
let measurement = case_measurements
    .get("data")
    .ok_or_else(|| format!("measurement JSON missing data for {case_id}"))?;
write_wpt_xml_case(config, case, case_id, measurement, report)?;
```

Use the actual local function names in `generator.rs`; do not add a parallel writer if the existing writer can be simplified.

- [x] **Step 3: Update WPT missing-key and unsupported tests**

Update WPT writer/helper tests that currently expect missing `contentBoxLtrData` or `borderBoxLtrData` to expect missing `data`. Leave non-WPT local fixture missing-variant tests unchanged; those should still expect missing `contentBoxLtrData` because local fixture variants are intentionally preserved.

Example expected assertion:

```rust
assert!(error.contains("measurement JSON missing data"));
```

- [x] **Step 4: Run generator unit tests**

Run:

```sh
cargo test -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: all generator tests pass.

Observed: 130 generator tests passed on 2026-06-19.

## Task 4: Regenerate And Verify WPT Corpus Shape

**Files:**
- Regenerate: `crates/surgeist/tests/layout_browser_parity/xml/wpt/**`
- Regenerate: `crates/surgeist/tests/layout_browser_parity/xml/generation-reports/*.json`

- [x] **Step 1: Run full generator**

Run:

```sh
cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: completes without fatal errors. Unsupported fixtures may be reported, but they should be represented in reports rather than silently skipped unless they are excluded visual-only inputs.

Observed: full generation completed with unsupported fixture reports. Because `check-corpus` also validates the scoped WPT report, Worker A then ran `SURGEIST_LAYOUT_GENERATE_FILTER=wpt cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate`, which also completed.

- [x] **Step 2: Assert no WPT synthetic variant filenames remain**

Run:

```sh
find crates/surgeist/tests/layout_browser_parity/xml/wpt -type f \( -name '*__border_box_ltr.xml' -o -name '*__content_box_ltr.xml' -o -name '*__border_box_rtl.xml' -o -name '*__content_box_rtl.xml' \) | head
```

Expected: no output.

Observed: no output.

- [x] **Step 3: Assert local non-WPT variants still exist**

Run:

```sh
find crates/surgeist/tests/layout_browser_parity/xml -path '*/wpt/*' -prune -o -type f -name '*__border_box_ltr.xml' -print | head
```

Expected: local non-WPT XML variants still exist.

Observed: local non-WPT `__border_box_ltr` XML variants still exist.

- [x] **Step 4: Run corpus freshness check**

Run:

```sh
cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```

Expected: pass.

Observed: passed on 2026-06-19.

## Task 5: Review, Parity Smoke, And Commit

**Files:**
- Review all modified generator/helper/report/XML files.

- [x] **Step 1: Run formatting and generator tests**

Run:

```sh
cargo fmt --check
cargo test -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
```

Expected: both pass.

Observed: `cargo fmt --check` passed and generator tests passed on 2026-06-19.

- [ ] **Step 2: Run focused flex abspos parity smoke**

Run:

```sh
SURGEIST_PARITY_FILTER=xml/wpt/flex/flex__abspos__position-absolute-002 cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected: this may still expose real flex engine failures, but it must no longer show stale `display="block"` expectations from unloaded support CSS or synthetic source variants.

- [ ] **Step 3: Dispatch clean-context generator review**

Ask a reviewer subagent to review:

- the final diff from the previous commit to `HEAD`;
- the WPT no-synthetic-fan-out requirement;
- whether generated WPT filenames, reports, and helper hashes are internally consistent;
- whether local non-WPT fixture variants were preserved.

Fix all Critical and Important findings, then repeat review if fixes were required.

- [ ] **Step 4: Commit logical generator change**

Run:

```sh
git add docs/superpowers/plans/2026-06-19-surgeist-wpt-single-assertion-generation.md \
  crates/surgeist/src/bin/surgeist-layout-generate/generator.rs \
  crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js \
  crates/surgeist/tests/layout_browser_parity/xml
git commit -m "Fix WPT single assertion generation"
```

Expected: one logical commit containing the policy change, matching generated output, and plan.

## Completion Gate

This goal is complete only after:

- WPT synthetic border/content and LTR/RTL XML fan-out is gone.
- Multi-assertion WPT HTML still generates one XML fixture per individual numeric assertion.
- Local non-WPT fixture variants are unchanged.
- `check-corpus` passes.
- Generator unit tests pass.
- A clean-context reviewer approves the generator changes after any recommendations are implemented.
- The accepted generator change is committed as a logical history point.
