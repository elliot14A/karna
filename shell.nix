{ pkgs ? import <nixpkgs> { } }:

let
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rust-analyzer
    rustfmt
    clippy
    pkg-config
    openssl
    openssl.dev
    bun
    trunk
    wasm-pack
    cargo-watch
    cargo-generate
  ];

  shellHook = ''
    export RUST_BACKTRACE=1
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.openssl
    ]}"
    
    echo "Karna development environment loaded"
  '';

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
