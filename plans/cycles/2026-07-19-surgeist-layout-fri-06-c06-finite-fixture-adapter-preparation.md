# FRI-06-C06 Finite Fixture Adapter Preparation

Status: complete

Cycle ID: `FRI-06-C06`

Owning repository: `surgeist-layout`

Cycle base: `6e3772f509b919ec9a9d027d8298600ed98ee531`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`7090ea13ba7d9e524ce432018c8b7c44c1b3b76428d2c666949d297656ce97c8`,
commit `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`, sections
`FRI-06.4 D-16`, browser/comparator portions of `FRI-06.9` and
`FRI-06.10`, finite adapter portions of `FRI-06.11`, and adapter evidence
portions of `FRI-06.14`.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`2cda27e19919abaef075600816d320d87a36d36be6c79bd27c2962684c66467b`,
commit `fde445a0ff74f8359bcf8c512e2cf9e331de36e6`, entry
`FRI-06-C06`.

## Outcome

Complete and publish the finite Rust browser-fixture adapter that compares
control and fragment output and lowers shaped text, atomic participation, bottom
alignment, and finite shape-band tables through production constructors and
front doors. Align one stale generator lifecycle test with the existing explicit
lease-release front door. Keep generator implementation, every generation input,
and every derived artifact unchanged.

## Boundary

FRI-06-C05 and the post-C05 containment window are published and remotely
verified at the cycle base. The three adapter tasks below are already committed,
independently task-clean, and locally verified. This reconciled plan preserves
their exact ranges under the reviewed sequence without changing their contracts
or implementation.

An attempted later derivation produced no commit and was invalidated when its
read-only parity diagnostic exposed the recovery matrices now owned by C07 and
C08 in the reviewed sequence. All 5,730 owned uncommitted candidate paths were
discarded. No helper, generator, HTML, manifest, XML, report, or production
`src/` change from that attempt remains.

The invalid attempt's test-only RED sidecar was compare-cleaned at exact SHA
`5ce8ae0f17da9c86f7141e11ec60ac0b1a87c61d`. Its former worktree
`/private/tmp/surgeist-layout-FRI06-C06-T4-worker-01-red.G14eje/worktree`
and `refs/surgeist/FRI06-C06-T4-worker-01/red` are absent. C07 creates fresh
production RED evidence at its published base; C08 creates fresh fixture RED
evidence at its later published base.

After the first holistic review returned `CLEAN`, the required final local
generator-feature gate exposed a pre-existing intermittent lifecycle-test defect.
`generation_lock_rejects_an_independent_handle_with_owner_metadata_then_releases`
failed twice at the same reacquisition after implicit `File` drop: once in
`just verify-generator` and once on the seventeenth bounded parallel-binary run
after sixteen passes. The focused test passed 100 consecutive process-isolated
runs. Source and history show `run_from_env` and the nearest cloned-descriptor
test deliberately use `GenerationLease::release()` after earlier explicit-unlock
fixes; only this older test still substitutes `drop(first)`. T4 corrects that
test contract and does not change the lease implementation.

Non-goals are production `src/`; helper, generator implementation, HTML,
manifest, XML, and report changes; corpus-generator execution of any scope;
dependencies, features, lockfile, MSRV, browser policy, task runner, base style,
root, siblings, docs, public API, expected failures, quarantines, FRI-09 through
FRI-13 behavior, and generator architecture. The only generator-owned path
permitted is the exact T4 lifecycle test in
`tests/bin/surgeist-layout-generate/generator.rs`. `just verify-generator` is
read-only Cargo feature verification and does not execute the corpus generator.

## Impacts

- **Public API and production behavior:** unchanged.
- **Dependencies, features, lockfile, MSRV, and browser policy:** unchanged.
- **Generated inputs and artifacts:** unchanged; generator implementation is
  unchanged and no corpus-generator command is allowed. One lifecycle test uses
  the existing explicit release front door. Read-only generator-feature
  verification remains required.
