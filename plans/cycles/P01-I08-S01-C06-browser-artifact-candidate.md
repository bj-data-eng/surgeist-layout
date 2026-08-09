# P01-I08-S01-C06 Browser Artifact Candidate

Status: in_progress

Cycle ID: `P01/I08/S01/C06`

Owning repository: `surgeist-layout`

Cycle base: `7c761f618a9779b947864272126ee791d441d168`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`a636dd9c9b896e2986fd13ab303f8506fba7eec6b0ba909e542eee9dc39770e6`,
commit `09bab4edc2bbff4aad42469937a328d0724989c0`: corrected
`FRI-08.3.2`, `D-07`, and `FRI-08.8.1`; complete `FRI-08.13`; and the
artifact, verification, architecture, finding, handoff, and acceptance portions
of `FRI-08.14` through `FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`5c0cadc59d5aea8863c1038a7f131b2549f527fd0e74a18f2bac468974b20324`,
commit `dfb7768c68def72b19e08199b91ef65914e12a9a`, entry
`P01/I08/S01/C06`.

Bounded outcome: adopt the exact 40 outputs and sole schema-3 report from the
already-completed authoritative ExistingPinned run, correct the one
browser-exposed ordinary auto-fit gutter defect without another generation, and
publish a remotely verified behavior/artifact candidate whose 72 owned rows
pass.

## 1 Boundary And Entry Evidence

The remotely verified C05 candidate at the cycle base is immutable. Public API,
helper, adapter, generator production, ten new HTML sources and case records,
expected geometry, browser pin/profile, and all other corpus inputs are settled.

The first reviewed C06 plan authorized one unfiltered ExistingPinned invocation.
The worker proved RED on stale report lineage and 40 missing owned outputs,
changed only the derived full-report expectation from 5,736 to 5,776 plus
focused tests, verified Chrome for Testing `149.0.7827.115` at the manifest-owned
path, and invoked the exact authoritative command once. It exited zero. No
filtered or managed run, acquisition, retry, cleanup, manual artifact edit, or
commit occurred.

The preserved post-run state is:

- 1,448 HTML and 5,776 comment-free XML files;
- exactly 40 untracked XML additions at the ten new source/four-variant paths,
  no XML deletion, and no tracked base-XML body rewrite;
- one modified sole `all.json`, schema 3 with a null filter and buckets 5,776
  generated, 16 unsupported, three FRI-07 expected-fail, zero quarantined, and
  zero failed-to-generate;
- modified C06 tests and the sole manifest count, with helper, adapter,
  generator production, HTML, browser input, and expected geometry unchanged;
- seven focused generator tests green; and
- exact owned layout parity RED only at the first checked new row:
  `fri08_auto_fit_occupied_track_collapse__border_box_ltr/0` expects x `50`
  while production returns x `40`.

The generated XML is correct. Corrected `D-07` and CSS Grid Level 2 require the
two gutters adjacent to an interior collapsed track to coincide, leaving one
shared `10px` gap between the nearest active tracks. Current
`OrdinaryGridAxisGuttersOf::new` independently zeros both boundaries, erasing
that shared gap. C06 must retain the valid artifacts and correct production
browser-free; a second generation is neither necessary nor authorized.

The exact new sources are:

1. `grid/fri08_auto_placement_span_after_occupied.html`;
2. `grid/fri08_explicit_overlap_no_implicit_growth.html`;
3. `grid/fri08_fit_content_flex_composition.html`;
4. `grid/fri08_template_areas_explicit_tracks.html`;
5. `grid/fri08_auto_fit_occupied_track_collapse.html`;
6. `grid/fri08_stretch_minmax_auto.html`;
7. `grid/fri08_duplicate_line_name_token.html`;
8. `grid/fri08_grid_composition.html`;
9. `grid-lanes/fri08_nested_indefinite_subgrid.html`; and
10. `subgrid/fri08_standalone_intrinsic_composition.html`.

The exact adopted controls remain the eight sources in `FRI-08.13.1`. They are
acceptance evidence, not regenerated additions.

Out of scope: public API/model/errors/reexports, docs, helper, adapter,
serializer, generator production, HTML, expected geometry, case
identity/status/reason, browser pin/profile/arguments, base style, dependencies,
features, lockfile, MSRV, task runner, root/sibling work, Taffy import, WPT
mirror, new generator path or command, any further generation, acquisition,
second report/provenance authority, FRI-08 exception bucket, manual XML/report
edit, lanes auto-fit policy redesign, suppression, unsafe, and unrelated cleanup.

## 2 Impacts

Public API, dependencies, features, lockfile, MSRV, docs, root integration, and
browser policy: unchanged. Production behavior changes only for corrected
ordinary collapsed-gutter geometry under `D-07`; lanes policy remains separate.

Manifest: only `[generation_reports.full].generated = 5736` becomes `5776`
in the preserved pre-run state. All other semantic records remain unchanged.

Generated artifacts: exactly 40 new FRI-08 XML files join the 5,736 existing
comment-free XML files; no existing body changed. `all.json` is replaced in
place and remains the sole provenance authority, binding global manifest/helper/
base-style/browser/profile facts and every source/resource/XML hash.

## 3 Tasks

### 3.1 `P01/I08/S01/C06/T01` Adopt The Sole Derived Corpus Lineage

**Files/area:** the preserved changes in the one report-count field of
`tests/layout/browser_parity/corpus.toml`; C06 artifact/inventory/lineage tests
in `tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`; exactly 40 generator-derived
`tests/layout/browser_parity/xml/**/*.xml` additions; and the sole
`tests/layout/browser_parity/xml/generation-reports/all.json`.

**Outcome:** curate and commit the already-derived artifact state as complete,
current centralized provenance. This task does not invoke a browser or require
production parity green; it preserves the exact generated `x=50` row as T02's
authoritative behavioral RED.

**Authoritative refresh evidence:** the prior worker recorded the exact
pre-run RED, browser path/version, one invocation, zero exit, 40 additions, no
base rewrite/deletion, and final buckets. Do not manufacture a second RED or
rerun generation. Remove or defer only a test whose name/acceptance incorrectly
requires production parity within T01; preserve inventory, parser, provenance,
hash, source, exception, and no-comment tests.

**Acceptance:** the diff contains only the preserved artifact/test/count files.
Exactly 40 new XML paths form the ten-source/four-variant matrix; total XML is
5,776; all parse; no XML contains a comment; no base XML changed or was deleted.
`all.json` is the only report, schema 3 with null filter, and its exact buckets
are 5,776/16/3/0/0. No FRI-08 source enters an exception. All global,
source/resource/XML hashes and report/XML inventory identity validate against
the final manifest/helper/base-style/browser/profile. Case rows, inputs,
generator production, and expected geometry remain frozen. The known x
`50`/`40` production mismatch is recorded, not suppressed or converted into an
expected fail.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri08_c06_
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
cargo fmt --check
git diff --check
```

