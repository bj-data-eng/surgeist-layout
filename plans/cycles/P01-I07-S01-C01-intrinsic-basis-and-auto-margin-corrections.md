# P01-I07-S01-C01 Intrinsic Basis And Auto-Margin Corrections

Status: reviewed

Cycle ID: `P01/I07/S01/C01`

Owning repository: `surgeist-layout`

Cycle base: `d386c7d796e5fe0c0856c15ac800516df1348f3b`

Reviewed specification:
`plans/specs/P01-I07-flex-algorithm-completeness.md`, normalized semantic-content
SHA-256 `6f5d480970116600bdf0cccfb7b684893e3c47f1b9cb0ba84113ef55bd21a3a5`,
commit `8afd4fef2a70cb93c08b3f1c5009f6ec104db3fc`: `FRI-07.4 D-07` through
`D-13`, `FRI-07.6`, `FRI-07.7`, and the margin, intrinsic-basis, verification,
module, and finding-closure portions of `FRI-07.9`, `.11`, `.12`, and `.15`.

Reviewed implementation sequence:
`plans/sequences/P01-I07-S01-flex-algorithm-completeness.md`, normalized
SHA-256 `e887915ef9d3081c64627fa75719382b459834597ef9f9ce2846c34fb75fd8c2`,
commit `dd6dfb58db7560a36adcb1948e39071f9e116e87`, entry
`P01/I07/S01/C01`.

Bounded outcome: close `FLEX-002`, `FLEX-003`, and `FLEX-004` through typed
intrinsic flex-basis measurement, the complete ordinary cross-axis auto-margin
matrix, and the inset-modified absolute flex-child margin equation while
preserving every adjacent capability and existing owner.

## 1 Boundary

The published FRI-06 candidate at the cycle base is the immutable behavioral
entry state. Current `FlexBasisOf` values and the finite fixture adapter already
preserve direct `MinContent` and `MaxContent`, but `dispatch_flex_basis` returns
typed unsupported capability for both and `ResolvedFlexBasis` cannot retain
their distinct measurement constraints. Existing supported ordinary numeric,
`calc-size(Any)`, and `calc-size(FullPercentage)` cells and all later-owned direct
and keyword-basis cells must remain exact.

Current ordinary cross-axis auto-margin resolution uses wrap-aware flex cross
start/end for every branch and divides negative free space between two auto
margins. The corrected overflow branch instead uses the normal logical start of
the cross dimension, unaffected by `wrap-reverse`. If that start margin is auto,
it becomes zero and the opposite used margin, even when non-auto, is replaced by
the value that makes the outer cross size equal the line. Flex line progression
and final placement remain wrap-aware after used margins resolve.

Current absolute flex-child margin resolution derives free space from the whole
container inner size, ignores resolved insets, and clamps negative distribution
to zero. The correction operates per physical axis on the inset-modified
containing padding box. Any auto inset makes auto margins in that axis zero;
otherwise signed remaining space is distributed, with the containing flow's
inline-start exception when both inline margins are auto and remaining space is
negative.

The three implementation fronts are semantically independent and share one
cycle only to avoid an artificial publication dependency. They may share
`src/flex.rs` but must not share an approximation: flex-basis dispatch,
ordinary cross-axis margins, and positioned margins remain separate phase
owners.

Out of scope: `FLEX-005` and collapse state; general visibility; general
positioned layout; new sizing values; public API additions; order, replaced,
overflow, cache, transaction, or axis-owner replacement; dependencies, features,
MSRV, manifests, lockfiles, docs, examples, root changes, and unrelated cleanup;
HTML, fixture, parser, helper, manifest, XML, report, corpus, browser pin, or
generator changes; generator architecture; and generator execution. No new lint
allowance or suppression and no Surgeist-owned `unsafe` are permitted.

## 2 Impacts

Public API: no signature, field, variant, reexport, or compatibility change.
Existing public `FlexBasisOf::MIN_CONTENT` and `MAX_CONTENT` values become
supported by flex layout as already modeled.

