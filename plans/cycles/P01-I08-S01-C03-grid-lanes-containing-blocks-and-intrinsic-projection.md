# P01-I08-S01-C03 Grid-Lanes Containing Blocks And Intrinsic Projection

Status: reviewed

Cycle ID: `P01/I08/S01/C03`

Owning repository: `surgeist-layout`

Cycle base: `91f9e5deea035f583c3f49d35165f15ed0106ebf`

## 1 Authority

This just-in-time plan implements the reviewed specification
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
committed as `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`. Its controlling sections are
D-08, D-14 through D-17, D-21, section 10, and the lanes, architecture, error,
finding-closure, and acceptance portions of sections 12 and 14 through 19. It
closes `GRID-002` and the nested-lanes portion of `GRID-010` only.

The durable sequence is
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
SHA-256 `62e6b43402a038e7df5bc22e5c28ee40b7e7ae1a1ac6fc28224c12626cc9ca7c`,
committed as `75801ea77e37af28c0dda32a28fd1647123e1293`. This is its C03 entry.

C02 is complete, published, and remotely verified at the cycle base. The
specification owns behavior and public compatibility, the sequence owns cycle
order, and this plan owns only C03 execution detail.

## 2 Entry State And Bounded Outcome

Grid-lanes currently measures the stacking axis from an item's tentative lane
margin box. `measure_lane_axis_margin_box_with_grid_axis` therefore constructs
a parent size with only the selected grid-axis span, while intrinsic collection
uses the container's physical inner size without expressing the same hybrid
contract. Final child layout then uses a local `GridArea` whose stacking extent
is the measured lane margin box. Percentage sizing, preflight, alignment,
subgrid context, final placement, and physical reversal do not share one
containing block.

Track expansion retains C01 auto-repeat origin metadata, but the pre-sizing
grid-lanes path still limits auto-fit with a visible-cell count and retains
child-count/`div_ceil` demand. C03 replaces only that lanes behavior with the
Level 3 heuristic: explicitly occupied auto-fit tracks remain; let `N` be the
sum of automatic in-flow item spans, keep the first `N` otherwise-unoccupied
candidate tracks, and collapse all remaining auto-fit tracks before lanes
placement. Collapsed candidates cannot receive automatic placement. Ordinary
post-placement collapse remains the distinct, frozen C02 policy.

`lane_intrinsic_sizing_with` currently groups all automatic items by span alone
and fabricates each possible start from the group maximum. Production collection
represents a nested indefinite lanes subgrid with a public unsupported kind,
returns `NestedGridLanesSubgridIndefiniteUnsupported`, and `src/grid/mod.rs`
silently drops that lower bound. C03 instead carries definite or automatic
candidate starts, baseline-sharing role, and edge facts through one private
projection. Equivalent facts may share componentwise maxima; non-equivalent
facts remain separate. Virtual contributions reuse the ordinary C02 track
contribution phases.

A nested indefinite lanes subgrid is flattened through its descendants in the
grid axis. Local spans translate to every parent candidate; automatic
descendants project to every allowed start; physical margin/border/padding and
start/end half-gap facts retain the maximum applicable edge contribution.
Definite wrappers limit candidates to their parent span. No placement result
feeds intrinsic sizing, no wrapper zero box substitutes for descendants, and
provider/error/cache/transaction behavior remains fallible and atomic.

The only public change is the breaking removal of
`LaneIntrinsicItemKind::NestedIndefiniteSubgrid`,
`LaneIntrinsicItemOf::nested_indefinite_subgrid`, and
`LanePlacementError::NestedGridLanesSubgridIndefiniteUnsupported`. Definite and
indefinite public virtual items remain. Root API artifacts and direct root uses
are separate root-owned follow-up after the final leaf candidate.

