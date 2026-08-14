# P01/I08/S02/R06A Planning-Path Dylint Audit Catalog

Cycle ID: `P01/I08/S02/R06A`

Owning repository: `surgeist-layout`

Status: `in_progress`

Cycle base: `20ad8202e536c4c63f0bd211f0872653462116bf`

Specification: `plans/specs/P01-I08-grid-subgrid-and-grid-lanes-completeness.md`,
reviewed semantic SHA-256
`65050fe9723a62ef832badd02426c3fc2cb461f7931a4549a4c48c2ea39614e7`,
commit `98d67b05b7570e84490c6bf0121ba4a0cc2ec224`, sections `FRI-08.20`
row `AR-008`, `FRI-08.21`, `FRI-08.27.1`, all of `FRI-08.27.2`, and
acceptance rows `FRI-08.28(1)`, `FRI-08.28(7)`, and `FRI-08.28(10)` through
`FRI-08.28(12)`.

Sequence: `plans/sequences/P01-I08-S02-architectural-remediation.md`,
reviewed semantic SHA-256
`bb3642deea547129932693820df949b3db365ba4e4f134814ab9611dbb2aa171`,
commit `5ec0eae900daac01d56dba8ea919080ea13be26e`, entry
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
catalog build, test, strict compiler, and formatting commands. It is not added to
`just`, CI, ordinary Cargo/Clippy commands, publication gates, or product tests. The lint
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
`tools/surgeist-layout-audits/{.cargo/config.toml,Cargo.toml,Cargo.lock,rust-toolchain.toml,src/lib.rs}`;
user-scoped Cargo and rustup installations authorized above.

**Outcome:** verify the already installed exact authorized
binaries/toolchain/components; create package `surgeist-layout-audits`, library
`surgeist_layout_audits`,
`publish = false`, `crate-type = ["cdylib"]`, its own `[workspace]`, and exact
crates.io pins; configure installed `dylint-link` only for the exact
`aarch64-apple-darwin` catalog target so release builds publish Dylint's
toolchain-suffixed library; exclude the tool tree from product packaging; keep
every catalog command on repository-root `target/dylint-audits` with no nested
target.

Every command that builds, tests, or checks the lint executes with the catalog
directory as its current directory so Cargo discovers the nested target linker
configuration; a repository-root variable keeps build output in the required
root target. The selected product audit remains at the product root, while
Dylint builds its `--path` library from that library's package root.

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

The later pre-audit setup failure adds one T01 correction RED without replacing
the retained entry evidence: the catalog linker configuration and required
toolchain-suffixed release library are absent while the ordinary unsuffixed
library exists. The T01 correction adds only the missing linker configuration
and proves the suffixed release artifact without invoking the product lint.

```sh
test ! -e tools/surgeist-layout-audits/.cargo/config.toml
test -f "target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin/release/libsurgeist_layout_audits.dylib"
test ! -e "target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin/release/libsurgeist_layout_audits@nightly-2026-05-28-aarch64-apple-darwin.dylib"
```

The correction uses only installed inventory and the following offline probes;
it does not rerun acquisition or lockfile generation:

