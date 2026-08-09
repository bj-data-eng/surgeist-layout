# P01-I08-S01-C05 Grid Composition And Settled Browser Inputs

Status: reviewed

Cycle ID: `P01/I08/S01/C05`

Owning repository: `surgeist-layout`

Cycle base: `ea1cf33c5bdb83f96d0d5c266bb55526e75c7f1e`

## 1 Authority And Outcome

This just-in-time plan implements the reviewed specification
`plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`, SHA-256
`150c26e6c5b5fa703f090e861261ea2f03a7662caf4f83dfa52f49e40accb0ba`,
committed as `c7d10c23c0cdfebfba6a6606d9ea5b89352572f5`. Its controlling sources are
`D-19` through `D-21`, the complete section 12 matrix, and the input, adapter,
fixture, documentation, architecture, finding, and pre-generation portions of
sections 13 through 19.

The durable sequence is
`plans/sequences/P01-I08-S01-grid-subgrid-and-grid-lanes-completeness.md`,
SHA-256 `62e6b43402a038e7df5bc22e5c28ee40b7e7ae1a1ac6fc28224c12626cc9ca7c`,
committed as `75801ea77e37af28c0dda32a28fd1647123e1293`. This is its C05 entry.

C04 is complete and remotely verified at the cycle base. C05 closes the eight
finding paths as one public-layout composition, completes the normalized grid
boundary documentation, and settles the exact finite browser inputs for C06.
It adds the ten specified HTML sources, adopts the eight existing controls, and
derives exactly 72 source/variant identities without generating artifacts.

## 2 Entry Evidence And Boundary

C01 through C04 independently close canonical topology/placement, ordinary
sizing/auto-fit, lanes projection, standalone subgrid measurement, inherited
baseline consumption, and canonical scroll contribution. Their combined
order, flow, replacedness, percentage, overflow, scrollbar, baseline-control,
cache, transaction, and rounding interactions lack one bounded acceptance
suite. Every finding therefore needs both its existing minimal oracle and a
composed public `compute_layout` oracle in f32 and f64.

All ten section 13.1 FRI-08 HTML sources are absent. The corpus has 1,438 HTML
inputs and 5,736 comment-free XML outputs. The sole schema-3 report records
5,736 generated, 16 unsupported, three expected-fail, zero quarantined, and
zero failed rows. Its SHA-256 is
`5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`;
the current helper is
`caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`;
and the current manifest is
`4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`.

The helper already serializes public grid tracks, auto flow, placement,
alignment, order, flow, overflow, gaps, sizing, edges, and intrinsic text.
`grid-template-areas` is accepted by the Rust adapter but is not serialized by
the helper. C05 may add that exact computed/authored value and only other
finite, public-grid facts proven necessary by a new source. Unknown explicit
tokens and malformed values must fail closed. No source name, expected geometry,
variant, or output identity may select input facts.

Out of scope are browser execution; any generation, filtered or full; XML or
report writes; a new parser layer; authored-CSS modeling; new public layout
input; duplicate topology, flow, order, baseline, scroll, cache, or transaction
owners; FRI-09 baseline distribution; FRI-10 grid-aligned absolute/static
positioning; stacking-axis lanes baseline alignment; fragmentation; root or
sibling changes; dependencies, features, MSRV, suppression, unsafe code,
generator architecture, and unrelated cleanup.

## 3 Impacts And Lineage State

Public layout API, dependencies, features, MSRV, and root integration remain
unchanged. Production edits are permitted only for a composition defect first
proven by T01. README changes describe the normalized layout boundary without a
broad CSS Grid Level 2/3 conformance claim.

C05 changes source inputs, not generated artifacts. HTML becomes exactly 1,448;
XML remains exactly 5,736 and byte-identical to the cycle base; `all.json`
remains byte-identical and intentionally records the C04 helper/manifest/source
lineage until C06. The manifest gains exactly the ten active cases while its
generated summary remains 5,736. Helper and manifest SHA-256 values are recorded
at completion for the C06 handoff.

