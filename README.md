# Brouclean Fantasy Games

This project is for friends to play fantasy games

## Prerequisites

Make sure you set up your Google OAuth, which you can find a link to set up [here.](https://console.cloud.google.com/apis/dashboard)

# Prepare your dev environment (example for Fedora)

## Install Rust and tools

Install rust : [https://www.rust-lang.org/tools/install]

Install rust formatter

```
rustup component add rustfmt
```

Install cargo make

```
cargo install cargo-make
```

## Install Postgresql locally

Download and install : [https://www.postgresql.org/download/]
For example on fedora, first clean the pgsql directory in case of former install

```
sudo rm -r /var/lib/pgsql
```

and then install and start the server

```
sudo dnf install postgresql-server
sudo postgresql-setup --initdb
sudo systemctl enable postgresql.service
sudo systemctl start postgresql.service
```

Create an app user

```
sudo -u postgres psql
CREATE ROLE devapp LOGIN PASSWORD '<db user password>';
\du
\q
```

If you want to connect from your own user you will have to change the local config to "trust" in the pg_hba.conf

```
sudo gnome-text-editor /var/lib/pgsql/data/pg_hba.conf
```

Create the database

```
sudo -u postgres psql
CREATE DATABASE brouclean_fantasy_games;
ALTER DATABASE brouclean_fantasy_games OWNER TO devapp;
\l
\q
```

To dump the data from production to local

```
pg_dump -Fc -h <host> -U <user> <db> > dump.dump
pg_restore --clean --if-exists --no-owner --no-acl -U devapp -d brouclean_fantasy_games dump.dump
```

## VSCode extensions

- rust-analyzer
- Even Better TOML
- Tailwind CSS IntelliSense

## Env variables

For the app to run locally you need to create a .env file containing

```
# General
APP_URL=<URL of the app, ex: http://localhost:8000>
ADMIN_EMAIL=<Your admin email>
LOG_LEVEL=INFO

# Database
DATABASE_URL=postgres://devapp:<db user password>@localhost:5432/brouclean_fantasy_games

# Google oauth
GOOGLE_OAUTH_CLIENT_ID=<Your client id>
GOOGLE_OAUTH_CLIENT_SECRET=<Your client secret>
```

## To build and run the app

To run the app locally (localhost)

```
cargo make run
```

To build the app

```
cargo build --release
```

To start the app

```
./target/release/brouclean-fantasy-games
```

## To deploy on Leapcell

Command to make it work with ssl

```
apt-get update && apt-get install -y pkg-config libssl-dev && cargo build --release
```

