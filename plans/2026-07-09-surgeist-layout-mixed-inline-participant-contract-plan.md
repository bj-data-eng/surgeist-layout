# Mixed Inline Participant Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the layout-owned mixed inline participant contract needed for future measured text fragments, atomic inline boxes, and forced line-break controls to share one inline formatting context without moving text shaping, style resolution, DOM normalization, or adapter ownership into `surgeist-layout`.

**Architecture:** This implements Phase 7 from `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md` as a planning/specification task only. The result is a crate-local contract spec plus refreshed coordination notes; no runtime code, public API, fixture generation, or generated XML changes are part of this plan. The spec should describe the typed layout-ready data that layout can consume later, the data other crates must produce, and the invariants that keep layout from recreating DOM/CSS/text engines.

**Tech Stack:** Rust 2024 design constraints, Surgeist layout planning docs, `guidance/surgeist-rust-modeling-guide.md`, existing `src/inline.rs`/`src/block.rs` inline architecture, Markdown planning artifacts.

---

## Source References

- Workflow: `AGENTS.md`
- Modeling guidance: `guidance/surgeist-rust-modeling-guide.md`
- Inline control spec: `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`
- Inline control sequence: `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`
- BR support matrix: `plans/2026-07-08-surgeist-layout-br-complete-support-matrix.md`
- BR cross-crate ledger: `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`
- Existing inline implementation: `src/inline.rs`
- Existing block integration: `src/block.rs`
- Existing inline tests: `src/inline_tests.rs`
- Browser parity harness: `tests/layout/browser_parity/support.rs`

## Scope

This plan does:

- create a mixed inline participant contract specification under `plans/specs/`;
- define the future layout-ready participant categories and invariants for:
  - atomic inline boxes;
  - forced line-break controls;
  - measured text fragments;
  - inline fragment boundaries only if the spec proves they are needed;
- identify what data layout needs from text/style/root/retained and what remains outside this crate;
- update the BR support matrix so it no longer says vertical writing-mode line breaks are unsupported;
- update the cross-crate ledger with the mixed inline participant producer requirements;
- require a clean-context review against the current code, modeling guide, and crate boundary.

This plan does not:

- change `src/inline.rs`, `src/block.rs`, `src/node_input.rs`, or public Rust APIs;
- implement measured text layout;
- implement inline fragment boundaries;
- implement text shaping, font lookup, whitespace collapsing, bidi segmentation, CSS parsing, DOM anonymous wrappers, or retained tree normalization;
- add browser parity fixtures or regenerate XML;
- add compatibility aliases or temporary lowering layers.

## Files

- Create: `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`
  - Defines the future mixed inline participant contract and cross-crate input requirements.
- Modify: `plans/2026-07-08-surgeist-layout-br-complete-support-matrix.md`
  - Refresh stale vertical writing-mode and browser fixture generation status.
  - Keep mixed text and outside-context work explicitly pending.
- Modify: `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`
  - Add root/style/text/retained follow-up entries needed to produce layout-ready measured text and inline participants.

No source files or generated artifacts should change.

## Task 1: Create Mixed Inline Participant Contract Spec

**Files:**
- Create: `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`

- [ ] **Step 1: Add the spec header and purpose**

Create the file with this header:

```markdown
# Surgeist Layout Mixed Inline Participant Contract Specification

## Purpose

This specification defines the layout-owned contract for future mixed inline
formatting contexts that contain atomic inline boxes, forced line-break controls,
and measured text fragments in one line-building stream.

The contract is layout-ready by construction. `surgeist-layout` may consume
measured inline participants and calculate line geometry, baselines, intrinsic
sizes, wrapping, and logical-to-physical placement. It must not classify DOM
nodes, parse CSS, shape text, choose fonts, collapse whitespace, perform bidi
segmentation, synthesize anonymous DOM wrappers, or call sibling crates to fill
missing data.
```

- [ ] **Step 2: Add ownership boundaries**

Append this section:

```markdown
## Ownership Boundary

Layout owns:

- typed layout-ready inline participant inputs;
- inline line construction over those participants;
- line metric aggregation and baseline reporting;
- intrinsic inline-size calculation over participant advances and forced breaks;
- logical-to-physical placement for supported writing modes;
- validation that layout-ready invariants are present and internally coherent.

Other crates own:

- DOM/retained tree classification and anonymous wrapper normalization;
- CSS parsing, cascade, inheritance, and computed style resolution;
- font selection, glyph shaping, text segmentation, whitespace collapsing, and
  bidi reordering;
- conversion from shaped text runs and style data into layout-ready measured
  inline participants;
- root-owned orchestration between retained, style, text, and layout.

Layout must reject or leave unsupported states explicit when these upstream
inputs are absent. It must not recover by deriving font metrics from CSS strings,
inspecting text content, or treating general inline DOM as block or atomic box
fallbacks.
```

