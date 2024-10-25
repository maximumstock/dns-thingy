let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [ (import rustOverlay) ];
  };
  dnspyre = import ./dnspyre.nix { };
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    (rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" "clippy" ];
    })
    dnspyre
    perf-tools # to create profiles and flamegraph diffs
    cargo-flamegraph # to generate flamegraphs from our Rust binaries
  ];

  RUST_BACKTRACE = 1;
}
