
# ЭТАП 1: Сборка
# Используем latest, чтобы точно поддержать Edition 2024 и новые lock-файлы
FROM rust:latest AS builder

WORKDIR /app

# Системные зависимости остаются те же
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Копируем файлы
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY backend/src ./backend/src

# Собираем
WORKDIR /app/backend
RUN cargo build --release


# ЭТАП 2: Рантайм
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарник из билдера
COPY --from=builder /app/backend/target/release/media_dms_api ./media_dms_api

EXPOSE 8080

CMD ["./media_dms_api"]
