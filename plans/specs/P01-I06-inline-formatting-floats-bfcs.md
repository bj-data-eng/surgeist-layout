# P01-I06 Inline Formatting, Floats, And BFCs


Design owner: `surgeist-layout`

## 1 FRI-06.1 Authority And Outcome

This specification is the authoritative desired-state contract for `FRI-06` in
`plans/P01-layout/P01-index.md`. It owns
the layout-ready inline participant boundary, mixed inline line construction,
line-box geometry, float exclusion, and currently representable block-formatting-
context behavior needed to close these 14 findings from
`plans/P01-layout/P01-initial-review-findings.md`:

- `BLOCK-014`, `FLOW-002`, `BLOCK-004`, `FLOW-001`, and `FLOW-003`;
- `BLOCK-005`, `BLOCK-006`, `BLOCK-008`, `BLOCK-009`, `BLOCK-011`,
  `BLOCK-012`, and `BLOCK-013`; and
- `TEST-003` and `TEST-004`.

The outcome is a scalar-generic inline formatting context that consumes
validated pre-line-layout text facts plus atomic boxes and controls, constructs
horizontal and vertical line boxes against current float exclusion bands,
publishes every participant and text fragment in physical coordinates, and
handles the current float/BFC surface without panic, overlap, silent omission,
or phase-unsafe fallback.

This is an intentional breaking pre-release correction at crate version
`0.1.0`. Backward compatibility is not required. Removed or expanded public
states are not hidden behind aliases, deprecated wrappers, duplicate fields,
catch-all compatibility variants, or guessed defaults.

This specification supersedes the reviewed snapshot's stale implementation
locations and capability claims where FRI-02 through FRI-05 already changed
source. The finding IDs and required observable closure remain authoritative.

## 2 FRI-06.2 Ownership And Non-Goals

`surgeist-layout` owns:

1. validated layout-ready shaped inline facts after shaping, bidi analysis,
   whitespace classification, and break-opportunity discovery;
2. line construction, break selection, per-line alignment, struts, line-relative
   baseline placement, mixed text/atomic/control participation, and physical
   participant output;
3. float placement, clear, rectangular exclusion, provider-backed non-rectangular
   exclusion queries, BFC avoidance, and float contribution to auto size;
4. cache invalidation and committed-fragment restoration for every layout-ready
   inline and exclusion fact; and
5. focused tests, constrained browser fixtures, parity lowering, comparison,
   generated expectations, and public documentation for this boundary.

Root `surgeist` owns cross-crate composition. It associates retained text and
box nodes, invokes the text and shape owners, lowers their validated outputs to
this crate, maps returned fragment/source identities back to retained content,
and refreshes root-owned API artifacts from compatible source revisions.

`surgeist-text` remains the owner of fonts, shaping, glyphs, clusters, bidi
analysis, whitespace processing, language-sensitive break opportunities,
hyphenation candidates, text source identity/revision, selection, cursor
movement, and render-ready glyph data. Its root adapter supplies the finite
pre-line-layout facts defined here. Layout does not accept text, fonts, glyph
buffers, locale, Unicode properties, or a text backend.

`surgeist-shape` remains the owner of normalized path and primitive geometry.
Root supplies a shape-backed implementation of layout's bounded exclusion-query
contract. Layout does not depend on `surgeist-shape`, retain a shape object, or
parse `shape-outside`.

Authored CSS, cascade, computed style, box generation, anonymous box creation,
DOM normalization, text shaping, painting, hit testing, selection, accessibility,
live scrolling, and root facade composition remain outside this leaf.

The following are not FRI-06 outcomes:

- the full `vertical-align`/baseline vocabulary and text justification owned by
  `FRI-09`;
- inline-sequence absolute/fixed/sticky hypothetical positioning owned by
  `FRI-10`;
- fragmentainers, multi-column layout, line clamping, and block ellipsis owned
  by `FRI-11`;
- `flow-root`, inline-flex, list markers, tables, ruby, containment, or display
  normalization owned by `FRI-12A` through `FRI-12F`;
- the aggregate corpus/WPT release gate owned by `FRI-13`; or
- generator architecture, a new generator, a dependency, or a reusable parser.

FRI-06 does not claim non-rectangular exclusion until a caller supplies the
typed provider result. Margin-box exclusion remains the real default. A missing
required provider result is an error, never a rectangular approximation.

## 3 FRI-06.3 Current Evidence

At canonical adaptation base `ed246a31d8af7957e5592c27e111345e86479fe6`,
the public and algorithmic FRI-06 contract is already present:

- `WritingMode` and `FlowAxes` cover five writing modes and ten
  writing-mode/direction mappings;
- typed shaped text, atomic participation, controls, fragment output, non-box
  pairing, float exclusion, and `VerticalAlign::Bottom` are public and reexported;
- one logical line algorithm handles horizontal, vertical, bidi, break, per-line
  alignment, float-band, BFC, baseline, percentage, and fast-path cases;
- `compute_layout_invalidated`, exact ancestor closure, phase-specific fragment
  storage, `LayoutBatchSink`, and warm fragment restoration implement the
  transactional cache/output contract;
- the comparator checks explicit break/fragment geometry and uses the reviewed
  closed directional interval relation; and
- the checked-in unfiltered report has 5,712 generated and 16 unsupported rows,
  zero quarantine/failure/expected-fail rows, and SHA-256
  `f46d8d8b50c722037127fdca79679649bd5cfd6db16fb24c0d69a7e5a082147a`.

The remaining source gap is confined to `D-18`. Current subgrid sizing still
reduces and republishes a fully inherited child-local baseline group instead of
placing flattened descendants directly in the ancestor group. Public geometry
therefore remains `(57, 125)` rather than browser `(62, 110)` for the nested
block control, `459` rather than `411` for auto rows, `415/570` rather than
`470/527` for inline columns, `202` rather than `196` for vertical auto rows,
and `168` rather than `153` for vertical nesting. The checked-in report records
144 passing and 244 failing activation rows; 96 stale browser-control rows are
expected to reach geometry only after the single settled replacement lineage.

The cycle base `8ffb4bc551a24d2283ad54436870ab3f5e66a473` remains the
immutable pre-activation artifact baseline: its report has 5,324 generated and
356 unsupported rows at SHA-256
`4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`.
The old finding claim that sideways writing modes and parent formatting roles
are absent is also no longer true; FRI-06 consumes those completed contracts.

## 4 FRI-06.4 Resolved Design Decisions

| ID | Decision |
| --- | --- |
| `D-01` | Text crosses into layout as validated pre-line-layout `InlineTextInputOf<S>` metric facts. Layout never calls a font or shaping backend and never stores authored text, glyph data, ink bounds, or a glyph/content offset. |
| `D-02` | A text input contains a nonempty ordered list of indivisible shaped segments with caller-local IDs, finite logical metrics, bidi level, whitespace edge behavior, and the break opportunity following each segment. An atomic inline box carries the corresponding bidi level and following break fact in a separate validated participation value. |
| `D-03` | The root text adapter supplies text segments and atomic participation facts from one composed paragraph in logical source order. Layout flattens tree children in source order, chooses line breaks, then performs stable per-line visual reordering over shaped segments, atomic boxes, and inline-boundary markers. Visible line breaks terminate before reordering and floats/out-of-flow boxes never enter the sequence. |
| `D-04` | Line fragments are published separately from `NodeOutputOf`: a completed batch owns immutable `InlineFragmentOutputEntryOf<Node,S>` values keyed by source node and segment ID. Their rect and baseline are metric line-fragment geometry derived from the supplied inline extent, baseline, and line extent, never glyph-ink bounds. A text node's ordinary `NodeOutputOf` is the physical union of those fragments and is not treated as a CSS box. |
| `D-05` | `InlineTextInputOf` is an owned layout-ready value with private fields. It has no `Default`; empty text produces no layout input during root box generation. Construction validates all numeric, ordering, bidi, whitespace, and break invariants atomically. |
| `D-06` | The line builder is one logical-axis algorithm for all ten flow mappings. Horizontal and vertical physical output differ only through `FlowAxes` projection; the separate forced-column implementation is removed. |
| `D-07` | Every line independently owns available inline start/end, used inline extent, block extent, baseline, participant list, alignment offset, and float-band provenance. No maximum-line proxy is reused for placement. |
| `D-08` | The supported FRI-06 alignment surface is legacy left/right/center plus start-aligned default. It applies per line. Text justification and the remaining alignment model stay typed later-owned capability, never an approximate center/start fallback. |
| `D-09` | Baseline, line-over (`top`), and line-under (`bottom`) are distinct algorithm states. FRI-06 includes `VerticalAlign::Bottom`; all other values remain FRI-09-owned and unrepresented rather than collapsed. |
| `D-10` | A forced break commits its current line and seeds the following line with the containing strut metrics. A break/control can establish first and last baselines even when no atomic/text participant exists. |
| `D-11` | Atomic inline fallback uses the block-end margin edge when no usable inner baseline exists. A non-visible used overflow forces fallback; visible used overflow may use the inner baseline. Top/bottom alignment is resolved after baseline line sizing. |
| `D-12` | Atomic inline percentage block size receives the containing block's definite physical/logical block basis when present. Anonymous inline-run size is never substituted as that basis. |
| `D-13` | Float left/right and clear left/right are line-relative values mapped by the containing `FlowAxes`; the public enum spellings remain source-compatible while algorithms do not treat them as physical x sides. |
| `D-14` | Margin-box float exclusion is internal and always available. Non-rectangular exclusion uses an explicit `FloatExclusion::Shape` input and a bounded `LayoutTree` provider query. Each returned interval retains its originating query privately; a mismatched query, missing provider, or provider failure is a typed layout error. |
| `D-15` | Float interaction is closed over the current model. An in-flow, non-floating, block-level child avoids active floats exactly when it is `Flex`, `Grid`, or `GridLanes`, or when it is non-replaced and its normalized computed overflow pair establishes an independent formatting context. Floats use the float path, atomic inline boxes use the line path while trapping their own internal formatting context, absolute boxes are excluded, and `None` produces no box. Future display roles do not enter this cycle. |
| `D-16` | Browser fixtures remain a finite adapter. FRI-06 activates the exact 340 initially unsupported variants identified below and includes exactly twelve named four-variant sources. Parser/helper/generator/comparator edits are permitted only for their shaped-segment/fragment, browser-observation category, finite anonymous/inline lowering, control, and exclusion facts. Intermediate diagnostics may synthesize bounded layout-ready facts, but final acceptance serializes those facts explicitly or derives them through generic input-only rules: fixture source/name and expected geometry never select, create, or alter layout input. The layout-ready-inline opt-in supplies level zero unless an exact source-indexed marker supplies another bidi level. A marker may explicitly scope its applicability to one computed `ltr` or `rtl` variant; direction selects that authored record but never derives its level. Pinned Chrome is the default geometry oracle. An exact row may instead be a visible known Chrome measurement failure only under the certainty, evidence-record, synthetic-substitute, and revalidation contract in `FRI-06.11`; disagreement with layout alone never qualifies. Inputs settle first, then one full regeneration owns all XML/report deltas. |
| `D-17` | Superseded by `D-18`. It correctly separated scalar size, group membership, and parent envelope phases, but its publication-as-an-exact-inverse premise still required reconstructing ancestor transform history from a child-local scalar. Three bounded corrections could satisfy the nested coordinates or the intrinsic and round-trip controls, but not all of them together. |
| `D-18` | Final subgrid-baseline controls use one axis-parametric flattened-membership model. In a fully inherited axis, the subgrid root is empty for track sizing and baseline grouping; its participating descendants enter the ancestor's group directly with the accumulated margin/border/padding and half-gutter adjustments already retained by subgrid traversal. A child physical baseline converts once to a typed ancestor-track logical distance. Scalar intrinsic contribution, ancestor baseline membership, reduced ancestor first/last group, and non-publishable child envelope view are distinct phases. The immutable ancestor group is reduced once, then sliced and mapped downward for child alignment; a child view is never republished upward and no inherited-axis fixed-point loop or publication inverse exists. Non-inherited axes retain ordinary local grid-container baseline behavior. Refreshed area sizing and the final physical projection remain owned by the containing grid's `FlowAxes`. |
| `D-19` | Browser control observations and model control geometry remain distinct evidence. The browser may use its non-model `<br>` rectangle only to report source/slot/neighboring-line effects; layout publishes the specification-required zero-size aligned control. When that model point lies within tolerance of the exact shared endpoint of both adjacent model neighbor intervals, closed overlap makes both geometric relations `Same` and the browser's categorical relation is not comparable to model geometry. The comparator records that one field as endpoint-unobservable instead of equating the two meanings. Exact neighboring node geometry remains strict, the browser observation remains serialized, and a private line-builder regression proves that the forced break commits between the two model participants. This generic predicate reads no fixture identity or expectation to create layout input, adds no public line-identity surface, and is not a Chrome failure, expected-fail, or synthetic geometry substitute. |

