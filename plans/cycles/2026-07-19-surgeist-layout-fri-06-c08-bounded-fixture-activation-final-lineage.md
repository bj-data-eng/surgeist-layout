# FRI-06-C08 Bounded Fixture Activation And Final Lineage

Status: in_progress

Cycle ID: `FRI-06-C08`

Owning repository: `surgeist-layout`

Cycle base: `bcdba3c49be09ad119c03ecdc4c77da803159132`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`5947ebb3ad527489634319c10629e438c5ff0cad4fb7e32b4fdc9225f771b56f`,
commit `55b0ad29c6f082041a19bb3e2e2e102d8011e582`: `FRI-06.4 D-01`,
`D-04`, `D-09`, `D-11`, and `D-16`; metric-fragment, atomic-baseline,
physical-placement, browser-comparator, fixture, and acceptance portions of
`FRI-06.5`, `FRI-06.7`, `FRI-06.9` through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at SHA-256
`e63cf81408a8ac1c9ec193ec3e8c026c632940900b285d2325d3a3450ab00b7c`,
commit `d956d801fdb4e8f5bdc774b89e08182c37cc7cea`, entry `FRI-06-C08` and
the Activation Recovery Evidence matrices.

## Outcome

Correct the exact 352 failing rows from the retained diagnostic lineage without
changing the fixed 388-row union or expanding the generator. Freshly revalidate
every affected task, freeze all helper/source/parser/adapter/production inputs,
then derive one replacement full corpus and prove all 388 comparisons, final
5,712/16 accounting, and semantic preservation of the other 5,324 outputs.

## Boundary

FRI-06-C07 is published and remotely verified at the cycle base. C08 owns only
the bounded fixture sources and helper/parser/serializer/comparator facts, the
finite fixture adapter, two confirmed production corrections, focused evidence,
and the derived XML/report replacement. The public model and C06 adapter shape
remain unchanged.

The immutable entry inputs are:

- report `tests/layout/browser_parity/xml/generation-reports/all.json`, SHA-256
  `4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`;
- manifest `tests/layout/browser_parity/corpus.toml`, SHA-256
  `bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`;
- helper `tests/layout/browser_parity/scripts/gentest/test_helper.js`, SHA-256
  `b7615b6783d3cf76ec0953533e409b9f1d5a348712fb2dd367cfc730e497584a`;
- base style, SHA-256
  `5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6`;
  it remains byte-identical; and
- existing-pinned Chrome for Testing `149.0.7827.115` at the manifest-owned
  repository-relative executable under `target/surgeist-browser`.

The retained diagnostic lineage has report SHA-256
`69d6dc3764e0f119fcba3f6608bc51664d886f6279d18fdf23ac2b7f372e2ff4`,
`filter: null`, 5,712 generated, and the immutable 16 missing-root unsupported
rows. It is diagnostic evidence only: 36 of 388 public comparisons pass and 352
fail. Its generated XML/report remain read-only while T1/T2 correct and freeze
helper/source inputs; R0 then discards the complete invalid lineage before T3.

Exact membership is committed in
`plans/2026-07-19-surgeist-layout-fri-06-c08-public-comparison-census.tsv`,
SHA-256 `e972e8d67e32919ce736f6d5428f017fa9a61ec5112fa75b2ec5b9d43b53e4f5`,
commit `5fa1d0bc8b48649d1e0f7335260f8f42ee6d049e`. Matrix digests are SHA-256
over sorted LF-terminated `source<TAB>variant` rows.

| Partition | Census selector | Rows | Digest |
| --- | --- | ---: | --- |
| Input | `artifact.*`, `identity.*`, `later_owned.flow_root_display_normalization` | 314 | `060a024d38f4331a3aefe5971dc9db9a2a740e2a88aa7b5d02e41d0c735e73e2` |
| Comparator | `comparator.*` | 10 | `240c1679d8343049d7ab3343e34a173da20ef49e03bfb45fa2d46a5e97d1a641` |
| Adapter | `adapter.*` | 24 | `efce04f838f358bda851df2b723c01c1c51b6247c39c662ffdc8e5fbd3f12aa2` |
| Production | `production.*` | 4 | `7b4fc8b3bb27f912d3f39d2aadc05c243ead274fed54c20dfa43bd0825f7c61f` |
| Passing control | `pass` | 36 | `97177ac281f2908dc5bcda26ef984100d7f36c67e458aa8d5b05a8c75ac59fa4` |
| Activation union | Every census data row | 388 | `3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1` |

The exact existing activation, new-source, semantic-preservation, and 5,324
base-generated memberships remain those pinned by the sequence. No worker may
add a source, variant, category, expected failure, quarantine, or fallback after
seeing output.

