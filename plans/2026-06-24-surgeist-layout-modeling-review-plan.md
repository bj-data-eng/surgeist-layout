# Surgeist Layout Modeling Review Plan

> **For agentic reviewers:** REQUIRED SUB-SKILL: Use superpowers:dispatching-parallel-agents for independent review lanes and superpowers:requesting-code-review when validating each written findings batch. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Review the full `surgeist-layout` crate from bottom to top against `AGENTS.md` modeling guidance before writing any implementation fix plan.

**Architecture:** This is a review plan, not a fix plan. Review proceeds from foundational value semantics up through algorithm phases, public contracts, and fixture tooling so context pressure stays bounded and findings can be reconciled before implementation planning. Findings from this plan should be recorded in a separate findings file, then re-reviewed against the codebase before any code-changing plan is written.

**Tech Stack:** Rust crate source under `src/`, crate-local tests under `tests/layout`, shared oracle support under `tests/support`, API artifact tooling under `api/`, browser parity tooling under `tests/bin/surgeist-layout-generate` and `tests/layout/browser_parity`, `rg`, `cargo test -p surgeist-layout`, `cargo clippy -p surgeist-layout --all-targets -- -D warnings`, `cargo fmt --check`.

---

## Review Rules

- Do not edit production code, tests, generated artifacts, or fixture tooling while running this review plan.
- Treat `AGENTS.md` as the review standard, especially the Type And Value Modeling section.
- Review from the bottom upward: primitive values first, then compute contracts, then algorithms, then public surface and tooling.
- For each finding, record the exact file and line, the invariant at risk, observed code shape, expected modeling shape, severity, and whether it is correctness-risk, API-hardening, internal-maintainability, or tooling-containment.
- Do not turn findings directly into fixes. The next artifact is a findings file; a later artifact is an implementation plan.
- Prefer many focused reviewers over one broad reviewer. Each reviewer should inspect a bounded slice and return findings only.
- Close reviewer agents after their result is integrated.

## Output Artifacts

This plan creates no code changes by itself.

Later review execution should create:

- `plans/2026-06-24-surgeist-layout-modeling-review-findings.md`

Later implementation planning should create a separate implementation plan after the findings file has been reconciled and reviewed.

## Severity Guide

- **P0:** Current correctness bug likely reachable by normal crate use or parity fixtures.
- **P1:** Strong correctness risk, invariant confusion, or public contract that makes invalid states easy to express.
- **P2:** Internal modeling debt that can cause regressions or makes algorithm phases hard to reason about.
- **P3:** Documentation, naming, or containment issue with low immediate correctness risk.

## Cross-Cutting Questions

Each phase should answer these questions:

- What values can be represented that should be impossible?
- Where do `None`, `0.0`, empty vectors, sentinel indices, or booleans carry multiple meanings?
- Where are symbolic values such as percent or calc resolved before the layer has the right basis/context?
- Where does public API expose fixture/debug/oracle details as reusable product contract?
- Where are algorithm phases represented by mutable bags rather than phase-specific types?
- Where do raw `Scalar`, `usize`, `isize`, `String`, or `Vec<T>` fields hide meaningful invariants?
- Which issues are bottom-layer correctness risks versus public API hardening?
- Where do panics, `unreachable!`, defaults, lint exceptions, or error values encode product behavior?
- Is there any `unsafe` code, broad lint allowance, or feature-gated behavior that needs explicit review?

---

## Phase 0: Baseline Inventory

**Files:**
- Read: `AGENTS.md`
- Read: `guidance/surgeist-rust-modeling-guide.md`
- Read: `src/lib.rs`
- Read: `Cargo.toml`
- Read: `README.md`
- Read: `plans/`

- [ ] **Step 1: Confirm repository state**

Run:

```sh
git status --short --branch
```

Expected: clean working tree unless the coordinator explicitly notes unrelated local changes.

- [ ] **Step 2: Map source and test ownership**

Run:

```sh
rg --files src tests/layout tests/support tests/bin api guidance plans | sort
```

Expected: source, tests, oracle support, generator, browser parity support, API tooling, modeling guidance, and plan files are visible for review scoping.

- [ ] **Step 3: Record review boundaries**

Add to the findings file:

