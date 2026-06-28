# Claude

## Текущий статус

PoC получает usage/limits через Claude CLI. Приложение запускает стандартную команду `claude`, открывает интерактивный TUI и отправляет slash-команду `/usage`.

---

## Provider Method: `claude_cli_usage`

Минимальные команды:

- проверить наличие CLI: `command -v claude`
- проверить версию CLI: `claude --version`
- официальный сайт: https://www.anthropic.com/claude-code
- документация CLI: https://code.claude.com/docs/en/setup

Проверенные детали PoC:

- запускается стандартная команда `claude` с флагом `--no-chrome`, чтобы не открывать дополнительный Chrome integration dialog
- для получения лимитов используется `/usage`
- `/status` по умолчанию открывает вкладку Status без лимитов
- PoC ждет готовность prompt по нижней строке `for shortcuts`
- `/usage` отправляется обычным вводом без bracketed paste
- пользовательский вывод показывает найденные строки `Current session`, `Current week`, `Total cost` и token usage
- парсер учитывает, что часть строк приходит через bare carriage return, поэтому cleaned/compacted output режется по `\n` и `\r`

---

## Ограничения

- полный вывод остается TUI-потоком
- способ зависит от текущего поведения Claude CLI и текста TUI
- запрос через CLI может занимать заметное время
- нужна проверка, расходует ли такой запрос пользовательские лимиты

---

## Другие варианты

| Вариант | Статус | Комментарий |
|---|---|---|
| Официальный API | Не исследовано | Может относиться к API-аккаунтам, но не обязательно к подписочным лимитам Claude Code |
| Локальные transcript JSONL | Кандидат для usage history | Проверить `~/.config/claude/projects/**/*.jsonl`, `~/.claude/projects/**/*.jsonl` и Xcode ClaudeAgentConfig; хорошо для токенов/стоимости/сессий, но не всегда для официального остатка лимита |
| Claude Code statusline `rate_limits` | Кандидат для live limits | Hook получает JSON через stdin от Claude Code и может дать официальный live-сигнал по 5h/7d лимитам; требует настройки statusline |
| Локальная SQLite/cache | Вспомогательный слой | Например `~/.claude/usage.db` у `claude-usage`: удобно для dashboard и инкрементального сканирования, но это derived data, не первичный источник |
| Frontend/dashboard API | Research-only | Возможен только при понятной и безопасной работе с cookie/session token |
| Наблюдение трафика | Research-only | Не рассматривать как продуктовый механизм |
