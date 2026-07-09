# CSS Scroll Geometry Implementation Sequence

Date: 2026-07-09

## Purpose

This sequence splits the CSS scroll geometry support matrix into
implementation-sized phases for `surgeist-layout`. Each numbered item should
produce its own reviewed implementation plan before code changes begin.

Inputs:

- `plans/2026-07-09-css-scroll-math-planning-directive.md`
- `plans/2026-07-09-surgeist-layout-css-scroll-support-matrix.md`
- `/Users/codex/Development/surgeist/guidance/surgeist-rust-modeling-guide.md`
- `/Users/codex/Development/surgeist/plans/2026-07-04-css-integration-support-inventory.md`

Layout remains a geometry engine. These phases must not add CSS parsing,
cascade, root lowering, live scroll state, platform input routing, smooth
scrolling, animation sampling, or scrollbar painting.

## Sequence Rules

- Create and review one implementation plan per sequence item.
- Implement one reviewed plan at a time through the `AGENTS.md` workflow.
- Keep scroll data scalar-generic over `S: LayoutScalar`.
- Prefer typed scroll geometry/input/output modules over expanding broad
  transport structs without invariants.
- Surface unsupported style values through typed layout diagnostics or require
  root rejection before they reach layout.
- Treat `scroll-padding`, `scroll-margin`, and scroll snap as deferred until a
  dedicated sequence item brings them into layout with typed inputs.
- Do not store live scroll offsets in layout. Runtime owns live offsets and
  applies layout-provided range/clamp facts.

## Phase 1: Typed Scroll Geometry Core

Define the public and internal typed geometry contract that later algorithms
will emit and consume.

Scope:

- scalar-generic scroll rect/range/clamp types;
- physical-axis geometry plus explicit writing-mode/direction metadata or
  helpers for logical access;
- scroll container classification per axis;
- a mixed-axis overflow policy that chooses exactly one boundary: either
  represent CSS visible-to-auto coupling in typed layout input, or require root
  to pre-resolve/reject coupled values before they reach layout;
- scrollport, overflow clip rect, scrollable overflow rect, maximum offset, and
  scrollbar gutter rect output facts;
- semantic unsupported-value diagnostics needed before layout can reject
  scroll-facing inputs intentionally.

Out of scope:

- algorithm integration beyond unit/contract tests;
- `overflow: auto` behavior;
- stable or both-edge gutter expansion;
- nested clipping propagation;
- scroll snap and target geometry.

Completion evidence:

- new types are reexported intentionally from `lib.rs` if public;
- the root/runtime/render-facing front door is named explicitly, even if later
  phases add more fields to the output;
- constructors enforce non-negative ranges and finite dimensions where
  applicable;
- tests cover `f32` and `f64` geometry, clamp behavior, hidden-versus-clip
  range exposure policy, and unsupported diagnostics.

## Phase 2: Central Scrollport And Gutter Math

Centralize the existing duplicated scrollbar reservation and scrollport
calculation logic before adding broad scroll output to algorithms.

Scope:

- shared helper/module for border-box, padding-box, scrollport, and gutter rect
  derivation;
- preservation of existing `overflow: scroll` gutter behavior in block, flex,
  grid, and leaf-style paths;
- LTR/RTL gutter placement tests;
- horizontal-tb, vertical-rl, and vertical-lr mapping tests where current
  layout inputs already provide writing mode and direction;
- output geometry for supported scrollbar reservation, including current
  `overflow: scroll` gutters.

Out of scope:

- changing CSS semantics for `overflow: clip` margin collapse;
- `scrollbar-gutter: stable` or `both-edges`;
- dynamic `overflow: auto` gutter decisions.

Completion evidence:

- current size behavior remains green;
- duplicated per-algorithm gutter math is reduced to the shared helper where
  practical;
- tests prove content-box and border-box sizing still interact correctly with
  padding, border, and gutter reservation.

## Phase 3: Root And Block Scroll Geometry Output

Emit usable scroll geometry facts for root and block layout before extending
the remaining algorithms.

Scope:

- root viewport/document scrollport facts;
- block scroll container facts for `visible`, `hidden`, `clip`, and `scroll`;
- scrollable overflow accumulation for ordinary in-flow block and inline
  descendants;
- explicit contributor rules for margin box, border box, padding box, and
  inline line contributions;
- hidden exposes programmatic range facts; clip exposes clipping without scroll
  range;
- `overflow-clip-margin` is explicitly rejected or diagnosed until a dedicated
  typed input and clip-edge geometry model exists;
- initial maximum offset and clamp facts.

