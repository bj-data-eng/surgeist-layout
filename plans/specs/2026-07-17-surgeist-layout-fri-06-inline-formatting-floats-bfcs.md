# FRI-06 Inline Formatting, Line Boxes, Floats, And BFCs

Status: draft

Design owner: `surgeist-layout`

## FRI-06.1 Authority And Outcome

This specification is the authoritative desired-state contract for `FRI-06` in
`plans/specs/2026-07-11-surgeist-layout-findings-resolution-index.md`. It owns
the layout-ready inline participant boundary, mixed inline line construction,
line-box geometry, float exclusion, and currently representable block-formatting-
context behavior needed to close these 14 findings from
`plans/2026-07-10-surgeist-layout-full-code-review-findings.md`:

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

## FRI-06.2 Ownership And Non-Goals

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
and refreshes root-owned API artifacts after candidate promotion.

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

## FRI-06.3 Current Evidence

At the FRI-06 design base, current source already provides these prerequisites:

- `WritingMode` and `FlowAxes` cover five writing modes and ten
  writing-mode/direction mappings;
- `LineBreakInputOf`, `InlineBoundaryInputOf`, and `InlineMetricsOf` carry
  validated layout-ready control metrics and containing-flow metadata;
- `ContainingLayoutContext` and `ParentFormattingContext` carry containing flow
  and parent formatting role through cache identity;
- order-modified traversal and independent replacedness are complete;
- canonical scroll contribution accepts final physical participant geometry;
- block layout has a rectangular `FloatExclusions` model and horizontal clear
  segmentation; and
- the browser harness lowers constrained `<br>` controls and atomic inline
  display values.

The surviving source gaps are:

1. `LayoutInputOf` has only box, line-break, and boundary variants; there is no
   typed text participant or fragment output;
2. `InlineParticipant` has only atomic boxes and controls, and current line
   state cannot split a text source at supplied break opportunities;
3. horizontal lines use one maximum run extent for every line's physical
   projection, while vertical lines wrap only at forced breaks;
4. ordinary inline runs bypass float bands unless a clearing break exists;
5. vertical clear still has two `should_panic` characterizations;
6. `VerticalAlign` represents only baseline and top, and top is consumed as a
   zero baseline rather than independent line-over alignment;
7. atomic fallback baselines ignore margin-edge and used-overflow rules;
8. atomic percentage block size receives no definite containing basis;
9. fixed-size fast paths treat all controls as unable to establish baselines;
10. the parity comparator returns before checking line-break output; and
11. the canonical report retains unsupported mixed-text, vertical-break, and
    outside-block-break buckets that correspond to owned capability gaps.

The old finding claim that sideways writing modes and parent formatting roles
are absent is no longer true. FRI-06 consumes those completed contracts rather
than defining substitutes.

## FRI-06.4 Resolved Design Decisions

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
| `D-09` | Baseline, line-over (`top`), and line-under (`bottom`) are distinct algorithm states. FRI-06 adds `VerticalAlign::Bottom`; all other values remain FRI-09-owned and unrepresented rather than collapsed. |
| `D-10` | A forced break commits its current line and seeds the following line with the containing strut metrics. A break/control can establish first and last baselines even when no atomic/text participant exists. |
| `D-11` | Atomic inline fallback uses the block-end margin edge when no usable inner baseline exists. A non-visible used overflow forces fallback; visible used overflow may use the inner baseline. Top/bottom alignment is resolved after baseline line sizing. |
| `D-12` | Atomic inline percentage block size receives the containing block's definite physical/logical block basis when present. Anonymous inline-run size is never substituted as that basis. |
| `D-13` | Float left/right and clear left/right are line-relative values mapped by the containing `FlowAxes`; the public enum spellings remain source-compatible while algorithms do not treat them as physical x sides. |
| `D-14` | Margin-box float exclusion is internal and always available. Non-rectangular exclusion uses an explicit `FloatExclusion::Shape` input and a bounded `LayoutTree` provider query. Each returned interval retains its originating query privately; a mismatched query, missing provider, or provider failure is a typed layout error. |
| `D-15` | Float interaction is closed over the current model. An in-flow, non-floating, block-level child avoids active floats exactly when it is `Flex`, `Grid`, or `GridLanes`, or when it is non-replaced and its normalized computed overflow pair establishes an independent formatting context. Floats use the float path, atomic inline boxes use the line path while trapping their own internal formatting context, absolute boxes are excluded, and `None` produces no box. Future display roles do not enter this cycle. |
| `D-16` | Browser fixtures remain a finite adapter. FRI-06 activates the exact 340 currently unsupported variants identified below and adds exactly twelve named four-variant sources. Parser/helper/generator/comparator edits are permitted only for their shaped-segment/fragment, browser-observation category, finite anonymous/inline lowering, control, and exclusion facts. Intermediate diagnostics may synthesize bounded layout-ready facts, but final acceptance serializes those facts explicitly or derives them through generic input-only rules: fixture source/name and expected geometry never select, create, or alter layout input. Inputs settle first, then one full regeneration owns all XML/report deltas. |

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

