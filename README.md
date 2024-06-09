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

## ToDo

- Command line app
- Persistence
- Refresh token handling
- Graceful OAuth2 server shutdown
- Implement Beancount exporter.
  - See: https://beancount.github.io/
  - See: https://github.com/beancount/fava
- Error handling. See: https://docs.monzo.com/?shell#errors
- Incremental transaction update since last update.
