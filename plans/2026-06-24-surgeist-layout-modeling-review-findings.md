# Surgeist Layout Modeling Review Findings

> Record only verified findings from `plans/2026-06-24-surgeist-layout-modeling-review-plan.md`. Do not use this file as an implementation plan.

## Review Boundaries

- Repo: `surgeist-layout`
- Scope: crate-local source, tests, browser parity tooling, generated artifact workflow, and public API reexports from `src/lib.rs`
- Out of scope: code edits, sibling crate fixes, root integration changes, implementation plan
- Standard: `AGENTS.md` and `guidance/surgeist-rust-modeling-guide.md`, especially Type And Value Modeling, Generated Artifacts, Crate Role, Dependency Direction, and Public API Surface

## Recording Rules

- Record findings only after they have exact file and line evidence.
- Keep findings descriptive, not prescriptive. Implementation choices belong in a later implementation plan.
- Group duplicate observations under one finding when they share the same invariant.
- Preserve reviewer provenance so later reviewers can distinguish verified issues from hypotheses.
- If a suspected issue is rejected, record it under Non-Findings with the reason.

## Severity Guide

- **P0:** Current correctness bug likely reachable by normal crate use or parity fixtures.
- **P1:** Strong correctness risk, invariant confusion, or public contract that makes invalid states easy to express.
- **P2:** Internal modeling debt that can cause regressions or makes algorithm phases hard to reason about.
- **P3:** Documentation, naming, or containment issue with low immediate correctness risk.

## Finding Template

Copy this template for each verified finding:

```markdown
### FINDING-ID: Short Title

- Severity:
- Category: correctness-risk | API-hardening | internal-maintainability | tooling-containment
- Confidence: high | medium | low
- Review phase:
- Owner: layout crate | root coordinator | style/css upstream | tooling-only
- Status: verified | needs re-review | accepted | rejected
- Evidence:
  - `path/to/file.rs:line`
- Invariant at risk:
- Observed code shape:
- Expected modeling shape:
- AGENTS/modeling guidance:
- Reproduction or existing coverage:
- Notes:
```

## Findings

### LAYOUT-MODEL-CALC-SYMBOLIC-COLLAPSE: Calc Identity And Resolver-Free Paths Collapse Symbolic Values

- Severity: P1
- Category: correctness-risk
- Confidence: high
- Review phase: phases 1, 3
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/value.rs:38`
  - `src/value.rs:42`
  - `src/value.rs:139`
  - `src/value.rs:292`
  - `src/value.rs:297`
  - `src/value.rs:302`
  - `src/value.rs:380`
  - `src/value.rs:385`
  - `src/value.rs:389`
  - `src/compute.rs:72`
  - `src/compute.rs:74`
  - `src/compute.rs:168`
  - `src/compute.rs:204`
  - `src/compute.rs:313`
  - `src/compute.rs:317`
  - `src/compute.rs:321`
  - `src/block.rs:1495`
  - `src/block.rs:1536`
  - `src/block.rs:1591`
  - `src/block.rs:1630`
  - `src/block.rs:1634`
  - `src/block.rs:1646`
  - `src/tests.rs:106`
  - `src/tests.rs:118`
  - `src/tests.rs:172`
- Invariant at risk: Symbolic calc values should stay tied to a resolver/store context and should not silently become absent or zero before the layer with the right basis can resolve them.
- Observed code shape: `CalcId` is publicly constructible, missing expressions resolve as unresolved/non-basis-dependent, resolver-free `Length` and `LengthAuto` calc paths become `None`, and several compute/block paths still use resolver-free helpers that turn absent values into zero.
- Expected modeling shape: Calc identity, expression ownership, missing-expression behavior, and resolver availability are phase-visible instead of encoded as a public index plus `None`/`0.0` fallback paths.
- AGENTS/modeling guidance: Keep symbolic values symbolic; prefer semantic errors; make invalid states hard to express; use intentional front-door public APIs.
- Reproduction or existing coverage: Existing unit tests verify fabricated ids and resolver-free calc collapse behavior; reviewers found no focused leaf/container coverage for calc in the cited resolver-free paths.
- Notes: Merged from Raman's `LAYOUT-MODEL-CALC-001`, Lagrange's `LAYOUT-P3-CALC-LEAF-001`, and Lagrange's `LAYOUT-P3-BLOCK-CALC-002`.

### LAYOUT-MODEL-CACHE-KEY-CONTEXT: Cache Entries Omit Layout-Relevant Compute Context

- Severity: P1
- Category: correctness-risk
- Confidence: high
- Review phase: phase 2
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/output.rs:38`
  - `src/output.rs:39`
  - `src/output.rs:40`
  - `src/output.rs:41`
  - `src/output.rs:42`
  - `src/output.rs:43`
  - `src/output.rs:44`
  - `src/cache.rs:5`
  - `src/cache.rs:48`
  - `src/cache.rs:60`
  - `src/cache.rs:126`
  - `src/traits.rs:19`
  - `src/traits.rs:29`
  - `tests/layout/unit/cache.rs:4`