## FRI-06.5 Public Model

### Shaped Segments

The crate adds these public scalar-generic input types and default-scalar aliases:

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

The public `LayoutInputOf<S>` adds `InlineText(InlineTextInputOf<S>)`,
`inline_text`, and `as_inline_text`. A text input is an inline participant, never
a box, leaf measurement, absolute child, float, or scroll container.

### Non-Box Tree Pairing

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

### Fragment Output

The crate adds:

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

`CompletedLayoutBatchOf` adds `unrounded_inline_fragments()` and
`final_inline_fragments()` backed by separate private vectors, matching its
existing node-output phases. The batch consumer atomically commits unrounded and
final node/fragment state before its cache updates. A failed layout returns no
node outputs, cache updates, or fragments.

### Float Exclusion Provider

`NodeInputOf<S>` adds a closed layout-ready field:

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

The crate adds validated query/result carriers:

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

`LayoutTree` adds this default method and layout calls it only for
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

### Vertical Alignment

`VerticalAlign` becomes:

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

The FRI-09 handoff must replace this subset with its reviewed complete typed
alignment model. FRI-06 does not add an unsupported enum variant to public input.

## FRI-06.6 Invariants And Errors

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
new exhaustive `LayoutOperation::FloatExclusionQuery` and
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

### Cache And Fragment State

FRI-01's invalidation model remains authoritative. `CacheKeyContext` stays the
zero-field unit value, `CacheKeyContext::new()` and
`LayoutTree::cache_context()` keep their signatures, and no text/style/shape or
provider revision token enters the cache key. The existing key continues to own
only recursive `ComputeInputOf` state plus that unit context.

The crate adds this public entry point while retaining `compute_layout` for a
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

To restore output on a valid warm hit, `LayoutTree` adds:

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

## FRI-06.7 Behavior Matrices

### Participant Matrix

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

### Break Selection Matrix

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

### Bidi And Whitespace Matrix

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

### Line Alignment Matrix

| `TextAlign` | LTR horizontal | RTL horizontal | Vertical/sideways |
| --- | --- | --- | --- |
| `LegacyLeft` | Physical/line-left | Physical/line-left | Containing-flow line-left |
| `LegacyRight` | Physical/line-right | Physical/line-right | Containing-flow line-right |
| `LegacyCenter` | Half non-negative free inline space | Same | Same logical rule projected by `FlowAxes` |

Each line uses its own float-adjusted start/end and used extent. Negative free
space never reverses or magnifies alignment; legacy center/right clamp their
offset to zero for overflow. FRI-09 owns start/end/match-parent/justify and
text-align-last expansion.

### Float And BFC Matrix

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

### Atomic Baseline Matrix

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

### Control And Clear Matrix

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

## FRI-06.8 Algorithm Contract

### Logical Line Builder

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

### Mixed Source Composition

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

### Float Bands

`FloatExclusions` becomes flow-relative and records each placed float's mapped
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

### Size, Baseline, Scroll, Cache, And Rounding

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

