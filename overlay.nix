final: prev:
let
  inherit (prev) lib;

  # root dir of this flake
  flakeRoot = ./.;

  # all packages from the local tree
  arinc653WasmPkgs = lib.filesystem.packagesFromDirectoryRecursive {
    # a special callPackage variant that contains our flakeRoot
    callPackage = lib.callPackageWith (final // { inherit flakeRoot; });

    # local tree of packages
    directory = ./pkgs;
  };
in

{
  # https://github.com/NixOS/nixpkgs/pull/42637
  requireFile =
    args:
    (prev.requireFile args).overrideAttrs (_: {
      allowSubstitutes = true;
    });

  # custom namespace for packages from the local tree
  inherit arinc653WasmPkgs;

  # TODO remove this package once we are on a nixpkgs version which has https://github.com/NixOS/nixpkgs/pull/429596
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

}
// arinc653WasmPkgs
