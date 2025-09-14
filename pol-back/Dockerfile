FROM debian:bookworm-slim

WORKDIR /app

# Установите необходимые библиотеки
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Копируем собранный бинарник из локальной системы
COPY target/release/user-backend ./

# Копируем миграции (если нужны)
COPY migrations/ ./migrations/

# Создаем пользователя для безопасности
RUN adduser --disabled-password --gecos '' appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8000
CMD ["./user-backend"]