Rejected alternatives:

- Calling `surgeist-text` or `surgeist-shape` directly from this crate would add
  sibling dependencies and violate root adapter ownership.
- Treating a whole text paragraph as one measured leaf cannot compose mixed
  atomic boxes, forced controls, per-line float bands, or per-segment output.
- Passing render glyphs into layout would couple geometry to a backend phase and
  duplicate text ownership.
- Keeping separate horizontal and vertical line algorithms would preserve the
  exact axis divergence FRI-02 made `FlowAxes` responsible for removing.
- Falling back from a missing shape provider to the margin box would silently
  change valid requested geometry.
- Adding the full vertical-alignment vocabulary here would duplicate FRI-09's
  cross-format alignment ownership.
- Rounding an honest fractional line envelope, re-adding a flattened subgrid's
  complete margin box, or repeatedly adjusting an untagged inherited baseline
  would hide an input or phase error rather than model browser layout.
- Publishing a subgrid's reduced child-local baseline group back into a fully
  inherited axis would model the subgrid root as a group member and require an
  inverse that cannot recover accumulated ancestor-edge provenance. CSS Grid 2
  instead makes the descendants participate directly with those adjustments.
- Moving a model control to Chrome's `<br>` rectangle, treating touching closed
  intervals as ordered, or reconstructing private line layout in the fixture
  parser would corrupt one of the two evidence domains rather than compare them.

## 5 FRI-06.5 Public Model

### 5.1 Shaped Segments

The public scalar-generic input contract contains these types and default-scalar
aliases:

```rust
pub struct InlineSegmentId { /* private value */ }

pub struct BidiLevel { /* private value */ }

pub enum InlineWhitespaceEdge {
    Preserve,
    DiscardAtLineStart,
    DiscardAtLineEnd,
    DiscardAtBoth,
}

pub enum InlineBreakKind {
    Prohibited,
    Allowed,
    AllowedWithReplacement,
    Mandatory,
}

pub struct InlineBreakOpportunityOf<S: LayoutScalar> { /* private */ }

pub struct ShapedInlineSegmentOf<S: LayoutScalar> { /* private */ }
pub struct InlineTextInputOf<S: LayoutScalar> { /* private */ }
pub struct AtomicInlineParticipationOf<S: LayoutScalar> { /* private */ }
```

`InlineSegmentId::new` and `get` are the only construction/access path. The ID is
unique within one `InlineTextInputOf`, stable only for that input value, and used
to associate output with the caller's shaped source. It is not a retained node,
text source, glyph, cluster, byte-range, or global identity.

`BidiLevel::try_new` accepts Unicode bidi embedding levels `0..=125`. It exposes
`get()` and `is_rtl()` only. Layout uses the standard level-based line reordering
on the complete final selected line-unit slice defined in FRI-06.7; it does not
infer bidi classes or paragraph direction.

`InlineBreakOpportunityOf` has private fields. `prohibited`, `allowed`, and
`mandatory` construct the payload-free states.
`try_allowed_with_replacement` accepts only a finite non-negative extent.
`kind()` returns `InlineBreakKind`; `replacement_inline_extent()` returns
`Some(extent)` exactly for `AllowedWithReplacement`. It has no `Default`, public
field, enum payload, unchecked constructor, or conversion from a scalar.

`ShapedInlineSegmentOf::try_new` receives:

- `InlineSegmentId`;
- finite non-negative logical inline extent;
- validated `InlineMetricsOf<S>` for line-over/baseline/line-under contribution;
- `BidiLevel`;
- one `InlineWhitespaceEdge` classification; and
- one following `InlineBreakOpportunityOf<S>`.

An allowed replacement extent is finite and non-negative. It contributes only
when the break is selected and belongs to the line before the break. A segment
with `AllowedWithReplacement` must use `InlineWhitespaceEdge::Preserve`;
`ShapedInlineSegmentOf::try_new` rejects every replacement/discard combination
because a discarded source segment has no fragment on which to publish the
replacement. A mandatory opportunity commits a line after that segment and
seeds the following containing strut, including a preserved final newline. Any
opportunity is valid on the last segment: ordinary input termination commits the
final line without selecting a prohibited/allowed break or inventing a
replacement.

`InlineTextInputOf::try_new(Vec<ShapedInlineSegmentOf<S>>)` rejects empty input,
duplicate IDs, non-finite metrics, invalid levels, replacement/discard pairs,
and structurally impossible break state. It exposes a read-only slice and
implements `Clone`, `Debug`, and `PartialEq`; it does not implement `Copy` or
`Default`.

`AtomicInlineParticipationOf::try_new` stores one `BidiLevel` and the break
opportunity following that atomic box. `Prohibited`, `Allowed`, and `Mandatory`
are valid; `AllowedWithReplacement` is rejected because replacement glyphs are
owned by a shaped text segment before its boundary, never by an indivisible
atomic placeholder. It is layout-ready output of the composed text adapter, not
authored box style. `NodeInputOf<S>` carries
`Option<AtomicInlineParticipationOf<S>>`; its default is `None`. A participating
atomic inline display requires `Some`, while a non-atomic box requires `None`,
and root-request validation rejects either mismatch before layout. There is no
guessed bidi level or break default. Rust-authored callers use
`AtomicInlineParticipationOf::try_new` explicitly.

The public `LayoutInputOf<S>` contains `InlineText(InlineTextInputOf<S>)`,
`inline_text`, and `as_inline_text`. A text input is an inline participant, never
a box, leaf measurement, absolute child, float, or scroll container.

### 5.2 Non-Box Tree Pairing

`LayoutTree` continues to expose both `node_input(node)` and
`layout_input(node)` for every node. `NodeInputOf<S>::non_box()` is the sole
valid companion for `LayoutInputOf::{InlineText, LineBreak, InlineBoundary}`. It
fully initializes every field: `display` is `Display::None`; atomic participation
is `None`; float exclusion is `MarginBox`; and every other field has its current
initial/default value. It is not inferred from `NodeInputOf::default()`, whose
box default remains `Display::Flex`, and no field of the non-box value supplies
text/control behavior.

Before computing or staging output for any encountered non-box node, root-request
validation requires exact `PartialEq` equality with `NodeInputOf::non_box()`,
`child_count(node) == 0`, and `has_leaf_measurement(node) == false`. A mismatch returns
`LayoutInvalidInputOf::NonBoxNodeRole` with a typed reason naming noncanonical
node input, non-box children, or non-box measurement. It never ignores an arbitrary
display, float, position, sizing, overflow, order, grid/flex, atomic, or shape
field. Line-break and boundary behavior remains wholly in their private-field
layout inputs; their existing tree shape is preserved.

An inline-text node bypasses box, hidden-box, leaf-measurement, float,
positioned, flex, grid, scroll, and child traversal. Validation happens before
cache lookup and before any public output/cache commit, so a contradictory
pairing cannot reuse or publish a result. FRI-06.6 defines cache invalidation and
fragment restoration; no text or shape revision is hidden in the unit cache
context.

### 5.3 Fragment Output

The fragment output contract contains:

```rust
pub struct InlineFragmentOutputOf<S: LayoutScalar> { /* private */ }
pub struct InlineFragmentOutputEntryOf<Node, S: LayoutScalar> { /* private */ }
```

`InlineFragmentOutputOf` exposes:

- `segment_id()`;
- physical `rect()` for the used metric segment box on one line;
- physical `baseline()` point;
- zero-based `line_index()` within the containing inline formatting context;
- `visual_index()` within that line; and
- `replacement_inline_extent()` used at the selected break, if any.

For a shaped segment, the logical inline extent is the supplied segment extent,
the logical block start is the completed line baseline minus the supplied
segment baseline, and the logical block extent is the supplied line extent.
Projection makes that metric box and line baseline physical. Neither the
fragment nor its text-node union represents browser glyph-ink bounds.

One source segment produces at most one fragment because segments are
indivisible. A whitespace segment discarded at a selected line edge produces no
fragment. It remains a zero-advance, zero-metric source anchor in the line's bidi
unit sequence so its node position and neighboring visual slots stay
deterministic; its absence from fragment output is observable through the
missing segment ID. Such a segment cannot carry a replacement opportunity. A
selected replacement belongs to the preceding preserved segment's fragment
metadata and does not invent another source identity.

An inline-text node's `NodeOutputOf` uses the physical bounding union of its
published fragment rects for `location`, `size`, and `content_size`; all edges
are zero and `scroll_geometry` is `None` because text input generates no CSS box.
The containing box's scroll accumulator includes the actual fragment rects, not
that potentially gapped bounding proxy, exactly once.

If the node publishes zero fragments, its `location` is the physical point of
its first source segment's zero-advance anchor after break selection, line
alignment, bidi ordering, and `FlowAxes` projection. Its `size` and
`content_size` are zero, all edges are zero, and `scroll_geometry` remains
`None`. A mandatory break still establishes its specified strut/control lines;
an otherwise all-discarded text run establishes no metrics or scroll extent.
The anchor is retained in committed unrounded fragment state and rounded once by
the root policy on both cold computation and valid warm-cache reuse.

`InlineFragmentOutputEntryOf` exposes the source layout node and fragment. The
completed batch preserves entries in source-tree order, then segment source
order; line/visual indices carry visual ordering without reordering public batch
identity. Hidden layout publishes no fragment entries. Rounding rebuilds fragment
rects and baselines from unrounded line sources exactly once with the same root
rounding policy as node outputs.

`CompletedLayoutBatchOf` exposes `unrounded_inline_fragments()` and
`final_inline_fragments()` backed by separate private vectors, matching its
existing node-output phases. The batch consumer atomically commits unrounded and
final node/fragment state before its cache updates. A failed layout returns no
node outputs, cache updates, or fragments.

### 5.4 Float Exclusion Provider

`NodeInputOf<S>` has this closed layout-ready field:

```rust
pub enum FloatExclusion {
    MarginBox,
    Shape,
}
```

Its real CSS initial/default is `MarginBox`. `Shape` means the tree promises a
provider for this floating node; it carries no foreign shape identity.

`Shape` is valid exactly on a visible in-flow floating box:
`LayoutInputOf::Box`, `display != Display::None`, `position !=
Position::Absolute`, and `float` equal to `Float::Left` or `Float::Right`.
Root-request validation rejects every other `Shape` combination as
`LayoutInvalidInputOf::FloatExclusionRole` before traversal or provider access.
`MarginBox` remains valid for every box and is ignored when that box is not
floated. Upstream box generation remains responsible for blockifying an authored
floating inline display; FRI-06 does not infer or rewrite display roles.

The validated query/result carriers are:

```rust
pub struct FloatExclusionQueryOf<S: LayoutScalar> { /* private */ }
pub struct FloatExclusionIntervalOf<S: LayoutScalar> { /* private */ }
```

The query exposes the final physical margin box, containing `FlowAxes`, and a
finite ordered physical line-band interval. The interval result contains zero or
one finite ordered physical exclusion interval for that band and privately
retains the exact query supplied to its constructor. Construction clips the
result to that query's float margin box and rejects inverted or non-finite
endpoints. An empty shape intersection is represented by `None`.

`LayoutTree` provides this default method and layout calls it only for
`FloatExclusion::Shape`:

```rust
fn float_exclusion_interval(
    &self,
    node: Self::Node,
    query: FloatExclusionQueryOf<Self::Scalar>,
) -> Option<Result<Option<FloatExclusionIntervalOf<Self::Scalar>>, Self::MeasureError>> {
    None
}
```

