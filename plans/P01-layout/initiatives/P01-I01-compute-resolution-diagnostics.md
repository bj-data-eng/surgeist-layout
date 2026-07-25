# P01-I01 Compute, Resolution, And Diagnostic Contracts


Design owner: `surgeist-layout`

Specification ID: `FRI-01`

## 1 FRI-01.1 Authority And Outcome

This specification is the direct desired-state contract for `FRI-01` in
`plans/P01-layout/P01-index.md`. It owns
closure of `CORE-001`, `CORE-002`, `CORE-003`, `CORE-004`, `CORE-007`, and
`DIAG-001` from
`plans/P01-layout/P01-initial-review-findings.md`.

The outcome is a layout computation substrate in which:

1. every public tree-backed compute request returns one typed result or one typed
   error that preserves the failing node and operation;
2. calc-bearing layout values are normalized affine length-percentage values
   owned by layout, with no resolver, store, generation, foreign identity, or
   hidden lifecycle;
3. contextual length-percentage resolution receives its percentage basis
   explicitly and reports `MissingBasis` only when the normalized value actually
   depends on that basis;
4. cached and uncached computations return the complete same
   `ComputeOutputOf<S>`;
5. root computation produces one validated layout-state batch and exposes no
   partially written node output or cache state on failure;
6. leaf measurement receives non-negative finite content-space constraints and
   invalid measurement output is rejected before layout output is produced;
7. scrollbar width and flex factors are property-specific validated values; and
8. unsupported, invalid, missing-context, measurement-provider, and
   internal-invariant states are distinguishable without panic, silent zero
   geometry, guessed context, or string matching.

This is a breaking pre-release correction. Backward compatibility is not
required. Removed APIs are not retained through aliases, adapters, overloads,
default resolver behavior, or duplicate compute paths.

## 2 FRI-01.2 Scope And Non-Goals

### 2.1 Owned Scope

This specification owns the layout crate's:

- normalized affine length-percentage value and contextual resolution contract;
- public root compute requests and private recursive compute input contract;
- compute session, tree read contract, validated layout-state batch, and root
  pass result;
- cache storage and complete-output equivalence;
- leaf measurement input and result validation boundary;
- validated scrollbar-width, flex-grow, and flex-shrink values;
- unified compute error envelope and current domain-error composition;
- public reexports, crate documentation, tests, and layout-owned parity support
  required by those contracts.

### 2.2 Explicit Non-Goals

This specification does not:

- implement missing inline, vertical-clear, grid, subgrid, grid-lanes, overflow,
  positioned, display-system, or fragmentation behavior owned by later FRI
  initiatives;
- treat a typed unsupported error as closure of any later capability finding;
- redesign the property-specific box, track, flex-basis, and intrinsic sizing
  families owned by `FRI-04`;
- complete scroll-geometry coherence owned by `FRI-05`;
- define authored CSS syntax, cascade, style resolution, text shaping, retained
  identity, painting, or live scroll state;
- add a dependency on `surgeist-style`, `surgeist-retained`, root `surgeist`, or
  any new third-party crate;
- edit root adapters, root facade exports, root API artifacts, or sibling crates;
  those changes are a root-owned integration handoff; or
- make the full browser corpus green when a fixture still exercises a finding
  owned by another initiative.

## 3 FRI-01.3 Historical Baseline Evidence

Unless an exact later revision is named, this section records the planning
baseline at `8ba9186909b7692a3bfcf88cf39254d16498562c`; it does not describe the
post-initiative repository tip.

