{ pkgs }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    openssl
    pkg-config

    (rust-bin.beta.latest.default.override {
      extensions = [ "rust-src" "clippy" ];
    })
    (import ./dnspyre.nix { inherit pkgs; })
  ];

  RUST_BACKTRACE = 1;
}
