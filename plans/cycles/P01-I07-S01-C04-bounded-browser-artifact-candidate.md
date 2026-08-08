# P01-I07-S01-C04 Bounded Browser And Artifact Candidate

Status: draft

Cycle ID: `P01/I07/S01/C04`

Owning repository: `surgeist-layout`

Cycle base: `5e1de7fe83326c4ec7c9b878fd39f4d3738fce5f`

Reviewed specification:
`plans/specs/P01-I07-flex-algorithm-completeness.md`, normalized semantic-content
SHA-256 `9e2a899476d27e09133a05531cb4bb4dfab1479d949d66167548c26ee1972b57`,
commit `451954e6aab6529ce7464c299be7e2aff6ea3753`: `FRI-07.4 D-15`
through `D-17`; complete `FRI-07.10`; and the fixture, adapter, generator,
artifact, browser, verification, root-handoff, and acceptance portions of
`FRI-07.11`, `.12`, `.14`, and `.16`.

Reviewed implementation sequence:
`plans/sequences/P01-I07-S01-flex-algorithm-completeness.md`, normalized
semantic-content SHA-256
`50f29d416df158ed8ceb799d4592fc2d53e36e6512a676bf498b05722a15b964`,
commit `0f0c19e50d8f6c27e300a5bc652e21ee5145b7cc`, entry
`P01/I07/S01/C04`.

Bounded outcome: add only the normalized flex-item-collapse fixture lowering,
its exact rejection and independence evidence, and the six specified
four-variant sources; freeze their active manifest records; then run one
unfiltered ExistingPinned derivation and adopt its exact 24-row browser lineage
without expanding generator architecture or changing production layout.

## 1 Boundary

The remotely verified C03 candidate at the cycle base is the immutable entry
state. FLEX-002 through FLEX-005 already compose through the public layout front
door, and README and crate documentation already state the normalized collapse
boundary. C04 owns only the finite fixture bridge and browser-derived evidence.

At the cycle base:

- neither `flex-item-collapse` nor any `fri07_*` source, manifest record, or XML
  output exists in the browser-parity corpus;
- the corpus contains 1,432 tracked HTML sources and 5,712 generated XML files;
  `all.json` reports 5,712 generated, 16 unsupported, and zero expected-fail,
  quarantined, and failed-to-generate rows;
- `corpus.toml` SHA-256 is
  `99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701`,
  `all.json` SHA-256 is
  `8d59c87d1fcc185bda0372968ae81dbeff74f241c17335db98629ad49f1f463f`,
  and `test_helper.js` SHA-256 is
  `42bf9ff77810b2e9fb5a184f525d9e22f74abae12a09f9486b3b49dc620188c2`;
- the manifest pins Chrome for Testing `149.0.7827.115`, and the one already
  installed matching executable is
  `target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`;
  no browser or other software acquisition is authorized; and
- no existing HTML source contains authored `visibility: collapse`, so the six
  owned sources are the only expected semantic input additions. Existing XML
  may receive generator-owned provenance updates after the helper hash changes,
  but unrelated input attributes and geometry bodies must remain unchanged.

The fixture bridge is one-way and finite. The helper reads computed visibility
only for an in-flow flex item and emits the normalized layout-ready fact; the
serializer writes only `flex-item-collapse="collapsed"`; the Rust adapter maps
that exact token to `FlexItemCollapse::Collapsed`, defaults absence to `Normal`,
and rejects every other explicit token. It does not parse authored visibility,
infer state from fixture names or expected geometry, or create a general
visibility model.

The exact sources are:

- `tests/layout/browser_parity/html/flex/fri07_cross_auto_margin_overflow.html`;
- `tests/layout/browser_parity/html/flex/fri07_absolute_auto_margin_insets.html`;
- `tests/layout/browser_parity/html/flex/fri07_intrinsic_flex_basis.html`;
- `tests/layout/browser_parity/html/flex/fri07_collapsed_strut_single_line.html`;
- `tests/layout/browser_parity/html/flex/fri07_collapsed_strut_wrapping.html`;
- `tests/layout/browser_parity/html/flex/fri07_flex_composition.html`.

Each source owns the existing border-box/content-box by LTR/RTL matrix, for
exactly 24 XML outputs. No scoped generator run is required or authorized by
this plan. After helper, parser, serializer, HTML, manifest, and focused behavior
inputs are task-clean, T03 invokes the unfiltered ExistingPinned generator once.
If that run reveals a confirmed input defect, it is diagnostic; correct the
input through the owning task and record why the run is invalid before the one
replacement allowed by `FRI-07.10.3`. Never rerun over unchanged inputs.

The expected-fail registry begins and is expected to remain empty. A browser
disagreement is blocking evidence, not authority to weaken an oracle. No
expected-fail entry or synthetic substitute may enter this reviewed range unless
all predicates in `FRI-07.10.2` are first established and this cycle plan is
semantically revised to record the exact source, variants, normative and
independent evidence, minimized reproduction, synthetic test, disposition, and
revalidation trigger, followed by a fresh plan review.

