# P01-I06-S01-C03 Post-C02 Sprawl Containment

Status: complete

Cycle ID: `P01/I06/S01/C03`

Owning repository: `surgeist-layout`

Cycle base: `350dded41c45fc3f4638d2d93214ce741be7c8bf`

Reviewed specification:
`plans/P01-layout/P01-I06-S01-C03-post-c02-sprawl-containment.md`
at normalized SHA-256
`49dddccabefc30a693eb9452bbdeb3b55ec6177be060d4d7d870b84d920990c1`,
commit `5b8107a76`, sections `FRI-06-MR01.1` through `FRI-06-MR01.7`.

Implementation sequence: none; this is a one-cycle, single-repository
sub-initiative.

## 1 Outcome

Remove the two validated post-C02 duplication points without changing behavior:
centralize the ordered non-box role classification used by root-tree validation,
and replace the two scalar-specific oracle `Compute` bodies with one
`LayoutScalar`-generic implementation. Publish the reviewed descendant before
P01/I06/S01/C04 begins.

## 2 Boundary

The cycle starts from clean, remotely verified `main` after C02 and the separate
mechanical-review evidence commit. The reviewed source evidence establishes that
the duplicated non-box branches have the same three checks and reason payload,
and that the two oracle `Compute` bodies differ only in scalar spelling.

This cycle owns only `src/compute.rs`, focused existing Rust test modules,
`src/test_support/layout_tree.rs`, this specification, and this cycle plan. It may
add focused characterization tests but may not alter intended layout, validation,
error, cache, transaction, panic, or harness behavior.

Public API, production tree abstractions, broad test-harness migration, local
specialized fakes, macros, P01/I06/S01/C04 participants, shaped-text optimization,
float/geometry/scalar/math consolidation, authored inputs, parser, helpers,
fixtures, HTML, XML, reports, provenance, manifests, dependencies, features,
lockfile, root, siblings, and generated artifacts are excluded. No generator
command runs. No new lint allowance or Surgeist-owned `unsafe` is permitted.

Behavior-preserving work uses passing characterization at the exact task base;
it does not fabricate RED evidence. Each worker records the test-only patch and
the passing pre-change command before modifying the implementation.

## 3 Impacts

Public API and compatibility: internal-only, behavior-preserving. Dependencies,
features, lockfile, MSRV, docs/examples, root follow-up, generated artifacts, and
fixture lineage: unchanged. Generator architecture and execution: absent. Owned
Rust remains free of `unsafe` and new lint allowances.

## 4 Tasks

### 4.1 `P01/I06/S01/C03/T01` Preserve Ordered Non-Box Role Classification Through One Helper

**Files:** `src/compute.rs` and focused existing contract/root Rust tests only.

**Outcome:** Add focused `fri06_mr01_non_box_` characterization for inline text,
line breaks, and inline boundaries in both scalar lanes. Prove the exact
`NonCanonicalNodeInput`, then `HasChildren`, then `HasLeafMeasurement` precedence
when invalid states compete. Extract one private
`non_box_node_role_error<Tree>` classifier returning
`Option<NonBoxNodeRoleError>`, and have both existing branches consume it while
retaining their current error construction, node site, return point, and
inline-text parent rule.

**Pre-change characterization:** Apply only the focused test changes at exact task
base `5b8107a76` plus the cycle-plan definition and status commits, record the
test-only diff digest, and run the focused command. It must pass before the helper
exists because the task preserves behavior. Do not claim RED. Remove no coverage
to make the extraction pass.

