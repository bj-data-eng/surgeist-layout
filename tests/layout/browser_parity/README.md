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

The sizing bridge is a finite fixture adapter, not app-facing CSS parsing. It
accepts finite px and percentage values plus the existing unitless fixture
values; affine `calc()`; nested `min()`, `max()`, and `clamp()`; one-argument
`fit-content()`; canonical `calc-size()`; and only property-valid keywords.
Expression depth is limited to 64. A finite, non-negative `fr` is accepted only
for a maximum track breadth, never a minimum track breadth or a box/flex
property. Integration layers outside this layout-ready harness own authored CSS
parsing and lowering.

Inline metrics attributes are layout-ready fixture data. They are not CSS
syntax. Root/style/text integration is expected to generate them from computed
style and text/font metrics later.

## FRI-05 Normalized Scroll Adapter

The FRI-05 bridge is likewise a bounded fixture adapter, not authored CSS
parsing. During the one authorized full C06 run, the generator read browser
computed style and serialized only these normalized attributes: `overflow-x`,
`overflow-y`, `scrollbar-width`, `overflow-clip-margin`, `scrollbar-gutter`, the
four physical `scroll-padding-*` edges, the four physical `scroll-margin-*`
edges, `scroll-snap-type`, `scroll-snap-align`, and `scroll-snap-stop`. It emits
the atomic overflow pair together and omits exact initial values where the
fixture schema permits. The Rust adapter accepts only the finite keyword,
pixel, percentage, and `calc()` subsets represented by the production layout
types; it rejects authored shorthands (`overflow`, `scroll-padding`, and
`scroll-margin`), CSS-wide keywords, variables, unresolved units, and ambiguous
snap forms. Root CSS/style integration owns full grammar, cascade, computed
value normalization, logical-to-physical lowering, and explicit scrollbar
environment selection.

Fixture `scroll_size` means the span of the canonical physical range:
`x.maximum() - x.minimum()` and `y.maximum() - y.minimum()`. An expected zero is
still compared; missing geometry, wrong x span, and wrong y span fail rather
than being skipped. The comparator does not treat a maximum endpoint as a size.
Target fields are layout-produced metadata only: this harness does not model
retained association, transformed coordinates, current offsets, snap selection,
CSSOM, host scrollbar UI, or runtime events.

Current source pins, import rules, artifact inventory, and expected outcome counts
are declared in `corpus.toml`. The full parity runner remains explicitly ignored
in ordinary Cargo test runs; invoke it separately to compare every checked-in
fixture. Generation infrastructure changes must preserve XML bytes and complete
per-fixture outcomes unless browser expectation changes are separately intended.

Run checked-in fixtures:

```sh
cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
```

Browser generation and source import require the shared generator's supported
mutation host, Apple-Silicon macOS. Default layout builds remain independent of
this tooling requirement.

Regenerate XML fixtures from constrained HTML fixtures:

```sh
cargo run --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH=tmp/surgeist-browser/.../Google\ Chrome\ for\ Testing cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
```

`generate` resolves the manifest-owned Chrome-for-Testing pin and may use the
managed cache/fetcher. `generate-existing` is the no-fetch artifact path: its
browser path must be nonempty, repository-relative, under the manifest cache,
executable, and report the exact pinned `--version`. Both modes use the one
manifest-owned launch profile, with disabled Chromium defaults/cache and
`use-mock-keychain`; batch, timeout, polling, retry, profile, and page lifetimes
are all manifest-owned.

`SURGEIST_BROWSER_CACHE` and `SURGEIST_BROWSER_VERSION` are not overrides and
are rejected for generation. `SURGEIST_LAYOUT_GENERATE_FILTER` is empty for an
unfiltered full run. With `generate-existing`, it may instead name one
normalized fixture path or prefix that matches at least one source. Such a
filtered run is an optional iteration diagnostic: it writes matching XML and
the private ownership ledger, leaves reports unchanged, and is not current
corpus verification evidence.

Import or verify the pinned Taffy green baseline:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

The Taffy baseline is acquired at the exact repository/revision declared in
`corpus.toml` under `tmp/surgeist-sources/taffy/<commit>`. Verified imports retain
Surgeist-authored HTML and write `html/.surgeist-source.json`, binding the source
pin and imported bytes. Adopting a corpus without this attestation first checks
its existing imports against the verified checkout. `check-taffy-corpus` verifies
the existing checkout and import without fetching.

Browser installations live under `tmp/surgeist-browser/`, separated by version.
Both caches are repository-local, already ignored by `tmp/`, and survive normal
`cargo clean`. Completed acquisitions are reused only after verification;
profile and publication recovery do not remove them.

`SURGEIST_LAYOUT_BROWSER_PARITY_ROOT` may select a self-contained corpus root.
The exact corpus and browser-owner roots are checked separately. A full run
validates declared accounting before atomically publishing XML and reports.
Filtered generation leaves full reports and unrelated artifacts unchanged.
A full run with per-fixture failures publishes diagnostic accounting and retains
outputs that were not regenerated. The private `xml/generation-ownership.json`
ledger permits a subsequent full run to repair filtered or diagnostic state; it
does not replace full-report provenance. Machine-local coordination and recovery
state lives in the ignored `.surgeist-generator/` directory beneath the corpus.

`xml/generation-reports/all.json` uses the shared browser engine's schema 4 and
is the generated-artifact provenance authority. It binds engine/host identity,
browser executable and launch settings, source/helper inputs, import attestation,
linked resources, XML hashes, and generated/unsupported/expected-failure/
quarantined/generation-failure accounting. XML remains comment-free.

`check-corpus` is browser-free and acquisition-free. It validates persisted
attestations, source/resource/artifact hashes, paths, inventory, and accounting
without requiring either cache. Browser-selection variables and generation
filters do not affect this command. XML parsing and semantic comparison remain
layout-owned tests; the generic engine treats artifact bytes as opaque.

Legacy schema-3 reports authorize only the first full migration after their
paths and hashes are checked. They cannot satisfy schema-4 provenance checks or
authorize filtered migration. Their historical metadata is not promoted into
current provenance without a complete generation run.

HTML fixtures are the human-readable source of truth. The constrained `html/`
fixtures are runnable today. XML fixtures are generated browser expectations for
fast Rust-side regression tests.

Unsupported browser features are not silently skipped. If Surgeist does not yet
support a construct such as `<br>` or mixed inline text, the harness should keep
the fixture visible and report a classified pass/fail bucket.

The supported entry point is layout's thin `surgeist-layout-generate` binary,
enabled by `layout-golden-generate`. It depends on an exact Git revision of
`surgeist-generator` with `browser-corpus`; default layout builds do not include
that tooling graph. The shared crate owns acquisition, browser processes,
provenance, accounting, and publication. Layout owns document/helper preparation,
measurement decoding, unsupported classification, and XML serialization.
`scripts/gentest` remains helper-only: `test_helper.js` and `test_base_style.css`.

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

Named grid placement syntax is parsed directly into layout inputs for parity
fixtures and exercised through production layout. The formerly blocked subgrid
line-name fixtures `subgrid_line_names_004_b_to_b_minus_1` and
`subgrid_line_names_repeat_outer_span_a_to_a_8` now run as ordinary parity
fixtures, so any future failures should report their concrete mismatch or
validation kind.