## FRI-06.9 Focused Evidence

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
| Comparator | Wrong/missing line-break position, text fragment, line index, visual index, and baseline each fail with named diagnostics |
| Fixture-input honesty | Exact-source inventories require every reviewed wrapper/container/strut marker; serializer tests prove the closed mapping in `FRI-06.11`; renamed test names and arbitrary expectation-only mutations preserve identical normalized parsed input; missing/corrupt markers fail source preflight or parser validation; transparent wrappers produce independent normalized input and expectation trees; static/runtime evidence proves no final input-lowering function reads fixture identity or expectations |
| Browser corpus | Owned mixed text/atomic/BR, vertical BR, float/BFC, unequal alignment, baseline, percentage atomic, and shape cases parse and compare |

Behavior changes require reconstructed RED evidence at each exact task base. A
test exercises `compute_layout`, block/inline public formatting, or the real
browser parity front door. Private line-builder tests supplement but never
replace front-door evidence.

The shape provider fake must implement the real query/result contract and expose
only query records that are themselves observable acceptance criteria. It cannot
return precomputed final line positions.

## FRI-06.10 Module And API Outline

| Module or artifact | Desired responsibility |
| --- | --- |
| `src/node_input.rs` | Shaped segment/text and atomic participation models, bidi/break/whitespace validation, canonical non-box input, float exclusion mode, bottom alignment, and `LayoutInputOf::InlineText` |
| `src/output.rs` | Fragment carriers, invalidation closure, phase-specific batch output, atomic sink contract, and source-preserving accessors |
| `src/traits.rs` | Bounded shape exclusion provider and committed unrounded-fragment readback methods on `LayoutTree` |
| `src/compute.rs` | Provider/error plumbing, invalidated entry point/path closure, cache bypass/staging, and cache/round fragment publication |
| `src/inline.rs` | One logical mixed-participant line builder, break/bidi/whitespace, line metrics, per-line alignment, and physical projection |
| `src/block.rs` | Inline composition, containing strut, float-band/BFC placement, percentage basis, fast-path baseline predicate, scroll contribution |
| `src/cache.rs` | Preserve FRI-01's unit key and support clear-then-store batch application with fragment restoration |
| Focused Rust tests | Model, line, block, root, cache, scalar, flow, provider, comparator, public surface, and failure evidence |
| `tests/layout/browser_parity/support.rs` | Exact shaped fixture lowering, fragment/control comparison, and named mismatch diagnostics |
| Existing generator/helper | Serialize/capture only the bounded new fixture facts; no architecture expansion |
| `corpus.toml`, HTML, XML, report | Active FRI-06 source inventory, one final full regeneration, provenance and bucket closure |
| `README.md`, crate/parity rustdoc | Layout/text/shape/root ownership and finite fixture boundary |

Private helper names and internal decomposition may differ. Public phases,
invariants, output association, provider failure, and ownership may not.

Every existing `#[allow(dead_code)]` in an FRI-06-owned inline/control path must
either become genuinely consumed or be removed with the dead item. No new lint
allowance, compatibility alias, test-only public API, or broad suppression is
permitted.

### Compatibility Classification

