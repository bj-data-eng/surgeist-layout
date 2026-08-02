# P01-I06-S01-C12 Final Production Correction And Lineage

Status: in_progress

Cycle ID: `P01/I06/S01/C12`

Owning repository: `surgeist-layout`

Cycle base: `8ffb4bc551a24d2283ad54436870ab3f5e66a473`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I06-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`35e62066a9e1593d3fd5111b897423688f3be77020d5c23c43e627063295166f`,
commit `b019df54310788d7b253d1f354a7ae3f5d59e76a`: `FRI-06.4 D-01`,
`D-04`, `D-06`, `D-07`, `D-09`, `D-11`, `D-12`, `D-13`, and `D-16`;
line, metric-fragment, atomic-baseline, physical-placement, comparator,
fixture, and acceptance portions of `FRI-06.5`, `FRI-06.7`, `FRI-06.9`
through `FRI-06.11`, and `FRI-06.14`.

Reviewed implementation sequence:
`plans/P01-layout/sequences/P01-I06-S01-inline-formatting-floats-bfcs.md`
at normalized SHA-256
`1513da19041bed2d38d24611dcda1e4b235a6e8309998f3588505c11e1ef0d69`,
commit `84ec3085f10b7b534407aa144c6264d630e0ed26`, entry `P01/I06/S01/C12`.

## 1 Outcome

Complete the reviewed T01-T04 production corrections, retain T05's successful
full generation as diagnostic evidence of an invalid final-lineage assumption,
and restore honest finite fixture/comparator inputs. Use the resulting scoped
diagnostic census to realize only the next exact production tasks. After all
inputs and production behavior settle, one later plan amendment authorizes the
single final full lineage and publication gate.

## 2 Boundary And Evidence

C08 is complete and remotely verified at the cycle base. Its task-clean public
fixture-shaped characterization records every current and browser coordinate,
line band, baseline, clear effect, float endpoint, and root size for the ten
rows. The production-browser assertion
`fri06_c08_recovery_characterization_browser_geometry_remains_unaccepted_production`
is exact RED; the companion current-geometry assertion prevents an incomplete
diagnosis from being mistaken for acceptance.

T01 is task-clean at `70d6e048c249b45fe5202b03063a9c76a926501e`; the amended
fixture-input and Chrome-oracle contracts do not change its traversal outcome or
evidence. The direct RTL defect was a redundant physical-direction reversal in
`src/inline.rs`: `reordered_inline_unit_indices` already yields bidi visual
order, after which `visual_order_opposes_logical_progression` reverses that
order again for horizontal RTL. The correction removes that second reversal
without changing source identity, resolved bidi levels, or mixed-level visual
ordering.

The vertical defect is confined to line metric resolution and physical
projection. Combining the forced-break strut with the atomic fallback baseline
expands or anchors the vertical line away from its 24px containing line band.
The browser requires two 24px bands with the atomic's 3px block-axis leading:
first atomic x 75, second atomic x 51, and cleared box x 30. The correction
must preserve D-11 fallback semantics and horizontal, top, and bottom controls.

The float defect is confined to continuation line phase and terminal automatic
block-size derivation. The current boundary-strut topology places the second
line at y 24 and the third at y 40, then the content-type predicate
`terminal_inline_run_has_mixed_text_atomic_line` conditionally adds 0.5px.
The browser line phase is y 0, 21.2, 21.2, and 42, producing an unrounded
terminal extent of 62.5 and rounded root height 63. The correction derives the
extent from resolved line/float geometry and does not retain or replace an ad
hoc content-type or integer-rounding adjustment.

T02 owns its eight exact `FRI-06.11` HTML marker sources, helper, narrow
serialization for the three closed XML forms, strict parser support, and focused
fixture-input tests. Public API, browser policy, base style, dependencies, root
integration, and generator architecture remain unchanged.

