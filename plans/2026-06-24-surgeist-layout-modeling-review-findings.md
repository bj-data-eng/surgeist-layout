# Surgeist Layout Modeling Review Findings

> Record only verified findings from `plans/2026-06-24-surgeist-layout-modeling-review-plan.md`. Do not use this file as an implementation plan.

## Review Boundaries

- Repo: `surgeist-layout`
- Scope: crate-local source, tests, browser parity tooling, generated artifact workflow, and public API reexports from `src/lib.rs`
- Out of scope: code edits, sibling crate fixes, root integration changes, implementation plan
- Standard: `AGENTS.md` and `guidance/surgeist-rust-modeling-guide.md`, especially Type And Value Modeling, Generated Artifacts, Crate Role, Dependency Direction, and Public API Surface

## Recording Rules

- Record findings only after they have exact file and line evidence.
- Keep findings descriptive, not prescriptive. Implementation choices belong in a later implementation plan.
- Group duplicate observations under one finding when they share the same invariant.
- Preserve reviewer provenance so later reviewers can distinguish verified issues from hypotheses.
- If a suspected issue is rejected, record it under Non-Findings with the reason.

## Severity Guide

- **P0:** Current correctness bug likely reachable by normal crate use or parity fixtures.
- **P1:** Strong correctness risk, invariant confusion, or public contract that makes invalid states easy to express.
- **P2:** Internal modeling debt that can cause regressions or makes algorithm phases hard to reason about.
- **P3:** Documentation, naming, or containment issue with low immediate correctness risk.

## Finding Template

Copy this template for each verified finding:

```markdown
### FINDING-ID: Short Title

- Severity:
- Category: correctness-risk | API-hardening | internal-maintainability | tooling-containment
- Confidence: high | medium | low
- Review phase:
- Owner: layout crate | root coordinator | style/css upstream | tooling-only
- Status: verified | needs re-review | accepted | rejected
- Evidence:
  - `path/to/file.rs:line`
- Invariant at risk:
- Observed code shape:
- Expected modeling shape:
- AGENTS/modeling guidance:
- Reproduction or existing coverage:
- Notes:
```

## Findings

No verified findings recorded yet.

## Non-Findings

No rejected findings recorded yet.

## Reconciliation Log

No reconciliation entries recorded yet.
