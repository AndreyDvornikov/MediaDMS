# MediaDMS: Полная документация

## 1. Назначение проекта

`MediaDMS` — API для поиска музыкальных данных (песни, альбомы, авторы) с:
- фильтрацией,
- сортировкой,
- использованием SAOD-алгоритмов:
  - digital sort (radix sort),
  - binary search,
  - A2Tree (опционально).

---

## 2. Структура репозитория

```text
MediaDMS/
├── Cargo.toml                  # Workspace root (server + saod)
├── Cargo.lock
├── .env
├── .gitignore
├── database/
│   ├── docker-compose.yml
│   └── init_db/01_schema.sql
├── server/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Axum startup + DB connect
│       ├── models.rs
│       ├── repo.rs             # PgRepository (sqlx)
│       ├── service.rs          # business logic + SAOD integration
│       ├── error.rs
│       └── logging.rs
├── saod/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── digital_sort.rs
│       ├── binary_search.rs
│       └── a2_tree.rs
└── docs/
    └── README.md
```

---

## 3. Архитектура

### 3.1 База данных (`database/`)
- PostgreSQL 16 (`postgres:16-alpine`)
- Инициализация схемы из `database/init_db/01_schema.sql`
- Связи:
  - `authors` -> `albums` (`ON DELETE CASCADE`)
  - `albums` -> `songs` (`ON DELETE CASCADE`)
  - `authors` -> `author_images` (`ON DELETE CASCADE`)

### 3.2 Сервер (`server/`)
- `Axum 0.8`
- Маршруты:
  - `GET /health`
  - `POST /api/v1/query`
  - `GET /api/v1/logs`
- Репозиторий:
  - `MediaRepository` (async trait)
  - `PgRepository` на `sqlx::PgPool`

### 3.3 SAOD (`saod/`)
- Подключен как workspace dependency в `server/Cargo.toml`
- Используется в `server/src/service.rs`:
  - `radix_sort_by_selected_field` (сортировка)
  - `binary_search_by` и `equal_range_by` (быстрый поиск)
  - `A2Tree` (опционально)

---

## 4. Конфигурация окружения

Файл `.env` в корне:

```env
POSTGRES_USER=admin
POSTGRES_PASSWORD=access
POSTGRES_DB=media_dms_db
DB_PORT=5531
DATABASE_URL=postgres://admin:access@localhost:5531/media_dms_db
```

Дополнительно (опционально):

```env
ENABLE_A2_TREE=1
```

Если включить `ENABLE_A2_TREE=1`, сервис строит дерево выдачи через `A2Tree`.

---

## 5. Запуск проекта

### 5.1 Поднять БД

```bash
docker-compose -f database/docker-compose.yml up -d
```

Проверка контейнера:

```bash
docker ps
```

Должен быть контейнер `mediadms_postgres`.

### 5.2 Запустить сервер

Из корня репозитория:

```bash
cargo run
```

Workspace настроен так, что по умолчанию запускается `server`.

---

## 6. API и примеры

### 6.1 Health

```http
GET /health
```

Ожидаемо:

```json
{"status":"ok"}
```

### 6.2 Query

```http
POST /api/v1/query
Content-Type: application/json
```

Пример запроса (songs):

```json
{
  "entity": "song",
  "filters": {
    "author": "Linkin Park",
    "year": { "min": 2000, "max": 2010 }
  },
  "sort": {
    "field": "year",
    "order": "desc"
  }
}
```

### 6.3 Logs

```http
GET /api/v1/logs
```

---

## 7. Как тестировать

### 7.1 Unit tests

```bash
cargo test -p server
```

Тестируются:
- фильтрация,
- сортировка через SAOD,
- базовые сценарии сервиса.

### 7.2 Проверка сборки

```bash
cargo check
```

### 7.3 Интеграционная проверка руками

1. Поднять PostgreSQL (`docker-compose ... up -d`)
2. Запустить сервер (`cargo run`)
3. Выполнить `POST /api/v1/query`
4. Убедиться, что:
   - ответ приходит из PostgreSQL,
   - сортировка соответствует параметрам запроса.

---

## 8. Troubleshooting

### Ошибка: `PoolTimedOut` при старте сервера

Это означает, что сервер не смог получить соединение с PostgreSQL за отведенное время.

Проверь:
1. Контейнер БД запущен:
   - `docker ps`
2. Совпадают порт и креды:
   - `.env` (`DB_PORT`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `DATABASE_URL`)
3. Порт не занят другим процессом.
4. БД успела стартовать (особенно после первого запуска).

В `server/src/main.rs` реализованы:
- retry-подключение к БД,
- логирование попыток,
- подсказка в сообщении об ошибке.

---

## 9. Полезные команды

```bash
# Поднять БД
docker-compose -f database/docker-compose.yml up -d

# Остановить БД
docker-compose -f database/docker-compose.yml down

# Запуск сервера
cargo run

# Проверка сборки
cargo check

# Тесты
cargo test -p server
```
