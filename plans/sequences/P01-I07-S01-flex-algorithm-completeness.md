# P01-I07-S01 Flex Algorithm Completeness Implementation Sequence

Sequence ID: `P01/I07/S01`

Owning repository: `surgeist-layout`

## 1 Authority

This sequence implements the independently reviewed specification at
`plans/specs/P01-I07-flex-algorithm-completeness.md`, normalized semantic-content
SHA-256
`df69716865bf7f88bf89a7ecfea979cffa3b879b69a2cde16586d7598edb1332`,
committed as `f86b0572863d8eb72da5c00364bf7020299c99b8`.

The specification owns behavior, public API, compatibility, ownership,
artifacts, errors, and acceptance. This sequence owns only durable dependency
order. Only the next ready cycle receives a detailed just-in-time plan.

## 2 Sequence Boundary

Every cycle mutates only `surgeist-layout`. Root computed-style lowering, facade
wiring, API artifacts, gitlink promotion, authored CSS parsing, rendering, and
general visibility behavior remain the separate handoff in `FRI-07.14`.

No cycle adds a dependency, feature, MSRV change, unsafe code, general CSS or
HTML parser, retained identity, rendering state, parallel axis/order/overflow
owner, reusable generator layer, second generator path, or behavior owned by a
later finding-remediation initiative. Generator changes remain limited to the
exact parser, fixture, serialization, and confirmed genuine-bug needs authorized
by `FRI-07.10` and `FRI-07.11`.

`FRI-07.10` owns the finite six-source, 24-row fixture and artifact contract.
Scoped generation may diagnose settled-input defects while the owning cycle is
in progress, but it is not acceptance evidence. All behavior and inputs settle
before the one full unfiltered existing-pinned regeneration in C04. C05 begins
only from that published artifact candidate and cannot rerun generation unless
the sprawl assessment confirms a genuine behavior, input-honesty, or artifact
defect that reopens the owning contract.

Each cycle verifies only its focused behavior and FRI-07-owned parity surface.
Repository-wide `just parity-all` remains the FRI-13 aggregate release gate and
may appear in FRI-07 evidence only as a diagnostic compared with an immutable
entry inventory.

## 3 Ordered Cycles

### 3.1 `P01/I07/S01/C01` Intrinsic Basis And Auto-Margin Corrections

**Specification sources:** `FRI-07.4 D-07` through `D-13`; `FRI-07.6` and
`FRI-07.7`; margin and intrinsic-basis portions of `FRI-07.9`, `FRI-07.11`,
`FRI-07.12`, and `FRI-07.15 FLEX-002` through `FLEX-004`.

**Prerequisites:** Published FRI-06 candidate and the clean reviewed FRI-07
specification revision recorded above.

**Entry state:** Public values and finite fixture parsing preserve min-content
and max-content flex bases, while typed dispatch rejects both direct cells and
collected flex state cannot retain their distinct constraints. Ordinary cross-
axis auto margins divide negative free space instead of anchoring the normal
logical cross-dimension start, and absolute flex-child margin resolution ignores
inset definiteness and the inset-modified containing block.

**Bounded outcome:** Preserve `MinContent` and `MaxContent` as distinct resolved
flex-basis states from dispatch through item measurement and final flex layout,
without changing any other direct or `calc-size()` cell. Independently implement
the complete ordinary cross-axis signed-margin matrix through `FlexAxes`,
including the normal-logical-start, wrap-reverse, and fixed-opposite-margin
rules, and implement the separate absolute inset-modified equation. Preserve
ordinary main-axis behavior and existing sizing, replaced, flow, cache, error,
scroll, source-index, transaction, scalar, and percentage phases.

**Observable exit evidence:** The two direct intrinsic cells produce distinct
provider constraints and geometry in both scalar lanes, with exact dispatcher
accounting for every other cell. Every `FRI-07.6` auto-edge, signed-space, inset,
flow, and fixed-opposite-margin row produces correct used margins and physical
placement. Composition, error, non-finite, cache, sizing, alignment, scroll,
source-index, and transaction controls preserve existing contracts.

**Handoff:** C02 consumes typed intrinsic bases and correct ordinary/positioned
margin owners without revisiting their independent dispatch or equation logic.

### 3.2 `P01/I07/S01/C02` Collapsed Flex-Item Semantics

**Specification sources:** `FRI-07.4 D-01` through `D-06`; `FRI-07.5`;
`FRI-07.8`; collapse portions of `FRI-07.9`, `FRI-07.11`, `FRI-07.12`, and
`FRI-07.15 FLEX-005`.

**Prerequisites:** `P01/I07/S01/C01` complete and remotely verified.

**Entry state:** Layout has no normalized flex-item collapse input, private line
strut, finite two-round orchestration, or collapsed-item publication path.

**Bounded outcome:** Add the two-state public layout-ready collapse model and
implement one finite two-round flex computation. Capture first-round used line
cross sizes after stretch, recollect with zero collapsed-box main size,
non-auto margins, zero auto margins, and collection gaps, assign struts by item
identity after rewrapping, ignore collapsed items after collection, floor each
second-round line by its largest strut, and publish zero collapsed output with
hidden descendants. Do not introduce general visibility or first-round public
state.

**Observable exit evidence:** Single-line, wrapping, all-collapsed,
multi-collapsed, baseline, order, gap, margin, intrinsic, overflow, absolute
descendant, scroll target, failure, cache, transaction, and both scalar-lane
cases prove the exact phase model, no third round, no leaked contribution, and
no behavior outside in-flow flex participation.

