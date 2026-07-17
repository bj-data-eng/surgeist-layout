# surgeist-layout Notice

`surgeist-layout` is maintained as part of the Surgeist UI framework.

The current Surgeist layout implementation is a Surgeist-shaped layout engine with its own public data model, traversal contracts, algorithm phases, and test oracle.

The implementation has been informed by:

- CSS layout specifications.
- Browser behavior observed through local parity fixtures.
- surgeist-layout was ported and adapted from Taffy 0.10.1 and has since diverged substantially. We still use Taffy's layout fixtures as supplemental tests.
- WebKit and Blink algorithms were studied as implementation references for grid, subgrid, grid-lanes, inline layout, and baseline behavior.
