# P01-I04 Property-Specific Sizing Values


Design owner: `surgeist-layout`

Specification ID: `FRI-04`

## 1 FRI-04.1 Authority And Outcome

This specification is the direct desired-state contract for `FRI-04` in
`plans/P01-layout/P01-index.md`. It owns
closure of `MODEL-005` and `MODEL-007` from
`plans/P01-layout/P01-initial-review-findings.md`.

The outcome is one layout-ready sizing model in which:

1. preferred size, minimum size, maximum size, flex basis, minimum track
   sizing, and maximum track sizing are distinct property domains;
2. box sizing and flex basis cannot contain a track-only fractional value, and
   a maximum size cannot contain the preferred/minimum-only `auto` state;
3. each property has its CSS initial value instead of sharing one accidental
   default;
4. finite affine length-percentage terms and unresolved `min()`, `max()`, and
   `clamp()` functions survive until the applicable percentage basis is known;
5. `fit-content(<length-percentage>)`, the `stretch`, `fit-content`, and
   `contain` keywords, `flex-basis: content`, and normalized `calc-size()` are
   representable only in the property domains that admit them;
6. track sizing retains track-only `fr`, validates its numeric factor, and
   consumes the same sizing-calculation substrate for numeric breadths and
   `fit-content()` limits;
7. every current algorithm branch consumes an explicit property state: a
   value is resolved correctly, routed to its existing semantic algorithm, or
   returned as a named later-owner capability result, never collapsed through
   generic `NonNumeric` control flow;
8. parser, browser, front-door, scalar, model, and documentation evidence
   proves the split without expanding generator architecture; and
9. root integration receives a finite breaking-API handoff without a leaf-side
   adapter, compatibility alias, or duplicate legacy path.

This is a breaking pre-release correction. Backward compatibility is not
required. Removed APIs are not retained through aliases, deprecated wrappers,
permissive conversions, duplicate fields, or a public legacy enum.

## 2 FRI-04.2 Scope And Non-Goals

### 2.1 Owned Scope

This specification owns:

- public property-specific sizing types and their default values;
- a private, iterative, structurally validated representation for unresolved
  sizing calculations;
- public construction of affine, `min()`, `max()`, and `clamp()` sizing
  calculations without parsing CSS strings;
- public normalized `calc-size()` calculation and property-specific basis
  types;
- non-negative used-value clamping for box and track sizing values;
- finite, non-negative track flex factors;
- removal of `Dimension`, `DimensionOf`, broad dimension-to-track conversions,
  and ordinary algorithm dependence on `LengthResolutionStatus::NonNumeric`;
- migration of `NodeInputOf` and every crate-owned construction and consumer;
- explicit flex `auto` versus `content` basis routing while preserving later
  `FLEX-004` ownership of intrinsic flex-basis algorithm completeness;
- property-specific XML fixture parsing and narrow helper/serializer support
  for the owned calculation syntax and values;
- exactly three active Surgeist browser sources for numeric sizing math, an
  explicit content flex basis, and track math;
- one final full ExistingPinned regeneration after all FRI-04 helper, parser,
  and fixture inputs settle, followed only by read-only corpus and parity
  checks;
- public reexports, crate documentation, focused tests, and root handoff; and
- explicit typed handoffs for format-specific contextual sizing behavior owned
  by later findings-resolution initiatives.

### 2.2 Explicit Non-Goals

This specification does not:

- parse authored CSS, run cascade, substitute variables, type-check CSS calc
  algebra, resolve relative units, or canonicalize arbitrary authored
  expressions; root and sibling owners lower computed style into the
  layout-ready constructors;
- create a general expression arena, expression identity, resolver callback,
  parser registry, serialization framework, or generator subsystem;
- expand generator architecture; generator changes are limited to preserving
  owned sizing tokens, serializing them with existing style attributes, adding
  three HTML fixtures, and regenerating their derived XML plus provenance;
- add a new generator command, report kind, filter, manifest mode, dependency,
  browser acquisition path, launch profile, or script;
- claim all CSS Values math functions. The layout-owned unresolved subset is
  finite affine sums plus `min()`, `max()`, and `clamp()` as named by the intake
  finding. Any other basis-independent computed math is lowered to a finite
  affine term upstream; a basis-dependent function outside this subset is not
  accepted as a layout-ready value until a later specification extends the
  closed calculation API;
- implement interpolation or animation of intrinsic sizing keywords;
- complete intrinsic flex-basis behavior owned by `FLEX-004`/`FRI-07`, grid
  algorithm completeness owned by `FRI-08`, inline/block formatting
  completeness owned by `FRI-06`, positioned sizing owned by `FRI-10`, or
  display/containment behavior owned by `FRI-12A` through `FRI-12F`;
- model natural object dimensions, text shaping, authored containment, style
  identity, rendering, or retained state;
- change margin, padding, border, inset, or gap to the new sizing-property
  types; those properties retain `LengthOf` or `LengthAutoOf`;
- edit root adapters, root facade exports, root API artifacts, the root gitlink,
  or sibling repositories;
- acquire software, change dependencies or feature flags, add `unsafe`, or
  change the crate's MSRV.

## 3 FRI-04.3 Standards And Current Evidence

### 3.1 Normative Evidence

CSS Sizing Level 3 gives preferred sizes and minimum sizes an initial value of
`auto`, gives maximum sizes an initial value of `none`, admits intrinsic
keywords and `fit-content(<length-percentage>)`, and defines the fit-content
formula as the clamp of the supplied limit between min-content and max-content:

- <https://drafts.csswg.org/css-sizing-3/#preferred-size-properties>
- <https://drafts.csswg.org/css-sizing-3/#min-size-properties>
- <https://drafts.csswg.org/css-sizing-3/#max-size-properties>
- <https://drafts.csswg.org/css-sizing-3/#valdef-width-fit-content-length-percentage>

