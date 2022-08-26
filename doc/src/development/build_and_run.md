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

If you want to get stacktraces on panics use

```sh
export RUST_BACKTRACE=1
```
