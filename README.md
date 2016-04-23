# Open Fair DB

A micro backend for [Karte von morgen](https://github.com/flosse/kartevonmorgen/)
written in [Rust](http://rustlang.org/).

[![Build Status](https://travis-ci.org/flosse/openfairdb.svg?branch=master)](https://travis-ci.org/flosse/openfairdb)

## Build

Requirements:

- [Rust](http://rustlang.org/) 1.5
- [Neo4j](http://neo4j.com/) 2.1.8

Start Neo4j:
- on Mac OS X: open "Neo4j Community Edition.app" in /Applications, click on "Start"
- (other platforms?)

```
git clone https://github.com/flosse/openfairdb
cd openfairdb/
cargo build
./target/debug/openfairdb --enable-cors
```

On NixOS you can build the project with:

```
nix-build -E '(import <nixpkgs>{}).callPackage ./default.nix {}'
```

## REST API

The current REST API is quite basic and will change within the near future.
The base URL is `http://api.ofdb.io/v0/`.

-  `GET /entries/:ID`
-  `GET /entries/:ID_1,:ID_2,...,:ID_n`
-  `POST /entries/`
-  `PUT /entries/:ID`
-  `GET /categories/`
-  `GET /categories/:ID`
-  `GET /search?text=TXT&bbox=LAT_min,LNG_min,LAT_max,LNG_max&categories=C_1,C_2,...,C_n`

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
  "categories"  : [String]
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

## Logging

    RUST_LOG=debug ./target/debug/openfairdb

If you want to get stacktraces on panics use

    export RUST_BACKTRACE=1

## DB Backups

The community edition of Neo4j
[does not support online backups](https://github.com/flosse/openfairdb/issues/10)
therefore we use a simple
[script](https://github.com/flosse/openfairdb/blob/master/scripts/backup.sh)
that copies the DB to `/var/lib/neo4j/backup/` once a day.

### Restore a backup

    systemctl stop neo
    tar -czf /var/lib/neo4j/backup/snapshot.tar.gz /var/lib/neo4j/data/graph.db
    rm -rf /var/lib/neo4j/data/graph.db
    tar --strip-components=4 -C /var/lib/neo4j/data -xvzf old-backup.tar.gz "var/lib/neo4j/data/"
    systemctl start neo

# License

Copyright (c) 2015 - 2016 Markus Kohlhase

This project is licensed unter the AGPLv3 license.
