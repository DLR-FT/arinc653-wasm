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

            # universal formatter
            treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
          in
          {
            # packages from `pkgs/`, injected into the `pkgs` via our `overlay.nix`
            packages = pkgs.arinc653WasmPkgs;

            devShells = {
              # devShell for the wasm stuff
              wasm = pkgs.callPackage ./shell.nix { };

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
