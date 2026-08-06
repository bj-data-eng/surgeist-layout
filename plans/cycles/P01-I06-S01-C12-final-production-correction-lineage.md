# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/specs/P01-I06-inline-formatting-floats-bfcs.md`, normalized SHA-256
`0f0a1f03eba4e79954efefcd6dd114547af27e891d2acc35ef54a398e542acad`,
commit `0344d20801c0b93d600bcf7f20f461c929b49ab9`: `FRI-06.3`,
`FRI-06.4 D-16` and `D-18`, subgrid and fixture portions of `FRI-06.7`,
module/test contracts in `FRI-06.9` and `.10`, browser/artifact contracts in
`FRI-06.11` and `.11.2`, and acceptance in `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`, normalized
SHA-256
`205e19677824de9768915405641ef268a1925b0031e08d08510c213d8492c308`,
commit `16991d7dd0b5dca7777518716a8a4f7ce4be2692`, entry
`P01/I06/S01/C12`.

## 1 Outcome

Preserve the task-clean T01-T07 results. Replace the superseded T08 inherited-
group publication model with D-18 direct descendant participation in one atomic
production task, then derive exactly one final full browser lineage after that
task is independently clean.

## 2 Boundary And Current Evidence

T01-T05 retain their exact reviewed ranges and outcomes:

| Task | Realized commits |
| --- | --- |
| T01 | `cff9204ed6119ed46609372529740addbcf0ce91`, `70d6e048c249b45fe5202b03063a9c76a926501e` |
| T02 | `37a776b3e7ffef463ef9a3e474aede5c6e0c1f76`, `48f7bfacdc64a66a26e4fce917727769e7d833ae` |
| T03 | `7f6a0657d1d2a8977ffa3fd7236f38aeb6283725`, `150a379a45ebd654aad0bb29ba3c778af5237663` |
| T04 | `515b712c85eb33e3b499755264332bb96c63ca09`, `90c7e8618d04e218874dc5b12b4113eda6b3dd4c` |
| T05 | `5ba51d3fcfe1a2ec46c47fb4e7ffbe0c169ba131`, `78876ebecd005a7a7f4e988a9d1c4c0a32dfcbd6`, `0bbfbc04b324826c95f9282e0409297b05acb4b9`, `40ccaeb2fbf012b017a58615a6f0f856e6918672` |

T07's complete ordered implementation evidence ends with:

- `323d73afa98ddc73e65fd9c1da223a5fbd85875e`;
- `d42a667494055e3ed4bba4b8502220a214b97ef4`;
- `89adbbc29ba3b2350c1fb64876a8a69520af8e07`;
- `2ed9382d0f3b2f47c5701aa5290e26567b44ac3a`; and
- `17fffd9374647633eb0a7dcd1ecbf56b0ed8a37c`.

Those changes measure BR baselines in Chrome, reject synthetic line-height
fallback, and make touching comparator intervals `Same`. T07 is task-clean and
is not reopened.

T08's historical spans are
`9ff1b91dabd7d53b32ee0942a7e6962515a80b79` and
`5f7f72c45090d9c230f7a2957bffadd5904625b4`. Their D-17 publication/inverse
premise is superseded. Three bounded corrections could satisfy either nested
coordinates or intrinsic and round-trip controls, but not all together; every
attempt was fully reverted and no diagnostic residue remains.

At source state `ed246a31d8af7957e5592c27e111345e86479fe6`, public
geometry is:

| Control | Browser | Current |
| --- | ---: | ---: |
| auto-row root height | 411 | 459 |
| inline-column LTR/RTL x | 470 / 527 | 415 / 570 |
| nested-block descendant/sibling y | 62 / 110 | 57 / 125 |
| vertical-auto x | 196 | 202 |
| vertical-nested x | 153 | 168 |

The checked-in report from T09 commit
`0a355604d0862a8f07811d323acfdece912921cd` remains diagnostic: 5,712
generated, 16 unsupported, 144 of 388 activation rows passing, and 244 failing.
Its report SHA-256 is
`f46d8d8b50c722037127fdca79679649bd5cfd6db16fb24c0d69a7e5a082147a`.
Inline-column and vertical-nested XML still carry the pre-T07 touching-interval
observation, so T08 uses public layout evidence for those families until T09.

## 3 Known Chrome Measurement Failures

None. Chrome remains authoritative; no synthetic substitute or expected-fail
entry is authorized.

## 4 Impacts

- **Public API and compatibility:** unchanged; every D-18 carrier is private.
- **Production:** T08 owns `src/grid/tracks.rs`, `src/grid/subgrid.rs`,
  `src/grid/child.rs`, and focused `src/grid_tests.rs` as one atomic correction.
- **Fixture/helper/parser/comparator:** unchanged in T08.
- **Generated artifacts:** T09 alone replaces the 5,712 XML files and
  `xml/generation-reports/all.json`; generator Rust changes are evidence constants
  only.
- **Dependencies, features, docs, examples, MSRV, root:** unchanged.
- **Safety:** no `unsafe`, lint suppression, parser layer, generator architecture,
  or later-owned behavior is permitted.

