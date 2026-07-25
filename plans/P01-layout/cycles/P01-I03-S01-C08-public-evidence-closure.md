# P01-I03-S01-C08 Public Surface, Evidence, And Initiative Closure
Status: complete
Cycle ID: `P01/I03/S01/C08`
Owning repository: `surgeist-layout`
Cycle base: `2c0a396bcb9299bd77fb33981d08b0a7c0244eb8`

Reviewed specification: `plans/P01-layout/initiatives/P01-I03-box-participation-contracts.md`
at `9482b43c7b3bed5355fa438a353c103625ff032a311a10b1a5c90c7e4f199d0b`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`, sections `FRI-03.5`,
`FRI-03.7` through `FRI-03.13`, and all acceptance items in `FRI-03.14`.

Reviewed sequence: `plans/P01-layout/sequences/P01-I03-S01-box-participation-contracts.md`
at `db716f78093f71cc58daf3f1b889bce5687384948f8dbe0c22b1e2b533791518`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `C08`.

C07 candidate `2c0a396bcb9299bd77fb33981d08b0a7c0244eb8` is published and
read back from `origin/main`. C01-C07 implement every FRI-03 production branch.
Bounded outcome: correct one confirmed flex-item-root fixture phase bug, derive
the final corpus once, seal the exact 32-output comparison, document the public
model, and publish one independently reviewed leaf candidate.

## 1 Boundary
C08 owns only flex-item host-allocation capture/serialization/parsing/lowering,
its generated XML/report refresh, the exact FRI-03 parity gate, and public docs.
Allowed files are the embedded helper, existing generator and support modules,
generated `xml/` plus `all.json`, `tests/layout/browser_parity.rs`, `README.md`,
and crate-level `//!` comments in `src/lib.rs`.

Confirmed cause: fixture support gives parent viewport availability both to
`FlexItemRootContext` and to the root request. A reversible probe supplied the
browser-observed allocated parent-inline item size separately and changed all
selected 400/80/60 cases from 400/80/60 to browser 160/80/80; all 16 selected
cases and the public-request control passed. The probe was removed without a
commit. Expected geometry must never become fixture input.

HTML, `corpus.toml`, browser runtime/pin/profile, base style, operational README,
production algorithms, public signatures/reexports, dependencies/features,
MSRV, lockfile, Justfile/scripts, root, and siblings are read-only. No schema
version, source/output count, report kind, command, module, import, acquisition,
or generator architecture is added. No replaced fixture or hand-edited XML.

Scoped generation remains available as optional iteration diagnosis, but it is
never mandated or accepted as verification evidence. After T01 helper/parser
inputs settle, run the full ExistingPinned command exactly once. Every later
command is read-only; a failed check does not authorize repeating generation.
Owned Rust remains free of `unsafe`.

Impacts: public API/production behavior/dependencies/features/MSRV - unchanged;
fixture schema - strict additive field for flex-item roots; artifacts - one
generator-owned final refresh; docs - completed; root - handoff only.

## 2 Tasks
### 2.1 `P01/I03/S01/C08/T01` - Separate Flex Host Allocation And Derive Final Corpus
Files: `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity/support.rs`, and generated
`tests/layout/browser_parity/xml/` including `generation-reports/all.json`.
Dependencies: published C01-C07 and the corrected spec/sequence above. T02 waits
for T01 task-clean.

Outcome: flex-item viewport JSON captures `hostInlineSize` from root border-box
width/height selected by actual parent axes. XML requires finite non-negative
`host-inline-size="...px"` with both parent-axis attributes. Lowering keeps the
viewport as percentage context, makes only the parent inline physical host axis
definite at that value, and leaves the other host axis max-content. Root
viewports reject all three flex-item attributes.

RED: extend the existing helper/serializer/strict-schema tests first and add
`flex_item_root_separates_host_inline_allocation_from_viewport_context`; they
fail because capture/serialization/parsing/lowering lack the value. The existing
filtered comparison reproduces 400 versus 160 and 60 versus 80.

Acceptance: horizontal and non-square orthogonal capture select width/height;
zero/fractional values pass; missing, stray, non-pixel, non-finite, negative, or
partial metadata fails closed; request accessors prove host/context separation;
all 16 affected XML carry the attribute and compare; all 5,268 XML carry current
helper provenance; counts remain 1,406 HTML, 5,268 XML, one 5,268/356 report,
zero failure buckets, and unsupported tuple SHA
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`.

Commands, in order:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::bundled_helper_captures_exact_order_and_flex_parent_axes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::xml_generation_serializes_exact_order_and_parent_axes -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::support::tests::viewport_parent_axes_schema_is_strict -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::support::tests::flex_item_root_separates_host_inline_allocation_from_viewport_context -- --exact
CARGO_NET_OFFLINE=true just fmt-check
test -x 'target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
SURGEIST_PARITY_FILTER=grid_available_space CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --exact --ignored
SURGEIST_PARITY_FILTER=chrome_issue_325928327 CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --exact --ignored
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```
Commit: `fix(parity): separate flex host allocation`.

