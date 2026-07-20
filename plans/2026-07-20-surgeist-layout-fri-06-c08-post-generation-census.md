# FRI-06 C08 Post-Generation Diagnostic Census

Status: retained diagnostic evidence; not acceptance lineage

## Run Identity

The first full unfiltered generation after the then-frozen C08 inputs ran once
from `2026-07-20T12:07:15Z` through `2026-07-20T12:22:49Z` and exited zero. No
scoped generation preceded or followed it. The invocation count for that exact
input lineage is one; it must never be rerun unchanged.

| Evidence | Value |
| --- | --- |
| Report | `tests/layout/browser_parity/xml/generation-reports/all.json` |
| Report SHA-256 | `65d88aa1b13f813392e27690d3f5ac9b79a2bbbc1cdb86ad78328665e3aeecd0` |
| Generated-artifact aggregate SHA-256 | `3fcd59bc93c9bb61c24eb2f5fd7b0cf45bc1b18a49c316821d209f4281765028` |
| Helper SHA-256 | `ee9976421ff6cfbf8d58e26aa10f11204452242ca80d8c2994f5abc4f5be28ac` |
| Corpus manifest SHA-256 | `99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701` |
| Base style SHA-256 | `5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6` |
| Browser | Chrome for Testing `149.0.7827.115` |
| Launch profile SHA-256 | `9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb` |
| Taffy source commit | `d1ff7e339b9ee35b33858779f8d7653197e93d92` |

The report has `filter: null`, 5,712 generated variants, exactly 16 unsupported
missing-root variants, and zero expected-fail, quarantined, or failed-to-generate
variants. Accounting and provenance are valid. Public comparison exposed input,
adapter, comparator, and production defects, so this lineage is diagnostic only.

## Fixed Activation Union

The fixed C08 activation union remains 388 `source<TAB>variant` rows with
SHA-256 `3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1`.
The following exact disjoint partition reconciles all 388 rows:

| Category | Rows | Matrix SHA-256 |
| --- | ---: | --- |
| `x` placement | 130 | `c9e430786eecd1c13ba67fc642470dc61db455b3f8ea3e7b94936add9c81fcdd` |
| control semantics | 72 | `d850837add72edfa23068732eaef3fe2cf0accd57c8f890cb9c588c3b290f7a8` |
| height | 52 | `4531d50b16ead71aba258e79272a68a020d9f13469c8dceff363efed655701f5` |
| scroll | 4 | `867d49ac49ba1a88543207d724cb2810c06e342ad522156f11bf482eb731e772` |
| float/clear lowering | 8 | `18fde128933b7cb6fcee54d87c8221f5e21119f632084bc3aaff8c4cfbad4219` |
| Range origin | 4 | `549007921e691b978442a3da93330b9d2cfee5d11fe78e9e7640142439a73e85` |
| shape bands | 4 | `0b96e7d9a39716b0121017cdbe67345381d72044918c9cef5b31ec216364de18` |
| finite later-owned adapter | 4 | `9c3930435c2a2c65d8bf87bdf22f082de00857e7727928140aec3a744d52238e` |
| parser predicate | 16 | `f9ac335e450b4ffd014ae91ef211e699b513676711f70e2c27414fb64f7455a3` |
| pass control | 94 | `58b1b6368e0639fa88d949588896045da3870b08a1724e8c0521e28d16a68a10` |

Matrix digests are over sorted, LF-terminated `source<TAB>variant` rows. The
selectors below use all four box-model/direction variants unless narrowed.

## Exact Selectors

### `x` Placement: 130

- `fri06_inline_mixed_text_atomic_wrap`: 4.
- `fri06_float_auto_height`: both RTL variants, 2.
- `fri06_float_line_exclusion`: 4.
- `subgrid_baseline_inline_column_*_(first|last)`: both RTL variants, 24.
- `subgrid_baseline_vertical_auto_rows_*_(first|last)`: 48.
- `subgrid_baseline_vertical_nested_*_(first|last)`: 48.

