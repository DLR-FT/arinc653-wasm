{
  lib,
  fetchFromGitHub,
  rustPlatform,
}:

let
  cargoToml = lib.trivial.importTOML ./Cargo.toml;

  # filter to get only rust files as source
  src =
    let
      # original source to read from
      src = ./.;

      # File suffices to include
      extensions = [
        "lock"
        "rs"
        "toml"
      ];
      # Files to explicitly include
      include = [ ];
      # Files to explicitly exclude
      exclude = [ ];

      filter = (
        path: type:
        let
          inherit (builtins) baseNameOf toString;
          inherit (lib.lists) any;
          inherit (lib.strings) hasSuffix removePrefix;
          inherit (lib.trivial) id;

          # consumes a list of bools, returns true if any of them is true
          anyof = any id;

          basename = baseNameOf (toString path);
          relative = removePrefix (toString src + "/") (toString path);
        in
        (anyof [
          (type == "directory")
          (any (ext: hasSuffix ".${ext}" basename) extensions)
          (any (file: file == relative) include)
        ])
        && !(anyof [ (any (file: file == relative) exclude) ])
      );
    in
    lib.sources.cleanSourceWith { inherit src filter; };
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  inherit src;

  cargoLock.lockFile = src + "/Cargo.lock";

  nativeBuildInputs = [ rustPlatform.bindgenHook ];

  meta = {
    inherit (cargoToml.package) description homepage;
    license = [
      lib.licenses.asl20
      # OR
      lib.licenses.mit
    ];
    maintainers = [ lib.maintainers.wucke13 ];
  };
})
