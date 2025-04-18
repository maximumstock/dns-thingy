{
  description = "dns-thingy";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

        in
        {
          devShells.default = import ./shell.nix { inherit pkgs; };
          devShells.ci = import ./ci.nix { inherit pkgs; };
          formatter = pkgs.nixpkgs-fmt;
        }

      );
}
