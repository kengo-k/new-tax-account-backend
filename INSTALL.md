# setup dependencies

## install dotenvy

```
$ cargo add dotenvy
```

## install Diesel

```
$ cargo add diesel --features  "sqlite"
```

## install Diesel CLI

```
$ cargo install diesel_cli --no-default-features --features sqlite
```

# create database setting

Create a `.env` file and set the `DATABASE_URL`. For SQLite, it would look like this:

```.env
DATABASE_URL=database.db
```

Place `database.db` in the current directory as a relative path.

# initialize database

```
$ diesel setup
```