Run deterministic scripts proving exact additions/deletions/base-body identity,
5,776 comment-free XML, sole report and buckets, all centralized hashes, frozen
inputs/generator production, exact scope, no new `allow`/`expect`, and zero
unsafe matches in every owned Rust file. No command invokes generation.

**Dependency:** the preserved successful one-shot state and the corrected spec,
sequence, and plan reviews.

**Intended commit:** `test(parity): derive FRI-08 grid corpus`.

### 3.2 `P01/I08/S01/C06/T02` Preserve One Coincident Interior Auto-Fit Gutter

**Files/area:** ordinary boundary-gutter ownership in `src/grid/tracks.rs`;
`src/grid/alignment.rs`, `src/grid/child.rs`, `src/grid/placement.rs`,
`src/grid/subgrid.rs`, and `src/grid/mod.rs` only where necessary to consume or
transport the same carrier; focused tests in `src/grid_tests.rs`; and final
owned-row parity evidence in `tests/layout/browser_parity.rs`.
`src/grid/lanes.rs` is writable only at its existing gutter-reconstruction sites
to transport the already-selected explicit active-boundary mask; its placement,
collapse, sizing, and alignment policies remain frozen. Stop before widening
the production envelope.

**Outcome:** represent collapsed gutters as coincident boundaries: each
contiguous interior collapsed run retains exactly one base gap between its
nearest non-collapsed tracks, while leading, trailing, and all-collapsed runs
create no outer gap. Every ordinary sizing/alignment/span/absolute/subgrid/
baseline/overflow consumer uses that one carrier without reconstructing a
uniform gap or independently zeroing boundaries. The explicit active-boundary
mask survives used-geometry construction, alignment, sizing-gutter
reconstruction, inherited slicing, child consumption, and semantic reversal;
numeric zero gutter values are never used to guess the policy.

