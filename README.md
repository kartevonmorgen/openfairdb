# Open Fair DB

The backend for [Karte von morgen](https://github.com/flosse/kartevonmorgen/)
written in [Rust](http://rustlang.org/).

[![Build Status](https://travis-ci.org/flosse/openfairdb.svg?branch=master)](https://travis-ci.org/flosse/openfairdb)
[![Coverage Status](https://coveralls.io/repos/github/flosse/openfairdb/badge.svg?branch=master)](https://coveralls.io/github/flosse/openfairdb?branch=master)
[![dependency status](https://deps.rs/repo/github/flosse/openfairdb/status.svg)](https://deps.rs/repo/github/flosse/openfairdb)
[![License](https://img.shields.io/badge/license-AGPLv3-blue.svg?style=flat)](https://github.com/flosse/openfairdb/blob/master/LICENSE)

## REST API

The API is available under `http://api.ofdb.io/v0/`. For examples how to use the API, open "network" in the developer tools in your browser and see the requests that https://kartevonmorgen.org sends.

When you want to use the API, please contact us at helmut@bildungsagenten.com. The API might still change sometimes. We will try to let you know in that case.

-  `GET /entries/:ID_1,:ID_2,...,:ID_n`
-  `POST /entries`
-  `PUT /entries/:ID`
-  `GET /categories/`
-  `GET /categories/:ID_1,:ID_2,...,:ID_n`
-  `GET /tags`
-  `GET /search?text=TXT&bbox=LAT_min,LNG_min,LAT_max,LNG_max&categories=C_1,C_2,...,C_n`
-  `POST /ratings`
-  `GET /ratings`
-  `GET /ratings/:ID_1,:ID_2,...,:ID_n`
-  `POST /login`
-  `POST /logout`
-  `GET /users/:USERNAME`
-  `POST /users`
-  `POST /confirm-email-address`
-  `GET /bbox-subscriptions`
-  `POST /subscribe-to-bbox`
-  `POST /unsubscribe-all-bboxes`
-  `GET /export/entries.csv?bbox=LAT_min,LNG_min,LAT_max,LNG_max`
-  `GET /count/entries`
-  `GET /count/tags`
-  `GET /server/version`

### Search
**Example:**
Search for "lebensmittel" with tags #unverpackt, #zerowaste: http://api.ofdb.io/v0/search?text=lebensmittel%20%23unverpackt%20%23zerowaste&bbox=47.29541440362851,2.3431777954101567,53.97012306226697,17.80094146728516

`categories` is an optional filter. We currently use the following two:
**Initiative (non-commercial):** 2cd00bebec0c48ba9db761da48678134
**Company:** 77b3c33a92554bcf8e8c2c86cedd6f6f

Search returns an object with the following structure:

```
{"visible":[
    {"id": ID1,"lat": LAT,"lng": LNG},
    {"id": ID1,"lat": LAT,"lng": LNG}
],
"invisible":[
    {"id": ID1,"lat": LAT,"lng": LNG},
    {"id": ID1,"lat": LAT,"lng": LNG}
]
```

Under `visible` are the entries that are in the given bounding box (`bbox`, area of the map). Under `invisible` are up to 5 entries outside the `bbox`.

### Login & Subscriptions

For the following requests one must be logged in:
`GET /users/:USERNAME`
`GET /bbox-subscriptions`
`POST /subscribe-to-bbox`
`POST /unsubscribe-all-bboxes`

`bbox-subscriptions` are subscriptions to a certain map area (bounding box,`bbox`): whenever a new entry is created or an entry is changed within that area, an email notification is sent to the user.

### Entry Export
**Example**: Export all entries in Germany:
http://api.ofdb.io/v0/export/entries.csv?bbox=47.497972542230855,0.7996758709088782,54.63407558981465,18.307256321725717

If you want to find out the coordinates for other map areas, open "network" in the "developer tools" in your browser and look at the search request under at the value of `bbox`.

### JSON structures

#### Entry
```
{
  "id"          : String,
  "version"     : Number,
  "created"     : Number,
  "title"       : String,
  "description" : String,
  "lat"         : Number,
  "lng"         : Number,
  "street"      : String,
  "zip"         : String,
  "city"        : String,
  "country"     : String,
  "email"       : String,
  "telephone"   : String,
  "homepage"    : String,
  "categories"  : [String],
  "tags"        : [String],
  "ratings"     : [String]
  "license"     : String
}
```

#### Rating
```
{
    "id"        : String,
    "title"     : String,
    "created"   : Number,
    "value"     : Number,
    "context"   : String,
    "source"    : String,
    "comments"  : [RatingComment]
}
```

#### RatingComment
```
{
    "id"        : String,
    "created"   : Number,
    "text"      : String
}
```

#### Category

```
{
  "id"      : String,
  "version" : Number,
  "created" : Number,
  "name"    : String,
  "parent"  : String
}
```

#### Rating

```
{
  "id"          : String,
  "created"     : Number,
  "title"       : String,
  "user"        : String,
  "value"       : Number,
  "context"     : String,
  "comments"    : Array,
}
```

#### User
```
{
    "username"  : String,
    "email"     : String
}
```

#### BboxSubscription
```
{
    "id"             : String,
    "south_west_lat" : Number,
    "south_west_lng" : Number,
    "north_east_lat" : Number,
    "north_east_lat" : Number
}
```

## Quick start

Download the latest build
[openfairdb-x86_64-linux-v0.3.7.tar.gz](https://github.com/flosse/openfairdb/releases/download/v0.3.7/openfairdb-x86_64-linux-v0.3.7.tar.gz),
unpack and start it:

    wget https://github.com/flosse/openfairdb/releases/download/v0.3.7/openfairdb-x86_64-linux-v0.3.7.tar.gz
    tar xzf openfairdb-x86_64-linux-v0.3.7.tar.gz
    ./openfairdb

## Build

Requirements:

- [Rust](https://www.rust-lang.org/) (nightly)
- [SQLite](https://sqlite.org/) 3.x

### Installing Rust & Cargo

If you're using Ubuntu 16.04 LTS you can run

```
sudo apt-get install curl libssl-dev gcc
curl https://sh.rustup.rs -sSf | sh
rustup install nightly
rustup default nightly
```

On windows you can download the installer from [rustup.rs](https://rustup.rs).
(But don't forget to install a
[C++ toolchain](http://landinghub.visualstudio.com/visual-cpp-build-tools) first).

Installing a specific nightly version with `rustup` (e.g. `2018-01-04`) is easy:

```
rustup default nightly-2018-01-04
```

### Installing SQLite & Diesel

On Ubuntu:

```
sudo apt-get install sqlite3 libsqlite3-dev
cargo install diesel_cli --no-default-features --features sqlite
```

### Compile & Run

```
git clone https://github.com/flosse/openfairdb
cd openfairdb/
diesel migration run
cargo build
./target/debug/openfairdb
```

On NixOS you can build the project with:

```
nix-build -E '(import <nixpkgs>{}).callPackage ./default.nix {}'
```

### Docker Build

The bundled Makefile can be used to build a static executable without installing any dependencies locally.

Depending on the permissions of your local Docker installation you may need to use `sudo` for the invocation of `make`.

#### Pull Build Image

The build requires the [muslrust](https://hub.docker.com/r/clux/muslrust/tags/) Docker image with the corresponding Rust toolchain:

```sh
make -f Makefile.x86_64-unknown-linux-musl pre-build
```

This command has to be executed at least once and can be repeated for updating the `muslrust` build image.

#### Execute Build

Use the following command from the project directory to start the build:

```sh
make -f Makefile.x86_64-unknown-linux-musl
```

The source folder is copied into `/tmp` and mounted as a volume into the Docker container. The Cargo cache is also stored in `/tmp` and reused on subsequent invocations.

The resulting executable is copied into the folder `bin/x86_64-unknown-linux-musl` after the build has completed successfully.

## Logging

    RUST_LOG=debug ./target/debug/openfairdb

If you want to get stacktraces on panics use

    export RUST_BACKTRACE=1

## DB Backups

At the moment the OpenFairDB does not support online backups.
Therefore we use a simple
[script](https://github.com/flosse/openfairdb/blob/master/scripts/backup-sqlite.sh)
that copies the DB file once a day.

# License

Copyright (c) 2015 - 2018 Markus Kohlhase

This project is licensed under the AGPLv3 license.