Dependencies, features, manifests, lockfiles, and MSRV: unchanged.

Generated artifacts: unchanged. This cycle performs no scoped or full
generation because it changes no HTML, fixture parser, fixture input, helper,
manifest, or generator code. `just parity-all`, `just corpus-check`, and
`just taffy-check` are read-only verification.

Docs/examples: unchanged except this canonical cycle plan. Root follow-up:
none beyond the published leaf candidate handed to C02. Owned Rust remains free
of `unsafe`.

## 3 Tasks

### 3.1 `P01/I07/S01/C01/T01` Preserve Intrinsic Flex-Basis Semantics

**Files/area:** `src/compute.rs`, `src/sizing.rs`, `src/flex.rs`, focused sizing
tests, and `src/flex_tests.rs`.

**Outcome:** add distinct private `ResolvedFlexBasis::MinContent` and
`MaxContent` states; support exactly the two direct dispatcher cells; carry the
selection through item collection; and measure the flex item's main size under
the matching `AvailableOf` constraint without consulting preferred main size or
falling through the generic `Content` max-content path. Preserve existing box-
sizing, padding/border, hypothetical clamping, aspect ratio, replaced sizing,
orthogonal flow, failure, percentage, cache, and scalar phases.

**RED evidence:** first add `fri07_c01_intrinsic_` dispatcher and public-front-
door tests. At the exact task base, direct min-content and max-content requests
must return their current typed unsupported capabilities; the layout regression
with provider results `20` and `100` must fail to produce distinct flex bases.
Record the exact failure before implementation.

**Acceptance:** direct min/max-content bases produce distinct provider
constraints and resulting geometry for leaf and child-container items in row,
column, and orthogonal flows in both scalar lanes. Provider failure and non-
finite output retain exact errors. Exhaustive dispatcher accounting proves
ordinary numeric and supported `calc-size()` cells are unchanged and every
later-owned direct or keyword-basis cell retains its exact payload.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_intrinsic_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout sizing::tests::
cargo fmt --check
```

**Dependency:** reviewed planning packet only. **Intended commit:**
`fix(layout): preserve intrinsic flex-basis semantics`.

### 3.2 `P01/I07/S01/C01/T02` Correct Cross-Axis Auto Margins

**Files/area:** `src/flex.rs` and `src/flex_tests.rs`.

**Outcome:** extend the sole `FlexAxes` owner with the non-wrap-reversible normal
logical cross-dimension start/end mapping needed by Flexbox 9.6. Preserve positive
distribution. For zero or negative remaining space, set an auto logical-start
margin to zero and set the opposite used margin to the exact value needed for
outer cross size to equal line cross size. Keep line progression and physical
placement on existing wrap-aware sides.

**RED evidence:** first add `fri07_c01_cross_auto_margin_` public-front-door
tests. Reconstruct the original `100x40` row container with a `20x60` item and
both cross margins auto; record the current `-10/-10` used margins and centered
overflow instead of the required `0/-20`. Also record the current failure of a
`60px` target in a `40px` line with auto logical-start and fixed `5px` logical-
end, whose required used result is `0/-20` rather than preserving `5` or using
the precomputed `-25` remainder.

**Acceptance:** positive, zero, and negative remaining space; all four auto-edge
patterns; fixed opposite margin replacement; row/column and main reversal;
paired wrap/wrap-reverse; all ten writing-mode/direction mappings; output used
margins; physical geometry; and both scalar lanes satisfy `FRI-07.6.1`.
Ordinary main-axis auto margins and non-auto cross alignment remain unchanged.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_cross_auto_margin_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_tests::
cargo fmt --check
```

**Dependency:** reviewed planning packet only; no T01 semantic dependency.
**Intended commit:** `fix(layout): correct cross-axis auto margins`.

### 3.3 `P01/I07/S01/C01/T03` Resolve Absolute Margins From Insets

**Files/area:** `src/flex.rs` and `src/flex_tests.rs`.

