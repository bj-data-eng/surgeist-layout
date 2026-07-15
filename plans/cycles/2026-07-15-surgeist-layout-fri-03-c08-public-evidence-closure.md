# FRI-03-C08 Public Surface, Evidence, And Initiative Closure
Status: reviewed
Cycle ID: `FRI-03-C08`
Owning repository: `surgeist-layout`
Cycle base: `2c0a396bcb9299bd77fb33981d08b0a7c0244eb8`

Reviewed specification: `plans/specs/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `6ca195b4ba560ae49bc6963176234f8494cfb50a91674f6dcec358d19fa9769c`,
commit `52d87a75751f9987251ec2fdf8200e75eba3e17b`, sections `FRI-03.5`,
`FRI-03.7` through `FRI-03.13`, and all acceptance items in `FRI-03.14`.

Reviewed sequence: `plans/sequences/2026-07-15-surgeist-layout-fri-03-box-participation-contracts.md`
at `d59317e1b80337ff4041a034c062867dc7e744048eb7047d2b2e7b412aea130a`,
commit `03e7582565fa2d4f3aa7f71973f6dfebe273c4fb`, entry `C08`.

C07 handoff: candidate `2c0a396bcb9299bd77fb33981d08b0a7c0244eb8`
was pushed to and read back from `origin/main`; local, tracking, and observed
remote `main` were equal and clean. C01-C07 now implement every FRI-03-owned
production branch.

Bounded outcome: one exact nonignored 32-output browser gate and the public
README/rustdoc describe the completed order, source identity, replacedness, and
containing-context model. Read-only evidence seals the already-derived corpus,
source absence, finding closure, and root handoff as one publishable leaf
candidate.

## Boundary
This cycle may change only `tests/layout/browser_parity.rs`, `README.md`, and
crate-level `//!` documentation at the start of `src/lib.rs`. The implementation
allowlist is exactly those three files.

C02 candidate `127f20b4450e2196b768e78e0c97006e7ea0fc84` already performed the
single final full ExistingPinned regeneration after its HTML, parser, and fixture
inputs settled. Its checked-in state contains 1,406 HTML sources, 5,268 XML
outputs, only `generation-reports/all.json`, a 5,268 generated / 356 unsupported
summary with zero failure buckets, current provenance, and unsupported tuple SHA
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`.
C08 verifies that state read-only. It must not run full or scoped generation or
browser capture. Scoped generation remains an optional diagnostic only when
iterating on HTML/parser/fixture changes; C08 has no such changes.

HTML, XML, `corpus.toml`, reports, browser-parity operational documentation,
parser/support code, helper, generator, scripts, Justfile, manifests, lockfile,
production algorithms, public signatures, root, and siblings are read-only.
Generator architecture expansion, report rewriting, hand-edited XML, new
dependencies/features, MSRV changes, root integration, and the ignored aggregate
parity corpus remain out of scope. Owned Rust remains free of `unsafe`.

Impacts: API and behavior - unchanged; dependencies/features/MSRV - unchanged;
artifacts - unchanged and read-only; docs - complete the public participation
contract; root - archival handoff only; unsafe - none.

## Tasks
### C08-T1 - Seal The Exact FRI-03 Browser Union
File: `tests/layout/browser_parity.rs` only.
Dependencies: published/read-back C01-C07, the immutable cycle base, and C02's
checked-in full-derivation artifacts. T2 waits for T1 task-clean.

Outcome: add one exact topology/membership test and one nonignored comparison
test for the 32-output FRI-03 union: four block-margin outputs, 12 item-order
outputs, and 16 flex-item-root parent-axis outputs. The matrix rejects a missing,
duplicate, misplaced, or extra output. The comparison parses and runs exactly
that union through the real browser-parity layout front door without claiming
`runs_all_checked_in_browser_parity_xml`.

RED evidence: the base test inventory has no `runs_fri_03_` test and no single
matrix that names the exact 32-output union. This is a closure-evidence gap over
already-implemented behavior, so no algorithm failure or artificial fixture
change is required.

Acceptance:
- `fri_03_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_outputs`
  proves the exact eight-source/four-variant topology and all three owned groups;
- `runs_fri_03_box_participation_against_surgeist_layout` runs exactly 32 unique
  checked-in XML outputs and all comparisons pass;
- the block nested-child `y=1`, order-source topology, and flex-item-root parent
  axes remain observable rather than being replaced by path-only assertions;
- the full report/count/inventory tests remain green and the ignored aggregate
  parity test remains ignored and visible; and
- no fixture, parser, report, corpus metadata, generator, or production source
  changes.

Commands:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::fri_03_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_outputs -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::runs_fri_03_box_participation_against_surgeist_layout -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::browser_parity_generation_report_counts_full_scope -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::browser_parity_generation_report_inventory_is_full_only -- --exact
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```

Commit: `test(parity): seal FRI-03 browser evidence`.

### C08-T2 - Document The Completed Participation Contract
Files: `README.md` and crate-level `//!` documentation in `src/lib.rs` only.
Depends on: C08-T1 task-clean.