```markdown
## Review Boundaries

- Repo: `surgeist-layout`
- Scope: crate-local source, tests, browser parity tooling, generated artifact workflow, and public API reexports from `src/lib.rs`
- Out of scope: code edits, sibling crate fixes, root integration changes, implementation plan
- Standard: `AGENTS.md` and `guidance/surgeist-rust-modeling-guide.md`, especially Type And Value Modeling, Generated Artifacts, Crate Role, Dependency Direction, and Public API Surface
```

---

## Phase 1: Foundational Value Model

**Files:**
- Review: `src/value.rs`
- Review: `src/geometry.rs`
- Review: `src/node_input.rs`
- Review: `src/tests.rs`
- Review: `tests/layout/unit/contract.rs`
- Review: `tests/layout/unit/cache.rs`

- [ ] **Step 1: Review scalar and geometry primitives**

Inspect `Scalar`, `Axis`, `Point<T>`, `Size<T>`, and `Edges<T>`.

Record whether raw `f32` semantics, optional sizes, axis mapping, edge mapping, and arithmetic helpers preserve units and layout phases clearly enough.

- [ ] **Step 2: Review layout value enums**

Inspect `Available`, `Length`, `LengthAuto`, `Dimension`, `MinTrackSizing`, `MaxTrackSizing`, `TrackSizing`, `TrackComponent`, `TrackRepeat`, `SubgridTrack`, `GridTemplateAreas`, and related constructors.

Record invalid representable states such as zero repeat counts, empty vectors, invalid `fr`, invalid percent, or mixed component structures.

- [ ] **Step 3: Review symbolic calc modeling**

Inspect `CalcId`, `CalcTerm`, `CalcExpression`, `CalcResolution`, `CalcResolver`, `NoCalcResolver`, and `LayoutCalcStore`.

Record:

- whether `CalcId` can be fabricated or used with the wrong store,
- whether missing calc expressions fail visibly or degrade silently,
- whether resolver-free APIs can collapse calc to `0.0` or `None`,
- whether calc structure is preserved until a basis is available.

- [ ] **Step 4: Review `NodeInput` as authored/lowered contract**

Inspect all fields in `NodeInput`, especially `aspect_ratio`, `scrollbar_width`, flex factors, grid placement, raw grid placement, template vectors, gap, margin, padding, and border.

Record fields that should become semantic newtypes, private fields with constructors, or phase-specific sub-structs.

- [ ] **Step 5: Run focused value tests for context**

Run:

```sh
cargo test -p surgeist-layout tests:: -- --nocapture
cargo test -p surgeist-layout --test layout layout::contract -- --nocapture
cargo test -p surgeist-layout --test layout layout::cache -- --nocapture
```

Expected: tests pass. If they fail, record the failure as environmental or current-code correctness evidence, but do not fix.

---

## Phase 2: Compute Contract, Cache, And Output Phases

**Files:**
- Review: `src/compute.rs`
- Review: `src/output.rs`
- Review: `src/traits.rs`
- Review: `src/cache.rs`
- Review: `tests/layout/unit/root.rs`
- Review: `tests/layout/unit/leaf.rs`
- Review: `tests/layout/unit/cache.rs`

- [ ] **Step 1: Review `ComputeInput` phase model**

Inspect `RunMode`, `SizingMode`, `RequestedAxis`, and `ComputeInput`.

Record impossible or nonsensical combinations that are representable, and paths that rely on convention such as `ComputeInput::HIDDEN` or `unreachable!`.

- [ ] **Step 2: Review `ComputeOutput` and `NodeOutput` contracts**

Inspect size, content size, baselines, collapsible margins, margin-collapse flags, output order, box edges, and rounded/unrounded separation.

Record any field groups that should be phase-specific types.

- [ ] **Step 3: Review traversal and cache contracts**

Inspect `Traverse`, `Compute`, `Round`, `CacheAccess`, and `compute_cached`.

Record whether cache keys include all layout-relevant inputs, whether calc resolver identity is part of the contract, and whether trait methods allow inconsistent tree state.