**Handoff:** C03 receives complete individual finding behavior through the
public layout front door.

### 3.3 `P01/I07/S01/C03` Flex Composition Closure

**Specification sources:** `FRI-07.4 D-14`; complete `FRI-07.9`; composition
and negative-control portions of `FRI-07.11`, `FRI-07.12`, `FRI-07.15`, and
`FRI-07.16`.

**Prerequisites:** `P01/I07/S01/C02` complete and remotely verified.

**Entry state:** FLEX-002 through FLEX-005 work independently, while their
combined order, flow, replaced sizing, overflow, scrollbar-settling, cache,
transaction, rounding, and scalar interactions have not yet been closed as one
candidate.

**Bounded outcome:** Exercise the four completed capabilities through composed
public-front-door layouts and correct only production defects exposed by those
interactions. Preserve the sole existing owners for order, axes, replaced state,
overflow, cache identity, and transaction phases. Complete crate documentation
for the normalized collapse boundary and intrinsic basis behavior without
claiming root-owned or later-owned capability.

**Observable exit evidence:** The complete `FRI-07.9` matrix passes with stable
source association, geometry, margins, overflow, cache, rollback, rounding,
and f32/f64 results. No duplicate owner, fixture-specific production branch,
new parser layer, dependency, feature, suppression, or unrelated cleanup enters
the candidate.

**Handoff:** C04 receives behavior-complete, documented flex semantics whose
browser inputs and expected geometry can be settled without redesign.

### 3.4 `P01/I07/S01/C04` Bounded Browser And Artifact Candidate

**Specification sources:** `FRI-07.4 D-15` through `D-17`; `FRI-07.10`;
fixture, adapter, generator, artifact, and browser portions of `FRI-07.11`,
`FRI-07.12`, `FRI-07.14`, and `FRI-07.16`.

**Prerequisites:** `P01/I07/S01/C03` complete and remotely verified.

**Entry state:** Production behavior is complete, but the finite fixture adapter
does not accept normalized collapsed participation and the exact six-source,
24-row FRI-07 browser inventory and generated lineage do not exist.

**Bounded outcome:** Add only the exact computed/layout-ready collapse
serialization and finite adapter token, their independence and rejection
controls, and the six specified four-variant browser sources. Settle every HTML,
helper, parser, manifest, provenance-schema, and behavior input. First migrate
the existing `all.json` report in place to the sole global/per-output provenance
authority and prove generated XML is comment-free. Then perform the one full
unfiltered existing-pinned regeneration, including the one-time removal of all
legacy XML provenance comments, and adopt its exact XML/report/corpus lineage.
Apply the known-Chrome-failure exception only when every certainty and
synthetic-substitute predicate is independently proven.

**Observable exit evidence:** Exactly 24 owned rows have honest input-derived
layout facts, visible oracle accounting, and centralized provenance. `all.json`
alone binds every source and generated XML hash; no XML contains embedded
provenance; and every pre-existing XML body after removal of its legacy first
comment is unchanged. The single settled full regeneration is followed by
read-only artifact, parity, corpus, and Taffy verification; every expected fail
has the complete required evidence or the registry remains empty. A second
report/provenance authority, new generator path, and fixture-name/expected-
geometry dispatch remain absent.

**Handoff:** Publish and remotely verify the behavior/artifact candidate with its
exact source inventory, report/helper/artifact hashes, browser provenance, and
known-failure disposition before any final sprawl work begins.

### 3.5 `P01/I07/S01/C05` Validated Sprawl Containment And Final Candidate

**Specification sources:** `FRI-07.4 D-18`; `FRI-07.13`; final architecture,
verification, finding-closure, handoff, and acceptance portions of
`FRI-07.11`, `FRI-07.12`, `FRI-07.14` through `FRI-07.16`.

**Prerequisites:** `P01/I07/S01/C04` complete, published, and remotely verified
with frozen behavior and artifact evidence; a complete sprawl assessment of the
FRI-07 implementation range has returned exact findings for validation.

**Entry state:** FLEX-002 through FLEX-005 and their bounded browser lineage are
closed in the immutable C04 candidate; every actionable FRI-07 sprawl finding
still requires source validation and final disposition.

**Bounded outcome:** Validate each review finding against current source and
implement every confirmed in-initiative mechanical consolidation with
characterization evidence, or disprove it with an exact counterexample. Preserve
the public API, behavior, fixture membership, generated lineage, dependencies,
features, and finding ownership unless the assessment confirms a genuine defect,
which reopens the exact owning behavior contract. Do not absorb crate-wide
advisory lint cleanup or later-initiative work.

**Observable exit evidence:** Every sprawl finding has one validated disposition;
the structural invariants in `FRI-07.13` hold; all four initial findings retain
their public-front-door closure; and behavior, browser lineage, dependencies,
features, and fixture membership remain unchanged unless an owning contract was
explicitly reopened. The final candidate is available from authority remote
`main` with complete sprawl dispositions and without an unneeded regeneration.

**Handoff:** Return the final remotely verified FRI-07 leaf candidate, complete
four-finding closure, validated sprawl dispositions, browser/artifact evidence,
public/root ownership boundary, and later-P01 continuation state.

## 4 Sequence Completion

This sequence ends at `P01/I07/S01/C05`; no later cycle is represented.