```sh
set -e
test "$(cargo dylint --version)" = 'cargo-dylint 6.0.3'
cargo install --list | perl -0ne 'exit !(/^cargo-dylint v6\.0\.3:\n    cargo-dylint$/m && /^dylint-link v6\.0\.3:\n    dylint-link$/m)'
rg -q '^\[target\.aarch64-apple-darwin\]$' tools/surgeist-layout-audits/.cargo/config.toml
rg -q '^linker = "dylint-link"$' tools/surgeist-layout-audits/.cargo/config.toml
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin" cargo +nightly-2026-05-28 build --release --locked --offline)
test -f "target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin/release/libsurgeist_layout_audits@nightly-2026-05-28-aarch64-apple-darwin.dylib"
git diff --exit-code bfd76dab2c52df5ec009f52595fba9ce6e5ac6e2 -- tools/surgeist-layout-audits/Cargo.toml tools/surgeist-layout-audits/Cargo.lock tools/surgeist-layout-audits/rust-toolchain.toml
cargo fmt --manifest-path tools/surgeist-layout-audits/Cargo.toml --check
git diff --check
test "$(git diff --name-only HEAD^..HEAD)" = 'tools/surgeist-layout-audits/.cargo/config.toml'
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
rg -q '^\[target\.aarch64-apple-darwin\]$' tools/surgeist-layout-audits/.cargo/config.toml
rg -q '^linker = "dylint-link"$' tools/surgeist-layout-audits/.cargo/config.toml
CARGO_NET_OFFLINE=true cargo +nightly-2026-05-28 generate-lockfile --offline --manifest-path tools/surgeist-layout-audits/Cargo.toml
test -f tools/surgeist-layout-audits/Cargo.lock
perl -0777 -ne 'exit !(/\[\[package\]\]\nname = "dylint_linting"\nversion = "6\.0\.3"\nsource = "registry\+https:\/\/github\.com\/rust-lang\/crates\.io-index"/s && /\[\[package\]\]\nname = "dylint_testing"\nversion = "6\.0\.3"\nsource = "registry\+https:\/\/github\.com\/rust-lang\/crates\.io-index"/s)' tools/surgeist-layout-audits/Cargo.lock
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" cargo +nightly-2026-05-28 check --locked --offline)
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin" cargo +nightly-2026-05-28 build --release --locked --offline)
test -f "target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin/release/libsurgeist_layout_audits@nightly-2026-05-28-aarch64-apple-darwin.dylib"
git diff --exit-code bfd76dab2c52df5ec009f52595fba9ce6e5ac6e2 -- tools/surgeist-layout-audits/Cargo.toml tools/surgeist-layout-audits/Cargo.lock tools/surgeist-layout-audits/rust-toolchain.toml
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
not fixed consumer paths or lexical masking. Expression tokens defined by a
macro in an allowed owner inherit that owner; caller-supplied expression tokens
retain caller ownership. Protected types and item escapes expanded into a
consumer remain violations.

Catalog documentation records the exact originating R06 plan/revision, audit
question, original scope, interpretation after later architecture changes, and
the superseded script. UI fixtures exercise allowed owner construction and each
rejected semantic escape, including aliases, reexports, UFCS, extracted method
items, nested modules, ordinary strings/comments, and both protected aggregate
families. They additionally prove an allowed owner-defined expression macro at a
consumer call site, a rejected caller-supplied aggregate expression, and rejected
macro-generated consumer type, alias, and visibility-reexport forms. Product
tests do not inspect source.

**RED/acceptance:** add the smallest UI fixtures first and prove the absent lint
or missing diagnostic is RED. The final selected UI suite passes with exact
diagnostics, the lint remains `Allow` when not explicitly selected, catalog
format and strict compiler checks pass with already-installed tooling, and
authored catalog Rust has no unsafe match.

The first reviewed lint revision passed its then-complete UI suite but the first
semantic product audit reported six false positives: the expanded HIR owners are
`grid` or `grid::lanes`, while every diagnostic span originates from
`project_grid_container` or `project_grid_child_input`, both defined in the
allowed `grid::input` owner. The current `is_projection_owner` checks only the HIR
owner and ignores rustc expansion provenance. The correction first adds the exact
macro UI cases above and proves the allowed owner-defined expression is RED under
the current lint. It then resolves the defining macro's compiler identity and
parent module from the expression span, without source paths or text parsing;
type and item rules remain caller-module rules. This is correction attempt one.

The earlier missing-driver setup failure is not RED evidence. The separately
authorized one-time driver bootstrap is complete, the exact `6.0.3` driver is
installed, and the catalog manifest, lockfile, and toolchain pin are unchanged.
The correction performs no acquisition and proves RED and GREEN with the focused
locked/offline `node_projection_boundary_ui` command in the matrix below.

```sh
set -e
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline node_projection_boundary_ui)
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline)
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" RUSTFLAGS='-F unsafe-code -D warnings' cargo +nightly-2026-05-28 check --locked --offline --all-targets)
(set -e; cd tools/surgeist-layout-audits; cargo +stable fmt --check)
test ! -e tools/surgeist-layout-audits/target
git diff --check
```

Dependency: T01. Commit:
`tooling(audit): detect node projection boundary escapes`; correction commit:
`fix(audit): honor owner-defined macro expressions`.

### 2.3 `P01/I08/S02/R06A/T03` Audit R06 And Retire The Lexical Script

**Area:** delete `scripts/audit-node-projection-boundaries.sh`; catalog reference
material and UI evidence; exact historical plan/source references only where
script deletion requires reconciliation.

**Coordinator transition:** after T02 is independently CLEAN and before T03 is
dispatched, the coordinator runs the explicitly selected lint against the
unchanged R06 product source and records zero diagnostics with the exact semantic
invocation below:

The first command attempt at `9b88ecafbaaba34d8d0e88d8b7cf30c5e1d5e84b`
stopped during library discovery before loading the lint or compiling product
source: the catalog lacked Dylint's required `dylint-link` target configuration,
so only the ordinary unsuffixed library existed. It produced no semantic audit
result and T03 was not dispatched. The corrected T01 range is independently
CLEAN and its suffixed-library probe passes.

The next command attempt at `a8e73cfefbd99fcf192e64055f81ded2dc1657b2`
loaded the library but supplied `-D` after Dylint's Cargo-argument separator, so
Cargo rejected it before invoking rustc or compiling product source. It also
produced no semantic audit result. Dylint `6.0.3` requires lint-level rustc flags
through `DYLINT_RUSTFLAGS`; the corrected invocation below produced the first
semantic product audit at lint revision `9b88ecafbaaba34d8d0e88d8b7cf30c5e1d5e84b`.
It reported exactly the six T02 macro-provenance false positives recorded above,
so that lint revision is not selected again and T03 remains undispatched.

After the corrected T02 ordered range is independently CLEAN, the coordinator
runs this command exactly once at that new reviewed lint revision. Zero
diagnostics permit T03. Another lint false positive returns to a new T02 UI
correction and re-review without reusing a lint revision; a genuine product
escape retains the script, stops R06A, and returns to a fresh reviewed R06
correction and republished candidate.

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$PWD/target/dylint-audits" DYLINT_RUSTFLAGS='-D p01_i08_s02_r06_t02_node_projection_boundary' cargo dylint --path tools/surgeist-layout-audits
```