| Evidence ID | Current source fact | Required correction |
| --- | --- | --- |
| `E-CACHE-1` | `src/cache.rs` stores only `output.size` for `RunMode::ComputeSize` and reconstructs the hit with `ComputeOutputOf::from_outer_size`. | Every cache slot stores and returns the full output. |
| `E-CALC-1` | `src/compute.rs::compute_leaf` installs `NoCalcResolver`; the resolver-aware helper is crate-private. | Resolver-free calc panics disappear because calc values carry their complete affine normalized meaning inline. |
| `E-CALC-2` | `src/grid/lanes.rs::place_lanes` and `lane_intrinsic_sizing` contain resolver-free paths. | Standalone and tree-backed lane APIs use the same explicit basis resolution and propagate typed failures. |
| `E-CALC-3` | `CalcId` is a `u32` index and `CalcGeneration` is store length. | IDs, stores, generations, and resolver traits are removed from the layout calc model. |
| `E-CALC-4` | The current calc domain is px and percent terms; style-side authored sums lower to px-plus-percent coefficients before layout consumes them. | Use one layout-owned affine value: `absolute_px + percent_fraction * basis`. |
| `E-MEASURE-1` | `compute_leaf` can subtract content-box insets into a negative `AvailableOf::Definite` before invoking measurement. | Convert to a measurement-specific non-negative content-space type after flooring at zero. |
| `E-NUMERIC-1` | `NodeInputOf` exposes raw `S` for `scrollbar_width`, `flex_grow`, and `flex_shrink`. | Use three distinct finite non-negative newtypes. |
| `E-DIAG-1` | `Compute::compute_child` and public compute functions return plain outputs while invalid and unsupported cases use panics, reports, silent fallback, or unrelated error types. | Return one site-aware result envelope and compose typed domain details and provider sources. |
| `E-WRITE-1` | Block, flex, grid, hidden, root computation, caches, and rounding mutate caller storage before the containing request is known to succeed. | Compute into one owned batch and expose completed state only after validation succeeds. |
| `E-ROOT-1` | Root owns style-to-layout lowering and previously constructed layout calc IDs while building `NodeInput`. | Root later lowers style calc into layout affine coefficients and uses the validated public request contracts. |
| `E-MSRV-1` | The user-selected project MSRV and current root manifest worktree are Rust 1.97; this leaf manifest has no independent `rust-version`. | Declare `rust-version = "1.97"` in this leaf and verify the implementation with the already-installed Rust 1.97 toolchain. |

## 4 FRI-01.4 Resolved Design Decisions

### 4.1 `D-01` Calc Is A Storeless Normalized Affine Value

Layout calc values are represented by
`LengthPercentageOf<S> { absolute_px: S, percent_fraction: S }` with private
fields. The invariant is that both coefficients are finite and signed zero is
canonicalized to positive zero. Equivalent px/percent/calc inputs therefore have
one normalized representation inside layout.

`LengthPercentageOf<S>` is the least powerful construct for the layout-owned
calc domain because layout only needs additive px and percentage terms. It
contains no authored syntax tree, no expression identity, no store identity, no
resolver trait, and no revision. Root style lowering owns parsing authored CSS
calc syntax and reducing it to these coefficients.

Rejected alternative: an arena or frozen store with `CalcId` would model a
lifecycle and identity that layout does not need. It would keep `CORE-002` and
`CORE-003` alive by making resolution depend on external resolver composition
instead of value semantics.

Rejected alternative: retaining separate public `Px`, `Percent`, and `Calc`
variants would preserve phase leakage and force every algorithm to decide which
variant is basis-dependent.

### 4.2 `D-02` Resolution Is Explicit Basis Evaluation

`LengthPercentageOf<S>::resolve_against` receives a
`PercentageBasisOf<S>`. A definite basis is finite and non-negative. A missing
basis is explicit. A definite basis returns the scalar
`absolute_px + percent_fraction * basis` only when the computed result is finite.
A missing basis returns `NumericResolutionOf::MissingBasis` only when
`percent_fraction` is nonzero. A basis-independent value resolves without
context. A non-finite computed result returns
`NumericResolutionOf::InvalidNumeric`; it is not guessed or saturated.

Missing basis is not automatically a compute error. Each consuming property
algorithm decides whether missing basis has a valid CSS/layout behavior or must
be raised as `LayoutMissingContext::RequiredBasis`.

Invalid numeric resolution is always an error when consumed by computation and
maps to `LayoutErrorKindOf::InvalidInput` with `LayoutOperation::ValueResolution`
and the node or standalone operation site that requested resolution.

Rejected alternative: mapping every missing basis to zero guesses context.
Rejected alternative: rejecting every missing basis would break valid intrinsic
and cyclic sizing behavior.

### 4.3 `D-03` Current Value Families Collapse Only Where Their Phase Is The Same

This initiative removes calc-specific variants but does not complete the later
property-specific sizing split. Current public value families move to the
smallest shape that closes `CORE-002` and `CORE-003`:

| Current family | FRI-01 shape | Note |
| --- | --- | --- |
| `LengthOf<S>` | `Normal` plus `Value(LengthPercentageOf<S>)` | `Normal` remains for current gap semantics until `FRI-04` narrows property families. |
| `LengthAutoOf<S>` | `Auto` plus `Value(LengthPercentageOf<S>)` | Auto remains symbolic. |
| `DimensionOf<S>` | Existing keyword/fr variants plus `Value(LengthPercentageOf<S>)` | `FRI-04` later removes illegal cross-property variants. |

No public layout value contains a calc ID or resolver. Construction from raw
scalar coefficients is fallible. `ZERO` is infallible because it uses layout-owned
finite constants; raw `px`, raw `percent_fraction`, and raw coefficient
constructors reject non-finite input. If implementation introduces a validated
finite scalar newtype, additional constructors that accept that type may be
infallible.

### 4.4 `D-04` The Compute Front Door Uses Requests, Not Public Algorithm Inputs