Generator architecture, acquisition, browser policy, launch profile, base style,
dependencies, features, lockfile, MSRV, root, siblings, public API, task runner,
and later-owned behavior are non-goals. Generator changes are allowed only for
the reviewed finite parser/serializer fields, focused fixtures, or a confirmed
genuine bug. No reusable parser, alternate line algorithm, text shaper, bidi
engine, CSS parser, display normalizer, or shape engine is permitted.

Scoped generation remains an optional diagnostic during implementation. It is
never acceptance evidence and leaves no retained output. No correction task runs
full generation. After every changed input is task-clean and frozen, T3 runs one
full unfiltered replacement derivation. No unchanged-input retry is permitted; a
failed run stops for a newly reviewed correction lineage.

## Impacts

- **Public API:** unchanged.
- **Production behavior:** C08-R1 retains the mixed-line RTL correction and adds
  only the exact four float-line final-height rows.
- **Dependencies, features, lockfile, MSRV, and browser policy:** unchanged.
- **Generated artifacts:** all XML and the full report are replaced only by T3's
  valid lineage; no artifact is hand-edited.
- **Docs/root:** only the reviewed planning and census evidence changes; C09 owns
  public evidence and handoff closure.
- **Safety:** no executable unsafe and no new `allow` or `expect` attribute in
  tracked or non-ignored owned Rust.

## Tasks

### `C08-T1` Settle Existing Activation Inputs

**Files/area:** the exact 85 entry-report HTML sources; helper
`scripts/gentest/test_helper.js`; narrow JSON/XML serialization in
`tests/bin/surgeist-layout-generate/generator.rs`; focused generator tests.

**Outcome:** The complete T1 range replaces the three exact unsupported families
with explicit layout-ready facts for all 340 existing rows and corrects every
input-census row on those sources. Atomic participation/break/visual facts derive
from computed/lowered atomic role after blockification; Range source order never
becomes visual order. Preserve stable identity, significant whitespace, the four
missing-root sources, and byte-identical base style.

**RED:** At the T1 task base, the focused command exits 101 because the helper
still classifies the exact activation families as unsupported, lacks finite
tuples/schema, and treats authored-inline/computed-block children as atomic. Tests
bind the exact 85-source/340-row digest, baseline subset, census intersection, and
a blockification negative control before implementation.

**GREEN/acceptance:** The same focused command passes with complete finite tuples,
parser serialization, and the 5,324-output preservation predicate without reading
a replacement report. No XML/report change or generation belongs to T1. Freshly
re-review T1's complete ordered range against this revised plan before T2/R0.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_existing_
CARGO_NET_OFFLINE=true just generator-check
CARGO_NET_OFFLINE=true just generator-clippy
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** published C07 production candidate and C06 adapter.

**Intended commit:** `test(parity): settle existing FRI-06 activation inputs`.

### `C08-T2` Add Twelve Finite FRI-06 Sources

**Files/area:** exactly twelve `FRI-06.11` HTML paths, twelve matching manifest
records, and focused manifest/helper tests. T1 helper/parser changes only when a
T2 RED proves one missing reviewed field.

**Outcome:** The complete T2 range authors the exact 48 variants for mixed text,
atomic wrapping/baselines/percentage basis, unequal line alignment, forced break,
vertical clear, bidi identity, line exclusion, BFC avoidance, float auto height,
logical clear, and finite shape bands, and corrects every input-census row on
those sources. BFC fixtures use supported overflow rather than `flow-root`.

**RED:** At the T2 task base, the focused command exits 101 because the twelve
sources, manifest records, 48-row expansion, and assigned finite facts do not yet
exist. Tests bind exact paths, IDs, ownership, four-variant digest, and accounting.

**GREEN/acceptance:** The same focused command passes with active status, exact
48-row facts, 5,712/16 replacement accounting, and no cross-family facts without
reading a replacement report. No XML/report change or generation belongs to T2.
Freshly re-review T2's complete ordered range before R0.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_new_
CARGO_NET_OFFLINE=true just generator-check
CARGO_NET_OFFLINE=true just generator-clippy
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T1 freshly task-clean under this revised plan.

**Intended commit:** `test(parity): add bounded FRI-06 browser sources`.

### `C08-R0` Correct Exact Input And Comparator Rows

**Files/area:** narrow parser/comparator code and focused tests; three stale
accounting tests; and complete cleanup of the retained invalid XML/report lineage.
Frozen T1/T2 helper/source inputs, production, and the 24 adapter rows are
read-only.

**Outcome:** Reconcile the frozen exact 314 input corrections supplied by T1/T2:
252 computed-role atomic markers, 38 non-visual Range identities, 20 supported-
overflow BFC variants, and four shaped-text identities. Correct exactly six
unrounded Range advances and four browser `<br>` observations. Range facts retain
source, line, physical flow-inline start, and unrounded advance only; browser
Range order supplies no visual index and `<br>` ink no model control geometry.

