{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        
        baseToolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.96.0";
          sha256 = "sha256-mvUGEOHYJpn3ikC5hckneuGixaC+yGrkMM/liDIDgoU=";
        };
        
        devRustToolchain = baseToolchain.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer" 
        ];

        packageRustToolchain = baseToolchain.withComponents [
          "cargo"
          "rustc"
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            devRustToolchain
            pkgs.pkg-config
            pkgs.lua5_5
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            pkgs.lua5_5
          ]);

          RUST_SRC_PATH = "${devRustToolchain}/lib/rustlib/src/rust/library";
        };
        packages.default = (pkgs.makeRustPlatform {
          cargo = packageRustToolchain;
          rustc = packageRustToolchain;
        }).buildRustPackage {
          pname = "my-keyboard";
          version = "0.1.0";

          meta = {
            description = "Keyboard lighting effects";
            mainProgram = "my-keyboard";
          };

          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "mlua-0.12.0-rc.2" = "sha256-Bhwnel4CvTmwZWR8S5jvPpR371yEkKMi6fb/qR7syMc=";
            };
          };

          nativeBuildInputs = (with pkgs; [
            pkg-config
            lua5_5
          ]);

          buildInputs = (with pkgs; [
            lua5_5
          ]);

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            pkgs.lua5_5
          ]);

          postFixup = ''
            patchelf --add-rpath "${pkgs.lib.makeLibraryPath (with pkgs; [
              lua5_5
            ])}" $out/bin/my-keyboard
          '';
        };
      }
    );
}