Outer `None` means missing provider and is an error for `Shape`; `Ok(None)` means
a valid empty intersection. Layout accepts `Ok(Some(interval))` only when the
interval's private originating query equals the exact current query. A provider
can therefore return a representable invalid result by constructing an interval
for a different valid query; layout rejects it as
`FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual }` without
clipping, fallback, or partial output. Non-finite, inverted, and otherwise
invalid raw endpoints fail at `FloatExclusionIntervalOf::try_new` before a
provider response exists and are never fabricated as runtime output. The
existing measurement error carrier preserves a safe underlying provider failure
with `LayoutErrorOf` context naming the float node and band.

Queries occur only for a line candidate whose block interval overlaps the float
margin box. The same float/band pair is queried at most once per candidate pass;
results are local algorithm state and do not enter the global cache separately.
Provider changes use the explicit tree-owned invalidation contract in FRI-06.6;
they never mutate `CacheKeyContext`.

### 5.5 Vertical Alignment

`VerticalAlign` is:

```rust
pub enum VerticalAlign {
    Baseline,
    Top,
    Bottom,
}
```

It remains a normalized layout-ready subset, not authored CSS. `Top` aligns the
participant margin box to line-over and `Bottom` to line-under after baseline
participants determine the initial line box. Top/bottom participants then expand
the line symmetrically only as required to contain every margin box; they do not
contribute a fake zero baseline.

FRI-09 owns replacement of this subset with its complete typed alignment model.
FRI-06 does not add an unsupported enum variant to public input.

## 6 FRI-06.6 Invariants And Errors

Every scalar-bearing input and provider output rejects NaN, infinity, negative
extent, inverted interval, and non-canonical signed zero where the surrounding
geometry contract canonicalizes zero. Constructors return no partial value.

The following are invalid caller inputs:

- empty shaped text;
- duplicate segment IDs;
- bidi level above 125;
- non-finite/negative segment, replacement, metric, or exclusion values;
- replacement opportunity paired with any discardable whitespace edge;
- non-box layout input paired with noncanonical node input, children, or a leaf
  measurement provider;
- `FloatExclusion::Shape` on a hidden, non-floating, or absolute box;
- a shape exclusion request without a provider;
- a changed node supplied to `compute_layout_invalidated` that is not reachable
  from the supplied root;
- a provider interval constructed for a query other than the exact current
  float/band query; and
- line-break or boundary flow metadata that differs from its containing inline
  flow after root normalization.

Public constructors return `InlineTextInputErrorOf<S>`,
`AtomicInlineParticipationErrorOf<S>`, or `FloatExclusionIntervalErrorOf<S>` as
applicable; each enum names the rejected invariant and preserves its validated
numeric error rather than a string. Root-request validation maps an atomic
display/participation mismatch to
`LayoutInvalidInputOf::AtomicInlineParticipation`, and a non-box pairing failure
to `LayoutInvalidInputOf::NonBoxNodeRole`. Layout maps invalid inline facts to
`LayoutInvalidInputOf::InlineText`, an invalid shape/float role to
`LayoutInvalidInputOf::FloatExclusionRole`, a provider interval whose private
originating query differs from the current query to
`LayoutInvalidInputOf::FloatExclusionProviderOutput` with
`FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual }`, and an
absent requested provider to
`LayoutMissingContext::FloatExclusionProvider`. Provider `Err(M)` uses the
existing `LayoutErrorKindOf::Measurement(M)`. Every shape-query error uses the
exhaustive `LayoutOperation::FloatExclusionQuery` and
`LayoutErrorSiteOf::ContainerSubject { container, subject: float }`; the query
values preserve the exact expected and actual bands. No path erases the provider
source to a string, panics, or returns a partial completed batch.

An unreachable invalidation subject uses
`LayoutInvalidInputOf::InvalidationNodeNotReachable`,
`LayoutOperation::CacheInvalidation`, and `LayoutErrorSiteOf::Node(subject)`.
Duplicate reachable subjects are valid and normalize to one closure entry.

Valid layout-ready combinations never panic. In particular, every
writing-mode/direction pair with `Clear::{None,Left,Right,Both}` is accepted for
visible line breaks. Clear directions are mapped through the containing flow;
the control's own stored flow is only a validation witness and never overrides
the container.

### 6.1 Cache And Fragment State

FRI-01's invalidation model remains authoritative. `CacheKeyContext` stays the
zero-field unit value, `CacheKeyContext::new()` and
`LayoutTree::cache_context()` keep their signatures, and no text/style/shape or
provider revision token enters the cache key. The existing key continues to own
only recursive `ComputeInputOf` state plus that unit context.

The crate provides this public entry point while retaining `compute_layout` for a
tree with no pending dirty nodes:

```rust
pub fn compute_layout_invalidated<Tree: LayoutTree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    changed_nodes: &[Tree::Node],
) -> LayoutResultOf<
    Tree::Node,
    CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>,
    Tree::Scalar,
    Tree::MeasureError,
>;
```

The owner supplies the minimal surviving dirty subjects:

- `NodeInputOf`, `LayoutInputOf`, child identity/order/topology, or measurement
  facts/output changes name that node; insertion, removal, or reorder names the
  surviving parent whose child list changed;
- shaped segment, atomic participation, break, whitespace, bidi, containing
  flow, vertical alignment, or text source/style/font generation names every
  affected text/control/atomic node; and
- float exclusion mode, shape/reference/transform/provider geometry, provider
  failure state, or any other exclusion-query result names every affected float.

The function performs one source-order DFS from `root`, rejects an unreachable
changed node as `LayoutInvalidInputOf::InvalidationNodeNotReachable`, and builds
the deduplicated union of every inclusive root-to-changed-node path. That union
is the exact invalidation closure. During this computation, cache reads and
stores are bypassed/staged respectively for every closure node; no existing
cache or committed output is mutated. `compute_layout` is equivalent to an empty
changed-node slice and is valid only when the owner has no pending dirty subject.

`CompletedLayoutBatchOf` privately retains the invalidation closure and exposes
`invalidated_nodes()` in source-tree order. It also gains:

```rust
pub trait LayoutBatchSink<Node, S: LayoutScalar> {
    type Error;
    type Prepared;

    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<Node, S>,
    ) -> Result<Self::Prepared, Self::Error>;

    fn commit_layout_batch(&mut self, prepared: Self::Prepared);
}

impl<Node, S: LayoutScalar> CompletedLayoutBatchOf<Node, S> {
    pub fn apply_to<Sink: LayoutBatchSink<Node, S>>(
        &self,
        sink: &mut Sink,
    ) -> Result<(), Sink::Error>;
}
```

`apply_to` first calls immutable, fallible `prepare_layout_batch`. Preparation
validates the entire batch and materializes an owned `Prepared` replacement for
every unrounded/final node output, unrounded/final fragment slice, invalidation
and hidden-layout cache clear, and cache store. Preparation may inspect existing
state but cannot require any mutation; an error returns before exclusive commit
access and leaves all state unchanged.

After successful preparation, `apply_to` calls infallible
`commit_layout_batch` under exclusive `&mut` access. Commit replaces all node
and fragment state, clears invalidation-closure and hidden-layout caches, applies
all stores after all clears, and releases no caller-owned dirty state. Because
`Prepared` owns everything commit needs and commit cannot fail, no rollback path
or partial error state exists. Sink implementations may not use interior
mutation during preparation or expose state concurrently during commit; focused
contract fakes prove preparation errors make zero mutations and commit performs
the complete ordered replacement.

Only after `apply_to` returns `Ok` may the owner remove corresponding dirty
subjects. A layout or preparation error leaves old committed state/caches and
dirty subjects intact for the next `compute_layout_invalidated`. There is no
direct pre-layout `CacheOf::clear()`, partial key mutation, or layout-owned
revision counter.

To restore output on a valid warm hit, `LayoutTree` provides:

```rust
fn unrounded_inline_fragments(
    &self,
    node: Self::Node,
) -> Option<&[InlineFragmentOutputOf<Self::Scalar>]> {
    None
}
```

`Some(&[])` is valid committed zero-fragment text; `None` means no committed
fragment state. During rounding, `ComputeSession` uses newly staged fragments
first and otherwise reads this method while traversing every node, just as
existing rounding reads staged or committed unrounded node output. An
`InlineText` node reached from a warm cache path with `None` returns
`LayoutInternalInvariant::MissingCachedInlineFragmentState`; it never publishes
an empty substitute. Rounding republishes the complete final fragment entries in
the returned batch without rerunning shaping, break selection, float queries, or
line construction.

No new public value has mutable fields. Output carriers have no public
constructor or `Default`. Input values have `Default` only where a real CSS or
layout initial exists.

## 7 FRI-06.7 Behavior Matrices

### 7.1 Participant Matrix

| Participant | Line advance | Line metrics | Break behavior | Public output |
| --- | --- | --- | --- | --- |
| Shaped segment | Used segment inline extent plus selected replacement | Supplied line-over/baseline/line-under metrics | Supplied prohibited/allowed/replacement/mandatory opportunity | Fragment entry; text node union output |
| Atomic inline box | Margin-box inline extent | Inner or fallback baseline, or top/bottom alignment | Supplied opportunity following the box; the prior participant owns the opportunity before it; box itself is indivisible | Ordinary node output |
| Visible line break | Zero | Containing strut/control metrics | Mandatory after control, then seed next-line strut; apply clear before next line | Zero-size node output at exact control position |
| Hidden line break | None | None | None | Hidden output, no fragment |
| Inline boundary start/end | Zero | Boundary/containing strut metrics | No independent opportunity | Zero-size node output at exact boundary position |
| Out-of-flow positioned box | None in FRI-06 | None | Does not create or remove break opportunity | Existing FRI-10 handoff retains hypothetical-position gap |
| Float | Removed from inline participants | None | Splits available bands for subsequent lines | Ordinary floated node output |

Text and atomic boxes count as breakable inline content. Controls alone create a
line only when their metrics or preserved mandatory break requires one. An empty
post-break line carries the containing strut when the break requires a following
line; it is not dropped merely because it has no box.

### 7.2 Break Selection Matrix

| Candidate state | Result |
| --- | --- |
| Next segment fits current band | Append it |
| Next segment does not fit and a prior allowed opportunity exists | Roll back to the latest allowed opportunity, apply its replacement if present, discard classified edge whitespace, and commit |
| Next segment does not fit and no prior opportunity exists | Place the indivisible segment on the current empty line and allow overflow |
| Mandatory opportunity | Commit immediately after its segment |
| Float band has no non-negative inline space | Advance to the next finite float transition and retry; if none exists, use the containing line band and allow indivisible overflow |
| Min-content query | Maximum indivisible post-whitespace segment/replacement contribution across mandatory groups |
| Max-content query | Maximum mandatory-line sum with all allowed opportunities unselected |

The algorithm is greedy and deterministic. It never asks the text owner to
reshape after choosing a break. A replacement extent supplied for a hyphenated
break is already shaped and final.

### 7.3 Bidi And Whitespace Matrix

| Fact | Layout behavior |
| --- | --- |
| Reorderable line units | Every surviving or discarded shaped segment, atomic inline box, and visible inline-boundary marker occupies one source-ordered unit. Text/atomic units use supplied levels; a boundary uses the containing paragraph base level (`0` for LTR, `1` for RTL). Stable descending-level reversal applies to this complete final-line unit slice. |
| Fragment `visual_index()` | Zero-based slot in the complete reordered unit slice, not a dense text-fragment index. Atomic boxes, boundaries, and discarded anchors can therefore create intentional gaps between published fragment indices. |
| Visible line break | Commits the source slice before reordering, is positioned at the aligned visual line-end point after reordered units, and has no visual slot. Its following line starts a new unit slice. |
| Hidden break, float, absolute box | Has no bidi unit or visual slot. Floats and out-of-flow boxes retain their separate placement paths. |
| `Preserve` | Always contributes and publishes |
| `DiscardAtLineStart` | Omitted only when it begins a selected line |
| `DiscardAtLineEnd` | Omitted only when it ends a selected line |
| `DiscardAtBoth` | Omitted at either selected edge; preserved in the middle |
| Discarded segment with break opportunity | Opportunity remains available at its source boundary, but no zero-size fragment is emitted |
| Discard classification with `AllowedWithReplacement` | Rejected during shaped-segment construction; replacement metadata always has a preserved source fragment |

