{ stdenv, pkgs, cargo, openssl, pkgconfig }:

let buildRustPackage = pkgs.callPackage <pkgs/build-support/rust/default.nix> {
  inherit cargo;
  rustRegistry = pkgs.callPackage ./rust-packages.nix {};
};

in buildRustPackage rec {
  version = "0.0.16";
  name = "openfairdb-${version}";
  src = ./.;

  buildInputs = with pkgs; [ openssl pkgconfig ];

  depsSha256 = "0pxv88hqk49xfw34xv90azx9h0543rz76nxcwjvgha9hic67xqd4";

  meta = with stdenv.lib; {
    description = "Mapping for Good";
    homepage = https://openfairdb.org;
    license = with licenses; [ agpl3 ];
    maintainers = [ maintainers.flosse ];
  };
}