`RunMode`, `SizingMode`, and recursive `ComputeInputOf<S>` construction stop
being public surface. Recursive, root, child, and hidden-layout inputs become
crate-private and are only produced by the layout session. `ComputeInputOf<S>`
retains only the validated public `leaf_layout` and `leaf_content_size`
constructors required by the standalone `compute_leaf` front door. Public
tree callers start from typed root requests such as:

```rust
pub struct LayoutRootRequestOf<S: LayoutScalar = DefaultScalar> { /* private */ }

pub enum LayoutRootContextOf<S: LayoutScalar = DefaultScalar> {
    Viewport,
    FlexItemUnderViewport(FlexItemRootContextOf<S>),
}
```

`LayoutRootRequestOf<S>` owns validated root availability, root context, and
rounding mode. Its constructors reject non-finite dimensions and negative
definite availability. `FlexItemUnderViewport` is kept because checked-in
layout-owned browser XML already needs a root that participates as a flex item
under a viewport; it is not a retained-tree or style-layer concept.

Crate-private algorithm inputs still distinguish root layout, child layout,
hidden layout, intrinsic sizing, requested axis, known size, parent size, and
available size. Those states are unavailable to callers unless a public request
type intentionally exposes the corresponding operation.

Rejected alternative: keeping `ComputeInputOf` publicly constructible would keep
invalid run-mode and availability combinations expressible and would make every
internal scheduling change a public product surface.

### 4.5 `D-05` Root Computation Produces A Completed Batch

Public root layout returns:

```rust
pub type LayoutResultOf<Node, T, S, M> = Result<T, LayoutErrorOf<Node, S, M>>;

pub struct CompletedLayoutBatchOf<Node, S: LayoutScalar = DefaultScalar> {
    /* private validated node output and cache updates */
}
```

The batch contains every unrounded output, final rounded output, cache store,
and cache clear produced by the successful request. Algorithms write only to the
session's private staging area. If any validation or provider error occurs, the
batch is dropped and no public result exposes partially applied layout state.

Tree implementations may offer crate-local or public helper methods to apply a
`CompletedLayoutBatchOf` only when that method's contract is atomic. The compute
front door itself does not require a caller tree to mutate during computation.

Rejected alternative: an infallible public `apply_batch` hook would be false for
tree implementations that allocate during apply. Requiring computation to return
the validated batch keeps failure atomicity inside layout without pretending all
host state writes are infallible.

### 4.6 `D-06` Cache Writes Are Batch Entries

Cache hits and stores use the same private algorithm input identity as
computation and store complete `ComputeOutputOf<S>`. Cache mutations are staged
inside the same completed batch as node outputs. A cache hit is valid only if it
can return the exact full output that an uncached computation would have returned
for the same node, algorithm input, and normalized layout-ready values.

Because calc values are storeless value semantics, cache keys do not include calc
store identity, generation, resolver pointer, or revision. They include the
normalized layout-ready values through the node input identity or the cache's
existing node/input invalidation contract. If a future nonlinear value adds
external identity, that future type must define its own cache key contract.

### 4.7 `D-07` Measurement Is Content-Space And Validated

Leaf measurement receives one validated input:

```rust
pub struct LeafMeasureInputOf<S: LayoutScalar = DefaultScalar> { /* private */ }
```

Known and definite available dimensions are content-box values after subtracting
padding, border, and reserved scrollbar insets and flooring at zero. Intrinsic
availability remains symbolic as min-content or max-content. The callback
receives only `LeafMeasureInputOf<S>` and returns `Result<Size<S>, M>`, where
`M` is the compute provider's associated measurement error. The direct leaf
helper returns `LayoutResultOf<(), ComputeOutputOf<S>, S, M>` with standalone
site identity. Value-resolution failures use the unified invalid-input or
missing-context envelope, while provider and invalid-output failures use the
leaf-measurement operation. The tree-backed root/session result uses the same
envelope with node-bearing sites.

`LeafMeasureErrorOf<S, M>` is a C02-local public error with two closed variants:
`Provider(M)` and `InvalidOutput(InvalidMeasurementOutputOf<S>)`.
`InvalidMeasurementOutputOf<S>` records the physical `Axis` and a
`NonNegativeFiniteScalarErrorOf<S>` for the rejected provider component.
Layout preserves `M` exactly as the typed safe source and rejects a negative or
non-finite successful component before adding padding, border, scrollbar,
cache, or output state.

### 4.8 `D-08` Numeric Properties Use Distinct Wrappers

Scrollbar width, flex grow, and flex shrink each use a private-field newtype:

