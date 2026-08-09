# P01-I08-S01-C06 Browser Artifact Candidate

Status: reviewed

Cycle ID: `P01/I08/S01/C06`

Owning repository: `surgeist-layout`

Cycle base: `7c761f618a9779b947864272126ee791d441d168`

Reviewed specification:
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, normalized
semantic-content SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
commit `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`: complete `FRI-08.13` and
the artifact, verification, architecture, finding, handoff, and acceptance
portions of `FRI-08.14` through `FRI-08.19`.

Reviewed implementation sequence:
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
normalized semantic-content SHA-256
`62e6b43402a038e7df5bc22e5c28ee40b7e7ae1a1ac6fc28224c12626cc9ca7c`,
commit `75801ea77e37af28c0dda32a28fd1647123e1293`, entry
`P01/I08/S01/C06`.

Bounded outcome: perform the sole unfiltered full ExistingPinned derivation of
the settled C05 browser inputs, adopt exactly 40 new XML outputs and the updated
sole schema-3 `all.json`, and publish a remotely verified behavior/artifact
candidate. The run is authoritative; every post-run check is read-only.

## 1 Boundary And Entry Evidence

The remotely verified C05 candidate at the cycle base is immutable. Production
behavior, public API state, helper, adapter, generator production, ten new HTML
sources and case records, expected geometry, browser pin/profile, and all other
corpus inputs are settled.

At the cycle base:

- the worktree is clean on `main`, local and remote `main` are the cycle base;
- the corpus contains 1,448 HTML and 5,736 comment-free XML files;
- the exact ten new sources exist as active Surgeist constrained-HTML records,
  but their 40 standard four-variant XML paths are absent;
- the eight adopted control sources already own 32 XML paths, making the exact
  FRI-08 inventory 18 sources and 72 unique rows;
- `all.json` reports 5,736 generated, 16 unsupported, three FRI-07
  expected-fail source records, zero quarantined, and zero failed-to-generate;
- SHA-256 values are `corpus.toml`
  `f104b274bb561ef601348a101159cd839f8e0704d633697eea0d1b56a6a4beb6`,
  helper
  `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`,
  and `all.json`
  `5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`;
- `just corpus-check` has the reviewed C06-entry failure because `all.json`
  still binds the prior helper/manifest lineage; other C05 gates are green;
- the manifest pins Chrome for Testing `149.0.7827.115` and the exact installed,
  executable path exists at
  `target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`;
  no acquisition is authorized; and
- `corpus.toml` necessarily changes its derived full-report expectation from
  5,736 to 5,776 before generation. This is C06 artifact accounting, not a case,
  source, helper, adapter, browser, or geometry input change. The generator
  hashes the manifest into `all.json`, so changing this field after derivation
  would create stale lineage and editing the report by hand is forbidden.

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

Out of scope: production, public API, model, errors, reexports, docs, helper,
adapter, serializer, generator production, HTML, expected geometry, case
identity/status/reason, browser pin/profile, launch arguments, base style,
dependencies, features, lockfile, MSRV, task runner, root/sibling work, Taffy
import, WPT mirror, new generator path or command, filtered/scoped generation,
managed-browser generation, acquisition, second report/provenance authority,
new expected-fail/quarantine/failure record, manual XML/report edits, a second
generation invocation, suppression, unsafe, and unrelated cleanup.

## 2 Impacts

Public API, production behavior, dependencies, features, lockfile, MSRV, docs,
root integration, and browser policy: unchanged.

Manifest: only `[generation_reports.full].generated = 5736` becomes `5776`
before derivation. All other bytes and semantic records remain unchanged.

Generated artifacts: exactly 40 new FRI-08 XML files join the 5,736 existing
comment-free XML files. The authoritative generator may rewrite existing XML
only when settled current inputs require it. `all.json` is replaced in place and
remains the sole provenance authority, binding global manifest/helper/base-style/
browser/profile facts and every source/resource/XML hash.

## 3 Task

### 3.1 `P01/I08/S01/C06/T01` Derive And Validate The Full Corpus Once

**Files/area:** the one report-count field in
`tests/layout/browser_parity/corpus.toml`; C06-only inventory, lineage, and
parity tests in `tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`; generator-derived
`tests/layout/browser_parity/xml/**/*.xml`; and the sole
`tests/layout/browser_parity/xml/generation-reports/all.json`.

**Outcome:** prove the final accounting and immutable inputs, invoke the
authoritative ExistingPinned generator exactly once, then adopt and validate its
complete 5,776-output lineage without a manual artifact edit.

