# CSS Scroll Geometry Support Matrix

Date: 2026-07-09

## Purpose

This matrix converts the root CSS scroll math directive into a layout-owned
support inventory. It is not an implementation plan. It records what
`surgeist-layout` must eventually own, what is already partially represented,
what belongs to sibling crates, and what should be sequenced into later
implementation plans.

Primary inputs:

- `plans/2026-07-09-css-scroll-math-planning-directive.md`
- `/Users/codex/Development/surgeist/plans/2026-07-04-css-integration-support-inventory.md`
- `/Users/codex/Development/surgeist/guidance/surgeist-rust-modeling-guide.md`
- Current `surgeist-layout` source as of `26d559f5`

## Boundary

Layout owns geometry derived from already-resolved layout inputs and computed
box results. Layout must not parse CSS, run the cascade, store live scroll
offsets, route input, animate scrolling, paint scrollbar chrome, or lower root
adapter data.

The required layout-facing model should be layout-ready and scalar-generic over
`S: LayoutScalar`. It should distinguish authored/resolved style values from
layout geometry and runtime scroll state:

- style/root provide resolved overflow-facing inputs;
- layout computes scrollport, overflow, range, clamp, clipping, and paint/hit
  geometry facts;
- runtime stores live offsets and applies clamp facts;
- render paints clipped content and scrollbar chrome from layout/runtime facts.

## Current Layout Baseline

| Area | Current status | Notes |
| --- | --- | --- |
| Overflow enum | Partial | `Overflow` has `Visible`, `Clip`, `Hidden`, and `Scroll`; there is no `Auto` representation. |
| Browser fixture overflow intake | Partial | Current browser fixture support may collapse unsupported/unknown overflow values such as `auto` into the existing `Overflow` surface; this must not be mistaken for semantic `overflow: auto` support. |
| Scrollability predicate | Partial | `Overflow::is_scrollable()` returns true only for `Scroll`. |
| Clipping predicate | Partial | `Clip`, `Hidden`, and `Scroll` clip contents; `Visible` does not. |
| Margin-collapse interaction | Partial | `Hidden` and `Scroll` block block margin collapse; `Clip` currently does not. |
| Scrollbar reservation | Partial | Block, flex, grid, and leaf-style paths reserve `scrollbar_width` only when the relevant overflow axis is `Scroll`. |
| Scrollbar gutter style | Missing | There is only a scalar `scrollbar_width`; no typed `scrollbar-gutter` policy exists. |
| Scrollport output | Missing | `NodeOutput` does not expose a scrollport, visual viewport, clipping rect, scrollable overflow area, or scroll ranges. |
| Scrollable overflow accumulation | Missing | Content overflow behavior exists in specific algorithms, but no typed scrollable overflow area contract exists. |
| Live scroll offset | Out of scope | Runtime owns live position; layout should only expose clamp/range geometry. |
| Browser fixture metadata | Partial | The browser helper captures `scrollWidth` and `scrollHeight`, but layout has no typed scroll geometry oracle yet. |

## Feature Support Matrix

