# FRI-02-C02 Signed Scroll Coordinate Contract
Status: in_progress
Cycle ID: `FRI-02-C02`
Owning repository: `surgeist-layout`
Cycle base: `1f1ac1d032ed3972feeefcade1b8e760b6042e76`
Reviewed specification:
`plans/specs/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `cc42db7e9bb5d895f10cb4d62e964bfc3cf4aef2b4e74a85e39d822a441e7d3f`,
commit `092f3b383ce87a9b72834ed444996861e3cfda2d`, decision `D-17`; the
four scroll conversions and complete public scroll surface in `FRI-02.6`;
`FRI-02.11`; scroll errors in `FRI-02.12`; the scroll row of `FRI-02.14`;
and scroll construction, conversion, clamp, geometry, and source evidence in
`FRI-02.17`.
Reviewed sequence:
`plans/sequences/2026-07-12-surgeist-layout-fri-02-logical-geometry-writing-modes.md`
at `4e11f24ba41bd6a98260155e0a5e6d2fd83eb5456a3a547f5ebeb4f3df8f5eb9`,
commit `2436454bc8aefffe7e3af55866ccdddbd6fce97a`, entry `C02`.
Bounded outcome: physical and flow-relative scroll offsets and axis ranges are
distinct, finite, signed, scalar-generic public values; `FlowAxes` owns the four
total conversions; current normal and rounded layout geometry reaches its
physical range only through the flow-relative projection pipeline.

## Boundary
Published `FRI-02-C01` supplies the reviewed `FlowAxes` mapping at the cycle
base. This cycle owns `OVERFLOW-004`: coordinate-space names, validation,
clamping, projection, `ScrollGeometryOf` storage, current range construction,
rounding, cache/output callers, public exports, and focused tests.
Current evidence: `ScrollOffsetOf` accepts every scalar infallibly;
`ScrollRangeOf` stores only a non-negative maximum and clamps from zero;
`ScrollUnsupportedFeature::InvalidScrollRange` erases axis and endpoint detail;
`ScrollGeometryOf` duplicates writing mode and direction and stores the
unqualified range; normal and rounded helpers each rebuild physical
`[0, extent]` ranges directly.

The cycle does not improve overflow magnitudes or origins, nested contribution,
flex/grid scroll geometry, gutter coherence, mixed-axis coupling, alignment
overflow, live scroll state, CSSOM policy, browser comparison, or any `FRI-05`
behavior. It does not edit fixtures, reports, generator/browser configuration,
root, or siblings. No compatibility alias, blanket conversion, default value,
second mapping table, physical-only shortcut, or fallback saturation is allowed.
The old unqualified types may remain only as a task-local compile bridge while
the first two tasks have live current callers; Task 3 deletes them rather than
adapting them.
## Impacts

Public API: intentionally breaking pre-release replacement of `ScrollOffsetOf`
and `ScrollRangeOf` with six coordinate-space-specific types and
`ScrollCoordinateErrorOf`; four public `FlowAxes` conversion methods;
`ScrollGeometryOf` stores/exposes one `FlowAxes` and one
`PhysicalScrollRangeOf`. `InvalidScrollRange` and duplicate geometry
writing-mode/direction/range accessors are removed;
`LayoutInternalInvariant::InvalidBlockScrollGeometry` is added.
Dependencies/features/artifacts: no dependency or feature change; no fixture,
report, browser, or generated artifact. Default and `layout-golden-generate`
build/test lanes remain required.
Docs/MSRV/root: public rustdoc explains coordinate signs, closed intervals,
construction failure, and root ownership of live state. Rust 1.97 is unchanged.
Root later updates adapters/facade/API artifacts; this leaf does not.
Unsafe: no Surgeist-owned unsafe may be added or retained.

## Tasks

Shared task gate after every task: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout`;
`CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings`; `cargo fmt --check`; `git diff --check`.
### C02-T1 - Validated Signed Coordinate Values
Files/area: `src/scroll.rs`, `src/scroll_tests.rs`, focused contract/rustdoc tests.
Outcome: add private-field physical and flow-relative offset, axis-range, and
two-axis range values plus `ScrollCoordinateErrorOf<S>`. Fallible constructors
distinguish coordinate space, semantic axis, endpoint/value, and inversion;
preserve finite negatives and scalar width; canonicalize signed zero; and never
panic. Axis ranges are constructible only by their enclosing range or a total
projection. Component-wise clamps cover below, endpoints, interior, and above in
both spaces and scalar lanes. The six values and error are `Copy + Debug +
PartialEq`, have default-scalar aliases, and have no `Default`, `Eq`, ordering,
public fields, or convenience conversions.
RED evidence: constructor/error/accessor/clamp and compile-fail tests do not
compile because the semantic types and errors do not exist.
Acceptance: every invalid component and inverted axis returns the exact typed
variant; signed-zero, finite-negative, clamp, idempotence, and f32/f64 tests pass;
existing geometry remains compiling through the temporary old implementation.
Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
scroll_coordinate -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked
-p surgeist-layout scroll_clamp -- --nocapture`; `CARGO_NET_OFFLINE=true cargo
test --locked -p surgeist-layout --doc`; shared task gate.

Depends on: published C01. Intended commit: `scroll: add signed coordinate values`.
### C02-T2 - Canonical FlowAxes Scroll Projection

Files/area: `src/scroll.rs`, `src/geometry.rs` only if required for the existing
mapping owner, and focused projection/property tests.

Outcome: add the four normative `FlowAxes` methods. Projection swaps physical
axes and negates reversed coordinates; reversed intervals swap and negate
endpoints so they stay ordered. Conversion is total for validated values and
defines no second `WritingMode` table. Methods remain crate-private until Task 3
publishes the complete surface. All ten mappings and both scalar lanes
cover offset/range round trips, nonzero signed intervals, signed zero, and the
commuting clamp law; clamped values remain contained and repeated clamp is
idempotent.

RED evidence: named mapping and clamp-law tests fail because no conversion path
exists.

Acceptance: exact mapping, round-trip, range ordering, clamp commutation,
containment, and idempotence evidence passes; the production duplicate-mapping
predicate remains empty.

Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
scroll_projection -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked
-p surgeist-layout scroll_conversion_clamp -- --nocapture`; `bash -lc 'if rg -n -U --pcre2
"(?s)\\bmatch\\b\\s*(?:(?!\\{).)*?(?:\\bwriting_mode\\b|\\.writing_mode)(?!\\s*(?:\\(\\))?\\.is_vertical)(?:(?!\\{).)*?\\{|_\\s*=>\\s*Direction::Ltr" src --glob "!geometry.rs" --glob "!*_tests.rs"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`;
shared task gate.

