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
  };
in
pkgs.callPackage (
  {
    lib,
    mkShellNoCC,
    llvmPackages,
    wabt,
    wamr,
    wasmtime,

    curl,
    findutils,
    gawk,
    libarchive,
  }:
  mkShellNoCC {
    nativeBuildInputs = [
      llvmPackages.bintools-unwrapped # for wasm-ld
      llvmPackages.clang-unwrapped # for clang, clang-format and clangd
    ];

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

    env = {
      CCC_OVERRIDE_OPTIONS = lib.strings.concatStringsSep " " [
        "#"
        "^-I${pkgs.stdenv.cc.libc.dev}/include"
        "^-nostdlibinc"
        "^-resource-dir=${pkgs.stdenv.cc}/resource-root"
        "^-frandom-seed=5z87fdpjmk"
        "^-Wno-unused-command-line-argument"
        "^-Wl,-L${pkgs.stdenv.cc.libc}/lib"
      ];
    };
  }
) { }
