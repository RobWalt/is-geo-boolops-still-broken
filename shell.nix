# shell.nix

{ pkgs ? import <nixpkgs> { } }:
with pkgs; mkShell rec {
  nativeBuildInputs = [
    pkgconfig
  ];
  buildInputs = [
    udev
    alsaLib
    vulkan-loader
    wayland
    libxkbcommon
    openssl
  ];
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  BROWSER = "${pkgs.librewolf}/bin/librewolf";
}
