{
  lib,
  stdenv,
  fetchurl,
  gawk,
  libarchive,
  wabt,
}:

let
  headers = {
    arinc653 = fetchurl {
      url = "https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653.h.zip";
      hash = "sha256-4sr+QMkK2tDLFG9O0u9PAKWA7iIL+//K7S3eMOJEtPY=";
    };
    arinc653p2 = fetchurl {
      url = "https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653p2.h.zip";
      hash = "sha256-a6/ma3kHkUgHaxL/nlcffA3WaQsPWe+pZad6z0g6kfo=";
    };
  };

in
stdenv.mkDerivation {
  name = "wasm-partitions";

  src = ../.;

  nativeBuildInputs = [
    gawk
    libarchive
    wabt
  ];

  postPatch = ''
    # install downloaded files
    mkdir --parent -- target/downloads
    for ZIP_FILE in ${
      lib.strings.escapeShellArgs [
        headers.arinc653
        headers.arinc653p2
      ]
    }
    do
      cp -- "$ZIP_FILE" target/downloads/"$(stripHash "$ZIP_FILE")"
    done

    # make awk script work
    patchShebangs scripts/*
  '';

  installPhase = ''
    runHook preInstall

    cp --recursive -- target "$out"

    runHook postInstall
  '';

  meta = {
    platforms = lib.platforms.wasi;
  };
}
