# P01-I07 Flex Algorithm Completeness

Design owner: `surgeist-layout`

## 1 FRI-07.1 Authority And Outcome

This specification is the authoritative desired-state contract for `FRI-07` in
`plans/P01-layout/P01-index.md`. It closes exactly these findings from
`plans/P01-layout/P01-initial-review-findings.md`:

- `FLEX-002`, cross-axis auto margins under overflow;
- `FLEX-003`, auto margins on absolutely positioned flex children;
- `FLEX-004`, intrinsic flex-basis keywords; and
- `FLEX-005`, collapsed flex-item struts.

The outcome is a scalar-generic flex algorithm in which normal and absolute
auto margins use their correct available-space equations, `min-content` and
`max-content` flex bases remain distinct through measurement, and a normalized
collapsed flex item participates only long enough to establish its line strut.
These behaviors compose with the already-published order, writing-mode,
replaced-item, and overflow contracts without reopening their ownership.

This is an intentional breaking pre-release correction at crate version
`0.1.0`. Backward compatibility is not required. The new layout-ready collapse
state is represented directly, without a compatibility boolean, CSS parser, or
painting visibility model.

This specification supersedes stale source locations and intermediate behavior
claims in the findings snapshot. It does not change the finding IDs or their
required observable closure.

## 2 FRI-07.2 Ownership And Non-Goals

`surgeist-layout` owns:

1. the normalized layout effect that says whether a child participates as a
   normal flex item or a collapsed flex item;
2. flex item collection, line breaking, sizing, alignment, strut capture,
   collapsed-item suppression, and physical output;
3. auto-margin used values and placement for ordinary and absolutely positioned
   children of flex containers;
4. flex-basis dispatch and intrinsic measurement after property-specific values
   have crossed the public boundary; and
5. focused, oracle, property, scalar, browser-parity, fixture, and documentation
   evidence for this initiative.

Root `surgeist` owns authored CSS, cascade and computed-style resolution, box
generation, and lowering from computed `visibility` to the normalized layout
effect. Root also owns facade composition and generated API artifacts. This leaf
publishes a source candidate and handoff; it does not edit root.

Rendering owns whether a normal item is painted as visible or hidden. Layout
does not accept a general `visibility` property and does not model painting,
hit testing, animations, counters, accessibility, or retained DOM state.

The following remain outside FRI-07:

- new alignment values or cross-format baseline semantics owned by `FRI-09`;
- general positioned-layout modes and containing-block semantics owned by
  `FRI-10`; FRI-07 changes only the already-supported absolute flex-child
  auto-margin equation;
- fragmentation and line limiting owned by `FRI-11`;
- inline-flex outer participation and normalized display roles owned by
  `FRI-12A` and `FRI-12B`;
- the aggregate WPT/release gate owned by `FRI-13`;
- authored CSS parsing, an HTML parser, pre-inspection of browser geometry, a new
  generator, generator architecture expansion, dependencies, features, or MSRV
  changes; and
- implementing later-owned flex-basis values such as `stretch`, bare
  `fit-content`, `contain`, `fit-content()`, or keyword-basis `calc-size()`.

Generator changes are allowed only when narrowly required to serialize the new
computed collapse fact, parse the six named fixtures, or correct a confirmed
genuine generator bug. Fixture identity and expected geometry never select or
alter layout input.

## 3 FRI-07.3 Current Evidence

The initiative base is the remotely verified FRI-06 candidate
`d386c7d796e5fe0c0856c15ac800516df1348f3b`. At that revision:

- `NodeInputOf<S>` has `item_order`, `item_is_replaced`, property-specific
  `FlexBasisOf<S>`, `WritingMode`, `Direction`, and normalized overflow, but no
  collapsed flex-item state;
- `FlexAxes` is the sole flex mapping owner for logical main/cross axes,
  direction, reversal, and physical edges;
- `collect_items` excludes absolute and `display: none` children, then
  `item_order_permutation` creates one stable order-modified item sequence;
- `collect_flex_lines` uses every collected item's hypothetical outer main size;
  `resolve_lines` performs flexible sizing, placement, cross-size calculation,
  alignment, and line alignment in one pass over those line ranges;
- `resolve_cross_axis_auto_margins` divides negative free space between two auto
  margins and assigns all negative space to a sole start auto margin, contrary
  to CSS Flexbox section 9.6;
