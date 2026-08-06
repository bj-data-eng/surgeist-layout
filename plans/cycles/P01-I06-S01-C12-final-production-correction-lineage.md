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

Preserve the task-clean T01-T06 results and the D-18-clean T08 result. Reopen T07
only for the confirmed vertical-lr BR-probe sign bug exposed by the first failed
full lineage attempt, then derive one final full browser lineage after the helper
input changes and the complete T07 range is independently clean again.

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
fallback, and make touching comparator intervals `Same`. T07 was task-clean, but
the first T09 full run exposed one retained fake/real sign mismatch and reopens
only that helper boundary.

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
Some activation XML still carry the pre-T07 touching-interval observation. In
particular, vertical-auto reaches every geometry comparison after T08 but its 48
rows still report expected `Later` versus observed `Same`; T08 uses public layout
evidence for stale families until T09 replaces the settled lineage.

At the final T08 head, the production correction resolves 96 of those diagnostic
activation failures without changing checked-in XML: all 48 auto-row and all 48
nested-block rows. The exact pre-generation activation state is therefore 240 of
388 rows passing and 148 failing, with no new failures: 144 stale subgrid
neighbor-line comparisons and four stale unequal-line block heights.

At T08 head `8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`, the first
unfiltered T09 attempt ran once and stopped without a commit. Its report generated
5,708 rows, retained the exact 16 missing-root unsupported rows, and recorded one
failed input: `block_br_vertical_lr_inline_block_metrics.html`, where the helper
rejected a negative signed distance. The failed report SHA-256 is
`2efb49434033b86bf53465914fcf507e49b773392469903f7824846e63f368bf`;
the complete XML SHA-256 is
`30c93c9d8c43b463da2c85da8e46f300d53a54d2c663d6007067dc4a97eb2a81`.
A single isolated scoped diagnostic then measured pinned Chrome markers at
top/baseline/bottom x `30/15/0`: all finite, with the baseline between both line
edges. The helper selected `baseline - top = -15`; its unit fake encoded the
opposite orientation. No diagnostic residue remains.

## 3 Known Chrome Measurement Failures

None. Chrome remains authoritative; the failed run confirmed a helper sign bug,
not a browser failure. No synthetic substitute or expected-fail entry is
authorized.

## 4 Impacts

- **Public API and compatibility:** unchanged; every D-18 carrier is private.
- **Browser helper:** reopened T07 owns only the non-horizontal BR distance in
  `test_helper.js` and its generator unit regression. It changes no fixture or
  generator architecture.
- **Production:** T08 owns `src/grid/tracks.rs`, `src/grid/subgrid.rs`,
  `src/grid/child.rs`, and focused `src/grid_tests.rs` as one atomic correction.
- **Fixture/parser/comparator:** unchanged.
- **Generated artifacts:** T09 alone replaces the 5,712 XML files and
  `xml/generation-reports/all.json`; T07 changes focused generator tests only,
  while T09 changes generator Rust evidence constants only.
- **Dependencies, features, docs, examples, MSRV, root:** unchanged.
- **Safety:** no `unsafe`, lint suppression, parser layer, generator architecture,
  or later-owned behavior is permitted.

## 5 Preserved Result And Tasks

### 5.1 `P01/I06/S01/C12/T07` Correct Vertical-LR BR Baseline Distance

**Files/area:**
`tests/layout/browser_parity/scripts/gentest/test_helper.js` and focused unit
tests in `tests/bin/surgeist-layout-generate/generator.rs`. Before edits, verify
the preserved failed-run inventory and restore only its 5,708 XML changes and
`xml/generation-reports/all.json` to the exact T08 head; no failed generated
output is retained or committed.

**RED:** In the bundled helper harness, reproduce pinned Chrome's vertical-lr
marker order: line-over x 30, baseline x 15, and line-under x 0 at line height 30.
The current writing-mode sign branch selects `15 - 30 = -15` and rejects the
measurement. The test fails for that exact reason without a browser or generator
run.

**Outcome:** Treat a non-horizontal BR baseline as the finite non-negative
physical x-distance magnitude between the line-over and baseline markers;
writing mode and direction do not assign a sign to that scalar. Preserve the
horizontal top-to-baseline calculation, finite/non-negative line-height checks,
zero-height fast path, line-height clamp, complete computed font context, and
probe cleanup. Make the unit fake reproduce Chrome's vertical-lr orientation.

**Acceptance:** The new vertical-lr regression reports baseline `15px` and line
height `30px`; existing horizontal, vertical-rl, sideways, zero, clamped,
nonfinite, normal-line-height, and cleanup controls remain green. The helper
change leaves checked-in evidence constants and XML stale for T09. No generation
command runs.

