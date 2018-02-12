# Open Fair DB

A micro backend for [Karte von morgen](https://github.com/flosse/kartevonmorgen/)
written in [Rust](http://rustlang.org/).

[![Build Status](https://travis-ci.org/flosse/openfairdb.svg?branch=master)](https://travis-ci.org/flosse/openfairdb)
[![Coverage Status](https://coveralls.io/repos/github/flosse/openfairdb/badge.svg?branch=master)](https://coveralls.io/github/flosse/openfairdb?branch=master)
[![dependency status](https://deps.rs/repo/github/flosse/openfairdb/status.svg)](https://deps.rs/repo/github/flosse/openfairdb)
[![License](https://img.shields.io/badge/license-AGPLv3-blue.svg?style=flat)](https://github.com/flosse/openfairdb/blob/master/LICENSE)

## Quick start

Download the latest build
[openfairdb-x86_64-linux-v0.3.2.tar.gz](https://github.com/flosse/openfairdb/releases/download/v0.3.2/openfairdb-x86_64-linux-v0.3.2.tar.gz),
unpack and start it:

    wget https://github.com/flosse/openfairdb/releases/download/v0.3.2/openfairdb-x86_64-linux-v0.3.2.tar.gz
    tar xzf openfairdb-x86_64-linux-v0.3.2.tar.gz
    ./openfairdb

The API is now available at `http://127.0.0.1:676`.

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

## REST API

The current REST API is quite basic and will change within the near future.
The base URL is `http://api.ofdb.io/v0/`.

-  `GET /entries/:ID_1,:ID_2,...,:ID_n`
-  `POST /entries`
-  `PUT /entries/:ID`
-  `GET /categories/`
-  `GET /categories/:ID`
-  `GET /search?text=TXT&bbox=LAT_min,LNG_min,LAT_max,LNG_max&categories=C_1,C_2,...,C_n`
-  `GET /count/entries`
-  `GET /count/tags`
-  `GET /server/version`
-  `POST /users`
-  `POST /ratings`
-  `GET /ratings`

#### JSON structures

The structure of an `entry` looks like follows:

```
{
  "id"          : String,
  "version"     : Number,
  "created"     : Number,
  "name"        : String,
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
  "license"     : String
}
```

The structure of a `category` looks like follows:

```
{
  "id"      : String,
  "version" : Number,
  "created" : Number,
  "name"    : String,
  "parent"  : String
}
```

The structure of an `rating` looks like follows:

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
