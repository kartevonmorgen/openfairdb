{ stdenv, fetchurl }:

stdenv.mkDerivation rec {
  version = "0.5.0";
  name = "openfairdb-${version}";
  src = fetchurl {
    url = "https://github.com/slowtec/openfairdb/releases/download/v${version}/openfairdb_x86_64-unknown-linux-musl_v${version}.tar.xz";
    sha256 = "1318j40ncfvcc8l34940m63fwh8nrrs3r498riwmqc4ihcrk1rs5";
  };

  sourceRoot = ".";

  installPhase = ''
    mkdir -p $out/bin
    install -D openfairdb $out/bin
  '';

  meta = with stdenv.lib; {
    description = "Mapping for Good";
    homepage = https://openfairdb.org;
    license = with licenses; [ agpl3 ];
    maintainers = [ maintainers.flosse ];
  };
}