Layout does not collapse adjacent text, infer whitespace classes, mirror glyphs,
or invent atomic bidi/break behavior. Every candidate break is associated with
the participant immediately before it. Root's composed text projection maps text
segments to `InlineTextInputOf` and every shaped atomic placeholder to the
corresponding child's `AtomicInlineParticipationOf`; containing direction owns
the base level of structural boundary markers.

### 7.4 Line Alignment Matrix

| `TextAlign` | LTR horizontal | RTL horizontal | Vertical/sideways |
| --- | --- | --- | --- |
| `LegacyLeft` | Physical/line-left | Physical/line-left | Containing-flow line-left |
| `LegacyRight` | Physical/line-right | Physical/line-right | Containing-flow line-right |
| `LegacyCenter` | Half non-negative free inline space | Same | Same logical rule projected by `FlowAxes` |

Each line uses its own float-adjusted start/end and used extent. Negative free
space never reverses or magnifies alignment; legacy center/right clamp their
offset to zero for overflow. FRI-09 owns start/end/match-parent/justify and
text-align-last expansion.

### 7.5 Float And BFC Matrix

| Subject | Active float interaction |
| --- | --- |
| Ordinary line box | Query the float band for that line's block interval; shrink or move below transitions until content can be placed |
| Ordinary block child | Outer edge remains at normal containing edge; its internal line boxes see floats |
| In-flow block-level `Flex`, `Grid`, or `GridLanes` child | Margin box avoids overlapping active exclusion intervals and uses the available band for auto inline size |
| In-flow non-replaced block-level child whose computed overflow pair establishes an independent formatting context | Same BFC avoidance and auto-inline-size rule |
| Floating child | Place as far to line-left/right as possible after prior floats, preserving source order and clear |
| Atomic inline child | Its margin box participates in the float-adjusted line; its inner block/grid context traps internal floats but does not enter block-child BFC placement |
| Absolutely positioned child | Does not affect float placement or exclusion in FRI-06 |
| `Display::None` | Produces no box and no exclusion interaction |
| Float-only container | Auto block size encloses in-flow floats that belong to its BFC, excluding container start inset exactly once |
| Nested BFC | Internal floats do not escape; outside floats affect only its own margin-box placement |

`Float::Left` and `Float::Right` mean line-left and line-right in the containing
writing mode. `Clear::Left`, `Right`, and `Both` clear the corresponding mapped
float sides. All calculations are logical until one `FlowAxes` projection.

For `FloatExclusion::MarginBox`, every overlapping line band receives the
margin-box inline interval. For `Shape`, the provider interval replaces that
interval for the queried band; `Ok(None)` produces no exclusion for that band.
`shape-margin` and CSS basic-shape parsing/resolution are root/shape handoff
facts represented only through the returned interval.

### 7.6 Atomic Baseline Matrix

| Atomic used overflow | Inner baseline present | Alignment | Used transverse placement |
| --- | --- | --- | --- |
| Visible | Yes | Baseline | Use selected inner first baseline plus margin/border position |
| Visible | No | Baseline | Use block-end margin edge |
| Clip/Hidden/Scroll/Auto | Any | Baseline | Use block-end margin edge |
| Any | Any | Top | Align margin box to line-over after baseline sizing |
| Any | Any | Bottom | Align margin box to line-under after baseline sizing |

The used-overflow decision uses FRI-05's private canonical used axis for the
containing line's block axis, including replaced-hidden conversion. It is not
recomputed from authored or independent raw axes.

### 7.7 Control And Clear Matrix

For each of the ten `FlowAxes` mappings, every visible line break with
`Clear::{None,Left,Right,Both}`:

1. validates against the containing inline flow;
2. publishes its physical zero-size position;
3. commits the current line with its metrics;
4. preserves the containing strut on the following line;
5. maps clear to current line-left/line-right floats;
6. advances only as far as the matching active exclusion requires; and
7. never panics.

An inline boundary contributes its metrics to the line it occurs on and is
compared by parity like any other zero-size control. Boundaries do not clear.
Browser neighbor-line comparison first applies one closed interval-overlap rule
with the existing `0.1` tolerance. A zero-size break is therefore `Same` with
either neighbor sharing its endpoint. Only a separated interval is classified
`Earlier` or `Later` from its center and the containing block progression; the
five-pixel unequal-line gap is `Later`.

## 8 FRI-06.8 Algorithm Contract

### 8.1 Logical Line Builder

`src/inline.rs` owns one algorithm state with these private concepts:

- normalized participant stream;
- `LogicalLineBandOf<S>` with inline start/end and block start;
- pending source-ordered participants and latest legal rollback point;
- line strut and baseline/top/bottom metric groups;
- selected line source plus visual-order projection; and
- fragment and participant publication sources retained for rounding.

The builder receives the containing `FlowAxes`, available logical inline/block
size, current float exclusion provider, and participants. It emits logical line
records. Physical projection happens after every line's used block extent and
independent inline alignment are final.

For each selected line, the builder retains one complete source-ordered unit
slice containing text segments, atomic boxes, and inline boundaries, including
zero-advance discarded anchors. It applies stable descending-level reversal to
that slice, assigns visual slots before physical placement, and then projects
each unit. A visible break is published at the resulting visual line end and is
not reversed. This same unit list is the sole source for atomic locations,
boundary positions, text anchors, and fragment `visual_index()` values.

Line construction uses a monotone block cursor. Float transitions are finite
margin-box block endpoints. A candidate retry must either append/commit content
or advance to a strictly later transition. This proves termination without an
arbitrary iteration count.

### 8.2 Mixed Source Composition

Block layout groups consecutive inline-level children into one formatting
context. It lowers:

- `LayoutInputOf::InlineText` segments directly;
- computed atomic inline child output plus style into an atomic participant;
- line-break and boundary controls through containing-flow validation; and
- floats/out-of-flow boxes into their existing separate paths.

Root's composed text projection supplies atomic-placeholder facts separately
from text segments. Block layout reads the required
`atomic_inline_participation` from the corresponding atomic child and inserts
that participant at the child's source position. A missing or extraneous fact is
rejected by root-request validation; root integration also has compile/static
and end-to-end evidence that every retained atomic placeholder supplies the
composed fact.

For fixture-only simple text that contains no bidi-reordered atomic placeholder,
one text child can supply only text segments. The adapter may not create a
synthetic measured box in place of text.

### 8.3 Float Bands

`FloatExclusions` is flow-relative and records each placed float's mapped
line side, physical margin box, block interval, exclusion mode, and source
order. It answers a line-band query with the unioned line-start and line-end
constraints plus the next later transition. Overlapping same-side floats choose
the farthest inward edge; opposing floats may reduce the band to zero.

Float placement probes at the current clear-adjusted block position, queries
the candidate's full margin-box block span, and moves monotonically to the next
transition until the margin box fits or no transition remains. An overwide float
is placed at its required line side and allowed to overflow; it never loops or
uses a negative available size.

An in-flow, non-floating block-level child enters BFC avoidance exactly when its
display is `Flex`, `Grid`, or `GridLanes`, or when it is non-replaced and its
normalized computed overflow pair returns true from
`establishes_independent_formatting_context()`. `Display::Block` with visible or
clip overflow remains an ordinary block child; inline-level, absolute, replaced,
and `None` roles do not satisfy this predicate. A qualifying child uses the same
full-span band query. If its inline size is auto, available inline size is the
saturated band width before child layout. If definite or overwide, its margin
box moves below transitions until it fits or overflows at the normal block
position after the final transition. Floating boxes are handled by float
placement and independently trap their internal floats.

### 8.4 Size, Baseline, Scroll, Cache, And Rounding

Min/max-content inline contributions include shaped text segments, selected
replacement extents, atomic margins, and mandatory breaks. Float/BFC intrinsic
rules use the same participant facts without performing final layout or calling
the shape provider for an indefinite line band.

Fixed-size `ComputeSize` may return early only when no normal-flow participant
can establish a requested baseline or other required output. Text, visible
metric-bearing line breaks, and inline boundaries all make that predicate true.

Line/text/float geometry enters the FRI-05 scroll accumulator once from final
physical output. Discarded whitespace and hidden controls contribute nothing.
Text fragment unions include zero-area extents without inventing a box margin.

Valid cache hits reuse the complete FRI-01 node output while rounding restores
the atomically committed unrounded fragment slice through `LayoutTree`. A stale
input/provider change is handled by the invalidated entry point's exact
root-to-dirty closure and the successful batch transaction, not a
revision-bearing key. Speculative size queries and failed passes cannot commit
or publish fragments or cache clears.

Rounding preserves source association, line/visual indices, segment IDs,
baselines, float bands, participant positions, and cache/cold equality. The
normal and rounded path both project from the same logical line source; rounded
layout does not rerun shaping, break selection, provider queries, or line
construction.

## 9 FRI-06.9 Focused Evidence

| Evidence family | Required proof |
| --- | --- |
| Input validation | Empty/duplicate/invalid segment, bidi, replacement, replacement/discard, metrics, placeholder, canonical non-box pairing/children/measurement, shape/float role, provider, and unreachable invalidation cases fail atomically in both scalar lanes; duplicate dirty subjects normalize |
| Text wrapping | Prohibited, allowed, replacement, mandatory, preserved/discarded whitespace, all-discarded zero-fragment anchor, overwide indivisible, min-content, and max-content cases |
| Bidi | Nested even/odd text and atomic levels plus boundary base levels reorder independently per selected line; source output order/IDs remain stable; visual slots include atomic, boundary, and discarded-unit gaps; visible breaks stay at visual line end |
| Mixed inline | Text before/between/after atomic boxes and controls wraps and bidi-reorders without synthetic leaves; placeholders map exactly once |
| Per-line alignment | Unequal wrapped lines under legacy left/right/center in all flow mappings use each line's own band and extent |
| Struts/controls | Leading, trailing, adjacent, and only-child breaks/boundaries preserve metrics, baselines, and following-line strut |
| Vertical lines | Soft wrap, forced break, bidi, top/bottom/baseline, and clear work in vertical-rl, vertical-lr, sideways-rl, and sideways-lr, both directions |
| Atomic baselines | Visible inner, absent inner, non-visible overflow fallback, bottom margin, top, bottom, and replaced cases |
| Percentage basis | Definite containing block resolves atomic percentage block size; indefinite remains unresolved by existing sizing rules |
| Fixed fast path | Metric controls/text prevent baseline-erasing early return; size-only control case still takes the valid fast path |
| Rectangular floats | Left/right/opposing/stacked/overwide/clear/float-only auto-height and ordinary line exclusion through block front door |
| Shape provider | Empty/partial/full intervals, missing provider, mismatched-query result, raw endpoint constructor rejection, provider error, cache-context change, and bounded query accounting |
| BFC avoidance | Current flex/grid/grid-lanes and non-replaced overflow-established block cases avoid floats and shrink auto width; ordinary block edges remain unchanged; floating and atomic boxes trap internal floats through their respective paths |
| Scroll/cache/rounding | Exact dirty-subject path closure bypasses stale hits; failed layout or immutable preparation makes zero mutations and retains dirty subjects; infallible exclusive commit replaces all node/fragment state and clears closure caches before stores; committed nonempty/empty slices republish identically cold/warm and normal/rounded; missing warm fragment state errors; geometry contributes once |
| Comparator | Wrong/missing line-break position, text fragment, line index, visual index, and baseline each fail with named diagnostics; interval comparison is closed within tolerance, so a zero-size break is `Same` with either neighbor sharing its endpoint and a neighbor beyond tolerance is ordered by block progression |
| Fixture-input honesty | The browser helper validates every reviewed wrapper/container/strut/bidi marker against the actual DOM and reports source-local marker use; serializer tests prove unconditional, matching-direction, inactive-direction, malformed-direction, and unknown-field bidi records plus the closed mapping in `FRI-06.11`; renamed test names and arbitrary expectation-only mutations preserve identical normalized parsed input; missing/corrupt markers fail helper validation, final full-run inventory accounting, or generated-XML parser validation; transparent wrappers produce independent normalized input and expectation trees; static/runtime evidence proves no final input-lowering function reads fixture identity or expectations |
| Subgrid baseline controls | Public-layout first/last controls prove one-time ancestor-logical conversion, scalar/member/group/view separation, direct descendant participation, start/end edge accumulation, row/column grouping, positive/equal/negative half-gutter and MBP transitions, reversed mappings, and idempotent envelope views in horizontal and vertical flows; no fully inherited-axis publication inverse remains |
| Chrome oracle exception | Default-zero exact registry; every entry proves the `FRI-06.11` certainty gate, minimized parser-independent reproduction, normative and independent corroborating evidence, exact variant scope, public-front-door synthetic RED/GREEN substitute, visible report/test disposition, and revalidation trigger; negative controls reject layout-only disagreement, ambiguous rounding/coordinates, missing evidence, and overbroad manifest status |
| Browser corpus | Owned mixed text/atomic/BR, vertical BR, float/BFC, unequal alignment, baseline, percentage atomic, and shape cases parse and compare |

