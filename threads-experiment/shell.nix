{
  pkgs ? import <nixpkgs> {
    system = "x86_64-linux";
    crossSystem = {
      config = "wasm32-unknown-wasi";
      useLLVM = true;
    };
  },
}:

pkgs.callPackage (
  {
    mkShell,
    wabt,
    wamr,
    wasmtime,
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
    ];
  }
) { }