Depends on: C02-T1. Intended commit: `scroll: project signed coordinates through flow`.
### C02-T3 - Geometry Migration And Public Closure

Files/area: `src/scroll.rs`, `src/lib.rs`, `src/compute.rs`, `src/block.rs`,
`src/output.rs`, cache/contract/root/block/scroll tests, rustdoc, and all direct
callers of current scroll geometry.

Outcome: `ScrollGeometryOf` stores `FlowAxes` and
`PhysicalScrollRangeOf`; its constructor and accessors expose those values and
preserve physical rectangles. One crate-private
`physical_scroll_range_from_overflow_rects` owner maps current exposed physical
magnitudes to logical magnitudes, constructs flow-relative `[0, extent]` or
`[0, 0]` intervals, and obtains the physical range only through `FlowAxes`.
Normal and rounded paths both call it, with rounding recomputing from rounded
rectangles and stored flow. All callers, cache records, tests, docs, aliases, and
exports migrate; old offset/range types and `InvalidScrollRange` are deleted.
Layout-produced coordinate errors map to
`ScrollUnsupportedFeature::InvalidScrollGeometry`; root then emits
`Node(root)`/`RootLayout`/`InvalidRootScrollGeometry`; block-own uses that tuple
in root run mode and otherwise emits
`Node(node)`/`ChildLayout`/`InvalidBlockScrollGeometry`; block-child emits
`ContainerSubject { container, subject }`/`ChildLayout`/that block kind; and
rounding emits `Node(node)`/`RoundingFinalization`/`InvalidRoundedScrollGeometry`.
Block own/child/float/absolute helpers propagate `Result` instead of `expect`.

RED evidence: geometry, cache, rounding, contract, and all-mapping signed-range
tests fail against the old constructor/storage and direct physical range helper.

Acceptance: normal and rounded tests prove every reversed physical axis and the
non-reversed opposite axis for all ten mappings in f32/f64; non-scrollable axes
accept exactly `[0, 0]`; geometry coherence retains its existing typed error;
finite arithmetic overflow reaches each exact root/block/rounding error tuple
through the public `compute_layout` front door without panic, partial batch, or
saturation;
old names/accessors are absent; production has no direct physical-range
construction from overflow extents; task review classifies the five-hit source
inventory as one helper declaration, its two executable callers, one
flow-relative construction, and the returned `FlowAxes` projection;
public types/docs match the specification;
`OVERFLOW-004` has objective closure evidence without claiming `FRI-05`.

