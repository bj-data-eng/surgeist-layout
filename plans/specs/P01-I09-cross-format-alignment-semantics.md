# P01-I09 Cross-Format Alignment Semantics

Design owner: `surgeist-layout`

## 1 FRI-09.1 Authority And Outcome

This specification is the authoritative desired-state contract for `FRI-09`
in `plans/P01-layout/P01-index.md`. It closes exactly `MODEL-006` from
`plans/P01-layout/P01-initial-review-findings.md`.

The outcome is one layout-ready alignment model in which:

1. content alignment and content justification cannot represent each other's
   invalid values;
2. block, flex, grid, subgrid, and grid-lanes consume one shared alignment
   policy without erasing format-specific fallback rules;
3. flex and grid can coordinate first- and last-baseline content alignment;
4. inline layout supports logical line alignment, explicit last-line policy,
   shaping-owned justification opportunities, and the complete resolved
   vertical-alignment geometry set;
5. alignment adjustments participate in intrinsic sizing, caching, baselines,
   overflow, and scalar-generic physical output; and
6. finite browser-parity evidence proves lowering and geometry without fixture
   identity dispatch or browser-derived layout input.

This is an intentional breaking pre-release correction at crate version
`0.1.0`. Source compatibility with the incomplete `TextAlign`,
`VerticalAlign`, or `JustifyContent` alias is not a requirement. The new public
surface must make invalid property combinations unrepresentable rather than
preserve ambiguous compatibility aliases.

This specification supersedes stale source line numbers and the two-value
vertical-align inventory in the 2026-06-17 findings snapshot. FRI-06 already
added `VerticalAlign::Bottom`; that control remains useful but does not close
the missing resolved vertical-alignment states.

## 2 FRI-09.2 Ownership And Non-Goals

`surgeist-layout` owns:

- typed layout-ready alignment inputs and their validation;
- logical-to-physical alignment geometry for every supported writing mode and
  direction;
- line selection, line alignment, last-line selection, and distribution of
  explicit justification opportunities;
- inline vertical metric grouping and line-relative placement;
- block-container block-axis content alignment;
- flex-line and grid-area baseline content-alignment coordination;
- propagation through measurement, cache identity, baselines, scroll geometry,
  and committed output;
- focused, scalar, property, oracle, browser-parity, and documentation evidence;
  and
- removal of obsolete or ambiguous public alignment states owned by this leaf.

Root `surgeist` owns authored CSS grammar, cascade, computed values, font and
shaping data, script-specific justification opportunity discovery, root facade
composition, generated API artifacts, and the leaf gitlink. Root lowers its
resolved facts into the finite layout-ready types specified here.

The following remain outside FRI-09:

- static, relative, absolute, fixed, sticky, and anchor positioning behavior
  owned by `FRI-10`; positioned descendants remain negative controls;
- fragmentation owned by `FRI-11`;
- table layout and the normalized display-system work owned by `FRI-12A`
  through `FRI-12F`;
- aggregate release qualification and broad WPT import owned by `FRI-13`;
- font shaping, glyph positioning, language-specific justification rules,
  script analysis, line breaking discovery, bidi resolution, or rendering;
- authored `normal`, `match-parent`, `justify-all`, `sub`, `super`, percentage,
  or font-relative parsing when those values can be normalized upstream;
- masonry or provisional alignment behavior not represented by a published
  stable layout contract;
- dependencies, features, MSRV changes, unsafe code, a new generator, or broad
  generator architecture work; and
- reopening findings already closed by FRI-01 through FRI-08.

Generator changes are allowed only when narrowly required to parse or serialize
the exact finite FRI-09 fixture facts, add the finite fixture set, or fix a
confirmed genuine defect reached by those fixtures. Fixture name, source path,
variant name, expected geometry, or expected-fail status must never select or
alter layout input.

## 3 FRI-09.3 Initiative Base And Current Evidence

The initiative base is the remotely verified FRI-08 candidate
`3cc186ee37b762893ca0441548727191337244ca`. At specification time, local
`main`, `origin/main`, and the authority remote `main` were equal at that
revision, and the worktree was clean.

At that revision:

- `TextAlign` contains only `Auto`, `LegacyLeft`, `LegacyRight`, and
  `LegacyCenter`;
- line positioning converts those values directly into one physical offset and
  has no last-line or justification phase;
- shaped segments carry extent, metrics, bidi level, whitespace-edge state,
  and a following break, but no explicit justification opportunity;
- atomic inline participation likewise carries only bidi and following-break
  state;
- `InlineFragmentOutputOf` has no justification allocation for a renderer;
- `VerticalAlign` contains only `Baseline`, `Top`, and `Bottom`, and
  `InlineControlAlignment` mirrors that subset;
- `AlignContent` lacks baseline values while `JustifyContent` is an alias to
  `AlignContent`, so merely adding baseline variants would create invalid
  justify-content states;
- flex and grid pack lines or tracks using `AlignContent` but cannot coordinate
  child content against a shared first or last baseline;
- block layout copies `align_content` into input state but does not apply it to
  the in-flow contents; and
- `ComputeInputOf` and `CacheKeyOf` have no baseline content-alignment carrier,
  so an otherwise identical child re-layout could be incorrectly reused.

The browser-free canonical report at the base has schema version 3, 5,776
generated variants, 16 unsupported variants, three expected-fail source
records, zero quarantined, and zero failed-to-generate. There are 1,448 HTML
sources and 5,776 comment-free XML outputs. The frozen hashes are:

- corpus manifest:
  `c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6`;
- helper:
  `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`;
- `all.json`:
  `c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e`;
- XML inventory:
  `a98d1ccceaeeb336ee3cb3c0151607589c0a4ae0376a46c560ba4341f95ad6ae`;
  and
- report/XML lineage:
  `bad8e418caee72cc62a123dc93efe89fdb07bfb5dee4345f3df7d8fd6fe44fdf`.

These hashes define the immutable pre-FRI-09 artifact state. The conforming
post-FRI-09 state differs only through the closed additions and canonical
report facts specified in section 11.

## 4 FRI-09.4 Normative Product Authorities

The normative product sources are:

- CSS Box Alignment Module Level 3, especially content-distribution,
  baseline-content-alignment, fallback, overflow alignment, block-container,
  flex, and grid application rules: `https://www.w3.org/TR/css-align-3/`;
