{
  description = ''
    A collection of metabuild company flakes.

    Take a look at the `flake.nix` file for all available options.
  '';

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/23.05";
    unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, unstable, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        unstable-pkgs = unstable.legacyPackages.${system};
        import-base = file: import file { inherit pkgs unstable-pkgs system fenix; };

        rust-base = import-base ./flake-modules/rust.nix;
      in
      {
        devShells =
          # base shell
          {
            inherit rust-base;
          }
          # customized shells
          //
          {
            rob = pkgs.mkShell {
              shellHook = ''
                nix develop .#rust-base --command zsh
                exit
              '';
            };
          };
      }
    );
}