CSS Sizing Level 4 adds `stretch`, bare `fit-content`, and `contain` to the
preferred, minimum, and maximum sizing properties. It defines bare
`fit-content` as `min(max-content, max(min-content, stretch))`, defines
stretch-fit fallback by property role when no definite stretch size exists,
and defines contain-fit in terms of a preferred aspect ratio and target
rectangle:

- <https://drafts.csswg.org/css-sizing-4/#sizing-values>
- <https://drafts.csswg.org/css-sizing-4/#stretch-fit-sizing>
- <https://drafts.csswg.org/css-sizing-4/#contain-fit-sizing>

CSS Flexbox defines `flex-basis` as `content | <'width'>`. `auto` first
retrieves the main-size property and becomes `content` only when that property
is also auto. `content` is a distinct content-based basis, and the width value
family carries its own intrinsic states:

- <https://drafts.csswg.org/css-flexbox/#flex-basis-property>

CSS Grid defines track breadths separately from box sizing, permits `<flex>`
only in maximum track breadths, and keeps track `fit-content()` distinct from
box fit-content sizing:

- <https://drafts.csswg.org/css-grid-2/#track-sizing>

CSS Values Level 4 defines `min()`, `max()`, and `clamp()`, permits `none` for a
clamp endpoint, and delays a property's range restriction until the calculated
value is consumed. An out-of-range branch is therefore not rejected merely
because an intermediate or unresolved result is negative:

- <https://drafts.csswg.org/css-values-4/#comp-func>
- <https://drafts.csswg.org/css-values-4/#calc-range>

CSS Values Level 5 defines `calc-size()` as a sizing basis plus a calculation.
The function behaves as its basis for layout, substitutes that basis's used
size for `size`, treats `any` as an unspecified definite basis, resolves
calculation percentages to zero when their basis is indefinite, and
canonicalizes nested or numeric bases so the layout-ready basis is only an
allowed keyword, `any`, or 100 percent:

- <https://drafts.csswg.org/css-values-5/#calc-size>
- <https://drafts.csswg.org/css-values-5/#calc-size-simplification>
- <https://drafts.csswg.org/css-values-5/#calc-size-resolution>

The Level 4 and Level 5 documents are current editor's drafts. FRI-04 models
their named sizing states because the verified finding requires those states;
it does not claim those exploratory drafts are a complete format-algorithm
specification or absorb the later initiative owners listed above.

### 3.2 Source Evidence At The Published Base

This table describes clean published commit
`299967928b9fa2877b3496bf83cf0954d455a32a`.

| Evidence ID | Current source fact | Required correction |
| --- | --- | --- |
| `E-BROAD-DIMENSION` | `DimensionOf<S>` in `src/value.rs` combines affine values, `Fr`, `Auto`, `MinContent`, and `MaxContent`. | Remove the public broad family and replace every consumer with its property type. |
| `E-NODE-FIELDS` | `NodeInputOf` uses `Size<DimensionOf<S>>` for preferred/min/max and `DimensionOf<S>` for flex basis. | Give each field its own type and CSS initial default. |
| `E-MAX-DEFAULT` | Both `NodeInput::DEFAULT` and generic `Default` initialize maximum size to `DimensionOf::AUTO`. | Initialize maximum size to an explicit `MaxSizeOf::NONE`. |
| `E-BOX-FR` | `DimensionOf::fr` is publicly constructable for every box and flex field. Its resolver returns `NonNumeric`, which ordinary algorithms turn into `None`. | Make `fr` constructable only through a validated track factor and remove silent box fallback. |
| `E-TRACK-CONVERSION` | Infallible `From<DimensionOf>` conversions map `Fr` to min-track `Auto`, max-track `Flex`, and a complete track pair. | Remove cross-property conversions and require explicit track construction. |
| `E-FLAT-CALC` | `LengthPercentageOf` stores only finite px and percentage coefficients. | Preserve basis-dependent `min`, `max`, and `clamp` structure in a sizing-only calculation value. |
| `E-NONNUMERIC` | Leaf, block, flex, grid, and track helpers use one broad resolver and interpret `NonNumeric` as zero or absent. | Match explicit property states; use numeric resolution only for numeric calculations. |
| `E-FLEX-BASIS` | Flex checks `DimensionOf::Auto`; every other unresolved basis follows one content/max-content-like fallback. | Distinguish `Auto`, `Content`, intrinsic keywords, and width-derived states before later algorithm routing. |
| `E-PARSER` | `support.rs` parses box, flex, and tracks through `parse_dimension_with_calc`; it accepts `fr` before converting to a track. | Add property-specific parsers and a track-only flex-factor path. |
| `E-HELPER` | `test_helper.js` and the Rust serializer preserve simple `calc()` but not sizing `min()`, `max()`, `clamp()`, `fit-content()`, `stretch`, `contain`, or `content`. | Preserve only the FRI-04-owned tokens needed by model tests and three active fixtures. |

The published browser corpus has 1,406 HTML sources, 5,268 XML outputs, 356
unsupported cases, zero expected failures, and one canonical generation report.
Root `surgeist` is clean at
`19590f6d9fa01c0df197c5ef07fb626c5cf18ced`; its committed layout gitlink is
`c0c6852610b835b60e46c680fbd1a4fb127d1d13`.

## 4 FRI-04.4 Resolved Design Decisions

### 4.1 `D-01` Property Domains Are Separate Closed Types

The public box and flex families are immutable wrappers with private semantic
representations:

```rust
pub struct PreferredSizeOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub struct MinSizeOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub struct MaxSizeOf<S: LayoutScalar = DefaultScalar> { /* private */ }
pub struct FlexBasisOf<S: LayoutScalar = DefaultScalar> { /* private */ }
```

Their default and keyword constants are:

