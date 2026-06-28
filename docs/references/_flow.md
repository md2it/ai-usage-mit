# Analog Research Flow

Note: "reference" is the working directory/file naming convention. The research
object may be a direct analog, a partial analog, or just a related product we can
learn from. Do not assume market overlap or user overlap.

## Document hierarchy

This directory contains these document roles:

- **README.md** — directory map. Use it to understand where to look first.
- **_INDEX.yaml** — quick overview across tools. Use it when you need status, short summaries, and fast comparison signals.
- **{reference-id}.md** — detailed analysis of one tool or product. Use it when you need evidence, documentation notes, hands-on findings, open questions, and lessons learned.
- **_flow.md** — research process. Use it only when conducting or updating research.

## 1. Discover

Sources: community mentions, Reddit, GitHub trending, product launches, user feedback.

Add a stub entry to `_INDEX.yaml` with `status: not_researched`.

## 2. Docs review → `status: docs_reviewed`

Goal: answer the required `_INDEX.yaml` comparison questions from documentation:

- `compare.supported_interfaces`
- `compare.providers`
- `compare.notifications`
- `compare.hard_stop`

In order:

1. **Homepage / landing page** — positioning, tagline, target audience.
2. **Pricing page** — tiers, limits, free plan.
3. **GitHub** (if open-source) — stars, forks, README, recent activity, issues.
4. **Official docs** — feature list, architecture, what is explicitly absent.
5. **Changelog / release notes** — look for removed or deprecated features; quote verbatim with a link.

Stop when you can answer the four comparison questions above and summarize: *What does it do, for whom, at what price, and what does it explicitly not do?*

If documentation reveals other relevant nuances, do not ignore them. Put them into the appropriate `{id}.md` sections and reflect only the short product relevance signal in `_INDEX.yaml` `summary`.

## 3. Hands-on → `status: hands_on`

Only if docs review leaves material uncertainty about a core feature.

Goal: verify the uncertain `_INDEX.yaml` comparison questions by running the product:

- `compare.supported_interfaces`
- `compare.providers`
- `compare.notifications`
- `compare.hard_stop`

1. Prepare `.hands-on/{reference-id}/` as the only hands-on workspace.
   - Put repos, virtualenvs, caches, test data, logs, and temporary HOME there.
   - Do not install, clone, cache, or write test data elsewhere without explicit user approval.
2. Install the product in `.hands-on/{reference-id}/`.
3. Run it as a real user would and verify the uncertain core feature directly.
4. Record hands-on findings in the reference document.
   - Separate actual usage from docs, marketing pages, README files, and changelogs.
   - Note what the UI exposes, what works, what is missing, and what differs from docs.
   - Put additional relevant nuances into the appropriate `{id}.md` sections; keep `_INDEX.yaml` focused on `compare` and `summary`.
5. Give the user a short summary.
   - What was installed and tested.
   - What was observed.
   - What changed in the document.
   - How to verify independently.
   - What remains installed or created locally.
6. Wait for the user to verify independently or ask to remove the tested tool and related artifacts.

## 4. Write `{id}.md`

Sections (in order):

- **CONCLUSION** — fill only after the relevant research stage is done.
  - **After documentation** — 1–5 short bullets with the documentation-level conclusion.
  - **After hands-on** — 1–10 short bullets with verified usage conclusions. Include how easy it was to install, required dependencies, how it reads data, and how convenient it is to use.
  - **Comparison to ai-usage-mit** — compact comparison and relevance assessment.
  - **What we can learn** — 1–5 short bullets: what the product solved well, why it matters to us, and whether to adopt, adapt, evaluate, watch, or ignore. Do not use tables here.
- **META** — date, URL/repository, package, checked version, relevance, pricing/business model, license, and sources.
- **DOCUMENTATION**
  - **Overview** — one paragraph, plain prose.
  - **Claimed features** — bullet list; no padding.
- **HANDS-ON CHECK** — required when `status: hands_on`.
  - **Overview** — what was installed or run, version/command if known, and the purpose of the check.
  - **Installing** — install path, dependencies, install complexity, and isolation notes.
  - **Data access** — how the tool finds or imports usage/spending data.
  - **Verified behavior** — what was confirmed by running the tool.
  - **Problems** — what failed, was confusing, was missing, or differed from docs, marketing pages, README files, or changelogs.
- **OPEN QUESTIONS** — only genuine unknowns that affect the relevance assessment.

Rules:
- Quote verbatim (with link) anything factually important — especially removed features, pricing, or explicit scope limits.
- No speculation beyond what sources support. Mark gaps as open questions.
- Avoid Markdown tables unless they materially improve readability. If it is unclear whether a list or a table is better, prefer a list.
- Keep learning notes compact. If a lesson needs detailed analysis, move that analysis to a separate product document and leave only a short reference here.

## 5. Update `_INDEX.yaml`

- Set `status`.
- Update `compare` after research:
  - `supported_interfaces` — array of supported interfaces, for example `cli`, `api`, `macos`, `windows`, `linux`, `android`, `iphone`.
  - `providers` — array of supported providers or data sources, for example `Claude`, `Cursor`, `Codex`, `Mistral`.
  - `notifications` — `true`, `false`, or `null` if not checked.
  - `hard_stop` — `true`, `false`, or `null` if not checked. Use this for hard spend/usage stop or enforced limit-control capabilities.
- Write `summary` in 2–4 lines: what it does, key differentiator vs. ai-usage-mit, and reference relevance. Include verbatim quotes for critical facts.