Out of scope: production layout changes; public API, model, errors, reexports,
or docs; authored CSS parsing; root or sibling work; new fixture vocabulary
beyond the one normalized token; general visibility, inline-flex, positioned
layout, or rendering behavior; new modules, helpers shared outside the existing
fixture bridge, generator paths, commands, schemas, reports, scripts, lints, CI,
dependencies, features, MSRV, browser pin/profile, base style, Taffy import, WPT
mirror, scoped report, quarantine, aggregate `just parity-all`, hand-edited XML
or report data, suppressions, and unrelated cleanup. No new `allow`, `expect`,
or Surgeist-owned `unsafe` is permitted.

## 2 Impacts

Public API and production behavior: unchanged. The existing public
`FlexItemCollapse` model and flex algorithms are only consumed by the private
fixture adapter.

Dependencies, features, manifest, lockfile, MSRV, browser pin, launch profile,
task runner, and root: unchanged. Root facade wiring, computed-style lowering,
API artifacts, and gitlink promotion remain root-owned and outside C04.

Generated artifacts: six human-readable HTML inputs and six active manifest
records derive exactly 24 new XML outputs. The single full run owns the updated
sole `all.json` and current provenance throughout the generated XML corpus.
Existing non-provenance input attributes and geometry must remain byte-equivalent
unless an exact owned source explains the delta. Final expected inventory is
1,438 HTML and 5,736 XML with 5,736 generated and 16 unsupported rows, and zero
expected-fail, quarantine, and failure buckets under this reviewed revision.

Docs/examples: unchanged. Unsafe: prohibited across all tracked and non-ignored
owned Rust files.

## 3 Tasks

### 3.1 `P01/I07/S01/C04/T01` Lower The Normalized Collapse Fixture Fact

**Files/area:**
`tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity/support.rs`, and their existing focused test
modules only.

**Outcome:** carry one computed flex-item collapse fact through the existing
helper JSON, generator serializer, and Rust fixture adapter without introducing
a CSS parser or a second lowering path.

**RED evidence:** add `fri07_c04_collapse_helper_`,
`fri07_c04_collapse_serializer_`, and `fri07_c04_collapse_parser_` tests first.
They fail at the task base because the helper JSON and XML serializer omit the
fact and the Rust adapter always leaves the public input at `Normal`.

**Acceptance:** helper evidence distinguishes an in-flow collapsed flex item
from normal, hidden, absolute, display-none, and non-flex controls. The serializer
emits exactly one kebab-case attribute only for the collapsed state. The parser
maps only `collapsed`, defaults absence to `Normal`, and rejects `normal`,
`visible`, `hidden`, CSS-wide values, empty/malformed values, duplicates, and
authored `visibility`. Renaming the fixture or changing only expectation geometry
does not change normalized input. Existing style fields, helper assets, generator
path, and XML remain otherwise untouched.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_collapse_helper_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_collapse_serializer_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_collapse_parser_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
cargo fmt --check
git diff --check
```

**Dependency:** remotely verified C03 candidate only.

**Intended commit:** `test(parity): lower normalized flex collapse`.

### 3.2 `P01/I07/S01/C04/T02` Add The Six Bounded Browser Sources

**Files/area:** exactly the six HTML paths named in the boundary,
`tests/bin/surgeist-layout-generate/generator.rs`, and
`tests/layout/browser_parity.rs` for source/input inventory tests. Manifest,
XML, and reports remain unchanged.

**Outcome:** add the human-readable inputs for the complete `FRI-07.10.1`
behavior table and prove their exact 24-path derivation and input honesty before
any generated artifact changes.

**RED evidence:** add `fri07_c04_fixture_sources_` and
`fri07_c04_fixture_input_` tests first. They fail because all six sources and
their 24 standard variants are absent.

**Acceptance:** each source exercises only its named specification behavior:
signed cross auto margins and overflow; absolute auto margins with insets;
distinct intrinsic flex bases with grow/shrink; single-line collapsed struts;
wrapped zero-main recollection and gap rules; and full order/flow/replaced/
overflow/collapse composition. Sources use existing constrained HTML/CSS plus
computed `visibility: collapse`; no authored expectation, fixture name, sibling
source, or generated geometry controls normalized input. Exactly six new HTML
files exist, the prospective four-variant set is exactly 24, total HTML is 1,438,
and no pre-existing HTML, manifest, XML, report, helper, or production file
changes in this task.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_fixture_sources_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_fixture_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
cargo fmt --check
git diff --check
```

Corpus checking is intentionally deferred because the six sources are
unmanifested until T03. No generator invocation occurs in T02.

**Dependency:** T01 is task-clean.

**Intended commit:** `test(parity): add FRI-07 browser sources`.

### 3.3 `P01/I07/S01/C04/T03` Freeze Inputs And Derive The Corpus Once

**Files/area:** `tests/layout/browser_parity/corpus.toml`, final inventory and
lineage tests in `tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`, generator-derived
`tests/layout/browser_parity/xml/`, and the sole
`tests/layout/browser_parity/xml/generation-reports/all.json`.

