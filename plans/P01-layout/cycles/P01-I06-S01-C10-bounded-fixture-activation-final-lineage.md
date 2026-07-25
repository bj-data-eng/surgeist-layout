# P01-I06-S01-C10 Bounded Fixture Activation And Final Lineage

Status: superseded

Cycle ID: `P01/I06/S01/C10`

Owning repository: `surgeist-layout`

Cycle base: `bcdba3c49be09ad119c03ecdc4c77da803159132`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`702fab3acac6c66b22333f5120212ab36e365c7b6a00734d70285c583fb3c212`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`: `FRI-06.4 D-01`,
`D-04`, `D-09`, `D-11`, `D-13`, and `D-16`; the applicable metric,
control, bidi, float, fixture, compatibility, and acceptance portions of
`FRI-06.5`, `FRI-06.7`, `FRI-06.9` through `FRI-06.11`, and
`FRI-06.14`.

Reviewed implementation sequence:
`plans/P01-layout/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`
at SHA-256
`b608fe1864b3bb34b4ef293055cc9d1015ec4fb6595295c7bd8f123d675c6b52`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `P01/I06/S01/C10`
and its Activation Recovery Evidence.

## 1 Outcome

Correct the exact post-generation categories in the fixed 388-row activation
union, preserve the affected base-corpus semantics, freeze every changed input,
then derive one valid full unfiltered 5,712/16 corpus lineage. No unchanged-input
generation, expected-failure reclassification, or generator architecture
expansion is permitted.

## 2 Boundary And Evidence

P01/I06/S01/C09 is published and remotely verified at the cycle base. C08 owns only
the bounded helper/serializer, twelve named HTML fixtures, strict
parser/comparator, finite fixture adapter, confirmed inline traversal correction,
focused tests, and generated XML/report lineage. Public model shape, dependencies,
features, lockfile, MSRV, browser policy, launch profile, base style, root,
siblings, task-runner architecture, and later-owned layout behavior do not change.

Implementation through `d8941011bd56b9371b8ef3bff9254fab6be08e14`
received the then-applicable task reviews. The subsequent full run supplied new
evidence and invalidated acceptance for every correction slice it touches. No
prior verdict substitutes for the fresh complete task reviews required below.

The retained diagnostic run is recorded immutably in
`plans/P01-layout/P01-I06-S01-C10-post-generation-census.md`,
SHA-256 `2c4179f559c5fa9e93c6933e0ba1a4969b758fc4a4f738d619c2751796b8bf00`,
commit `96206cb2da33dae354eba90661ff8c5823fe7928`. Its exact facts are:

- one full unfiltered invocation, exit zero, and no scoped invocation;
- report SHA-256
  `65d88aa1b13f813392e27690d3f5ac9b79a2bbbc1cdb86ad78328665e3aeecd0`;
- 5,712 generated, exactly 16 missing-root unsupported, and zero other buckets;
- valid pinned browser, launch, helper, manifest, base-style, and Taffy
  provenance; and
- 94 passing and 294 failing rows in the fixed 388-row union, partitioned into
  130 placement, 72 control, 52 height, four scroll, eight float/clear, four
  Range-origin, four shape-band, four finite later-owned adapter, and 16 parser
  rows.

That run is diagnostic only and the committed census is its durable evidence.
Before T01's first implementation write, its worker restores the uncommitted
drafts in `tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs` plus every tracked generated artifact to HEAD,
removes only the untracked XML outputs enumerated by the diagnostic report, and
proves no other path changed. The restored checked-in artifacts become T06's
stale-entry RED. No worker hand-edits generated XML or a report.

Scoped generation is optional during correction work only when a worker needs a
new diagnostic after a material input change. It is never required, retained, or
reported as verification evidence. No full generation occurs in T01 through T05.
After all changed inputs are task-clean and frozen, T06 runs the full generator
exactly once. A nonzero exit or unexpected output stops the lineage; it is never
retried unchanged.

## 3 Impacts

- **Public API and compatibility:** unchanged.
- **Production behavior:** T04 completes only shaped/boundary RTL traversal
  already required by FRI-06; no float-rounding or shared compute change.
- **Fixtures and artifacts:** bounded helper/parser/HTML corrections precede one
  full derived XML/report replacement.
- **Generator:** only narrow serializer tests or a confirmed genuine bug may
  change; architecture, acquisition, browser policy, and task running do not.
- **Docs and root:** planning evidence only; C09 owns public closure and the leaf
  candidate handoff.
- **Safety and lints:** no Surgeist-owned unsafe and no new `allow` or
  `expect` attribute.

## 4 Tasks

### 4.1 `P01/I06/S01/C10/T01` Correct Helper And Serializer Semantics

**Files/area:** `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
narrow serialization in `tests/bin/surgeist-layout-generate/generator.rs`,
focused tests, and only the static HTML/manifest-target inventory assertions in
`tests/layout/browser_parity.rs`.

