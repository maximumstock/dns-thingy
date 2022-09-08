{ pkgs ? import <nixpkgs> {} }:

pkgs.buildGoModule rec {
  pname = "dnspyre-${version}";
  version = "2.3.0";

  src = pkgs.fetchFromGitHub {
    owner = "Tantalor93";
    repo = "dnspyre";
    rev = "v2.3.0";
    sha256 = "sha256-D9S1IOpbWsZLqIe4Br3QTlbAS8wnYyBErODMSzy3DdY=";
  };

  vendorSha256 = "sha256-Fj/OeDTQ8F9CApcHX+6dVyPTEqzlDurgn7k6l4ez2vs=";#lib.fakeSha256;

}
