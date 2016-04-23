{ stdenv, pkgs, cargo, openssl, pkgconfig }:

let buildRustPackage = pkgs.callPackage <pkgs/build-support/rust/default.nix> {
  inherit cargo;
  rustRegistry = pkgs.callPackage ./rust-packages.nix {};
};

in buildRustPackage rec {
  version = "0.0.15";
  name = "openfairdb-${version}";
  src = ./.;

  buildInputs = with pkgs; [ openssl pkgconfig ];

  depsSha256 = "1z78f6rmy3a7wcj1halwdg0hx5p8d4y7mzsp1ijhc56iiadr6l8w";

  meta = with stdenv.lib; {
    description = "Mapping for Good";
    homepage = http://www.openfairdb.org;
    license = with licenses; [ agpl3 ];
    maintainers = [ maintainers.flosse ];
  };
}