**Precondition:** Preserve the committed census, discard only the two exact draft
paths and invalid generated lineage named above, and prove the remaining worktree
matches HEAD before adding RED.

**Outcome:** Make Range starts local to the nearest explicit layout-ready inline
root; emit model control participation only for an explicitly computed/lowered
inline BR role; retain blockified BR as an ordinary box; and suppress legacy raw
text fallback when typed inline children exist. Serializer state rejects malformed
or non-BR control facts and preserves the 64 spill variants' prior semantics.
Reconcile the already-committed twelve-source inventory to 1,432 total HTML
sources, 1,186 outside subgrid/grid-lanes, and the manifest's 5,712 target.
Checked-in XML/report assertions remain at stale-entry 5,324/356 until T06.

**RED:** Focused helper/serializer tests reproduce the direct-parent Range
offset, source-tag BR promotion, malformed control acceptance, and duplicate text
fallback. Negative controls prove authored tags and activation ancestors are
insufficient. The full generator gate separately reproduces the stale 1,420 and
1,174 static HTML inventory assertions.

**Acceptance:** Exact root-local Range, explicit-control, blockification,
replacement, and 64-variant preservation matrices pass without XML/report
changes or generation. Static inventory agrees with the committed HTML and
manifest target while retaining stale-entry XML/report expectations. A fresh
reviewer covers the complete reopened T01 range.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_t1_
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** the reviewed plan and committed post-generation census.

**Intended commit:** `fix(parity): preserve lowered inline fixture roles`.

### 4.2 `P01/I06/S01/C10/T02` Correct The Named HTML Fixture Oracles

**Files/area:** only the named FRI-06 mixed-wrap, BFC-avoidance, and shape-
exclusion HTML sources plus focused source tests.

**Outcome:** Add the allowed break after the first 18px atomic in
`fri06_inline_mixed_text_atomic_wrap`; remove text labels that create unintended
scroll overflow in `fri06_float_bfc_avoidance`; and encode the two observed
shape query bands `0..21.2` and `21.2..37.2`, both returning `0..44`, in
`fri06_float_shape_exclusion`. Do not add a source, manifest record, layout
feature, or general shape representation.

**RED:** Focused source/query-recorder tests fail on the absent break, nonzero
scroll caused by labels, and guessed `0..20/20..40/40..60` bands.

**Acceptance:** The three exact source matrices and negative controls pass with
the twelve-source/48-variant inventory and manifest hash otherwise unchanged.
No generation or artifact change occurs. A fresh reviewer covers complete T02.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_t2_
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T01 is task-clean.

**Intended commit:** `test(parity): correct bounded C08 fixture oracles`.

### 4.3 `P01/I06/S01/C10/T03` Correct Observation, Comparator, And Token Parsing

**Files/area:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused tests.

**Outcome:** Keep Range observations limited to source, line, flow-inline start,
and advance. For nonwrapping flex, compare terminal source position and adjacent
flex-line membership without browser BR ink or model-control geometry; wrapped
flex remains rejected. Parse `inline-start`/`inline-end` as line-relative
float/clear aliases, `none` directly, and clear-only `both`; reject unrelated
tokens.