A worker never performs a selected semantic audit. The final zero-diagnostic run
does not recur in the final matrix.

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
coordinator's final zero-diagnostic transition result and the earlier immutable
false-positive result together are the cycle's selected semantic evidence.

```sh
set -e
test ! -e scripts/audit-node-projection-boundaries.sh
test ! -e tools/surgeist-layout-audits/target
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline)
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

The coordinator's selected-lint results are immutable cycle evidence. The final
matrix does not repeat any selected invocation.

```sh
set -e
test "$(cargo dylint --version)" = 'cargo-dylint 6.0.3'
cargo install --list | perl -0ne 'exit !(/^cargo-dylint v6\.0\.3:\n    cargo-dylint$/m && /^dylint-link v6\.0\.3:\n    dylint-link$/m)'
rustup component list --toolchain nightly-2026-05-28 --installed | rg -q '^rustc-dev-'
rustup component list --toolchain nightly-2026-05-28 --installed | rg -q '^llvm-tools-'
driver_root="${DYLINT_DRIVER_PATH:-${HOME}/.dylint_drivers}"; driver_path="$driver_root/nightly-2026-05-28-aarch64-apple-darwin/dylint-driver"; test -x "$driver_path"; test "$("$driver_path" -V | awk '{print $NF}')" = '6.0.3'
rg -q '^\[target\.aarch64-apple-darwin\]$' tools/surgeist-layout-audits/.cargo/config.toml; rg -q '^linker = "dylint-link"$' tools/surgeist-layout-audits/.cargo/config.toml
test -f "target/dylint-audits/dylint/libraries/nightly-2026-05-28-aarch64-apple-darwin/release/libsurgeist_layout_audits@nightly-2026-05-28-aarch64-apple-darwin.dylib"
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" cargo +nightly-2026-05-28 test --locked --offline)
audit_repo_root="$PWD"; (set -e; cd tools/surgeist-layout-audits; CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$audit_repo_root/target/dylint-audits" RUSTFLAGS='-F unsafe-code -D warnings' cargo +nightly-2026-05-28 check --locked --offline --all-targets)
(set -e; cd tools/surgeist-layout-audits; cargo +stable fmt --check)
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
