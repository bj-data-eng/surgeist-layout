# P01/I08/S02/R06A Planning-Path Dylint Audit Catalog

Cycle ID: `P01/I08/S02/R06A`

Owning repository: `surgeist-layout`

Status: `in_progress`

Cycle base: `20ad8202e536c4c63f0bd211f0872653462116bf`

Specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`,
reviewed semantic SHA-256
`f12c1aed35aabcb35231cca372eca6381daff57bfe2b6a053679bcf2f4d2d94f`,
commit `a7ade7927e053ab8114bb49697d1493f578242d2`, sections `FRI-08.20`
row `AR-008`, `FRI-08.21`, `FRI-08.27.1`, all of `FRI-08.27.2`, and
acceptance rows `FRI-08.28(1)`, `FRI-08.28(7)`, and `FRI-08.28(10)` through
`FRI-08.28(12)`.

Sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`,
reviewed semantic SHA-256
`fa6b9ff466b2d61053ddb2961602671f64d19cdf5bb5efa2fc2a19f1a448b284`,
commit `a372699c9301893a385890ed8c6c59178bf08891`, entry
`P01/I08/S02/R06A`.

Bounded outcome: one leaf-local, package-excluded, opt-in Dylint catalog
retains the planning-path-named node-projection semantic audit, replaces the
superseded lexical script, and records pilot lessons without entering product
tests, standing commands, CI, publication gates, or permanent architecture
policy.

## 1 Boundary And Impacts

The immutable entry is published `main` at
`20ad8202e536c4c63f0bd211f0872653462116bf`, with clean process state and no
repository `target/`. On 2026-08-14 the user explicitly authorized acquisition
of exactly `cargo-dylint` `6.0.3`, `dylint-link` `6.0.3`, crates.io catalog
dependencies `dylint_linting = "=6.0.3"` and
`dylint_testing = "=6.0.3"`, and Rust `nightly-2026-05-28` with `rustc-dev`
and `llvm-tools-preview` from the official rustup distribution, solely for this
audit-tooling pilot.

T01's initial preflight preserved RED evidence that this exact stack, catalog,
and package exclusion were absent. That attempt completed only the authorized
user-scoped installations before the original direct-binary version probes were
shown incompatible with Dylint `6.0.3`; the repository returned clean without a
task commit. T01 resumes by verifying the installed packages through Cargo's
package inventory and must not uninstall or reacquire them merely to repeat RED.

On 2026-08-14 the user additionally authorized Dylint `6.0.3`'s one-time
crates.io acquisition and local build of exact `dylint_driver = "=6.0.3"` and
its Cargo-resolved transitive dependencies. Dylint may create its ephemeral
toolchain wrapper package
`dylint_driver-nightly-2026-05-28-aarch64-apple-darwin` `0.1.0` and persist only
the resulting driver beneath the configured `DYLINT_DRIVER_PATH` or default
user Dylint-driver directory. This authority does not permit another driver
version, toolchain, catalog dependency, lockfile update, or repository artifact.

The catalog lives only at `tools/surgeist-layout-audits/`, has its own
workspace, lockfile, and toolchain, and writes all build state to
`target/dylint-audits` at the repository root. The product package excludes the
catalog path. Product dependencies, features, `Cargo.lock`, MSRV, targets,
public API, behavior, fixtures, and generated artifacts remain unchanged.

The catalog lint is `Allow` by default. Only an explicit coordinator or
reviewer invocation may select it with `-D`; workers run only default-Allow
catalog build, test, Clippy, and formatting commands. It is not added to `just`, CI,
ordinary Cargo/Clippy commands, publication gates, or product tests. The lint
records a historical planning question; a later intentional architecture
change may reinterpret or retire it.

Repository-authored Rust, including catalog source and UI fixtures, contains no
unsafe construct. The external `dylint_linting` macro alone owns Dylint's
dynamic-library ABI expansion. If the pinned stack requires any other authored
or retained unsafe construct, implementation stops. No Git dependency,
`clippy_utils`, wrapper script, source-parsing Rust test, generated artifact,
browser execution, corpus acquisition, or shared skill-reference work belongs
to this cycle.

## 2 Tasks

### 2.1 `P01/I08/S02/R06A/T01` Pin And Isolate The Audit Catalog

**Area:** product `Cargo.toml`; new
`tools/surgeist-layout-audits/{Cargo.toml,Cargo.lock,rust-toolchain.toml,src/lib.rs}`;
user-scoped Cargo and rustup installations authorized above.

**Outcome:** verify the already installed exact authorized
binaries/toolchain/components; create package `surgeist-layout-audits`, library
`surgeist_layout_audits`,
`publish = false`, `crate-type = ["cdylib"]`, its own `[workspace]`, and exact
crates.io pins; exclude the tool tree from product packaging; keep every catalog
command on repository-root `target/dylint-audits` with no nested target.

