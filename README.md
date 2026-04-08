# MediaDMS: Полная документация

## 1. Назначение проекта

**MediaDMS** — это клиент-серверная система для работы с музыкальными данными:

* песни
* альбомы
* авторы

Система поддерживает:

* фильтрацию
* сортировку
* добавление данных через UI
* использование алгоритмов SAOD:

  * digital sort (radix sort)
  * binary search
  * A2Tree (опционально)

---

## 2. Структура проекта

```
MediaDMS/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── service.rs
│       ├── repo.rs
│       ├── models.rs
│
├── saod/
│   ├── Cargo.toml
│   └── src/
│       ├── digital_sort.rs
│       ├── binary_search.rs
│       ├── a2_tree.rs
│
├── client/
│   ├── app.py
│   └── covers/
│       └── placeholder.png
│
├── database/
│   ├── docker-compose.yml
│   └── init_db/01_schema.sql
│
├── .env
├── Cargo.toml
└── README.md
```

---

## 3. Архитектура

```
[ PyQt6 Client ] ⇄ HTTP ⇄ [ Rust Backend (Axum) ] ⇄ SQL ⇄ [ PostgreSQL ]
```

---

## 4. Компоненты системы

### 4.1 Клиент (client/app.py)

GUI-приложение на:

* PyQt6 — интерфейс
* requests — HTTP-запросы
* pygame — воспроизведение аудио

Функции клиента:

* поиск треков
* фильтрация по:

  * id
  * автору
  * альбому
  * году
  * длительности
* сортировка по клику на колонку
* добавление:

  * авторов
  * альбомов
  * песен
* отображение обложек

---

### 4.2 Backend (Rust + Axum)

API-сервер:

* Axum 0.8
* sqlx (PostgreSQL)
* асинхронная обработка запросов

Маршруты:

```
GET  /health
POST /api/v1/query
GET  /api/v1/logs
```

---

### 4.3 База данных (PostgreSQL)

Используется PostgreSQL 16 (Docker)

Связи:

```
authors → albums → songs
```

Таблицы:

* authors
* albums
* songs
* author_images

---

### 4.4 SAOD алгоритмы (saod/)

Используются в backend:

* **digital sort (radix sort)** — сортировка
* **binary search** — быстрый поиск
* **A2Tree** — структура хранения (опционально)

---

## 5. Конфигурация

Файл `.env`:

```
POSTGRES_USER=admin
POSTGRES_PASSWORD=access
POSTGRES_DB=media_dms_db
DB_PORT=5531

DATABASE_URL=postgres://admin:access@localhost:5531/media_dms_db
```

Опционально:

```
ENABLE_A2_TREE=1
```

---

## 6. Запуск проекта

### 6.1 Запуск базы данных

```
docker-compose -f database/docker-compose.yml up -d
```

Проверка:

```
docker ps
```

---

### 6.2 Запуск backend

```
cargo run
```

Сервер будет доступен:

```
http://127.0.0.1:8080
```

---

### 6.3 Запуск клиента

```
cd client
python3 app.py
```

---

### 6.4 Запуск на Raspberry Pi 3

Клиент использует PyQt5 вместо PyQt6 (файл `client_rpi.py`), так как PyQt6 недоступен на Raspberry Pi OS.

#### Установка зависимостей

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y python3-pyqt5 libsdl2-mixer-2.0-0 libsdl2-2.0-0
pip3 install pygame requests --break-system-packages
```

#### Установка Rust (для backend)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Установка Docker (для базы данных)

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
```

После этого перелогиниться или выполнить:

```bash
newgrp docker
```

#### Запуск

```bash
# База данных
docker-compose -f database/docker-compose.yml up -d

# Backend
cargo run

# Клиент (в отдельном терминале)
cd client
python3 client_rpi.py
```

#### Если запуск через SSH (с монитором)

```bash
export DISPLAY=:0
python3 client_rpi.py
```

#### Если нет монитора (headless)

```bash
sudo apt install -y xvfb
Xvfb :1 -screen 0 1024x768x16 &
DISPLAY=:1 python3 client_rpi.py
```

#### Возможные проблемы

| Ошибка | Решение |
|---|---|
| `No module named PyQt5` | `sudo apt install python3-pyqt5` |
| `cannot connect to X server` | `export DISPLAY=:0` перед запуском |
| `pygame.error: No available audio device` | `sudo apt install pulseaudio` или запустить с `SDL_AUDIODRIVER=dummy python3 client_rpi.py` |
| Медленная компиляция backend | Нормально для RPi 3, первая сборка занимает 20–40 минут |

---

## 7. API

### 7.1 Health

```
GET /health
```

Ответ:

```
{ "status": "ok" }
```

---

### 7.2 Query

```
POST /api/v1/query
```

Пример:

```json
{
  "method": "read",
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

---

### 7.3 Добавление данных

Пример (song):

```json
{
  "method": "write",
  "entity": "song",
  "data": {
    "album_id": 1,
    "song_name": "Numb",
    "duration_sec": 180
  }
}
```

---

## 8. Логика работы

### Поиск

1. Пользователь вводит фильтры в UI
2. Клиент отправляет POST-запрос
3. Backend формирует SQL
4. PostgreSQL возвращает данные
5. Клиент отображает таблицу

---

### Сортировка

1. Клик по колонке
2. Клиент отправляет `sort`
3. Backend применяет digital sort
4. Результат возвращается

---

### Добавление

1. Пользователь вводит данные
2. Клиент отправляет `write`
3. Backend делает INSERT
4. UI обновляется

---

## 9. Тестирование

### Unit tests

```
cargo test -p server
```

---

### Проверка сборки

```
cargo check
```

---

### Ручное тестирование

1. Запустить БД
2. Запустить backend
3. Запустить клиент
4. Проверить:

   * поиск
   * сортировку
   * добавление

---

## 10. Troubleshooting

### Ошибка подключения к БД

Проверь:

* запущен ли контейнер
* совпадают ли креды в `.env`
* порт не занят

---

### Connection refused

Значит backend не запущен:

```
cargo run
```

---

### Данные не отображаются

* проверить backend логи
* проверить SQL
* проверить JSON ответ

---

## 11. Полезные команды

```
# База
docker-compose -f database/docker-compose.yml up -d
docker-compose -f database/docker-compose.yml down

# Backend
cargo run
cargo check
cargo test -p server

# Client (обычный)
python3 app.py

# Client (Raspberry Pi)
python3 client_rpi.py
```

---

## 12. Итог

Проект реализует:

* полноценную клиент-серверную архитектуру
* работу с PostgreSQL
* интеграцию алгоритмов SAOD
* GUI-интерфейс на PyQt6
* REST API на Rust

---