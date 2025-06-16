{
  description = "Reproducible Canisters Environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        ic-wasm = pkgs.stdenv.mkDerivation {
          name = "ic-wasm";
          version = "0.9.1";
          src = pkgs.fetchurl {
            url = if pkgs.stdenv.isDarwin then
              "https://github.com/dfinity/ic-wasm/releases/download/0.9.1/ic-wasm-x86_64-apple-darwin.tar.gz"
            else if pkgs.stdenv.isLinux then
              "https://github.com/dfinity/ic-wasm/releases/download/0.9.1/ic-wasm-x86_64-unknown-linux-gnu.tar.gz"
            else
              throw "Unsupported platform";

            sha256 = if pkgs.stdenv.isDarwin then
              "sha256-ixMlyqn8wQcsYRfMaPqnE9MJsOABJhZABJ6BO2YojVM="
            else if pkgs.stdenv.isLinux then
              "sha256-nU2O8sCV82dKaeDH86pj9gOW/01ZnOyZ7P5vnZOPpLE="
            else
              throw "Unsupported platform";
          };
          unpackPhase = ''
            tar xzf $src
          '';
          installPhase = ''
            mkdir -p $out/bin
            cp ic-wasm $out/bin/
            chmod +x $out/bin/ic-wasm
          '';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            which
            curl
            git
            gcc
            wabt
            ic-wasm
            rustToolchain
          ];
          TZ = "UTC";
        };
      });
}
