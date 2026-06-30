{
  lib,
  stdenv,
  fetchFromGitHub,
  fetchpatch,
  fetchurl,

  c-abi-lens,
  cmake,
  pkg-config,
  pkgsCross,

  wamr,
  wasmtime,

  arinc653HeaderZip ? fetchurl {
    url = "https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653.h.zip";
    hash = "sha256-4sr+QMkK2tDLFG9O0u9PAKWA7iIL+//K7S3eMOJEtPY=";
  },
}@args:

let
  c-abi-lens = args.c-abi-lens.overrideAttrs (_: {
    patches = [
      (fetchpatch {
        url = "https://github.com/DLR-FT/arinc653-wasm/pull/37.patch";
        hash = "sha256-O0fCAP6ksQlN40cmpASzi+KTEez5a0ToxuK3KL+jvn0=";
      })
    ];
    patchFlags = [ "-p3" ];
  });
in

stdenv.mkDerivation {
  name = "a653lib-wasm";
  src = fetchFromGitHub {
    owner = "psiegl";
    repo = "a653lib";
    rev = "5256ae9cc2dc51e391bbea4dfaeec4b63897ff59"; # branch main, 2026-06-08
    hash = "sha256-8zYyGwznNStJyhZRU6j1Ij4Ku2E5ZZ17ix63U7zy5is=";
  };

  nativeBuildInputs = [
    cmake
    pkg-config
    pkgsCross.wasi32.stdenv.cc
  ];

  postPatch = ''
    substituteInPlace  CMakeLists.txt \
      --replace-fail 'wasm32-wasip1' 'wasm32-unknown-wasi'
  '';

  buildInputs = [
    wamr
    wasmtime
  ];

  cmakeFlags = [
    "-DA653LIB_BUILD_WASM=on"
    "-DARINC653_ZIP=${arinc653HeaderZip}"

    "-DA653LIB_FETCH_C_ABI_LENS=off"
    "-DC_ABI_LENS_EXECUTABLE=${lib.meta.getExe c-abi-lens}"

    "-DWASM_CLANG=${lib.meta.getExe' pkgsCross.wasi32.stdenv.cc "${pkgsCross.wasi32.stdenv.cc.targetPrefix}cc"}"
  ];

  makeFlags = [
    "partition_a_wasm"
    "partition_b_wasm"
  ];
}
