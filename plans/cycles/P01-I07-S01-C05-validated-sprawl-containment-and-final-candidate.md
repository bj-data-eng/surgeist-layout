# P01-I07-S01-C05 Validated Sprawl Containment And Final Candidate

Status: in_progress

Cycle ID: `P01/I07/S01/C05`

Owning repository: `surgeist-layout`

Cycle base: `3ad856b7826eb4fff2fb80ce63b51dc479a78fa9`

Reviewed specification:
`plans/specs/P01-I07-flex-algorithm-completeness.md`, normalized semantic-content
SHA-256 `df69716865bf7f88bf89a7ecfea979cffa3b879b69a2cde16586d7598edb1332`,
commit `f86b0572863d8eb72da5c00364bf7020299c99b8`: `FRI-07.4 D-18` and
`FRI-07.11` through `FRI-07.16`.

Reviewed implementation sequence:
`plans/sequences/P01-I07-S01-flex-algorithm-completeness.md`, normalized
semantic-content SHA-256
`2774cf6c8ce74afdead6fe018d5d0f299f8d208af7a3a19107ffda7277550cea`,
commit `9fe46f932b8538ee570af7d413a1be111078609f`, entry
`P01/I07/S01/C05`.

Bounded outcome: validate every current-source FRI-07 sprawl disposition,
consolidate the one confirmed duplicate test-inventory projection without
changing behavior or artifacts, and publish the final FRI-07 leaf candidate.

## 1 Boundary

C04 candidate `3ad856b7826eb4fff2fb80ce63b51dc479a78fa9` is clean,
published, and remotely read back from `origin/main`. It freezes the completed
FLEX-002 through FLEX-005 behavior and the browser/artifact candidate. C05 may
change only test-harness structure proven behaviorally equivalent by the complete
assessment below.

The assessment reviewed the exact FRI-07 range
`d386c7d796e5fe0c0856c15ac800516df1348f3b..3ad856b7826eb4fff2fb80ce63b51dc479a78fa9`
and the directly affected flex, sizing, model, test-support, fixture, and
generator boundaries. Its current-source dispositions are:

| ID | Disposition |
| --- | --- |
| `SP-001` | Confirmed mechanical test-harness sprawl. `tests/bin/surgeist-layout-generate/generator.rs` repeats the six FRI-07 sources in `fri07_c04_fixture_source_contracts` and `fri07_c04_case_ids`; `tests/layout/browser_parity.rs` repeats them in its HTML inventory and output-path helper. Each duplicate only adds/removes `.html` and derives the same standard four variants. |
| `SP-002` | Disproved duplicate axis owner. Production has one `FlexAxes`; no parallel flex edge or projection mapper exists. |
| `SP-003` | Disproved duplicate order owner. `collect_items` creates one collection and `item_order_permutation` orders it before both collapse rounds; `CollectedFlexItem` carries collapse state. |
| `SP-004` | Disproved duplicate basis owner. `ResolvedFlexBasis` is the sole typed state and preserves direct `MinContent` and `MaxContent`. |
| `SP-005` | Disproved equivalent margin helpers. Ordinary cross margins consume line free space; absolute margins require both resolved insets and use inset-modified space. An auto inset is an exact counterexample. |
| `SP-006` | Disproved duplicate collapse orchestration or first-round publication. One branch captures private struts and performs one replay; only final layout commits zero collapsed output. |
| `SP-007` | Disproved fixture-specific production behavior. No FRI-07 name or expected-geometry branch exists in production layout. |
| `SP-008` | Disproved parser/generator duplication. One bounded computed-collapse lowering, one generic XML parser/traversal, one generator path, and sole `all.json` provenance authority remain. |
| `SP-009` | Disproved hidden complexity. The range adds no lint allowance, `expect`, or executable `unsafe`. |

`SP-001` preserves two independent oracle classes that must not be merged into
the inventory owner: expected-fail reason literals in the generator target and
exact Chrome mismatch signatures in the parity target. Their independence
detects a changed reason or a different first layout mismatch.

Out of scope: production source; public API, types, errors, reexports, or docs;
layout, geometry, scalar, cache, transaction, error, parser, fixture, manifest,
report, or generator behavior; HTML, XML, generated artifacts, corpus membership,
browser helper, browser execution, generator execution, root or sibling work;
dependencies, features, MSRV, manifests, lockfiles, scripts, CI, new modules,
shared helpers across test targets, macros, broad lint cleanup, suppressions, and
unrelated cleanup. No new `allow` or `expect` and no Surgeist-owned `unsafe` are
permitted.

## 2 Impacts

Public API and production behavior: unchanged. No `src/` file changes.

Dependencies, features, manifests, lockfile, MSRV, browser pin/profile, root,
and all 59 finding-owner assignments: unchanged.

Generated artifacts and fixtures: unchanged. No generator or browser invocation
is authorized. Frozen hashes at the cycle base are:

- `tests/layout/browser_parity/corpus.toml`:
  `4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc`;
