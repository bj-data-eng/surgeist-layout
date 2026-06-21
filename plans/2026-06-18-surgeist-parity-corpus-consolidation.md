# Surgeist Parity Corpus Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Surgeist browser parity corpus repeatable, idempotent, auditable, and generated through one supported pipeline while preserving the current Taffy green baseline and adding raw WPT HTML as first-class source input.

**Architecture:** Keep `crates/surgeist/tests/layout_browser_parity` as the single parity root. Treat `html/` as the currently active constrained HTML corpus, add `wpt/` for raw WPT HTML imported into Surgeist-owned domain folders, add a manifest that records source provenance and case fan-out, and make `surgeist-layout-generate` the only supported XML generation path. The WPT importer rewrites links as files move so the checked-in corpus is self-contained and does not rely on upstream path layout at runtime. Continue emitting one XML file per runnable generated case at first, even when one raw WPT file contains multiple assertions, so the existing Rust parity runner remains simple while the generator learns multi-assertion fan-out.

**Tech Stack:** Rust generator in `crates/surgeist/src/bin/surgeist-layout-generate`, browser helpers in `crates/surgeist/tests/layout_browser_parity/scripts/gentest`, parity runner/parser in `crates/surgeist/tests/layout_browser_parity.rs` and `support.rs`, corpus inputs under `crates/surgeist/tests/layout_browser_parity`, pinned Taffy fixture source from `target/surgeist-sources/taffy/d1ff7e339b9ee35b33858779f8d7653197e93d92`, pinned WPT fixture source from `target/surgeist-sources/wpt/f01d00b6963a`, verification with focused `cargo test -p surgeist --test layout_browser_parity ...`, `cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture`, `cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus`, `cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus`, and `cargo fmt --check`.

---

## Current Baseline

- Current full parity command:

```sh
cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

- Current result before this plan: `76` failing checked-in XML fixtures, all in `grid-lanes` and `subgrid`.
- Current checked-in active corpus shape:
  - `html/`: `1,333` HTML files.
  - `xml/`: `5,260` generated XML files.
  - Taffy-derived green baseline source in `target/surgeist-sources/taffy/d1ff7e339b9ee35b33858779f8d7653197e93d92/test_fixtures`: `1,103` HTML files.
  - Current non-WPT-ish active corpus excluding `subgrid/` and `grid-lanes/`: `1,106` HTML files, which is the Taffy baseline plus three named-grid Surgeist additions.
  - Taffy suite counts:
    - `block`: `205`
    - `blockflex`: `7`
    - `blockgrid`: `14`
    - `flex`: `569`
    - `float`: `5`
    - `grid`: `283`
    - `gridflex`: `6`
    - `leaf`: `14`
  - Every Taffy `.html` file exists byte-identically at the same relative path under `crates/surgeist/tests/layout_browser_parity/html`.
  - The three non-Taffy, non-`subgrid`, non-`grid-lanes` additions are:
    - `grid/grid_named_negative_occurrence.html`
    - `grid/grid_named_repeated_line_names.html`
    - `grid/grid_named_template_area_generated_names.html`
  - The current WPT-derived excluded corpus is `226` HTML files: `subgrid` has `210`; `grid-lanes` has `16`.
- Raw WPT source inventory:
  - Prefer the generator-managed source cache at `target/surgeist-sources/wpt/f01d00b6963a` as the import/check source so the corpus does not depend on a manually prepared checkout.
  - Use `tmp/servo/tests/wpt/tests` only for comparison unless a manifest records Servo's source SHA for a copied file.
  - Initial layout-relevant roots in the pinned WPT source cache include:
    - `css/css-flexbox`: `1,940` HTML files.
    - `css/css-grid`: `2,877` HTML files.
    - `css/css-sizing`: `855` HTML files.
    - `css/css-align`: `340` HTML files.
    - `css/css-overflow`: `1,073` HTML files.
    - `css/css-writing-modes`: `790` HTML files.
    - `css/css-display`: `155` HTML files.
    - `css/css-box`: `176` HTML files.
    - `css/CSS2/box-display`: `63` HTML files.
    - `css/CSS2/normal-flow`: `94` HTML files.
    - `css/CSS2/visuren`: `40` HTML files.
    - `css/CSS2/visudet`: `37` HTML files.
    - `css/CSS2/linebox`: `30` HTML files.
    - `css/CSS2/floats`: `121` HTML files.
    - `css/CSS2/floats-clear`: `39` HTML files.
    - `css/CSS2/positioning`: `29` HTML files.
    - `css/CSS2/tables`: `25` HTML files.
- Current generator risks:
  - `scripts/gentest` and `surgeist-layout-generate` both exist, but only the Rust binary should remain supported.
  - Full generation deletes `xml/`; scoped generation can leave stale XML.
  - Unsupported/quarantine handling is split across JS helper logic, Rust generator behavior, `x*.html` filename skips, and Rust-side fixture filtering.
  - XML has no durable provenance header or sidecar report.
  - WPT-derived fixtures have been manually converted into constrained single-assertion HTML files under `html/`, which hides the original upstream test shape.
  - Unsupported constructs such as `<br>` and mixed inline text/element semantics are currently hidden by generator or runner filters instead of appearing in pass/fail buckets.
  - `html/.DS_Store` is present and should be removed or made impossible through fixture discovery checks.

## Target Corpus Layout

```text
crates/surgeist/tests/layout_browser_parity/
  README.md
  corpus.toml
  html/
    block/
    blockflex/
    blockgrid/
    flex/
    float/
    grid/
    grid-lanes/
    gridflex/
    leaf/
    subgrid/
  wpt/
    block/
    flex/
    grid/
    grid-lanes/
    subgrid/
    inline/
    sizing/
    alignment/
    overflow/
    writing-modes/
    box/
    resources/
    manifests/
    expectations/
  xml/
    ...
  scripts/gentest/
    test_base_style.css
    test_helper.js