Out of scope:

- flex and grid container output;
- absolute/out-of-flow contribution unless required to keep existing block
  tests coherent;
- nested inherited clipping across multiple scroll containers;
- `overflow: auto`.

Completion evidence:

- root and block tests cover scrollport construction, no-overflow ranges,
  vertical overflow, horizontal overflow, both-axis overflow, hidden versus
  clip, visible descendant overflow, and clamp math;
- output remains layout geometry only and carries no live scroll position.

## Phase 4: Flex And Grid Scroll Geometry Output

Extend the root/block scroll contract to flex and grid without forking the
model.

Scope:

- flex container scrollable overflow and range output;
- flex item gutter/scrollport facts where item layout already computes them;
- grid container scrollable overflow and range output;
- grid item/subgrid interaction tests that preserve existing intrinsic sizing
  behavior;
- reuse of Phase 1 and Phase 2 types/helpers.

Out of scope:

- new grid placement behavior;
- broad subgrid semantic changes unrelated to scroll overflow;
- `overflow: auto` and stable gutters unless earlier phases explicitly
  completed their typed inputs.

Completion evidence:

- flex and grid tests cover scrollport, scrollable overflow, gutter output, and
  range facts;
- existing green-baseline XML corpus remains green where applicable;
- no duplicate scroll model appears inside flex or grid internals.

## Phase 5: Nested Clipping And Out-Of-Flow Contribution

Add inherited clipping and decide the remaining scrollable overflow
contributors with explicit tests.

Scope:

- clipping rectangles inherited by descendants;
- nested scroll container geometry where inner ranges are independent from
  outer clipping;
- absolute and floating contribution rules for scrollable overflow;
- partially clipped scroll container tests;
- hidden/visibility interactions with scroll geometry output.

Out of scope:

- runtime invalidation or clip stack storage;
- render paint commands;
- platform hit testing behavior.

Completion evidence:

- tests cover clipped parent with overflowing child, inner scroll container
  inside clipped outer container, and out-of-flow contribution decisions;
- tests cover hidden and display-none paths so hidden output does not leak stale
  scroll geometry;
- output facts give root/runtime/render enough geometry to derive clip chains
  without layout storing runtime state.

## Phase 6: Overflow Auto And Scrollbar Gutter Policy

Represent and compute post-layout scrollability decisions that cannot be known
before layout.

Scope:

- typed layout-ready representation for `overflow: auto`;
- auto scroll container/range behavior after overflow measurement;
- typed layout-ready representation for `scrollbar-gutter: auto`, `stable`,
  and `both-edges`;
- scrollport and gutter rect behavior for auto, stable, and both-edge gutters;
- strict diagnostics or root rejection for any platform scrollbar policy that
  remains unsupported after these CSS gutter values are modeled;
- writing-mode and direction tests for auto/stable gutter placement.

Out of scope:

- platform overlay scrollbar policy unless supplied as a typed input;
- live scroll offset updates;
- scrollbar painting or hit state.

Completion evidence:

- tests prove auto with no overflow, auto with overflow, stable gutter without
  overflow, both-edge gutter, and unsupported platform-policy diagnostics;
- browser fixture shortcuts no longer masquerade as semantic `auto` support.

## Phase 7: Scroll Target And Snap Geometry

Add target geometry only after the basic scroll container contract is stable.

Scope:

- typed `scroll-padding` input and snapport output;
- typed `scroll-margin` input and target-area output;
- `scroll-snap-type` and `scroll-snap-align` geometry facts if root chooses to
  enable snap in this pass;
- strict unsupported diagnostics for any snap feature still outside layout;
- tests for target geometry across writing modes and nested scroll containers.

Out of scope:

- runtime snap selection, kinetic scrolling, smooth scrolling, or animation;
- CSS parsing and style cascade;
- render painting.

Completion evidence:

- layout emits static target/snap geometry only;
- runtime-owned behavior remains outside layout;
- root can decide whether to integrate snap features based on typed layout
  readiness.

## Final Sequence Gate

After all phases are implemented:

- run `cargo test -p surgeist-layout`;
- run `cargo clippy -p surgeist-layout --all-targets -- -D warnings`;
- run `cargo fmt --check`;
- run `git diff --check`;
- run the green-baseline XML corpus command used by the active implementation
  plans, if any phase touched browser parity behavior;
- assign a final clean-context holistic reviewer to inspect the full scroll
  geometry result against this sequence, the support matrix, crate boundary,
  tests, and the modeling guide.

The full goal is complete only after that final holistic review is clean.