Every changed behavior has regression evidence through `compute_layout`,
block/inline public formatting, or the real browser parity front door. Private
line-builder tests supplement but never replace front-door evidence.

The shape provider fake must implement the real query/result contract and expose
only query records that are themselves observable acceptance criteria. It cannot
return precomputed final line positions.

## 10 FRI-06.10 Module And API Outline

| Module or artifact | Desired responsibility |
| --- | --- |
| `src/node_input.rs` | Shaped segment/text and atomic participation models, bidi/break/whitespace validation, canonical non-box input, float exclusion mode, bottom alignment, and `LayoutInputOf::InlineText` |
| `src/output.rs` | Fragment carriers, invalidation closure, phase-specific batch output, atomic sink contract, and source-preserving accessors |
| `src/traits.rs` | Bounded shape exclusion provider and committed unrounded-fragment readback methods on `LayoutTree` |
| `src/compute.rs` | Provider/error plumbing, invalidated entry point/path closure, cache bypass/staging, and cache/round fragment publication |
| `src/inline.rs` | One logical mixed-participant line builder, break/bidi/whitespace, line metrics, per-line alignment, and physical projection |
| `src/block.rs` | Inline composition, containing strut, float-band/BFC placement, percentage basis, fast-path baseline predicate, scroll contribution |
| `src/grid/tracks.rs` | Extend the existing axis-parametric subgrid traversal to emit separate `FlattenedScalarContribution` and `AncestorBaselineMember` records; suppress a fully inherited root only from the ordinary scalar pass, reduce direct and flattened members before intrinsic shims, and preserve the immutable ancestor group for final alignment |
| `src/grid/subgrid.rs` | Own checked ancestor-span, mapped-edge, reversal, and accumulated margin/border/padding and half-gutter traversal facts. It maps an immutable `AncestorBaselineGroup` to a non-publishable `ChildBaselineEnvelopeView`; no fully inherited-axis child-to-parent inverse exists |
| `src/grid/child.rs` | Consume the immutable ancestor group and child envelope views through one `GridAxisKind` path, apply alignment offsets once, retain ordinary local baseline behavior on non-inherited axes, and keep containing-grid area sizing plus the single final physical projection |
| `src/cache.rs` | Preserve FRI-01's unit key and support clear-then-store batch application with fragment restoration |
| Focused Rust tests | Model, line, block, root, cache, scalar, flow, provider, comparator, public surface, and failure evidence |
| `tests/layout/browser_parity/support.rs` | Exact shaped fixture lowering, fragment/control comparison, and named mismatch diagnostics |
| Existing generator/helper | Serialize/capture only the bounded new fixture facts; no architecture expansion |
| `corpus.toml`, HTML, XML, report | Active FRI-06 source inventory, one final full regeneration, provenance and bucket closure |
| `README.md`, crate/parity rustdoc | Layout/text/shape/root ownership and finite fixture boundary |

Private helper names and internal decomposition may differ. Public phases,
invariants, output association, provider failure, and ownership may not.

The private grid carriers have no `Default` and no public surface. Their checked
constructors require the source, grid axis, physical baseline axis, ancestor span,
mapped edge, first/last role, containing-logical distance, and accumulated edge
adjustments appropriate to their phase. `FlattenedScalarContribution` is the only
channel allowed to grow a track by a complete item contribution.
`AncestorBaselineMember` can join a group without carrying scalar size;
`AncestorBaselineGroup` contains only the reduced alignment targets; and
`ChildBaselineEnvelopeView` can only be constructed from a group plus a checked
child mapping. The axis-parametric operation selects rows or columns from
`GridAxisKind`; it does not branch on writing mode, direction, source name, or
fixture family. `FlowAxes` performs the physical-to-logical conversion and the
final projection.

Focused public-layout tests must expose the complete transition: horizontal
auto rows move from the current duplicate `459` to browser `411`; inline-column
first/last groups align in LTR and RTL; nested parent-gap `10`/child-gap `20`
produces the descendant/direct-sibling tuple `(62, 110)` from the clean-base
`(57, 125)`, with equal, negative, reversed, MBP, and repeated-view controls;
vertical auto rows transform pre-flex `[163,145,145]` to final
`[212,194,194]`, inherited area `381`, child width `371`, and x `196`; the
vertical nested projection is x `153`. Existing containing-grid `FlowAxes`
refresh controls remain unchanged.

Every existing `#[allow(dead_code)]` in an FRI-06-owned inline/control path must
either become genuinely consumed or be removed with the dead item. No new lint
allowance, compatibility alias, test-only public API, or broad suppression is
permitted.

### 10.1 Compatibility Classification

At the canonical adaptation base, every leaf surface in this table already
exists. `D-18` changes no public API; the classifications remain the compatibility
contract for downstream integration.

| Public surface | Current compatibility contract |
| --- | --- |
| `LayoutInputOf::InlineText` | Exhaustive downstream matches handle text explicitly; no wildcard reinterprets it as `Box`, measurement, line break, or boundary. |
| `NodeInputOf::non_box()` and non-box pairing validation | Text, line-break, and boundary callers use the canonical companion; no compatibility path ignores contradictory box fields, children, or measurement. |
| `compute_layout_invalidated`, `invalidated_nodes`, `LayoutBatchSink`, and `CompletedLayoutBatchOf::apply_to` | `compute_layout` keeps its signature but is valid only with no pending dirty nodes. Mutation-driven integration uses immutable fallible preparation plus infallible exclusive commit; no direct pre-layout cache clear remains. |
| `NodeInputOf::atomic_inline_participation` | The field's real default is `None`; default and struct-update callers retain no atomic participation, while every participating atomic child sets a validated `Some` explicitly. |
| `NodeInputOf::float_exclusion` | The field's real default is `MarginBox`; only a visible in-flow `Float::Left`/`Right` box whose tree can satisfy the provider contract sets `Shape`, and every other shape/role pair is rejected. |
| `VerticalAlign::Bottom` | Every leaf and downstream exhaustive match handles `Bottom` explicitly; no compatibility wildcard maps it to baseline or top. |
| Shaped-text, bidi, break, whitespace, fragment, and exclusion carriers plus default-scalar aliases | Private invariant-bearing fields and validated constructors/accessors remain exact. `FloatExclusionIntervalOf` retains its originating query privately. No legacy alias or permissive conversion exists; downstream facade/API artifacts expose these exact names. |
| `FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual }` | Exhaustive construction/provider-output error matches include the named state; no wildcard or existing numeric error is reused for mismatched provenance. |
| `CompletedLayoutBatchOf::{unrounded_inline_fragments, final_inline_fragments}` and private storage | Read-only phase-specific output retains existing node/cache accessor semantics. Root commits unrounded fragment state with unrounded nodes and consumes final fragment association for render/text integration. |
| Defaulted `LayoutTree` exclusion-provider method | Existing implementors remain source-compatible and return no provider result. Margin-box layouts never call it; a requested shape without an override is a typed missing-provider error, never a silent margin-box fallback. |
| Defaulted `LayoutTree::unrounded_inline_fragments` | Existing implementors remain source-compatible and return `None`. Trees that commit inline text return `Some(slice)`, including `Some(&[])`, so valid warm hits can republish fragments. |
| Variants in existing `#[non_exhaustive]` invalid-input, missing-context, and internal-invariant error carriers | Callers preserve their required wildcard handling; typed variants retain node/band/provider/cache context and no string compatibility error exists. |
| `LayoutOperation::{FloatExclusionQuery, CacheInvalidation}` | Leaf and downstream exhaustive diagnostics handle both variants explicitly and retain the provider error plus container/float site or the unreachable dirty subject. |
| Existing line-break, inline-boundary, block, float, baseline, cache, and scroll behavior | Public shapes remain, but valid represented behavior is corrected. No legacy panic, whole-run alignment, measured-text substitution, skipped comparator output, or overlapping-float behavior is retained as a compatibility mode. |

Every in-repository public match, struct literal, trait implementation, test tree,
and fixture adapter handles the rows above, with source and compile evidence that
the old omissions are absent. Root and sibling repositories are not edited here.
Root integration updates facade reexports, exhaustive matches, box/text adapters,
tree provider implementation, fragment consumption, cache invalidation, and the
root-owned generated API artifacts against an exact compatible leaf revision. It
retains no old leaf pin, defaulted atomic placeholder, shape fallback, or
unhandled new enum state.

No Cargo feature, dependency, lockfile entry, MSRV, browser pin, launch profile,
generator architecture, or leaf-side generated API artifact changes for the
remaining FRI-06 work.

## 11 FRI-06.11 Browser Fixture And Artifact Contract

FRI-06 activates bounded source families rather than the entire later-format
corpus. The immutable starting report is
`tests/layout/browser_parity/xml/generation-reports/all.json` at SHA-256
`4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`.
Its FRI-06 inventory is selected by exact reason, not by a later planning
choice:

| Exact starting reason | Sources | Variants | FRI-06 transition |
| --- | ---: | ---: | --- |
| `Unsupported mixed text/element content` | 25 | 100 | Generated and active |
| `Unsupported vertical <br> line-break semantics` | 36 | 144 | Generated and active |
| `Unsupported <br> outside block inline-run semantics` | 24 | 96 | Generated and active |
| `Unsupported missing #test-root fixture root` | 4 | 16 | Remains unsupported |

The first three rows are the complete 85-source, 340-variant FRI-06 activation
set. Their exact source names are the unique `source` values under each reason in
that immutable report; changing a reason, omitting one entry, or selecting only a
subset is a gate failure. The 16 missing-root variants are not rewritten to
manufacture a fixture root and remain the only expected unsupported bucket.

FRI-06 also includes exactly these Surgeist-authored sources, each with all four
existing direction/box-sizing variants:

| Source | Owned matrix |
| --- | --- |
| `html/block/fri06_inline_mixed_text_atomic_wrap.html` | Mixed shaped text, atomic participation, soft wrap, fragment association |
| `html/block/fri06_inline_unequal_line_alignment.html` | Per-line left/right/center alignment with unequal line extents |
| `html/block/fri06_forced_break_strut.html` | Preserved forced break, empty following line, strut and control geometry |
| `html/block/fri06_vertical_break_clear.html` | Vertical/sideways break projection and logical clear |
| `html/block/fri06_atomic_inline_baseline.html` | Inner and fallback atomic baselines, margins, top and bottom alignment |
| `html/block/fri06_atomic_inline_percentage_block_size.html` | Definite containing block percentage basis |
| `html/block/fri06_bidi_mixed_inline.html` | Source association and per-line visual bidi order |
| `html/float/fri06_float_line_exclusion.html` | Mixed inline line exclusion by opposing rectangular floats |
| `html/float/fri06_float_bfc_avoidance.html` | Exact current BFC predicate and auto inline size |
| `html/float/fri06_float_auto_height.html` | Float-only auto block size and nested containment |
| `html/float/fri06_float_logical_clear.html` | Logical float sides and clear in non-horizontal flows |
| `html/float/fri06_float_shape_exclusion.html` | Finite provider-backed exclusion bands |