```

Rules:

- `html/` remains the active constrained corpus for the current parity runner until raw WPT fan-out is implemented.
- `wpt/` stores imported WPT HTML by Surgeist layout domain: `wpt/subgrid`, `wpt/grid-lanes`, `wpt/grid`, `wpt/flex`, `wpt/block`, `wpt/inline`, and related box/sizing/alignment domains.
- Importing WPT files may move them away from upstream paths, but the importer must rewrite `<link>`, `<script>`, `@import`, CSS `url(...)`, reftest references, and other relative or root-relative URLs so the checked-in corpus is self-contained.
- Shared WPT support files that imported tests reference live under `wpt/resources/` or domain-local `support/`, `reference/`, `references/`, or `resources/` directories after rewrite.
- `xml/` remains generated output and must not be manually edited for expected browser geometry.
- Every XML output must be traceable to a manifest entry, source file hash, generator version/schema, browser version/path, helper hash, and variant.
- Quarantine and expected-failure status must live in explicit metadata, not in filename prefixes or hidden parser filters.
- No test is silently skipped because Surgeist lacks support for a feature such as `<br>`, mixed inline text, or a WPT helper pattern. Unsupported behavior must appear in pass/fail bucket output as an explicit classification.

## Manifest Model

Create `crates/surgeist/tests/layout_browser_parity/corpus.toml`.

The manifest should begin with explicit source roots:

```toml
[source_roots.taffy]
kind = "taffy"
path = "html"
upstream_commit = "d1ff7e339b9ee35b33858779f8d7653197e93d92"
description = "Pinned Taffy browser fixture baseline copied from target/surgeist-sources/taffy/<commit>/test_fixtures."

[source_roots.surgeist]
kind = "surgeist"
path = "html"
description = "Surgeist-authored constrained HTML fixtures layered on top of the Taffy baseline."

[source_roots.wpt]
kind = "wpt"
path = "wpt"
description = "Raw upstream WPT HTML organized by layout domain."
```

Each case entry should include:

```toml
[[cases]]
id = "flex/align_items_center"
source_root = "taffy"
source = "flex/align_items_center.html"
generator = "constrained-html"
variants = ["border_box_ltr", "content_box_ltr", "border_box_rtl", "content_box_rtl"]
status = "active"

[[cases]]
id = "grid/named_template_area_generated_names"
source_root = "surgeist"
source = "grid/grid_named_template_area_generated_names.html"
generator = "constrained-html"
variants = ["border_box_ltr", "content_box_ltr", "border_box_rtl", "content_box_rtl"]
status = "active"

