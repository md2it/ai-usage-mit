# Cursor api2.cursor.sh

## Статус

Research-документ. Описывает неофициальный способ получения usage/limits Cursor через backend `api2.cursor.sh`.

---

## Проблема

Для личных Pro/Ultra/Team тарифов публичный официальный usage API не подтвержден. При этом access token после `cursor agent login` позволяет получить текущие лимиты через endpoint, который использует клиентский backend Cursor.

---

## Основной Проверенный Endpoint

Endpoint:

```text
POST https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage
```

Заголовки:

```text
Authorization: Bearer <access_token>
Content-Type: application/json
Connect-Protocol-Version: 1
```

Тело:

```json
{}
```

Пример ответа:

```json
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
```

Поля:

- `planUsage.remaining` — оставшийся included usage
- `planUsage.limit` — included usage limit
- `planUsage.totalPercentUsed` — общий процент использования
- `planUsage.autoPercentUsed` — процент использования Auto
- `planUsage.apiPercentUsed` — процент использования API models
- `billingCycleStart` — начало billing cycle в Unix ms
- `billingCycleEnd` — конец billing cycle в Unix ms
- `displayMessage` — человекочитаемое сообщение Cursor

---

## Токены

На macOS после `cursor agent login` токены можно найти в Keychain:

```sh
security find-generic-password -s cursor-access-token -w
security find-generic-password -s cursor-refresh-token -w
```

Access token короткоживущий. При `401` нужен новый login или refresh через OAuth endpoint Cursor.

Refresh token является чувствительным секретом. Приложение не должно читать, логировать или обновлять его без отдельного security review и явного пользовательского сценария.

---

## Альтернативы

| Вариант | План/доступность | Статус | Комментарий |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Реализовано в PoC | Использует access token после `cursor agent login`; неофициальный контракт |
| Dashboard API `cursor.com/api/...` | Любой | Research-only | Нужна cookie веб-сессии; высокий security-риск |
| Admin API `api.cursor.com` | Enterprise | Официальный | Подходит для Enterprise-мониторинга; на Pro/Teams без Enterprise ожидается 403 |

---

## Ограничения

- `api2.cursor.sh` не является публично документированным контрактом
- endpoint может измениться без предупреждения
- response schema должна валидироваться осторожно
- cookie dashboard не должны использоваться как продуктовый механизм по умолчанию
- refresh token нельзя считать обычной настройкой приложения

---

## Рекомендация

Для личного Pro/Ultra/Team сценария основной кандидат — `GetCurrentPeriodUsage` через access token Cursor Agent.

Перед production-использованием нужен отдельный security review:

- какие токены читаются
- где токены хранятся
- какие данные запрещено логировать или сохранять
- как показывается ошибка при истекшем token
- нужно ли приложению самостоятельно делать refresh