**Outcome:** give the private absolute margin resolver the resolved insets and
the same containing padding-box size used for absolute sizing. Resolve each axis
from inset definiteness and signed inset-modified remaining space. Use the
containing `FlowAxes` for the negative two-auto-inline-margin start exception;
do not import general positioned-layout behavior or reuse the ordinary flex
cross-margin equation.

**RED evidence:** first replace the erroneous legacy expectation with a
`fri07_c01_absolute_auto_margin_` regression that preserves the original
`100x40` container, `20px` child, definite left `0`, auto right, and two auto
horizontal margins. Record current x `40` and margins `40/40` versus required x
`0` and margins `0/0`. Add failing inset-modified positive and signed negative
cases before implementation.

**Acceptance:** either-auto-inset zeroing; both-definite insets with zero, one,
or two auto margins; positive and negative remaining space; negative inline-
start handling under LTR, RTL, and vertical/sideways containing flow; negative
block-axis division; padding, border, and box sizing; existing absolute sizing,
alignment, scroll contribution, source association, transaction behavior; and
both scalar lanes satisfy `FRI-07.6.2`.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_absolute_auto_margin_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_tests::
cargo fmt --check
```

**Dependency:** reviewed planning packet only; no T01 or T02 semantic dependency.
**Intended commit:** `fix(layout): resolve absolute flex margins from insets`.

### 3.4 `P01/I07/S01/C01/T04` Close C01 Composition Evidence

**Files/area:** `src/flex_tests.rs` and only C01 production files when a focused
composition test exposes a genuine defect.

**Outcome:** exercise intrinsic-basis items with corrected cross margins and
absolute siblings through real public-front-door layouts. Cover order-modified
source association, replaced and non-replaced sizing, overflow and scrollbar
settling, cache cold/warm equivalence, failed-measurement atomicity, rounding,
and f32/f64 agreement without adding a second axis, sizing, or error owner.

**RED evidence:** write `fri07_c01_composition_` tests before any task-local
production correction. If T01 through T03 already compose correctly, record the
passing tests as characterization and do not fabricate RED. Any observed defect
must remain a failing public-front-door regression until its smallest in-scope
correction is implemented.

**Acceptance:** the complete C01 cross-capability matrix passes; the three
individual focused groups remain green; no capability payload, geometry, cache,
transaction, or error assertion is weakened; and the task introduces no public
surface, fixture-specific branch, parser, generator change, or suppression.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_composition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Dependency:** T01 through T03. **Intended commit:**
`test(layout): close FRI-07 C01 composition`.

## 4 Completion

The canonical implementation, task-review, status, holistic-review, landing,
publication, readback, and cleanup lifecycle applies. C01 product acceptance is:

1. `FLEX-002`, `FLEX-003`, and `FLEX-004` each satisfy their exact
   `FRI-07.15` closure row through the public front door;
2. only direct min/max-content flex-basis capability cells change;
3. every ordinary and absolute margin matrix row passes, including logical-
   start under wrap reversal and fixed opposite-margin replacement;
4. composition controls preserve all existing owners and both scalar lanes;
5. no public API, dependency, feature, MSRV, docs, root, fixture, corpus,
   generated artifact, helper, parser, manifest, browser pin, or generator file
   changes, and no generator command runs; and
6. the remotely available C01 candidate handoff gives C02 the exact planning
   revisions, task ranges, finding evidence, verification, and unchanged-
   artifact proof.

At the complete-status head, run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_intrinsic_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_cross_auto_margin_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_absolute_auto_margin_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_composition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c01_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just parity-all
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --exit-code d386c7d796e5fe0c0856c15ac800516df1348f3b..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' src tests
git diff --check d386c7d796e5fe0c0856c15ac800516df1348f3b..HEAD
git status --short
```

The protected-path diff and unsafe scan must print no output; `git status` must
be clean. The parity, corpus, Taffy, and generator-feature commands verify the
checked-in artifacts and configurations without regeneration. `FLEX-005`
remains assigned only to C02. Genuine blockers are limited to those defined by
the installed workflow; none is currently known.
