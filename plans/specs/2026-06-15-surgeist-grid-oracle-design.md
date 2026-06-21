# Surgeist Grid Oracle Design

## Purpose

Surgeist needs a durable oracle for CSS Grid before subgrid and grid-lanes work begins. The oracle must be thorough enough to validate complex grid behavior, but tight enough that it does not become a second hidden layout engine. Its job is to make the grid algorithm inspectable: given explicit algorithm inputs, it returns expected intermediate and final values that production layout can be compared against.

The oracle is not a browser fixture generator, a tree layout engine, a CSS parser, or a style resolver. It is a set of transparent spec-phase calculators and scenario checks.

## Core Principle

The production layout engine answers:

```text
Given this styled tree, what is every node's layout?
```

The oracle answers:

```text
Given this isolated grid algorithm state, what should this phase produce?
```

The oracle may compose phases for curated scenarios, but each phase must remain independently testable and must expose intermediate values. If a result disagrees with layout, the oracle should make it possible to identify which phase diverged.

## Non-Goals

- Do not parse HTML, CSS, XML, or stylesheets.
- Do not resolve style declarations, shorthands, inheritance, selectors, or cascade.
- Do not traverse retained UI trees.
- Do not measure text or children.
- Do not call production `compute_grid` or other production layout algorithms from oracle solvers.
- Do not hide defaults in broad builders.
- Do not model subgrid or grid-lanes until the base grid oracle is reliable.

The test harness may compare production layout output with oracle output. The oracle itself must remain independent.

## Module Shape

The current `crates/surgeist/tests/support/oracle/grid.rs` should split as it grows:

```text
tests/support/oracle/grid/
  mod.rs
  tracks.rs
  placement.rs
  contributions.rs
  alignment.rs
  scenario.rs
```

Each module owns one phase vocabulary. Cross-phase composition belongs only in `scenario.rs`.

### tracks

Owns explicit and implicit track construction, track sizing inputs, base sizes, growth limits, flexing, stretch growth for auto tracks, and final track sizes.

Inputs:
- available grid space: definite or indefinite
- gap
- track definitions
- resolved placement spans
- item contribution summaries

Outputs:
- initialized base sizes
- initialized growth limits
- base sizes after intrinsic minimums
- base sizes after content-based minimums
- base sizes after spanning items
- growth limits after spanning items
- base sizes after maximize-tracks free-space growth
- flex fraction
- base sizes after stretch-auto-track growth
- final track sizes

### placement

Owns line and span resolution, occupied-cell maps, auto-placement cursors, dense backfill, and implicit track count expansion.

Inputs:
- explicit column and row counts
- placement declarations as oracle-native values
- auto-flow mode
- item order

Outputs:
- resolved item grid areas
- occupied cell map
- implicit tracks before and after the explicit grid
- final cursor state

### contributions

Owns conversion from item facts into track-sizing contributions. This module does not measure children and does not resolve style. Tests feed it item facts explicitly. If a contribution depends on a fact that is not supplied, the oracle must return an unsupported-case error instead of deriving the fact from layout-like behavior.

Inputs:
- item grid area
- min-content contribution
- max-content contribution
- preferred size
- min size
- max size
- margins
- preferred aspect ratio when the test is about aspect-ratio contribution behavior
- replaced-element flag and transferred size facts when the test is about replaced elements
- overflow and automatic-minimum eligibility flags when the test is about automatic minimums
- box sizing facts needed by contribution arithmetic

Outputs:
- minimum contribution
- min-content contribution
- max-content contribution
- limited contribution after clamps

The contribution module may compute arithmetic from supplied facts. It may not compute intrinsic text sizes, replaced intrinsic dimensions, aspect-ratio transfer, automatic-minimum eligibility, or style-derived box facts unless those are the specific contribution rule under test and all required primitive inputs are explicit.

### alignment

Owns offset and gap distribution after track sizing. It does not stretch tracks. CSS Grid `stretch` for auto tracks belongs to the track sizing phase and must appear in the track sizing report as stretch-auto-track growth.

Inputs:
- final track sizes
- container size
- gap
- alignment mode
- overflow safety mode when modeled

Outputs:
- leading offset
- distributed gap
- final track offsets

### scenario

Owns curated end-to-end oracle cases that compose placement, contributions, tracks, and alignment. It may produce expected item rectangles, but only from explicit oracle inputs and phase outputs.

Scenario tests should stay small. They prove phase composition, not every matrix case.

## Inputs Must Be Explicit

Oracle inputs should be plain typed values. They should not borrow production layout types unless the type is a stable primitive with no algorithm behavior. Good oracle inputs look like:

```rust
GridTrack::Fixed(80.0)
GridTrack::Flex(1.0)
GridItemContribution {
    area,
    min_content: 40.0,
    max_content: 120.0,
    preferred: Auto,
}
```

Avoid inputs that imply hidden work:

```rust
NodeInput { ... }
StyleSheet { ... }
Tree { ... }
```

`OracleTree` remains useful for layout comparison tests because it stubs child measurement and records layout inputs. It is not the oracle's core model.

## Intermediate State Is Required

