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

Layout owns algorithm inputs, traversal contracts, caches, reports, and box
output. Retained tree identity, root ownership, and sibling coordination belong
to the retained/root integration layers that provide the tree implementation to
this crate.

## API Artifact

The committed API coordination artifact lives at `api/public-api.txt`, but the
generator is owned by the root `surgeist` repo.

Refresh this crate's artifact from the root repo with:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-layout
```

API refresh tooling is command-only and must not run as part of normal `cargo test`.

## Modeling Contracts

`surgeist-layout` exposes layout-ready contracts rather than authored CSS syntax.
Public placement, aspect ratio, track repetition, lane, and calc values preserve
their invariants through typed constructors and resolver-aware APIs.