These sources contribute 48 generated variants. Starting from the cycle-base
5,324 generated and 356 unsupported variants, the one settled replacement must
therefore report 5,712
generated, 16 unsupported, zero quarantined or failed-to-generate variants, and
zero expected-fail variants unless the known-Chrome-failure registry contains one
or more exact entries satisfying the contract below. In that exceptional case,
the report's expected-fail inventory and count equal the registry exactly; no
unregistered row or broader source/variant set is permitted. The manifest at both
the cycle base and canonical adaptation base already contains these twelve
records and has SHA-256
`99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701`.

### 11.1 Known Chrome Measurement Failure Exception

Chrome remains authoritative unless every item below is satisfied. Uncertainty,
an implementation disagreement, a Taffy result, or a synthetic expected value
alone leaves the browser mismatch blocking.

1. Reduce the behavior to one exact source and generated variant set and record
   pinned Chrome version/platform, browser-observed values, specification-required
   values, and the smallest reproducer independent of Surgeist fixture parsing.
2. Cite an unambiguous normative CSS requirement or a pinned WPT test with an
   authoritative expected result. Supply one independent corroboration: another
   browser engine, a distinct existing WPT oracle, or a complete invariant
   derivation whose inputs are directly measured and whose result does not depend
   on Surgeist output. Two Chrome APIs observing the same implementation are not
   independent.
3. Prove the serialized layout-ready input satisfies `FRI-06.11`, parser/name/
   expectation independence passes, and the discrepancy remains before the
   comparator. A lowering, coordinate-space, rounding, or tolerance ambiguity
   disqualifies the exception.
4. Add a public-front-door synthetic regression using explicit layout-ready
   input and the specification-required geometry. It must fail before the layout
   correction and pass afterward; a private line-builder test is not a substitute.
5. Each accepted known-Chrome-failure record names exact source and variants,
   observed and required values, reason, normative and corroborating evidence,
   minimized reproduction, synthetic test, manifest/report disposition, and
   revalidation trigger. The registry is empty when no entry qualifies.
6. Incomplete or conflicting evidence keeps Chrome authoritative. Quarantine is
   never an alternative.

An accepted entry uses the existing manifest/report `expected-fail` mechanism
only when its status scope equals the exact proven source/variant set. If the
manifest can express only a broader source set, the exception must not be marked
there;
the exact comparator/test registry remains visible instead. The final 388-row
activation accounting records each row as either browser-pass or one reviewed
known-Chrome-fail with a passing synthetic substitute. A browser-pin, normative
specification, corroborating-engine, or WPT-expectation change reopens the entry;
new authoritative WPT coverage supersedes the synthetic substitute.

The fixture adapter may add only these layout-ready concepts:

- ordered shaped segments with stable local IDs, logical extent, line metrics,
  bidi level, whitespace edge, and following break opportunity;
- atomic placeholder bidi/break facts mapped to one child source index;
- explicit expected model-line fragment rect/baseline/line/visual data;
- browser `Range` observations categorized separately for source/line
  association and flow-inline start/advance only;
- browser control observations categorized separately for source, terminal
  visual slot, and neighboring line effects rather than `<br>` ink geometry;
- `vertical-align: bottom` lowering; and
- a finite shape-exclusion band table for provider tests.

The finite band table is fixture data, not authored `shape-outside` syntax. It
contains validated physical query band/result intervals and is consumed through
the same `LayoutTree` provider method as root integration.

The JavaScript helper may read browser-computed geometry and DOM ranges needed
for the named text/control fragments. A `Range` ink rect never supplies or
overrides a model fragment's block-axis start, block extent, baseline, or the
text node's metric-box union. The comparator uses Range data only for its named
source/line/flow-inline observations; browser Range order never supplies a model
fragment visual index. A Range start is physical-flow-relative to the nearest
explicit layout-ready inline containing root whose child sequence the adapter
lowers, never to an intermediate DOM text parent that the adapter removes.

For a lowered inline `<br>`, the helper obtains its containing strut baseline
from an isolated browser-laid-out marker pair using the same computed font,
line-height, writing mode, and direction. One zero-size marker establishes the
line-over edge and one establishes the browser baseline; their logical block
distance, clamped to the finite computed line height, supplies
`inlineBaseline`. The helper removes the probe immediately and serializes the
existing `inlineBaseline`/`inlineLineHeight` fields. A font-size ratio, glyph-ink
metric, authored fixture constant, expected geometry, source identity, or
production rounding is not a valid substitute. Zero line height remains an
exact zero metric.

A browser `<br>` rect never supplies model control geometry. Explicit model-line
and model-control expectations remain strict. Model line-control participation is
a separate explicit fixture fact emitted only when the computed/lowered `<br>`
role participates as an inline control in that containing formatting context. A
source tag, ancestor activation marker, or browser-control observation alone does
not make a blockified `<br>` a model control. For a non-wrapping flex containing
context, browser terminal-slot and neighboring-line observations are compared
from source position and flex-line membership without consulting either browser
BR ink or model control-point geometry; wrapped flex remains fail-closed.

The exact default-block restoration inventory is 363 direct `<br>`-parent `div`s:
six in each of the 60 sources matching
`subgrid_baseline_{nested_block,vertical_nested,auto_rows,vertical_auto_rows,inline_column}_*`,
ported from pinned WPT `subgrid-baseline-005` through `-009`, and three in
`fri06_inline_unequal_line_alignment.html`. In the audited baseline each has no
authored display and is made flex only by the shared corpus stylesheet. In the
final fixture HTML each exact parent authors inline `display:block`, which Chrome
interprets normally; no helper or Rust path selects a source, parses CSS, or
restores display from topology. This preserves the pinned/default input and
creates no blockified-`<br>` role, control, metric, or production special case.