### 2.2 `P01/I03/S01/C08/T02` - Seal The Exact FRI-03 Browser Union
File: `tests/layout/browser_parity.rs` only. Depends on T01 task-clean; T03 waits.
Outcome: add a matrix test rejecting missing, duplicate, misplaced, or extra
owned paths and `runs_fri_03_box_participation_against_surgeist_layout`, which
parses and compares exactly eight source families times four variants.
RED: both names are absent; the prior attempted comparison exposed T01's now-fixed
input bug and created no commit.
Acceptance: 32 unique paths comprise four block-margin, 12 order, and 16
flex-item-root outputs; block nested `y=1`, order/source topology, explicit host
and parent context, and real layout comparison stay observable; aggregate parity
remains ignored and visible.
Commands:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::fri_03_fixture_matrix_rejects_missing_duplicate_misplaced_and_extra_outputs -- --exact
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout layout::browser_parity::runs_fri_03_box_participation_against_surgeist_layout -- --exact
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```
Commit: `test(parity): seal FRI-03 browser evidence`.

### 2.3 `P01/I03/S01/C08/T03` - Document The Completed Participation Contract
Files: `README.md` and crate-level `//!` comments in `src/lib.rs` only. Depends
on T02 task-clean.
Outcome: both docs explain typed item order/source identity, owning algorithms,
replaced behavior, complete containing context/cache identity, separate flex host
allocation/viewport context, and root-owned lowering/invalidation/facade/API work.
RED: neither document states the complete model. Acceptance: docs agree with
`FRI-03.5`/`.10`; no old alias/signature/fallback is added; reexports stay exact.
Commands:
```sh
rg -n 'ItemOrder|SourceIndex|ContainingLayoutContext|ParentFormattingContext|item_is_replaced' README.md
rg -n '^//!.*(ItemOrder|SourceIndex|ContainingLayoutContext|ParentFormattingContext|item_is_replaced)' src/lib.rs
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```
Commit: `docs: close FRI-03 participation contract`.

## 3 Completion
Rerun every task command except the preflight and generation command. Then run:
```sh
CARGO_NET_OFFLINE=true just fmt-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
CARGO_NET_OFFLINE=true RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p surgeist-layout
find tests/layout/browser_parity/html -type f -name '*.html' | wc -l
find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l
find tests/layout/browser_parity/xml/generation-reports -type f -name '*.json' -print
rg -l 'host-inline-size=' tests/layout/browser_parity/xml --glob '*.xml' | wc -l
jq -e '.summary.generated == 5268 and .summary.unsupported == 356 and .summary.expected_fail == 0 and .summary.quarantined == 0 and .summary.failed_to_generate == 0' tests/layout/browser_parity/xml/generation-reports/all.json
jq -S '.unsupported | map({name, source, variant, reason}) | sort_by(.name, .source, .variant, .reason)' tests/layout/browser_parity/xml/generation-reports/all.json | shasum -a 256
! rg -n 'NodeOutputOf::order|NodeOutputOf::with_order|pub order:|\bfn with_order\s*\(' src/output.rs README.md src/lib.rs
! rg -U --pcre2 'pub fn leaf_(?:layout|content_size)\([^)]*\bFlowAxes\b' src/output.rs
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' --glob '!target/**' .
git diff --check 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8
git diff --name-only 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8
git diff --exit-code 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8 -- src ':(exclude)src/lib.rs'
git diff --exit-code 2c0a396bcb9299bd77fb33981d08b0a7c0244eb8 -- tests/layout/browser_parity/html tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/README.md tests/layout/browser_parity/scripts/gentest/test_base_style.css tests/bin/surgeist-layout-generate.rs Cargo.toml Cargo.lock justfile scripts
git status --short
```
Counts must be 1,406 HTML, 5,268 XML, one `all.json`, and 16 host attributes;
the digest must equal the T01 SHA. The final head must be task-clean and
holistic-clean before immutable-SHA publication and fresh remote readback. No
final check may modify source or generated artifacts.

Emit canonical `SURGEIST_HANDOFF: CRATE_CANDIDATE` with ID
`surgeist-layout-fri-03-c08`: authority remote/URL, base/head/push/readback SHAs,
commits, reviewed revisions, all task ranges/evidence/reviews, effects, unsafe
and publication proof, and root actions for CSS order, box replacedness,
flex-parent axes, invalidation, consumer rename, facade, gitlink, integration
tests, and API artifacts. Do not edit root.

Genuine blockers are missing cached browser, artifact/provenance/count drift,
named FRI-03 failure, unsafe, out-of-bound change, or required architecture/root
expansion. A failed check/review returns to `in_progress`; it never authorizes a
second full derivation of the same settled inputs.