**Outcome:** add one active Surgeist record for each settled source, prove the
browser executable and final input inventory, invoke ExistingPinned once without
a filter, and adopt the exact generator-owned 24-row lineage and full-corpus
provenance.

**RED evidence:** add `fri07_c04_manifest_`, `fri07_c04_corpus_`, and
`fri07_c04_browser_parity_` tests before derivation. At the task base they fail
because the six case records, 24 XML outputs, new report accounting, and current
provenance do not exist. Artifact RED is authoritative and must not be replaced
with hand-written XML.

**Pre-derivation acceptance:** `corpus.toml` has exactly six new active records,
each with `source_root = "surgeist"`, the exact source path,
`generator = "constrained-html"`, and no expected-fail/quarantine entry. The
focused helper/parser/serializer/source tests and both default/generator suites
pass. Exactly one executable matches the pinned path and reports
`Google Chrome for Testing 149.0.7827.115`. No scoped report exists, no generation
filter or browser override is inherited, and all input changes are reviewed
before the full invocation.

**Single derivation command:** run exactly once after pre-derivation acceptance:

```sh
env -u SURGEIST_BROWSER_CACHE -u SURGEIST_BROWSER_VERSION -u SURGEIST_LAYOUT_BROWSER_PARITY_ROOT CARGO_NET_OFFLINE=true SURGEIST_LAYOUT_GENERATE_FILTER= SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

**Final acceptance:** exactly 24 new `xml/flex/fri07_*.xml` outputs exist in the
standard four-variant matrix; all parse and match production layout. The corpus
has 1,438 HTML and 5,736 XML. `all.json` and manifest accounting report 5,736
generated, 16 unchanged unsupported, zero expected-fail, quarantine, and failure
rows, and no scoped report. Every pre-existing XML delta is limited to
generator-owned provenance caused by the helper revision; non-provenance input
attributes and geometry remain unchanged. The report and every XML carry current
manifest, helper, base-style, browser, and launch-profile provenance. Record the
final manifest, helper, report, and owned-24-output hashes for the C04 handoff.
After derivation all artifact commands are read-only; do not regenerate.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_manifest_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_corpus_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_browser_parity_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
cargo fmt --check
git diff --check
```

**Dependency:** T01 and T02 are task-clean; every input and manifest record is
settled before the single derivation.

**Intended commit:** `test(parity): derive FRI-07 flex corpus`.

## 4 Completion

The canonical implementation, task-review, status, holistic-review, landing,
publication, readback, and cleanup lifecycle applies. C04 acceptance is:

1. the helper, serializer, and Rust adapter carry exactly one normalized
   collapse fact with complete omission, rejection, and independence controls;
2. the exact six sources and 24 variants cover `FRI-07.10.1`, and each active
   record and output is source-derived rather than expectation- or name-derived;
3. one settled unfiltered ExistingPinned invocation owns all final XML/report
   changes, with no redundant run, manual edit, stale output, scoped report, or
   generator architecture expansion;
4. all 24 rows pass browser parity with an empty exception registry under this
   reviewed revision; any contrary evidence returns to the semantic plan-review
   gate required by `FRI-07.10.2` rather than weakening acceptance;
5. unrelated inputs, geometry, report buckets, browser policy, dependencies,
   features, MSRV, production, public API, root, and finding ownership remain
   unchanged; and
6. the remotely verified handoff records exact source/output inventory, final
   provenance hashes, Chrome version/path, command/review evidence, and the
   empty known-Chrome-failure registry for C05 sprawl assessment.

At the complete-status head, run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_collapse_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_fixture_sources_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --exit-code 5e1de7fe83326c4ec7c9b878fd39f4d3738fce5f..HEAD -- . ':(exclude)plans/cycles/P01-I07-S01-C04-bounded-browser-artifact-candidate.md' ':(exclude)tests/layout/browser_parity/scripts/gentest/test_helper.js' ':(exclude)tests/bin/surgeist-layout-generate/generator.rs' ':(exclude)tests/layout/browser_parity/support.rs' ':(exclude)tests/layout/browser_parity.rs' ':(exclude)tests/layout/browser_parity/corpus.toml' ':(exclude)tests/layout/browser_parity/html/flex/fri07_*.html' ':(exclude)tests/layout/browser_parity/xml/flex/fri07_*.xml' ':(exclude)tests/layout/browser_parity/xml/generation-reports/all.json' ':(exclude)tests/layout/browser_parity/xml/**/*.xml'
! git diff --unified=0 5e1de7fe83326c4ec7c9b878fd39f4d3738fce5f..HEAD -- '*.rs' | rg --pcre2 '^\+(?!\+\+).*#\s*\[.*\b(?:allow|expect)\s*\('
! rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' src tests
git diff --check 5e1de7fe83326c4ec7c9b878fd39f4d3738fce5f..HEAD
git status --short
```

The scope, suppression, unsafe, diff, and status gates print no output. No final
command invokes generation. Genuine blockers are limited to those defined by the
installed workflow; none is currently known.