The 240 subgrid-baseline variants in that inventory are also final-lineage
controls for `D-18`. [CSS Grid Layout Level 2, Subgrids](https://www.w3.org/TR/css-grid-2/#subgrids)
defines a subgrid as empty for track sizing in its inherited axis while its
descendants participate directly, with each subgrid edge and half-gutter
difference accumulated as an additional margin layer. The production model uses
that direct-participation rule for both intrinsic sizing and final baseline
alignment; it does not reconstruct the ancestor result by publishing a reduced
child group.

The private algorithm phases are:

| Phase | Required value and permitted transition |
| --- | --- |
| `FlattenedScalarContribution` | One descendant's intrinsic size in one ancestor axis, including its accumulated edge and half-gutter margins. A fully inherited subgrid root is recorded only as the scalar node suppressed from the ordinary direct-item pass; suppressing it does not suppress any descendant. |
| `AncestorBaselineMember` | One participating direct item or flattened descendant, carrying source node, `GridAxisKind`, physical baseline axis, ancestor track span, selected startmost/endmost track, first/last role, containing-logical distance, and the accumulated start/end edge and half-gutter adjustments used to obtain that distance. Construction performs the only physical-to-containing-logical conversion. |
| `AncestorBaselineGroup` | The immutable per-track first/last reduction of `AncestorBaselineMember` values in one ancestor grid. It supplies intrinsic baseline shims and final shared alignment targets. A fully inherited subgrid root is never an additional member. |
| `ChildBaselineEnvelopeView` | A non-publishable slice of an `AncestorBaselineGroup`, mapped to one child's local track order and logical direction. It can align child-local items but cannot enter ancestor reduction or scalar sizing. |

One `GridAxisKind`-parameterized traversal produces the scalar and member records
for rows or columns. First-baseline members select the startmost compatible
ancestor track and add the accumulated adjustment on that mapped start edge;
last-baseline members select the endmost compatible track and add the accumulated
adjustment on that mapped end edge. Reversal changes track and edge mapping, not
the stored first/last preference. Positive, zero, and negative gutter differences
therefore accumulate once while traversing outward. No later phase subtracts or
re-adds them.

Intrinsic sizing reduces the flattened members before applying per-contribution
baseline shims. Each leaf's scalar contribution is applied exactly once, the
fully inherited root is omitted from the ordinary scalar pass exactly once, and
the reduced ancestor group remains available even when that root scalar is
omitted. This distinction must produce the 411px auto-row control rather than the
459px duplicate-size result or the 492px phase-mixed result.

Final layout obtains one immutable ancestor group after tracks and item baseline
facts are known, derives child envelope views from it, and aligns each affected
item once. Repeating view derivation and placement with unchanged inputs produces
the identical groups and geometry. Fully inherited axes do not use the former
`publish_row_baseline_groups` inverse or an inherited-baseline fixed-point loop;
ordinary non-inherited grid-container baseline publication remains unchanged.
The containing grid sizes refreshed areas and projects the resulting logical
offset once; child axes govern only child-internal layout and baseline reading.

Named focused tests prove:

- one public nested-block computation reports the independently serialized
  descendant/direct-sibling tuple `(62, 110)`, with clean-base RED `(57, 125)`;
- the corresponding ancestor row's last-baseline member distance is 40, while
  the direct item contributes 25 and therefore does not win;
- auto-row scalar/member/group separation produces root height 411 and omitting
  the root scalar does not omit its descendants;
- LTR/RTL inline-column groups report x 470/x 527 through the same operation;
- vertical auto rows retain the 18px envelope, area 381, child width 371, and
  x 196, while vertical nested placement reports x 153;
- positive, zero, negative, reversed, and margin/border/padding traversal maps
  each member once and repeated envelope-view derivation is identical; and
- the former inherited-publication round-trip test is removed rather than
  updated: it exercises the superseded `D-17` inverse, not a product contract.

The helper emits atomic participation only when the computed/lowered child role
is atomic, never from authored inline display after blockification. Typed inline
children replace, rather than accompany, the same legacy measured-text fallback.
Fixture sources exercise the current overflow-based BFC predicate and never
require later-owned `flow-root` normalization. Intermediate diagnostics may
synthesize the finite anonymous grid text wrapper, secondary inline boundaries,
and containing strut required by the fixed matrix. The final lineage instead
serializes each such layout-ready fact explicitly from an authored finite
`data-surgeist-*` marker or derives it through a generic input-only rule over the
computed/lowered role. The generated-XML parser never dispatches on the test or
source name, and parsed expectations are not passed to, inspected by, or
structurally mutated during input lowering. HTML parsing, CSS interpretation,
and DOM topology remain owned by Chrome and the browser helper: the Rust
generator must not reconstruct tags, attributes, nesting, style declarations,
comments, raw-text elements, or marker placement from HTML source. The
serializer normalizes any transparent browser-only wrapper before writing the
independent input and expectation trees. Renaming a test or mutating only
expectations must leave the parsed layout input identical; removing or corrupting
a required explicit fact must fail closed rather than restore synthesis. Every
lowered text or atomic participant receives level zero from the explicit
layout-ready-inline adapter contract unless one consumed source-indexed marker
supplies another level. Lowering reads computed direction only to activate an
explicit `whenDirection` record; it never derives the record's level or reads
text content, geometry, fixture identity, or expectations to choose one. Float
and clear lowering uses this closed table; the public `Left`/`Right` variants in
the layout-ready model mean line start/end:

The four formerly synthesized facts use only this closed final-lineage schema:

| Layout-ready fact | Authored HTML marker | Serialized input | Validity and absence behavior |
| --- | --- | --- | --- |
| Anonymous grid text wrapper | `data-surgeist-anonymous-grid-text-wrapper="true"` on the exact grid element | `layout-ready-anonymous-grid-text-wrapper="true"` on that generated box node | The value is exactly `true`; computed display establishes grid or grid-lanes formatting; the marked node has only the reviewed direct typed-text child shape and no raw-text fallback. Unsupported value, role, duplicate lowering, or mixed fallback rejects in the browser helper. Final full-run marker-use accounting rejects a missing marker on the five reviewed source stems. |
| Transparent secondary inline container | `data-surgeist-transparent-inline-container="true"` on either reviewed inline `bdo` | The container box is absent from `<input>`; one `<inline-boundary kind="start"/>`, its one direct typed-text child, and one `<inline-boundary kind="end"/>` appear in its source position | The value is exactly `true`; computed display is `inline`; source tag is `bdo`; there is exactly one direct shaped-text child and no other text, box, or control. The helper applies the same input-only transparent projection before independently serializing expectations. Invalid role/topology rejects. The exact bidi source inventory rejects either missing marker. |
| Explicit containing strut | `data-surgeist-inline-struts` on the layout-ready containing root, containing a nonempty JSON array of `{ "beforeSourceIndex": N, "baseline": B, "lineHeight": H }` | `<inline-boundary kind="start" inline-baseline="B" inline-line-height="H"/>` immediately before the one lowered child selected by DOM `sourceIndex` | Each object has exactly those fields; `N` is a unique existing child-node index that lowers to one typed atomic child; `B` and `H` are finite, `H > 0`, and `0 <= B <= H`. Missing target, duplicate target, extra field, nonfinite/out-of-range metric, or non-atomic target rejects. Exact-source inventory requires the reviewed mixed-wrap and float-line records; no topology or fixture name restores an absent record. |
| Explicit nonzero bidi level | `data-surgeist-inline-bidi-levels` on the direct parent whose child-node source indices it addresses, containing a nonempty JSON array of either `{ "sourceIndex": N, "bidiLevel": L }` or `{ "sourceIndex": N, "bidiLevel": L, "whenDirection": D }` | The addressed shaped segment or atomic placeholder carries `bidi-level="L"` instead of the adapter's level zero when the record is unconditional or computed direction equals `D`; a nonmatching scoped record leaves the level at zero | `N` is unique within the table and names one existing direct shaped-text or atomic participant; `L` is an integer in `1..=125`; optional `D` is exactly `ltr` or `rtl`. Every applicable record is consumed once; every inactive scoped record is still validated and reported in marker-use accounting. Missing target, duplicate target, unknown/partial fields, zero/out-of-range level, invalid direction, nonparticipant target, or unused applicable record rejects. Computed direction selects only an explicit scoped record and never supplies `L`. |

The authored marker inventory is exactly:

| HTML path under `tests/layout/browser_parity/html/` | Required marker count and placement |
| --- | --- |
| `subgrid/subgrid_baseline_auto_columns_first_item.html` | Exactly two anonymous-wrapper markers, one on each direct inline-grid item under the sole direct inline-grid subgrid child of `#test-root` |
| `subgrid/subgrid_baseline_auto_columns_second_item.html` | Exactly two anonymous-wrapper markers at the same two-item topology |
| `subgrid/subgrid_baseline_standalone_axis_first_item.html` | Exactly two anonymous-wrapper markers at the same two-item topology |
| `subgrid/subgrid_baseline_standalone_axis_second_item.html` | Exactly two anonymous-wrapper markers at the same two-item topology |
| `subgrid/subgrid_auto_track_sizing_min_content_text_runs.html` | Exactly one anonymous-wrapper marker on the innermost grid that directly owns the four typed text runs, beneath the sole min-content outer grid |
| `block/fri06_bidi_mixed_inline.html` | Exactly two transparent-inline-container markers, one on each direct inline `bdo` child of `#test-root`, plus one `data-surgeist-inline-bidi-levels` record `{ "sourceIndex": 0, "bidiLevel": 1 }` on the RTL `bdo` |
| `block/fri06_atomic_inline_percentage_block_size.html` | Exactly one root `data-surgeist-inline-bidi-levels` marker containing three records: `{ "sourceIndex": 0, "bidiLevel": 1, "whenDirection": "rtl" }`, `{ "sourceIndex": 1, "bidiLevel": 1, "whenDirection": "rtl" }`, and `{ "sourceIndex": 2, "bidiLevel": 1, "whenDirection": "rtl" }` |
| `block/fri06_inline_mixed_text_atomic_wrap.html` | Exactly one root `data-surgeist-inline-struts` record: `{ "beforeSourceIndex": 2, "baseline": 14.8, "lineHeight": 20 }` |
| `float/fri06_float_line_exclusion.html` | Exactly one root `data-surgeist-inline-struts` record `{ "beforeSourceIndex": 5, "baseline": 12, "lineHeight": 20 }` and one root `data-surgeist-inline-bidi-levels` record `{ "sourceIndex": 4, "bidiLevel": 1, "whenDirection": "rtl" }` |

The browser helper validates marker values, roles, metrics, and topology against
Chrome's actual DOM and emits source-local marker-use facts independently of
geometry expectations. During the final full generation, the Rust generator
compares those helper-reported facts and their source paths with this exact
nine-source inventory. Missing, extra, duplicate, or differently placed facts
and use on any other HTML source reject the run. Rust may pin exact source bytes
or literal marker counts as diagnostic drift evidence, but such checks neither
parse HTML nor constitute final inventory acceptance. The four generated
variants of each source inherit the same authored marker inventory; direction
selects only the four explicitly scoped record entries above, and box sizing does not
alter it.

The only new generated-XML structural forms for these facts remain
`layout-ready-anonymous-grid-text-wrapper` and the closed `inline-boundary`
element. The bidi marker feeds the existing required `bidi-level` attribute on
each shaped segment or atomic placeholder: the helper writes zero when no
nonzero marker applies, and the parser retains its existing `0..=125`
validation. An `inline-boundary` permits only `kind="start"` or `kind="end"`;
metrics are either both absent or the complete finite
`inline-baseline`/`inline-line-height` pair, and only a `start` boundary may carry
metrics. It has no payload or expectation node and is always a canonical non-box
input. Unknown attributes, partial metrics, payload, or invalid placement reject.
No final parser helper accepts the test name and mutable input plus mutable
expectations in one call.

| Fixture token | Layout-ready value |
| --- | --- |
| `none` | `Float::None` or `Clear::None` |
| `inline-start` | `Float::Left` or `Clear::Left` |
| `inline-end` | `Float::Right` or `Clear::Right` |
| `both` | `Clear::Both`; invalid for float |
| physical `left`/`right` | Map to `Left`/`Right` only when that physical side equals the containing `FlowAxes::inline_start()`/`inline_end()`; otherwise reject as an unsupported fixture-lowering form |

Consequently, `HorizontalTb` LTR maps physical left/right to line start/end and
`HorizontalTb` RTL maps them to line end/start. For both directions of
`VerticalRl`, `VerticalLr`, `SidewaysRl`, and `SidewaysLr`, physical left/right
are block-axis sides and fail closed for both float and clear rather than being
reinterpreted as inline sides. Focused adapter coverage enumerates all five
writing modes by both directions for float and clear, including the eight
vertical/sideways negative mappings, the two horizontal direction-sensitive
mappings, the direct line-relative aliases, `none`, and clear-only `both`.

This lowering is a finite `FlowAxes` projection, not a style engine. The adapter
must not become a general text shaper, CSS parser, bidi implementation, display
normalizer, or alternate line algorithm. Rust parser and serializer changes
remain exact to these finite categories and attributes. Final parity evidence
proves layout calculation from the serialized layout-ready input; it does not
claim that this crate owns HTML/CSS-to-layout composition.

Final generated artifacts have one full, unfiltered lineage after all owned
HTML/parser/helper/fixture inputs are settled. Their report has `filter: null`,
the reviewed existing-pinned browser version/provenance, the replacement
manifest/helper hashes, and no scoped report. That lineage uses the repository's
no-fetch existing-pinned mode; managed acquisition-capable generation and a
substituted browser are outside FRI-06. Only the one-lineage outcome under the
no-redundant-generation constraint qualifies as final evidence.

The final report retains exactly the 16 immutable missing-`#test-root` variants
and no other unsupported case. All 340 mixed-text/element, vertical `<br>`, and
`<br>`-outside-block variants leave unsupported accounting, and all 48 named
FRI-06 variants enter active comparison. Any count, source, variant, reason, or
bucket difference blocks closure rather than being reclassified in the final
evidence.

### 11.2 Activation Recovery Membership

The C10/C11 recovery boundary is the exact 388-row activation union with sorted
`source<TAB>variant` SHA-256
`3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1`.
Its immutable supporting records are:

- `plans/P01-layout/P01-I06-S01-C10-public-comparison-census.tsv`, SHA-256
  `0630d2606f1e53c56b69cd226665b899bbfd96ed60ad7ac3c80ec5d9423b5691`;
- `plans/P01-layout/P01-I06-S01-C10-post-generation-census.md`, SHA-256
  `2c4179f559c5fa9e93c6933e0ba1a4969b758fc4a4f738d619c2751796b8bf00`;
  and
- `plans/P01-layout/P01-I06-S01-C10-second-lineage-census.md`, SHA-256
  `a56b09ed4d68ee901dbc385db3d78b66bf5faeb82f844f1d531c94aef10a23b9`.

This specification owns the corrected recovery partition below. Each row-set
digest is over sorted, LF-terminated `source<TAB>variant` rows:

| Recovery partition | Rows | Row-set SHA-256 |
| --- | ---: | --- |
| Passing control | 104 | `29f9cf9ac175c105317ff38a183048a1f0429707e22fd3b076d85b455e6504a1` |
| Blockified-BR ordinary-box helper/serializer input | 244 | `eb9c8d005c76b0a52d9333fb39710f4b8f263189b88d74cf6f2ba7922b768460` |
| Range explicit-root coordinate translation | 18 | `ae9121d16226cabbb602c2f326fb5cfa1034c23f612104600cba560d7fa80b23` |
| Direct-root RTL physical placement | 2 | `a0620971c825fe0be6909c2331add26e26478c9970e1bd9eb4ff5b8d28321b40` |
| Range root-wide line identity | 4 | `9c3930435c2a2c65d8bf87bdf22f082de00857e7727928140aec3a744d52238e` |
| Shape-fixture explicit atomic break | 4 | `0b96e7d9a39716b0121017cdbe67345381d72044918c9cef5b31ec216364de18` |
| Mixed-wrap continuation strut | 4 | `8a59f6f6231bcf5478f51ab9fe200169ba81198c817f923116534e09d268facc` |
| Vertical line placement | 4 | `e2c95514201e376def0b87d6ad61940d719e5d1f84526bae47b814cfa90a9a79` |
| Float-line final height | 4 | `7b4fc8b3bb27f912d3f39d2aadc05c243ead274fed54c20dfa43bd0825f7c61f` |

The nine row sets are pairwise disjoint, total 388, and their union has the
activation digest above. This corrected literal partition supersedes every
earlier inferred pass/fail or category narrative. Later work may not add, omit,
reclassify, or dynamically widen a row. Aggregate failures outside this union
remain FRI-13-owned. The pinned records are immutable evidence, not independent
planning authority or acceptance lineage.

### 11.3 Browser-Control Endpoint Observability

Browser neighboring-line observations are compared to model geometry only when
the model relation is observable. An endpoint is unobservable exactly when all
of these predicates hold after ordinary node geometry has compared:

1. the model source is a visible forced break with a zero-size output;
2. both adjacent model neighbors have unrounded output;
3. the control point is within the existing `0.1` tolerance of the shared
   physical block endpoint of both neighbor intervals; and
4. closed overlap therefore classifies both model relations as `Same` while the
   serialized browser observation distinguishes the following line.

The comparator preserves the browser value, reports the field through a typed
endpoint-unobservable result, and skips only that non-equivalent categorical
equality. It does not change the closed relation, model output, input parser,
fixture, helper, or generated XML. The activation test accounts for exactly 144
such fields across the inline-column, vertical-auto-row, and vertical-nested
subgrid families; every other browser-control field remains directly compared.
The private line-builder regression independently proves that the control ends
the preceding source slice and the following atomic begins the next line. Any
different count, family, missing exact node-geometry comparison, or non-endpoint
reason is a failure. This capability state is neither an expected-fail nor an
entry in the known-Chrome-failure registry.

## 12 FRI-06.12 Root And Sibling Handoff

At inspected root revision `19590f6d9fa01c0df197c5ef07fb626c5cf18ced`,
root pins `surgeist-text@754707f27feb04fb7ff31e0574ff43ded552d360`.
That text crate owns full paragraph shaping/line-breaking output and inline box
facts, while root has style-to-text and style-to-layout adapters but no composed
text-to-layout inline participant path.

The required upstream transition is:

1. `surgeist-text` adds a source-derived, validated pre-line-layout projection
   exposing shaped indivisible segment extent, line metrics, bidi level,
   whitespace edge classification, break opportunity, optional shaped break
   replacement, and caller-local text-segment/atomic-placeholder association. It does not
   expose backend-native Parley objects through its public contract.
2. The text projection preserves text source identity/revision and maps every
   segment ID back to source cluster/glyph data inside `surgeist-text`; layout
   receives only local IDs and geometry facts.
3. Root composes retained text nodes and atomic inline children into exact
   `InlineTextInputOf` and `AtomicInlineParticipationOf` facts, pairs every text
   and control node with `NodeInputOf::non_box()`, retains affected nodes as
   dirty subjects when text source/style/font generation changes, invokes
   `compute_layout_invalidated`, immutably prepares then infallibly commits the
   successful batch, and maps final fragment output back to text
   render/selection data.
4. `surgeist-style`/root lower the currently supported baseline/top/bottom subset
   without broadening FRI-09 alignment. Root normalizes line-break and boundary
   flow from the containing inline context, not the `<br>` node's own direction.
5. For shape-backed floats, root uses `surgeist-shape` or another root-selected
   safe provider to answer the exact band query. It marks every affected float
   dirty when shape, shape margin, reference box, transform, provider result, or
   provider failure state changes and retains that dirty state through any
   failed layout or batch preparation.
6. Missing text projection, atomic association, or requested shape provider
   returns a typed root adapter/layout error. Root never falls back to a measured
   text leaf, margin-box shape, horizontal flow, zero metrics, or default style.
7. Root retains ownership of authored CSS parsing, anonymous inline/box
   generation, DOM/text association, render glyph data, hit testing, selection,
   accessibility, and invalidation.
8. Root adapters target exact compatible text and layout revisions, update
   facade/docs/examples/tests, and regenerate root-owned API artifacts from those
   pinned sources.

Any required `surgeist-text` or root work remains outside this leaf initiative.
This specification does not authorize edits to text, shape, style, CSS, retained,
render, or root repositories.

## 13 FRI-06.13 Finding Traceability

| Finding | Required closure evidence |
| --- | --- |
| `BLOCK-014` | All ten flow mappings and four clear states complete without panic and match focused geometry |
| `FLOW-002` | One logical line builder soft-wraps, forces breaks, aligns, clears, and projects vertical/sideways lines |
| `BLOCK-004` | Every ordinary line queries active float bands; old overlap characterization is replaced by browser-backed placement |
| `FLOW-001` | Typed shaped segments participate with atomic boxes/controls and publish source-associated fragments; no measured-text substitute remains |
| `FLOW-003` | Float placement, clear, BFC avoidance/auto width/auto height, nested float containment, logical sides, and shape-provider exclusion have focused proof |
| `BLOCK-005` | Fixture/root lowering derives control flow from the containing inline context and invalid mismatches remain typed diagnostics |
| `BLOCK-006` | Following-line strut and baseline survive leading/trailing/adjacent forced breaks |
| `BLOCK-008` | Unequal lines use independent band, used extent, and legacy alignment offset |
| `BLOCK-009` | Top/bottom are independent line-edge alignment states and top no longer becomes a zero baseline |
| `BLOCK-011` | Atomic visible-inner and margin-edge/non-visible fallback baselines are exact with bottom margins |
| `BLOCK-012` | Atomic percentage block size receives the definite containing basis and preserves indefinite behavior |
| `BLOCK-013` | Metric text/controls prevent baseline-erasing fixed-size fast paths |
| `TEST-003` | Comparator checks every visible line-break/boundary position and fails on wrong or missing output |
| `TEST-004` | Owned mixed-text, vertical/outside-block BR, and active float/BFC surfaces leave unsupported accounting and pass active comparison |

FRI-06 is incomplete while any row lacks its named source, front-door test, and
applicable browser/artifact evidence.

### 13.1 Mechanical Containment Contract

The immutable mechanical-review evidence is
`plans/P01-layout/P01-I06-mechanical-refactoring-review-findings.md`, SHA-256
`11437dd9dfe83d41ae6b01e41453d9cc1a893172c6977e5b3d77346aa3948f34`.
Its six findings are behavior-preserving opportunities, not new correctness
findings and not authority to change public API, dependencies, features, MSRV,
fixtures, generator logic, generated artifacts, or repository ownership.

Two bounded containment results are already part of the required FRI-06 state:

- C03 realizes `MR-006` and only the scalar-generic `OracleTreeOf<S>` slice of
  `MR-002` under the retained contract at
  `plans/P01-layout/P01-I06-S01-C03-post-c02-sprawl-containment.md`, SHA-256
  `0c88ec011067e25d61d0ddfdf90ad47e9e5db0149dbdc214e8326668665711e4`.
  One private non-box classifier preserves node-input, child, and measurement
  first-error order; one generic oracle implementation preserves both scalar
  lanes and every specialized observation/failure behavior.
- C07 realizes `MR-001`, `MR-004`, and `MR-005` under the retained contract at
  `plans/P01-layout/P01-I06-S01-C07-post-c05-sprawl-containment.md`, SHA-256
  `3e9d894791ffc3fa9ce772350a7fd9d667979dc5d86c37affebc2922b8c1322d`.
  Shaped-text validation and selection remain linear without changing operation
  order; scroll-padding and geometry-error glue preserve physical edges, sites,
  run modes, and dependency direction; signed zero, layout rounding, and
  physical-edge selection have one crate-private policy each.

The final C13 containment result evaluates the same immutable six-item set at the
completed C12 source. Each row has one exact disposition: its already-realized
contract remains present and characterized; its remaining equivalent duplication
is consolidated under the rule below; or a named current-source counterexample
proves that the original equivalence predicate no longer holds. Absence of a
disposition is not closure.

| Item | Final required disposition |
| --- | --- |
| `MR-001` shaped-text processing | Preserve C07's source-order first-duplicate result, allocation-free intrinsic summary, incremental candidate scan, operation order, and deterministic linear scaling evidence. Remove only a reintroduced equivalent scan or allocation. |
| `MR-002` test tree harnesses | Preserve the one generic `OracleTreeOf<S>`. Classify every remaining local tree as an ordinary map-backed input tree or a specialized failure/observation/order/cache/topology fake. Consolidate the ordinary equivalent class through typed test support; retain each specialized fake with its distinguishing behavior visible. |
| `MR-003` layout math helpers | For option fallback/unwrapping, optional addition, aspect-ratio projection, resolution-to-zero/optional resolution, and containing-flow padding/border resolution, consolidate only call sites with identical percentage basis, scalar operation order, zero clamping, and min/max order. Keep policy-specific operations local and named; each unmerged candidate identifies the concrete differing predicate. |
| `MR-004` scroll and geometry glue | Preserve C07's one scroll-padding conversion and compute-owned own/child geometry adapters, including physical-edge mapping, site, run mode, error variant, and module dependency direction. Remove only reintroduced exact duplicates. |
| `MR-005` scalar and geometry primitives | Preserve C07's one signed-zero canonicalizer, exact `(value + 0.5).floor()` layout-coordinate rounding, and physical-edge selector. General scalar rounding, logical-edge selection, validation, and clamp policy remain distinct. |
| `MR-006` non-box validation | Preserve C03's one private reason classifier, exact first-error order, node/site payload, and role-specific parent handling. Remove only reintroduced exact duplicates. |

Mechanical consolidation is accepted only when all existing public geometry,
error, cache, fragment, scalar-lane, and parity observations remain unchanged.
The final C12 manifest, helper, report, and XML bodies remain byte-identical; C13
does not run or alter the generator. No macro-driven wholesale rewrite, broad
helper trait, public test API, new lint allowance, or executable `unsafe` is part
of this contract.

## 14 FRI-06.14 Initiative Acceptance

FRI-06 is complete only when:

1. all 14 owned findings satisfy `FRI-06.13` and remain assigned only to FRI-06
   in the findings-resolution index;
2. every public shaped-text, atomic participation, bidi, break, whitespace,
   fragment, float exclusion, canonical non-box pairing, and bottom-alignment
   state is intrinsically valid and scalar-generic;
3. mixed text, atomic boxes, line breaks, boundaries, floats, and out-of-flow
   placeholders form one deterministic source-associated participant stream;
4. prohibited, allowed, replacement, mandatory, edge-whitespace, overwide,
   min-content, and max-content cases have focused proof;
5. bidi reordering is per final line across shaped, atomic, boundary, and
   discarded-anchor units; fragment visual slots include intervening units,
   visible breaks remain at visual line end, and source association never
   changes;
6. all ten flow mappings soft-wrap and force-break through one logical line
   algorithm with per-line bands, metrics, and alignment;
7. all clear values map through containing flow and no valid vertical clear path
   panics;
8. rectangular and provider-backed shape exclusions, invalid shape/float roles,
   mismatched provider queries, raw endpoint rejection, float placement, BFC
   avoidance/auto width/auto height, nested containment, and logical sides have
   focused front-door proof;
9. baseline/top/bottom alignment, visible/non-visible atomic fallback, bottom
   margin, definite percentage basis, and fixed fast-path baseline behavior are
   exact;
10. completed batches publish immutable fragment outputs atomically with node and
    cache updates, zero-fragment text publishes its reviewed source anchor and
    zero geometry, and failures publish nothing;
11. unit cache context remains unchanged; changed text/style/tree/provider facts
    produce the exact dirty-subject ancestor closure; stale cache reads are
    bypassed; layout/preparation failure makes zero mutation and retains dirty
    state; infallible exclusive commit replaces all output/fragment state and
    clears closure caches before new stores; only successful `apply_to` releases
    caller dirty state; valid cold/warm and normal/rounded output agrees for
    lines, restored fragments, controls, floats, baselines, content size, and
    scroll contribution;
12. comparator negative controls detect wrong/missing explicit model-control and
    model-fragment geometry, source/line/visual identity, and baseline, while
    Range-ink and browser-control observations cannot masquerade as model block
    geometry or invent a model visual index; the `FRI-06.11.3` endpoint state is
    typed, accounts for exactly 144 fields, retains strict neighboring geometry,
    and has independent private forced-break line-membership proof;
13. final fixture lowering obeys the closed `FRI-06.11` marker/XML table;
    fixture names and expectations cannot influence parsed layout input;
    renamed-name and expectation-only equality controls pass; Chrome/helper
    validates actual-DOM marker semantics and the final full run accounts for
    every required source-local marker fact without a Rust HTML pre-parser;
    malformed or incomplete facts fail closed; computed direction only activates
    an authored `whenDirection` record and never derives a bidi level; and the
    final browser result is calculated from independently serialized layout-ready
    input; BR metrics come from the browser marker measurement above rather than
    the former font-ratio estimate;
14. absent an entry satisfying the complete `FRI-06.11` evidence contract, the
    known-Chrome-failure registry and expected-fail count are zero; every accepted
    entry has exact browser/correct values, certainty evidence, a passing
    public-front-door synthetic substitute, visible row accounting, and a future
    revalidation trigger;
15. the bounded HTML/parser/helper/fixture inputs and `D-18` production model
    settle before exactly one acceptance full regeneration; earlier
    assumption-failing full runs remain diagnostic, and subsequent acceptance
    checks are read-only and provenance-clean;
16. FRI-06-owned mixed-text, vertical/outside-block BR, and active float/BFC
    cases leave unsupported accounting and pass focused parity;
17. public exports and crate/parity docs describe the text/layout/shape/root
    ownership boundary without claiming authored CSS, shaping, rendering, or
    later initiative behavior;
18. default and generator-feature verification, focused parity, corpus/Taffy,
    docs, formatting, full configured Clippy with `-F unsafe-code -D warnings`,
    diff/provenance, and the tracked/non-ignored Rust unsafe scan are clean;
19. all FRI-06-owned dead-code allowances are removed and no new lint suppression
    or executable `unsafe` exists; and
20. the retained C03/C07 containment contracts and all six final dispositions in
    `FRI-06.13.1` hold without public behavior or artifact change; and
21. no dependency, feature, MSRV, generator architecture, root/sibling,
    FRI-09/10/11/12 behavior, FRI-13 aggregate gate, or unrelated change becomes
    part of FRI-06.