- CSS Text Module Level 3, especially `text-align`, `text-align-last`, and
  justification behavior: `https://www.w3.org/TR/css-text-3/`;
- CSS Inline Layout Module Level 3, especially `vertical-align`,
  `alignment-baseline`, `baseline-shift`, line-relative alignment, and inline
  box alignment: `https://www.w3.org/TR/css-inline-3/`;
- CSS Flexible Box Layout Module Level 1 baseline alignment rules:
  `https://www.w3.org/TR/css-flexbox-1/`;
- CSS Grid Layout Module Level 2 baseline alignment and track sizing rules:
  `https://www.w3.org/TR/css-grid-2/`; and
- CSS Writing Modes Level 4 for logical-axis and baseline orientation:
  `https://www.w3.org/TR/css-writing-modes-4/`.

When a draft and an existing repository contract differ, the implementation
must follow this specification's finite layout-ready boundary. It must not grow
an authored CSS parser inside the layout crate.

## 5 FRI-09.5 Public Alignment Model

### 5.1 Separate content alignment from content justification

`AlignContent` remains a public enum and gains exactly:

```rust
Baseline,
LastBaseline,
```

It retains the current positional, safe positional, stretch, and distribution
states. `Baseline` means first-baseline content alignment;
`LastBaseline` means last-baseline content alignment.

`JustifyContent` becomes a distinct public enum. It contains the current
non-baseline `AlignContent` values and never contains `Baseline` or
`LastBaseline`. Removing the type alias is mandatory. The two types may project
into one crate-private `ContentDistribution` policy, but callers cannot convert
baseline content alignment into content justification.

Both public enums expose only property-valid reversal, unsafe-position, and
safe-fallback operations. Shared internal interpretation must be centralized;
format modules must not maintain divergent copies of the same spacing formula.

### 5.2 Replace legacy text alignment with logical line alignment

The old `TextAlign` enum is removed. The public replacement is:

```rust
pub enum TextLineAlignment {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

pub enum TextLastLineAlignment {
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

pub struct TextAlignment {
    all: TextLineAlignment,
    last: TextLastLineAlignment,
}
```

`TextAlignment` has a const constructor, getters, and a default of
`Start`/`Auto`. `NodeInputOf::text_align` becomes
`NodeInputOf::text_alignment: TextAlignment`. No deprecated field or alias
retains the old ambiguous `Auto` and `Legacy*` surface.

Root lowering maps computed `text-align: normal` to the applicable logical
start policy; maps `match-parent` before entering the leaf; maps physical
left/right without rewriting them to logical values; and maps `justify-all` to
`Justify`/`Justify`. The leaf never guesses those authored distinctions.

### 5.3 Make resolved vertical alignment scalar-generic

The old `VerticalAlign` enum is replaced by
`VerticalAlignOf<S: LayoutScalar = DefaultScalar>` and the default-scalar alias
`VerticalAlign`.

It represents exactly these layout-ready states:

- baseline;
- a finite signed baseline shift;
- text-top with an explicit non-negative finite parent text-over distance;
- text-bottom with an explicit non-negative finite parent text-under distance;
- middle with an explicit non-negative finite parent x-height;
- line-top; and
- line-bottom.

Construction is through named constants or fallible constructors. Variant
payloads remain private. `Baseline` is the default. The public vocabulary uses
`LineTop` and `LineBottom`, not ambiguous `Top` and `Bottom` spellings.

Root resolves authored `sub`, `super`, lengths, percentages, font-relative
values, parent text edges, and x-height into these finite facts. A signed
baseline shift accepts any finite scalar. Parent over/under distances and
x-height must be finite and non-negative. Invalid input returns a typed error;
it never becomes zero, auto, or baseline silently.

`ShapedInlineSegmentOf<S>` gains a required
`vertical_alignment: VerticalAlignOf<S>` field. Its complete fallible
constructor receives the alignment with the segment extent, metrics, bidi,
whitespace, and break facts; its getter returns that exact value. Validation of
the alignment payload occurs before the segment can enter `InlineTextInputOf`.
`InlineTextInputOf` remains only the validated nonempty segment collection and
does not impose one alignment on every segment.

This segment field is the sole vertical-alignment owner for shaped text.
`NodeInputOf::non_box()` continues to be required for a shaped-text node and
requires the node-level `vertical_align` field to remain its default baseline,
so two carriers cannot conflict. Root/shaping copies each resolved inline
participant alignment into the segment constructor. The typed browser helper
emits the same per-source-index field described in section 11.2.

Atomic inline boxes continue to take their `VerticalAlignOf<S>` from their box
`NodeInputOf<S>`. Forced-break and inline-boundary controls continue to take it
from `LineBreakInputOf<S>`. Those inputs, block constants, and every other
alignment-bearing generic type use the same scalar lane. Converting an `f64`
vertical-alignment payload to `f32` through an implicit default alias is not
allowed.

## 6 FRI-09.6 Shaping-Owned Justification Contract

### 6.1 Opportunity input

Justification opportunity discovery belongs to shaping. The leaf adds a public
validated `InlineJustificationOpportunityOf<S>` with a default-scalar alias.
It stores one strictly positive finite distribution weight. There is no public
unchecked constructor.

`ShapedInlineSegmentOf` and `AtomicInlineParticipationOf` each gain a required
constructor argument and read-only getter of type
`Option<InlineJustificationOpportunityOf<S>>`. The opportunity applies after
that participant in source order. The complete shaped-segment constructor also
receives the vertical alignment from section 5.3. A named convenience builder
may supply `None`, but no internal heuristic may synthesize an opportunity from
whitespace, break kind, source text, segment identity, or fixture metadata.

The opportunity is a layout opportunity, not a glyph mutation instruction.
Root/shaping remains responsible for selecting legal opportunities and weights
for the script, language, font, and text-justify policy.

### 6.2 Eligible line selection

A selected line uses the all-line policy unless it is either paragraph-final or
terminated by a forced break. Such a line uses `TextLastLineAlignment`.
`Auto` resolves to logical start when the all-line policy is `Justify`, and to
the all-line policy otherwise.

A line is eligible for justification only when its resolved policy is
`Justify`, its available inline extent is definite, it has positive free space,
and it contains at least one surviving legal opportunity. An overfull line, an
indefinite line, or a line with no opportunity uses logical start. Justification
never contracts content in this initiative.