[[cases]]
id = "wpt/subgrid/alignment-in-subgridded-axes-001"
source_root = "wpt"
source = "subgrid/alignment-in-subgridded-axes-001.html"
upstream_path = "css/css-grid/subgrid/alignment-in-subgridded-axes-001.html"
upstream_commit = "<pinned-wpt-commit>"
generator = "wpt-multi-assertion"
variants = ["border_box_ltr", "content_box_ltr", "border_box_rtl", "content_box_rtl"]
status = "active"

[[cases.assertions]]
id = "start-start-item"
selector = "#start-start-item"
expect = "layout"

[[cases.assertions]]
id = "baseline-baseline-item"
selector = "#baseline-baseline-item"
expect = "layout"
```

Status values:

- `active`: generate and run.
- `expected-fail`: generate and run; report as an expected-failure bucket until the layout issue is fixed.
- `unsupported`: generate a runnable classified result where possible; when XML cannot be generated yet, emit an explicit manifest/report entry that is counted as `unsupported`, not silently skipped.
- `quarantined`: generate and run in reporting mode, or emit an explicit counted quarantine result if execution is impossible; must include a reason and follow-up reference.

## Task 1: Commit The Corpus Contract

**Files:**
- Create: `docs/superpowers/plans/2026-06-18-surgeist-parity-corpus-consolidation.md`

- [ ] **Step 1: Save this plan.**

Use this file as the implementation contract before touching generator code.

- [ ] **Step 2: Verify no production code changed.**

Run:

```sh
git diff --stat
```

Expected: only this plan file is new.

- [ ] **Step 3: Commit.**

Run:

```sh
git add docs/superpowers/plans/2026-06-18-surgeist-parity-corpus-consolidation.md
git commit -m "Plan parity corpus consolidation"
```

Expected: a documentation-only commit.

## Task 2: Add The Manifest Skeleton And Corpus Inventory Tests

**Files:**
- Create: `crates/surgeist/tests/layout_browser_parity/corpus.toml`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`

- [ ] **Step 1: Add a failing manifest existence test.**

Add a test to `crates/surgeist/tests/layout_browser_parity.rs`:

```rust
#[test]
fn browser_parity_corpus_manifest_exists() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout_browser_parity/corpus.toml");
    assert!(
        manifest.is_file(),
        "expected browser parity corpus manifest at {}",
        manifest.display()
    );
}
```

Run:

```sh
cargo test -p surgeist --test layout_browser_parity browser_parity_corpus_manifest_exists
```

Expected: FAIL because `corpus.toml` does not exist.

- [ ] **Step 2: Create the manifest skeleton.**

Create `crates/surgeist/tests/layout_browser_parity/corpus.toml` with:

```toml
schema_version = 1

[source_roots.taffy]
kind = "taffy"
path = "html"
upstream_commit = "d1ff7e339b9ee35b33858779f8d7653197e93d92"
description = "Pinned Taffy browser fixture baseline copied from target/surgeist-sources/taffy/<commit>/test_fixtures."

[source_roots.surgeist]
kind = "surgeist"
path = "html"
description = "Surgeist-authored constrained HTML fixtures layered on top of the Taffy baseline."

[source_roots.wpt]
kind = "wpt"
path = "wpt"
description = "Raw upstream WPT HTML organized by layout domain."
```

- [ ] **Step 3: Add an inventory test for the Taffy baseline count.**

Add:

```rust
#[test]
fn browser_parity_taffy_baseline_count_is_documented() {
    let html_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout_browser_parity/html");
    let mut non_wpt_count = 0usize;
    for file in support::fixture_files_in(&html_root, "html")
        .expect("html fixtures should be readable")
    {
        let path = file.to_string_lossy();
        if !path.contains("/subgrid/") && !path.contains("/grid-lanes/") {
            non_wpt_count += 1;
        }
    }
    assert_eq!(
        non_wpt_count, 1106,
        "expected the Taffy baseline plus three Surgeist named-grid additions"
    );
}
```

If `fixture_files_in` does not exist yet, add it in Task 3 instead and keep this test as the first failing test for that helper.