- `resolve_absolute_margins` uses the whole container inner size, ignores
  insets, clamps negative free space to zero, and cannot apply the inline-start
  overflow exception from CSS Positioned Layout section 4.2;
- `FlexBasisOf<S>` and the finite fixture parser preserve `MinContent` and
  `MaxContent`, while `dispatch_flex_basis` intentionally returns exact
  FRI-07-owned unsupported-capability payloads for both; and
- the checked-in full browser report has 5,712 generated, 16 unsupported, zero
  expected-fail, zero quarantined, and zero failed-to-generate variants at
  SHA-256 `8d59c87d1fcc185bda0372968ae81dbeff74f241c17335db98629ad49f1f463f`.

The current fixture helper SHA-256 is
`42bf9ff77810b2e9fb5a184f525d9e22f74abae12a09f9486b3b49dc620188c2`,
the base-style SHA-256 is
`5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6`,
and the manifest SHA-256 is
`99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701`.
The 16 unsupported rows are the unchanged missing-root float sources; none is
FRI-07-owned.

The normative product sources are:

- CSS Flexible Box Layout Module Level 1 sections 4.4, 7.2.3, 9.4, 9.5, and
  9.6: `https://www.w3.org/TR/css-flexbox-1/`;
- CSS Positioned Layout Module Level 3 section 4.2:
  `https://www.w3.org/TR/css-position-3/#abspos-margins`; and
- CSS Box Sizing Module Level 3 intrinsic sizing values:
  `https://www.w3.org/TR/css-sizing-3/#sizing-values`.

The WPT sources linked by the Flexbox specification include
`flexbox_visibility-collapse.html`,
`flexbox_visibility-collapse-line-wrapping.html`,
`flexbox-collapsed-item-baseline-001.html`, and
`flexbox-collapsed-item-horiz-001.html` through `-003.html`. They are normative
corroborating evidence, not files imported into this repository by FRI-07.

## 4 FRI-07.4 Resolved Design Decisions

