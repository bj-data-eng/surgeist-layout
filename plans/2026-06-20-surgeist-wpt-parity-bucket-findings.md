# Surgeist WPT Parity Bucket Findings

Date: 2026-06-20

## Context

This note records the largest common failure buckets from the current checked-in WPT browser parity corpus after the local Surgeist/Taffy parity and local subgrid parity work was made green.

Command used for the current WPT slice:

```sh
SURGEIST_PARITY_FILTER=xml/wpt cargo test -p surgeist --test layout_browser_parity -- runs_all_checked_in_browser_parity_xml --ignored --nocapture
```

At the time of this snapshot:

- WPT XML fixtures: 11,929
- WPT failures: 9,753
- WPT passes: 2,176
- WPT pass rate: about 18.24%

For comparison, before the three local subgrid commits, commit `0422675c1` had:

- WPT failures: 9,775
- WPT passes: 2,154
- WPT pass rate: about 18.06%

Measured separately with the same `SURGEIST_PARITY_FILTER=xml/wpt ...` command checked out at `0422675c1`.

Net movement from the subgrid work:

- +22 WPT fixtures passing
- about +0.18 percentage points

## Largest Actionable Bucket

The largest actionable feature-area bucket is WPT grid alignment:

```text
xml/wpt/grid/grid__alignment__*
```

Count:

- 2,027 failures / 2,267 fixtures

Failure kind split:

- x mismatch: 1,336
- y mismatch: 377
- width mismatch: 156
- unsupported parity fixture vertical-align: 72
- height mismatch: 40
- unsupported position: 24
- unsupported display: 22

Representative families:

- `grid/grid__alignment__grid-align-content-distribution*`: 432 / 504
- `grid/grid__alignment__grid-content-alignment-and-self-alignment-*`: 192 / 208
- `grid/grid__alignment__grid-gutters-and-alignment`: 94 / 109
- `grid/grid__alignment__grid-align`: 80 / 80
- `grid/grid__alignment__grid-align-justify-stretch`: 80 / 80

Representative fixture paths:

- `crates/surgeist/tests/layout_browser_parity/xml/wpt/grid/grid__alignment__grid-align-content-distribution__items-10.xml`
- `crates/surgeist/tests/layout_browser_parity/xml/wpt/grid/grid__alignment__grid-align-content-distribution-vertical-lr__items-10.xml`
- `crates/surgeist/tests/layout_browser_parity/xml/wpt/grid/grid__alignment__grid-content-alignment-and-self-alignment-001__items-1.xml`
- `crates/surgeist/tests/layout_browser_parity/xml/wpt/grid/grid__alignment__grid-align__items-1.xml`

Interpretation:

The failures are heavily placement-oriented, especially `x` and `y` mismatches, and concentrated under WPT `css/css-grid/alignment`. This points at Surgeist's grid box-alignment pass: content distribution, item/self alignment, stretch sizing, gutters, vertical writing modes, RTL, and cross-axis offset handling.

## Largest Exact Family

If selecting by a single exact WPT family rather than a broader feature area, the largest family is:

```text
flex/flex__alignment__multiline-align-self
```

Count:

- 596 failures / 720 fixtures

Failure kind split:

- x mismatch: 492
- y mismatch: 58
- width mismatch: 30
- height mismatch: 16

Interpretation:

This points at wrapped flex line placement and cross-axis alignment, especially `align-self`, `align-content`, RTL, vertical writing modes, and column-wrap behavior.

## Next Candidate Buckets

Flex cross-axis wrapping/alignment:

- 1,435 failures / 1,704 fixtures
- Includes `flex__alignment__*`, `flex__align-content-*`, `flex__multiline-*`, and `flex__inline-flexbox-wrap*`
- Points at wrapped flex line placement, `align-self`, `align-content`, RTL, vertical writing mode, and column-wrap behavior.

Alignment abspos:

- 1,068 failures / 1,080 fixtures
- Path shape: `xml/wpt/alignment/alignment__abspos__*`
- Mixed real x/y alignment failures and unsupported alignment values.
- Likely points at absolute positioning static-position and box-alignment support, but this bucket needs sub-clustering because it mixes unsupported parser/lowering cases with real geometry mismatches.

Grid item minimum sizing:

- 990 failures / 1,396 fixtures
- Mostly width/height mismatches under `grid__grid-items__grid-items-minimum-width*` and `grid__grid-items__grid-minimum-size-grid-items*`
- Points at grid item automatic minimum size and intrinsic contribution behavior.

## Method

Broad feature-area counts used prefix filters such as `xml/wpt/grid/grid__alignment__`.

Exact family counts used filters ending in `__items-` to avoid substring overmatch, for example `xml/wpt/flex/flex__alignment__multiline-align-self__items-`.

## Caveats

- The layout parity harness reports only the first failing assertion per XML fixture.
- Substring filters can overmatch; exact family counts should use suffixes such as `__items-` where possible.
- The numbers are a snapshot of the checked-in corpus at commit `00ddd4c64` plus the two preceding local subgrid commits.
- The largest broad bucket and largest exact family are different. Grid alignment is the largest feature area; flex multiline `align-self` is the largest single-family win.
