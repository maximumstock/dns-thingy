let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [ (import rustOverlay) ];
  };
  dnspyre = import ./dnspyre.nix {};
in
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    # DNS debugging
    dig
    xxd
    delta

    # Compiler Chain
    mold
    clang
    pkg-config
    (rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" "clippy" ];
    })

    # Dev Tooling
    rust-analyzer
    cargo-edit
    cargo-feature
    cargo-udeps
    cargo-bloat

    # DNS Benchmarking
    dnspyre
    graph-cli
  ];

  RUST_BACKTRACE = 1;
  MOLD_PATH = "${pkgs.mold.out}/bin/mold";
  RUSTFLAGS = "-Clink-arg=-fuse-ld=${MOLD_PATH} -Clinker=clang";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
