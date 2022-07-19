let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustChannel = pkgs.rustChannelOf {
     channel = "stable";
  };
  rust = (rustChannel.rust.override {
    targets = [
      "x86_64-unknown-linux-musl"
      "wasm32-unknown-unknown"
    ];
  });
in
  with pkgs;
  mkShell {
    buildInputs = [
      rust
      cmake
      pkgconfig
      openssl
      pre-commit
      nodejs
    ];
    SQLITE3_DIR = "${sqlite.dev}";
    SQLITE3_LIB_DIR = "${sqlite.out}/lib";
    SQLITE3_INCLUDE_LIB_DIR = "${sqlite.out}/include";
}
