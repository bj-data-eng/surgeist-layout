# FRI-05-C06 Bounded Fixtures Comparator And Final Regeneration
Status: in_progress
Cycle ID: `FRI-05-C06`
Owning repository: `surgeist-layout`
Cycle base: `2a3c96ac9b4ab65511ec15b1e59288a588d473c1`
Reviewed specification:
`plans/specs/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`747dcd6c12ae7d883999b5517572d6877d3c803bdb611143af7affc5afd44f39`,
commit `50c83f01ded0fe4a284e087ffcbd677bfc12af2a`, sections
`FRI-05.4 D-04`, `D-06`, `D-08`, `D-09`, `D-12`, and `D-13`; the fixture
and comparator portions of `FRI-05.8` and `FRI-05.9`; `FRI-05.11`; and
acceptance items 9 through 11 and 13 in `FRI-05.15`.
Reviewed sequence:
`plans/sequences/2026-07-16-surgeist-layout-fri-05-overflow-scroll-geometry.md`
at SHA-256
`69ac95b20bfc8e4827a620487168e2f23c94bdd8046fe6765961acc75b8a4816`,
commit `96b54aeae38059f2b65d8ebf0bfd4de952a70fde`, entry
`FRI-05-C06`.

## Outcome
Activate browser scroll expectation comparison against canonical physical range
spans, lower only the named computed-style scroll fields, add the exact eleven
active FRI-05 sources, remove the legacy authored-overflow transition, and
retain the complete frozen corpus derived by ExistingPinned. Close
the block reserved-gutter range-basis omission exposed by that corpus.

## Boundary
### Included
- range-span comparison and named wrong-x, wrong-y, and missing-geometry
  diagnostics for paired browser scroll expectations;
- computed-style helper capture, existing serializer attributes, and private
  fixture parsing for only the D-13 overflow, clip-margin, gutter,
  scroll-padding, scroll-margin, snap-type, snap-align, and snap-stop forms;
- exactly the eleven FRI-05 HTML sources and matching active manifest records;
- removal of authored overflow coupling after generated inputs emit both
  computed axes atomically;
- the completed frozen-manifest derivation, its 44 owned XML outputs, canonical
  updates to existing XML provenance, and sole `all.json` report;
- the narrow block range-basis correction required for reserved gutters to be
  excluded from range span while remaining in complete overflow;
- focused parity, corpus, Taffy, normal, generator-feature, provenance, and
  unsafe verification.

### Excluded
- production outside the named block range-basis correction, public API,
  unrelated layout algorithms, authored CSS parsing,
  root adapters, retained identity, current offsets, rendering, scrolling UI,
  snap selection, or later FRI behavior;
- changes to pre-existing HTML except a confirmed genuine source-input bug;
- general CSS tokens, CSS-wide keywords, relative units, `var()`, transforms,
  parser-only production variants, new report/schema kinds, browser policy,
  browser pin/profile, base style, task-runner recipes, dependencies, features,
  MSRV, documentation, root, or siblings;
- generator architecture expansion, hand-edited XML/report data, expected
  failures, quarantines, scoped reports, and the ignored aggregate
  `just parity-all` release gate.

### Current Evidence And Decisions
- C05 is published and remotely read back at the cycle base. Every owned
  formatting context now emits canonical scroll geometry.
- The current comparator parses paired `scroll_width` and `scroll_height`
  but does not compare them.
- The helper currently retains authored overflow axes, scrollbar width, and
  browser scroll/client dimensions. The serializer couples authored axes in
  the fixture adapter and emits no other FRI-05 scroll property.
- At cycle entry, checked-in XML contained 96 explicit legacy cross-group pairs.
  T4's settled inputs and derived corpus remove that transition without extending
  it to new properties.
- The frozen base has 1,409 HTML sources, 5,280 generated XML outputs, 356
  unsupported outputs, no failure classes, one `all.json`, no scoped report,
  and manifest SHA-256
  `d7e205908915b5330159d995429339d50179286d357a96153f385abcf3bfa072`.
- The already-cached executable is
  `target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`.
  No acquisition is authorized.
- No scoped ExistingPinned run occurred. T4 executed one unfiltered full run,
  invalidated it only for the confirmed source-input defects below, and consumed
  the one permitted replacement after those inputs settled.
- The settled full run exposed two indefinite-block-size source defects. After
  correcting only those HTML inputs, one permitted replacement full run produced
  the frozen corpus. Chromium then reported zero horizontal delta for the stable
  both-edge block because its scroll and client widths are both 70px; Surgeist
  reported 15px because block contributions retain a padding-box range basis.
  Flex and grid already select the accumulator's scrollport range basis. This is
  a confirmed block production omission, not a fixture-oracle defect. The frozen
  artifacts remain authoritative and no further generation is permitted.
- The settled 1,420-source manifest SHA-256 is
  `bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`.

## Impacts
- **Public API:** unchanged. **Production behavior:** block reserved gutters no
  longer create scroll range by themselves; complete overflow is unchanged.
- **Dependencies and features:** unchanged.
- **Generated artifacts:** eleven HTML sources produce 44 new XML outputs; the
  full XML corpus and sole report receive current generator-owned provenance.
- **Docs and examples:** unchanged.
- **MSRV:** Rust 1.97 and edition 2024 remain unchanged.
- **Root follow-up:** none until C07 candidate closure.
- **Unsafe:** prohibited in every tracked and non-ignored owned Rust file.

## Tasks
### `C06-T1` Activate Range-Span Comparator Diagnostics
**Files:** `tests/layout/browser_parity.rs`,
`tests/layout/browser_parity/support.rs`, and focused comparator tests.

**Outcome:** When a parsed expectation carries the paired scroll delta, require
canonical output geometry and compare its physical x/y range spans with the
existing scalar tolerance before descending into children.

**RED:** Add `fri05_c06_comparator_` tests first. Correct non-zero and zero
expectations currently pass without observation, so wrong x, wrong y, and
missing geometry do not produce their named mismatch.

**Acceptance:** Correct non-zero and explicit zero deltas pass. A wrong x span,
wrong y span, and absent geometry each fail with a stable named diagnostic.
Comparison uses `maximum - minimum` independently on each physical axis, not
an endpoint, absolute endpoint, content size, or overflow-rectangle size.
Paired parser presence remains strict and child comparison occurs only after
both spans pass.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_comparator_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```

