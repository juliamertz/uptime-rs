{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, ... }:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import inputs.rust-overlay) ];
        pkgs = import (inputs.nixpkgs) { inherit system overlays; };
        inherit (pkgs) lib;
        inherit (lib) getExe;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.minimal;
          rustc = pkgs.rust-bin.stable.latest.minimal;
        };
        nativeBuildInputs = with pkgs; [ pkg-config rustPlatform.bindgenHook ];
        tools = pkgs.callPackage ./tools.nix { };
        buildInputs = with pkgs; [ openssl tailwindcss ];

        # for file in schemas/*.sql; do
        #   echo "Running $file"
        #   # cat $file
        #   sqlite3 $DB_PATH < $file
        # done

      in {
        packages.default = rustPlatform.buildRustPackage {
          inherit buildInputs nativeBuildInputs;

          name = "uptime-rs";
          src = ./.;
          version = self.shortRev or "dev";

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
        };

        devShell = pkgs.mkShell {
          name = "uptime-rs-shell";
          inherit nativeBuildInputs;

          buildInputs = buildInputs ++ (with pkgs.rust-bin; [
            (stable.latest.minimal.override {
              extensions = [ "clippy" "rust-src" ];
            })
            nightly.latest.clippy
            nightly.latest.rustfmt
            nightly.latest.rust-analyzer

            pkgs.nodejs

            tools.formatter
            tools.buildTailwind
            (pkgs.writeShellScriptBin "dev" # sh
              "${getExe pkgs.cargo-watch} watch --exec run ")
          ]);
        };
      });
}
