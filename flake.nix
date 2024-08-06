{
  description = "WaterNeuron development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
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

        commonPackages = with pkgs; [
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
          shfmt
          bazel_7
          bazel-buildtools
        ];

      in {
        packages = {
          libunwind = customLibunwind;
          default = customLibunwind;
        };

        devShells.default = pkgs.mkShellNoCC {
          packages = commonPackages;

          shellHook = ''
            echo "Welcome to WaterNeuron development environment!"
          '';
        };
      }
    );
}