An opportunity after discarded trailing whitespace, after the final surviving
participant, or after a participant discarded by line selection is excluded.
An opportunity before a selected forced break remains eligible only if another
surviving participant follows it on that line.

### 6.3 Distribution and determinism

Positive free space is distributed in visual order in proportion to the
eligible positive weights. Arithmetic remains in `S`. The final visual
opportunity receives the residual after earlier allocations so the sum of
allocations equals the original free space exactly in that scalar lane.

Bidi reordering changes where cumulative offsets are applied, not which source
participant owns an opportunity. Every participant after an opportunity in
visual order receives its cumulative offset. Mixed direction, vertical and
sideways writing modes, forced breaks, float-narrowed bands, replacement break
extents, and discarded whitespace must use this same carrier.

### 6.4 Committed output

`InlineFragmentOutputOf` gains a read-only
`justification_inline_adjustment: S` getter. Its `rect` represents the used
layout advance of that shaped segment, including the allocation owned after the
segment. The adjustment lets a renderer distribute the extra advance inside
the shaped segment according to the same shaping contract.

For an atomic participant, its own border-box size does not grow. Its
opportunity shifts following visual participants, and the committed node output
retains the atomic box geometry. Fragment order remains source order; existing
`visual_index` remains the visual-order authority.

Justification affects line used extent, inline fragment rectangles, following
participant positions, content/scroll extent, and committed fragment state. It
does not alter intrinsic min-content or max-content measurement, because the
distributed amount exists only against definite positive free space.

## 7 FRI-09.7 Vertical-Alignment Geometry

### 7.1 Baseline-relative group

Inline layout first forms a baseline-relative metric group from baseline,
baseline-shift, text-top, text-bottom, and middle participants.

- Baseline aligns the participant baseline to the line baseline.
- Positive baseline shift moves the participant toward line-over; negative
  shift moves it toward line-under.
- Text-top aligns the participant's line-over edge to the parent text-over edge
  represented by the supplied parent metric.
- Text-bottom aligns its line-under edge to the parent text-under edge.
- Middle aligns the participant midpoint to the parent baseline plus one half
  of the supplied parent x-height toward line-over, projected through the
  current writing mode.

The line's before- and after-baseline envelopes include these shifts before the
line-relative group is solved. Atomic replaced baselines, overflow visibility,
inline boundary controls, forced breaks, and shaped text use the same metric
equations.

### 7.2 Line-relative group

Line-top and line-bottom participants are excluded from the initial baseline
envelope. After the baseline-relative line box is known, line-top aligns the
participant margin-box line-over edge to the line-over edge, and line-bottom
aligns its margin-box line-under edge to the line-under edge.

If a line-relative participant is larger than the current line box, the line
box expands and the group is solved to a fixed point with a finite monotone
envelope calculation. It must not alternate top/bottom placement or repeatedly
translate already-settled baseline participants.

### 7.3 Axis and output invariants

All seven states are logical. Horizontal, vertical-rl, vertical-lr,
sideways-rl, and sideways-lr project through `FlowAxes`; direction affects the
inline axis but does not reinterpret line-over and line-under. `f32` and `f64`
take the same branches.

Vertical alignment updates fragment and atomic node positions, line extent,
line and container baselines, content extent, scroll geometry, and committed
output as one operation. It never changes source identity or reconstructs font
facts from geometry.

## 8 FRI-09.8 Block-Container Content Alignment

### 8.1 Formatting-context boundary

For an ordinary block container, an authored non-default `align_content`
establishes an independent block formatting context. Its in-flow contents are
one alignment subject in the content box's block axis. Margin collapse through
that container is disabled; outside sibling collapse remains unchanged.

The default `None` preserves the existing normal block behavior and collapse
rules. This initiative does not turn every block into an independent formatting
context.

### 8.2 Subject and free space

The subject envelope contains all in-flow block children, inline line boxes,
and contained floats after normal block layout. It excludes out-of-flow
positioned descendants. The envelope is measured in logical block coordinates
relative to the content box and retains a nonzero or negative origin.

Content alignment applies only when the content-box block size is definite.
Indefinite free space falls back to start. Positive free space is consumed by
start, end, center, stretch, or distributed alignment as applicable to a single
subject; distribution values use their specification fallback for one subject.
Negative free space applies safe fallback before physical projection.

Baseline falls back to start when no parent-established shared baseline context
exists. Last-baseline falls back to safe end. A block container acting as a
baseline-aligned flex or grid item receives the shared adjustment described in
section 9 instead of treating baseline as an independent packing keyword.

Stretch does not resize arbitrary block children. For a single block-content
subject it follows the block-container content-distribution rule and otherwise
falls back to start.

### 8.3 Atomic translation

The resolved subject offset translates every in-flow child border box, inline
fragment, float, descendant baseline, and associated overflow/scroll
contribution together. It must not translate the container's own border box or
double-apply the offset to nested content.

Out-of-flow boxes remain excluded from the subject. Their complete static and
containing-block semantics remain FRI-10-owned. FRI-09 provides negative
controls proving that enabling block content alignment does not independently
move an explicitly positioned box or change its size.

Scrollable overflow preserves a reachable logical start region. Safe overflow
alignment clamps to start; an unsafe alignment may place the subject toward the
requested edge but must compose with the canonical signed-origin scroll
geometry rather than discard the start-side extent.

## 9 FRI-09.9 Baseline Content-Alignment Coordination

### 9.1 Typed adjustment carrier

The crate adds a private scalar-generic
`BaselineContentAdjustmentOf<S>`. It contains a logical axis and two finite
non-negative scalars, `before` and `after`. A first-baseline adjustment has
`before > 0` and `after == 0`; a last-baseline adjustment has `before == 0` and
`after > 0`; zero has both fields zero. Construction rejects non-finite,
negative, both-edge-positive, or axis-mismatched state.

`ComputeInputOf` carries this adjustment for recursive layout. `CacheKeyOf`
includes it exactly. Every child-input constructor either propagates an
explicit adjustment or deliberately supplies zero. Direct public leaf
constructors always supply zero.

The adjustment is the general compute-input form of the existing grid
`BaselineShim<S>`. Grid does not create a second shim type or a second baseline
group: it converts the existing `BaselineShim { before, after }` into the
validated compute-input carrier at the child-layout boundary. Flex constructs
the same carrier from its flex-line baseline reduction. The carrier does not
mutate public padding or expose a second public box model.