- Invariant at risk: Cache reuse should distinguish every compute input dimension and resolver-dependent semantic that can affect layout output.
- Observed code shape: `ComputeInput` carries run mode, sizing mode, requested axis, known size, parent size, and available size, while cache entries persist and compare only known/available values; calc resolver context is available through `Compute` but not part of the cache contract.
- Expected modeling shape: Cache identity is a typed snapshot of all layout-relevant phase and resolver context, or cache access is constrained so omitted context cannot affect reused output.
- AGENTS/modeling guidance: Prefer typed snapshots and explicit cache invalidation contracts; preserve symbolic context until the correct resolver/basis is available.
- Reproduction or existing coverage: Existing cache tests cover reuse for identical inputs; reviewers found no coverage varying sizing mode, axis, parent, or resolver while holding known/available constant.
- Notes:

### LAYOUT-MODEL-ASPECT-RATIO-RAW-SCALAR: Aspect Ratio Is Raw Optional Scalar

- Severity: P1
- Category: API-hardening
- Confidence: high
- Review phase: phase 1
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/node_input.rs:487`
  - `src/compute.rs:207`
  - `src/compute.rs:212`
  - `src/compute.rs:223`
  - `src/compute.rs:351`
  - `src/compute.rs:356`
  - `src/compute.rs:357`
- Invariant at risk: Aspect ratio should be positive and finite before layout arithmetic divides or multiplies by it.
- Observed code shape: `NodeInput::aspect_ratio` is `Option<Scalar>`, and sizing helpers apply it directly through division/multiplication without a semantic wrapper or validation boundary.
- Expected modeling shape: Aspect ratio carries positive/finite invariants at construction or at the authored-to-layout boundary.
- AGENTS/modeling guidance: Use semantic types over raw primitives when values carry invariants; keep invariants at construction.
- Reproduction or existing coverage: Reviewers found no focused coverage for zero, negative, NaN, or infinite aspect ratios.
- Notes:

### GRID-PLACEMENT-PUBLIC-INVALID-STATES: Public GridPlacement Admits Invalid Line And Span States

- Severity: P1
- Category: API-hardening
- Confidence: high
- Review phase: phase 5
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/node_input.rs:333`
  - `src/node_input.rs:335`
  - `src/node_input.rs:337`
  - `src/node_input.rs:348`
  - `src/node_input.rs:375`
  - `src/node_input.rs:393`
  - `src/grid/placement.rs:89`
  - `src/grid/placement.rs:105`
  - `api/public-api.txt:456`
- Invariant at risk: Layout-ready grid placement should not represent invalid line zero, zero spans, or conflicting start/end/span combinations.
- Observed code shape: `GridPlacement` publicly exposes raw optional line and span fields; constructors accept raw `isize` and `usize`; placement normalization clamps zero span with `max(1)`.
- Expected modeling shape: Layout-ready placement uses semantic line/span types or validated constructors that encode valid states.
- AGENTS/modeling guidance: Use typed models for indexes/spans; make invalid states hard to express; treat public APIs as product contracts.
- Reproduction or existing coverage: Existing tests cover invalid `RawGridLine` fallback behavior; reviewers found no equivalent coverage for public `GridPlacement::line(0)` or `GridPlacement::span(0)`.
- Notes:

### LAYOUT-PARITY-PROVENANCE-STYLE-HASH: Generated XML Provenance Omits Injected Base Stylesheet

- Severity: P1
- Category: tooling-containment
- Confidence: high
- Review phase: phase 8
- Owner: tooling-only
- Status: verified
- Evidence:
  - `tests/bin/surgeist-layout-generate/generator.rs:42`
  - `tests/bin/surgeist-layout-generate/generator.rs:1095`
  - `tests/bin/surgeist-layout-generate/generator.rs:1983`
  - `tests/bin/surgeist-layout-generate/generator.rs:2341`
  - `tests/bin/surgeist-layout-generate/generator.rs:2347`
  - `tests/layout/browser_parity/html/grid/grid_basic.html:5`
  - `tests/layout/browser_parity/xml/grid/grid_basic__border_box_ltr.xml:1`
- Invariant at risk: Checked-in XML goldens should be provably fresh for every helper asset that affects browser measurements.
- Observed code shape: Generator loads and injects `test_base_style.css`, but report metadata hashes only `TEST_HELPER_SOURCE`; linked-resource provenance currently returns an empty list. A sampled generated XML provenance comment includes source/helper/browser metadata but no linked-resource or base-style hash.
- Expected modeling shape: Generated artifact provenance covers every browser-input resource that can affect captured layout data.
- AGENTS/modeling guidance: Generated artifacts should carry enough provenance to explain expected deltas; test/fixture state should stay aligned with real semantics.
- Reproduction or existing coverage: `1274` checked-in HTML fixture files reference `test_base_style.css`; `0` checked-in XML files contain `linked-resource-sha256`; existing corpus and provenance checks pass.
- Notes:

### LAYOUT-MODEL-TRACK-REPEAT-INVALID-STATES: Track Repetition Allows Empty And Zero-Count Layout-Ready States

- Severity: P2
- Category: API-hardening
- Confidence: medium
- Review phase: phase 1
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/value.rs:849`
  - `src/value.rs:851`
  - `src/value.rs:856`
  - `src/value.rs:858`
  - `src/value.rs:859`
  - `src/value.rs:864`
  - `src/value.rs:879`
- Invariant at risk: Layout-ready grid track repeat structures should not represent invalid repeat counts or empty repeated track bodies as ordinary valid values.
- Observed code shape: `TrackRepeat::Count(usize)` accepts zero, and `TrackRepetition` exposes repeat/components as public fields with constructors that accept arbitrary counts and component vectors.
- Expected modeling shape: Authored repeat syntax and validated layout-ready repeated tracks are phase-distinct, with repeat count and component-body invariants represented by construction.
- AGENTS/modeling guidance: Prefer semantic types and constructors for invariants; avoid public field combinations whose validity depends on convention.
- Reproduction or existing coverage: Reviewers found no focused coverage rejecting or reporting zero-count or empty-component repetitions.
- Notes:

### LAYOUT-MODEL-BLOCK-MARGIN-OPTION-STATES: Block Margin Resolution Uses None For Multiple Semantic States

- Severity: P2
- Category: internal-maintainability
- Confidence: medium
- Review phase: phase 3
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/block.rs:448`
  - `src/block.rs:459`
  - `src/block.rs:1030`
  - `src/block.rs:1035`
  - `src/block.rs:1036`
  - `src/block.rs:1040`
  - `src/block.rs:1047`
  - `src/block.rs:1049`
  - `src/value.rs:385`
  - `src/value.rs:388`
  - `src/value.rs:389`
- Invariant at risk: Explicit `auto`, unresolved symbolic values, and absent basis are separate states with different layout meaning.
- Observed code shape: In-flow margin is represented as `Edges<Option<Scalar>>`; `None` participates in horizontal auto-margin distribution and vertical zero fallback, while `LengthAuto::resolve_optional` also returns `None` for `Auto`, `Calc`, and percent without basis.
- Expected modeling shape: Margin resolution state distinguishes explicit auto, unresolved symbolic value, and resolved scalar.
- AGENTS/modeling guidance: Avoid `None` carrying multiple meanings; make invalid or ambiguous states hard to express.
- Reproduction or existing coverage: Existing tests cover auto-margin expansion and one resolved calc margin path, but reviewers found no coverage for unresolved symbolic margin under an indefinite basis.
- Notes:

### LAYOUT-MODEL-FLEX-PHASE-BAG: Flex Item State Mixes Algorithm Phases

- Severity: P2
- Category: internal-maintainability
- Confidence: medium
- Review phase: phase 4
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/flex.rs:190`
  - `src/flex.rs:195`
  - `src/flex.rs:196`
  - `src/flex.rs:198`
  - `src/flex.rs:200`
  - `src/flex.rs:215`
  - `src/flex.rs:783`
  - `src/flex.rs:784`
  - `src/flex.rs:801`
  - `src/flex.rs:2143`
  - `src/flex.rs:2155`
  - `src/flex.rs:2219`
  - `src/flex.rs:2255`
- Invariant at risk: Base sizing, line resolution, cross-size reruns, and final layout outputs should not be readable in the wrong phase.
- Observed code shape: One mutable `FlexItem` carries authored size, initial output, flex basis, hypothetical size, target size, baseline, offsets, and final rerun output; line resolution mutates sizing/offset fields, and final layout overwrites output/baseline later.
- Expected modeling shape: Phase-specific flex item states distinguish pre-rerun measurements, resolved target sizes, aligned offsets, and final layout outputs.
- AGENTS/modeling guidance: Model phases explicitly; avoid algorithm phases represented by mutable bags.
- Reproduction or existing coverage: Existing flex tests cover rerun-sensitive behavior, but stale phase reads remain type-permitted.
- Notes:

### GRID-NAMED-ERRORS-SILENT-FALLBACK: Named Grid Errors Collapse Into Auto Or Empty Context

- Severity: P2
- Category: correctness-risk
- Confidence: medium
- Review phase: phase 6
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/grid/named.rs:91`
  - `src/grid/named.rs:238`
  - `src/grid/named.rs:243`
  - `src/grid/named.rs:245`
  - `src/grid/named.rs:842`
  - `src/grid/mod.rs:737`
  - `src/grid/mod.rs:740`
  - `src/grid/mod.rs:741`
- Invariant at risk: Named grid validation errors should remain visible at the authored-to-normalized boundary instead of becoming ordinary auto/default layout behavior.
- Observed code shape: Semantic `NamedGridError` variants exist, but placement resolution maps any error to `GridPlacement::AUTO`, invalid template area facts are dropped with `.ok()`, and named context build errors fall back to an empty context.
- Expected modeling shape: Invalid named grid state is represented as a semantic validation result or report at the phase boundary.
- AGENTS/modeling guidance: Prefer semantic errors; keep authored/normalized phase boundaries explicit.
- Reproduction or existing coverage: Current tests lock some fallback behavior for invalid raw placement and invalid template areas.
- Notes:

### GRID-LANES-INTRINSIC-ITEM-PHASE-BAG: Public LaneIntrinsicItem Encodes Multiple Kinds In One Field Bag

