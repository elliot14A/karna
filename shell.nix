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
    cmake
    gcc
    libcxx
    duckdb
    sqlx-cli
    litecli
    tokei
  ];


  shellHook = ''
    export RUST_BACKTRACE=1
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
    export DATABASE_URL="sqlite:./karna/sqlite/db.sqlite"
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.openssl
      pkgs.duckdb
    ]}"
    
    echo "Karna development environment loaded"
  '';

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