- [ ] **Step 4: Update README with the target layout.**

Document:

```text
html/ contains currently active constrained HTML.
wpt/ contains raw upstream WPT HTML grouped by Surgeist layout domain.
xml/ contains generated browser expectations and should be regenerated, not edited.
corpus.toml is the auditable source/provenance/quarantine manifest.
```

- [ ] **Step 5: Run and commit.**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity browser_parity_corpus_manifest_exists
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Commit:

```sh
git add crates/surgeist/tests/layout_browser_parity.rs crates/surgeist/tests/layout_browser_parity/README.md crates/surgeist/tests/layout_browser_parity/corpus.toml
git commit -m "Add parity corpus manifest"
```

## Task 2A: Add Taffy Baseline Import And Integrity Check

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate.rs`
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/corpus.toml`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`

- [ ] **Step 1: Add the Taffy source contract to the manifest.**

Extend `corpus.toml` with a source import contract:

```toml
[imports.taffy]
repo = "https://github.com/DioxusLabs/taffy.git"
commit = "d1ff7e339b9ee35b33858779f8d7653197e93d92"
source_dir = "test_fixtures"
destination = "html"
expected_count = 1103
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/grid_named_negative_occurrence"
source_root = "surgeist"
source = "grid/grid_named_negative_occurrence.html"
generator = "constrained-html"
status = "active"
```

- [ ] **Step 2: Add a read-only check command.**

Add a generator command mode such as:

```sh
cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

The command should:

- Read the Taffy import contract.
- Resolve the pinned source from a durable cache such as `target/surgeist-sources/taffy/d1ff7e339b9ee35b33858779f8d7653197e93d92`.
- Fail if the cache is missing and print the exact import command to fetch it.
- Compare the `1,103` expected Taffy HTML files byte-for-byte against `html/`.
- Fail on unexpected non-Taffy extras outside manifest-declared local constrained cases.
- Fail on junk files such as `html/.DS_Store`.

- [ ] **Step 3: Add an import command.**

Add a generator command mode such as:

```sh
cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
```

The command should clone/fetch the pinned Taffy commit into `target/surgeist-sources/taffy/<sha>`, copy only `.html` fixtures from `test_fixtures` into `html/`, and leave `subgrid/`, `grid-lanes/`, and manifest-declared Surgeist constrained cases untouched. It must be idempotent: running it twice should produce no diff.

- [ ] **Step 4: Run and commit.**

Run:

```sh
cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
cargo fmt --check
```

Commit:

```sh
git add crates/surgeist/src/bin/surgeist-layout-generate.rs crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/corpus.toml crates/surgeist/tests/layout_browser_parity/README.md
git commit -m "Check Taffy parity baseline"
```

## Task 3: Make Fixture Discovery Explicit And Non-Skipping

**Files:**
- Modify: `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`

- [ ] **Step 1: Add tests for deterministic discovery and filtering.**

Add tests that prove:

```rust
#[test]
fn fixture_discovery_is_sorted() {
    let files = support::fixture_files("xml").expect("fixtures should load");
    let mut sorted = files.clone();
    sorted.sort();
    assert_eq!(files, sorted);
}

#[test]
fn fixture_discovery_does_not_hide_quarantine_by_filename() {
    assert!(
        !support::fixture_skip_policy_mentions_x_prefix(),
        "quarantine must be manifest-driven, not filename-prefix-driven"
    );
}

#[test]
fn fixture_discovery_does_not_silently_skip_unsupported_constructs() {
    assert!(
        !support::fixture_skip_policy_filters_unsupported_constructs(),
        "unsupported constructs must be reported as buckets, not removed from discovery"
    );
}
```

- [ ] **Step 2: Extract discovery helpers.**

Change `support.rs` to expose a helper shaped like:

```rust
pub fn fixture_files_in(root: &Path, extension: &str) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    collect_files_with_extension(root, extension, &mut files)?;
    files.sort();
    Ok(files)
}
```

Keep the old `fixture_files("xml")` public API intact, but route it through the helper.

- [ ] **Step 3: Remove implicit skip policy from discovery.**

