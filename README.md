# Open Fair DB

The backend for [Karte von morgen](https://github.com/kartevonmorgen/kartevonmorgen/)
written in [Rust](http://rustlang.org/).

[![GitHub CI](https://github.com/kartevonmorgen/openfairdb/actions/workflows/continuous-integration.yaml/badge.svg?branch=main)](https://github.com/kartevonmorgen/openfairdb/actions/workflows/continuous-integration.yaml)
[![License](https://img.shields.io/badge/license-AGPLv3-blue.svg?style=flat)](https://github.com/kartevonmorgen/openfairdb/blob/main/LICENSE)

## Quick start

Download the latest build
[openfairdb_x86_64-unknown-linux-musl_v0.10.5.tar.xz](https://github.com/kartevonmorgen/openfairdb/releases/download/v0.10.5/openfairdb_x86_64-unknown-linux-musl_v0.10.5.tar.xz),
unpack and start it:

```sh
wget https://github.com/kartevonmorgen/openfairdb/releases/download/v0.10.5/openfairdb_x86_64-unknown-linux-musl_v0.10.5.tar.xz
tar xJf openfairdb_x86_64-unknown-linux-musl_v0.10.5.tar.xz
RUST_LOG=info ./openfairdb
```

## Documentation

The project documentation can be found in the `docs` folder
or at <https://book.ofdb.io>.

## License

Copyright (c) 2018 - 2022 [slowtec GmbH](https://slowtec.de)\
Copyright (c) 2015 - 2018 Markus Kohlhase

This project is licensed under the AGPLv3 license.