| Type | Valid range | Neutral default | Owner |
| --- | --- | --- | --- |
| `ScrollbarWidthOf<S>` | finite and `>= 0` | zero | layout |
| `FlexGrowOf<S>` | finite and `>= 0` | zero | layout |
| `FlexShrinkOf<S>` | finite and `>= 0` | one | layout |

They do not implement `Deref`, arithmetic traits, or an infallible conversion
from arbitrary scalar. Accessors return the scalar only after construction has
proved the invariant.

### 4.9 `D-09` Failures Are Classified, Typed, And Site-Aware

All tree-backed compute functions use `LayoutResultOf<Node, T, S, M>`. A
`LayoutErrorOf<Node, S, M>` records the original offending site, operation, and
one typed failure kind: invalid input, missing context, unsupported capability,
measurement failure, or internal invariant. A site can be one node or a
container/subject relationship. Deep child errors propagate unchanged.

Current report-only invalid states become errors. Existing domain types remain
where they carry useful detail, but they compose under the single compute error
envelope. Human prose is produced only by `Display`; control flow matches enums.

Public error detail enums are `#[non_exhaustive]` only where later FRI
initiatives are expected to add domain-specific detail. Closed construction
errors such as non-negative finite scalar rejection remain exhaustive.

### 4.10 `D-10` Rounding Is A Named Output Policy

The current `round(value + 0.5).floor()` behavior is named by an explicit
rounding mode such as `NearestCssPixel`. Public callers do not select
`Round` as an ambiguous verb. Rounding happens as a finalization step that
produces final node output entries in the completed batch while preserving
unrounded output for algorithms that need it during the same session.

## 5 FRI-01.5 Ownership And Phase Model

| Concept | Owner | Phase | Construction authority | Consumed by |
| --- | --- | --- | --- | --- |
| `LengthPercentageOf<S>` | layout | normalized symbolic value | layout constructors and root-owned lowering | layout value families and property algorithms |
| `PercentageBasisOf<S>` | layout | contextual resolution input | property algorithms with known basis facts | length-percentage resolution |
| `NumericResolutionOf<S>` | layout | contextual resolution outcome | `LengthPercentageOf::resolve_against` | property algorithms |
| `LayoutRootRequestOf<S>` | layout | public request | public validated constructors | root layout front door |
| crate-private algorithm input | layout | algorithm state | compute session only | block, flex, grid, lane, leaf algorithms |
| `CompletedLayoutBatchOf<Node, S>` | layout | output transaction | successful compute session only | root/root-adapter-owned application |
| `LeafMeasureInputOf<S>` | layout | provider request | leaf algorithm after box-inset normalization | measurement provider |
| numeric wrappers | layout | normalized property values | public constructors and `NodeInput` builders | algorithms |
| `LayoutErrorOf<Node, S, M>` | layout | diagnostic result | compute/session/domain boundaries | callers, tests, root integration |

## 6 FRI-01.6 Length-Percentage Contract

### 6.1 Public Shape

The public value API exposes semantic constructors and accessors, not fields:

```rust
pub struct LengthPercentageOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub enum PercentageBasisOf<S: LayoutScalar = DefaultScalar> {
    Missing,
    Definite(NonNegativeFiniteOf<S>),
}
pub enum NumericResolutionOf<S: LayoutScalar = DefaultScalar> {
    Resolved(S),
    MissingBasis { value: LengthPercentageOf<S> },
    InvalidNumeric {
        value: LengthPercentageOf<S>,
        basis: PercentageBasisOf<S>,
        resolved: S,
    },
}

impl<S: LayoutScalar> LengthPercentageOf<S> {
    pub const ZERO: Self;
    pub fn px(value: S) -> Result<Self, FiniteScalarErrorOf<S>>;
    pub fn from_percent_fraction(value: S) -> Result<Self, FiniteScalarErrorOf<S>>;
    pub fn from_coefficients(
        absolute_px: S,
        percent_fraction: S,
    ) -> Result<Self, LengthPercentageErrorOf<S>>;
    pub fn absolute_px(self) -> S;
    pub fn percent_fraction(self) -> S;
    pub fn depends_on_basis(self) -> bool;
    pub fn resolve_against(self, basis: PercentageBasisOf<S>) -> NumericResolutionOf<S>;
}
```

`from_coefficients(10px, 0.25)` represents `10px + 25%`. Subtraction and nested
authored calc expression normalization happen before layout receives the value;
root style lowering is responsible for producing the final coefficients.
`PercentageBasisOf::Definite` can only be constructed from a finite,
non-negative scalar or an already validated `NonNegativeFiniteOf<S>`.

### 6.2 Resolution Matrix

