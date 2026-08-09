# P01-I08-S01-C02 Ordinary Track Sizing And Auto-Fit

Status: reviewed

Cycle ID: `P01/I08/S01/C02`

Owning repository: `surgeist-layout`

Cycle base: `77c35d34607e054db28782d7253e4b9787bcce15`

## 1 Authority

This just-in-time plan implements the reviewed specification
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
committed as `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`. Its controlling sections are
D-07, D-09 through D-11, D-21, section 8, and the ordinary-grid sizing,
auto-fit, architecture, error, finding-closure, and acceptance portions of
sections 12 and 14 through 17. It closes `GRID-003`, `GRID-006`, and `GRID-007`
only.

The durable sequence is
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
SHA-256 `62e6b43402a038e7df5bc22e5c28ee40b7e7ae1a1ac6fc28224c12626cc9ca7c`,
committed as `75801ea77e37af28c0dda32a28fd1647123e1293`. This is its C02 entry.

C01 is complete, published, and remotely verified at the cycle base. The
specification owns behavior, the sequence owns cycle order, and this plan owns
only C02 execution detail.

## 2 Entry State And Bounded Outcome

C01 supplies canonical expanded topology, origin-complete auto-repeat identity,
settled integer areas, and placement-before-sizing. Ordinary track sizing still
has two parallel scalar paths. `resolve_inline_tracks` returns early for any
fit-content maximum, so flexible and other intrinsic tracks in that axis do not
finish their ordinary phases. Both sizing paths count stretch eligibility only
for exact `auto/auto`. Auto-fit identity survives placement but no ordinary
post-placement collapse consumes settled occupancy, and uniform-gap arithmetic
cannot express collapsed adjacent gutters.

C02 introduces one private ordinary per-axis sizing state in
`src/grid/tracks.rs`. Each track state carries its sizing functions, base size,
growth limit, resolved fit-content limit when present, flex factor when present,
auto-max stretch eligibility, and collapsed status. The same state transitions
serve rows and columns and both scalar lanes. It is not a public model and does
not replace `ExpandedGridTopology` as the owner of track identity, names,
origins, or placed areas.

`ExpandedGridTopology` owns one collapsed bit per expanded track. After ordinary
placement and before intrinsic contribution or sizing, it marks an auto-fit
repetition track collapsed exactly when no in-flow settled area intersects that
track. Implicit, inherited, auto-fill, and non-auto-repeat tracks never collapse
through this policy. Collapsed tracks retain line/name/origin identity but enter
the solver as fixed zero, cannot receive contribution/flex/stretch, and have
zero gutter on both adjacent edges.

`src/grid/tracks.rs` owns the resulting used axis geometry: settled sizes plus
line offsets derived from the topology's collapsed mask and the authored gap.
The private `UsedGridAxisGeometryOf<S>`-equivalent contains settled sizes, the
collapsed mask, one gutter-after value per track boundary, and line offsets. It
alone answers active-gap total, total extent, half-open span extent, and line
offset. `src/grid/placement.rs`, child layout, intrinsic span accounting,
absolute placement, baseline collection, and overflow consume those methods;
no second collapse predicate or uniform-gap fallback may reconstruct them. When
a collapsed track lies between two active tracks, both gutters adjacent to the
collapsed track are zero.

Inherited ordinary-subgrid contexts carry the same used-axis geometry rather
than reducing it to `tracks + uniform gap`. `src/grid/child.rs` passes the
carrier into `src/grid/subgrid.rs`, which slices or reverses sizes, boundary
gutters, and line offsets together when projecting an inherited axis. This is
geometry propagation only: C02 does not change standalone/nested traversal,
intrinsic flattening, baseline grouping, or any C03/C04 subgrid policy.

The existing shared track-sizing entry receives an explicit private
`GridTrackSizingPolicy::{Ordinary, Lanes}`-equivalent from its formatting-context
caller. `Ordinary` selects the C02 state/phases. `Lanes` preserves the published
pre-C02 lanes resolver and accepts only sizing functions and other inert shared
facts until C03 replaces its policy. Display checks are confined to this one
orchestration boundary; no phase contains scattered ordinary/lanes conditionals.
Focused negative controls freeze lanes fit-content, stretch, auto-fit, and
placement behavior throughout C02.

C02 integrates fit-content as a per-track growth cap inside intrinsic,
spanning, and flexible phases. It then stretches every non-collapsed track whose
maximum is `Auto` from positive definite remaining space after active gutters,
settled fixed/intrinsic bases, and flex use. The minimum remains a floor.

Out of scope are placement/name redesign, Level 3 lanes auto-fit and intrinsic
projection, lanes containing blocks, nested/standalone subgrid policy, baseline
ownership, browser inputs, generation/provenance/artifacts, public API, root,
dependencies, features, MSRV, and FRI-09 through FRI-13 behavior. C02 adds no
unsafe code, lint suppression, expected-fail row, quarantine, fixture dispatch,
or compatibility path.

