{
  description = "devenv";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        pathPackages = with pkgs; [
          # bazel
          bazelisk
          bazel-buildtools

          # nix
          nixfmt-rfc-style
        ];

      in
      {
        devShell = pkgs.mkShell {
          shellHook = ''
            alias bb='bazelisk '

            echo "welcome to your nix shell"
          '';
        };
      }
    );
}