**Acceptance:** All nine single/competing invalid-state families across the three
roles pass in `f32` and `f64`; every error keeps operation `RootLayout`, the
subject node site, `InvalidInput(NonBoxNodeRole { reason })`, and first-error
ordering. Inline text still applies parent acceptance only after shared
validation. Line breaks and boundaries still return immediately after valid
non-box validation. One helper contains the three checks in their original
order; neither branch duplicates them; no box, atomic, or parent-role validation
enters the helper.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr01_non_box_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'fn non_box_node_role_error' src/compute.rs
```

**Dependency:** Reviewed cycle plan and `in_progress` status. This task precedes
T02 so the validation extraction has its own exact range and review.

**Intended commit:** `refactor(layout): centralize non-box role validation`.

### 4.2 `P01/I06/S01/C03/T02` Preserve Oracle Behavior Through One Scalar-Generic Compute Impl

**Files:** `src/test_support/layout_tree.rs` and focused existing Rust test modules
only.

**Outcome:** Add focused `fri06_mr01_oracle_generic_` characterization that runs
the same typed helper for `f32` and `f64`. Prove input recording, recorded
measurement precedence, hidden output staging, and representative algorithm
dispatch. Replace only `impl Compute for OracleTree` and
`impl Compute for OracleTreeOf<f64>` with
`impl<S: LayoutScalar> Compute for OracleTreeOf<S>`.

**Pre-change characterization:** Apply only the focused test changes at the exact
post-T01 task base, record the test-only diff digest, and run the focused command.
It must pass through both existing scalar-specific implementations before the
generic body replaces them. Do not claim RED.

**Acceptance:** Both supported scalars preserve stored node/layout input lookup,
exact missing/non-box panic text, unrounded output storage, compute-input
recording before result selection, recorded-measurement precedence, block/flex/
grid/grid-lanes dispatch, hidden zero-source staging, and unreachable inline
display handling. Exactly one generic oracle `Compute` implementation remains;
both scalar-specific implementations are absent. The alias, builders, fields,
`Traverse`, `Round`, call sites, local fakes, and broader harness inventory are
unchanged. No macro, new trait, dynamic dispatch, production API, or migration is
introduced.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr01_oracle_generic_
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true just fmt-check
rg -n '^impl<S: LayoutScalar> Compute for OracleTreeOf<S>' src/test_support/layout_tree.rs
rg -n '^impl Compute for OracleTree|^impl Compute for OracleTreeOf<f64>' src/test_support/layout_tree.rs
```

The final `rg` command must return no matches; that exit status is expected and
is recorded as a static absence check, not a failed verification gate.

**Dependency:** T01 implementation and independent task review are clean.

**Intended commit:** `refactor(layout): generalize oracle compute harness`.

## 5 Completion

Cycle acceptance requires both task ranges to have clean independent reviews and
the plan to be committed `complete` in a status-only commit. At the immutable
cycle head, run:

```sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_mr01_
CARGO_NET_OFFLINE=true just fmt-check
rg -n 'fn non_box_node_role_error' src/compute.rs
rg -n '^impl<S: LayoutScalar> Compute for OracleTreeOf<S>' src/test_support/layout_tree.rs
rg -n '^impl Compute for OracleTree|^impl Compute for OracleTreeOf<f64>' src/test_support/layout_tree.rs
git diff --check 350dded41c45fc3f4638d2d93214ce741be7c8bf..HEAD
git diff --name-only 350dded41c45fc3f4638d2d93214ce741be7c8bf..HEAD
git ls-files --cached --others --exclude-standard '*.rs' > /tmp/surgeist-layout-fri06-mr01-owned-rust.txt
rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{' $(cat /tmp/surgeist-layout-fri06-mr01-owned-rust.txt)
rm /tmp/surgeist-layout-fri06-mr01-owned-rust.txt
git status --short
```

The scalar-specific-impl `rg` must return no matches; every other named static
search must return the expected owned declaration. The unsafe `rg` must scan the
complete manifest and return no executable matches; its expected no-match exit
status is recorded and any textual match is classified before completion. The
manifest must contain tracked and non-ignored untracked owned Rust, including
test and generator Rust, while naturally excluding ignored build/dependency
roots, and must be removed after the scan. The exact inventory may
contain only this specification, this cycle plan, `src/compute.rs`,
`src/test_support/layout_tree.rs`, and focused existing Rust test modules. Scan
all manifest entries and require zero executable `unsafe` matches.

No generator, generator-feature, browser, parity, fixture, or corpus command runs
because no input, output, feature, or artifact in those domains changes. Any
unexpected change or failing final check returns the plan to `in_progress` and
routes the correction through a fresh worker and task reviewer.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`350dded41c45fc3f4638d2d93214ce741be7c8bf..cycle_head`. Then land the immutable
candidate on local `main`, rerun required gates there, push by fast-forward to the
authority remote `main`, fetch and read the remote branch back, and prove local
`main`, `origin/main`, and observed remote `main` agree with the candidate or a
normal descendant containing it. Remove every cycle-owned temporary resource.

The handoff records the published candidate as P01/I06/S01/C04's base and preserves
the later mechanical windows without beginning them. Blocker: none at planning
time.
