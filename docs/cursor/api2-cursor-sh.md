# Cursor: api2.cursor.sh

## Статус

Research-документ. Описывает неофициальные варианты получения usage/limits Cursor, найденные в ходе обсуждения и проверки PoC.

## Проблема

Для Pro (как у вас) официального API usage нет, но токен из cursor agent login уже позволяет получить лимиты.

## Из обсуждения с Composer 2.5 

### Сессия про 3 подхода

Три подхода

┌────────────────────────────────────┬────────────────┬─────────────────────────────┐
│ Подход                             │ План           │ Стабильность                │
├────────────────────────────────────┼────────────────┼─────────────────────────────┤
│ Admin API (api.cursor.com)         │ Enterprise     │ Официальный                 │
├────────────────────────────────────┼────────────────┼─────────────────────────────┤
│ IDE backend (api2.cursor.sh)       │ Pro/Ultra/Team │ Неофициальный, но удобный   │
├────────────────────────────────────┼────────────────┼─────────────────────────────┤
│ Dashboard API (cursor.com/api/...) │ Любой          │ Неофициальный, нужна cookie │
└────────────────────────────────────┴────────────────┴─────────────────────────────┘

CURSOR_API_KEY и cursor agent about не показывают billing usage — только identity и per-agent tokens.

────────────────────────────────────────



Вариант 1: через токен CLI (рекомендую для Pro)

После cursor agent login токен лежит в macOS Keychain:

ACCESS_TOKEN="$(security find-generic-password -s cursor-access-token -w)"
USER_ID="$(cursor agent status --format json | python3 -c 'import sys,json;
print(json.load(sys.stdin)["userInfo"]["userId"])')"
curl -s -X POST 'https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage' \
   -H "Authorization: Bearer $ACCESS_TOKEN" \
   -H 'Content-Type: application/json' \
   -H 'Connect-Protocol-Version: 1' \
   -d '{}'

Пример ответа у вас сейчас:

{
   "planUsage": {
   "remaining": 2000,
   "limit": 2000,
   "autoPercentUsed": 0,
   "apiPercentUsed": 0,
   "totalPercentUsed": 0
   },
   "displayMessage": "You've used 0% of your included usage",
   "billingCycleStart": "1782614703000",
   "billingCycleEnd": "1785206703000"
}

Поля:
• planUsage.remaining / limit — included usage (в центах)
• totalPercentUsed — общий процент
• billingCycleStart / End — период биллинга (Unix ms)

Мини-скрипт с человекочитаемым выводом:

#!/usr/bin/env bash
set -euo pipefail
token="$(security find-generic-password -s cursor-access-token -w 2>/dev/null || true)"
if [[ -z "$token" ]]; then
   echo "Нет токена. Запустите: cursor agent login" >&2
   exit 1
fi
json="$(curl -s -X POST \
   'https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage' \
   -H "Authorization: Bearer $token" \
   -H 'Content-Type: application/json' \
   -H 'Connect-Protocol-Version: 1' \
   -d '{}')"
python3 - <<'PY' "$json"
import json, sys
from datetime import datetime, timezone
d = json.loads(sys.argv[1])
pu = d.get("planUsage", {})
used = pu.get("limit", 0) - pu.get("remaining", 0)
limit = pu.get("limit", 0)
def fmt_cents(c): return f"${c/100:.2f}"
start = datetime.fromtimestamp(int(d["billingCycleStart"])/1000, tz=timezone.utc)
end   = datetime.fromtimestamp(int(d["billingCycleEnd"])/1000, tz=timezone.utc)
print(f"Plan usage: {fmt_cents(used)} / {fmt_cents(limit)} ({pu.get('totalPercentUsed', 0)}%)")
print(f"Auto: {pu.get('autoPercentUsed', 0)}%  |  API models: {pu.get('apiPercentUsed', 0)}%")
print(f"Cycle: {start.date()} → {end.date()}")
print(d.get("displayMessage", ""))
PY

Токен истекает — при 401 обновите через cursor agent login или refresh:

REFRESH="$(security find-generic-password -s cursor-refresh-token -w)"
curl -s -X POST 'https://api2.cursor.sh/oauth/token' \
   -H 'Content-Type: application/json' \
   -d "{\"grant_type\":\"refresh_token\",\"refresh_token\":\"$REFRESH\"}"

────────────────────────────────────────

Вариант 2: через cookie дашборда

1. Откройте cursor.com/dashboard/usage (https://cursor.com/dashboard/usage)
2. DevTools → Application → Cookies → WorkosCursorSessionToken
3. Запрос:

curl -s 'https://cursor.com/api/usage-summary' \
   -H 'Cookie: WorkosCursorSessionToken=YOUR_COOKIE'

Плюс: тот же endpoint, что у веб-UI. Минус: cookie нужно обновлять вручную.

────────────────────────────────────────

Вариант 3: официальный Admin API (Enterprise)

curl -X POST 'https://api.cursor.com/teams/spend' \
   -u 'crsr_YOUR_ADMIN_KEY:' \
   -H 'Content-Type: application/json' \
   -d '{"page": 1, "pageSize": 100}'

Ключ с scope admin:* в Dashboard → API Keys (https://cursor.com/dashboard). На Pro/Teams без Enterprise будет 403.

────────────────────────────────────────

Замечания

1. api2.cursor.sh и cursor.com/api/* — не документированы, могут сломаться без предупреждения.
2. В вашем ai-usage-mit сейчас Cursor берётся только из cursor agent about/status — там нет usage; имеет смысл
   добавить вызов GetCurrentPeriodUsage.
3. Для production-мониторинга — только Enterprise Admin API; для личного Pro — скрипт выше.

### Сессия про api2.cursor.sh

**Кратко:**  
Сессия и авторизация для api2.cursor.sh завязаны на OAuth2, типичный рабочий процесс — через refresh-token, получаемый при входе (например, `cursor agent login`).

**Получение access token:**
- Обычно используется рефреш-токен (`refresh_token`), сохранённый в Keychain или после первого логина через CLI.
- Пример получения нового токена см. выше в разделе "Токен истекает".

**Использование:**  
С полученным access_token можно делать авторизованные вызовы к api2.cursor.sh (например, `GET /api/usage/summaries/current`), подставляя его в заголовок:

```sh
curl 'https://api2.cursor.sh/api/usage/summaries/current' \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

**Примечания:**
- Срок жизни access_token обычно 1 час, потом нужен новый через refresh.
- Сброс refresh-токена (отозвался, изменился пароль, сменился SSO) приведёт к необходимости заново войти через CLI.
- Сам refresh-токен можно посмотреть у себя после `cursor agent login`:  
  `security find-generic-password -s cursor-refresh-token -w`

**Относительно сессии и лимитов:**
- `api2.cursor.sh/api/usage/summaries/current` возвращает usage для текущего пользователя и тарифного плана.
- Для мониторинга своих Pro/Teams лимитов это основной endpoint (до Enterprise).

**Возможные проблемы:**
- API не публично документирован, возможны внезапные изменения.
- При 401 - скорее всего, токен протух или refresh недействителен (например, смена устройства или сброс пароля).
- Иногда (редко) CLI может сбросить refresh-токен локально — надо релогиниться.

**Диагностика и отладка:**
- Можно запускать с `-v` (curl) для просмотра headers/ответа.
- Если лимиты вдруг не отображаются — смотрите, какой access_token реально использует CLI (можно отловить в debug-логах).

**Ссылки:**
- [OAuth2 Authorization Code Flow (общая инфа)](https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow)
- [Cursor CLI repo (на случай поиска внутренних деталей)](https://github.com/getcursor/cursor)

---
