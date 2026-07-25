# P01-I04-S01-C05 Bounded Fixtures And Final Derivation
Status: complete
Cycle ID: `P01/I04/S01/C05`
Owning repository: `surgeist-layout`
Cycle base: `601f4ad4700827465096dc62c029b4d8147336b8593b6b80ae7abfb09fc22577`
Reviewed specification: `plans/P01-layout/initiatives/P01-I04-property-specific-sizing-values.md`
at SHA-256 `49ede2ba2672a91f99ba193651dbb1350ede7b80`, commit `5d33f3a4ab694f12985d713f7dbc74b251d55fb6`, sections `FRI-04.4 D-07`, `FRI-04.7`, the parser/helper/serializer/artifact evidence in `FRI-04.8`, the fixture rows of `FRI-04.9`, and acceptance items 10 through 12 and 14.

Reviewed sequence: `plans/P01-layout/sequences/P01-I04-S01-property-specific-sizing-values.md`
at SHA-256 `2e006d30b0250c526e10bba13a37e58e111ff60791b34f8d7c2e4d0e527db13f`, commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `P01/I04/S01/C05`.

## 1 Outcome
Add the finite property-specific fixture grammar, exact helper and serializer
preservation, canonical unsupported-report digest test, and three active FRI-04
browser sources. Once every generation input is settled, derive the complete
corpus with one full ExistingPinned run and prove the exact 12-output owned
parity matrix, report inventory, and provenance.

## 2 Boundary
The cycle starts from the published and remotely verified C04 candidate. It
owns only `tests/layout/browser_parity/support.rs`, the existing browser helper
and generator serializer/tests, `corpus.toml`, the three named HTML sources,
browser-parity inventory and owned-matrix tests, and generator-derived XML and
`all.json`.

The fixture parser remains a private typed adapter. It accepts only px,
percentage, existing unitless fixture lengths, affine `calc`, nested
`min`/`max`/`clamp`, one-argument `fit-content`, property-valid keywords,
canonical `calc-size` bases/calculations, and maximum-track-only finite
non-negative `fr`. Structured recursive descent is bounded at 64 sizing
function levels before descent; public checked constructors own normalized
values and invalid-state rejection.

Helper and serializer changes only preserve the finite owned token strings
through existing style attributes. They do not parse general CSS or add a
command, report kind, module, schema, dependency, feature, browser path, or
generator subsystem. Generator architecture expansion remains out of scope.

Scoped ExistingPinned runs are optional diagnostics while inputs are changing.
They are never required, retained, or cited as verification. After parser,
helper, serializer, manifest, tests, and HTML inputs settle, run one unfiltered
full ExistingPinned generation. No second full run is allowed over the same
inputs. A later confirmed input bug invalidates that run and permits one
replacement only after the corrected inputs settle.

Use only the already-cached Chrome `149.0.7827.115`; no acquisition is allowed.
The exact 12 FRI-04 outputs constitute this cycle's complete browser comparison.
The ignored 5,280-output aggregate remains the `FRI-13` release gate, so
`just parity-all` is not a C05 command.

Production `src/`, public API, algorithms, docs, examples, root, siblings,
dependencies, features, MSRV, browser pin/profile, base style, scripts, and
task-runner recipes are read-only. XML is never hand-edited.

## 3 Impacts
Public API, production behavior, dependencies, features, docs, examples, MSRV,
root, and siblings: unchanged. Fixture parsing and generator serialization are
private test infrastructure. Generated artifacts: three HTML inputs produce 12
new XML while all 5,280 XML and the sole report receive current generator-owned
provenance. Safety: all owned Rust remains free of `unsafe`.

