# P01-I01-S01-C04 Initiative Closure

Status: complete

Cycle ID: `P01/I01/S01/C04`

Owning repository: `surgeist-layout`

Cycle base: `63cc63941057fcd44578a261bcc26363763a64af`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I01-compute-resolution-diagnostics.md`
at `38263f35b0e9782db12e28088e97e36a7f953db6af22a26b110cd004c8da51f9`
committed at `49ede2ba2672a91f99ba193651dbb1350ede7b80`, sections
`FRI-01.12` through `FRI-01.19`.

Reviewed sequence:
`plans/P01-layout/sequences/P01-I01-S01-compute-resolution-diagnostics.md`
at `307ef2c0c6446a107e42f7a81d4e474dcde56fc509a4b4ed3cc76b1b82b89dbb`,
committed at `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry
`P01/I01/S01/C04`.

Bounded outcome: the three active calc fixture families become durable default
integration regressions, the manifest and layout-owned documentation state the
implemented FRI-01 contracts and Rust 1.97 MSRV, and final initiative evidence
closes FRI-01 without compatibility aliases, duplicated root adapters, or root
work.

## 1 Boundary

This cycle owns `tests/layout/browser_parity.rs`, `Cargo.toml`, `README.md`,
crate-level rustdoc in `src/lib.rs`, and the browser-parity README only where
needed to describe the verified layout-owned fixture path.

It does not change affine value behavior, compute algorithms, request/error/batch
semantics, fixture schema, HTML/XML fixtures, generated reports, the generator,
dependencies, root adapters, root API artifacts, sibling repositories, or later
FRI behavior. The public reexport audit has no expected symbol removal:
`ComputeInputOf` and `ComputeOutputOf` remain intentional direct-leaf and
provider/cache contracts, while recursive modes, mutation traits, and direct
tree algorithms are already crate-private.

Current evidence at the cycle base:

- each of `block_calc_width_margin`, `flex_calc_basis_margin_gap`, and
  `grid_calc_track_and_item_margin` has four checked-in box-sizing/direction
  variants and passes the filtered ignored runner through `compute_layout`;
- the normal integration suite has no dedicated non-ignored test that runs each
  complete calc family;
- `cargo metadata --no-deps --format-version 1` reports
  `"rust_version":null`, while the already-installed active compiler reports
  Rust 1.97.0;
- `README.md` still says calc values use resolver-aware APIs and does not state
  the public request, completed-batch, and typed-error contract;
- crate rustdoc still says the boundary is being ported; and
- source search finds no obsolete calc ID, store, generation, resolver, public
  recursive mutation, or direct tree-algorithm export.

## 2 Impacts

Public API: no symbol or behavior change is expected. Documentation identifies
`LayoutRootRequestOf`, `compute_layout`, `CompletedLayoutBatchOf`,
`LayoutErrorOf`, and the fallible direct `compute_leaf` boundary. No
compatibility alias is added.

Dependencies/features/artifacts: no dependency, feature, fixture, XML, report,
generator, or root-owned API artifact change.

Docs/MSRV: add `rust-version = "1.97"`; require the active `rustc` and `cargo`
to report Rust 1.97 before compiling all targets offline; update layout-owned
README and crate rustdoc to the implemented contracts. The browser-parity README
changes only if needed to name the now-default calc regression coverage. No
toolchain acquisition is required or authorized.

Root follow-up: none during the current rapid leaf-crate churn. The archival
record preserves these deferred requirements: lower style calc to
`LengthPercentageOf` coefficients, construct validated scrollbar/flex numeric
wrappers, construct `LayoutRootRequestOf`, apply or translate
`CompletedLayoutBatchOf`, and refresh root-owned API artifacts. It does not
message root or request pointer promotion, adapters, API refresh, or root tests.

Unsafe: no Surgeist-owned unsafe may be added or retained.

## 3 Tasks

### 3.1 `P01/I01/S01/C04/T01` - Run Calc Parity Families By Default

**Files/area:** `tests/layout/browser_parity.rs`

**Intended behavior/outcome:** Add shared, non-ignored characterization coverage for every required variant of each active block, flex, and grid calc family through the production public layout path.

**RED evidence:** Characterization baseline: source has no dedicated non-ignored calc-family tests, while all three filtered ignored commands already pass; no artificial failing behavior test is required.

**Acceptance criteria:** One shared helper compares each discovered family path set to the exact suffix set `border_box_ltr`, `border_box_rtl`, `content_box_ltr`, and `content_box_rtl`, then parses and compares every fixture with its path retained in failures. Three named tests cover the block, flex, and grid families; the generic full-corpus runner remains ignored.

