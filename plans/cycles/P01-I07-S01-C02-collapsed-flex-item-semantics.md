# P01-I07-S01-C02 Collapsed Flex-Item Semantics

Status: complete

Cycle ID: `P01/I07/S01/C02`

Owning repository: `surgeist-layout`

Cycle base: `95a4248253b266257382e924e7339cf3fb0dbcc1`

Reviewed specification:
`plans/specs/P01-I07-flex-algorithm-completeness.md`, normalized semantic-content
SHA-256 `9e2a899476d27e09133a05531cb4bb4dfab1479d949d66167548c26ee1972b57`,
commit `451954e6aab6529ce7464c299be7e2aff6ea3753`: `FRI-07.4 D-01` through
`D-06`, `FRI-07.5`, `FRI-07.8`, and the collapse portions of `FRI-07.9`,
`.11`, `.12`, and `.15 FLEX-005`.

Reviewed implementation sequence:
`plans/sequences/P01-I07-S01-flex-algorithm-completeness.md`, normalized
SHA-256 `50f29d416df158ed8ceb799d4592fc2d53e36e6512a676bf498b05722a15b964`,
commit `0f0c19e50d8f6c27e300a5bc652e21ee5145b7cc`, entry
`P01/I07/S01/C02`.

Bounded outcome: close `FLEX-005` through one public two-state layout-ready
collapse input and one private finite two-round flex computation that captures
used line cross-size struts, redoes line collection, suppresses collapsed-item
participation after collection, and publishes source-associated zero output
with hidden descendants and no leaked contribution.

## 1 Boundary

The remotely verified C01 candidate at the cycle base is the immutable entry
state. `NodeInputOf` has independent item-order and replacedness facts but no
collapsed flex-item input. `compute_flex_inner` currently collects one stable
order-modified item sequence, collects lines once, resolves and stretches those
lines, performs final layout once, and uses the resulting items plus absolute
children as the sole flex contribution source. `layout_hidden_children` already
owns zero publication and hidden recursive computation for `display: none`, but
collapsed in-flow items need the same descendant behavior without becoming
display-none before their first-round strut is known.

The collapse state is meaningful only for an in-flow child of a flex container.
Absolute and display-none children, flex roots, and children of block, grid,
grid-lanes, subgrid, leaf, or positioned formatting contexts retain existing
behavior. A collapsed item remains in the existing `(ItemOrder, SourceIndex)`
sequence in both rounds. The first round treats it normally and captures its
settled line cross size after ordinary cross-size calculation and align-content
stretch. The second round starts from immutable collected measurements and
identity-keyed struts; it never feeds a third round.

Second-round line collection treats the collapsed box main size as zero, keeps
its resolved non-auto main-axis margins, holds its auto main-axis margins at
zero, and keeps normal collection gap positions. After collection the collapsed
item is excluded from flexible sizing, committed gaps, alignment, baselines,
intrinsic and content contribution, scroll geometry and targets, and absolute-
descendant geometry. Each second-round line is floored by the largest strut of
the collapsed identities assigned to it after rewrapping. Empty and all-
collapsed containers retain the existing required line with no summed strut.

Out of scope: authored or computed CSS visibility; general visibility or box
generation; table collapse; fixture attributes, HTML, XML, corpus manifests,
reports, parser, helper, serializer, browser pin, or generator changes;
generation or artifact-writing generator modes; README or root adapters; C03
documentation and four-capability composition; C04 fixtures and regeneration;
C05 sprawl review; dependencies, features, MSRV, manifests, lockfiles, API
artifacts, new modules, reusable parser or generator layers, parallel order,
axis, overflow, cache, or transaction owners, lint suppressions, unrelated
cleanup, and Surgeist-owned `unsafe`.

## 2 Impacts

Public API: source-breaking before release as specified. Add public
`FlexItemCollapse::{Normal, Collapsed}`, reexport it at crate root, and add
`NodeInputOf::flex_item_collapse`. Default, `NodeInput::DEFAULT`, and
`NodeInputOf::non_box()` use `Normal`. No public output, error, request, cache,
trait, scalar, alias, or conversion changes.

Dependencies, features, manifests, lockfiles, and MSRV: unchanged.

