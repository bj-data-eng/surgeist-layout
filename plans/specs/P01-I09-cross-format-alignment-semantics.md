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
  `c6e6f142caed39d5f301516580b42b8731373adb92a0bb35ab70937659f0a8f6`;
- helper:
  `c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36`;
- `all.json`:
  `c10dc550737010f0f854f54d1d4ca0d9f63323fa897db8258d53a3b352e06c0e`;
- XML inventory:
  `a98d1ccc4dd041415587852416572ac37360299d281c80abf19ec989461656ae`;
  and
- report/XML lineage:
  `bad8e418267cbe9537aa4280ca5cb97320d754df5be330249fffa42957264fdf`.

These hashes are controls until a reviewed, explicitly authorized artifact
cycle replaces them transactionally.

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

`LineBreakInputOf`, atomic inline input, block constants, and every other
alignment-bearing generic type use the same scalar lane. Converting an `f64`
vertical-alignment payload to `f32` through an implicit default alias is not
allowed.

## 6 FRI-09.6 Shaping-Owned Justification Contract

### 6.1 Opportunity input

Justification opportunity discovery belongs to shaping. The leaf adds a public
validated `InlineJustificationOpportunityOf<S>` with a default-scalar alias.
It stores one strictly positive finite distribution weight. There is no public
unchecked constructor.

`ShapedInlineSegmentOf` and `AtomicInlineParticipationOf` each gain an optional
opportunity that applies after that participant in source order. Existing
constructors either gain an explicit option or are replaced by complete
constructors; convenience builders may supply `None`, but no internal heuristic
may synthesize an opportunity from whitespace, break kind, source text,
segment identity, or fixture metadata.

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
`BaselineContentAdjustmentOf<S>`. It contains a logical axis, first-or-last
preference, and a finite non-negative adjustment applied at exactly one edge of
the child's content box. It has an explicit zero/default state.

`ComputeInputOf` carries this adjustment for recursive layout. `CacheKeyOf`
includes it exactly. Every child-input constructor either propagates an
explicit adjustment or deliberately supplies zero. Direct public leaf
constructors always supply zero.

The adjustment behaves like parent-reserved interior space for child layout:
it changes the child's available content geometry, descendant positions,
reported baselines, intrinsic contribution where the specification requires,
scroll geometry, and final output. It does not mutate public padding or expose a
second box model.

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
distance to the target. Direction and writing-mode reversal are handled by
logical edge projection, not by negating the adjustment.

### 9.3 Two-phase convergence

The parent performs an unadjusted measurement/layout phase, computes the group
target, and re-lays out only children with a nonzero changed adjustment. The
adjusted result then participates in final line/track sizing and container
intrinsic contribution. If the adjusted result changes the selected baseline,
the group target is recomputed once from the adjusted facts and must reach the
same monotone maximum. Any implementation needing an unbounded retry loop is a
design defect.

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

FRI-09 adds exactly 18 authored HTML sources, each expanded through the existing
four writing-mode/direction variants, for 72 generated XML outputs:

1. logical text start/end and physical left/right;
2. centered text with bidi visual order;
3. justified text with two unequal opportunities;
4. justified forced and paragraph-final lines;
5. justified text with trailing discarded whitespace;
6. justified mixed bidi segments;
7. baseline shift positive and negative;
8. text-top and text-bottom;
9. middle with explicit x-height fact;
10. line-top and line-bottom oversized participants;
11. block content start/end/center;
12. block safe overflow alignment;
13. block alignment with inline lines and floats;
14. flex first-baseline content alignment;
15. flex last-baseline content alignment;
16. grid first/last baseline content alignment;
17. subgrid baseline propagation; and
18. positioned-child and no-baseline fallback controls.

The reviewed sequence may split these sources across cycles, but it may not add
unreviewed open-ended fixture families. Any source-count change requires a
reviewed specification revision before generation.

### 11.2 Typed fixture facts

The existing helper's typed inline marker schema may be extended with:

- one justification weight per shaped or atomic participant;
- the resolved line-alignment and last-line-alignment state;
- finite signed baseline shift;
- finite non-negative parent text-over/text-under distances; and
- finite non-negative parent x-height.

The parser validates counts, identity association, finiteness, sign rules, and
the absence of a trailing justification opportunity. Malformed, missing,
duplicate, non-finite, negative where forbidden, or length-mismatched facts are
rejected with typed diagnostics. The helper may read computed styles to produce
these facts; it must not calculate expected Surgeist geometry.