Let `b` and `a` be the before and after values in the child's logical alignment
axis. Let `C` be a definite unadjusted content-box available extent, `U` the
inner used extent produced after laying out descendants, `F` a first-baseline
distance from the inner logical start, and `L` a last-baseline coordinate from
that same inner start. The padding-equivalent equations are:

```text
inner_available = max(0, C - b - a)       when C is definite
auto_content_used = b + U + a
definite_content_used = C
first_distance_from_start = b + F
last_distance_from_end = a + (U - L)
inner_logical_origin = b
intrinsic_contribution = b + inner_intrinsic_contribution + a
```

For a definite content-box extent, the border-box used size remains unchanged
and only the inner available extent/origin changes. For an auto or intrinsic
extent, the used content extent grows by `b + a`. Percentages and child
available-size resolution use `inner_available`; the public authored padding
remains unchanged. Descendants, inline fragments, floats, and their baselines
are translated once by `b`. The after adjustment contributes to the used or
reserved end extent but is not a second descendant translation.

Scroll/content geometry is accumulated from the adjusted content-box start,
the translated inner subject, and the reserved after edge. Thus before-side
and after-side adjustments remain reachable and cannot be discarded by a
zero-origin size-only accumulator. Cold and warm layout expose the same
adjusted used size, baselines, descendants, fragments, and scroll geometry.

### 9.2 Eligibility and grouping

Flex baseline content alignment groups eligible items per flex line. Grid groups
eligible items in the startmost or endmost row or column appropriate to first
or last baseline alignment. The relevant child inline axis must be parallel to
the parent grouping axis. Replaced items without a usable baseline,
orthogonal items, auto-margin conflicts, and items whose coordinated
self-alignment makes them ineligible use the required positional fallback.

Subgrid items participate in their inherited ordinary-grid baseline group.
Grid-lanes participate only where their represented grid parent exposes the
same finite group; this initiative does not invent masonry-axis baseline
stacking.

The parent derives a target from unrounded child baseline distances:

- first baseline uses the maximum distance from the group start edge to the
  selected first baseline; and
- last baseline uses the maximum distance from the group end edge to the
  selected last baseline.

Each eligible child receives only the non-negative difference from its own
distance to the target:

```text
first: b = max(0, target_start_distance - child_start_distance), a = 0
last:  b = 0, a = max(0, target_end_distance - child_end_distance)
```

Missing baseline, missing group target, ineligible participation, zero
difference, or positional fallback produces the exact zero carrier. Direction
and writing-mode reversal are handled by logical edge projection, not by
negating the adjustment.

Grid and subgrid reuse the FRI-06 one-way model without modification of its
ownership:

1. `AncestorBaselineMember` remains the only flattened member census;
2. `AncestorBaselineGroup` remains the immutable owner-coordinate first/last
   reduction and supplies the existing intrinsic `BaselineShim`;
3. `CheckedOwnerToCurrentPlacementMap` remains the only owner-to-current track
   and frame mapping;
4. `InheritedCurrentGridBaselinePlacement` remains the only mapped final target
   for an item direct to an inherited current grid; and
5. `ChildBaselineEnvelopeView` remains downward-only and non-publishable.

For an owner-direct grid item, the immutable group target and member distance
produce the existing `BaselineShim`, which converts once into the compute-input
adjustment. For an inherited-current-grid item, the checked placement map first
derives the current-frame immutable target; that target and the current direct
witness distance then produce the same shim. Neither path reduces another
group, copies a group into current coordinates, mutates a target, feeds a child
view upward, or adds frame/gutter translation to `b` or `a` a second time.

### 9.3 Two-phase convergence

The parent performs an unadjusted measurement/layout phase, reduces the
immutable group target, derives the edge adjustment, and re-lays out only
children whose carrier changed. Intrinsic track sizing consumes the existing
grid `BaselineShim` exactly where FRI-06 already applies it; final child layout
consumes the equivalent compute-input carrier and must not add that intrinsic
shim again. The adjusted result participates in final line/track sizing and
container intrinsic contribution through those existing phase boundaries.

The group target is a maximum of pre-adjustment baseline distances. Applying
the non-negative difference makes every participating adjusted distance equal
to that immutable target, so no adjusted output is republished into group
reduction. Flex follows the same one-way census/reduce/apply rule. Any second
group reduction, adjusted-member publication, or retry loop is a design defect.

Parent line or track packing occurs after the baseline group is settled. It
must not use `Baseline` or `LastBaseline` as an ordinary spacing formula. When a
format lacks an eligible shared group, first baseline falls back to start and
last baseline falls back to safe end.

The child's selected baseline and adjustment are unrounded until the repository
rounding boundary. Cache hits must reproduce the same committed descendants and
inline fragments as a cold adjusted layout.

## 10 FRI-09.10 Shared Alignment Policy

The implementation has one private alignment-policy module or equivalently one
clearly owned shared source location. It owns:

- safe overflow fallback;
- positional reversal;
- one-subject distribution fallback;
- distributed offset and gap calculation;
- first/last baseline fallback; and
- logical start/end projection inputs.

Block, flex, grid, subgrid, and grid-lanes call that policy and retain only
format-specific subject construction and eligibility. Copying match arms or
spacing arithmetic across algorithms is not an acceptable result.

Alignment calculations use validated `LayoutScalar` values and return finite
geometry. They do not use `f32`-specific epsilon branches, saturating casts,
string keywords, or authored-style defaults.

## 11 FRI-09.11 Browser-Parity Boundary

### 11.1 Finite source set

FRI-09 adds exactly 18 authored HTML sources. The existing generator expands
each source into `border_box_ltr`, `border_box_rtl`, `content_box_ltr`, and
`content_box_rtl`; these are box-sizing/direction variants, not writing-mode
variants. The 18 sources therefore add exactly 72 XML outputs.

Writing mode is authored inside each source. Because every source receives both
LTR and RTL variants, sources 1 through 5 cover all five supported writing modes
in both directions. The closed source and marker-use inventory is:

| # | Exact source stem | Authored writing mode | Required cases | Explicit layout-ready marker records |
| --- | --- | --- | --- | --- |
| 1 | `fri09_text_logical_physical_alignment` | `horizontal-tb` | start, end, left, right, center, bidi order | none |
| 2 | `fri09_text_vertical_rl_alignment` | `vertical-rl` | start/end/physical-edge projection | none |
| 3 | `fri09_text_vertical_lr_alignment` | `vertical-lr` | start/end/physical-edge projection | none |
| 4 | `fri09_text_sideways_rl_alignment` | `sideways-rl` | start/end/center projection | none |
| 5 | `fri09_text_sideways_lr_alignment` | `sideways-lr` | start/end/center projection | none |
| 6 | `fri09_text_justification_weights` | `horizontal-tb` | unequal shaped and atomic opportunities | J: `0=1`, `1=2`, `2=1` |
| 7 | `fri09_text_justification_line_endings` | `horizontal-tb` | wrapped, forced, paragraph-final, explicit last line | J: `0=1`, `2=1`, `4=2`, `6=1` |
| 8 | `fri09_text_justification_bidi_trailing` | `horizontal-tb` | mixed bidi, visual residual, trailing discard | J: `0=1`, `1=3`, `3=1` |
| 9 | `fri09_inline_baseline_shift` | `horizontal-tb` | positive and negative shift | V: `0=baseline-shift(4)`, `2=baseline-shift(-3)` |
| 10 | `fri09_inline_parent_text_edges` | `horizontal-tb` | text-top and text-bottom | V: `0=text-top(8)`, `2=text-bottom(4)` |
| 11 | `fri09_inline_middle_and_line_edges` | `horizontal-tb` | middle, oversized line-top and line-bottom | V: `0=middle(10)`, `2=line-top`, `4=line-bottom` |
| 12 | `fri09_block_content_positions` | `horizontal-tb` | start, end, center, one-subject distribution fallback | none |
| 13 | `fri09_block_content_safe_overflow` | `vertical-rl` | safe/unsafe overflow and reachable start | none |
| 14 | `fri09_block_content_inline_float` | `horizontal-tb` | inline lines, floats, collapse boundary | none |
| 15 | `fri09_flex_first_baseline_content` | `horizontal-tb` | first group, wrap, orthogonal/no-baseline fallback | none |
| 16 | `fri09_flex_last_baseline_content` | `vertical-lr` | last group, auto-margin and replaced fallback | none |
| 17 | `fri09_grid_baseline_content` | `horizontal-tb` | row/column first/last groups and implicit tracks | none |
| 18 | `fri09_subgrid_baseline_content_controls` | `vertical-rl` | inherited target, no-baseline fallback, positioned negative control | none |

The exact corpus identities, repository-relative sources, and outputs are:

```text
case_id = "block/fri09_text_logical_physical_alignment"
source = "tests/layout/browser_parity/html/block/fri09_text_logical_physical_alignment.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_logical_physical_alignment__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_logical_physical_alignment__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_logical_physical_alignment__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_logical_physical_alignment__content_box_rtl.xml",
]

case_id = "block/fri09_text_vertical_rl_alignment"
source = "tests/layout/browser_parity/html/block/fri09_text_vertical_rl_alignment.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_rl_alignment__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_rl_alignment__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_rl_alignment__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_rl_alignment__content_box_rtl.xml",
]

case_id = "block/fri09_text_vertical_lr_alignment"
source = "tests/layout/browser_parity/html/block/fri09_text_vertical_lr_alignment.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_lr_alignment__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_lr_alignment__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_lr_alignment__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_vertical_lr_alignment__content_box_rtl.xml",
]

case_id = "block/fri09_text_sideways_rl_alignment"
source = "tests/layout/browser_parity/html/block/fri09_text_sideways_rl_alignment.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_rl_alignment__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_rl_alignment__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_rl_alignment__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_rl_alignment__content_box_rtl.xml",
]

case_id = "block/fri09_text_sideways_lr_alignment"
source = "tests/layout/browser_parity/html/block/fri09_text_sideways_lr_alignment.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_lr_alignment__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_lr_alignment__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_lr_alignment__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_sideways_lr_alignment__content_box_rtl.xml",
]

case_id = "block/fri09_text_justification_weights"
source = "tests/layout/browser_parity/html/block/fri09_text_justification_weights.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_justification_weights__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_weights__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_weights__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_weights__content_box_rtl.xml",
]

case_id = "block/fri09_text_justification_line_endings"
source = "tests/layout/browser_parity/html/block/fri09_text_justification_line_endings.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_justification_line_endings__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_line_endings__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_line_endings__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_line_endings__content_box_rtl.xml",
]

case_id = "block/fri09_text_justification_bidi_trailing"
source = "tests/layout/browser_parity/html/block/fri09_text_justification_bidi_trailing.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_text_justification_bidi_trailing__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_bidi_trailing__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_bidi_trailing__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_text_justification_bidi_trailing__content_box_rtl.xml",
]

case_id = "block/fri09_inline_baseline_shift"
source = "tests/layout/browser_parity/html/block/fri09_inline_baseline_shift.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_inline_baseline_shift__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_baseline_shift__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_baseline_shift__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_baseline_shift__content_box_rtl.xml",
]

case_id = "block/fri09_inline_parent_text_edges"
source = "tests/layout/browser_parity/html/block/fri09_inline_parent_text_edges.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_inline_parent_text_edges__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_parent_text_edges__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_parent_text_edges__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_parent_text_edges__content_box_rtl.xml",
]

case_id = "block/fri09_inline_middle_and_line_edges"
source = "tests/layout/browser_parity/html/block/fri09_inline_middle_and_line_edges.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_inline_middle_and_line_edges__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_middle_and_line_edges__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_middle_and_line_edges__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_inline_middle_and_line_edges__content_box_rtl.xml",
]

case_id = "block/fri09_block_content_positions"
source = "tests/layout/browser_parity/html/block/fri09_block_content_positions.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_block_content_positions__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_positions__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_positions__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_positions__content_box_rtl.xml",
]

case_id = "block/fri09_block_content_safe_overflow"
source = "tests/layout/browser_parity/html/block/fri09_block_content_safe_overflow.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_block_content_safe_overflow__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_safe_overflow__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_safe_overflow__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_safe_overflow__content_box_rtl.xml",
]

case_id = "block/fri09_block_content_inline_float"
source = "tests/layout/browser_parity/html/block/fri09_block_content_inline_float.html"
outputs = [
  "tests/layout/browser_parity/xml/block/fri09_block_content_inline_float__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_inline_float__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_inline_float__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/block/fri09_block_content_inline_float__content_box_rtl.xml",
]

case_id = "flex/fri09_flex_first_baseline_content"
source = "tests/layout/browser_parity/html/flex/fri09_flex_first_baseline_content.html"
outputs = [
  "tests/layout/browser_parity/xml/flex/fri09_flex_first_baseline_content__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/flex/fri09_flex_first_baseline_content__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/flex/fri09_flex_first_baseline_content__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/flex/fri09_flex_first_baseline_content__content_box_rtl.xml",
]

case_id = "flex/fri09_flex_last_baseline_content"
source = "tests/layout/browser_parity/html/flex/fri09_flex_last_baseline_content.html"
outputs = [
  "tests/layout/browser_parity/xml/flex/fri09_flex_last_baseline_content__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/flex/fri09_flex_last_baseline_content__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/flex/fri09_flex_last_baseline_content__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/flex/fri09_flex_last_baseline_content__content_box_rtl.xml",
]

case_id = "grid/fri09_grid_baseline_content"
source = "tests/layout/browser_parity/html/grid/fri09_grid_baseline_content.html"
outputs = [
  "tests/layout/browser_parity/xml/grid/fri09_grid_baseline_content__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/grid/fri09_grid_baseline_content__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/grid/fri09_grid_baseline_content__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/grid/fri09_grid_baseline_content__content_box_rtl.xml",
]

case_id = "subgrid/fri09_subgrid_baseline_content_controls"
source = "tests/layout/browser_parity/html/subgrid/fri09_subgrid_baseline_content_controls.html"
outputs = [
  "tests/layout/browser_parity/xml/subgrid/fri09_subgrid_baseline_content_controls__border_box_ltr.xml",
  "tests/layout/browser_parity/xml/subgrid/fri09_subgrid_baseline_content_controls__border_box_rtl.xml",
  "tests/layout/browser_parity/xml/subgrid/fri09_subgrid_baseline_content_controls__content_box_ltr.xml",
  "tests/layout/browser_parity/xml/subgrid/fri09_subgrid_baseline_content_controls__content_box_rtl.xml",
]
```