**RED/acceptance:** retained historical RED proves the exact binaries, nightly,
components, catalog, and package exclusion were absent at cycle entry. The
current retry first proves the installed-package/toolchain inventory is GREEN
without reacquisition, then remains RED only because the catalog and package
exclusion are absent. After scaffolding, exact
version/component/package/lock/isolation probes pass. Product `Cargo.lock`,
dependency graph, features, MSRV, and ordinary stable package checks remain
unchanged.

Current retry RED:

```sh
test ! -e tools/surgeist-layout-audits
if rg -q '^exclude = \["tools/surgeist-layout-audits/\*\*"\]$' Cargo.toml; then exit 1; fi
```

```sh
set -e
test "$(cargo dylint --version)" = 'cargo-dylint 6.0.3'
cargo install --list | perl -0ne 'exit !(/^cargo-dylint v6\.0\.3:\n    cargo-dylint$/m && /^dylint-link v6\.0\.3:\n    dylint-link$/m)'
rustup toolchain list | rg -q '^nightly-2026-05-28-'
rustup component list --toolchain nightly-2026-05-28 --installed | rg -q '^rustc-dev-'
rustup component list --toolchain nightly-2026-05-28 --installed | rg -q '^llvm-tools-'
rg -q '^name = "surgeist-layout-audits"$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^publish = false$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^name = "surgeist_layout_audits"$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^crate-type = \["cdylib"\]$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^dylint_linting = "=6\.0\.3"$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^dylint_testing = "=6\.0\.3"$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^\[workspace\]$' tools/surgeist-layout-audits/Cargo.toml
rg -q '^channel = "nightly-2026-05-28"$' tools/surgeist-layout-audits/rust-toolchain.toml
rg -q '^components = \["rustc-dev", "llvm-tools-preview"\]$' tools/surgeist-layout-audits/rust-toolchain.toml
rg -q '^profile = "minimal"$' tools/surgeist-layout-audits/rust-toolchain.toml
cargo +nightly-2026-05-28 generate-lockfile --manifest-path tools/surgeist-layout-audits/Cargo.toml
test -f tools/surgeist-layout-audits/Cargo.lock
perl -0777 -ne 'exit !(/\[\[package\]\]\nname = "dylint_linting"\nversion = "6\.0\.3"\nsource = "registry\+https:\/\/github\.com\/rust-lang\/crates\.io-index"/s && /\[\[package\]\]\nname = "dylint_testing"\nversion = "6\.0\.3"\nsource = "registry\+https:\/\/github\.com\/rust-lang\/crates\.io-index"/s)' tools/surgeist-layout-audits/Cargo.lock
CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo +nightly-2026-05-28 check --locked --manifest-path tools/surgeist-layout-audits/Cargo.toml
test ! -e tools/surgeist-layout-audits/target
package_files="$(CARGO_NET_OFFLINE=true cargo package --locked --offline --allow-dirty --list)"; test -z "$(printf '%s\n' "$package_files" | rg '^tools/surgeist-layout-audits/' || true)"
rg -q '^exclude = \["tools/surgeist-layout-audits/\*\*"\]$' Cargo.toml
diff -u <(git show 20ad8202e536c4c63f0bd211f0872653462116bf:Cargo.toml | awk '1; /^description = / { print "exclude = [\"tools/surgeist-layout-audits/**\"]" }') Cargo.toml
test "$(git diff 20ad8202e536c4c63f0bd211f0872653462116bf -- Cargo.lock | wc -l | tr -d ' ')" = 0
CARGO_NET_OFFLINE=true cargo check --locked --offline -p surgeist-layout
cargo fmt --check; git diff --check
```

Dependency: published R06 plus exact user acquisition authority. Commit:
`tooling(audit): pin isolated Dylint catalog`.

### 2.2 `P01/I08/S02/R06A/T02` Implement The Node-Projection Semantic Audit

**Area:** catalog lint source, catalog-owned documentation, and isolated Dylint
UI fixtures only.

**Outcome:** implement `p01_i08_s02_r06_t02_node_projection_boundary` at
`Allow`. It inspects production HIR in the `block`, `inline`, `flex`, `grid`, and
`scroll` module trees; permits aggregate borrowing only in the six specified
projection-construction owners; and reports resolved aggregate identities,
aliases, visibility reexports, direct/UFCS/extracted `LayoutTree::node_input`
uses, and newly compiled descendants everywhere else. It uses compiler identity,
not fixed consumer paths or lexical masking.

Catalog documentation records the exact originating R06 plan/revision, audit
question, original scope, interpretation after later architecture changes, and
the superseded script. UI fixtures exercise allowed owner construction and each
rejected semantic escape, including aliases, reexports, UFCS, extracted method
items, nested modules, ordinary strings/comments, and both protected aggregate
families. Product tests do not inspect source.

