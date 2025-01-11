# Centurion

Centurion is a bot to claim minecraft usernames.

## Proxy

A proxy is required to use the bot. It was made to support [ProxyScrape](proxyscrape.com) but could probably support other proxies that support sessions.

## Configuration

Configuration is done through the `.env` file.

The following variables are available:

- `PROXY_USERNAME`: Self explanatory
- `PROXY_PASSWORD`: Self explanatory
- `PROXY_URL`: The base URL of the proxy to use
- `TARGET_NAME`: The name to change to
- `DROPTIME_MIN`: When to start changing names [UNUSED]
- `DROPTIME_MAX` When to stop changing names [UNUSED]

Unused Env vars must be set in code directly.

## Accounts

Acccounts must be in the `accs.txt` file.

The file must be in the following format:

```
email1:password1
email2:password2
```

## Building

- Clone the repository
- Install [Rust](https://www.rust-lang.org/tools/install)
- Run `cargo build --release`

## Running

- Run `cargo run --release`
