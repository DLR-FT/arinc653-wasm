{  ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  settings.global.excludes = [
    "tests/specification/testsuite/*"
  ];
  programs.clang-format.enable = true;
  programs.nixfmt.enable = true;
  programs.prettier.enable = true;
}
