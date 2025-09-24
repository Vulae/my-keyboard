{
  description = "Nix flake for my-keyboard";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "my-keyboard";
          src = ./.;
          buildInputs = [ ];
          nativeBuildInputs = [ ];
          # cargoHash = pkgs.lib.fakeHash;
          cargoHash = "sha256-ZGHVIP9Eoa5qaP0RTJ+hLNpmL6AGyn0XU56kQT9RjNc=";
          meta = {
            description = "Keyboard lighting effects";
            mainProgram = "my-keyboard";
          };
        };
      }
    );
}
