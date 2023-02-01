let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [ (import rustOverlay) ];
  };
  dnspyre = import ./dnspyre.nix { };
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    (rust-bin.stable."1.67.0".default.override {
      extensions = [ "rust-src" "clippy" ];
    })
    dnspyre
  ];

  RUST_BACKTRACE = 1;
}
