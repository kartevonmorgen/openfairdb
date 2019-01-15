let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustChannel = pkgs.rustChannelOf {
     date = "2018-12-08";
     channel = "nightly";
  };
in
  with pkgs;
  stdenv.mkDerivation {
    name = "rust-ofdb-dev-env";
    buildInputs = [
      rustChannel.rust
      cmake
      pkgconfig
      openssl
    ];
    SQLITE3_DIR = "${sqlite.dev}";
    SQLITE3_LIB_DIR = "${sqlite.out}/lib";
    SQLITE3_INCLUDE_LIB_DIR = "${sqlite.out}/include";
}