| ID | Decision |
| --- | --- |
| `D-01` | Add public `FlexItemCollapse` with exactly `Normal` and `Collapsed`, and add `NodeInputOf::flex_item_collapse`. The type names the normalized flex-layout effect, not authored or computed visibility. `Normal` is the default. |
| `D-02` | The field is meaningful only when the node is an in-flow child of a flex container. Other formatting contexts preserve their existing behavior; root supplies `Normal` outside collapsed flex participation. Absolute and `display: none` children never create struts. |
| `D-03` | A collapsed item remains in the order-modified sequence. The first layout round treats it normally and records the settled cross size of its line as that item's strut. The second round redoes line collection with the collapsed box's main size treated as zero while retaining its resolved main-axis margins and normal line-collection gap positions. After line collection it is ignored for every later sizing/alignment/contribution phase, whose gaps are only between remaining normal items. Its second-round line is floored at the largest collapsed-item strut assigned to that line. |
| `D-04` | A collapsed item's first-round strut is the used line cross size after ordinary line cross-size calculation and `align-content: stretch`, matching Flexbox section 9.4. It is not the item's own outer cross size, baseline extent, or final container cross size. |
| `D-05` | The second round is finite and runs at most once. It starts from immutable collected item measurements and immutable per-item struts; second-round geometry never feeds a third round. Scrollbar-settling remains the existing outer fixed-point owner and may rerun the complete finite flex computation with a different available box. |
| `D-06` | A collapsed item publishes a source-indexed zero `NodeOutputOf` and its descendants take the existing hidden-computation path. It contributes no margin, gap, baseline, intrinsic main size, scrollable overflow, scroll target, container content size, or absolute-child containing geometry. Its strut is private line state, not a public output box. |
| `D-07` | Cross-axis auto margins follow Flexbox section 9.6 exactly. Positive difference is shared among auto margins. Under overflow, logical cross-start auto resolves to zero and cross-end receives the signed remainder; if only cross-end is auto it receives the signed remainder, and if only cross-start is auto it resolves to zero while the non-auto opposite edge remains unchanged. |
| `D-08` | Ordinary main-axis auto margins retain Flexbox section 9.5 behavior: distribute only positive remaining space; otherwise every main-axis auto margin resolves to zero. FRI-07 does not apply the cross-axis overflow rule to the main axis. |
| `D-09` | Absolutely positioned flex-child auto margins are resolved per physical axis using the inset-modified containing block. If either inset in that axis is auto, every auto margin in that axis is zero. Otherwise remaining space is inset-modified size minus used size minus non-auto margins and is divided among auto margins. |
| `D-10` | For an absolutely positioned child with two auto margins and negative inline-axis remaining space, containing-block inline-start is zero and inline-end receives the full signed remainder. In the block axis, negative remaining space is divided normally. Containing `FlowAxes`, not the child's writing mode or a physical-left shortcut, selects inline start/end. |
| `D-11` | `ResolvedFlexBasis<S>` gains distinct `MinContent` and `MaxContent` states. Dispatch removes exactly those two FRI-07 capability cells. A min-content basis measures the flex item's main size under `AvailableOf::MIN_CONTENT`; a max-content basis measures under `AvailableOf::MAX_CONTENT`. Neither consults the preferred main size as `auto` does, and neither enters the `content` max-content fallback. |
| `D-12` | Intrinsic flex-basis measurement keeps box-sizing, padding/border floor, min/max clamping, aspect ratio, replaced-item sizing, orthogonal flow, error propagation, scalar type, and percentage context in their existing phases. The intrinsic keyword selects the measurement constraint; it does not bypass those contracts. |
| `D-13` | `stretch`, bare `fit-content`, `contain`, `fit-content()`, and keyword-basis `calc-size()` retain their exact typed unsupported-capability results. Enabling `MinContent` and `MaxContent` must not broaden another cell. |
| `D-14` | Existing `ItemOrder`, `FlexAxes`, `item_is_replaced`, normalized overflow, cache, and transaction contracts remain the sole owners of their facts. FRI-07 composes with them and adds no parallel order sort, axis mapping, replaced heuristic, overflow pair, or cache identity. |
| `D-15` | Browser fixtures use explicit computed/layout-ready facts only. The helper may serialize computed `visibility` as `flex-item-collapse=collapsed` for an exact collapsed flex item; the Rust fixture adapter accepts only `collapsed`, otherwise defaults to `Normal`, and rejects every other explicit token. This is bounded lowering, not a CSS visibility parser. |
| `D-16` | Exactly six named four-variant sources form the FRI-07 browser set. Inputs and behavior settle before one unfiltered full regeneration. Scoped generation may be used during iteration as a diagnostic, but is not required verification evidence and never writes a report. No redundant full generation is permitted. |
| `D-17` | Pinned Chrome is the default browser oracle. A source/variant may be an expected fail only under the complete certainty contract in `FRI-07.10.2`, with a public-front-door synthetic substitute. A Surgeist/Chrome disagreement alone never qualifies. |
| `D-18` | The final initiative cycle validates touched flex architecture and implements every confirmed in-initiative sprawl finding after the behavior/artifact candidate is published. It changes no FRI-07 behavior, public API, fixture membership, generator output, dependency, feature, or finding ownership unless review proves a genuine defect, which reopens the owning behavior contract. |

Rejected alternatives:

- A public `visibility` enum would import painting and non-flex semantics that
  layout does not own.
- A boolean named `is_collapsed` would not identify the flex-specific phase and
  would invite reuse by table, rendering, or retained-tree code.
- Removing collapsed items before the first pass cannot establish the required
  line strut; retaining their normal main size in the second pass prevents
  required rewrapping.
- Publishing the strut as the collapsed node's geometry would make it contribute
  to overflow and expose a box that the algorithm otherwise ignores.
- Treating intrinsic keywords as `content` or as numeric unresolved values would
  recreate `FLEX-004` and erase the selected measurement constraint.
- Reusing the ordinary flex cross-axis margin helper for absolute children would
  ignore inset-modified containing blocks and positioned-layout inline overflow.
- Building a Rust HTML/CSS parser or inferring fixture input from expected
  geometry would violate the crate boundary and make parity evidence circular.

## 5 FRI-07.5 Public Model And Compatibility