Because the authoritative report is intentionally stale after input settlement,
`just corpus-check` is an expected lineage-freshness failure in C05 and is not
run as an acceptance gate. Historical C04 report constants remain historical;
current-source freeze tests use distinct C05 values. C06 alone may restore
freshness by its one unfiltered ExistingPinned run.

## 4 Task Order

Tasks execute sequentially. Each receives a fresh implementation worker,
test-first or explicit characterization evidence, one logical commit, and a
fresh exact-range task review before the next dependent task.

### 4.1 `P01/I08/S01/C05/T01` Close Public Grid Composition

**Owned files:** focused tests in `src/grid_tests.rs`; only the responsibility-
shaped production files under `src/grid/` required by an assertion-level RED.
`src/scroll.rs`, public input types, fixture tooling, README, and browser corpus
files remain fixed.

**Outcome:** Add bounded public-front-door compositions covering every finding
and independently varied pairs from section 12: placement/topology/auto-fit;
fit-content/flex/stretch; names/areas/subgrid; lanes/hybrid percentages;
standalone/inherited boundaries; all flow axes; order/replaced/source identity;
canonical overflow/scrollbar settling; and scalar/cache/error/transaction/
rounding reliability. Correct only a proven call-site interaction while
preserving the existing sole owners.

**Required pre-change prefix:** `fri08_c05_composition_` runs in f32 and f64 and
must include one composed case for each GRID-001/002/003/005/006/007/008/010 in
addition to the existing minimal oracles. It also records unchanged FRI-09 and
FRI-10 negative controls and default non-grid behavior. If all new cases pass,
classify them as characterization and make no production edit.

**Acceptance:** All valid cases return finite exact geometry with stable source
association. Cold/warm/invalidation and deterministic rounding agree; provider,
non-finite, and internal errors publish no batch or cache mutation. No fixture
dispatch, compatibility flag, sentinel geometry, alternate solver, or duplicate
owner appears.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c05_composition_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c0
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri09_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri10_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout non_grid
cargo fmt --check
```

**Commit:** `test(grid): close FRI-08 public composition`

### 4.2 `P01/I08/S01/C05/T02` Settle The Finite Browser Adapter

**Owned files:**
`tests/layout/browser_parity/scripts/gentest/test_helper.js`, the existing
adapter/parser and focused tests in `tests/layout/browser_parity/support.rs` and
`tests/layout/browser_parity.rs`, and test-only characterization/freezes in
`tests/bin/surgeist-layout-generate/generator.rs`. Generator production logic,
HTML, manifest, XML, report, README, and Rust layout production remain fixed.

**Outcome:** Serialize `grid-template-areas` and any other demonstrably required
existing public-grid value through the one current helper-to-XML-to-adapter
path. Preserve authored CSS as browser oracle input while lowering only finite
layout-ready values already represented by `NodeInput`. Reject unknown,
partial, non-finite, contradictory, or out-of-domain explicit values.

**Required RED prefix:** `fri08_c05_adapter_` proves each newly serialized fact
is absent before change, survives helper serialization and XML parsing, reaches
public layout independent of source name/variant/expectations, and rejects
unknown or malformed values. Mutation tests rename sources and alter expected
geometry without changing parsed inputs. Existing valid values remain stable.

**Acceptance:** One helper and one existing Rust parser path own the finite
lowering. No new parser layer, generated geometry input, source/expected-name
switch, permissive fallback, browser invocation, or generated artifact delta
exists. C04 report tests continue to use historical helper/manifest hashes;
separate C05 current-input freeze evidence may change.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate fri08_c05_adapter_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate fri07_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate fri06_c08r_final_lineage_
CARGO_NET_OFFLINE=true just verify-generator
cargo fmt --check
```

**Commit:** `feat(parity): lower finite grid composition inputs`

### 4.3 `P01/I08/S01/C05/T03` Add Exact Sources And Document The Boundary

