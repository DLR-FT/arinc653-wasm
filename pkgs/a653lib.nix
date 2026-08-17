{
  lib,
  stdenv,
  fetchFromGitHub,
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
}:

stdenv.mkDerivation {
  name = "a653lib";
  src = fetchFromGitHub {
    owner = "airbus";
    repo = "a653lib";
    rev = "51e54d734998bbb4e63ffb0743e5c9f3f7fda176"; # branch main, 2026-06-10
    hash = "sha256-IZPdTKJEfo5HghwshAPv47GgtlqFG/An93N2WFbxOYE=";
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
    # Avoid the following error upon linking a653lib
    #
    # undefined references to `__aarch64_ldadd4_acq'
    (wamr.overrideAttrs (_: {
      env = lib.attrsets.optionalAttrs (stdenv.hostPlatform.isAarch) {
        NIX_CFLAGS_COMPILE = "-mno-outline-atomics";
      };
    }))
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

  postInstall = ''
    for engine in wamr wasmtime
    do
      target_dir="$out/bin/with-$engine"
      mkdir --parent -- "$target_dir"
      ln --relative --symbolic --force -- $out/bin/*wasm "$target_dir"
      ln --relative --symbolic --force -- "$out/bin/p_$engine" "$target_dir/wasm32_rt"
    done
  '';

  meta = {
    description = "AIRBUS' Linux based ARINC 653 playground, now with Wasm";
    license = lib.licenses.lgpl21Plus;
    maintainers = with lib.maintainers; [ wucke13 ];
    mainProgram = "a653lib_main";
    platforms = lib.platforms.linux;
  };
}