The public layout-ready type is:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FlexItemCollapse {
    #[default]
    Normal,
    Collapsed,
}
```

`NodeInputOf<S>` adds:

```rust
pub flex_item_collapse: FlexItemCollapse,
```

`NodeInput::DEFAULT`, generic `Default`, and `NodeInputOf::non_box()` use
`Normal`. `FlexItemCollapse` is reexported from the crate root. There is no
constructor alias, legacy boolean, conversion from a CSS value, or hidden
variant.

Adding the public field breaks exhaustive struct literals. Functional record
update from `NodeInput::default()` or `NodeInput::DEFAULT` receives `Normal`.
The root handoff must add explicit computed-style lowering before promoting this
candidate; this leaf supplies no root adapter.

No public output, error, request, cache key, scalar alias, trait, dependency,
feature, or MSRV changes. Collapsed participation changes geometry only when the
new field is `Collapsed` on an in-flow flex item. Every existing default input
remains behaviorally unchanged except the three named defect corrections.

## 6 FRI-07.6 Auto-Margin Behavior

### 6.1 Ordinary Cross-Axis Auto Margins

Let `line_cross` be the used cross size of the item's flex line. Let
`outer_without_auto` be the item's target cross size plus both non-auto cross
margins. Let `remaining = line_cross - outer_without_auto`.

| Auto edges | `remaining > 0` | `remaining <= 0` |
| --- | --- | --- |
| neither | Preserve both margins; alignment applies normally | Preserve both margins; alignment applies normally |
| start only | start receives `remaining`; preserve end | start is zero; preserve end |
| end only | end receives `remaining`; preserve start | end receives `remaining`; preserve start |
| start and end | each receives `remaining / 2` | start is zero; end receives `remaining` |

Start/end are the flex cross-axis sides selected by `FlexAxes`, including
`wrap-reverse`, vertical and sideways writing modes, and RTL. Used negative end
margin is observable in `NodeOutputOf::margin`; placement remains anchored at
cross-start. The engine must not compare browser computed-style margin strings
to infer the used layout margin.

Focused tests cover positive, zero, and negative remaining space; each auto-edge
combination; row, column, reverse, and wrap-reverse; all ten writing-mode and
direction mappings; both scalar lanes; and output margin plus physical geometry.

### 6.2 Absolutely Positioned Flex Children

For each physical axis, derive:

```text
inset_modified_size = containing_padding_box_size - start_inset - end_inset
remaining = inset_modified_size - used_border_box_size - non_auto_margins
```

The axis matrix is:

| Insets | Auto margins | Result |
| --- | --- | --- |
| either inset auto | any | Every auto margin in that axis is zero; the existing stronger-inset/static-position placement handles location. |
| both definite | none | Preserve margins; existing self-alignment rules apply. |
| both definite | one | The auto margin receives signed `remaining`. |
| both definite | two, non-negative | Divide `remaining` equally. |
| both definite | two, negative inline axis | containing inline-start margin is zero; inline-end receives all `remaining`. |
| both definite | two, negative block axis | Divide `remaining` equally. |

The margin resolver receives resolved insets and the same containing padding-box
size already used by absolute sizing. It does not reconstruct availability from
`node_inner_size` after sizing. Physical output, source index, scroll geometry,
and existing absolute alignment remain unchanged except where this equation
changes margins or location.

The current test `flex_absolute_child_expands_auto_margins` is an erroneous
legacy characterization. Its replacement proves the one-auto-inset case uses
zero auto margins and x `0` in the original `100x40`/`20px` scenario. Additional
tests cover both definite insets, one/two auto margins, positive/negative space,
inline RTL start/end, vertical writing, block-axis negative division, box
sizing, padding/border, and both scalar lanes.

## 7 FRI-07.7 Intrinsic Flex Basis

`dispatch_flex_basis` produces these exact FRI-07 results:

| Public basis | Resolved basis | Measurement |
| --- | --- | --- |
| `Auto` | `Auto` | Existing preferred-main-size consultation; content only when preferred is auto. |
| `Content` | `Content` | Existing content path and algorithm-selected available constraint. |
| `MinContent` | `MinContent` | Item main-size measurement under `AvailableOf::MIN_CONTENT`. |
| `MaxContent` | `MaxContent` | Item main-size measurement under `AvailableOf::MAX_CONTENT`. |
| supported numeric/calc-size | `Definite(S)` | Existing percentage and box-sizing path. |
| every later-owned value | exact unsupported capability | Unchanged payload and owner. |

The collected item retains whether its basis is definite and whether it uses the
generic content path. It also retains the intrinsic selection needed by final
layout so padding-floor overflow suppression and child recomputation cannot
mistake an intrinsic basis for `Content`.

When a provider returns min-content `20` and max-content `100` under otherwise
identical input, the two flex bases and resulting unflexed main sizes are `20`
and `100`. Flex grow/shrink can subsequently change target main size according
to the ordinary algorithm; that does not erase the distinct base sizes.

Focused evidence covers leaf measurement, child containers, replaced and
non-replaced items, content-box and border-box, padding/border floors, min/max
clamping, definite and indefinite container main sizes, row and column main
axes, orthogonal child writing modes, grow/shrink controls, provider failure,
non-finite provider output, cache cold/warm equivalence, and both scalar lanes.
Dispatcher tests prove only the two direct capability cells changed.

## 8 FRI-07.8 Collapsed-Item Algorithm

### 8.1 Phase Model

The flex computation uses these private phases:

1. `CollectedFlexItem` stores normal item measurements, source index, order, and
   `FlexItemCollapse`.
2. First-round line collection and resolution run with every collected item as
   normal.
3. Each collapsed item records its first-round line's used cross size as a
   private strut.
4. Second-round line collection retains the order-modified item slot, treats its
   box main size as zero, and still accounts for its resolved main-axis margins
   and normal line-collection gap positions.
5. Second-round sizing, intrinsic contribution, main/cross alignment, baseline,
   container-content, and scroll contribution operate only on normal items.
6. After ordinary second-round line cross-size calculation and before item
   cross alignment, each line is floored by its largest assigned strut.
7. Final layout publishes normal items and explicitly publishes collapsed items
   through the zero-output/hidden-descendant path.

An empty or all-collapsed flex container still has one line when the existing
algorithm requires one. An all-collapsed line receives its largest strut and no
main-size, gap, baseline, or scroll contribution. Multiple collapsed items on a
line use the largest strut, not their sum.

### 8.2 Wrapping And Order

Second-round line breaking is redone from the beginning. A collapsed box has
zero main size while its resolved main-axis margins and the normal gaps between
collected item slots still affect line membership. This is the exact
line-collection exception; it does not replace the collapsed item with
`display:none` before its strut can be assigned.

After line collection, the collapsed item is ignored. Flexible sizing,
main-axis placement, and final line size count gaps only between remaining
normal items, so no committed gap is attached to the collapsed slot.

Struts are attached to collapsed item identity, then applied to the line that
contains that zero-main item in the second round. They are not attached to the
first-round line index because rewrapping can change line membership.

Order-modified traversal remains stable by `(ItemOrder, SourceIndex)`. Source
association in outputs remains raw source index. Collapse does not reorder a
node and does not suppress its order slot before strut placement.

### 8.3 Sizing, Baselines, And Overflow

The first-round line cross size includes ordinary baseline participation,
cross margins, min/max constraints, replaced sizing, and alignment stretch.
That complete used line size is the strut. In the second round the collapsed
item contributes no baseline; the strut can floor a line whose normal items
would otherwise establish a smaller baseline group.

Collapsed items contribute no main/cross intrinsic size except the required
line cross strut during actual flex layout. In intrinsic passes that execute the
flex algorithm, the same two-round rule applies; there is no separate
max-content approximation.

The zero output of a collapsed item is excluded from `FlexChildContribution`,
container scrollable overflow, target inventory, and content size. A collapsed
item whose normal layout would overflow, reserve a scrollbar, contain an
absolute descendant, or publish nested scroll geometry cannot leak those facts
from the first diagnostic round into the committed output.

Tests cover one-line stability, wrapped reflow, multiple struts, all-collapsed,
baseline alignment, stretch, gaps, order, row/column/reverse/wrap-reverse, all
ten flow mappings, replaced controls, intrinsic child measurement, min/max,
overflow and scrollbar settling, absolute descendants, cold/warm cache, failed
measurement atomicity, and f32/f64 equivalence.

## 9 FRI-07.9 Cross-Capability Composition

FRI-07 acceptance includes these composed controls:

| Existing contract | Required FRI-07 composition |
| --- | --- |
| `ItemOrder` | Normal, intrinsic-basis, and collapsed items use one stable order-modified sequence while outputs retain source indices. |
| `FlexAxes` | Every margin edge, intrinsic main-axis measurement, strut cross axis, reversal, and physical output projects through the existing axis owner. |
| `item_is_replaced` | Existing automatic minimum and aspect-ratio behavior remains distinct for normal and first-round collapsed measurement. |
| normalized overflow | Existing automatic-minimum, gutter, scroll range, and settled-scrollbar behavior remains correct; collapsed first-round overflow is never committed. |
| property sizing | Only flex-basis `MinContent` and `MaxContent` direct behavior cells become supported. Other property/algorithm cells do not change. |
| transaction/cache | Failed first or second round commits no partial public output; cold and warm output agree for all fields. |

Property tests generate bounded finite inputs over normal/collapsed state,
order, flow mapping, wrap, intrinsic basis, auto-margin edge pattern, replaced
state, and overflow pair. They assert finite non-negative box sizes, stable
source association, at-most-two collapse rounds, cache equivalence, no collapsed
scroll contribution, and scalar agreement within the existing tolerance.

## 10 FRI-07.10 Browser, Oracle, And Artifact Contract

### 10.1 Finite Fixture Set

The exact six Surgeist-authored sources are:

| Source | Required behavior |
| --- | --- |
| `html/flex/fri07_cross_auto_margin_overflow.html` | Cross-start/cross-end used margins and placement for positive and negative remaining space. |
| `html/flex/fri07_absolute_auto_margin_insets.html` | Auto-inset zeroing, inset-modified positive distribution, and negative inline/block behavior. |
| `html/flex/fri07_intrinsic_flex_basis.html` | Distinct min-content and max-content basis geometry with grow/shrink controls. |
| `html/flex/fri07_collapsed_strut_single_line.html` | One-line cross-size stability, baseline, multiple struts, and zero collapsed output. |
| `html/flex/fri07_collapsed_strut_wrapping.html` | Zero-main second-round line collection with retained collection margins/gaps, post-collection gap suppression, and changed wrapping. |
| `html/flex/fri07_flex_composition.html` | Order, vertical/sideways flow, replaced sizing, overflow, and collapse composition. |

Each source has the existing four generated box-sizing/direction variants, for
exactly 24 owned rows. Starting from the FRI-07 base, the final report accounts
for all 24 as generated browser passes or exact reviewed expected-fail rows.
The 16 unrelated unsupported rows remain unchanged, and quarantine and
failed-to-generate remain zero. The manifest records exact source ownership and
does not import, fetch, or mirror WPT.

The helper serializes computed facts. It does not inspect expected XML, fixture
filename, Surgeist output, sibling source name, or browser geometry to decide
`flex-item-collapse`. Parser negative controls reject `visible`, `hidden`, CSS-
wide values, malformed tokens, duplicate/conflicting records, and expectation-
derived state. Renaming a fixture or changing only expected geometry leaves
parsed layout input byte-for-byte equivalent.

### 10.2 Known Chrome Measurement Failure Exception

Chrome remains authoritative unless every item below is satisfied. Uncertainty,
a Surgeist disagreement, a Taffy result, or a synthetic expected value alone
leaves the browser mismatch blocking.

1. Reduce the behavior to one exact source and generated variant set. Record the
   pinned Chrome version/platform, browser-observed values, specification-
   required values, and smallest reproducer independent of fixture parsing.
2. Cite an unambiguous normative CSS rule or pinned WPT expected result. Supply
   one independent corroboration: another browser engine, a distinct existing
   WPT oracle, or a complete invariant derivation from directly measured inputs
   that does not depend on Surgeist output. Two Chrome APIs are not independent.
3. Prove serialized layout-ready input is correct and independent of name and
   expectation, and that the discrepancy exists before comparison. Any lowering,
   coordinate, rounding, used-margin, or tolerance ambiguity disqualifies it.
4. Add a public-front-door synthetic regression with explicit layout-ready input
   and specification-required geometry. It fails before correction and passes
   afterward; a private helper test is not a substitute.
5. Record exact source/variants, observed and required values, reason, normative
   and corroborating evidence, minimized reproduction, synthetic test,
   manifest/report disposition, and revalidation trigger in the implementing
   cycle plan. The registry is empty when no entry qualifies.
6. Use existing manifest `expected-fail` only when it can express exactly the
   proven source/variant set. Quarantine is never an alternative. A browser pin,
   specification, corroborating engine, or WPT expectation change reopens the
   entry.

### 10.3 Generation Discipline

Scoped existing-pinned generation is permitted during fixture/parser iteration
as a diagnostic and does not count as verification evidence. After all HTML,
helper, parser, manifest, and behavior inputs settle, run the existing-pinned
unfiltered full generator exactly once. Verify the resulting 24-row delta,
provenance, report inventory, XML inventory, corpus/Taffy checks, and clean
generated tree. Do not rerun the full generator merely to repeat evidence.

If the single full run reveals a genuine input or production defect, that run is
diagnostic: correct the defect, settle all inputs again, and run one replacement
full generation. Record why the prior run was invalid. Repeated runs over
unchanged inputs are forbidden.

## 11 FRI-07.11 Module And Code Outline

| Area | Required change |
| --- | --- |
| `src/node_input.rs` | Define/default `FlexItemCollapse`; add the field to both default paths and non-box construction. |
| `src/lib.rs` | Reexport and document the normalized collapse boundary and completed intrinsic-basis behavior. |
| `src/compute.rs` | Preserve `MinContent` and `MaxContent` in `ResolvedFlexBasis`; dispatch and errors remain typed. |
| `src/sizing.rs` | Support exactly the two direct flex-basis behavior cells and retain every later-owned unsupported cell. |
| `src/flex.rs` | Implement intrinsic constraint selection, both auto-margin equations, finite collapse rounds, strut association, zero-output publication, and existing-contract composition. |
| `src/flex_tests.rs` and focused test support | Add RED-first public/front-door, scalar, oracle, property, cache, failure, and regression evidence; replace the wrong abspos expectation. |
| `tests/layout/browser_parity/support.rs` | Parse only the exact normalized collapse attribute and test its independence and rejection surface. |
| `tests/layout/browser_parity/scripts/gentest/test_helper.js` | Serialize computed collapse for the exact layout-ready field without parsing authored CSS. |
| `tests/bin/surgeist-layout-generate/generator.rs` | Serialize the one new fixture attribute and verify exact report/inventory behavior; no architecture change. |
| `tests/layout/browser_parity/html/flex`, `xml/flex`, `corpus.toml`, report | Add only the six sources, 24 generated variants, provenance, and exact accounting. |
| `README.md` | Describe the new layout-ready collapse effect and root lowering boundary when the public surface changes. |

Private helper/type boundaries may differ from this table when they preserve the
specified phases and reduce complexity. There is no authorization for a new
module, reusable parser, script, lint, CI rule, dependency, feature, or generator
path.

## 12 FRI-07.12 Verification And Negative Controls

At minimum, focused evidence proves:

1. default/generic/public construction and crate-root reexport of
   `FlexItemCollapse`, with no general visibility surface;
2. direct `MinContent`/`MaxContent` dispatcher support and unchanged exact
   capability payloads for every later-owned flex-basis member;
3. distinct `20`/`100` intrinsic provider results through real flex layout,
   including grow/shrink, box-sizing, replaced, orthogonal, error, cache, and
   both scalar lanes;
4. every ordinary cross-axis auto-margin row in `FRI-07.6.1`, including used
   signed margins and all flow mappings;
5. every absolute auto-margin row in `FRI-07.6.2`, including the corrected
   original characterization and containing inline-start under RTL/vertical
   flow;
6. first-round line-size strut capture, exactly one second round, zero-main line
   recollection, gap suppression, largest-strut floor, and no third round;
7. collapsed zero output and hidden descendants with no baseline, intrinsic,
   content, scroll, target, or absolute-descendant contribution;
8. order, writing mode, reversal, replaced sizing, overflow, scrollbar settling,
   cache, transaction, rounding, and f32/f64 composition;
9. fixture parser/helper/generator negative controls proving input independence,
   exact token rejection, and no fixture-name or expectation dispatch;
10. exact 24-row browser accounting, with every expected fail satisfying
    `FRI-07.10.2`, or an empty registry and zero expected-fail rows;
11. one settled full regeneration followed only by read-only artifact checks;
12. unchanged dependencies, features, lockfile, browser pin, launch profile,
    Taffy import, MSRV, root-owned artifacts, and unrelated corpus rows; and
13. zero executable Surgeist-owned `unsafe`, no new allow/expect/suppression,
    clean diff checks, configured formatting, check, test, Clippy, generator-
    feature, full parity, corpus, and Taffy gates.

The configured acceptance commands derive from `justfile`, `Cargo.toml`, and the
browser-parity README. They include `just verify`, `just verify-generator`,
`just parity-all`, `just corpus-check`, and `just taffy-check`, plus focused
task commands and repository-wide owned-Rust unsafe scanning. All Cargo commands
run locked/offline with already-present tooling.

## 13 FRI-07.13 Architecture And Sprawl Containment

The final candidate has these structural invariants:

- one `FlexAxes` owner selects every flex edge and physical projection;
- one order-modified collected item sequence carries normal and collapsed state;
- one typed resolved-flex-basis state preserves each supported semantic basis;
- one ordinary cross-auto-margin helper and one absolute inset-aware margin
  helper implement their different equations;
- one finite two-round collapse orchestration owns strut capture and replay;
- no first-round output is committed or observable;
- no fixture-specific branch enters production layout;
- no copied parser, HTML traversal layer, expected-geometry synthesis, or second
  generator path exists; and
- no new lint allowance or suppression hides touched complexity.

After the behavior/artifact candidate is published, an independent holistic
sprawl review inspects the complete FRI-07 range and directly affected flex,
sizing, model, test-support, fixture, and generator boundaries. Every actionable
in-initiative finding is validated against source and either implemented in the
last FRI-07 cycle or disproven with an exact counterexample. That cycle may
consolidate only behaviorally equivalent code with characterization evidence.
Unrelated crate-wide advisory lint cleanup remains outside FRI-07.

If review discovers a genuine behavior, input-honesty, or artifact defect, the
owning contract reopens and receives RED-first correction before final holistic
review. A mechanical-only result preserves public API, geometry, fixtures,
generated artifacts, report counts and hashes, dependencies, features, and all
59 finding-owner assignments.

## 14 FRI-07.14 Root Handoff And Documentation

The leaf handoff records:

- final remotely verified `surgeist-layout` candidate SHA and authority remote
  readback;
- public `FlexItemCollapse` and `NodeInputOf::flex_item_collapse` semantics;
- the required root computed-style lowering from flex-item
  `visibility: collapse` to `Collapsed`, with `Normal` otherwise;
- no root-owned rendering visibility or CSS parser move into layout;
- exact six-source/24-row artifact inventory, report and helper hashes, and any
  reviewed known-Chrome-failure registry;
- all focused/full verification and independent review evidence; and
- confirmation that root API generation, facade wiring, and gitlink promotion
  remain separate root-owned work.

Crate docs and README describe the normalized layout effect, intrinsic basis
behavior, and root ownership without claiming general visibility, inline-flex,
positioned-layout completeness, or release/WPT completion.

## 15 FRI-07.15 Finding Closure Matrix

| Finding | Closure evidence |
| --- | --- |
| `FLEX-002` | Cross-axis auto margins implement the signed start/end matrix through `FlexAxes`; used margins and physical geometry pass focused, scalar, property, browser, and composition evidence. |
| `FLEX-003` | Absolute flex-child auto margins use resolved insets and inset-modified containing size, including auto-inset zeroing and negative inline-start handling; the wrong legacy test is replaced. |
| `FLEX-004` | `MinContent` and `MaxContent` remain typed from public value through dispatch and distinct constrained measurement; every later-owned basis remains exactly unsupported. |
| `FLEX-005` | Public normalized collapse state drives a finite two-round algorithm with line struts, rewrapping, zero committed item geometry, and no leaked contribution. |

No later initiative may claim closure for these IDs. FRI-09, FRI-10, FRI-11,
FRI-12, and FRI-13 consume this completed flex behavior without reopening it.

## 16 FRI-07.16 Initiative Acceptance

FRI-07 is complete only when:

1. all four findings satisfy `FRI-07.15` through the public layout front door;
2. the public collapse model has exactly the specified two states and ownership;
3. margin equations and output used values satisfy every row in `FRI-07.6`;
4. intrinsic flex bases remain distinct through measurement and final layout;
5. collapse uses exactly two finite rounds when needed, redoes wrapping, floors
   line cross size by per-item struts, and commits no first-round residue;
6. order, flow, replaced, overflow, cache, transaction, scalar, and error
   composition passes without duplicate owners or approximations;
7. all 24 owned fixture rows are visibly accounted, every Chrome exception meets
   `FRI-07.10.2`, and unsupported/quarantine/failure accounting is unchanged;
8. exactly one settled full regeneration owns final fixture/report/XML changes,
   with no redundant generator run or architecture expansion;
9. the final sprawl cycle validates and resolves every applicable FRI-07
   opportunity while preserving the completed contract;
10. all configured focused and full gates, independent task reviews, holistic
    reviews, publication, remote readback, and cleanup are complete; and
11. the candidate handoff records the public/root boundary and exact evidence
    required for later P01 work.