| Public surface | Compatibility and migration contract |
| --- | --- |
| `LayoutInputOf::InlineText` | Adding a variant breaks exhaustive downstream matches. Leaf algorithms, fixture adapters, root adapters, and facade-facing matches must handle text explicitly; no wildcard may reinterpret it as `Box`, measurement, line break, or boundary. |
| `NodeInputOf::non_box()` and non-box pairing validation | Additive constructor plus corrected validation for text, line-break, and boundary nodes. Existing callers must replace arbitrary/default box companions with the canonical value; no compatibility path ignores contradictory box fields, children, or measurement. |
| `compute_layout_invalidated`, `invalidated_nodes`, `LayoutBatchSink`, and `CompletedLayoutBatchOf::apply_to` | Additive transactional invalidation/application surface. Existing `compute_layout` keeps its signature but is valid only with no pending dirty nodes. Root migrates mutation-driven layout to the invalidated entry point and implements immutable fallible preparation plus infallible exclusive commit; no direct pre-layout cache clear remains. |
| `NodeInputOf::atomic_inline_participation` | Adding the public field breaks exhaustive struct literals. Its real default is `None`; callers using `NodeInputOf::default()`, `NodeInputOf::DEFAULT`, or Rust struct-update syntax retain no atomic participation, while every atomic child created by root or fixtures sets a validated `Some` explicitly. |
| `NodeInputOf::float_exclusion` | Adding the public field breaks exhaustive struct literals. Its real default is `MarginBox`; only a visible in-flow `Float::Left`/`Right` box whose tree can satisfy the provider contract sets `Shape`, and every other shape/role pair is rejected. |
| `VerticalAlign::Bottom` | Adding a variant breaks exhaustive downstream matches. Every leaf and root/facade match handles `Bottom` explicitly; no compatibility wildcard maps it to baseline or top. |
| New shaped-text, bidi, break, whitespace, fragment, and exclusion carriers plus default-scalar aliases | Additive names, private invariant-bearing fields, and reviewed constructors/accessors. `FloatExclusionIntervalOf` retains its originating query privately without changing construction or accessors. They have no legacy aliases or permissive conversions. Root facade/API artifacts expose exactly the reviewed names during the later root promotion. |
| `FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual }` | Breaking expansion of the existing exhaustive public construction/provider-output error. FRI-06 is an intentional pre-release breaking correction; leaf and later root exhaustive matches add the named state, and no wildcard or existing numeric error is reused for mismatched provenance. |
| `CompletedLayoutBatchOf::{unrounded_inline_fragments, final_inline_fragments}` and their new private storage | Additive read-only phase-specific output. External construction was already impossible; existing node/cache accessors keep their signatures and semantics. Root commits unrounded fragment state with unrounded nodes and consumes final fragment association for render/text integration. |
| Defaulted `LayoutTree` exclusion-provider method | Source-compatible for existing implementors and returns no provider result. Existing margin-box layouts never call it. Root and fixture trees override it only when supplying `FloatExclusion::Shape`; the default becomes a typed missing-provider error for a requested shape, never a silent margin-box fallback. |
| Defaulted `LayoutTree::unrounded_inline_fragments` | Source-compatible for existing implementors and returns `None`. Trees without inline text need no override; trees that commit inline text return `Some(slice)`, including `Some(&[])`, so valid warm hits can republish fragments. |
| New variants in existing `#[non_exhaustive]` invalid-input, missing-context, and internal-invariant error carriers | Additive under their existing non-exhaustive contract. Callers preserve their required wildcard handling; typed variants retain node/band/provider/cache context and no string compatibility error is added. |
| `LayoutOperation::{FloatExclusionQuery, CacheInvalidation}` | Adding variants breaks exhaustive downstream matches. Leaf and root/facade diagnostics handle them explicitly and retain the provider error plus container/float site or the unreachable dirty subject. |
| Existing line-break, inline-boundary, block, float, baseline, cache, and scroll behavior | Public shapes remain, but valid represented behavior is corrected. No legacy panic, whole-run alignment, measured-text substitution, skipped comparator output, or overlapping-float behavior is retained as a compatibility mode. |

The leaf cycle migrates every in-repository public match, struct literal, trait
implementation, test tree, and fixture adapter and proves the old omissions are
absent by source and compile evidence. Root and sibling repositories are not
edited here. The final candidate handoff enumerates every row above; a later root
promotion must update facade reexports, exhaustive matches, box/text adapters,
tree provider implementation, fragment consumption, cache invalidation, and the
root-owned generated API artifacts against the exact published leaf SHA. Root
integration evidence must show no old leaf pin, defaulted atomic placeholder,
shape fallback, or unhandled new enum state remains.

No Cargo feature, dependency, lockfile entry, MSRV, browser pin, launch profile,
generator architecture, or leaf-side generated API artifact changes for this
compatibility migration.

## FRI-06.11 Browser Fixture And Artifact Contract

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

FRI-06 also adds exactly these Surgeist-authored sources, each with all four
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

This adds 48 generated variants. Starting from 5,324 generated and 356
unsupported variants, the one settled replacement must therefore report 5,712
generated, 16 unsupported, and zero expected-fail, quarantined, or
failed-to-generate variants. The starting manifest SHA-256 is
`bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`;
the fixture task records its reviewed replacement hash after adding these twelve
records.

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

