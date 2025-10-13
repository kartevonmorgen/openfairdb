{
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    wasm-bindgen-flake.url = "github:fusion44/wasm-bindgen-cli-flake";
  };
  outputs =
  { self, nixpkgs, flake-utils, rust-overlay, wasm-bindgen-flake }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          platform_packages =
            if pkgs.stdenv.isLinux then
              with pkgs; [ ]
            else if pkgs.stdenv.isDarwin then
              with pkgs.darwin.apple_sdk.frameworks; [
                CoreFoundation
                Security
                SystemConfiguration
              ]
            else
              throw "unsupported platform";

          # Currently wasm-bindgen-cli is not up to date in nixpkgs
          # so we use this external flake:
          wasm-bindgen-cli = wasm-bindgen-flake.packages.${system}.wasm-bindgen-cli;

          # TODO: Move into separate nix file
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
        rec {

          # Currently trunk is up to date (v0.21.14) in nixpkgs
          # so we don't need to build it locally,
          # but in case it's outdated, you can update and use it here:
          #
          # trunk = pkgs.callPackage ./trunk.nix {
          #   inherit (darwin.apple_sdk.frameworks) CoreServices Security SystemConfiguration;
          # };

          # These are not actual dependencies
          # but tools that can help with development
          dev_env_packages =  [
              git      # version control
              tig      # TUI to browse the GIT repo
              helix    # a modern vim alternative
          ];

          devShells.default = mkShell {
            buildInputs = [
              rust
              cargo-edit
              cargo-zigbuild   # required for static musl builds
              trunk            # required to bundle the frontend
              binaryen         # required to minify WASM files with wasm-opt
              wasm-bindgen-cli # required to generate JS files to bootstrap WASM in the browser
              wasm-pack        # required to run wasm-bindgen-tests
              just             # task runner
              tailwindcss      # build CSS files
              nodejs           # required to install tailwind plugins
              graphviz
              plantuml
              mdbook
              mdbook-plantuml
              pre-commit
              sass
            ] ++ dev_env_packages ++ platform_packages;

            # This is required if the backend is to be built in a isolated environment
            # (e.g. created by `nix develop -i`).
            shellHook = ''
              cache_dir=$(mktemp -d)
              export ZIG_LOCAL_CACHE_DIR="$cache_dir"
              export ZIG_GLOBAL_CACHE_DIR="$cache_dir"
            '';
          };
        }
      );
}