T05's preflight and replacement runs exposed zero-fragment whitespace and empty
Range schema defects. Their focused corrections are committed. The next full run
succeeded structurally at 5,712 generated and 16 expected missing-root
unsupported variants, then honestly failed 314 of the 388 activation rows. Its
uncommitted XML/report residue and exact hashes are diagnostic evidence, not
lineage. Independent audits partition the failures into comparator transparency
and line identity, fixture adaptation, truthful bidi input, RTL projection,
float phase, and only then any residual production behavior.

The attempted hard line-count task at `134db06f7` was rejected because it moved
large bodies into closure and function-pointer wrappers instead of separating
cohesive responsibilities. Commit `71ea9617c` exactly restores the pre-task tree.
That experiment contributes no task evidence, acceptance requirement, or
dependency; its retired T06 identity is not reused.

## 3 Known Chrome Measurement Failures

None. Chrome remains authoritative; an entry requires the full `FRI-06.11` gate.

## 4 Impacts

- **Public API and compatibility:** unchanged.
- **Production:** T01-T04 remain scoped; later production work is realized only
  from the corrected-input diagnostic.
- **Tests/fixtures:** exact comparator, default-block, and bidi-input honesty.
- **Generated artifacts:** T05 residue is diagnostic and uncommitted; scoped
  generation remains diagnostic only; one later reviewed full run owns lineage.
- **Lint:** configured repository gates remain unchanged by this cycle.
- **Dependencies, features, docs, examples, MSRV, and root:** unchanged.
- **Safety:** Surgeist-owned code remains free of unsafe; no new `allow` or
  `expect` attribute is permitted.

## 5 Tasks

### 5.1 Completed Task Evidence

T01-T04 are historical, independently reviewed, task-clean contracts. They must
not be re-executed or widened by later work.

| Task | Realized commits | Clean outcome |
| --- | --- | --- |
| `T01` Correct Direct RTL Visual Traversal | `cff9204e`, `70d6e048` | Consume visual order once; composition-independent RTL controls pass |
| `T02` Replace Synthetic Fixture Dispatch With Explicit Inputs | `37a776b3`, `48f7bfac` | Explicit marker/XML lowering remains; rejected Rust HTML pre-parser is absent |
| `T03` Preserve Vertical Line-Band Phase | `7f6a0657`, `150a379a` | Canonical census path and vertical 24px line-band projection pass |
| `T04` Derive Float Continuation And Terminal Extent | `515b712c`, `90c7e861` | Float continuation and terminal geometry pass within T04 scope |

Each task's recorded RED/GREEN, focused/default gates, independent task review,
and commit validation remain authoritative.

### 5.5 `P01/I06/S01/C12/T05` Capture The Diagnostic Lineage Failure

**Files/area:** historical span `90c7e861..0bbfbc04`, whose T05 code commits are
exactly `5ba51d3f`, `78876ebe`, and `0bbfbc04`; focused
generator/parser/helper/comparator tests; uncommitted diagnostic residue. No new
code, generation, or commit.

**Outcome:** Preserve the input freeze, zero-width whitespace, and empty Range
schema corrections. Record the structurally successful full run as diagnostic,
not final, because 314/388 activation rows fail.

**RED:** The preflight failed on two zero-fragment whitespace sources; the first
replacement failed on empty `rangeInks`; the structurally successful replacement
then failed the exact activation union at 314 rows. These historical failures and
their intervening focused GREEN commits are the task's TDD evidence.

