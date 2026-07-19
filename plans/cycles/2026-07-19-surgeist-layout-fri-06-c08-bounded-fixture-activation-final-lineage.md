# FRI-06-C08 Bounded Fixture Activation And Final Lineage

Status: reviewed

Cycle ID: `FRI-06-C08`

Owning repository: `surgeist-layout`

Cycle base: `bcdba3c49be09ad119c03ecdc4c77da803159132`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`7090ea13ba7d9e524ce432018c8b7c44c1b3b76428d2c666949d297656ce97c8`,
commit `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`: `FRI-06.4 D-16`, browser,
fixture, and artifact portions of `FRI-06.9` and `FRI-06.10`, all of
`FRI-06.11`, and artifact acceptance portions of `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at SHA-256
`c442d1176e66d7d54999e272c1627cd63531e702a7662eefc031528b7b5c079e`,
commit `42cd301e2fb6eeabba30be381dfbfa8561dd9c3e`, entry `FRI-06-C08` and
the Activation Recovery Evidence matrices.

## Outcome

Activate the exact 340 existing FRI-06 variants and add the exact twelve named
sources and 48 variants through bounded fixture/helper/parser/serializer facts.
After every input is reviewed and frozen, derive one full unfiltered corpus with
the existing pinned browser and verify the exact 388-row FRI-06 matrix, final
report accounting, and semantic preservation of every pre-existing output.

## Boundary

FRI-06-C07 is published and remotely verified at the cycle base. Its production
behavior and C06's finite Rust adapter are stable. C08 owns only fixture inputs,
the narrow browser helper and generator data serializer needed to express those
inputs, the manifest, the derived XML/report replacement, and focused parity
evidence. Production `src/` is read-only.

The immutable entry inputs are:

- full report `tests/layout/browser_parity/xml/generation-reports/all.json`,
  SHA-256 `4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`;
- manifest `tests/layout/browser_parity/corpus.toml`, SHA-256
  `bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`;
- helper `tests/layout/browser_parity/scripts/gentest/test_helper.js`, SHA-256
  `b7615b6783d3cf76ec0953533e409b9f1d5a348712fb2dd367cfc730e497584a`;
- base style, SHA-256
  `5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6`;
  it remains byte-identical; and
- existing-pinned Chrome for Testing `149.0.7827.115` at the manifest-owned
  repository-relative executable under `target/surgeist-browser`.

Matrix digests are SHA-256 over sorted LF-terminated
`source<TAB>variant` rows. The four standard variants are `border_box_ltr`,
`content_box_ltr`, `border_box_rtl`, and `content_box_rtl`.

| Matrix | Fixed membership | Rows | Digest |
| --- | --- | ---: | --- |
| Existing activation | Entry-report rows whose reason is exactly `Unsupported mixed text/element content`, `Unsupported vertical <br> line-break semantics`, or `Unsupported <br> outside block inline-run semantics` | 340 across 85 sources | `2df58c8127c8567a93b21cec2713e1b7ebb7541d8dce19df6d401bf442ae4375` |
| New source | All four variants of the twelve `FRI-06.11` source paths | 48 | `17e19f30a6b4f2a97880dc090dc056fac2f5a061679768891f12cccf026261b7` |
| Activation union | Existing activation plus new source | 388 | `3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1` |
| Fixture correction | The exact sequence predicate | 256 | `35dc887d32232c365e132f38032021ae0b64147480ab7536971765b3fa5d0214` |
| Baseline helper | The sequence's four named baseline sources, all variants | 16 | `f9ac335e450b4ffd014ae91ef211e699b513676711f70e2c27414fb64f7455a3` |
| Semantic preservation | Four named `block_br_*inline_block_metrics` sources, all variants | 16 | `ff3b0c67a33ed008235891b3019e4491783fd7933a37c7b50589fec6b573a8b1` |
| Base generated | Every entry-report generated `source`, `variant`, `output` tuple | 5,324 | `3381162173bc2c09bbbae736391d9420c5e96c375083fb9fd0b337bcec12cffb` |

The exact three starting reasons own all existing membership. The helper may
ignore indentation whitespace between inline-display children when their parent
establishes grid layout, but may not suppress significant inline whitespace or
general mixed content. New shaped/control/fragment and finite shape-band output
is explicit fixture opt-in on matrix sources; it is never inferred corpus-wide.

The 240 vertical/outside-block break rows receive layout-ready control metrics
without changing authored CSS geometry. The fixture-correction and baseline
helper matrices bind every other correction. A worker may not add a source,
variant, reason, expected failure, quarantine, or fallback after seeing output.

Generator architecture, acquisition paths, browser policy, launch profile, base
style, dependencies, features, lockfile, MSRV, root, siblings, production code,
public API, docs, task runner, and later-owned behavior are non-goals. Changes in
`tests/bin/surgeist-layout-generate/generator.rs` are allowed only for the narrow
JSON/parser/serializer fields required by the reviewed fixture schema or a
confirmed genuine bug. No new generator layer, reusable parser, alternate line
algorithm, text shaper, bidi engine, CSS parser, or shape engine is permitted.

During T1 and T2, a scoped existing-pinned generation may be used only to
diagnose a concrete input defect that focused tests cannot localize. It must use
the narrowest matching source or prefix, produces no report, is never acceptance
evidence, and leaves no committed XML. It is optional and is not a task command.
Never rerun unchanged inputs. After T1 and T2 are task-clean, T3 runs exactly one
full unfiltered existing-pinned regeneration. If that attempt proves a genuine
input defect, its lineage is invalid: discard its artifacts, repair and review
the input, then run one replacement full regeneration after the new inputs
settle. There is never more than one valid full lineage.

## Impacts

- **Public API and production behavior:** unchanged.
- **Dependencies, features, lockfile, MSRV, and browser policy:** unchanged.
- **Generated artifacts:** all XML and the full report are replaced only by T3's
  valid full lineage; no artifact is hand-edited.
- **Docs/examples and root follow-up:** unchanged in C08; C09 owns public and
  handoff evidence.
- **Unsafe and lint policy:** no executable unsafe and no new `allow` or `expect`
  attribute in tracked or non-ignored owned Rust.

## Tasks

### `C08-T1` Settle Existing Activation Inputs

**Files/area:** the exact 85 entry-report HTML sources; helper
`scripts/gentest/test_helper.js`; narrow helper JSON/XML serialization in
`tests/bin/surgeist-layout-generate/generator.rs`; and focused generator tests.
The Rust fixture adapter and all XML/report files are read-only.

**Outcome:** Replace the three exact unsupported classifications with explicit,
layout-ready shaped text, atomic/control, fragment, and break facts for all 340
existing rows. Correct the baseline subset's grid-parent indentation
classification and the 240 break rows' finite metrics without changing browser
geometry or any source outside the 85-source predicate.

**RED:** Add `fri06_c08_existing_` helper/serializer and matrix tests proving the
entry helper rejects the three supported families, misclassifies the baseline
indentation, lacks the required finite output facts, or cannot serialize them to
the C06 schema. Tests reconstruct the exact 85-source/340-row membership and
digest and the baseline-helper subset's exact four sources, 16 rows, and digest.

**Acceptance:** Every source is selected only by the pinned report predicate.
Helper output uses DOM ranges and computed geometry only for explicit fixture
facts, preserves stable source/segment/child identity, validates complete finite
tuples, and retains the exact unsupported response for unmarked mixed content.
The four missing-root sources remain untouched. Base style is byte-identical.
No XML/report file changes and no full generation occurs.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_existing_
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate
CARGO_NET_OFFLINE=true ./scripts/run-cargo-task.sh generator-clippy
cargo fmt --check
```

