# P01-I04-S01-C01 Validated Sizing Calculations

Status: complete

Cycle ID: `P01/I04/S01/C01`

Owning repository: `surgeist-layout`

Cycle base: `5543ef5e9273ee73c187803c79191b8b71949fc0`

Reviewed specification:
`plans/P01-layout/initiatives/P01-I04-property-specific-sizing-values.md`
at SHA-256
`601f4ad4700827465096dc62c029b4d8147336b8593b6b80ae7abfb09fc22577`,
commit `49ede2ba2672a91f99ba193651dbb1350ede7b80`, sections `FRI-04.4 D-02`,
the generic calculation portion of `D-03`, the calculation types in
`FRI-04.5`, the calculation status matrix in `FRI-04.6`, the corresponding
evidence in `FRI-04.8`, and acceptance items 5 and the calculation portion of 6.

Reviewed sequence:
`plans/P01-layout/sequences/P01-I04-S01-property-specific-sizing-values.md`
at SHA-256
`2e006d30b0250c526e10bba13a37e58e111ff60791b34f8d7c2e4d0e527db13f`,
commit `0a666f8f698703cd7979194a7f75f834e4c9b522`, entry `P01/I04/S01/C01`.

## 1 Outcome

Add the private iterative validated program substrate plus ordinary and
calc-size calculation values. Prove shape, coefficient, basis, numeric, and
deep-ownership behavior in both scalar lanes without connecting the model to
property wrappers, `NodeInputOf`, public reexports, or production formatting.

## 2 Boundary

This cycle owns one focused private `src/sizing.rs` module, its private module
declaration in `src/lib.rs`, and tests colocated with the model. It may reuse
the existing validated affine values, percentage basis, finite-scalar errors,
length-resolution result, and `LayoutScalar` operations.

It must not add preferred/min/max/flex property wrappers, property-specific
calc-size bases, `CalcSizeConstructionError`, track changes, `NodeInputOf`
changes, algorithm consumers, parser/helper/generator/HTML/XML changes,
dependencies, features, docs, MSRV changes, root or sibling edits, public
reexports, unsafe code, or compatibility paths.

The private program is a flattened postfix instruction slice with validated
arity and iterative evaluation. Ordinary calculations use affine
length-percentage leaves. Calc-size calculations use finite absolute,
percentage, and size coefficients. Neither calculation applies a property's
non-negative used range in this cycle.

No generation command is applicable because no HTML, helper, serializer,
fixture parser, or generated artifact input changes.

## 3 Impacts

Public API: unchanged; the new module and its types remain private to the crate.

Dependencies, features, artifacts, docs, MSRV, root, and siblings: unchanged.

Safety: all owned Rust remains unsafe-free.

## 4 Tasks

### 4.1 `P01/I04/S01/C01/T01` Ordinary Sizing Calculation Program

**Files:** `src/sizing.rs`, `src/lib.rs` private module declaration.

**Outcome:** Add `SizingCalculationOf<S>` and
`SizingCalculationError::EmptyArguments`. Construction supports one affine
value, nonempty min/max arguments, and clamp with a required preferred value
and optional endpoints. The private representation validates every stack
shape, evaluates and drops iteratively, and reports through existing
`LengthResolutionOf<S>` statuses.

**RED:** Add focused tests named with the `sizing_calculation_` prefix. Before
implementation they fail because the module and type do not exist. Record the
expected compile/test failure before implementing.

**Acceptance:** Tests cover one and many min/max arguments, empty rejection,
all clamp endpoint combinations, minimum-wins bound conflict, nested programs,
signed zero, f32/f64 values, finite overflow, and a deeply nested program whose
evaluation and drop do not recurse. With a missing basis, any nonzero percentage
leaf makes the whole program `MissingBasis`; an all-zero-percentage program
resolves normally at every nesting depth.

**Commands:**

```sh
cargo test -p surgeist-layout sizing_calculation_
just fmt-check
just test
```

**Dependency:** None beyond the cycle base.

**Intended commit:** `feat(layout): add validated sizing calculations`.

### 4.2 `P01/I04/S01/C01/T02` Calc-Size Calculation Program

**Files:** `src/sizing.rs`.

**Outcome:** Add `CalcSizeCalculationOf<S>` and
`CalcSizeCalculationErrorOf<S>` with exact invalid absolute, percentage, and
size coefficient variants. Construction supports affine length-percentage,
the unit `size` term, checked three-coefficient terms, and the same validated
min/max/clamp shapes. Evaluation accepts an optional already-validated basis
size and an explicit percentage basis; a missing calc-size calculation
percentage contributes zero. Basis-size dependency is syntactic over the
complete program: if any leaf has a nonzero size coefficient and the basis size
is absent, the whole program returns `MissingBasis` before numeric evaluation,
regardless of algebraic cancellation or branch dominance. Only after that check
does evaluation report a non-finite intermediate as `InvalidNumeric`.

**RED:** Add focused tests named with the `calc_size_calculation_` prefix. They
fail before the calc-size value and coefficient errors exist. Record the
expected failure before implementation.

**Acceptance:** Tests cover every coefficient and error variant, canonical
signed zero, `depends_on_size`, definite and missing basis size, definite and
missing percentage basis, nested min/max/clamp, minimum-wins clamp conflict,
negative finite results without property clamping, f32/f64 overflow, and deep
iterative evaluation/drop. Nested, dominated, and coefficient-canceling size
terms all return `MissingBasis` when the size is absent, including a case whose
numeric evaluation would otherwise overflow; a missing percentage still
contributes zero. Property-basis `Any` rejection is absent and remains
explicitly assigned to C02.

**Commands:**

```sh
cargo test -p surgeist-layout calc_size_calculation_
just fmt-check
just test
```

**Dependency:** `T01` supplies the private program conventions and shape
error.

**Intended commit:** `feat(layout): add validated calc-size calculations`.

## 5 Cycle Acceptance

1. `T01` and `T02` satisfy their RED and acceptance evidence through
   separate implementation ranges and independent task reviews.
2. The new calculation representation is private, iterative, intrinsically
   valid, scalar-generic, and free of identity/resolver callbacks.
3. Ordinary missing-basis behavior is syntactic and calc-size calculation
   percentages use zero when their basis is missing.
4. Invalid shapes and non-finite coefficients/results remain typed; no panic,
   saturation, property clamping, or automatic fallback is introduced.
5. Existing source behavior and generated artifacts are unchanged.
6. No C02 property interface or later algorithm/fixture work enters the range.

## 6 Final Verification

```sh
just verify
just verify-generator
git diff --check
```

## 7 Handoff And Blockers

The completed cycle hands C02 a reviewed private calculation substrate. It does
not emit a root handoff or claim any public FRI-04 API yet.

A genuine blocker exists only if the specified iterative representation cannot
be implemented with the current standard library and existing crate types, or
if completing it requires a new dependency, public surface, unsafe code, or a
change to the reviewed calculation semantics. Such evidence returns to plan or
specification review; it does not authorize scope expansion.