## 3 Impacts And Frozen Evidence

Public API, documented errors, dependencies, features, MSRV, docs/examples, and
root follow-up are unchanged. The behavior effect is internal ordinary-grid
track sizing and geometry only. Existing transaction, cache, source association,
`FlowAxes`, overflow, and replaced-item owners remain unchanged. All owned code
remains free of `unsafe`.

Browser and generator inputs are frozen throughout C02. The schema-3 report
remains the sole provenance record with 5,736 generated rows, 16 unsupported
rows, 3 expected-fail source records, zero quarantined rows, and zero failed
rows. Frozen SHA-256 values are report
`5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`,
helper `caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`,
and corpus manifest
`4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`.
The frozen inventory is 1,438 HTML inputs and 5,736 comment-free XML files.

## 4 Task Order

Tasks execute sequentially. Each receives a fresh implementation worker,
behavioral RED before production edits, focused GREEN evidence, one exact
commit, and a fresh exact-range task review before the next task starts.

### 4.1 `P01/I08/S01/C02/T01` Unify Fit-Content With Ordinary Track Phases

**Owned files:** `src/grid/tracks.rs`; the single private ordinary/lanes policy
dispatch in `src/grid/mod.rs`; and focused tracked tests in `src/grid_tests.rs`.
No topology collapse or ordinary stretch-policy change.

**Outcome:** Replace the collection-wide fit-content branch with one private
ordinary track state and phase pipeline. Initialize distinct base and growth
limit facts, retain fit-content limits per track, apply non-spanning and spanning
intrinsic contributions to those facts, cap fit-content growth only when its
argument becomes limiting, and resolve flex tracks from remaining definite
space without bypass. Use the same implementation for rows/columns and f32/f64.
Existing external helper signatures may become thin adapters only when they do
not own a parallel ordinary solver. The pre-C02 shared resolver becomes the
explicit `Lanes` policy path; C02 fit-content phases are unreachable from that
path until C03.

**Required behavioral RED prefix:** `fri08_c02_fit_content_` runs through public
layout and proves at least:

- `[fit-content(50px),1fr]` in a definite `200px` axis with settled intrinsic
  bases `[20,0]` yields `[20,180]`, not a collection-wide fit-content result;
- the row-axis equivalent produces the same logical sizes;
- a spanning contribution grows eligible non-fit tracks while the fit-content
  track stops at its argument;
- definite/indefinite percentage limits, min/max-content companions, and
  sub-one flex factors retain their existing typed semantics;
- vertical/sideways projection and f32/f64 agree within existing tolerance.
- rows-only and columns-only grid-lanes fit-content controls retain their exact
  published pre-C02 geometry.

**Acceptance:** No fit-content maximum causes an early return from
`resolve_inline_tracks` or any replacement entry point. One state transition
pipeline owns base/growth/flex/fit facts. Existing checked-in fit-content/span,
intrinsic contribution, percentage, flex, invalid numeric, rounding, cache, and
rollback families remain green without source-name dispatch.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_fit_content_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fit_content
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout intrinsic_span
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout flex_track
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_lanes_negative_
cargo fmt --check
```

**Commit:** `refactor(grid): unify fit-content track phases`

### 4.2 `P01/I08/S01/C02/T02` Collapse Ordinary Auto-Fit From Settled Occupancy

**Owned files:** collapse state in `src/grid/topology.rs`; ordinary auto-fit
policy and used axis geometry in `src/grid/tracks.rs`; placement-to-collapse
orchestration in `src/grid/mod.rs`; `src/grid/placement.rs` as the sole consumer
for definite, automatic, span, and absolute used-axis geometry;
`src/grid/child.rs` for ordinary child and inherited-context carriage;
`src/grid/subgrid.rs` only to slice and reverse the same geometry carrier across
an already-supported inherited axis; and focused tracked tests in
`src/grid_tests.rs`. T01 sizing phases and policy dispatch may be consumed but
not redesigned. Standalone/nested traversal and contribution policy are frozen.

**Outcome:** After C01 placement, mark each ordinary auto-fit repeated track
occupied when any in-flow settled area spans it, then collapse every other such
track. Retain track count, line names, negative-line behavior, repeat-group and
repetition identity. Feed collapsed bits into the T01 state as fixed zero and
derive one axis geometry whose adjacent collapsed gutters are zero. Every
ordinary child, span contribution, baseline, overflow, and absolute-placement
offset consumes `UsedGridAxisGeometryOf` methods rather than sizes plus uniform
gap. An inherited subgrid receives the same boundary gutters and line offsets,
including reversed-axis order, without reconstructing a scalar gap. Auto-fill
and the `Lanes` policy do not use this collapse.

**Required behavioral RED prefix:** `fri08_c02_auto_fit_` proves at least:

- three `40px` auto-fit columns in `120px` with two children overlapping the
  first leave one `40px` track centered at x `40`;
- an item spanning a repeated track prevents that track from collapsing;
- an all-empty repetition collapses all repeated tracks and adjacent gutters;
- active tracks separated by collapsed repetitions receive no ghost gutter;
- an in-flow span and an absolute item crossing collapsed repetitions consume
  canonical line/span geometry with no uniform-gap reconstruction;
- an ordinary inherited subgrid crossing collapsed repetitions preserves zero
  adjacent gutters in forward and reversed axes, including baseline and
  scroll-overflow projection;
- names and positive/negative lines still address retained collapsed lines;
- auto-fill remains expanded, and row/column flow plus f32/f64 are symmetric.

**Acceptance:** Collapse derives only from settled in-flow ordinary occupancy.
No child-count cap, deletion of track identity, second occupancy walk, or gap
reconstruction remains. Collapsed tracks receive no intrinsic, spanning, flex,
or later stretch growth. Existing area geometry, absolute placement, baseline,
overflow, names, order, cache, error, and rollback tests remain green. Public
crossing-span, absolute-item, inherited-subgrid reversal, baseline, and
scroll-overflow controls prove that every downstream consumer uses the same
boundary gutters and offsets.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_auto_fit_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout auto_fit
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout named_grid
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_auto_placement
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_lanes_negative_
cargo fmt --check
```