Outcome: public docs explain that `ItemOrder` is a layout-ready signed value,
`SourceIndex` is stable source identity, flex/grid/grid-lanes alone consume the
stable order-modified traversal, and outputs remain source-associated. They
explain the independent replaced fact and its owned block/root, flex, grid, and
grid-lanes behavior; the complete `ContainingLayoutContext`; explicit flex-parent
axes; cache identity; and root ownership of CSS lowering, box-generation facts,
invalidation, consumer migration, facade composition, and API artifacts.

RED evidence: neither public document currently states that complete
participation model or its root ownership handoff. This is a documentation
contract RED; no artificial behavior test is required.

Acceptance:
- README and crate rustdoc agree with the source and `FRI-03.5`/`FRI-03.10`;
- replacedness remains distinct from table role, measurement, aspect ratio, and
  explicit stretch;
- no compatibility alias, old output-order name, bare-flow leaf constructor, or
  inferred flex-parent axes are documented or added;
- only crate-level comments change in `src/lib.rs`; all reexports and public
  signatures remain identical to the cycle base; and
- doctests, rustdoc, deterministic documentation checks, and source absence
  checks pass.

Commands:
```sh
rg -n 'ItemOrder|SourceIndex|ContainingLayoutContext|ParentFormattingContext|item_is_replaced' README.md
rg -n '^//!.*(ItemOrder|SourceIndex|ContainingLayoutContext|ParentFormattingContext|item_is_replaced)' src/lib.rs
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
! rg -n 'NodeOutputOf::order|NodeOutputOf::with_order|pub order:|\bfn with_order\s*\(' src/output.rs README.md src/lib.rs
rg -n 'pub fn leaf_(layout|content_size)' src/output.rs
! rg -U --pcre2 'pub fn leaf_(?:layout|content_size)\([^)]*\bFlowAxes\b' src/output.rs
! rg -U --pcre2 'leaf_(?:layout|content_size)[\s\S]{0,240}\bFlowAxes\b|\bFlowAxes\b[\s\S]{0,240}leaf_(?:layout|content_size)' README.md src/lib.rs
! rg -n 'TODO|TBD|FIXME|\?\?\?' README.md src/lib.rs
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

Commit: `docs: close FRI-03 participation contract`.

## Completion
Run the four focused T1 commands, then:

```sh
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
find tests/layout/browser_parity/html -type f -name '*.html' | wc -l
find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l
find tests/layout/browser_parity/xml/generation-reports -type f -name '*.json' -print
jq -e '.summary.generated == 5268 and .summary.unsupported == 356 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0' tests/layout/browser_parity/xml/generation-reports/all.json
jq -S '.unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)' tests/layout/browser_parity/xml/generation-reports/all.json | shasum -a 256
! rg -n 'NodeOutputOf::order|NodeOutputOf::with_order|pub order:|\bfn with_order\s*\(' src/output.rs README.md src/lib.rs
rg -n 'pub fn leaf_(layout|content_size)' src/output.rs
! rg -U --pcre2 'pub fn leaf_(?:layout|content_size)\([^)]*\bFlowAxes\b' src/output.rs
! rg -U --pcre2 'leaf_(?:layout|content_size)[\s\S]{0,240}\bFlowAxes\b|\bFlowAxes\b[\s\S]{0,240}leaf_(?:layout|content_size)' README.md src/lib.rs
rg -n 'item_order' src/flex.rs src/grid
rg -n 'item_is_replaced' src/block.rs src/compute.rs src/flex.rs src/grid
rg -n 'ContainingLayoutContext|ParentFormattingContext|parent_flow_axes' src
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
git diff --check 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8
git diff --name-only 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8
git diff --exit-code 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8 -- tests/layout/browser_parity/html tests/layout/browser_parity/xml tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/README.md tests/layout/browser_parity/support.rs tests/bin scripts Cargo.toml Cargo.lock justfile
git status --short
```

The count commands must report 1,406 HTML and 5,268 XML; the report command must
print only `all.json`; the unsupported digest must equal the recorded SHA; and
both negated scans must find no executable unsafe or old output-order surface.
`verify-generator` compiles, tests, and lints the feature configuration but does
not generate artifacts. `corpus-check` is the read-only `check-corpus` path.

Cycle acceptance: both task ranges are independently clean; the complete cycle
is holistic clean; all commands pass at the exact head; `MODEL-001`, `CORE-005`,
and `BLOCK-007` have traceable specification, source, focused, browser, corpus,
docs, and absence evidence; and the immutable candidate is pushed to and read
back from authority `origin/main`.

After publication and readback, emit the complete canonical
`SURGEIST_HANDOFF: CRATE_CANDIDATE` payload with handoff ID
`surgeist-layout-fri-03-c08`. Record repository/crate, authority remote and URL,
branch, base/head/push/readback SHAs, commits, objective, reviewed spec/sequence/
plan revisions, both task ranges, behavior/final evidence, all review verdicts,
API/dependency/artifact/docs/unsafe effects, full publication proof, no brief,
and observations. Root actions must name CSS `order` parsing/lowering,
box-generation replacedness, flex-parent axes, invalidation, consumer rename,
facade, gitlink, integration tests, and API artifacts. Do not edit root.

Genuine blockers are artifact/provenance drift, a named FRI-03 failure, unsafe,
an out-of-bound change, or required generator/root expansion. A failed check or
review returns the plan to `in_progress`; it never authorizes regeneration.