**RED:** Census-bound tests prove all exact selectors/digests and the frozen 314
input rows, then expose each old parser/comparator behavior with finite controls.
The three stale accounting tests reconstruct entry or census membership rather
than consulting the replacement report. Restore tracked XML/report to the cycle
entry and remove every untracked diagnostic XML file; preserve the census.

**Acceptance:** Exact 314/10 membership passes with no spill into adapter,
production, or 36 controls. Explicit model fragment/control expectations remain
strict. Record frozen helper, manifest, and source-set hashes. No XML/report
change and no full generation remain. Append corrections to the existing R0 task
range, then freshly re-review R0 and the complete T1/T2 ranges in the composed
parser/comparator state; prior clean verdicts do not carry.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_existing_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_new_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_range_ink_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_range_ink_
CARGO_NET_OFFLINE=true just generator-check
CARGO_NET_OFFLINE=true just generator-clippy
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T1 and T2 freshly task-clean under this revised plan, and the
committed census. R0's composed changes then require the named post-R0 re-reviews.

**Intended commit:** `test(parity): correct bounded FRI-06 observations`.

### `C08-R1` Close Exact Production Rows

**Files/area:** `src/inline.rs`, `src/block.rs`, and focused public-compute tests.
All fixture, helper, adapter, artifact, and unrelated production paths are
read-only.

**Outcome:** Retain the reviewed mixed shaped-text/atomic RTL placement correction
and correct only the four `fri06_float_line_exclusion` final-height rows, where
public layout returns 62 instead of browser 63. Preserve LTR, all-atomic RTL,
baseline/top/bottom selection, C07 boundaries, and all other float geometry.

**RED:** Public-compute tests reproduce both mixed RTL box models and all four
float-line variants through production code. The float test isolates the first
incorrect final block geometry and retains horizontal/directional controls.

**Acceptance:** Exact source-equivalent regressions pass through public
`compute_layout`; applicable C07 and inline/bidi/writing-mode/float suites remain
green. Append the correction to the existing R1 task range and freshly re-review
its complete ordered range. No generator run or input/artifact change occurs.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c08_mixed_inline_rtl_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c08_float_line_
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just generator-check
CARGO_NET_OFFLINE=true just generator-clippy
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** R0, T1, and T2 freshly task-clean after the composed R0 changes.

**Intended commit:** `fix(layout): close remaining FRI-06 C08 geometry`.

### `C08-R2` Lower Exact Finite Adapter Rows

**Files/area:** `tests/layout/browser_parity/support.rs` and focused adapter tests
only. Helper/source/parser, production, generated artifacts, and public model are
read-only.

**Outcome:** For the exact 24 census rows, synthesize only 16 anonymous grid text
wrappers, four secondary inline boundaries, and four containing 20px struts. No
general display lowering, text shaping, bidi analysis, or fallback is added.

**RED:** Census-bound tests reconstruct exact membership/digest and finite
source-equivalent parsed fixtures reproduce each old adapter rejection. Negative
controls prove the adapter remains fail-closed outside those named structures.

**Acceptance:** All exact 24 rows lower through the existing private adapter shape
with stable identity and no change to other rows. Tests do not require retained
generated XML and no generation occurs.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_adapter_
CARGO_NET_OFFLINE=true just check
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** committed census and C06 finite adapter.

**Intended commit:** `test(parity): lower finite FRI-06 adapter forms`.

### `C08-T3` Derive And Verify The Final Lineage

**Files/area:** generated XML, `xml/generation-reports/all.json`, and focused
matrix/parity/accounting tests. Every reviewed input and production path is
read-only at its frozen task-clean commit.

**Outcome:** Record stale-entry RED, run one full unfiltered existing-pinned
derivation, then prove provenance, exact accounting, all 388 public comparisons,
and preservation of the other 5,324 outputs.

