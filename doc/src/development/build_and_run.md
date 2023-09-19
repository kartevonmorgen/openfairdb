# Build & Run

## Build

```sh
git clone https://github.com/kartevonmorgen/openfairdb
cd openfairdb/
cargo build
```

## Run

```sh
./target/debug/openfairdb
```

### Logging

```sh
RUST_LOG=debug ./target/debug/openfairdb
```

The following log levels are available:

- `error
- `warn`
- `info`
- `debug`
- `trace`

More information can be found in the
[documentation of the `env_logger` crate](https://docs.rs/env_logger/latest/env_logger/).

If you want to get stacktraces on panics use

```sh
export RUST_BACKTRACE=1
```
