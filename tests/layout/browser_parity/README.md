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

Artifact ownership is intentionally finite. Exactly one unfiltered full C06 run
was allowed after all eleven FRI-05 HTML inputs and active manifest records
settled; that run owned pruning and wrote the canonical XML plus `all.json`.
After C06, those 44 FRI-05 outputs, the report buckets (5,324 passed, 356
unsupported, and no failure bucket), and manifest hash
`bc39d26ba27e64c85b743c577f20b3cb290fe78326432ad6210f2c2b44e5fbb1`
are frozen and read-only for C07. Filtered generation remains an iteration
diagnostic, never verification evidence; a confirmed input defect returns to
C06 and replaces the full run only after corrected inputs settle.

Run checked-in fixtures:

```sh
cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
```

Regenerate XML fixtures from constrained HTML fixtures:

```sh
cargo run --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate
CARGO_NET_OFFLINE=true SURGEIST_BROWSER_PATH=target/surgeist-browser/.../Google\ Chrome\ for\ Testing cargo run --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- generate-existing
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
filtered run is an optional, report-free iteration diagnostic: it writes only
matching XML, writes or prunes no report, and is not verification evidence.

Import or verify the pinned Taffy green baseline:

```sh
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy
cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-taffy-corpus
```

The Taffy baseline is fetched from the pinned upstream repository and commit in
`corpus.toml` into `target/surgeist-sources/taffy/<commit>`, then copied into
`html/`. The Taffy-only check verifies that checked-in baseline.

`SURGEIST_LAYOUT_BROWSER_PARITY_ROOT` may select a self-contained corpus root.
Only one unfiltered full run writes and prunes the canonical report and XML
artifacts used as final evidence. A successful full run writes the manifest's
`all.json` and removes non-manifest reports. `all.json` is the sole provenance
authority: its schema-versioned global metadata records the generator, stable
repository-relative browser provenance, launch profile, helper, base style,
corpus manifest, and Taffy revision once; each generated entry records its
repository-relative source/output identity, source hash, linked-resource hashes,
and XML hash. Generated XML is comment-free.

`check-corpus` is browser-free: it reads neither browser selection variables nor
the generation filter, and validates the exact manifest report/XML inventory,
global metadata, strict paths and identities, uniqueness, every source,
linked-resource, and XML hash, and the absence of embedded XML provenance.
`check-taffy-corpus` and
`import-taffy` are also browser-free; import remains an acquisition-capable
operation and should be run only with explicit authority.

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

Named grid placement syntax is parsed directly into layout inputs for parity
fixtures and exercised through production layout. The formerly blocked subgrid
line-name fixtures `subgrid_line_names_004_b_to_b_minus_1` and
`subgrid_line_names_repeat_outer_span_a_to_a_8` now run as ordinary parity
fixtures, so any future failures should report their concrete mismatch or
validation kind.
