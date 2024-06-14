# Monzo

A rust client for the Monzo API.

## Configuration

Create file `configuration.yaml` in the root of the project with the following content, replacing `xx` with your Monzo client id and secret:

```yaml
oath_credentials:
  client_id: xx
  client_secret: xx
  redirect_uri: http://localhost:3000/oauth/callback
access_tokens:
  access_token: None
  client_id: None
  expires_in: 0
  refresh_token: None
  token_type: None
  user_id: None
```

## Database

```bash
sqlx database create
sqlx migrate add <name>
sqlx migrate run
cargo sqlx prepare
```

## ToDo

- Incremental transaction update since last update.
- Implement Beancount exporter.
  - See: https://beancount.github.io/
  - See: https://github.com/beancount/fava
- Refresh token handling
- Error handling. See: https://docs.monzo.com/?shell#errors
- implement Axiom telemetry: https://axiom.co/docs

## Beancount cheat sheet

Types of Accounts:

- Balance Sheet - A balance that is meaningful at a point in time.
  - Assets (+ve) Something owned
  - Liabilities (-ve) Something owed
- Income Statement - A balance that is meaningful over a period of time.
  - Income (-ve) Something received
  - Expenses (+ve) Something given away