Commands: `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
scroll_geometry -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p
surgeist-layout round_scroll_geometry -- --nocapture`; `CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout scroll_geometry_error -- --nocapture`; `CARGO_NET_OFFLINE=true
cargo test --locked -p surgeist-layout contract -- --nocapture`;
`CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout cache --
--nocapture`; `bash -lc 'if rg -n "\\b(?:ScrollOffset|ScrollOffsetOf|ScrollRange|ScrollRangeOf|InvalidScrollRange)\\b" src tests; then exit 1; else rc=$?; test "$rc" -eq 1; fi'`;
`bash -lc 'set -euo pipefail; hits=$(rg -n -o "physical_scroll_range_from_overflow_rects|FlowRelativeScrollRangeOf[^[:space:]]*::try_new|\\.physical_scroll_range\\(" src/scroll.rs); test "$(wc -l <<<"$hits" | tr -d "[:space:]")" -eq 5; printf "%s\\n" "$hits"'`;
shared task gate.

Depends on: C02-T2. Intended commit: `scroll: migrate geometry to signed flow ranges`.
## Completion

Cycle acceptance: all six types and semantic errors match the reviewed model;
four conversions are the only projection path; current normal/rounded geometry
stores signed physical ranges derived from flow-relative magnitudes; ambiguous
old names and duplicate geometry context are absent; all C02 evidence is green;
no `FRI-05`, browser, artifact, root/sibling, compatibility, or unsafe work enters
the range.

Final commands:
```sh
CARGO_NET_OFFLINE=true cargo check --locked -p surgeist-layout --all-targets
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --features layout-golden-generate
CARGO_NET_OFFLINE=true cargo test --locked -p surgeist-layout --doc
RUSTDOCFLAGS="-D warnings" CARGO_NET_OFFLINE=true cargo doc --locked -p surgeist-layout --no-deps
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo clippy --locked -p surgeist-layout --all-targets --features layout-golden-generate -- -F unsafe-code -D warnings
cargo fmt --check
git diff --check
bash -lc 'if rg -n "\\b(?:ScrollOffset|ScrollOffsetOf|ScrollRange|ScrollRangeOf|InvalidScrollRange)\\b" src tests; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'set -euo pipefail; hits=$(rg -n -o "physical_scroll_range_from_overflow_rects|FlowRelativeScrollRangeOf[^[:space:]]*::try_new|\\.physical_scroll_range\\(" src/scroll.rs); test "$(wc -l <<<"$hits" | tr -d "[:space:]")" -eq 5; printf "%s\\n" "$hits"'
bash -lc 'if rg -n --pcre2 "pub const fn (?:writing_mode|direction|range)\\b|^\\s+(?:writing_mode: WritingMode|direction: Direction|range: ScrollRangeOf)" src/scroll.rs; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'if rg -n -U --pcre2 "(?s)\\bmatch\\b\\s*(?:(?!\\{).)*?(?:\\bwriting_mode\\b|\\.writing_mode)(?!\\s*(?:\\(\\))?\\.is_vertical)(?:(?!\\{).)*?\\{|_\\s*=>\\s*Direction::Ltr" src --glob "!geometry.rs" --glob "!*_tests.rs"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
bash -lc 'set -euo pipefail; files=(); while IFS= read -r -d "" file; do files[${#files[@]}]="$file"; done < <(git ls-files -z --cached --others --exclude-standard -- "*.rs"); test "${#files[@]}" -gt 0; if rg -n --pcre2 '\''#\s*!?\s*\[[^]]*(?:unsafe\s*\(|\b(?:no_mangle|export_name|link_section|naked)\b|\b(?:allow|expect)\s*\([^]]*\b(?:unsafe_code|unsafe_op_in_unsafe_fn)\b)|\bunsafe\s*(?:\{|fn\b|trait\b|impl\b|extern\b)|\bstatic\s+mut\b|\bextern\s*(?:"[^"]*")?\s*\{'\'' "${files[@]}"; then exit 1; else rc=$?; test "$rc" -eq 1; fi'
```

Three searches are negative; the five-hit inventory is classified during task
and holistic source review against the exact projection contract.

Required handoff: `C04` through `C07` may consume only the reviewed signed scroll
projection contract. No root handoff is emitted before complete FRI-02 closure.

Genuine blockers: a need for `FRI-05` range/origin correctness, live root scroll
state, browser acquisition, fixture regeneration, root/sibling edits, or an
unresolved coordinate policy stops this cycle rather than adding a workaround.