- [ ] **Step 4: Run focused compute/cache tests for context**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::root -- --nocapture
cargo test -p surgeist-layout --test layout layout::leaf -- --nocapture
cargo test -p surgeist-layout --test layout layout::cache -- --nocapture
```

Expected: tests pass. Record failures only.

---

## Phase 3: Leaf, Inline, And Block Algorithms

**Files:**
- Review: `src/compute.rs`
- Review: `src/inline.rs`
- Review: `src/block.rs`
- Review: `tests/layout/unit/leaf.rs`
- Review: `tests/layout/unit/block.rs`

- [ ] **Step 1: Review leaf resolution paths**

Inspect `compute_leaf` and helper functions it uses.

Record any resolver-free use of `Length`, `LengthAuto`, `Dimension`, percent, calc, margins, padding, border, size, min, max, and aspect ratio.

- [ ] **Step 2: Review atomic inline model**

Inspect `AtomicInlineInput`, `AtomicInlineItem`, `AtomicInlineOutput`, `AtomicInlineLayoutItem`, and pending line state.

Record phase mixing, baseline semantics, wrapping state, and raw scalar invariants.

- [ ] **Step 3: Review block in-flow layout**

Inspect in-flow child size, margin, margin collapse, relative offsets, text-align offsets, child availability, and calc resolver use.

Record where `auto`, unresolved percent, unresolved calc, and absent values are conflated.

- [ ] **Step 4: Review block absolute layout**

Inspect absolute sizing, insets, static positions, auto margins, final positions, and hidden/display-none paths.

Record sentinel values, ambiguous option fields, and premature resolution.

- [ ] **Step 5: Run focused leaf/block tests for context**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::leaf -- --nocapture
cargo test -p surgeist-layout --test layout layout::block -- --nocapture
```

Expected: tests pass. Record failures only.

---

## Phase 4: Flex Algorithm Phases

**Files:**
- Review: `src/flex.rs`
- Review: `tests/layout/unit/flex.rs`

- [ ] **Step 1: Review flex container constants**

Inspect `Constants` and derived layout constants.

Record whether available size, inner/outer size, max/min size, gap, direction, wrapping, and alignment contexts are distinct enough.

- [ ] **Step 2: Review flex item phase transitions**

Inspect `FlexItem`, collection, flex line collection, base sizing, hypothetical sizing, target sizing, rerun layout, and final output.

Record fields that are valid only in certain phases and risks of stale measurements being read after rerun.

- [ ] **Step 3: Review auto-minimum and calc/percent behavior**

Inspect automatic min main size, percent-dependent calc handling, cross-axis reruns, and stretch sizing.

Record premature resolution or phase confusion.

- [ ] **Step 4: Review flex absolute/hidden children**

Inspect absolute child layout, hidden child layout, order, relative offsets, and scrollbar output.

Record any shared helpers that use ambiguous option/scalar states.

