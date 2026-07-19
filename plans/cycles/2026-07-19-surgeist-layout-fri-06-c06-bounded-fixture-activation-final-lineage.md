# FRI-06-C06 Bounded Fixture Activation And Final Lineage

Status: reviewed

Cycle ID: `FRI-06-C06`

Owning repository: `surgeist-layout`

Cycle base: `6e3772f509b919ec9a9d027d8298600ed98ee531`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`7090ea13ba7d9e524ce432018c8b7c44c1b3b76428d2c666949d297656ce97c8`,
commit `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`, sections `FRI-06.4 D-16`,
browser/comparator portions of `FRI-06.9` and `FRI-06.10`, `FRI-06.11`, and
artifact portions of `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`f00bacc82e2df19a71a0e21479eb626376afbed5c8368d5ebc0983547d246f1b`,
commit `2738da48794fbafcb3f81add9b6c12147458cd74`, entry `FRI-06-C06`.

## Outcome

Activate the exact 340 currently unsupported FRI-06 variants, add the twelve
specified four-variant Surgeist sources, compare control and fragment identity
and geometry, and derive one final 5,712-output corpus from settled inputs through
the existing-pinned no-fetch path. Preserve the generator architecture.

## Boundary

FRI-06-C01 through C05 and both mandatory containment windows are published and
remotely verified. Product behavior is frozen at the cycle base. C06 changes only
the finite browser adapter, helper/serializer tests, twelve named HTML sources,
their manifest records, derived XML/report artifacts, and directly required
browser-parity tests.

The immutable entry state is:

- `all.json` SHA-256 is
  `4f18b4299765d7f0cf996fa5c2510724cfadb577651c3a438c3f2904cc4b94ab`,
  with `filter: null`, 5,324 generated, 356 unsupported, and zero expected-fail,
  quarantined, or failed-to-generate variants;
- `corpus.toml` SHA-256 is
  `bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`;
- 1,420 HTML sources, 5,324 XML outputs, and only `all.json` exist;
- the exact three FRI-06 transition reasons identify 85 unique sources and 340
  variants; the four missing-root sources and 16 variants remain unsupported;
- Chrome for Testing 149.0.7827.115 is already executable at
  `target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`;
  no acquisition is authorized; and
- no C06 generator run has occurred. Scoped existing-pinned runs are optional
  iteration diagnostics only. They are never required or cited as verification,
  and their outputs may not enter a task commit. After every owned helper,
  serializer, parser, HTML, and manifest input settles, T4 runs one unfiltered
  full regeneration. Every later check is read-only.

Non-goals are production `src/` changes; public API, dependency, feature,
lockfile, MSRV, browser pin, launch profile, task-runner, base-style, root,
sibling, docs, general CSS/text/bidi/shape parsing, rendering state, FRI-09+
behavior, expected failures, quarantines, scoped reports, and generator
architecture. A confirmed production defect returns to diagnosis and planning;
it is not repaired inside this artifact cycle implicitly.

## Impacts

- **Public API and production behavior:** unchanged.
- **Dependencies, features, lockfile, MSRV, and browser policy:** unchanged.
- **Generated artifacts:** twelve HTML sources and manifest records add 48
  variants; 340 existing variants leave unsupported accounting; settled inputs
  produce 5,712 XML outputs and one replacement `all.json`.
- **Docs/examples and root follow-up:** unchanged here; C07 owns closure docs and
  the leaf/root/text/shape handoff.
- **Unsafe:** prohibited in every tracked and non-ignored owned Rust file. No new
  `allow` or `expect` attribute is permitted.

## Tasks

### `C06-T1` Compare Control And Fragment Output Exactly

**Files:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused comparator/parser tests.

**Outcome:** Extend the finite expectation model and comparator to observe line
break/control geometry and final inline fragments, including source segment ID,
line index, visual index, physical rect, and baseline. Preserve ordinary node and
scroll comparisons.

**RED:** Add `fri06_c06_comparator_` tests first. Wrong or missing control
geometry and wrong or missing fragment rect, source ID, line index, visual index,
and baseline currently pass because controls are skipped and fragment output is
not compared.

**Acceptance:** Correct control and zero-geometry cases pass. Each named wrong or
missing field fails with a stable field-specific diagnostic before unrelated
child comparison. Fragment comparison uses final batch fragment output in source
association order, preserves intervening visual slots, distinguishes an empty
valid fragment set from absent state, and uses the existing scalar tolerance only
for numeric geometry/baselines. Existing XML with no fragment expectations keeps
its current meaning.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_comparator_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** published C05 output and fragment contracts.

**Intended commit:** `test(parity): compare inline control and fragment output`.

### `C06-T2` Lower Finite Shaped And Atomic Fixture Inputs

**Files:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused fixture-adapter tests.

**Outcome:** Parse only the reviewed shaped-segment, text metrics, bidi,
whitespace, following-break, and atomic-placeholder facts and lower them through
the production `InlineTextInput`, canonical non-box, and atomic participation
constructors. Store phase-correct fragment slices in the fixture tree for cold,
warm, and rounded comparison.

**RED:** Add `fri06_c06_inline_input_` tests first. Mixed text is currently a
measured leaf/anonymous box and the fixture tree cannot construct shaped text,
canonical text pairing, atomic participation, or committed fragment readback.

**Acceptance:** Strict parsing rejects empty/duplicate IDs, partial tuples,
non-finite metrics, invalid bidi/whitespace/break values, unmatched atomic child
indices, contradictory box fields/children/measurement, and unknown attributes.
Valid text becomes `LayoutInput::InlineText`, never `Box` or measurement. Atomic
facts bind exactly one existing child and preserve source order. Both empty and
nonempty committed fragment slices republish through the real layout tree method;
no authored text, glyph, font backend, shaper, or permissive fallback enters the
adapter.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_inline_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T1 defines the expectation side consumed by final fragments.

**Intended commit:** `test(parity): lower shaped inline fixture inputs`.

### `C06-T3` Lower Bottom Alignment And Finite Shape Bands

**Files:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused adapter/provider tests.

**Outcome:** Add exact `vertical-align: bottom` lowering and a fixture-only finite
physical shape-band table consumed through the production exclusion-provider
method.

**RED:** Add `fri06_c06_shape_input_` tests first. Bottom is rejected and the
fixture tree has no `FloatExclusion::Shape` input, validated query/result table,
provider record, or typed missing/mismatched result path.

**Acceptance:** Bottom maps only to `VerticalAlign::Bottom`. The strict band
schema accepts finite physical query bands and intervals, validates them through
production constructors, associates each result with its exact query, and binds
tables only to visible in-flow left/right shape floats. Empty, partial, and full
results plus missing/mismatched/provider-failure diagnostics use the real
provider front door. No `shape-outside` syntax, path geometry, general parser,
alternate band algorithm, or precomputed line position is added.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_shape_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T2 establishes the complete finite fixture tree.

**Intended commit:** `test(parity): lower finite shape fixture inputs`.

### `C06-T4` Settle Inputs And Derive The Final Corpus Once

**Files:** `tests/layout/browser_parity/scripts/gentest/test_helper.js`,
`tests/bin/surgeist-layout-generate/generator.rs`,
`tests/layout/browser_parity/support.rs` only if serialization requires a
narrow parser correction, `tests/layout/browser_parity.rs`,
`tests/layout/browser_parity/corpus.toml`, exactly the twelve HTML sources named
in `FRI-06.11`, and generator-derived
`tests/layout/browser_parity/xml/` plus
`tests/layout/browser_parity/xml/generation-reports/all.json`.

**Outcome:** Serialize only T1-T3's finite facts, remove the three exact obsolete
unsupported classifications, add the twelve specified four-variant sources and
active manifest records, freeze all input hashes, then run the sole full
existing-pinned derivation.

**Pre-derivation evidence:** Add `fri06_c06_helper_`,
`fri06_c06_serializer_`, `fri06_c06_source_inventory_`, and
`fri06_c06_manifest_` tests first. They fail because the helper still rejects the
85 sources, the serializer lacks shaped/control/fragment/shape facts, the twelve
sources/records are absent, and entry inventory remains 1,420/5,324/356. Make
those focused tests green without running the full generator. Optional scoped
existing-pinned diagnostics may be used while inputs are changing; they are not
verification evidence and no diagnostic XML/report is committed.

**Input acceptance:** The helper reads only DOM/range geometry and the bounded
layout-ready facts required by `FRI-06.11`. It removes exactly
`Unsupported mixed text/element content`,
`Unsupported vertical <br> line-break semantics`, and
`Unsupported <br> outside block inline-run semantics`, while preserving
`Unsupported missing #test-root fixture root`. Serializer/parser pairs are
strict, round-trip every T1-T3 field, and reject partial or unknown forms. Exactly
the seven `html/block/fri06_*.html` and five `html/float/fri06_*.html` sources
listed in `FRI-06.11` exist, each has one active `source_root = "surgeist"`,
`generator = "constrained-html"` manifest record and four standard variants.
The post-input manifest SHA-256 is recorded before derivation and must remain
unchanged afterward.

