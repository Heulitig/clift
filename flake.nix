{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    # TODO: use nixpkgs/unstable when this is merged:
    # https://github.com/NixOS/nixpkgs/pull/282798
    nixpkgs.url = "github:junjihashimoto/nixpkgs/feature/rust-dup";
  };

  outputs = { self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ ];
        };


        clift = pkgs.pkgsStatic.callPackage ./clift.nix { };
        clift-win = pkgs.pkgsStatic.pkgsCross.mingwW64.callPackage ./clift.nix { };
      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = clift;

        packages = {
          inherit clift;
          inherit clift-win;
        };

        # nix develop
        devShell = pkgs.mkShell {
          name = "clift-shell";
          nativeBuildInputs = with pkgs; [
            rustc
            rustfmt
            clippy
            cargo
            pkg-config
            openssl.dev
            postgresql_14
            diesel-cli
            rust-analyzer
          ];

          shellHook = ''
            export PATH="$PATH:$HOME/.cargo/bin"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
