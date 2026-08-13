# P01-I09-S01 Cross-Format Alignment Semantics Implementation Sequence

Sequence ID: `P01/I09/S01`

Owning repository: `surgeist-layout`

## 1 Authority

This sequence implements the independently reviewed specification at
`plans/specs/P01-I09-cross-format-alignment-semantics.md`, normalized
semantic-content SHA-256
`9a910e88833c032fa6d6ed859d6dee127e9d7acc60947f944e250a8984867ad4`,
committed as `a71091180af89db80343d40e56bf9dc019cf6384`.

The specification owns behavior, model, public API, module ownership, finite
artifact state, compatibility, exclusions, and acceptance. This sequence owns
only dependency order and cycle boundaries. Only the next ready cycle receives
a detailed just-in-time plan.

## 2 Sequence Boundary

Every cycle is owned by `surgeist-layout`. Root authored CSS and font/shaping
resolution, facade lowering, generated API artifacts, integration tests, and
gitlink promotion remain a separate handoff after the final leaf candidate.

No cycle adds a dependency, feature, MSRV change, unsafe code, suppression,
authored CSS parser, shaper, renderer, retained identity, parallel flow/cache/
baseline-group owner, new generator, generator-architecture expansion, or
FRI-10 through FRI-13 behavior. Generator changes remain limited to parser
updates, the exact specified fixtures, and confirmed genuine defects.

Each cycle enters only when its immutable predecessor candidate is available.
Artifact-writing browser execution remains permission-gated at the applicable
cycle.

## 3 Durable Order

```text
C01 public model and shared policy
  -> C02 text alignment and justification
  -> C03 resolved vertical alignment
  -> C04 block-container content alignment
  -> C05 flex and grid baseline coordination
  -> C06 closed browser adapter and canonical artifacts
  -> C07 finite whole-crate structural containment
```

The order follows `FRI-09.14`. If the frozen accepted structural findings in
C07 cannot fit within one cycle of at most eight tasks, this sequence receives a
reviewed revision adding bounded continuation cycles before implementation.

## 4 `P01/I09/S01/C01` Public Model And Shared Policy

**Owner:** `surgeist-layout`.

**Specification:** `FRI-09.5.1`, `FRI-09.5.2`, `FRI-09.5.3`, `FRI-09.9.1`,
`FRI-09.10`, `FRI-09.12.1`, `FRI-09.13`, `FRI-09.14(1)`, and applicable
acceptance rows in `FRI-09.15`.

**Entry:** this specification and sequence are independently clean; the cycle
base contains only the published FRI-08 candidate and FRI-09 planning commits.

**Outcome:** the property-valid public alignment vocabulary, private shared
policy, and cache-visible adjustment contract exist without changing legacy-
representable geometry.

**Exit evidence:** public construction and compile-fail contracts, scalar lanes,
cache identity, legacy-equivalent behavior, documentation, and unchanged
artifact state satisfy the cited specification sections.

**Handoff:** publish the immutable public-model candidate to C02.

## 5 `P01/I09/S01/C02` Text Alignment And Justification

**Owner:** `surgeist-layout`.

**Specification:** `FRI-09.5.2`, `FRI-09.6.1` through `FRI-09.6.4`,
`FRI-09.10`, the inline/output rows of `FRI-09.12.1`, the inline alignment and
justification anchors in `FRI-09.12.2`, `FRI-09.14(2)`, and applicable
acceptance rows in `FRI-09.15`.

**Entry:** the C01 public model and cache contract are published.

**Outcome:** logical all-line and last-line alignment plus explicit shaping-
owned justification produce deterministic committed geometry.

**Exit evidence:** line selection, opportunity eligibility, distribution,
writing-mode, scalar, bidi, overflow, transaction, and output contracts satisfy
the cited specification sections.

**Handoff:** publish the immutable line/output carrier to C03.

## 6 `P01/I09/S01/C03` Resolved Vertical Alignment

**Owner:** `surgeist-layout`.

**Specification:** `FRI-09.5.3`, `FRI-09.7.1` through `FRI-09.7.3`, the shaped,
atomic, control, inline, output, and cache rows of `FRI-09.12.1`, the vertical-
alignment anchors in `FRI-09.12.2`, `FRI-09.14(3)`, and applicable acceptance
rows in `FRI-09.15`.

**Entry:** C02 has published stable line selection, fragment advance, and output
carriers.