- `tests/layout/browser_parity/scripts/gentest/test_helper.js`:
  `caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019`;
- sole `tests/layout/browser_parity/xml/generation-reports/all.json`:
  `5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f`;
- sorted 24-output hash-list aggregate:
  `07a0998379b3bb9db90f818d5c5897b6051ec6e33b739fbe454524ab8ce412ee`;
- complete XML aggregate:
  `4d951ba1c022466db5c3903dc84072064a0ec17a7a8aadb008363c442d1a4a96`;
- XML inventory hash:
  `0468935c1a2165886cd842572d50a5f6099d7dc0712e9b0b104ad7a6879ad11b`.

Docs and examples: unchanged except this canonical plan. Root follow-up is the
final leaf-candidate handoff only. Owned Rust remains free of `unsafe`.

## 3 Tasks

### 3.1 `P01/I07/S01/C05/T01` Consolidate Bounded Fixture Inventory Projections

**Files/area:** only
`tests/bin/surgeist-layout-generate/generator.rs` and
`tests/layout/browser_parity.rs`, inside their existing FRI-07 test support.

**Outcome:** in the generator target, retain
`fri07_c04_fixture_source_contracts` as its single six-source inventory and
derive case IDs, manifest IDs, and standard output paths from each source path.
In the parity target, retain one local six-source inventory and derive both full
HTML paths and the standard four-variant XML paths from it. Do not create a
cross-target helper or a production-visible owner.

**Pre-change characterization:** before editing either file, run the two focused
FRI-07 suites. They pass at the exact task base with 14 generator tests and 13
layout tests. Record the exact pass totals. A failure is diagnosis, not authority
to change behavior or weaken an oracle.

**Acceptance:** each target contains one six-source inventory; every derived ID
is exactly its repository-relative source path without `.html`; the standard
four-variant mapping and ordering remain unchanged; exact source inventory,
manifest status/reason, report lineage/hashes, 24 output paths, 12 ordinary
matches, 12 reviewed mismatch signatures, and three public-front-door synthetic
oracles retain their assertions and pass. Expected-fail reasons and mismatch
signatures remain independent literals. Test names and production code remain
unchanged. The composed diff reduces duplicate inventory literals without adding
a macro, module, abstraction shared across targets, lint suppression, or artifact
change.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_single_line_keeps_strut_and_suppresses_committed_gap
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_zero_main_rewrap_keeps_collection_gaps_and_identity_strut
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_composed_layout_exact_geometry_margins_strut_absolute_and_scroll
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
```

**Dependency:** published C04 candidate and the complete validated assessment in
Section 1. **Intended commit:**
`refactor(parity): consolidate FRI-07 fixture inventory`.

## 4 Completion

T01 receives a fresh `surgeist-worker`, characterization evidence, and a distinct
fresh `surgeist-task-reviewer`. After T01 is task-clean, change only `Status` to
`complete` in a separate coordinator commit. At that exact head:

1. every `SP-001` duplicate projection is removed and `SP-002` through `SP-009`
   retain their source-validated counterexamples or single owners;
2. FLEX-002 through FLEX-005 retain their public-front-door closure;
3. source inventory, manifest statuses/reasons, 24 output paths, ordinary rows,
   exact expected-fail signatures, and three substitutes remain green;
4. no production, API, docs, dependency, feature, manifest, lockfile, fixture,
   helper, report, XML, generated, browser, root, or finding-owner delta exists;
5. no generator/browser command ran; and
6. final review, landing, publication, remote readback, and the FRI-07 leaf
   candidate handoff complete under the canonical workflow.

Run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri07_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri07_c04_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_single_line_keeps_strut_and_suppresses_committed_gap
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c02_collapse_round_zero_main_rewrap_keeps_collection_gaps_and_identity_strut
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri07_c03_composed_layout_exact_geometry_margins_strut_absolute_and_scroll
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
git diff --exit-code 3ad856b7826eb4fff2fb80ce63b51dc479a78fa9..HEAD -- src Cargo.toml Cargo.lock README.md tests/layout/browser_parity/README.md tests/layout/browser_parity/corpus.toml tests/layout/browser_parity/html tests/layout/browser_parity/scripts tests/layout/browser_parity/xml
test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | cut -d ' ' -f 1)" = 4419c4aab9429d1f81ac46426095719e19cf92cfbf51caf66d4f737c07c452cc
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | cut -d ' ' -f 1)" = caafa5a48787c9b80a45d8b2c8ac6f91b8ad7ab14a85e5bcdf3a3e922ebce019
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | cut -d ' ' -f 1)" = 5c560f240d27ad28d00023156b0bf2744aa8392d34fe916d800e02894e10353f
```

The final handoff records the remotely verified candidate SHA, all nine sprawl
dispositions, four-finding closure, unchanged artifact hashes and known-Chrome
registry, complete verification/review evidence, public/root ownership boundary,
and later-P01 continuation state. Root facade/API generation/gitlink promotion
remain separate root-owned work.
