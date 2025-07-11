{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    bear # to generate compile_commands.json for clangd
    llvmPackages.clang-tools # for clang-format and clangd
    llvmPackages.clang-unwrapped # a C compiler that can generate Wasm files
    wabt # wasm binary tools, to show Wasm Text (Wat) of a Wasm binary
    wamr # bytecode-alliance's micro runtime, an almost reference implementation of an interpreter
    wasmtime # bytecode-alliance's Wasm interpreter with advanced AOT compilation

    # generic cli tools
    curl # to download stuff
    findutils # for xargs
    gnused # for sed
    libarchive # bsdtar, to unpack zip files
  ];
}