**Dependency:** C05 canonical output is published.

**Intended commit:** `test(parity): compare canonical scroll range spans`.

### `C06-T2` Serialize And Parse The Finite Scroll Fixture Contract
**Files:** `tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity/support.rs`, and focused serializer and parser
tests. The browser helper remains byte-identical in this task.

**Outcome:** Serialize the exact D-13 computed-style values as only non-default
kebab-case attributes while emitting both computed overflow axes atomically,
and parse those attributes through existing checked production constructors
without widening the fixture adapter.

**RED:** Add `fri05_c06_serializer_` and `fri05_c06_parser_` tests first.
They fail because the serializer drops the named fields and the fixture parser
lacks their finite forms.

**Acceptance:** Serializer tests cover computed overflow,
`overflowClipMargin`, `scrollbarGutter`, four physical scroll-padding and
scroll-margin edges, `scrollSnapType`, `scrollSnapAlign`, and
`scrollSnapStop`. Defaults are omitted except that either non-default
overflow axis emits both computed axes. Parser tests accept exactly D-13's
keywords, finite values, affine length-percentage forms, and canonical snap
states and reject shorthand ambiguity, CSS-wide keywords, relative units,
`var()`, transforms, non-finite/out-of-domain values, wrong snap arity, and
invalid computed overflow pairs. Existing XML remains accepted through the
bounded transition until T4. The unchanged helper keeps old-corpus provenance
valid.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_serializer_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_parser_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```

**Dependency:** No semantic dependency on C06-T1; this task is independently
startable from the C05 base. It executes after T1 only because both tasks edit
`tests/layout/browser_parity/support.rs` in the sequential cycle worktree.

**Intended commit:** `test(parity): lower computed scroll fixture fields`.

### `C06-T3` Settle The Eleven Browser Sources
**Files:** exactly the eleven HTML paths listed in `FRI-05.11`,
`tests/layout/browser_parity.rs`, and narrowly required generator-feature
inventory tests. `corpus.toml`, XML, and reports remain unchanged.

**Outcome:** Add the bounded human-readable sources and an exact source/output
contract without generating derived artifacts or changing the frozen manifest.

**RED:** Add `fri05_c06_fixture_sources_` tests first and update the staged
HTML inventory assertions. They fail because the eleven sources and their exact
four-variant path matrix are absent and the base HTML counts still describe
1,409 sources.

**Acceptance:** Exactly these extensionless IDs exist under their specified
block, flex, grid, and grid-lanes directories:
`fri05_overflow_auto_cross_axis` in block/flex/grid,
`grid/fri05_hidden_auto_minimum`,
`grid-lanes/fri05_hidden_auto_minimum`,
`block/fri05_mixed_axis_clip_margin`,
`block/fri05_scrollbar_gutter_stable_both_edges`,
`flex/fri05_nested_zero_axis_overflow`,
`grid/fri05_nested_zero_axis_overflow`,
`grid/fri05_scroll_extent_area_origin`, and
`block/fri05_scroll_target_geometry`.
They use only existing constrained vocabulary plus D-13 fields, exercise the
named behavior rather than duplicate variants, and map to exactly four standard
outputs each. HTML inventory is exactly 1,420 total: 1,174 outside the subgrid
and grid-lanes suites, 219 subgrid, and 27 grid-lanes. XML, manifest, and report
count assertions remain at the base values until T4. No pre-existing source,
manifest record, XML, or report changes.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_fixture_sources_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

Corpus validation is intentionally deferred: before T4 adds the matching
manifest records, the corpus checker must reject these new Surgeist sources as
unmanifested. T4's completed derivation ran `just corpus-check` only after the
records and full corpus were frozen.

**Dependency:** C06-T2 settles every serializer and parser field used by the
sources.

**Intended commit:** `test(parity): add FRI-05 browser sources`.

### `C06-T4` Freeze Inputs And Derive The Final Corpus Once
**Files:** `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs` for focused helper tests,
`tests/layout/browser_parity/corpus.toml`,
`tests/layout/browser_parity/support.rs` and its transition tests,
`tests/layout/browser_parity.rs` for final XML, manifest, and report inventory,
generator-derived `tests/layout/browser_parity/xml/`, and
`tests/layout/browser_parity/xml/generation-reports/all.json`; plus
`src/block.rs` and focused `src/block_tests.rs` coverage for the confirmed
reserved-gutter range-basis omission.

**Outcome:** The exact computed-style helper fields, eleven active records,
frozen report counts, removed authored-overflow coupling, settled manifest, and
full ExistingPinned corpus are completed derivation evidence. The remaining
work preserves that corpus while correcting block layout to exclude reserved
gutters from physical range, then performs only read-only verification.

**Historical RED:** `fri05_c06_helper_`,
`fri05_c06_computed_overflow_transition_`,
`fri05_c06_manifest_freeze_`, and
`fri05_c06_computed_overflow_corpus_` tests failed because the
helper drops the named fields, authored cross-group pairs still couple inside
the parser, the manifest and derived corpus do not yet express the eleven
computed-style sources, and final inventory assertions still expect 5,280 XML.
The remaining smallest block front-door regression must fail
with a 15px horizontal range where both-edge gutters and non-overflowing content
require zero, matching the corpus failure.

**Completed pre-derivation evidence:** The helper captures all D-13 computed
fields without reading authored shorthand and its focused smoke tests pass.
`corpus.toml` has one matching active record per source,
`source_root = "surgeist"`,
`generator = "constrained-html"`, no scoped report, sole full report
`all.json`, and exact buckets 5,324 generated, 356 unsupported, and zero
expected-fail, quarantined, and failed-to-generate. The parser consumes the
computed pair directly through `ComputedOverflow::try_new`; authored coupling
and the 96-pair transition evidence are absent. The byte-exact manifest SHA-256
is `bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`;
every later read-only hash check requires that exact value.

**Completed pre-derivation commands; reusable only as read-only checks:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_helper_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_computed_overflow_transition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_manifest_freeze_
shasum -a 256 tests/layout/browser_parity/corpus.toml
test -x 'target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
```

