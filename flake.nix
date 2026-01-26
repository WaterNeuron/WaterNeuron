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
        ic-wasm = pkgs.stdenv.mkDerivation rec {
          name = "ic-wasm";
          version = "0.9.3";
          src = pkgs.fetchurl {
            url = "https://github.com/dfinity/ic-wasm/releases/download/${version}/ic-wasm-x86_64-${if pkgs.stdenv.isDarwin then "apple-darwin" else "unknown-linux-gnu"}.tar.gz";
            sha256 = if pkgs.stdenv.isDarwin
              then "sha256-WmHu3peNyJMcbdPAVbwic5+42K3eHyFv49y/QCdPe/M="
              else "sha256-WSj7x+uKUP6Bcoyk2HPLCr3MJRHW7DRPy2V++r8HWX0=";
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
        pocket-ic = pkgs.stdenv.mkDerivation rec {
          name = "pocket-ic";
          version = "12.0.0";
          src = pkgs.fetchurl {
            url = "https://github.com/dfinity/pocketic/releases/download/${version}/pocket-ic-x86_64-${if pkgs.stdenv.isDarwin then "darwin" else "linux"}.gz";
            sha256 = if pkgs.stdenv.isDarwin 
              then "sha256-Z7qlb8SvuqiTXhTviNnWD3fQEZR/BUitxNQkw8iBnjU="
              else "sha256-kUBboS/oqEAs1ZA/tD9ZJ3caVJOpcM/POuDfvYvbRaw=";
          };
          nativeBuildInputs = [ pkgs.gzip ];
          unpackPhase = ''
            gunzip -c $src > pocket-ic
          '';
          installPhase = ''
            mkdir -p $out/bin
            cp pocket-ic $out/bin/
            chmod +x $out/bin/pocket-ic
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
            pocket-ic
            rustToolchain
          ];
          TZ = "UTC";
          POCKET_IC_BIN = "${pocket-ic}/bin/pocket-ic";
          
          shellHook = ''
            echo "Entering nix-shell"
            export PS1='\[\033[1;32m\][nix]\[\033[0m\] \w\$ '
          '';
        };
      });
}
