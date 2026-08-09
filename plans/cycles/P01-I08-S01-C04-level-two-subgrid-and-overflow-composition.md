# P01-I08-S01-C04 Level 2 Subgrid And Overflow Composition

Status: draft

Cycle ID: `P01/I08/S01/C04`

Owning repository: `surgeist-layout`

Cycle base: `dd17b395dc19abdd9bbda437799be4483741a5e7`

## 1 Authority And Outcome

This just-in-time plan implements the reviewed specification
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
committed as `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`. Its controlling sources are
`D-18` through `D-21`, section 11, the subgrid, baseline-control, overflow,
composition, architecture, error, finding-closure, and acceptance portions of
sections 12 and 14 through 19, and the remaining standalone/baseline/overflow
portion of `GRID-010`.

The durable sequence is
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
SHA-256 `62e6b43402a038e7df5bc22e5c28ee40b7e7ae1a1ac6fc28224c12626cc9ca7c`,
committed as `75801ea77e37af28c0dda32a28fd1647123e1293`. This is its C04 entry.

C03 is complete and remotely verified at the cycle base. C04 makes a standalone
queried subgrid axis an ordinary measured boundary, then composes the completed
FRI-08 topology/sizing paths with the existing FRI-06 baseline carrier and
FRI-05 scroll-contribution owner. It corrects only defects exposed by those
compositions. C05 receives complete individual finding behavior without any
remaining private standalone-traversal error.

## 2 Entry Evidence And Boundary

`src/grid/subgrid.rs` already models `SubgridTraversalAxis::Standalone`, but
`apply_subgrid_edge_placeholders` returns
`StandaloneSubgridTraversalUnsupported`, and production child collection drops
the standalone branch. The checked-in
`subgrid/subgrid_standalone_axis_column_autoflow` family consequently returns an
unsupported min-content grid-sizing error in all four variants.

A standalone queried axis must stop ancestor flattening at that node. The node
is measured under its translated parent-span constraint and contributes one
minimum/min-content/max-content margin-box leaf for the active phase, including
accumulated outer MBP and half-gap facts once. Its descendants remain inside its
ordinary local grid layout. Its other axis is independent: it may still inherit,
and `FlowAxes` performs contextual physical projection.

FRI-06 already owns immutable inherited baseline groups, first/last roles,
owner-to-current mappings, half-gap and MBP adjustment, physical projection,
and the no-publication-inverse/no-sizing-fixed-point rule. C04 adds composed
regressions over the settled FRI-08 topology and standalone boundary; it creates
no baseline algorithm or Level 3 stacking-axis lanes alignment.

FRI-05 already owns canonical scroll contributions. Current frozen parity
evidence exposes four failures per source: `grid_overflow_inline_axis_scroll`
reports scroll height `15` instead of `0`;
`subgrid_overflow_hidden_does_not_prohibit` reports width `50` instead of `100`;
and each sibling-footer source reports height `236` instead of `308`. C04 must
feed final container-relative physical border boxes and retained descendant
intervals through that owner without a grid-specific scroll rectangle, erased
negative/reversed range, or duplicated area origin.

Out of scope are C05 browser input/adapter settlement; C06 generation; XML,
HTML, reports, provenance, fixtures, manifests, helper/generator code, and corpus
changes; ordinary baseline-distribution work owned by FRI-09; grid-aligned
absolute/static-position work owned by FRI-10; stacking-axis lanes baseline
alignment; fragmentation; authored CSS; root/sibling changes; dependencies,
features, MSRV, docs/examples, and unrelated cleanup. C04 adds no unsafe code,
suppression, compatibility state, second axis/order/cache/transaction/baseline/
scroll owner, fixture dispatch, or unsupported fallback.

## 3 Impacts And Frozen Evidence

Public API effect is internal-only. C04 removes the private standalone traversal
error and its oracle mirror; public layout inputs, outputs, errors, traits,
defaults, and reexports are unchanged. Dependencies, features, MSRV,
docs/examples, browser inputs, generated artifacts, and root are unchanged.

