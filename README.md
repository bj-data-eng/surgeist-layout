# surgeist-layout

Layout primitives, algorithms, oracle tests, and fixture tooling for Surgeist document surfaces.

## Scalar Precision

`surgeist-layout` keeps its default public API at browser-style coordinate
precision: `DefaultScalar` is `f32`, and `Scalar` aliases `DefaultScalar`.
Most applications should use the default aliases such as `NodeInput`,
`ComputeInput`, and `ComputeOutput`.

For applications that need more coordinate precision, scalar-bearing APIs also
provide generic `*Of<S>` forms, and `LayoutScalar` is implemented for both
`f32` and `f64`. Pick one scalar type for a layout tree and run it end-to-end:
do not mix `f32` and `f64` values inside one tree, cache, traversal, or layout
run.

Browser parity XML and its generator remain a default-precision fixture
boundary. Use those fixtures to check the default `f32` contract; add separate
crate-local tests when a behavior specifically needs `f64` coverage.

## Browser Parity Runtime

The browser-parity corpus pins its Chrome-for-Testing source, executable version,
cache root, launch profile, and report inventory in
`tests/layout/browser_parity/corpus.toml`. `generate` is the managed-pinned mode;
it is the only command that may use the configured browser fetcher.
`generate-existing` is the existing-pinned, no-fetch artifact mode: it requires
`SURGEIST_BROWSER_PATH` to be a repository-relative executable beneath the
manifest cache and verifies its exact `--version` output before any XML or
report write. Both modes use the same headless launch profile, including
`use-mock-keychain`.

`check-corpus`, `check-taffy-corpus`, and `import-taffy` are browser-free command
paths. In particular, `check-corpus` validates the committed report and XML
provenance without reading a browser selection or generation filter.

Layout owns normalized layout values, algorithm inputs, traversal contracts,
caches, reports, and box output. Retained tree identity and sibling coordination
belong to the root integration layer that provides the tree implementation.

## Modeling Contracts

`surgeist-layout` exposes layout-ready contracts rather than authored CSS syntax.
`LengthPercentageOf<S>` is a normalized finite affine value: px plus a percentage
coefficient. Resolve it only against an explicit `PercentageBasisOf<S>`;
`PercentageBasisOf::definite` rejects invalid values at construction. Resolution
reports `MissingBasis` for a required missing basis and `InvalidNumeric` for a
non-finite evaluation; no value is guessed.

Preferred size, minimum size, maximum size, and flex basis are distinct closed
property domains. Their role-valid keywords cannot cross property boundaries.
`SizingCalculationOf<S>` combines finite affine leaves with nested `min`, `max`,
and `clamp` in a validated program that is evaluated iteratively. Percentages
remain symbolic until layout receives an explicit basis, and a required missing
basis remains unresolved.

Canonical layout-ready `calc-size()` input pairs a property-specific basis with a
validated `CalcSizeCalculationOf<S>` containing finite absolute-pixel,
percentage, and size coefficients. Track flex is separate: construct a finite,
non-negative `TrackFlexFactorOf<S>` through `try_new` and place it only in a
maximum track breadth. A valid sizing behavior owned by a later algorithm is
reported through `LayoutUnsupportedCapability::SizingBehavior` with the exact
property, behavior, algorithm, and axis instead of an automatic fallback.

`LayoutRootRequestOf<S>` validates root input for the public
`compute_layout` front door. A successful call returns a
`CompletedLayoutBatchOf<Node, S>` containing the staged layout and cache updates;
a `LayoutErrorOf<Node, S, M>` returns no partial public result. Recursive
algorithm modes remain internal.

`compute_leaf` is the direct, fallible leaf-measurement boundary. Its provider
receives non-negative content-space constraints, and invalid provider output or a
provider error becomes a typed layout error.

`ItemOrder` is the layout-ready signed order value. `SourceIndex` is stable
source-sibling identity: outputs remain source-associated while flex, ordinary
grid, and grid-lanes consume one stable order-modified traversal sorted by item
order and then source index.

`item_is_replaced` is an independent box-generation fact. It is not inferred
from table role, measurement, aspect ratio, or stretch. Block and root sizing
use it to avoid ordinary auto-inline fill, flex uses it when selecting automatic
main-size suggestions, and grid and grid-lanes use it when resolving normal
alignment while preserving explicit stretch.

`ContainingLayoutContext` keeps the containing flow axes and
`ParentFormattingContext` role together as the complete containing context and
cache identity. Flex-item roots require explicit parent flow axes and keep the
host allocation in the root request separate from the viewport percentage
context in `FlexItemRootContext`.

Root `surgeist` owns cross-crate adapters, including canonicalizing authored CSS
sizing values, lowering computed-style values and authored CSS order into these
layout contracts, and rejecting property-invalid authored states before layout.
This crate does not parse authored CSS. Root also owns box-generation
replacedness, invalidation, consumer migration and renames, facade composition,
integration, and generated API artifacts; this crate carries no root adapters or
API artifact copies.

## Geometry, Flow, And Scroll Contracts

The public physical geometry contract uses x/y points, width/height sizes, and
top/right/bottom/left edges. Public layout outputs, cached geometry, and scroll
geometry remain physical. Layout algorithms may use
crate-private logical algorithm geometry while working in inline/block
coordinates. Those carriers stay private until the owning `FlowAxes` projects
them to physical geometry at a contextual boundary.

`FlowAxes` is the sole production owner of writing-mode mapping for
`HorizontalTb`, `VerticalRl`, `VerticalLr`, `SidewaysRl`, and `SidewaysLr`. Its
`Direction` is the already-resolved used inline direction, not authored or
otherwise unresolved CSS. Root `surgeist` owns computed-style lowering and
supplies that used value through its cross-crate adapters.

The signed physical scroll ranges and signed flow-relative scroll ranges keep
finite ordered minimum and maximum bounds. When an axis runs in reverse,
`FlowAxes` swaps and negates the endpoints so negative minima and maxima retain
their meaning. Layout owns scroll-container geometry, not a current offset;
root integration owns live scroll state and host/CSSOM policy.

Root also owns generated API artifacts.
The later inline, overflow, flex, grid, alignment, and positioned initiatives
remain outside this geometry closure and are not claimed here.

## Inline Metrics Contract

`InlineMetricsOf<S>` is layout-ready line box data. Layout consumes it for inline
line construction and does not derive it from authored CSS or fonts. Integration
layers should provide metrics from computed style and text/font measurement
before constructing `LineBreakInputOf<S>`.