| Value | Basis | Resolution |
| --- | --- | --- |
| `0px + 0%` | missing or definite | `Resolved(0)` |
| `absolute + 0%` | missing or definite | `Resolved(absolute)` |
| `absolute + percent%` where percent is nonzero | definite finite non-negative basis | `Resolved(absolute + percent * basis)` if finite |
| `absolute + percent%` where percent is nonzero | missing | `MissingBasis { value }` |
| coefficient is non-finite | any | construction error |
| computed scalar is non-finite | definite basis | `InvalidNumeric { value, basis, resolved }`, mapped to `InvalidInput` by compute |

There is no `MissingResolver`, `MissingExpression`, `NonNumeric`, foreign-store,
or stale-ID state in the layout value model.

## 7 FRI-01.7 Compute Front Door And Session Contract

The public root entry point accepts a tree/read provider, root node, and
`LayoutRootRequestOf<S>`, then returns a completed batch:

```rust
pub fn compute_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree;
```

The exact trait names may follow local style during implementation, but the
contract is fixed:

- public compute receives immutable layout input and provider callbacks plus an
  explicit request;
- recursive algorithms receive crate-private algorithm inputs from one session;
- algorithms cannot call public constructors for run modes or write node state
  directly;
- hidden layout, intrinsic sizing, and child layout are session operations, not
  caller-selected public modes;
- viewport root and flex-item-under-viewport root contexts are represented by
  distinct validated request state;
- every invalid root request is rejected before traversal begins.

`compute_leaf` remains a public pure helper with the validated public leaf
measurement contract and
`LayoutResultOf<(), ComputeOutputOf<S>, S, M>`. It composes value-resolution
and leaf-measurement failures directly at the standalone site; callers do not
perform a later envelope-wrapping step. The helper must not install an implicit
no-calc context because no resolver context exists after this initiative.

## 8 FRI-01.8 Cache Contract

Cache storage owns complete outputs:

| Case | Required behavior |
| --- | --- |
| Cold compute-size output | Store the complete `ComputeOutputOf<S>`, including content size, scroll geometry, baselines, collapsible margins, and collapse-through flag. |
| Hit for the same private algorithm input | Return exactly the stored complete output. |
| Hidden layout or invalidated node | Stage a cache clear in the completed batch. |
| Error during compute | Drop staged cache changes with the rest of the session. |

`CacheKeyContext` remains a public zero-field unit value, and
`LayoutTree::cache_context()` returns it. It cannot carry resolver generation,
consumer identity, style revision, or any other caller-defined cache dimension.
The authoritative cache identity is the node plus the complete private
`ComputeInputOf<S>` projection: run mode, sizing mode, requested axis, known
size, parent size, containing layout context, available size, settled
auto-scrollbar state, containing auto-scrollbar pass, and the unit context.

`LayoutTree::cache_get(node, &input, context)` is the read-only cache query and
returns a complete `ComputeOutputOf<S>`. A container hit is reusable only when
the tree can also return each required committed unrounded descendant through
`unrounded_layout(node)`; absence prevents warm-subtree restoration rather than
inventing state.

Cache validity is a mandatory caller/tree boundary. `cache_get` may return an
entry only for the current committed node, subtree, topology, and provider
snapshot. A change to normalized `node_input`, `layout_input`, child structure,
or provider-owned measurement facts makes the affected node a dirty subject
before the next compute. The caller supplies those subjects to
`compute_layout_invalidated`; layout derives the inclusive ancestor closure,
bypasses stale reads for that closure, and stages its clears. `compute_layout`
is only the convenience entry point for a tree with no pending dirty subjects.

Cache stores and clears are emitted as typed entries in
`CompletedLayoutBatchOf`, prepared with all other output state, and committed
atomically by the caller's `LayoutBatchSink`. Hidden layout bypasses cache, and
a failed session publishes no cache mutation.

FRI-01 establishes the fail-closed extension rule for later output classes:
every state needed to reconstruct and publish a warm subtree must have a
committed readback path. FRI-06 applies that rule with
`LayoutTree::unrounded_inline_fragments`. `None` means absent committed state,
`Some(&[])` means committed empty state, and a warm inline-text node without its
required fragment state fails closed rather than fabricating fragments.

## 9 FRI-01.9 Measurement Contract

`LeafMeasureInputOf<S>` exposes:

- `known_content_size() -> Size<Option<S>>`;
- `available_content_size() -> Size<MeasurementAvailableOf<S>>`;
- accessors for writing mode and other layout-owned facts needed by the existing
  leaf callback surface.

`MeasurementAvailableOf<S>` is a closed enum with private definite construction:

| Variant | Meaning |
| --- | --- |
| `Definite(NonNegativeFiniteOf<S>)` | finite content-space limit greater than or equal to zero |
| `MinContent` | intrinsic min-content request |
| `MaxContent` | intrinsic max-content request |