Out of scope are ordinary grid behavior; standalone Level 2 subgrid boundaries;
FRI-06 baseline ownership; FRI-05 overflow composition; stacking-axis lanes
baseline alignment; authored CSS; browser inputs or generation; fixtures,
XML, reports, provenance, root, dependencies, features, MSRV, and FRI-09 through
FRI-13. C03 adds no unsafe code, suppression, compatibility state, zero-box
fallback, fixture dispatch, or second ordering/cache/transaction owner.

## 3 Impacts And Frozen Evidence

The public API effect is the breaking three-symbol removal named in section 2;
all other public inputs, outputs, errors, defaults, traits, and reexports remain
unchanged. Dependencies, features, MSRV, docs/examples, and leaf-owned generated
artifacts are unchanged. Root must later update direct symbol use and regenerate
its root-owned API audit artifacts while promoting the final leaf gitlink.

Browser and generator inputs and artifacts are frozen. The schema-3 report
remains the sole provenance record with 5,736 generated rows, 16 unsupported
rows, three expected-fail source records, zero quarantined rows, and zero failed
rows. Frozen SHA-256 values are report
`5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`,
helper `caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`,
and corpus manifest
`4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`.
The frozen inventory is 1,438 HTML inputs and 5,736 comment-free XML files.
All owned code remains free of `unsafe`.

## 4 Task Order

Tasks execute sequentially. Each receives a fresh implementation worker,
behavioral RED before production edits, focused GREEN evidence, one exact
commit, and a fresh exact-range task review before the next task starts.

### 4.1 `P01/I08/S01/C03/T01` Use One Hybrid Lanes Containing Block

**Owned files:** the lanes measurement and final-layout paths in
`src/grid/lanes.rs`; only required orchestration inputs in `src/grid/mod.rs`;
and focused tracked tests in `src/grid_tests.rs`. Track expansion, auto-fit,
intrinsic grouping, public symbols, and subgrid traversal are frozen.

**Outcome:** Introduce one private per-item hybrid containing-block value whose
grid-axis component is the selected settled track span and whose stacking-axis
component is the grid-lanes container content box. Project it through
`FlowAxes` once. Percentage margin/padding/preferred/min/max resolution,
stretch/aspect-ratio preflight, intrinsic measurement, final child layout,
self-alignment, subgrid child context, and RTL/vertical/sideways placement all
consume that value. The measured lane margin box remains output, never its own
percentage basis.

**Required behavioral RED prefix:** `fri08_c03_containing_block_` proves at
least:

- rows-only lanes with a `100px` content-box inline extent gives a `width:100%`
  child width `100` in LTR and RTL, in content-box and border-box modes;
- the columns-only counterpart uses the hybrid physical axis without swapping
  source fields;
- percentage margin, padding, min/max, stretch, aspect ratio, and replaced
  controls resolve against the same parent facts;
- min/max-content container sizing, all flow mappings, and f32/f64 preserve the
  same logical contract; and
- final location, subgrid child context, baseline synthesis, and scroll geometry
  do not reintroduce a tentative stacking-axis parent.