| Type | Initial/default | Other direct keyword constants |
| --- | --- | --- |
| `PreferredSizeOf<S>` | `AUTO` | `MIN_CONTENT`, `MAX_CONTENT`, `STRETCH`, `FIT_CONTENT`, `CONTAIN` |
| `MinSizeOf<S>` | `AUTO` | `MIN_CONTENT`, `MAX_CONTENT`, `STRETCH`, `FIT_CONTENT`, `CONTAIN`, `ZERO` |
| `MaxSizeOf<S>` | `NONE` | `MIN_CONTENT`, `MAX_CONTENT`, `STRETCH`, `FIT_CONTENT`, `CONTAIN`, `ZERO` |
| `FlexBasisOf<S>` | `AUTO` | `CONTENT`, `MIN_CONTENT`, `MAX_CONTENT`, `STRETCH`, `FIT_CONTENT`, `CONTAIN`, `ZERO` |

Each family has `value(LengthPercentageOf<S>)`,
`calculation(SizingCalculationOf<S>)`,
`fit_content_function(SizingCalculationOf<S>)`, and its property-specific
`calc_size` constructor. `ZERO` is an infallible finite calculation. Public
`is_*` predicates expose semantic queries needed by integration without
exposing private variants. Crate-private pattern views drive exhaustive
algorithm matches.

`NodeInputOf<S>` becomes:

```rust
pub size: Size<PreferredSizeOf<S>>,
pub min_size: Size<MinSizeOf<S>>,
pub max_size: Size<MaxSizeOf<S>>,
pub flex_basis: FlexBasisOf<S>,
```

`Dimension` and `DimensionOf` are absent from `src/lib.rs` and production
source. There is no `fr` constructor on any box or flex type. There is no
`AUTO` on `MaxSizeOf`. There is no `CONTENT` on a box or track type.

Rejected alternative: one public enum plus per-property runtime validation
would leave illegal states constructable and preserve the broad-enum defect.

Rejected alternative: type aliases around a common enum do not prevent mixing
or invalid construction.

Rejected alternative: retaining `Dimension` as a compatibility alias leaves
root and fixture callers on the wrong contract and makes closure unprovable.

### 4.2 `D-02` Sizing Calculations Use An Iterative Validated Program

`SizingCalculationOf<S>` is a public immutable value with private storage. It
represents exactly:

- one finite `LengthPercentageOf<S>` leaf;
- `min()` over one or more calculations;
- `max()` over one or more calculations; or
- `clamp()` with a required preferred calculation and independently optional
  minimum and maximum calculations.

Its public construction API is:

```rust
pub fn value(value: LengthPercentageOf<S>) -> Self;
pub fn min(arguments: Vec<Self>) -> Result<Self, SizingCalculationError>;
pub fn max(arguments: Vec<Self>) -> Result<Self, SizingCalculationError>;
pub fn clamp(min: Option<Self>, preferred: Self, max: Option<Self>) -> Self;
pub fn depends_on_basis(&self) -> bool;
pub fn resolve_against(
    &self,
    basis: PercentageBasisOf<S>,
) -> LengthResolutionOf<S>;
```

`SizingCalculationError::EmptyArguments` is returned for an empty `min` or
`max`; malformed stack shapes are not public construction states. The private
representation is a flattened postfix instruction slice with validated arity.
Construction composes child programs iteratively, evaluation uses an explicit
value stack, and drop has no recursive ownership chain. Arbitrarily nested
valid input therefore does not consume the call stack during evaluation or
destruction.

Every leaf retains its own percentage dependency. Ordinary sizing calculation
resolution uses a deliberately syntactic missing-basis rule over the complete
non-negative basis domain `[0, +infinity)`: if any leaf has a nonzero percentage
coefficient and the basis is missing, the complete program returns
`MissingBasis` before numeric evaluation. It does not use branch dominance,
interval inference, or algebraic cancellation to resolve a coincidentally
constant result. A program whose every percentage coefficient is zero resolves
normally with a missing basis, including nested min/max/clamp programs.

For example, `min(10px, 20px)` resolves to 10px with a missing basis, while
`min(10px, 50%)` is `MissingBasis`; `max(10px, 20px)` resolves to 20px, while
`max(10px, 50%)` is `MissingBasis`; and `clamp(0px, 10px, 20px)` resolves to
10px, while `clamp(0px, 50%, 20px)` is `MissingBasis`. The same rule applies at
every nesting depth even when the known branches would dominate for all
non-negative bases.

A non-finite intermediate or final numeric value returns `InvalidNumeric`. The
calculation itself does not apply a non-negative range; the consuming sizing
property applies that range after the complete function resolves.

For `clamp(min, preferred, max)`, the minimum wins when the bounds conflict,
matching `max(minimum, min(preferred, maximum))`. A missing endpoint represents
CSS `none`. Signed zero is canonicalized by the existing scalar constructors.

Rejected alternative: a public recursive expression enum allows malformed and
unbounded recursive ownership, makes every internal operation an exhaustive
public commitment, and risks stack exhaustion during drop.

Rejected alternative: an arena/ID plus resolver callback reintroduces identity
and lifecycle into a normalized value that has no identity-bearing semantics.

Rejected alternative: resolving all functions upstream guesses the percentage
basis that the layout algorithm owns.

### 4.3 `D-03` `calc-size()` Arrives In Canonical Layout-Ready Form

`CalcSizeCalculationOf<S>` uses the same private iterative program shape, but
each affine leaf has three finite coefficients:

```text
absolute_px + percent_fraction * percentage_basis + size_fraction * basis_size
```

It provides `value(LengthPercentageOf<S>)`, `size()`, a fallible
`from_coefficients(absolute_px, percent_fraction, size_fraction)`, and the same
`min`, `max`, and `clamp` constructors. All coefficients are finite and signed
zero is canonical. `depends_on_size()` is exact.

Each property has a separate public basis enum:

```rust
pub enum PreferredSizeCalcBasis {
    Any, FullPercentage, Auto, MinContent, MaxContent,
    Stretch, FitContent, Contain,
}

pub enum MinSizeCalcBasis {
    Any, FullPercentage, Auto, MinContent, MaxContent,
    Stretch, FitContent, Contain,
}

pub enum MaxSizeCalcBasis {
    Any, FullPercentage, None, MinContent, MaxContent,
    Stretch, FitContent, Contain,
}

pub enum FlexBasisCalcBasis {
    Any, FullPercentage, Auto, Content, MinContent, MaxContent,
    Stretch, FitContent, Contain,
}
```