**RED/acceptance:** add the smallest UI fixtures first and prove the absent lint
or missing diagnostic is RED. The final selected UI suite passes with exact
diagnostics, the lint remains `Allow` when not explicitly selected, catalog
format and strict compiler checks pass with already-installed tooling, and
authored catalog Rust has no unsafe match.

The first UI attempt established only a setup failure because the authorized
toolchain driver was absent; it is not RED evidence. Before resuming test-first
work, run the catalog UI test once with network access solely so Dylint can
acquire and build the newly authorized exact driver. Prove the driver exists and
the catalog manifest, lockfile, and toolchain pin are unchanged, then rerun the
focused UI test locked and offline. Only that offline missing-diagnostic failure
is the valid T02 RED.

One-time authorized driver bootstrap:

```sh
driver_root="${DYLINT_DRIVER_PATH:-${HOME}/.dylint_drivers}"
driver_path="$driver_root/nightly-2026-05-28-aarch64-apple-darwin/dylint-driver"
test ! -e "$driver_path"
CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --manifest-path tools/surgeist-layout-audits/Cargo.toml node_projection_boundary_ui
test -x "$driver_path"
test "$("$driver_path" -V | awk '{print $NF}')" = '6.0.3'
git diff --exit-code bfd76dab2c52df5ec009f52595fba9ce6e5ac6e2 -- tools/surgeist-layout-audits/Cargo.toml tools/surgeist-layout-audits/Cargo.lock tools/surgeist-layout-audits/rust-toolchain.toml
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml node_projection_boundary_ui
```

```sh
set -e
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" RUSTFLAGS='-F unsafe-code -D warnings' cargo +nightly-2026-05-28 check --locked --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml --all-targets
cargo fmt --manifest-path tools/surgeist-layout-audits/Cargo.toml --check
test ! -e tools/surgeist-layout-audits/target
git diff --check
```

Dependency: T01. Commit:
`tooling(audit): detect node projection boundary escapes`.

### 2.3 `P01/I08/S02/R06A/T03` Audit R06 And Retire The Lexical Script

**Area:** delete `scripts/audit-node-projection-boundaries.sh`; catalog reference
material and UI evidence; exact historical plan/source references only where
script deletion requires reconciliation.