| Feature or property | Layout responsibility | Current support | First-pass target | Non-layout owner or gate |
| --- | --- | --- | --- | --- |
| `overflow-x: visible` | No clip on the physical x axis unless forced by another ancestor; visible overflow contributes to ancestor scrollable overflow. | Partially modeled as `Overflow::Visible`. | Compute visible descendant overflow and inherited clip interactions explicitly. | Style/root lower resolved axis value. |
| `overflow-y: visible` | Same as x axis for physical y. | Partially modeled as `Overflow::Visible`. | Same as x axis. | Style/root lower resolved axis value. |
| `overflow: hidden` | Clip at the padding edge and expose a programmatic scroll range if content overflows. | Clips and blocks margin collapse; no scroll range output. | Add scroll container geometry and ranges while preserving hidden clipping. | Runtime may choose not to expose user scrolling. |
| `overflow: clip` | Clip at the overflow clip edge and expose no scroll range. | Clips; no clip edge or range output. | Add non-scrollable clip geometry; reject unsupported `overflow-clip-margin` until modeled. | Style/root reject or defer `overflow-clip-margin`. |
| `overflow: scroll` | Always reserve scrollbar gutter where applicable and expose scroll range even when range is zero. | Partially reserves gutter for `Scroll`; no range output. | Preserve existing gutter reservation, add scrollport and max offset facts. | Runtime owns offset state. |
| `overflow: auto` | Decide scroll container geometry from overflow need after layout; reserve gutter according to policy. | Missing. | Add a layout-ready value or policy that can represent auto resolution without re-parsing CSS. | Style/root must decide how `auto` reaches layout. |
| Mixed-axis overflow | Compute independent horizontal and vertical clip/range/gutter facts while honoring CSS visible-to-auto coupling if root/style preserve it. | Partially axis-aware with `Point<Overflow>`. | Make axis coupling explicit in the input contract or require root to resolve it before layout. | Root/style own CSS computed-value rules. |
| `scrollbar-gutter: auto` | Reserve gutter only according to overflow policy and platform width facts supplied to layout. | Missing; implicit classic gutter for `Scroll`. | Add typed gutter policy if first pass includes gutters beyond current `Scroll` behavior. | Root/style lower resolved gutter; window/platform supplies width if not fixed. |
| `scrollbar-gutter: stable` | Reserve gutter even when not currently scrollable, depending on axis and writing direction. | Missing. | Include in first implementation pass if `auto` is supported; otherwise root rejects. | Style/root must expose resolved value. |
| `scrollbar-gutter: both-edges` | Reserve symmetric gutters on both inline edges. | Missing. | Defer unless first gutter plan includes stable gutters. | Root rejects until layout type exists. |
| `scroll-padding` | Provide snap/scroll target inset geometry, not box layout sizing. | Missing. | Deferred with strict root rejection or an explicit unsupported diagnostic until target geometry is in scope. | Root must not silently store or pass this to layout before layout has a typed input. |
| `scroll-margin` | Provide target-area expansion for snap/scroll-into-view geometry, not layout sizing. | Missing. | Deferred with strict root rejection or an explicit unsupported diagnostic until target geometry is in scope. | Root/style must not silently materialize this as inert data before layout has a typed input. |
| `scroll-snap-type` | Compute snapport and axis facts from scrollport and scroll padding. | Missing. | Defer from first scroll geometry pass unless root explicitly prioritizes snap. | Runtime owns choosing snap positions during scrolling. |
| `scroll-snap-align` | Compute candidate snap areas from border box plus scroll margin. | Missing. | Defer with diagnostics until snap geometry pass. | Root/style lower values; runtime uses candidates. |
| Writing mode | Map logical scroll axes, scrollbar gutter edges, and reported geometry consistently. | Partial; gutter placement currently uses direction and physical axes in algorithms. | Add tests and typed mapping for horizontal-tb, vertical-rl, and vertical-lr. | Style/root lower resolved writing mode. |
| Direction | Place inline-axis scrollbar gutters and horizontal ranges consistently for LTR/RTL. | Partial; gutter edge uses direction in block/flex/grid. | Define whether ranges are logical-positive or physical-positive and expose that in types. | Runtime adapts live offsets to platform conventions. |
| Border and padding | Scrollport is padding box adjusted by scrollbar gutters; content overflow starts from padding/content geometry as specified. | Partial through content-box inset calculations. | Centralize scrollport construction from border box, padding, border, and gutter. | Style/root provide resolved border/padding lengths. |
| Box sizing | Authored size interacts with border/padding/gutter before scrollport calculation. | Partial in existing algorithms. | Preserve current sizing behavior and add scroll geometry tests around content-box and border-box. | Style/root provide resolved box sizing. |
| Nested scroll containers | Each clipping ancestor constrains descendant visible rect; inner scroll ranges are computed from inner overflow, not outer clipping. | Missing as typed output. | Add inherited clipping rectangles and nested scrollport tests. | Runtime/render consume the resulting clip chain. |
| Out-of-flow descendants | Determine whether absolute/floating descendants contribute to scrollable overflow. | Not modeled as scroll overflow. | Explicitly decide and test per CSS rules before claiming full support. | Root/retained supply tree structure; layout computes geometry. |
| Root viewport | Root scrollport/viewport geometry must be reported separately from ordinary block containers. | Missing. | Required in the first scroll geometry contract as viewport/document scroll facts. | Runtime/window own live viewport state and platform events. |
| Unsupported values | Reject or diagnose values layout cannot consume instead of silently ignoring them. | Missing as typed diagnostics. | Add semantic unsupported diagnostics or require root rejection before layout. | Root owns integration diagnostics surfaced to users. |