**Historical regeneration command:** This command completed once for the initial
inputs and once more as the already-consumed replacement after two confirmed
source-input defects. It is derivation evidence only and must not be run again.
```sh
CARGO_NET_OFFLINE=true SURGEIST_LAYOUT_GENERATE_FILTER= SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

**Frozen derivation evidence:** The manifest hash remains exactly
`bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`. Final
inventory is 1,420 HTML and 5,324 XML, each owned source has exactly four
outputs, and only `all.json` exists. Its metadata names the frozen manifest,
current helper/base-style/launch-profile provenance, and the exact five frozen
buckets. The staged XML, manifest, and report inventory assertions are updated
from 5,280 to 5,324 while retaining HTML 1,420 and
unsupported 356. Every FRI-05 output parses and matches Surgeist layout;
comparator negative controls remain effective. No pre-existing source changes,
scoped report, stale XML, hand edit, expected failure, quarantine, or
unexplained XML body delta remains.

The frozen stable-both-edge browser expectations remain unchanged. Block layout
selects the existing scrollport range basis before canonical geometry in every
current block path that constructs such contributions, including retained-child
fallback geometry, without changing complete overflow, public API, flex/grid
behavior, or generator architecture. The focused block regression and original
44-output corpus comparison pass. This correction and all subsequent checks are
read-only with respect to generated artifacts; do not regenerate.

**Remaining read-only commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_helper_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_computed_overflow_transition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_manifest_freeze_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_computed_overflow_corpus_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c06_block_reserved_gutter_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
```