The provider result is validated for finite non-negative width and height before
the algorithm adds padding, border, scrollbar, or cache state. A provider error
keeps the original `M` as `LeafMeasureErrorOf::Provider(M)`.

`compute_leaf` is the public standalone leaf front door:

```rust
pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    measure: impl FnMut(LeafMeasureInputOf<S>) -> Result<Size<S>, M>,
) -> LayoutResultOf<(), ComputeOutputOf<S>, S, M>
where
    S: LayoutScalar;
```

The internal leaf-measurement boundary produces
`LeafMeasureErrorOf::InvalidOutput` only for successful provider output whose
width or height fails `NonNegativeFiniteOf<S>` construction, then composes it as
`LayoutErrorKindOf::InvalidInput` at standalone site and
`LayoutOperation::LeafMeasurement`. A provider failure becomes
`LayoutErrorKindOf::Measurement(M)` at the same site and operation.
`InvalidMeasurementOutputOf<S>` exposes `axis() -> PhysicalAxis` and
`error() -> NonNegativeFiniteScalarErrorOf<S>` accessors. Value-resolution
failures occur before measurement and retain standalone site plus the
value-resolution operation. No failure exposes batch state or partial output.

## 10 FRI-01.10 Error Contract

`LayoutErrorOf<Node, S, M>` contains:

- `site: LayoutErrorSiteOf<Node>`;
- `operation: LayoutOperation`;
- `kind: LayoutErrorKindOf<S, M>`.

`LayoutOperation` distinguishes at least root layout, child layout, hidden
layout, leaf measurement, value resolution, cache access, rounding/finalization,
and domain-specific standalone operations such as grid-lane placement.

`LayoutErrorKindOf<S, M>` is a non-exhaustive typed payload enum:

| Kind | Required detail |
| --- | --- |
| `InvalidInput(LayoutInvalidInputOf<S>)` | `RootAvailability { axis, error }`, `MeasurementOutput(InvalidMeasurementOutputOf<S>)`, or `InvalidNumeric { value: S }` preserves the applicable typed scalar detail. |
| `MissingContext(LayoutMissingContext)` | `RequiredBasis` marks a consumed percentage whose operation has no valid indefinite behavior. |
| `UnsupportedCapability(LayoutUnsupportedCapability)` | `LaterFriBehavior` identifies represented behavior intentionally deferred by this initiative; later initiatives may add typed capability descriptors under the non-exhaustive enum. |
| `Measurement(M)` | Preserves the provider's original error value. |
| `InternalInvariant(LayoutInternalInvariant)` | Uses an exact layout-owned invariant such as invalid root scroll geometry, missing leaf provider, or missing staged unrounded output. |

`NumericResolutionOf::MissingBasis { value }` remains available while the
property algorithm decides whether indefinite behavior is valid. If it is not,
the envelope records `MissingContext::RequiredBasis` at the consuming site and
`ValueResolution` operation. `NumericResolutionOf::InvalidNumeric {
value, basis, resolved }` maps to
`LayoutInvalidInputOf::InvalidNumeric { value: resolved }` at that same site and
operation. This mapping intentionally retains the failing scalar and
classification rather than copying the affine value and basis into the final
compute envelope; `DIAG-001` requires callers to distinguish valid, unresolved,
unsupported, and invalid outcomes, not to recover the pre-consumption affine
expression. A rejected successful measurement maps to
`LayoutInvalidInputOf::MeasurementOutput`, preserving
`InvalidMeasurementOutputOf<S>` and therefore its physical axis and
`NonNegativeFiniteScalarErrorOf<S>`; provider `Err(M)` maps directly to
`Measurement(M)`.

There is no calc identity error because FRI-01 removes calc identity.

## 11 FRI-01.11 Domain Error Composition

Existing report or panic paths become typed errors inside the unified envelope:

| Area | Required composition |
| --- | --- |
| Root scroll geometry `expect` paths | Return `InternalInvariant` with root site and finalization operation. |
| Calc-bearing leaf, block, flex, grid, lane paths | Use length-percentage resolution and raise only real missing-basis or invalid-input errors. |
| Grid lane standalone APIs | Return `LayoutResultOf<(), T, S, Infallible>` or a domain alias preserving operation and invalid input. |
| Unsupported later-FRI capability | Return `UnsupportedCapability` only when the value is represented but intentionally unsupported in this initiative. |

Temporary unsupported errors do not close later behavior findings.

## 12 FRI-01.12 Public Surface Outline