**RED:** Exact Range, 72 control, 24 masked RTL flex probes, and token-table tests
reproduce the current category mixing and missing aliases. Wrapped-flex,
malformed-token, and BR-geometry controls fail closed.

**Acceptance:** Category boundaries and exact matrices pass without weakening
model fragment/control assertions. No helper, HTML, production, generated output,
or general CSS parser changes. A fresh reviewer covers the complete reopened T03
range.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_r0_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T02 is task-clean.

**Intended commit:** `test(parity): separate C08 browser observations`.

### 4.4 `P01/I06/S01/C10/T04` Complete RTL Shaped-Unit Traversal

**Files/area:** `src/inline.rs` and focused inline/public-compute tests only.

**Outcome:** Traverse the complete reordered shaped, boundary, and atomic unit
sequence whenever the containing RTL inline progression decreases. Preserve
stable visual identities, source association, LTR, vertical/sideways projection,
whitespace, baselines, line metrics, rounding phase, and all-float behavior.

**RED:** Public-compute regressions reproduce the four bidi Range rows and the
affected placement rows with mixed shaped/boundary/atomic units. Existing
all-atomic and whitespace controls remain green.

**Acceptance:** Exact mixed-unit RTL geometry passes in both scalar lanes and all
ten flow mappings without a shared `compute.rs`, float-rounding, fixture, or
artifact change. A fresh reviewer covers the complete reopened T04 range.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c08_r1_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T03 is task-clean.

**Intended commit:** `fix(layout): complete RTL shaped unit traversal`.

### 4.5 `P01/I06/S01/C10/T05` Complete The Finite Fixture Adapter

**Files/area:** `tests/layout/browser_parity/support.rs` and focused adapter
tests only.

**Outcome:** Match computed `grid` after blockification for the exact 16 parser
rows; keep blockified BR as a box; map physical CSS float/clear left/right only
when `FlowAxes` identifies that side as inline start/end; reject block-axis
physical sides in all vertical/sideways flows; add required continuation struts;
and group the four typed shaped runs in the one finite anonymous grid wrapper
without duplicate raw text.

**RED:** Source-equivalent adapter tests reproduce all parser, placement, height,
float/clear, and later-owned rows. The lowering table enumerates all five writing
modes by both directions, including two horizontal mappings and eight
vertical/sideways physical-side rejections.

**Acceptance:** Exact finite structures lower through production constructors;
`inline-grid`, altered topology, wrapped flex, unrelated displays, physical
block-axis float/clear, and duplicate fallback remain rejected. No general display
normalizer, CSS parser, text shaper, generator, or artifact change. A fresh
reviewer covers the complete reopened T05 range.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_r2_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T04 is task-clean.

**Intended commit:** `test(parity): complete finite C08 adapter lowering`.

### 4.6 `P01/I06/S01/C10/T06` Derive And Verify The Final Lineage

**Files/area:** final-lineage tests in
`tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`, generated XML, and
`tests/layout/browser_parity/xml/generation-reports/all.json`. All helper,
HTML, parser, comparator, adapter, production, and manifest inputs are read-only
at their reviewed frozen hashes; T06 changes only test modules in the two Rust
sources and does not alter generator behavior.

**Outcome:** Confirm the stale-entry RED and the exact pinned-browser preflight,
then run one full unfiltered existing-pinned generation. Verify the resulting
report, all 388 activated comparisons, 5,324 base-output preservation, and the
absence of scoped reports or untracked artifacts.

**RED:** Add final-accounting, fixed-matrix, semantic-preservation, and provenance
tests. Before generation they fail only because the restored report/artifacts
retain entry accounting. Input-freeze and semantic-oracle tests are green.

**Generation:** Run this command exactly once after the task-clean freeze:

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

