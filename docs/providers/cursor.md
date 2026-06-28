# Cursor

## Текущий статус

PoC получает числовые usage/limits Cursor через внутренний endpoint `api2.cursor.sh` и access token, созданный `cursor agent login`.

Если токен не найден, запрос отклонен или формат ответа изменился, PoC использует fallback через стандартную команду `cursor` и подкоманду `agent`. Этот fallback показывает только identity/status/model/tier, потому что текущая проверенная версия Cursor Agent не отдает числовые usage/limits через стабильную CLI-команду.

Исследование `api2.cursor.sh`: [../references/cursor-api2-cursor-sh.md](../references/cursor-api2-cursor-sh.md).

---

## Provider Method: `cursor_api2_usage`

Основной PoC-метод получает числовые usage/limits через `api2.cursor.sh`.

Метод:

- использует access token после `cursor agent login`
- вызывает `GetCurrentPeriodUsage`
- возвращает included usage, проценты использования и billing cycle
- зависит от неофициального backend-контракта Cursor
- требует отдельного security review перед production-использованием

Подробности endpoint: [../references/cursor-api2-cursor-sh.md](../references/cursor-api2-cursor-sh.md).

---

## Provider Method: `cursor_cli_agent_status`

Минимальные команды:

- проверить наличие CLI: `command -v cursor`
- проверить версию CLI: `cursor --version`
- официальный сайт: https://cursor.com
- страница установки: https://cursor.com/install

Fallback PoC-метод не отдает числовые usage/limits, но помогает показать состояние Cursor Agent.

Проверенные детали:

- запускается стандартная команда `cursor` и подкоманда `agent`
- явная команда `usage`/`limits` в текущей проверенной версии Cursor Agent не найдена
- интерактивный TUI запускается, но числовую usage-сводку через стабильную CLI-команду не отдает
- fallback PoC выполняет `cursor agent about` и `cursor agent status`
- доступные данные: subscription tier, текущая модель, CLI version и auth status
- пользовательский вывод явно показывает, что текущий CLI build не отдает числовые usage/limits

---

## Известные варианты получения usage

| Вариант | План/доступность | Статус | Комментарий |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Реализовано в PoC | Использует access token после `cursor agent login`; неофициальный контракт |
| Cursor CLI `about/status` | Pro/Ultra/Team | Fallback в PoC | Дает identity/auth/model/tier, но не billing usage |
| Dashboard API `cursor.com/api/...` | Любой | Research-only | Нужна cookie веб-сессии; высокий security-риск |
| Admin API `api.cursor.com` | Enterprise | Официальный | Подходит для Enterprise-мониторинга; на Pro/Teams без Enterprise ожидается 403 |

---

## Рекомендация

Для личного Pro/Ultra/Team основной вариант в PoC - локально авторизованный Cursor Agent и `api2.cursor.sh`. Метод остается неофициальным provider method и требует отдельного security review перед production-использованием.

Для production/enterprise мониторинга предпочтителен официальный Admin API, если он доступен тарифу и дает нужную детализацию.

---

## Ограничения

- `api2.cursor.sh` и `cursor.com/api/*` не являются публично документированным контрактом и могут измениться без предупреждения
- access token короткоживущий
- refresh token является чувствительным секретом
- автоматическая работа с cookie dashboard должна быть запрещена по умолчанию