- Severity: P2
- Category: API-hardening
- Confidence: high
- Review phase: phases 6, 9
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/lib.rs:26`
  - `src/lib.rs:27`
  - `src/lib.rs:28`
  - `src/grid/lanes.rs:108`
  - `src/grid/lanes.rs:111`
  - `src/grid/lanes.rs:112`
  - `src/grid/lanes.rs:114`
  - `src/grid/lanes.rs:127`
  - `src/grid/lanes.rs:128`
  - `src/grid/lanes.rs:150`
  - `src/grid/lanes.rs:160`
  - `src/grid/lanes.rs:319`
  - `src/grid/lanes.rs:323`
  - `src/grid/lanes.rs:336`
  - `api/public-api.txt:484`
  - `api/public-api.txt:486`
  - `api/public-api.txt:488`
  - `api/public-api.txt:489`
  - `tests/support/oracle/grid/lanes.rs:271`
- Invariant at risk: A lane intrinsic item should be exactly one semantic kind: definite, indefinite, or nested indefinite subgrid.
- Observed code shape: `LaneIntrinsicItem` is public and exposes `span`, `definite_span`, and `nested_indefinite_subgrid` simultaneously; the definite constructor writes sentinel `span: 0`; dispatch checks a boolean, then option, then clamps indefinite span with `max(1)`.
- Expected modeling shape: Phase-specific variants or private fields prevent mixed definite/indefinite/nested states and zero-span sentinels.
- AGENTS/modeling guidance: Model phases explicitly; use semantic span/range types; keep public fields from making invalid states easy to express.
- Reproduction or existing coverage: Oracle support duplicates the same field-bag shape; reviewers found no direct test exercising malformed public combinations.
- Notes: Merged from Helmholtz's `GRID-LANES-INTRINSIC-ITEM-PHASE-BAG` and Volta's `API-LANES-PUBLIC-PHASE-BAG`.

### GRID-LANES-ERROR-CONTEXT: LanePlacementError Buckets Lose Validation Context

- Severity: P2
- Category: API-hardening
- Confidence: high
- Review phase: phase 10
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/grid/lanes.rs:41`
  - `src/grid/lanes.rs:44`
  - `src/grid/lanes.rs:244`
  - `src/grid/lanes.rs:245`
  - `src/grid/lanes.rs:248`
  - `src/grid/lanes.rs:249`
  - `src/grid/lanes.rs:308`
  - `src/grid/lanes.rs:313`
  - `src/grid/lanes.rs:323`
  - `src/grid/lanes.rs:326`
  - `api/public-api.txt:168`
  - `api/public-api.txt:171`
- Invariant at risk: Public validation errors should identify the rejected value, violated invariant, and phase/boundary where validation failed.
- Observed code shape: `LanePlacementError` exposes broad buckets, and several distinct failures collapse into `SpanOutOfRange`, including zero start/span, overflow past track count, invalid content-sized indexes, and invalid definite spans.
- Expected modeling shape: Public errors carry typed context for the invalid field/value and specific invariant.
- AGENTS/modeling guidance: Prefer semantic errors that name rejected values, violated invariants, phases, and caller misuse or unsupported input.
- Reproduction or existing coverage: Reviewers found no direct unit coverage asserting contextual error information.
- Notes:

### GRID-LANES-PUBLIC-TRACE-REPORT: Public LanePlacementReport Exposes Internal Packing Trace State

