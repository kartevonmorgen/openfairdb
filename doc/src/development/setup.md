# Setup

## Prerequisites

- Installation of Rust with the `stable` toolchain
- [SQLite](https://sqlite.org/) 3.x

## Development tooling

The installation of various development tools is automated by a [`just`](https://github.com/casey/just) recipe named `setup`:

```sh
cargo install just
just setup
```

Check the configuration in `.justfile` if any of the recipe's steps should fail.
Running the recipe repeatedly is also possible.

The setup includes the installation of a Git pre-commit hook in `.git/hooks/`.

## Platforms and environments

### NixOS

On [NixOS](https://nixos.org/) you can run `nix-shell` within the root
of the repository to pull all required dependencies.

### macOS

If setting up MacOS in order to build, be sure to install a C compiler
via `$ xcode-select --install`. Otherwise `cargo install` will not
behave as expected.

### Ubuntu

Install required packages:

```sh
sudo apt-get install curl libssl-dev gcc sqlite3 libsqlite3-dev
```

## Periodic tasks

Both the development toolchain, tools and the Rust dependencies should be upgraded
periodically by running the following `just` recipe:

```sh
just upgrade
```

Review the changes in the configuration files and commit them selectively as desired.