**RED evidence before invocation:** add focused `fri08_c06_` tests that freeze
the exact 18-source/72-row inventory, require the manifest full-report count
5,776, require exactly 40 absent new outputs at the task base, require the final
report metadata/counts/hash coverage and final exact parity set, and reject any
FRI-08 exception bucket. Before derivation, artifact assertions fail because the
40 new XML paths and current report lineage do not exist. Do not substitute
hand-written XML or report data for this RED.

**Pre-derivation acceptance:** the diff is limited to the one manifest count and
focused tests; all ten case records remain exact active Surgeist records; the
ten new HTML files, helper, adapter/parser, generator production prefix, base
style, browser manifest/profile, expected geometry, existing 5,736 XML bodies,
and existing report are byte-frozen against C05. The prospective output matrix
is exactly 40 missing paths with no collision. The report directory contains
only `all.json`. No generation lease is active; generation-filter, alternate
root, cache, and version overrides are absent or explicitly cleared. The exact
browser path is executable and its version probe reports
`Google Chrome for Testing 149.0.7827.115`. The version probe and generation
must use the already-present executable and may not fetch or install software.

**Sole derivation command:** after all pre-derivation acceptance evidence, run
this exact command once:

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_LAYOUT_GENERATE_FILTER= SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

If this invocation exits nonzero or yields an invalid corpus, stop and report
the exact state. Do not correct inputs, manually edit artifacts, or invoke any
generation command again. A replacement would require a reviewed plan amendment
that accounts for the failed invocation and grants exact replacement authority.

**Final acceptance:** exactly 40 added XML files exist at the ten named
source/four-standard-variant paths; total XML is 5,776; there are no XML
deletions or embedded provenance comments. `all.json` is the only JSON report
and has schema 3, a null filter, 5,776 generated entries, the same 16 unrelated
unsupported rows, the same three FRI-07 expected-fail records, zero quarantined,
and zero failed-to-generate. No generated entry for an FRI-08 source appears in
any exception bucket. Global metadata matches the final manifest, helper, base
style, browser, and launch profile. Source, linked-resource, and XML hashes are
complete, unique, repository-relative, and current; report outputs exactly equal
the XML inventory.

All 72 owned rows pass focused parity. The later-owned FRI-09 baseline and
FRI-10 positioned-layout mismatch controls remain separately visible and
unchanged. Existing XML changes, if any, are enumerated and justified solely by
settled input hashes; otherwise all 5,736 base XML files are byte-identical.
Record final manifest, helper, report, all-XML, new-40-XML, owned-72 inventory,
browser, bucket, and negative-control evidence. After generation every command
is read-only.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri08_c06_
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

Also run deterministic read-only scripts that prove the exact 40-path addition,
zero deletion, 5,776 total XML, no embedded provenance, only `all.json`, exact
report/XML identity, unchanged non-report manifest bytes except the count,
unchanged helper/base-style/browser/profile/generator-production hashes, exact
bucket projections, exact 18-source/72-row pass set, unchanged FRI-09/F10
controls, no out-of-scope file, no new `allow`/`expect`, and zero unsafe matches
across every tracked and non-ignored owned Rust file.

**Dependency:** remotely verified C05 candidate only.

**Intended commit:** `test(parity): derive FRI-08 grid corpus`.

## 4 Completion

The canonical implementation, task-review, status, holistic-review, landing,
publication, readback, and cleanup lifecycle applies. C06 acceptance is:

1. the reviewed pre-run state differs from C05 only by the derived report-count
   expectation and C06 tests;
2. one and only one unfiltered ExistingPinned invocation through the existing
   authoritative command produces the candidate without acquisition;
3. the exact 40 new outputs complete the 18-source/72-row owned set, all pass,
   and no FRI-08 exception bucket exists;
4. 5,776 comment-free XML outputs and sole schema-3 `all.json` have complete,
   current, exact centralized provenance and inventory identity;
5. the 16 unrelated unsupported variants, three FRI-07 expected-fail records,
   later-owned FRI-09/F10 controls, settled inputs, production/public behavior,
   dependencies/features/MSRV, and repository boundaries remain unchanged; and
6. the clean immutable candidate is published to authority `main`, fetched, and
   read back by exact SHA before C07's fresh holistic sprawl assessment begins.

No final or review command invokes generation. A genuine failure follows the
explicit stop boundary above; no blocker is currently known.
