# FRI-06 C08 Second-Lineage Diagnostic Census

Status: retained diagnostic evidence; not acceptance lineage

## Run Identity

After C08 T1, T2, R0, R1, and R2 were task-clean, the reviewed T3 preflight
passed and the exact full unfiltered existing-pinned generation command ran once
from plan-only HEAD `b2ae58c2825356f1b9e843b648307bcd617668e6` with exit zero. No scoped or
additional generation ran. An earlier malformed-path invocation had stopped at
executable canonicalization before browser launch or artifact writes and is not
a lineage run.

| Evidence | Value |
| --- | --- |
| Report | `tests/layout/browser_parity/xml/generation-reports/all.json` |
| Report SHA-256 | `81b29941e7925aa471bd7a96091fc35b4a0eb5ff389cea532c5f7086fce476bb` |
| Generated XML aggregate SHA-256 | `c1568fca84a70956c73f8e19de18977113aa0c1074dee7dc6887dd2171adb877` |
| Helper SHA-256 | `fd668b064fcccb00ebb1632183e4f2522ce29f1b390f2f0c012bdade906ed18c` |
| Corpus manifest SHA-256 | `99bb6fda5641c9f81704ddf391930934fb441f719090cf6ca4b84e31636c3701` |
| Base style SHA-256 | `5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6` |
| Browser | Chrome for Testing `149.0.7827.115` |
| Launch profile SHA-256 | `9e2b5a4850e8d5ae31cf133c30f7129f1e214705f7a848697ca42c7c1b7551cb` |
| Taffy source commit | `d1ff7e339b9ee35b33858779f8d7653197e93d92` |

The report has `filter: null`, 5,712 generated variants, exactly 16 unsupported
missing-root variants, and zero expected-fail, quarantined, or
failed-to-generate variants. All 5,324 entry-generated XML bodies preserve exact
semantics; their changes are provenance-only. Accounting and provenance are
valid, but the fixed 388-row public comparison matrix is not.

The XML aggregate is the SHA-256 of the sorted `shasum -a 256` output for all
5,712 `*.xml` files. Matrix digests below cover sorted, LF-terminated
`source<TAB>variant` rows.

## Executed Outcome

| Partition | Rows | SHA-256 |
| --- | ---: | --- |
| Fixed activation union | 388 | `3a0f78a7fdefc9f49feee9f0fcb5a035bc87f381f8fc8d96049eaa0cdcbc2eb1` |
| Pass | 104 | `29f9cf9ac175c105317ff38a183048a1f0429707e22fd3b076d85b455e6504a1` |
| Fail | 284 | `89152d321e60d65d4c6beb238ce20cfbc000aac66c2be7714a3494d437fefca2` |

The 104 passing rows are:

- both LTR variants of `fri06_atomic_inline_baseline` and
  `fri06_atomic_inline_percentage_block_size`: four;
- all four variants of `fri06_bidi_mixed_inline`, `fri06_forced_break_strut`,
  `fri06_float_auto_height`, `fri06_float_bfc_avoidance`, and
  `fri06_float_logical_clear`: 20;
- all four variants of both `grid_lanes_not_inhibited_*_packing` sources: eight;
- all four variants of the 16-source
  `subgrid_alignment_{baseline,center,end,start}_{baseline,center,end,start}_item`
  cross-product: 64; and
- all four variants of `subgrid_standalone_axis_max_width_clamp` and
  `subgrid_standalone_axis_min_content_wrapping`: eight.

The first observable failure partition is:

| Mismatch | Rows | SHA-256 |
| --- | ---: | --- |
| `x mismatch` | 148 | `e88366a2551b9e16f6b6f16383297bdbcb5d71fe95a14ee0e35c8e9ca0e38820` |
| `height mismatch` | 112 | `c589b0452bb8a28014fb5dd54f11f07ed41265296145fc0aa638dc341ac1a734` |
| Range physical inline-start | 20 | `2463184c170479d02d28de8ebbc5c3886154001a0769dc0734a3686738f3f1c0` |
| Range line index | 4 | `9c3930435c2a2c65d8bf87bdf22f082de00857e7727928140aec3a744d52238e` |

## Prior-Census Correction