| Module | Required change |
| --- | --- |
| `src/value.rs` or a focused value submodule | Add `LengthPercentageOf`, `PercentageBasisOf`, `NumericResolutionOf`; remove calc IDs, stores, generations, resolver traits, and no-calc sentinels. |
| `src/node_input.rs` | Replace raw calc-bearing variants and raw numeric fields along current construction paths; preserve later `FRI-04` sizing-family split as future work. |
| `src/output.rs` | Make recursive, root, child, and hidden algorithm input construction private or crate-private; retain only validated public `ComputeInputOf::leaf_layout` and `ComputeInputOf::leaf_content_size` construction for the standalone leaf helper; expose validated request/result/batch types. |
| `src/traits.rs` | Replace mutation-oriented compute hooks with read/provider/session contracts needed by public request computation. |
| `src/cache.rs` | Store and retrieve complete `ComputeOutputOf<S>` and stage cache changes in the completed batch. |
| `src/compute.rs` and algorithm modules | Thread session-owned private algorithm input and `LayoutResultOf`; remove panic and implicit no-calc paths. |
| `src/lib.rs` | Reexport only intentional public front-door types; remove obsolete resolver/generation/key exports. |
| `tests/layout/browser_parity/support.rs` | Consume validated public request contracts and affine calc values without local resolver or style/retained lowering. |

Removed public surface includes `CalcId`, `CalcGeneration`, `CalcResolver`,
`NoCalcResolver`, `LayoutCalcStore`, `CalcExpression`, `CalcTerm`,
resolver-free calc-capable resolution methods, and public field or
recursive/root/child/hidden construction of `ComputeInputOf`. The validated
direct-leaf constructors remain public.

## 13 FRI-01.13 Root Integration Handoff

Root later owns all integration changes. The handoff must state that root:

1. lowers authored style calc to `LengthPercentageOf<S>` coefficients;
2. stops constructing calc IDs, stores, generations, or resolver objects;
3. constructs layout validated numeric wrappers for scrollbar and flex factors;
4. uses `LayoutRootRequestOf<S>` instead of raw `ComputeInputOf<S>`;
5. applies or translates `CompletedLayoutBatchOf<Node, S>` through root-owned
   retained/facade state; and
6. refreshes root-owned API artifacts after the pushed layout candidate is
   visible.

Layout does not implement root adapters, root facade exports, or generated API
artifacts.

## 14 FRI-01.14 Dependency, Feature, Artifact, And MSRV Impact

No new third-party dependency is allowed or required. The default feature remains
the normal layout engine. Browser-parity and generator feature behavior remains
owned by the existing layout test infrastructure.

This leaf declares `rust-version = "1.97"` to match the user-selected project
MSRV. Verification records `rustc --version` and requires the active,
already-installed Rust 1.97 compiler before running the configured offline Cargo
gates. No additional toolchain acquisition is required or authorized.

Generated XML is not regenerated solely for API migration. Existing calc XML is
part of the verification surface. If generator execution reveals fixture-schema
drift caused by this contract, that becomes new evidence requiring the owning
plan to include the generator output.

Surgeist-owned Rust remains free of `unsafe`.

## 15 FRI-01.15 Documentation Contract

README and rustdoc update the public contract in layout-owned terms:

- layout consumes normalized layout-ready values and does not parse CSS;
- calc in layout is a finite px-plus-percentage value resolved against explicit
  basis context;
- root requests are validated public inputs; recursive run modes are internal;
- root computation returns a completed batch or typed error;
- measurement is content-space and non-negative;
- root owns adapters and API artifacts.

Examples must construct valid requests and match semantic error classes. They do
not show raw numeric field assignment, resolver traits, or calc stores.

## 16 FRI-01.16 Required Test Evidence

### 16.1 Calc And Resolution Tests

Focused tests cover:

- `px`, `percent_fraction`, mixed coefficients, subtraction-normalized negative
  percent, and zero canonicalization;
- rejection of non-finite coefficients, invalid basis construction, and
  non-finite resolved output;
- missing basis only when percentage coefficient is nonzero;
- `LengthOf`, `LengthAutoOf`, and `DimensionOf` construction and resolution
  without calc IDs or resolver traits;
- block, flex, grid, grid-lane, and measured-leaf calc paths through real layout
  front doors.

Tests prove `CORE-002` no longer has an implicit no-calc path and `CORE-003` no
longer has foreign-store or generation state.

### 16.2 Cache Tests

Tests prove compute-size cold and hit outputs are field-for-field equal for:

- content size;
- scroll geometry;
- first and last baselines;
- top and bottom collapsible margins;
- collapse-through flag; and
- scalar modes `f32` and `f64`.

Mutation tests require changed layout-ready node, subtree, topology, and
measurement-provider facts to enter the dirty-subject set before warm reuse.
The invalidated closure bypasses stale reads and stages matching clears. Later
output classes add their own cold/warm equality evidence; FRI-06 includes
inline-fragment descendants and distinguishes absent from committed-empty
fragment state.

