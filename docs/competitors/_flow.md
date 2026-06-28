# Competitor Research Flow

## 1. Discover

Sources: community mentions, Reddit, GitHub trending, product launches, user feedback.

Add a stub entry to `_INDEX.yaml` with `status: not_researched`.

## 2. Docs review → `status: docs_reviewed`

In order:

1. **Homepage / landing page** — positioning, tagline, target audience.
2. **Pricing page** — tiers, limits, free plan.
3. **GitHub** (if open-source) — stars, forks, README, recent activity, issues.
4. **Official docs** — feature list, architecture, what is explicitly absent.
5. **Changelog / release notes** — look for removed or deprecated features; quote verbatim with a link.

Stop when you can answer: *What does it do, for whom, at what price, and what does it explicitly not do?*

## 3. Hands-on → `status: hands_on`

Only if docs review leaves material uncertainty about a core feature.

Run the product. Capture screenshots. Note what the UI exposes vs. what docs claim.

## 4. Write `{id}.md`

Sections (in order):

- **META** — date, URL, relevance, pricing, license.
- **OVERVIEW** — one paragraph, plain prose.
- **PRODUCTS** — table if multiple repos/tools.
- **FEATURES** — bullet list; no padding.
- **PRICING / BUSINESS MODEL** — factual, no speculation.
- **COMPARISON WITH AI-USAGE-MIT** — table: dimension / competitor / ai-usage-mit. Then threat level with one-line rationale.
- **OPEN QUESTIONS** — only genuine unknowns that affect the threat assessment.
- **SOURCES** — every URL cited above, as markdown links.

Rules:
- Quote verbatim (with link) anything factually important — especially removed features, pricing, or explicit scope limits.
- No speculation beyond what sources support. Mark gaps as open questions.

## 5. Update `_INDEX.yaml`

- Set `status`.
- Update `supported_interfaces`.
- Write `summary` in 2–4 lines: what it does, key differentiator vs. ai-usage-mit, threat level. Include verbatim quotes for critical facts.
