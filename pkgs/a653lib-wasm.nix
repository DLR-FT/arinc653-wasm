{
  lib,
  pkgs,
  pkgsBuildBuild,
  fetchurl,
  symlinkJoin,
}:

let
  # nixpkgs configured for cross-compilation to wasm32-unknown-wasi
  pkgsWasm = import (pkgs.path) {
    system = "x86_64-linux";
    crossSystem = {
      config = "wasm32-unknown-wasi";
      useLLVM = true;
    };
  };

  # old version of the c-abi-lens required to build the a653 lib
  c-abi-lens-builder =
    { rustPlatform, fetchFromGitHub }:
    rustPlatform.buildRustPackage rec {
      name = "c-abi-lens-old";
      src = fetchFromGitHub {
        owner = "psiegl";
        repo = "arinc653-wasm";
        rev = "287a3419c7adc9c51f0c7cc2863fbfa15e3a9d39"; # branch psiegl-old, 2025-10-23
        hash = "sha256-e6ixhgT5/bHfRoUz9+vrMMLgFPHXjUV/SLGlwWnMYOk=";
      };
      sourceRoot = "${src.name}/pkgs/c-abi-lens";
      cargoHash = "sha256-cLhf7AfPfr2Ud5wvMEyMZIIq4KUzs5yTbRnbkPfu5vc=";
      nativeBuildInputs = [ rustPlatform.bindgenHook ];
      meta.mainProgram = "c-abi-lens";
    };
  c-abi-lens = pkgsBuildBuild.callPackage c-abi-lens-builder { };

  arinc653HeaderZip = fetchurl {
    url = "https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653.h.zip";
    hash = "sha256-4sr+QMkK2tDLFG9O0u9PAKWA7iIL+//K7S3eMOJEtPY=";
  };

  # actual builder, either for the wasm partitions, or the native code
  builder =
    {
      stdenv,
      fetchFromGitHub,
      pkgsBuildBuild,
      c-abi-lens,
      libarchive,
      wasmtime,
      wamr,
      buildWasmStuff ? stdenv.hostPlatform.isWasm || stdenv.hostPlatform.isWasi,
    }:
    stdenv.mkDerivation {
      name = "a653lib-wasm";
      src = fetchFromGitHub {
        owner = "psiegl";
        repo = "a653lib";
        rev = "ed50347737655a90f093931e36ea0cc9bba7c743";
        hash = "sha256-CFL36Bg0Ri4KC1F3wEP52yy2j/KcwwsltPU7dtg3Ujo=";
      };

      # WAMR offers both libiwasm.so and libvmlib.a, the latter offers the static symbols needed for the native code
      postPatch = ''
        substituteInPlace a653_lib_wasm32/Makefile \
          --replace '/usr/lib/libiwasm.a' "-lvmlib"
      ''
      # Makefile hardcodes to clang, but for the nixpkgs clang is wrapped so that the sysroot is discovered
      # Setting the target explicitly is not needed, and the nixpkgs call the target wasm32-unknown-wasi, not wasm32-wasi
      + lib.strings.optionalString buildWasmStuff ''
        substituteInPlace Makefile \
          --replace 'clang' "${stdenv.cc.targetPrefix}cc" \
          --replace '--target=wasm32-wasi' ""
      '';

      nativeBuildInputs = [
        libarchive # use to extract the ARINC 653 header zip file
      ]
      # the wasm stuff still invokes part of the Makefile that requires a gcc...
      ++ lib.lists.optional buildWasmStuff pkgsBuildBuild.stdenv.cc;

      buildInputs = lib.lists.optionals stdenv.hostPlatform.isElf [
        wasmtime
        (wamr.overrideAttrs (old: {
          # the wamr package per default only builds the minimum product, not the lib
          sourceRoot = null;
        }))
      ];

      hardeningDisable = [ "all" ];

      makeFlags = [
        "mk_build_dir" # all other build steps need the temporary build dir
      ]
      ++ lib.lists.optional buildWasmStuff "part_wasm_guest"
      ++ lib.lists.optionals (!buildWasmStuff) [
        "alib"
        "amain_wasm"
        "part_wasmtime"
        "part_wamr"
      ];

      preBuild = ''
        # weirdly the makefile creates a top-level tmp dir in my $HOME dir
        export HOME="$PWD"

        # the `tmp/` prefix is not a typo!
        install -Dm644 ${arinc653HeaderZip} "tmp/download/arinc653.h.zip"
        install -Dm755 ${lib.meta.getExe c-abi-lens} "tmp/arinc653-wasm/pkgs/c-abi-lens/target/debug/c-abi-lens"
      '';

      installPhase = ''
        runHook preInstall

        mkdir -- "$out"
        cp --recursive --no-target-directory -- bin "$out/wasmtime"
        cp --recursive --no-target-directory -- bin "$out/wamr"

        for FILE in "$out/wamr/p_wamr" "$out/wasmtime/p_wasmtime"
        do     
          if [ -f "$FILE" ]
          then
            ln --relative --symbolic -- "$FILE" "''${FILE%/*}/wasm32_rt"
          fi
        done

        runHook postInstall
      '';

      passthru = {
        inherit c-abi-lens;
      };
    };

  a653lib-native = pkgs.callPackage builder { inherit c-abi-lens; };
  a653lib-wasm = pkgsWasm.callPackage builder { inherit c-abi-lens; };
in
symlinkJoin {
  name = "a653lib-wasm";
  paths = [
    a653lib-native
    a653lib-wasm
  ];
  passthru = {
    inherit a653lib-native a653lib-wasm;
  };
}
