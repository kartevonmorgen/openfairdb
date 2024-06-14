let
  fenix_overlay = import "${fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz"}/overlay.nix";
  pkgs = import <nixpkgs> { overlays = [ fenix_overlay ]; };
  fenix = pkgs.fenix;
  rust = fenix.fromToolchainFile {
    file = ./rust-toolchain.toml;
  };
  sass = with pkgs; stdenv.mkDerivation rec {
    name = "dart-sass-${version}";
    version = "1.77.5";
    system = "x86_64-linux";
    isExecutable = true;
    src = fetchurl {
      sha256 = "sha256-d0PT7p2d5/RsA3ZTe2v0MqLkDIT/qVNI4pfeviXwfns=";
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
      just
      pkg-config
      openssl # TODO: do we still need this?
      pre-commit
      graphviz
      plantuml
      sass
      nodejs # TODO: do we still need this?
      nodePackages.tailwindcss
      trunk
    ];
    SQLITE3_DIR = "${sqlite.dev}";
    SQLITE3_LIB_DIR = "${sqlite.out}/lib";
    SQLITE3_INCLUDE_LIB_DIR = "${sqlite.out}/include";
}
