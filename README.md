# Open Fair DB

The backend for [Karte von morgen](https://github.com/flosse/kartevonmorgen/)
written in [Rust](http://rustlang.org/).

[![Build Status](https://travis-ci.org/slowtec/openfairdb.svg?branch=master)](https://travis-ci.org/slowtec/openfairdb)
[![Coverage Status](https://coveralls.io/repos/github/slowtec/openfairdb/badge.svg?branch=master)](https://coveralls.io/github/slowtec/openfairdb?branch=master)
[![dependency status](https://deps.rs/repo/github/slowtec/openfairdb/status.svg)](https://deps.rs/repo/github/slowtec/openfairdb)
[![License](https://img.shields.io/badge/license-AGPLv3-blue.svg?style=flat)](https://github.com/slowtec/openfairdb/blob/master/LICENSE)

## REST API

The API is available under `http://api.ofdb.io/v0/`.

The current API is documented within the [openapi.yaml](https://github.com/slowtec/openfairdb/blob/master/openapi.yaml) file.

For examples how to use the API, open "network" in the developer tools in your browser and see the requests that https://kartevonmorgen.org sends.

When you want to use the API, please contact us at helmut@bildungsagenten.com. The API might still change sometimes. We will try to let you know in that case.

-  `GET /entries/:ID_1,:ID_2,...,:ID_n`
-  `PUT /entries/:ID`
-  `GET /categories/`
-  `GET /categories/:ID_1,:ID_2,...,:ID_n`
-  `POST /ratings`
-  `GET /ratings`
-  `GET /ratings/:ID_1,:ID_2,...,:ID_n`
-  `POST /users`
-  `POST /confirm-email-address`
-  `GET /bbox-subscriptions`
-  `POST /subscribe-to-bbox`
-  `POST /unsubscribe-all-bboxes`

### Login & Subscriptions

For the following requests one must be logged in:
`GET /users/:USERNAME`
`GET /bbox-subscriptions`
`POST /subscribe-to-bbox`
`POST /unsubscribe-all-bboxes`

`bbox-subscriptions` are subscriptions to a certain map area (bounding box,`bbox`): whenever a new entry is created or an entry is changed within that area, an email notification is sent to the user.

## Quick start

Download the latest build
[openfairdb-x86_64-linux-v0.3.7.tar.gz](https://github.com/slowtec/openfairdb/releases/download/v0.3.7/openfairdb-x86_64-linux-v0.3.7.tar.gz),
unpack and start it:

    wget https://github.com/slowtec/openfairdb/releases/download/v0.3.7/openfairdb-x86_64-linux-v0.3.7.tar.gz
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
git clone https://github.com/slowtec/openfairdb
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
[script](https://github.com/slowtec/openfairdb/blob/master/scripts/backup-sqlite.sh)
that copies the DB file once a day.

# Domain Model

*![The rendered class diagram should appear here!](http://www.plantuml.com/plantuml/svg/RLJ1Yjim4BtxAqIEWLtQQp1XswM7maAXsvx3n1uKiVQCaSRj9gN_NbLZErQK76BhlQStencDduA0bx7lgghf80JpgMqznkUVoiHVu-IyCw_Y7La5U2JnEHR48qe6NTomhF_Erf-F_5vL___Dzk5XRpQ1HpaTVcCGyt5ZdfbzwmW4rnfY7pK8XMPb-ZeUG-FT88x9r3MInBJt-wegoCrsOv9jzFePq9kT2SeVCHXXKvTxjlC6pL_3FeEWPN_EmaqKztt4CcR6eiqI_pk88nipQ9GCPcL10erCJS0UN9ULzyGz3c0n0mKx74vCM5R-MhR9iWFPcHSG9sEBYf2D29DLQDdwXIGxvMpW6gIG9-1wi7WOVNS7xHozPLGCeDRQalHOYXfheg_kWi7KfV87s2WIi0kxj6aktYtymj7JCIq7-tNRf8H4RN556eyWceXAxYUYR9b83XU9NDVpswJzyFWOvTD0tf831vUMTwVYcxT0xg8RYkR1u0x2RqZhRcHRYXFstA87mTKbrVjRkZTCWk_vzy0dxSvyZPH5dx30es-mk13tPqHZrqjixZ157ljby5AcnJXg3wzmELCQEydc7YN_gdf2QiU--mS0)*

Note: Currently the rendered class diagram must be updated manually by uploading the contents of the file [classes.puml](classes.puml) to the [PlantUML Online Editor](http://www.plantuml.com/plantuml/uml/) and replace the link for the rendered diagram with one of the generated URLs.

# License

Copyright (c) 2015 - 2018 Markus Kohlhase

This project is licensed under the AGPLv3 license.