Frozen SHA-256 values remain report
`5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`,
helper `caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`,
and corpus manifest
`4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`.
The frozen inventory is 1,438 HTML inputs, 5,736 comment-free XML files, and a
schema-3 report with 5,736 generated, 16 unsupported, three expected-fail, zero
quarantined, and zero failed rows. All owned Rust remains free of `unsafe`.

## 4 Task Order

Tasks execute sequentially. Each receives a fresh implementation worker,
test-first or explicit characterization evidence, one logical commit, and a
fresh exact-range task review before the next dependent task.

### 4.1 `P01/I08/S01/C04/T01` Measure Standalone Subgrid Axis Boundaries

**Owned files:** standalone traversal in `src/grid/subgrid.rs`; only required
measurement/orchestration boundaries in `src/grid/mod.rs`, `src/grid/tracks.rs`,
and `src/grid/child.rs`; the exact oracle mirror in
`src/test_support/oracle/grid/subgrid.rs`; and focused tests in
`src/grid_tests.rs`. Baseline placement and scroll contribution policy remain
fixed for T02 and T03.

**Outcome:** Retain standalone as an explicit traversal result. Translate its
parent span, stop recursion in that axis, and measure the standalone grid
container as one ordinary leaf under the contextual area constraint for the
active minimum/min-content/max-content phase. Add accumulated outer edge and
half-gap facts exactly once. Local descendants execute ordinary local grid
layout; the other axis may inherit independently. Remove the private
`StandaloneSubgridTraversalUnsupported` state and all leaf/oracle uses.

**Required RED prefix:** `fri08_c04_standalone_` proves through production
layout that a standalone column and row axis contributes its descendants'
intrinsic margin box instead of erroring or disappearing. Cover one/both-axis
inheritance, nested boundaries, definite/indefinite constraints, minimum,
min/max-content, auto-flow, area names, percentages, unequal gaps, MBP,
reversal, all flow mappings, f32/f64, provider/non-finite errors, cold/warm
cache, retry, order, and rollback. The focused RED must be an assertion or typed
error mismatch caused by the old drop/unsupported path.

**Acceptance:** Standalone nodes produce one phase-correct ancestor leaf and no
descendant leakage. The four checked-in
`subgrid_standalone_axis_column_autoflow` variants pass unchanged. No boolean
history flag, wrapper-zero substitute, second measurement cache, or ignored
lower bound exists. Existing inherited-only traversal remains green.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c04_standalone_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout subgrid_traversal
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_standalone_axis_column_autoflow cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout subgrid
cargo fmt --check
```

**Commit:** `fix(subgrid): measure standalone axis boundaries`

### 4.2 `P01/I08/S01/C04/T02` Preserve FRI-06 Baseline Views In Composed Grid Layout

**Owned files:** consumption of existing baseline views in
`src/grid/subgrid.rs`, `src/grid/child.rs`, and only required orchestration in
`src/grid/mod.rs`; focused tests in `src/grid_tests.rs`. FRI-06 carrier types,
group ownership, and ordinary FRI-09 distribution remain fixed.

**Outcome:** Verify the immutable FRI-06 ancestor views remain authoritative
after C01-C03 topology/sizing and T01 standalone measurement. Area-created and
implicit tracks, auto-fit, standalone boundaries, and lanes subgrids consume
the existing direct-owner/inherited-current-grid targets with first/last roles,
edge/gap adjustment, and physical mapping intact. Correct only a composed call-
site defect proven by the new behavior evidence.

**Required pre-change prefix:** `fri08_c04_baseline_` exercises production
layout for first/last groups across area-created, implicit, auto-fit,
standalone, and lanes-subgrid cases; both axes, horizontal/vertical/sideways
flows, reversal, unequal gaps/MBP, order, f32/f64, cache, error, and rollback.
If a case fails, preserve the assertion-level RED and make the smallest fix. If
all cases pass, classify them honestly as characterization and do not edit
production merely to manufacture RED.

**Acceptance:** Ancestor groups remain immutable; owner/current mappings and
first/last roles remain distinct; adjustment occurs once; and neither sizing
reads published placement nor publication re-enters sizing. Existing FRI-06
no-fixed-point, synthesized-cycle, reversal, flow, and baseline-group controls
remain green. No FRI-09 or stacking-axis lanes behavior is claimed.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c04_baseline_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c12_t08_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout baseline_group_axis
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout baseline
cargo fmt --check
```