`J` records are `data-surgeist-inline-justification` entries and `V` records
are `data-surgeist-inline-vertical-alignments` entries as defined below. The
source DOM is authored so every listed source index exists, every J record has a
later surviving participant on the tested line, and no unlisted new marker is
present. A source using a listed marker must consume every record in every
active direction variant.

The exact identities and paths above are the complete FRI-09 artifact surface.
A different source count, case identity, directory, variant, or output path is
nonconforming.

### 11.2 Typed fixture facts

On a `data-surgeist-layout-ready-inline="true"` root, the new marker schemas are
exactly:

```json
data-surgeist-inline-justification='[
  {"sourceIndex": 0, "weight": 1}
]'

data-surgeist-inline-vertical-alignments='[
  {"sourceIndex": 0, "kind": "baseline-shift", "value": 4}
]'
```

Each array must be nonempty when present. `sourceIndex` is the same integer
participant identity used by the current break, strut, and bidi marker tables
and may identify shaped text or an atomic inline child. Duplicate indices in
one table are forbidden.

The justification table permits exactly `sourceIndex` and `weight`; weight must
be finite and strictly positive. Absence for a participant means no
opportunity. A record after the final surviving participant, a record consumed
only by discarded trailing whitespace, or an unused record is invalid.

The vertical table permits exactly `sourceIndex`, `kind`, and conditionally
`value`. `kind` is one of `baseline`, `baseline-shift`, `text-top`,
`text-bottom`, `middle`, `line-top`, or `line-bottom`.
`baseline-shift`, `text-top`, `text-bottom`, and `middle` require `value`;
the other kinds forbid it. Baseline shift accepts any finite value. Text-top,
text-bottom, and middle require a finite non-negative value. Absence for shaped
text means baseline. Atomic boxes normally use their computed node style; when
a V record names an atomic participant it is the required resolved payload and
must agree with that style's vertical-align category.

The helper reads computed `textAlign` and `textAlignLast` from every marked
inline root and maps only `start`, `end`, `left`, `right`, `center`, and
`justify`, with `auto` additionally allowed for the last line. These computed
keywords are the layout-ready line policies. The helper does not use a marker
to override them.

Numeric vertical metrics and justification weights are never derived from
browser rectangles, expected XML, Range geometry, font probes, source name, or
variant name. The explicit J/V tables are their sole source. The helper may use
computed `verticalAlign` only to validate that the authored source category
matches the V record (`top` to line-top, `bottom` to line-bottom, and the
corresponding text/middle/baseline or numeric class); it may not derive the
record's numeric payload.

Helper JSON adds `textAlignLast` beside `style.textAlign`. Each emitted
`inlineSegments` entry contains `verticalAlignment: { kind, value? }` and
optional `justificationWeight`. Each `atomicInlineParticipation` entry contains
optional `justificationWeight`; an atomic V payload is emitted in its node's
typed vertical-alignment fields.

Generated XML uses existing node `text-align` and adds `text-align-last` when
non-auto. Each `<segment>` adds `vertical-align-kind`, optional
`vertical-align-value`, and optional `justification-weight`. Atomic placeholder
participation adds optional `justification-weight`; its node adds
`vertical-align-kind` and optional `vertical-align-value`. Existing XML without
these fields parses to start/auto, baseline, and no opportunity exactly where
the old source represented those defaults; the serializer omits default new
attributes so prior outputs remain byte-identical.

Frozen XML compatibility is closed by this parser table:

| Existing XML attribute | Accepted token | Layout-ready value |
| --- | --- | --- |
| `text-align` | absent or `start` | `TextLineAlignment::Start` |
| `text-align` | `end` | `TextLineAlignment::End` |
| `text-align` | `left` or `-webkit-left` | `TextLineAlignment::Left` |
| `text-align` | `right` or `-webkit-right` | `TextLineAlignment::Right` |
| `text-align` | `center` or `-webkit-center` | `TextLineAlignment::Center` |
| `text-align` | `justify` | `TextLineAlignment::Justify` |
| `text-align-last` | absent or `auto` | `TextLastLineAlignment::Auto` |
| `text-align-last` | `start`, `end`, `left`, `right`, `center`, or `justify` | Corresponding non-auto `TextLastLineAlignment` value |
| legacy `vertical-align` | absent or `baseline` | `VerticalAlignOf::Baseline` |
| legacy `vertical-align` | `top` | `VerticalAlignOf::LineTop` |
| legacy `vertical-align` | `bottom` | `VerticalAlignOf::LineBottom` |

