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
  sass = with pkgs; stdenv.mkDerivation rec {
    name = "dart-sass-${version}";
    version = "1.62.1";
    system = "x86_64-linux";

    isExecutable = true;

    src = fetchurl {
      sha256 = "AbBHLUrVMk2t0ma+hlinCA7C71a5TrloODbM1SZmuG0=";
      url = "https://github.com/sass/dart-sass/releases/download/${version}/dart-sass-${version}-linux-x64.tar.gz";
    };

    phases = "unpackPhase installPhase fixupPhase";

    fixupPhase = ''
      patchelf \
        --set-interpreter ${binutils.dynamicLinker} \
        $out/src/dart
    '';

    installPhase = ''
      mkdir -p $out/bin
      cp -r . $out
      ln -s $out/sass $out/bin/sass
    '';
  };

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
      sass
      nodejs # TODO: do we still need this?
      nodePackages.tailwindcss
    ];
    SQLITE3_DIR = "${sqlite.dev}";
    SQLITE3_LIB_DIR = "${sqlite.out}/lib";
    SQLITE3_INCLUDE_LIB_DIR = "${sqlite.out}/include";
}
