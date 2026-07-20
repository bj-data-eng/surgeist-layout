# FRI-06 Inline Formatting, Floats, And BFCs Implementation Sequence

Status: reviewed

Sequence ID: `FRI-06`

Owning repository: `surgeist-layout`

## Authority

This sequence implements the independently reviewed specification at
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`,
normalized semantic-content SHA-256
`7090ea13ba7d9e524ce432018c8b7c44c1b3b76428d2c666949d297656ce97c8`,
committed as `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`.

The specification owns behavior, API, compatibility, ownership, artifacts,
errors, and acceptance. This sequence owns only durable dependency order. Only
the next ready cycle receives a detailed just-in-time plan.

## Sequence Boundary

Every cycle mutates only `surgeist-layout`. Root composition, text shaping and
source association, shape geometry, authored CSS/style lowering, facade/API
artifacts, and gitlink promotion remain the separate handoff in `FRI-06.12`.

No cycle adds a dependency, feature, MSRV change, unsafe code, generator
architecture, general text shaper/parser, general CSS parser, bidi analyzer,
shape engine, rendering state, or behavior owned by FRI-09 through FRI-13.
FRI-01's unit cache key remains unchanged. Transactional invalidation uses only
the reviewed FRI-06 entry point, exact ancestor closure, and two-phase batch
application.

Scoped generation remains optional diagnostic work rather than completion
evidence. The artifact cycle exclusively owns the bounded HTML/parser/helper,
manifest, XML, and report changes. Its final artifacts have one full unfiltered
existing-pinned no-fetch lineage after all owned inputs settle. Later closure is
read-only for generation inputs and outputs; a confirmed input defect returns to
that artifact cycle and invalidates its prior lineage.

## Activation Recovery Evidence

The immutable entry report is
`tests/layout/browser_parity/xml/generation-reports/all.json` at SHA-256
`4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`.
Matrix digests below are SHA-256 over sorted, LF-terminated
`source<TAB>variant` rows. The base-generated digest also appends
`TABoutput`. The four standard variants are `border_box_ltr`,
`content_box_ltr`, `border_box_rtl`, and `content_box_rtl`.

- **Activation matrix:** the report's 340 entries carrying one of the three exact
  D-16 transition reasons plus all four variants of the twelve sources named in
  `FRI-06.11`; 388 rows, digest
  `3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1`.
- **Fixture-correction matrix:** the report's 240 entries carrying either exact
  vertical/outside-block break reason; all four variants of
  `fri06_inline_unequal_line_alignment` and `fri06_bidi_mixed_inline`; and both
  RTL variants of `fri06_inline_mixed_text_atomic_wrap`,
  `fri06_atomic_inline_baseline`,
  `fri06_atomic_inline_percentage_block_size`, and
  `fri06_float_auto_height`; 256 rows, digest
  `35dc887d32232c365e132f38032021ae0b64147480ab7536971765b3fa5d0214`.
- **Production-correction matrix:** all four variants of
  `grid_lanes_not_inhibited_normal_packing`,
  `grid_lanes_not_inhibited_overflow_hidden_packing`,
  `subgrid_auto_track_sizing_min_content_text_runs`,
  `fri06_vertical_break_clear`, and `fri06_float_logical_clear`; plus both RTL
  variants of the sixteen
  `subgrid_alignment_<self>_<item>_item` sources formed by the exact cross product
  `{baseline,center,end,start}` x `{baseline,center,end,start}`; 52 rows across 21
  sources, digest
  `d5cd1140c094fb43fa9960ff7beb21ce52161e6b7d58ddb5f33e7dff2dee761e`.
- **C08 baseline-helper subset:** all four variants of
  `html/subgrid/subgrid_baseline_auto_columns_first_item.html`,
  `html/subgrid/subgrid_baseline_auto_columns_second_item.html`,
  `html/subgrid/subgrid_baseline_standalone_axis_first_item.html`, and
  `html/subgrid/subgrid_baseline_standalone_axis_second_item.html`; 16 rows,
  digest `f9ac335e450b4ffd014ae91ef211e699b513676711f70e2c27414fb64f7455a3`.
- **Semantic-preservation matrix:** all four variants of
  `block_br_inline_block_metrics`, `block_br_vertical_lr_inline_block_metrics`,
  `block_br_vertical_rl_inline_block_metrics`, and
  `block_br_vertical_rl_rtl_inline_block_metrics`; 16 rows, digest
  `ff3b0c67a33ed008235891b3019e4491783fd7933a37c7b50589fec6b573a8b1`.
- **Base-generated matrix:** every generated `source`, `variant`, and `output`
  tuple in the entry report; 5,324 rows, digest
  `3381162173bc2c09bbbae736391d9420c5e96c375083fb9fd0b337bcec12cffb`.

These predicates own recovery membership. Later cycle plans may partition them
into executable tasks but may not add, omit, reclassify, or dynamically widen a
row. C07 exact-base public-compute probes proved the fixed baseline-helper subset
and all four variants of `html/block/fri06_forced_break_strut.html` already
satisfy the production oracles. One bounded diagnostic proved every baseline-
helper row stops in `unsupportedChildNodesReason` via
`isSignificantInlineWhitespace` before Rust lowering; C08 therefore owns that
helper correction, while its existing twelve-source activation set already owns
the forced-break rows. This evidence replaces their invalid production
classification; it does not add rows or authorize another diagnostic derivation.
The 408 failures observed by the aggregate diagnostic are not a recovery matrix
and do not move the final aggregate gate out of FRI-13.

## Ordered Cycles

### `FRI-06-C01` Public Inline Model And Transaction Substrate

**Specification sources:** `FRI-06.4`; `FRI-06.5`; cache, validation, and error
portions of `FRI-06.6`; module and compatibility portions of `FRI-06.10`.

**Prerequisites:** Published FRI-05 candidate and the clean reviewed FRI-06
specification revision recorded above.

**Entry state:** Layout has no shaped-text participant or fragment output,
non-box tree pairing, shape-exclusion model, exact invalidation closure, or
transactional fragment/cache application surface.

**Bounded outcome:** Add the private-field scalar-generic shaped segment, bidi,
break, whitespace, text, atomic participation, fragment, float-exclusion, and
provider-query values; canonical non-box pairing; reviewed errors and public
reexports; phase-specific fragment batch output; committed-fragment readback;
unit-key-preserving invalidated layout entry; and immutable-prepare/infallible-
commit batch application. Do not replace the line algorithm or invoke the shape
provider yet.

**Observable exit evidence:** Public construction admits only reviewed states in
both scalar lanes; contradictory non-box/atomic/shape roles and unreachable
dirty subjects fail with exact diagnostics; duplicate dirty subjects normalize;
inclusive root-path closure, stale-hit bypass, failed layout/preparation, ordered
commit, empty/nonempty fragment restoration, and missing warm fragment state all
match the transaction contract without a cache revision token.

**Handoff:** Later algorithms can consume validated participants and publish
fragments without redesigning public state, errors, invalidation, or commit
atomicity.

### `FRI-06-C02` Unified Shaped-Text Line Construction

**Specification sources:** `FRI-06.4 D-01` through `D-10`; shaped text and
fragment portions of `FRI-06.5`; text, break, bidi, whitespace, alignment, and
intrinsic portions of `FRI-06.7` through `FRI-06.9`.

**Prerequisites:** `FRI-06-C01` complete and remotely verified.

**Entry state:** Validated text and fragment phases exist, while production
inline layout remains box/control-only, splits horizontal and vertical behavior,
and lacks soft wrapping, source-associated text fragments, and per-line visual
ordering.

**Bounded outcome:** Replace text-only line construction with one logical-axis
builder that consumes shaped segments, performs greedy allowed/replacement/
mandatory breaking, edge-whitespace handling, min/max-content contribution,
per-line band and legacy alignment, complete-unit bidi ordering, physical
projection for all ten flows, and phase-correct text-node/fragment publication.
The full-band interface remains ready for later float exclusion; atomic and
existing control integration remain in C03.

**Observable exit evidence:** Both scalar lanes prove deterministic wrapping,
replacement ownership, rejected replacement/discard state, mandatory/final
break behavior, overwide progress, all-discarded anchoring, visual-slot gaps,
source-order batch identity, all flow mappings, per-line alignment, intrinsic
sizes, rounding, scroll contribution, and cold/warm fragment restoration without
synthetic measured text.

**Handoff:** One reviewed line source can accept atomic boxes, controls, and
float-adjusted bands without another axis algorithm.

### `FRI-06-C03` Mixed Atomic And Control Line Completion

**Specification sources:** `FRI-06.4 D-03`, `D-06` through `D-13`; atomic,
control, baseline, percentage, and non-box portions of `FRI-06.5` through
`FRI-06.9`.

**Prerequisites:** `FRI-06-C02` complete and remotely verified.

**Entry state:** Shaped text lines work through the unified builder, while atomic
boxes, line breaks, boundaries, top/bottom alignment, percentage block basis,
vertical clear, and fixed-size baseline paths still use incomplete legacy
composition.

**Bounded outcome:** Compose text, atomic boxes, visible/hidden breaks, boundary
markers, floats/out-of-flow placeholders, and source association through the one
participant stream. Complete mixed bidi slots, struts and empty lines,
per-control geometry/comparison, logical clear in all flows, inner/fallback
atomic baselines, top/bottom placement, definite percentage block basis, and
metric-aware fixed-size behavior. Remove the separate vertical and obsolete
whole-run shortcuts owned by FRI-06.

**Observable exit evidence:** Mixed text-atomic-control lines preserve exact
source and visual identity; leading/trailing/adjacent controls, empty lines,
vertical/sideways clear, unequal line alignment, baseline fallback/margins,
top/bottom expansion, replaced atomics, definite/indefinite percentages, and
fixed fast paths complete without panic, silent omission, or guessed facts.

**Handoff:** The complete inline participant engine is ready for production
float bands and BFC placement.

### `FRI-06-C04` Rectangular Float And BFC Geometry

**Specification sources:** `FRI-06.4 D-13` and `D-15`; rectangular float, clear,
BFC, sizing, baseline, scroll, and cache portions of `FRI-06.7` through
`FRI-06.9`.

**Prerequisites:** `FRI-06-C03` complete and remotely verified.

**Entry state:** Inline lines are complete against a full containing band, while
ordinary lines can overlap floats and current float/BFC placement is physical-
horizontal, incomplete for auto size/height, and not closed over current display
and overflow roles.

**Bounded outcome:** Make rectangular exclusions flow-relative and source
ordered; place line-left/right, opposing, stacked, cleared, and overwide floats;
query full line spans monotonically; map float/clear through containing flow;
enclose owned floats in auto block size; trap nested floats; and implement the
exact current flex/grid/grid-lanes plus non-replaced-overflow BFC avoidance and
auto-inline-size predicate. Keep provider-backed shapes for C05.

**Observable exit evidence:** Every float/clear side and flow mapping, mixed line
exclusion, finite-transition progress, overwide behavior, float-only auto height,
nested containment, ordinary block edge, current BFC predicate, auto/definite
width, intrinsic behavior, scroll contribution, rounding, and invalidation path
has production front-door proof with no overlap or second side table.

**Handoff:** Margin-box exclusion is complete and the bounded provider can refine
the same band query without changing placement ownership.

### `FRI-06-C05` Provider-Backed Shape Exclusion

**Specification sources:** `FRI-06.4 D-14`; shape provider, error, band, cache,
fake, and root-handoff portions of `FRI-06.5` through `FRI-06.12`.

**Prerequisites:** `FRI-06-C04` complete and remotely verified.

**Entry state:** Rectangular exclusion is complete, while `Shape` requests do
not yet invoke the reviewed tree provider or refine bands with typed empty,
partial, full, invalid, missing, and failed results.

**Bounded outcome:** Integrate the bounded physical band query into the existing
float exclusion pass, preserve exact container/float/band diagnostics, clip and
validate provider intervals, bound query repetition, and connect provider result
changes to the reviewed dirty-float ancestor transaction. Do not add shape
identity, shape parsing, a sibling dependency, or a general geometry engine.

**Observable exit evidence:** Empty, partial, full, clipped, invalid, missing,
and failed provider results; non-float shape rejection; query bounds; cache
invalidation; failed recomputation; and cold/warm/rounded geometry agree through
the real provider and block-line front doors in both scalar lanes.

**Handoff:** The focused C05 production baseline and fixture-facing facts are
stable for adapter preparation. Activated-fixture validation and any confirmed
production correction remain explicitly owned by C07.

### `FRI-06-C06` Finite Fixture Adapter Preparation

**Specification sources:** `FRI-06.4 D-16`; browser/comparator portions of
`FRI-06.9` and `FRI-06.10`; `FRI-06.11`; artifact portions of `FRI-06.14`.

**Prerequisites:** `FRI-06-C05` and its post-C05 containment window complete and
remotely verified.

**Entry state:** Product behavior and finite production fixture-facing facts have
focused C05 evidence but have not yet faced the activation matrix; C07 owns that
validation. The browser adapter cannot compare control/fragment output or lower
shaped text, atomic participation, bottom alignment, and finite shape-band
tables. Generation inputs and derived artifacts remain frozen. Final
generator-feature verification also exposed one pre-existing lifecycle test that
still models implicit file close instead of the generator's explicit lease-release
front door.

**Bounded outcome:** Extend only the Rust fixture adapter and comparator to parse,
lower, store, and compare the reviewed control, fragment, shaped-text, atomic,
bottom-alignment, and finite shape-band facts through production constructors and
front doors. Correct only that stale generator lifecycle test to use the existing
explicit release front door. Do not edit generator implementation, helper, HTML,
manifest, XML, report, or production source and do not run generation.

**Observable exit evidence:** Strict negative controls detect every named
geometry, identity, schema, role, query, and provider mismatch. Valid controls,
fragments, shaped/atomic input, bottom alignment, and empty/partial/full shape
bands agree through the real fixture-tree and production front doors. Default and
generator-feature verification pass with generation inputs and artifacts
unchanged, and the lease lifecycle test covers explicit release and reacquisition
without relying on implicit file-close timing.

**Handoff:** The finite adapter is stable. Browser activation may now diagnose
whether the frozen production handoff and proposed fixture inputs satisfy all 388
owned comparisons before any valid final derivation.

### `FRI-06-C07` Activated-Fixture Production Corrections

**Specification sources:** `FRI-06.4 D-03`, `D-06`, `D-07`, `D-10`, `D-12`, and
`D-13`; line, intrinsic, bidi, control, flow, and clear portions of
`FRI-06.7` through `FRI-06.9`; behavioral portions of `FRI-06.14`.

**Prerequisites:** `FRI-06-C06` complete and remotely verified. Its invalid
diagnostic derivation is discarded and never counts as artifact lineage.

**Entry state:** Focused activation diagnostics and exact-base typed probes prove
52 valid FRI-06 variants across 21 sources still miss reviewed production
behavior: 12 intrinsic/track height results and 40 physical/logical placements.
The fixed baseline-helper subset instead requires C08 correction before lowering,
and all variants of `html/block/fri06_forced_break_strut.html` require only C08
fixture activation. Their typed production oracles already pass. The 408 pre-existing aggregate failures
remain outside this cycle, the final aggregate gate remains FRI-13-owned, and all
fixture-input failures remain C08-owned.

**Bounded outcome:** Correct only the confirmed FRI-06 production paths reached
by those 21 sources, preserving exact shaped-participant, intrinsic, bidi,
writing-mode, RTL, control, and logical-clear semantics through the
public compute front door. Do not edit fixture inputs or artifacts, run
generation, absorb inherited aggregate failures, or enter later-owned behavior.

**Observable exit evidence:** Exact source-level regressions prove all 52
validated production cases in both applicable directions and box models. Default
and generator-feature verification, formatting, Clippy, unsafe absence, and
unrelated focused regression suites are clean without fixture or artifact deltas.

**Handoff:** Production behavior required by the 388 owned fixture variants is
stable before helper, source, manifest, and final-lineage work resumes.

### `FRI-06-C08` Bounded Fixture Activation And Final Lineage
**Specification sources:** `FRI-06.4 D-09`, `D-11`, and `D-16`; atomic-baseline
and physical-placement portions of `FRI-06.7`, `FRI-06.9`, and `FRI-06.14`;
browser/comparator portions of `FRI-06.9` and `FRI-06.10`; `FRI-06.11`; and
artifact portions of `FRI-06.14`.
**Prerequisites:** `FRI-06-C07` complete and remotely verified; its production
and finite adapter decisions are the C08 entry handoff.
**Entry state:** Adapter and product behavior are complete. Bounded diagnostics
proved fixture-owned failures: invalid zero-line-height break metrics, collapsed-
space advance loss, mismatched bidi/float fixture semantics, atomics-plus-break
helper overreach, and grid-parent indentation whitespace misclassified as mixed
inline content for the fixed baseline-helper subset. The exact 340 existing
unsupported variants, including those 16 rows, twelve new sources including
`fri06_forced_break_strut`, manifest records, and valid final derived artifacts
are absent.
**Bounded outcome:** Correct only those finite input defects; settle the reviewed
helper/parser/serializer facts; activate all 85 existing FRI-06 sources; add
exactly twelve named sources and 48 variants; preserve semantic input for every
other source; and route the confirmed atomic-baseline defect exposed after the
reviewed input checkpoint through the sole C08-R1 production-recovery gate before
retaining the final corpus. R1 may use a scoped existing-pinned diagnostic only
when focused evidence cannot localize the defect; it retains no report or
artifact and is never acceptance evidence. It never runs a full generation.
Generator architecture remains unchanged.
**Observable exit evidence:** The final full report has 5,712 generated, exactly
16 immutable missing-root unsupported variants, `filter: null`, reviewed browser
and helper/manifest provenance, and zero failure classes or scoped reports. All
388 activated/new variants compare through an explicit frozen matrix; the other
5,324 outputs change only generator provenance; artifacts are derived rather
than hand-edited. The FRI-13 aggregate gate remains unclaimed.
**Handoff:** Generator inputs and outputs are frozen for read-only initiative
closure.

### `FRI-06-C09` Public Evidence And Leaf Candidate Closure

**Specification sources:** `FRI-06.10`; `FRI-06.12` through `FRI-06.14`.

**Prerequisites:** `FRI-06-C08` complete and remotely verified with its valid
final artifact lineage.

**Entry state:** Behavior and browser artifacts are complete, while aggregate
public docs/exports, compatibility inventory, finding trace, dead-code cleanup,
complete verification, and root/text/shape handoff remain unreconciled.

**Bounded outcome:** Reconcile the reviewed public front door and documentation,
remove every FRI-06-owned dead-code allowance and obsolete path, prove all 14
finding rows and initiative acceptance, and record the exact breaking leaf,
text/shape adapter, transactional invalidation, artifact, and root-promotion
handoff. Do not change generation inputs or outputs.

**Observable exit evidence:** Every acceptance item is traceable to current
source, focused and browser evidence, public/static negative surfaces,
transaction behavior, artifacts, docs, and compatibility accounting. Complete
default/generator verification, focused parity, corpus/Taffy checks, formatting,
warnings-denied Clippy, unsafe absence, and range/provenance review are clean.

**Handoff:** Publish the reviewed leaf candidate to remote `main`, verify remote
readback, and return its exact SHA plus the complete FRI-06 root/text/shape
integration contract.

## Sequence Completion

The sequence is complete when `FRI-06-C01` through `FRI-06-C09` satisfy their
observable exits in order and every `FRI-06.14` criterion is traceable. A later
cycle cannot begin before its predecessor is published and remotely verified.
A material specification change returns to specification review before this
sequence is revised.