No other legacy token is accepted. When `vertical-align-kind` is present it is
the typed authority; a simultaneously present legacy `vertical-align` must be
one of the three rows above and agree with the typed kind. Payload-bearing new
states are represented only by `vertical-align-kind` plus the conditionally
required `vertical-align-value`; the parser never derives their payload from a
legacy string.

Parser coverage includes representative frozen XML for every legacy alias,
absence defaults, disagreement, and unknown-token rejection. This compatibility
path changes normalized input only; no pre-FRI-09 XML file is rewritten.

Helper and Rust parsers validate supported fields, source association,
finiteness, sign rules, conditional value presence, computed-category
agreement, and complete marker consumption. Malformed JSON, empty tables,
missing or duplicate identities, unknown fields/kinds, non-finite values,
negative values where forbidden, trailing opportunities, and unused records
are rejected. Source-name, variant-name, and expected-geometry mutation tests
prove identical normalized input.

The generator must continue to parse current sources and preserve all prior
outputs byte-for-byte outside the 72 new variants. Unrelated generator defects
are outside this specification.

### 11.3 Canonical artifact state

The conforming post-FRI-09 corpus has exactly 1,466 HTML sources and 5,848 XML
outputs. The schema-3 canonical report has exactly 5,848 generated variants,
16 unchanged unsupported variants, three unchanged expected-fail source
records, zero quarantined, and zero failed-to-generate.

The only new source and XML identities are the 18 cases and 72 paths in section
11.1. Every pre-FRI-09 XML body is byte-identical. Every XML file is free of
provenance comments. Browser, source, helper, resource, and output provenance is
stored only in the single canonical `all.json` report.

The corpus manifest names all 18 cases and no open-ended directory import. The
helper and Rust parser implement exactly the closed fields in section 11.2.
The report's source/resource/helper hashes correspond to the canonical files,
and its generated identity set equals the filesystem XML identity set.

## 12 FRI-09.12 Verification Surface

Behavioral coverage exercises both `f32` and `f64` unless the boundary is
inherently textual or filesystem-only.

Required focused families include:

- construction, validation, defaulting, and compile-fail invalid-state tests
  for every new public type;
- proof that `JustifyContent` cannot contain baseline states;
- logical and physical text alignment under all five writing modes and both
  directions;
- last-line auto/explicit selection, forced breaks, paragraph-final lines,
  indefinite width, overfull lines, and no-opportunity fallback;
- weighted justification, exact residual assignment, bidi visual ordering,
  atomic opportunities, trailing discard, float-narrowed bands, and committed
  fragment adjustments;
- every resolved vertical-alignment state, oversized line-relative groups,
  replacement/overflow baselines, controls and boundaries, and cache replay;
- block alignment with block children, inline lines, floats, collapse-through,
  definite/indefinite size, safe overflow, vertical writing, and positioned
  negative controls;
- flex first/last groups, wrapped lines, orthogonal and replaced fallbacks,
  auto-margin controls, intrinsic sizing, and warm cache;
- grid row/column groups, implicit tracks, spanning controls, subgrid
  propagation, grid-lanes supported projection, and fallbacks;
- no fixed-point drift or unbounded recomputation;
- cold/warm and batched/transactional committed-output equivalence;
- exact parser rejection and source-independence tests; and
- all 72 finite browser-parity ownership rows from section 11.1.

Existing FRI-01 through FRI-08 focused families remain regression coverage.
FRI-10 positioning, FRI-11 fragmentation, FRI-12 display/table, and FRI-13
aggregate rows remain negative controls and are not silently re-owned.

### 12.1 Concrete module and API ownership

The implementation is allocated to the current crate surfaces as follows:

| Path | FRI-09 ownership |
| --- | --- |
| `src/node_input.rs` | Public `AlignContent` completion; distinct `JustifyContent`; `TextLineAlignment`, `TextLastLineAlignment`, and `TextAlignment`; `VerticalAlignOf` and errors; `InlineJustificationOpportunityOf`; complete shaped-segment and atomic-participation carriers; renamed `NodeInputOf::text_alignment`; scalar-generic line-break and node fields |
| `src/alignment.rs` | New crate-private sole owner of shared content-distribution normalization, safe fallback, reversal, one-subject distribution fallback, spacing arithmetic, baseline fallback, and validated `BaselineContentAdjustmentOf`/`BaselineShim` conversion |
| `src/output.rs` | Baseline adjustment in `ComputeInputOf`; renderer-facing justification adjustment in `InlineFragmentOutputOf`; constructors/getters and direct-leaf zero default |
| `src/cache.rs` | Exact baseline-adjustment cache-key identity |
| `src/inline.rs` | All-line/last-line policy selection, shaping-owned weighted distribution, visual residual allocation, adjusted fragment advance, baseline-relative vertical group, line-relative monotone envelope, and logical-to-physical projection |
| `src/block.rs` | Typed input propagation; atomic/control vertical carriers; ordinary block content subject construction, independent-formatting-context boundary, logical subject translation, and overflow composition |
| `src/flex.rs` | Per-line first/last baseline-content eligibility, immutable maximum reduction, adjustment derivation, one-way re-layout consumption, and fallback |
| `src/grid/tracks.rs` | Reuse of `AncestorBaselineGroup::intrinsic_shim` and the existing immutable target reduction; no second group or target representation |
| `src/grid/subgrid.rs` | Reuse of `CheckedOwnerToCurrentPlacementMap`, `InheritedCurrentGridBaselinePlacement`, and downward-only child view without inverse publication |
| `src/grid/child.rs` | Owner-direct or inherited-current target selection, exact conversion of the existing `BaselineShim` at the child compute boundary, and no double accounting |
| `src/grid/mod.rs` | Transport of the existing owner group/map carrier and the scalar-generic compute adjustment without replacement or cloned ownership |
| `src/compute.rs` | Recursive adjustment propagation, committed descendant/fragment staging, cold/warm equivalence, and one-time physical output translation |
| `src/lib.rs`, `src/lib_tests.rs`, `README.md` | Public reexports, removal of obsolete aliases/variants, compile-fail invalid-state coverage, public API inventory, and layout-ready ownership documentation |
| `src/inline_tests.rs`, `src/block_tests.rs`, `src/flex_tests.rs`, `src/grid_tests.rs`, `src/cache_tests.rs`, `src/compute_tests.rs`, `src/root_tests.rs` | Focused scalar, composition, cache, transaction, and public-layout behavior from this section |
| `src/test_support/oracle/inline.rs`, `src/test_support/oracle/grid/alignment.rs`, `src/test_support/oracle/grid/baseline.rs`, `src/test_support/oracle/grid/subgrid.rs` | Independent arithmetic/reference projections only; no copied production state machine or fixture-specific expectation dispatch |
| `tests/layout/browser_parity/support.rs` | Closed legacy/new XML lowering, typed rejection, 72-row comparison ownership, and source-independence assertions |
| `tests/layout/browser_parity/scripts/gentest/test_helper.js` | Exact computed text-policy capture, explicit J/V table validation and consumption, and typed JSON emission without geometry inference |
| `tests/bin/surgeist-layout-generate/generator.rs` | Narrow serialization/parsing of section 11.2 fields, frozen compatibility tests, exact manifest/report validation, and no general generator refactor |
| `tests/layout/browser_parity.rs`, `tests/layout/browser_parity/corpus.toml`, and the exact section 11.1 HTML/XML/report paths | Finite case registration, artifact identity/count assertions, and canonical artifact state |

