# Лимиты Cursor

## Текущий статус

PoC получает числовые usage/limits Cursor через внутренний endpoint `api2.cursor.sh` и access token, созданный `cursor agent login`.

Если токен не найден, запрос отклонен или формат ответа изменился, PoC использует fallback через стандартную команду `cursor` и подкоманду `agent`. Этот fallback показывает только identity/status/model/tier, потому что текущая проверенная версия Cursor Agent не отдает числовые usage/limits через стабильную CLI-команду.

Справка по CLI: [cli.md](cli.md).

Исследование `api2.cursor.sh`: [api2-cursor-sh.md](api2-cursor-sh.md).

## CLI

Проверенные детали PoC:

- Запускается стандартная команда `cursor` и подкоманда `agent`.
- Явная команда `usage`/`limits` в текущей проверенной версии Cursor Agent не найдена.
- Интерактивный TUI запускается, но числовую usage-сводку через стабильную CLI-команду не отдает.
- Fallback PoC выполняет `cursor agent about` и `cursor agent status`.
- Доступные данные: subscription tier, текущая модель, CLI version и auth status.
- Пользовательский вывод явно показывает, что текущий CLI build не отдает числовые usage/limits.

## Известные варианты получения usage

| Вариант | План/доступность | Статус | Комментарий |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Реализовано в PoC | Использует access token после `cursor agent login`; неофициальный контракт. |
| Cursor CLI `about/status` | Pro/Ultra/Team | Fallback в PoC | Дает identity/auth/model/tier, но не billing usage. |
| Dashboard API `cursor.com/api/...` | Любой | Research-only | Нужна cookie веб-сессии. Высокий security-риск. |
| Admin API `api.cursor.com` | Enterprise | Официальный | Подходит для Enterprise-мониторинга; на Pro/Teams без Enterprise ожидается 403. |

## Рекомендация

Для личного Pro/Ultra/Team основной вариант в PoC - локально авторизованный Cursor Agent и `api2.cursor.sh`. Метод остается неофициальным provider method и требует отдельного security review перед production-использованием.

Для production/enterprise мониторинга предпочтителен официальный Admin API, если он доступен тарифу и дает нужную детализацию.

## Ограничения

- `api2.cursor.sh` и `cursor.com/api/*` не являются публично документированным контрактом и могут измениться без предупреждения.
- Access token короткоживущий; refresh token является чувствительным секретом.
- Автоматическая работа с cookie dashboard должна быть запрещена по умолчанию.
