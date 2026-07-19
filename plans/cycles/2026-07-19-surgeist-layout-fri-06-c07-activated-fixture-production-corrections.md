# FRI-06-C07 Activated-Fixture Production Corrections

Status: in_progress

Cycle ID: `FRI-06-C07`

Owning repository: `surgeist-layout`

Cycle base: `189787e6de5e83ee39cce9d9771c94847dd799e8`

Reviewed specification:
`plans/specs/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`7090ea13ba7d9e524ce432018c8b7c44c1b3b76428d2c666949d297656ce97c8`,
commit `cc2a8486f9e4e7719c9a28cc68321b7e630d9ded`: `FRI-06.4 D-01` through
`D-03`, `D-06`, `D-07`, `D-09` through `D-13`, and `D-15`; `FRI-06.7`
Participant Matrix rows Shaped segment, Visible line break, and Float, Break
Selection Matrix rows Min-content query and Max-content query, Bidi And
Whitespace Matrix row Visible line break, Float And BFC Matrix rows Floating
child, Float-only container, and Nested BFC, the Atomic Baseline Matrix, and all
seven Control And Clear Matrix requirements; `FRI-06.8` Logical Line Builder,
Float Bands, and Size, Baseline, Scroll, Cache, And Rounding; `FRI-06.9` evidence
families Text wrapping, Struts/controls, Vertical lines, Atomic baselines, and
Rectangular floats; `FRI-06.10` rows `src/inline.rs`, `src/block.rs`, and Focused
Rust tests; and `FRI-06.14` acceptance items 4 through 9 and 16 through 18.

Reviewed implementation sequence:
`plans/sequences/2026-07-17-surgeist-layout-fri-06-inline-formatting-floats-bfcs.md`
at normalized semantic-content SHA-256
`2cda27e19919abaef075600816d320d87a36d36be6c79bd27c2962684c66467b`,
commit `fde445a0ff74f8359bcf8c512e2cf9e331de36e6`, entry
`FRI-06-C07` and the production-correction matrix in Activation Recovery
Evidence.

## Outcome

Correct the production behavior reached by exactly 72 validated variants across
26 named sources. Preserve shaped-participant, intrinsic, baseline,
forced-control, bidi, writing-mode, RTL, and logical-clear semantics through the
public compute front door without changing any fixture input or derived artifact.

## Boundary

FRI-06-C06 is published and remotely verified at the cycle base. Its finite
fixture adapter can lower and compare the reviewed facts. The invalid diagnostic
derivation was discarded and is not artifact lineage, but its read-only parity
result fixed the production-correction membership before C07.

The immutable production matrix has digest
`56b464ee7b2e5e4ec640d57f3732ee348c1c4534272ee60c82cc27560265518d`
over sorted LF-terminated `source<TAB>variant` rows. This plan partitions it
once as follows:

| Partition | Sources and variants | Rows |
| --- | --- | ---: |
| Intrinsic and packing height | All four standard variants of `grid_lanes_not_inhibited_normal_packing`, `grid_lanes_not_inhibited_overflow_hidden_packing`, and `subgrid_auto_track_sizing_min_content_text_runs` | 12 |
| Baseline and forced-control placement | All four standard variants of `subgrid_baseline_auto_columns_first_item`, `subgrid_baseline_auto_columns_second_item`, `subgrid_baseline_standalone_axis_first_item`, `subgrid_baseline_standalone_axis_second_item`, and `fri06_forced_break_strut` | 20 |
| RTL subgrid alignment | Both RTL variants of the sixteen `subgrid_alignment_<self>_<item>_item` sources from `{baseline,center,end,start}` x `{baseline,center,end,start}` | 32 |
| Vertical and logical clear | All four standard variants of `fri06_vertical_break_clear` and `fri06_float_logical_clear` | 8 |

The standard variants are `border_box_ltr`, `content_box_ltr`,
`border_box_rtl`, and `content_box_rtl`. Each task records an explicit finite
source/variant table in private Rust test code, asserts its own row count, and
constructs equivalent typed inputs through production constructors and
`compute_layout`. The test does not parse or alter HTML/XML and does not add a
production-visible test surface. The four tables must be disjoint and their
union must equal the 72-row matrix above.

The typed inputs and observable oracles are fixed before implementation:

| Family | Typed input and exact observable oracle |
| --- | --- |
| Grid-lanes packing | A 120px logical-inline content box with two 60px lanes and 60px, 30px, 30px children. Used logical content size is 120x60; logical child origins are `(0,0)`, `(60,0)`, `(60,30)`. Horizontal LTR physical x origins are `0,60,60`; RTL origins relative to the content box are `60,0,0`. Normal and computed hidden overflow are identical. |
| Subgrid min-content text runs | A 100px track with preserved 25px, 100px, 50px, and 75px indivisible participants. Used grid size is 100x100; logical origins are `(0,0)`, `(0,25)`, `(0,50)`, `(0,75)`. Horizontal RTL physical x origins are respectively `75,0,50,25`; LTR x origins are all zero. |
| Subgrid baseline families | Two baseline-aligned items expose used heights 15px and 30px and baseline offsets 12px and 24px. The first physical y is 12, the second is 0, and both used baselines are physical y=24 in every standard variant. |
| Forced-break strut | A 10px preserved segment followed by a visible break with a 20px strut and 15px baseline under max-content sizing. The used root is 10x40, first/last baselines are y=15/y=35, and the zero-size break is `(10,15)` in LTR and `(0,15)` in RTL; box sizing does not alter it. |
| RTL subgrid alignment | A 400x400 horizontal root, 100px inherited tracks, 10px each margin/border/padding, and a 40x40 subject. Logical inline origins map `start=30,end=130,center=230,baseline=330`; block origins use the same mapping for the item alignment. Every RTL row has physical `x = 400 - logical_inline - 40`, unchanged physical y equal to the item mapping, and size 40x40. Thus RTL x maps `start=330,end=230,center=130,baseline=30` in both box-sizing variants. |
| Vertical break and clear | `VerticalRl` with a 40x40 logical containing box, one 10px segment, the same 20px/15px break strut, and a matching line-start exclusion ending at logical block 20. The break's logical point is `(10,15)`, the following strut baseline is logical block 35, and root logical block extent is 40. Physical break origin is `(25,10)` for LTR and `(25,30)` for RTL; first/last physical block-axis baselines are x=25/x=5. |
| Logical float clear | Independent `VerticalRl` 100x160 logical roots. A line-start float occupies logical `(0,0)` size 20x20 and its `Clear::Left` 50x10 block starts at logical `(0,20)`; a line-end float occupies `(70,0)` size 30x40 and its `Clear::Right` block starts at `(0,40)`. Their physical cleared-block origins are `(130,0)`/`(110,0)` in LTR and `(130,50)`/`(110,50)` in RTL. |

Each row asserts these constants directly after `compute_layout`; using current
production output to calculate the expectation is forbidden. Direction and box
sizing select only the stated physical projection and never alter the logical
oracle.

The 256 fixture corrections remain C08-owned. The 408 failures from the broad
diagnostic are not a recovery matrix; aggregate parity remains FRI-13-owned.
No task may use those sets to add a case dynamically.

Non-goals are fixture adapter changes; HTML, helper, parser, serializer,
manifest, XML, report, and generation changes; corpus-generator execution of any
scope; browser acquisition; dependencies, features, lockfile, MSRV, public API,
docs, examples, root, siblings, task runner, expected failures, quarantine, and
FRI-09 through FRI-13 behavior. Generator architecture and implementation remain
unchanged. `just verify-generator` is read-only Cargo feature verification and
does not execute corpus generation.

Broad `MR-002` test-harness migration and `MR-003` shared layout-math extraction
wait until the final leaf candidate handoff. C07 may make only a task-local
extraction required for the confirmed correction and covered by the same focused
RED; it does not begin either broad refactor.

Every worker starts with its task's listed production modules. A reconstructed
RED may expand the write set only to a direct caller, provider, or public-front-
door path on that exact failing call chain, and the worker must identify the
concrete call edge in its evidence. Any broader production module or unrelated
path requires a plan amendment, a new semantic revision, and fresh plan review.

## Impacts

- **Public API:** unchanged; this is a correction of valid represented behavior.
- **Dependencies, features, lockfile, MSRV, and browser policy:** unchanged.
- **Generated inputs and artifacts:** unchanged; no corpus generation is allowed.
- **Docs/examples and root follow-up:** unchanged in C07; the published production
  candidate becomes C08's input.
- **Unsafe and lint policy:** no executable unsafe and no new `allow` or `expect`
  attribute in tracked or non-ignored owned Rust.

## Tasks

### `C07-T1` Correct Intrinsic And Packing Heights

**Files/area:** `src/grid/lanes.rs`, `src/grid/tracks.rs`, direct grid sizing
callers reached by the RED, and private regressions in `src/grid_tests.rs`.

**Outcome:** Preserve grid-lanes packing and subgrid descendant min-content
contributions when shaped inline participants replace legacy box-only inputs.

**RED:** Add `fri06_c07_height_` public-compute regressions with an explicit
12-row table. At the task base they must reproduce the diagnosed wrong used
block size or track contribution for the three named sources, not fail during
input construction.

**Acceptance:** Normal and computed-overflow-hidden grid-lanes cases pack one
60px item and two 30px items into two 60px lanes without inflating the used
block size. The subgrid min-content text-run case retains the longest 100px
indivisible contribution and its 100px wrapped block contribution. Direction
and box sizing do not change those intrinsic/packing results. Every regression
uses the public compute front door in both scalar lanes where the production
path is scalar-generic, and the explicit table has exactly 12 unique rows.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c07_height_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** published C06 adapter candidate and fixed 12-row partition.

**Intended commit:** `fix(layout): correct activated intrinsic heights`.

### `C07-T2` Correct Baseline And Forced-Control Placement

**Files/area:** `src/grid/child.rs`, `src/grid/tracks.rs`, `src/inline.rs`,
`src/block.rs`, and direct private regressions in `src/grid_tests.rs`,
`src/inline_tests.rs`, or `src/root_tests.rs`.

**Outcome:** Preserve baseline-group placement through subgridded and standalone
axes and retain a forced break's control geometry, committed line, following
strut, and baselines.

**RED:** Add `fri06_c07_baseline_control_` public-compute regressions with an
explicit 20-row table. At the task base the four subgrid families reproduce the
diagnosed y/baseline mismatch and the forced-break family reproduces the missing
or displaced control/following strut.

**Acceptance:** For auto-column and standalone-axis subgrids, the 15px first item
uses y=12 while the 30px second item remains at y=0, with their 15px/30px used
heights and baseline group preserved in every standard variant. A forced break
commits the current line, publishes the zero-size control at the final visual line
end, and retains the empty following line's containing strut plus first/last
baseline evidence. The explicit table has exactly 20 unique rows; both scalar
lanes and the public compute front door are covered.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c07_baseline_control_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T1 task-clean; intrinsic track geometry is stable before baseline
placement is corrected.

**Intended commit:** `fix(layout): correct activated baseline placement`.

### `C07-T3` Correct RTL Subgrid Alignment Projection

**Files/area:** `src/grid/child.rs`, `src/grid/lanes.rs`, `src/grid/tracks.rs`,
and private regressions in `src/grid_tests.rs`. Do not change authored fixture
alignment or add a second alignment algorithm.

**Outcome:** Project inherited subgrid areas, margin/border/padding, and resolved
self/item alignment exactly once through RTL physical placement.

**RED:** Add `fri06_c07_subgrid_rtl_` public-compute regressions with the explicit
sixteen-source self/item cross product and both RTL box-sizing variants. At the
task base all 32 rows reproduce the diagnosed physical x mismatch while retaining
their expected y and 40px used size.

**Acceptance:** Start, center, end, and baseline combinations retain their
logical inherited row/column, 10px margin/border/padding offsets, and 40px used
size while RTL mirrors only the physical inline origin. Alignment is resolved in
the correct owning axis with no physical/logical double reversal. The explicit
table has exactly 32 unique rows, and both scalar lanes use the public compute
front door.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c07_subgrid_rtl_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T2 task-clean; baseline fallback and group placement are stable
before the complete alignment cross product is projected.

**Intended commit:** `fix(layout): correct RTL subgrid alignment projection`.

### `C07-T4` Correct Vertical Control And Logical Clear Projection

**Files/area:** `src/inline.rs`, `src/block.rs`, direct flow-mapping helpers, and
private regressions in `src/inline_tests.rs` or `src/root_tests.rs`.

**Outcome:** Keep visible break placement and float/clear matching logical in
vertical and sideways containing flows until one final physical projection.

**RED:** Add `fri06_c07_logical_clear_` public-compute regressions with an explicit
eight-row table for every standard variant of the two named sources. At the task
base they reproduce the diagnosed control/float physical-position mismatch, not
an unsupported-flow error.

**Acceptance:** The vertical-break case validates against containing flow,
publishes the control at the projected visual line end, preserves its following
strut, and advances only for the matching logical clear side. The float case maps
left/right and clear left/right through containing `FlowAxes`, preserves source
order and block progression, and does not reinterpret them as physical x sides.
All ten flow mappings remain covered by the existing focused suite. The explicit
table has exactly eight unique rows, and both scalar lanes use the public compute
front door.

**Commands:**

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c07_logical_clear_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
```