**Commands:** `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout calc -- --nocapture`; the three filtered ignored commands from `FRI-01.17`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout`; `cargo fmt --check`

**Depends on:** Published `P01/I01/S01/C03`

**Intended commit:** `test: run calc parity families by default`

### 3.2 `P01/I01/S01/C04/T02` - Close The Public Contract

**Files/area:** `Cargo.toml`, `README.md`, `src/lib.rs`, optional `tests/layout/browser_parity/README.md`

**Intended behavior/outcome:** Declare and verify the project Rust 1.97 MSRV and make layout-owned docs/rustdoc describe normalized affine values, explicit basis resolution, validated root requests, atomic batch/error results, measurement, scalar lanes, and ownership. Preserve the exact C03 reexport region without adding aliases or changing symbols.

**RED evidence:** Deterministic RED: metadata reports no rust version; README contains `resolver-aware`; crate rustdoc says the boundary is being ported. The exact absence/reexport gates are green, and the already-installed active Rust 1.97 toolchain is available for the MSRV gate.

**Acceptance criteria:** Metadata reports exactly Rust 1.97; active Rust 1.97 cargo/rustc compile all targets offline; stale doc strings are absent and required contract terms are present; the reexport-region diff against the cycle base is empty and therefore preserves intentional `ComputeInputOf` and `ComputeOutputOf`; rustdoc builds with warnings denied; no dependency or feature changes.

**Commands:** Exact T02 source/MSRV gates below; `RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc -p surgeist-layout --no-deps`; `CARGO_NET_OFFLINE=true cargo test -p surgeist-layout`; `CARGO_NET_OFFLINE=true cargo clippy -p surgeist-layout --all-targets -- -F unsafe-code -D warnings`; `cargo fmt --check`; `git diff --check`

**Depends on:** `P01/I01/S01/C04/T01`

**Intended commit:** `docs: close FRI-01 public contract`

#### 3.2.1 Exact Source Gates

```sh
rustc --version | rg -q '^rustc 1\.97\.'
cargo --version | rg -q '^cargo 1\.97\.'
CARGO_NET_OFFLINE=true cargo check -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo metadata --no-deps --format-version 1 | rg -q '"rust_version":"1\.97"'
bash -lc 'if rg -n -e "\b(CalcId|CalcGeneration|CalcResolver|NoCalcResolver|LayoutCalcStore|CalcExpression|CalcTerm)\b" src tests README.md; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'if rg -n -e "pub use .*\b(RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\b" src/lib.rs || rg -n -e "pub (enum|trait|fn) (RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\b" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
diff -u <(git show 63cc63941057fcd44578a261bcc26363763a64af:src/lib.rs | awk '/^pub use cache/{keep=1} /^mod block_tests;/{keep=0} keep') <(awk '/^pub use cache/{keep=1} /^mod block_tests;/{keep=0} keep' src/lib.rs)
bash -lc 'if rg -n -e "resolver-aware|being ported" README.md src/lib.rs; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'for term in LengthPercentageOf PercentageBasisOf LayoutRootRequestOf compute_layout CompletedLayoutBatchOf LayoutErrorOf; do rg -q "\b${term}\b" README.md src/lib.rs || exit 1; done'
```

## 4 Completion

Cycle acceptance:

1. all twelve checked-in calc fixtures run in named non-ignored integration
   regressions and in the specification's filtered runner;
2. no obsolete calc identity/resolver/store/generation or public recursive
   mutation/tree-algorithm surface exists;
3. `rust-version = "1.97"`, README, crate rustdoc, and intentional reexports
   match the implemented request/result/batch/value contracts;
4. no fixture, generator, dependency, feature, or root-owned artifact changes;
5. all FRI-01 finding-closure evidence and final gates are green; and
6. publication produces an archival leaf candidate record only, with root
   integration explicitly deferred until the user authorizes root catch-up.

Final commands:

```sh
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --features layout-golden-generate
SURGEIST_PARITY_FILTER=block/block_calc_width_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=flex/flex_calc_basis_margin_gap CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
SURGEIST_PARITY_FILTER=grid/grid_calc_track_and_item_margin CARGO_NET_OFFLINE=true cargo test -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
rustc --version | rg -q '^rustc 1\.97\.'
cargo --version | rg -q '^cargo 1\.97\.'
CARGO_NET_OFFLINE=true cargo check -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo metadata --no-deps --format-version 1 | rg -q '"rust_version":"1\.97"'
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
bash -lc 'if rg -n -e "\b(CalcId|CalcGeneration|CalcResolver|NoCalcResolver|LayoutCalcStore|CalcExpression|CalcTerm)\b" src tests README.md; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'if rg -n -e "pub use .*\b(RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\b" src/lib.rs || rg -n -e "pub (enum|trait|fn) (RunMode|SizingMode|RequestedAxis|Compute|Round|CacheAccess|compute_cached|compute_root|compute_hidden|round_layout|compute_block|compute_flex|compute_grid)\b" src; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
diff -u <(git show 63cc63941057fcd44578a261bcc26363763a64af:src/lib.rs | awk '/^pub use cache/{keep=1} /^mod block_tests;/{keep=0} keep') <(awk '/^pub use cache/{keep=1} /^mod block_tests;/{keep=0} keep' src/lib.rs)
bash -lc 'if rg -n -e "resolver-aware|being ported" README.md src/lib.rs; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'for term in LengthPercentageOf PercentageBasisOf LayoutRootRequestOf compute_layout CompletedLayoutBatchOf LayoutErrorOf; do rg -q "\b${term}\b" README.md src/lib.rs || exit 1; done'
bash -lc 'files=$(git ls-files "*.rs"; git ls-files --others --exclude-standard "*.rs"); test -n "$files"; printf "%s\n" "$files" | xargs rg -n --pcre2 '\''#\s*\[\s*(?:unsafe\s*\(|no_mangle\b|export_name\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\''; rc=$?; test "$rc" -eq 1'
```

Required record: after publication, preserve the immutable layout candidate SHA,
review evidence, implemented public contracts, and the deferred root requirements
to lower style calc, construct numeric wrappers and root requests, apply batches,
and refresh root-owned API artifacts. Do not message or schedule root work.

Genuine blockers: a calc family failure is a blocker only when its cause belongs
to FRI-01; if exact evidence identifies a later reviewed FRI behavior as the
cause, return the fixture, mismatch, owning finding, and preserved state instead
of weakening or quarantining the regression. If the active compiler preflight no
longer reports Rust 1.97, do not invoke `rustup` or any acquisition-capable
command; return the missing-tooling blocker and wait for explicit permission.