## 4 Tasks
### 4.1 `P01/I04/S01/C05/T01` Bounded Property-Specific Fixture Parsing
**Files:** `tests/layout/browser_parity/support.rs` and its focused tests.
**Outcome:** Replace the affine-only sizing adapter with balanced, arity-checked,
depth-bounded parsing that lowers each preferred, minimum, maximum, flex-basis,
minimum-track, and maximum-track state through its public checked constructor.
**RED:** Add tests named with the `fri04_c05_parser_` prefix first. They fail
because nested sizing functions, calc-size, and the expanded keyword domains
are absent or because malformed input is not diagnosed at the owned boundary.
**Acceptance:** Tests cover every property-valid construction row, nested
`min`/`max`/`clamp`, omitted clamp endpoints, `fit-content`, canonical
`calc-size` with `any`, `100%`, or a property-valid keyword basis, affine
px/percentage/size coefficients, and track-only `fr`. They reject empty or
wrong-arity functions, malformed or unbalanced delimiters, depth 65, non-finite
numbers, `size` outside calc-size, size-dependent `Any`, cross-property
`none`/`content`/`fr`, and flex in a minimum track. Existing unitless fixture
lengths and all checked-in XML remain valid without fallback or panic.
**Commands:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri04_c05_parser_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
```
**Dependency:** Published C04 production and parser-value decisions at the cycle base.
**Intended commit:** `test(parity): parse bounded property sizing fixtures`.

### 4.2 `P01/I04/S01/C05/T02` Preserve Tokens And Derive The Final Corpus
**Files:** `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity/corpus.toml`,
`tests/layout/browser_parity/html/block/fri04_sizing_math_functions.html`,
`tests/layout/browser_parity/html/flex/fri04_flex_basis_content.html`,
`tests/layout/browser_parity/html/grid/fri04_track_math_functions.html`,
`tests/layout/browser_parity.rs`, and generator-derived `xml/` plus
`xml/generation-reports/all.json`.
**Outcome:** Preserve the finite owned sizing strings without widening box
fields to `fr`; serialize them through the existing attributes; add the three
active sources and exact 12-path matrix; assert the canonical unsupported
projection in the generator feature; then perform the one final full derivation.
**RED:** Add tests named with `fri04_c05_helper_serializer_` and
`fri04_c05_fixture_` first. They fail because the helper drops the new strings,
the serializer lacks their representation, the sources and exact matrix are
absent, and the final inventory is still 1,406 HTML and 5,268 XML.
**Characterization:** Add `fri04_c05_unsupported_` against the published base;
it starts green at the canonical digest and must remain green after derivation.
**Acceptance:** Helper tests preserve all finite grammar members exactly and
reject unsupported syntax and box `fr`; serializer tests emit exact existing
width/height/min/max/flex-basis/track attributes. A generator-feature test
projects, sorts, pretty-serializes, and hashes the 356 unsupported tuples to
`c44aaae7f939ebc07341cb984ca3f040512ec4dd5462d75454b178a713492030`
using existing `serde_json` and `sha2` support.

The three sources exercise only C04-supported block calculation, explicit flex
`Content`, and fixed/fit-content grid-track behavior. Their four standard
variants are the exact 12 owned outputs; the matrix rejects missing, duplicate,
misplaced, or extra paths and every output parses and matches Surgeist layout.
Final inventory is 1,409 HTML, 5,280 XML, and only `all.json`; its summary is
5,280 generated, 356 unsupported, and zero in every failure class. Every XML
has current helper provenance, source hashes change only for the three new
HTML. Any body, style-input, or geometry delta outside the 12 owned outputs is
individually source-backed and explained rather than treated as owned parity.
**Commands before final derivation:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri04_c05_helper_serializer_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri04_c05_unsupported_
test -x 'target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
```
**One final derivation:**
```sh
CARGO_NET_OFFLINE=true SURGEIST_LAYOUT_GENERATE_FILTER= SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```
**Read-only commands after derivation:**
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri04_c05_fixture_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
```
**Dependency:** `T01` is task-clean and every generation input is settled.
**Intended commit:** `test(parity): derive FRI-04 sizing corpus`.

## 5 Cycle Acceptance
1. Both task ranges have independent clean task reviews and compile-stable ordered boundaries.
2. The adapter accepts exactly the finite D-07 grammar through property-specific checked APIs and rejects every named invalid state at depth 65 or earlier.
3. Helper and serializer preserve exact owned strings without general CSS parsing, box `fr`, or generator architecture expansion.
4. The three active sources produce exactly 12 owned outputs and their complete matrix parses and matches layout.
5. One settled-input full derivation produces 1,409 HTML, 5,280 XML, sole `all.json`, current provenance, 356 canonically unchanged unsupported tuples, and zero failure classes.
6. Existing XML receives current helper provenance; any other delta outside the 12 new outputs has exact source-backed accounting.
7. No production, public API, docs, dependency, feature, MSRV, browser policy, task runner, root, sibling, ignored-test, or expected-failure expansion enters the range.

## 6 Final Verification
Run this read-only set once after both tasks are clean:
```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri04_c05_parser_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri04_c05_helper_serializer_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri04_c05_unsupported_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri04_c05_fixture_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
git diff --check
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' --glob '*.rs' .
```

The final unsafe scan must report no executable match. All final checks are
read-only, and the complete cycle diff must match the boundary above.

## 7 Handoff And Blockers
The published cycle hands C06 a final, remotely verified parser and generated
inventory so public docs, traceability, and the leaf candidate handoff can close.
It does not emit the final FRI-04 or root handoff.

A genuine blocker is a missing exact cached browser, required acquisition,
unsupported-digest drift, an unexplained existing XML body change, failure of a
named owned fixture that requires later-format behavior, or a need to expand
generator architecture. Stop and return to planning; do not broaden grammar,
weaken an oracle, hand-edit XML, or repeat a full run over unchanged inputs.