## 5 Tasks

### 5.1 `P01/I06/S01/C12/T08` Flatten And Consume Ancestor Baseline Groups

**Files/area:** `src/grid/tracks.rs`, `src/grid/subgrid.rs`,
`src/grid/child.rs`, and `src/grid_tests.rs`.

**Outcome:** Extend the existing axis-parametric subgrid traversal to emit
separate `FlattenedScalarContribution` and `AncestorBaselineMember` values.
Suppress a fully inherited root only from the ordinary scalar pass, retain every
descendant exactly once, and reduce row and column members into one immutable
ancestor first/last group before intrinsic baseline shims. Derive a non-
publishable `ChildBaselineEnvelopeView` from that immutable group, map it into
each child's local track order and logical direction, and align each affected
item once. Remove the fully inherited child-to-parent publication inverse and
fixed-point premise while preserving ordinary non-inherited baseline publication
and containing-grid `FlowAxes` projection.

**RED:** Before production edits, add public-layout regressions named with the
`fri06_c12_t08_` prefix. The auto-row control expects 411 and currently reports
459. A focused traversal control expects ancestor last-member distance 40 while
the direct item contributes 25; exercise the same reduction for row and column
axes. One nested computation expects descendant/sibling y `(62, 110)` and
currently yields `(57, 125)`. Shared inline-column placement expects LTR/RTL x
`470/527` and currently yields `415/570`. Vertical auto expects the retained 18px
envelope, area 381, child width 371, and x 196; current x is 202. Vertical nesting
expects x 153; current x is 168. Repeated view derivation must produce identical
groups and geometry. Reconstruct the observable RED at the current source state
without generation.

**Acceptance:** Auto-row scalar/member/group separation reports 411 and omitting
the inherited root scalar never omits descendants. Row and column reductions use
the same `GridAxisKind` path. First/last selection accumulates positive, zero,
negative, reversed, margin/border/padding, and half-gutter edge adjustments once.
Nested block, inline columns, vertical auto, and vertical nesting match their
exact public values. Child views never enter scalar sizing or ancestor reduction,
and repeated placement is idempotent. The former inherited-publication round-trip
test is removed because it tests superseded D-17; public geometry and view-mapping
controls replace it. Ordinary grid and non-inherited subgrid controls remain
unchanged. No generator command runs.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout fri06_c12_t08_
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid_baseline_auto_rows cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid_baseline_nested_block cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid_baseline_vertical_auto_rows cargo test --locked --offline -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored --nocapture
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** T07 is task-clean. Review the complete reconciled T08
implementation lineage: both historical spans and the fresh correction span.
This full review replaces every task verdict invalidated by D-18.

**Intended commit:** `fix(layout): align subgrids from ancestor baseline groups`.

### 5.2 `P01/I06/S01/C12/T09` Replace Final Browser Lineage

**Files/area:** `tests/bin/surgeist-layout-generate/generator.rs` evidence
constants, `tests/layout/browser_parity.rs` evidence constants, all 5,712
manifest-owned XML files, and `xml/generation-reports/all.json`. No helper, HTML,
parser, serializer, comparator, production, API, manifest, dependency, feature,
or generator logic.

**RED:** At the committed T08 head, the stale evidence constants and checked-in
pre-T07 XML fail their focused freeze and activation tests. Reproduce those
failures once without modifying the worktree. Do not edit evidence constants
before generation.

**Outcome:** With a clean worktree, no filter or browser override except the
explicit existing pin, and no generator process, run full unfiltered
`generate-existing` exactly once. Run no scoped generation in T09. Update only
the resulting XML/report and their exact evidence constants. If the run exposes
an unmet input or production assumption, preserve its evidence and return to the
affected task before any replacement run; never rerun unchanged inputs.

**Acceptance:** Report is `filter: null`, 5,712 generated, the exact 16
missing-root unsupported rows, and empty expected-fail, quarantine, generation-
failure, and failed-to-generate buckets. All 388 activation rows pass with no
substitute. The other 5,324 XML bodies preserve semantics. Record exact helper,
report, complete XML, activation, preserved-body, and inventory hashes. No scoped
report, process, or temporary resource remains.

**Commands:**
```sh
git status --porcelain
pgrep -f '[s]urgeist-layout-generate'
"target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" --version
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
```

The first status is empty, the process probe finds no process, and Chrome reports
`149.0.7827.115`. The Cargo run is the sole generation. All later commands are
read-only verification.

**Dependency:** T08 is task-clean. Append the replacement span after
T09 diagnostic commit `0a355604d0862a8f07811d323acfdece912921cd` and review
the complete ordered T09 lineage.

**Intended commit:** `test(parity): replace final FRI-06 lineage`.

## 6 Cycle Completion

After both tasks are clean, change only `Status` to `complete` in a separate
commit. At that exact head run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
git diff --check 8ffb4bc551a24d2283ad54436870ab3f5e66a473..HEAD
git status --short
```

The unsafe scan returns no match and final status is clean. Record exact task
ranges and artifact hashes, then follow the canonical holistic review,
publication, remote readback, cleanup, and C13 handoff contracts. Blocker: none.
