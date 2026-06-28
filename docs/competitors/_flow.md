# Analog Research Flow

Note: "competitor" is a working directory/file name. The research object may be a
direct analog, a partial analog, or just a related product we can learn from. Do
not assume market competition or competition for users.

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

1. Prepare `.hands-on/{competitor-id}/` as the only hands-on workspace.
   - Put repos, virtualenvs, caches, test data, logs, and temporary HOME there.
   - Do not install, clone, cache, or write test data elsewhere without explicit user approval.
2. Install the product in `.hands-on/{competitor-id}/`.
3. Run it as a real user would and verify the uncertain core feature directly.
4. Record hands-on findings in the competitor document.
   - Separate actual usage from docs, marketing pages, README files, and changelogs.
   - Note what the UI exposes, what works, what is missing, and what differs from docs.
5. Give the user a short summary.
   - What was installed and tested.
   - What was observed.
   - What changed in the document.
   - How to verify independently.
   - What remains installed or created locally.
6. Wait for the user to verify independently or ask to remove the tested tool and related artifacts.

## 4. Write `{id}.md`

Sections (in order):

- **META** — date, URL, relevance, pricing, license.
- **OVERVIEW** — one paragraph, plain prose.
- **PRODUCTS** — table if multiple repos/tools.
- **FEATURES** — bullet list; no padding.
- **PRICING / BUSINESS MODEL** — factual, no speculation.
- **COMPARISON WITH AI-USAGE-MIT** — table: dimension / competitor / ai-usage-mit. Then threat level with one-line rationale.
- **WHAT WE CAN LEARN** — 1–5 short bullets: what the product solved well, why it matters to us, and whether to adopt, adapt, evaluate, watch, or ignore. Do not use tables here.
- **OPEN QUESTIONS** — only genuine unknowns that affect the threat assessment.
- **SOURCES** — every URL cited above, as markdown links.

Rules:
- Quote verbatim (with link) anything factually important — especially removed features, pricing, or explicit scope limits.
- No speculation beyond what sources support. Mark gaps as open questions.
- Keep learning notes compact. If a lesson needs detailed analysis, move that analysis to a separate product document and leave only a short reference here.

## 5. Update `_INDEX.yaml`

- Set `status`.
- Update `supported_interfaces`.
- Write `summary` in 2–4 lines: what it does, key differentiator vs. ai-usage-mit, threat level. Include verbatim quotes for critical facts.