**Acceptance:** Report has `filter:null`, 5,712 generated, 16 missing-root
unsupported, zero expected-fail/quarantine/generation failures, report SHA
`91b37caf03dcb418def6ecf45ebccc50705ab323c153a9b7f2cba7e9522f0028`,
XML aggregate
`19eba32b808f51e08fafd106878b3f93dee9e8e0bf3cef8580851fb0e7be5cc3`,
and activation aggregate
`3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1`.
Focused GREEN, corpus/Taffy, and format checks pass; the exact activation and
full verification failures are retained. No generated residue is committed.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c08r_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c08r_
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
```

The layout command retains one aggregate RED; `verify` retains six and
`verify-generator` three. Every other result matches the recorded T05 evidence.
T05 creates no additional commit.

**Dependency:** T01-T04 are task-clean.

### 5.7 `P01/I06/S01/C12/T07` Restore Honest Fixture And Comparator Inputs

**Files/area:** the exact 61 HTML sources/363 direct BR-parent divs in the
reviewed specification; `fri06_bidi_mixed_inline.html`; browser helper;
`support.rs`; narrow serializer/accounting tests in the generator; focused
browser-parity tests; T05-owned uncommitted provenance edits in `generator.rs`,
all 5,712 generated XML paths under `tests/layout/browser_parity/xml/`, and
`xml/generation-reports/all.json`. No production, base style, manifest,
dependency, feature, or generator architecture.

**Outcome:** Author `display:block` on every audited default-block parent; add
and validate the one source-indexed nonzero bidi marker; make level zero the
layout-ready-inline adapter rule without reading computed direction; keep typed
inline boundaries expectation-transparent; and normalize Range line identity to
the explicit containing root's physical block coordinates with 0.1px tolerance.
Remove T05 generated residue only after its hashes/evidence are read back.

**RED:** Focused tests prove unauthored parents compute flex, direction changes
currently change bidi input, boundaries inflate expected child counts, and local
wrapper Range lines disagree with root-local line identity.

**Acceptance:** Chrome computes all 363 corrected parents as block and their
pinned WPT target roles remain ordinary boxes. Marker syntax, range, target,
uniqueness, and consumption reject malformed input. Renaming and
expectation-only mutation preserve normalized input; direction-only changes do
not choose bidi levels. The 28 explicit-adapter/comparator rows pass without
source-name dispatch or Rust HTML/CSS parsing. Focused/default/generator checks
are clean except for the exact corrected final-activation aggregates retained as
diagnostic RED; format passes and no full generation runs.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t07_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t07_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

Before edits, read back the recorded T05 hashes, then restore the 5,324 modified
and 388 untracked XML files, report, and uncommitted provenance-only generator
delta to `0bbfbc04`. Prove no diagnostic residue remains before T07's authored
changes. The two full verification commands may fail only at the exact final
activation aggregates; their corrected census is T07 diagnostic output.

**Diagnostic:** After the committed task is clean, scoped generation may be used
only to census the changed 388-row activation union. Each run must follow a real
input/adapter iteration and must not be repeated over unchanged files. Its
output is diagnostic, is cleaned after hashes and categories are recorded, and
is never verification or lineage evidence.

**Dependency:** T05 is task-clean.

**Intended commit:** `fix(parity): restore honest inline fixture inputs`.

## 6 Current Revision Completion

This revision exits only when:

1. T05 has a clean independent task review, its recorded hashes are read back,
   and all T05 XML/report/provenance residue is removed before T07 edits;
2. T07 is committed and independently task-clean against the exact task contract;
3. the focused generator and parity tests pass, both configured verification
   matrices retain only the exact corrected final-activation aggregate RED, format
   and diff checks pass, and the latest changed-input scoped diagnostic records
   its input commit, filter, report hash, complete 388-row partition, and exact
   remaining production categories; and
4. every scoped diagnostic output is removed after that evidence is recorded, so
   no generated XML, report, or provenance residue remains.

The final evidence commands for this revision are:

```sh
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate fri06_c12_t07_
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout --test layout fri06_c12_t07_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just fmt-check
git diff --check
```

The handoff is the clean T05 verdict, exact T07 task range and clean verdict,
command evidence, and cleaned scoped-diagnostic census. This state does not mark
C12 complete and grants no final generation, holistic review, publication, or
candidate-handoff authority.

## 7 Successive Planning Gate

After the current-revision handoff, amend and independently review this plan with
only the exact remaining production categories and tasks proved by the cleaned
diagnostic census. Do not create placeholder tasks. The later reviewed amendment
owns one final full unfiltered pinned-browser generation after all
HTML/helper/parser/fixture inputs and production code settle, then the complete
verification, independent task reviews, holistic review, publication, remote
readback, closure, and handoff gates.

Blocker: current T05 residue is diagnostic and awaits reviewed replacement.
