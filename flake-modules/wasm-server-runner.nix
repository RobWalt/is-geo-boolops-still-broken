{ pkgs, lib, ... }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "wasm-server-runner";
  version = "0.4.0";

  src = pkgs.fetchFromGitHub {
    owner = "jakobhellermann";
    repo = "wasm-server-runner";
    rev = "v${version}";
    hash = "sha256-u/HY7DH/7naiTT35Iucf5u7mFNd5h1nF55Ae6Mr/UJs=";
  };

  cargoPatches = [ ./cargo-lock.patch ];

  cargoHash = "sha256-vB62IkLic4AbyQWyA6wP5yZ25ry6qjbJb6lwR/bnc+U=";
  nativeBuildInputs = [ pkgs.pkg-config ];

  buildInputs = [ pkgs.openssl ]
    ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.CoreServices pkgs.Security ];

  meta = with lib; {
    description = "cargo run for the browser ";
    homepage = "https://github.com/jakobhellermann/wasm-server-runner";
    license = with licenses; [ mit ];
    maintainers = with maintainers; [ robwalt ];
  };
}