## Geometry Output Matrix

| Output fact | Meaning | First-pass status |
| --- | --- | --- |
| Scroll container flag | Whether this node establishes scroll container geometry for each axis. | Required. |
| Border-box rect | Existing layout output rect used as the outer geometry basis. | Already available indirectly. |
| Padding-box rect | Basis for most scrollport and clip calculations. | Required as typed derived geometry. |
| Scrollport rect | Visible scrollport after border, padding, and gutter rules. | Required. |
| Overflow clip rect | Rect inherited by descendants for `hidden`, `clip`, `scroll`, and `auto` when clipping. | Required. |
| Scrollable overflow rect | Union of scrollable descendant geometry in container coordinates, with explicit rules for descendant margin boxes, border boxes, padding boxes, and out-of-flow boxes. | Required. |
| Maximum scroll offset | Non-negative max offset per axis derived from scrollable overflow minus scrollport. | Required. |
| Clamp operation | Pure math to clamp a proposed runtime offset into valid range. | Required; no live state. |
| Scrollbar gutter rects | Geometry root/render/runtime can use to derive scrollbar hit/paint rects for every supported scrollbar reservation, including current `overflow: scroll` gutters. | Required for supported scrollbar reservation. |
| Snapport rect | Scrollport adjusted by scroll padding. | Deferred unless snap is in first pass. |
| Snap candidate areas | Target border boxes expanded by scroll margin. | Deferred unless snap is in first pass. |

## Algorithm Coverage Matrix

| Layout algorithm | Required scroll work | Notes for sequencing |
| --- | --- | --- |
| Root layout | Report viewport/document scrollport and document scroll range without storing live offset. | Should be sequenced early because runtime/render need this front door. |
| Block layout | Compute scrollport, content overflow, inherited clipping, and child contributions including inline runs. | Natural first algorithm after shared types. |
| Inline layout | Contribute inline line boxes and inline participants to ancestor scrollable overflow; do not become a scroll container by itself unless represented as a box. | Depends on block integration. |
| Flex layout | Preserve current gutter sizing behavior and add scrollable overflow/range for flex containers and items. | Should follow shared scroll geometry helpers to avoid algorithm drift. |
| Grid layout | Preserve current gutter sizing behavior and add scrollable overflow/range for grid containers/items/subgrid cases. | Needs focused tests because grid already uses overflow in intrinsic sizing. |
| Absolute/out-of-flow layout | Decide contribution rules and coordinate space for scrollable overflow. | Should be a separate implementation task if not fully known. |
| Hidden layout | Hidden nodes should not leak scroll geometry unless their visible ancestor output contract requires zero/hidden facts. | Should be tested with existing hidden compute paths. |

## Proposed Sequencing Implications

1. Define shared typed scroll input/output contracts.
   Include scalar-generic geometry types, axis/range/clamp types, and an
   explicit unsupported diagnostic strategy.