**RED:** With entry artifacts restored, `fri06_c08_` lineage/accounting tests fail
only for stale 5,324/356 report accounting and absent 388 outputs. Fixed matrix,
cycle-base semantic, and input-freeze tests pass before generation.

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08_
```

**Preflight:** Confirm the manifest-owned Chrome executable reports exactly
`149.0.7827.115`, no browser cache/version/filter override is set, every task is
freshly clean, changed helper/source/parser inputs equal the recorded freeze, and
the worktree contains entry artifacts rather than the invalid diagnostic lineage.

**Preflight commands:**

```sh
set -e
test -x 'target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
browser_version_file="$(mktemp)"; trap 'rm -f "$browser_version_file"' EXIT
'target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' --version > "$browser_version_file"
test "$(wc -l < "$browser_version_file")" -eq 1
grep -Eq '^Google Chrome for Testing 149\.0\.7827\.115[[:blank:]]*$' "$browser_version_file"
test -z "${SURGEIST_BROWSER_CACHE+x}${SURGEIST_BROWSER_VERSION+x}${SURGEIST_LAYOUT_GENERATE_FILTER+x}${SURGEIST_LAYOUT_BROWSER_PARITY_ROOT+x}"
```

**Generation:** Run exactly once. A nonzero exit or unexpected output stops for a
new reviewed correction lineage; never rerun unchanged inputs.

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_GENERATE_FILTER -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

**Acceptance:** The report has `filter: null`, 5,712 generated, exactly 16
unsupported rows containing only `Unsupported missing #test-root fixture root`,
and zero other buckets. It records exact browser, launch, helper, and manifest
provenance with no scoped report. Every exact census row compares through the C06
adapter and public layout front door. The 16 semantic-preservation rows match
cycle-base semantics, and all 5,324 base-generated XML documents differ only in
the provenance comment. Record report/helper/manifest hashes and artifact digest.

**Commands after generation:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c08_
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

**Dependency:** T1, T2, R0, R1, and R2 freshly task-clean with all inputs and
production frozen.

**Intended commit:** `test(parity): derive final FRI-06 browser lineage`.

## Completion

All six task ranges are independently `CLEAN`, including fresh complete reviews
of T1, T2, R0, and R1 after their invalidating changes. T3's input and production
freeze equals those reviewed heads, and its generated changes form one complete
valid lineage. There is no scoped report, hand-edited artifact, unexpected
source, inherited-failure reclassification, or generator architecture expansion.

Run the full T3 command set and verify the exact cycle range has no whitespace
errors, executable unsafe, new Rust `allow`/`expect`, dependency/feature/task-runner
change, or path outside the reviewed planning, census, fixture/helper/parser/
comparator, adapter, production regression, generated XML, and report inventory.
Record the tracked/non-ignored owned-Rust manifest count and Clippy
`-F unsafe-code -D warnings` evidence. Require a clean worktree.

```sh
git diff --check bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD
git diff --name-only -z bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD | while IFS= read -r -d '' file_path; do case "$file_path" in plans/2026-07-19-surgeist-layout-fri-06-c08-public-comparison-census.tsv|plans/cycles/2026-07-19-surgeist-layout-fri-06-c08-bounded-fixture-activation-final-lineage.md|plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md|plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md|src/block.rs|src/block_tests.rs|src/inline.rs|src/root_tests.rs|tests/bin/surgeist-layout-generate/generator.rs|tests/layout/browser_parity.rs|tests/layout/browser_parity/corpus.toml|tests/layout/browser_parity/support.rs|tests/layout/browser_parity/scripts/gentest/test_helper.js|tests/layout/browser_parity/xml/generation-reports/all.json|tests/layout/browser_parity/xml/*.xml) ;; tests/layout/browser_parity/html/*) rg -q -F "$(printf '\t%s\t' "${file_path#tests/layout/browser_parity/}")" plans/2026-07-19-surgeist-layout-fri-06-c08-public-comparison-census.tsv || exit 1 ;; *) exit 1 ;; esac; done
test -z "$(git diff --unified=0 bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD -- '*.rs' | rg --pcre2 '^\+(?!\+\+\+).*#\s*\[\s*(allow|expect)\s*\(')"
test -z "$(git diff --name-only bcdba3c49be09ad119c03ecdc4c77da803159132..HEAD -- Cargo.toml Cargo.lock Justfile README.md)"
owned_rust_manifest="$(mktemp)"; trap 'rm -f "$owned_rust_manifest"' EXIT
git ls-files -co --exclude-standard -z -- '*.rs' ':(exclude)target/**' ':(exclude)vendor/**' > "$owned_rust_manifest"
test -s "$owned_rust_manifest"; tr '\0' '\n' < "$owned_rust_manifest" | wc -l
xargs -0 sh -c 'rg -n --pcre2 "#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:\"[^\"]*\")?\s*\{" -- "$@"; status=$?; test "$status" -eq 1' sh < "$owned_rust_manifest"
CARGO_NET_OFFLINE=true just clippy
CARGO_NET_OFFLINE=true just generator-clippy
test -z "$(git status --porcelain)"
```

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`bcdba3c49be09ad119c03ecdc4c77da803159132..cycle_head`. Rerun all read-only
checks on local `main`, publish the immutable cycle head to authority remote
`main` by leased fast-forward, fetch/read back, and prove local `main`, its
tracking ref, `FETCH_HEAD`, and live remote `main` agree. Remove every temporary
resource.

The handoff is the published, frozen final browser lineage for C09's read-only
public evidence and leaf-candidate closure. Blocker: none.