- **Docs/examples and root follow-up:** unchanged in C06.
- **Unsafe and lint policy:** no executable unsafe and no new `allow` or
  `expect` attribute in tracked or non-ignored owned Rust.

## Tasks

### `C06-T1` Compare Control And Fragment Output Exactly

**Files:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused comparator/parser tests.

**Outcome:** Extend the finite expectation model and comparator to observe line
break/control geometry and final inline fragments, including source segment ID,
line index, visual index, physical rect, and baseline. Preserve ordinary node and
scroll comparisons.

**RED:** `fri06_c06_comparator_` tests prove wrong or missing control geometry
and wrong or missing fragment rect, source ID, line index, visual index, and
baseline were previously unobserved.

**Acceptance:** Correct control and zero geometry pass. Each wrong or missing
field fails with a stable field-specific diagnostic before unrelated child
comparison. Fragment comparison uses final batch output in source association
order, preserves intervening visual slots, distinguishes valid empty output from
absent state, and applies scalar tolerance only to numeric geometry/baselines.
Existing XML without fragment expectations retains its prior meaning.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_comparator_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** published C05 output and fragment contracts.

**Task range:** ordered spans
`5c13dd8cfa4ff57884c3bbb988a5c806b165c849..69e9d437810d0b5b2cc7a14eda5e7e5d49afe5c1`
and
`69e9d437810d0b5b2cc7a14eda5e7e5d49afe5c1..a04ac60d228ec4f1388c66e6f18f3cc0703aa0e1`;
fresh full-range task review returned `CLEAN`.

**Intended commits:** `test(parity): compare inline control and fragment output`
and focused review fix `test(parity): enforce strict fragment expectations`.

### `C06-T2` Lower Finite Shaped And Atomic Fixture Inputs

**Files:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused fixture-adapter tests.

**Outcome:** Parse only reviewed shaped-segment, text-metric, bidi, whitespace,
following-break, and atomic-placeholder facts and lower them through production
`InlineTextInput`, canonical non-box, and atomic participation constructors.
Store phase-correct fragment slices for cold, warm, and rounded comparison.

**RED:** `fri06_c06_inline_input_` tests prove the fixture tree previously could
not construct shaped text, canonical text pairing, atomic participation, or
committed fragment readback.

**Acceptance:** Strict parsing rejects empty/duplicate IDs, partial tuples,
non-finite metrics, invalid choices, unmatched atomic indices, contradictory box
fields/children/measurement, and unknown attributes. Valid text is
`LayoutInput::InlineText`. Atomic facts bind one existing child in source order.
Committed empty and nonempty fragment slices republish through the real tree
method; no authored text, glyph, font backend, shaper, or permissive fallback is
added.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_inline_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T1's final-fragment expectation model.

**Task range:**
`a04ac60d228ec4f1388c66e6f18f3cc0703aa0e1..dab8b10e339842bd7a2d0e7740bab92dbe2f5aa1`;
fresh task review returned `CLEAN`.

**Intended commit:** `test(parity): lower shaped inline fixture inputs`.

### `C06-T3` Lower Bottom Alignment And Finite Shape Bands

**Files:** `tests/layout/browser_parity/support.rs`,
`tests/layout/browser_parity.rs`, and focused adapter/provider tests.

**Outcome:** Add exact `vertical-align: bottom` lowering and a fixture-only
finite physical shape-band table consumed through the production exclusion
provider method.

**RED:** `fri06_c06_shape_input_` tests prove bottom was rejected and the
fixture tree lacked validated `FloatExclusion::Shape` query/result/provider
input and typed missing/mismatched/failure paths.

**Acceptance:** Bottom maps only to `VerticalAlign::Bottom`. Strict finite
physical query bands and intervals validate through production constructors,
retain exact query association, and bind only to visible in-flow left/right shape
floats. Empty, partial, full, missing, mismatched, and failed results use the real
provider and compute front doors. No shape syntax, path geometry, general parser,
alternate band algorithm, or precomputed line position is added.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_shape_input_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T2's complete finite fixture tree.