**Dependency:** published C07 production candidate and C06 adapter.

**Intended commit:** `test(parity): settle existing FRI-06 activation inputs`.

### `C08-T2` Add Twelve Finite FRI-06 Sources

**Files/area:** exactly the twelve `FRI-06.11` HTML paths, exactly twelve matching
`[[cases]]` records in `corpus.toml`, and focused manifest/helper tests. T1's
helper/parser may change only when a T2 RED proves one missing reviewed field.

**Outcome:** Author all 48 new variants for mixed text/atomic wrapping, unequal
line alignment, forced break strut, vertical break clear, atomic baselines and
percentage basis, bidi identity, line exclusion, BFC avoidance, float auto
height, logical clear, and finite shape bands.

**RED:** Add `fri06_c08_new_` tests that require the exact twelve paths, case IDs,
source-root ownership, active status, four-variant expansion, 48-row digest, and
the finite schema facts assigned to each source. Missing, duplicate, extra, or
cross-family facts fail before generation. Reconstruct the 256-row fixture-
correction predicate from the pinned entry report and the exact new-source rows;
assert its complete membership, count, and digest without checking the stale
derived report.

**Acceptance:** The twelve sources are exactly those listed in `FRI-06.11` and
the manifest replacement count is fixed to 5,712 generated and 16 unsupported.
The 16 new-source fixture-correction rows have the reviewed semantics in every
applicable direction; other variants use the same authored behavior rather than
variant-specific expected values. Shape data is a finite physical band table,
not CSS shape syntax. No XML/report file changes and no full generation occurs.
Record the replacement manifest SHA-256 in task evidence.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_new_
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets --features layout-golden-generate
CARGO_NET_OFFLINE=true ./scripts/run-cargo-task.sh generator-clippy
cargo fmt --check
```

**Dependency:** T1 task-clean; helper and serializer facts are stable first.

**Intended commit:** `test(parity): add bounded FRI-06 browser sources`.

### `C08-T3` Derive And Verify The Final Lineage

**Files/area:** generated XML, `xml/generation-reports/all.json`, and focused
matrix/parity tests in `tests/layout/browser_parity.rs`. All T1/T2 inputs are
read-only at their task-clean commits.

**Outcome:** Run the one full unfiltered existing-pinned derivation and prove its
report, provenance, fixed activation comparisons, and pre-existing semantics.

**Acceptance:** Before generation, verify the exact cached executable exists and
reports `Google Chrome for Testing 149.0.7827.115`; both cache/version override
variables and the generation filter are absent. Run the following command once:

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

The report has `filter: null`, 5,712 generated, exactly 16 unsupported carrying
only `Unsupported missing #test-root fixture root`, and zero expected-fail,
quarantined, or failed-to-generate entries. It records the exact browser, launch
profile, replacement helper/manifest hashes, and no scoped report. Add a focused
test that reconstructs the 388-row union and compares every row through the C06
adapter and public layout front door. Reconstruct and assert the exact four-
source, 16-row semantic-preservation membership and digest, then prove those
rows' semantics. For all 5,324 base-generated outputs, compare parsed XML after
excluding only the provenance comment against the cycle-base blob; any semantic
delta blocks. Record the final report/helper/manifest hashes. Do not rerun the
generator after successful derivation.

