# Surgeist Layout Browser Parity

This harness runs Surgeist layout against browser-derived parity fixtures. The
corpus is rooted at `tests/layout/browser_parity`:

- `html/`: current constrained HTML corpus. This includes the pinned Taffy
  green baseline plus Surgeist-authored fixtures.
- `xml/`: generated browser expectations consumed by the Rust parity runner.
- `corpus.toml`: source provenance, import contracts, and expected-failure or
  quarantine accounting.
- `scripts/gentest/`: shared browser measurement helper and base CSS.

XML files are generated artifacts. Do not edit browser geometry in XML by hand;
update the source fixture, importer, manifest, or generator instead.

Run checked-in fixtures:

```sh
cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
```

Regenerate XML fixtures from constrained HTML fixtures:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
SURGEIST_PARITY_FILTER=subgrid cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
```

Import or verify the pinned Taffy green baseline:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

The Taffy baseline is fetched from the pinned upstream repository and commit in
`corpus.toml` into `target/surgeist-sources/taffy/<commit>`, then copied into
`html/`. The Taffy-only check verifies that checked-in baseline.

The generator is a Rust binary. It fetches a pinned Chrome-for-Testing build into
`target/surgeist-browser` unless `SURGEIST_BROWSER_PATH` points at an explicit
browser executable. The default fetched version is `149.0.7827.115`, pinned for
the current corpus and the Rust CDP driver used by the generator. The browser
runs headless with a temporary profile. The Rust generation path handles
constrained `html/` fixtures with `getTestData()`, then writes XML into `xml/`.

Optional generator environment:

- `SURGEIST_BROWSER_PATH`: explicit Chromium-compatible executable to run.
- `SURGEIST_BROWSER_CACHE`: project-local browser download cache override.
- `SURGEIST_BROWSER_VERSION`: optional Chrome-for-Testing version accepted by
  `chromiumoxide`.
- `SURGEIST_LAYOUT_BROWSER_PARITY_ROOT`: override a self-contained root with
  `html/`, `xml/`, and `corpus.toml`.

Generation writes scope-specific reports under `xml/generation-reports/`, such
as `subgrid.json` or `all.json`. Each report records the active filter,
generated XML files, unsupported inputs, expected failures, quarantine entries,
and failed generation attempts.

HTML fixtures are the human-readable source of truth. The constrained `html/`
fixtures are runnable today. XML fixtures are generated browser expectations for
fast Rust-side regression tests.

Unsupported browser features are not silently skipped. If Surgeist does not yet
support a construct such as `<br>` or mixed inline text, the harness should keep
the fixture visible and report a classified pass/fail bucket.

The supported generator is the Rust `surgeist-layout-generate` binary. The
`scripts/gentest` directory is helper-only and must contain exactly
`test_helper.js` and `test_base_style.css`, both loaded by the Rust generator.
The old standalone Rust generator crate has been removed so there is one
generation path.

Inline display values are parsed so constrained fixtures remain readable:
`inline-block`, `inline-grid`, and `inline-grid-lanes` participate as atomic
inline boxes with inner block, grid, or grid-lanes formatting contexts. Full
parity failures from fixtures containing those displays now report their actual
geometry mismatch kind rather than an unsupported-inline bucket.

Atomic inline displays (`inline-block`, `inline-grid`, and `inline-grid-lanes`)
are expected to reach normal geometry comparison. The tree layout engine still
does not model non-atomic inline text/span behavior; if those fixtures enter the
harness, they should get an explicit classification separate from atomic inline
display handling.

Inline engine impact measured on 2026-06-17:

- `SURGEIST_PARITY_FILTER=subgrid`: 632 failures, with no unsupported inline,
  named placement lowering, or grid-line validation bucket. Remaining failures
  are geometry buckets: height mismatch 248, width mismatch 188, x mismatch
  172, y mismatch 24.
- `SURGEIST_PARITY_FILTER=grid-lanes`: 48 failures, with no parse, lowering, or
  inline-dispatch failures. Remaining failures are geometry buckets: height
  mismatch 24, width mismatch 24.

Named grid placement syntax is parsed into style declarations for parity
fixtures and lowered through production layout. The formerly blocked subgrid
line-name fixtures `subgrid_line_names_004_b_to_b_minus_1` and
`subgrid_line_names_repeat_outer_span_a_to_a_8` now run as ordinary parity
fixtures, so any future failures should report their concrete mismatch or
validation kind.
