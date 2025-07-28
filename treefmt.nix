{ lib, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  settings.global.excludes = [
    "tests/specification/testsuite/*"
  ];
  programs.clang-format.enable = true;
  programs.nixfmt.enable = true;
  programs.prettier.enable = true;
  programs.rustfmt = {
    enable = true;
    edition = (lib.importTOML ./pkgs/c-abi-lens/Cargo.toml).package.edition;
  };
}