- [ ] **Step 3: Add current starting point**

Append this section:

```markdown
## Current Starting Point

Current inline layout is centered on `src/inline.rs`:

- `AtomicInlineInput<S>` contains one ordered list of `AtomicInlineItem<S>`.
- `AtomicInlineItem<S>` currently has `Box(AtomicInlineBoxItem<S>)` and
  `ForcedLineBreak(ForcedLineBreakControlOf<S>)`.
- `ForcedLineBreakControlOf<S>` already carries order, flow, metrics, alignment,
  and clear.
- `layout_atomic_inline_items` supports horizontal and vertical forced breaks
  for atomic boxes and line-break controls.
- `src/block.rs` builds atomic inline runs from inline-level box children and
  `LayoutInputOf::LineBreak`.

Current layout does not have a public or internal measured text participant
type. Browser parity support has fixture-only text measurement helpers, but
those are test support and must not become the production layout/text contract.
```

- [ ] **Step 4: Add participant categories**

Append this section:

```markdown
## Participant Categories

The future mixed inline stream should be a closed typed domain. The exact Rust
names may be chosen by a later implementation plan, but the semantic categories
are:

```rust
pub(crate) enum InlineParticipantOf<S: LayoutScalar = DefaultScalar> {
    AtomicBox(AtomicInlineBoxParticipantOf<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    MeasuredText(MeasuredTextParticipantOf<S>),
}
```

`InlineParticipantOf<S>` is not a generic transport bag. Each variant must carry
only data needed by layout for that participant's line construction behavior.
Future inline fragment boundaries should be added only if a later plan proves
that layout needs a typed boundary participant for geometry or baseline
calculation; they should not be added just to mirror DOM nodes.
```

- [ ] **Step 5: Add atomic box contract**

Append this section:

```markdown
## Atomic Box Participant Contract

Atomic inline boxes are existing layout-owned box outputs adapted into an inline
formatting context.

Required layout-ready data:

- stable order;
- border-box size;
- content size;
- margin, padding, border, and scrollbar size;
- first or last baseline participation, when available;
- resolved vertical alignment in layout-ready form.

Layout-owned behavior:

- contributes inline advance and block-axis metrics;
- participates in wrapping and intrinsic inline-size calculation;
- maps logical line placement to physical output location;
- preserves box decorations and overflow contribution already computed by the
  box layout path.

Non-goals:

- atomic boxes must not carry DOM tag names or CSS syntax;
- atomic boxes must not be used as a fallback representation for measured text;
- text that should shape with adjacent text must not be coerced into atomic
  boxes to avoid implementing mixed inline layout.
```

- [ ] **Step 6: Add forced line-break contract**

Append this section:

```markdown
## Forced Line-Break Participant Contract

Forced line breaks are already modeled by `ForcedLineBreakControlOf<S>` and
`LayoutInputOf<S>::LineBreak(LineBreakInputOf<S>)`.

Required layout-ready data:

- stable order;
- `InlineFlowOf<S>` containing writing mode, direction, and available inline
  extent;
- validated `InlineMetricsOf<S>`;
- layout-ready alignment;
- resolved `Clear`.

Layout-owned behavior:

- terminates the current line;
- contributes line metrics and baseline data;
- creates metric-bearing empty lines when consecutive breaks occur;
- has zero output size and no box decorations;
- applies clear only from resolved `Clear`, never from HTML attributes or CSS
  strings.

Non-goals:

- no separate vertical line-break type;
- no DOM `<br>` classification in layout;
- no inference of line metrics from `font-size` or `line-height` text.
```

- [ ] **Step 7: Add measured text participant contract**

Append this section:

```markdown
## Measured Text Participant Contract

Measured text participants are future layout-ready inline participants produced
outside `surgeist-layout` after style resolution and text shaping.

Required layout-ready data:

- stable order;
- logical inline advance;
- logical block-axis metrics:
  - baseline;
  - line extent;
  - after-baseline extent, either explicit or derivable from the validated pair;
- optional ink/content overflow in logical coordinates if root/text expects
  layout to include text overflow in `content_size`;
- break behavior already resolved into participant boundaries and opportunities
  that layout is allowed to consume.

Data layout must not require:

- raw text strings;
- font family names;
- font handles;
- glyph IDs;
- grapheme clusters;
- bidi levels;
- CSS `white-space`, `text-transform`, `letter-spacing`, or `word-break` syntax;
- DOM node identities beyond a stable output order or an explicit owner-provided
  output association.

Layout-owned behavior:

- treats measured text as an inline participant with advance and metrics;
- wraps only at owner-provided boundaries/opportunities;
- aggregates text metrics with atomic boxes and forced line breaks;
- places the output association point or fragment geometry if the later public
  contract requires text output nodes.

Non-goals:

- no shaping or measuring text in layout;
- no whitespace collapsing in layout;
- no bidi reordering in layout;
- no font fallback or glyph-level overflow calculation in layout.
```

- [ ] **Step 8: Add boundary/open decisions section**

Append this section:

```markdown
## Decisions Required Before Runtime Implementation

A later implementation plan must answer these before changing Rust APIs:

1. Whether measured text participants are internal-only data supplied by root, or
   whether a public layout-ready text fragment type is needed.
2. Whether text output geometry belongs in layout outputs now, or whether root
   keeps text fragment output association outside the initial layout contract.
3. Whether inline fragment boundaries are needed by layout, or whether root can
   flatten retained/style/text output into participant runs before layout.
4. How owner-provided wrap opportunities are represented without pulling
   Unicode line breaking or CSS white-space handling into layout.
5. How scalar-generic text metrics are produced for both `f32` and `f64` layout
   lanes without narrowing.

Until these decisions are made, runtime work should stay private/internal and
avoid public API exposure.
```

- [ ] **Step 9: Add verification and cross-crate requirements**

Append this section:

```markdown
## Verification Requirements For Later Runtime Plans

Later runtime implementation plans should include:

- unit tests proving mixed participant line metric aggregation;
- unit tests proving forced breaks split measured text segments;
- intrinsic-size tests over text, atomic boxes, and forced breaks;
- horizontal and vertical writing-mode tests when measured text metrics are
  layout-ready;
- tests proving layout rejects missing or invalid metrics instead of inferring
  them;
- browser parity fixtures only after root/text/style can provide complete
  layout-ready measured participants.

## Cross-Crate Requirements

Root/style/text/retained follow-up work must provide:

- retained/root classification of inline formatting contexts and anonymous
  wrappers;
- style-owned computed values for display, writing mode, direction, alignment,
  and text-related properties;
- text-owned shaping and measurement into scalar-compatible logical advances
  and metrics;
- root-owned conversion into ordered layout-ready participant streams;
- fixture or integration tests proving the single production path does not
  duplicate browser-parity fixture lowering.
```

- [ ] **Step 10: Run spec artifact checks**

Run:

```sh
PLACEHOLDER_PATTERN='T''BD|TO''DO|FIX''ME|implement'' later|fill'' in|Similar'' to|appropri''ate'
rg -n "$PLACEHOLDER_PATTERN" plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md
```

Expected: no matches.

Run:

```sh
rg -n "parse CSS|shape text|font lookup|DOM classification|anonymous wrapper|compatibility alias" plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md
```

Expected: matches only in boundary/non-goal text, not in layout-owned responsibilities.

## Task 2: Refresh BR Matrix For Completed Vertical Work

**Files:**
- Modify: `plans/2026-07-08-surgeist-layout-br-complete-support-matrix.md`

- [ ] **Step 1: Update vertical writing status**

Replace the `Vertical writing modes` row with:

```markdown
| Vertical writing modes | Vertical `<br>` advances in the relevant block/inline axes. | Supported for layout-ready `LineBreakInputOf<S>` in constrained block inline-run contexts when `Clear::None`; browser parity has layout-owned vertical fixtures. Vertical `clear` and complex subgrid/baseline vertical `<br>` cases remain unsupported until their surrounding contracts are reviewed. | Preserve logical-axis forced-break behavior and expand only through layout-ready participant contracts. Keep vertical clear explicitly unsupported until modeled. | Style/root lower writing mode and clear; text supplies vertical metrics if needed. |
```

- [ ] **Step 2: Update browser fixture generation status**

Replace the `Browser fixture generation` row with:

```markdown
| Browser fixture generation | Browser-derived XML can express metric-bearing `<br>` cases. | Supported for horizontal and constrained vertical layout-owned fixtures; remaining unsupported buckets are explicit. | Keep XML parser strict: complete metric pairs only, fixture syntax only. | Root-owned generators/schemas coordinate with layout-ready contract. |
```

- [ ] **Step 3: Update completion checklist**

Change:

```markdown
- [ ] Define and implement vertical writing-mode line-break geometry.
```

to:

```markdown
- [x] Define and implement vertical writing-mode line-break geometry for layout-ready block inline-run contexts.
```

Add this checklist item directly after it:

```markdown
- [ ] Keep complex vertical `<br>` cases with subgrid/baseline dependencies explicitly unsupported until those surrounding contracts are implemented.
```

- [ ] **Step 4: Update planning notes**

Replace the final paragraph beginning with `Vertical writing and broader outside-context support` with:

```markdown
Vertical writing-mode forced breaks are now supported for layout-ready block
inline-run contexts. Broader outside-context support remains larger because it
touches inline formatting context modeling, anonymous wrapper ownership, and
mixed inline participant streams. Those should be planned through the mixed
inline participant contract before runtime code changes.
```

## Task 3: Update Cross-Crate Ledger For Mixed Inline Participants

**Files:**
- Modify: `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`

- [ ] **Step 1: Append text metrics producer entry**

Append this entry under `## Entries`:

```markdown
### Text/root need to provide layout-ready measured inline participants

- Status: `open`
- Owning crate: root `surgeist` plus `surgeist-text`
- Affected API: future mixed inline participant contract in `surgeist-layout`
- Observed behavior: layout can combine atomic inline boxes and forced
  line-break controls, but it does not yet have production layout-ready measured
  text participants that can share the same inline formatting context.
- Expected behavior: text/root should provide ordered measured text fragments
  with scalar-compatible logical advance, baseline, line extent, and any
  owner-approved wrap opportunities. Layout should consume those values without
  shaping text, parsing CSS, choosing fonts, or inspecting raw text.
- Required owning change: root should coordinate text/style/retained plans after
  the layout mixed inline participant contract is accepted.
- Verification note: layout-side runtime work should stay blocked on this
  contract until root/text can produce complete layout-ready participant data.
```

- [ ] **Step 2: Append retained/root inline context entry**

Append this entry after the text metrics entry:

```markdown
### Retained/root need to normalize mixed inline formatting contexts

- Status: `open`
- Owning crate: root `surgeist` plus retained tree integration
- Affected API: future mixed inline participant stream consumed by
  `surgeist-layout`
- Observed behavior: layout browser parity can test constrained inline runs, but
  production app trees still need root/retained ownership for inline formatting
  context boundaries, anonymous wrappers, and output association.
- Expected behavior: retained/root should present layout with normalized
  ordered inline participant streams, including atomic boxes, line-break
  controls, and measured text fragments, without requiring layout to inspect DOM
  structure.
- Required owning change: root should create integration plans once layout
  defines whether inline fragment boundaries are needed by the layout API.
- Verification note: layout must not implement fallback DOM normalization to
  unblock these cases locally.
```

- [ ] **Step 3: Run ledger checks**

Run:

```sh
rg -n "layout-ready measured inline participants|normalize mixed inline formatting contexts|shape text|DOM" plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md
```

Expected: new ledger entries are present and assign ownership outside layout.

## Task 4: Self-Review And Clean-Context Review

**Files:**
- Inspect:
  - `plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md`
  - `plans/2026-07-08-surgeist-layout-br-complete-support-matrix.md`
  - `plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md`

- [ ] **Step 1: Run plan artifact checks**

Run:

```sh
PLACEHOLDER_PATTERN='T''BD|TO''DO|FIX''ME|implement'' later|fill'' in|Similar'' to|appropri''ate'
rg -n "$PLACEHOLDER_PATTERN" plans/2026-07-09-surgeist-layout-mixed-inline-participant-contract-plan.md plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md plans/2026-07-08-surgeist-layout-br-complete-support-matrix.md plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md
```

Expected: no matches.

Run:

```sh
git diff --check
git status --short --branch
```

Expected: no whitespace errors; status shows only the three planned document changes plus this plan file.

- [ ] **Step 2: Clean-context review**

Ask a clean-context reviewer to inspect the plan and document changes against:

- `AGENTS.md`;
- `guidance/surgeist-rust-modeling-guide.md`;
- `plans/specs/2026-07-08-surgeist-layout-inline-control-item-spec.md`;
- `plans/2026-07-08-surgeist-layout-inline-control-item-sequencing.md`;
- current `src/inline.rs` and `src/block.rs`.

The reviewer must confirm:

- the plan implements Phase 7 as a contract/specification task, not premature runtime code;
- layout remains a calculation engine and does not take ownership of text shaping, CSS, DOM, retained tree, or root adapter work;
- the participant categories are typed and not a generic data bag;
- cross-crate requirements are recorded without treating them as local blockers;
- the BR matrix cleanup reflects already implemented vertical work without overclaiming complex subgrid/baseline support;
- no generated files or source code are changed.

- [ ] **Step 3: Commit after clean review**

After the review is clean, commit:

```sh
git add plans/2026-07-09-surgeist-layout-mixed-inline-participant-contract-plan.md \
  plans/specs/2026-07-09-surgeist-layout-mixed-inline-participant-contract-spec.md \
  plans/2026-07-08-surgeist-layout-br-complete-support-matrix.md \
  plans/2026-06-29-surgeist-layout-br-line-break-cross-crate-ledger.md
git commit -m "Plan mixed inline participant contract"
```

## Final Gate

Completion requires:

- the plan file exists and has no placeholders;
- the mixed inline participant contract spec exists;
- BR support matrix and cross-crate ledger are refreshed;
- clean-context review comes back clean;
- `git diff --check` passes;
- `git status --short --branch` is clean after the commit.