- Severity: P3
- Category: tooling-containment
- Confidence: high
- Review phase: phases 6, 7, 9
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/grid/lanes.rs:31`
  - `src/grid/lanes.rs:36`
  - `src/grid/lanes.rs:38`
  - `api/public-api.txt:524`
  - `tests/support/grid_layout_comparison.rs:160`
  - `tests/support/oracle/grid/lanes.rs:99`
- Invariant at risk: Public lane placement reports should expose stable semantic layout facts, not internal step-by-step packing trace state intended for diagnostics or fixtures.
- Observed code shape: `LanePlacementReport` publicly includes `running_positions_after_each_item` and `final_cursor`, and oracle/comparison support mirrors and asserts whole reports.
- Expected modeling shape: Product-facing reports are separated from fixture/debug traces or use an explicit diagnostic-only contract.
- AGENTS/modeling guidance: Public APIs are product contracts; keep fixture/oracle/debug state contained; expose intentional front doors.
- Reproduction or existing coverage: Grid-lanes tests pass and rely on the current report shape through support/oracle infrastructure.
- Notes:

### LAYOUT-PARITY-STRINGLY-CASE-STATUS: Corpus Case Workflow State Is Duplicated String Matching

- Severity: P3
- Category: tooling-containment
- Confidence: high
- Review phase: phase 8
- Owner: tooling-only
- Status: verified
- Evidence:
  - `tests/bin/surgeist-layout-generate/generator.rs:132`
  - `tests/bin/surgeist-layout-generate/generator.rs:134`
  - `tests/bin/surgeist-layout-generate/generator.rs:136`
  - `tests/bin/surgeist-layout-generate/generator.rs:137`
  - `tests/bin/surgeist-layout-generate/generator.rs:554`
  - `tests/bin/surgeist-layout-generate/generator.rs:1285`
- Invariant at risk: Corpus workflow state should be one closed, validated domain shared by manifest validation and generation/reporting.
- Observed code shape: `CorpusCase` stores `source_root`, `generator`, and `status` as strings; validation and generation each match the status string domain.
- Expected modeling shape: Manifest parsing normalizes workflow distinctions into semantic states reused by validation and generation/reporting.
- AGENTS/modeling guidance: Prefer typed commands, reports, and state over loose runtime behavior; duplicated string validation is a weak boundary.
- Reproduction or existing coverage: Validation coverage exists for nearby manifest fields, but the status domain remains duplicated string matching.
- Notes:

### LAYOUT-PARITY-UNTYPED-TOLERANCE-POLICY: Oracle And Browser Comparisons Use Independent Raw Tolerances

- Severity: P3
- Category: tooling-containment
- Confidence: medium
- Review phase: phase 7
- Owner: tooling-only
- Status: verified
- Evidence:
  - `tests/layout/browser_parity/support.rs:990`
  - `tests/layout/browser_parity/support.rs:991`
  - `tests/support/grid_layout_comparison.rs:747`
  - `tests/support/grid_layout_comparison.rs:749`
- Invariant at risk: Oracle/browser comparisons should expose deliberate tolerance policy rather than independent magic scalar thresholds.
- Observed code shape: Browser parity comparison uses local raw `TOLERANCE: Scalar = 0.1`; grid layout comparison separately uses raw `0.000_1`.
- Expected modeling shape: Comparison tolerance is represented as an intentional test-support contract or report field so failures are comparable and reviewable across oracle and browser parity paths.
- AGENTS/modeling guidance: Use semantic types for units/meaningful invariants; keep test fixture state explicit and aligned.
- Reproduction or existing coverage: No failing coverage observed; verified by code inspection.
- Notes:

### LINT-ALLOW-WITHOUT-REASON: Production Lint Exceptions Use Bare allow Attributes

- Severity: P3
- Category: internal-maintainability
- Confidence: high
- Review phase: phase 10
- Owner: layout crate
- Status: verified
- Evidence:
  - `src/grid/child.rs:116`
  - `src/grid/named.rs:87`
  - `src/grid/named.rs:825`
  - `src/grid/named.rs:832`
  - `src/grid/axis.rs:9`
  - `src/grid/subgrid.rs:51`
  - `src/grid/subgrid.rs:58`
  - `src/grid/subgrid.rs:71`
  - `src/grid/subgrid.rs:413`
  - `src/grid/subgrid.rs:420`
  - `src/grid/subgrid.rs:430`
  - `src/grid/subgrid.rs:442`
  - `src/grid/subgrid.rs:458`
  - `src/grid/subgrid.rs:471`
  - `src/grid/subgrid.rs:491`
  - `src/grid/subgrid.rs:498`
  - `src/grid/subgrid.rs:564`
  - `src/grid/subgrid.rs:625`
  - `src/grid/subgrid.rs:641`
  - `src/grid/subgrid.rs:648`
  - `src/grid/subgrid.rs:765`
  - `src/grid/subgrid.rs:889`
  - `src/grid/lanes.rs:651`
- Invariant at risk: Lint exceptions should be intentional, scoped, and explain why compiler/clippy feedback is being suppressed.
- Observed code shape: Production code contains multiple bare `#[allow(...)]` attributes without reasons.
- Expected modeling shape: Lint exceptions are narrow and documented as intentional exceptions to the product/modeling contract.
- AGENTS/modeling guidance: Do not add broad `#[allow(...)]` attributes to quiet warnings; prefer scoped `#[expect(...)]` with a reason when intentional.
- Reproduction or existing coverage: Verified by search; not test-covered.
- Notes:

## Non-Findings

No rejected findings recorded yet.

## Reconciliation Log

- 2026-06-25: Ran clean-context review lanes for phases 0-2, 3-4, 5-6, 7-8, and 9-10. Merged overlapping calc findings into `LAYOUT-MODEL-CALC-SYMBOLIC-COLLAPSE` and overlapping lane item public API findings into `GRID-LANES-INTRINSIC-ITEM-PHASE-BAG`. Recorded verified findings only after local line-evidence checks.
