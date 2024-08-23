<div align="center">
    <img src="https://raw.githubusercontent.com/peterprototypes/dontpanic-server/253282285864ef092281bc63be70f79bdb10670b/static/dontpanic-ferris-logo.svg" width="20%" />
</div>

<h1 align="center">dontpanic-server</h1>

<p align="center">
    Backend web server for the [dontpanic](https://crates.io/crates/dontpanic) crate. Receives and displays panic reports and log messages. Optionally sends notifications via multiple configurable channels.
</p>

## Features

- Friendly UI
- Authentication & Authorization
- View reported panics & errors
- View panic backtrace and log messages
- Mark a panic as resolved
- Setup and manage notifications
- Organizations and project management
- User management

## Running via docker compose

### SQLite

```yml
services:
  dontpanic:
    image: ptodorov/dontpanic-server:latest
    ports:
      - 8080:8080
    environment:
      DATABASE_URL: sqlite:///data/dontpanic.sqlite?mode=rwc
      DEFAULT_USER_EMAIL: admin@example.com
      DEFAULT_USER_PASSWORD: admin123
    volumes:
      - database-data:/data

volumes:
  database-data:
```

### MariaDB/MySQL

```yml
services:

  dontpanic:
    image: ptodorov/dontpanic-server:latest
    ports:
      - 8080:8080
    links:
      - database
    environment:
      DATABASE_URL: mysql://dontpanic:32as1e78gdfbqwe5tx@database/dontpanic
      DEFAULT_USER_EMAIL: admin@example.com
      DEFAULT_USER_PASSWORD: admin123

  database:
    image: mariadb
    environment:
      MARIADB_DATABASE: dontpanic
      MARIADB_USER: dontpanic
      MARIADB_PASSWORD: 32as1e78gdfbqwe5tx
      MARIADB_RANDOM_ROOT_PASSWORD: 1
    volumes:
      - database-data:/var/lib/mysql

volumes:
  database-data:
```

## Environment Variables

| Variable                      | Description                                                                                                                           | Default
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|------------------
| `BIND_ADDRESS`                | The address:port that can access dontpanic web interface. Use 0.0.0.0:8080 to allow anybody to connect.                               | `0.0.0.0:8080`
| `BASE_URL`                    | Url to use when generating links in notifications and emails.                                                                         | `localhost`
| `SCHEME`                      | Http scheme to use when generating links in notifications and emails.  Possible values: `http`, `https`                               | `http`
| `COOKIE_SECRET`               | The secret key used to encrypt session cookies. Set this to a random string. If this is not set, a random secret will be generated on each startup. Every time this key is changed, all sessions will be dropped and all users will need to log in again. | Random
| `DATABASE_URL`                | Mysql: `mysql://username:password@host/database` Sqlite: `sqlite://[PATH_TO_FILE].sqlite?mode=rwc`                                    | None
| `EMAIL_URL`                   | Optional SMTP connection url for email notifications. Format: `smtps://username:password@smtp.example.com/client.example.com:465`, [Documentation](https://docs.rs/lettre/latest/lettre/transport/smtp/struct.AsyncSmtpTransport.html#method.from_url) | None
| `EMAIL_FROM`                  | Source email address for email notifications.                                                                                         | no-rely@dontpanic.rs
| `DEFAULT_USER_EMAIL`          | This account will be created at startup if it does not exist. Use this to login into your self-hosted deployment.                     | None
| `DEFAULT_USER_PASSWORD`       | Password for the default user. Min 8 characters long. Both email and password must be provided or no account will be created.         | None
| `DEFAULT_USER_TIMEZONE`       | IANA Timezone name for the default user and for all new registrations.                                                                | `UTC`
| `DEFAULT_USER_ORGANIZATION`   | Organization name to use when creating the default user.                                                                              | `Default Organization`
| `REGISTRATION_ENABLED`        | Enable/disable account creation. `yes`, `1`, `true` all count as true, anything else is false. Care must be taken when setting this to yes. Anyone with access to the server can register. | `true`
| `REQUIRE_EMAIL_VERIFICATION`  | Require new users to verify their email address before they can login. A working `EMAIL_URL` configuration is required.               | `true`
| `SLACK_CLIENT_ID`             | Slack app client id. Required for Slack notifications to work. See [this](https://api.slack.com/quickstart)                           | None
| `SLACK_CLIENT_SECRET`         | Slack app client secret. Keep this secure.                                                                                            | None

## Development

One liner to start hacking. This creates a sqlite database in the current dir and a user:

```bash
DATABASE_URL="sqlite://localdev.sqlite?mode=rwc" DEFAULT_USER_EMAIL=dev@example.com DEFAULT_USER_PASSWORD=$DEFAULT_USER_EMAIL cargo run
```

http://localhost:8080  
User: `dev@example.com`  
Pass: `dev@example.com`  

### Database changes

Don't Panic uses the excellent [SeaORM](https://www.sea-ql.org/SeaORM/) project. Make sure you have the cli installed:  
`cargo install sea-orm-cli`

Set your database url environment var to avoid typing it before every command:  
`DATABASE_URL="sqlite://localdev.sqlite?mode=rwc"`

To create a database migration run:  
`sea-orm-cli migrate generate NAME_OF_MIGRATION`

After populating the migration file, execute it:  
`sea-orm-cli migrate up`

And finally regenerate the entity files:  
`sea-orm-cli generate entity -o src/entity --with-serde serialize`

## License

Fair Core License, Version 1.0, MIT Future License [LICENSE.md](LICENSE.md)