The retained TSV at SHA-256
`e972e8d67e32919ce736f6d5428f017fa9a61ec5112fa75b2ec5b9d43b53e4f5`
literally records 36 pass and 352 fail rows. The first post-generation prose
census's 94/294 statement was an inferred reclassification that treated 58 raw
failures as pass controls; it was not an executed result.

Against the literal TSV, 68 rows moved from fail to pass, 284 remained failing,
36 remained passing, and no literal pass regressed. Against the earlier inferred
94/294 partition, 14 rows moved to pass and four forecast pass controls failed,
for the net increase to 104:

| Transition | Rows | SHA-256 |
| --- | ---: | --- |
| Inferred fail to pass | 14 | `dc79675a0368f7e9f3f4261f7859281f17686a1fc718277315ac8c0754b38743` |
| Inferred pass to fail | 4 | `40478170979eaf742ad3aaee5f12b61511e01d33d8e56560756ff0edaa9beb3e` |

The 14 are all variants of `fri06_bidi_mixed_inline`, both RTL variants of
`fri06_float_auto_height`, and all variants of `fri06_float_bfc_avoidance` and
`fri06_float_logical_clear`. The four forecast failures are both RTL variants of
`fri06_atomic_inline_baseline` and `fri06_atomic_inline_percentage_block_size`.

## Validated Cause Partition

| Earliest boundary | Rows | Matrix SHA-256 |
| --- | ---: | --- |
| Blockified BR ordinary-box helper/serializer input | 244 | `eb9c8d005c76b0a52d9333fb39710f4b8f263189b88d74cf6f2ba7922b768460` |
| Range explicit-root coordinate translation | 18 | `ae9121d16226cabbb602c2f326fb5cfa1034c23f612104600cba560d7fa80b23` |
| Direct-root RTL physical placement | 2 | `a0620971c825fe0be6909c2331add26e26478c9970e1bd9eb4ff5b8d28321b40` |
| Range root-wide line identity | 4 | `9c3930435c2a2c65d8bf87bdf22f082de00857e7727928140aec3a744d52238e` |
| Shape-fixture explicit atomic break | 4 | `0b96e7d9a39716b0121017cdbe67345381d72044918c9cef5b31ec216364de18` |
| Mixed-wrap continuation strut | 4 | `8a59f6f6231bcf5478f51ab9fe200169ba81198c817f923116534e09d268facc` |
| Vertical line placement | 4 | `e2c95514201e376def0b87d6ad61940d719e5d1f84526bae47b814cfa90a9a79` |
| Float-line final height | 4 | `7b4fc8b3bb27f912d3f39d2aadc05c243ead274fed54c20dfa43bd0825f7c61f` |

The first three boundaries are confirmed existing helper/serializer/comparator
defects: blockified BR boxes lose their source identity and used inline metrics;
Range starts are explicit-root-relative while the comparator consumes node-local
fragment rectangles; and each independent text node is serialized with line
index zero. The shape source lacks the exact break required before its 42px
atomic. The mixed-wrap adapter's fixed strut produces 44px rather than 46px.
The two direct-root percentage RTL rows receive no ancestor translation and
remain a separate physical-placement defect. The final eight rows reach complete
lowered input and expose two more narrow C08 production defects in vertical
inline placement and float-line report height.

No failed row requires subgrid expansion, a general CSS parser, a text shaper,
shape architecture, generator architecture, or behavior owned by a later
initiative. The fixed 388-row membership remains authoritative.

## Test-Lifecycle Correction

The T3 preflight test correctly proved stale artifacts before generation, but
its `fri06_c08_` name also selected it after generation. Its stale report/XML and
two-file worktree assertions must be preflight-only. Post-generation evidence
must instead retain immutable input hashes and assert final accounting,
provenance, 5,324 semantic preservation, and all 388 public comparisons.

## Recovery Boundary

This lineage remains diagnostic. Before changing any helper, serializer, HTML,
adapter, comparator, or production input, preserve this census, restore the two
T3 Rust drafts and every generated artifact to the committed entry state, and
remove only the 388 report-enumerated untracked XML files. Then correct the seven
finite cause partitions with public-fixture-shaped RED evidence. After every
changed input is task-clean and frozen, one future full unfiltered generation may
derive the replacement lineage. Scoped generation remains optional diagnostics
only; no unchanged-input generation is permitted.