Every non-trivial solver must return a report, not only final numbers. A track sizing result should expose named phases:

```rust
TrackSizingReport {
    initialized: TrackState,
    after_intrinsic_minimums: TrackState,
    after_content_based_minimums: TrackState,
    after_spanning_items: TrackState,
    after_flexing: TrackState,
    final_tracks: Vec<SolvedTrack>,
}
```

Tests should assert intermediate values whenever a rule is being added. Final rectangle assertions alone are too weak for oracle work.

## Naming Rules

Broad names are reserved for broad behavior. If a helper models a narrow slice, the name must say so.

Good:

```rust
EqualShareIntrinsicTracks
UnboundedSpanningContribution
DefiniteLinePlacement
PositiveFreeSpaceAlignment
```

Too broad until fully implemented:

```rust
IntrinsicTracks
GridOracle
TrackSizer
Layout
```

Renames are expected as helpers mature. Compatibility shims are not allowed.

## Thoroughness Strategy

Coverage grows by phase matrices, then curated scenarios.

Placement matrix:
- definite start/end lines
- start line plus span
- span plus end line
- auto plus span
- row auto-flow
- column auto-flow
- row dense backfill
- column dense backfill
- implicit tracks after explicit grid
- implicit tracks before explicit grid for negative lines when supported

Track sizing matrix:
- fixed tracks
- percent tracks in definite and indefinite spaces
- `auto`
- `min-content`
- `max-content`
- `fit-content`
- `minmax`
- flex tracks
- indefinite available space
- definite available space
- max-content and min-content sizing modes
- spanning items across homogeneous tracks
- spanning items across mixed track categories
- growth limits
- flex fraction calculation

Contribution matrix:
- minimum contribution
- min-content contribution
- max-content contribution
- preferred definite size
- automatic minimums
- min/max clamps
- margins and percent margins where relevant
- replaced/aspect-ratio facts when grid uses them

Alignment matrix:
- start
- end
- center
- space-between
- space-around
- space-evenly
- safe overflow fallback
- row and column axes

Scenario matrix:
- one scenario per important phase composition
- one or two high-signal cases for existing tricky layout tests
- no giant fixture trees

## Comparison With Production Layout

Production layout tests may compare against oracle reports, but the comparison must be explicit. A layout test should say which oracle phase explains the expected number.

Good:

```rust
let report = TrackSizingOracle::new(...)
    .with_contributions(...)
    .solve();

assert_eq!(layout[child].size.width, report.final_tracks[1].size);
assert_eq!(report.after_spanning_items.tracks[1].base, 60.0);
```

Weak:

```rust
assert_eq!(layout, OracleGrid::layout(tree));
```

The second form hides too much and recreates the production API.

## Failure Mode

Oracle tests should fail loudly when an input is not modeled. Silent fallback is dangerous. Prefer:

```rust
UnsupportedOracleCase::IndefinitePercentTrack
```

over guessing a value. If a case is intentionally simplified, the type and test name must say so.

## Implementation Order

1. Split `oracle::grid` into phase modules without changing behavior.
2. Build the numeric base-grid placement oracle to completion because subgrid and grid-lanes depend on line/span semantics. This includes numeric lines, spans, auto placement, dense placement, implicit track expansion, `auto / auto` defaults, and row/column flow. Named lines, named areas, and writing-mode or direction remapping are deferred unless they become part of Surgeist's base grid surface before subgrid.
3. Build track sizing initialization and fixed/percent/flex resolution with phase reports.
4. Add maximize-tracks free-space growth and stretch-auto-track growth to the track sizing report.
5. Add contribution inputs and single-span intrinsic sizing.
6. Add spanning contribution distribution with growth limits and mixed track categories.
7. Add alignment phase reports.
8. Add curated scenario composition.
9. Replace hand-coded layout expectations with oracle comparisons where the oracle phase is complete enough.

Each step should be test-first. Every new rule starts with a failing oracle test. Production layout comparisons come after the oracle rule is independently green.

## Completion Criteria

The grid oracle is ready to support subgrid and grid-lanes when:

- placement covers the numeric base grid placement matrix listed above, including `auto / auto` defaults and explicit unsupported errors for named lines or named areas if those remain out of scope;
- track sizing reports initialization, intrinsic minimums, content-based minimums, spanning item growth, maximize-tracks free-space growth, flex fraction resolution, stretch-auto-track growth, and final track sizes;
- contribution inputs cover min-content, max-content, minimum contribution, preferred sizes, clamps, margins, automatic-minimum eligibility, and explicit aspect-ratio or replaced-element facts for tests that exercise those rules;
- spanning item distribution handles mixed track categories and growth limits;
- alignment covers positive free-space offset/gap distribution and safe overflow fallback without owning stretch track growth;
- scenario tests compose phases without calling production layout;
- selected existing grid layout tests assert against oracle phase outputs;
- unsupported cases fail explicitly instead of guessing;
- a clean-context review finds no overclaiming names or hidden production-layout dependency.

## Approval Gate

This document defines the oracle architecture. Implementation should wait for explicit review approval, then proceed through a separate step-by-step implementation plan.
