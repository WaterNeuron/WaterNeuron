{
  description = "Reproducible Canisters Environment";

  inputs = {
    nixpkgs.url       = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url   = "github:numtide/flake-utils";
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
          name    = "ic-wasm";
          version = "0.9.1";
          src     = pkgs.fetchurl {
            url    = "https://github.com/dfinity/ic-wasm/releases/download/0.9.1/ic-wasm-x86_64-unknown-linux-gnu.tar.gz";
            sha256 = "sha256-nU2O8sCV82dKaeDH86pj9gOW/01ZnOyZ7P5vnZOPpLE=";
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

        llvmPkgs = pkgs.llvmPackages_18;

        pocketIcVersion = "7.0.0";
        platform        = if pkgs.stdenv.isDarwin then "darwin" else "linux";

        pocket-ic = pkgs.stdenv.mkDerivation {
          pname   = "pocket-ic";
          version = pocketIcVersion;
          src = pkgs.fetchurl {
            url =
              "https://github.com/dfinity/pocketic/releases/download/${pocketIcVersion}/pocket-ic-x86_64-${platform}.gz";
            sha256 = "sha256-CTXubs4xJxmq5Oq93sLfxq801e2930069TvM0bNjYEQ=";
          };
          phases = [ "unpackPhase" "installPhase" ];

          unpackPhase = ''
            gzip -dc $src > pocket-ic
          '';

          installPhase = ''
            mkdir -p $out/bin
            install -m755 pocket-ic $out/bin/
          '';
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            which curl git gcc wabt ic-wasm pkg-config protobuf

            rustToolchain

            libunwind lmdb

            llvmPackages_18.clang
            llvmPkgs.libclang
            stdenv.cc.cc.lib

            pocket-ic
          ];

          TZ = "UTC";

          LIBCLANG_PATH = "${llvmPkgs.libclang.lib}/lib";

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            llvmPkgs.libclang
            pkgs.stdenv.cc.cc.lib
          ];

          LMDB_H_PATH             = "${pkgs.lmdb.dev}/include/lmdb.h";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.lmdb.dev}/include";

          POCKET_IC_BIN = "${pocket-ic}/bin/pocket-ic";
        };
      });
}
