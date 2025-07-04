# 💻 zerotier-manager

An interactive CLI tool to manage your ZeroTier controller directly from the terminal.

### ⚙️ Configuration

To use the tool, you need to set the following environment variables:

1.  **`TOKEN`** (Required)
    You must set the `TOKEN` environment variable to your ZeroTier API token. You can find more information on how to get a token [here](https://docs.zerotier.com/api/tokens/#zerotierone-service-token).

2.  **`URL`** (Optional)
    If your controller is not running at the default `http://localhost:9993`, you can set the `URL` environment variable to point to your custom address.

### ▶️ Usage

```bash
# Set the required token
export TOKEN=your_secret_zerotier_token

# Optionally set a custom URL
# export URL=http://your.custom.controller:9993

# Run the manager
./zerotier-manager
```

---

Интерактивная утилита для управления вашим контроллером ZeroTier прямо из терминала.

### ⚙️ Конфигурация

Для работы утилиты необходимо задать следующие переменные окружения:

1.  **`TOKEN`** (Обязательно)
    Вам необходимо установить переменную окружения `TOKEN`, указав в ней ваш API токен от ZeroTier. Подробнее о том, как получить токен, можно прочитать [здесь](https://docs.zerotier.com/api/tokens/#zerotierone-service-token).

2.  **`URL`** (Опционально)
    Если ваш контроллер запущен по адресу, отличному от стандартного `http://localhost:9993`, вы можете задать переменную окружения `URL`, указав ваш адрес.

### ▶️ Использование

```bash
# Устанавливаем обязательный токен
export TOKEN=ваш_секретный_токен_zerotier

# Опционально указываем свой URL
# export URL=http://your.custom.controller:9993

# Запускаем менеджер
./zerotier-manager
```