Do not keep known unsupported cases hidden from the normal run. If a fixture contains unsupported constructs, the runner should include it and report a classified failure such as `unsupported: br element` or `unsupported: inline text run`, not remove it from the fixture list. If temporary isolation is necessary while the bucket code lands, put it behind a clearly named legacy function and delete that function in Task 8:

```rust
fn legacy_supported_fixture_file(path: &Path) -> bool {
    ...
}
```

Then add `fixture_skip_policy_mentions_x_prefix()` and `fixture_skip_policy_filters_unsupported_constructs()` returning `false`, because `x*.html`, `<br>`, and mixed inline text skips must be replaced by explicit bucketed reporting.

- [ ] **Step 4: Run and commit.**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity fixture_discovery_is_sorted
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Commit:

```sh
git add crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout_browser_parity.rs
git commit -m "Make parity fixture discovery non-skipping"
```

## Task 4: Make Scoped Generation Idempotent

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`

- [ ] **Step 1: Add unit tests for filter matching and stale output removal.**

Move pure helper tests into the generator module under `#[cfg(test)]`:

```rust
#[test]
fn fixture_filter_matches_directory_or_stem() {
    let root = Path::new("html");
    assert!(fixture_matches_filter(root, Path::new("html/grid/basic.html"), "grid"));
    assert!(fixture_matches_filter(root, Path::new("html/grid/basic.html"), "grid/basic"));
    assert!(!fixture_matches_filter(root, Path::new("html/flex/basic.html"), "grid"));
}

#[test]
fn output_paths_for_fixture_include_all_variants() {
    let paths = output_paths_for_fixture(
        Path::new("html"),
        Path::new("xml"),
        Path::new("html/grid/basic.html"),
    )
    .expect("paths should be computed");
    assert_eq!(paths.len(), 4);
    assert!(paths.iter().any(|path| path.ends_with("grid/basic__border_box_ltr.xml")));
}
```

- [ ] **Step 2: Add output path computation.**

Implement:

```rust
fn output_paths_for_fixture(
    html_root: &Path,
    xml_root: &Path,
    fixture: &Path,
) -> Result<Vec<PathBuf>, String> {
    let rel = fixture.strip_prefix(html_root).map_err(|error| {
        format!("failed to make fixture path relative to {}: {error}", html_root.display())
    })?;
    let group = rel.parent().unwrap_or_else(|| Path::new(""));
    let stem = fixture
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("fixture has no UTF-8 stem: {}", fixture.display()))?;
    Ok(fixture_cases()
        .into_iter()
        .map(|(variant, _)| xml_root.join(group).join(format!("{stem}__{variant}.xml")))
        .collect())
}
```

- [ ] **Step 3: Remove stale outputs before regenerating each selected fixture.**

At the start of `write_fixture_goldens`, delete all paths from `output_paths_for_fixture`. This makes scoped generation idempotent:

```rust
for stale in output_paths_for_fixture(&config.html_root, &config.xml_root, fixture)? {
    std::fs::remove_file(stale).ok();
}
```

- [ ] **Step 4: Run and commit.**

Run:

```sh
cargo test -p surgeist --bin surgeist-layout-generate
cargo fmt --check
```

Commit:

```sh
git add crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/README.md
git commit -m "Make parity generation idempotent"
```

## Task 5: Add Generated Provenance

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`

- [ ] **Step 1: Add a parser test for XML provenance comments.**

Add:

```rust
#[test]
fn parses_generated_xml_with_provenance_comment() {
    let golden = support::Golden::parse(
        r#"
        <!-- generated-by: surgeist-layout-generate schema=1 source=html/block/basic.html source-sha256=abc helper-sha256=def browser=Chrome/149 -->
        <test name="with-provenance" use-rounding="true">
            <viewport width="max-content" height="max-content" />
            <input><div /></input>
            <expectations><node x="0" y="0" width="0" height="0" /></expectations>
        </test>
        "#,
    )
    .expect("provenance comments should not break parsing");
    assert_eq!(golden.name, "with-provenance");
}
```

- [ ] **Step 2: Emit provenance comments.**

Change `generate_xml` to accept a `GeneratedProvenance` struct:

```rust
struct GeneratedProvenance {
    source: String,
    source_sha256: String,
    helper_sha256: String,
    browser: String,
}
```

Put the provenance comment before `<test>`.

- [ ] **Step 3: Hash source and helper.**

Use a deterministic SHA-256 helper. If the crate lacks `sha2`, prefer adding it only to the generator's existing dependency scope if possible. Do not use ad hoc partial hashes.

- [ ] **Step 4: Run and commit.**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity parses_generated_xml_with_provenance_comment
cargo test -p surgeist --bin surgeist-layout-generate
cargo fmt --check
```

