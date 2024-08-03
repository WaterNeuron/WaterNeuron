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
          name = "custom_dev_environment";

          packages = with pkgs; [
            gcc
            gnumake
            binutils
            coreutils
            pkg-config
            openssl
            customLibunwind
            libusb1
            sqlite
            zlib
            llvmPackages_18.libclang
            protobuf
            llvm
            lmdb
            xz
            pkg-config
          ];

          shellHook = ''
            echo "Custom development environment loaded"
            echo "Checking libunwind installation:"
            echo "libunwind.out: ${customLibunwind.out}"
            echo "libunwind.dev: ${customLibunwind.dev}"
            echo "Searching for libunwind.h:"
            find ${customLibunwind.dev} -name libunwind.h || echo "libunwind.h not found"

            export C_INCLUDE_PATH=${customLibunwind.dev}/include:$C_INCLUDE_PATH
            export CFLAGS="-I${customLibunwind.dev}/include $CFLAGS"
            export CPPFLAGS="-I${customLibunwind.dev}/include $CPPFLAGS"
            export LIBRARY_PATH=${customLibunwind.out}/lib:$LIBRARY_PATH
            export LD_LIBRARY_PATH=${customLibunwind.out}/lib:$LD_LIBRARY_PATH
            export PKG_CONFIG_PATH=${customLibunwind.dev}/lib/pkgconfig:$PKG_CONFIG_PATH


            echo "Updated C_INCLUDE_PATH: $C_INCLUDE_PATH"
            echo "Updated LIBRARY_PATH: $LIBRARY_PATH"
            echo "Updated LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
            echo "Updated PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
          '';
        };
      });
}
