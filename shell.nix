{
  pkgs ? import <nixpkgs> {
    crossSystem = {
      config = "wasm32-unknown-wasi";
      useLLVM = true;
    };
  },
}:

pkgs.callPackage (
  {
    mkShell,
    bear,
    llvmPackages,
    wabt,
    wamr,
    wasmtime,

    gawk,
    curl,
    findutils,
    gnused,
    libarchive,
  }:
  mkShell {
    nativeBuildInputs = [ ];

    # devtools that don't need to know about the target arch
    #
    # Note: Here we are intentionally opting out of Nix' cross-compilation splicing machinery
    depsBuildBuild = [
      bear # to generate compile_commands.json for clangd
      llvmPackages.clang-tools # for clang-format and clangd

      # wasm tools
      wabt # wasm binary tools, to show Wasm Text (Wat) of a Wasm binary
      wamr # bytecode-alliance's micro runtime, an almost reference implementation of an interpreter
      wasmtime # bytecode-alliance's Wasm interpreter with advanced AOT compilation

      # generic cli tools
      gawk
      curl # to download stuff
      findutils # for xargs
      gnused # for sed
      libarchive # bsdtar, to unpack zip files
    ];

    env = {
      # SYSTEM_INCLUDE_DIR = pkgs.stdenv.cc.libc.dev + "/include";
      LIBRARY_PATH = pkgs.stdenv.cc.libc + "/lib";
    };
  }
) { }