The generator must continue to parse current sources and preserve all prior
outputs byte-for-byte outside the 72 new variants unless a separately proven
genuine generator defect requires a reviewed correction.

### 11.3 Permission and transaction

Browser execution, corpus acquisition, generator execution, and artifact
replacement require the explicit external-software permission and reviewed
cycle authorization required by the Surgeist workflow. No earlier initiative's
permission is inherited.

An authorized artifact transaction must:

1. freeze the pre-run manifest, helper, report, XML lineage, counts, browser
   binary, and pinned version;
2. run the authoritative generator exactly once unless a reviewed correction
   authorizes another run;
3. prove the exact 18-source/72-output addition and zero unrelated body delta;
4. write provenance only to the single canonical report, never XML comments;
5. validate schema 3 identities and generated/unsupported/expected-fail/
   quarantined/failed counts;
6. run the browser-free comparison and all focused ownership filters; and
7. commit the complete source/helper/parser/report/XML transaction atomically.

## 12 FRI-09.12 Testing Contract

Every behavior-changing task starts from assertion-level RED evidence unless
the task is purely mechanical and its reviewed plan defines an equivalent
characterization gate. Tests cover both `f32` and `f64` unless the boundary is
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
- finite browser-parity ownership rows when the artifact cycle is authorized.

Existing FRI-01 through FRI-08 focused families remain regression gates. FRI-10
positioning, FRI-11 fragmentation, FRI-12 display/table, and FRI-13 aggregate
rows remain negative controls and are not silently re-owned.

The initiative final gate includes, at minimum:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --features layout-golden-generate --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

It also includes the repository's current `just verify`, `just
verify-generator`, corpus/Taffy checks when their already-pinned inputs are
available without acquisition, the exact owned-Rust unsafe scan, suppression
inventory, scope proof, artifact lineage proof, and all reviewed focused
commands.

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
Any exception requires a reviewed specification revision and cannot be granted
only in a task plan.

## 14 FRI-09.14 Implementation Boundaries And Sequence Constraints

The implementation sequence must preserve these dependency boundaries:

1. land the public model and shared policy before format algorithms consume it;
2. land text alignment and justification before vertical alignment so line
   selection/output carriers settle once;
3. land vertical alignment before block content alignment so aligned inline
   subject envelopes are authoritative;
4. land block content alignment before parent baseline coordination so the
   child adjustment composes with its own content policy;
5. land flex and grid baseline coordination only after cache identity carries
   the adjustment;
6. perform the permission-gated browser adapter/artifact transaction after all
   owned production behavior is green; and
7. run a whole-crate sprawl review last, then contain every accepted FRI-09
   finding in the originating initiative.

A conforming sequence is expected to use at least these cycles:

- C01: public model, validation, shared policy, reexports, and docs;
- C02: text alignment, shaping-owned opportunities, justification, and output;
- C03: resolved vertical alignment and inline metric grouping;
- C04: block-container content alignment;
- C05: flex/grid/subgrid baseline content-alignment coordination and cache;
- C06: finite parser/helper/browser-artifact transaction, permission-gated;
- C07: whole-crate sprawl review, containment, final verification, and
  publication.

If the sprawl review or another reviewed discovery cannot fit within eight
tasks, the coordinator must author and review another cycle. It must not omit,
compress, or defer an accepted finding to satisfy a nominal cycle count.

Each implementation task has one exact ownership envelope and one independent
task review. Fixes use a fresh worker and receive a full fresh review. Each
cycle receives a fresh holistic review before publication, is pushed with an
explicit lease, is read back from the authority remote, and ends with
`cargo clean` plus stale-process verification.

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
9. finite browser-parity evidence is committed transactionally if and only if
   the required external permission is granted;
10. no fixture identity or expected geometry enters layout input;
11. all prior initiative controls, full package tests, strict Clippy matrices,
    formatting, artifact, suppression, unsafe, and scope gates are green;
12. a whole-crate sprawl review has been completed and every accepted FRI-09
    finding is contained in the initiative, using another reviewed cycle when
    necessary;
13. every task and cycle has the required independent CLEAN verdict; and
14. the immutable candidate is published with an explicit lease, read back,
    process-clean, worktree-clean, and followed by `cargo clean`.

The leaf handoff records the final commit, reviewed specification and sequence
semantic hashes, task and holistic verdicts, verification matrix, public API
delta, artifact lineage, remote readback, and remaining ownership. Root then
owns facade lowering, generated API artifacts, integration verification, and
gitlink promotion. Leaf FRI-09 publication does not authorize those root
mutations.