**Acceptance:** One carrier owns hybrid parent/available/known facts for
measurement and layout. The exact four checked-in
`grid_lanes_item_containing_block_content_width` variants and both intrinsic
container families use production behavior without fixture-name dispatch.
Existing invalid-basis, provider, cache, rounding, rollback, order, overflow,
and ordinary-grid controls remain green.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c03_containing_block_
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid-lanes/grid_lanes_item_containing_block_content_width cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_lanes_content
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_lanes_axes
cargo fmt --check
```

**Commit:** `fix(grid-lanes): use hybrid item containing blocks`

### 4.2 `P01/I08/S01/C03/T02` Apply The Level 3 Lanes Auto-Fit Policy

**Owned files:** lanes auto-fit policy over shared origins in
`src/grid/topology.rs` and `src/grid/lanes.rs`; only necessary expansion and
orchestration integration in `src/grid/tracks.rs` and `src/grid/mod.rs`; and
focused tracked tests in `src/grid_tests.rs`. Ordinary C02 collapse and sizing
remain fixed.

**Outcome:** Before lanes placement, classify explicitly occupied auto-fit
tracks and automatic in-flow item spans. Retain explicit occupancy plus the
first `N` remaining candidates, where `N` is the checked sum of automatic spans;
collapse the rest with zero size and adjacent gutters while preserving track,
line, name, repeat-group, and repetition identity. Automatic lanes placement
searches only retained candidates. Auto-fill, implicit, inherited, and ordinary
tracks do not collapse through this policy.

**Required behavioral RED prefix:** `fri08_c03_auto_fit_` proves at least:

- explicit placements keep their covered repeated tracks, including overlap;
- automatic spans, not raw item count or total cell area, determine `N`;
- zero automatic demand collapses every otherwise-unused auto-fit repetition;
- automatic placement skips collapsed candidates while definite and named lines
  still address retained line identity;
- adjacent gutters collapse without changing lanes running offsets; and
- both axes, row/column flow, dense/tolerance controls, writing modes, and
  f32/f64 agree while ordinary auto-fit and auto-fill retain C02 behavior.

**Acceptance:** No lanes auto-fit limit derives from visible child count,
`div_ceil`, ordinary settled occupancy, or post-placement child count. One named
lanes policy consumes topology origins, checked automatic spans, and explicit
placement facts before resolved lanes placement. Existing order, error, cache,
rollback, and ordinary negative controls remain green.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c03_auto_fit_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout auto_fit
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout grid_lanes
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c02_auto_fit_
cargo fmt --check
```

**Commit:** `fix(grid-lanes): apply level three auto-fit`

### 4.3 `P01/I08/S01/C03/T03` Project Intrinsic Contributions At Every Candidate

**Owned files:** the lanes intrinsic model and projection in
`src/grid/lanes.rs`; only the shared ordinary contribution call boundary in
`src/grid/tracks.rs` and orchestration in `src/grid/mod.rs`; and focused tracked
tests in `src/grid_tests.rs`. Nested descendant flattening and public removal
remain T04.

**Outcome:** Retain definite items at their exact grid-axis span and project
each automatic item at every active start where its span fits. A private
equivalence key includes span, candidate set, baseline-sharing role, and edge
facts; only equal keys take componentwise maxima. Feed every resulting virtual
item through the C02 ordinary contribution state so minimum, automatic minimum,
min-content, max-content, MBP, gaps, fixed companions, and content-sized tracks
share one phase pipeline.

**Required behavioral RED prefix:** `fri08_c03_intrinsic_` proves at least:

- definite placement contributes at one exact span and automatic placement at
  every allowed start for spans one and greater than one;
- non-equivalent candidate, baseline-role, and edge facts never merge by span
  alone, while equivalent items take componentwise maxima;
- mixed fixed/content-sized tracks and gaps distribute only the required
  contribution at each candidate;
- the eight min/max-content container variants close through public layout; and
- item order, tolerance, both axes, all flow mappings, automatic minima,
  provider failures, cache cold/warm, rollback, and f32/f64 remain deterministic.

