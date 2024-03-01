# Helius Telegram Bot

Example bot using the [selene-helius-sdk](https://github.com/dougEfresh/selene-helius-sdk/)

## Install

```shell
cargo install --git https://github.com/dougEfresh/selene-helius-bot.git
```

## Prerequisites

Set up a telegram [bot](https://velenux.wordpress.com/2022/09/12/how-to-configure-prometheus-alertmanager-to-send-alerts-to-telegram/)

## Usage

* create your webhook
```shell
selene-helius-bot webhook create --helius-api-key=<your_api_key> --url=<your_webhook_url> [ADDRESSES]...
```

* run your server
* 
```shell
selene-helius-bot serve --help
A telegram bot for helius webhooks, listens on port 3030

Usage: selene-helius-bot serve [OPTIONS] --helius-api-key <HELIUS_API_KEY> --chat-id <CHAT_ID> --teloxide-token <TELOXIDE_TOKEN>

```

* `TELOXIDE_TOKEN` is your bot API token.