### Control Semantics: 72

- `subgrid_baseline_inline_column_*_(first|last)`: both LTR variants, 24.
- `subgrid_baseline_nested_block_*_(first|last)`: 48.

### Height: 52

- `fri06_inline_unequal_line_alignment`: 4.
- `subgrid_baseline_auto_rows_*_(first|last)`: 48.

### Remaining Finite Categories

- Scroll: `fri06_float_bfc_avoidance`, 4.
- Float/clear: `fri06_vertical_break_clear` and
  `fri06_float_logical_clear`, 8.
- Range origin: `fri06_bidi_mixed_inline`, 4.
- Shape bands: `fri06_float_shape_exclusion`, 4.
- Finite later-owned adapter: `subgrid_auto_track_sizing_min_content_text_runs`,
  4.
- Parser predicate: `subgrid_baseline_auto_columns_(first|second)_item` and
  `subgrid_baseline_standalone_axis_(first|second)_item`, 16.
- Pass controls: the 94-row complement in the fixed activation union.

## Validated Causes And Correction Ownership

| Category | Validated cause | Reopened C08 work |
| --- | --- | --- |
| Range | Helper subtracts a removed direct text parent; general RTL shaped/boundary traversal is incomplete | T1 helper origin; R0 category checks; R1 traversal |
| Browser control spill | Serializer marks every source-tag BR instead of only an explicitly lowered inline control | T1 helper/serializer and preservation evidence |
| `x` | Blockified BR is lowered as a line break; RTL physical float sides and atomic continuation struts are incomplete | T1, T2, R1, R2 |
| Control | Nonwrapping flex comparison derives source/neighbor facts from model control geometry | R0 comparator; wrapped flex remains fail-closed |
| Height | Blockified BR is incorrectly modeled as an inline control | T1 and R2 |
| Scroll | BFC source labels introduce unintended wrapping overflow | T2 fixture correction |
| Float/clear | Finite parser lacks line-relative aliases and physical-side flow-axis rejection | R0 finite lowering |
| Shape | Fixture bands do not match the two recorded browser query intervals | T2 source and query-recorder evidence |
| Later-owned | Typed shaped children coexist with duplicate raw text fallback; exact anonymous grid grouping is absent | T1 fallback suppression; R2 finite wrapper |
| Parser | Exact adapter predicate expects authored `inline-grid` after computed blockification yields `grid` | R2 predicate correction |

The browser-control spill affects 64 variants across these 16 source families:
`block_basic_with_br`, `block_border_fixed_size_with_br`,
`block_br_empty_lines_metrics`, `block_br_inline_block_metrics`,
`block_br_vertical_lr_inline_block_metrics`,
`block_br_vertical_rl_empty_lines_metrics`,
`block_br_vertical_rl_inline_block_metrics`,
`block_br_vertical_rl_rtl_inline_block_metrics`, `block_direction_rtl_with_br`,
`block_margin_x_fixed_auto_left_and_right_with_br`,
`block_margin_x_fixed_auto_left_with_br`,
`block_margin_y_collapse_through_blocked_by_padding_bottom_with_br`,
`block_margin_y_collapse_through_positive_with_br`,
`block_margin_y_simple_positive_with_br`, `block_padding_border_fixed_size_with_br`,
and `block_padding_fixed_size_with_br`.

## Lineage Rule

Retain the current generated files only while they supply correction diagnostics
and focused RED evidence. Before materially changed helper/parser/HTML/fixture
inputs are implemented, discard this invalid generated lineage without treating
it as a hand edit or acceptance artifact. After all correction tasks and their
independent reviews are clean and every input is frozen, C08 permits exactly one
replacement full unfiltered generation. Scoped runs remain optional diagnostic
tools and never supply acceptance evidence. No unchanged-input generation is
permitted.
