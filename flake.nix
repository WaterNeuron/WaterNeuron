{
  inputs = {
    # Track a specific tag on the nixpkgs repo.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

    # The flake format itself is very minimal, so the use of this
    # library is common.
    flake-utils.url = "github:numtide/flake-utils";
  };

  # Here we can define various kinds of "outputs": packages, tests,
  # and so on, but we will only define a development shell.

  outputs = { nixpkgs, flake-utils, ... }:

    # For every platform that Nix supports, we ...
    flake-utils.lib.eachDefaultSystem (system:

      # ... get the package set for this particular platform ...
      let pkgs = import nixpkgs { inherit system; };
      in
      {
        # ... and define a development shell for it ...
        devShells.default =

          # ... with no globally-available CC toolchain ...
          pkgs.mkShellNoCC {
            name = "custom_dev_environment";

            # ... which makes available the following dependencies,
            # all sourced from the `pkgs` package set:
            packages = with pkgs; [
              # Equivalent to build-essential
              gcc
              gnumake
              binutils
              coreutils

              pkg-config
              openssl
              libunwind
              libusb1
              sqlite
              zlib
              llvmPackages_18.libclang
              protobuf
              llvm
              lmdb
              xz
            ];

            shellHook = ''
              echo "Custom development environment loaded"
            '';
          };
      });
}
