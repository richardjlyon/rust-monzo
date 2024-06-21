# monzo-rust

A rust command line application for downloading Monzo transaction history to
an SQLITE database.

Crates: [monzo-cli](https://crates.io/crates/monzo-cli)

## Installation

As a command line app:

```bash
cargo install --git https://github.com/richardjlyon/rust-monzo
```

As a library:

```bash
cargo add monzo-cli
```

## Usage

```rust
> monzo-cli

Usage: monzo-cli <COMMAND>

Commands:
  update    Update transactions
  balances  Account balances
  auth      (Re)authorise the application
  reset     Reset the database (WARNING: This will delete all data!)
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)

## Configuration

### Authorisation

Edit the file `configuration.toml` with the following content:

```yaml
start_date = "2024-01-01T00:00:00"
default_days_to_update = 14

[database]
database_path = "db.sqlite"
max_connections = 5

[oath_credentials]
client_id = "XXX"
client_secret = "XXX"
redirect_uri = "http://localhost:3000/oauth/callback"
```

Create a new OAuth client in the Monzo developer console and replace the
`client_id` and `client_secret` with the values from the new client. Replace`start_date` with the date of the earliest transaction you want to download.

### Custom categories

Create file `configuration.yaml` in the root of the project with the following content:

```yaml
category_0000Aebc1dJeps1a2lDFKb: "InternetMobile"
category_0000AeNDWV8K5iX53Ohyld: "Car"
category_0000AiofUPXe8c5I6Zxp6g: "Clothing"
category_0000AeR7gxWtXy4Hzy0ULL: "Energy"
category_0000AgThBIKMlNhgpZPRhZ: "Maintenance"
category_0000AeU961sBDb6GUYcQh1: "Subscriptions"
```

Replace the category IDs with the IDs of the categories you want to use. These
can be found in the database.

## Notes

1. For security reasons, the Monzo API limits the period in which all transactions can to downloaded to a 5 minute window following authentication. This means that the first time you run the application, you will need to run the `auth` command and follow the instructions to authenticate the application. This will only need to be done once.
2. **The Monzo API is severely limited by the lack of any API access to transactions within "Pots". There is nothing I can do about this.**