**Outcome:** every resolved vertical-alignment state participates in one
baseline-relative group and one monotone line-relative envelope.

**Exit evidence:** scalar construction, metric grouping, line-edge placement,
writing-mode projection, baselines, overflow, cache replay, and committed output
satisfy the cited specification sections.

**Handoff:** publish authoritative inline subject envelopes to C04.

## 7 `P01/I09/S01/C04` Block-Container Content Alignment

**Owner:** `surgeist-layout`.

**Specification:** the block-applicable model in `FRI-09.5.1`, `FRI-09.8.1`
through `FRI-09.8.3`, `FRI-09.10`, the block/cache/output rows of
`FRI-09.12.1`, the block anchors in `FRI-09.12.2`, `FRI-09.14(4)`, and
applicable acceptance rows in `FRI-09.15`.

**Entry:** C03 has published authoritative inline, float, baseline, and scroll
envelopes.

**Outcome:** represented ordinary block content alignment establishes its
specified formatting-context boundary and translates one coherent in-flow
subject.

**Exit evidence:** definite and indefinite sizing, floats, inline fragments,
margins, writing modes, safe overflow, scroll geometry, and transaction behavior
satisfy the cited specification sections.

**Handoff:** publish stable child-internal content alignment to C05.

## 8 `P01/I09/S01/C05` Flex And Grid Baseline Coordination

**Owner:** `surgeist-layout`.

**Specification:** the baseline-applicable model in `FRI-09.5.1`, `FRI-09.9.1`
through `FRI-09.9.3`, `FRI-09.10`, the flex/grid/subgrid/cache/output rows of
`FRI-09.12.1`, the baseline-coordination anchors in `FRI-09.12.2`,
`FRI-09.14(5)`, and applicable acceptance rows in `FRI-09.15`.

**Entry:** C04 has published stable content subjects and the C01 cache-visible
adjustment contract remains immutable.

**Outcome:** eligible flex, grid, subgrid, and supported grid-lanes items consume
one-way first/last baseline content adjustments without a second owner or fixed
point.

**Exit evidence:** eligibility, grouping, fallbacks, intrinsic and definite
sizing, scalar/writing-mode projection, cache equivalence, and committed subtree
geometry satisfy the cited specification sections.

**Handoff:** publish the complete production-behavior candidate to C06.

## 9 `P01/I09/S01/C06` Closed Browser Adapter And Canonical Artifacts

**Owner:** `surgeist-layout`.

**Specification:** `FRI-09.11.1` through `FRI-09.11.3`, the browser/parser/
generator rows of `FRI-09.12.1`, the artifact anchors in `FRI-09.12.2`,
`FRI-09.13`, `FRI-09.14(6)`, and artifact acceptance rows in `FRI-09.15`.

**Entry:** C05 has published all production behavior; explicit permission is
required before the first artifact-writing browser execution.

**Outcome:** the closed fixture schema, exact finite sources and outputs, legacy
XML compatibility, corpus registration, and report-only provenance form the
specified canonical artifact state.

**Exit evidence:** parser rejection, identity independence, exact inventories,
hash lineage, comment-free XML, report/filesystem equality, and browser-free
reproducibility satisfy the cited specification sections.

**Handoff:** publish the immutable production-plus-artifact candidate to C07.

## 10 `P01/I09/S01/C07` Finite Whole-Crate Structural Containment

**Owner:** `surgeist-layout`.

**Specification:** `FRI-09.14.1` and `FRI-09.15(11)` through `FRI-09.15(14)`.

**Entry:** C06 is published and the complete post-artifact crate is available
for one frozen read-only whole-crate assessment.

**Outcome:** every actionable row in that finite assessment has an accepted or
adjudicated disposition, and every accepted row is contained without widening
generator ownership.

**Exit evidence:** the frozen report, complete dispositions, independently
reviewed accepted remediations, full crate and feature verification, public and
artifact inventories, suppression/unsafe checks, and clean process/worktree
state satisfy the cited specification sections.

**Handoff:** publish and read back the immutable FRI-09 leaf candidate for the
separate root integration owner.

## 11 Sequence Completion

The sequence is complete only when C01 through C07, plus any reviewed bounded
C07 continuation cycles, satisfy their exits in order. FRI-09 then closes
exactly `MODEL-006`. This planning publication does not begin implementation.
