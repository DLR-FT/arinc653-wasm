{
  description = "An ARINC 653 WebAssembly SDK";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      treefmt-nix,
      ...
    }:
    {
      overlays.default = import ./overlay.nix;
    }
    //
      flake-utils.lib.eachSystem
        [
          "x86_64-linux"
          "i686-linux"
          "aarch64-linux"
        ]
        (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              # import our overlay for the package in pkgs/
              overlays = [ self.overlays.default ];
            };

            pkgsWasm = import nixpkgs {
              system = "x86_64-linux";
              crossSystem = {
                config = "wasm32-unknown-wasi";
                useLLVM = true;
              };
              # import our overlay for the package in pkgs/
              overlays = [ self.overlays.default ];
            };

            # universal formatter
            treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
          in
          {
            # packages from `pkgs/`, injected into the `pkgs` via our `overlay.nix`
            packages = pkgs.arinc653WasmPkgs // {
              wasm-partitions = pkgsWasm.callPackage pkgs/wasm-partitions.nix { };
            };

            devShells = {
              # devShell for the wasm stuff
              wasm = pkgsWasm.mkShell {
                # devtools that don't need to know about the target arch
                #
                # Note: Here we are intentionally opting out of Nix' cross-compilation splicing machinery
                depsBuildBuild = with pkgsWasm.pkgsBuildBuild; [
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
              };

              # devShell for the rust code
              rust = pkgs.callPackage (
                {
                  mkShell,
                  cargo-llvm-cov,
                  c-abi-lens,
                  clippy,
                  rustfmt,
                }:
                mkShell {
                  inputsFrom = [ c-abi-lens ];
                  env = { inherit (cargo-llvm-cov) LLVM_COV LLVM_PROFDATA; };
                  nativeBuildInputs = [
                    cargo-llvm-cov
                    clippy
                    rustfmt
                  ];
                }
              ) { };
              default = self.devShells.${system}.wasm;
            };

            # for `nix fmt`
            formatter = treefmtEval.config.build.wrapper;

            # for `nix flake check`
            checks = {
              # check that all files are properly formatted
              formatting = treefmtEval.config.build.check self;
            };
          }
        );
}