The first T06 assignment stopped without retry after invoking Cargo with the
noncanonical browser prefix `target/surgeist-browser/chrome-mac-arm-149.0.7827.115/`.
Executable canonicalization failed before browser launch or artifact writes;
the prescribed command above was not executed, and the stale report and XML
aggregate retained their entry hashes. This reviewed amendment authorizes one
replacement assignment to preserve the completed RED tests and execute the
exact command above once. It must first reconfirm the executable, frozen inputs,
stale artifact hashes, and absence of generator output from the invalid attempt.
Any failure or unexpected output from the replacement stops the lineage with no
further invocation.

The replacement preflight is exact. Prove `HEAD^` equals
`991f31d5fe446efaea09da0f6a59aa451472af63`, `rev-list --count` from that T05
revision is one, and `diff-tree --name-only` for HEAD contains only this cycle
plan. Exact porcelain v1 status, including untracked files, must contain only
unstaged modifications to `tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`; any other line fails. Against T05,
`git diff --quiet` must pass for `Cargo.toml`, `Cargo.lock`, `Justfile`, `src/`,
`corpus.toml`, complete `html/` and `scripts/gentest/` trees, `support.rs`, XML,
and the report. Update and run only
`fri06_c08_t3_inputs_match_reviewed_terminal_freeze` with all four overrides
unset: it must separately verify the specification, sequence, census, production
generator, manifest, helper, base-style, source-set, task-runner, and Rust input
digests, plus this cycle plan's normalized hash from the fresh review receipt;
the whole-`plans/` diff check is replaced by those exact planning checks.
`test -x` for the prescribed browser and `--version` `149.0.7827.115` must pass.
The stale report hash below and the stale XML aggregate below, computed from
sorted `shasum -a 256` output for every tracked `*.xml`, must match. Record every
status and value before the replacement.

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_t3_inputs_match_reviewed_terminal_freeze
```

**Acceptance:** The report records `filter: null`, 5,712 generated, exactly 16
missing-root unsupported variants, zero other buckets, and exact browser, launch,
helper, manifest, base-style, and Taffy provenance. All 388 rows pass; the other
5,324 outputs preserve semantics; generated changes are complete and derived.
The single invocation and output hashes are recorded. A fresh reviewer covers T06.

**Commands after generation:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T01, T02, T03, T04, and T05 are freshly task-clean; every input hash is
frozen; the pinned browser reports `149.0.7827.115`; no generator override or
filter is set. The invalid preflight attempt left report SHA-256
`4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab` and XML
aggregate SHA-256
`d8fad6bbab9ad0b5bece5299a983e588935cfd591d9430d38ddac900ec9eea1d`
unchanged.

**Intended commit:** `test(parity): derive final FRI-06 browser lineage`.

## 5 Completion

All six complete task ranges receive independent `CLEAN` verdicts. T06's
recorded input hashes equal the reviewed task heads. The cycle has one valid full
lineage, no retained scoped output, no hand-edited artifact, no new source or
bucket, no broad lint suppression, and no generator architecture expansion. Make
the separate status-only `complete` commit and set `cycle_head` before final
checks. A later failure first receives a status-only `in_progress` commit.

Run the configured full gates and focused activation comparison:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_
git diff --check bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD
git ls-files --cached --others --exclude-standard '*.rs'
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(git ls-files --cached --others --exclude-standard '*.rs')
git diff --unified=0 bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD -- '*.rs' | rg '^\+.*#\s*\[\s*(allow|expect)\s*\('
```

The owned-Rust manifest must be nonempty. The unsafe and added-allowance scans
must return no matches with status 1; any match or tool error fails. Record their
outcomes with the exact changed-path inventory, clean worktree, report/artifact
hashes, test totals, and task verdict scopes. A fresh
`surgeist-holistic-reviewer` must return `CLEAN` for exact range
`bcdba3c49be09ad119c03ecdc4c77da803159132..cycle_head`.

After the holistic verdict, rerun the required gates on local `main`, push by
fast-forward to authority `main`, fetch and read back the remote, and prove
local, tracking, `FETCH_HEAD`, and live remote agreement. Remove every
cycle-owned temporary resource. The handoff freezes all generator inputs and
outputs for read-only P01/I06/S01/C09 closure.

Blocker: none at planning time.