### 16.3 Atomic Compute Tests

Tests prove that a failing root request or provider error returns no completed
batch and exposes no partial unrounded output, final output, cache store, or
cache clear. Successful computation produces one batch containing all expected
updates.

### 16.4 Measurement Tests

Tests prove:

- insets larger than known or available size floor content-space constraints at
  zero before invoking the provider;
- provider `Err(M)` is preserved;
- negative, infinite, and NaN provider dimensions are rejected;
- both scalar modes behave the same within established scalar tolerances.

### 16.5 Numeric Wrapper Tests

Tests cover construction, defaults, public node-input builders/defaults, and
algorithm consumption for scrollbar width, flex grow, and flex shrink. Raw
negative or non-finite construction through every public path is rejected.

### 16.6 Diagnostics Tests

Tests assert exact failure class, site, operation, and the typed detail retained
by the public envelope for:

- invalid root availability;
- invalid numeric property;
- required missing percentage basis;
- measurement provider error;
- invalid provider output;
- unsupported capability used as a temporary later-FRI boundary; and
- root finalization invariant failure.

No public input path owned by `FRI-01`, calc-bearing path, or invalid
measurement output may panic. Negative-margin overflow accumulation and small
scroll-container geometry remain `FRI-05` behavior closure; this initiative only
requires that any shared diagnostic substrate it introduces can carry those
future errors without preserving panic-only APIs.

### 16.7 Browser-Parity Tests

All four box-sizing/direction variants for each active calc fixture family must
reach comparison rather than panic:

- `block/block_calc_width_margin`;
- `flex/flex_calc_basis_margin_gap`; and
- `grid/grid_calc_track_and_item_margin`.

The fixture support uses the same public layout request and affine value
contract. No test-local resolver implementation or duplicated style/retained
lowering may exist in layout.

## 17 FRI-01.17 Verification Surface

Every implementation cycle derived from this specification records the focused
and complete evidence required by its cycle plan. Exact commands, feature
coverage, linting, formatting, diff checks, unsafe scanning, and review gates
are owned by that cycle plan and the repository command inventory rather than
copied into this specification.

## 18 FRI-01.18 Finding Closure Matrix

| Finding | Closure condition | Evidence |
| --- | --- | --- |
| `CORE-001` | Every cache entry stores complete `ComputeOutputOf<S>` and a hit is field-for-field equal. | Full-state cold/hit tests in both scalar modes. |
| `CORE-002` | Calc-bearing public paths use inline affine values and explicit basis resolution; no resolver-free panic path remains. | Unit/domain tests plus all active calc fixture variants reaching comparison. |
| `CORE-003` | Calc IDs, stores, generations, resolver identities, and foreign-store states are absent from source and public API. | Source/API search plus value-normalization tests. |
| `CORE-004` | Measurement receives typed non-negative finite content-space constraints and returns a provider-preserving typed result. | Below-inset, provider, invalid-output, and scalar tests. |
| `CORE-007` | Scrollbar width and flex factors are distinct validated newtypes on every construction path. | Constructor/default/root-handoff/API tests. |
| `DIAG-001` | Tree compute returns one site-aware scalar/provider-preserving result; current invalid/report/panic/silent paths compose into it; all layout-owned state is atomic. | Exact error matrix, failure-batch tests, and public API review. |

## 19 FRI-01.19 Initiative Acceptance

`FRI-01` is complete only when all of the following are true:

1. obsolete resolver, store, generation, calc ID, extensible/raw cache-context,
   per-node compute write, and infallible calc-capable APIs are absent from
   public surface and source; the remaining `CacheKeyContext` is a zero-field
   unit value;
2. `LengthPercentageOf<S>` and its resolution outcome obey the modeling guide
   owner, phase, invariant, construction, and context rules;
3. public root requests reject invalid intrinsic input and recursive algorithm
   states are not public construction surface;
4. root computation returns either a completed validated batch or one typed error
   without public partial state;
5. compute-size cache cold and hit outputs are semantically equivalent;
6. measurement inputs and outputs are finite, non-negative, content-space, and
   provider-error preserving;
7. scrollbar width, flex grow, and flex shrink are distinct validated property
   values throughout construction and algorithm consumption;
8. diagnostics preserve failure class, site, operation, and the typed detail
   defined in `FRI-01.10`, and no public-input panic or silent fallback remains
   in `FRI-01`-owned paths;
9. all three calc browser fixture families pass every checked-in direction and
   box-sizing variant that is not blocked by a later-FRI finding;
10. leaf documentation, public reexports, Rust 1.97 MSRV, feature behavior, and
    root handoff match the implemented contract; and
11. the applicable cycle-plan and repository verification contract is satisfied
    with clean, recorded evidence.
