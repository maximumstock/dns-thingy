{
  description = "dns-thingy";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
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

        packages = {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "dns-block-tokio"; # should match the binary name in Cargo.toml
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            # Only if you need system libraries, e.g. for openssl-sys:
            # nativeBuildInputs = [ pkgs.pkg-config ];
            # buildInputs = [ pkgs.openssl ];

            meta.mainProgram = "dns-block-tokio";
          };
        };
      }

    );
}