**Commands after generation:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_
CARGO_NET_OFFLINE=true ./scripts/run-browser-parity-task.sh corpus-check
CARGO_NET_OFFLINE=true ./scripts/run-browser-parity-task.sh taffy-check
CARGO_NET_OFFLINE=true ./scripts/run-verification.sh default
CARGO_NET_OFFLINE=true ./scripts/run-verification.sh generator
cargo fmt --check
```

**Dependency:** T1 and T2 task-clean with all inputs frozen.

**Intended commit:** `test(parity): derive final FRI-06 browser lineage`.

## Completion

All three task ranges must be independently `CLEAN`. T3's input freeze must equal
the reviewed T1/T2 heads, and its generated changes must be one complete valid
lineage. There is no scoped report, hand-edited artifact, changed production
path, unexpected source, inherited-failure reclassification, or generator
architecture expansion.

Run the T3 command set, then:

```sh
git diff --check bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD
git diff --quiet -G'(^|[^.[:alnum:]_])(allow|expect)[[:space:]]*\(' bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD -- '*.rs'
test -z "$(git diff --name-only bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD -- src Cargo.toml Cargo.lock justfile README.md scripts plans/specs plans/sequences)"
test -z "$(git status --porcelain)"
```

Run the canonical fail-closed unsafe scan over every tracked or non-ignored owned
Rust file. Inspect the full changed-path inventory: only this cycle plan, the
exact T1/T2 input and focused-test paths, the complete generated XML replacement,
and `all.json` are allowed.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`bcdba3c49be09ad119c03ecdc4c77da803159132..cycle_head`. Rerun all read-only
checks on local `main`, publish the immutable cycle head to authority remote
`main` with a leased fast-forward, fetch/read back, and prove local `main`, its
tracking ref, `FETCH_HEAD`, and live remote `main` agree. Remove every temporary
resource.

The handoff is the published, frozen final browser lineage for C09's read-only
public evidence and leaf-candidate closure. Blocker: none.