**Commit:** `test(grid): verify inherited baseline composition`

### 4.3 `P01/I08/S01/C04/T03` Preserve Canonical Scroll Contributions

**Owned files:** final grid/subgrid physical contribution assembly in
`src/grid/child.rs`; only required settled-geometry orchestration in
`src/grid/mod.rs` and `src/grid/subgrid.rs`; and focused tests in
`src/grid_tests.rs`. `src/scroll.rs` and normalized overflow semantics are the
existing FRI-05 owner and remain fixed absent a separately proven owner defect.

**Outcome:** Supply each in-flow grid child and inherited descendant to the
canonical accumulator with its final container-relative physical border box,
positive margin outsets, normalized overflow, and propagatable descendant
intervals. Preserve signed/reversed ranges and terminal padding. Do not count
an area origin twice, promote track-local coordinates, or let overflow alter
intrinsic eligibility except through the existing automatic-minimum rule.

**Required RED prefix:** `fri08_c04_overflow_` reproduces the four exact current
failure kinds and proves visible/hidden/clip/scroll/auto, nested propagation,
reversal, zero/negative origin, scrollbar settling, both axes/flows/scalars,
order, cache, provider/non-finite failure, and rollback through production
layout. Existing frozen XML supplies independent values; tests must not dispatch
on fixture names.

**Acceptance:** All 16 variants across the four sources in section 2 pass
unchanged. `grid_scroll_contributions` remains the grid adapter to the canonical
FRI-05 accumulator; no grid-local scroll rectangle, overflow reparser, alternate
range union, or intrinsic-sizing shortcut is introduced. Block/flex and T01/T02
controls remain green.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c04_overflow_
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid/grid_overflow_inline_axis_scroll cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_overflow_hidden_does_not_prohibit cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_sibling_overflow_footer_second_matches_first cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_sibling_overflow_footer_third_matches_first cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri05_
cargo fmt --check
```

**Commit:** `fix(grid): preserve canonical scroll contributions`

## 5 Cycle Completion Gate

After T03 is task-clean, follow the canonical status-only `complete` transition,
then run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c04_
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_standalone_axis_column_autoflow cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=grid/grid_overflow_inline_axis_scroll cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_overflow_hidden_does_not_prohibit cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_sibling_overflow_footer_second_matches_first cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true SURGEIST_PARITY_FILTER=subgrid/subgrid_sibling_overflow_footer_third_matches_first cargo test --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
git diff --exit-code dd17b395dc19abdd9bbda437799be4483741a5e7..HEAD -- Cargo.toml Cargo.lock README.md tests/layout/browser_parity tests/layout/browser_parity.rs tests/bin scripts
```

Build an explicit tracked/non-ignored owned-Rust manifest and apply the canonical
unsafe scan from the implementation gate. Inspect the complete C04 range for
new allow/expect attributes, standalone errors or boolean history, descendant
leakage, recreated baseline groups, sizing/publication feedback, grid-specific
scroll rectangles, track-local/area-relative publication, overflow-driven
intrinsic shortcuts, fixture dispatch, compatibility state, and later-cycle
policy. Verify the frozen hashes/counts in section 3 without generation or
browser execution, and confirm the later-owned FRI-09/FRI-10 fixtures remain
unchanged and unclaimed.

C04 is publication-ready only when all three ordered task ranges are clean; the
standalone axis contributes one ordinarily measured phase leaf; FRI-06 baseline
views and FRI-05 scroll contributions remain sole owners; all five frozen source
families pass; the full gate passes; and a fresh holistic review of the exact
cycle range returns `CLEAN`. Follow the canonical publication/readback/cleanup
gate. C05 receives complete individual GRID-010 behavior and owns browser input,
finite adapter, documentation, and final composition settlement.