- [ ] **Step 5: Run focused flex tests for context**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::flex -- --nocapture
```

Expected: tests pass. Record failures only.

---

## Phase 5: Grid Placement, Tracks, And Child Layout

**Files:**
- Review: `src/grid/mod.rs`
- Review: `src/grid/axis.rs`
- Review: `src/grid/placement.rs`
- Review: `src/grid/tracks.rs`
- Review: `src/grid/child.rs`
- Review: `src/grid/alignment.rs`
- Review: `src/grid/tests.rs`
- Review: `tests/layout/unit/grid.rs`

- [ ] **Step 1: Review public and internal placement types**

Inspect `GridPlacement`, `RawGridLine`, `RawGridPlacement`, `GridArea`, placement reports, placement normalization, out-of-grid handling, and sentinel areas.

Record invalid representable states, sentinel values, fallback behavior, and raw index risks.

- [ ] **Step 2: Review axis modeling**

Inspect `GridAxisKind`, grid axis mapping inputs/reports, physical axis types, parent/child/local axis conversion, and writing mode interactions.

Record where one enum carries multiple phases that should be distinct.

- [ ] **Step 3: Review track sizing model**

Inspect inline/block track resolution, intrinsic contributions, flex tracks, percent/calc detection, percent fraction logic, fit-content, minmax, repeat, and gap.

Record where symbolic calc or percent structure is collapsed before a basis is available.

- [ ] **Step 4: Review grid child layout**

Inspect item known size, intrinsic contributions, alignment, baseline participation, absolute grid item behavior, content size, and scrollbar handling.

Record value-phase confusion and raw scalar/option risks.

- [ ] **Step 5: Run focused grid tests for context**

Run:

```sh
cargo test -p surgeist-layout --test layout layout::grid -- --nocapture
```

Expected: tests pass. Record failures only.

---

## Phase 6: Named Grid, Subgrid, And Grid Lanes

**Files:**
- Review: `src/grid/named.rs`
- Review: `src/grid/subgrid.rs`
- Review: `src/grid/lanes.rs`
- Review: `src/grid/tests.rs`
- Review: `tests/layout/unit/grid.rs`

- [ ] **Step 1: Review named grid syntax and area facts**

Inspect `NamedGridLines`, `LineNameEntry`, `GridAreaNameFacts`, `GridAreaNameRectangle`, raw names, generated names, clipping, inheritance, and validation errors.

Record raw string/index risks, invalid area states, and whether errors are semantic enough.

- [ ] **Step 2: Review subgrid inheritance and traversal**

Inspect parent context, inherited axes, subgrid eligibility, traversal reports, baseline propagation, gap differences, edge adjustments, and nested subgrid behavior.

Record whether contexts distinguish parent/local/inherited axes and whether invalid subgrid states are representable.

- [ ] **Step 3: Review grid lanes public contract**

Inspect `LanePlacementInput`, `LaneItem`, `LaneItemOffset`, `LanePlacementReport`, `LaneTrackSpan`, `LaneIntrinsicSizingInput`, `LaneIntrinsicItem`, `DefiniteLaneIntrinsicItem`, `IndefiniteLaneContributionGroup`, and `LaneIntrinsicSizingReport`.

Record public fixture/debug leakage, unvalidated spans, `span: 0` conventions, boolean phase flags, and raw `Scalar`/`usize` fields.

- [ ] **Step 4: Run focused named/subgrid/lane tests for context**

Run:

```sh
cargo test -p surgeist-layout grid:: -- --nocapture
cargo test -p surgeist-layout --test layout subgrid -- --nocapture
cargo test -p surgeist-layout --test layout grid_lanes -- --nocapture
```

Expected: command selection may need adjustment if filters do not match. Record exact command and result; do not fix code.

---

## Phase 7: Oracle Support And Comparison Tooling

**Files:**
- Review: `tests/support/mod.rs`
- Review: `tests/support/oracle_tree.rs`
- Review: `tests/support/grid_layout_comparison.rs`
- Review: `tests/support/oracle/mod.rs`
- Review: `tests/support/oracle/inline.rs`
- Review: `tests/support/oracle/grid/mod.rs`
- Review: `tests/support/oracle/grid/alignment.rs`
- Review: `tests/support/oracle/grid/axis.rs`
- Review: `tests/support/oracle/grid/baseline.rs`
- Review: `tests/support/oracle/grid/contributions.rs`
- Review: `tests/support/oracle/grid/lanes.rs`
- Review: `tests/support/oracle/grid/named.rs`
- Review: `tests/support/oracle/grid/placement.rs`
- Review: `tests/support/oracle/grid/scenario.rs`
- Review: `tests/support/oracle/grid/subgrid.rs`
- Review: `tests/support/oracle/grid/tracks.rs`

- [ ] **Step 1: Review oracle tree and comparison contracts**

Inspect oracle tree nodes, layout comparison reports, tolerance handling, and expected/actual data shapes.

Record where oracle-only values may hide layout invariants, where comparison tolerances should be typed, and whether failures produce actionable typed reports.

- [ ] **Step 2: Review grid oracle model**

Inspect oracle grid axis, track, placement, baseline, named grid, subgrid, lane, and scenario modules.

Record divergence between oracle model and production model, duplicated semantics that may drift, and raw `usize`/`Scalar`/`String` usage that should be semantic in test support too.

- [ ] **Step 3: Review inline oracle model**

Inspect inline oracle modeling and how it represents text, atomic inline boxes, baselines, wrapping, and expected measurements.

Record whether oracle abstractions are precise enough to catch modeling regressions without becoming product CSS/text parsing.

- [ ] **Step 4: Run oracle support tests for context**

Run:

```sh
cargo test -p surgeist-layout --test layout oracle -- --nocapture
cargo test -p surgeist-layout --test layout comparison -- --nocapture
```

Expected: command selection may need adjustment if filters do not match. Record exact command and result; do not fix code.

---

## Phase 8: Browser Parity, Fixture Tooling, And Generated Artifacts

**Files:**
- Review: `tests/layout/browser_parity.rs`
- Review: `tests/layout/browser_parity/support.rs`
- Review: `tests/layout/browser_parity/README.md`
- Review: `tests/layout/browser_parity/corpus.toml`
- Review: `tests/bin/surgeist-layout-generate.rs`
- Review: `tests/bin/surgeist-layout-generate/generator.rs`
- Review: `tests/layout/browser_parity/scripts/gentest/test_helper.js`
- Sample: `tests/layout/browser_parity/html/block/**`
- Sample: `tests/layout/browser_parity/html/blockflex/**`
- Sample: `tests/layout/browser_parity/html/blockgrid/**`
- Sample: `tests/layout/browser_parity/html/flex/**`
- Sample: `tests/layout/browser_parity/html/float/**`
- Sample: `tests/layout/browser_parity/html/grid/**`
- Sample: `tests/layout/browser_parity/html/grid-lanes/**`
- Sample: `tests/layout/browser_parity/html/gridflex/**`
- Sample: `tests/layout/browser_parity/html/leaf/**`
- Sample: `tests/layout/browser_parity/html/subgrid/**`
- Sample: `tests/layout/browser_parity/xml/block/**`
- Sample: `tests/layout/browser_parity/xml/blockflex/**`
- Sample: `tests/layout/browser_parity/xml/blockgrid/**`
- Sample: `tests/layout/browser_parity/xml/flex/**`
- Sample: `tests/layout/browser_parity/xml/float/**`
- Sample: `tests/layout/browser_parity/xml/grid/**`
- Sample: `tests/layout/browser_parity/xml/grid-lanes/**`
- Sample: `tests/layout/browser_parity/xml/gridflex/**`
- Sample: `tests/layout/browser_parity/xml/leaf/**`
- Sample: `tests/layout/browser_parity/xml/subgrid/**`
- Review: `tests/layout/browser_parity/xml/generation-reports/**`

- [ ] **Step 1: Review fixture schema modeling**

Inspect XML parsing, style attributes, calc grammar, track list grammar, text fixtures, root contexts, and fixture-local syntax boundaries.

Record where fixture syntax looks like product CSS parsing or where raw strings hide schema states.

- [ ] **Step 2: Review corpus manifest modeling**

Inspect `corpus.toml` fields and generator structs for source roots, generator kinds, statuses, expected failures, quarantine, unsupported cases, and filters.

Record stringly typed workflow distinctions and duplicated runtime validation.

- [ ] **Step 3: Review generated artifact provenance**

Inspect XML provenance comments, report buckets, source/helper/browser hashes, freshness checks, and skipped/unsupported classification.

Record generated-artifact weakening risks and whether provenance is complete enough to support review.

- [ ] **Step 4: Sample checked-in fixture corpus by suite**

Sample at least one HTML/XML pair from each checked-in suite class: block, blockflex, blockgrid, flex, float, grid, grid-lanes, gridflex, leaf, and subgrid, plus generation reports.

Record whether fixture source, generated XML, report buckets, and corpus statuses agree, and whether any suite has different modeling conventions that need separate review.

- [ ] **Step 5: Run fixture-tooling checks for context**

Run:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
cargo test -p surgeist-layout --test layout layout::browser_parity::parses_all_checked_in_browser_parity_xml -- --nocapture
cargo test -p surgeist-layout --test layout layout::browser_parity::all_checked_in_browser_parity_xml_has_generator_provenance -- --nocapture
```

Expected: tests pass. Record failures only.

---

## Phase 9: API Artifact, Public API, And Boundary Review

**Files:**
- Review: `src/lib.rs`
- Review: all `pub` and reexported items from `src/`
- Review: `api/public-api.txt`
- Review: `api/generator/Cargo.toml`
- Review: `api/generator/src/main.rs`
- Review: `README.md`
- Review: `Cargo.toml`

- [ ] **Step 1: Build the public surface map**

Run:

```sh
rg -n '^pub |^pub\\(|pub struct|pub enum|pub trait|pub type|pub use' src
```

Expected: all candidate public contracts are visible.

- [ ] **Step 2: Classify public items**

For each public/reexported type, classify it as:

- algorithm entry point,
- layout value contract,
- traversal/compute contract,
- diagnostic/report contract,
- fixture/oracle support,
- accidental exposure candidate.

- [ ] **Step 3: Review crate boundary and dependency direction**

Inspect dependencies, dev-dependencies, feature-gated generator dependencies, and references to `surgeist-style`/`surgeist-retained` in tests.

Record any dependency sink pressure or sibling-crate boundary drift.

- [ ] **Step 4: Review API artifact provenance**

Inspect `api/public-api.txt` and the API generator.

Record whether the artifact is source-derived, whether it captures the intended public surface, whether it omits feature-gated APIs, and whether generator assumptions could make handwritten API truth drift from source.

- [ ] **Step 5: Compare public surface to bottom-up findings**

Any internal finding that reaches public API should be marked as API-hardening and release-facing. Any public item that exists mainly for tests should be marked as possible containment work.

---

## Phase 10: Cross-Cutting Safety, Defaults, And Error Modeling

**Files:**
- Review: all files under `src/`
- Review: `tests/support/**`
- Review: `tests/layout/browser_parity.rs`
- Review: `tests/layout/browser_parity/support.rs`
- Review: `tests/bin/surgeist-layout-generate.rs`
- Review: `tests/bin/surgeist-layout-generate/generator.rs`
- Review: `api/generator/src/main.rs`

- [ ] **Step 1: Search for unsafe and lint exceptions**

Run:

```sh
rg -n 'unsafe|#\\[allow|#\\[expect' src tests api
```

Expected: every match is reviewed for scope, reason, and whether AGENTS.md requires a tighter contract.

- [ ] **Step 2: Search for panics, unreachable paths, and unchecked assumptions**

Run:

```sh
rg -n 'panic!|unreachable!|expect\\(|unwrap\\(|assert!|assert_eq!' src tests api
```

Expected: every production-code panic or unreachable path is classified as invariant enforcement, bug risk, or test-only expectation.

- [ ] **Step 3: Review defaults as product behavior**

Inspect every `Default` implementation and `DEFAULT` constant in source, support, generator, and API tooling.

Record whether defaults are intentional layout contracts, fixture conveniences, or accidental behavior.

- [ ] **Step 4: Review error modeling**

Inspect error enums, string errors, report buckets, unsupported/quarantine statuses, and generator failures.

Record where errors should carry typed context instead of strings or broad buckets.

---

## Phase 11: Findings Reconciliation

**Files:**
- Create later: `plans/2026-06-24-surgeist-layout-modeling-review-findings.md`
- Review: all reviewer outputs
- Review: source files cited by findings

- [ ] **Step 1: Merge duplicate findings**

Group duplicates by invariant, not by file. For example, `calc collapses to None`, `auto margin uses None`, and `resolver-free calc APIs` may belong to one symbolic-resolution finding with separate examples.

- [ ] **Step 2: Rank findings**

Sort findings by severity:

1. correctness bugs likely reachable today,
2. correctness risks from ambiguous modeling,
3. public API hardening,
4. internal maintainability,
5. tooling containment.

- [ ] **Step 3: Add evidence and confidence**

For each finding, add:

- exact file/line references,
- confidence: high/medium/low,
- why AGENTS.md guidance applies,
- likely owner: layout crate, root coordinator, style/css upstream, or tooling-only,
- whether a focused reproduction test already exists.

- [ ] **Step 4: Request findings review**

Send the findings file to at least two fresh read-only reviewers:

- one reviewer focused on coverage gaps,
- one reviewer focused on severity and false positives.

Expected: reviewers either return clean or identify missing/overstated findings to reconcile.

---

## Phase 12: Completion Criteria

The review plan is complete only when:

- every phase above has been run or explicitly deferred with a reason,
- findings are recorded in the separate findings file,
- all findings have exact file/line evidence,
- at least two fresh reviewers have reviewed the findings file,
- the final findings file distinguishes correctness, API hardening, internal maintainability, and tooling containment,
- no implementation fix plan has been started prematurely.

After completion, create a separate implementation plan for accepted fixes, with worker/reviewer cycles and logical commits.
