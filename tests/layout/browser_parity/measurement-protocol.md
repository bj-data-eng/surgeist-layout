# Browser measurement protocol, version 1

This is the private interchange between layout's bundled JavaScript helper and
its Rust browser-corpus adapter. It describes measured fixture data, not authored
CSS, a public layout API, or the generator crate's report schema. Browser reports
continue to use schema 4 independently of this protocol version.

## Transport and evaluation

The adapter prepares one document and evaluates `getTestData()` once. The helper
returns a **JSON string**, whose object has exactly these five required fields:

| Field | Type | Measurement state / case suffix |
| --- | --- | --- |
| `schemaVersion` | Integer `1` | Version of this measurement contract |
| `borderBoxLtrData` | Node object | `border-box ltr` / `border_box_ltr` |
| `contentBoxLtrData` | Node object | `content-box ltr` / `content_box_ltr` |
| `borderBoxRtlData` | Node object | `border-box rtl` / `border_box_rtl` |
| `contentBoxRtlData` | Node object | `content-box rtl` / `content_box_rtl` |

Measurements occur in the table's order by changing `document.body.className`
and describing the same `#test-root` each time. The protocol does not add another
document, measurement, readiness condition, or browser wait. A missing root
returns all four variants as `layoutInput: "unsupported"` nodes with
`unsupportedReason: "Unsupported missing #test-root fixture root"`; it performs
no measurements or body-class changes.

The helper is injected only if `getTestData` is missing. A fixture's preloaded
helper remains in effect and must implement version 1. Unversioned and unknown
versions are errors; there is no production legacy decoder or automatic helper
replacement. Preservation tests explicitly transform captured old measurements
into the canonical version-1 wire shape offline. That test harness is not part
of the production decoder.

## Decoding and classification

The adapter first decodes the complete envelope into closed wire structures.
Object fields cannot be repeated or undeclared. Field types and discriminants
must match the declared alternatives; strings are not numbers, null is not an
empty object, and malformed tracks cannot be silently dropped.

Only after wire decoding succeeds does each variant select its first unsupported
reason: inspect the root, then visit children in their existing order, recursively
depth first. That reason classifies the entire variant. A measured box can carry
an unsupported reason; the explicit `unsupported` tag is the reason-only form.
Unsupported variants do not need complete supported-measurement geometry. A
malformed wire value still fails decoding even when another node
contains an unsupported reason.

Supported variants undergo semantic validation before XML writing. Validated
values distinguish boxes, inline text, inline boundaries, and line controls;
viewport context; observation forms; dimensions and tracks; and coupled inline
metrics. XML writing receives those values and formats them without discovering
malformed payloads or substituting fabricated zeroes.

Errors identify the affected case/variant when available, the node path, the
field path, and the error kind. JSON decoding errors retain their underlying
source. An invalid supported measurement is an adapter error, not an unsupported
outcome. The shared generator retains responsibility for fixture accounting and
failure-atomic publication.

## Field conventions and alternatives

JavaScript `undefined` properties are omitted by `JSON.stringify`. Optional
fields describe absence, rather than an instruction to infer another observed
value. Optional object fields reject explicit null; a null grid-area cell is a
separate semantic value. Explicit empty observations are preserved.

| Fields or structure | Meaning of omission, null, or an empty value |
| --- | --- |
| Envelope version and four variants | Required; omission and null are errors. |
| `layoutInput` | Required explicit tag: `box`, `inline-text`, `inline-boundary`, or `unsupported`. No node kind is inferred from a missing tag. |
| `unsupportedReason` | Optional classification on a measured node; required by the reason-only `unsupported` tag. Classification uses the first reason after wire validation. |
| `children` | Required ordered array on supported nodes; `[]` means no children. Unsupported reason-only nodes omit it. |
| Optional `style` fields | Omission means no explicit fixture declaration. XML default-value omission remains a serialization rule; the decoder does not fabricate an observed value. |
| `style.inlineMetrics` | Optional numeric object with required `baseline` and `lineHeight`. Omit the object when metrics are absent. Empty strings, individual metric fields, and null are invalid. |
| `size`, `minSize`, `maxSize`, `gap`, edge groups | Optional structured dimensions; missing axes or edges remain absent. The helper omits an entire group when none of its entries is present. |
| `gridTemplateAreas` | Optional nonempty rows of area-name identifiers or null cells. Rows have equal width, and every repeated name fills one rectangle. A null cell is the empty grid-area token `.`; it is not an omitted row. |
| `fragments`, `rangeInks` | Absence means no observation of that category. `[]` records an explicitly empty observation and produces `<fragments/>` or `<range-inks/>`. The categories cannot coexist on one node. |
| `shapeBands` | When present, a nonempty ordered table. Each query has required `bandMinimum` and `bandMaximum`, and optionally `interval: { minimum, maximum }`. Both interval endpoints are required when the interval exists. Flat endpoints and null are invalid. |
| `inlineBoundary` | A `start` or `end` descriptor. A start has either no metrics or both `baseline` and `lineHeight`; an end has no metrics. Boundaries have no layout observation or child payload. |
| `replacementInlineExtent` | Present only with `followingBreak: "allowed-with-replacement"`, where it is required. Other break kinds cannot carry replacement data. |
| Layout-ready marker fields | When present, `layoutReadyInlineRoot` and `layoutReadyAnonymousGridTextWrapper` are exactly `true`. Absence means no marker. |

