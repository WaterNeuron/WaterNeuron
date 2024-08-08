{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        customLibunwind = pkgs.libunwind.overrideAttrs (oldAttrs: {
          outputs = [ "out" "dev" ];
          configureFlags = (oldAttrs.configureFlags or []) ++ [
            "--enable-shared"
            "--enable-static"
          ];
          postInstall = ''
            ${oldAttrs.postInstall or ""}
            mkdir -p $dev/include
            cp -r include/* $dev/include/
          '';
        });
      in
      {
        devShells.default = pkgs.mkShellNoCC {

        packages = with pkgs; [
            gcc
            gnumake
            binutils
            coreutils
            pkg-config
            openssl
            libusb1
            sqlite
            zlib
            llvmPackages_18.libclang
            protobuf
            llvm
            lmdb
            xz
            pkg-config

            shfmt
            customLibunwind

            bazel_7
            bazel-buildtools
          ];

          shellHook = ''
            echo "Custom development environment loaded"
          '';
        };
      });
}