**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t07_br_inline_metrics_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --lib
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

**Dependency:** Append the correction span to T07's five historical commits and
review the complete ordered T07 lineage. The failed full run and scoped probe are
diagnostic evidence, not verification or task spans.

**Intended commit:** `fix(parity): measure vertical-lr BR baseline magnitude`.

### 5.2 Preserved T08 Result

T08 is clean at `8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`; it is not an
executable task after this amendment. Its reviewed ordered ranges are:

- `89adbbc29ba3b2350c1fb64876a8a69520af8e07..9ff1b91dabd7d53b32ee0942a7e6962515a80b79`;
- `9ff1b91dabd7d53b32ee0942a7e6962515a80b79..5f7f72c45090d9c230f7a2957bffadd5904625b4`;
- `a64b3272c675e52fecec61fa9617c9e972e2b514..e36830143235e28625ac010489d8c7aa998d714f`;
- `e36830143235e28625ac010489d8c7aa998d714f..f2a3e0485adbc63521276f688ddf7e1f71fa448e`;
- `f2a3e0485adbc63521276f688ddf7e1f71fa448e..e367a493f4d6b574a1d1a53b31314528a5e5a213`; and
- `e367a493f4d6b574a1d1a53b31314528a5e5a213..8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`.

Its clean verdict covers D-18 direct descendant participation, root-empty
grouping, private child views, removal of the D-17 inverse, and the exact public
auto, nested, inline, and vertical geometry. T09 alone replaces its stale
interval-relation artifacts.

### 5.3 `P01/I06/S01/C12/T09` Replace Final Browser Lineage

**Files/area:** `tests/bin/surgeist-layout-generate/generator.rs` evidence
constants, `tests/layout/browser_parity.rs` evidence constants, all 5,712
manifest-owned XML files, and `xml/generation-reports/all.json`. No helper, HTML,
parser, serializer, comparator, production, API, manifest, dependency, feature,
or generator logic.

**RED:** At the committed T07 correction head, the stale evidence constants and
checked-in pre-T07 XML fail their focused freeze and activation tests. Reproduce
those failures once without modifying the worktree. The activation-only test
enumerates exactly 388 rows and must report exactly 148 failing paths: 144
subgrid neighbor-line mismatches and four unequal-line block-height mismatches,
so 240 rows pass. Separately, pre-generation `just parity-all` must fail with the
settled 520-fixture whole-corpus state. The broad command is not activation-count
evidence. Do not edit evidence constants before generation.

**Outcome:** With a clean worktree, no filter or browser override except the
explicit existing pin, and no generator process, run full unfiltered
`generate-existing` exactly once after the T07 helper input changes. The earlier
failed full attempt is retained only as evidence and is not a replacement
lineage. Run no scoped generation in T09. Update only the resulting XML/report
and their exact evidence constants. If the run exposes another unmet input or
production assumption, preserve its evidence and return to the affected task
before any replacement run; never rerun unchanged inputs.

**Acceptance:** Report is `filter: null`, 5,712 generated, the exact 16
missing-root unsupported rows, and empty expected-fail, quarantine, generation-
failure, and failed-to-generate buckets. All 388 activation rows pass with no
substitute. The other 5,324 XML bodies preserve semantics. Record exact helper,
report, complete XML, activation, preserved-body, and inventory hashes. No scoped
report, process, or temporary resource remains.

**Commands:**
```sh
test -z "$(git status --porcelain)"
! pgrep -f '[s]urgeist-layout-generate'
"target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" --version
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_final_activation_union_browser_passes_without_substitutes -- --nocapture
CARGO_NET_OFFLINE=true just parity-all
env -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH="target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
CARGO_NET_OFFLINE=true just parity-all
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
```

The first status is empty, the process probe finds no process, and Chrome reports
`149.0.7827.115`. The Cargo run is the sole generation. All later commands are
read-only verification.

**Dependency:** Reopened T07 is task-clean and T08 remains preserved-clean at
`8740d5ef3432c80f49eb7086e65bbd9c012cb1aa`. Append the replacement span after
T09 diagnostic commit `0a355604d0862a8f07811d323acfdece912921cd` and review
the complete ordered T09 lineage.

**Intended commit:** `test(parity): replace final FRI-06 lineage`.

## 6 Cycle Completion

After both executable tasks are clean, change only `Status` to `complete` in a
separate commit. At that exact head run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just parity-all
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
git diff --check 8ffb4bc551a24d2283ad54436870ab3f5e66a473..HEAD
test -z "$(git status --short)"
```

The unsafe scan returns no match and final status is clean. Record exact task
ranges and artifact hashes, then follow the canonical holistic review,
publication, remote readback, cleanup, and C13 handoff contracts. Blocker: none.
