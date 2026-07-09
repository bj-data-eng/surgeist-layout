# CSS Scroll Math Planning Directive

This directive asks `surgeist-layout` to design a crate-local implementation
plan for CSS-spec scroll geometry support.

Layout should own scroll math that follows from computed layout boxes and
resolved overflow-facing style inputs. Layout should not own live scroll state,
platform input handling, animation scheduling, scrollbar painting, resource
loading, CSS parsing, style cascade, or root lowering.

## Scope

Write an implementation plan for layout-owned scroll geometry. The plan should
cover only layout responsibilities:

- scroll container geometry
- scrollport and viewport rectangle calculations
- scrollable overflow area calculations
- scroll range and maximum scroll offset calculations
- scroll offset clamping math
- overflow clipping rectangles inherited by descendants
- scrollbar gutter participation in layout when applicable
- geometry needed by root/runtime/render to derive scrollbar hit and paint
  rectangles without storing runtime state in layout
- strict unsupported-value diagnostics for scroll-related style inputs layout
  cannot consume yet

## CSS Features This Enables

The layout plan should explicitly account for the geometry implications of:

- `overflow-x`
- `overflow-y`
- `overflow: visible`
- `overflow: hidden`
- `overflow: clip`
- `overflow: scroll`
- `overflow: auto`
- `scrollbar-gutter`
- `scroll-padding`
- `scroll-margin`
- `scroll-snap-type`, `scroll-snap-align`, and related snap geometry inputs,
  if the plan decides to include snap math now
- writing mode and direction interactions with scroll axes
- border, padding, and box-sizing interactions with scrollport computation
- nested scroll containers and inherited clipping

If a feature is represented by CSS/style but layout should not support it in
the first implementation pass, the plan should say so explicitly and define the
diagnostic or root rejection expected before it reaches layout.

## Boundary Rules

Do not add CSS parsing, cascade, selector matching, retained identity, runtime
state, host input routing, rubberbanding physics, smooth scrolling, scroll
timelines, animation sampling, or render paint decisions to layout.

The expected boundary is:

- CSS owns syntax and authored value contracts.
- Style owns resolved scroll-related property values.
- Root owns Surgeist-to-Surgeist lowering and decides which resolved style
  values reach layout.
- Layout owns geometry derived from layout boxes and resolved scroll inputs.
- Runtime owns live scroll positions, user/programmatic scroll updates,
  overscroll/rubberband state, smooth-scroll scheduling, and invalidation.
- Window owns platform scroll and pointer input events.
- Render owns painting clipped content and scrollbar chrome from root/runtime
  snapshots.
- Test owns broad integration, app-surface, and benchmarking coverage once the
  pipeline is connected.

Scrollbar chrome may need geometry from layout, but layout should not own
hover/pressed/dragged state, colors, visibility animations, backend paint
commands, or platform scrollbar policy.

## Planning Requirements

The implementation plan should:

- identify the public layout APIs root/runtime/render would call
- define typed scroll geometry inputs and outputs with explicit units
- decide whether new scroll types should be generic over `S: LayoutScalar`
- reuse existing layout geometry types where they preserve meaning clearly
- specify how scroll geometry is computed for block, flex, grid, inline, and
  root layout outputs
- specify how scrollable overflow includes descendants, margins, borders,
  padding, and out-of-flow boxes, or explicitly defer unsupported cases
- specify how `overflow: auto` is represented after root/style lowering and how
  layout determines whether scrolling is actually needed
- specify how `overflow: hidden` and `overflow: clip` differ for clipping and
  scroll range exposure
- specify how writing mode and direction affect horizontal and vertical scroll
  ranges, scrollbar gutter placement, and reported geometry
- specify nested clipping behavior and tests for partially clipped scroll
  containers
- specify whether scroll snap geometry is in this pass or deferred
- include tests for scrollport computation, scrollable overflow, clipping,
  nested scroll containers, scrollbar gutter sizing, scroll range clamping,
  writing-mode/direction interactions, and unsupported-value diagnostics

## Initial Non-Goals

- live scroll state storage
- wheel/touchpad/keyboard scroll routing
- rubberbanding or overscroll physics
- smooth scrolling
- scroll-linked animation
- scrollbar painting or colors
- pseudo-element scrollbar styling
- host/platform scrollbar integration
- CSS parsing or style cascade

These may be integrated later through root/runtime/render, but this directive
is only about layout-owned CSS scroll math.

## Review Gate

Before implementation, have a clean-context reviewer check the layout plan
against:

- this directive
- layout's crate boundary
- `/Users/codex/Development/surgeist/guidance/surgeist-rust-modeling-guide.md`
- the root inventory at
  `/Users/codex/Development/surgeist/plans/2026-07-04-css-integration-support-inventory.md`

Completion for this directive is a reviewed implementation plan in layout's
`plans/` folder, not code changes.