**Acceptance:** No placement result feeds intrinsic sizing, no source-specific
offset rule exists, and span alone is not an equivalence key. Container
min/max-content grid-axis sizing is the aggregate of the same virtual
contributions used for track lower bounds. T01 hybrid bases and T02 active
candidates are authoritative.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c03_intrinsic_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout lane_intrinsic
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid-lanes/grid_lanes_min_content_container_sizing cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid-lanes/grid_lanes_max_content_container_sizing cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
cargo fmt --check
```

**Commit:** `fix(grid-lanes): project intrinsic candidates`

### 4.4 `P01/I08/S01/C03/T04` Flatten Nested Lanes Subgrids And Remove Unsupported State

**Owned files:** production descendant collection and public lane model in
`src/grid/lanes.rs`; existing inherited-axis facts only where required in
`src/grid/subgrid.rs`; error orchestration and reexports in `src/grid/mod.rs` and
`src/lib.rs`; exact source-derived public-removal evidence in `src/lib_tests.rs`;
and behavioral tests in `src/grid_tests.rs`. Standalone-axis policy is C04.

**Outcome:** For a nested indefinite lanes subgrid, flatten every contributing
descendant into T03 candidate groups. Translate local definite/automatic spans
to every allowed parent start, restrict definite wrappers to their parent span,
and retain maximum start/end physical MBP and half-gap facts independently.
Remove the public nested-indefinite kind, constructor, and error variant; delete
the ignored-lower-bound branch. Public `indefinite` remains an already-aggregated
convenience and is not a production replacement for tree flattening.

**Required behavioral RED prefix:** `fri08_c03_nested_` plus exact public-removal
evidence proves at least:

- an automatically placed nested lanes subgrid contributes descendants at
  every parent candidate instead of publishing the wrapper's zero lower bound;
- definite wrappers and automatic descendants translate spans without escaping
  the wrapper's allowed parent tracks;
- unequal gaps, reversal, MBP, and start/end edge maxima are applied once;
- nesting depth, both axes, all flows, order, tolerance, f32/f64, provider error,
  non-finite value, cache cold/warm, and transaction rollback remain correct;
- the three removed public symbols are absent from declarations and reexports,
  and definite/indefinite callers retain their public behavior.

**Acceptance:** Production has no unsupported nested-indefinite branch, ignored
lower bound, compatibility state, or wrapper-zero substitution. Flattening
reuses T03 projection and ordinary contribution state. The structural absence
test is exact API-artifact evidence paired with real public nested-tree behavior,
not a proxy for it. Root follow-up is recorded but not implemented here.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c03_nested_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c03_public_removal_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout lane_intrinsic
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout subgrid
CARGO_NET_OFFLINE=true just verify
cargo fmt --check
```

**Commit:** `feat(grid-lanes): flatten nested intrinsic subgrids`

## 5 Cycle Completion Gate

After T04 is task-clean, follow the canonical planning gate for the status-only
`complete` transition, then run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c03_
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid-lanes/grid_lanes_item_containing_block_content_width cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid-lanes/grid_lanes_min_content_container_sizing cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid-lanes/grid_lanes_max_content_container_sizing cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
git diff --exit-code 91f9e5deea035f583c3f49d35165f15ed0106ebf..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin scripts
surgeist_c03_owned_rust_manifest=$(mktemp /tmp/surgeist-layout-c03-owned-rust.XXXXXX)
git ls-files --cached --others --exclude-standard '*.rs' > "$surgeist_c03_owned_rust_manifest"
! xargs rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' < "$surgeist_c03_owned_rust_manifest"
rm "$surgeist_c03_owned_rust_manifest"
git status --short --branch
```

The unsafe scan passes only with no match across every tracked and non-ignored
Rust source. Inspect the complete C03 diff for new allow/expect attributes,
child-count or `div_ceil` lanes demand, ordinary/lanes auto-fit conflation,
span-only intrinsic grouping, duplicate contribution or containing-block
owners, ignored nested lower bounds, compatibility states, fixture dispatch,
and later-cycle policy. Verify the frozen hashes/counts in section 3 without
browser acquisition or generation.

The C03 candidate is publication-ready only when all four task ranges and
reviews are clean; one hybrid containing block feeds measurement and layout;
the Level 3 auto-fit heuristic consumes topology origins before placement;
candidate projection and nested descendant flattening feed the ordinary
contribution state; the three unsupported public symbols are absent; the full
gate passes; and a fresh holistic review returns no finding.

Follow the canonical planning/publication gates for status recovery, final
review, landing, remote verification, and cleanup. C04 receives complete Level
3 grid-axis sizing and placement with no nested-indefinite unsupported branch.
C04 alone adds standalone Level 2 subgrid measurement boundaries and composes
FRI-06 baseline and FRI-05 overflow behavior.