**Focused pre-derivation commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c06_helper_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c06_serializer_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c06_source_inventory_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c06_manifest_
test -x 'target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
shasum -a 256 tests/layout/browser_parity/corpus.toml
```

**One final full derivation:** After every input above is settled, run exactly
once:

```sh
CARGO_NET_OFFLINE=true SURGEIST_LAYOUT_GENERATE_FILTER= SURGEIST_BROWSER_PATH='target/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing' cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

**Derived acceptance:** No later generator command runs. HTML inventory is
1,432; XML inventory and generated report count are 5,712. `all.json` has
`filter: null`, exactly 16 unsupported variants all carrying only the immutable
missing-root reason, and zero expected-fail, quarantined, or failed-to-generate
variants. All 340 activated and 48 new variants parse and match; helper, manifest,
source, browser, launch-profile, and XML provenance are current. Only `all.json`
exists, no stale/scoped report or XML remains, and all generated artifacts are
generator-owned rather than hand-edited. The frozen post-input manifest hash is
unchanged.

**Read-only commands after derivation:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just parity-all
git diff --check
zsh <<'SURGEIST_C06_SAFETY'
set -eu
set -o pipefail
added_rust=/tmp/surgeist-layout-fri06-c06-added.rs
test ! -e "$added_rust"
trap 'rm "$added_rust"' EXIT
git diff --unified=0 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD -- '*.rs' | sed -n '/^+++ /d; /^+/s/^+//p' > "$added_rust"
if rg -n -U --pcre2 '(?<![.\w])(?:allow|expect)\s*\(' "$added_rust"; then exit 1; else rg_status=$?; test "$rg_status" -eq 1; fi
rm "$added_rust"
trap - EXIT
owned_rust=("${(@f)$(git ls-files --cached --others --exclude-standard '*.rs')}")
test "${#owned_rust[@]}" -gt 0
if rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' "${owned_rust[@]}"; then exit 1; else rg_status=$?; test "$rg_status" -eq 1; fi
SURGEIST_C06_SAFETY
```

**Dependency:** T1-T3 are task-clean; every fixture input and focused test is
green before the one full derivation.

**Intended commit:** `test(parity): derive FRI-06 browser corpus`.

## Completion

After all four task ranges are independently clean, make the separate
status-only `complete` commit and set the immutable cycle head. Run only read-only
verification:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just parity-all
git diff --check 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD
git diff --name-only --no-renames 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD
zsh <<'SURGEIST_C06_SAFETY'
set -eu
set -o pipefail
added_rust=/tmp/surgeist-layout-fri06-c06-added.rs
test ! -e "$added_rust"
trap 'rm "$added_rust"' EXIT
git diff --unified=0 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD -- '*.rs' | sed -n '/^+++ /d; /^+/s/^+//p' > "$added_rust"
if rg -n -U --pcre2 '(?<![.\w])(?:allow|expect)\s*\(' "$added_rust"; then exit 1; else rg_status=$?; test "$rg_status" -eq 1; fi
rm "$added_rust"
trap - EXIT
owned_rust=("${(@f)$(git ls-files --cached --others --exclude-standard '*.rs')}")
test "${#owned_rust[@]}" -gt 0
if rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' "${owned_rust[@]}"; then exit 1; else rg_status=$?; test "$rg_status" -eq 1; fi
SURGEIST_C06_SAFETY
git status --short
```

The changed-path inventory may contain only the corrected FRI-06 sequence, this
plan, T1-T4's named browser-parity adapter/helper/generator/test paths, the twelve
specified HTML files, `corpus.toml`, and generator-derived XML/`all.json`.
Production `src/`, Cargo/lockfile, README, scripts/task runner, base style,
browser policy, root, sibling, API artifacts, expected-failure/quarantine, and
unrelated paths fail completion.

The added-allowance and owned-Rust commands must return no matches; the owned
manifest must be nonempty. Static checks must also prove one report, exact
1,432/5,712/16/zero bucket counts, only the missing-root reason, `filter: null`,
the frozen post-input manifest hash, and current generated provenance. No
generator command is part of completion verification.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`6e3772f509b919ec9a9d027d8298600ed98ee531..cycle_head`. Rerun the complete
read-only final set on local `main`, publish the immutable SHA to authority remote
`main` with the standard leased fast-forward, fetch/read back, and prove local,
tracking, `FETCH_HEAD`, and live remote agreement. Remove every cycle-owned
temporary resource.

The handoff freezes all generator inputs and outputs for read-only C07 closure.
Blocker: none at planning time.
