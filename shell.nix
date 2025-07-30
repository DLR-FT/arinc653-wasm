{
  path ? <nixpkgs>,
}:

let
  pkgs = import path {
    system = "x86_64-linux";
    crossSystem = {
      config = "wasm32-unknown-wasi";
      useLLVM = true;
    };
    # TODO remove this package once we are on a nixpkgs version which has https://github.com/NixOS/nixpkgs/pull/429596
    overlays = [
      (final: prev: {
        wasilibc = prev.wasilibc.overrideAttrs (old: {

          # override wasilibc version to newest release
          pname = "wasilibc";
          version = "27-unstable-2025-07-26";

          src = final.buildPackages.fetchFromGitHub {
            inherit (old.src) owner repo;
            rev = "3f7eb4c7d6ede4dde3c4bffa6ed14e8d656fe93f";
            hash = "sha256-RIjph1XdYc1aGywKks5JApcLajbNFEuWm+Wy/GMHddg=";
            fetchSubmodules = true;
          };

          prePatch = "patchShebangs scripts/";

          preBuild = ''
            export SYSROOT_LIB=${builtins.placeholder "out"}/lib
            export SYSROOT_INC=${builtins.placeholder "dev"}/include
            export SYSROOT_SHARE=${builtins.placeholder "share"}/share
            mkdir -p "$SYSROOT_LIB" "$SYSROOT_INC" "$SYSROOT_SHARE"
            makeFlagsArray+=(
              "SYSROOT_LIB:=$SYSROOT_LIB"
              "SYSROOT_INC:=$SYSROOT_INC"
              "SYSROOT_SHARE:=$SYSROOT_SHARE"
              "THREAD_MODEL:=posix"
            )
          '';
        });
      })
    ];
  };
in
pkgs.callPackage (
  {
    mkShell,
    wabt,
    wamr,
    wasmtime,

    curl,
    findutils,
    gawk,
    libarchive,
  }:
  mkShell {
    # devtools that don't need to know about the target arch
    #
    # Note: Here we are intentionally opting out of Nix' cross-compilation splicing machinery
    depsBuildBuild = [
      # wasm tools
      wabt # wasm binary tools, to show Wasm Text (Wat) of a Wasm binary
      wamr # bytecode-alliance's micro runtime, an almost reference implementation of an interpreter
      wasmtime # bytecode-alliance's Wasm interpreter with advanced AOT compilation

      # generic cli tools
      curl # to download stuff
      findutils # for xargs
      gawk # for awk to preprocess header files
      libarchive # bsdtar, to unpack zip files
    ];
  }
) { }