`FullPercentage` is the canonical 100-percent basis produced by upstream
calc-size simplification. A raw numeric basis, a nested calc-size basis, and a
fit-content function basis do not appear in the layout-ready API: upstream
normalization substitutes or canonicalizes them according to CSS Values Level
5 before construction.

Each property `calc_size(basis, calculation)` constructor returns
`CalcSizeConstructionError::SizeReferenceWithAnyBasis` if `basis` is `Any` and
the calculation depends on `size`. No other listed combination is invalid.
Property-specific basis enums make `None` unavailable to preferred/min/flex
and make `Content` unavailable outside flex basis.

At resolution, a calc-size value behaves as its basis for algorithm selection.
After that basis produces an original used size, the calculation replaces it,
with `size` bound to the original size. Calculation percentages use the normal
definite percentage basis and use zero when that basis is missing. The
`FullPercentage` basis itself retains ordinary percentage behavior, including
a property-specific auto/none/content fallback when 100 percent is indefinite;
its calculation percentages remain independently zero-on-missing.

`Any` is an unspecified definite basis. Its calculation does not receive a
basis size and, by construction, cannot require one. A resolved negative result
is clamped to zero by the consuming property after the complete calculation.

Rejected alternative: storing nested calc-size values makes equivalent
canonical forms differ and moves CSS substitution complexity into layout.

Rejected alternative: using one unrestricted basis enum makes invalid
cross-property keyword states constructable again.

### 4.4 `D-04` Track Breadths Own Track Flex And Sizing Math

`TrackFlexFactorOf<S>` is a public semantic scalar newtype. `try_new` accepts a
finite non-negative value, canonicalizes signed zero, and returns the existing
`NonNegativeFiniteScalarErrorOf<S>` for invalid input. `ZERO`, `get`, and
`Default` are available.

Track sizing becomes:

```rust
pub enum MinTrackSizingOf<S> {
    Calculation(SizingCalculationOf<S>),
    Auto,
    MinContent,
    MaxContent,
}

pub enum MaxTrackSizingOf<S> {
    Calculation(SizingCalculationOf<S>),
    Flex(TrackFlexFactorOf<S>),
    Auto,
    MinContent,
    MaxContent,
    FitContent(SizingCalculationOf<S>),
}
```

`TrackSizingOf<S>` keeps explicit `new`, `minmax`, and keyword constants. It
adds `calculation`, `flex(TrackFlexFactorOf<S>)`, and
`fit_content(SizingCalculationOf<S>)`. No raw-scalar `fr` constructor remains.
No `From<PreferredSizeOf>`, `From<FlexBasisOf>`, or legacy broad conversion is
implemented. Numeric convenience conversions, if retained, accept only
`SizingCalculationOf` or `LengthPercentageOf` and cannot carry a keyword.

Track numeric calculations resolve against the track percentage basis using
the same status semantics as box calculations. The existing track algorithm
continues to own intrinsic and flex distribution. A missing percentage basis
retains the existing CSS cyclic/intrinsic track behavior at the exact current
call sites; invalid numeric results remain computation errors.

`LengthOf::Normal` remains valid for gap and related non-sizing properties but
cannot enter a track breadth through the new API.

Rejected alternative: reusing `NonNegativeFiniteOf` for `fr` permits accidental
mixing of unrelated semantic scalars and obscures the track-only boundary.

### 4.5 `D-05` Resolution Has Property State, Numeric State, And Used State

FRI-04 distinguishes three phases:

1. a property value identifies the valid keyword, numeric calculation,
   fit-content function, or calc-size state;
2. numeric resolution evaluates the calculation against an explicit
   `PercentageBasisOf<S>` and returns resolved, missing-basis, or invalid-numeric
   status; and
3. the owning algorithm turns the resolved value or contextual keyword into a
   non-negative used size and applies box-sizing, intrinsic, margin-box, and
   formatting-context rules.

No property wrapper exposes a `resolve_optional` API that erases a keyword.
Crate-private resolution returns a tagged property request. Numeric requests
carry a calculation; `Auto`, `None`, `Content`, intrinsic, fit-content,
stretch, contain, and calc-size requests remain distinct until their owning
algorithm consumes them.

All resolved preferred/min/max/track numeric values are clamped to zero before
used-value comparison. A negative intermediate inside `min`, `max`, or `clamp`
is retained until the complete function resolves. Invalid non-finite numeric
results return `LayoutInvalidInputOf::InvalidNumeric`; they are never clamped,
converted to auto, or converted to zero.

Missing percentage context retains the property's defined behavior. It is not
globally an error: intrinsic/cyclic measurement may treat a percentage as auto,
content, zero, or indefinite at its existing specification-owned site. A path
that semantically requires a definite basis returns
`LayoutMissingContext::RequiredBasis`.

Rejected alternative: retaining `LengthResolutionStatus::NonNumeric` as the
ordinary dispatch mechanism erases which property keyword was supplied and
recreates both findings.

### 4.6 `D-06` Current Algorithms Must Dispatch Explicitly Without Stealing Later Findings

FRI-04 implements the common model and numeric behavior now and leaves named
format-algorithm findings with their existing owners. The typed later-owner
result extends `LayoutUnsupportedCapability` with this closed public contract:

```rust
pub enum SizingProperty {
    Preferred,
    Minimum,
    Maximum,
    FlexBasis,
}

pub enum SizingAlgorithm {
    Leaf,
    Block,
    Flex,
    Grid,
    GridLanes,
    Positioned,
}

pub enum CalcSizeBehaviorBasis {
    Auto,
    None,
    Content,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

pub enum SizingBehavior {
    MinContent,
    MaxContent,
    FitContentFunction,
    Stretch,
    FitContent,
    Contain,
    CalcSize(CalcSizeBehaviorBasis),
}

pub struct UnsupportedSizingBehavior {
    property: SizingProperty,
    behavior: SizingBehavior,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
}

pub enum LayoutUnsupportedCapability {
    LaterFriBehavior,
    SizingBehavior(UnsupportedSizingBehavior),
}
```