Generated artifacts: unchanged. C02 changes no fixture or generator input and
runs no generation or artifact-writing mode. `just verify-generator`,
`just corpus-check`, and `just taffy-check` are read-only verification.

Docs/examples: unchanged in C02; C03 owns complete crate documentation after
individual flex findings compose. Root follow-up is deferred to the FRI-07 leaf
handoff and must lower computed visibility explicitly. Owned Rust remains free
of `unsafe`.

## 3 Tasks

### 3.1 `P01/I07/S01/C02/T01` Add The Layout-Ready Collapse Model

**Files/area:** `src/node_input.rs`, `src/lib.rs`, `src/lib_tests.rs`, and their
focused tests.

**Outcome:** add the exact two-state public enum and `NodeInputOf` field at the
normalized layout-input phase. Update the const default, generic default, and
non-box construction paths to `Normal`; reexport the type from the crate root.
Do not add CSS conversions, a boolean alias, another variant, or algorithm
behavior.

**RED evidence:** first add `fri07_c02_model_` compile/runtime tests that import
the missing crate-root type and field and exercise all three construction paths.
At the exact task base they must fail because the public model does not exist.
Record that missing-surface failure before implementation. After adding the
model and before changing `src/flex.rs`, add passing characterization that sets
`Collapsed` and compares it with `Normal` through the public layout front door
outside in-flow flex participation; do not fabricate a behavioral RED for this
inert-state control.

**Acceptance:** the enum derives exactly the specified traits and defaults to
`Normal`; the field is public and scalar-independent; all default/non-box paths
are `Normal`; exhaustive matching exposes only two states; ordinary default
layout retains every observable output field in f32 and f64. Collapsed versus
normal output is field-equal for a root and for children of block, grid,
grid-lanes, subgrid, leaf, and positioned contexts, plus absolute and
display-none children of flex containers; no other public surface or formatting-
context behavior changes.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_model_
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Dependency:** reviewed planning packet only. **Intended commit:**
`feat(layout): add collapsed flex-item input`.

### 3.2 `P01/I07/S01/C02/T02` Implement Finite Strut Reflow

**Files/area:** `src/flex.rs` and `src/flex_tests.rs`.

**Outcome:** carry `FlexItemCollapse` in the existing collected item sequence and
add one private identity-keyed strut state. When at least one in-flow flex item
is collapsed, run one normal first round through line cross-size settlement and
align-content stretch, capture each collapsed identity's used line cross size,
then run exactly one second round from immutable collected measurements. Redo
line collection with zero collapsed box main size, non-auto main margins, zero
auto main margins, and collection gaps; assign struts by item identity after
rewrapping; ignore collapsed items after collection; suppress their committed
gaps; and floor each line by its largest strut before item cross alignment.

**RED evidence:** first add `fri07_c02_collapse_round_` public-front-door tests
for single-line stability, wrapping changed only by zero main size, retained
fixed versus zero auto margins, collection versus committed gaps, rewrapped
identity assignment, largest-not-summed multiple struts, all-collapsed lines,
baseline/stretch capture, and an observable measurement ledger proving no third
round. At the exact task base, `Collapsed` behaves as a normal item and those
geometry and call-count assertions fail for the specified cause.

**Acceptance:** first-round struts use settled used line cross sizes rather than
item or container size; align-content stretch is included; row/column,
reverse/wrap-reverse, all ten flow mappings, zero/finite main sizes, min/max,
baseline, intrinsic items, replacedness, and both scalar lanes map through the
existing `FlexAxes`; no collapsed item participates after second collection;
normal items retain existing flexible sizing and auto-margin phases; no first-
round geometry is published; no third round or second collection owner exists.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_tests::
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Dependency:** T01. **Intended commit:**
`feat(layout): implement collapsed flex struts`.

### 3.3 `P01/I07/S01/C02/T03` Suppress Collapsed Output And Contributions

**Files/area:** `src/flex.rs` and `src/flex_tests.rs`.

