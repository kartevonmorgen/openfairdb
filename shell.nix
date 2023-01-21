let
  fenix_overlay = import "${fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz"}/overlay.nix";
  pkgs = import <nixpkgs> { overlays = [ fenix_overlay ]; };
  fenix = pkgs.fenix;
  rust_stable = fenix.stable;
  rust_targets = with fenix.targets; [
    wasm32-unknown-unknown.stable.rust-std
    x86_64-unknown-linux-musl.stable.rust-std
  ];
  rust = with rust_stable; (fenix.combine [
    rustc
    cargo
    clippy
    rustfmt
    rust_targets
  ]);
in
  with pkgs;
  mkShell {
    buildInputs = [
      rust
      cmake
      pkgconfig
      openssl # TODO: do we still need this?
      pre-commit
      graphviz
      plantuml
      sassc # dart-sass is only available as flake
      nodejs # TODO: do we still need this?
      nodePackages.tailwindcss
    ];
    SQLITE3_DIR = "${sqlite.dev}";
    SQLITE3_LIB_DIR = "${sqlite.out}/lib";
    SQLITE3_INCLUDE_LIB_DIR = "${sqlite.out}/include";
}
