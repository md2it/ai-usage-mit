# CPA-Manager-Plus

## META

| Field | Value |
|---|---|
| Date researched | 2026-06-28 |
| URL | https://github.com/seakee/CPA-Manager-Plus |
| Relevance | Management panel + usage analytics companion for CLIProxyAPI |
| Pricing | Free (MIT license, no SaaS tier) |
| License | MIT |
| Stars / Forks | 944 / 113 |
| Last release | v1.9.1 — June 26, 2026 |
| Stack | TypeScript/React 19 (frontend), Go 1.24 (backend), SQLite |

## OVERVIEW

CPA-Manager-Plus — это самохостируемый дашборд управления и аналитики, построенный поверх CLIProxyAPI (CPA). Сам по себе не является AI-продуктом: это ops-панель, которая превращает очередь использования CPA в SQLite-журнал запросов и обеспечивает мониторинг, аналитику по стоимости/токенам, управление аккаунтами и интеграцию плагинов. Ориентирован на самохостеров, которые уже запустили CLIProxyAPI и хотят видимость в расходах и квотах. Проект активно разрабатывается: 24 релиза за короткий период, последний — 26 июня 2026 г.

## PRODUCTS

| Компонент | Описание |
|---|---|
| Manager Server (порт 18317) | Go-бэкенд с полным набором функций: мониторинг, аналитика, хранилище |
| CPA Panel (порт 8317) | Лёгкая панель без мониторинга и аналитики — только управление CPA |
| Docker-образ | `seakee/cpa-manager-plus` (Docker Hub) и `ghcr.io/seakee/cpa-manager-plus` |

## FEATURES

- **SQLite-журнал запросов** — «Converts CPA usage queue into SQLite request ledger for live monitoring, historical search, import/export, and analytics» (README)
- **Аналитика стоимости и токенов** — разбивка по модели, провайдеру, аккаунту, API-ключу, проекту, каналу и временному окну
- **Response header observability** (v1.9.0) — выводит «current window, usage rate, recovery time, and plan type» из заголовков квоты Codex
- **Codex account inspection** — проверка квотного окна, статус OAuth-токена, маскировка квоты для скриншотов
- **Синхронизация цен моделей** — из LiteLLM и OpenRouter
- **Плагин-экосистема** (v1.5.0) — маркетплейс, управление, поддержка OAuth-провайдеров
- **Safeguards пула аккаунтов** — авто-восстановление отключённых auth-файлов при сбросе квоты
- **Privacy masking** (v1.8.1) — маскировка данных аккаунта при шаринге скриншотов
- **Admin-key аутентификация** + шифрованное хранилище управляющего ключа CPA
- **Cross-platform packaging** — Linux/macOS/Windows (amd64, arm64)

## PRICING / BUSINESS MODEL

Продукт полностью бесплатный и открытый (MIT). Нет SaaS-уровней, нет платных планов, нет pricing-страницы. Монетизация не просматривается.

## COMPARISON WITH AI-USAGE-MIT

| Измерение | CPA-Manager-Plus | ai-usage-mit |
|---|---|---|
| Цель | Ops-панель и аналитика для CLIProxyAPI | Трекинг AI-использования (TBD) |
| Развёртывание | Self-hosted (Docker / native binary) | — |
| Источник данных | Только CLIProxyAPI / Codex | — |
| Аналитика стоимости | Есть (по модели, ключу, проекту) | — |
| Поддержка провайдеров | Claude Code, Codex, Gemini CLI (через CPA) | — |
| Ценообразование | Бесплатно / MIT | — |
| Целевая аудитория | Самохостеры CPA (преимущественно CN-рынок) | — |
| Лицензия | MIT | — |

**Уровень угрозы: Низкий.** CPA-Manager-Plus — companion-инструмент, жёстко привязанный к CLIProxyAPI. Без CPA он бесполезен. Если ai-usage-mit ориентирован на более широкий или агентский трекинг использования вне экосистемы CLIProxyAPI — прямой конкуренции нет.

## OPEN QUESTIONS

- Каков точный scope ai-usage-mit? Если продукт перекрывает SQLite-аналитику для CPA-пользователей — угроза выше.
- Есть ли планы у авторов CPA-Manager-Plus выйти за пределы CLIProxyAPI (поддержка других прокси / провайдеров напрямую)?
- Насколько велик Chinese self-hosting community как целевой рынок ai-usage-mit?

## SOURCES

- [GitHub репозиторий](https://github.com/seakee/CPA-Manager-Plus)
- [README](https://github.com/seakee/CPA-Manager-Plus#readme)
- [Wiki](https://github.com/seakee/CPA-Manager-Plus/wiki)
- [Releases / Changelog](https://github.com/seakee/CPA-Manager-Plus/releases)
- [CLIProxyAPI (upstream)](https://github.com/router-for-me/CLIProxyAPI)
