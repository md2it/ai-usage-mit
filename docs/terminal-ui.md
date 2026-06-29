# Terminal UI

Документ описывает фактический терминальный интерфейс `ai-usage`.

---

## Общий Формат

Каждый ответ `ai-usage` печатается внутри общей рамки.

Верхняя рамка:

```text
=-=-=-=-=-=-=-=-=-=-=-=-= AI USAGE =-=-=-=-=-=-=-=-=-=-=-=-=
```

Нижняя рамка:

```text
=-=-=-=-=-=-=-=-=-=-=-=-=-= DONE =-=-=-=-=-=-=-=-=-=-=-=-=-=
=-=-=-=-=-=-=-=-=-=-=-=-=-= PART =-=-=-=-=-=-=-=-=-=-=-=-=-=
=-=-=-=-=-=-=-=-=-=-=-=-=-= FAIL =-=-=-=-=-=-=-=-=-=-=-=-=-=
```

Перед верхней рамкой, после верхней рамки, перед нижней рамкой и после нижней рамки печатается пустая строка.

Статусы:

| Статус | Значение |
| --- | --- |
| `DONE` | Все запрошенные источники вернули результат или корректное unavailable-состояние. |
| `PART` | Часть источников вернула результат, часть завершилась ошибкой. |
| `FAIL` | Команда не получила пригодный результат. |

---

## Help

`ai-usage --help` использует общую рамку.

Формат:

```text

=-=-=-=-=-=-=-=-=-=-=-=-= AI USAGE =-=-=-=-=-=-=-=-=-=-=-=-=

Usage:
  ai-usage [OPTIONS]

Options:
  --help, -h      Show this help
  --init-config   Create the user config file if it does not exist
  --all, -a       Query all current sources, ignoring config defaults
  --codex-cli     Query Codex through the Codex CLI
  --claude-cli    Query Claude through the Claude CLI
  --cursor-api2   Query Cursor through api2.cursor.sh

Config:
  ~/.config/ai-usage/config.toml

  default_sources = ["codex_cli", "claude_cli", "cursor_api2"]


=-=-=-=-=-=-=-=-=-=-=-=-=-= DONE =-=-=-=-=-=-=-=-=-=-=-=-=-=

```

---

## Ошибки CLI

Ошибки CLI печатаются внутри общей рамки.

Формат:

```text

=-=-=-=-=-=-=-=-=-=-=-=-= AI USAGE =-=-=-=-=-=-=-=-=-=-=-=-=

ai-usage: unknown argument `--bad`

=-=-=-=-=-=-=-=-=-=-=-=-=-= FAIL =-=-=-=-=-=-=-=-=-=-=-=-=-=

```

---

## Блок Источника

Каждый готовый источник печатается отдельным блоком.

Заголовок блока:

```text
            ~~~~~~~~~~ CURSOR-API2 ~~~~~~~~~~
            ~~~~~~~~~~ CODEX-CLI ~~~~~~~~~~
            ~~~~~~~~~~ CLAUDE-CLI ~~~~~~~~~~
```

После заголовка печатается пустая строка, затем результат источника.

Формат:

```text
            ~~~~~~~~~~ CURSOR-API2 ~~~~~~~~~~

Cursor usage:
Cursor api2 usage unavailable: token not found; run `cursor agent login`

```

---

## Loader

Loader показывает активную работу источника и не показывает процент прогресса.

Формат:

```text
⠋ waiting codex-cli
⠙ waiting claude-cli
```

Unicode spinner frames:

```text
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

ASCII spinner frames:

```text
- \ | /
```

ASCII spinner используется, когда stdout не является TTY или окружение не выглядит как UTF-8.

Loader начинает отображаться, если источник работает дольше `350ms`.

Если источник завершился до первого показа loader-а, loader не печатается.

После завершения источника loader очищается, затем печатается блок результата источника.

---

## Параллельная Модель

Выбранные источники запускаются параллельно.

Модель выполнения:

```text
provider worker threads
        ↓
channel events
        ↓
cli event loop
        ↓
terminal renderer
```

Если несколько источников ожидаются одновременно, отображается несколько строк loader-а.

Формат:

```text
⠋ waiting codex-cli
⠙ waiting claude-cli
```

Когда источник завершился, его loader очищается, результат печатается сразу по готовности.

---

## Цвет

Terminal UI не использует цвет.

Вывод не содержит ANSI color codes для рамок, заголовков, loader-а и контента.

---

## Очистка Loader-А

В интерактивном терминале loader перерисовывается на месте.

При каждом обновлении:

1. предыдущие loader-строки очищаются;
2. текущие loader-строки печатаются заново;
3. курсор остается в зоне loader-а.

При завершении источника loader очищается перед печатью результата.

При завершении работы `TerminalUi` loader очищается через `Drop`.

---

## Архитектурные Границы

Размещение:

```text
src/cli/mod.rs
  - парсит аргументы
  - запускает provider worker threads
  - получает события через channel
  - передает состояние в terminal renderer
  - печатает результаты источников

src/infra/loader.rs
  - выбирает unicode/ascii spinner
  - рисует loader-строки
  - очищает loader-строки
  - печатает рамки и заголовки

src/get_limits.rs
  - вызывает provider methods
  - возвращает нормализованный SourceReport

src/providers/*
  - получают данные источника
  - не рисуют terminal UI
```