A browser `<br>` rect never supplies model control geometry. Explicit model-line
and model-control expectations remain strict. Model line-control participation is
a separate explicit fixture fact emitted only when the computed/lowered `<br>`
role participates as an inline control in that containing formatting context. A
source tag, ancestor activation marker, or browser-control observation alone does
not make a blockified `<br>` a model control. For a non-wrapping flex containing
context, browser terminal-slot and neighboring-line observations are compared
from source position and flex-line membership without consulting either browser
BR ink or model control-point geometry; wrapped flex remains fail-closed.

The helper emits atomic participation only when the computed/lowered child role
is atomic, never from authored inline display after blockification. Typed inline
children replace, rather than accompany, the same legacy measured-text fallback.
Fixture sources exercise the current overflow-based BFC predicate and never
require later-owned `flow-root` normalization. Intermediate diagnostics may
synthesize the finite anonymous grid text wrapper, secondary inline boundaries,
and containing strut required by the fixed matrix. The final lineage instead
serializes each such layout-ready fact explicitly from an authored finite
`data-surgeist-*` marker or derives it through a generic input-only rule over the
computed/lowered role. The parser never dispatches on the test or source name,
and parsed expectations are not passed to, inspected by, or structurally mutated
during input lowering. The serializer normalizes any transparent browser-only
wrapper before writing the independent input and expectation trees. Renaming a
test or mutating only expectations must leave the parsed layout input identical;
removing or corrupting a required explicit fact must fail closed rather than
restore synthesis. Float and clear lowering uses this closed table; the public
`Left`/`Right` variants in the layout-ready model mean line start/end:

The three formerly synthesized facts use only this closed final-lineage schema:

| Layout-ready fact | Authored HTML marker | Serialized input | Validity and absence behavior |
| --- | --- | --- | --- |
| Anonymous grid text wrapper | `data-surgeist-anonymous-grid-text-wrapper="true"` on the exact grid element | `layout-ready-anonymous-grid-text-wrapper="true"` on that generated box node | The value is exactly `true`; computed display establishes grid or grid-lanes formatting; the marked node has only the reviewed direct typed-text child shape and no raw-text fallback. Unsupported value, role, duplicate lowering, or mixed fallback rejects. A static exact-source inventory rejects a missing marker on the five reviewed source stems. |
| Transparent secondary inline container | `data-surgeist-transparent-inline-container="true"` on either reviewed inline `bdo` | The container box is absent from `<input>`; one `<inline-boundary kind="start"/>`, its one direct typed-text child, and one `<inline-boundary kind="end"/>` appear in its source position | The value is exactly `true`; computed display is `inline`; source tag is `bdo`; there is exactly one direct shaped-text child and no other text, box, or control. The helper applies the same input-only transparent projection before independently serializing expectations. Invalid role/topology rejects. The exact bidi source inventory rejects either missing marker. |
| Explicit containing strut | `data-surgeist-inline-struts` on the layout-ready containing root, containing a nonempty JSON array of `{ "beforeSourceIndex": N, "baseline": B, "lineHeight": H }` | `<inline-boundary kind="start" inline-baseline="B" inline-line-height="H"/>` immediately before the one lowered child selected by DOM `sourceIndex` | Each object has exactly those fields; `N` is a unique existing child-node index that lowers to one typed atomic child; `B` and `H` are finite, `H > 0`, and `0 <= B <= H`. Missing target, duplicate target, extra field, nonfinite/out-of-range metric, or non-atomic target rejects. Exact-source inventory requires the reviewed mixed-wrap and float-line records; no topology or fixture name restores an absent record. |

The authored marker inventory is exactly:

| HTML path under `tests/layout/browser_parity/html/` | Required marker count and placement |
| --- | --- |
| `subgrid/subgrid_baseline_auto_columns_first_item.html` | Exactly two anonymous-wrapper markers, one on each direct inline-grid item under the sole direct inline-grid subgrid child of `#test-root` |
| `subgrid/subgrid_baseline_auto_columns_second_item.html` | Exactly two anonymous-wrapper markers at the same two-item topology |
| `subgrid/subgrid_baseline_standalone_axis_first_item.html` | Exactly two anonymous-wrapper markers at the same two-item topology |
| `subgrid/subgrid_baseline_standalone_axis_second_item.html` | Exactly two anonymous-wrapper markers at the same two-item topology |
| `subgrid/subgrid_auto_track_sizing_min_content_text_runs.html` | Exactly one anonymous-wrapper marker on the innermost grid that directly owns the four typed text runs, beneath the sole min-content outer grid |
| `block/fri06_bidi_mixed_inline.html` | Exactly two transparent-inline-container markers, one on each direct inline `bdo` child of `#test-root` and nowhere else |
| `block/fri06_inline_mixed_text_atomic_wrap.html` | Exactly one root `data-surgeist-inline-struts` record: `{ "beforeSourceIndex": 2, "baseline": 14.8, "lineHeight": 20 }` |
| `float/fri06_float_line_exclusion.html` | Exactly one root `data-surgeist-inline-struts` record: `{ "beforeSourceIndex": 5, "baseline": 12, "lineHeight": 20 }` |

The source preflight rejects any missing, extra, duplicate, or differently placed
marker or strut record in this inventory and rejects these three marker names on
every other HTML source. The four generated variants of each source inherit the
same authored marker inventory; direction and box sizing do not alter it.

The generated XML parser recognizes only
`layout-ready-anonymous-grid-text-wrapper` and the closed `inline-boundary`
element above for these facts. An `inline-boundary` permits only `kind="start"`
or `kind="end"`; metrics are either both absent or the complete finite
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
substituted browser are outside FRI-06. The later fixture cycle plan owns the
exact diagnostic, command, blocker, and verification procedure under the
canonical workflow and the user's no-redundant-generation instruction.

The final report retains exactly the 16 immutable missing-`#test-root` variants
and no other unsupported case. All 340 mixed-text/element, vertical `<br>`, and
`<br>`-outside-block variants leave unsupported accounting, and all 48 named
FRI-06 variants enter active comparison. Any count, source, variant, reason, or
bucket difference blocks closure rather than being reclassified during the
fixture cycle.

## FRI-06.12 Root And Sibling Handoff

At inspected root revision `19590f6d9fa01c0df197c5ef07fb626c5cf18ced`,
root pins `surgeist-text@38001cc8effde426f06d7876f9e8eed1e082459a`.
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
8. Root promotes exact published text and layout candidates before adapter work,
   updates facade/docs/examples/tests, and regenerates root-owned API artifacts
   from the pinned sources.

This handoff may require a separate `surgeist-text` initiative and root cycle.
It does not authorize this leaf to edit text, shape, style, CSS, retained, render,
or root repositories.

## FRI-06.13 Finding Traceability

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

FRI-06 status cannot advance to complete while any row lacks its named source,
front-door test, and applicable browser/artifact evidence.

## FRI-06.14 Initiative Acceptance

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
    geometry or invent a model visual index;
13. final fixture lowering obeys the closed `FRI-06.11` marker/XML table;
    fixture names and expectations cannot influence parsed layout input;
    renamed-name and expectation-only equality controls pass; every required
    marker is inventory-pinned; malformed or incomplete facts fail closed; and
    the final browser result is calculated from the independently serialized
    layout-ready input;
14. the bounded HTML/parser/helper/fixture inputs settle before exactly one final
    full regeneration; subsequent checks are read-only and provenance-clean;
15. FRI-06-owned mixed-text, vertical/outside-block BR, and active float/BFC
    cases leave unsupported accounting and pass focused parity;
16. public exports and crate/parity docs describe the text/layout/shape/root
    ownership boundary without claiming authored CSS, shaping, rendering, or
    later initiative behavior;
17. default and generator-feature verification, focused parity, corpus/Taffy,
    docs, formatting, Clippy with `-F unsafe-code -D warnings`, diff/provenance,
    and the tracked/non-ignored Rust unsafe scan are clean;
18. all FRI-06-owned dead-code allowances are removed and no new lint suppression
    or executable `unsafe` exists; and
19. no dependency, feature, MSRV, generator architecture, root/sibling,
    FRI-09/10/11/12 behavior, FRI-13 aggregate gate, or unrelated change enters
    the reviewed range.

The final reviewed leaf candidate is published to remote `main`, read back, and
handed to root with the exact breaking public API, text/shape adapter contract,
artifact inventory, task/holistic reviews, verification evidence, and immutable
candidate SHA.