**Owned files:** exactly the ten new HTML files listed below;
`tests/layout/browser_parity/corpus.toml`; focused input-inventory tests in
`tests/layout/browser_parity.rs` and test-only generator characterization in
`tests/bin/surgeist-layout-generate/generator.rs`; and `README.md`. Layout and
generator production, helper/parser, existing HTML, XML, report, dependencies,
features, scripts, and all other docs remain fixed.

**Exact new sources:**

1. `grid/fri08_auto_placement_span_after_occupied.html`
2. `grid/fri08_explicit_overlap_no_implicit_growth.html`
3. `grid/fri08_fit_content_flex_composition.html`
4. `grid/fri08_template_areas_explicit_tracks.html`
5. `grid/fri08_auto_fit_occupied_track_collapse.html`
6. `grid/fri08_stretch_minmax_auto.html`
7. `grid/fri08_duplicate_line_name_token.html`
8. `grid/fri08_grid_composition.html`
9. `grid-lanes/fri08_nested_indefinite_subgrid.html`
10. `subgrid/fri08_standalone_intrinsic_composition.html`

**Outcome:** Author each source with ordinary CSS for the browser oracle and
explicit finite layout-ready facts for the existing adapter. Add exactly one
active manifest case per source. Adopt these eight existing controls without
editing them: `grid_overflow_inline_axis_scroll`, all three specified
`grid_lanes_*content*` sources, `subgrid_overflow_hidden_does_not_prohibit`, both
specified sibling-footer sources, and
`subgrid_standalone_axis_column_autoflow`. Document layout-owned normalized grid
facts and root-owned CSS lowering without claiming broad Level 2/3 support.

**Required RED prefix:** `fri08_c05_inputs_` first fails on the missing exact
ten-source inventory/manifest rows. GREEN proves exactly 1,448 HTML inputs;
exactly eighteen owned sources; four canonical output variants each; exactly 72
unique prospective rows; one helper and base-style reference per new source;
one `test-root`; required behavior fragments; finite adapter parse and public
layout for synthetic serialized inputs; and fixture-name independence. New XML
paths must remain absent.

**Acceptance:** The manifest summary remains 5,736 generated and the report/XML
remain byte-identical to base. No FRI-08 expected-fail, quarantine, unsupported,
or failure row is added. README assigns authored CSS/adapters/root integration
correctly. Record new helper and manifest hashes for C06.

**Verification:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate fri08_c05_inputs_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate fri08_c05_adapter_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just taffy-check
cargo fmt --check
```

**Commit:** `test(parity): settle FRI-08 browser inputs`

## 5 Cycle Completion Gate

After T03 is task-clean, follow the canonical status-only `complete` transition,
then run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c05_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri08_c0
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

Run the eight existing-control parity filters read-only. Do not run parity for
the ten new sources because their XML does not exist until C06. Build the exact
prospective 72-row set from the eighteen source identities and four canonical
variants; require uniqueness and require exactly the 32 existing-control XML
paths present and 40 new-source XML paths absent.

Verify base-to-HEAD scope; no new allow/expect; no dependency/feature/MSRV,
browser, XML, report, existing-HTML, script, generator-production, or later-
owned negative-control delta; and the canonical tracked/non-ignored owned-Rust
unsafe scan. Recompute 1,448 HTML, 5,736 XML, report SHA/buckets, current helper
and manifest hashes, and exact new-source aggregate. Confirm every XML is still
comment-free and the report is the only provenance authority.

`just corpus-check` is not an acceptance command in C05: its expected failure
must identify stale helper, manifest, or new-source lineage and must not mutate
files. Any other failure is a blocker. C05 is publication-ready only when all
three ordered task ranges are clean, the exact public composition and finite
adapter evidence pass, the immutable 72-row input set is settled, the full
non-lineage gate passes, and a fresh holistic review of the exact cycle range
returns `CLEAN`. Follow the canonical publication/readback/cleanup gate. C06
receives the immutable production/helper/adapter/HTML/manifest inputs and alone
owns exactly one unfiltered full ExistingPinned generation.
