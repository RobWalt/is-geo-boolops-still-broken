{ pkgs, unstable-pkgs, system, fenix, ... }:
pkgs.mkShell rec {
  buildInputs = [

    # rust environment
    (with fenix.packages.${system};  combine [
      stable.cargo
      stable.rustc
      stable.rust-src
      stable.rust-analyzer
      stable.clippy
      complete.rustfmt
      complete.rust-std
      targets.wasm32-unknown-unknown.stable.rust-std
    ])

    unstable-pkgs.wasm-bindgen-cli # building wasm release
    pkgs.cargo-make # task runner
    (pkgs.callPackage ./wasm-server-runner.nix { }) # running wasm locally
    pkgs.butler # upload release to itch.io

    # various dependencies of bevy
    pkgs.protobuf
    pkgs.pkgconfig
    pkgs.alsaLib
    pkgs.udev
    pkgs.openssl
    pkgs.glib

    # For libvulkan.so.1 (RUST_LOG=wgpu_hal=debug).
    pkgs.vulkan-loader

    # 🚧🚧 TODO 🚧🚧 : Make this optional based on something, see rofi-pass-wayland as an example
    # X
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    # Wayland
    # https://github.com/gfx-rs/wgpu/issues/2519
    pkgs.libxkbcommon
    pkgs.wayland
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