**Commit:** `fix(grid): collapse ordinary auto-fit from occupancy`

### 4.3 `P01/I08/S01/C02/T03` Stretch Every Non-Collapsed Auto Maximum

**Owned files:** ordinary stretch phase in `src/grid/tracks.rs`; only narrowly
required integration in `src/grid/mod.rs`; and focused tracked tests in
`src/grid_tests.rs`. T01 state and T02 collapse/geometry ownership remain fixed.

**Outcome:** Compute definite remaining space after active gaps and settled
fixed, intrinsic, fit-content, and flex sizes. For normal/stretch content
distribution, divide only positive remainder equally among every non-collapsed
track whose maximum is `Auto`, regardless of minimum kind. Add the share to its
base while preserving its minimum floor. Other maxima and indefinite or
nonpositive remainder receive no stretch.

**Required behavioral RED prefix:** `fri08_c02_stretch_` proves at least:

- one `minmax(0,auto)` track in a definite `100px` axis resolves to `100px`;
- `minmax(min-content,auto)` and `minmax(max-content,auto)` stretch above their
  floors in both axes;
- fixed, fit-content, min/max-content maxima, flex, and collapsed tracks do not
  enter the eligibility divisor;
- flex and fit-content use is subtracted before the equal share;
- normal/stretch versus other distribution, gaps, writing modes, and f32/f64
  preserve logical/physical equivalence.
- grid-lanes fit-content/stretch and auto-fit negative controls retain the
  published pre-C02 result through the explicit `Lanes` policy.

**Acceptance:** Exact `auto/auto` matching is absent from stretch eligibility.
One auto-maximum predicate is consumed by both ordinary sizing paths. Stretch
runs after flex, uses positive definite remainder only, respects T02 active
gutters/collapse, and leaves existing alignment, intrinsic, rounding, cache,
error, rollback, baseline, and overflow controls green.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_stretch_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout stretch_auto
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_stretch
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_lanes_negative_
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Commit:** `fix(grid): stretch every auto maximum`

## 5 Cycle Completion Gate

After T03 is task-clean, follow the canonical planning gate for the status-only
`complete` transition, then run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
git diff --exit-code 77c35d34607e054db28782d7253e4b9787bcce15..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin scripts
surgeist_c02_owned_rust_manifest=$(mktemp /tmp/surgeist-layout-c02-owned-rust.XXXXXX)
git ls-files --cached --others --exclude-standard '*.rs' > "$surgeist_c02_owned_rust_manifest"
! xargs rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' < "$surgeist_c02_owned_rust_manifest"
rm "$surgeist_c02_owned_rust_manifest"
git status --short --branch
```

The unsafe scan passes only with no match across every tracked and non-ignored
Rust source. Also inspect the complete C02 diff for new allow/expect attributes,
parallel sizing/collapse/gap owners, stale fit-content early return, exact
`auto/auto` stretch matching, scalar-gap reconstruction in ordinary or inherited
subgrid geometry, fixture dispatch, and later-cycle policy. Verify the frozen
hashes/counts in section 3 without browser acquisition or generation.

The C02 candidate is publication-ready only when all three task ranges and
reviews are clean; one state pipeline owns ordinary base/growth/fit/flex facts;
topology-owned ordinary auto-fit collapse and canonical used gaps feed all
consumers; every non-collapsed auto maximum stretches correctly; the complete
gate passes; and a fresh holistic review returns no finding.

Follow the canonical planning/publication gates for status recovery, final
review, landing, remote verification, and cleanup. C03 receives the published
C02 tip, stable ordinary track state, topology collapse metadata, and canonical
used axis geometry. C03 alone adds Level 3 lanes containing-block, intrinsic,
auto-fit, nested-subgrid, and public-removal policies.