**Task range:**
`dab8b10e339842bd7a2d0e7740bab92dbe2f5aa1..a49c6c2bd81f27b2962928a31ae6126b26b211fe`;
fresh task review returned `CLEAN`.

**Intended commit:** `test(parity): lower finite shape fixture inputs`.

### `C06-T4` Align Lease Lifecycle Test With Explicit Release

**Files:** the exact lifecycle test in
`tests/bin/surgeist-layout-generate/generator.rs`.

**Outcome:** Make the owner-metadata and reacquisition test exercise the same
explicit `GenerationLease::release()` lifecycle used by `run_from_env` and the
stable cloned-descriptor regression.

**RED:** Final `just verify-generator` failed at reacquisition after
`drop(first)`. A bounded parallel-binary diagnostic reproduced the same failure
on run 17 after 16 passes, while 100 isolated focused runs passed. The mismatch
is test lifecycle, not generator output or implementation behavior.

**Acceptance:** Preserve the independent-handle rejection and exact owner
metadata assertions. Release through the existing fallible front door before
reacquisition, retain both lock files, and keep production generator code,
inputs, outputs, and architecture byte-identical. The focused lifecycle test
and full read-only generator-feature verification pass.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate generator::tests::generation_lock_rejects_an_independent_handle_with_owner_metadata_then_releases -- --exact
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T1-T3 task-clean, initial holistic review clean, and the
subsequent final-gate diagnosis above.

**Intended commit:** `test(generator): exercise explicit lease release`.

## Completion

All four task ranges must retain clean reviews. T4 changes only the stale
generator lifecycle test; generator implementation and every generation input
and artifact remain unchanged.

Run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --test layout fri06_c06_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
cargo fmt --check
git diff --check 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD
git diff --quiet -G'(^|[^.[:alnum:]_])(allow|expect)[[:space:]]*\(' 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD -- '*.rs'
test "$(git diff --name-only --no-renames 6e3772f509b919ec9a9d027d8298600ed98ee531..HEAD)" = $'plans/cycles/2026-07-19-surgeist-layout-fri-06-c06-finite-fixture-adapter-preparation.md\nplans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md\ntests/bin/surgeist-layout-generate/generator.rs\ntests/layout/browser_parity/support.rs'
test ! -e /private/tmp/surgeist-layout-FRI06-C06-T4-worker-01-red.G14eje/worktree
test -z "$(git for-each-ref --format='%(refname)' refs/surgeist)"
test "$(git worktree list --porcelain | rg -c '^worktree ')" -eq 1
test -z "$(git status --porcelain)"
zsh <<'SURGEIST_C06_SAFETY'
set -eu
owned_rust=("${(@f)$(git ls-files --cached --others --exclude-standard '*.rs')}")
test "${#owned_rust[@]}" -gt 0
if rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' "${owned_rust[@]}"; then
  exit 1
else
  test "$?" -eq 1
fi
SURGEIST_C06_SAFETY
```

The pickaxe and owned-Rust gates fail on a match or tool error. The exact
changed-path comparison permits only the reviewed FRI-06 sequence, this
reconciled C06 plan path, the exact T4 generator lifecycle test file, and
`tests/layout/browser_parity/support.rs`. Production `src/`, generator
implementation/helper/HTML, manifest/XML/report, Cargo/lockfile, docs, root,
sibling, task-runner, and unrelated paths fail completion. The worktree/ref
predicates prove the invalid-attempt resources remain absent and only the
canonical worktree remains.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`6e3772f509b919ec9a9d027d8298600ed98ee531..cycle_head`. Rerun the complete
read-only set on local `main`, publish the immutable cycle head to authority
remote `main` with a leased fast-forward, fetch/read back, and prove local
`main`, its tracking ref, `FETCH_HEAD`, and live remote `main` agree.
Require the recorded temporary-resource absence predicates to pass.

The handoff is the published finite adapter candidate plus the reviewed sequence
recovery matrices. C07 owns only the 72 production rows; C08 later owns the 256
fixture rows, 16 semantic-preservation rows, 388 focused parity rows, and one
valid final derivation. Blocker: none.