`UnsupportedSizingBehavior` is output-only: it has public `property`,
`behavior`, `algorithm`, and `axis` accessors but no public constructor. The
closed enums and struct derive `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, and
`PartialEq`. The node is already carried by `LayoutErrorSiteOf`.

`SizingAlgorithm` identifies the algorithm that needed the value. Root sizing
uses the root's actual leaf/inner-display algorithm; it does not create a
separate root variant. A size consumed by absolute-position sizing is
`Positioned` even when that box later establishes another inner formatting
context. Flex basis is consumed only by `Flex`. Every production capability
constructor is crate-private and is tested against the exact matrix below.

In the matrix, `S` means FRI-04 must return supported geometry or the ordinary
missing/invalid numeric result. `U06`, `U07`, `U08`, and `U10` mean it must
return `SizingBehavior` with the row's exact property/behavior, the column's
algorithm and physical axis, and later owner `FRI-06`, `FRI-07`, `FRI-08`, or
`FRI-10`, respectively.

| Property request | Leaf | Block | Flex | Grid | GridLanes | Positioned |
| --- | --- | --- | --- | --- | --- | --- |
| Preferred `Auto` | S | S | S | S | S | S |
| Preferred numeric calculation | S | S | S | S | S | S |
| Preferred `MinContent` | S | S | U07 | S | S | U10 |
| Preferred `MaxContent` | S | S | U07 | S | S | U10 |
| Preferred fit-content function | U06 | U06 | U07 | U08 | U08 | U10 |
| Preferred `Stretch` | U06 | U06 | U07 | U08 | U08 | U10 |
| Preferred bare `FitContent` | U06 | U06 | U07 | U08 | U08 | U10 |
| Preferred `Contain` | U06 | U06 | U07 | U08 | U08 | U10 |
| Minimum `Auto` | S | S | S | S | S | S |
| Minimum numeric calculation | S | S | S | S | S | S |
| Minimum `MinContent` or `MaxContent` | U06 | U06 | U07 | U08 | U08 | U10 |
| Minimum fit-content function | U06 | U06 | U07 | U08 | U08 | U10 |
| Minimum `Stretch`, bare `FitContent`, or `Contain` | U06 | U06 | U07 | U08 | U08 | U10 |
| Maximum `None` | S | S | S | S | S | S |
| Maximum numeric calculation | S | S | S | S | S | S |
| Maximum `MinContent` or `MaxContent` | U06 | U06 | U07 | U08 | U08 | U10 |
| Maximum fit-content function | U06 | U06 | U07 | U08 | U08 | U10 |
| Maximum `Stretch`, bare `FitContent`, or `Contain` | U06 | U06 | U07 | U08 | U08 | U10 |

Each grouped cell returns the exact `SizingBehavior` matching the supplied
member; grouping does not merge variants. Preferred `MinContent` and
`MaxContent` are FRI-04-supported only in the source-proven leaf, block, grid,
and grid-lanes intrinsic-availability routes. FRI-04 must add focused geometry
evidence for those routes and must not generalize that support to minimum or
maximum constraints.

Calc-size dispatch is exhaustive and separate:

| Calc-size basis | FRI-04 result |
| --- | --- |
| `Any` | Supported in every property/algorithm cell; resolve the calculation as a definite numeric value. |
| `FullPercentage` | Supported in every property/algorithm cell; use the property's existing 100-percent missing-basis rule, then apply the calculation when that basis produces a size. |
| Any keyword basis | Return `SizingBehavior::CalcSize` with the exact `CalcSizeBehaviorBasis`, property, consuming algorithm, and axis. The later owner is determined solely by the algorithm column above: leaf/block `FRI-06`, flex `FRI-07`, grid/grid-lanes `FRI-08`, positioned `FRI-10`. |

FRI-04 therefore does not claim keyword-basis calc-size used-value transforms;
it does make every basis representable and impossible to confuse with an
ordinary keyword or numeric calculation. `Any` and `FullPercentage` are not
`CalcSizeBehaviorBasis` variants because they are implemented by FRI-04 and
never appear in this capability result.

Flex basis has one consuming algorithm and this exact matrix:

| Flex-basis request | Flex result | Owner if unsupported |
| --- | --- | --- |
| `Auto` | Supported: consult preferred main size and use content only when that preferred value is auto. | None |
| `Content` | Supported: enter the existing content-based measurement path directly. | None |
| Numeric calculation, calc-size `Any`, or calc-size `FullPercentage` | Supported with the flex container's main-size percentage basis. | None |
| `MinContent` | `SizingBehavior(FlexBasis, MinContent, Flex, axis)` | `FRI-07` / `FLEX-004` |
| `MaxContent` | `SizingBehavior(FlexBasis, MaxContent, Flex, axis)` | `FRI-07` / `FLEX-004` |
| Fit-content function, `Stretch`, bare `FitContent`, or `Contain` | Exact matching direct `SizingBehavior` with property `FlexBasis`. | `FRI-07` |
| Any keyword-basis calc-size | Exact matching `SizingBehavior::CalcSize` with property `FlexBasis`. | `FRI-07` |

Track numeric calculations and fit-content limits resolve through the shared
calculation substrate inside existing track algorithms. Track intrinsic/flex
states remain distinct. Track breadths do not use this capability escape hatch
for FRI-04-owned numeric resolution; `FRI-08` retains ownership only of its
unrelated grid-completeness findings.

This capability is not a browser unsupported bucket or a closure claim for a
later behavior finding. No active FRI-04 fixture expects it. Later initiatives
remove exact matrix cells as they implement their owned algorithms; they do not
reinterpret or broaden the payload.

Rejected alternative: implementing `FLEX-004` inside FRI-04 duplicates the
findings index and entangles a model foundation with flex algorithm completion.

Rejected alternative: silently treating every contextual keyword as auto would
make the new type surface cosmetic and leave `MODEL-005` behaviorally unsafe.

### 4.7 `D-07` Property-Specific Fixture Lowering Is Narrow And Exact

The browser helper preserves a sizing token only when it belongs to the finite
FRI-04 fixture grammar:

- simple px and percentage values;
- affine `calc()` already accepted by the harness;
- nested `min()`, `max()`, and `clamp()` whose leaves use that affine grammar;
- `fit-content()` with one sizing calculation;
- `auto`, `none`, `content`, `min-content`, `max-content`, `stretch`, bare
  `fit-content`, and `contain`; and
- track-only non-negative finite `fr`.

The helper tags the preserved string as a sizing calculation/value; it does not
parse general CSS, infer unsupported syntax from used geometry, or make `fr`
available to box fields. The Rust serializer emits the exact normalized string
through the existing width/height/min/max/flex-basis/track attributes.

`support.rs` has separate parsers for preferred size, minimum size, maximum
size, flex basis, minimum track breadth, and maximum track breadth. Its sizing
calculation parser is balanced and depth-independent, validates function arity,
accepts only the finite fixture grammar, and constructs public values through
their checked APIs. It rejects empty argument lists, malformed delimiters,
non-finite coefficients, `size` outside calc-size, `size` with an `Any` basis,
`fr` outside maximum tracks, `none` outside maximum size, and `content` outside
flex basis.

This parser is a typed fixture adapter, not an app-facing CSS parser. It may use
structured recursive parsing internally, but it must lower into the iterative
production calculation value and impose a documented finite fixture nesting
limit before recursive parser descent.

## 5 FRI-04.5 Public Contract

The completed public front door includes and reexports default-scalar aliases
and generic forms for:

- `PreferredSize` / `PreferredSizeOf`;
- `MinSize` / `MinSizeOf`;
- `MaxSize` / `MaxSizeOf`;
- `FlexBasis` / `FlexBasisOf`;
- `SizingCalculation` / `SizingCalculationOf`;
- `CalcSizeCalculation` / `CalcSizeCalculationOf`;
- `SizingCalculationError`;
- `CalcSizeCalculationErrorOf` for invalid finite coefficients;
- `CalcSizeConstructionError`;
- `PreferredSizeCalcBasis`, `MinSizeCalcBasis`, `MaxSizeCalcBasis`, and
  `FlexBasisCalcBasis`;
- `TrackFlexFactor` / `TrackFlexFactorOf`;
- the revised minimum/maximum/combined track types; and
- `UnsupportedSizingBehavior`, `SizingProperty`, `SizingAlgorithm`,
  `SizingBehavior`, and `CalcSizeBehaviorBasis` through the existing layout
  error front door.

The following old public surface is absent:

- `Dimension` and `DimensionOf`;
- `DimensionOf::fr`, broad keyword variants, and broad resolution methods;
- `From<DimensionOf>` implementations for track types;
- raw scalar `MaxTrackSizingOf::fr` and `TrackSizingOf::fr`; and
- any property-erasing conversion among preferred, minimum, maximum, flex, and
  track sizing values.

The property wrappers and calculations derive `Clone`, `Debug`, and
`PartialEq`. They are intentionally not `Copy` because preserved calculations
own variable-length normalized programs. Their basis enums, calculation shape
error, and capability descriptors derive the ordinary copy/equality/hash/order
traits appropriate to scalar-independent closed choices.

No new trait, dependency, feature, allocator callback, resolver callback, or
runtime identity is public. Existing `LayoutScalar` remains sealed to `f32` and
`f64`; the calculation evaluator uses only its current finite arithmetic,
comparison, and conversion contract.

## 6 FRI-04.6 Construction And Resolution Matrices

### 6.1 Property Construction Matrix

| State | Preferred | Minimum | Maximum | Flex basis | Min track | Max track |
| --- | --- | --- | --- | --- | --- | --- |
| Numeric calculation | Yes | Yes | Yes | Yes | Yes | Yes |
| `auto` | Yes | Yes | No | Yes | Yes | Yes |
| `none` | No | No | Yes | No | No | No |
| `content` | No | No | No | Yes | No | No |
| `min-content` | Yes | Yes | Yes | Yes | Yes | Yes |
| `max-content` | Yes | Yes | Yes | Yes | Yes | Yes |
| `fit-content(<calculation>)` | Yes | Yes | Yes | Yes | No | Yes |
| bare `fit-content` | Yes | Yes | Yes | Yes | No | No |
| `stretch` | Yes | Yes | Yes | Yes | No | No |
| `contain` | Yes | Yes | Yes | Yes | No | No |
| `calc-size()` | Yes | Yes | Yes | Yes | No | No |
| `fr` | No | No | No | No | No | Yes |

### 6.2 Initial And Missing-Basis Matrix

| Property | Initial value | Definite numeric basis | Missing numeric basis |
| --- | --- | --- | --- |
| Preferred size | `Auto` | Resolve and clamp to zero | Use the formatting context's existing percentage-as-auto/indefinite rule or return required-basis when the path requires a definite value. |
| Minimum size | `Auto` | Resolve and clamp to zero | Preserve automatic minimum behavior; an explicit basis-dependent calculation remains distinguishable from `Auto`. |
| Maximum size | `None` | Resolve and clamp to zero | Preserve no-limit behavior only where CSS defines the unresolved percentage that way; do not rewrite the authored state to `None`. |
| Flex basis | `Auto` | Resolve against the flex container's inner main-size basis | An unresolved percentage becomes content only at the Flexbox-owned rule; `Content` remains separately observable. |
| Track breadth | `Auto` pair | Resolve against the grid container's applicable axis basis | Preserve existing intrinsic/cyclic track handling; invalid numeric remains an error. |

### 6.3 Calculation Status Matrix

| Condition | Result |
| --- | --- |
| All required coefficients and intermediates are finite and every required basis exists | `Resolved(value)`, followed by property range clamping |
| Every ordinary sizing leaf has a zero percentage coefficient | Resolve the complete nested program normally despite `PercentageBasisOf::Missing` |
| Any ordinary sizing leaf has a nonzero percentage coefficient | `MissingBasis` for the complete program, without branch-dominance or interval analysis |
| A calc-size calculation percentage has no definite basis | Evaluate that percentage contribution as zero |
| A calc-size `FullPercentage` basis has no definite basis | Preserve the underlying property's unresolved 100-percent behavior |
| Any arithmetic intermediate or final result is non-finite | `InvalidNumeric` with no fallback |
| Empty min/max construction | `SizingCalculationError::EmptyArguments` |
| Non-finite calc-size coefficient | `CalcSizeCalculationErrorOf` naming the invalid coefficient |
| `Any` calc-size basis plus a `size` reference | `CalcSizeConstructionError::SizeReferenceWithAnyBasis` |

## 7 FRI-04.7 Browser, Fixture, And Artifact Contract

### 7.1 Owned Browser Sources

The new active sources are:

- `block/fri04_sizing_math_functions`;
- `flex/fri04_flex_basis_content`; and
- `grid/fri04_track_math_functions`.

Each source generates the standard four box-sizing/direction variants, for 12
new XML outputs. The fixtures use only behavior FRI-04 implements directly:

- the block source composes affine `calc`, `min`, `max`, and `clamp` across
  preferred/min/max box sizing with a definite containing basis;
- the flex source distinguishes explicit `flex-basis: content` from an authored
  preferred main size; and
- the grid source uses sizing calculations in fixed min/max track breadths and
  a track `fit-content()` limit without changing the grid algorithm's later
  completeness boundary.

No active fixture expects a later-owner `SizingBehavior` capability result.
Model/parser tests cover the representable contextual keywords and calc-size
bases that are not yet parity claims.

### 7.2 One Final Full Regeneration

Scoped ExistingPinned runs are optional diagnostics during implementation. They
are not required, retained, cited, or counted as verification evidence.

After every FRI-04 HTML, helper, serializer, and fixture-parser input is settled,
one successful unfiltered full ExistingPinned run regenerates the corpus. No
second full run is performed over the same settled inputs. All subsequent
evidence is read-only: generator verification, corpus validation, full parity,
and ordinary crate checks. If a later fix changes an input to generation, the
previous full run is invalidated and one replacement full run is performed
after the corrected inputs settle.

The final corpus contains exactly 1,409 HTML sources and 5,280 XML outputs. The
full report contains 5,280 generated cases, 356 unsupported cases, and zero in
every failure class. It remains the sole report in
`xml/generation-reports/all.json`.

The unsupported digest has one canonical byte contract. Project each report
entry to `name`, `source`, `variant`, and `reason`; sort entries
lexicographically by `(name, source, variant, reason)`; serialize the array as
two-space pretty JSON with object keys in lexical order
`name, reason, source, variant`, LF line endings, and one final LF; then hash
those UTF-8 bytes with SHA-256. The published base and completed FRI-04 report
must both produce
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`.