**Dependency:** C06-T3 is task-clean and every generation input is settled.

**Intended commit:** `test(parity): derive FRI-05 scroll corpus`.

## Cycle Acceptance
1. All four task ranges have genuine RED/GREEN or authoritative derivation
   evidence, clean independent task reviews, and coordinator-rerun acceptance.
2. Browser scroll expectations observe canonical physical range spans,
   including explicit zero, and wrong x/y or missing geometry cannot pass.
3. Helper, serializer, and parser preserve exactly D-13's computed finite forms
   without authored CSS parsing, production variants, or generator expansion.
4. The eleven exact active sources and manifest records produce 44 owned
   outputs; the full inventory is 1,420 HTML, 5,324 XML, sole `all.json`, 356
   unsupported, and zero in every failure class.
5. The legacy authored-overflow transition and 96-pair evidence are absent.
   Every generated non-default pair is computed and atomic.
6. Of the two historical full runs, only the consumed replacement used settled
   inputs and owns all XML pruning and report output. The frozen manifest hash
   remains byte-identical afterward, and all later checks are read-only.
7. Focused FRI-05 parity, corpus and Taffy validation, normal and
   generator-feature gates, provenance, diff, unsafe, and scope review pass.
8. The confirmed block omission has a block-front-door RED/GREEN test; the fix
   reuses the existing accumulator range basis and preserves complete overflow.
9. No other production, public API, docs, dependency, feature, MSRV, browser policy,
   base style, task runner, root, sibling, expected-failure, quarantine, or
   unrelated source change enters the range.

## Final Verification
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_comparator_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_parser_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_serializer_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_fixture_sources_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_helper_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_computed_overflow_transition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri05_c06_manifest_freeze_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_computed_overflow_corpus_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_c06_block_reserved_gutter_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri05_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
git ls-files -co --exclude-standard -- '*.rs'
! git ls-files -co --exclude-standard -z -- '*.rs' | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'
```

The owned-Rust manifest and scan cover every tracked and non-ignored Rust file.
Every textual match is classified and no executable match may remain. Final
range inspection proves exactly eleven new HTML sources and records, 44 owned
new XML outputs, one canonical report, no scoped report or hand edit, no added
lint suppression, no production delta beyond the named block correction, no
generator-architecture delta, and byte-identical frozen manifest content after
the settled replacement run.

## Handoff And Blockers
The completed, reviewed, published, and remotely read-back cycle hands C07
frozen read-only generator inputs and derived artifacts, active comparator
evidence, and no legacy overflow transition.

A genuine blocker is an unsupported/report bucket drift, an unexplained
pre-existing XML body change, a newly confirmed input bug, a named source that
requires later-format behavior, a parser form outside D-13, or any need to
expand generator architecture. The replacement allocation is consumed; any new
input defect requires a separately reviewed replan. Do not broaden grammar,
weaken an oracle, edit XML/report data, or rerun the generator.
