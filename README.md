<div align="center">
  <img src="https://raw.githubusercontent.com/peterprototypes/dontpanic-server/253282285864ef092281bc63be70f79bdb10670b/static/dontpanic-ferris-logo.svg" width="20%" />
  <h1>dontpanic-server</h1>
  <p>
    Backend web server for the <a href="https://crates.io/crates/dontpanic">dontpanic</a> crate. Receives and displays panic reports and log messages. Optionally sends notifications via multiple configurable channels.
  </p>
</div>

## Features

- Friendly UI
- Authentication & Authorization
- View reported panics & errors
- View panic backtrace and log messages
- Mark a panic as resolved
- Setup and manage notifications
- Organizations and project management
- User management

## Screenshots

List of panic!()s and error!()s                | Individual report
:---------------------------------------------:|:--------------------------------------------------:
![](/static/img/screenshot_list.png?raw=true)  |  ![](/static/img/screenshot_report.png?raw=true)

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
    restart: always
    container_name: dontpanic
    ports:
      - 8080:8080
    environment:
      DATABASE_URL: mysql://dontpanic:32as1e78gdfbqwe5tx@database/dontpanic
      DEFAULT_USER_EMAIL: admin@example.com
      DEFAULT_USER_PASSWORD: admin123
    depends_on:
      database:
        condition: service_healthy

  database:
    image: mariadb
    restart: always
    container_name: database
    command:
      [
        "mariadbd",
        "--character-set-server=utf8mb4",
        "--collation-server=utf8mb4_unicode_ci",
      ]
    ports:
      - 3306:3306
    environment:
      MARIADB_DATABASE: dontpanic
      MARIADB_USER: dontpanic
      MARIADB_PASSWORD: 32as1e78gdfbqwe5tx
      MARIADB_RANDOM_ROOT_PASSWORD: 1
    healthcheck:
      test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
      start_period: 10s
      interval: 10s
      timeout: 5s
      retries: 3
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
| `RUST_LOG`                    | Logging level. Valid values are `trace`, `debug`, `info`, `warn`, `error`                                                             | `info`
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
| `ORGANIZATION_REQUESTS_LIMIT` | Limits the number of reports events each new organization can submit each month across all of its projects. Not set - unlimited       | None
| `SLACK_CLIENT_ID`             | Slack app client id. Required for Slack notifications to work. See [this](https://api.slack.com/quickstart)                           | None
| `SLACK_CLIENT_SECRET`         | Slack app client secret. Keep this secure.                                                                                            | None

## Development

In one console start the frontend:
```bash
cd frontend
npm i
npm run dev
```

In another start the project:
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
`DATABASE_URL="mysql://user:pass@127.0.0.1:3306/database"`

To create a database migration run:
`sea-orm-cli migrate generate NAME_OF_MIGRATION`

After populating the migration file, execute it:
`sea-orm-cli migrate up`

And finally regenerate the entity files:
`sea-orm-cli generate entity -o src/entity --with-serde serialize`

**Note that the entity ids generate as i32 when using sqlite. The entity folder in this project was generated from a mysql database which supports unsigned integers and all ids are u32s**

### Contributing

All commit messages must follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

## License

GNU Affero General Public License v3.0 only [LICENSE.md](LICENSE.md)