A generator-feature test in
`tests/bin/surgeist-layout-generate/generator.rs` owns that projection and
digest assertion using the already-present `serde_json` and `sha2` feature
dependencies. `just verify-generator` executes it. The cycle plan does not
reconstruct the projection with an ad hoc shell pipeline.

The helper provenance hash changes on all 5,280 outputs. Source hashes change
only for new or intentionally edited HTML. XML geometry and style-input deltas
outside the 12 owned outputs require source-backed explanation; no XML is
hand-edited. The pinned Chrome remains `149.0.7827.115`, its repository-relative
cached executable and manifest launch profile remain unchanged, and no software
is acquired.

## 8 FRI-04.8 Focused Evidence

The implementation supplies at least:

- construction, equality, defaults, and keyword-domain tests for all four box
  and flex property families in both scalar lanes;
- compile-time/public-surface evidence that box/flex values have no `fr`, max
  size has no `auto`, non-flex values have no `content`, and legacy `Dimension`
  is not reexported;
- calculation tests for one/many-argument min/max, both/one/no clamp endpoint,
  conflicting clamp bounds, nested composition, missing and definite bases,
  negative final clamping, signed zero, f32/f64 overflow, invalid numeric, and
  deep iterative evaluation/drop;
- calc-size tests for keyword, `FullPercentage`, and `Any` bases; size and
  percentage coefficients; zero-on-missing calculation percentages; ordinary
  missing behavior for the 100-percent basis; nested min/max/clamp calculation;
  non-finite coefficients; and rejection of size with `Any`;