2. Centralize scrollport and gutter math.
   Replace duplicated block/flex/grid/leaf-style gutter calculations only after
   tests lock current behavior.
3. Implement overflow accumulation and scroll range math for block/root.
   This gives root/runtime/render a usable contract while keeping scope small.
   The first implementation plan must define whether each contributor uses its
   margin box, border box, padding box, or a more specific CSS overflow shape.
4. Extend flex and grid to emit the same scroll geometry facts.
   Use existing overflow/gutter tests as regression anchors.
5. Add inherited clipping and nested scroll container output.
   Verify that visible overflow, clipped overflow, and nested scroll ranges
   remain separate facts.
6. Decide auto/gutter broadening.
   Either support `overflow: auto` and `scrollbar-gutter` with typed inputs or
   require root rejection until a dedicated plan lands.
7. Defer snap, scroll-padding, and scroll-margin unless root chooses snap as an
   early product requirement.
   These need target geometry and runtime behavior beyond basic scroll ranges.

## Test Matrix For Later Plans

| Test area | Required examples |
| --- | --- |
| Scrollport construction | Border-box and content-box sizing; padding; border; gutter on LTR and RTL. |
| Overflow keywords | Visible, hidden, clip, scroll, and auto once represented. |
| Scroll ranges | No overflow, inline overflow, block overflow, both-axis overflow, negative proposed offset clamp, over-max clamp. |
| Writing mode | Horizontal-tb LTR/RTL, vertical-rl, vertical-lr, mixed axis overflow. |
| Nested clipping | Clipped parent with overflowing child; inner scroll container inside clipped outer container. |
| Algorithm coverage | Block, root, flex, grid, inline contribution, absolute/out-of-flow contribution decision. |
| Gutter policy | Scroll-only current behavior, stable, both-edges, and overlay/no-gutter policy if represented. |
| Unsupported diagnostics | Snap values before snap support, scroll padding/margin before target support, overflow auto before auto support if deferred. |
| Browser parity | Fixtures using generated `scrollWidth`/`scrollHeight` once the oracle can compare typed scroll geometry. |

## Open Decisions Before Sequencing

| Decision | Why it matters | Recommended first answer |
| --- | --- | --- |
| Should layout accept `Overflow::Auto` directly? | `auto` needs post-layout overflow knowledge; omitting it forces root to reject or pre-resolve something it cannot know. | Add a typed layout-ready auto variant or scroll policy in layout. |
| Are scroll ranges physical or logical in public output? | Runtime and render need stable interpretation across writing modes and platform conventions. | Store physical geometry plus explicit axis metadata; provide helper methods for logical access. |
| Should `overflow: hidden` expose ranges? | Browsers allow programmatic scrolling for hidden but not clip. | Yes, expose range facts for hidden and scroll/auto; expose none for clip. |
| Does `overflow: clip` block margin collapse here? | Current `blocks_margin_collapse()` excludes `Clip`; CSS behavior may need confirmation before changing. | Audit before implementation; do not change as part of matrix work. |
| Are scroll snap geometry and scroll target geometry in the first implementation sequence? | Snap needs scroll-padding, scroll-margin, candidate areas, and runtime use. | Defer from first sequence unless root explicitly prioritizes snap. |
| Do out-of-flow boxes contribute to scrollable overflow in pass one? | CSS has specific contribution rules; incorrect inclusion can distort ranges. | Make this a focused subtask with browser parity fixtures. |

## Root Coordination Notes

- Root/style must provide resolved scroll-facing values; layout should not
  consume authored CSS syntax.
- Runtime must store live offsets and call layout-provided clamp math; layout
  should not persist offset state.
- Render should receive enough geometry to paint clipped content and scrollbar
  chrome, but layout should not choose colors, animations, hover state, or
  backend paint commands.
- Unsupported scroll properties accepted by CSS/style must become explicit root
  diagnostics until layout exposes typed inputs for them.