**Coordinator transition:** after T02 is independently CLEAN and before T03 is
dispatched, the coordinator runs the explicitly selected lint against the
unchanged R06 product source and records zero diagnostics with this exact sole
selected invocation:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo dylint --path tools/surgeist-layout-audits -- -D p01_i08_s02_r06_t02_node_projection_boundary
```

A failure returns to T02; a worker never performs or repeats this selected-lint
invocation, and it does not recur in the final matrix.

**Outcome:** consume the coordinator's zero-diagnostic semantic audit evidence
and delete the superseded lexical script.
Record concrete pilot lessons covering setup/pins, HIR identity resolution,
fixture design, diagnostics, maintenance, review, opt-in interpretation, and the
boundary for a later separately authorized shared skill reference. No replacement
script or standing command is added.

**RED/acceptance:** before deletion, coordinator evidence proves the selected
semantic lint passes and the script exists; after deletion, the default-Allow
isolated UI suite passes, no standing command references the catalog, and product
API/behavior/package/artifact evidence is identical to the R06 entry. The
coordinator's recorded transition result is the cycle's sole selected semantic
audit and is not repeated.

```sh
set -e
test ! -e scripts/audit-node-projection-boundaries.sh
test ! -e tools/surgeist-layout-audits/target
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml
CARGO_NET_OFFLINE=true cargo test --locked --offline -p surgeist-layout
CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check; git diff --check
```

Dependency: T02. Commit:
`tooling(audit): replace lexical projection audit`.

## 3 Completion

R06A requires three independently CLEAN task ranges, status `complete`, a GREEN
final matrix, CLEAN holistic review, publication/readback, process hygiene,
successful repository-root `cargo clean`, absence of both target paths, and an
immutable R07/R08 handoff. No catalog output, product dependency/API/behavior,
fixture, generated artifact, browser, generator, acquisition beyond the exact
authorized stack, or shared skill-reference delta is permitted.

The coordinator's single selected-lint result recorded between T02 and T03 is
immutable cycle evidence. The final matrix does not repeat that invocation.

```sh
set -e
test "$(cargo dylint --version)" = 'cargo-dylint 6.0.3'
cargo install --list | perl -0ne 'exit !(/^cargo-dylint v6\.0\.3:\n    cargo-dylint$/m && /^dylint-link v6\.0\.3:\n    dylint-link$/m)'
rustup component list --toolchain nightly-2026-05-28 --installed | rg -q '^rustc-dev-'
rustup component list --toolchain nightly-2026-05-28 --installed | rg -q '^llvm-tools-'
driver_root="${DYLINT_DRIVER_PATH:-${HOME}/.dylint_drivers}"; driver_path="$driver_root/nightly-2026-05-28-aarch64-apple-darwin/dylint-driver"; test -x "$driver_path"; test "$("$driver_path" -V | awk '{print $NF}')" = '6.0.3'
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" RUSTFLAGS='-F unsafe-code -D warnings' cargo +nightly-2026-05-28 check --locked --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml --all-targets
cargo fmt --manifest-path tools/surgeist-layout-audits/Cargo.toml --check
test ! -e tools/surgeist-layout-audits/target
test ! -e scripts/audit-node-projection-boundaries.sh
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
CARGO_NET_OFFLINE=true just corpus-check
CARGO_NET_OFFLINE=true just taffy-check
package_files="$(CARGO_NET_OFFLINE=true cargo package --locked --offline --list)"; test -z "$(printf '%s\n' "$package_files" | rg '^tools/surgeist-layout-audits/' || true)"
rg -q '^exclude = \["tools/surgeist-layout-audits/\*\*"\]$' Cargo.toml
diff -u <(git show 20ad8202e536c4c63f0bd211f0872653462116bf:Cargo.toml | awk '1; /^description = / { print "exclude = [\"tools/surgeist-layout-audits/**\"]" }') Cargo.toml
test "$(git diff 20ad8202e536c4c63f0bd211f0872653462116bf..HEAD -- Cargo.lock | wc -l | tr -d ' ')" = 0
actual_paths="$(git diff --name-only 20ad8202e536c4c63f0bd211f0872653462116bf..HEAD | LC_ALL=C sort -u)"; test -z "$(printf '%s\n' "$actual_paths" | rg -v '^(Cargo\.toml|plans/cycles/P01-I08-S02-R06A-dylint-audit-catalog\.md|scripts/audit-node-projection-boundaries\.sh|tools/surgeist-layout-audits/)')"
standing_paths=(Justfile src tests); test ! -d .github || standing_paths+=(.github); if rg -n 'cargo[[:space:]]+dylint|surgeist-layout-audits|p01_i08_s02_r06_t02_node_projection_boundary' "${standing_paths[@]}"; then exit 1; fi
base_suppressions="$(while IFS= read -r p; do git show "20ad8202e536c4c63f0bd211f0872653462116bf:$p" | perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m=$&; $m=~s/\s+/ /g; print "$m\n" }'; done < <(git ls-tree -r --name-only 20ad8202e536c4c63f0bd211f0872653462116bf | rg '\.rs$') | LC_ALL=C sort)"; current_suppressions="$({ git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 perl -0777 -ne 'while (/^[ \t]*#\s*\[\s*(?:allow|expect|cfg_attr)\b[^\]]*\]/gms) { $m=$&; $m=~s/\s+/ /g; print "$m\n" }' | LC_ALL=C sort)"; test -z "$(comm -13 <(printf '%s\n' "$base_suppressions") <(printf '%s\n' "$current_suppressions"))"
if { git ls-files -z -- '*.rs'; git ls-files -z --others --exclude-standard -- '*.rs'; } | sort -zu | xargs -0 rg -n --pcre2 '#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'; then exit 1; fi
test "$(shasum -a 256 tests/layout/browser_parity/corpus.toml | awk '{print $1}')" = c6e6f1422e14a5e4aa474c143998063ce0de4d0a9123b69875b35a4ed009a8f6
test "$(shasum -a 256 tests/layout/browser_parity/scripts/gentest/test_helper.js | awk '{print $1}')" = c684c7f167d95997a4a9f0250467bbaf72c1b73e69e0f707a2ef32f4d25f7f36
test "$(shasum -a 256 tests/layout/browser_parity/xml/generation-reports/all.json | awk '{print $1}')" = c10dc550d260a239c8bf9dd553f5272ca3bcc2826099bc182f800986b8b94c0e
test "$(find tests/layout/browser_parity/html -type f -name '*.html' | wc -l | tr -d ' ')" = 1448
test "$(find tests/layout/browser_parity/xml -type f -name '*.xml' | wc -l | tr -d ' ')" = 5776
cargo fmt --check; git diff --check; test -z "$(git status --porcelain=v1)"
```

After publication/readback, prove no cycle-owned Dylint, Cargo, Rust, layout,
test, or generator process remains. Run repository-root `cargo clean`; prove
`target/` and `tools/surgeist-layout-audits/target` absent and Git clean. Record
the immutable candidate, exact pins/acquisition, three ordered task ranges and
verdicts, selected-lint and UI evidence, script deletion, pilot lessons, product
equivalence, remote readback, cleanup, and R07/R08 handoff.
