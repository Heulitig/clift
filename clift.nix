{ rustPlatform, stdenv, pkg-config, lib, windows, openssl }:
let
  cliftCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  version = cliftCargo.package.version;
in
rustPlatform.buildRustPackage {
  name = "clift";
  inherit version;
  src = lib.cleanSource ./.;

  doCheck = false; # set this to true to run cargo test

  nativeBuildInputs = [ pkg-config ];

  buildInputs = lib.optional stdenv.targetPlatform.isWindows [
    windows.mingw_w64_pthreads
    windows.pthreads
  ];

  # https://docs.rs/pkg-config/latest/pkg_config/
  PKG_CONFIG_ALL_STATIC = "1";

  PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";

  RUSTFLAGS = "-C target-feature=+crt-static";

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };
}