- default `NodeInputOf` tests proving preferred/min/flex `Auto` and maximum
  `None` for default and generic scalar inputs;
- exhaustive source tests proving every old broad resolver/conversion and
  `Dimension` reexport is absent;
- leaf, root, block, flex, grid, grid-lanes, and positioned/front-door tests for
  numeric min/max/clamp values, missing-basis rules, non-negative used ranges,
  and invalid numeric errors;
- table-driven tests covering every D-06 property/request/algorithm cell and
  asserting either supported behavior or the exact capability payload and later
  owner, including every grouped direct behavior and calc-size basis member;
- flex tests proving `Auto` consults preferred main size, explicit `Content`
  bypasses it, and min-content/max-content cannot silently share the same
  max-content fallback before `FRI-07`;
- track tests proving only a validated maximum breadth carries flex, a minimum
  breadth cannot carry flex, calculation resolution preserves cyclic handling,
  and fit-content calculation limits clamp correctly;
- parser tests accepting every property-valid state and rejecting each
  cross-property state, malformed function, empty function, excessive fixture
  depth, invalid finite value, and invalid calc-size basis/calculation pair;
- helper/serializer tests proving exact token preservation and that `fr` remains
  track-only;
- the generator-feature canonical unsupported-projection test proving all 356
  sorted tuples hash to the specified digest;
- exact nonignored inventory tests for the 12 owned generated outputs;
- full browser parity and corpus validation after the single final full
  regeneration;
- public docs/examples using property-specific constructors instead of legacy
  dimensions; and
- no `unsafe`, dependency, feature, MSRV, generator architecture, ignored-test,
  or expected-failure expansion.