**RED evidence:** first retain the generated
`fri08_auto_fit_occupied_track_collapse__border_box_ltr` failure x `50` versus
x `40`. Add or correct direct `fri08_c06_collapsed_gutter_` tests before
production changes for one and multiple interior collapsed tracks, leading,
trailing, and all-collapsed runs, both axes, f32/f64, start/center/distributed
alignment, spans and absolute areas, and forward/reversed inherited subgrid
geometry. The current carrier fails the interior cases because it zeros both
adjacent boundaries. Existing C02 tests with the incorrect no-gap expectation
must become corrected RED evidence, not be deleted or weakened.
Add production-path GridLanes tests whose local `[active, collapsed,
collapsed, active]`-equivalent policy passes through child and lanes
reconstruction before forward/reversed inherited `SpaceBetween` consumption;
they fail if either reconstruction regenerates the ordinary coincident mask.

**Acceptance:** the generated row and all four variants match exact browser
geometry: occupied track three starts at x `50`, and the automatic span-two item
starts at x `100` with width `90`. Interior runs retain one shared gap regardless
of run length; outer/all-collapsed cases retain none. Active-gap totals,
intrinsic/flex/stretch free space, content distribution, line offsets, spans,
absolute areas, inherited/reversed carriers, baselines, and overflow agree.
Auto-fill, lanes auto-fit, named/negative lines, occupancy, public API, errors,
artifacts, and inputs remain unchanged. GridLanes retains zero boundaries
adjacent to collapsed tracks through every reconstruction and inherited
distributed-alignment path in f32/f64 and forward/reversed axes. All 72 owned
rows pass; FRI-09/F10 controls remain separately visible. No browser or
generator runs.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c06_collapsed_gutter_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri08_c02_auto_fit_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri08_c06_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri08_c0
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Also prove exact T02 scope, no artifact/input/hash delta from T01, no new
`allow`/`expect`, and zero unsafe matches in every owned Rust file.

**Dependency:** T01 is task-clean.

**Intended commit:** `fix(grid): preserve coincident auto-fit gutter`.

## 4 Completion

The canonical implementation, task-review, status, holistic-review, landing,
publication, readback, and cleanup lifecycle applies. C06 acceptance is:

1. one and only one unfiltered ExistingPinned invocation produced the preserved
   artifacts without acquisition, retry, cleanup, or manual edit;
2. the exact 40 new outputs complete the 18-source/72-row owned set and no
   FRI-08 exception bucket exists;
3. the corrected ordinary gutter carrier retains one coincident interior gap,
   no outer gap, and every ordinary consumer and all 72 owned rows pass;
4. 5,776 comment-free XML outputs and sole schema-3 `all.json` have complete,
   current, exact centralized provenance and inventory identity;
5. the 16 unrelated unsupported variants, three FRI-07 expected-fail records,
   later-owned FRI-09/F10 controls, settled inputs, public behavior outside the
   D-07 correction, dependencies/features/MSRV, and boundaries remain unchanged;
6. the clean immutable candidate is published to authority `main`, fetched, and
   read back by exact SHA before C07's fresh holistic sprawl assessment begins.

No final or review command invokes generation. No blocker is currently known.