No other module becomes an alignment, shaping, baseline-group, cache, flow-axis,
or artifact authority. Mechanical call-site changes may consume these APIs but
must not create another policy owner.

### 12.2 Named test anchors

The final source contains at least these exact named test anchors; parameterized
helpers beneath an anchor cover both scalar lanes where the name says so:

- `fri09_model_content_domains_are_property_valid`;
- `fri09_model_text_and_vertical_inputs_validate_both_scalars`;
- `fri09_model_non_box_shaped_alignment_has_one_owner`;
- `fri09_inline_text_alignment_all_flows_both_scalars`;
- `fri09_inline_last_line_and_no_opportunity_fallback_both_scalars`;
- `fri09_inline_weighted_justification_visual_residual_both_scalars`;
- `fri09_inline_atomic_trailing_and_float_band_justification_both_scalars`;
- `fri09_inline_vertical_baseline_group_both_scalars`;
- `fri09_inline_line_relative_monotone_envelope_both_scalars`;
- `fri09_block_content_alignment_subject_both_scalars`;
- `fri09_block_safe_overflow_and_independent_context_both_scalars`;
- `fri09_flex_first_last_baseline_content_adjustment_both_scalars`;
- `fri09_flex_baseline_content_fallbacks_and_cache_both_scalars`;
- `fri09_grid_baseline_content_reuses_owner_group_both_scalars`;
- `fri09_subgrid_baseline_content_uses_checked_owner_map_both_scalars`;
- `fri09_grid_baseline_adjustment_not_double_counted_both_scalars`;
- `fri09_cache_key_distinguishes_baseline_content_adjustment_both_scalars`;
- `fri09_adjusted_cold_warm_committed_output_match_both_scalars`;
- `fri09_frozen_xml_alignment_aliases_map_without_rewrite`;
- `fri09_fixture_schema_rejects_malformed_or_identity_dependent_input`;
- `fri09_generator_serializes_closed_alignment_fields`;
- `fri09_artifact_identity_and_counts_are_exact`; and
- `fri09_browser_parity_all_72_owned_rows`.

Parser rejection anchors include a table-driven case for every forbidden field,
token, payload, identity, and marker-use state named in section 11.2. Existing
FRI-01 through FRI-08 regression names are not renamed to satisfy this list.

## 13 FRI-09.13 Documentation And Compatibility

The public front door reexports the complete new alignment surface and removes
the obsolete types or aliases. `README.md` documents:

- the layout-ready boundary;
- shaping-owned justification opportunities;
- resolved vertical metric inputs;
- block/flex/grid baseline content-alignment support;
- scalar genericity and supported writing modes; and
- root ownership of authored CSS and font/shaping resolution.

All examples compile against the final public surface. Compile-fail tests prove
private payloads and invalid property states cannot be constructed.

No dependency, feature, edition, rust-version, repository URL, license, or
crate-boundary change is permitted. No unsafe code or suppression is permitted.

## 14 FRI-09.14 Durable Technical Dependencies

Any implementation sequence derived from this specification must preserve
these product dependencies:

1. land the public model and shared policy before format algorithms consume it;
2. land text alignment and justification before vertical alignment so line
   selection/output carriers settle once;
3. land vertical alignment before block content alignment so aligned inline
   subject envelopes are authoritative;
4. land block content alignment before parent baseline coordination so the
   child adjustment composes with its own content policy;
5. land flex and grid baseline coordination only after cache identity carries
   the adjustment; and
6. settle all production behavior and the closed fixture-input schema before
   an artifact transaction can replace the frozen lineage.

These are dependency constraints, not a pre-authored execution plan. A later
implementation sequence may decompose them without changing their order.

## 15 FRI-09.15 Acceptance And Closure

FRI-09 closes `MODEL-006` only when all of the following are true:

1. content alignment and justification are distinct public types and invalid
   baseline justification is unrepresentable;
2. the old text and vertical alignment subsets are removed rather than retained
   as ambiguous compatibility paths;
3. logical/physical line alignment and explicit last-line policy work across
   every supported writing mode and both scalar lanes;
4. explicit shaping-owned opportunities produce deterministic justification
   geometry and committed renderer-facing adjustments;
5. every resolved vertical-alignment state affects line metrics, output,
   baselines, and overflow correctly;
6. represented block-container content alignment is live and composes with
   floats, inline fragments, margins, and scroll geometry;
7. eligible flex and grid items coordinate first and last baseline content
   alignment with typed cache-visible child adjustments;
8. fallbacks for ineligible, orthogonal, replaced, indefinite, overfull, and
   no-baseline cases are explicit and tested;
9. the canonical artifact state has exactly the identities, counts, provenance
   location, and byte-preservation properties in section 11;
10. no fixture identity or expected geometry enters layout input;
11. prior published behavior remains unchanged outside this specification's
    explicit delta, and the crate remains dependency/feature/MSRV-stable,
    warning-free, suppression-free, and unsafe-free; and
12. the resulting source and public documentation contain no second alignment,
    baseline-group, shaping, flow-axis, cache, or artifact authority.

After this product acceptance is satisfied, root still owns facade lowering,
generated API artifacts, integration verification, and gitlink promotion. A
leaf implementation does not authorize those root mutations.