**Outcome:** publish every collapsed in-flow child as a source-indexed zero
`NodeOutputOf`, schedule its descendants through the existing hidden-computation
input, and exclude it from `FlexChildContribution`, container content and scroll
geometry, scroll-target inventory, baselines, intrinsic contribution, and
absolute-descendant containing geometry. Preserve display-none and absolute
child owners and transaction atomicity.

**RED evidence:** first add `fri07_c02_collapsed_output_` public-front-door
regressions whose collapsed children would otherwise carry margins, baseline,
overflow, scroll targets, nested scroll geometry, measured content, or absolute
descendants. Assert exact zero output/source association, hidden descendants,
unchanged container contributions, and no partial batch on first- or second-
round failure. Keep each failing regression until the smallest suppression path
is implemented.

**Acceptance:** collapsed output has zero location, size, content, edges,
baselines, and scroll geometry with its raw source index; descendants are hidden
without erasing the private line strut; collapsed normal and absolute descendants
cannot affect container content/scroll ranges or targets; display-none and
absolute siblings remain exact; cache cold/warm output and failed-measurement
state are atomic in f32 and f64; no public strut or duplicate hidden algorithm is
introduced.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapsed_output_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Dependency:** T02. **Intended commit:**
`fix(layout): hide collapsed flex output`.

### 3.4 `P01/I07/S01/C02/T04` Close Collapse Composition Evidence

**Files/area:** `src/flex_tests.rs` and only C02 production files when a focused
composition test exposes a genuine defect.

**Outcome:** complete the C02 interaction and bounded property matrix through
real public-front-door layouts. Compose collapse with item order and source
association, every axis/reversal, intrinsic min/max bases, replaced and
non-replaced sizing, overflow and scrollbar settling, absolute/display-none
siblings, cache cold/warm equivalence, first/second-round provider failures,
rounding, and f32/f64 agreement without adding another owner.

**RED evidence:** write `fri07_c02_composition_` tests before any task-local
production correction. If T01 through T03 already compose correctly, record
passing characterization and do not fabricate RED. Any observed defect remains
a failing public-front-door regression until its smallest in-scope correction.

**Acceptance:** deterministic examples plus bounded finite property cases cover
normal/collapsed state, order, flow, wrap, intrinsic basis, auto-margin pattern,
replacedness, and overflow pair; outputs remain finite with non-negative box
sizes and stable source identity; no collapsed scroll contribution or partial
publication occurs; at most two collapse rounds execute; scalar results agree
within existing tolerance; all individual focused groups remain green; no API,
fixture, parser, generator, documentation, suppression, or unrelated change.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_composition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Dependency:** T01 through T03. **Intended commit:**
`test(layout): close FRI-07 C02 composition`.

## 4 Completion

The canonical implementation, task-review, status, holistic-review, landing,
publication, readback, and cleanup lifecycle applies. C02 product acceptance is:

1. `FLEX-005` satisfies its exact `FRI-07.15` row through the public front door;
2. `FlexItemCollapse` has exactly two states and affects only in-flow flex items;
3. collapse runs at most two finite rounds, redoes wrapping from immutable
   measurements, and floors lines with identity-associated used cross struts;
4. collapsed items publish zero source-associated output with hidden descendants
   and no baseline, intrinsic, content, scroll, target, or absolute contribution;
5. all order, axis, sizing, replaced, overflow, cache, transaction, rounding,
   and scalar controls pass without another owner;
6. dependencies, features, MSRV, manifests, lockfiles, README, root, fixtures,
   corpus, generated artifacts, helper, parser, reports, browser pin, and
   generator files remain unchanged, and no generation or artifact-writing mode
   runs; and
7. the remotely available C02 handoff gives C03 the exact public collapse model,
   task ranges, finding evidence, checks, and unchanged-artifact proof.

At the complete-status head, run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_model_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapsed_output_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_composition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --exit-code 95a4248253b266257382e924e7339cf3fb0dbcc1..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' src tests
git diff --check 95a4248253b266257382e924e7339cf3fb0dbcc1..HEAD
git status --short
```

The protected-path diff and unsafe scan print no output; status is clean.
Generator-feature, corpus, and Taffy commands verify checked-in artifacts without
regeneration. Repository-wide `just parity-all` remains the FRI-13 aggregate
gate and is not C02 acceptance. No genuine blocker is currently known.