An ordinary box uses geometry observations and may include explicit fragments.
Inline text has at least one complete `inlineSegments` entry and may use either
geometry/fragment observations or Range-ink observations. Range-ink text has no
ordinary node geometry, scroll state, fragments, or child expectations. An empty
Range-ink array remains a valid observation of significant collapsed whitespace.

Each text segment carries its source ID, inline extent, baseline, line height,
bidi level, whitespace-edge behavior, and following-break behavior. Whitespace
edges are `preserve`, `discard-at-start`, `discard-at-end`, or `discard-at-both`.
Breaks are `prohibited`, `allowed`, `allowed-with-replacement`, or `mandatory`.
Atomic participation carries the same applicable bidi/break information and
refers to an actual box child. It cannot use a text replacement break. Explicit
line-control participation is `forced-break` on an inline `br`; a block `br`
remains a box.

The supported root requires an explicit `useRounding` boolean and a viewport
with width/height constraints and a required `rootContext`. Its context is `root`
or `flex-item`; only a flex-item context includes parent writing mode, parent
direction, and a finite nonnegative host inline size. Child viewport data does
not change the root's context.
Viewport dimensions are exactly `px`, `min-content`, or `max-content`; a supported
pixel constraint must be nonnegative.

Dimensions use an explicit `unit`: numeric `px`, `percent`, or `fraction`;
symbolic `calc` or `sizing` strings; or keyword-only `auto`, `none`, `content`,
`max-content`, `min-content`, `stretch`, `fit-content`, or `contain`.
Percent values are fractions, so `0.5` writes `50%`. Symbolic sizing strings keep
their existing spelling; the adapter does not add a CSS parser.
Structured units are constrained by their property. Margins and insets allow
signed lengths or `auto`; padding, border, and gaps require nonnegative lengths.
Preferred/minimum sizes accept lengths and their sizing keywords; maximum sizes
use `none` instead of `auto`. Only flex basis admits `content`, and fractions are
track maxima, not box dimensions. Numeric font, line-height, flex-basis, size, and
track extents must be nonnegative for supported measurements. Symbolic strings
remain opaque and do not gain an authored-CSS validation path.

Captured style discriminants are closed, including alignment, float/clear,
text alignment, and vertical alignment. Unknown values fail wire validation
before unsupported classification. Alignment uses the fixture consumer's
explicit keywords and supported `safe`/`unsafe` forms.

Track alternatives have explicit `kind` tags: scalar dimensions, line-name
lists, subgrid line-name lists, and function forms. A scalar track must include
`kind: "scalar"`; a subgrid must include `lineNames`, using `[]` for no names.
`fit-content` takes one length limit; `minmax` takes a non-fractional minimum and
a maximum that may be fractional; `repeat` takes a repetition and at least one track. A
repetition is a positive integer, `auto-fill`, or `auto-fit`. Grid placements are
`auto`, numeric lines/spans, or named lines/spans. A named placement has a name
and optional `occurrence`. Omission selects the unindexed named form; an explicit
line occurrence is signed and nonzero, and an explicit span occurrence is
positive. A zero occurrence and a named-placement `value` field are invalid.

## Numeric and XML preservation

Browser numbers remain `f64` during decoding, validation, and lowering.
Coordinates and interval endpoints must be finite; extents must also be
nonnegative. Coupled baseline and line-height values satisfy
`0 <= baseline <= lineHeight`. Zero-height text/control metrics remain legal;
a metric-bearing start boundary has the stricter `lineHeight > 0` rule. Shape
queries and optional intervals have ordered endpoints. Integer identifiers and
indices are bounded to their consuming fixture types; bidi levels are `0..=125`.
Supported text and style strings must contain only XML 1.0 characters. Tabs,
line feeds, carriage returns, and legal Unicode remain unchanged; forbidden
control characters fail at their original node and field location. This check
does not parse symbolic sizing expressions or other opaque style strings.

Only ordinary layout geometry crosses the existing `f32` formatting boundary,
after representability has been checked. Segment metrics, fragments, Range inks,
and dimensions retain their existing `f64` formatting. The root's `useRounding`
chooses smart-rounded or unrounded geometry for the tree. Root `x` and `y` write
as zero; child geometry stays relative to its parent. Scroll extents use the
existing nonnegative difference from naive client measurements.

Source segment IDs retain their helper-provided values. Atomic placeholder
indices count lowered input children, including inline boundaries. Expectation
children omit those boundaries; browser-control observation indices refer to
that filtered expectation order. Range line indices retain the helper's
containing-root line registry. No protocol step renumbers these identities.

XML attribute order, default-value omission, whitespace, escaping, text trimming,
and formatting are preservation contracts. The fixture reader in `support.rs`
remains an independent consumer: it parses the generated XML and compares layout
results without importing these private measurement types. Full-corpus changes
must preserve XML bytes, accounting, and each fixture's parity outcome unless a
separate browser-expectation change is explicitly intended.