## 9 FRI-04.9 Module And Implementation Outline

| Area | Required result |
| --- | --- |
| `src/value.rs` or a focused private sizing module | Add iterative sizing/calc-size programs, property wrappers and basis enums, track flex factor, revised track values, numeric resolution, errors, and focused scalar/model tests; remove broad dimension types and conversions. |
| `src/node_input.rs` | Replace the four broad fields and correct both default paths. |
| `src/compute.rs` | Replace broad leaf/root resolution with property requests, add the named sizing capability payload, and preserve invalid/missing error mapping. |
| `src/block.rs`, `src/flex.rs`, `src/grid.rs` and grid modules | Consume explicit property types, shared numeric calculation resolution, property defaults, content basis, and exact later-owner capability branches. |
| `src/lib.rs` and crate docs | Reexport the new front door, remove legacy dimensions, and document the layout-ready versus authored boundary. |
| Existing unit/contract/scalar tests | Migrate constructors by property role and add front-door behavior and invalid-state evidence. |
| `tests/layout/browser_parity/support.rs` | Add bounded property-specific fixture parsing and exact invalid-state diagnostics. |
| `tests/layout/browser_parity/scripts/gentest/test_helper.js` | Preserve only the owned sizing tokens/functions. |
| `tests/bin/surgeist-layout-generate/generator.rs` | Serialize the finite owned token representation through existing attributes and test it. |
| `tests/layout/browser_parity/html/` and generated `xml/` | Add three sources and derive exactly 12 outputs plus updated provenance through one final full run. |
| `README.md` and browser parity README | Replace dimension examples and describe the finite layout-ready calculation/fixture grammar without presenting the harness as a CSS parser. |

The calculation evaluator may live in `src/value.rs` or one new private
`src/sizing.rs` module. That file placement is reversible and does not change
the selected public model. No other new production module or artifact class is
authorized.

## 10 FRI-04.10 Root Integration Handoff

Root `surgeist` later owns all integration changes. The leaf candidate report
must tell root to:

1. replace every `Dimension`/`DimensionOf` import and construction with the
   property type matching the destination field;
2. lower computed preferred, min, max, flex-basis, and track values through the
   checked property-specific APIs;
3. canonicalize supported authored calc algebra to finite affine leaves plus
   the FRI-04 min/max/clamp program without resolving layout-owned percentages;
4. simplify nested/numeric calc-size bases to the property-specific keyword,
   `Any`, or `FullPercentage` basis and construct the checked calc-size
   calculation;
5. convert computed `fr` only through `TrackFlexFactorOf::try_new` and only for
   a maximum track breadth;
6. map style/cascade invalidity before layout rather than attempting an invalid
   leaf construction;
7. update facade reexports, adapters, examples, and root tests;
8. regenerate root-owned API audit artifacts only after the published leaf SHA
   is pinned; and
9. retain the leaf's named later-owner sizing capability until the corresponding
   format initiative implements that behavior.

The leaf does not provide a CSS parser, legacy conversion shim, root adapter,
or API artifact. Root must not resolve percentages early, reconstruct private
calculation instructions, or translate an invalid cross-property state into an
automatic value.

## 11 FRI-04.11 Durable Sequence Seams

The implementation sequence derives cycle boundaries from these dependency
facts:

1. the calculation and property model can be added and tested before replacing
   the broad public fields;
2. removal of `DimensionOf` requires one coherent crate-wide construction and
   consumer migration, including fixture parsing, so no compatibility alias is
   needed between cycles;
3. numeric calculation consumption depends on the migrated property fields;
4. calc-size and contextual capability routing depend on explicit property
   dispatch rather than broad resolution;
5. helper/serializer/HTML changes and generated artifacts occur only after all
   parser and production inputs settle, enabling one final full regeneration;
6. public evidence and docs close only after the final generated inventory and
   behavior are stable.

These are durable dependency boundaries, not executable task plans. The
implementation sequence owns cycle IDs and order; only the next ready cycle has
a just-in-time cycle plan.

## 12 FRI-04.12 Initiative Acceptance

FRI-04 is complete only when:

1. `MODEL-005` and `MODEL-007` map to this specification and to concrete tests,
   source, generated artifacts, and root handoff evidence;
2. preferred, minimum, maximum, flex-basis, minimum-track, and maximum-track
   values are distinct public domains with the construction matrix above;
3. `Dimension`/`DimensionOf`, broad dimension resolution, dangerous
   dimension-to-track conversions, and box/flex `fr` construction are absent;
4. `NodeInputOf` defaults to preferred/min/flex `Auto` and maximum `None` in
   both default-scalar and generic paths;
5. affine, min, max, and clamp sizing calculations apply the exact syntactic
   missing-basis rule at every nesting depth and report invalid states without
   guessing;
6. normalized calc-size values enforce property-specific bases, reject size
   with `Any`, and apply basis, percentage, and used-range semantics exactly;
7. track flex factors are finite, non-negative, and track-only, while numeric
   track breadths and fit-content limits use the shared calculation substrate;
8. every production sizing consumer matches explicit property states and no
   valid keyword travels through `NonNumeric` or an auto-like silent fallback;
9. all FRI-04-owned numeric/content behavior is implemented, every D-06 cell is
   covered, and every format-specific behavior still owned by a later initiative
   returns the exact property/behavior/algorithm/axis capability payload rather
   than incorrect geometry;
10. property-specific parser/helper/serializer tests and the three browser
    sources prove the active supported surface without importing CSS parsing;
11. exactly one final full regeneration over settled inputs produces 1,409 HTML,
    5,280 XML, 356 unsupported tuples with the canonical specified digest, zero
    failures, current provenance, and only `all.json`;
12. all configured default, generator, corpus, parity, scalar, formatting, and
    lint checks pass with no ignored or expected-failure expansion;
13. source, docs, public reexports, and the complete candidate handoff agree;
14. no dependency, feature, MSRV, unsafe, generator-architecture, root, sibling,
    or unrelated change enters the initiative.
