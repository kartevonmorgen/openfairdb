# Open Fair DB

A micro backend for [Karte von morgen](https://github.com/flosse/kartevonmorgen/)
written in [Rust](http://rustlang.org/).

[![Build Status](https://travis-ci.org/flosse/openfairdb.svg?branch=master)](https://travis-ci.org/flosse/openfairdb)
[![Coverage Status](https://coveralls.io/repos/github/flosse/openfairdb/badge.svg?branch=master)](https://coveralls.io/github/flosse/openfairdb?branch=master)
[![License](https://img.shields.io/badge/license-AGPLv3-blue.svg?style=flat)](https://github.com/flosse/openfairdb/blob/master/LICENSE)

## Build

Requirements:

- [Rust](http://rustlang.org/) >= 1.6
- [Neo4j](http://neo4j.com/) >= 2.1.8

### Installing Rust & Cargo

If you're using Ubuntu 16.04 LTS you can run

```
sudo apt-get install rustc cargo libssl-dev
```

### Installing Neo4j

This readme describes the process with Neo4j Version 2.1.8.

According to [debian.neo4j.org](http://debian.neo4j.org/):

    wget -O - https://debian.neo4j.org/neotechnology.gpg.key | sudo apt-key add -
    echo 'deb http://debian.neo4j.org/repo stable/' >/tmp/neo4j.list
    sudo mv /tmp/neo4j.list /etc/apt/sources.list.d
    sudo apt-get update
    sudo apt-get install neo4j=2.1.8
    
or follow these instructions for different operating systems: http://neo4j.com/docs/stable/server-installation.html 

Add the neo4j console command: 
    sudo ln -s /var/lib/neo4j/bin/neo4j /usr/bin/neo4j

To disable the authentication add the following line to
`/etc/neo4j/neo4j.properties` (or `~/Documents/Neo4j/default.graphdb/neo4j.properties` on Mac OS X):

    dbms.security.auth_enabled=false


After installation, Neo4j should be running. You can check this with the following command

    service neo4j-service status

To start Neo4j, run:
    neo4j start
or
    neo4j start-no-wait
To stop it:
    neo4j stop

### Compile & Run

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
  "categories"  : [String],
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