**Dependency:** T3 task-clean; grid alignment projection is stable before the
remaining containing-flow projection correction.

**Intended commit:** `fix(layout): correct logical clear projection`.

## Completion

The four explicit row tables must be disjoint, contain exactly
`12 + 20 + 32 + 8 = 72` rows across the exact 26 sources, and admit no dynamic
membership. Each task range must be independently `CLEAN`. No fixture or
artifact path changes, no corpus generator command runs, and no broad mechanical
refactor begins.

Run:

```sh
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout fri06_c07_
CARGO_NET_OFFLINE=true just verify
CARGO_NET_OFFLINE=true just verify-generator
cargo fmt --check
git diff --check 189787e6de5e83ee39cce9d9771c94847dd799e8..HEAD
git diff --quiet -G'(^|[^.[:alnum:]_])(allow|expect)[[:space:]]*\(' 189787e6de5e83ee39cce9d9771c94847dd799e8..HEAD -- '*.rs'
test -z "$(git diff --name-only 189787e6de5e83ee39cce9d9771c94847dd799e8..HEAD -- Cargo.toml Cargo.lock justfile README.md scripts plans/specs plans/sequences tests/bin 'tests/layout/browser_parity/**')"
test -z "$(git status --porcelain)"
```

Run the fail-closed owned-Rust executable-unsafe scan from the canonical Surgeist
gate over every tracked or non-ignored `*.rs` file. Inspect the final changed-path
inventory and require every implementation/test path to be one of the direct
owners listed by the four tasks; the cycle plan itself is the only planning path.

A fresh `surgeist-holistic-reviewer` must return `CLEAN` for exact range
`189787e6de5e83ee39cce9d9771c94847dd799e8..cycle_head`. Rerun the complete
read-only set on local `main`, publish the immutable cycle head to authority
remote `main` with a leased fast-forward, fetch/read back, and prove local
`main`, its tracking ref, `FETCH_HEAD`, and live remote `main` agree. Remove any
agent-created temporary resources.

The handoff is the published production candidate for C08's bounded fixture
activation and single valid final derivation. C08 may change only its fixed
fixture/input/artifact matrices; C07 does not pre-author that work. Blocker: none.