Commit:

```sh
git add crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout_browser_parity.rs Cargo.lock crates/surgeist/Cargo.toml
git commit -m "Record parity fixture provenance"
```

## Task 6: Add Raw WPT Source Root

**Files:**
- Create: `crates/surgeist/tests/layout_browser_parity/wpt/`
- Modify: `crates/surgeist/tests/layout_browser_parity/corpus.toml`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`

- [ ] **Step 1: Create raw WPT directories.**

Create:

```text
wpt/block/
wpt/flex/
wpt/grid/
wpt/grid-lanes/
wpt/subgrid/
wpt/inline/
wpt/sizing/
wpt/alignment/
wpt/overflow/
wpt/writing-modes/
wpt/box/
wpt/resources/
wpt/manifests/
wpt/expectations/
```

- [ ] **Step 2: Copy the first raw WPT seed files into domain folders without assertion splitting.**

Start with the WPT files already referenced by existing manually converted fixtures:

```text
css/css-grid/subgrid/alignment-in-subgridded-axes-001.html
css/css-grid/subgrid/subgrid-baseline-005.html
css/css-grid/subgrid/subgrid-baseline-006.html
css/css-grid/subgrid/subgrid-baseline-007.html
css/css-grid/subgrid/subgrid-baseline-008.html
css/css-grid/subgrid/subgrid-baseline-009.html
css/css-grid/subgrid/subgrid-stretch.html
css/css-grid/subgrid/line-names-004.html
css/css-grid/grid-lanes/grid-lanes-container-minimum-size-single-axis-scroll-container.html
```

Copy them into domain paths, for example:

```text
crates/surgeist/tests/layout_browser_parity/wpt/subgrid/alignment-in-subgridded-axes-001.html
crates/surgeist/tests/layout_browser_parity/wpt/grid-lanes/grid-lanes-container-minimum-size-single-axis-scroll-container.html
```

- [ ] **Step 3: Rewrite resource dependencies.**

For every copied WPT file, inspect `<link>`, `<script>`, `@import`, CSS `url(...)`, reftest `<link rel="match">`, reftest `<link rel="mismatch">`, image/video URLs, and relative reference URLs. Copy only required resource files into domain-local support folders or `wpt/resources/`, then rewrite links so browser loading works from the new domain path.

Required shared harness/support families include:

```text
/resources/testharness.js
/resources/testharnessreport.js
/resources/check-layout-th.js
/resources/testdriver.js
/resources/testdriver-actions.js
/resources/testdriver-vendor.js
/common/reftest-wait.js
/common/rendering-utils.js
/common/blank.html
/common/get-host-info.sub.js
/fonts/ahem.css
/fonts/Ahem.ttf
/css/support/*
/css/reference/*
```

Keep CSS-local `reference/`, `references/`, `support/`, and `resources/` directories beside copied tests.

- [ ] **Step 4: Add manifest entries.**

Add `[[cases]]` entries for each raw WPT source with `generator = "wpt-multi-assertion"` and `status = "active"` when selectors are known. If selectors are not known yet, use `status = "unsupported"` only as a counted reporting bucket with a reason such as `needs assertion selector mapping`; do not silently skip it.

Add domain manifests such as `wpt/manifests/subgrid.toml` and `wpt/manifests/grid_lanes.toml` mapping friendly filter names to imported domain paths and original upstream paths. The imported domain path is the runtime path; the upstream path is provenance only.

- [ ] **Step 5: Run and commit.**

Run:

```sh
find crates/surgeist/tests/layout_browser_parity/wpt -type f | sort
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Commit:

```sh
git add crates/surgeist/tests/layout_browser_parity/wpt crates/surgeist/tests/layout_browser_parity/corpus.toml crates/surgeist/tests/layout_browser_parity/README.md
git commit -m "Add raw WPT parity corpus seed"
```

## Task 7: Add Multi-Assertion WPT Fan-Out

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js`
- Modify: `crates/surgeist/tests/layout_browser_parity/corpus.toml`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`

- [ ] **Step 1: Add a helper-level test fixture for multi-case output.**

Create a tiny raw WPT-like HTML fixture under a test-only temporary directory in the generator tests:

```html
<div id="case-a" data-expected-width="50" style="display:grid;width:50px;height:20px"></div>
<div id="case-b" data-expected-width="70" style="display:grid;width:70px;height:20px"></div>
```

The generator test should assert that one source fixture can produce output names:

```text
wpt/grid/example__case-a__border_box_ltr.xml
wpt/grid/example__case-b__border_box_ltr.xml
```

- [ ] **Step 2: Introduce generated case IDs.**

Add a `GeneratedCase` struct:

```rust
struct GeneratedCase {
    id: String,
    variant: &'static str,
    data: serde_json::Value,
}
```

For constrained HTML, generate one case per variant with the current stem. For raw WPT, generate one case per assertion id per variant.

- [ ] **Step 3: Extend `test_helper.js` for selector-driven cases.**

Add a function that accepts assertion selectors from the manifest and returns a map:

```javascript
window.getSurgeistWptCases = function(selectors) {
  return Object.fromEntries(selectors.map(({ id, selector }) => {
    const root = document.querySelector(selector);
    if (!root) throw new Error(`Missing WPT assertion selector ${selector}`);
    return [id, getTestDataForRoot(root)];
  }));
};
```

Keep existing `getTestData()` behavior for constrained HTML.

- [ ] **Step 4: Wire manifest assertion selectors into generation.**

For `generator = "wpt-multi-assertion"`, call the new helper with selectors from the manifest and emit one XML file per assertion/variant.

- [ ] **Step 5: Run and commit.**

Run:

```sh
cargo test -p surgeist --bin surgeist-layout-generate
SURGEIST_LAYOUT_GENERATE_FILTER=wpt/subgrid/alignment-in-subgridded-axes-001 cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Commit:

```sh
git add crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/scripts/gentest/test_helper.js crates/surgeist/tests/layout_browser_parity/corpus.toml crates/surgeist/tests/layout_browser_parity/README.md crates/surgeist/tests/layout_browser_parity/xml
git commit -m "Generate WPT multi-assertion parity cases"
```

## Task 8: Replace Hidden Skips With Pass/Fail Bucketing

**Files:**
- Modify: `crates/surgeist/src/bin/surgeist-layout-generate/generator.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/support.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity.rs`
- Modify: `crates/surgeist/tests/layout_browser_parity/corpus.toml`

- [ ] **Step 1: Add tests for explicit status handling.**

Add parser/manifest tests proving `unsupported`, `quarantined`, and `expected-fail` are visible in a report and not hidden by filename or construct scanning.

- [ ] **Step 2: Stop skipping `x*.html` in `collect_html_into`.**

Remove:

```rust
&& !path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with('x'))
```

Instead, require any formerly skipped fixture to have a manifest status.

- [ ] **Step 3: Remove Rust-side source HTML filtering from normal fixture discovery.**

Delete `source_has_unsupported_inline_semantics` from `fixture_files("xml")` once equivalent bucket reporting exists. `<br>` and mixed inline text must produce classified output, not fixture removal.

- [ ] **Step 4: Emit a generation/reporting summary.**

Write scope-specific reports under `xml/generation-reports/` with generated, unsupported, quarantined, expected-fail, and failed-to-generate counts. Do not include a generic `skipped` bucket unless it is zero or broken down into explicit user-visible reasons.

- [ ] **Step 5: Run and commit.**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
cargo test -p surgeist --bin surgeist-layout-generate
cargo fmt --check
```

Commit:

```sh
git add crates/surgeist/src/bin/surgeist-layout-generate/generator.rs crates/surgeist/tests/layout_browser_parity/support.rs crates/surgeist/tests/layout_browser_parity.rs crates/surgeist/tests/layout_browser_parity/corpus.toml crates/surgeist/tests/layout_browser_parity/xml
git commit -m "Bucket parity corpus failures explicitly"
```

## Task 9: Refresh Commands And Documentation

**Files:**
- Modify: `justfile`
- Modify: `crates/surgeist/tests/layout_browser_parity/README.md`

- [ ] **Step 1: Fix stale README commands.**

Document the actual current commands:

```sh
just test parity
just test parity generate
just test parity generate subgrid
cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

- [ ] **Step 2: Add WPT filter examples.**

Document:

```sh
SURGEIST_LAYOUT_GENERATE_FILTER=wpt/subgrid cargo run -p surgeist --features layout-golden-generate --bin surgeist-layout-generate
SURGEIST_PARITY_FILTER=wpt/subgrid cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

- [ ] **Step 3: Document helper-only `scripts/gentest` fate.**

The old `crates/surgeist/tests/layout_browser_parity/scripts/gentest/src/main.rs` path should not exist. Keep `test_helper.js` and `test_base_style.css` because the Rust generator includes/loads them, document that `scripts/gentest` is helper-only, and guard that directory in corpus checks.

- [ ] **Step 4: Run and commit.**

Run:

```sh
just --list
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
```

Commit:

```sh
git add justfile crates/surgeist/tests/layout_browser_parity/README.md crates/surgeist/tests/layout_browser_parity/scripts/gentest
git commit -m "Document parity corpus generation"
```

## Task 10: Review Cycle And Final Verification

**Files:**
- All files changed by Tasks 2-9.

- [ ] **Step 1: Dispatch clean-context reviewers.**

Use at least two reviewers:

```text
Reviewer A: Check idempotence and generator provenance against this plan.
Reviewer B: Check raw WPT corpus shape, multi-assertion fan-out, and quarantine accounting.
```

- [ ] **Step 2: Implement reviewer recommendations.**

Use focused commits for accepted recommendations.

- [ ] **Step 3: Run focused verification.**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity parses_all_checked_in_browser_parity_xml
cargo test -p surgeist --bin surgeist-layout-generate
cargo fmt --check
```

- [ ] **Step 4: Run full parity signal and record the current expected failures.**

Run:

```sh
cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

Expected at plan start: the command fails with `76` geometry failures in `grid-lanes` and `subgrid`. The corpus consolidation goal is not required to fix layout geometry, but it must not hide or silently rewrite these failures.

- [ ] **Step 5: Final commit if needed.**

Commit any review/verification updates:

```sh
git add <reviewed-files>
git commit -m "Review parity corpus consolidation"
```

## Completion Checklist

- [ ] The Taffy green baseline is explicitly documented and reproducible from a durable source, not from untracked `tmp/`.
- [ ] Raw WPT HTML lives under `crates/surgeist/tests/layout_browser_parity/wpt/` and is filterable by layout domain.
- [ ] The generator can handle one raw WPT source producing multiple generated cases.
- [ ] All XML comes from `surgeist-layout-generate`.
- [ ] Scoped generation deletes stale XML for selected fixtures.
- [ ] Generated XML or a sidecar report records source/generator/browser provenance.
- [ ] Unsupported, expected-fail, and quarantined fixtures are explicit, auditable, and counted in output.
- [ ] Hidden filters based on `x` prefixes, `<br>`, inline text scans, or manual XML drift are removed.
- [ ] README and `justfile` commands match reality.
- [ ] Clean-context reviewer recommendations have been implemented or explicitly rejected with reasons.
- [ ] Logical commits exist at the checkpoints above.

## Self-Review

- Spec coverage: This plan covers the one-root parity corpus, `html/` active constrained fixtures, new `wpt/` raw WPT fixtures, Taffy baseline provenance, single-generator XML output, idempotent scoped generation, multi-assertion WPT fan-out, explicit quarantine/expected-failure accounting, documentation, review cycles, and logical commits.
- Placeholder scan: No step depends on a `TODO` or unspecified behavior. Unknown upstream WPT commit values are deliberately represented as manifest fields to fill when the raw files are copied from the selected local source checkout.
- Type consistency: The plan keeps the existing `Golden` one-XML-one-case parser shape and introduces WPT fan-out in the generator rather than requiring a multi-case XML parser in the first pass.
